# Data Model: IDEA 风格的极简 Git 工作台

**Feature**: IDEA 风格的极简 Git 工作台  
**Date**: 2026-03-23  
**Branch**: `005-idea-minimal-shell`

## Entities

### 1. WorkspaceContextSwitcher

```rust
pub struct WorkspaceContextSwitcher {
    pub repository_name: String,
    pub repository_path: String,
    pub branch_name: String,
    pub sync_status: Option<String>,
    pub has_incoming: bool,
    pub has_outgoing: bool,
    pub trigger_label: String,
}
```

**Purpose**: 主工作区中唯一高优先级上下文入口，表达当前仓库与当前分支，并作为打开动作面板的触发器。

### 2. PrimaryWorkspaceChrome

```rust
pub struct PrimaryWorkspaceChrome {
    pub show_product_title: bool,
    pub show_persistent_banner: bool,
    pub show_context_summary: bool,
    pub visible_toolbar_actions: Vec<PrimaryToolbarAction>,
}
```

**Purpose**: 约束主工作区哪些元素允许长期可见，确保“精简”是可被建模和验证的。

### 3. PrimaryToolbarAction

```rust
pub enum PrimaryToolbarAction {
    Refresh,
    Commit,
    ShowDiffModeToggle,
    OpenContextSwitcher,
}
```

**Purpose**: 定义允许常驻在主工作区一级工具栏上的高频动作集合。

### 4. BranchActionsPanel

```rust
pub struct BranchActionsPanel {
    pub search_query: String,
    pub current_branch: String,
    pub common_actions: Vec<PanelAction>,
    pub recent_branches: Vec<BranchEntry>,
    pub local_branches: Vec<BranchEntry>,
    pub remote_branches: Vec<BranchEntry>,
    pub selected_entry: Option<String>,
}
```

**Purpose**: 承载分支与次要 Git 动作的集中式弹层模型。

### 5. BranchEntry

```rust
pub struct BranchEntry {
    pub name: String,
    pub group: BranchGroup,
    pub is_current: bool,
    pub is_favorite: bool,
    pub tracking_target: Option<String>,
    pub incoming_outgoing_hint: Option<String>,
}

pub enum BranchGroup {
    Recent,
    Local,
    Remote,
}
```

**Purpose**: 支撑类似 IDEA 的最近/本地/远程分组和分支状态提示。

### 6. PanelAction

```rust
pub struct PanelAction {
    pub id: String,
    pub label: String,
    pub kind: PanelActionKind,
    pub emphasis: ActionEmphasis,
    pub is_enabled: bool,
}

pub enum PanelActionKind {
    Refresh,
    Commit,
    Pull,
    Push,
    NewBranch,
    Checkout,
    OpenHistory,
    OpenTags,
    OpenStashes,
    OpenRemotes,
    OpenRebase,
}

pub enum ActionEmphasis {
    Primary,
    Secondary,
    Overflow,
}
```

**Purpose**: 区分上下文面板中的高频动作、次级动作与更深层入口。

### 7. MinimalFeedbackState

```rust
pub struct MinimalFeedbackState {
    pub kind: FeedbackKind,
    pub message: String,
    pub source: FeedbackSource,
    pub persistence: FeedbackPersistence,
}

pub enum FeedbackPersistence {
    Ephemeral,
    StickyUntilDismissed,
}
```

**Purpose**: 控制哪些反馈短时出现、哪些必须持续显示，避免长期占位。

### 8. WorkspaceLayoutFocus

```rust
pub struct WorkspaceLayoutFocus {
    pub primary_pane: WorkspacePane,
    pub secondary_pane: Option<WorkspacePane>,
    pub selected_change_path: Option<String>,
    pub occupies_majority_area: bool,
}

pub enum WorkspacePane {
    ChangesTree,
    DiffViewer,
    CommitInput,
    ConflictEditor,
}
```

**Purpose**: 验证主工作区是否仍将改动与差异作为主焦点，而不是让装饰性元素占据主体。

## State Transitions

### Main Workspace Focus

```text
Welcome
  └─► RepositoryOpened
        └─► MinimalWorkspace
              ├─► ChangeSelected
              ├─► ContextSwitcherOpened
              ├─► AuxiliaryViewOpened
              └─► FeedbackVisible
```

### Context Switcher Lifecycle

```text
Closed
  └─► Opened
        ├─► SearchUpdated
        ├─► ActionTriggered
        ├─► BranchSelected
        └─► Closed
```

### Feedback Lifecycle

```text
Idle
  ├─► EphemeralInfo
  ├─► EphemeralSuccess
  ├─► StickyWarning
  └─► StickyError

Ephemeral* ──► Idle
Sticky* ──► Dismissed / Resolved
```

## Validation Rules

| Entity | Field | Rule |
|--------|-------|------|
| WorkspaceContextSwitcher | repository_name / branch_name | 必须在仓库打开后始终可见于同一主上下文入口 |
| PrimaryWorkspaceChrome | show_product_title | 仓库工作区默认必须为 `false` |
| PrimaryToolbarAction | visible_toolbar_actions | 仅允许高频动作常驻；次要动作必须通过面板或二级入口触发 |
| BranchActionsPanel | search_query | 分支数量较多时必须支持快速过滤 |
| BranchEntry | group | 仅允许 `Recent` / `Local` / `Remote` 三类顶层分组 |
| MinimalFeedbackState | persistence | 成功类反馈默认应为 `Ephemeral`，错误与阻断类可为 `StickyUntilDismissed` |
| WorkspaceLayoutFocus | occupies_majority_area | 主工作区中改动树和差异区必须占主要面积 |

## Relationships

```text
WorkspaceContextSwitcher "1" ──opens──► "1" BranchActionsPanel
PrimaryWorkspaceChrome "1" ──contains──► "n" PrimaryToolbarAction
BranchActionsPanel "1" ──contains──► "n" PanelAction
BranchActionsPanel "1" ──contains──► "n" BranchEntry
MinimalFeedbackState "0..1" ──overlays──► "1" PrimaryWorkspaceChrome
WorkspaceLayoutFocus "1" ──describes──► "1" PrimaryWorkspaceChrome
```
