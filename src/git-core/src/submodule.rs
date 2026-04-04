//! Submodule detection for git-core

use crate::error::GitError;
use crate::repository::Repository;
use log::info;

/// Information about a submodule change
#[derive(Debug, Clone)]
pub struct SubmoduleChange {
    /// Submodule path relative to repository root
    pub path: String,
    /// Submodule name
    pub name: String,
    /// Submodule URL
    pub url: Option<String>,
    /// Previous commit hash (HEAD)
    pub old_commit: Option<String>,
    /// New commit hash (index or workdir)
    pub new_commit: Option<String>,
    /// Human-readable summary of the change (e.g., "abc1234..def5678 (3 commits)")
    pub summary: Option<String>,
}

/// Detect if a path is a submodule
pub fn is_submodule(repo: &Repository, path: &str) -> bool {
    let repo_lock = repo.inner.read().unwrap();
    let result = repo_lock.find_submodule(path).is_ok();
    result
}

/// List all submodules and their current status
pub fn list_submodules(repo: &Repository) -> Result<Vec<SubmoduleChange>, GitError> {
    info!("Listing submodules");

    let repo_lock = repo.inner.read().unwrap();
    let submodules = repo_lock.submodules().map_err(|e| GitError::OperationFailed {
        operation: "list_submodules".to_string(),
        details: format!("Failed to enumerate submodules: {}", e),
    })?;

    let mut changes = Vec::new();

    for sm in &submodules {
        let name = sm.name().unwrap_or("").to_string();
        let path = sm.path().to_string_lossy().to_string();
        let url = sm.url().map(|u| u.to_string());

        let head_id = sm.head_id().map(|oid| oid.to_string());
        let index_id = sm.index_id().map(|oid| oid.to_string());
        let workdir_id = sm.workdir_id().map(|oid| oid.to_string());

        // Determine old and new commits
        let old_commit = head_id.clone();
        let new_commit = workdir_id.or(index_id);

        // Build summary
        let summary = match (&old_commit, &new_commit) {
            (Some(old), Some(new)) if old != new => {
                Some(format!("{}..{}", &old[..7.min(old.len())], &new[..7.min(new.len())]))
            }
            (None, Some(new)) => Some(format!("new: {}", &new[..7.min(new.len())])),
            _ => None,
        };

        changes.push(SubmoduleChange {
            path,
            name,
            url,
            old_commit,
            new_commit,
            summary,
        });
    }

    info!("Found {} submodules", changes.len());
    Ok(changes)
}

/// Get summary string for a submodule change (for display in changelist)
pub fn submodule_summary(repo: &Repository, path: &str) -> Option<String> {
    let repo_lock = repo.inner.read().unwrap();
    let sm = repo_lock.find_submodule(path).ok()?;

    let head_id = sm.head_id()?;
    let index_id = sm.index_id().or(sm.workdir_id())?;

    if head_id == index_id {
        return None;
    }

    Some(format!(
        "{}..{}",
        &head_id.to_string()[..7],
        &index_id.to_string()[..7]
    ))
}
