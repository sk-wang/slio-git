//! Diff operations for git-core

use crate::error::GitError;
use crate::repository::Repository;
use git2::{Diff as Git2Diff, DiffOptions};
use std::cell::{Cell, RefCell};
use std::path::Path;

/// Origin of a diff line
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffLineOrigin {
    Context,
    Addition,
    Deletion,
    Header,
    HunkHeader,
}

impl From<char> for DiffLineOrigin {
    fn from(c: char) -> Self {
        match c {
            '+' => DiffLineOrigin::Addition,
            '-' => DiffLineOrigin::Deletion,
            ' ' => DiffLineOrigin::Context,
            '@' => DiffLineOrigin::HunkHeader,
            _ => DiffLineOrigin::Header,
        }
    }
}

/// A span within a diff line marking character-level changes.
/// Used for GitHub-style inline highlighting of specific changed characters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InlineChangeSpan {
    /// Byte offset in the line content
    pub start: usize,
    /// Byte length of the span
    pub len: usize,
    /// Whether this span represents a change (true) or unchanged text (false)
    pub changed: bool,
}

/// A single line in a diff
#[derive(Debug, Clone)]
pub struct DiffLine {
    pub content: String,
    pub origin: DiffLineOrigin,
    pub old_lineno: Option<u32>,
    pub new_lineno: Option<u32>,
    /// Character-level change spans within this line (empty = whole line is one span)
    pub inline_changes: Vec<InlineChangeSpan>,
}

/// A hunk in a diff
#[derive(Debug, Clone)]
pub struct DiffHunk {
    pub header: String,
    pub lines: Vec<DiffLine>,
    pub old_start: u32,
    pub old_lines: u32,
    pub new_start: u32,
    pub new_lines: u32,
}

/// A complete diff for a file
#[derive(Debug, Clone)]
pub struct FileDiff {
    pub old_path: Option<String>,
    pub new_path: Option<String>,
    pub hunks: Vec<DiffHunk>,
    pub additions: u32,
    pub deletions: u32,
}

/// A collection of file diffs
#[derive(Debug, Clone)]
pub struct Diff {
    pub files: Vec<FileDiff>,
    pub total_additions: u32,
    pub total_deletions: u32,
}

/// Get diff between working tree and index
pub fn diff_workdir_to_index(repo: &Repository) -> Result<Diff, GitError> {
    let repo_lock = repo.inner.read().unwrap();

    let mut diff_options = DiffOptions::new();
    diff_options.include_untracked(true);
    diff_options.show_untracked_content(true);

    let diff = repo_lock
        .diff_index_to_workdir(None, Some(&mut diff_options))
        .map_err(|e| GitError::OperationFailed {
            operation: "diff_workdir_to_index".to_string(),
            details: e.to_string(),
        })?;

    extract_diff_info(&diff)
}

/// Get diff between two commits
pub fn diff_commits(repo: &Repository, old_oid: &str, new_oid: &str) -> Result<Diff, GitError> {
    let repo_lock = repo.inner.read().unwrap();

    let old = git2::Oid::from_str(old_oid).map_err(|_| GitError::CommitNotFound {
        id: old_oid.to_string(),
    })?;
    let new = git2::Oid::from_str(new_oid).map_err(|_| GitError::CommitNotFound {
        id: new_oid.to_string(),
    })?;

    let old_commit = repo_lock
        .find_commit(old)
        .map_err(|_| GitError::CommitNotFound {
            id: old_oid.to_string(),
        })?;
    let new_commit = repo_lock
        .find_commit(new)
        .map_err(|_| GitError::CommitNotFound {
            id: new_oid.to_string(),
        })?;

    let old_tree = old_commit.tree().map_err(|e| GitError::OperationFailed {
        operation: "diff_commits".to_string(),
        details: e.to_string(),
    })?;
    let new_tree = new_commit.tree().map_err(|e| GitError::OperationFailed {
        operation: "diff_commits".to_string(),
        details: e.to_string(),
    })?;

    let diff = repo_lock
        .diff_tree_to_tree(Some(&old_tree), Some(&new_tree), None)
        .map_err(|e| GitError::OperationFailed {
            operation: "diff_commits".to_string(),
            details: e.to_string(),
        })?;

    extract_diff_info(&diff)
}

/// Get diff between two refs, branches, or other git revspecs.
pub fn diff_refs(repo: &Repository, old_ref: &str, new_ref: &str) -> Result<Diff, GitError> {
    let repo_lock = repo.inner.read().unwrap();
    let old_tree = resolve_tree_from_spec(&repo_lock, old_ref, "diff_refs")?;
    let new_tree = resolve_tree_from_spec(&repo_lock, new_ref, "diff_refs")?;

    let diff = repo_lock
        .diff_tree_to_tree(Some(&old_tree), Some(&new_tree), None)
        .map_err(|e| GitError::OperationFailed {
            operation: "diff_refs".to_string(),
            details: e.to_string(),
        })?;

    extract_diff_info(&diff)
}

/// Get diff between a ref and the current working tree, including staged changes.
pub fn diff_ref_to_workdir(repo: &Repository, ref_spec: &str) -> Result<Diff, GitError> {
    let repo_lock = repo.inner.read().unwrap();
    let tree = resolve_tree_from_spec(&repo_lock, ref_spec, "diff_ref_to_workdir")?;

    let mut diff_options = DiffOptions::new();
    diff_options.include_untracked(true);
    diff_options.show_untracked_content(true);

    let diff = repo_lock
        .diff_tree_to_workdir_with_index(Some(&tree), Some(&mut diff_options))
        .map_err(|e| GitError::OperationFailed {
            operation: "diff_ref_to_workdir".to_string(),
            details: e.to_string(),
        })?;

    extract_diff_info(&diff)
}

/// Get diff for a single file between working tree and index
pub fn diff_file_to_index(repo: &Repository, file_path: &Path) -> Result<Diff, GitError> {
    let repo_lock = repo.inner.read().unwrap();

    let mut diff_options = DiffOptions::new();
    diff_options.include_untracked(true);
    diff_options.recurse_untracked_dirs(true);
    diff_options.show_untracked_content(true);
    diff_options.pathspec(file_path);

    let diff = repo_lock
        .diff_index_to_workdir(None, Some(&mut diff_options))
        .map_err(|e| GitError::OperationFailed {
            operation: "diff_file_to_index".to_string(),
            details: e.to_string(),
        })?;

    extract_diff_info(&diff)
}

/// Get diff for a single file between index and HEAD
#[allow(dead_code)]
pub fn diff_index_to_head(repo: &Repository, file_path: &Path) -> Result<Diff, GitError> {
    let repo_lock = repo.inner.read().unwrap();

    // Get HEAD tree
    let head = repo_lock.head().map_err(|e| GitError::OperationFailed {
        operation: "diff_index_to_head".to_string(),
        details: e.to_string(),
    })?;

    let commit = head
        .peel_to_commit()
        .map_err(|e| GitError::OperationFailed {
            operation: "diff_index_to_head".to_string(),
            details: e.to_string(),
        })?;

    let head_tree = commit.tree().map_err(|e| GitError::OperationFailed {
        operation: "diff_index_to_head".to_string(),
        details: e.to_string(),
    })?;

    let mut diff_options = DiffOptions::new();
    diff_options.pathspec(file_path);

    // Compare HEAD tree to the current index so staged preview does not leak
    // unstaged worktree edits into the "already staged" diff.
    let diff = repo_lock
        .diff_tree_to_index(Some(&head_tree), None, Some(&mut diff_options))
        .map_err(|e| GitError::OperationFailed {
            operation: "diff_index_to_head".to_string(),
            details: e.to_string(),
        })?;

    extract_diff_info(&diff)
}

/// Extract diff information from a git2 Diff
fn extract_diff_info(diff: &Git2Diff) -> Result<Diff, GitError> {
    let files = RefCell::new(Vec::new());
    let total_additions = Cell::new(0u32);
    let total_deletions = Cell::new(0u32);

    diff.foreach(
        &mut |delta, _progress| {
            files.borrow_mut().push(FileDiff {
                old_path: delta
                    .old_file()
                    .path()
                    .map(|path| path.to_string_lossy().to_string()),
                new_path: delta
                    .new_file()
                    .path()
                    .map(|path| path.to_string_lossy().to_string()),
                hunks: Vec::new(),
                additions: 0,
                deletions: 0,
            });
            true
        },
        None,
        Some(&mut |_delta, hunk| {
            if let Some(file) = files.borrow_mut().last_mut() {
                file.hunks.push(DiffHunk {
                    header: String::from_utf8_lossy(hunk.header())
                        .trim_end_matches(['\r', '\n'])
                        .to_string(),
                    lines: Vec::new(),
                    old_start: hunk.old_start(),
                    old_lines: hunk.old_lines(),
                    new_start: hunk.new_start(),
                    new_lines: hunk.new_lines(),
                });
            }
            true
        }),
        Some(&mut |_delta, _hunk, line| {
            let origin = match line.origin() {
                '+' | '>' => DiffLineOrigin::Addition,
                '-' | '<' => DiffLineOrigin::Deletion,
                ' ' | '=' => DiffLineOrigin::Context,
                _ => return true,
            };

            if let Some(file) = files.borrow_mut().last_mut() {
                match origin {
                    DiffLineOrigin::Addition => {
                        total_additions.set(total_additions.get() + 1);
                        file.additions += 1;
                    }
                    DiffLineOrigin::Deletion => {
                        total_deletions.set(total_deletions.get() + 1);
                        file.deletions += 1;
                    }
                    DiffLineOrigin::Context
                    | DiffLineOrigin::Header
                    | DiffLineOrigin::HunkHeader => {}
                }

                if let Some(hunk) = file.hunks.last_mut() {
                    hunk.lines.push(DiffLine {
                        content: String::from_utf8_lossy(line.content())
                            .trim_end_matches(['\r', '\n'])
                            .to_string(),
                        origin,
                        old_lineno: line.old_lineno(),
                        new_lineno: line.new_lineno(),
                        inline_changes: Vec::new(),
                    });
                }
            }

            true
        }),
    )
    .map_err(|e| GitError::OperationFailed {
        operation: "extract_diff_info".to_string(),
        details: e.to_string(),
    })?;

    let mut result_files = files.into_inner();

    // Enhance each hunk with character-level inline change spans
    for file in &mut result_files {
        for hunk in &mut file.hunks {
            enhance_hunk_with_inline_changes(hunk);
        }
    }

    Ok(Diff {
        files: result_files,
        total_additions: total_additions.get(),
        total_deletions: total_deletions.get(),
    })
}

fn resolve_tree_from_spec<'a>(
    repo: &'a git2::Repository,
    spec: &str,
    operation: &str,
) -> Result<git2::Tree<'a>, GitError> {
    let object = repo
        .revparse_single(spec)
        .map_err(|_| GitError::CommitNotFound {
            id: spec.to_string(),
        })?;
    let commit = object
        .peel_to_commit()
        .map_err(|_| GitError::CommitNotFound {
            id: spec.to_string(),
        })?;

    commit.tree().map_err(|e| GitError::OperationFailed {
        operation: operation.to_string(),
        details: e.to_string(),
    })
}

// ============================================================================
// Three-way diff for merge conflicts
// ============================================================================

/// A single line in a three-way diff
#[derive(Debug, Clone)]
pub struct ConflictLine {
    pub base_line: Option<String>,
    pub ours_line: Option<String>,
    pub theirs_line: Option<String>,
    pub line_type: ConflictLineType,
}

/// Type of conflict line
#[derive(Debug, Clone, PartialEq)]
pub enum ConflictLineType {
    /// Lines are the same in all versions
    Unchanged,
    /// Lines only changed in ours
    OursOnly,
    /// Lines only changed in theirs
    TheirsOnly,
    /// Lines changed differently in ours and theirs
    Modified,
    /// Conflict markers (<<<<<<, ======, >>>>>>)
    ConflictMarker,
    /// Empty line
    Empty,
}

/// A hunk in a three-way diff
#[derive(Debug, Clone)]
pub struct ConflictHunk {
    pub base_start: u32,
    pub ours_start: u32,
    pub theirs_start: u32,
    pub base_lines: u32,
    pub ours_lines: u32,
    pub theirs_lines: u32,
    pub lines: Vec<ConflictLine>,
}

/// A complete three-way diff for a conflicted file
#[derive(Debug, Clone)]
pub struct ThreeWayDiff {
    pub path: String,
    pub hunks: Vec<ConflictHunk>,
    pub has_conflicts: bool,
    pub base_content: String,
    pub ours_content: String,
    pub theirs_content: String,
}

/// Get three-way diff content for a conflicted file
/// Reads base / ours / theirs blobs directly from the index conflict stages.
pub fn get_conflict_diff(repo: &Repository, file_path: &Path) -> Result<ThreeWayDiff, GitError> {
    let path_str = file_path.to_string_lossy();
    let base_content = get_stage_content(repo, file_path, 1).unwrap_or_default();
    let ours_content = get_stage_content(repo, file_path, 2)
        .or_else(|_| read_workdir_file(repo, file_path))
        .unwrap_or_default();
    let theirs_content = get_stage_content(repo, file_path, 3).unwrap_or_default();
    let hunks = parse_conflict_hunks(&ours_content, &theirs_content, &base_content);
    let has_conflicts = !hunks.is_empty();

    Ok(ThreeWayDiff {
        path: path_str.to_string(),
        hunks,
        has_conflicts,
        base_content,
        ours_content,
        theirs_content,
    })
}

/// Parse conflict hunks from three-way content
fn parse_conflict_hunks(ours: &str, theirs: &str, base: &str) -> Vec<ConflictHunk> {
    let mut hunks = Vec::new();

    // Split content into lines
    let our_lines: Vec<&str> = ours.lines().collect();
    let their_lines: Vec<&str> = theirs.lines().collect();
    let base_lines: Vec<&str> = base.lines().collect();

    let max_lines = our_lines.len().max(their_lines.len()).max(base_lines.len());

    let mut current_hunk: Option<ConflictHunk> = None;
    let mut hunk_started = false;

    for i in 0..max_lines {
        let ours_line = our_lines.get(i).map(|s| s.to_string());
        let theirs_line = their_lines.get(i).map(|s| s.to_string());
        let base_line = base_lines.get(i).map(|s| s.to_string());

        let line_type = classify_line(
            ours_line.as_deref(),
            theirs_line.as_deref(),
            base_line.as_deref(),
        );

        // Check if this starts a new hunk
        if line_type != ConflictLineType::Unchanged || ours_line.as_deref() == Some("<<<<<<<") {
            if !hunk_started {
                hunk_started = true;
                current_hunk = Some(ConflictHunk {
                    base_start: i as u32,
                    ours_start: i as u32,
                    theirs_start: i as u32,
                    base_lines: 0,
                    ours_lines: 0,
                    theirs_lines: 0,
                    lines: Vec::new(),
                });
            }
        } else if line_type == ConflictLineType::Unchanged && hunk_started {
            // End of hunk
            if let Some(h) = current_hunk.take() {
                hunk_started = false;
                hunks.push(h);
            }
        }

        if let Some(ref mut hunk) = current_hunk {
            let base_is_some = base_line.is_some();
            let ours_is_some = ours_line.is_some();
            let theirs_is_some = theirs_line.is_some();

            hunk.lines.push(ConflictLine {
                base_line,
                ours_line,
                theirs_line,
                line_type,
            });

            if ours_is_some {
                hunk.ours_lines += 1;
            }
            if theirs_is_some {
                hunk.theirs_lines += 1;
            }
            if base_is_some {
                hunk.base_lines += 1;
            }
        }
    }

    // Don't forget the last hunk
    if let Some(h) = current_hunk {
        hunks.push(h);
    }

    hunks
}

/// Classify a line to determine its type in the three-way diff
fn classify_line(ours: Option<&str>, theirs: Option<&str>, base: Option<&str>) -> ConflictLineType {
    let ours = ours.unwrap_or("");
    let theirs = theirs.unwrap_or("");
    let base = base.unwrap_or("");

    // Handle conflict markers
    if ours.contains("<<<<<<<")
        || ours.contains(">>>>>>>")
        || theirs.contains("<<<<<<<")
        || theirs.contains(">>>>>>>")
    {
        return ConflictLineType::ConflictMarker;
    }

    // Handle empty lines
    if ours.is_empty() && theirs.is_empty() && base.is_empty() {
        return ConflictLineType::Empty;
    }

    // All the same
    if ours == theirs && theirs == base {
        return ConflictLineType::Unchanged;
    }

    // Only ours changed
    if ours != base && theirs == base {
        return ConflictLineType::OursOnly;
    }

    // Only theirs changed
    if ours == base && theirs != base {
        return ConflictLineType::TheirsOnly;
    }

    // Both changed differently
    ConflictLineType::Modified
}

/// Resolve a conflict by choosing ours, theirs, or a combination
pub fn resolve_conflict(
    repo: &Repository,
    file_path: &Path,
    resolution: ConflictResolution,
) -> Result<(), GitError> {
    let index_path = repo_relative_path(repo, file_path);
    let content = match resolution {
        ConflictResolution::Ours => {
            // Git stages: 1 = base, 2 = ours, 3 = theirs
            get_stage_content(repo, index_path.as_path(), 2)?
        }
        ConflictResolution::Theirs => get_stage_content(repo, index_path.as_path(), 3)?,
        ConflictResolution::Base => get_stage_content(repo, index_path.as_path(), 1)?,
        ConflictResolution::Custom(ref s) => s.clone(),
    };

    // Write the resolved content to the working directory
    let workdir_path = workdir_file_path(repo, file_path);
    if let Some(parent) = workdir_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| GitError::OperationFailed {
            operation: "resolve_conflict".to_string(),
            details: format!("Failed to prepare resolved file parent: {}", e),
        })?;
    }
    std::fs::write(&workdir_path, content).map_err(|e| GitError::OperationFailed {
        operation: "resolve_conflict".to_string(),
        details: format!("Failed to write resolved file: {}", e),
    })?;

    // Stage the resolved file
    let repo_lock = repo.inner.read().unwrap();
    let mut index = repo_lock.index().map_err(|e| GitError::OperationFailed {
        operation: "resolve_conflict".to_string(),
        details: format!("Failed to get index: {}", e),
    })?;

    // Remove conflict entries and add the resolved file
    index.remove_path(index_path.as_path()).ok();

    index
        .add_path(index_path.as_path())
        .map_err(|e| GitError::OperationFailed {
            operation: "resolve_conflict".to_string(),
            details: format!("Failed to add resolved file: {}", e),
        })?;

    index.write().map_err(|e| GitError::OperationFailed {
        operation: "resolve_conflict".to_string(),
        details: format!("Failed to write index: {}", e),
    })?;

    Ok(())
}

/// Get content from a specific stage in the index
#[allow(dead_code)]
fn get_stage_content(repo: &Repository, file_path: &Path, stage: u32) -> Result<String, GitError> {
    let repo_lock = repo.inner.read().unwrap();

    let index = repo_lock.index().map_err(|e| GitError::OperationFailed {
        operation: "get_stage_content".to_string(),
        details: format!("Failed to get index: {}", e),
    })?;

    // Find the entry at the specified stage using get_path.
    // Git stage values: 1 = base, 2 = ours, 3 = theirs.
    let entry =
        index
            .get_path(file_path, stage as i32)
            .ok_or_else(|| GitError::OperationFailed {
                operation: "get_stage_content".to_string(),
                details: format!("No entry at stage {} for file {:?}", stage, file_path),
            })?;

    let blob = repo_lock
        .find_blob(entry.id)
        .map_err(|e| GitError::OperationFailed {
            operation: "get_stage_content".to_string(),
            details: format!("Failed to find blob: {}", e),
        })?;

    Ok(String::from_utf8_lossy(blob.content()).to_string())
}

fn read_workdir_file(repo: &Repository, file_path: &Path) -> Result<String, GitError> {
    let workdir_path = workdir_file_path(repo, file_path);
    std::fs::read_to_string(&workdir_path).map_err(|e| GitError::OperationFailed {
        operation: "read_workdir_file".to_string(),
        details: format!("Failed to read conflicted file: {}", e),
    })
}

fn repo_relative_path(repo: &Repository, file_path: &Path) -> std::path::PathBuf {
    if file_path.is_absolute() {
        if let Some(workdir) = repo.workdir.as_ref() {
            if let Ok(relative) = file_path.strip_prefix(workdir) {
                return relative.to_path_buf();
            }
        }
    }

    file_path.to_path_buf()
}

fn workdir_file_path(repo: &Repository, file_path: &Path) -> std::path::PathBuf {
    if file_path.is_absolute() {
        return file_path.to_path_buf();
    }

    if let Some(workdir) = repo.workdir.as_ref() {
        return workdir.join(file_path);
    }

    if repo.path.file_name().and_then(|name| name.to_str()) == Some(".git") {
        if let Some(parent) = repo.path.parent() {
            return parent.join(file_path);
        }
    }

    repo.path.join(file_path)
}

/// Conflict resolution options
#[derive(Debug, Clone)]
pub enum ConflictResolution {
    /// Keep our version
    Ours,
    /// Keep their version
    Theirs,
    /// Keep base version
    Base,
    /// Custom merged content
    Custom(String),
}

/// Type of conflict hunk (for auto-merge algorithm)
#[derive(Debug, Clone, PartialEq)]
pub enum ConflictHunkType {
    /// All lines are unchanged
    Unchanged,
    /// Only our side changed (auto-merge safe)
    OursOnly,
    /// Only their side changed (auto-merge safe)
    TheirsOnly,
    /// Both sides changed differently (true conflict - manual resolution needed)
    Modified,
}

/// Result of auto-merge operation
#[derive(Debug, Clone)]
pub struct AutoMergeResult {
    /// The merged content
    pub content: String,
    /// Whether there are remaining conflicts after auto-merge
    pub has_conflicts: bool,
    /// Number of hunks auto-merged
    pub merged_hunks: usize,
    /// Number of hunks that remain as conflicts
    pub remaining_conflicts: usize,
}

/// Automatically merge non-conflicting hunks in a three-way diff
///
/// For each hunk:
/// - If only our side changed → take our content
/// - If only their side changed → take their content
/// - If both changed differently → keep as conflict (need manual resolution)
pub fn auto_merge_conflict(three_way_diff: &ThreeWayDiff) -> AutoMergeResult {
    let mut merged_content = String::new();
    let mut merged_hunks = 0;
    let mut remaining_conflicts = 0;
    let mut has_conflicts = false;

    for hunk in &three_way_diff.hunks {
        let hunk_type = classify_hunk_type(hunk);

        // Extract lines from the ConflictLine array
        let ours_lines: Vec<&str> = hunk
            .lines
            .iter()
            .filter_map(|l| l.ours_line.as_deref())
            .collect();
        let theirs_lines: Vec<&str> = hunk
            .lines
            .iter()
            .filter_map(|l| l.theirs_line.as_deref())
            .collect();
        let base_lines: Vec<&str> = hunk
            .lines
            .iter()
            .filter_map(|l| l.base_line.as_deref())
            .collect();

        match hunk_type {
            ConflictHunkType::OursOnly => {
                // Take our version entirely
                merged_content.push_str(&ours_lines.join("\n"));
                merged_content.push('\n');
                merged_hunks += 1;
            }
            ConflictHunkType::TheirsOnly => {
                // Take their version entirely
                merged_content.push_str(&theirs_lines.join("\n"));
                merged_content.push('\n');
                merged_hunks += 1;
            }
            ConflictHunkType::Unchanged => {
                // Take base version
                merged_content.push_str(&base_lines.join("\n"));
                merged_content.push('\n');
                merged_hunks += 1;
            }
            ConflictHunkType::Modified => {
                // True conflict - keep conflict markers
                has_conflicts = true;
                remaining_conflicts += 1;
                merged_content.push_str("<<<<<<< HEAD\n");
                merged_content.push_str(&ours_lines.join("\n"));
                merged_content.push('\n');
                merged_content.push_str("=======\n");
                merged_content.push_str(&theirs_lines.join("\n"));
                merged_content.push('\n');
                merged_content.push_str(">>>>>>>\n");
            }
        }
    }

    AutoMergeResult {
        content: merged_content,
        has_conflicts,
        merged_hunks,
        remaining_conflicts,
    }
}

/// Classify a conflict hunk to determine its type
fn classify_hunk_type(hunk: &ConflictHunk) -> ConflictHunkType {
    let mut ours_only_count = 0;
    let mut theirs_only_count = 0;
    let mut modified_count = 0;
    let mut unchanged_count = 0;

    for line in &hunk.lines {
        match line.line_type {
            ConflictLineType::OursOnly => ours_only_count += 1,
            ConflictLineType::TheirsOnly => theirs_only_count += 1,
            ConflictLineType::Modified => modified_count += 1,
            ConflictLineType::Unchanged => unchanged_count += 1,
            ConflictLineType::Empty => {}
            ConflictLineType::ConflictMarker => {}
        }
    }

    // If any line is Modified, it's a true conflict
    if modified_count > 0 {
        ConflictHunkType::Modified
    } else if ours_only_count > 0 && theirs_only_count == 0 {
        ConflictHunkType::OursOnly
    } else if theirs_only_count > 0 && ours_only_count == 0 {
        ConflictHunkType::TheirsOnly
    } else if unchanged_count > 0 && ours_only_count == 0 && theirs_only_count == 0 {
        ConflictHunkType::Unchanged
    } else {
        // Mixed state - treat as conflict
        ConflictHunkType::Modified
    }
}

/// Resolve a single conflict hunk with the given resolution
///
/// Returns the resolved content for this hunk only.
pub fn resolve_conflict_hunk(hunk: &ConflictHunk, resolution: &ConflictResolution) -> String {
    // Extract lines from the ConflictLine array
    let ours_lines: Vec<&str> = hunk
        .lines
        .iter()
        .filter_map(|l| l.ours_line.as_deref())
        .collect();
    let theirs_lines: Vec<&str> = hunk
        .lines
        .iter()
        .filter_map(|l| l.theirs_line.as_deref())
        .collect();
    let base_lines: Vec<&str> = hunk
        .lines
        .iter()
        .filter_map(|l| l.base_line.as_deref())
        .collect();

    match resolution {
        ConflictResolution::Ours => ours_lines.join("\n"),
        ConflictResolution::Theirs => theirs_lines.join("\n"),
        ConflictResolution::Base => base_lines.join("\n"),
        ConflictResolution::Custom(ref content) => content.clone(),
    }
}

// ── Character-level inline diff (013 - similar crate) ─────────────────────

/// Meld-compatible character-level inline diff.
///
/// Matches Meld's `InlineMyersSequenceMatcher` behavior:
/// - Uses character-level diff (similar crate = Myers algorithm)
/// - Filters out equal matches shorter than `MIN_MATCH_LEN` (3 chars), marking
///   them as changed — this reduces visual noise from trivial shared characters
/// - Skips inline diff entirely if total characters exceed `MAX_INLINE_CHARS` (20K)
///   to avoid performance problems on very long lines
const MIN_INLINE_MATCH_LEN: usize = 3;
const MAX_INLINE_CHARS: usize = 20_000;

pub fn compute_inline_changes(
    old_line: &str,
    new_line: &str,
) -> (Vec<InlineChangeSpan>, Vec<InlineChangeSpan>) {
    // Meld threshold: skip inline diff for very long lines
    if old_line.len() + new_line.len() > MAX_INLINE_CHARS {
        return (
            vec![InlineChangeSpan {
                start: 0,
                len: old_line.len(),
                changed: true,
            }],
            vec![InlineChangeSpan {
                start: 0,
                len: new_line.len(),
                changed: true,
            }],
        );
    }

    use similar::{ChangeTag, TextDiff};

    let diff = TextDiff::from_chars(old_line, new_line);
    let mut old_spans = Vec::new();
    let mut new_spans = Vec::new();
    let mut old_pos = 0usize;
    let mut new_pos = 0usize;

    for change in diff.iter_all_changes() {
        let text = change.value();
        let len = text.len();

        match change.tag() {
            ChangeTag::Equal => {
                // Meld filter: equal matches shorter than 3 chars that are not at
                // the very start or end are treated as changed (visual noise reduction)
                let at_start = old_pos == 0 && new_pos == 0;
                let at_end_old = old_pos + len == old_line.len();
                let at_end_new = new_pos + len == new_line.len();
                let too_short =
                    len < MIN_INLINE_MATCH_LEN && !at_start && !(at_end_old && at_end_new);

                if too_short {
                    // Mark as changed on both sides
                    old_spans.push(InlineChangeSpan {
                        start: old_pos,
                        len,
                        changed: true,
                    });
                    new_spans.push(InlineChangeSpan {
                        start: new_pos,
                        len,
                        changed: true,
                    });
                } else {
                    old_spans.push(InlineChangeSpan {
                        start: old_pos,
                        len,
                        changed: false,
                    });
                    new_spans.push(InlineChangeSpan {
                        start: new_pos,
                        len,
                        changed: false,
                    });
                }
                old_pos += len;
                new_pos += len;
            }
            ChangeTag::Delete => {
                old_spans.push(InlineChangeSpan {
                    start: old_pos,
                    len,
                    changed: true,
                });
                old_pos += len;
            }
            ChangeTag::Insert => {
                new_spans.push(InlineChangeSpan {
                    start: new_pos,
                    len,
                    changed: true,
                });
                new_pos += len;
            }
        }
    }

    // Merge adjacent spans with same changed status
    (merge_spans(old_spans), merge_spans(new_spans))
}

fn merge_spans(spans: Vec<InlineChangeSpan>) -> Vec<InlineChangeSpan> {
    let mut merged = Vec::with_capacity(spans.len());
    for span in spans {
        if let Some(last) = merged.last_mut() {
            let last: &mut InlineChangeSpan = last;
            if last.changed == span.changed && last.start + last.len == span.start {
                last.len += span.len;
                continue;
            }
        }
        merged.push(span);
    }
    merged
}

/// Enhance a hunk by computing inline changes for paired deletion/addition lines.
/// Pairs consecutive delete+insert lines and computes character-level diffs.
pub fn enhance_hunk_with_inline_changes(hunk: &mut DiffHunk) {
    let lines = &mut hunk.lines;
    let mut i = 0;

    while i < lines.len() {
        // Look for a deletion followed by an addition
        if lines[i].origin == DiffLineOrigin::Deletion {
            // Collect consecutive deletions
            let del_start = i;
            while i < lines.len() && lines[i].origin == DiffLineOrigin::Deletion {
                i += 1;
            }
            let del_end = i;

            // Collect consecutive additions
            let add_start = i;
            while i < lines.len() && lines[i].origin == DiffLineOrigin::Addition {
                i += 1;
            }
            let add_end = i;

            // Pair up deletions and additions
            let pairs = (del_end - del_start).min(add_end - add_start);
            for p in 0..pairs {
                let (old_spans, new_spans) = compute_inline_changes(
                    &lines[del_start + p].content,
                    &lines[add_start + p].content,
                );
                lines[del_start + p].inline_changes = old_spans;
                lines[add_start + p].inline_changes = new_spans;
            }
        } else {
            i += 1;
        }
    }
}

// ── IDEA-style side-by-side: full file content retrieval ───────────────────

/// A single row in IDEA-style full-file side-by-side diff.
#[derive(Debug, Clone)]
pub struct SideBySideRow {
    /// "equal", "insert", "delete", or "replace"
    pub tag: &'static str,
    pub old_lineno: Option<u32>,
    pub new_lineno: Option<u32>,
    pub old_text: String,
    pub new_text: String,
    /// Character-level inline changes for old side (only for replace rows)
    pub old_inline: Vec<InlineChangeSpan>,
    /// Character-level inline changes for new side (only for replace rows)
    pub new_inline: Vec<InlineChangeSpan>,
}

/// Full-file side-by-side diff result.
#[derive(Debug, Clone)]
pub struct SideBySideDiff {
    pub rows: Vec<SideBySideRow>,
    pub old_path: String,
    pub new_path: String,
}

/// IDE/editor-oriented block kind for split diff rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorDiffBlockKind {
    Equal,
    Insert,
    Delete,
    Replace,
}

/// A single logical line inside the editor-oriented diff model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorDiffLine {
    /// Zero-based logical line index in the backing editor buffer.
    pub index: usize,
    /// One-based visible line number shown in the gutter.
    pub line_number: u32,
    pub content: String,
    pub inline_changes: Vec<InlineChangeSpan>,
}

/// A diff block inside a hunk, expressed in editor terms.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorDiffBlock {
    pub kind: EditorDiffBlockKind,
    pub old_range: std::ops::Range<usize>,
    pub new_range: std::ops::Range<usize>,
    pub old_lines: Vec<EditorDiffLine>,
    pub new_lines: Vec<EditorDiffLine>,
}

/// Line correlation used for scroll sync between left/right editors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorLineMapEntry {
    pub old_index: Option<usize>,
    pub new_index: Option<usize>,
    pub kind: EditorDiffBlockKind,
}

/// A hunk adapted for editor-backed rendering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorDiffHunk {
    pub id: usize,
    pub header: String,
    pub old_range: std::ops::Range<usize>,
    pub new_range: std::ops::Range<usize>,
    pub blocks: Vec<EditorDiffBlock>,
}

/// Full editor-oriented diff payload for a file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorDiffModel {
    pub left_text: String,
    pub right_text: String,
    pub hunks: Vec<EditorDiffHunk>,
    pub line_map: Vec<EditorLineMapEntry>,
    pub old_path: Option<String>,
    pub new_path: Option<String>,
}

/// Build an IDEA-style full-file side-by-side diff.
///
/// - `staged = false`: compares HEAD (old) vs workdir (new)
/// - `staged = true`:  compares HEAD (old) vs index (new)
///
/// Diffs the full files line-by-line with the `similar` crate, pairing
/// deletions with additions as "replace" rows with character-level inline
/// highlights.
pub fn build_side_by_side_diff(
    repo: &Repository,
    file_path: &str,
    staged: bool,
) -> Result<SideBySideDiff, GitError> {
    use similar::{ChangeTag, TextDiff};

    let path = Path::new(file_path);
    let repo_lock = repo.inner.read().unwrap();

    // 1. Get old content from HEAD
    let old_content = (|| -> Option<String> {
        let head = repo_lock.head().ok()?;
        let commit = head.peel_to_commit().ok()?;
        let tree = commit.tree().ok()?;
        let entry = tree.get_path(path).ok()?;
        let blob = repo_lock.find_blob(entry.id()).ok()?;
        Some(String::from_utf8_lossy(blob.content()).to_string())
    })()
    .unwrap_or_default();

    // 2. Get new content from index or workdir
    let new_content = if staged {
        // Read from index (stage 0 = normal staged entry)
        let index = repo_lock.index().ok();
        let entry = index.as_ref().and_then(|idx| idx.get_path(path, 0));
        entry
            .and_then(|e| repo_lock.find_blob(e.id).ok())
            .map(|blob| String::from_utf8_lossy(blob.content()).to_string())
            .unwrap_or_default()
    } else {
        drop(repo_lock);
        read_workdir_file(repo, path).unwrap_or_default()
    };

    #[allow(clippy::needless_borrows_for_generic_args)]
    let _ = &repo; // keep borrow alive if not dropped above

    // 3. Line-level diff
    let text_diff = TextDiff::from_lines(&old_content, &new_content);
    let mut rows = Vec::new();
    let mut old_lineno: u32 = 0;
    let mut new_lineno: u32 = 0;

    // Collect changes into groups for pairing delete+insert as replace
    let changes: Vec<_> = text_diff.iter_all_changes().collect();
    let mut i = 0;

    while i < changes.len() {
        let change = &changes[i];
        match change.tag() {
            ChangeTag::Equal => {
                old_lineno += 1;
                new_lineno += 1;
                rows.push(SideBySideRow {
                    tag: "equal",
                    old_lineno: Some(old_lineno),
                    new_lineno: Some(new_lineno),
                    old_text: change.value().trim_end_matches('\n').to_string(),
                    new_text: change.value().trim_end_matches('\n').to_string(),
                    old_inline: Vec::new(),
                    new_inline: Vec::new(),
                });
                i += 1;
            }
            ChangeTag::Delete => {
                // Collect consecutive deletes
                let del_start = i;
                while i < changes.len() && changes[i].tag() == ChangeTag::Delete {
                    i += 1;
                }
                // Collect consecutive inserts that follow
                let ins_start = i;
                while i < changes.len() && changes[i].tag() == ChangeTag::Insert {
                    i += 1;
                }
                let del_count = ins_start - del_start;
                let ins_count = i - ins_start;
                let pair_count = del_count.min(ins_count);

                // Paired lines → replace with inline diff
                for p in 0..pair_count {
                    old_lineno += 1;
                    new_lineno += 1;
                    let old_text = changes[del_start + p]
                        .value()
                        .trim_end_matches('\n')
                        .to_string();
                    let new_text = changes[ins_start + p]
                        .value()
                        .trim_end_matches('\n')
                        .to_string();
                    let (old_spans, new_spans) = compute_inline_changes(&old_text, &new_text);
                    rows.push(SideBySideRow {
                        tag: "replace",
                        old_lineno: Some(old_lineno),
                        new_lineno: Some(new_lineno),
                        old_text,
                        new_text,
                        old_inline: old_spans,
                        new_inline: new_spans,
                    });
                }
                // Remaining unpaired deletes
                for p in pair_count..del_count {
                    old_lineno += 1;
                    rows.push(SideBySideRow {
                        tag: "delete",
                        old_lineno: Some(old_lineno),
                        new_lineno: None,
                        old_text: changes[del_start + p]
                            .value()
                            .trim_end_matches('\n')
                            .to_string(),
                        new_text: String::new(),
                        old_inline: Vec::new(),
                        new_inline: Vec::new(),
                    });
                }
                // Remaining unpaired inserts
                for p in pair_count..ins_count {
                    new_lineno += 1;
                    rows.push(SideBySideRow {
                        tag: "insert",
                        old_lineno: None,
                        new_lineno: Some(new_lineno),
                        old_text: String::new(),
                        new_text: changes[ins_start + p]
                            .value()
                            .trim_end_matches('\n')
                            .to_string(),
                        old_inline: Vec::new(),
                        new_inline: Vec::new(),
                    });
                }
            }
            ChangeTag::Insert => {
                // Lone insert (no preceding delete)
                new_lineno += 1;
                rows.push(SideBySideRow {
                    tag: "insert",
                    old_lineno: None,
                    new_lineno: Some(new_lineno),
                    old_text: String::new(),
                    new_text: change.value().trim_end_matches('\n').to_string(),
                    old_inline: Vec::new(),
                    new_inline: Vec::new(),
                });
                i += 1;
            }
        }
    }

    Ok(SideBySideDiff {
        rows,
        old_path: file_path.to_string(),
        new_path: file_path.to_string(),
    })
}

const MAX_EDITOR_TEXT_BYTES: usize = 1_048_576; // 1 MB
const MAX_EDITOR_TEXT_LINES: usize = 10_000;

/// Build an editor-oriented diff model for a single file.
///
/// Returns `Ok(None)` for non-text, oversized, or empty diff inputs so the UI
/// can continue using its existing empty-state branches.
pub fn build_editor_diff_model(
    repo: &Repository,
    file_path: &str,
    staged: bool,
) -> Result<Option<EditorDiffModel>, GitError> {
    let path = Path::new(file_path);

    let diff = if staged {
        diff_index_to_head(repo, path)?
    } else {
        diff_file_to_index(repo, path)?
    };

    let Some(file_diff) = diff.files.first() else {
        return Ok(None);
    };

    if file_diff.hunks.is_empty() {
        return Ok(None);
    }

    let old_bytes = read_head_bytes(repo, path).unwrap_or_default();
    let new_bytes = if staged {
        read_index_bytes(repo, path).unwrap_or_default()
    } else {
        read_workdir_bytes(repo, path).unwrap_or_default()
    };

    if text_bytes_should_fallback(&old_bytes) || text_bytes_should_fallback(&new_bytes) {
        return Ok(None);
    }

    let left_text = String::from_utf8_lossy(&old_bytes).to_string();
    let right_text = String::from_utf8_lossy(&new_bytes).to_string();
    let line_map = build_editor_line_map(&left_text, &right_text);
    let hunks = file_diff
        .hunks
        .iter()
        .enumerate()
        .map(|(id, hunk)| build_editor_hunk(id, hunk))
        .collect();

    Ok(Some(EditorDiffModel {
        left_text,
        right_text,
        hunks,
        line_map,
        old_path: file_diff.old_path.clone(),
        new_path: file_diff.new_path.clone(),
    }))
}

fn text_bytes_should_fallback(bytes: &[u8]) -> bool {
    if bytes.len() > MAX_EDITOR_TEXT_BYTES {
        return true;
    }

    let sample_len = bytes.len().min(8192);
    if bytes[..sample_len].contains(&0) {
        return true;
    }

    let line_count = bytes.iter().filter(|&&byte| byte == b'\n').count()
        + usize::from(!bytes.is_empty() && *bytes.last().unwrap_or(&b'\n') != b'\n');
    line_count > MAX_EDITOR_TEXT_LINES
}

fn read_head_bytes(repo: &Repository, file_path: &Path) -> Option<Vec<u8>> {
    let repo_lock = repo.inner.read().ok()?;
    let head = repo_lock.head().ok()?;
    let commit = head.peel_to_commit().ok()?;
    let tree = commit.tree().ok()?;
    let entry = tree.get_path(file_path).ok()?;
    let blob = repo_lock.find_blob(entry.id()).ok()?;
    Some(blob.content().to_vec())
}

fn read_index_bytes(repo: &Repository, file_path: &Path) -> Option<Vec<u8>> {
    let repo_lock = repo.inner.read().ok()?;
    let index = repo_lock.index().ok()?;
    let entry = index.get_path(file_path, 0)?;
    let blob = repo_lock.find_blob(entry.id).ok()?;
    Some(blob.content().to_vec())
}

fn read_workdir_bytes(repo: &Repository, file_path: &Path) -> Option<Vec<u8>> {
    let abs_path = workdir_file_path(repo, file_path);
    std::fs::read(abs_path).ok()
}

fn build_editor_line_map(old_text: &str, new_text: &str) -> Vec<EditorLineMapEntry> {
    use similar::{ChangeTag, TextDiff};

    let diff = TextDiff::from_lines(old_text, new_text);
    let changes: Vec<_> = diff.iter_all_changes().collect();
    let mut map = Vec::new();
    let mut old_index = 0usize;
    let mut new_index = 0usize;
    let mut i = 0usize;

    while i < changes.len() {
        match changes[i].tag() {
            ChangeTag::Equal => {
                map.push(EditorLineMapEntry {
                    old_index: Some(old_index),
                    new_index: Some(new_index),
                    kind: EditorDiffBlockKind::Equal,
                });
                old_index += 1;
                new_index += 1;
                i += 1;
            }
            ChangeTag::Delete => {
                let del_start = i;
                while i < changes.len() && changes[i].tag() == ChangeTag::Delete {
                    i += 1;
                }
                let ins_start = i;
                while i < changes.len() && changes[i].tag() == ChangeTag::Insert {
                    i += 1;
                }

                let del_count = ins_start - del_start;
                let ins_count = i - ins_start;
                let pair_count = del_count.min(ins_count);

                for _ in 0..pair_count {
                    map.push(EditorLineMapEntry {
                        old_index: Some(old_index),
                        new_index: Some(new_index),
                        kind: EditorDiffBlockKind::Replace,
                    });
                    old_index += 1;
                    new_index += 1;
                }

                for _ in pair_count..del_count {
                    map.push(EditorLineMapEntry {
                        old_index: Some(old_index),
                        new_index: None,
                        kind: EditorDiffBlockKind::Delete,
                    });
                    old_index += 1;
                }

                for _ in pair_count..ins_count {
                    map.push(EditorLineMapEntry {
                        old_index: None,
                        new_index: Some(new_index),
                        kind: EditorDiffBlockKind::Insert,
                    });
                    new_index += 1;
                }
            }
            ChangeTag::Insert => {
                map.push(EditorLineMapEntry {
                    old_index: None,
                    new_index: Some(new_index),
                    kind: EditorDiffBlockKind::Insert,
                });
                new_index += 1;
                i += 1;
            }
        }
    }

    map
}

fn build_editor_hunk(id: usize, hunk: &DiffHunk) -> EditorDiffHunk {
    let mut blocks = Vec::new();
    let mut old_index = hunk.old_start.saturating_sub(1) as usize;
    let mut new_index = hunk.new_start.saturating_sub(1) as usize;
    let mut cursor = 0usize;

    while cursor < hunk.lines.len() {
        match hunk.lines[cursor].origin {
            DiffLineOrigin::Context => {
                let old_start = old_index;
                let new_start = new_index;
                let mut old_lines = Vec::new();
                let mut new_lines = Vec::new();

                while cursor < hunk.lines.len()
                    && hunk.lines[cursor].origin == DiffLineOrigin::Context
                {
                    let line = &hunk.lines[cursor];
                    old_lines.push(editor_line_from_diff(line, old_index, false));
                    new_lines.push(editor_line_from_diff(line, new_index, true));
                    old_index += 1;
                    new_index += 1;
                    cursor += 1;
                }

                blocks.push(EditorDiffBlock {
                    kind: EditorDiffBlockKind::Equal,
                    old_range: old_start..old_index,
                    new_range: new_start..new_index,
                    old_lines,
                    new_lines,
                });
            }
            DiffLineOrigin::Deletion => {
                let old_start = old_index;
                let new_start = new_index;
                let mut old_lines = Vec::new();
                let mut new_lines = Vec::new();

                while cursor < hunk.lines.len()
                    && hunk.lines[cursor].origin == DiffLineOrigin::Deletion
                {
                    let line = &hunk.lines[cursor];
                    old_lines.push(editor_line_from_diff(line, old_index, false));
                    old_index += 1;
                    cursor += 1;
                }

                while cursor < hunk.lines.len()
                    && hunk.lines[cursor].origin == DiffLineOrigin::Addition
                {
                    let line = &hunk.lines[cursor];
                    new_lines.push(editor_line_from_diff(line, new_index, true));
                    new_index += 1;
                    cursor += 1;
                }

                let kind = if new_lines.is_empty() {
                    EditorDiffBlockKind::Delete
                } else {
                    EditorDiffBlockKind::Replace
                };

                blocks.push(EditorDiffBlock {
                    kind,
                    old_range: old_start..old_index,
                    new_range: new_start..new_index,
                    old_lines,
                    new_lines,
                });
            }
            DiffLineOrigin::Addition => {
                let old_start = old_index;
                let new_start = new_index;
                let mut new_lines = Vec::new();

                while cursor < hunk.lines.len()
                    && hunk.lines[cursor].origin == DiffLineOrigin::Addition
                {
                    let line = &hunk.lines[cursor];
                    new_lines.push(editor_line_from_diff(line, new_index, true));
                    new_index += 1;
                    cursor += 1;
                }

                blocks.push(EditorDiffBlock {
                    kind: EditorDiffBlockKind::Insert,
                    old_range: old_start..old_start,
                    new_range: new_start..new_index,
                    old_lines: Vec::new(),
                    new_lines,
                });
            }
            DiffLineOrigin::Header | DiffLineOrigin::HunkHeader => {
                cursor += 1;
            }
        }
    }

    EditorDiffHunk {
        id,
        header: hunk.header.clone(),
        old_range: hunk.old_start.saturating_sub(1) as usize
            ..hunk.old_start.saturating_sub(1) as usize + hunk.old_lines as usize,
        new_range: hunk.new_start.saturating_sub(1) as usize
            ..hunk.new_start.saturating_sub(1) as usize + hunk.new_lines as usize,
        blocks,
    }
}

fn editor_line_from_diff(line: &DiffLine, index: usize, is_new_side: bool) -> EditorDiffLine {
    let line_number = if is_new_side {
        line.new_lineno.unwrap_or(index as u32 + 1)
    } else {
        line.old_lineno.unwrap_or(index as u32 + 1)
    };

    EditorDiffLine {
        index,
        line_number,
        content: line.content.clone(),
        inline_changes: line.inline_changes.clone(),
    }
}

// ── Full file preview support (012-idea-ui-refactor) ──────────────────────

const MAX_PREVIEW_BYTES: usize = 1_048_576; // 1 MB
const MAX_PREVIEW_LINES: usize = 5000;

/// Check if a file is binary by looking for null bytes in the first 8KB.
pub fn file_is_binary(path: &Path) -> bool {
    use std::io::Read;
    let Ok(mut f) = std::fs::File::open(path) else {
        return false;
    };
    let mut buf = [0u8; 8192];
    let Ok(n) = f.read(&mut buf) else {
        return false;
    };
    buf[..n].contains(&0)
}

/// Result of building a full-file preview diff.
pub struct FullFilePreview {
    pub diff: FileDiff,
    pub is_binary: bool,
    pub is_truncated: bool,
}

/// Build a FileDiff that shows the entire file content as additions.
/// Used when there's no diff to display (new/untracked files).
/// Respects MAX_PREVIEW_BYTES / MAX_PREVIEW_LINES limits.
pub fn build_full_file_diff(
    repo: &Repository,
    file_path: &Path,
) -> Result<FullFilePreview, GitError> {
    let abs_path = repo.path().join(file_path);

    if file_is_binary(&abs_path) {
        return Ok(FullFilePreview {
            diff: FileDiff {
                old_path: None,
                new_path: Some(file_path.to_string_lossy().to_string()),
                hunks: Vec::new(),
                additions: 0,
                deletions: 0,
            },
            is_binary: true,
            is_truncated: false,
        });
    }

    let content = std::fs::read_to_string(&abs_path).map_err(|e| GitError::OperationFailed {
        operation: "build_full_file_diff".to_string(),
        details: format!("Failed to read file {:?}: {}", file_path, e),
    })?;

    let total_lines = content.lines().count();
    let is_truncated = content.len() > MAX_PREVIEW_BYTES || total_lines > MAX_PREVIEW_LINES;

    let lines: Vec<&str> = content.lines().take(MAX_PREVIEW_LINES).collect();
    let line_count = lines.len() as u32;

    let diff_lines: Vec<DiffLine> = lines
        .iter()
        .enumerate()
        .map(|(i, line)| DiffLine {
            content: line.to_string(),
            origin: DiffLineOrigin::Addition,
            old_lineno: None,
            new_lineno: Some(i as u32 + 1),
            inline_changes: Vec::new(),
        })
        .collect();

    let hunk = DiffHunk {
        header: format!("@@ -0,0 +1,{} @@", line_count),
        lines: diff_lines,
        old_start: 0,
        old_lines: 0,
        new_start: 1,
        new_lines: line_count,
    };

    Ok(FullFilePreview {
        diff: FileDiff {
            old_path: None,
            new_path: Some(file_path.to_string_lossy().to_string()),
            hunks: vec![hunk],
            additions: line_count,
            deletions: 0,
        },
        is_binary: false,
        is_truncated,
    })
}

#[cfg(test)]
mod tests {
    use super::build_editor_diff_model;
    use super::diff_file_to_index;
    use super::DiffLineOrigin;
    use super::EditorDiffBlockKind;
    use crate::commit;
    use crate::index;
    use crate::repository::Repository;
    use std::fs;
    use std::path::Path;
    use tempfile::TempDir;

    fn configure_signature(repo: &Repository) {
        let repo_lock = repo.inner.read().expect("repo lock");
        let mut config = repo_lock.config().expect("config");
        config
            .set_str("user.name", "slio-git tests")
            .expect("user.name");
        config
            .set_str("user.email", "tests@slio-git.local")
            .expect("user.email");
    }

    fn commit_file(repo: &Repository, root: &Path, path: &str, content: &str, message: &str) {
        let file_path = root.join(path);
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).expect("parent dir");
        }
        fs::write(&file_path, content).expect("write file");
        index::stage_all(repo).expect("stage all");
        commit::create_commit(repo, message, "", "").expect("create commit");
    }

    #[test]
    fn diff_file_to_index_returns_targeted_file_hunks() {
        let temp_dir = TempDir::new().expect("temp dir");
        let repo = Repository::init(temp_dir.path()).expect("init repo");
        let file_path = temp_dir.path().join("notes.txt");

        fs::write(&file_path, "line one\n").expect("seed file");
        index::stage_all(&repo).expect("stage baseline");
        fs::write(&file_path, "line one\nline two\n").expect("modify file");

        let diff = diff_file_to_index(&repo, Path::new("notes.txt")).expect("load diff");

        assert_eq!(diff.files.len(), 1);
        assert_eq!(diff.files[0].new_path.as_deref(), Some("notes.txt"));
        assert_eq!(diff.total_additions, 1);
        assert_eq!(diff.files[0].additions, 1);
        assert!(!diff.files[0].hunks.is_empty());
        assert!(diff.files[0]
            .hunks
            .iter()
            .flat_map(|hunk| hunk.lines.iter())
            .any(|line| {
                line.origin == DiffLineOrigin::Addition && line.content.contains("line two")
            }));
    }

    #[test]
    fn diff_file_to_index_includes_untracked_files() {
        let temp_dir = TempDir::new().expect("temp dir");
        let repo = Repository::init(temp_dir.path()).expect("init repo");
        let file_path = temp_dir.path().join("drafts").join("new_note.txt");

        fs::create_dir_all(file_path.parent().expect("parent dir")).expect("create parent dir");
        fs::write(&file_path, "fresh line\n").expect("write untracked file");

        let diff = diff_file_to_index(&repo, Path::new("drafts/new_note.txt")).expect("load diff");

        assert_eq!(diff.files.len(), 1);
        assert_eq!(
            diff.files[0].new_path.as_deref(),
            Some("drafts/new_note.txt")
        );
        assert_eq!(diff.total_additions, 1);
        assert_eq!(diff.files[0].additions, 1);
        assert!(!diff.files[0].hunks.is_empty());
        assert!(diff.files[0]
            .hunks
            .iter()
            .flat_map(|hunk| hunk.lines.iter())
            .any(|line| {
                line.origin == DiffLineOrigin::Addition && line.content.contains("fresh line")
            }));
    }

    #[test]
    fn build_editor_diff_model_produces_replace_block_and_line_map() {
        let temp_dir = TempDir::new().expect("temp dir");
        let repo = Repository::init(temp_dir.path()).expect("init repo");
        configure_signature(&repo);
        commit_file(
            &repo,
            temp_dir.path(),
            "notes.txt",
            "alpha\nbravo old\ncharlie\n",
            "baseline",
        );

        fs::write(
            temp_dir.path().join("notes.txt"),
            "alpha\nbravo new\ncharlie\n",
        )
        .expect("modify file");

        let model = build_editor_diff_model(&repo, "notes.txt", false)
            .expect("build editor diff")
            .expect("text diff model");

        assert_eq!(model.old_path.as_deref(), Some("notes.txt"));
        assert_eq!(model.new_path.as_deref(), Some("notes.txt"));
        assert!(model.left_text.contains("bravo old"));
        assert!(model.right_text.contains("bravo new"));
        assert_eq!(model.hunks.len(), 1);

        let replace_block = model.hunks[0]
            .blocks
            .iter()
            .find(|block| block.kind == EditorDiffBlockKind::Replace)
            .expect("replace block");

        assert_eq!(replace_block.old_lines.len(), 1);
        assert_eq!(replace_block.new_lines.len(), 1);
        assert!(replace_block.old_lines[0]
            .inline_changes
            .iter()
            .any(|span| span.changed));
        assert!(
            model.line_map.iter().any(|entry| {
                entry.kind == EditorDiffBlockKind::Replace
                    && entry.old_index == Some(1)
                    && entry.new_index == Some(1)
            }),
            "expected replace entry for second line"
        );
    }

    #[test]
    fn build_editor_diff_model_tracks_insertions_and_deletions_in_line_map() {
        let temp_dir = TempDir::new().expect("temp dir");
        let repo = Repository::init(temp_dir.path()).expect("init repo");
        configure_signature(&repo);
        commit_file(
            &repo,
            temp_dir.path(),
            "notes.txt",
            "one\ntwo\nthree\n",
            "baseline",
        );

        fs::write(temp_dir.path().join("notes.txt"), "zero\none\nthree\n").expect("modify file");

        let model = build_editor_diff_model(&repo, "notes.txt", false)
            .expect("build editor diff")
            .expect("text diff model");

        assert!(
            model.line_map.iter().any(|entry| {
                entry.kind == EditorDiffBlockKind::Insert
                    && entry.old_index.is_none()
                    && entry.new_index == Some(0)
            }),
            "expected inserted first line"
        );
        assert!(
            model.line_map.iter().any(|entry| {
                entry.kind == EditorDiffBlockKind::Delete
                    && entry.old_index == Some(1)
                    && entry.new_index.is_none()
            }),
            "expected deleted second line"
        );
    }

    #[test]
    fn build_editor_diff_model_staged_mode_reads_index_not_workdir() {
        let temp_dir = TempDir::new().expect("temp dir");
        let repo = Repository::init(temp_dir.path()).expect("init repo");
        configure_signature(&repo);
        commit_file(&repo, temp_dir.path(), "notes.txt", "one\n", "baseline");

        let file_path = temp_dir.path().join("notes.txt");
        fs::write(&file_path, "one staged\n").expect("stage candidate");
        index::stage_all(&repo).expect("stage candidate");
        fs::write(&file_path, "one staged\nplus unstaged\n").expect("unstaged follow-up");

        let model = build_editor_diff_model(&repo, "notes.txt", true)
            .expect("build editor diff")
            .expect("text diff model");

        assert_eq!(model.right_text, "one staged\n");
        assert!(!model.right_text.contains("plus unstaged"));
    }

    #[test]
    fn build_editor_diff_model_keeps_multibyte_inline_offsets_on_char_boundaries() {
        let temp_dir = TempDir::new().expect("temp dir");
        let repo = Repository::init(temp_dir.path()).expect("init repo");
        configure_signature(&repo);
        commit_file(
            &repo,
            temp_dir.path(),
            "notes.txt",
            "你好世界\n",
            "baseline",
        );

        fs::write(temp_dir.path().join("notes.txt"), "你好同学\n").expect("modify file");

        let model = build_editor_diff_model(&repo, "notes.txt", false)
            .expect("build editor diff")
            .expect("text diff model");

        let replace_block = model.hunks[0]
            .blocks
            .iter()
            .find(|block| block.kind == EditorDiffBlockKind::Replace)
            .expect("replace block");
        let old_line = &replace_block.old_lines[0];

        let changed_span = old_line
            .inline_changes
            .iter()
            .find(|span| span.changed)
            .expect("changed inline span");
        let end = changed_span.start + changed_span.len;
        let changed_slice = old_line
            .content
            .get(changed_span.start..end)
            .expect("valid utf-8 boundaries");

        assert_eq!(changed_slice.chars().count(), 2);
        assert_eq!(changed_slice, "世界");
    }

    #[test]
    fn build_editor_diff_model_preserves_absolute_block_ranges_inside_hunk() {
        let temp_dir = TempDir::new().expect("temp dir");
        let repo = Repository::init(temp_dir.path()).expect("init repo");
        configure_signature(&repo);
        commit_file(
            &repo,
            temp_dir.path(),
            "notes.txt",
            "alpha\nbravo\ncharlie\ndelta\n",
            "baseline",
        );

        fs::write(
            temp_dir.path().join("notes.txt"),
            "alpha\nbravo updated\ncharlie\ndelta\necho\n",
        )
        .expect("modify file");

        let model = build_editor_diff_model(&repo, "notes.txt", false)
            .expect("build editor diff")
            .expect("text diff model");

        let hunk = model.hunks.first().expect("hunk");
        let replace_block = hunk
            .blocks
            .iter()
            .find(|block| block.kind == EditorDiffBlockKind::Replace)
            .expect("replace block");
        let insert_block = hunk
            .blocks
            .iter()
            .find(|block| block.kind == EditorDiffBlockKind::Insert)
            .expect("insert block");

        assert_eq!(replace_block.old_range, 1..2);
        assert_eq!(replace_block.new_range, 1..2);
        assert_eq!(insert_block.old_range, 4..4);
        assert_eq!(insert_block.new_range, 4..5);
    }
}

// ═══════════════════════════════════════
// Merge editor model — 三栏编辑器数据层
// ═══════════════════════════════════════

use std::ops::Range;

/// Chunk type in a three-way merge
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergeChunkType {
    /// Same in all three versions
    Equal,
    /// Only ours changed from base (auto-merge safe)
    OursOnly,
    /// Only theirs changed from base (auto-merge safe)
    TheirsOnly,
    /// Both sides changed — true conflict
    Conflict,
}

/// A chunk in the merge editor model
#[derive(Debug, Clone)]
pub struct MergeChunk {
    pub id: usize,
    pub chunk_type: MergeChunkType,
    /// Line range in ours full text
    pub ours_range: Range<usize>,
    /// Line range in theirs full text
    pub theirs_range: Range<usize>,
    /// Line range in base full text
    pub base_range: Range<usize>,
    pub lines_ours: Vec<String>,
    pub lines_theirs: Vec<String>,
    pub lines_base: Vec<String>,
}

/// The editor-oriented model for the Meld-style 3-column merge view
#[derive(Debug, Clone)]
pub struct MergeEditorModel {
    pub path: String,
    pub ours_text: String,
    pub theirs_text: String,
    pub base_text: String,
    pub chunks: Vec<MergeChunk>,
}

impl ThreeWayDiff {
    /// Convert to merge-editor-oriented model with chunk-level granularity.
    pub fn to_merge_editor_model(&self) -> MergeEditorModel {
        let ours_lines: Vec<&str> = self.ours_content.lines().collect();
        let theirs_lines: Vec<&str> = self.theirs_content.lines().collect();
        let base_lines: Vec<&str> = self.base_content.lines().collect();
        let max_lines = ours_lines.len().max(theirs_lines.len()).max(base_lines.len());

        let mut chunks: Vec<MergeChunk> = Vec::new();
        let mut chunk_id = 0usize;

        // Build per-line classification
        let mut line_types = Vec::with_capacity(max_lines);
        for i in 0..max_lines {
            let o = ours_lines.get(i).copied().unwrap_or("");
            let t = theirs_lines.get(i).copied().unwrap_or("");
            let b = base_lines.get(i).copied().unwrap_or("");
            line_types.push(classify_merge_line(o, t, b));
        }

        // Group consecutive lines with the same classification into chunks
        let mut i = 0;
        while i < max_lines {
            let start = i;
            let first_type = line_types[i];

            // Extend to cover all consecutive lines of the same type
            while i < max_lines && line_types[i] == first_type {
                i += 1;
            }

            let ours_start = start.min(ours_lines.len());
            let ours_end = i.min(ours_lines.len());
            let theirs_start = start.min(theirs_lines.len());
            let theirs_end = i.min(theirs_lines.len());
            let base_start = start.min(base_lines.len());
            let base_end = i.min(base_lines.len());

            chunks.push(MergeChunk {
                id: chunk_id,
                chunk_type: first_type,
                ours_range: ours_start..ours_end,
                theirs_range: theirs_start..theirs_end,
                base_range: base_start..base_end,
                lines_ours: ours_lines[ours_start..ours_end]
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
                lines_theirs: theirs_lines[theirs_start..theirs_end]
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
                lines_base: base_lines[base_start..base_end]
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
            });
            chunk_id += 1;
        }

        MergeEditorModel {
            path: self.path.clone(),
            ours_text: self.ours_content.clone(),
            theirs_text: self.theirs_content.clone(),
            base_text: self.base_content.clone(),
            chunks,
        }
    }
}

fn classify_merge_line(ours: &str, theirs: &str, base: &str) -> MergeChunkType {
    if ours == theirs && theirs == base {
        MergeChunkType::Equal
    } else if ours != base && theirs == base {
        MergeChunkType::OursOnly
    } else if ours == base && theirs != base {
        MergeChunkType::TheirsOnly
    } else {
        MergeChunkType::Conflict
    }
}
