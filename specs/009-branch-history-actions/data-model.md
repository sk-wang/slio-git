# Data Model: 分支视图提交动作补齐

**Feature**: 分支视图提交动作补齐  
**Date**: 2026-03-25  
**Branch**: `009-branch-history-actions`

## Entities

### 1. BranchTimelineState

```rust
pub struct BranchTimelineState {
    pub scope_branch: String,
    pub entries: Vec<TimelineCommitEntry>,
    pub selected_commit: Option<String>,
    pub open_action_menu_for: Option<String>,
    pub load_state: TimelineLoadState,
}

pub enum TimelineLoadState {
    Idle,
    Loading,
    Partial,
    Failed(String),
}
```

**Purpose**: 表达“当前在分支视图里围绕哪个分支、看到了哪些提交、当前选中了哪条提交、动作菜单开在谁身上”。

### 2. TimelineCommitEntry

```rust
pub struct TimelineCommitEntry {
    pub id: String,
    pub subject: String,
    pub author_name: String,
    pub timestamp: i64,
    pub parent_ids: Vec<String>,
    pub decorations: Vec<CommitDecoration>,
    pub is_current_head: bool,
    pub is_selected_branch_tip_path: bool,
}

pub enum CommitDecoration {
    Head,
    LocalBranch(String),
    RemoteBranch(String),
    Tag(String),
}
```

**Purpose**: 描述分支视图 / 历史视图共享的单条提交行，既要承载图谱信息，也要表达当前分支、标签和远端引用装饰。

### 3. CommitActionMenuState

```rust
pub struct CommitActionMenuState {
    pub commit_id: String,
    pub groups: Vec<CommitActionGroup>,
    pub eligibility: CommitEligibilityState,
    pub pending_confirmation: Option<ConfirmationRequest>,
    pub follow_up: Option<CommitFollowUpState>,
}

pub enum CommitActionKey {
    CopyRevision,
    CreatePatch,
    CompareWithLocal,
    CheckoutCommit,
    CreateBranchFromCommit,
    CreateTagFromCommit,
    CherryPick,
    Revert,
    ResetCurrentBranch,
    PushCurrentBranchToCommit,
    Reword,
    Fixup,
    Squash,
    Drop,
    RebaseFromHere,
    UndoLastCommit,
}

pub struct CommitActionGroup {
    pub title: String,
    pub items: Vec<CommitActionItem>,
}

pub struct CommitActionItem {
    pub key: CommitActionKey,
    pub label: String,
    pub detail: Option<String>,
    pub enabled: bool,
    pub disabled_reason: Option<String>,
}

pub struct ConfirmationRequest {
    pub title: String,
    pub summary: String,
    pub impact_items: Vec<String>,
}
```

**Purpose**: 把一条提交在当前上下文中能做什么、为什么能做或不能做，以及是否需要进一步确认，统一收束到一个菜单状态对象中。

### 4. CommitEligibilityState

```rust
pub struct CommitEligibilityState {
    pub belongs_to_current_branch: bool,
    pub is_published: bool,
    pub is_merge_commit: bool,
    pub is_root_commit: bool,
    pub has_upstream: bool,
    pub repository_has_in_progress_operation: bool,
    pub rewrite_capability: RewriteCapability,
    pub disabled_reasons: Vec<ActionDisabledReason>,
}

pub enum RewriteCapability {
    NotAllowed,
    UndoLastCommit,
    Reword,
    Fixup,
    Squash,
    Drop,
    RebaseFromHere,
}

pub struct ActionDisabledReason {
    pub action: CommitActionKey,
    pub reason: String,
}
```

**Purpose**: 统一表达“为什么这个动作能做 / 不能做”，避免 UI 自己散落地推断 Git 资格。

### 5. CommitOperationRequest

```rust
pub enum CommitOperationRequest {
    CopyRevision { commit_id: String },
    CreatePatch { commit_id: String },
    CompareWithLocal { commit_id: String },
    CheckoutCommit { commit_id: String },
    CreateBranchFromCommit { commit_id: String, branch_name: String },
    CreateTagFromCommit { commit_id: String, tag_name: String },
    CherryPick { commit_id: String },
    Revert { commit_id: String },
    ResetCurrentBranch { commit_id: String },
    PushCurrentBranchToCommit { commit_id: String },
    Rewrite { commit_id: String, action: RewriteCapability },
}
```

**Purpose**: 定义分支视图向 `git-core` 发出的提交级动作请求，帮助主消息循环、确认弹层和回归测试围绕同一种请求模型协作。

### 6. PublicationTarget

```rust
pub struct PublicationTarget {
    pub remote_name: String,
    pub local_branch_name: String,
    pub upstream_ref: String,
    pub selected_commit: String,
    pub is_fast_forward: bool,
    pub requires_force_confirmation: bool,
    pub blocked_reason: Option<String>,
}
```

**Purpose**: 描述“推送到这里”到底要把哪个远端分支推进到哪个提交，以及是否允许、是否需要额外确认。

### 7. RewriteSession

```rust
pub struct RewriteSession {
    pub base_commit: String,
    pub selected_commit: String,
    pub action: RewriteCapability,
    pub affected_commits: Vec<String>,
    pub status: RewriteSessionStatus,
}

pub enum RewriteSessionStatus {
    Ready,
    Confirming,
    Running,
    PausedWithConflicts,
    Completed,
    Aborted,
}
```

**Purpose**: 为改说明、fixup、squash、drop、从这里开始整理等 rewrite 动作提供明确边界与生命周期。

### 8. CommitFollowUpState

```rust
pub struct CommitFollowUpState {
    pub operation: CommitActionKey,
    pub stage: FollowUpStage,
    pub summary: String,
    pub next_actions: Vec<String>,
}

pub enum FollowUpStage {
    Completed,
    WaitingForConfirmation,
    WaitingForConflictResolution,
    WaitingForContinueSkipAbort,
    Failed,
}
```

**Purpose**: 表达 cherry-pick、revert、reset、rewrite 等动作执行后的后续状态，帮助 UI 在失败与暂停场景中继续引导，而不是只给一次性报错。

## State Transitions

### Branch Timeline Focus Flow

```text
BranchSelected
  └─► TimelineLoaded
        └─► CommitSelected
              └─► ActionMenuOpened
```

### Commit Action Execution Flow

```text
ActionMenuOpened
  ├─► NonDestructiveActionExecuted
  ├─► ConfirmationRequested
  └─► OperationStarted
        ├─► Completed
        ├─► Failed
        └─► FollowUpRequired
```

### Rewrite Flow

```text
RewriteActionChosen
  └─► RewriteSessionCreated
        ├─► Confirmed
        ├─► Running
        ├─► PausedWithConflicts
        ├─► Continued / Skipped / Aborted
        └─► Completed
```

### Push-To-Here Flow

```text
CommitSelected
  └─► PublicationTargetResolved
        ├─► Blocked
        ├─► Confirmed
        └─► PushStarted
              ├─► Synced
              └─► Rejected / Failed
```

## Validation Rules

| Entity | Field | Rule |
|--------|-------|------|
| BranchTimelineState | scope_branch | 必须始终对应当前分支视图左侧选中的分支 |
| BranchTimelineState | selected_commit | 当时间线非空时，应始终允许快速回到一个明确的选中提交 |
| TimelineCommitEntry | decorations | 当前 HEAD、分支、远端、标签装饰必须与真实仓库状态同步刷新 |
| CommitActionMenuState | groups | 动作必须按高频优先、危险靠后分组，且每组文案保持中文 |
| CommitEligibilityState | rewrite_capability | 只有当前分支中的本地未发布提交才允许进入 rewrite 能力 |
| CommitEligibilityState | disabled_reasons | 对不可用动作必须给出原因，而不是只隐藏入口 |
| PublicationTarget | local_branch_name / upstream_ref | “推送到这里”只能针对当前分支及其上游生成目标 |
| RewriteSession | affected_commits | 必须在执行前可解释，不能让用户不知道会改动哪些提交 |
| CommitFollowUpState | next_actions | 当动作进入暂停状态时，至少要能告诉用户下一步可以继续、跳过、取消或先解决冲突 |

## Relationships

```text
BranchTimelineState "1" ──contains──► "many" TimelineCommitEntry
BranchTimelineState "1" ──opens──► "0..1" CommitActionMenuState
CommitActionMenuState "1" ──evaluates──► "1" CommitEligibilityState
CommitActionMenuState "1" ──creates──► "0..1" CommitOperationRequest
CommitOperationRequest "0..1" ──targets──► "0..1" PublicationTarget
CommitOperationRequest "0..1" ──starts──► "0..1" RewriteSession
CommitOperationRequest "0..1" ──yields──► "0..1" CommitFollowUpState
```
