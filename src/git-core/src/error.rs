//! Error types for git-core operations

use thiserror::Error;

#[derive(Error, Debug)]
pub enum GitError {
    #[error("Repository not found at path: {path}")]
    RepositoryNotFound { path: String },

    #[error("Invalid repository: {reason}")]
    InvalidRepository { reason: String },

    #[error("Git operation failed: {operation} - {details}")]
    OperationFailed { operation: String, details: String },

    #[error("Branch not found: {name}")]
    BranchNotFound { name: String },

    #[error("Commit not found: {id}")]
    CommitNotFound { id: String },

    #[error("Stash not found at index: {index}")]
    StashNotFound { index: u32 },

    #[error("Tag not found: {name}")]
    TagNotFound { name: String },

    #[error("Remote operation failed: {remote} - {details}")]
    RemoteFailed { remote: String, details: String },

    #[error("Merge conflict detected")]
    MergeConflict,

    #[error("Authentication failed for remote: {remote}")]
    AuthenticationFailed { remote: String },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Git library error: {0}")]
    Git2(#[from] git2::Error),
}

impl GitError {
    /// Log the error with context
    pub fn log_context(&self, operation: &str, path: &std::path::Path) {
        log::error!("git-core error in {} at {:?}: {}", operation, path, self);
    }

    /// Check if this error indicates a retryable condition
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            GitError::AuthenticationFailed { .. } | GitError::RemoteFailed { .. }
        )
    }
}
