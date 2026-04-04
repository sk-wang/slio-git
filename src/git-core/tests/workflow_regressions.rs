mod test_helpers;

use git_core::{
    abort_in_progress_commit_action, cherry_pick_commit, continue_in_progress_commit_action,
    create_commit, diff_file_to_index, diff_index_to_head, export_commit_patch, get_conflict_diff,
    get_history_for_ref, get_in_progress_commit_action, list_branch_scoped_remotes,
    push_current_branch_to_commit, rebase_start, reset_current_branch_to_commit, resolve_conflict,
    resolve_push_current_branch_target, revert_commit, ConflictResolution,
    InProgressCommitActionKind, Repository, SyncStatus,
};
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::tempdir;
use test_helpers::TestRepo;

fn git(repo_path: &Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(repo_path)
        .output()
        .map_err(|error| error.to_string())?;

    if !output.status.success() {
        return Err(format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn create_conflicted_repository() -> (TestRepo, String) {
    let repo = TestRepo::new().expect("failed to create test repository");
    repo.add_and_commit("shared.txt", "base\n", "base commit")
        .expect("failed to create base commit");

    let repository = Repository::open(repo.path()).expect("failed to reopen test repository");
    let main_branch = repository
        .current_branch()
        .expect("failed to read current branch")
        .expect("expected a branch");

    git(repo.path(), &["checkout", "-b", "feature"]).expect("failed to create feature branch");
    repo.write_file("shared.txt", "theirs\n")
        .expect("failed to write feature version");
    git(repo.path(), &["add", "shared.txt"]).expect("failed to stage feature version");
    git(repo.path(), &["commit", "-m", "feature change"]).expect("failed to commit feature");

    git(repo.path(), &["checkout", &main_branch]).expect("failed to switch back to main branch");
    repo.write_file("shared.txt", "ours\n")
        .expect("failed to write main version");
    git(repo.path(), &["add", "shared.txt"]).expect("failed to stage main version");
    git(repo.path(), &["commit", "-m", "main change"]).expect("failed to commit main");

    let output = Command::new("git")
        .args(["merge", "feature"])
        .current_dir(repo.path())
        .output()
        .expect("failed to merge feature branch");
    assert!(
        !output.status.success(),
        "expected merge to create a conflict"
    );

    (repo, main_branch)
}

fn create_cherry_pick_conflict_repository() -> (TestRepo, String) {
    let repo = TestRepo::new().expect("failed to create test repository");
    repo.add_and_commit("shared.txt", "base\n", "base commit")
        .expect("failed to create base commit");

    let repository = Repository::open(repo.path()).expect("failed to reopen test repository");
    let main_branch = repository
        .current_branch()
        .expect("failed to read current branch")
        .expect("expected a branch");

    git(repo.path(), &["checkout", "-b", "feature/cherry-conflict"])
        .expect("failed to create feature branch");
    repo.write_file("shared.txt", "feature\n")
        .expect("failed to write feature version");
    git(repo.path(), &["add", "shared.txt"]).expect("failed to stage feature version");
    git(repo.path(), &["commit", "-m", "feature conflict commit"])
        .expect("failed to commit feature version");

    let feature_commit =
        git(repo.path(), &["rev-parse", "HEAD"]).expect("failed to read feature commit");

    git(repo.path(), &["checkout", &main_branch]).expect("failed to switch back to main");
    repo.write_file("shared.txt", "main\n")
        .expect("failed to write main version");
    git(repo.path(), &["add", "shared.txt"]).expect("failed to stage main version");
    git(repo.path(), &["commit", "-m", "main conflict commit"])
        .expect("failed to commit main version");

    (repo, feature_commit)
}

#[test]
fn create_commit_allows_first_commit_on_unborn_head() {
    let repo = TestRepo::new().expect("failed to create test repository");
    repo.write_file("hello.txt", "hello world\n")
        .expect("failed to write hello file");
    git(repo.path(), &["add", "hello.txt"]).expect("failed to stage hello file");

    let repository = Repository::open(repo.path()).expect("failed to reopen test repository");
    let commit_id =
        create_commit(&repository, "initial commit", "", "").expect("first commit should work");

    assert!(!commit_id.is_empty());
    let message = git(repo.path(), &["log", "--format=%s", "-1", "HEAD"])
        .expect("failed to read latest commit message");
    assert_eq!(message, "initial commit");
}

#[test]
fn repository_display_metadata_prefers_worktree_path_after_open_and_refresh() {
    let repo = TestRepo::new().expect("failed to create test repository");
    let mut repository = Repository::open(repo.path()).expect("failed to open test repository");
    let expected_path = std::fs::canonicalize(repo.path()).expect("failed to canonicalize path");

    assert_eq!(
        std::fs::canonicalize(repository.path()).expect("failed to canonicalize repo path"),
        expected_path
    );
    assert_eq!(
        repository.name(),
        repo.path()
            .file_name()
            .and_then(|name| name.to_str())
            .expect("temp dir should have a name")
    );

    repository.refresh().expect("failed to refresh repository");
    assert_eq!(
        std::fs::canonicalize(repository.path()).expect("failed to canonicalize repo path"),
        expected_path
    );
}

#[test]
fn repository_init_keeps_worktree_path_for_new_repo_display() {
    let repo = TestRepo::empty().expect("failed to create empty temp dir");
    let repository = Repository::init(repo.path()).expect("failed to init test repository");
    let expected_path = std::fs::canonicalize(repo.path()).expect("failed to canonicalize path");

    assert_eq!(
        std::fs::canonicalize(repository.path()).expect("failed to canonicalize repo path"),
        expected_path
    );
    assert_eq!(
        repository.name(),
        repo.path()
            .file_name()
            .and_then(|name| name.to_str())
            .expect("temp dir should have a name")
    );
}

#[test]
fn list_branches_includes_upstream_tracking_and_recent_order() {
    let repo = TestRepo::new().expect("failed to create test repository");
    repo.add_and_commit("tracked.txt", "base\n", "base commit")
        .expect("failed to create base commit");

    let remote = tempdir().expect("failed to create remote tempdir");
    git(remote.path(), &["init", "--bare"]).expect("failed to init bare remote");

    let repository = Repository::open(repo.path()).expect("failed to open local repository");
    let main_branch = repository
        .current_branch()
        .expect("failed to read current branch")
        .expect("expected branch name");

    git(
        repo.path(),
        &[
            "remote",
            "add",
            "origin",
            &remote.path().display().to_string(),
        ],
    )
    .expect("failed to add origin");
    git(repo.path(), &["push", "-u", "origin", &main_branch]).expect("failed to push main");

    git(repo.path(), &["checkout", "-b", "feature/minimal-shell"])
        .expect("failed to create feature branch");
    repo.write_file("tracked.txt", "feature\n")
        .expect("failed to update tracked file");
    git(repo.path(), &["add", "tracked.txt"]).expect("failed to stage feature file");
    git(repo.path(), &["commit", "-m", "feature commit"]).expect("failed to commit feature");
    git(
        repo.path(),
        &["push", "-u", "origin", "feature/minimal-shell"],
    )
    .expect("failed to push feature branch");

    let repository = Repository::open(repo.path()).expect("failed to reopen repository");
    assert_eq!(
        repository.current_upstream_ref().as_deref(),
        Some("origin/feature/minimal-shell")
    );

    let mirror = tempdir().expect("failed to create mirror tempdir");
    git(mirror.path(), &["init", "--bare"]).expect("failed to init mirror remote");
    git(
        repo.path(),
        &[
            "remote",
            "add",
            "mirror",
            &mirror.path().display().to_string(),
        ],
    )
    .expect("failed to add mirror remote");

    let branch_scoped_remotes =
        list_branch_scoped_remotes(&repository).expect("failed to resolve branch scoped remotes");
    assert_eq!(branch_scoped_remotes.len(), 1);
    assert_eq!(branch_scoped_remotes[0].name, "origin");

    let branches = repository
        .list_branches()
        .expect("failed to list repository branches");

    let feature = branches
        .iter()
        .find(|branch| branch.name == "feature/minimal-shell" && !branch.is_remote)
        .expect("missing local feature branch");
    assert!(feature.is_head, "feature branch should be current HEAD");
    assert_eq!(
        feature.upstream.as_deref(),
        Some("origin/feature/minimal-shell")
    );
    assert!(
        feature.tracking_status.is_some(),
        "feature branch should expose tracking status"
    );
    assert_eq!(
        feature.tracking_status.as_deref(),
        Some("✓"),
        "freshly pushed current branch should be in sync with upstream"
    );
    assert!(
        feature.last_commit_timestamp.is_some(),
        "feature branch should expose commit timestamp metadata"
    );

    let local_branch_names: Vec<&str> = branches
        .iter()
        .filter(|branch| !branch.is_remote)
        .map(|branch| branch.name.as_str())
        .collect();
    assert_eq!(
        local_branch_names.first().copied(),
        Some("feature/minimal-shell"),
        "most recent local branch should appear first"
    );
    assert!(
        branches
            .iter()
            .any(|branch| branch.is_remote && branch.name == "origin/feature/minimal-shell"),
        "remote tracking branch should be listed"
    );
}

#[test]
fn list_branches_hides_remote_head_symbolic_ref() {
    let repo = TestRepo::new().expect("failed to create test repository");
    repo.add_and_commit("tracked.txt", "base\n", "base commit")
        .expect("failed to create base commit");

    let remote = tempdir().expect("failed to create remote tempdir");
    git(remote.path(), &["init", "--bare"]).expect("failed to init bare remote");

    let repository = Repository::open(repo.path()).expect("failed to open local repository");
    let main_branch = repository
        .current_branch()
        .expect("failed to read current branch")
        .expect("expected branch name");

    git(
        repo.path(),
        &[
            "remote",
            "add",
            "origin",
            &remote.path().display().to_string(),
        ],
    )
    .expect("failed to add origin");
    git(repo.path(), &["push", "-u", "origin", &main_branch]).expect("failed to push main");
    git(repo.path(), &["remote", "set-head", "origin", &main_branch])
        .expect("failed to set remote HEAD");

    let repository = Repository::open(repo.path()).expect("failed to reopen repository");
    let branches = repository
        .list_branches()
        .expect("failed to list repository branches");

    assert!(
        branches
            .iter()
            .all(|branch| !(branch.is_remote && branch.name == "origin/HEAD")),
        "remote HEAD symbolic ref should be hidden from branch list"
    );
    assert!(
        branches
            .iter()
            .any(|branch| branch.is_remote && branch.name == format!("origin/{main_branch}")),
        "actual remote branch should still be listed"
    );
}

#[test]
fn get_history_for_ref_returns_selected_branch_head_first() {
    let repo = TestRepo::new().expect("failed to create test repository");
    repo.add_and_commit("tracked.txt", "base\n", "base commit")
        .expect("failed to create base commit");

    let repository = Repository::open(repo.path()).expect("failed to reopen repository");
    let main_branch = repository
        .current_branch()
        .expect("failed to read current branch")
        .expect("expected branch name");

    git(repo.path(), &["checkout", "-b", "feature/history-view"])
        .expect("failed to create feature branch");
    repo.write_file("tracked.txt", "base\nfeature\n")
        .expect("failed to update tracked file");
    git(repo.path(), &["add", "tracked.txt"]).expect("failed to stage feature file");
    git(repo.path(), &["commit", "-m", "feature head"]).expect("failed to commit feature head");

    let feature_head =
        git(repo.path(), &["rev-parse", "HEAD"]).expect("failed to read feature HEAD");

    git(repo.path(), &["checkout", &main_branch]).expect("failed to switch back to main branch");
    repo.write_file("main.txt", "main\n")
        .expect("failed to write main-only file");
    git(repo.path(), &["add", "main.txt"]).expect("failed to stage main-only file");
    git(repo.path(), &["commit", "-m", "main only"]).expect("failed to commit main-only change");

    let repository = Repository::open(repo.path()).expect("failed to reopen repository");
    let history = get_history_for_ref(&repository, "feature/history-view", Some(10))
        .expect("failed to load feature branch history");

    assert!(
        history.len() >= 2,
        "feature branch history should include its own head and base commit"
    );
    assert_eq!(
        history[0].id, feature_head,
        "selected branch history should start from the selected branch head"
    );
    assert_eq!(history[0].message.trim(), "feature head");
    assert!(
        history
            .iter()
            .all(|entry| entry.message.trim() != "main only"),
        "selected branch history should not jump to unrelated commits on another branch"
    );
}

#[test]
fn export_commit_patch_writes_patch_file() {
    let repo = TestRepo::new().expect("failed to create test repository");
    repo.add_and_commit("tracked.txt", "base\n", "base commit")
        .expect("failed to create base commit");

    let commit_id = git(repo.path(), &["rev-parse", "HEAD"]).expect("failed to read HEAD");
    let repository = Repository::open(repo.path()).expect("failed to reopen repository");
    let patch_path = repo.path().join("exported.patch");

    export_commit_patch(&repository, &commit_id, &patch_path).expect("failed to export patch");

    let patch = fs::read_to_string(&patch_path).expect("failed to read exported patch");
    assert!(patch.contains("Subject: [PATCH] base commit"));
    assert!(patch.contains("tracked.txt"));
}

#[test]
fn cherry_pick_commit_applies_selected_commit_on_current_branch() {
    let repo = TestRepo::new().expect("failed to create test repository");
    repo.add_and_commit("shared.txt", "base\n", "base commit")
        .expect("failed to create base commit");

    let repository = Repository::open(repo.path()).expect("failed to reopen repository");
    let main_branch = repository
        .current_branch()
        .expect("failed to read current branch")
        .expect("expected branch name");

    git(repo.path(), &["checkout", "-b", "feature/cherry-pick"])
        .expect("failed to create feature branch");
    repo.write_file("feature.txt", "feature\n")
        .expect("failed to write feature file");
    git(repo.path(), &["add", "feature.txt"]).expect("failed to stage feature file");
    git(repo.path(), &["commit", "-m", "feature commit"]).expect("failed to commit feature");

    let feature_commit =
        git(repo.path(), &["rev-parse", "HEAD"]).expect("failed to read feature HEAD");

    git(repo.path(), &["checkout", &main_branch]).expect("failed to switch back to main");
    let repository = Repository::open(repo.path()).expect("failed to reopen main branch");
    cherry_pick_commit(&repository, &feature_commit).expect("failed to cherry-pick feature commit");

    let head_message =
        git(repo.path(), &["log", "--format=%s", "-1", "HEAD"]).expect("failed to read HEAD");
    let feature_file =
        fs::read_to_string(repo.path().join("feature.txt")).expect("failed to read feature file");

    assert_eq!(head_message, "feature commit");
    assert_eq!(feature_file, "feature\n");
}

#[test]
fn in_progress_cherry_pick_can_be_detected_continued_and_aborted() {
    let (repo, feature_commit) = create_cherry_pick_conflict_repository();
    let repository = Repository::open(repo.path()).expect("failed to reopen repository");

    let error = cherry_pick_commit(&repository, &feature_commit)
        .expect_err("expected cherry-pick conflict");
    assert!(
        error.to_string().contains("cherry-pick"),
        "error should mention cherry-pick failure"
    );

    let in_progress =
        get_in_progress_commit_action(&repository).expect("failed to read in-progress action");
    let in_progress = in_progress.expect("expected cherry-pick to be in progress");
    assert_eq!(in_progress.kind, InProgressCommitActionKind::CherryPick);
    assert_eq!(
        in_progress.commit_id.as_deref(),
        Some(feature_commit.as_str())
    );
    assert_eq!(in_progress.conflicted_files, vec!["shared.txt".to_string()]);

    fs::write(repo.path().join("shared.txt"), "resolved\n").expect("failed to write resolved file");
    continue_in_progress_commit_action(&repository, InProgressCommitActionKind::CherryPick)
        .expect("failed to continue cherry-pick after resolving conflict");

    assert!(
        get_in_progress_commit_action(&repository)
            .expect("failed to refresh in-progress action")
            .is_none(),
        "cherry-pick should be finished after continue"
    );
    let head_message =
        git(repo.path(), &["log", "--format=%s", "-1", "HEAD"]).expect("failed to read HEAD");
    assert_eq!(head_message, "feature conflict commit");

    let (repo, feature_commit) = create_cherry_pick_conflict_repository();
    let repository = Repository::open(repo.path()).expect("failed to reopen repository");
    cherry_pick_commit(&repository, &feature_commit).expect_err("expected cherry-pick conflict");
    abort_in_progress_commit_action(&repository, InProgressCommitActionKind::CherryPick)
        .expect("failed to abort cherry-pick");

    assert!(
        get_in_progress_commit_action(&repository)
            .expect("failed to refresh in-progress action after abort")
            .is_none(),
        "cherry-pick should be cleared after abort"
    );
    let current_content =
        fs::read_to_string(repo.path().join("shared.txt")).expect("failed to read restored file");
    assert_eq!(current_content, "main\n");
}

#[test]
fn revert_commit_creates_inverse_commit_and_restores_content() {
    let repo = TestRepo::new().expect("failed to create test repository");
    repo.add_and_commit("shared.txt", "base\n", "base commit")
        .expect("failed to create base commit");

    repo.write_file("shared.txt", "base\nchanged\n")
        .expect("failed to write changed content");
    git(repo.path(), &["add", "shared.txt"]).expect("failed to stage changed content");
    git(repo.path(), &["commit", "-m", "change shared"]).expect("failed to commit changed content");

    let commit_to_revert =
        git(repo.path(), &["rev-parse", "HEAD"]).expect("failed to read changed HEAD");
    let repository = Repository::open(repo.path()).expect("failed to reopen repository");
    revert_commit(&repository, &commit_to_revert).expect("failed to revert commit");

    let head_message = git(repo.path(), &["log", "--format=%s", "-1", "HEAD"])
        .expect("failed to read revert HEAD");
    let restored =
        fs::read_to_string(repo.path().join("shared.txt")).expect("failed to read reverted file");

    assert!(head_message.contains("Revert"));
    assert_eq!(restored, "base\n");
}

#[test]
fn reset_current_branch_to_commit_rewinds_head_to_selected_ancestor() {
    let repo = TestRepo::new().expect("failed to create test repository");
    repo.add_and_commit("tracked.txt", "base\n", "base commit")
        .expect("failed to create base commit");

    repo.write_file("tracked.txt", "base\none\n")
        .expect("failed to write first update");
    git(repo.path(), &["add", "tracked.txt"]).expect("failed to stage first update");
    git(repo.path(), &["commit", "-m", "first update"]).expect("failed to commit first update");
    let reset_target =
        git(repo.path(), &["rev-parse", "HEAD"]).expect("failed to read reset target");

    repo.write_file("tracked.txt", "base\none\ntwo\n")
        .expect("failed to write second update");
    git(repo.path(), &["add", "tracked.txt"]).expect("failed to stage second update");
    git(repo.path(), &["commit", "-m", "second update"]).expect("failed to commit second update");

    let repository = Repository::open(repo.path()).expect("failed to reopen repository");
    reset_current_branch_to_commit(&repository, &reset_target)
        .expect("failed to reset current branch");

    let head_after_reset =
        git(repo.path(), &["rev-parse", "HEAD"]).expect("failed to read HEAD after reset");
    let file_content =
        fs::read_to_string(repo.path().join("tracked.txt")).expect("failed to read reset file");

    assert_eq!(head_after_reset, reset_target);
    assert_eq!(file_content, "base\none\n");
}

#[test]
fn push_current_branch_to_commit_rewinds_upstream_with_force_with_lease() {
    let repo = TestRepo::new().expect("failed to create test repository");
    repo.add_and_commit("tracked.txt", "base\n", "base commit")
        .expect("failed to create base commit");

    let remote = tempdir().expect("failed to create remote tempdir");
    git(remote.path(), &["init", "--bare"]).expect("failed to init bare remote");

    let repository = Repository::open(repo.path()).expect("failed to open local repository");
    let branch_name = repository
        .current_branch()
        .expect("failed to read current branch")
        .expect("expected branch name");

    git(
        repo.path(),
        &[
            "remote",
            "add",
            "origin",
            &remote.path().display().to_string(),
        ],
    )
    .expect("failed to add origin");
    git(repo.path(), &["push", "-u", "origin", &branch_name]).expect("failed to push base branch");

    repo.write_file("tracked.txt", "base\none\n")
        .expect("failed to write first local update");
    git(repo.path(), &["add", "tracked.txt"]).expect("failed to stage first local update");
    git(repo.path(), &["commit", "-m", "first local update"])
        .expect("failed to commit first local update");
    let publish_target =
        git(repo.path(), &["rev-parse", "HEAD"]).expect("failed to read publish target");

    repo.write_file("tracked.txt", "base\none\ntwo\n")
        .expect("failed to write second local update");
    git(repo.path(), &["add", "tracked.txt"]).expect("failed to stage second local update");
    git(repo.path(), &["commit", "-m", "second local update"])
        .expect("failed to commit second local update");
    git(repo.path(), &["push", "origin", &branch_name])
        .expect("failed to push second local update");

    let repository = Repository::open(repo.path()).expect("failed to reopen repository");
    let target = resolve_push_current_branch_target(&repository, &publish_target)
        .expect("failed to resolve push target");
    assert!(target.requires_force_with_lease);

    push_current_branch_to_commit(&repository, &target)
        .expect("failed to push branch to selected commit");

    let remote_head = git(
        remote.path(),
        &["rev-parse", &format!("refs/heads/{branch_name}")],
    )
    .expect("failed to read remote head");
    assert_eq!(remote_head, publish_target);
}

#[test]
fn sync_status_resets_after_pulling_current_upstream() {
    let repo = TestRepo::new().expect("failed to create test repository");
    repo.add_and_commit("tracked.txt", "base\n", "base commit")
        .expect("failed to create base commit");

    let remote = tempdir().expect("failed to create remote tempdir");
    git(remote.path(), &["init", "--bare"]).expect("failed to init bare remote");

    let repository = Repository::open(repo.path()).expect("failed to open local repository");
    let branch_name = repository
        .current_branch()
        .expect("failed to read current branch")
        .expect("expected branch name");

    git(
        repo.path(),
        &[
            "remote",
            "add",
            "origin",
            &remote.path().display().to_string(),
        ],
    )
    .expect("failed to add origin");
    git(repo.path(), &["push", "-u", "origin", &branch_name]).expect("failed to push main");

    let peer = tempdir().expect("failed to create peer tempdir");
    let peer_path = peer.path().display().to_string();
    git(
        repo.path(),
        &[
            "clone",
            "-b",
            &branch_name,
            &remote.path().display().to_string(),
            &peer_path,
        ],
    )
    .expect("failed to clone peer repository");
    git(peer.path(), &["config", "user.email", "peer@example.com"])
        .expect("failed to configure peer email");
    git(peer.path(), &["config", "user.name", "Peer User"]).expect("failed to configure peer name");
    fs::write(peer.path().join("tracked.txt"), "base\npeer\n").expect("failed to update peer file");
    git(peer.path(), &["add", "tracked.txt"]).expect("failed to stage peer file");
    git(peer.path(), &["commit", "-m", "peer update"]).expect("failed to commit peer update");
    git(peer.path(), &["push", "origin", &branch_name]).expect("failed to push peer update");

    let repository = Repository::open(repo.path()).expect("failed to reopen local repository");
    git_core::remote::fetch(&repository, "origin", None).expect("failed to fetch origin");
    assert_eq!(
        repository.sync_status(),
        SyncStatus::Behind(1),
        "local branch should report behind after remote update"
    );

    git_core::remote::pull(&repository, "origin", &branch_name, None)
        .expect("failed to pull current upstream");
    assert_eq!(
        repository.sync_status(),
        SyncStatus::Synced,
        "pulling current upstream should clear behind status"
    );
}

#[test]
fn conflict_diff_uses_correct_stages_and_resolve_conflict_writes_workdir_file() {
    let (repo, _) = create_conflicted_repository();
    let repository = Repository::open(repo.path()).expect("failed to reopen conflicted repository");

    let diff = get_conflict_diff(&repository, Path::new("shared.txt"))
        .expect("failed to build three-way diff for conflict");
    assert_eq!(diff.base_content.trim(), "base");
    assert_eq!(diff.ours_content.trim(), "ours");
    assert_eq!(diff.theirs_content.trim(), "theirs");

    resolve_conflict(
        &repository,
        Path::new("shared.txt"),
        ConflictResolution::Ours,
    )
    .expect("failed to resolve conflict with ours");

    let content =
        fs::read_to_string(repo.path().join("shared.txt")).expect("failed to read resolved file");
    assert_eq!(content, "ours\n");
    assert!(
        !git_core::index::has_conflicts(&repository),
        "repository should no longer report merge conflicts"
    );
}

#[test]
fn staged_diff_preview_stays_isolated_from_unstaged_edits() {
    let repo = TestRepo::new().expect("failed to create test repository");
    repo.add_and_commit("tracked.txt", "base\n", "base commit")
        .expect("failed to create base commit");

    repo.write_file("tracked.txt", "base\nstaged\n")
        .expect("failed to write staged content");
    git(repo.path(), &["add", "tracked.txt"]).expect("failed to stage tracked file");

    let repository = Repository::open(repo.path()).expect("failed to reopen repository");
    let staged_only = diff_index_to_head(&repository, Path::new("tracked.txt"))
        .expect("failed to diff index against HEAD");
    let staged_lines = staged_only
        .files
        .iter()
        .flat_map(|file| file.hunks.iter())
        .flat_map(|hunk| hunk.lines.iter())
        .map(|line| line.content.as_str())
        .collect::<Vec<_>>();
    assert!(
        staged_lines.iter().any(|line| line.contains("staged")),
        "staged preview should include staged content"
    );

    repo.write_file("tracked.txt", "base\nstaged\nunstaged\n")
        .expect("failed to write unstaged content");

    let repository = Repository::open(repo.path()).expect("failed to reopen repository");
    let staged_diff = diff_index_to_head(&repository, Path::new("tracked.txt"))
        .expect("failed to diff staged content after unstaged edit");
    let unstaged_diff = diff_file_to_index(&repository, Path::new("tracked.txt"))
        .expect("failed to diff worktree against index");

    let staged_lines = staged_diff
        .files
        .iter()
        .flat_map(|file| file.hunks.iter())
        .flat_map(|hunk| hunk.lines.iter())
        .map(|line| line.content.as_str())
        .collect::<Vec<_>>();
    let unstaged_lines = unstaged_diff
        .files
        .iter()
        .flat_map(|file| file.hunks.iter())
        .flat_map(|hunk| hunk.lines.iter())
        .map(|line| line.content.as_str())
        .collect::<Vec<_>>();

    assert!(
        staged_lines.iter().any(|line| line.contains("staged")),
        "staged preview should keep staged content visible"
    );
    assert!(
        staged_lines.iter().all(|line| !line.contains("unstaged")),
        "staged preview should not include unstaged worktree edits"
    );
    assert!(
        unstaged_lines.iter().any(|line| line.contains("unstaged")),
        "worktree preview should include only unstaged content"
    );
}

#[test]
fn checkout_remote_branch_creates_local_tracking_branch() {
    let repo = TestRepo::new().expect("failed to create test repository");
    repo.add_and_commit("tracked.txt", "base\n", "base commit")
        .expect("failed to create base commit");

    let remote = tempdir().expect("failed to create remote tempdir");
    git(remote.path(), &["init", "--bare"]).expect("failed to init bare remote");

    let repository = Repository::open(repo.path()).expect("failed to open local repository");
    let main_branch = repository
        .current_branch()
        .expect("failed to read current branch")
        .expect("expected branch name");

    git(
        repo.path(),
        &[
            "remote",
            "add",
            "origin",
            &remote.path().display().to_string(),
        ],
    )
    .expect("failed to add origin");
    git(repo.path(), &["push", "-u", "origin", &main_branch]).expect("failed to push main");

    git(repo.path(), &["checkout", "-b", "feature/remote-checkout"])
        .expect("failed to create feature branch");
    repo.write_file("tracked.txt", "base\nfeature\n")
        .expect("failed to update tracked file");
    git(repo.path(), &["add", "tracked.txt"]).expect("failed to stage feature file");
    git(repo.path(), &["commit", "-m", "feature commit"]).expect("failed to commit feature");
    git(
        repo.path(),
        &["push", "-u", "origin", "feature/remote-checkout"],
    )
    .expect("failed to push feature branch");

    git(repo.path(), &["checkout", &main_branch]).expect("failed to switch back to main branch");
    git(repo.path(), &["branch", "-D", "feature/remote-checkout"])
        .expect("failed to delete local feature branch");

    let repository = Repository::open(repo.path()).expect("failed to reopen repository");
    let local_branch = repository
        .checkout_remote_branch("origin/feature/remote-checkout")
        .expect("failed to checkout remote branch");
    assert_eq!(local_branch, "feature/remote-checkout");

    let repository = Repository::open(repo.path()).expect("failed to reopen repository");
    assert_eq!(
        repository
            .current_branch()
            .expect("failed to read current branch"),
        Some("feature/remote-checkout".to_string())
    );

    let branches = repository
        .list_branches()
        .expect("failed to list branches after remote checkout");
    let local_feature = branches
        .iter()
        .find(|branch| branch.name == "feature/remote-checkout" && !branch.is_remote)
        .expect("missing local tracking branch");
    assert!(local_feature.is_head);
    assert_eq!(
        local_feature.upstream.as_deref(),
        Some("origin/feature/remote-checkout")
    );
}

#[test]
fn rebase_start_rebases_current_branch_onto_target_branch() {
    let repo = TestRepo::new().expect("failed to create test repository");
    repo.add_and_commit("shared.txt", "base\n", "base commit")
        .expect("failed to create base commit");

    let repository = Repository::open(repo.path()).expect("failed to reopen test repository");
    let main_branch = repository
        .current_branch()
        .expect("failed to read current branch")
        .expect("expected a branch");

    git(repo.path(), &["checkout", "-b", "feature"]).expect("failed to create feature branch");
    repo.write_file("feature.txt", "feature\n")
        .expect("failed to write feature file");
    git(repo.path(), &["add", "feature.txt"]).expect("failed to stage feature file");
    git(repo.path(), &["commit", "-m", "feature commit"]).expect("failed to commit feature");

    git(repo.path(), &["checkout", &main_branch]).expect("failed to switch to main branch");
    repo.write_file("main.txt", "main\n")
        .expect("failed to write main file");
    git(repo.path(), &["add", "main.txt"]).expect("failed to stage main file");
    git(repo.path(), &["commit", "-m", "main commit"]).expect("failed to commit main");

    git(repo.path(), &["checkout", "feature"]).expect("failed to switch back to feature branch");
    let repository = Repository::open(repo.path()).expect("failed to reopen feature branch");

    rebase_start(&repository, &main_branch).expect("rebase should start successfully");

    let head_message =
        git(repo.path(), &["log", "--format=%s", "-1", "HEAD"]).expect("failed to read HEAD");
    let parent_message =
        git(repo.path(), &["log", "--format=%s", "-1", "HEAD^"]).expect("failed to read HEAD^");

    assert_eq!(head_message, "feature commit");
    assert_eq!(parent_message, "main commit");
}
