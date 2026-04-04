//! Commit operations for git-core

use crate::error::GitError;
use crate::repository::Repository;
use log::info;

/// A Git commit
#[derive(Debug, Clone)]
pub struct CommitInfo {
    pub id: String,
    pub message: String,
    pub author_name: String,
    pub author_email: String,
    pub author_time: i64,
    pub committer_name: String,
    pub committer_email: String,
    pub committer_time: i64,
    pub parent_ids: Vec<String>,
}

/// Create a new commit
pub fn create_commit(
    repo: &Repository,
    message: &str,
    _author_name: &str,
    _author_email: &str,
) -> Result<String, GitError> {
    info!("Creating commit: {}", message);

    let repo_lock = repo.inner.read().unwrap();

    // Get the index
    let mut index = repo_lock.index().map_err(|e| GitError::OperationFailed {
        operation: "create_commit".to_string(),
        details: e.to_string(),
    })?;

    // Write the tree
    let tree_oid = index.write_tree().map_err(|e| GitError::OperationFailed {
        operation: "create_commit".to_string(),
        details: e.to_string(),
    })?;

    let tree = repo_lock
        .find_tree(tree_oid)
        .map_err(|e| GitError::OperationFailed {
            operation: "create_commit".to_string(),
            details: e.to_string(),
        })?;

    // May be None if this is the first commit on an unborn branch.
    let parent_commit: Option<git2::Commit> = repo_lock
        .head()
        .ok()
        .and_then(|head| head.peel_to_commit().ok());

    // Create signature
    let signature = repo_lock
        .signature()
        .map_err(|e| GitError::OperationFailed {
            operation: "create_commit".to_string(),
            details: e.to_string(),
        })?;

    // Create the commit - git2 expects &[] for empty parents
    let commit_oid = if let Some(ref parent) = parent_commit {
        repo_lock.commit(
            Some("HEAD"),
            &signature,
            &signature,
            message,
            &tree,
            &[parent],
        )
    } else {
        repo_lock.commit(Some("HEAD"), &signature, &signature, message, &tree, &[])
    }
    .map_err(|e| GitError::OperationFailed {
        operation: "create_commit".to_string(),
        details: e.to_string(),
    })?;

    info!("Commit created: {}", commit_oid);

    Ok(commit_oid.to_string())
}

/// Amend a commit with a new message
pub fn amend_commit(repo: &Repository, commit_id: &str, message: &str) -> Result<String, GitError> {
    info!("Amending commit: {}", commit_id);

    let repo_lock = repo.inner.write().unwrap();

    let oid = git2::Oid::from_str(commit_id).map_err(|_| GitError::CommitNotFound {
        id: commit_id.to_string(),
    })?;

    let commit = repo_lock
        .find_commit(oid)
        .map_err(|_| GitError::CommitNotFound {
            id: commit_id.to_string(),
        })?;

    // Get the tree from the commit
    let tree = commit.tree().map_err(|e| GitError::OperationFailed {
        operation: "amend_commit".to_string(),
        details: e.to_string(),
    })?;

    // Create signature
    let signature = repo_lock
        .signature()
        .map_err(|e| GitError::OperationFailed {
            operation: "amend_commit".to_string(),
            details: e.to_string(),
        })?;

    // Amend the commit using the commit object
    let amend_oid = commit
        .amend(
            Some("HEAD"),
            Some(&signature),
            Some(&signature),
            None,
            Some(message),
            Some(&tree),
        )
        .map_err(|e| GitError::OperationFailed {
            operation: "amend_commit".to_string(),
            details: e.to_string(),
        })?;

    info!("Commit amended: {}", amend_oid);
    Ok(amend_oid.to_string())
}

/// Create a signature for commits
pub fn create_signature(
    _repo: &Repository,
    name: &str,
    email: &str,
) -> Result<git2::Signature<'static>, GitError> {
    git2::Signature::now(name, email).map_err(|e| GitError::OperationFailed {
        operation: "create_signature".to_string(),
        details: e.to_string(),
    })
}

/// Get the default signature (user's git config)
pub fn get_default_signature(repo: &Repository) -> Result<git2::Signature<'static>, GitError> {
    let repo_lock = repo.inner.read().unwrap();

    repo_lock
        .signature()
        .map_err(|e| GitError::OperationFailed {
            operation: "get_default_signature".to_string(),
            details: e.to_string(),
        })
}

/// Validate a commit reference (hash, branch name, tag, etc.)
/// Returns the resolved full hash and first line of commit message if valid.
pub fn validate_commit_ref(repo: &Repository, reference: &str) -> Result<(String, String), GitError> {
    let repo_lock = repo.inner.read().unwrap();
    let object = repo_lock
        .revparse_single(reference)
        .map_err(|_| GitError::CommitNotFound {
            id: reference.to_string(),
        })?;
    let commit = object
        .peel_to_commit()
        .map_err(|_| GitError::CommitNotFound {
            id: reference.to_string(),
        })?;
    let hash = commit.id().to_string();
    let summary = commit.summary().unwrap_or("").to_string();
    Ok((hash, summary))
}

/// Get commit information
pub fn get_commit(repo: &Repository, commit_id: &str) -> Result<CommitInfo, GitError> {
    let repo_lock = repo.inner.read().unwrap();

    let oid = git2::Oid::from_str(commit_id).map_err(|_| GitError::CommitNotFound {
        id: commit_id.to_string(),
    })?;

    let commit = repo_lock
        .find_commit(oid)
        .map_err(|_| GitError::CommitNotFound {
            id: commit_id.to_string(),
        })?;

    let author = commit.author();
    let committer = commit.committer();

    Ok(CommitInfo {
        id: commit_id.to_string(),
        message: commit.message().unwrap_or("").to_string(),
        author_name: author.name().unwrap_or("").to_string(),
        author_email: author.email().unwrap_or("").to_string(),
        author_time: commit.time().seconds(),
        committer_name: committer.name().unwrap_or("").to_string(),
        committer_email: committer.email().unwrap_or("").to_string(),
        committer_time: committer.when().seconds(),
        parent_ids: commit.parents().map(|p| p.id().to_string()).collect(),
    })
}

// --- Commit message history persistence ---

use std::collections::HashMap;
use std::path::{Path, PathBuf};

const MAX_RECENT_MESSAGES: usize = 10;

fn config_dir() -> Option<PathBuf> {
    dirs_next::config_dir().map(|d| d.join("slio-git"))
}

fn history_file_path() -> Option<PathBuf> {
    config_dir().map(|d| d.join("commit-messages.json"))
}

/// Load recent commit messages for a specific repository path.
/// Returns up to 10 messages, newest first.
pub fn load_recent_messages(repo_path: &Path) -> Vec<String> {
    let Some(file_path) = history_file_path() else {
        return Vec::new();
    };

    let Ok(content) = std::fs::read_to_string(&file_path) else {
        return Vec::new();
    };

    let key = repo_path.to_string_lossy().to_string();
    let map: HashMap<String, Vec<String>> =
        serde_json::from_str(&content).unwrap_or_default();

    map.get(&key).cloned().unwrap_or_default()
}

/// Save a commit message to the recent history for a repository.
/// Keeps the last MAX_RECENT_MESSAGES messages, newest first.
pub fn save_recent_message(repo_path: &Path, message: &str) {
    if message.trim().is_empty() {
        return;
    }

    let Some(file_path) = history_file_path() else {
        return;
    };

    // Ensure config directory exists
    if let Some(dir) = file_path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }

    let key = repo_path.to_string_lossy().to_string();

    // Load existing
    let mut map: HashMap<String, Vec<String>> = std::fs::read_to_string(&file_path)
        .ok()
        .and_then(|c| serde_json::from_str(&c).ok())
        .unwrap_or_default();

    let messages = map.entry(key).or_default();

    // Remove duplicate if exists
    messages.retain(|m| m != message);

    // Insert at front
    messages.insert(0, message.to_string());

    // Trim to max
    messages.truncate(MAX_RECENT_MESSAGES);

    // Save
    if let Ok(json) = serde_json::to_string_pretty(&map) {
        let _ = std::fs::write(&file_path, json);
    }

    info!("Saved commit message to history");
}
