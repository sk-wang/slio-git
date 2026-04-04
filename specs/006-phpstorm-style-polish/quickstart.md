# Quickstart: PhpStorm 风格的轻量化样式收敛

## 1. Build & Verification

在实现完成后执行：

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test
```

## 2. Launch the UI

```bash
cargo run -p src-ui
```

启动后确认：

- 中文字体正常显示
- 打开仓库后主工作区不再把 `slio-git` 作为主视觉标题
- 顶部持久 chrome 收敛为一到两层细窄条带

## 3. Visual Walkthrough

1. 在默认窗口尺寸下打开一个仓库，确认第一眼焦点是改动树与 diff，而不是卡片和工具条。
2. 观察顶部，确认仓库/分支上下文集中在一个紧凑入口中，且顶部不超过两条明显水平带。
3. 检查顶部与主体之间的衔接，确认不再存在多层厚边框、厚阴影和大圆角容器堆叠。
4. 点击当前分支入口，确认分支弹层以搜索、动作项、最近/本地/远程分组列表为主。
5. 在分支弹层中验证：
   - 当前分支可快速识别
   - 高频动作先于大量分支项出现
   - 分支项主要显示名称与必要跟踪信息
6. 返回主工作区，检查左侧改动列表项、badge 和分组标题是否更紧凑、更平。
7. 检查 diff 顶栏、底部状态区和普通反馈提示，确认它们更薄、更弱化，但仍清晰可读。
8. 测试长仓库名、长分支名、无改动、错误/冲突提示与窄窗口，确认布局保持稳定且没有重新出现大卡片空状态。

## 4. Capability Regression Matrix

| Flow | Steps | Expected Result |
|------|-------|-----------------|
| Repository open | 打开仓库 | 首屏主体连续、顶部更薄、内容成为焦点 |
| Branch popup | 点击当前分支 | 弹出轻量列表式面板 |
| Refresh | 点击刷新 | 状态反馈简洁，不出现长期厚 banner |
| Stage / unstage | 选择文件执行暂存/取消暂存 | 列表与选中态清晰，功能可达 |
| Commit | 打开提交入口并提交 | 入口仍可发现，主视图不被厚工具条占满 |
| Pull / Push | 从分支面板或相关入口执行 | 高频远程动作路径仍清晰 |
| History / Tags / Stash / Rebase | 进入相关视图 | 能力可达，风格与主工作区连续 |
| Error / Conflict | 触发失败或冲突提示 | 关键状态仍然突出，但非错误状态保持轻量 |

## 5. Comparison Notes

人工对照时分成两类基线：

- `~/git/intellij-community`：用于确认 Git 交互能力和入口语义仍保持 IntelliJ-compatible
- 用户提供的 PhpStorm 截图：用于确认视觉密度、层级、分支弹层节奏和整体“轻量感”是否收敛到位

## 6. Edge-Case Matrix

| Scenario | What to inspect | Expected Result |
|----------|-----------------|-----------------|
| 长仓库名 / 长分支名 | 顶部上下文条、分支 popup 标题区 | 文本保持单行横向滚动或紧凑截断，不把顶部 chrome 撑高 |
| 无仓库 | 欢迎页 + 底部状态区 | 维持轻量空状态，不出现大卡片堆叠 |
| 无改动 | 变更列表空状态 + diff 空状态 | 说明简短，主界面仍保持连续工作台 |
| 分支较多 | 分支 popup 三个分组滚动区 | 搜索框、高频动作与列表分区清晰，元信息不抢主文案 |
| 远程失败 / 冲突 / rebase 中断 | popup、冲突视图、rebase 视图的本地状态面 | 只在局部做高强调，稳定状态仍保持轻量 |
| 窄窗口 | 顶部两层 chrome、底部状态区、左右主面板 | 保住改动树 / diff / 当前上下文，辅助动作压缩但仍可达 |

## 7. Acceptance Mapping

| Success Criteria | Implementation Checkpoint |
|------------------|---------------------------|
| SC-001 | `src-ui/src/views/main_window.rs` 使用两层细窄 top chrome；共享高度和内边距受 `src-ui/src/theme.rs` token 约束 |
| SC-002 | `src-ui/src/main.rs` 将变更列表和 diff 面板改成低 padding、低卡片感的连续主体布局 |
| SC-003 | `src-ui/src/state.rs` 的 `WorkspaceContextStrip` 与 `LightweightStatusSurface` 保留仓库 / 分支 / 当前状态的单一入口 |
| SC-004 | `src-ui/src/views/branch_popup.rs` 以“当前分支 + 搜索 + 高频动作 + 最近 / 本地 / 远程列表”重排 popup |
| SC-005 | `src-ui/src/widgets/changelist.rs`、`src-ui/src/widgets/diff_viewer.rs`、`src-ui/src/widgets/statusbar.rs` 收敛 badge、header 和状态面重量 |
| SC-006 | `src-ui/src/views/main_window.rs` 与 popup / secondary views 继续保留提交、远程、储藏、标签、历史、rebase 入口 |
| SC-007 | 本文件的 walkthrough + edge-case matrix 用于最终人工截图对照；自动化质量门禁只覆盖编译与回归测试 |

> 注：`cargo test` 与 `cargo clippy --workspace --all-targets -- -D warnings` 已覆盖代码质量门禁；截图基线对照仍需在启动 UI 后人工确认。
