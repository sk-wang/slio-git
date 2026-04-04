# Implementation Plan: IDEA Git 视图重构

**Branch**: `012-idea-ui-refactor` | **Date**: 2026-04-04 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/012-idea-ui-refactor/spec.md`

## Summary

重构 6 个现有 UI 视图（提交历史、分支面板、diff 预览、标签对话框、贮藏面板、变基编辑器），使其右键菜单、布局和交互与 IntelliJ IDEA 的 Git 工具窗口完全对齐。新增 3 个 git-core 操作（uncommit、keep_index stash、unstash_as）和无 diff 文件全文预览功能。

## Technical Context

**Language/Version**: Rust (edition 2021+)
**Primary Dependencies**: Iced 0.14 (UI), git2 0.19 (libgit2 bindings), notify 8 (file watching), syntect (syntax highlighting)
**Storage**: File-based (git repositories)
**Testing**: cargo test (unit + integration), 91 existing tests
**Target Platform**: macOS (primary), Linux, Windows
**Project Type**: Desktop application (native GUI)
**Performance Goals**: 全文预览 <0.5s, 菜单操作响应 <100ms, 拖拽重排 60fps
**Constraints**: <80MB memory idle, MotionSites dark theme preserved
**Scale/Scope**: Single-repository, files up to 1MB/5000 lines for preview

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
| --------- | ------ | ----- |
| I. IntelliJ Compatibility (NON-NEGOTIABLE) | PASS | 6 个视图直接对标 IDEA 的 Git.Log.ContextMenu、Git.Branch.Backend、GitTagDialog、Git.Stash.Operations、GitInteractiveRebaseDialog |
| II. Rust + Iced Stack | PASS | 纯 Rust + Iced 重构，无 WebView |
| III. Library-First Architecture | PASS | 3 个新 git-core 操作（uncommit、keep_index、unstash_as）在库层实现 |
| IV. Integration Testing for Git Parity | PASS | 每个新 git-core 操作配套集成测试 |
| V. Observability | PASS | 新操作均包含结构化日志 |
| VI. 中文本地化支持 (NON-NEGOTIABLE) | PASS | FR-015 明确要求所有菜单项使用中文标签 |

## Project Structure

### Documentation (this feature)

```text
specs/012-idea-ui-refactor/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── contracts/           # Phase 1 output
│   └── ui-contracts.md  # Menu structure contracts
└── tasks.md             # Phase 2 output
```

### Source Code (repository root)

```text
src/
└── git-core/
    └── src/
        ├── commit_actions.rs  # EXTEND: add uncommit_to_commit (soft reset range)
        ├── stash.rs           # EXTEND: add keep_index param, unstash_as_branch
        └── lib.rs             # UPDATE: export new functions

src-ui/
└── src/
    ├── main.rs              # UPDATE: new Message variants for all menu actions
    ├── i18n.rs              # UPDATE: new Chinese labels for missing menu items
    ├── views/
    │   ├── history_view.rs  # REFACTOR: commit context menu restructured by IDEA groups
    │   ├── branch_popup.rs  # REFACTOR: branch context menu restructured, add Track/Pull/Push per-branch
    │   ├── tag_dialog.rs    # REWRITE: IDEA-style layout with validate button
    │   ├── stash_panel.rs   # REWRITE: IDEA-style with Keep Index, Unstash As, Clear All
    │   └── rebase_editor.rs # REWRITE: toolbar + 3-column table + detail panel, drag-and-drop, inline edit
    └── widgets/
        ├── diff_viewer.rs   # EXTEND: full file preview for no-diff files
        ├── changelist.rs    # EXTEND: detect binary files, show preview hint
        └── menu.rs          # EXTEND: submenu support for grouped context menus
```

**Structure Decision**: No new files needed. All changes are refactors/extensions of existing views and widgets from 011. Three small git-core function additions.

## Complexity Tracking

No constitution violations requiring justification. All work extends existing architecture.
