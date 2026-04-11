//! Repository management for git-core

use crate::error::GitError;
use chrono::{Local, LocalResult, TimeZone};
use git2::Repository as Git2Repository;
use log::info;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

/// Repository state
#[derive(Debug, Clone, PartialEq)]
pub enum RepositoryState {
    Clean,
    Dirty,
    Merging,
    Rebasing,
    ApplyMailbox,
    Bisect,
    CherryPick,
    Revert,
}

/// Sync status with upstream branch
#[derive(Debug, Clone, PartialEq)]
pub enum SyncStatus {
    Ahead(usize),
    Behind(usize),
    Diverged { ahead: usize, behind: usize },
    Synced,
    NoUpstream,
    Unknown,
}

impl SyncStatus {
    /// Get the display text for this sync status
    pub fn display_text(&self) -> String {
        match self {
            SyncStatus::Ahead(n) => format!("↑{}", n),
            SyncStatus::Behind(n) => format!("↓{}", n),
            SyncStatus::Diverged { ahead, behind } => format!("↕{}/{}", ahead, behind),
            SyncStatus::Synced => "✓".to_string(),
            SyncStatus::NoUpstream => "○".to_string(),
            SyncStatus::Unknown => "?".to_string(),
        }
    }

    /// Get the display color hint (R, G, B)
    pub fn display_color(&self) -> [f32; 3] {
        match self {
            SyncStatus::Ahead(_) => [0.0, 0.5, 0.0],        // Green
            SyncStatus::Behind(_) => [0.8, 0.4, 0.0],       // Orange
            SyncStatus::Diverged { .. } => [0.8, 0.0, 0.0], // Red
            SyncStatus::Synced => [0.0, 0.6, 0.0],          // Green
            SyncStatus::NoUpstream => [0.5, 0.5, 0.5],      // Gray
            SyncStatus::Unknown => [0.5, 0.5, 0.5],         // Gray
        }
    }

    /// Get a localized hint suitable for compact UI display.
    pub fn hint_text(&self) -> Option<String> {
        match self {
            SyncStatus::Ahead(count) => Some(format!("领先上游 {count} 个提交")),
            SyncStatus::Behind(count) => Some(format!("落后上游 {count} 个提交")),
            SyncStatus::Diverged { ahead, behind } => {
                Some(format!("与上游分叉：领先 {ahead} / 落后 {behind}"))
            }
            SyncStatus::Synced => Some("与上游同步".to_string()),
            SyncStatus::NoUpstream => None,
            SyncStatus::Unknown => Some("上游状态未知".to_string()),
        }
    }
}

/// A Git repository managed by git-core
#[derive(Clone)]
#[allow(clippy::arc_with_non_send_sync)]
pub struct Repository {
    pub path: PathBuf,
    pub workdir: Option<PathBuf>,
    pub state: RepositoryState,
    pub(crate) inner: Arc<RwLock<Git2Repository>>,
}

impl Repository {
    /// Get repository path
    pub fn path(&self) -> &Path {
        self.workdir.as_deref().unwrap_or(&self.path)
    }

    /// Get a working directory suitable for running git commands that need a work tree.
    pub fn command_cwd(&self) -> PathBuf {
        if let Some(workdir) = self.workdir.as_ref() {
            return workdir.clone();
        }

        if self.path.file_name().and_then(|name| name.to_str()) == Some(".git") {
            if let Some(parent) = self.path.parent() {
                return parent.to_path_buf();
            }
        }

        self.path.clone()
    }

    /// Get repository name (last component of path)
    pub fn name(&self) -> String {
        self.path()
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string()
    }

    /// Check if this is a worktree (not the main repository)
    pub fn is_worktree(&self) -> bool {
        self.path.join(".git").is_file()
    }

    /// Get worktree paths for this repository (if it's the main repo)
    pub fn list_worktrees(&self) -> Vec<PathBuf> {
        let mut worktrees = Vec::new();
        let worktrees_dir = self.path.join(".git").join("worktrees");
        if let Ok(entries) = std::fs::read_dir(worktrees_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    worktrees.push(path);
                }
            }
        }
        worktrees
    }

    /// Discover and open a repository at the given path
    pub fn discover(path: &Path) -> Result<Self, GitError> {
        info!("Discovering repository at {:?}", path);

        let repo = Git2Repository::discover(path).map_err(|_| GitError::RepositoryNotFound {
            path: path.display().to_string(),
        })?;

        Self::from_git2(repo)
    }

    /// Initialize a new repository at the given path
    pub fn init(path: &Path) -> Result<Self, GitError> {
        info!("Initializing new repository at {:?}", path);

        let repo = Git2Repository::init(path).map_err(|e| GitError::OperationFailed {
            operation: "init".to_string(),
            details: e.to_string(),
        })?;

        Self::from_git2(repo)
    }

    /// Open an existing repository
    pub fn open(path: &Path) -> Result<Self, GitError> {
        info!("Opening repository at {:?}", path);

        let repo = Git2Repository::open(path).map_err(|_| GitError::RepositoryNotFound {
            path: path.display().to_string(),
        })?;

        Self::from_git2(repo)
    }

    /// Create a Repository from a git2 Repository
    #[allow(clippy::arc_with_non_send_sync)]
    fn from_git2(repo: Git2Repository) -> Result<Self, GitError> {
        let path = repo.path().to_path_buf();
        let workdir = repo.workdir().map(|p| p.to_path_buf());
        let state = convert_state(repo.state());

        info!("Repository opened: {:?}, state: {:?}", path, state);

        Ok(Self {
            path,
            workdir,
            state,
            inner: Arc::new(RwLock::new(repo)),
        })
    }

    /// Get the current branch name
    pub fn current_branch(&self) -> Result<Option<String>, GitError> {
        let repo = self.inner.read().unwrap();
        let head = repo.head().map_err(|e| GitError::OperationFailed {
            operation: "get_head".to_string(),
            details: e.to_string(),
        })?;

        if head.is_branch() {
            let name = head.shorthand().map(|s| s.to_string());
            Ok(name)
        } else {
            Ok(None) // Detached HEAD
        }
    }

    /// Get a compact label for the current branch or detached HEAD.
    pub fn current_branch_display(&self) -> String {
        self.current_branch()
            .ok()
            .flatten()
            .unwrap_or_else(|| "detached HEAD".to_string())
    }

    /// Get a short sync hint suitable for compact UI display.
    pub fn sync_status_hint(&self) -> Option<String> {
        self.sync_status().hint_text()
    }

    /// Resolve the remote name of the current branch upstream, if configured.
    pub fn current_upstream_remote(&self) -> Option<String> {
        self.current_upstream_ref()
            .as_deref()
            .and_then(parse_remote_name_from_ref)
            .map(str::to_string)
    }

    /// Resolve the full upstream ref of the current branch, if configured.
    pub fn current_upstream_ref(&self) -> Option<String> {
        let branch_name = self.current_branch().ok().flatten()?;
        let upstream_ref_spec = format!("{branch_name}@{{upstream}}");
        let repo_path = self.command_cwd();
        let output = std::process::Command::new("git")
            .args([
                "rev-parse",
                "--abbrev-ref",
                "--symbolic-full-name",
                &upstream_ref_spec,
            ])
            .current_dir(&repo_path)
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let upstream = String::from_utf8_lossy(&output.stdout);
        let upstream = upstream.trim();
        (!upstream.is_empty()).then(|| upstream.to_string())
    }

    /// Get a compact repository state hint for workspace chrome.
    pub fn state_hint(&self) -> Option<String> {
        match self.get_state() {
            RepositoryState::Clean => None,
            RepositoryState::Dirty => Some("有未提交修改".to_string()),
            RepositoryState::Merging => Some("合并中".to_string()),
            RepositoryState::Rebasing => Some("变基中".to_string()),
            RepositoryState::ApplyMailbox => Some("应用补丁中".to_string()),
            RepositoryState::Bisect => Some("二分定位中".to_string()),
            RepositoryState::CherryPick => Some("Cherry-pick 中".to_string()),
            RepositoryState::Revert => Some("回退中".to_string()),
        }
    }

    /// Get sync status with upstream branch
    pub fn sync_status(&self) -> SyncStatus {
        // Get current branch name
        let branch_name = match self.current_branch() {
            Ok(Some(name)) => name,
            _ => return SyncStatus::NoUpstream,
        };

        let upstream_ref = format!("{}@{{upstream}}", branch_name);
        let revspec = format!("{branch_name}...{upstream_ref}");
        let repo_path = self.command_cwd();
        let output = std::process::Command::new("git")
            .args(["rev-list", "--left-right", "--count", &revspec])
            .current_dir(&repo_path)
            .output();

        match output {
            Ok(output) if output.status.success() => {
                let output_str = String::from_utf8_lossy(&output.stdout);
                let parts: Vec<&str> = output_str.split_whitespace().collect();
                if parts.len() >= 2 {
                    let ahead: usize = parts[0].parse().unwrap_or(0);
                    let behind: usize = parts[1].parse().unwrap_or(0);

                    if ahead == 0 && behind == 0 {
                        SyncStatus::Synced
                    } else if ahead > 0 && behind == 0 {
                        SyncStatus::Ahead(ahead)
                    } else if ahead == 0 && behind > 0 {
                        SyncStatus::Behind(behind)
                    } else {
                        SyncStatus::Diverged { ahead, behind }
                    }
                } else {
                    SyncStatus::NoUpstream
                }
            }
            _ => SyncStatus::NoUpstream,
        }
    }

    /// Get repository state
    pub fn get_state(&self) -> RepositoryState {
        let repo = self.inner.read().unwrap();
        convert_state(repo.state())
    }

    /// Refresh repository state from disk
    pub fn refresh(&mut self) -> Result<(), GitError> {
        let new_repo =
            Git2Repository::open(&self.path).map_err(|_| GitError::RepositoryNotFound {
                path: self.path.display().to_string(),
            })?;

        self.state = convert_state(new_repo.state());
        *self.inner.write().unwrap() = new_repo;

        info!("Repository refreshed, state: {:?}", self.state);
        Ok(())
    }
}

/// Convert git2 repository state to our state enum
fn convert_state(state: git2::RepositoryState) -> RepositoryState {
    use crate::repository::RepositoryState as OurState;
    match state {
        git2::RepositoryState::Clean => OurState::Clean,
        git2::RepositoryState::Merge => OurState::Merging,
        git2::RepositoryState::Rebase => OurState::Rebasing,
        git2::RepositoryState::ApplyMailbox => OurState::ApplyMailbox,
        git2::RepositoryState::Bisect => OurState::Bisect,
        git2::RepositoryState::CherryPick => OurState::CherryPick,
        git2::RepositoryState::Revert => OurState::Revert,
        _ => OurState::Clean,
    }
}

fn parse_remote_name_from_ref(reference: &str) -> Option<&str> {
    reference
        .split_once('/')
        .map(|(remote, _)| remote)
        .filter(|remote| !remote.is_empty())
}

impl std::fmt::Debug for Repository {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Repository")
            .field("path", &self.path)
            .field("workdir", &self.workdir)
            .field("state", &self.state)
            .finish()
    }
}

pub(crate) fn compact_branch_sync_hint(
    upstream: Option<&str>,
    tracking_status: Option<&str>,
) -> Option<String> {
    match (upstream, tracking_status) {
        (Some(upstream), Some(status)) if !status.is_empty() => {
            Some(format!("{status} {upstream}"))
        }
        (Some(upstream), None) => Some(upstream.to_string()),
        (None, Some(status)) if !status.is_empty() => Some(status.to_string()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_remote_name_from_ref_extracts_remote_segment() {
        assert_eq!(parse_remote_name_from_ref("origin/main"), Some("origin"));
        assert_eq!(
            parse_remote_name_from_ref("upstream/feature/demo"),
            Some("upstream")
        );
        assert_eq!(parse_remote_name_from_ref("main"), None);
        assert_eq!(parse_remote_name_from_ref(""), None);
    }

    // ── SyncStatus tests ────────────────────────────────────────────────────────

    #[test]
    fn sync_status_display_text_formats_correctly() {
        assert_eq!(SyncStatus::Ahead(3).display_text(), "↑3");
        assert_eq!(SyncStatus::Behind(5).display_text(), "↓5");
        assert_eq!(
            SyncStatus::Diverged {
                ahead: 2,
                behind: 4
            }
            .display_text(),
            "↕2/4"
        );
        assert_eq!(SyncStatus::Synced.display_text(), "✓");
        assert_eq!(SyncStatus::NoUpstream.display_text(), "○");
        assert_eq!(SyncStatus::Unknown.display_text(), "?");
    }

    #[test]
    fn sync_status_display_color_returns_rgb_values() {
        // Just verify all variants return valid color arrays
        let _ = SyncStatus::Ahead(1).display_color();
        let _ = SyncStatus::Behind(1).display_color();
        let _ = SyncStatus::Diverged {
            ahead: 1,
            behind: 1,
        }
        .display_color();
        let _ = SyncStatus::Synced.display_color();
        let _ = SyncStatus::NoUpstream.display_color();
        let _ = SyncStatus::Unknown.display_color();
    }

    #[test]
    fn sync_status_hint_text_formats_correctly() {
        assert_eq!(
            SyncStatus::Ahead(3).hint_text(),
            Some("领先上游 3 个提交".to_string())
        );
        assert_eq!(
            SyncStatus::Diverged {
                ahead: 2,
                behind: 4
            }
            .hint_text(),
            Some("与上游分叉：领先 2 / 落后 4".to_string())
        );
        assert_eq!(SyncStatus::NoUpstream.hint_text(), None);
    }

    // ── compact_branch_sync_hint tests ─────────────────────────────────────────

    #[test]
    fn compact_branch_sync_hint_with_upstream_and_status() {
        assert_eq!(
            compact_branch_sync_hint(Some("origin/main"), Some("↑2")),
            Some("↑2 origin/main".to_string())
        );
    }

    #[test]
    fn compact_branch_sync_hint_with_upstream_only() {
        assert_eq!(
            compact_branch_sync_hint(Some("origin/main"), None),
            Some("origin/main".to_string())
        );
    }

    #[test]
    fn compact_branch_sync_hint_with_status_only() {
        assert_eq!(
            compact_branch_sync_hint(None, Some("↓5")),
            Some("↓5".to_string())
        );
    }

    #[test]
    fn compact_branch_sync_hint_returns_none_when_empty() {
        assert_eq!(compact_branch_sync_hint(None, None), None);
        assert_eq!(compact_branch_sync_hint(Some(""), Some("")), None);
        assert_eq!(compact_branch_sync_hint(None, Some("")), None);
    }

    // ── compact_relative_time tests ────────────────────────────────────────────

    #[test]
    fn compact_relative_time_returns_none_for_none() {
        assert_eq!(compact_relative_time(None), None);
    }

    #[test]
    fn compact_relative_time_formats_recent_commits() {
        let now = Local::now().timestamp();
        let five_minutes_ago = now - 5 * 60;
        let result = compact_relative_time(Some(five_minutes_ago));
        assert!(result.is_some());
        assert!(result.unwrap().contains("分钟前"));
    }

    #[test]
    fn compact_relative_time_formats_hours() {
        let now = Local::now().timestamp();
        let three_hours_ago = now - 3 * 3600;
        let result = compact_relative_time(Some(three_hours_ago));
        assert!(result.is_some());
        assert!(result.unwrap().contains("小时前"));
    }

    #[test]
    fn compact_relative_time_formats_days() {
        let now = Local::now().timestamp();
        let two_days_ago = now - 2 * 24 * 3600;
        let result = compact_relative_time(Some(two_days_ago));
        assert!(result.is_some());
        assert!(result.unwrap().contains("天前"));
    }

    #[test]
    fn compact_relative_time_formats_old_commits_as_date() {
        let now = Local::now().timestamp();
        let ten_days_ago = now - 10 * 24 * 3600;
        let result = compact_relative_time(Some(ten_days_ago));
        assert!(result.is_some());
        // Should be in MM-DD format
        let formatted = result.unwrap();
        assert!(formatted.contains('-'));
    }

    // ── RepositoryManager tests ────────────────────────────────────────────────

    #[test]
    fn repository_manager_new_is_empty() {
        let manager = RepositoryManager::new();
        assert!(manager.is_empty());
        assert_eq!(manager.len(), 0);
    }

    #[test]
    fn repository_manager_default_is_empty() {
        let manager: RepositoryManager = Default::default();
        assert!(manager.is_empty());
    }

    #[test]
    fn repository_manager_init_adds_repository() {
        let temp_dir = tempfile::tempdir().unwrap();
        let mut manager = RepositoryManager::new();

        let repo = manager.init(temp_dir.path()).unwrap();
        let repo_path = repo.path.clone();
        assert_eq!(manager.len(), 1);
        assert!(!manager.is_empty());

        // Verify we can get it back
        let retrieved = manager.get(&repo_path);
        assert!(retrieved.is_some());
    }

    #[test]
    fn repository_manager_open_discovers_repository() {
        let temp_dir = tempfile::tempdir().unwrap();
        // First init a repo
        let _ = Repository::init(temp_dir.path()).unwrap();

        let mut manager = RepositoryManager::new();
        let repo = manager.open(temp_dir.path()).unwrap();
        let repo_path = repo.path.clone();

        assert_eq!(manager.len(), 1);
        let retrieved = manager.get(&repo_path);
        assert!(retrieved.is_some());
    }

    #[test]
    fn repository_manager_remove_deletes_repository() {
        let temp_dir = tempfile::tempdir().unwrap();
        let mut manager = RepositoryManager::new();
        let repo = manager.init(temp_dir.path()).unwrap();
        let path = repo.path.clone();

        assert!(manager.remove(&path));
        assert!(manager.is_empty());
        assert!(!manager.remove(&path)); // Second remove should return false
    }

    #[test]
    fn repository_manager_list_returns_all_repositories() {
        let temp_dir1 = tempfile::tempdir().unwrap();
        let temp_dir2 = tempfile::tempdir().unwrap();

        let mut manager = RepositoryManager::new();
        manager.init(temp_dir1.path()).unwrap();
        manager.init(temp_dir2.path()).unwrap();

        let list = manager.list();
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn repository_manager_get_returns_none_for_unknown_path() {
        let manager = RepositoryManager::new();
        let unknown_path = PathBuf::from("/nonexistent/path");
        assert!(manager.get(&unknown_path).is_none());
    }

    // ── RepositoryState tests ──────────────────────────────────────────────────

    #[test]
    fn repository_state_debug_and_clone() {
        let state = RepositoryState::Clean;
        let cloned = state.clone();
        assert_eq!(state, cloned);
        assert_eq!(format!("{:?}", state), "Clean");
    }

    #[test]
    fn repository_state_all_variants() {
        // Just verify all variants can be created and compared
        let states = [
            RepositoryState::Clean,
            RepositoryState::Dirty,
            RepositoryState::Merging,
            RepositoryState::Rebasing,
            RepositoryState::ApplyMailbox,
            RepositoryState::Bisect,
            RepositoryState::CherryPick,
            RepositoryState::Revert,
        ];
        assert_eq!(states.len(), 8);
    }
}

pub(crate) fn compact_relative_time(timestamp: Option<i64>) -> Option<String> {
    let timestamp = timestamp?;
    let commit_time = match Local.timestamp_opt(timestamp, 0) {
        LocalResult::Single(value) => value,
        _ => return None,
    };
    let delta = Local::now().signed_duration_since(commit_time);

    if delta.num_minutes() < 60 {
        Some(format!("{} 分钟前", delta.num_minutes().max(1)))
    } else if delta.num_hours() < 24 {
        Some(format!("{} 小时前", delta.num_hours()))
    } else if delta.num_days() < 7 {
        Some(format!("{} 天前", delta.num_days()))
    } else {
        Some(commit_time.format("%m-%d").to_string())
    }
}

/// Manager for tracking multiple repository instances
#[derive(Default)]
pub struct RepositoryManager {
    repositories: std::collections::HashMap<PathBuf, Repository>,
}

impl RepositoryManager {
    /// Create a new repository manager
    pub fn new() -> Self {
        Self {
            repositories: std::collections::HashMap::new(),
        }
    }

    /// Open or discover a repository and add it to the manager
    pub fn open(&mut self, path: &Path) -> Result<&Repository, GitError> {
        let repo = Repository::discover(path)?;
        let path = repo.path.clone();
        self.repositories.insert(path.clone(), repo);
        Ok(self.repositories.get(&path).unwrap())
    }

    /// Initialize a new repository and add it to the manager
    pub fn init(&mut self, path: &Path) -> Result<&Repository, GitError> {
        let repo = Repository::init(path)?;
        let path = repo.path.clone();
        self.repositories.insert(path.clone(), repo);
        Ok(self.repositories.get(&path).unwrap())
    }

    /// Get a repository by path
    pub fn get(&self, path: &Path) -> Option<&Repository> {
        self.repositories.get(path)
    }

    /// Remove a repository from the manager
    pub fn remove(&mut self, path: &Path) -> bool {
        self.repositories.remove(path).is_some()
    }

    /// List all tracked repositories
    pub fn list(&self) -> Vec<&Repository> {
        self.repositories.values().collect()
    }

    /// Get the number of tracked repositories
    pub fn len(&self) -> usize {
        self.repositories.len()
    }

    /// Check if manager has any repositories
    pub fn is_empty(&self) -> bool {
        self.repositories.is_empty()
    }
}
