# Implementation Plan: JetBrains-Styled Diff File List Panel

**Branch**: `003-jetbrains-diff-ui` | **Date**: 2026-03-22 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/003-jetbrains-diff-ui/spec.md`

## Summary

重构整个应用程序UI以匹配IntelliJ IDEA的Darcula主题风格和UI结构，同时实现差异文件列表功能。核心是让用户可以在一个完全复刻IDEA外观和行为的界面中查看git变更文件列表并查看差异。

## Technical Context

**Language/Version**: Rust 2021+
**Primary Dependencies**: iced 0.13 (UI framework), git2 0.19 (libgit2 bindings), notify 8 (file watching)
**Storage**: N/A (git repositories are file-based)
**Testing**: cargo test
**Target Platform**: Windows 10+, macOS 11+, Ubuntu 20.04+
**Project Type**: desktop-app (native Rust UI)
**Performance Goals**: 60fps UI rendering, <100ms perceived latency for git operations, <300ms startup
**Constraints**: Full Chinese localization required, Darcula dark theme mandatory, IDEA UI structure replication required
**Scale/Scope**: Single-user desktop application, 1-2 UI panels visible at once

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. IntelliJ Compatibility | ✅ PASS | Feature explicitly requires IDEA UI replication |
| II. Rust + Iced Stack | ✅ PASS | Confirmed: iced 0.13, pure Rust |
| III. Library-First Architecture | ✅ PASS | git-core library + Iced UI layer |
| IV. Integration Testing | ⚠️ REVIEW | Need diff view parity tests |
| V. Observability | ✅ PASS | Structured logging for git ops |
| VI. 中文本地化 | ✅ PASS | All UI text in Chinese |

**GATE Result**: ✅ PASS - All gates satisfied, proceed to Phase 0

## Project Structure

### Documentation (this feature)

```text
specs/003-jetbrains-diff-ui/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output (if needed)
└── tasks.md             # Phase 2 output (/speckit.tasks)
```

### Source Code (repository root)

```text
src/
├── main.rs              # Application entry point
├── lib.rs               # Library exports
├── ui/                  # Iced UI layer
│   ├── mod.rs
│   ├── theme.rs         # Darcula theme definition
│   ├── diff_panel.rs    # Diff file list panel
│   ├── diff_view.rs     # File diff viewer
│   └── components/     # Reusable UI components
│       ├── mod.rs
│       ├── file_tree.rs
│       └── status_icons.rs
└── git_core/            # Git operations library
    ├── mod.rs
    ├── status.rs
    └── diff.rs

src-ui/                  # Platform-specific UI entry (if separate)

tests/
├── unit/
├── integration/
└── parity/             # IntelliJ parity tests
```

**Structure Decision**: Single Rust project with two logical layers:
- `git_core` library module for all git operations (importable independently)
- `ui` module for Iced-based UI components
- `theme.rs` for Darcula color palette and styling constants

## Phase 0: Research

### Research Tasks

1. **IntelliJ IDEA Diff Panel UI Research**
   - Investigate IDEA's git diff panel structure, component hierarchy, and visual styling
   - Identify exact color codes for Darcula theme
   - Document component layout (menu bar, toolbar, file list, diff viewer split)

2. **iced Framework Darcula Implementation**
   - Research how to implement custom themes in iced 0.13
   - Document theme customization patterns (Color, Font, StyleSheet)
   - Identify any limitations in replicating IDEA's UI with iced

3. **Chinese Font Integration**
   - Verify Chinese font stacks for each platform (already defined in constitution)
   - Research font loading approach in iced

### Research Output

[See research.md](./research.md)

## Phase 1: Design & Contracts

### Data Model

[See data-model.md](./data-model.md)

### Quickstart

[See quickstart.md](./quickstart.md)

## Complexity Tracking

> No violations requiring justification at this time.

All constitutional principles are satisfied. The feature scope (full UI replication + diff panel) is necessary per user requirements.
