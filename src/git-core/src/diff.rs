//! Diff operations for git-core

use crate::error::GitError;
use crate::repository::Repository;
use git2::{Diff as Git2Diff, DiffOptions};
use std::cell::{Cell, RefCell};
use std::path::Path;

/// Origin of a diff line
#[derive(Debug, Clone, PartialEq)]
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
#[derive(Debug, Clone)]
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

/// Compute character-level change spans for paired deletion/addition lines.
/// This powers GitHub-style inline highlighting where specific changed
/// characters within a line are marked with a brighter background.
pub fn compute_inline_changes(old_line: &str, new_line: &str) -> (Vec<InlineChangeSpan>, Vec<InlineChangeSpan>) {
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
                old_spans.push(InlineChangeSpan { start: old_pos, len, changed: false });
                new_spans.push(InlineChangeSpan { start: new_pos, len, changed: false });
                old_pos += len;
                new_pos += len;
            }
            ChangeTag::Delete => {
                old_spans.push(InlineChangeSpan { start: old_pos, len, changed: true });
                old_pos += len;
            }
            ChangeTag::Insert => {
                new_spans.push(InlineChangeSpan { start: new_pos, len, changed: true });
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
pub fn build_full_file_diff(repo: &Repository, file_path: &Path) -> Result<FullFilePreview, GitError> {
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
    use super::diff_file_to_index;
    use super::DiffLineOrigin;
    use crate::index;
    use crate::repository::Repository;
    use std::fs;
    use std::path::Path;
    use tempfile::TempDir;

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
}
