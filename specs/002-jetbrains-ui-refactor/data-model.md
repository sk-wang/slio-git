# Data Model: JetBrains风格Git UI重构

**Feature**: JetBrains风格Git UI重构
**Date**: 2026-03-22
**Branch**: `002-jetbrains-ui-refactor`

## Entities

### 1. Repository

```rust
pub struct Repository {
    pub path: PathBuf,
    pub current_branch: String,
    pub is_clean: bool,
    pub ahead: u32,
    pub behind: u32,
    pub has_remote: bool,
}
```

**States**: Open, Closed, Error
**Transitions**: Open → Error (on corruption), Error → Open (on refresh)

### 2. Change

```rust
pub enum ChangeStatus {
    Added,        // 新增
    Modified,     // 修改
    Deleted,      // 删除
    Renamed,      // 重命名
    Copied,       // 复制
    Untracked,    // 未跟踪
    Conflict,     // 冲突
}

pub struct Change {
    pub path: String,
    pub status: ChangeStatus,
    pub staged: bool,
    pub old_path: Option<String>,  // for renamed files
}
```

### 3. Branch

```rust
pub struct Branch {
    pub name: String,
    pub is_current: bool,
    pub is_remote: bool,
    pub is_local: bool,
    pub upstream: Option<String>,
}
```

### 4. Commit

```rust
pub struct Commit {
    pub hash: String,
    pub short_hash: String,
    pub author: String,
    pub author_email: String,
    pub date: DateTime<Utc>,
    pub message: String,
    pub parents: Vec<String>,
}
```

### 5. ThreeWayDiff (冲突解决)

```rust
pub struct ConflictHunk {
    pub base_lines: Vec<String>,
    pub ours_lines: Vec<String>,
    pub theirs_lines: Vec<String>,
    pub line_type: ConflictHunkType,
}

pub enum ConflictHunkType {
    Unchanged,      // 三方相同，无需处理
    OursOnly,       // 只有 Ours 修改 → 自动合并安全
    TheirsOnly,     // 只有 Theirs 修改 → 自动合并安全
    Modified,       // 双方都修改了 → 真正冲突，手动解决
    Empty,
    ConflictMarker,
}

pub struct ThreeWayDiff {
    pub path: String,
    pub hunks: Vec<ConflictHunk>,
    pub has_conflicts: bool,
    pub base_content: String,
    pub ours_content: String,
    pub theirs_content: String,
}
```

### 6. StashEntry

```rust
pub struct StashEntry {
    pub index: u32,
    pub message: String,
    pub branch: String,
    pub hash: String,
}
```

---

## State Machines

### AppState (主应用状态)

```rust
pub enum ViewMode {
    Welcome,           // 欢迎界面（未打开仓库）
    Repository,       // 仓库视图
    CommitDialog,     // 提交对话框
    BranchPopup,      // 分支选择弹窗
    ConflictResolver,  // 冲突解决视图
    HistoryView,      // 历史视图
}

pub struct AppState {
    pub view_mode: ViewMode,
    pub current_repository: Option<Repository>,
    pub changes: Vec<Change>,
    pub selected_change: Option<usize>,
    pub stashes: Vec<StashEntry>,
    pub branches: Vec<Branch>,
    pub conflict_files: Vec<ThreeWayDiff>,
    pub selected_conflict_file: Option<usize>,
}
```

### State Transitions

```
Welcome
  │
  ├─► open_repository() ──► Repository
  │
  └─► init_repository() ──► Repository

Repository
  │
  ├─► click_commit_button() ──► CommitDialog
  │
  ├─► click_branch_button() ──► BranchPopup
  │
  ├─► click_stash_button() ──► StashPanel
  │
  ├─► detect_conflicts() ──► ConflictResolver
  │
  ├─► click_history_button() ──► HistoryView
  │
  └─► close_repository() ──► Welcome

CommitDialog ──► submit() ──► Repository
CommitDialog ──► cancel() ──► Repository

BranchPopup ──► select_branch() ──► Repository
BranchPopup ──► create_branch() ──► Repository
BranchPopup ──► close() ──► Repository

ConflictResolver ──► resolve_all() ──► Repository
ConflictResolver ──► auto_merge() ──► ConflictResolver (remaining conflicts)
```

---

## Validation Rules

| Entity | Field | Rule |
|--------|-------|------|
| Change | path | 必须非空，不能包含空字符 |
| Commit | message | 提交信息不能为空 |
| Branch | name | 不能包含空格、`/`、`\`、`@` 等字符 |
| ThreeWayDiff | path | 必须对应一个存在的冲突文件 |

---

## Relationships

```
Repository "1" ──has many─► "n" Change
Repository "1" ──has many─► "n" Branch (includes remote branches)
Repository "1" ──has many─► "n" StashEntry
Repository "1" ──has many─► "n" Commit
Change "n" ──belongs to─► "1" Repository
ThreeWayDiff "n" ──belongs to─► "1" Repository
```
