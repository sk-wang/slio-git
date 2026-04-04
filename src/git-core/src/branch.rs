//! Branch operations for git-core

use crate::error::GitError;
use crate::index;
use crate::repository::{compact_branch_sync_hint, compact_relative_time, Repository, SyncStatus};
use log::info;
use std::process::Command;

/// A Git branch
#[derive(Debug, Clone)]
pub struct Branch {
    pub name: String,
    pub oid: String,
    pub is_remote: bool,
    pub is_head: bool,
    pub upstream: Option<String>,
    pub tracking_status: Option<String>,
    pub sync_hint: Option<String>,
    pub recency_hint: Option<String>,
    pub last_commit_timestamp: Option<i64>,
    /// Hierarchical group path for tree display (e.g., ["feature", "auth"] for "feature/auth")
    pub group_path: Option<Vec<String>>,
}

impl Branch {
    /// Compute the group_path from the branch name by splitting on '/'
    pub fn compute_group_path(&mut self) {
        let display_name = if self.is_remote {
            // For remote branches like "origin/feature/auth", skip the remote name
            self.name.split_once('/').map(|x| x.1)
                .unwrap_or(&self.name)
        } else {
            &self.name
        };

        let parts: Vec<&str> = display_name.split('/').collect();
        if parts.len() > 1 {
            // All but the last part form the group path
            self.group_path = Some(parts[..parts.len() - 1].iter().map(|s| s.to_string()).collect());
        } else {
            self.group_path = None;
        }
    }

    /// Get the leaf name (last segment after '/')
    pub fn leaf_name(&self) -> &str {
        self.name.rsplit('/').next().unwrap_or(&self.name)
    }
}

impl Repository {
    /// Create a new branch
    /// Check if a local branch is fully merged into HEAD.
    pub fn is_branch_merged(&self, name: &str) -> Result<bool, GitError> {
        let repo_lock = self.inner.read().unwrap();
        let branch = repo_lock
            .find_branch(name, git2::BranchType::Local)
            .map_err(|_| GitError::BranchNotFound {
                name: name.to_string(),
            })?;
        let branch_oid = branch.get().target().ok_or_else(|| GitError::BranchNotFound {
            name: name.to_string(),
        })?;
        let head_oid = repo_lock
            .head()
            .ok()
            .and_then(|h| h.target())
            .ok_or_else(|| GitError::OperationFailed {
                operation: "is_branch_merged".to_string(),
                details: "No HEAD reference found".to_string(),
            })?;
        let merge_base = repo_lock
            .merge_base(branch_oid, head_oid)
            .map_err(|e| GitError::OperationFailed {
                operation: "is_branch_merged".to_string(),
                details: e.to_string(),
            })?;
        Ok(merge_base == branch_oid)
    }

    pub fn create_branch(&self, name: &str, oid: &str) -> Result<Branch, GitError> {
        self.create_branch_from_start_point(name, oid)
    }

    /// Create a new branch from a commit, ref, or other git revspec.
    pub fn create_branch_from_start_point(
        &self,
        name: &str,
        start_point: &str,
    ) -> Result<Branch, GitError> {
        info!("Creating branch '{}' from '{}'", name, start_point);

        let repo_path = self.command_cwd();

        // Use git branch command
        let output = Command::new("git")
            .args(["branch", name, start_point])
            .current_dir(&repo_path)
            .output()
            .map_err(|e| GitError::OperationFailed {
                operation: "create_branch".to_string(),
                details: format!("Failed to execute git branch: {}", e),
            })?;

        if !output.status.success() {
            return Err(GitError::OperationFailed {
                operation: "create_branch".to_string(),
                details: format!(
                    "git branch failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                ),
            });
        }

        Ok(Branch {
            name: name.to_string(),
            oid: String::new(),
            is_remote: false,
            is_head: false,
            upstream: None,
            tracking_status: None,
            sync_hint: None,
            recency_hint: None,
            last_commit_timestamp: None,
            group_path: None,
        })
    }

    /// Delete a branch
    pub fn delete_branch(&self, name: &str) -> Result<(), GitError> {
        info!("Deleting branch '{}'", name);

        let repo_path = self.command_cwd();

        // Use git branch -d command
        let output = Command::new("git")
            .args(["branch", "-d", name])
            .current_dir(&repo_path)
            .output()
            .map_err(|e| GitError::OperationFailed {
                operation: "delete_branch".to_string(),
                details: format!("Failed to execute git branch: {}", e),
            })?;

        if !output.status.success() {
            return Err(GitError::OperationFailed {
                operation: "delete_branch".to_string(),
                details: format!(
                    "git branch -d failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                ),
            });
        }

        Ok(())
    }

    /// Rename a branch
    pub fn rename_branch(&self, old_name: &str, new_name: &str) -> Result<Branch, GitError> {
        info!("Renaming branch '{}' to '{}'", old_name, new_name);

        let repo_path = self.command_cwd();

        // Use git branch -m command
        let output = Command::new("git")
            .args(["branch", "-m", old_name, new_name])
            .current_dir(&repo_path)
            .output()
            .map_err(|e| GitError::OperationFailed {
                operation: "rename_branch".to_string(),
                details: format!("Failed to execute git branch: {}", e),
            })?;

        if !output.status.success() {
            return Err(GitError::OperationFailed {
                operation: "rename_branch".to_string(),
                details: format!(
                    "git branch -m failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                ),
            });
        }

        Ok(Branch {
            name: new_name.to_string(),
            oid: String::new(),
            is_remote: false,
            is_head: false,
            upstream: None,
            tracking_status: None,
            sync_hint: None,
            recency_hint: None,
            last_commit_timestamp: None,
            group_path: None,
        })
    }

    /// Configure the local branch to track an upstream branch.
    pub fn set_branch_upstream(&self, branch_name: &str, upstream: &str) -> Result<(), GitError> {
        info!(
            "Setting upstream of branch '{}' to '{}'",
            branch_name, upstream
        );

        let repo_path = self.command_cwd();

        let output = Command::new("git")
            .args(["branch", "--set-upstream-to", upstream, branch_name])
            .current_dir(&repo_path)
            .output()
            .map_err(|e| GitError::OperationFailed {
                operation: "set_branch_upstream".to_string(),
                details: format!("Failed to execute git branch: {}", e),
            })?;

        if !output.status.success() {
            return Err(GitError::OperationFailed {
                operation: "set_branch_upstream".to_string(),
                details: format!(
                    "git branch --set-upstream-to failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                ),
            });
        }

        Ok(())
    }

    /// Checkout a branch
    pub fn checkout_branch(&self, name: &str) -> Result<(), GitError> {
        info!("Checking out branch '{}'", name);

        let repo_path = self.command_cwd();

        // Use git checkout command
        let output = Command::new("git")
            .args(["checkout", name])
            .current_dir(&repo_path)
            .output()
            .map_err(|e| GitError::OperationFailed {
                operation: "checkout_branch".to_string(),
                details: format!("Failed to execute git checkout: {}", e),
            })?;

        if !output.status.success() {
            return Err(GitError::OperationFailed {
                operation: "checkout_branch".to_string(),
                details: format!(
                    "git checkout failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                ),
            });
        }

        Ok(())
    }

    /// Checkout a remote branch by creating or reusing a local tracking branch.
    pub fn checkout_remote_branch(&self, remote_ref: &str) -> Result<String, GitError> {
        info!("Checking out remote branch '{}'", remote_ref);

        let Some((_, local_branch_name)) = remote_ref.split_once('/') else {
            return Err(GitError::OperationFailed {
                operation: "checkout_remote_branch".to_string(),
                details: format!("invalid remote branch ref: {remote_ref}"),
            });
        };

        let repo_path = self.command_cwd();
        let local_ref = format!("refs/heads/{local_branch_name}");
        let local_branch_exists = Command::new("git")
            .args(["show-ref", "--verify", "--quiet", &local_ref])
            .current_dir(&repo_path)
            .status()
            .map_err(|e| GitError::OperationFailed {
                operation: "checkout_remote_branch".to_string(),
                details: format!("Failed to inspect local branch refs: {}", e),
            })?
            .success();

        let args = if local_branch_exists {
            vec!["checkout", local_branch_name]
        } else {
            vec!["checkout", "--track", remote_ref]
        };

        let output = Command::new("git")
            .args(&args)
            .current_dir(&repo_path)
            .output()
            .map_err(|e| GitError::OperationFailed {
                operation: "checkout_remote_branch".to_string(),
                details: format!("Failed to execute git checkout: {}", e),
            })?;

        if !output.status.success() {
            return Err(GitError::OperationFailed {
                operation: "checkout_remote_branch".to_string(),
                details: format!(
                    "git checkout failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                ),
            });
        }

        Ok(local_branch_name.to_string())
    }

    /// Merge a branch into current branch
    pub fn merge_branch(&self, branch_name: &str) -> Result<(), GitError> {
        info!("Merging branch '{}' into current branch", branch_name);

        let repo_path = self.command_cwd();

        // Use git merge command
        let output = Command::new("git")
            .args(["merge", branch_name])
            .current_dir(&repo_path)
            .output()
            .map_err(|e| GitError::OperationFailed {
                operation: "merge_branch".to_string(),
                details: format!("Failed to execute git merge: {}", e),
            })?;

        if !output.status.success() {
            if index::has_conflicts(self) {
                return Err(GitError::MergeConflict);
            }

            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let details = if !stderr.is_empty() {
                stderr
            } else if !stdout.is_empty() {
                stdout
            } else {
                "git merge 返回失败，但没有输出可读错误详情".to_string()
            };

            return Err(GitError::OperationFailed {
                operation: "merge_branch".to_string(),
                details: format!("git merge failed: {details}"),
            });
        }

        Ok(())
    }

    /// Get list of all branches
    pub fn list_branches(&self) -> Result<Vec<Branch>, GitError> {
        info!("Listing all branches");

        let repo_path = self.command_cwd();

        let output = Command::new("git")
            .args([
                "for-each-ref",
                "--sort=-committerdate",
                "--format=%(refname:short)\t%(objectname:short)\t%(upstream:short)\t%(upstream:trackshort)\t%(committerdate:unix)\t%(HEAD)",
                "refs/heads",
            ])
            .current_dir(&repo_path)
            .output()
            .map_err(|e| GitError::OperationFailed {
                operation: "list_branches".to_string(),
                details: format!("Failed to execute git branch: {}", e),
            })?;

        if !output.status.success() {
            return Err(GitError::OperationFailed {
                operation: "list_branches".to_string(),
                details: format!(
                    "git branch failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                ),
            });
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        let mut branches = Vec::new();
        let current_branch = self.current_branch().ok().flatten().unwrap_or_default();

        for line in output_str.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split('\t').collect();
            let name = parts.first().copied().unwrap_or("").trim().to_string();
            let oid = parts.get(1).copied().unwrap_or("").trim().to_string();
            let upstream = parts
                .get(2)
                .copied()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string);
            let tracking_status = normalize_tracking_status(parts.get(3).copied().unwrap_or(""));
            let last_commit_timestamp = parts
                .get(4)
                .copied()
                .and_then(|value| value.trim().parse::<i64>().ok());
            let is_head = parts
                .get(5)
                .copied()
                .map(str::trim)
                .is_some_and(|value| value == "*")
                || name == current_branch;

            if name.is_empty() {
                continue;
            }

            let tracking_status = if is_head {
                current_branch_tracking(self)
            } else {
                tracking_status
            };
            let sync_hint =
                compact_branch_sync_hint(upstream.as_deref(), tracking_status.as_deref());

            let mut branch = Branch {
                name: name.clone(),
                oid,
                is_remote: false,
                is_head,
                upstream,
                sync_hint,
                recency_hint: compact_relative_time(last_commit_timestamp),
                tracking_status,
                last_commit_timestamp,
                group_path: None,
            };
            branch.compute_group_path();
            branches.push(branch);
        }

        let remote_output = Command::new("git")
            .args([
                "for-each-ref",
                "--sort=-committerdate",
                "--format=%(refname:short)\t%(symref)\t%(objectname:short)\t%(committerdate:unix)",
                "refs/remotes",
            ])
            .current_dir(&repo_path)
            .output()
            .map_err(|e| GitError::OperationFailed {
                operation: "list_branches".to_string(),
                details: format!("Failed to execute git branch -r: {}", e),
            })?;

        if remote_output.status.success() {
            let remote_str = String::from_utf8_lossy(&remote_output.stdout);
            for line in remote_str.lines() {
                let line = line.trim();
                if line.is_empty() || line.contains("->") {
                    continue;
                }

                let parts: Vec<&str> = line.split('\t').collect();
                let name = parts.first().copied().unwrap_or("").trim().to_string();
                let symref = parts.get(1).copied().unwrap_or("").trim();
                let oid = parts.get(2).copied().unwrap_or("").trim().to_string();
                let last_commit_timestamp = parts
                    .get(3)
                    .copied()
                    .and_then(|value| value.trim().parse::<i64>().ok());

                if !name.is_empty() && symref.is_empty() && !name.ends_with("/HEAD") {
                    let mut branch = Branch {
                        name,
                        oid,
                        is_remote: true,
                        is_head: false,
                        upstream: None,
                        tracking_status: None,
                        sync_hint: None,
                        recency_hint: compact_relative_time(last_commit_timestamp),
                        last_commit_timestamp,
                        group_path: None,
                    };
                    branch.compute_group_path();
                    branches.push(branch);
                }
            }
        }

        Ok(branches)
    }
}

fn current_branch_tracking(repo: &Repository) -> Option<String> {
    match repo.sync_status() {
        SyncStatus::Ahead(count) => Some(format!("↑{count}")),
        SyncStatus::Behind(count) => Some(format!("↓{count}")),
        SyncStatus::Diverged { ahead, behind } => Some(format!("↕{ahead}/{behind}")),
        SyncStatus::Synced => Some("✓".to_string()),
        SyncStatus::NoUpstream => None,
        SyncStatus::Unknown => Some("?".to_string()),
    }
}

fn normalize_tracking_status(raw: &str) -> Option<String> {
    match raw.trim() {
        ">" => Some("↑".to_string()),
        "<" => Some("↓".to_string()),
        "<>" => Some("↕".to_string()),
        "=" => Some("✓".to_string()),
        "" => None,
        other => Some(other.to_string()),
    }
}
