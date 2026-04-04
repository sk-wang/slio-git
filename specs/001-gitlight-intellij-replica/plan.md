# Implementation Plan: slio-git

**Branch**: `001-gitlight-intellij-replica` | **Date**: 2026-03-22 | **Spec**: [spec.md](./spec.md)
**Input**: 功能规格说明书 from `/specs/001-gitlight-intellij-replica/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/plan-template.md` for the execution workflow.

## Summary

slio-git 是一个跨平台的轻量级 Git 客户端，基于 Pure Rust + Iced 构建，一比一复刻 IntelliJ IDEA 的 git 交互算法和 UI 布局。核心价值在于提供与 IntelliJ 完全一致的 git 操作体验，同时保持极致的轻量级和性能优势。

## Technical Context

**Language/Version**: Rust (edition 2021+)
**Primary Dependencies**: Iced (pure Rust UI), git2-rs (libgit2 bindings), notify (file watching)
**Storage**: N/A (直接操作文件系统中的 .git 目录)
**Testing**: cargo test (单元测试), 自定义集成测试框架
**Target Platform**: Windows 10+, macOS 11+, Ubuntu 20.04+
**Project Type**: 跨平台桌面应用 (desktop-app)
**Performance Goals**: 启动 < 300ms, git 操作感知延迟 < 100ms
**Constraints**: 内存占用 < 80MB (空闲状态，无 WebView/JS 运行时)
**Scale/Scope**: 支持 50,000+ 提交的大型仓库, 10,000+ 文件的工作树

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| 原则 | 状态 | 说明 |
|------|------|------|
| I. IntelliJ 兼容性 (NON-NEGOTIABLE) | ✅ PASS | FR-018 明确要求复刻 IntelliJ 的核心 git 操作算法和 UI 布局 |
| II. Rust + Iced 技术栈 | ✅ PASS | Pure Rust 架构，无 Tauri/WebView |
| III. Library-First 架构 | ✅ PASS | git-core 库直接从 Iced UI 调用，无 IPC |
| IV. Git Parity 集成测试 | ✅ PASS | 将建立针对 IntelliJ git 行为的对比测试框架 |
| V. 可观测性 | ✅ PASS | 所有 git 操作将发出结构化日志，包含上下文信息 |

## Project Structure

### Documentation (this feature)

```text
specs/001-gitlight-intellij-replica/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
└── tasks.md            # Phase 2 output (/speckit.tasks)
```

### Source Code (repository root)

```text
slio-git/
├── src/
│   └── git-core/        # 独立的 Rust git 库 (Library-First 架构)
│       ├── src/
│       │   ├── repository.rs    # 仓库检测与管理
│       │   ├── commands.rs      # git 命令实现
│       │   ├── branch.rs        # 分支操作
│       │   ├── commit.rs        # 提交操作
│       │   ├── remote.rs        # 远程操作
│       │   ├── stash.rs         # 储藏操作
│       │   ├── diff.rs          # diff 生成
│       │   ├── history.rs       # 历史查看
│       │   ├── index.rs         # 暂存区管理
│       │   └── lib.rs           # 库入口
│       ├── tests/                # git-core 单元测试
│       └── Cargo.toml
│
├── src-ui/              # Pure Iced UI 应用 (中文界面)
│   ├── src/
│   │   ├── main.rs           # Iced Application 入口
│   │   ├── widgets/           # UI 组件 (复刻 IntelliJ 布局)
│   │   ├── views/            # 主视图
│   │   ├── state.rs          # 应用状态管理
│   │   └── i18n.rs           # 中文本地化
│   └── Cargo.toml
│
├── tests/
│   ├── integration/           # 集成测试
│   │   └── parity/           # IntelliJ parity 测试
│   └── fixtures/              # 测试用 git 仓库 fixtures
│
└── Cargo.toml           # workspace 根配置
```

**Structure Decision**: 2-crate workspace 结构:
1. `git-core`: 纯 Rust git 库，无 UI 依赖，通过 cargo test 独立测试
2. `src-ui`: Pure Iced UI 应用，直接调用 git-core API，中文界面

这符合 Constitution Principle II (Pure Iced) 和 Principle III (Library-First Architecture) 的要求。

**优势**:
- 无 Tauri 运行时开销，启动更快
- 无 IPC/RPC 层，架构更简单
- 更小的二进制体积
- 更低的内存占用

## Phase 0: Research ✅ COMPLETE

### 研究成果

| 未知项 | 决策 | 说明 |
|--------|------|------|
| Iced UI 架构 | `Widget` trait + `Application` trait | Elm 风格状态管理，委托模式构建组件 |
| git2-rs API 覆盖 | 以 git2-rs 为主，CLI 为辅 | 95% 覆盖，CLI 处理变基/worktree 等边界情况 |
| IntelliJ git 操作流程 | 参考 git4idea 模块 | GitRepositoryManager, GitBranchWorker 等核心类 |
| 跨平台文件监视 | 使用 `notify` crate | 跨平台高性能文件监视 |
| Iced 中文支持 | `Shaping::Advanced` | Iced 内置 UTF-8 支持 |

**输出**: `research.md` ✅

## Phase 1: Design & Contracts ✅ COMPLETE

**输出**:
- `data-model.md` ✅: 实体模型定义 (Repository, Branch, Commit, Change 等)
- `quickstart.md` ✅: 开发者快速入门指南

## Complexity Tracking

> 无复杂度违规需要记录。所有设计决策均符合 Constitution 约束。

---

**Plan 状态**: Phase 1 完成，准备进入 Phase 2 (/speckit.tasks)
**Generated**: 2026-03-22
