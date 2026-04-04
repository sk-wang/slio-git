//! git-core: Core Git operations library for slio-git
//!
//! This library provides a clean, testable API for Git operations,
//! decoupled from any UI framework.

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// Helper to create a test repository - returns both the repo and the temp dir
    fn create_test_repo() -> Result<(Repository, TempDir), GitError> {
        let temp_dir = TempDir::new().map_err(|e| GitError::Io(std::io::Error::other(e)))?;
        let repo = Repository::init(temp_dir.path())?;
        Ok((repo, temp_dir))
    }

    #[test]
    fn test_init_repository() {
        let result = create_test_repo();
        assert!(result.is_ok());
    }

    #[test]
    fn test_discover_repository() {
        let (repo, _temp_dir) = create_test_repo().expect("Failed to create test repo");
        let discovered = Repository::discover(repo.path());
        assert!(discovered.is_ok());
    }

    #[test]
    fn test_discover_non_repository() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let result = Repository::discover(temp_dir.path());
        assert!(result.is_err());
    }
}

pub mod blame;
pub mod branch;
pub mod commit;
pub mod commit_actions;
pub mod diff;
pub mod error;
pub mod git_utils;
pub mod graph;
pub mod history;
pub mod index;
pub mod rebase;
pub mod remote;
pub mod repository;
pub mod signature;
pub mod stash;
pub mod submodule;
pub mod tag;
pub mod worktree;

pub use branch::Branch;
pub use commit::{
    amend_commit, create_commit, create_signature, get_commit, get_default_signature,
    load_recent_messages, save_recent_message, validate_commit_ref, CommitInfo,
};
pub use commit_actions::{
    abort_in_progress_commit_action, cherry_pick_commit, continue_in_progress_commit_action,
    drop_commit_from_history, edit_commit_message, export_commit_patch, fixup_commit_to_previous,
    get_in_progress_commit_action, push_current_branch_to_commit, reset_current_branch_to_commit,
    resolve_push_current_branch_target, revert_commit, squash_commit_to_previous,
    uncommit_to_commit, InProgressCommitAction, InProgressCommitActionKind,
    PushCurrentBranchTarget, RewriteExecution,
};
pub use diff::{
    auto_merge_conflict, build_full_file_diff, diff_file_to_index, diff_index_to_head,
    diff_ref_to_workdir, diff_refs, file_is_binary, get_conflict_diff, resolve_conflict,
    resolve_conflict_hunk, AutoMergeResult, ConflictHunk, ConflictHunkType, ConflictLine,
    ConflictLineType, ConflictResolution, Diff, DiffHunk, DiffLine, DiffLineOrigin, FileDiff,
    FullFilePreview, InlineChangeSpan, ThreeWayDiff,
};
pub use error::GitError;
pub use history::{
    get_history, get_history_for_author, get_history_for_date_range, get_history_for_path,
    get_history_for_ref, search_history, HistoryEntry,
};
pub use index::{
    get_file_hunks, stage_hunk, unstage_hunk, Change, ChangeStatus, Hunk, HunkLine, Index,
    IndexEntry,
};
pub use rebase::{
    get_current_rebase_step, get_rebase_status, get_rebase_todo, has_rebase_conflicts,
    prepare_interactive_rebase_plan, rebase_abort, rebase_continue, rebase_skip, rebase_start,
    start_interactive_rebase, InteractiveRebasePlan, RebaseResult, RebaseStatus, RebaseTodoEntry,
};
pub use remote::{
    fetch, force_push, list_branch_scoped_remotes, list_remotes, pull, push, RemoteInfo,
};
pub use repository::{Repository, RepositoryManager, SyncStatus};
pub use stash::{
    list_stashes, stash_apply, stash_clear, stash_diff, stash_drop, stash_pop, stash_save,
    stash_save_with_options, unstash_as_branch, StashInfo,
};
pub use tag::{
    create_lightweight_tag, create_tag, delete_remote_tag, delete_tag, list_tags, push_tag, TagInfo,
};

// New modules for IDEA git parity
pub use blame::{blame_file, BlameEntry};
pub use graph::{compute_graph, compute_ref_labels, EdgeType, GraphEdge, GraphNode, RefLabel, RefType};
pub use signature::{verify_commit_signature, SignatureCache, SignatureStatus, SignatureType};
pub use submodule::{is_submodule, list_submodules, submodule_summary, SubmoduleChange};
pub use worktree::{create_worktree, list_worktrees, remove_worktree, WorkingTree};

use log::info;
use std::path::Path;

/// Initialize logging for git-core
pub fn init_logging() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    info!("git-core initialized");
}

/// Discover a git repository at the given path
pub fn discover_repository(path: &Path) -> Result<Repository, GitError> {
    Repository::discover(path)
}

/// Initialize a new git repository at the given path
pub fn init_repository(path: &Path) -> Result<Repository, GitError> {
    Repository::init(path)
}
