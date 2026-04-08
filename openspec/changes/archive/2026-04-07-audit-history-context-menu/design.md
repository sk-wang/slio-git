## Context

IDEA 的 `Git.Log.ContextMenu` (intellij.vcs.git.xml:327-350) 定义了标准菜单结构。slio-git 的实现在 `history_view.rs:678-944`。大部分菜单项已对齐，需要补充缺失项并验证每项行为。

## Goals / Non-Goals

**Goals:**
- 补齐 IDEA 有而 slio-git 缺失的菜单项
- 验证每个已有菜单项的行为与 IDEA 一致
- 保持 slio-git 的增值功能（导出 Patch、复制哈希）

**Non-Goals:**
- 不实现 IDEA 的动态分支操作组（复杂度高，可后续迭代）
- 不改变菜单的中文分组风格

## Decisions

1. **Squash Commits 菜单项**：在"历史重写"组中新增"压缩选中提交..."，复用已有的 `SquashSelectedCommits` message variant
2. **禁用态规则**：与 IDEA 一致——非 HEAD 链上的提交禁用 reword/fixup/squash/drop；merge 提交禁用 revert；root 提交禁用 fixup/squash
3. **验证清单**：逐项测试每个菜单项的点击行为，确认不 crash、不 noop

## Risks / Trade-offs

- 动态分支操作组（Checkout/Rebase onto/Merge at commit）暂不实现，功能缺口在后续迭代补齐
