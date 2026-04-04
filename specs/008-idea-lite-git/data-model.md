# Data Model: IDEA 式 Git 工作台主线

**Feature**: IDEA 式 Git 工作台主线  
**Date**: 2026-03-25  
**Branch**: `008-idea-lite-git`

## Entities

### 1. GitWorkspaceShell

```rust
pub struct GitWorkspaceShell {
    pub repository_name: String,
    pub repository_path: String,
    pub branch_name: String,
    pub sync_hint: Option<String>,
    pub risk_state: Option<RiskState>,
    pub primary_section: PrimarySection,
    pub auxiliary_view: Option<AuxiliaryView>,
}

pub enum PrimarySection {
    Changes,
    Conflicts,
}
```

**Purpose**: 表达“打开仓库后用户真正停留的主工作台”，作为所有高频 Git 活动的稳定容器。

### 2. RepositoryContext

```rust
pub struct RepositoryContext {
    pub repository_name: String,
    pub branch_name: String,
    pub upstream_name: Option<String>,
    pub sync_state: Option<String>,
    pub operation_in_progress: Option<String>,
    pub last_refresh_at: Option<String>,
}
```

**Purpose**: 用最小但完整的上下文帮助用户先判断“我现在在哪、仓库处于什么状态、适合做什么”。

### 3. ChangeReviewSurface

```rust
pub struct ChangeReviewSurface {
    pub visible_groups: Vec<ChangeGroup>,
    pub selected_file: Option<String>,
    pub diff_mode: DiffMode,
    pub has_preview: bool,
    pub staging_actions_enabled: bool,
}

pub enum ChangeGroup {
    Unstaged,
    Staged,
    Conflicted,
    Untracked,
}

pub enum DiffMode {
    Unified,
    Split,
}
```

**Purpose**: 承载“看变更 → 做判断 → 执行动作”的第一现场，是产品的核心阅读与决策面。

### 4. GitActionFlow

```rust
pub struct GitActionFlow {
    pub commit_available: bool,
    pub remote_actions_available: bool,
    pub branch_actions_available: bool,
    pub last_action_result: Option<ActionResultFeedback>,
}

pub struct ActionResultFeedback {
    pub level: FeedbackLevel,
    pub title: String,
    pub detail: Option<String>,
    pub next_step_hint: Option<String>,
}
```

**Purpose**: 统一描述用户在主工作台中可触发的高频 Git 动作，以及动作结果如何回到工作流里。

### 5. ProjectMemory

```rust
pub struct ProjectMemory {
    pub last_open_repository: Option<String>,
    pub recent_projects: Vec<ProjectMemoryEntry>,
}

pub struct ProjectMemoryEntry {
    pub name: String,
    pub path: String,
    pub last_seen_at: Option<String>,
}
```

**Purpose**: 支撑 IDE 式连续工作体验，帮助用户快速回到最近仓库或切换到其他项目。

### 6. RiskState

```rust
pub enum RiskState {
    DetachedHead,
    Conflict,
    MergeInterrupted,
    RebaseInProgress,
    CherryPickInProgress,
    RevertInProgress,
    RemoteRejected,
    AuthenticationFailed,
    WorkingTreeStale,
}
```

**Purpose**: 标识会打断正常 Git 节奏、需要额外解释和处理入口的风险状态。

### 7. RiskResolutionSession

```rust
pub struct RiskResolutionSession {
    pub state: RiskState,
    pub entry_label: String,
    pub summary: String,
    pub available_actions: Vec<String>,
    pub can_return_to_workspace: bool,
}
```

**Purpose**: 把“风险状态”从普通报错提升为有入口、有说明、有后续动作的正式工作面。

### 8. AuxiliaryPeekSurface

```rust
pub struct AuxiliaryPeekSurface {
    pub surface: AuxiliarySurfaceKind,
    pub entry_context: String,
    pub return_target: PrimarySection,
    pub preserves_selection: bool,
}

pub enum AuxiliarySurfaceKind {
    History,
    Tags,
    Stashes,
    Remotes,
    Branches,
}
```

**Purpose**: 定义历史、标签、储藏、远端、分支等辅助视图的存在方式：可进入、可判断、可快速返回。

## State Transitions

### Main Workspace Loop

```text
NoRepository
  └─► RepositoryOpened
        └─► GitWorkspaceShell
              ├─► ChangeReviewSurfaceFocused
              ├─► GitActionTriggered
              ├─► AuxiliaryPeekOpened
              └─► RiskStateDetected
```

### High-Frequency Action Loop

```text
ChangeSelected
  └─► StageOrUnstage
        └─► CommitPrepared
              └─► RemoteActionTriggered
                    └─► FeedbackShown
                          └─► WorkspaceRefreshed
```

### Risk-State Continuation

```text
RiskStateDetected
  ├─► ExplanationVisible
  ├─► ResolutionSessionOpened
  ├─► ActionTaken
  └─► BackToWorkspace / StillBlocked
```

### Auxiliary Peek Flow

```text
WorkspaceFocused
  └─► AuxiliaryPeekOpened
        ├─► ContextReviewed
        └─► BackToWorkspace
```

## Validation Rules

| Entity | Field | Rule |
|--------|-------|------|
| GitWorkspaceShell | primary_section | 打开仓库后必须默认落在 `Changes`，而不是独立首页或辅助视图 |
| RepositoryContext | repository_name / branch_name | 必须始终可见于主工作台的主要上下文区域 |
| ChangeReviewSurface | selected_file | 当存在可审阅改动时，必须能快速定位并切换目标文件 |
| ChangeReviewSurface | visible_groups | 必须能区分至少未暂存、已暂存、冲突或其他重要变更类别 |
| GitActionFlow | last_action_result | 所有改变仓库状态的动作都必须回写明确反馈 |
| ProjectMemory | recent_projects | 必须保持去重、最近优先，并在路径失效时安全降级 |
| RiskResolutionSession | available_actions | 风险状态不能只有报错，必须至少有解释或下一步动作 |
| AuxiliaryPeekSurface | return_target | 用户从辅助上下文返回后必须快速回到之前的主流程位置 |

## Relationships

```text
GitWorkspaceShell "1" ──contains──► "1" RepositoryContext
GitWorkspaceShell "1" ──focuses──► "1" ChangeReviewSurface
GitWorkspaceShell "1" ──uses──► "0..1" GitActionFlow
GitWorkspaceShell "1" ──restores-from──► "0..1" ProjectMemory
GitWorkspaceShell "1" ──elevates──► "0..1" RiskResolutionSession
GitWorkspaceShell "1" ──opens──► "0..1" AuxiliaryPeekSurface
```
