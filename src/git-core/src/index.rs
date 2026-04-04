//! Index (staging area) operations for git-core

use crate::error::GitError;
use crate::repository::Repository;
use git2::{DiffOptions, StatusOptions};
use log::info;
use std::path::Path;

/// Status of a file change
#[derive(Debug, Clone, PartialEq)]
pub enum ChangeStatus {
    Modified,
    Added,
    Deleted,
    Renamed,
    Untracked,
    Ignored,
    Conflict,
}

/// A file change
#[derive(Debug, Clone)]
pub struct Change {
    pub path: String,
    pub status: ChangeStatus,
    pub staged: bool,
    pub unstaged: bool,
    pub old_oid: Option<String>,
    pub new_oid: Option<String>,
    /// Whether this change is a submodule entry
    pub is_submodule: bool,
    /// Commit range summary for submodule changes (e.g., "abc1234..def5678")
    pub submodule_summary: Option<String>,
}

/// An entry in the index
#[derive(Debug, Clone)]
pub struct IndexEntry {
    pub path: String,
    pub oid: String,
    pub mode: u32,
    pub stage: u32,
}

/// Get the current index
pub fn get_index(repo: &Repository) -> Result<Index, GitError> {
    let repo_lock = repo.inner.read().unwrap();
    let index = repo_lock.index().map_err(|e| GitError::OperationFailed {
        operation: "get_index".to_string(),
        details: e.to_string(),
    })?;

    Ok(Index { inner: index })
}

/// Stage a file
pub fn stage_file(repo: &Repository, path: &Path) -> Result<(), GitError> {
    info!("Staging file: {:?}", path);

    let repo_lock = repo.inner.write().unwrap();
    let mut index = repo_lock.index().map_err(|e| GitError::OperationFailed {
        operation: "stage_file".to_string(),
        details: e.to_string(),
    })?;

    index
        .add_path(path)
        .map_err(|e| GitError::OperationFailed {
            operation: "stage_file".to_string(),
            details: e.to_string(),
        })?;

    index.write().map_err(|e| GitError::OperationFailed {
        operation: "stage_file".to_string(),
        details: e.to_string(),
    })?;

    Ok(())
}

/// Unstage a file by resetting HEAD to the given path
pub fn unstage_file(repo: &Repository, path: &Path) -> Result<(), GitError> {
    info!("Unstaging file: {:?}", path);

    let repo_lock = repo.inner.read().unwrap();

    // Get the HEAD commit's tree for the path
    let head = repo_lock.head().map_err(|e| GitError::OperationFailed {
        operation: "unstage_file".to_string(),
        details: e.to_string(),
    })?;

    let commit = head
        .peel_to_commit()
        .map_err(|e| GitError::OperationFailed {
            operation: "unstage_file".to_string(),
            details: e.to_string(),
        })?;

    let tree = commit.tree().map_err(|e| GitError::OperationFailed {
        operation: "unstage_file".to_string(),
        details: e.to_string(),
    })?;

    // Reset the index entry to match the HEAD tree
    let mut index = repo_lock.index().map_err(|e| GitError::OperationFailed {
        operation: "unstage_file".to_string(),
        details: e.to_string(),
    })?;

    // If file exists in HEAD tree, we need to restore it to index from HEAD
    if let Ok(entry) = tree.get_path(path) {
        // Remove the staged version
        index
            .remove_path(path)
            .map_err(|e| GitError::OperationFailed {
                operation: "unstage_file".to_string(),
                details: e.to_string(),
            })?;

        // Create an IndexEntry from the tree entry and add it back
        let id = entry.id();
        let mode = entry.filemode();
        let path_str = path.to_string_lossy().to_string();

        // Create IndexEntry manually for git2 0.19
        let index_entry = git2::IndexEntry {
            dev: 0,
            ino: 0,
            id,
            mode: mode as u32,
            uid: 0,
            gid: 0,
            file_size: 0,
            mtime: git2::IndexTime::new(0, 0),
            ctime: git2::IndexTime::new(0, 0),
            path: path_str.into_bytes(),
            flags: 0,
            flags_extended: 0,
        };

        index
            .add(&index_entry)
            .map_err(|e| GitError::OperationFailed {
                operation: "unstage_file".to_string(),
                details: e.to_string(),
            })?;
    } else {
        // File didn't exist in HEAD, just remove from index
        index.remove_path(path).ok(); // Ignore error if not in index
    }

    index.write().map_err(|e| GitError::OperationFailed {
        operation: "unstage_file".to_string(),
        details: e.to_string(),
    })?;

    Ok(())
}

/// Stage all files
pub fn stage_all(repo: &Repository) -> Result<(), GitError> {
    info!("Staging all files");

    let repo_lock = repo.inner.write().unwrap();
    let mut index = repo_lock.index().map_err(|e| GitError::OperationFailed {
        operation: "stage_all".to_string(),
        details: e.to_string(),
    })?;

    index
        .add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)
        .map_err(|e| GitError::OperationFailed {
            operation: "stage_all".to_string(),
            details: e.to_string(),
        })?;

    index.write().map_err(|e| GitError::OperationFailed {
        operation: "stage_all".to_string(),
        details: e.to_string(),
    })?;

    Ok(())
}

/// Unstage all files
pub fn unstage_all(repo: &Repository) -> Result<(), GitError> {
    use std::process::Command;

    let repo_path = repo.command_cwd();

    // Run git reset HEAD -- .
    let output = Command::new("git")
        .args(["reset", "HEAD", "--", "."])
        .current_dir(&repo_path)
        .output()
        .map_err(|e| GitError::OperationFailed {
            operation: "unstage_all".to_string(),
            details: format!("Failed to execute git reset: {}", e),
        })?;

    if !output.status.success() {
        return Err(GitError::OperationFailed {
            operation: "unstage_all".to_string(),
            details: format!(
                "git reset failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ),
        });
    }

    Ok(())
}

/// Convert git2 status flags to our ChangeStatus
fn convert_status(status: git2::Status) -> ChangeStatus {
    use git2::Status;
    if status.intersects(Status::INDEX_NEW) {
        ChangeStatus::Added
    } else if status.intersects(Status::INDEX_MODIFIED) {
        ChangeStatus::Modified
    } else if status.intersects(Status::INDEX_DELETED) {
        ChangeStatus::Deleted
    } else if status.intersects(Status::INDEX_RENAMED) {
        ChangeStatus::Renamed
    } else if status.intersects(Status::WT_NEW) {
        ChangeStatus::Untracked
    } else if status.intersects(Status::WT_MODIFIED) {
        ChangeStatus::Modified
    } else if status.intersects(Status::WT_DELETED) {
        ChangeStatus::Deleted
    } else if status.intersects(Status::CONFLICTED) {
        ChangeStatus::Conflict
    } else if status.intersects(Status::IGNORED) {
        ChangeStatus::Ignored
    } else {
        ChangeStatus::Modified
    }
}

fn has_staged_status(status: git2::Status) -> bool {
    status.intersects(
        git2::Status::INDEX_NEW
            | git2::Status::INDEX_MODIFIED
            | git2::Status::INDEX_DELETED
            | git2::Status::INDEX_RENAMED
            | git2::Status::INDEX_TYPECHANGE,
    )
}

fn has_unstaged_status(status: git2::Status) -> bool {
    status.intersects(
        git2::Status::WT_MODIFIED
            | git2::Status::WT_DELETED
            | git2::Status::WT_RENAMED
            | git2::Status::WT_TYPECHANGE
            | git2::Status::WT_NEW,
    )
}

/// Get file status changes
pub fn get_status(repo: &Repository) -> Result<Vec<Change>, GitError> {
    let repo_lock = repo.inner.read().unwrap();

    let mut status_options = StatusOptions::new();
    status_options.include_untracked(true);
    status_options.recurse_untracked_dirs(true);

    let statuses =
        repo_lock
            .statuses(Some(&mut status_options))
            .map_err(|e| GitError::OperationFailed {
                operation: "get_status".to_string(),
                details: e.to_string(),
            })?;

    let mut changes = Vec::new();

    for entry in statuses.iter() {
        let status = entry.status();

        // Skip ignored files
        if status.intersects(git2::Status::IGNORED) {
            continue;
        }

        let file_path = entry.path().unwrap_or("").to_string();
        let is_submodule = entry.status().intersects(git2::Status::WT_TYPECHANGE)
            || crate::submodule::is_submodule(repo, &file_path);
        let submodule_summary = if is_submodule {
            crate::submodule::submodule_summary(repo, &file_path)
        } else {
            None
        };

        changes.push(Change {
            path: file_path,
            status: convert_status(status),
            staged: has_staged_status(status),
            unstaged: has_unstaged_status(status),
            old_oid: None,
            new_oid: None,
            is_submodule,
            submodule_summary,
        });
    }

    Ok(changes)
}

/// Check if repository has merge conflicts
pub fn has_conflicts(repo: &Repository) -> bool {
    let changes = get_status(repo).ok();
    changes
        .map(|c| c.iter().any(|ch| ch.status == ChangeStatus::Conflict))
        .unwrap_or(false)
}

/// Get list of conflicted files
pub fn get_conflicted_files(repo: &Repository) -> Result<Vec<String>, GitError> {
    let changes = get_status(repo)?;
    Ok(changes
        .into_iter()
        .filter(|c| c.status == ChangeStatus::Conflict)
        .map(|c| c.path)
        .collect())
}

/// The index/staging area
pub struct Index {
    inner: git2::Index,
}

impl Index {
    /// List entries in the index (simplified)
    pub fn list_entries(&self) -> Vec<IndexEntry> {
        self.inner
            .iter()
            .map(|entry| IndexEntry {
                path: String::from_utf8_lossy(&entry.path).to_string(),
                oid: entry.id.to_string(),
                mode: entry.mode,
                stage: ((entry.flags >> 12) & 0x3) as u32,
            })
            .collect()
    }
}

/// A hunk in a file diff
#[derive(Debug, Clone)]
pub struct Hunk {
    pub old_start: u32,
    pub old_lines: u32,
    pub new_start: u32,
    pub new_lines: u32,
    pub header: String,
    pub lines: Vec<HunkLine>,
}

/// A line in a hunk
#[derive(Debug, Clone)]
pub struct HunkLine {
    pub origin: char,
    pub content: String,
}

/// Get hunks for a specific file (between workdir and index)
pub fn get_file_hunks(repo: &Repository, file_path: &Path) -> Result<Vec<Hunk>, GitError> {
    let repo_lock = repo.inner.read().unwrap();

    let mut diff_opts = DiffOptions::new();
    diff_opts.pathspec(file_path);

    let diff = repo_lock
        .diff_index_to_workdir(None, Some(&mut diff_opts))
        .map_err(|e| GitError::OperationFailed {
            operation: "get_file_hunks".to_string(),
            details: e.to_string(),
        })?;

    let mut hunks = Vec::new();

    diff.print(git2::DiffFormat::Patch, |_delta, hunk, line| {
        if let Some(hunk) = hunk {
            let header = String::from_utf8_lossy(hunk.header()).to_string();
            let (old_start, old_lines, new_start, new_lines) = parse_hunk_header(&header);

            let mut hunk_lines = Vec::new();

            // Read the hunk lines
            let content = String::from_utf8_lossy(line.content()).to_string();
            let origin = line.origin();
            hunk_lines.push(HunkLine {
                origin,
                content: content.clone(),
            });

            hunks.push(Hunk {
                old_start,
                old_lines,
                new_start,
                new_lines,
                header,
                lines: hunk_lines,
            });
        } else {
            // Add line to the last hunk
            if let Some(last_hunk) = hunks.last_mut() {
                let content = String::from_utf8_lossy(line.content()).to_string();
                let origin = line.origin();
                last_hunk.lines.push(HunkLine { origin, content });
            }
        }
        true
    })
    .map_err(|e| GitError::OperationFailed {
        operation: "get_file_hunks".to_string(),
        details: e.to_string(),
    })?;

    Ok(hunks)
}

/// Parse hunk header to extract line information
fn parse_hunk_header(header: &str) -> (u32, u32, u32, u32) {
    // Format: "@@ -old_start[,old_lines] +new_start[,new_lines] @@"
    let parts: Vec<&str> = header.split_whitespace().collect();
    if parts.len() < 2 {
        return (0, 0, 0, 0);
    }

    let old_part = parts.get(1).unwrap_or(&"");
    let new_part = parts.get(2).unwrap_or(&"");

    let (old_start, old_lines) = parse_range_part(old_part);
    let (new_start, new_lines) = parse_range_part(new_part);

    (old_start, old_lines, new_start, new_lines)
}

/// Parse a range part like "-1,5" or "-1"
fn parse_range_part(part: &str) -> (u32, u32) {
    let s = part.trim_start_matches('-').trim_start_matches('+');
    let nums: Vec<&str> = s.split(',').collect();

    let start = nums.first().and_then(|n| n.parse().ok()).unwrap_or(0);
    let lines = nums.get(1).and_then(|n| n.parse().ok()).unwrap_or(1);

    (start, lines)
}

/// Stage a specific hunk of a file
pub fn stage_hunk(repo: &Repository, file_path: &Path, hunk_index: usize) -> Result<(), GitError> {
    info!("Staging hunk {} of file: {:?}", hunk_index, file_path);

    // Get the diff for this file
    let hunks = get_file_hunks(repo, file_path)?;

    let hunk = hunks
        .get(hunk_index)
        .ok_or_else(|| GitError::OperationFailed {
            operation: "stage_hunk".to_string(),
            details: format!("Hunk {} not found", hunk_index),
        })?;

    // Generate a patch for just this hunk
    let patch = generate_hunk_patch(file_path, hunk)?;

    // Apply the patch to the index using git apply --cached
    apply_patch_cached(repo, &patch)?;

    Ok(())
}

/// Generate a patch string for a single hunk
fn generate_hunk_patch(file_path: &Path, hunk: &Hunk) -> Result<String, GitError> {
    let mut patch = String::new();

    // Add the hunk header
    patch.push_str(&format!(
        "diff --git a/{} b/{}\n",
        file_path.to_string_lossy(),
        file_path.to_string_lossy()
    ));
    patch.push_str(&format!("--- a/{}\n", file_path.to_string_lossy()));
    patch.push_str(&format!("+++ b/{}\n", file_path.to_string_lossy()));
    patch.push_str(&hunk.header);
    patch.push('\n');

    // Add the hunk lines
    for line in &hunk.lines {
        patch.push(line.origin);
        patch.push_str(&line.content);
        if !line.content.ends_with('\n') {
            patch.push('\n');
        }
    }

    Ok(patch)
}

/// Apply a patch to the index using git apply --cached
fn apply_patch_cached(repo: &Repository, patch: &str) -> Result<(), GitError> {
    use std::process::Command;

    let repo_path = repo.command_cwd();

    // Write patch to a temporary file
    let mut temp_path = std::env::temp_dir();
    temp_path.push(format!("patch_{}_{}", std::process::id(), std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()));

    std::fs::write(&temp_path, patch).map_err(|e| GitError::OperationFailed {
        operation: "apply_patch_cached".to_string(),
        details: format!("Failed to write patch file: {}", e),
    })?;

    // Run git apply --cached
    let output = Command::new("git")
        .args(["apply", "--cached", "--unidiff-zero", "--whitespace=nowarn"])
        .arg(temp_path.as_path())
        .current_dir(&repo_path)
        .output()
        .map_err(|e| GitError::OperationFailed {
            operation: "apply_patch_cached".to_string(),
            details: format!("Failed to execute git apply: {}", e),
        })?;

    // Clean up temp file
    let _ = std::fs::remove_file(&temp_path);

    if !output.status.success() {
        return Err(GitError::OperationFailed {
            operation: "apply_patch_cached".to_string(),
            details: format!(
                "git apply failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ),
        });
    }

    Ok(())
}

/// Unstage a hunk (move changes back to workdir)
/// Uses git checkout HEAD -- file to reset the specific file, then re-apply other hunks
pub fn unstage_hunk(
    repo: &Repository,
    file_path: &Path,
    hunk_index: usize,
) -> Result<(), GitError> {
    info!("Unstaging hunk {} of file: {:?}", hunk_index, file_path);

    // Get the diff between index and HEAD for this file
    let hunks = get_index_hunks(repo, file_path)?;

    let hunk = hunks
        .get(hunk_index)
        .ok_or_else(|| GitError::OperationFailed {
            operation: "unstage_hunk".to_string(),
            details: format!("Hunk {} not found in index", hunk_index),
        })?;

    // Generate a reverse patch for this hunk
    let patch = generate_reverse_hunk_patch(file_path, hunk)?;

    // Apply the reverse patch to the workdir using git apply
    apply_patch_workdir(repo, &patch)?;

    // Also reset the file in the index but keep other hunks staged
    // This is a simplified approach - complex but correct unstage is very difficult
    reset_file_in_index(repo, file_path)?;

    // Re-stage the other hunks
    re_stage_other_hunks(repo, file_path, hunk_index)?;

    Ok(())
}

/// Get hunks between index and HEAD for a specific file
fn get_index_hunks(repo: &Repository, file_path: &Path) -> Result<Vec<Hunk>, GitError> {
    let mut hunks = Vec::new();

    {
        let repo_lock = repo.inner.read().unwrap();

        let head = repo_lock.head().map_err(|e| GitError::OperationFailed {
            operation: "get_index_hunks".to_string(),
            details: e.to_string(),
        })?;

        let commit = head
            .peel_to_commit()
            .map_err(|e| GitError::OperationFailed {
                operation: "get_index_hunks".to_string(),
                details: e.to_string(),
            })?;

        let head_tree = commit.tree().map_err(|e| GitError::OperationFailed {
            operation: "get_index_hunks".to_string(),
            details: e.to_string(),
        })?;

        let mut diff_opts = DiffOptions::new();
        diff_opts.pathspec(file_path);

        let diff = repo_lock
            .diff_tree_to_index(Some(&head_tree), None, Some(&mut diff_opts))
            .map_err(|e| GitError::OperationFailed {
                operation: "get_index_hunks".to_string(),
                details: e.to_string(),
            })?;

        // Process diff within the lock scope
        let mut current_hunk: Option<(String, u32, u32, u32, u32, Vec<HunkLine>)> = None;

        diff.print(git2::DiffFormat::Patch, |_delta, hunk, line| {
            if let Some(hunk) = hunk {
                // Save previous hunk if exists
                if let Some((header, old_start, old_lines, new_start, new_lines, lines)) =
                    current_hunk.take()
                {
                    hunks.push(Hunk {
                        header,
                        old_start,
                        old_lines,
                        new_start,
                        new_lines,
                        lines,
                    });
                }

                let header = String::from_utf8_lossy(hunk.header()).to_string();
                let (old_start, old_lines, new_start, new_lines) = parse_hunk_header(&header);

                let content = String::from_utf8_lossy(line.content()).to_string();
                let origin = line.origin();
                let hunk_lines = vec![HunkLine { origin, content }];

                current_hunk = Some((
                    header, old_start, old_lines, new_start, new_lines, hunk_lines,
                ));
            } else if let Some((header, old_start, old_lines, new_start, new_lines, mut lines)) =
                current_hunk.take()
            {
                let content = String::from_utf8_lossy(line.content()).to_string();
                let origin = line.origin();
                lines.push(HunkLine { origin, content });
                current_hunk = Some((header, old_start, old_lines, new_start, new_lines, lines));
            }
            true
        })
        .map_err(|e| GitError::OperationFailed {
            operation: "get_index_hunks".to_string(),
            details: e.to_string(),
        })?;

        // Save last hunk
        if let Some((header, old_start, old_lines, new_start, new_lines, lines)) = current_hunk {
            hunks.push(Hunk {
                header,
                old_start,
                old_lines,
                new_start,
                new_lines,
                lines,
            });
        }
    }

    Ok(hunks)
}

/// Generate a reverse patch for unstaking (addition becomes deletion)
fn generate_reverse_hunk_patch(file_path: &Path, hunk: &Hunk) -> Result<String, GitError> {
    let mut patch = String::new();

    patch.push_str(&format!(
        "diff --git a/{} b/{}\n",
        file_path.to_string_lossy(),
        file_path.to_string_lossy()
    ));
    patch.push_str(&format!("--- a/{}\n", file_path.to_string_lossy()));
    patch.push_str(&format!("+++ b/{}\n", file_path.to_string_lossy()));
    patch.push_str(&hunk.header);
    patch.push('\n');

    // Reverse the hunk lines
    for line in &hunk.lines {
        let reversed_origin = match line.origin {
            '+' => '-',
            '-' => '+',
            c => c,
        };
        patch.push(reversed_origin);
        patch.push_str(&line.content);
        if !line.content.ends_with('\n') {
            patch.push('\n');
        }
    }

    Ok(patch)
}

/// Apply a patch to the workdir using git apply
fn apply_patch_workdir(repo: &Repository, patch: &str) -> Result<(), GitError> {
    use std::process::Command;

    let repo_path = repo.command_cwd();

    // Write patch to a temporary file
    let mut temp_path = std::env::temp_dir();
    temp_path.push(format!("patch_{}_{}", std::process::id(), std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()));

    std::fs::write(&temp_path, patch).map_err(|e| GitError::OperationFailed {
        operation: "apply_patch_workdir".to_string(),
        details: format!("Failed to write patch file: {}", e),
    })?;

    // Run git apply (not --cached, applies to workdir)
    let output = Command::new("git")
        .args(["apply", "--unidiff-zero", "--whitespace=nowarn"])
        .arg(temp_path.as_path())
        .current_dir(&repo_path)
        .output()
        .map_err(|e| GitError::OperationFailed {
            operation: "apply_patch_workdir".to_string(),
            details: format!("Failed to execute git apply: {}", e),
        })?;

    // Clean up temp file
    let _ = std::fs::remove_file(&temp_path);

    if !output.status.success() {
        return Err(GitError::OperationFailed {
            operation: "apply_patch_workdir".to_string(),
            details: format!(
                "git apply failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ),
        });
    }

    Ok(())
}

/// Reset a file in the index to HEAD state
fn reset_file_in_index(repo: &Repository, file_path: &Path) -> Result<(), GitError> {
    use std::process::Command;

    let repo_path = repo.command_cwd();

    // Run git reset HEAD -- file_path
    let output = Command::new("git")
        .args(["reset", "HEAD", "--"])
        .arg(file_path)
        .current_dir(&repo_path)
        .output()
        .map_err(|e| GitError::OperationFailed {
            operation: "reset_file_in_index".to_string(),
            details: format!("Failed to execute git reset: {}", e),
        })?;

    if !output.status.success() {
        return Err(GitError::OperationFailed {
            operation: "reset_file_in_index".to_string(),
            details: format!(
                "git reset failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ),
        });
    }

    Ok(())
}

/// Re-stage hunks except the one at hunk_index
fn re_stage_other_hunks(
    repo: &Repository,
    file_path: &Path,
    skip_hunk_index: usize,
) -> Result<(), GitError> {
    // Get workdir hunks and re-stage all except the skipped one
    let workdir_hunks = get_file_hunks(repo, file_path)?;

    for (i, hunk) in workdir_hunks.iter().enumerate() {
        if i != skip_hunk_index {
            let patch = generate_hunk_patch(file_path, hunk)?;
            apply_patch_cached(repo, &patch)?;
        }
    }

    Ok(())
}

/// Discard changes for a file: reset both index and worktree to HEAD.
/// For untracked files, removes the file from the working directory.
pub fn discard_file(repo: &Repository, file_path: &Path) -> Result<(), GitError> {
    use std::process::Command;

    let repo_path = repo.command_cwd();
    let full_path = repo_path.join(file_path);

    // Check if file is tracked
    let tracked = {
        let repo_lock = repo.inner.read().unwrap();
        repo_lock
            .revparse_single("HEAD")
            .ok()
            .and_then(|obj| obj.peel_to_commit().ok())
            .and_then(|commit| commit.tree().ok())
            .map(|tree| tree.get_path(file_path).is_ok())
            .unwrap_or(false)
    };

    if tracked {
        // git checkout HEAD -- file_path (restore to HEAD in both index and worktree)
        let output = Command::new("git")
            .args(["checkout", "HEAD", "--"])
            .arg(file_path)
            .current_dir(&repo_path)
            .output()
            .map_err(|e| GitError::OperationFailed {
                operation: "discard_file".to_string(),
                details: format!("Failed to execute git checkout: {}", e),
            })?;

        if !output.status.success() {
            return Err(GitError::OperationFailed {
                operation: "discard_file".to_string(),
                details: format!(
                    "git checkout failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                ),
            });
        }
    } else {
        // Untracked: remove file/directory
        if full_path.is_dir() {
            std::fs::remove_dir_all(&full_path).map_err(|e| GitError::OperationFailed {
                operation: "discard_file".to_string(),
                details: format!("Failed to remove directory: {}", e),
            })?;
        } else {
            std::fs::remove_file(&full_path).map_err(|e| GitError::OperationFailed {
                operation: "discard_file".to_string(),
                details: format!("Failed to remove file: {}", e),
            })?;
        }
    }

    Ok(())
}
