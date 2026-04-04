# Data Model: PhpStorm 风格的轻量化样式收敛

**Feature**: PhpStorm 风格的轻量化样式收敛  
**Date**: 2026-03-23  
**Branch**: `006-phpstorm-style-polish`

## Entities

### 1. CompactChromeProfile

```rust
pub struct CompactChromeProfile {
    pub max_visible_top_bars: u8,
    pub toolbar_height: u16,
    pub control_height: u16,
    pub container_radius: u16,
    pub section_gap: u16,
    pub content_padding: u16,
    pub elevation: ChromeElevation,
}

pub enum ChromeElevation {
    Flat,
    Subtle,
    Emphasized,
}
```

**Purpose**: 定义主工作区 chrome 的密度上限，确保“轻量化”可以被实现与验收。

### 2. WorkspaceContextStrip

```rust
pub struct WorkspaceContextStrip {
    pub repository_name: String,
    pub repository_path: String,
    pub branch_name: String,
    pub sync_hint: Option<String>,
    pub state_hint: Option<String>,
    pub secondary_label: Option<String>,
    pub overflow_behavior: OverflowBehavior,
}

pub enum OverflowBehavior {
    TruncateTail,
    HorizontalScroll,
    SecondaryLine,
}
```

**Purpose**: 表达仓库工作区顶部唯一主上下文入口，以及长文本的呈现规则。

### 3. BranchPopupLayout

```rust
pub struct BranchPopupLayout {
    pub search_query: String,
    pub current_branch: String,
    pub action_items: Vec<PopupActionItem>,
    pub recent_branches: Vec<BranchPopupEntry>,
    pub local_branches: Vec<BranchPopupEntry>,
    pub remote_branches: Vec<BranchPopupEntry>,
    pub selected_entry: Option<String>,
    pub metadata_density: MetadataDensity,
}

pub enum MetadataDensity {
    Minimal,
    Compact,
}
```

**Purpose**: 约束分支弹层必须以搜索、动作项和分组列表为核心，而非卡片式管理页。

### 4. BranchPopupEntry

```rust
pub struct BranchPopupEntry {
    pub name: String,
    pub section: BranchSection,
    pub is_current: bool,
    pub tracking_target: Option<String>,
    pub sync_hint: Option<String>,
    pub status_hint: Option<String>,
}

pub enum BranchSection {
    Recent,
    Local,
    Remote,
}
```

**Purpose**: 为分支项保留“必要且足够”的信息量，避免冗余说明堆叠。

### 5. LightweightStatusSurface

```rust
pub struct LightweightStatusSurface {
    pub message: Option<String>,
    pub severity: StatusSeverity,
    pub persistence: StatusPersistence,
    pub placement: StatusPlacement,
    pub emphasis: StatusEmphasis,
}

pub enum StatusSeverity {
    Info,
    Success,
    Warning,
    Error,
}

pub enum StatusPersistence {
    Ephemeral,
    StickyUntilDismissed,
}

pub enum StatusPlacement {
    Banner,
    Inline,
    StatusBar,
}

pub enum StatusEmphasis {
    Low,
    Medium,
    High,
}
```

**Purpose**: 统一成功、失败、加载和冲突反馈的轻重边界。

### 6. ChangesListPresentation

```rust
pub struct ChangesListPresentation {
    pub row_height: u16,
    pub group_header_height: u16,
    pub item_padding_x: u16,
    pub item_padding_y: u16,
    pub badge_style: BadgePresentation,
    pub selected_row_emphasis: SelectionEmphasis,
}

pub enum BadgePresentation {
    TextOnly,
    FlatBadge,
    EmphasizedBadge,
}

pub enum SelectionEmphasis {
    ThinHighlight,
    FilledHighlight,
}
```

**Purpose**: 约束左侧改动树/列表的密度、选中态和 badge 风格。

### 7. DiffPanePresentation

```rust
pub struct DiffPanePresentation {
    pub toolbar_height: u16,
    pub section_padding: u16,
    pub header_weight: HeaderWeight,
    pub line_density: LineDensity,
}

pub enum HeaderWeight {
    Minimal,
    Compact,
}

pub enum LineDensity {
    Comfortable,
    Compact,
}
```

**Purpose**: 确保 diff 区保留主视觉地位，同时头部和外围样式足够克制。

### 8. EmptyStatePresentation

```rust
pub struct EmptyStatePresentation {
    pub scenario: EmptyScenario,
    pub max_copy_lines: u8,
    pub primary_action: Option<String>,
    pub uses_large_card: bool,
}

pub enum EmptyScenario {
    NoRepository,
    NoChanges,
    NoBranches,
    NarrowWindow,
}
```

**Purpose**: 保证空状态与窄窗口场景也遵循轻量规则。

## State Transitions

### Workspace Chrome

```text
NoRepository
  └─► RepositoryOpened
        └─► CompactWorkspace
              ├─► BranchPopupOpened
              ├─► ChangeSelected
              ├─► DiffFocused
              └─► StatusSurfaceElevated
```

### Branch Popup

```text
Closed
  └─► Opened
        ├─► SearchUpdated
        ├─► ActionTriggered
        ├─► BranchHighlighted
        └─► Closed
```

### Status Feedback

```text
Idle
  ├─► LowEmphasisInfo
  ├─► EphemeralSuccess
  ├─► MediumWarning
  └─► HighError

EphemeralSuccess ──► Idle
MediumWarning ──► Idle / Escalated
HighError ──► Dismissed / Resolved
```

## Validation Rules

| Entity | Field | Rule |
|--------|-------|------|
| CompactChromeProfile | max_visible_top_bars | 仓库打开后必须 `<= 2` |
| CompactChromeProfile | container_radius / elevation | 默认不得形成明显厚卡片感 |
| WorkspaceContextStrip | repository_name / branch_name | 必须始终可见于同一顶部上下文区域 |
| WorkspaceContextStrip | overflow_behavior | 长仓库名/分支名必须保持布局稳定 |
| BranchPopupLayout | action_items | 必须先于大多数分支列表呈现高频动作 |
| BranchPopupEntry | tracking_target / sync_hint | 仅在有意义时显示，不能为每项填充占位说明 |
| LightweightStatusSurface | persistence | `Success/Info` 默认应为 `Ephemeral`，`Error` 可为 `StickyUntilDismissed` |
| ChangesListPresentation | row_height | 必须低于当前重卡片式列表的行高基线 |
| DiffPanePresentation | header_weight | diff 顶栏必须为 `Minimal` 或 `Compact` |
| EmptyStatePresentation | uses_large_card | 除欢迎态外默认必须为 `false` |

## Relationships

```text
CompactChromeProfile "1" ──styles──► "1" WorkspaceContextStrip
CompactChromeProfile "1" ──styles──► "1" ChangesListPresentation
CompactChromeProfile "1" ──styles──► "1" DiffPanePresentation
WorkspaceContextStrip "1" ──opens──► "1" BranchPopupLayout
BranchPopupLayout "1" ──contains──► "n" PopupActionItem
BranchPopupLayout "1" ──contains──► "n" BranchPopupEntry
LightweightStatusSurface "0..1" ──overlays──► "1" CompactChromeProfile
EmptyStatePresentation "1" ──shares density rules with──► "1" CompactChromeProfile
```

## Supporting Types

```rust
pub struct PopupActionItem {
    pub id: String,
    pub label: String,
    pub enabled: bool,
    pub emphasis: ActionEmphasis,
}

pub enum ActionEmphasis {
    Primary,
    Secondary,
    Overflow,
}
```
