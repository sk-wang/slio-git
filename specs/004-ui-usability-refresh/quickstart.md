# Quickstart: 主界面可用性与视觉改造

## 1. Automated Quality Gates

在 2026-03-23 收尾阶段，执行并通过以下命令：

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test
```

说明：

- `cargo fmt --all` 在稳定版工具链下会打印 `rustfmt.toml` 的 nightly 配置提示，但格式化本身成功完成。
- `cargo clippy --workspace --all-targets -- -D warnings` 现已作为强门禁通过。
- `cargo test` 覆盖 `git-core`、`src-ui` 与 `workflow_regressions`。

## 2. Launch the UI

```bash
cargo run -p src-ui
```

启动后确认：

- Darcula 深色主题生效
- 中文标签正常渲染
- 默认窗口尺寸为 `1280x800`
- 最小窗口尺寸为 `800x600`

## 3. Manual Regression Matrix

| Flow | Steps | Expected Result | Verification |
|------|-------|-----------------|--------------|
| Welcome → Workspace | 启动应用，在欢迎态点击 `打开仓库` 或 `初始化仓库` | 进入统一 shell，首屏落在 `概览` 而不是旧式散乱面板 | manual |
| Repository metadata | 打开仓库后观察顶部标签、概览文案与状态栏路径 | 仓库名称与路径指向工作区根目录，而不是 `.git` 目录 | automated + manual |
| Change selection | 切换到 `变更`，连续选择不同文件 | 右侧 diff 与上下文说明立即刷新，未选中文件时出现明确空状态 | manual |
| Stage / unstage | 在 `变更` 中执行 `暂存`、`取消暂存`、`暂存全部`、`取消暂存全部` | 文件在 staged / unstaged / untracked 分组间正确移动，并显示反馈 banner | manual |
| Auxiliary views | 依次打开 `提交`、`分支`、`历史`、`远程`、`标签`、`储藏`、`Rebase` | 所有辅助全屏视图均可进入、返回，并继承统一 Darcula 视觉语言 | manual |
| Remote actions | 在远程面板执行 `Pull` / `Push` | 可进入对应面板，错误与成功反馈可见，刷新后状态同步 | manual |
| Conflict resolution | 在冲突仓库进入 `冲突` 页并选择 `ours` / `theirs` / 自动合并 | 解析结果写回工作区文件，冲突列表自动刷新 | automated + manual |
| Rebase flow | 从 Rebase 面板输入目标分支并开始 | 命令参数正确，rebase 进入预期状态或完成 | automated |
| First commit | 初始化新仓库、暂存文件并首次提交 | 不再因 unborn `HEAD` 失败 | automated |

## 4. Screen Walkthrough Notes

按以下顺序走查整套 UI：

1. 无仓库启动：确认欢迎 Hero 区明确暴露 `打开仓库` / `初始化仓库`。
2. 打开仓库：确认顶部 chip、左侧导航、状态栏同时更新，并落在 `概览`。
3. 进入 `变更`：检查 staged / unstaged / untracked 三段式列表、选中态与空状态说明。
4. 进入 `冲突`：若仓库存在冲突，确认左侧 badge、文件切换、hunk 级别选择和整体应用入口可用。
5. 打开辅助页面：逐个验证 `提交`、`分支`、`历史`、`远程`、`标签`、`储藏`、`Rebase` 的进入与返回路径。
6. 执行刷新：确认 banner、上下文说明和变更统计一起更新，不残留旧状态。

## 5. Defect Sweep Verification

关闭本特性前，需要同时满足：

1. 所有发现的问题均已记入 `defect-matrix.md`。
2. 每个条目都有复现路径、修复动作和验证结果。
3. 自动化回归覆盖首次提交、冲突写回、rebase 启动、仓库路径显示语义。
4. defect ledger 中不存在剩余 `open` / `blocked` 项。

## 6. Completion Check

满足以下条件即可视为本特性交付完成：

- 任务清单 `T033-T042` 全部关闭
- 质量门禁命令全部通过
- 所有成功标准已在 `spec.md` 中验收
- 缺陷台账已关闭且与当前实现一致
