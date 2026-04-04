# Data Model: 主界面可用性与视觉改造

**Feature**: 主界面可用性与视觉改造  
**Date**: 2026-03-22  
**Branch**: `004-ui-usability-refresh`

## Entities

### 1. AppShellState

```rust
pub struct AppShellState {
    pub active_screen: ScreenId,
    pub current_repository: Option<RepositoryContext>,
    pub navigation_items: Vec<NavigationItem>,
    pub primary_action: Option<PrimaryAction>,
    pub feedback: Option<FeedbackState>,
    pub theme_profile: ThemeProfile,
    pub has_blocking_issue: bool,
}
```

**Purpose**: 统一描述应用壳层当前所在界面、导航可达性、主操作和全局反馈。

### 2. ScreenId

```rust
pub enum ScreenId {
    Welcome,
    RepositoryWorkspace,
    CommitDialog,
    BranchPopup,
    StashPanel,
    HistoryView,
    RemoteDialog,
    TagDialog,
    RebaseEditor,
    ConflictResolver,
}
```

**Purpose**: 枚举仓库内所有现有 UI 界面，支撑“全应用 UI”统一改造与导航重构。

### 3. NavigationItem

```rust
pub struct NavigationItem {
    pub id: String,
    pub label: String,
    pub target: ScreenId,
    pub is_enabled: bool,
    pub badge: Option<String>,
    pub shortcut_hint: Option<String>,
}
```

**Purpose**: 描述新的导航结构、入口命名和可达性规则。

### 4. RepositoryContext

```rust
pub struct RepositoryContext {
    pub path: String,
    pub branch_name: String,
    pub sync_state: SyncState,
    pub change_summary: ChangeSummary,
    pub issue_flags: Vec<IssueFlag>,
}
```

**Purpose**: 把仓库路径、分支、同步状态、变更概览和当前问题标记集中到 UI 壳层可消费的上下文对象。

### 5. ChangeSummary

```rust
pub struct ChangeSummary {
    pub staged_count: usize,
    pub unstaged_count: usize,
    pub untracked_count: usize,
    pub conflict_count: usize,
    pub selected_path: Option<String>,
}
```

**Purpose**: 支撑主工作区、状态栏、导航角标和空状态文案。

### 6. FeedbackState

```rust
pub struct FeedbackState {
    pub kind: FeedbackKind,
    pub message: String,
    pub source: FeedbackSource,
    pub primary_follow_up: Option<String>,
}

pub enum FeedbackKind {
    Loading,
    Success,
    Warning,
    Error,
    Empty,
}
```

**Purpose**: 统一表示加载、成功、失败、空状态和告警信息，避免不同视图各自定义反馈风格。

### 7. ThemeProfile

```rust
pub struct ThemeProfile {
    pub theme_name: String,
    pub palette_family: String,
    pub typography_family: String,
    pub spacing_scale: String,
    pub component_density: String,
}
```

**Purpose**: 把 Darcula 主题的颜色、字体、间距和控件密度提升为壳层级配置实体。

### 8. DefectRecord

```rust
pub struct DefectRecord {
    pub id: String,
    pub area: String,
    pub severity: DefectSeverity,
    pub reproduction_path: String,
    pub fix_status: DefectFixStatus,
}
```

**Purpose**: 支撑“仓库内发现问题全部修”的统一跟踪与任务拆分。

### 9. IssueFlag

```rust
pub struct IssueFlag {
    pub code: String,
    pub label: String,
    pub blocks_primary_flow: bool,
}
```

**Purpose**: 在仓库上下文中标记需要 UI 反馈的异常、缺失能力或 defect 风险。

## State Machines

### App Shell Lifecycle

```text
Boot
  │
  ├─► Welcome
  │     ├─► OpenRepository ──► RepositoryWorkspace
  │     └─► InitRepository ──► RepositoryWorkspace
  │
  └─► FatalError

RepositoryWorkspace
  │
  ├─► OpenCommitDialog ──► CommitDialog
  ├─► OpenBranchPopup ──► BranchPopup
  ├─► OpenStashPanel ──► StashPanel
  ├─► OpenHistoryView ──► HistoryView
  ├─► OpenRemoteDialog ──► RemoteDialog
  ├─► OpenTagDialog ──► TagDialog
  ├─► OpenRebaseEditor ──► RebaseEditor
  ├─► DetectConflicts ──► ConflictResolver
  └─► CloseRepository ──► Welcome

Any Screen
  ├─► LoadingFeedback ──► Same Screen
  ├─► ErrorFeedback ──► Same Screen
  └─► SuccessFeedback ──► Same Screen
```

### Feedback Lifecycle

```text
Idle
  ├─► Loading
  ├─► Empty
  ├─► Warning
  ├─► Error
  └─► Success

Loading ──► Success
Loading ──► Error
Error ──► Loading (retry)
Empty ──► Loading (reload or open action)
```

### Defect Remediation Lifecycle

```text
Detected ──► Triaged ──► InProgress ──► Fixed ──► Verified
                     └─► Blocked
```

## Validation Rules

| Entity | Field | Rule |
|--------|-------|------|
| NavigationItem | label | 必须为中文，且在同一导航层级中不可重复 |
| AppShellState | active_screen | 必须映射到仓库内现有 UI 界面之一 |
| FeedbackState | message | 必须为用户可读文案，不直接暴露原始底层错误 |
| ThemeProfile | theme_name | 本 feature 中必须固定为 Darcula |
| DefectRecord | reproduction_path | 必须能指向一个可验证的界面或功能路径 |
| RepositoryContext | path | 为空时只能处于 Welcome 或 Empty/Loading 等无仓库状态 |

## Relationships

```text
AppShellState "1" ──uses──► "n" NavigationItem
AppShellState "1" ──has──► "0..1" RepositoryContext
AppShellState "1" ──has──► "0..1" FeedbackState
AppShellState "1" ──has──► "1" ThemeProfile
RepositoryContext "1" ──has──► "1" ChangeSummary
RepositoryContext "1" ──has──► "n" IssueFlag
DefectRecord "n" ──maps to──► "1" ScreenId or functional area
NavigationItem "n" ──targets──► "1" ScreenId
```
