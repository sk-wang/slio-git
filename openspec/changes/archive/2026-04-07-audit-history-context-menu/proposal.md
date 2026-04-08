## Why

用户要求逐项验证历史视图右键菜单是否一比一还原了 IDEA 的 Git Log 右键菜单 (`Git.Log.ContextMenu`)。通过对比 IDEA 源码 (`~/git/idea/plugins/git4idea/resources/intellij.vcs.git.xml` 327-350行) 和 slio-git 实现 (`history_view.rs` 678-944行)，发现以下差异需要修复。

## What Changes

### 缺失项（IDEA 有，slio-git 没有）

1. **Squash Commits...** — IDEA 有独立的多选 Squash 操作 (`Git.Squash.Commits`)，slio-git 已有 `SquashSelectedCommits` enum variant 但未接入右键菜单
2. **动态分支/标签操作** — IDEA 的 `Git.BranchOperationGroup` 根据选中提交上的 refs 动态生成：Checkout branch/tag、Rebase onto、Merge、Compare with branch 等。slio-git 只有静态的"新建分支"和"新建标签"

### 行为修正

3. **还原提交** — 已在 `fix-revert-commit-behavior` change 中修复为直接执行（与 IDEA 一致）
4. **Cherry-pick** — IDEA 里 Cherry-pick 是 `Git.CherryPick.In.Log`，与 slio-git 的行为一致，保持
5. **Fixup/Squash 命名** — IDEA: "Fixup..." / "Squash Into..."，slio-git: "Fixup 到此提交" / "Squash 到此提交"，中文翻译合理，保持

### slio-git 额外项（IDEA 没有）

6. **导出 Patch** / **复制提交哈希** — 这些是 slio-git 的增值功能，保留不删

## Capabilities

### New Capabilities

### Modified Capabilities

## Impact

- `src-ui/src/views/history_view.rs` — 右键菜单构建函数 `build_commit_context_menu_overlay()`
- `src-ui/src/main.rs` — 对应的 HistoryMessage handler
- 不影响 git-core 层
