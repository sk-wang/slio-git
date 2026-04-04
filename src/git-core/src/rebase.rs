//! Rebase operations for git-core
//!
//! Provides rebase functionality using git commands

use crate::error::GitError;
use crate::index;
use crate::repository::{Repository, RepositoryState};
use git2::Oid;
use log::info;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

/// Rebase operation result
#[derive(Debug, Clone)]
pub struct RebaseResult {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RebaseTodoEntry {
    pub action: String,
    pub commit: String,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct InteractiveRebasePlan {
    pub base_ref: Option<String>,
    pub start_commit: String,
    pub entries: Vec<RebaseTodoEntry>,
}

fn git_dir(repo: &Repository) -> PathBuf {
    let workdir = repo.command_cwd();
    let dot_git = workdir.join(".git");
    if dot_git.is_dir() {
        return dot_git;
    }

    if dot_git.is_file() {
        if let Ok(contents) = fs::read_to_string(&dot_git) {
            let trimmed = contents.trim();
            if let Some(path) = trimmed.strip_prefix("gitdir:") {
                let candidate = workdir.join(path.trim());
                if candidate.exists() {
                    return candidate;
                }
            }
        }
    }

    repo.path.join(".git")
}

fn is_rebase_in_progress(repo: &Repository) -> bool {
    let git_dir = git_dir(repo);
    git_dir.join("rebase-merge").exists() || git_dir.join("rebase-apply").exists()
}

fn current_head_oid(repo: &Repository, operation: &str) -> Result<Oid, GitError> {
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

fn resolve_commit_oid(repo: &Repository, spec: &str, operation: &str) -> Result<Oid, GitError> {
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
    let _ = operation;
    Ok(commit.id())
}

fn is_ancestor(repo: &Repository, ancestor: Oid, descendant: Oid) -> Result<bool, GitError> {
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

fn ensure_clean_worktree(repo: &Repository, operation: &str) -> Result<(), GitError> {
    match repo.get_state() {
        RepositoryState::Clean | RepositoryState::Dirty => {}
        _ => {
            return Err(GitError::OperationFailed {
                operation: operation.to_string(),
                details: format!(
                    "当前仓库正处于 {}，请先完成或中止当前流程",
                    repo.state_hint()
                        .unwrap_or_else(|| "进行中的 Git 操作".to_string())
                ),
            });
        }
    }

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

fn current_branch_first_parent_chain(
    repo: &Repository,
    operation: &str,
) -> Result<Vec<Oid>, GitError> {
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

fn commit_subject_for_oid(
    repo_lock: &git2::Repository,
    oid: Oid,
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

fn ensure_local_interactive_rebase_allowed(
    repo: &Repository,
    start_oid: Oid,
    operation: &str,
) -> Result<(), GitError> {
    if let Some(upstream_ref) = repo.current_upstream_ref() {
        let upstream_oid = resolve_commit_oid(repo, &upstream_ref, operation)?;
        if is_ancestor(repo, start_oid, upstream_oid)? {
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

fn interactive_rebase_temp_dir(operation: &str) -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    std::env::temp_dir().join(format!(
        "slio-git-{operation}-{}-{timestamp}",
        std::process::id()
    ))
}

fn write_sequence_editor_script(todo_path: &Path, script_path: &Path) -> Result<(), GitError> {
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

    Ok(())
}

fn parse_todo_line(line: &str) -> Option<RebaseTodoEntry> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') || trimmed == "noop" {
        return None;
    }

    let mut parts = trimmed.splitn(3, char::is_whitespace);
    let action = parts.next()?.trim();
    let commit = parts.next()?.trim();
    let message = parts.next().unwrap_or("").trim();
    if action.is_empty() || commit.is_empty() {
        return None;
    }

    Some(RebaseTodoEntry {
        action: action.to_string(),
        commit: commit.to_string(),
        message: message.to_string(),
    })
}

fn read_last_todo_entry(path: &Path) -> Result<Option<RebaseTodoEntry>, GitError> {
    if !path.exists() {
        return Ok(None);
    }

    let contents = fs::read_to_string(path).map_err(|e| GitError::OperationFailed {
        operation: "read_rebase_done".to_string(),
        details: format!("读取 rebase 步骤文件失败: {e}"),
    })?;

    Ok(contents.lines().rev().find_map(parse_todo_line))
}

fn build_todo_contents(entries: &[RebaseTodoEntry]) -> Result<String, GitError> {
    if entries.is_empty() {
        return Err(GitError::OperationFailed {
            operation: "interactive_rebase".to_string(),
            details: "至少需要保留一条 todo 项".to_string(),
        });
    }

    let mut contents = String::new();
    for (index, entry) in entries.iter().enumerate() {
        let normalized_action = entry.action.trim().to_lowercase();
        let action = if normalized_action == "reword" {
            "edit"
        } else {
            normalized_action.as_str()
        };
        if action.is_empty() {
            return Err(GitError::OperationFailed {
                operation: "interactive_rebase".to_string(),
                details: format!("第 {} 条 todo 缺少动作", index + 1),
            });
        }
        if index == 0 && (action == "fixup" || action == "squash") {
            return Err(GitError::OperationFailed {
                operation: "interactive_rebase".to_string(),
                details: "首条 todo 不能直接使用 fixup 或 squash".to_string(),
            });
        }
        if entry.commit.trim().is_empty() {
            return Err(GitError::OperationFailed {
                operation: "interactive_rebase".to_string(),
                details: format!("第 {} 条 todo 缺少提交哈希", index + 1),
            });
        }
        contents.push_str(action);
        contents.push(' ');
        contents.push_str(entry.commit.trim());
        if !entry.message.trim().is_empty() {
            contents.push(' ');
            contents.push_str(entry.message.trim());
        }
        contents.push('\n');
    }

    Ok(contents)
}

fn command_result_message(output: &std::process::Output) -> String {
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    match (stdout.is_empty(), stderr.is_empty()) {
        (true, true) => String::new(),
        (false, true) => stdout,
        (true, false) => stderr,
        (false, false) => format!("{stderr}\n{stdout}"),
    }
}

/// Start a rebase onto the given branch / ref.
pub fn rebase_start(repo: &Repository, onto: &str) -> Result<String, GitError> {
    info!("Starting rebase onto '{}'", onto);

    let repo_path = repo.command_cwd();

    let output = Command::new("git")
        .args(["rebase", onto])
        .current_dir(&repo_path)
        .output()
        .map_err(|e| GitError::OperationFailed {
            operation: "rebase_start".to_string(),
            details: format!("Failed to execute git rebase: {}", e),
        })?;

    if !output.status.success() {
        return Err(GitError::OperationFailed {
            operation: "rebase_start".to_string(),
            details: format!(
                "git rebase failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ),
        });
    }

    info!("Rebase started successfully");
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if stdout.is_empty() {
        Ok(format!("Rebase started onto {onto}"))
    } else {
        Ok(stdout)
    }
}

pub fn prepare_interactive_rebase_plan(
    repo: &Repository,
    start_commit: &str,
) -> Result<InteractiveRebasePlan, GitError> {
    info!(
        "Preparing interactive rebase plan from commit '{}'",
        start_commit
    );
    ensure_clean_worktree(repo, "interactive_rebase_prepare")?;

    let current_branch = repo
        .current_branch()
        .map_err(|e| GitError::OperationFailed {
            operation: "interactive_rebase_prepare".to_string(),
            details: e.to_string(),
        })?;
    if current_branch.is_none() {
        return Err(GitError::OperationFailed {
            operation: "interactive_rebase_prepare".to_string(),
            details: "当前为 detached HEAD，不能围绕当前分支开始交互式变基".to_string(),
        });
    }

    let start_oid = resolve_commit_oid(repo, start_commit, "interactive_rebase_prepare")?;
    let head_oid = current_head_oid(repo, "interactive_rebase_prepare")?;
    if !is_ancestor(repo, start_oid, head_oid)? {
        return Err(GitError::OperationFailed {
            operation: "interactive_rebase_prepare".to_string(),
            details: "只能从当前分支第一父链上的祖先提交开始整理历史".to_string(),
        });
    }

    ensure_local_interactive_rebase_allowed(repo, start_oid, "interactive_rebase_prepare")?;

    let chain = current_branch_first_parent_chain(repo, "interactive_rebase_prepare")?;
    let Some(start_index) = chain.iter().position(|oid| *oid == start_oid) else {
        return Err(GitError::OperationFailed {
            operation: "interactive_rebase_prepare".to_string(),
            details: "暂只支持围绕当前分支第一父链开始交互式变基".to_string(),
        });
    };

    let repo_lock = repo.inner.read().unwrap();
    let mut entries = Vec::new();
    for oid in &chain[start_index..] {
        let commit = repo_lock
            .find_commit(*oid)
            .map_err(|e| GitError::OperationFailed {
                operation: "interactive_rebase_prepare".to_string(),
                details: e.to_string(),
            })?;
        if commit.parent_count() > 1 {
            return Err(GitError::OperationFailed {
                operation: "interactive_rebase_prepare".to_string(),
                details: "当前整理范围里包含 merge 提交，暂不支持在这里开始交互式变基".to_string(),
            });
        }
        entries.push(RebaseTodoEntry {
            action: "pick".to_string(),
            commit: oid.to_string(),
            message: commit_subject_for_oid(&repo_lock, *oid, "interactive_rebase_prepare")?,
        });
    }

    let base_ref = if start_index == 0 {
        None
    } else {
        Some(chain[start_index - 1].to_string())
    };

    Ok(InteractiveRebasePlan {
        base_ref,
        start_commit: start_commit.to_string(),
        entries,
    })
}

pub fn start_interactive_rebase(
    repo: &Repository,
    base_ref: Option<&str>,
    entries: &[RebaseTodoEntry],
) -> Result<String, GitError> {
    info!(
        "Starting interactive rebase with {} todo entries",
        entries.len()
    );
    ensure_clean_worktree(repo, "interactive_rebase_start")?;

    let todo_contents = build_todo_contents(entries)?;
    let temp_dir = interactive_rebase_temp_dir("interactive_rebase");
    fs::create_dir_all(&temp_dir).map_err(GitError::Io)?;

    let todo_path = temp_dir.join("git-rebase-todo");
    let script_path = if cfg!(windows) {
        temp_dir.join("sequence-editor.cmd")
    } else {
        temp_dir.join("sequence-editor.sh")
    };

    fs::write(&todo_path, todo_contents).map_err(GitError::Io)?;
    write_sequence_editor_script(&todo_path, &script_path)?;

    let mut command = Command::new("git");
    command.current_dir(repo.command_cwd());
    command.env("GIT_SEQUENCE_EDITOR", &script_path);
    if entries
        .iter()
        .any(|entry| entry.action.eq_ignore_ascii_case("squash"))
    {
        command.env("GIT_EDITOR", "true");
    }

    command.arg("rebase").arg("-i");
    if let Some(base_ref) = base_ref {
        command.arg(base_ref);
    } else {
        command.arg("--root");
    }

    let output = command.output().map_err(|e| GitError::OperationFailed {
        operation: "interactive_rebase_start".to_string(),
        details: format!("Failed to execute git rebase -i: {e}"),
    })?;

    let _ = fs::remove_dir_all(&temp_dir);

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        return Ok(if stdout.is_empty() {
            "交互式变基已启动".to_string()
        } else {
            stdout
        });
    }

    if is_rebase_in_progress(repo) {
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let message = if !stderr.is_empty() { stderr } else { stdout };
        return Ok(if message.is_empty() {
            "交互式变基已进入待继续状态".to_string()
        } else {
            message
        });
    }

    Err(GitError::OperationFailed {
        operation: "interactive_rebase_start".to_string(),
        details: format!(
            "git rebase -i failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ),
    })
}

/// Continue a rebase after resolving conflicts
pub fn rebase_continue(repo: &Repository) -> Result<RebaseResult, GitError> {
    info!("Continuing rebase");

    let repo_path = repo.command_cwd();

    // First add the resolved files
    let add_output = Command::new("git")
        .args(["add", "-A"])
        .current_dir(&repo_path)
        .output()
        .map_err(|e| GitError::OperationFailed {
            operation: "rebase_continue".to_string(),
            details: format!("Failed to execute git add: {}", e),
        })?;

    if !add_output.status.success() {
        return Err(GitError::OperationFailed {
            operation: "rebase_continue".to_string(),
            details: format!(
                "git add failed: {}",
                String::from_utf8_lossy(&add_output.stderr)
            ),
        });
    }

    // Then continue the rebase
    let output = Command::new("git")
        .args(["rebase", "--continue"])
        .current_dir(&repo_path)
        .output()
        .map_err(|e| GitError::OperationFailed {
            operation: "rebase_continue".to_string(),
            details: format!("Failed to execute git rebase: {}", e),
        })?;

    let result = RebaseResult {
        success: output.status.success(),
        message: command_result_message(&output),
    };

    if !output.status.success() {
        info!("Rebase continue failed: {}", result.message);
    } else {
        info!("Rebase continued successfully");
    }

    Ok(result)
}

/// Abort the current rebase
pub fn rebase_abort(repo: &Repository) -> Result<(), GitError> {
    info!("Aborting rebase");

    let repo_path = repo.command_cwd();

    let output = Command::new("git")
        .args(["rebase", "--abort"])
        .current_dir(&repo_path)
        .output()
        .map_err(|e| GitError::OperationFailed {
            operation: "rebase_abort".to_string(),
            details: format!("Failed to execute git rebase: {}", e),
        })?;

    if !output.status.success() {
        return Err(GitError::OperationFailed {
            operation: "rebase_abort".to_string(),
            details: format!(
                "git rebase --abort failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ),
        });
    }

    info!("Rebase aborted successfully");
    Ok(())
}

/// Skip the current commit during rebase
pub fn rebase_skip(repo: &Repository) -> Result<RebaseResult, GitError> {
    info!("Skipping current commit during rebase");

    let repo_path = repo.command_cwd();

    let output = Command::new("git")
        .args(["rebase", "--skip"])
        .current_dir(&repo_path)
        .output()
        .map_err(|e| GitError::OperationFailed {
            operation: "rebase_skip".to_string(),
            details: format!("Failed to execute git rebase: {}", e),
        })?;

    let result = RebaseResult {
        success: output.status.success(),
        message: command_result_message(&output),
    };

    if !output.status.success() {
        info!("Rebase skip failed: {}", result.message);
    } else {
        info!("Rebase skipped successfully");
    }

    Ok(result)
}

/// Get the current rebase status
pub fn get_rebase_status(repo: &Repository) -> Result<Option<RebaseStatus>, GitError> {
    let git_dir = git_dir(repo);

    // Check if we're in a rebase
    let rebase_merge = git_dir.join("rebase-merge");
    let rebase_apply = git_dir.join("rebase-apply");

    let is_rebasing = rebase_merge.exists() || rebase_apply.exists();

    if !is_rebasing {
        return Ok(None);
    }

    // Get the current step info
    let step_file = rebase_merge.join("msgnum");
    let last_file = rebase_merge.join("last");
    let end_file = rebase_merge.join("end");

    let total_file = if last_file.exists() {
        last_file
    } else {
        end_file
    };

    let (current, total) = if step_file.exists() && total_file.exists() {
        let step = std::fs::read_to_string(&step_file)
            .map(|s| s.trim().parse::<u32>().unwrap_or(1))
            .unwrap_or(1);
        let last = std::fs::read_to_string(&total_file)
            .map(|s| s.trim().parse::<u32>().unwrap_or(1))
            .unwrap_or(1);
        (step, last)
    } else {
        (0, 0)
    };

    Ok(Some(RebaseStatus {
        is_interactive: rebase_merge.exists(),
        current_step: current,
        total_steps: total,
        progress: if total > 0 {
            current as f32 / total as f32
        } else {
            0.0
        },
    }))
}

pub fn get_rebase_todo(repo: &Repository) -> Result<Vec<RebaseTodoEntry>, GitError> {
    let git_dir = git_dir(repo);
    let todo_path = git_dir.join("rebase-merge").join("git-rebase-todo");
    if !todo_path.exists() {
        return Ok(Vec::new());
    }

    let contents = fs::read_to_string(&todo_path).map_err(|e| GitError::OperationFailed {
        operation: "get_rebase_todo".to_string(),
        details: format!("读取 git-rebase-todo 失败: {e}"),
    })?;

    Ok(contents.lines().filter_map(parse_todo_line).collect())
}

pub fn get_current_rebase_step(repo: &Repository) -> Result<Option<RebaseTodoEntry>, GitError> {
    let git_dir = git_dir(repo);
    read_last_todo_entry(&git_dir.join("rebase-merge").join("done"))
}

/// Check if there are rebase conflicts
pub fn has_rebase_conflicts(repo: &Repository) -> Result<bool, GitError> {
    let repo_path = repo.command_cwd();

    // Check for conflict markers in the index
    let output = Command::new("git")
        .args(["diff", "--name-only", "--diff-filter=U"])
        .current_dir(&repo_path)
        .output()
        .map_err(|e| GitError::OperationFailed {
            operation: "has_rebase_conflicts".to_string(),
            details: format!("Failed to execute git diff: {}", e),
        })?;

    let conflicted_files = String::from_utf8_lossy(&output.stdout);
    let has_conflicts = !conflicted_files.trim().is_empty();

    Ok(has_conflicts)
}

/// Rebase status information
#[derive(Debug, Clone)]
pub struct RebaseStatus {
    pub is_interactive: bool,
    pub current_step: u32,
    pub total_steps: u32,
    pub progress: f32,
}

#[cfg(test)]
mod tests {
    use super::{
        build_todo_contents, get_current_rebase_step, get_rebase_status, get_rebase_todo,
        is_rebase_in_progress, prepare_interactive_rebase_plan, rebase_abort,
        start_interactive_rebase, RebaseTodoEntry,
    };
    use crate::error::GitError;
    use crate::repository::Repository;
    use std::fs;
    use std::path::Path;
    use std::process::Command;
    use tempfile::TempDir;

    fn run_git(repo_path: &Path, args: &[&str]) {
        let output = Command::new("git")
            .args(args)
            .current_dir(repo_path)
            .output()
            .expect("run git command");
        assert!(
            output.status.success(),
            "git {:?} failed:\nstdout: {}\nstderr: {}",
            args,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    fn git_stdout(repo_path: &Path, args: &[&str]) -> String {
        let output = Command::new("git")
            .args(args)
            .current_dir(repo_path)
            .output()
            .expect("run git command");
        assert!(
            output.status.success(),
            "git {:?} failed:\nstdout: {}\nstderr: {}",
            args,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8_lossy(&output.stdout).trim().to_string()
    }

    fn create_commit(repo_path: &Path, file_name: &str, contents: &str, message: &str) {
        fs::write(repo_path.join(file_name), contents).expect("write file");
        run_git(repo_path, &["add", file_name]);
        run_git(repo_path, &["commit", "-m", message]);
    }

    fn create_linear_history_repo() -> (Repository, TempDir, Vec<String>) {
        let temp_dir = TempDir::new().expect("temp dir");
        run_git(temp_dir.path(), &["init"]);
        run_git(temp_dir.path(), &["config", "user.name", "slio-git"]);
        run_git(
            temp_dir.path(),
            &["config", "user.email", "slio-git@example.com"],
        );

        create_commit(temp_dir.path(), "notes.txt", "one\n", "first");
        create_commit(temp_dir.path(), "notes.txt", "one\ntwo\n", "second");
        create_commit(temp_dir.path(), "notes.txt", "one\ntwo\nthree\n", "third");

        let repo = Repository::discover(temp_dir.path()).expect("discover repo");
        let commits = git_stdout(temp_dir.path(), &["rev-list", "--reverse", "HEAD"])
            .lines()
            .map(ToOwned::to_owned)
            .collect();

        (repo, temp_dir, commits)
    }

    #[test]
    fn build_todo_contents_translates_reword_and_blocks_invalid_first_actions() {
        let contents = build_todo_contents(&[RebaseTodoEntry {
            action: "reword".to_string(),
            commit: "abc123".to_string(),
            message: "rename commit".to_string(),
        }])
        .expect("build todo");

        assert_eq!(contents, "edit abc123 rename commit\n");

        let error = build_todo_contents(&[RebaseTodoEntry {
            action: "squash".to_string(),
            commit: "abc123".to_string(),
            message: "rename commit".to_string(),
        }])
        .expect_err("first squash should be rejected");

        match error {
            GitError::OperationFailed { details, .. } => {
                assert!(details.contains("首条 todo 不能直接使用 fixup 或 squash"));
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn prepare_interactive_rebase_plan_uses_selected_commit_as_tail_boundary() {
        let (repo, _temp_dir, commits) = create_linear_history_repo();
        let plan =
            prepare_interactive_rebase_plan(&repo, &commits[1]).expect("prepare interactive plan");

        assert_eq!(plan.base_ref.as_deref(), Some(commits[0].as_str()));
        assert_eq!(plan.start_commit, commits[1]);
        assert_eq!(plan.entries.len(), 2);
        assert_eq!(plan.entries[0].commit, commits[1]);
        assert_eq!(plan.entries[1].commit, commits[2]);
        assert!(plan.entries.iter().all(|entry| entry.action == "pick"));
    }

    #[test]
    fn start_interactive_rebase_exposes_current_step_and_remaining_todo() {
        let (repo, _temp_dir, commits) = create_linear_history_repo();
        let entries = vec![
            RebaseTodoEntry {
                action: "reword".to_string(),
                commit: commits[1].clone(),
                message: "second".to_string(),
            },
            RebaseTodoEntry {
                action: "pick".to_string(),
                commit: commits[2].clone(),
                message: "third".to_string(),
            },
        ];

        start_interactive_rebase(&repo, Some(&commits[0]), &entries)
            .expect("start interactive rebase");

        assert!(is_rebase_in_progress(&repo));

        let status = get_rebase_status(&repo)
            .expect("read rebase status")
            .expect("status should exist");
        assert!(status.is_interactive);
        assert_eq!(status.current_step, 1);
        assert_eq!(status.total_steps, 2);

        let current_step = get_current_rebase_step(&repo)
            .expect("read current step")
            .expect("current step should exist");
        assert_eq!(current_step.action, "edit");
        assert_eq!(current_step.commit, commits[1]);
        assert_eq!(current_step.message, "second");

        let remaining = get_rebase_todo(&repo).expect("read remaining todo");
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].commit, commits[2]);
        assert_eq!(remaining[0].action, "pick");

        rebase_abort(&repo).expect("abort rebase");
        assert!(!is_rebase_in_progress(&repo));
    }
}
