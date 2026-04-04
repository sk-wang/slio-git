//! Commit-level actions for branch/history workflows.

use crate::commit;
use crate::error::GitError;
use crate::git_utils::{current_head_oid, is_ancestor, resolve_commit_oid};
use crate::index;
use crate::repository::{Repository, RepositoryState};
use log::info;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InProgressCommitActionKind {
    CherryPick,
    Revert,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InProgressCommitAction {
    pub kind: InProgressCommitActionKind,
    pub commit_id: Option<String>,
    pub subject: Option<String>,
    pub conflicted_files: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PushCurrentBranchTarget {
    pub remote_name: String,
    pub local_branch_name: String,
    pub upstream_ref: String,
    pub upstream_branch_name: String,
    pub selected_commit: String,
    pub is_fast_forward: bool,
    pub requires_force_with_lease: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RewriteExecution {
    Completed,
    InProgress,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RewriteKind {
    EditMessage,
    Fixup,
    Squash,
    Drop,
}

#[derive(Debug, Clone)]
struct RewriteSelection {
    selected_oid: git2::Oid,
    base_spec: Option<String>,
    chain: Vec<git2::Oid>,
    selected_index: usize,
}

fn parse_upstream_ref(reference: &str) -> Option<(&str, &str)> {
    reference.split_once('/')
}

fn git_dir(repo: &Repository) -> &Path {
    &repo.path
}

fn has_rebase_in_progress(repo: &Repository) -> bool {
    let git_dir = git_dir(repo);
    git_dir.join("rebase-merge").exists() || git_dir.join("rebase-apply").exists()
}

fn commit_subject(message: &str) -> &str {
    message
        .lines()
        .find(|line| !line.trim().is_empty())
        .unwrap_or(message)
}

fn ensure_no_in_progress_operation(repo: &Repository, operation: &str) -> Result<(), GitError> {
    match repo.get_state() {
        RepositoryState::Clean | RepositoryState::Dirty => Ok(()),
        _ => Err(GitError::OperationFailed {
            operation: operation.to_string(),
            details: format!(
                "当前仓库正处于 {}，请先完成或中止当前流程",
                repo.state_hint()
                    .unwrap_or_else(|| "进行中的 Git 操作".to_string())
            ),
        }),
    }
}

fn ensure_clean_worktree(repo: &Repository, operation: &str) -> Result<(), GitError> {
    ensure_no_in_progress_operation(repo, operation)?;

    if index::get_status(repo)
        .map_err(|e| GitError::OperationFailed {
            operation: operation.to_string(),
            details: e.to_string(),
        })?
        .is_empty()
    {
        Ok(())
    } else {
        Err(GitError::OperationFailed {
            operation: operation.to_string(),
            details: "当前仓库还有未提交改动，请先提交、暂存或清理工作区".to_string(),
        })
    }
}

fn run_git_command(repo: &Repository, operation: &str, args: &[String]) -> Result<(), GitError> {
    let output = Command::new("git")
        .args(args)
        .current_dir(repo.command_cwd())
        .output()
        .map_err(|e| GitError::OperationFailed {
            operation: operation.to_string(),
            details: format!("Failed to execute git {operation}: {e}"),
        })?;

    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let details = if stderr.trim().is_empty() {
        stdout.trim().to_string()
    } else {
        stderr.trim().to_string()
    };

    Err(GitError::OperationFailed {
        operation: operation.to_string(),
        details: format!("git {operation} failed: {details}"),
    })
}

fn current_branch_first_parent_chain(
    repo: &Repository,
    operation: &str,
) -> Result<Vec<git2::Oid>, GitError> {
    let head_oid = current_head_oid(repo, operation)?;
    let repo_lock = repo.inner.read().unwrap();
    let mut commit = repo_lock
        .find_commit(head_oid)
        .map_err(|e| GitError::OperationFailed {
            operation: operation.to_string(),
            details: e.to_string(),
        })?;

    let mut chain = Vec::new();
    loop {
        chain.push(commit.id());
        if commit.parent_count() == 0 {
            break;
        }
        commit = commit.parent(0).map_err(|e| GitError::OperationFailed {
            operation: operation.to_string(),
            details: e.to_string(),
        })?;
    }
    chain.reverse();
    Ok(chain)
}

fn ensure_local_rewrite_allowed(
    repo: &Repository,
    selected_oid: git2::Oid,
    operation: &str,
) -> Result<(), GitError> {
    if let Some(upstream_ref) = repo.current_upstream_ref() {
        let upstream_oid = resolve_commit_oid(repo, &upstream_ref, operation)?;
        if is_ancestor(repo, selected_oid, upstream_oid)? {
            return Err(GitError::OperationFailed {
                operation: operation.to_string(),
                details: format!(
                    "提交已经包含在当前上游 {upstream_ref} 中，暂不支持直接改写已发布历史"
                ),
            });
        }
    }
    Ok(())
}

fn resolve_rewrite_selection(
    repo: &Repository,
    commit_id: &str,
    operation: &str,
    kind: RewriteKind,
) -> Result<RewriteSelection, GitError> {
    ensure_clean_worktree(repo, operation)?;

    let current_branch = repo
        .current_branch()
        .map_err(|e| GitError::OperationFailed {
            operation: operation.to_string(),
            details: e.to_string(),
        })?;
    if current_branch.is_none() {
        return Err(GitError::OperationFailed {
            operation: operation.to_string(),
            details: "当前为 detached HEAD，不能直接改写当前分支历史".to_string(),
        });
    }

    let selected_oid = resolve_commit_oid(repo, commit_id, operation)?;
    let head_oid = current_head_oid(repo, operation)?;
    if !is_ancestor(repo, selected_oid, head_oid)? {
        return Err(GitError::OperationFailed {
            operation: operation.to_string(),
            details: "只能改写当前分支主线上的历史提交".to_string(),
        });
    }

    let chain = current_branch_first_parent_chain(repo, operation)?;
    let Some(selected_index) = chain.iter().position(|oid| *oid == selected_oid) else {
        return Err(GitError::OperationFailed {
            operation: operation.to_string(),
            details: "暂只支持改写当前分支第一父链上的提交".to_string(),
        });
    };

    let selected_info = commit::get_commit(repo, commit_id)?;
    if selected_info.parent_ids.len() > 1 {
        return Err(GitError::OperationFailed {
            operation: operation.to_string(),
            details: "merge 提交暂不支持直接改说明、fixup、squash 或删除".to_string(),
        });
    }

    if matches!(kind, RewriteKind::Fixup | RewriteKind::Squash) && selected_index == 0 {
        return Err(GitError::OperationFailed {
            operation: operation.to_string(),
            details: "根提交前面没有可合并的目标提交".to_string(),
        });
    }

    ensure_local_rewrite_allowed(repo, selected_oid, operation)?;

    let range_start = match kind {
        RewriteKind::EditMessage | RewriteKind::Drop => selected_index,
        RewriteKind::Fixup | RewriteKind::Squash => selected_index.saturating_sub(1),
    };

    let base_spec = if range_start == 0 {
        None
    } else {
        Some(chain[range_start - 1].to_string())
    };

    Ok(RewriteSelection {
        selected_oid,
        base_spec,
        chain,
        selected_index,
    })
}

fn commit_subject_for_oid(
    repo_lock: &git2::Repository,
    oid: git2::Oid,
    operation: &str,
) -> Result<String, GitError> {
    let commit = repo_lock
        .find_commit(oid)
        .map_err(|e| GitError::OperationFailed {
            operation: operation.to_string(),
            details: e.to_string(),
        })?;
    Ok(commit
        .summary()
        .unwrap_or("(no subject)")
        .replace(['\n', '\r'], " "))
}

fn build_rewrite_todo(
    repo: &Repository,
    selection: &RewriteSelection,
    operation: &str,
    selected_action: &str,
) -> Result<String, GitError> {
    let start_index = selection
        .base_spec
        .as_ref()
        .and_then(|base| {
            selection
                .chain
                .iter()
                .position(|oid| oid.to_string() == *base)
        })
        .map(|index| index + 1)
        .unwrap_or(0);
    let repo_lock = repo.inner.read().unwrap();

    selection.chain[start_index..]
        .iter()
        .map(|oid| {
            let subject = commit_subject_for_oid(&repo_lock, *oid, operation)?;
            let action = if *oid == selection.selected_oid {
                selected_action
            } else {
                "pick"
            };
            Ok(format!("{action} {oid} {subject}\n"))
        })
        .collect()
}

fn rewrite_temp_dir(operation: &str) -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    std::env::temp_dir().join(format!(
        "slio-git-{operation}-{}-{timestamp}",
        std::process::id()
    ))
}

fn write_sequence_editor_script(
    operation: &str,
    todo_path: &Path,
    script_path: &Path,
) -> Result<(), GitError> {
    #[cfg(unix)]
    let contents = format!("#!/bin/sh\ncat '{}' > \"$1\"\n", todo_path.display());
    #[cfg(windows)]
    let contents = format!("@echo off\r\ntype \"{}\" > %1\r\n", todo_path.display());

    fs::write(script_path, contents).map_err(GitError::Io)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(script_path)
            .map_err(GitError::Io)?
            .permissions();
        permissions.set_mode(0o700);
        fs::set_permissions(script_path, permissions).map_err(GitError::Io)?;
    }

    let _ = operation;
    Ok(())
}

fn run_scripted_interactive_rebase(
    repo: &Repository,
    operation: &str,
    base_spec: Option<&str>,
    todo_contents: &str,
    auto_accept_editor: bool,
) -> Result<RewriteExecution, GitError> {
    let temp_dir = rewrite_temp_dir(operation);
    fs::create_dir_all(&temp_dir).map_err(GitError::Io)?;

    let todo_path = temp_dir.join("git-rebase-todo");
    let script_path = if cfg!(windows) {
        temp_dir.join("sequence-editor.cmd")
    } else {
        temp_dir.join("sequence-editor.sh")
    };

    fs::write(&todo_path, todo_contents).map_err(GitError::Io)?;
    write_sequence_editor_script(operation, &todo_path, &script_path)?;

    let mut command = Command::new("git");
    command.current_dir(repo.command_cwd());
    command.env("GIT_SEQUENCE_EDITOR", &script_path);
    if auto_accept_editor {
        command.env("GIT_EDITOR", "true");
    }

    command.arg("rebase").arg("-i");
    if let Some(base_spec) = base_spec {
        command.arg(base_spec);
    } else {
        command.arg("--root");
    }

    let output = command.output().map_err(|e| GitError::OperationFailed {
        operation: operation.to_string(),
        details: format!("Failed to execute git {operation}: {e}"),
    })?;

    let cleanup_result = fs::remove_dir_all(&temp_dir);
    if cleanup_result.is_err() {
        let _ = cleanup_result;
    }

    if output.status.success() {
        return Ok(if has_rebase_in_progress(repo) {
            RewriteExecution::InProgress
        } else {
            RewriteExecution::Completed
        });
    }

    if has_rebase_in_progress(repo) {
        return Ok(RewriteExecution::InProgress);
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let details = if stderr.trim().is_empty() {
        stdout.trim().to_string()
    } else {
        stderr.trim().to_string()
    };

    Err(GitError::OperationFailed {
        operation: operation.to_string(),
        details: format!("git {operation} failed: {details}"),
    })
}

pub fn export_commit_patch(
    repo: &Repository,
    commit_id: &str,
    output_path: &Path,
) -> Result<(), GitError> {
    info!(
        "Exporting patch for commit '{}' to '{}'",
        commit_id,
        output_path.display()
    );

    let output = Command::new("git")
        .args(["format-patch", "--stdout", "-1", commit_id])
        .current_dir(repo.command_cwd())
        .output()
        .map_err(|e| GitError::OperationFailed {
            operation: "format-patch".to_string(),
            details: format!("Failed to execute git format-patch: {e}"),
        })?;

    if !output.status.success() {
        return Err(GitError::OperationFailed {
            operation: "format-patch".to_string(),
            details: String::from_utf8_lossy(&output.stderr).trim().to_string(),
        });
    }

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(output_path, output.stdout)?;
    Ok(())
}

pub fn get_in_progress_commit_action(
    repo: &Repository,
) -> Result<Option<InProgressCommitAction>, GitError> {
    let (kind, head_file) = match repo.get_state() {
        RepositoryState::CherryPick => (
            InProgressCommitActionKind::CherryPick,
            git_dir(repo).join("CHERRY_PICK_HEAD"),
        ),
        RepositoryState::Revert => (
            InProgressCommitActionKind::Revert,
            git_dir(repo).join("REVERT_HEAD"),
        ),
        _ => return Ok(None),
    };

    let commit_id = fs::read_to_string(&head_file)
        .ok()
        .map(|contents| contents.trim().to_string())
        .filter(|contents| !contents.is_empty());
    let subject = commit_id.as_deref().and_then(|commit_id| {
        commit::get_commit(repo, commit_id)
            .ok()
            .map(|info| commit_subject(&info.message).to_string())
    });
    let conflicted_files = index::get_conflicted_files(repo).unwrap_or_default();

    Ok(Some(InProgressCommitAction {
        kind,
        commit_id,
        subject,
        conflicted_files,
    }))
}

pub fn cherry_pick_commit(repo: &Repository, commit_id: &str) -> Result<(), GitError> {
    info!("Cherry-picking commit '{}'", commit_id);
    ensure_clean_worktree(repo, "cherry-pick")?;

    let args = vec![
        "cherry-pick".to_string(),
        "--no-edit".to_string(),
        commit_id.to_string(),
    ];
    run_git_command(repo, "cherry-pick", &args)
}

pub fn revert_commit(repo: &Repository, commit_id: &str) -> Result<(), GitError> {
    info!("Reverting commit '{}'", commit_id);
    ensure_clean_worktree(repo, "revert")?;

    let args = vec![
        "revert".to_string(),
        "--no-edit".to_string(),
        commit_id.to_string(),
    ];
    run_git_command(repo, "revert", &args)
}

pub fn edit_commit_message(
    repo: &Repository,
    commit_id: &str,
) -> Result<RewriteExecution, GitError> {
    info!("Starting edit-message rewrite for '{}'", commit_id);
    let selection = resolve_rewrite_selection(repo, commit_id, "reword", RewriteKind::EditMessage)?;
    let todo = build_rewrite_todo(repo, &selection, "reword", "edit")?;
    run_scripted_interactive_rebase(repo, "reword", selection.base_spec.as_deref(), &todo, false)
}

pub fn fixup_commit_to_previous(
    repo: &Repository,
    commit_id: &str,
) -> Result<RewriteExecution, GitError> {
    info!("Fixup commit '{}' into its previous commit", commit_id);
    let selection = resolve_rewrite_selection(repo, commit_id, "fixup", RewriteKind::Fixup)?;
    let todo = build_rewrite_todo(repo, &selection, "fixup", "fixup")?;
    run_scripted_interactive_rebase(repo, "fixup", selection.base_spec.as_deref(), &todo, false)
}

pub fn squash_commit_to_previous(
    repo: &Repository,
    commit_id: &str,
) -> Result<RewriteExecution, GitError> {
    info!("Squashing commit '{}' into its previous commit", commit_id);
    let selection = resolve_rewrite_selection(repo, commit_id, "squash", RewriteKind::Squash)?;
    let todo = build_rewrite_todo(repo, &selection, "squash", "squash")?;
    run_scripted_interactive_rebase(repo, "squash", selection.base_spec.as_deref(), &todo, true)
}

pub fn drop_commit_from_history(
    repo: &Repository,
    commit_id: &str,
) -> Result<RewriteExecution, GitError> {
    info!("Dropping commit '{}' from current history", commit_id);
    let selection = resolve_rewrite_selection(repo, commit_id, "drop", RewriteKind::Drop)?;
    if selection.selected_index == 0 && selection.chain.len() == 1 {
        return Err(GitError::OperationFailed {
            operation: "drop".to_string(),
            details: "当前分支只剩下这一条根提交，不能直接删除".to_string(),
        });
    }
    let todo = build_rewrite_todo(repo, &selection, "drop", "drop")?;
    run_scripted_interactive_rebase(repo, "drop", selection.base_spec.as_deref(), &todo, false)
}

pub fn continue_in_progress_commit_action(
    repo: &Repository,
    kind: InProgressCommitActionKind,
) -> Result<(), GitError> {
    let operation = match kind {
        InProgressCommitActionKind::CherryPick => "cherry-pick",
        InProgressCommitActionKind::Revert => "revert",
    };

    let add_output = Command::new("git")
        .args(["add", "-A"])
        .current_dir(repo.command_cwd())
        .output()
        .map_err(|e| GitError::OperationFailed {
            operation: format!("{operation}_continue"),
            details: format!("Failed to execute git add: {e}"),
        })?;

    if !add_output.status.success() {
        return Err(GitError::OperationFailed {
            operation: format!("{operation}_continue"),
            details: format!(
                "git add failed: {}",
                String::from_utf8_lossy(&add_output.stderr)
            ),
        });
    }

    let args = match kind {
        InProgressCommitActionKind::CherryPick => {
            vec![
                "-c".to_string(),
                "core.editor=true".to_string(),
                "cherry-pick".to_string(),
                "--continue".to_string(),
            ]
        }
        InProgressCommitActionKind::Revert => {
            vec![
                "-c".to_string(),
                "core.editor=true".to_string(),
                "revert".to_string(),
                "--continue".to_string(),
            ]
        }
    };

    run_git_command(repo, &format!("{operation}_continue"), &args)
}

pub fn abort_in_progress_commit_action(
    repo: &Repository,
    kind: InProgressCommitActionKind,
) -> Result<(), GitError> {
    let (operation, args) = match kind {
        InProgressCommitActionKind::CherryPick => (
            "cherry-pick",
            vec!["cherry-pick".to_string(), "--abort".to_string()],
        ),
        InProgressCommitActionKind::Revert => {
            ("revert", vec!["revert".to_string(), "--abort".to_string()])
        }
    };

    run_git_command(repo, &format!("{operation}_abort"), &args)
}

pub fn reset_current_branch_to_commit(repo: &Repository, commit_id: &str) -> Result<(), GitError> {
    info!("Resetting current branch to '{}'", commit_id);
    ensure_clean_worktree(repo, "reset")?;

    let current_branch = repo
        .current_branch()
        .map_err(|e| GitError::OperationFailed {
            operation: "reset".to_string(),
            details: e.to_string(),
        })?;
    if current_branch.is_none() {
        return Err(GitError::OperationFailed {
            operation: "reset".to_string(),
            details: "当前为 detached HEAD，无法重置当前分支".to_string(),
        });
    }

    let head_oid = current_head_oid(repo, "reset")?;
    let target_oid = resolve_commit_oid(repo, commit_id, "reset")?;
    if !is_ancestor(repo, target_oid, head_oid)? {
        return Err(GitError::OperationFailed {
            operation: "reset".to_string(),
            details: "只能把当前分支重置到它自己的历史祖先提交".to_string(),
        });
    }

    let args = vec![
        "reset".to_string(),
        "--hard".to_string(),
        commit_id.to_string(),
    ];
    run_git_command(repo, "reset", &args)
}

pub fn resolve_push_current_branch_target(
    repo: &Repository,
    commit_id: &str,
) -> Result<PushCurrentBranchTarget, GitError> {
    info!("Resolving push-to-here target for '{}'", commit_id);

    let local_branch_name = repo
        .current_branch()
        .map_err(|e| GitError::OperationFailed {
            operation: "push-to-here".to_string(),
            details: e.to_string(),
        })?
        .ok_or_else(|| GitError::OperationFailed {
            operation: "push-to-here".to_string(),
            details: "当前为 detached HEAD，无法解析当前分支的远端目标".to_string(),
        })?;
    let upstream_ref = repo
        .current_upstream_ref()
        .ok_or_else(|| GitError::OperationFailed {
            operation: "push-to-here".to_string(),
            details: format!("当前分支 {local_branch_name} 还没有配置上游"),
        })?;
    let (remote_name, upstream_branch_name) =
        parse_upstream_ref(&upstream_ref).ok_or_else(|| GitError::OperationFailed {
            operation: "push-to-here".to_string(),
            details: format!("无法解析上游分支 {upstream_ref}"),
        })?;
    let remote_name = remote_name.to_string();
    let upstream_branch_name = upstream_branch_name.to_string();

    let head_oid = current_head_oid(repo, "push-to-here")?;
    let selected_oid = resolve_commit_oid(repo, commit_id, "push-to-here")?;
    if !is_ancestor(repo, selected_oid, head_oid)? {
        return Err(GitError::OperationFailed {
            operation: "push-to-here".to_string(),
            details: "只能把当前分支的远端发布到当前分支历史上的祖先提交".to_string(),
        });
    }

    let upstream_oid = resolve_commit_oid(repo, &upstream_ref, "push-to-here")?;
    let is_fast_forward = is_ancestor(repo, upstream_oid, selected_oid)?;

    Ok(PushCurrentBranchTarget {
        remote_name,
        local_branch_name,
        upstream_ref,
        upstream_branch_name,
        selected_commit: commit_id.to_string(),
        is_fast_forward,
        requires_force_with_lease: !is_fast_forward,
    })
}

pub fn push_current_branch_to_commit(
    repo: &Repository,
    target: &PushCurrentBranchTarget,
) -> Result<(), GitError> {
    info!(
        "Pushing current branch '{}' to '{}' at '{}'",
        target.local_branch_name, target.upstream_ref, target.selected_commit
    );

    ensure_no_in_progress_operation(repo, "push-to-here")?;

    let refspec = format!(
        "{}:refs/heads/{}",
        target.selected_commit, target.upstream_branch_name
    );
    let mut args = vec!["push".to_string()];
    if target.requires_force_with_lease {
        args.push("--force-with-lease".to_string());
    }
    args.push(target.remote_name.clone());
    args.push(refspec);

    run_git_command(repo, "push", &args)
}

/// Uncommit: soft-reset from HEAD to the parent of the given commit.
/// All changes from the removed commits are returned to the staging area.
/// Equivalent to IDEA's "Uncommit" action.
pub fn uncommit_to_commit(repo: &Repository, commit_id: &str) -> Result<(), GitError> {
    info!(
        "Uncommitting from HEAD to commit {} (soft reset to parent)",
        commit_id
    );

    let repo_path = repo.command_cwd();

    // Resolve the target commit's parent
    let parent_ref = format!("{}^", commit_id);

    let output = Command::new("git")
        .args(["reset", "--soft", &parent_ref])
        .current_dir(&repo_path)
        .output()
        .map_err(|e| GitError::OperationFailed {
            operation: "uncommit_to_commit".to_string(),
            details: format!("Failed to execute git reset --soft: {}", e),
        })?;

    if !output.status.success() {
        return Err(GitError::OperationFailed {
            operation: "uncommit_to_commit".to_string(),
            details: format!(
                "git reset --soft failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ),
        });
    }

    info!("Uncommit completed — changes returned to staging area");
    Ok(())
}
