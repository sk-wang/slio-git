## Context

当前 revert 流程：`HistoryMessage::PrepareRevertCommit` → 加载分支 → 转发到 `BranchPopupMessage::PrepareRevertCommit` → `prepare_revert_commit()` 设置 `pending_commit_action` → 打开分支面板 → 用户手动点"继续执行" → `confirm_pending_commit_action()` → `git_core::revert_commit()`。

IDEA 的 revert 流程：右键 → Revert Commit → 直接执行 `git revert --no-edit` → 刷新 → 完成。

核心底层函数 `revert_commit()` 已经正确实现了 `git revert --no-edit`，问题纯粹在 UI 层的多余确认步骤。

## Goals / Non-Goals

**Goals:**
- "还原提交"点击后直接执行 revert，无需确认
- 成功后刷新历史视图，显示 toast 通知
- 冲突时自动跳转冲突解决视图
- 失败时在当前视图显示错误

**Non-Goals:**
- 不改变 cherry-pick/reset/push-to-commit 等其他操作的确认流程
- 不改变 `git_core::revert_commit()` 底层实现
- 不处理 merge commit 的 revert（当前已有"暂不支持"提示）

## Decisions

1. **直接执行而非确认**：在 `HistoryMessage::PrepareRevertCommit` 处理中直接调用 `git_core::revert_commit()`，不再转发到 branch_popup 的 pending_commit_action 流程
2. **保留 branch_popup 的 revert 入口**：branch_popup 右键菜单中的 revert 仍走确认流程（branch_popup 里选中的提交可能不是当前浏览的，确认更安全）
3. **错误/冲突处理**：复用现有的 `refresh_repository_after_action()` + 冲突检测逻辑

## Risks / Trade-offs

- 移除确认步骤后，误操作可以通过 `git revert` 再次 revert 来回退，风险很低
- branch_popup 和 history_view 两处的 revert 行为略有差异（一个有确认，一个没有），但这符合 IDEA 的实际行为
