//! History operations for git-core

use crate::error::GitError;
use crate::graph::RefLabel;
use crate::repository::Repository;
use crate::signature::SignatureStatus;
use git2::Sort;
use log::info;

/// Commit history entry
#[derive(Debug, Clone)]
pub struct HistoryEntry {
    pub id: String,
    pub message: String,
    pub author_name: String,
    pub author_email: String,
    pub timestamp: i64,
    pub parent_ids: Vec<String>,
    /// Committer name (if different from author)
    pub committer_name: Option<String>,
    /// Committer email
    pub committer_email: Option<String>,
    /// Branch/tag labels pointing to this commit
    pub refs: Vec<RefLabel>,
    /// GPG/SSH verification result
    pub signature_status: Option<SignatureStatus>,
}

/// Helper to build a HistoryEntry from a git2 Commit
fn entry_from_commit(commit: &git2::Commit, oid: git2::Oid) -> HistoryEntry {
    let author_name = commit.author().name().unwrap_or("").to_string();
    let author_email = commit.author().email().unwrap_or("").to_string();
    let committer_name_str = commit.committer().name().unwrap_or("").to_string();
    let committer_email_str = commit.committer().email().unwrap_or("").to_string();

    let committer_name = if committer_name_str != author_name {
        Some(committer_name_str)
    } else {
        None
    };
    let committer_email = if committer_email_str != author_email {
        Some(committer_email_str)
    } else {
        None
    };

    HistoryEntry {
        id: oid.to_string(),
        message: commit.message().unwrap_or("").to_string(),
        author_name,
        author_email,
        timestamp: commit.time().seconds(),
        parent_ids: commit.parents().map(|p| p.id().to_string()).collect(),
        committer_name,
        committer_email,
        refs: Vec::new(),
        signature_status: None,
    }
}

/// Get commit history
pub fn get_history(
    repo: &Repository,
    max_count: Option<usize>,
) -> Result<Vec<HistoryEntry>, GitError> {
    info!("Getting commit history, max: {:?}", max_count);

    let repo_lock = repo.inner.read().unwrap();

    let mut revwalk = repo_lock.revwalk().map_err(|e| GitError::OperationFailed {
        operation: "get_history".to_string(),
        details: e.to_string(),
    })?;

    revwalk.push_head().map_err(|e| GitError::OperationFailed {
        operation: "get_history".to_string(),
        details: e.to_string(),
    })?;

    let _ = revwalk.set_sorting(Sort::TIME | Sort::TOPOLOGICAL);

    let mut history = Vec::new();
    let limit = max_count.unwrap_or(100);

    for (i, oid_result) in revwalk.enumerate() {
        if i >= limit {
            break;
        }

        if let Ok(oid) = oid_result {
            if let Ok(commit) = repo_lock.find_commit(oid) {
                history.push(entry_from_commit(&commit, oid));
            }
        }
    }

    info!("Retrieved {} history entries", history.len());
    Ok(history)
}

/// Get commit history starting from a specific ref, branch, tag, or commit id.
pub fn get_history_for_ref(
    repo: &Repository,
    reference: &str,
    max_count: Option<usize>,
) -> Result<Vec<HistoryEntry>, GitError> {
    info!(
        "Getting commit history for reference '{}', max: {:?}",
        reference, max_count
    );

    let repo_lock = repo.inner.read().unwrap();

    let object = repo_lock
        .revparse_single(reference)
        .map_err(|e| GitError::OperationFailed {
            operation: "get_history_for_ref".to_string(),
            details: e.to_string(),
        })?;
    let commit = object
        .peel_to_commit()
        .map_err(|e| GitError::OperationFailed {
            operation: "get_history_for_ref".to_string(),
            details: e.to_string(),
        })?;

    let mut revwalk = repo_lock.revwalk().map_err(|e| GitError::OperationFailed {
        operation: "get_history_for_ref".to_string(),
        details: e.to_string(),
    })?;

    revwalk
        .push(commit.id())
        .map_err(|e| GitError::OperationFailed {
            operation: "get_history_for_ref".to_string(),
            details: e.to_string(),
        })?;

    let _ = revwalk.set_sorting(Sort::TIME | Sort::TOPOLOGICAL);

    let mut history = Vec::new();
    let limit = max_count.unwrap_or(100);

    for (i, oid_result) in revwalk.enumerate() {
        if i >= limit {
            break;
        }

        if let Ok(oid) = oid_result {
            if let Ok(commit) = repo_lock.find_commit(oid) {
                history.push(entry_from_commit(&commit, oid));
            }
        }
    }

    info!(
        "Retrieved {} history entries for reference '{}'",
        history.len(),
        reference
    );
    Ok(history)
}

/// Search commits by message
pub fn search_history(
    repo: &Repository,
    pattern: &str,
    max_count: Option<usize>,
) -> Result<Vec<HistoryEntry>, GitError> {
    info!("Searching history for '{}'", pattern);

    let repo_lock = repo.inner.read().unwrap();

    let mut revwalk = repo_lock.revwalk().map_err(|e| GitError::OperationFailed {
        operation: "search_history".to_string(),
        details: e.to_string(),
    })?;

    revwalk.push_head().map_err(|e| GitError::OperationFailed {
        operation: "search_history".to_string(),
        details: e.to_string(),
    })?;

    let _ = revwalk.set_sorting(Sort::TIME | Sort::TOPOLOGICAL);

    let mut history = Vec::new();
    let limit = max_count.unwrap_or(100);
    let pattern_lower = pattern.to_lowercase();

    for (i, oid_result) in revwalk.enumerate() {
        if i >= limit {
            break;
        }

        if let Ok(oid) = oid_result {
            if let Ok(commit) = repo_lock.find_commit(oid) {
                let message = commit.message().unwrap_or("").to_lowercase();
                if message.contains(&pattern_lower) {
                    history.push(entry_from_commit(&commit, oid));
                }
            }
        }
    }

    info!("Found {} matching history entries", history.len());
    Ok(history)
}

/// Get commit history filtered by author name
pub fn get_history_for_author(
    repo: &Repository,
    author: &str,
    max_count: Option<usize>,
) -> Result<Vec<HistoryEntry>, GitError> {
    info!("Getting history for author '{}'", author);

    let repo_lock = repo.inner.read().unwrap();
    let mut revwalk = repo_lock.revwalk().map_err(|e| GitError::OperationFailed {
        operation: "get_history_for_author".to_string(),
        details: e.to_string(),
    })?;
    revwalk.push_head().map_err(|e| GitError::OperationFailed {
        operation: "get_history_for_author".to_string(),
        details: e.to_string(),
    })?;
    let _ = revwalk.set_sorting(Sort::TIME | Sort::TOPOLOGICAL);

    let mut history = Vec::new();
    let limit = max_count.unwrap_or(500);
    let author_lower = author.to_lowercase();

    for oid_result in revwalk {
        if history.len() >= limit {
            break;
        }
        if let Ok(oid) = oid_result {
            if let Ok(commit) = repo_lock.find_commit(oid) {
                let name = commit.author().name().unwrap_or("").to_lowercase();
                let email = commit.author().email().unwrap_or("").to_lowercase();
                if name.contains(&author_lower) || email.contains(&author_lower) {
                    history.push(entry_from_commit(&commit, oid));
                }
            }
        }
    }

    info!("Found {} entries for author '{}'", history.len(), author);
    Ok(history)
}

/// Get commit history filtered by file path
pub fn get_history_for_path(
    repo: &Repository,
    file_path: &str,
    max_count: Option<usize>,
) -> Result<Vec<HistoryEntry>, GitError> {
    info!("Getting history for path '{}'", file_path);

    let repo_lock = repo.inner.read().unwrap();
    let mut revwalk = repo_lock.revwalk().map_err(|e| GitError::OperationFailed {
        operation: "get_history_for_path".to_string(),
        details: e.to_string(),
    })?;
    revwalk.push_head().map_err(|e| GitError::OperationFailed {
        operation: "get_history_for_path".to_string(),
        details: e.to_string(),
    })?;
    let _ = revwalk.set_sorting(Sort::TIME | Sort::TOPOLOGICAL);

    let mut history = Vec::new();
    let limit = max_count.unwrap_or(500);
    let target_path = std::path::Path::new(file_path);

    for oid_result in revwalk {
        if history.len() >= limit {
            break;
        }
        if let Ok(oid) = oid_result {
            if let Ok(commit) = repo_lock.find_commit(oid) {
                // Check if this commit touches the target path
                let touches_path = if commit.parent_count() == 0 {
                    // Initial commit — check if file exists in tree
                    commit
                        .tree()
                        .ok()
                        .and_then(|t| t.get_path(target_path).ok())
                        .is_some()
                } else if let Ok(parent) = commit.parent(0) {
                    // Compare parent tree to this commit's tree
                    let old_tree = parent.tree().ok();
                    let new_tree = commit.tree().ok();
                    if let (Some(old), Some(new)) = (old_tree, new_tree) {
                        let diff = repo_lock
                            .diff_tree_to_tree(Some(&old), Some(&new), None)
                            .ok();
                        diff.map(|d| {
                            d.deltas().any(|delta| {
                                delta
                                    .new_file()
                                    .path()
                                    .map(|p| p == target_path)
                                    .unwrap_or(false)
                                    || delta
                                        .old_file()
                                        .path()
                                        .map(|p| p == target_path)
                                        .unwrap_or(false)
                            })
                        })
                        .unwrap_or(false)
                    } else {
                        false
                    }
                } else {
                    false
                };

                if touches_path {
                    history.push(entry_from_commit(&commit, oid));
                }
            }
        }
    }

    info!("Found {} entries for path '{}'", history.len(), file_path);
    Ok(history)
}

/// Get commit history filtered by date range (Unix timestamps)
pub fn get_history_for_date_range(
    repo: &Repository,
    start_time: i64,
    end_time: i64,
    max_count: Option<usize>,
) -> Result<Vec<HistoryEntry>, GitError> {
    info!(
        "Getting history for date range {} - {}",
        start_time, end_time
    );

    let repo_lock = repo.inner.read().unwrap();
    let mut revwalk = repo_lock.revwalk().map_err(|e| GitError::OperationFailed {
        operation: "get_history_for_date_range".to_string(),
        details: e.to_string(),
    })?;
    revwalk.push_head().map_err(|e| GitError::OperationFailed {
        operation: "get_history_for_date_range".to_string(),
        details: e.to_string(),
    })?;
    let _ = revwalk.set_sorting(Sort::TIME | Sort::TOPOLOGICAL);

    let mut history = Vec::new();
    let limit = max_count.unwrap_or(500);

    for oid_result in revwalk {
        if history.len() >= limit {
            break;
        }
        if let Ok(oid) = oid_result {
            if let Ok(commit) = repo_lock.find_commit(oid) {
                let ts = commit.time().seconds();
                // Stop early if we've passed the start of the range
                if ts < start_time {
                    break;
                }
                if ts <= end_time {
                    history.push(entry_from_commit(&commit, oid));
                }
            }
        }
    }

    info!(
        "Found {} entries in date range {} - {}",
        history.len(),
        start_time,
        end_time
    );
    Ok(history)
}
