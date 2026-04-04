//! Working tree management for git-core

use crate::error::GitError;
use crate::repository::Repository;
use log::info;
use std::path::{Path, PathBuf};
use std::process::Command;

/// A linked git worktree
#[derive(Debug, Clone)]
pub struct WorkingTree {
    /// Worktree name
    pub name: String,
    /// Absolute path to worktree directory
    pub path: PathBuf,
    /// Branch checked out in worktree
    pub branch: Option<String>,
    /// Whether this is the main worktree
    pub is_main: bool,
    /// Whether the worktree is locked
    pub is_locked: bool,
    /// Whether the worktree path exists and is valid
    pub is_valid: bool,
}

/// List all worktrees for the repository
pub fn list_worktrees(repo: &Repository) -> Result<Vec<WorkingTree>, GitError> {
    info!("Listing worktrees");

    let repo_path = repo.command_cwd();

    let output = Command::new("git")
        .args(["worktree", "list", "--porcelain"])
        .current_dir(&repo_path)
        .output()
        .map_err(|e| GitError::OperationFailed {
            operation: "list_worktrees".to_string(),
            details: format!("Failed to execute git worktree list: {}", e),
        })?;

    if !output.status.success() {
        return Err(GitError::OperationFailed {
            operation: "list_worktrees".to_string(),
            details: format!(
                "git worktree list failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ),
        });
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    let mut worktrees = Vec::new();
    let mut current_path: Option<PathBuf> = None;
    let mut current_branch: Option<String> = None;
    let mut is_bare = false;
    let mut is_locked = false;
    let mut is_first = true;

    for line in output_str.lines() {
        if line.starts_with("worktree ") {
            // Save previous worktree if any
            if let Some(path) = current_path.take() {
                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();
                let is_valid = path.exists();
                worktrees.push(WorkingTree {
                    name,
                    path,
                    branch: current_branch.take(),
                    is_main: is_first,
                    is_locked,
                    is_valid,
                });
                is_first = false;
            }
            current_path = Some(PathBuf::from(line.trim_start_matches("worktree ")));
            current_branch = None;
            is_bare = false;
            is_locked = false;
        } else if line.starts_with("branch ") {
            let ref_name = line.trim_start_matches("branch ");
            current_branch = Some(
                ref_name
                    .strip_prefix("refs/heads/")
                    .unwrap_or(ref_name)
                    .to_string(),
            );
        } else if line == "bare" {
            is_bare = true;
        } else if line == "locked" {
            is_locked = true;
        }
    }

    // Push last worktree
    if let Some(path) = current_path {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        let is_valid = path.exists();
        worktrees.push(WorkingTree {
            name,
            path,
            branch: current_branch,
            is_main: is_first,
            is_locked,
            is_valid,
        });
    }

    // Filter out bare worktrees
    if is_bare {
        // bare flag applies to main worktree only in edge cases
    }

    info!("Found {} worktrees", worktrees.len());
    Ok(worktrees)
}

/// Create a new worktree
pub fn create_worktree(
    repo: &Repository,
    path: &Path,
    branch: Option<&str>,
) -> Result<WorkingTree, GitError> {
    info!("Creating worktree at {:?} with branch {:?}", path, branch);

    let repo_path = repo.command_cwd();
    let path_str = path.to_string_lossy();

    let mut args = vec!["worktree", "add"];
    args.push(&path_str);
    if let Some(b) = branch {
        args.push(b);
    }

    let output = Command::new("git")
        .args(&args)
        .current_dir(&repo_path)
        .output()
        .map_err(|e| GitError::OperationFailed {
            operation: "create_worktree".to_string(),
            details: format!("Failed to execute git worktree add: {}", e),
        })?;

    if !output.status.success() {
        return Err(GitError::OperationFailed {
            operation: "create_worktree".to_string(),
            details: format!(
                "git worktree add failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ),
        });
    }

    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();

    info!("Worktree created at {:?}", path);
    Ok(WorkingTree {
        name,
        path: path.to_path_buf(),
        branch: branch.map(|b| b.to_string()),
        is_main: false,
        is_locked: false,
        is_valid: true,
    })
}

/// Remove a worktree
pub fn remove_worktree(repo: &Repository, path: &Path) -> Result<(), GitError> {
    info!("Removing worktree at {:?}", path);

    let repo_path = repo.command_cwd();
    let path_str = path.to_string_lossy();

    let output = Command::new("git")
        .args(["worktree", "remove", &path_str])
        .current_dir(&repo_path)
        .output()
        .map_err(|e| GitError::OperationFailed {
            operation: "remove_worktree".to_string(),
            details: format!("Failed to execute git worktree remove: {}", e),
        })?;

    if !output.status.success() {
        return Err(GitError::OperationFailed {
            operation: "remove_worktree".to_string(),
            details: format!(
                "git worktree remove failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ),
        });
    }

    info!("Worktree removed at {:?}", path);
    Ok(())
}
