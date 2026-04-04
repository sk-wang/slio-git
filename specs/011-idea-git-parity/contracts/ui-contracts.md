# UI Component Contracts: IDEA Git Feature Parity

**Branch**: `011-idea-git-parity` | **Date**: 2026-04-04

## Overview

This document defines the interface contracts for new and modified UI components. Each contract specifies inputs (props/state), outputs (messages), and layout behavior.

---

## 1. ChangeList Widget (REWRITE)

**File**: `src-ui/src/widgets/changelist.rs`

### Input State
- `staged_changes: &[Change]` - Files in staged group
- `unstaged_changes: &[Change]` - Files in unstaged group (includes untracked)
- `selected_path: Option<&str>` - Currently selected file
- `display_mode: FileDisplayMode` - Flat or Tree
- `staged_collapsed: bool` - Whether staged group is collapsed
- `unstaged_collapsed: bool` - Whether unstaged group is collapsed

### Output Messages
- `StageFile(String)` - Move file to staged
- `UnstageFile(String)` - Move file to unstaged
- `SelectFile(String)` - Select file for diff preview
- `ToggleStaged` - Collapse/expand staged group
- `ToggleUnstaged` - Collapse/expand unstaged group
- `ToggleDisplayMode` - Switch flat ↔ tree
- `ContextMenu(String, Point)` - Right-click on file at position
- `DragStart(String)` - Begin dragging a file
- `DragDrop(String, DropZone)` - Drop file into zone (Staged/Unstaged)

### Layout
```
┌──────────────────────────┐
│ [Flat|Tree] [StageAll]   │ ← Toolbar (display mode toggle + bulk actions)
├──────────────────────────┤
│ ▼ Staged Changes (3)     │ ← Collapsible header with count
│   + file_a.rs    M       │ ← File with status icon, "+" = unstage button
│   + file_b.rs    A       │
│   + file_c.rs    D       │
├──────────────────────────┤
│ ▼ Unstaged Changes (5)   │ ← Collapsible header with count
│   + file_d.rs    M       │ ← "+" = stage button
│   + file_e.rs    ?       │ ← Untracked
│   ...                    │
└──────────────────────────┘
```

---

## 2. CommitPanel Widget (EXTEND)

**File**: `src-ui/src/widgets/commit_panel.rs`

### Input State
- `message: &str` - Current commit message text
- `amend: bool` - Amend mode toggle
- `recent_messages: &[String]` - Last 10 commit messages
- `staged_count: usize` - Number of staged files (for button enable state)
- `can_commit: bool` - Whether commit is allowed (message non-empty + staged files)

### Output Messages
- `MessageChanged(String)` - Commit message text updated
- `ToggleAmend` - Toggle amend mode
- `SelectRecentMessage(usize)` - Select message from history dropdown
- `Commit` - Execute commit
- `CommitAndPush` - Execute commit then push

### Layout
```
┌──────────────────────────┐
│ [History ▼] [☐ Amend]    │ ← Recent messages dropdown + amend toggle
├──────────────────────────┤
│ Commit message...        │ ← Multi-line text input
│                          │
│                          │
├──────────────────────────┤
│ [Commit] [Commit & Push] │ ← Action buttons
└──────────────────────────┘
```

---

## 3. CommitGraph Widget (NEW)

**File**: `src-ui/src/widgets/commit_graph.rs`

### Input State
- `nodes: &[GraphNode]` - Computed graph layout
- `commits: &[HistoryEntry]` - Commit metadata
- `refs: &HashMap<String, Vec<RefLabel>>` - Ref labels by commit ID
- `selected_commit: Option<&str>` - Currently selected commit
- `viewport_start: usize` - First visible row index
- `viewport_size: usize` - Number of visible rows
- `max_lanes: u32` - Maximum lane count for width calculation

### Output Messages
- `SelectCommit(String)` - Click on commit row
- `CommitContextMenu(String, Point)` - Right-click on commit
- `ScrollTo(usize)` - Scroll to row index
- `OpenBranchTab(String)` - Double-click ref label to open in new tab

### Layout
```
┌─────┬──────────────────────────────────────────────────┐
│Graph│ Hash    │ Message          │ Author  │ Date      │
├─────┼─────────┼──────────────────┼─────────┼───────────┤
│ ●─┐ │ abc1234 │ feat: add login  │ Alice   │ 2h ago    │
│ │ ● │ def5678 │ fix: auth bug    │ Bob     │ 3h ago    │
│ ●─┘ │ ghi9012 │ merge: feature   │ Alice   │ 5h ago    │
│ ●   │ jkl3456 │ refactor: utils  │ Carol   │ 1d ago    │
└─────┴─────────┴──────────────────┴─────────┴───────────┘
```

---

## 4. LogTabs Widget (NEW)

**File**: `src-ui/src/widgets/log_tabs.rs`

### Input State
- `tabs: &[LogTab]` - All open tabs
- `active_tab: usize` - Currently active tab index

### Output Messages
- `SelectTab(usize)` - Switch to tab
- `CloseTab(usize)` - Close tab (not emitted for "All" tab)
- `NewTab` - Create new empty tab

### Layout
```
┌────────┬──────────┬──────────┬─────┐
│  All   │ main [×] │ dev  [×] │ [+] │ ← Tab bar with close buttons
└────────┴──────────┴──────────┴─────┘
```

---

## 5. TreeWidget (NEW - generic)

**File**: `src-ui/src/widgets/tree_widget.rs`

### Input State
- `nodes: &[TreeNode]` - Tree structure (id, label, children, expanded, icon)
- `selected_id: Option<&str>` - Currently selected node
- `search_filter: Option<&str>` - Filter text (hides non-matching nodes)

### Output Messages
- `SelectNode(String)` - Click on node
- `ToggleNode(String)` - Expand/collapse node
- `NodeContextMenu(String, Point)` - Right-click on node

Used by: Branch popup (Local/Remote/Recent tree), Branches dashboard, File tree display mode.

---

## 6. ProgressBar Widget (NEW)

**File**: `src-ui/src/widgets/progress_bar.rs`

### Input State
- `operation: &str` - Operation name (e.g., "Fetching origin")
- `progress: Option<f32>` - 0.0-1.0 (None = indeterminate)
- `status_text: Option<&str>` - Additional status (e.g., "Receiving objects: 45%")

### Output Messages
- `Cancel` - User clicked cancel button

### Layout
```
┌──────────────────────────────────────────────┐
│ Fetching origin... ████████░░░░ 65% [Cancel] │
└──────────────────────────────────────────────┘
```

---

## 7. Context Menu Contracts

### File Context Menu (Changes tab)

**Unstaged file actions**:
| Action | Message | Condition |
| ------ | ------- | --------- |
| 暂存 (Stage) | `StageFile(path)` | Always |
| 显示差异 (Show Diff) | `ShowDiff(path)` | Always |
| 放弃更改 (Discard) | `DiscardFile(path)` | Not untracked |
| 显示历史 (Show History) | `ShowFileHistory(path)` | Always |
| 注解 (Annotate) | `ShowBlame(path)` | Not untracked/added |
| 复制路径 (Copy Path) | `CopyPath(path)` | Always |
| 在编辑器中打开 (Open in Editor) | `OpenInEditor(path)` | Always |

**Staged file actions**:
| Action | Message | Condition |
| ------ | ------- | --------- |
| 取消暂存 (Unstage) | `UnstageFile(path)` | Always |
| 显示差异 (Show Diff) | `ShowDiff(path)` | Always |
| 复制路径 (Copy Path) | `CopyPath(path)` | Always |
| 在编辑器中打开 (Open in Editor) | `OpenInEditor(path)` | Always |

### Commit Context Menu (Log tab)

| Action | Message | Condition |
| ------ | ------- | --------- |
| 拣选 (Cherry-Pick) | `CherryPick(hash)` | Not HEAD |
| 还原 (Revert) | `Revert(hash)` | Always |
| 从此处创建分支 (Create Branch) | `CreateBranch(hash)` | Always |
| 创建标签 (Create Tag) | `CreateTag(hash)` | Always |
| 重置当前分支到此处 (Reset to Here) | `ResetToCommit(hash)` | Not HEAD |
| 复制提交哈希 (Copy Hash) | `CopyHash(hash)` | Always |
| 在新标签页中打开 (Open in New Tab) | `OpenInNewTab(hash)` | Always |

### Branch Context Menu (popup + dashboard)

| Action | Message | Condition |
| ------ | ------- | --------- |
| 检出 (Checkout) | `CheckoutBranch(name)` | Not current |
| 从此分支新建 (New Branch From) | `NewBranchFrom(name)` | Always |
| 合并到当前 (Merge into Current) | `MergeBranch(name)` | Not current |
| 变基当前到此 (Rebase onto) | `RebaseOnto(name)` | Not current |
| 与当前比较 (Compare with Current) | `CompareBranch(name)` | Not current |
| 重命名 (Rename) | `RenameBranch(name)` | Local only |
| 删除 (Delete) | `DeleteBranch(name)` | Not current |

---

## 8. Changes Tab Layout Contract

### Overall Layout
```
┌─────────────────────────────────────────────────────────────┐
│ Toolbar: [Branch ▼] [Refresh] [Pull ▼] [Commit] [Push]     │
├────────────────────┬────────────────────────────────────────┤
│                    │                                        │
│   ChangeList       │   DiffViewer                           │
│   (staged +        │   (with stage/unstage hunk buttons)    │
│    unstaged         │                                        │
│    tree groups)    │                                        │
│                    ├────────────────────────────────────────┤
│                    │   CommitPanel                          │
│                    │   (message + amend + history + commit) │
│                    │                                        │
├────────────────────┴────────────────────────────────────────┤
│ StatusBar: [repo] [branch] [sync status] [progress]         │
└─────────────────────────────────────────────────────────────┘
```

**Split ratios**:
- ChangeList : Right panel = 5:8 (resizable)
- DiffViewer : CommitPanel = 7:3 (resizable)

---

## 9. Log Tab Layout Contract

### Overall Layout
```
┌─────────────────────────────────────────────────────────────┐
│ LogTabs: [All] [main ×] [dev ×] [+]                         │
├────────────────────────────────────────────────────────────┤
│ FilterBar: [Branch ▼] [Author ▼] [Date ▼] [Path] [Search]  │
├──────────┬─────────────────────────────────────────────────┤
│          │                                                  │
│ Branches │   CommitGraph                                    │
│ Dashboard│   (graph + hash + message + author + date)       │
│          │                                                  │
│ ▼ Local  │                                                  │
│   main   ├──────────────────────────────────────────────────┤
│   dev    │   CommitDetail                                   │
│ ▼ Remote │   (hash, author, date, message, parents,         │
│   o/main │    changed files, signature badge)               │
│          │                                                  │
├──────────┴──────────────────────────────────────────────────┤
│ StatusBar                                                    │
└─────────────────────────────────────────────────────────────┘
```

**Split ratios**:
- Branches Dashboard : Main area = 1:4 (resizable, collapsible)
- CommitGraph : CommitDetail = 6:4 (resizable)
