# Implementation Plan: PhpStorm 风格的轻量化样式收敛

**Branch**: `006-phpstorm-style-polish` | **Date**: 2026-03-23 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/006-phpstorm-style-polish/spec.md`

## Summary

将当前 `slio-git` 的仓库工作区继续从“偏厚的多层桌面卡片”收敛为更接近 PhpStorm/JetBrains Changes 面板的轻量工作台：顶部 chrome 压缩为细窄连续条带，主体视觉优先让给改动树与 diff；分支弹层改造成搜索驱动、动作优先、分组列表为主的紧凑 popup；badge、状态条、列表项和 diff 顶栏统一向“薄、紧、平”靠拢。技术上保持现有 Rust workspace 与 IntelliJ-compatible Git 算法不变，主要在 `src-ui` 的主题 token、主窗口布局、分支弹层和核心 widgets 中完成样式与密度收敛。

## Technical Context

**Language/Version**: Rust 2021+  
**Primary Dependencies**: iced 0.13（native UI）, git2 0.19, notify 8, tokio 1, rfd 0.15, log/env_logger, chrono, once_cell, syntect 6  
**Storage**: N/A（Git 仓库与本地文件系统；UI 状态以内存为主）  
**Testing**: `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, 手工 UI walkthrough（对照用户提供的 PhpStorm 截图与 IntelliJ Git 工作流）  
**Target Platform**: macOS 11+, Windows 10+, Ubuntu 20.04+  
**Project Type**: Native desktop application（Rust workspace: `git-core` library + `src-ui` UI app）  
**Performance Goals**: 启动 <300ms；打开仓库后的首屏在 1s 内稳定可交互；分支 popup 打开保持 <100ms 感知延迟；不因样式收敛引入额外常驻性能负担  
**Constraints**: Pure Rust + Iced；完整中文界面；Git 交互算法保持 IntelliJ-compatible；视觉上以用户提供的 PhpStorm 截图为基线；不得通过隐藏核心 Git 能力换取“轻量”；不复刻完整 IDE 框架，仅收敛当前仓库工作区样式  
**Scale/Scope**: 聚焦 `src-ui` 的共享主题、主窗口顶栏/主体衔接、分支 popup、改动列表、diff 顶栏、状态栏及相关输入/按钮组件；预计影响 8–12 个 UI 文件与 4 个设计文档产物

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. IntelliJ Compatibility | ⚠ CONDITIONAL PASS | 本特性主要调整视觉密度与样式层级，不变更 `git-core` 的 Git 算法。视觉基线参考用户提供的 PhpStorm 截图，但交互能力路径、动作语义和分支工作流仍需维持 IntelliJ-compatible。 |
| II. Rust + Iced Stack | ✅ PASS | 保持 Rust + Iced，不引入 WebView、Electron、Tauri 或其他运行时。 |
| III. Library-First Architecture | ✅ PASS | 计划集中在 `src-ui`；`git-core` 继续作为独立 Git 行为库存在。 |
| IV. Integration Testing for Git Parity | ✅ PASS | 本轮不改变 Git 行为，但仍要求保留 `git-core` 回归测试并补充手工 UI/能力可达性回归。 |
| V. Observability | ✅ PASS | 允许收敛 banner 和视觉反馈，但结构化日志、错误上下文和可操作错误信息必须保留。 |
| VI. 中文本地化支持 | ✅ PASS | 所有新文案、状态反馈与面板标签继续保持中文与中文字体策略。 |

**Gate Result**: 通过。唯一受控偏离是：视觉风格向 PhpStorm/JetBrains 的轻量 Changes 面板靠拢，但不会复制完整 IDE shell；IntelliJ-compatible 的重点继续落在 Git 交互能力与算法，而非逐像素 UI 复刻。

**Post-Design Re-check**: 通过。Phase 1 产物把轻量化约束落实到共享样式 token、顶部单一上下文条、列表式分支 popup 与薄型状态面，不触碰 `git-core` 边界，仍满足 Constitution II–VI；Principle I 以“算法不变、视觉收敛”的方式执行。

## Project Structure

### Documentation (this feature)

```text
specs/006-phpstorm-style-polish/
├── plan.md                            # This file
├── research.md                        # Phase 0 output
├── data-model.md                      # Phase 1 output
├── quickstart.md                      # Phase 1 output
├── contracts/
│   └── phpstorm-style-contracts.md    # UI density / popup / status contracts
└── tasks.md                           # Phase 2 output (/speckit.tasks - NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
src/
└── git-core/
    ├── Cargo.toml
    ├── src/
    │   ├── branch.rs
    │   ├── diff.rs
    │   ├── remote.rs
    │   ├── repository.rs
    │   └── lib.rs
    └── tests/
        └── workflow_regressions.rs

src-ui/
└── src/
    ├── i18n.rs
    ├── state.rs
    ├── theme.rs
    ├── views/
    │   ├── main_window.rs
    │   └── branch_popup.rs
    └── widgets/
        ├── button.rs
        ├── changelist.rs
        ├── diff_viewer.rs
        ├── split_diff_viewer.rs
        ├── statusbar.rs
        ├── text_input.rs
        └── scrollable.rs
```

**Structure Decision**: 保持当前两 crate 架构不变。`git-core` 继续负责 Git 行为与回归测试；`src-ui` 承担本次轻量化样式收敛，包括 theme token 调整、workspace chrome 压缩、分支 popup 列表化，以及 changes/diff/status 相关 widgets 的统一瘦身。

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| 不复刻完整 PhpStorm/IntelliJ IDE 壳层，只提炼其 Git/Changes 视觉基线与 popup 节奏 | 当前产品是更轻量的独立 Git 桌面工具，用户要的是“更轻、更像参考图”，不是整套 IDE 工具窗口体系 | 逐像素搬运 IDE 框架会超出产品范围；仅在现有界面上删几段文案又无法解决结构性“重”感 |
