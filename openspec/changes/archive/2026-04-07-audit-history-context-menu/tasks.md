## 1. 验证已有菜单项行为（代码 + IDEA 对比）

- [x] 1.1 **重置当前分支到此处** — 走确认面板 ✓ 但硬编码 `--hard`，IDEA 有 soft/mixed/hard 选择 ⚠️（记录为后续迭代）
- [x] 1.2 **还原提交** — 直接执行 `git revert` ✓ 与 IDEA 一致
- [x] 1.3 **撤销提交** — soft reset 到 parent ✓ 与 IDEA 一致
- [x] 1.4 **修改提交消息...** — rebase reword 流程 ✓ InProgress 时打开 amend 面板 ✓ 与 IDEA 一致
- [x] 1.5 **Fixup 到此提交** — 直接执行 rebase fixup ✓ 冲突时打开 rebase 面板 ✓ 与 IDEA 一致
- [x] 1.6 **Squash 到此提交** — 直接执行 rebase squash ✓ 与 IDEA 一致
- [x] 1.7 **丢弃提交** — 直接执行 rebase drop ✓ 冲突处理 ✓ 与 IDEA 一致
- [x] 1.8 **交互式变基...** — 打开 rebase editor ✓ 与 IDEA 一致
- [x] 1.9 **推送到此提交** — **已修复**: 改为走确认面板 (`PreparePushCurrentBranchToCommit`)，与 IDEA 弹 dialog 确认一致
- [x] 1.10 **新建分支...** — 转发到 branch_popup 输入框 ✓ 与 IDEA 一致
- [x] 1.11 **新建标签...** — 转发到 tag dialog ✓ 与 IDEA 一致
- [x] 1.12 **Cherry-pick** — **已修复**: 改为直接执行（不再走确认面板），与 IDEA 一致
- [x] 1.13 **复制提交哈希** — 复制到剪贴板 + toast ✓
- [x] 1.14 **导出 Patch** — 弹文件选择器 + 导出 ✓

## 2. 已修复的行为差异

- [x] 2.1 **Cherry-pick 多余确认步骤** — 改为直接执行 `git cherry_pick_commit()`，与 IDEA 一致
- [x] 2.2 **推送到此提交无确认** — 菜单改为 `PreparePushCurrentBranchToCommit`（走 branch_popup 确认），与 IDEA 弹 push dialog 一致
- [x] 2.3 merge commit 禁用"还原提交" — 已在 line 808 实现 ✓

## 3. 已知差距（后续迭代）

- [ ] 3.1 **重置模式选择** — 当前硬编码 `--hard`，IDEA 的 `GitNewResetDialog` 支持 soft/mixed/hard 选择
- [ ] 3.2 **Squash Commits（多选）** — IDEA 有 `Git.Squash.Commits` 多选压缩，slio-git 有 enum variant 但未接入 UI
- [ ] 3.3 **动态分支操作组** — IDEA 的 `Git.BranchOperationGroup` 在提交上动态显示 Checkout/Rebase onto/Merge 等

## 4. 编译验证

- [x] 4.1 `cargo build --release` 编译通过 ✓
