# UI Component Contracts: JetBrains风格Git UI重构

**Feature**: JetBrains风格Git UI重构
**Date**: 2026-03-22
**Branch**: `002-jetbrains-ui-refactor`

## Contract: MainWindow

```rust
/// Main window must render JetBrains-style Git tool window layout
///
/// Layout structure:
/// ┌─────────────────────────────────────────────────────┐
/// │ Toolbar (refresh, commit, pull, push, stash, ...) │
/// ├──────────────────┬──────────────────────────────────┤
/// │                  │                                  │
/// │  Changes List    │      Diff Viewer                 │
/// │  (file tree)     │      (unified or split)          │
/// │                  │                                  │
/// │                  │                                  │
/// ├──────────────────┴──────────────────────────────────┤
/// │ Status Bar (branch, repo path, sync status)        │
/// └─────────────────────────────────────────────────────┘
///
/// Requirements:
/// - FR-001: Three-region layout (toolbar + content + statusbar)
/// - SC-007: Window size 1280x800 default, 800x600 minimum
/// - SC-006: Chinese font (PingFang SC) must render correctly
```

## Contract: Toolbar

```rust
/// Toolbar button definitions
///
/// Buttons required (per FR-002):
/// - 刷新 (Refresh)
/// - 提交 (Commit)
/// - 拉取 (Pull)
/// - 推送 (Push)
/// - 暂存全部 (Stage All)
/// - 取消暂存全部 (Unstage All)
/// - 藏匿 (Stash)
/// - 分支选择器 (Branch selector - shows current branch name)
///
/// Interaction:
/// - Each button must respond within 100ms (SC-002)
/// - Icons + optional text labels
```

## Contract: ChangesList

```rust
/// Changes list widget
///
/// Data: Vec<Change> where Change contains:
/// - path: String
/// - status: ChangeStatus (Added|Modified|Deleted|Renamed|Conflict)
/// - staged: bool
///
/// Display:
/// - Tree structure grouped by directory
/// - Status icon per file (green=added, red=deleted, blue=modified, yellow=conflict)
/// - Staged indicator (checkbox or highlight)
///
/// Interaction (per FR-003):
/// - Click: select file, show diff
/// - Double-click: open diff view
/// - Right-click: context menu (stage/unstage/revert)
///
/// Performance:
/// - Must handle 1000+ files without lag
```

## Contract: DiffViewer

```rust
/// Diff viewer widget (per FR-004)
///
/// Display modes:
/// - Unified: single column with +/- prefixes
/// - Split: two columns (left=ours, right=theirs)
///
/// Highlighting:
/// - Added lines: green background (#e6ffed)
/// - Deleted lines: red background (#ffebe9)
/// - Modified lines: yellow background (#fff5b1)
///
/// Line numbers: must sync with scroll
///
/// Performance:
/// - 1000 lines must load within 500ms (SC-004)
```

## Contract: ConflictResolver

```rust
/// Three-way merge conflict resolver (per FR-009~FR-012)
///
/// Data: ThreeWayDiff containing:
/// - path: String
/// - hunks: Vec<ConflictHunk>
///   - Each hunk has base_lines, ours_lines, theirs_lines
///   - Each hunk has line_type: Unchanged|OursOnly|TheirsOnly|Modified
///
/// Display:
/// ┌──────────────────────────────────────────────────────────┐
/// │ Ours (Stage 2)    │ Base (Stage 1) │ Theirs (Stage 3)   │
/// ├──────────────────────────────────────────────────────────┤
/// │                                                              │
/// │              Result (auto-merged preview)                  │
/// │                                                              │
/// └──────────────────────────────────────────────────────────┘
/// Buttons: [Accept Ours] [Accept Theirs] [Auto Merge] [Done]
///
/// Auto-merge algorithm (per FR-010):
/// - For each hunk with line_type = OursOnly → copy ours_lines to result
/// - For each hunk with line_type = TheirsOnly → copy theirs_lines to result
/// - For each hunk with line_type = Modified → keep as conflict for manual resolution
///
/// Per-hunk resolution (per FR-012):
/// - User can click on individual hunks to choose resolution
/// - Mark conflict as resolved when user selects
```

## Contract: CommitDialog

```rust
/// Commit dialog widget (per FR-006)
///
/// Layout:
/// ┌─────────────────────────────────────────┐
/// │ Commit Message                          │
││ ┌─────────────────────────────────────┐ │
/// │ (multiline text input)               │ │
/// │                                     │ │
/// └─────────────────────────────────────┘ │
││                                          │
││ Changes to commit (selectable list)      │
││ ☑ src/main.rs                           │
││ ☑ src/lib.rs                            │
││                                          │
││         [Cancel]  [Commit]               │
│└─────────────────────────────────────────┘
///
/// Validation:
/// - Message cannot be empty
/// - At least one file must be selected
```

## Contract: StatusBar

```rust
/// Status bar widget (per FR-007)
///
/// Display:
/// - Repository path (left)
/// - Current branch name with icon (center-left)
/// - Ahead/Behind arrows with count (center)
/// - Sync status icon: ✓ synced, ↑ push needed, ↓ pull needed (right)
///
/// Updates:
/// - Refresh on every repository operation
/// - Show uncommitted change count if any
```
