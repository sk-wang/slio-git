//! Additional integration tests for git-core modules.
//!
//! Tests: index operations, diff, commit operations, graph, submodule

mod test_helpers;

use git_core::Repository;
use std::path::Path;
use test_helpers::TestRepo;

fn default_branch(repo: &TestRepo) -> String {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(repo.path())
        .output()
        .expect("failed to get default branch");

    String::from_utf8(output.stdout)
        .expect("default branch should be utf-8")
        .trim()
        .to_string()
}

// ── T200: Index Operations ────────────────────────────────────────────────────

#[test]
fn stage_file_adds_to_index() {
    let repo = TestRepo::new().unwrap();
    repo.add_and_commit("initial.txt", "initial", "initial commit")
        .unwrap();

    // Create a new file
    repo.write_file("new_file.txt", "new content").unwrap();

    let r = Repository::discover(repo.path()).unwrap();

    // Stage the file
    git_core::stage_file(&r, Path::new("new_file.txt")).unwrap();

    // Verify it's staged
    let changes = git_core::get_status(&r).unwrap();
    let staged = changes
        .iter()
        .find(|c| c.path == "new_file.txt" && c.staged);
    assert!(staged.is_some(), "file should be staged");
}

#[test]
fn unstage_file_removes_from_index() {
    let repo = TestRepo::new().unwrap();
    repo.add_and_commit("initial.txt", "initial", "initial commit")
        .unwrap();

    // Create and stage a file
    repo.write_file("staged.txt", "staged content").unwrap();
    let r = Repository::discover(repo.path()).unwrap();
    git_core::stage_file(&r, Path::new("staged.txt")).unwrap();

    // Verify it's staged
    let changes = git_core::get_status(&r).unwrap();
    let staged = changes.iter().find(|c| c.path == "staged.txt" && c.staged);
    assert!(staged.is_some(), "file should be staged initially");

    // Unstage it
    git_core::unstage_file(&r, Path::new("staged.txt")).unwrap();

    // Verify it's unstaged (untracked)
    let changes = git_core::get_status(&r).unwrap();
    let unstaged = changes.iter().find(|c| c.path == "staged.txt" && !c.staged);
    assert!(unstaged.is_some(), "file should be unstaged");
}

#[test]
fn stage_modified_file_updates_index() {
    let repo = TestRepo::new().unwrap();
    repo.add_and_commit("file.txt", "original", "initial commit")
        .unwrap();

    // Modify the file
    repo.write_file("file.txt", "modified content").unwrap();

    let r = Repository::discover(repo.path()).unwrap();

    // Verify unstaged change exists
    let changes = git_core::get_status(&r).unwrap();
    let unstaged = changes.iter().find(|c| c.path == "file.txt" && !c.staged);
    assert!(unstaged.is_some(), "should have unstaged changes");

    // Stage the modification
    git_core::stage_file(&r, Path::new("file.txt")).unwrap();

    // Verify it's staged
    let changes = git_core::get_status(&r).unwrap();
    let staged = changes.iter().find(|c| c.path == "file.txt" && c.staged);
    assert!(staged.is_some(), "modified file should be staged");
}

#[test]
fn discard_file_changes_restores_original() {
    let repo = TestRepo::new().unwrap();
    repo.add_and_commit("file.txt", "original content", "initial commit")
        .unwrap();

    // Modify the file
    repo.write_file("file.txt", "modified content").unwrap();

    let r = Repository::discover(repo.path()).unwrap();

    // Verify unstaged change exists
    let changes = git_core::get_status(&r).unwrap();
    assert!(
        changes.iter().any(|c| c.path == "file.txt"),
        "should have changes"
    );

    // Discard changes
    git_core::discard_file(&r, Path::new("file.txt")).unwrap();

    // Verify no changes and file restored
    let changes = git_core::get_status(&r).unwrap();
    assert!(
        !changes.iter().any(|c| c.path == "file.txt"),
        "should have no changes"
    );

    let content = std::fs::read_to_string(repo.path().join("file.txt")).unwrap();
    assert_eq!(content, "original content", "file should be restored");
}

#[test]
fn get_changes_detects_untracked_files() {
    let repo = TestRepo::new().unwrap();
    repo.add_and_commit("initial.txt", "initial", "initial commit")
        .unwrap();

    // Create an untracked file
    repo.write_file("untracked.txt", "untracked content")
        .unwrap();

    let r = Repository::discover(repo.path()).unwrap();
    let changes = git_core::get_status(&r).unwrap();

    let untracked = changes.iter().find(|c| c.path == "untracked.txt");
    assert!(untracked.is_some(), "should detect untracked file");
    assert!(
        !untracked.unwrap().staged,
        "untracked file should not be staged"
    );
}

#[test]
fn get_changes_detects_deleted_files() {
    let repo = TestRepo::new().unwrap();
    repo.add_and_commit("file.txt", "content", "initial commit")
        .unwrap();

    // Delete the file
    std::fs::remove_file(repo.path().join("file.txt")).unwrap();

    let r = Repository::discover(repo.path()).unwrap();
    let changes = git_core::get_status(&r).unwrap();

    let deleted = changes.iter().find(|c| c.path == "file.txt");
    assert!(deleted.is_some(), "should detect deleted file");
    assert!(matches!(
        deleted.unwrap().status,
        git_core::ChangeStatus::Deleted
    ));
}

// ── T201: Diff Operations ─────────────────────────────────────────────────────

#[test]
fn diff_workdir_to_index_detects_unstaged_changes() {
    let repo = TestRepo::new().unwrap();
    repo.add_and_commit("file.txt", "original", "initial commit")
        .unwrap();

    // Modify without staging
    repo.write_file("file.txt", "modified").unwrap();

    let r = Repository::discover(repo.path()).unwrap();
    let diff = git_core::diff_workdir_to_index(&r).unwrap();

    assert!(!diff.files.is_empty(), "should have diff");
    let file_diff = diff
        .files
        .iter()
        .find(|f| f.new_path.as_ref().unwrap() == "file.txt");
    assert!(file_diff.is_some(), "should have diff for file.txt");
}

#[test]
fn diff_index_to_head_detects_staged_changes() {
    let repo = TestRepo::new().unwrap();
    repo.add_and_commit("file.txt", "original", "initial commit")
        .unwrap();

    // Modify and stage
    repo.write_file("file.txt", "modified").unwrap();
    let r = Repository::discover(repo.path()).unwrap();
    git_core::stage_file(&r, Path::new("file.txt")).unwrap();

    let diff = git_core::diff_index_to_head(&r, Path::new("file.txt")).unwrap();

    assert!(!diff.files.is_empty(), "should have staged diff");
}

#[test]
fn diff_commits_shows_changes_between_commits() {
    let repo = TestRepo::new().unwrap();
    repo.add_and_commit("file.txt", "version 1", "commit 1")
        .unwrap();
    repo.add_and_commit("file.txt", "version 2", "commit 2")
        .unwrap();

    let r = Repository::discover(repo.path()).unwrap();
    let history = git_core::get_history(&r, Some(10)).unwrap();

    let commit1_id = &history[1].id; // Second commit in history (older)
    let commit2_id = &history[0].id; // First commit in history (newer)

    let diff = git_core::diff_commits(&r, commit1_id, commit2_id).unwrap();

    assert!(!diff.files.is_empty(), "should have diff between commits");

    // Check for the content change
    let file_diff = diff
        .files
        .iter()
        .find(|f| f.new_path.as_ref().unwrap() == "file.txt");
    assert!(file_diff.is_some(), "should have diff for file.txt");
}

#[test]
fn diff_file_to_index_shows_single_file_changes() {
    let repo = TestRepo::new().unwrap();
    repo.add_and_commit("file.txt", "original", "initial commit")
        .unwrap();
    repo.add_and_commit("other.txt", "other", "second commit")
        .unwrap();

    // Modify only file.txt
    repo.write_file("file.txt", "modified").unwrap();

    let r = Repository::discover(repo.path()).unwrap();
    let diff = git_core::diff_file_to_index(&r, Path::new("file.txt")).unwrap();

    assert_eq!(diff.files.len(), 1, "should only have one file diff");
    assert_eq!(diff.files[0].new_path.as_ref().unwrap(), "file.txt");
}

#[test]
fn diff_refs_shows_changes_between_branches() {
    let repo = TestRepo::new().unwrap();
    repo.add_and_commit("file.txt", "main content", "main commit")
        .unwrap();

    let default_branch = default_branch(&repo);

    // Create and switch to feature branch
    std::process::Command::new("git")
        .args(["checkout", "-b", "feature"])
        .current_dir(repo.path())
        .output()
        .unwrap();

    repo.add_and_commit("file.txt", "feature content", "feature commit")
        .unwrap();

    let r = Repository::discover(repo.path()).unwrap();
    let diff = git_core::diff_refs(&r, &default_branch, "feature").unwrap();

    assert!(!diff.files.is_empty(), "should have diff between branches");
}

#[test]
fn compute_inline_changes_detects_character_diffs() {
    let repo = TestRepo::new().unwrap();
    repo.add_and_commit("file.txt", "hello world", "initial commit")
        .unwrap();

    repo.write_file("file.txt", "hello rustacean").unwrap();

    let r = Repository::discover(repo.path()).unwrap();
    let diff = git_core::diff_workdir_to_index(&r).unwrap();

    // Compute inline changes for a deletion/addition pair
    if let Some(file) = diff.files.first() {
        if let Some(hunk) = file.hunks.first() {
            let deletions: Vec<_> = hunk.lines.iter()
                .filter(|l| l.origin == git_core::DiffLineOrigin::Deletion)
                .collect();
            let additions: Vec<_> = hunk.lines.iter()
                .filter(|l| l.origin == git_core::DiffLineOrigin::Addition)
                .collect();
            if let (Some(del), Some(add)) = (deletions.first(), additions.first()) {
                let (old_spans, new_spans) = git_core::compute_inline_changes(&del.content, &add.content);
                // Should have some spans marked as changed
                assert!(!old_spans.is_empty() || !new_spans.is_empty(), "should have inline change spans");
            }
        }
    }
}

// ── T202: Commit Operations ───────────────────────────────────────────────────

#[test]
fn amend_commit_updates_message() {
    let repo = TestRepo::new().unwrap();
    repo.add_and_commit("file.txt", "content", "original message")
        .unwrap();

    let r = Repository::discover(repo.path()).unwrap();
    let history_before = git_core::get_history(&r, Some(10)).unwrap();
    let commit_id = history_before[0].id.clone();

    // Amend the commit
    let new_id = git_core::amend_commit(&r, &commit_id, "amended message").unwrap();

    // Verify the commit was amended
    let history_after = git_core::get_history(&r, Some(10)).unwrap();
    assert_eq!(history_after[0].message, "amended message");
    assert_ne!(new_id, commit_id, "commit id should change after amend");
}

#[test]
fn validate_commit_ref_accepts_full_hash() {
    let repo = TestRepo::new().unwrap();
    repo.add_and_commit("file.txt", "content", "test commit")
        .unwrap();

    let r = Repository::discover(repo.path()).unwrap();
    let history = git_core::get_history(&r, Some(10)).unwrap();
    let full_hash = &history[0].id;

    let (hash, summary) = git_core::validate_commit_ref(&r, full_hash).unwrap();
    assert_eq!(hash, *full_hash);
    assert_eq!(summary, "test commit");
}

#[test]
fn validate_commit_ref_accepts_short_hash() {
    let repo = TestRepo::new().unwrap();
    repo.add_and_commit("file.txt", "content", "test commit")
        .unwrap();

    let r = Repository::discover(repo.path()).unwrap();
    let history = git_core::get_history(&r, Some(10)).unwrap();
    let short_hash = &history[0].id[..7];

    let (hash, summary) = git_core::validate_commit_ref(&r, short_hash).unwrap();
    assert_eq!(summary, "test commit");
    assert!(hash.starts_with(short_hash));
}

#[test]
fn validate_commit_ref_accepts_branch_name() {
    let repo = TestRepo::new().unwrap();
    repo.add_and_commit("file.txt", "content", "branch commit")
        .unwrap();

    let r = Repository::discover(repo.path()).unwrap();
    let branch = default_branch(&repo);

    let (hash, summary) = git_core::validate_commit_ref(&r, &branch).unwrap();
    assert_eq!(summary, "branch commit");
    assert!(!hash.is_empty());
}

#[test]
fn get_commit_returns_full_info() {
    let repo = TestRepo::new().unwrap();
    repo.add_and_commit("file.txt", "content", "test commit")
        .unwrap();

    let r = Repository::discover(repo.path()).unwrap();
    let history = git_core::get_history(&r, Some(10)).unwrap();
    let commit_id = &history[0].id;

    let commit = git_core::get_commit(&r, commit_id).unwrap();

    assert_eq!(commit.id, *commit_id);
    assert_eq!(commit.message.trim(), "test commit");
    assert_eq!(commit.author_name, "Codex Test");
    assert_eq!(commit.author_email, "codex@example.com");
}

#[test]
fn create_commit_on_unborn_branch_works() {
    let repo = TestRepo::new().unwrap();

    // Create a file without committing
    repo.write_file("file.txt", "initial content").unwrap();

    let r = Repository::discover(repo.path()).unwrap();
    git_core::stage_file(&r, Path::new("file.txt")).unwrap();

    // Create first commit
    let commit_id =
        git_core::create_commit(&r, "first commit", "Test", "test@example.com").unwrap();

    assert!(!commit_id.is_empty(), "should return commit id");

    let history = git_core::get_history(&r, Some(10)).unwrap();
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].message, "first commit");
}

// ── T203: Graph Visualization ─────────────────────────────────────────────────

#[test]
fn compute_graph_handles_merge_commits() {
    let repo = TestRepo::new().unwrap();
    repo.add_and_commit("file.txt", "main 1", "main commit 1")
        .unwrap();
    repo.add_and_commit("file.txt", "main 2", "main commit 2")
        .unwrap();

    let default_branch = default_branch(&repo);

    // Create feature branch
    std::process::Command::new("git")
        .args(["checkout", "-b", "feature"])
        .current_dir(repo.path())
        .output()
        .unwrap();
    repo.add_and_commit("feature.txt", "feature content", "feature commit")
        .unwrap();

    // Merge feature into default branch
    std::process::Command::new("git")
        .args(["checkout", &default_branch])
        .current_dir(repo.path())
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["merge", "--no-ff", "feature", "-m", "merge feature"])
        .current_dir(repo.path())
        .output()
        .unwrap();

    let r = Repository::discover(repo.path()).unwrap();
    let history = git_core::get_history(&r, Some(10)).unwrap();
    let ids: Vec<String> = history.iter().map(|e| e.id.clone()).collect();

    let nodes = git_core::compute_graph(&r, &ids).unwrap();

    // Find the merge commit
    let merge_node = nodes.iter().find(|n| n.is_merge);
    assert!(merge_node.is_some(), "should have a merge commit node");

    // Merge commit should have multiple parent edges
    assert!(
        merge_node.unwrap().parent_edges.len() > 1,
        "merge should have multiple parents"
    );
}

#[test]
fn compute_graph_assigns_increasing_lanes_for_branches() {
    let repo = TestRepo::new().unwrap();
    repo.add_and_commit("file.txt", "main 1", "main commit 1")
        .unwrap();

    let default_branch = default_branch(&repo);

    // Create feature branch
    std::process::Command::new("git")
        .args(["checkout", "-b", "feature"])
        .current_dir(repo.path())
        .output()
        .unwrap();
    repo.add_and_commit("feature.txt", "feature", "feature commit")
        .unwrap();

    // Go back to default branch and commit
    std::process::Command::new("git")
        .args(["checkout", &default_branch])
        .current_dir(repo.path())
        .output()
        .unwrap();
    repo.add_and_commit("file.txt", "main 2", "main commit 2")
        .unwrap();

    // Merge feature so all commits are reachable from HEAD
    std::process::Command::new("git")
        .args(["merge", "--no-ff", "feature", "-m", "merge feature"])
        .current_dir(repo.path())
        .output()
        .unwrap();

    let r = Repository::discover(repo.path()).unwrap();
    let history = git_core::get_history(&r, Some(10)).unwrap();
    let ids: Vec<String> = history.iter().map(|e| e.id.clone()).collect();

    let nodes = git_core::compute_graph(&r, &ids).unwrap();

    // Should have multiple lanes for divergent branches
    let max_lane = nodes.iter().map(|n| n.lane).max().unwrap();
    assert!(
        max_lane >= 1,
        "should have at least 2 lanes for divergent branches"
    );
}

#[test]
fn compute_ref_labels_includes_all_ref_types() {
    let repo = TestRepo::new().unwrap();
    repo.add_and_commit("file.txt", "content", "initial commit")
        .unwrap();

    // Create a tag
    std::process::Command::new("git")
        .args(["tag", "v1.0.0"])
        .current_dir(repo.path())
        .output()
        .unwrap();

    // Create a remote branch reference (simulate)
    std::process::Command::new("git")
        .args(["update-ref", "refs/remotes/origin/main", "HEAD"])
        .current_dir(repo.path())
        .output()
        .unwrap();

    let r = Repository::discover(repo.path()).unwrap();
    let labels = git_core::compute_ref_labels(&r).unwrap();

    let has_local_branch = labels
        .values()
        .flatten()
        .any(|l| matches!(l.ref_type, git_core::RefType::LocalBranch));
    let has_tag = labels
        .values()
        .flatten()
        .any(|l| matches!(l.ref_type, git_core::RefType::Tag));

    assert!(has_local_branch, "should have local branch labels");
    assert!(has_tag, "should have tag labels");
}

// ── T204: Submodule Operations ────────────────────────────────────────────────

#[test]
fn is_submodule_detects_submodule_path() {
    let repo = TestRepo::new().unwrap();
    repo.add_and_commit("file.txt", "content", "initial commit")
        .unwrap();

    // Create a submodule
    let submodule_path = repo.path().join("sub");
    std::fs::create_dir(&submodule_path).unwrap();

    // Initialize a git repo in the submodule
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(&submodule_path)
        .output()
        .unwrap();

    std::process::Command::new("git")
        .args(["config", "user.name", "Sub"])
        .current_dir(&submodule_path)
        .output()
        .unwrap();

    std::process::Command::new("git")
        .args(["config", "user.email", "sub@example.com"])
        .current_dir(&submodule_path)
        .output()
        .unwrap();

    std::fs::write(submodule_path.join("subfile.txt"), "sub content").unwrap();
    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(&submodule_path)
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["commit", "-m", "initial"])
        .current_dir(&submodule_path)
        .output()
        .unwrap();

    // Add as submodule to main repo
    std::process::Command::new("git")
        .args(["submodule", "add", "./sub", "sub"])
        .current_dir(repo.path())
        .output()
        .unwrap();

    let r = Repository::discover(repo.path()).unwrap();

    assert!(
        git_core::is_submodule(&r, "sub"),
        "should detect sub as submodule"
    );
    assert!(
        !git_core::is_submodule(&r, "file.txt"),
        "should not detect regular file as submodule"
    );
}

#[test]
fn list_submodules_returns_empty_for_no_submodules() {
    let repo = TestRepo::new().unwrap();
    repo.add_and_commit("file.txt", "content", "initial commit")
        .unwrap();

    let r = Repository::discover(repo.path()).unwrap();
    let submodules = git_core::list_submodules(&r).unwrap();

    assert!(
        submodules.is_empty(),
        "should return empty list when no submodules"
    );
}

#[test]
fn submodule_summary_returns_none_for_unchanged_submodule() {
    let repo = TestRepo::new().unwrap();
    repo.add_and_commit("file.txt", "content", "initial commit")
        .unwrap();

    // Create and add a submodule
    let submodule_path = repo.path().join("sub");
    std::fs::create_dir(&submodule_path).unwrap();
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(&submodule_path)
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["config", "user.name", "Sub"])
        .current_dir(&submodule_path)
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["config", "user.email", "sub@example.com"])
        .current_dir(&submodule_path)
        .output()
        .unwrap();
    std::fs::write(submodule_path.join("file.txt"), "content").unwrap();
    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(&submodule_path)
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["commit", "-m", "initial"])
        .current_dir(&submodule_path)
        .output()
        .unwrap();

    std::process::Command::new("git")
        .args(["submodule", "add", "./sub", "sub"])
        .current_dir(repo.path())
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["commit", "-m", "add submodule"])
        .current_dir(repo.path())
        .output()
        .unwrap();

    let r = Repository::discover(repo.path()).unwrap();

    // Submodule at HEAD, no changes
    let summary = git_core::submodule_summary(&r, "sub");
    assert!(
        summary.is_none(),
        "should return None for unchanged submodule"
    );
}

// ── T205: Edge Cases ──────────────────────────────────────────────────────────

#[test]
fn stage_nonexistent_file_fails() {
    let repo = TestRepo::new().unwrap();
    repo.add_and_commit("file.txt", "content", "initial commit")
        .unwrap();

    let r = Repository::discover(repo.path()).unwrap();
    let result = git_core::stage_file(&r, Path::new("nonexistent.txt"));

    assert!(result.is_err(), "staging nonexistent file should fail");
}

#[test]
fn validate_invalid_commit_ref_fails() {
    let repo = TestRepo::new().unwrap();
    repo.add_and_commit("file.txt", "content", "initial commit")
        .unwrap();

    let r = Repository::discover(repo.path()).unwrap();
    let result = git_core::validate_commit_ref(&r, "invalid-ref-12345");

    assert!(result.is_err(), "validating invalid ref should fail");
}

#[test]
fn diff_between_same_commit_is_empty() {
    let repo = TestRepo::new().unwrap();
    repo.add_and_commit("file.txt", "content", "initial commit")
        .unwrap();

    let r = Repository::discover(repo.path()).unwrap();
    let history = git_core::get_history(&r, Some(10)).unwrap();
    let commit_id = &history[0].id;

    let diff = git_core::diff_commits(&r, commit_id, commit_id).unwrap();

    assert!(
        diff.files.is_empty(),
        "diff between same commit should be empty"
    );
    assert_eq!(diff.total_additions, 0);
    assert_eq!(diff.total_deletions, 0);
}

#[test]
fn get_commit_with_invalid_id_fails() {
    let repo = TestRepo::new().unwrap();
    repo.add_and_commit("file.txt", "content", "initial commit")
        .unwrap();

    let r = Repository::discover(repo.path()).unwrap();
    let result = git_core::get_commit(&r, "0000000000000000000000000000000000000000");

    assert!(
        result.is_err(),
        "getting commit with invalid id should fail"
    );
}

#[test]
fn amend_nonexistent_commit_fails() {
    let repo = TestRepo::new().unwrap();
    repo.add_and_commit("file.txt", "content", "initial commit")
        .unwrap();

    let r = Repository::discover(repo.path()).unwrap();
    let result = git_core::amend_commit(
        &r,
        "0000000000000000000000000000000000000000",
        "new message",
    );

    assert!(result.is_err(), "amending nonexistent commit should fail");
}
