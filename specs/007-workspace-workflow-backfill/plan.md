# Implementation Plan: 工作区工作流补齐与直改回填

**Branch**: `007-workspace-workflow-backfill` | **Date**: 2026-03-25 | **Spec**: [spec.md](./spec.md)
**Input**: Retrospective backfill for recent directly implemented user requests.

## Summary

本次回填覆盖一组最近直接落地、但尚未正式进入 specs 的工作区能力补齐：最近项目记忆与自动恢复、当前 upstream 远端状态自动刷新、系统凭据复用、成功 toast 反馈、diff / 冲突代码高亮、提交面板文件预览、冲突列表 + 三栏合并工作台，以及 iced 0.14 运行时升级。

这些改动共同目标不是增加更多独立页面，而是把 `slio-git` 从“能点几个 Git 按钮的轻量 GUI”往“可以连续完成日常 Git 主线任务的桌面工作区”推进一步。

## Technical Context

**Language/Version**: Rust 2021+  
**Primary Dependencies**: iced 0.14（native UI）, git2 0.19, notify 8, tokio 1, rfd 0.15, log/env_logger, chrono, once_cell, syntect 5.3  
**Storage**: Git 仓库与本地文件系统；最近项目记忆使用本地文本文件持久化  
**Testing**: `cargo check -p src-ui`, `cargo test --workspace --no-run`, `cargo test --workspace`，以及手工 UI walkthrough  
**Target Platform**: macOS 优先，兼容现有桌面平台目标  
**Project Type**: Native desktop application（Rust workspace: `git-core` + `src-ui`）  
**Performance Goals**: 工作区自动刷新与远端轮询保持节流；辅助面板打开时避免无意义刷新  
**Constraints**: 保持中文界面；不引入 Web runtime；尽量沿用现有 shell 结构；远端状态目前只聚焦当前分支 upstream  
**Scale/Scope**: 影响 `src-ui` 的主状态管理、主窗口、提交/远程/冲突视图，以及 `git-core` 的远端认证逻辑

## Constitution Check

| Principle | Status | Notes |
|-----------|--------|-------|
| I. IntelliJ Compatibility | ✅ PASS | 本轮增强集中在工作区连续性与 Git 交互补齐，没有改变 `git-core` 的库边界；冲突处理、远程认证和项目记忆都服务于更接近 IDE 的使用节奏。 |
| II. Rust + Iced Stack | ✅ PASS | UI 保持 Rust + Iced；本轮还将运行时升级到 iced 0.14。 |
| III. Library-First Architecture | ✅ PASS | 远程认证链与仓库状态能力仍下沉在 `git-core`，UI 只做编排与渲染。 |
| IV. Integration Testing for Git Parity | ✅ PASS | 保持 workspace 编译与测试可运行，并维持 `git-core/tests/workflow_regressions.rs` 作为关键 Git 回归入口。 |
| V. Observability | ✅ PASS | 成功 toast 与错误 banner 共存，用户操作后仍有清晰反馈，不会静默失败。 |
| VI. 中文本地化支持 | ✅ PASS | 项目记忆、远程反馈、提交与冲突处理文案均保持中文。 |

## Project Structure

### Documentation (this feature)

```text
specs/007-workspace-workflow-backfill/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── spec.md
└── tasks.md
```

### Source Code (relevant implementation surface)

```text
src/git-core/
├── src/
│   ├── remote.rs
│   ├── repository.rs
│   ├── diff.rs
│   └── lib.rs
└── tests/
    └── workflow_regressions.rs

src-ui/src/
├── main.rs
├── state.rs
├── theme.rs
├── views/
│   ├── main_window.rs
│   ├── remote_dialog.rs
│   └── commit_dialog.rs
└── widgets/
    ├── syntax_highlighting.rs
    └── conflict_resolver.rs
```

**Structure Decision**: 不改变现有双 crate 架构；`git-core` 负责远端认证、冲突 diff 与仓库状态基础能力，`src-ui` 负责记忆恢复、自动刷新编排、toast、差异高亮、提交预览和冲突工作台。

## Implementation Slices

1. **Workspace continuity**
   - 最近项目持久化
   - 上次仓库自动恢复
   - 左侧项目快速切换
2. **Remote reliability**
   - 当前 upstream 自动轮询
   - 远端检查节流与暂停条件
   - SSH / credential helper 凭据复用
   - 成功 toast 与状态同步
3. **Readable review surfaces**
   - unified / split diff 语法高亮
   - 提交面板文件预览
   - 冲突三栏高亮
4. **Conflict workflow depth**
   - 冲突列表摘要
   - 三栏逐块决策
   - 自动合并与整文件接受
5. **Runtime modernization**
   - iced 0.14 迁移
   - builder / theme / widget API 适配
   - 构建与 DMG 打包链继续可用

## Complexity Tracking

| Decision | Why Needed | Simpler Alternative Rejected Because |
|----------|------------|--------------------------------------|
| 最近项目记忆使用本地文本文件而不是数据库 | 足够简单，易于恢复和清理 | 引入额外持久层会让轻量桌面工具过度复杂 |
| SSH 远端优先走系统 git | 兼容系统 ssh-agent、公钥私钥和已有用户环境 | 单纯依赖 libgit2 会让 SSH 行为与用户系统环境脱节 |
| 自动远端检查只针对当前 upstream | 满足当前用户最强需求，且风险可控 | 一次性做完整多分支 outgoing/incoming 管理会显著扩大范围 |
| 通过共享语法高亮器同时服务 diff 与冲突页 | 保持渲染风格一致，减少重复实现 | 每个页面各自处理语法会放大维护成本 |
| 通过兼容层保留部分现有 Iced 组件写法 | 降低 iced 0.14 迁移成本并保护现有 UI 行为 | 一次性彻底重写所有 widgets 风险高且收益有限 |
