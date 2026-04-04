//! Shared internal helpers used by both commit_actions and rebase modules.
//!
//! These functions are not part of the public API.

use crate::error::GitError;
use crate::repository::Repository;
use git2::Oid;

/// Resolve the current HEAD commit OID.
pub(crate) fn current_head_oid(repo: &Repository, operation: &str) -> Result<Oid, GitError> {
    let repo_lock = repo.inner.read().unwrap();
    let head = repo_lock.head().map_err(|e| GitError::OperationFailed {
        operation: operation.to_string(),
        details: e.to_string(),
    })?;
    let commit = head
        .peel_to_commit()
        .map_err(|e| GitError::OperationFailed {
            operation: operation.to_string(),
            details: e.to_string(),
        })?;
    Ok(commit.id())
}

/// Resolve a revision spec (e.g. "HEAD~3", a SHA, a ref) to a commit OID.
pub(crate) fn resolve_commit_oid(
    repo: &Repository,
    spec: &str,
    _operation: &str,
) -> Result<Oid, GitError> {
    let repo_lock = repo.inner.read().unwrap();
    let object = repo_lock
        .revparse_single(spec)
        .map_err(|_| GitError::CommitNotFound {
            id: spec.to_string(),
        })?;
    let commit = object
        .peel_to_commit()
        .map_err(|_| GitError::CommitNotFound {
            id: spec.to_string(),
        })?;
    Ok(commit.id())
}

/// Check whether `ancestor` is an ancestor of `descendant`.
pub(crate) fn is_ancestor(
    repo: &Repository,
    ancestor: Oid,
    descendant: Oid,
) -> Result<bool, GitError> {
    if ancestor == descendant {
        return Ok(true);
    }

    let repo_lock = repo.inner.read().unwrap();
    repo_lock
        .graph_descendant_of(descendant, ancestor)
        .map_err(|e| GitError::OperationFailed {
            operation: "graph_descendant_of".to_string(),
            details: e.to_string(),
        })
}

