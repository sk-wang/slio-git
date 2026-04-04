# Implementation Plan: IDEA 式 Git 工作台主线

**Branch**: `008-idea-lite-git` | **Date**: 2026-03-25 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/008-idea-lite-git/spec.md`

## Summary

本轮规划的目标，是把 `slio-git` 从“功能逐步补齐的 Git GUI”继续收敛成一款真正可日常使用的 Git-first 工作台：像 IDEA/PhpStorm 的 Git 面板一样，以当前仓库、当前分支、改动审阅和高频 Git 动作为绝对主线，但主动放弃代码编辑器、插件平台和通用 IDE 壳层能力。

技术上不推翻现有 Rust workspace，而是在既有 `git-core` + `src-ui` 双 crate 结构上继续演进：用稳定的主工作台承载改动审阅，用辅助视图承载历史/标签/储藏等次级上下文，用更明确的风险状态编排承接冲突、同步异常与进行中的 Git 流程，让用户尽量不必退回终端完成日常闭环。

## Technical Context

**Language/Version**: Rust 2021+  
**Primary Dependencies**: iced 0.14（native UI）, git2 0.19, notify 8, tokio 1, rfd 0.15, log/env_logger, chrono, once_cell, syntect 5.3  
**Storage**: Git 仓库与本地文件系统；最近项目与工作区记忆使用本地文件持久化  
**Testing**: `cargo check -p src-ui`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`，对照 IntelliJ/PhpStorm Git 流程的手工 walkthrough，以及基于 `quickstart.md` 的启动/常见 Git 操作性能验证  
**Target Platform**: macOS 优先，同时保持 Windows 10+、Ubuntu 20.04+ 的桌面兼容目标  
**Project Type**: Native desktop application（Rust workspace: `git-core` library + `src-ui` app）  
**Performance Goals**: 启动保持 <300ms 目标；常见 Git 操作保持 <100ms 感知延迟；最近项目恢复与仓库切换在 5 秒内回到可操作工作台  
**Constraints**: 必须保持 IntelliJ-compatible Git 交互语义；完整中文界面；不引入 Web runtime；不扩展成代码编辑器；主工作台必须始终把“看变更 + 做 Git”放在最高优先级  
**Scale/Scope**: 重点影响 `src-ui` 的主窗口、状态编排、改动树、diff、分支/提交/历史/远程/冲突等视图，以及 `git-core` 中支撑仓库状态、远端、历史、分支与冲突语义的模块

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. IntelliJ Compatibility | ✅ PASS | 本特性强调“像 IDEA 的 Git 工作台”，收敛的是产品范围而不是 Git 行为；所有高频 Git 流程仍以 IntelliJ/PhpStorm 交互语义为基线。 |
| II. Rust + Iced Stack | ✅ PASS | 继续保持 Rust + Iced 原生桌面栈，不引入 Electron/WebView/Tauri。 |
| III. Library-First Architecture | ✅ PASS | `git-core` 继续承载 Git 语义与状态推导，`src-ui` 负责工作台编排与呈现。 |
| IV. Integration Testing for Git Parity | ✅ PASS | 计划保留并扩展 `src/git-core/tests/workflow_regressions.rs`，显式覆盖 commit、branch、merge/conflict、stash 与 diff viewing 等主流程，确保行为保持 IntelliJ-compatible。 |
| V. Observability | ✅ PASS | 所有关键 Git 流程继续保留结构化日志、错误上下文和用户可理解反馈。 |
| VI. 中文本地化支持 | ✅ PASS | 工作台、风险提示、历史上下文和辅助视图都必须维持中文文案与中文字体策略。 |

**Gate Result**: 通过。当前规划把产品边界明确收敛为“Git-first IDEA Lite”，但并未偏离 Constitution 对 IntelliJ Git 兼容性、Rust/Iced 栈、库分层和中文本地化的硬约束。

**Post-Design Re-check**: 通过。Phase 1 设计产物保持“主工作台优先、辅助视图次级、风险状态站内处理”的方向，同时继续以 `git-core` 作为 Git 语义源，未引入新的架构偏离。

## Project Structure

### Documentation (this feature)

```text
specs/008-idea-lite-git/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── spec.md
├── contracts/
│   └── idea-lite-workspace-contracts.md
└── tasks.md
```

### Source Code (repository root)

```text
src/git-core/
├── src/
│   ├── branch.rs
│   ├── commit.rs
│   ├── diff.rs
│   ├── history.rs
│   ├── index.rs
│   ├── remote.rs
│   ├── repository.rs
│   ├── rebase.rs
│   ├── stash.rs
│   ├── tag.rs
│   └── lib.rs
└── tests/
    └── workflow_regressions.rs

src-ui/src/
├── main.rs
├── state.rs
├── i18n.rs
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
├── widgets/
│   ├── changelist.rs
│   ├── conflict_resolver.rs
│   ├── diff_viewer.rs
│   ├── split_diff_viewer.rs
│   ├── statusbar.rs
│   └── syntax_highlighting.rs
└── components/
    ├── rail_icons.rs
    └── status_icons.rs
```

**Structure Decision**: 保持现有双 crate 架构与 UI shell 基础，不引入新的应用层。`git-core` 继续负责仓库状态、历史、分支、远端、冲突等 Git 能力；`src-ui` 继续负责主工作台层级、视图切换、交互节奏和状态反馈。新增设计重点是重新定义“主工作台 vs 辅助能力”的边界，而不是扩展新的技术分层。

## Implementation Slices

1. **Git workspace shell**
   - 打开仓库后的单一主工作台
   - 默认直接进入 `Changes` 工作区，而不是独立 overview 首页
   - 仓库 / 分支 / 同步 / 风险上下文集中呈现
   - 主线优先于管理型入口
2. **Change review loop**
   - 改动树 / 分组 / 选中文件 / diff 预览
   - 暂存 / 取消暂存 / 放弃改动 / 提交前审阅
   - diff viewing parity 回归
   - “先看懂再操作”的节奏固定下来
3. **High-frequency Git actions**
   - 提交、抓取、拉取、推送
   - 分支浏览 / 搜索 / 切换 / 创建
   - 结果反馈、同步状态和操作后刷新
4. **Workspace continuity**
   - 最近项目
   - 上次工作上下文恢复
   - 多仓库快速切换
5. **Risk-state continuation**
   - detached HEAD、冲突、认证失败、远端异常、rebase/merge/cherry-pick/revert 中断
   - 站内解释风险状态
   - 尽量在应用内继续完成处理
6. **Context peeks**
   - 历史、标签、储藏、远端信息等次级上下文
   - stash management parity 回归
   - 快速进入 / 快速返回主工作台
   - 不打断当前 Git 主流程

## Phase Framing

### MVP

- 稳定单仓库主工作台
- 改动审阅 + 暂存/取消暂存 + 提交
- 当前分支的抓取 / 拉取 / 推送
- 最近项目与上下文恢复

### V1

- 分支工作流进一步对齐 IntelliJ/PhpStorm
- 历史、标签、储藏作为辅助上下文稳定接入
- 风险状态反馈与操作后状态同步继续收敛

### V2

- 冲突解决、rebase 中、异常恢复等复杂状态进一步深化
- “主工作台 + 辅助 peek + 风险处理台”三层工作模型稳定下来
- 让应用在大多数日常 Git 场景中成为默认主工具

## Complexity Tracking

| Decision | Why Needed | Simpler Alternative Rejected Because |
|----------|------------|--------------------------------------|
| 继续围绕单一 Git 工作台组织产品，而不是把每项能力都做成平级页面 | 符合用户“像 IDEA 一样快速看变更和做 Git”的目标 | 平级页面式信息架构会让用户不断跳转，丢失当前仓库上下文 |
| 明确把历史、标签、储藏定义成辅助上下文而不是主视觉中心 | 高频价值来自改动审阅和 Git 闭环，不是管理页浏览 | 把低频能力抬到主入口会稀释主线，首屏不再聚焦“当前改了什么” |
| 保持 `git-core` 为唯一 Git 语义源，而不是在 UI 层写更多状态规则 | 保护 IntelliJ-compatible 语义和可测试性 | UI 层散落 Git 规则会导致回归难测且更容易与 IntelliJ 行为漂移 |
| 风险状态尽量站内处理，而不是默认跳外部工具或只给报错 | 主工具价值来自异常时也能继续工作 | 只显示错误或跳出外部工具会让产品在关键时刻失去连续性 |
| 产品明确不扩展为代码编辑器 | 避免目标漂移，保持 Git-first 聚焦 | 若引入编辑器诉求，界面与能力复杂度会快速膨胀，偏离当前产品使命 |
