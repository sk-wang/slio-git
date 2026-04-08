## Why

slio-git 的"还原提交"（Revert Commit）行为与 IDEA 不一致。IDEA 中点击"还原提交"后直接执行 `git revert --no-edit`，立即生成一个新的反向提交并刷新历史视图。当前 slio-git 却先打开分支面板（AuxiliaryView::Branches），显示一个确认对话框，需要用户再点"继续执行"。此外，上一轮修复中发现如果没有选中任何分支，确认面板根本不渲染，导致 revert 完全无效。

## What Changes

- **移除确认步骤**：点击"还原提交"后直接执行 `git revert --no-edit <commit_id>`，不再弹出分支面板的确认对话框
- **就地刷新**：revert 完成后刷新仓库状态和历史视图，用 toast 通知结果
- **冲突处理**：如果 revert 产生冲突，自动跳转到冲突解决视图
- **错误处理**：如果 revert 失败（如工作区不干净），在原视图显示错误提示

## Capabilities

### New Capabilities

### Modified Capabilities

## Impact

- `src-ui/src/main.rs` — `HistoryMessage::PrepareRevertCommit` 和 `BranchPopupMessage::PrepareRevertCommit` 处理逻辑
- `src-ui/src/views/history_view.rs` — 右键菜单消息发送
- `src/git-core/src/commit_actions.rs` — `revert_commit()` 底层逻辑不变
- 不影响 branch_popup 的其他 pending_commit_action 功能（cherry-pick, reset 等仍走确认流程）
