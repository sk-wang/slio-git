# Implementation Plan: IDEA 风格的极简 Git 工作台

**Branch**: `005-idea-minimal-shell` | **Date**: 2026-03-23 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/005-idea-minimal-shell/spec.md`

## Summary

将当前 `slio-git` 的仓库工作区从“多层标题、芯片、说明、常驻按钮并列堆叠”的壳层，收敛为更接近 IntelliJ IDEA Changes/Git 工作流的极简桌面工作台：主界面优先展示仓库/分支上下文、改动树和差异预览；分支切换与次要 Git 动作集中收纳到上下文切换器/弹层；成功与失败反馈改为更短、更克制的表达。技术上保持现有 Rust workspace 不变，由 `src-ui` 重组 shell、导航和视觉层级，`git-core` 继续维持 Git 行为边界和 IntelliJ-compatible 算法。

## Technical Context

**Language/Version**: Rust 2021+  
**Primary Dependencies**: iced 0.13（native UI）, git2 0.19, notify 8, tokio 1, rfd 0.15, log/env_logger, chrono, once_cell  
**Storage**: N/A（Git 仓库与本地文件系统；UI 状态以内存为主）  
**Testing**: `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, 手工 UI walkthrough，对照 IDEA 的视觉/交互回归检查  
**Target Platform**: macOS 11+, Windows 10+, Ubuntu 20.04+  
**Project Type**: Native desktop application（Rust workspace: `git-core` library + `src-ui` UI app）  
**Performance Goals**: 启动 <300ms；打开仓库后首屏主工作区在 1s 内稳定可交互；上下文切换器/分支弹层在 <100ms 感知延迟内出现；保持空闲内存 <80MB  
**Constraints**: Pure Rust + Iced；中文界面必须完整；Git 交互算法保持 IntelliJ 兼容；简化的是 UI 常驻元素与入口层级，不是核心 Git 能力；不得通过隐藏核心能力来换取“简洁”  
**Scale/Scope**: 聚焦 `src-ui` 的应用壳层、主工作区、分支/动作入口、反馈层与相关辅助视图入口重组；主要影响 10–15 个 UI 文件与 4 个设计文档产物

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. IntelliJ Compatibility | ⚠ CONDITIONAL PASS | 本特性对齐 IntelliJ 的 Git/Changes 交互节奏与动作组织方式，但不追求复刻完整 IDE 壳层。Git 算法、能力语义、常见动作顺序和分支操作逻辑保持 IntelliJ-compatible；简化的是当前产品中过量常驻的 UI chrome。 |
| II. Rust + Iced Stack | ✅ PASS | 保持当前 Rust + Iced 栈，不引入其他 UI 运行时。 |
| III. Library-First Architecture | ✅ PASS | `git-core` 继续负责 Git 行为；本轮主要在 `src-ui` 侧重组壳层与入口。 |
| IV. Integration Testing for Git Parity | ✅ PASS | 计划保留现有 `git-core` 回归测试，并增加手工 walkthrough，确保入口收纳不影响关键 Git 流程完成。 |
| V. Observability | ✅ PASS | 需要把反馈层由“常驻 banner”改为更克制表达，但仍要保留结构化日志与关键失败上下文。 |
| VI. 中文本地化支持 | ✅ PASS | 所有新入口、上下文切换器和弹层仍保持中文文案与中文字体策略。 |

**Gate Result**: 通过。唯一受控偏离是：不复刻 IntelliJ 的完整 IDE 框架与工具窗口体系，而是提炼其 Git/Changes 交互原则，应用到当前更小范围的原生桌面 Git 工具中。

**Post-Design Re-check**: 通过。Phase 1 设计将“单一上下文入口 + 渐进展开次要动作 + 主工作区聚焦改动与差异”落实到 `src-ui` 壳层，不触碰 `git-core` 的 Git 行为边界，仍满足 Constitution II–VI；Principle I 以“算法一致、界面更克制”的方式执行。

## Project Structure

### Documentation (this feature)

```text
specs/005-idea-minimal-shell/
├── plan.md                         # This file
├── research.md                     # Phase 0 output
├── data-model.md                   # Phase 1 output
├── quickstart.md                   # Phase 1 output
├── contracts/
│   └── minimal-shell-contracts.md  # UI shell / context switcher / feedback contracts
└── tasks.md                        # Phase 2 output (/speckit.tasks - NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
src/
└── git-core/
    ├── Cargo.toml
    ├── src/
    │   ├── lib.rs
    │   ├── repository.rs
    │   ├── branch.rs
    │   ├── commit.rs
    │   ├── remote.rs
    │   ├── stash.rs
    │   ├── history.rs
    │   ├── tag.rs
    │   └── rebase.rs
    └── tests/
        └── workflow_regressions.rs

src-ui/
├── Cargo.toml
└── src/
    ├── main.rs
    ├── state.rs
    ├── i18n.rs
    ├── logging.rs
    ├── theme.rs
    ├── views/
    │   ├── main_window.rs
    │   ├── branch_popup.rs
    │   ├── commit_dialog.rs
    │   ├── history_view.rs
    │   ├── remote_dialog.rs
    │   ├── stash_panel.rs
    │   ├── tag_dialog.rs
    │   └── rebase_editor.rs
    └── widgets/
        ├── changelist.rs
        ├── statusbar.rs
        ├── button.rs
        ├── scrollable.rs
        └── conflict_resolver.rs
```

**Structure Decision**: 保持当前两 crate 架构。`src-ui` 承担极简壳层、上下文切换器、动作收纳、反馈层瘦身与视图入口重组；`git-core` 只在必要时提供更适合最小化上下文展示的仓库/分支信息，不接收新的 UI 负担。

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| 不复刻 IntelliJ 完整工具窗口与 IDE 框架，只抽取其 Git/Changes 交互原则 | 当前产品是更轻量的 Git 桌面工具，用户也明确要求“尽量精简”，不希望出现多余壳层元素 | 逐步继续在现有 004 壳层上删点文案会留下结构性冗余，且完整移植 IDEA 工具窗口体系会超出产品范围与用户需要 |
