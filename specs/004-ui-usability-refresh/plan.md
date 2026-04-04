# Implementation Plan: 主界面可用性与视觉改造

**Branch**: `004-ui-usability-refresh` | **Date**: 2026-03-22 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/004-ui-usability-refresh/spec.md`

## Summary

对 `slio-git` 的全部现有 UI 界面进行一次完整的 Darcula 风格重构，允许重做导航、入口命名和主要操作流，同时修复在仓库范围内发现的现有功能问题。技术实现保持当前两 crate 架构不变：所有 Git 行为与算法继续由 `git-core` 承担，`src-ui` 负责新的应用壳层、导航结构、统一主题系统、状态反馈和各视图的一致化改造。设计上参考 JetBrains 桌面工具的视觉语言，但对信息层级和操作路径允许按当前产品可用性目标重新组织。

## Technical Context

**Language/Version**: Rust 2021+  
**Primary Dependencies**: iced 0.13 (native UI), git2 0.19, notify 8, tokio 1, rfd 0.15, log/env_logger, chrono, once_cell  
**Storage**: N/A（Git 仓库与本地文件系统；UI 状态以内存为主）  
**Testing**: `cargo test`, `cargo clippy`, `git-core` 集成/回归测试，UI 手工验收矩阵，重构后视图与流程回归验证  
**Target Platform**: macOS 11+, Windows 10+, Ubuntu 20.04+  
**Project Type**: Native desktop application（Rust workspace: `git-core` library + `src-ui` UI app）  
**Performance Goals**: 启动 <300ms；常见导航与状态切换保持 <100ms 感知延迟；打开仓库后主工作区在 1s 内完成首屏渲染；空闲内存 <80MB  
**Constraints**: Pure Rust + Iced；中文本地化必须完整；统一 Darcula 深色主题；Git 交互算法保持 IntelliJ 兼容；允许 UI 壳层与导航重构；仓库范围问题修复并入本次交付  
**Scale/Scope**: 6 个用户故事、19 条功能需求；覆盖 `src-ui` 所有现有 views/widgets/components 与相关 `git-core` 缺陷；涉及 30+ 现有源码文件与新增回归验证文档/测试资产

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. IntelliJ Compatibility | ⚠ CONDITIONAL PASS | 用户已明确批准允许重做导航、入口命名和主要操作流。该偏离只适用于应用壳层和信息架构；Git 行为、快捷键语义、冲突处理算法及核心工作流仍必须保持 IntelliJ 兼容。 |
| II. Rust + Iced Stack | ✅ PASS | 保持当前 Rust workspace 与 Iced 0.13，不引入 WebView/Tauri/Electron。 |
| III. Library-First Architecture | ✅ PASS | `git-core` 继续承载 Git 行为；UI 重构不将业务逻辑移入 `src-ui`。 |
| IV. Integration Testing for Git Parity | ✅ PASS | 计划包含 Git 行为回归、视图烟雾验证与跨界面手工验收矩阵，确保 UI 改造不破坏 Git parity。 |
| V. Observability | ✅ PASS | 需要扩展 UI 侧导航、加载、失败和 defect-fix 相关日志，但现有 `logging.rs` 可承载。 |
| VI. 中文本地化支持 | ✅ PASS | 全应用保持中文文案与平台中文字体；Darcula 风格改造不能破坏中文可读性。 |

**Gate Result**: 通过。存在 1 个经用户明确批准的 UI 壳层偏离：允许重做导航、入口命名和主要操作流，但不能改变 `git-core` 的核心 Git 交互算法与 IntelliJ parity 要求。

**Post-Design Re-check**: 通过共享主题令牌、统一状态反馈模型、壳层导航重构与 `git-core` 边界保持，设计仍满足 Constitution II–VI；Principle I 继续以“壳层可变、Git 行为不变”的受控偏离执行。

## Project Structure

### Documentation (this feature)

```text
specs/004-ui-usability-refresh/
├── plan.md                      # This file
├── research.md                  # Phase 0 output
├── data-model.md                # Phase 1 output
├── quickstart.md                # Phase 1 output
├── contracts/
│   └── ui-redesign-contracts.md # UI shell / feedback / navigation contracts
└── tasks.md                     # Phase 2 output (/speckit.tasks - NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
src/
└── git-core/
    ├── Cargo.toml
    ├── src/
    │   ├── lib.rs
    │   ├── repository.rs
    │   ├── index.rs
    │   ├── diff.rs
    │   ├── branch.rs
    │   ├── commit.rs
    │   ├── remote.rs
    │   ├── stash.rs
    │   ├── history.rs
    │   ├── rebase.rs
    │   ├── tag.rs
    │   └── error.rs
    └── tests/
        └── test_helpers.rs

src-ui/
├── Cargo.toml
└── src/
    ├── main.rs
    ├── state.rs
    ├── theme.rs
    ├── i18n.rs
    ├── logging.rs
    ├── keyboard.rs
    ├── file_watcher.rs
    ├── thread_pool.rs
    ├── components/
    │   ├── mod.rs
    │   └── status_icons.rs
    ├── views/
    │   ├── main_window.rs
    │   ├── commit_dialog.rs
    │   ├── branch_popup.rs
    │   ├── stash_panel.rs
    │   ├── history_view.rs
    │   ├── remote_dialog.rs
    │   ├── tag_dialog.rs
    │   ├── rebase_editor.rs
    │   └── mod.rs
    └── widgets/
        ├── button.rs
        ├── text_input.rs
        ├── scrollable.rs
        ├── changelist.rs
        ├── diff_viewer.rs
        ├── split_diff_viewer.rs
        ├── conflict_resolver.rs
        ├── commit_compare.rs
        ├── file_picker.rs
        ├── statusbar.rs
        └── mod.rs
```

**Structure Decision**: 保持现有两 crate 架构。`src-ui` 承担全应用壳层、导航、统一主题、反馈和视图重构；`git-core` 保持 Git 操作与算法边界，并承接在仓库范围内发现的相关功能修复。这样既满足 Constitution III，也避免 UI 重构把 Git 逻辑拉进视图层。

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| UI 壳层不再强制复刻 IntelliJ 原有导航与入口布局 | 用户明确要求可自由重做导航、入口命名和主要操作流，以解决当前“太丑且不好用”的问题 | 保持 IntelliJ 壳层布局会限制本项目现状的可用性修复，也无法兑现用户已批准的范围 |
