## 1. 修改历史视图的 revert 处理

- [x] 1.1 在 `main.rs` 的 `HistoryMessage::PrepareRevertCommit` handler 中，直接调用 `git_core::revert_commit()` 而非转发到 branch_popup
- [x] 1.2 成功后调用 `refresh_repository_after_action()` 刷新仓库状态
- [x] 1.3 成功时显示 toast 通知 "已还原提交 <short_hash>"
- [x] 1.4 冲突时自动跳转到冲突解决视图（通过 refresh_repository_after_action prefer_conflicts=true）
- [x] 1.5 失败时（dirty worktree / merge commit）显示错误消息

## 2. 验证

- [x] 2.1 `cargo build --release` 编译通过
- [ ] 2.2 在真实仓库中测试：选中一个普通提交 → 右键 → 还原提交 → 验证立即生成 revert commit
