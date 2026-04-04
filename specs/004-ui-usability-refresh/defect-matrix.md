# Defect Matrix

## Ledger Schema

| ID | Severity | Area | Summary | Reproduction | Owner | Fix Status | Verification | Notes |
|----|----------|------|---------|--------------|-------|------------|--------------|-------|

Severity:

- `S0`: 阻断主流程
- `S1`: 严重影响可用性
- `S2`: 有明显体验或反馈缺口
- `S3`: 低风险收尾项

Fix Status:

- `open`
- `in_progress`
- `fixed`
- `blocked`

Verification:

- `not_run`
- `manual_verified`
- `automated_verified`

## Tracked Findings

| ID | Severity | Area | Summary | Reproduction | Owner | Fix Status | Verification | Notes |
|----|----------|------|---------|--------------|-------|------------|--------------|-------|
| UI-001 | S1 | `src-ui/src/main.rs` | 原主界面在 `main.rs` 内联堆砌，欢迎态和工作区缺少统一壳层，入口不清晰 | 启动应用后仅看到简单工具栏与文本块，无法形成主入口 -> 工作区的清晰路径 | Codex | fixed | manual_verified | 本轮已重构为统一 shell + sidebar + overview/changes/conflicts |
| UI-002 | S1 | `src-ui/src/state.rs` + `src/git-core/src/index.rs` | 变更分类无法区分 staged / unstaged，导致主列表含混 | 打开有修改的仓库时，文件容易同时被错误地归到多个区域或语义不清 | Codex | fixed | manual_verified | 为 `Change` 增加 `staged` / `unstaged` 标记，并在 UI 侧重新分类 |
| UI-003 | S2 | `src-ui/src/main.rs` | 文件选择后差异区缺少明确空状态与主路径引导 | 启动后未选择文件时，右侧区域只有弱提示 | Codex | fixed | manual_verified | 新差异区提供空状态、上下文件导航和一致化说明 |
| UI-004 | S2 | `src-ui/src/main.rs` | `Commit` / `Pull` / `Push` / `Stash` 入口仍未接入新壳层后的完整实现 | 在新壳层点击这些动作，仅能看到过渡性反馈 | Codex | fixed | automated_verified | 已接入 commit / remote / stash 子视图，并补充分支、历史、标签、rebase 的辅助入口；通过 `cargo check -p src-ui`、`cargo test -p src-ui` 验证编译与回归 |
| UI-005 | S2 | `src-ui/src/widgets/conflict_resolver.rs` | 冲突处理已可浏览，但选择结果尚未写回仓库 | 在冲突界面执行自动合并或分块选择后，无法完成真正的解析写回 | Codex | fixed | automated_verified | 已在 UI 层保存 hunk 选择并联动 `git-core::diff::resolve_conflict` 写回，完成后自动刷新剩余冲突；另以 `src/git-core/tests/workflow_regressions.rs` 覆盖冲突阶段读取与写回路径 |
| UI-006 | S2 | `src/git-core/src/repository.rs` + `src-ui/src/state.rs` | 仓库显示名/路径沿用了 `.git` 目录语义，打开或刷新后概览与状态栏可能展示隐藏 git 目录路径 | 打开任意仓库后观察顶部仓库标签、概览说明和状态栏，显示路径可能指向 `.git` 而不是工作区根目录 | Codex | fixed | automated_verified | `Repository::path()` / `name()` 改为优先使用 workdir，刷新后保持一致；由 `src/git-core/tests/workflow_regressions.rs` 的仓库元数据回归测试覆盖 |
| CORE-001 | S1 | `src/git-core/src/diff.rs` | 冲突解析读取 / 写回使用了错误的 stage 映射，且相对路径未落到仓库工作区 | 在冲突界面应用解决方案时，可能读取错误版本，或把解析结果写到错误路径 | Codex | fixed | automated_verified | 已修正 Git stage 映射为 `1=base / 2=ours / 3=theirs`，并按 workdir 定位真实文件再重新加入索引；由 `workflow_regressions.rs` 的冲突用例覆盖 |
| CORE-002 | S2 | `src/git-core/src/rebase.rs` | `rebase_start` 误用 `git rebase --onto <onto> -i`，与当前 UI 输入模型不匹配 | 在 Rebase 面板输入目标分支并开始时，命令参数错误导致启动失败 | Codex | fixed | automated_verified | 已改为 `git rebase <onto>`，并统一所有命令型 git-core 操作使用 worktree cwd；由 `workflow_regressions.rs` 的 rebase 用例覆盖 |
| CORE-003 | S1 | `src/git-core/src/commit.rs` | 首次提交在 unborn `HEAD` 下会直接失败 | 初始化新仓库后暂存文件并首次提交，`create_commit` 会因为 `head()` 失败而中断 | Codex | fixed | automated_verified | 已允许无父提交路径创建首个提交，避免新仓库主流程被阻断；由 `workflow_regressions.rs` 的首次提交用例覆盖 |
| QA-001 | S3 | workspace quality gates | 工作区累计的 clippy 告警会阻断 `cargo clippy --workspace --all-targets -- -D warnings`，无法把本轮改造作为完整交付关闭 | 运行 `cargo clippy --workspace --all-targets -- -D warnings`，会在 `git-core` 与 `src-ui` 内看到一组机械性告警/建议 | Codex | fixed | automated_verified | 已清理告警并补充必要的定向 `allow`；2026-03-23 重新执行 `cargo fmt --all`、`cargo clippy --workspace --all-targets -- -D warnings`、`cargo test` 全部通过。稳定版 `rustfmt` 仍会提示 `rustfmt.toml` 中的 nightly 配置项，但不影响格式化完成 |

## Closure Summary

- 截至 2026-03-23，当前 defect ledger 中的条目均已关闭，无剩余 `open` / `blocked` 状态。
- 自动化验证覆盖：`cargo clippy --workspace --all-targets -- -D warnings`、`cargo test`、`src/git-core/tests/workflow_regressions.rs`。
- 手动回归矩阵与屏幕走查说明见 `specs/004-ui-usability-refresh/quickstart.md`。
