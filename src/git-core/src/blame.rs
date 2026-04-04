//! Git blame/annotate operations for git-core

use crate::error::GitError;
use crate::repository::Repository;
use log::info;
use std::path::Path;

/// Per-hunk blame attribution data
#[derive(Debug, Clone)]
pub struct BlameEntry {
    /// Commit that last modified these lines
    pub commit_id: String,
    /// Author of the commit
    pub author_name: String,
    /// Author email
    pub author_email: String,
    /// Commit timestamp (Unix epoch)
    pub timestamp: i64,
    /// First line of commit message
    pub message: String,
    /// First line number (1-based)
    pub start_line: u32,
    /// Number of lines in this hunk
    pub line_count: u32,
}

/// Get blame/annotate information for a file
pub fn blame_file(repo: &Repository, file_path: &Path) -> Result<Vec<BlameEntry>, GitError> {
    info!("Computing blame for file: {:?}", file_path);

    let repo_lock = repo.inner.read().unwrap();
    let mut opts = git2::BlameOptions::new();

    let blame = repo_lock
        .blame_file(file_path, Some(&mut opts))
        .map_err(|e| GitError::OperationFailed {
            operation: "blame_file".to_string(),
            details: format!("Failed to compute blame for {:?}: {}", file_path, e),
        })?;

    let mut entries = Vec::new();

    for i in 0..blame.len() {
        let hunk = blame.get_index(i).ok_or_else(|| GitError::OperationFailed {
            operation: "blame_file".to_string(),
            details: format!("Failed to get blame hunk at index {}", i),
        })?;

        let commit_id = hunk.final_commit_id();
        let sig = hunk.final_signature();

        let author_name = sig.name().unwrap_or("Unknown").to_string();
        let author_email = sig.email().unwrap_or("").to_string();
        let timestamp = sig.when().seconds();

        // Try to get commit message
        let message = repo_lock
            .find_commit(commit_id)
            .ok()
            .and_then(|c| c.summary().map(|s| s.to_string()))
            .unwrap_or_default();

        entries.push(BlameEntry {
            commit_id: commit_id.to_string(),
            author_name,
            author_email,
            timestamp,
            message,
            start_line: hunk.final_start_line() as u32,
            line_count: hunk.lines_in_hunk() as u32,
        });
    }

    info!(
        "Blame computed: {} hunks for {:?}",
        entries.len(),
        file_path
    );
    Ok(entries)
}
