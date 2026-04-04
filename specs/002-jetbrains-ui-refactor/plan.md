# Implementation Plan: JetBrains风格Git UI重构

**Branch**: `002-jetbrains-ui-refactor` | **Date**: 2026-03-22 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/002-jetbrains-ui-refactor/spec.md`

## Summary

重构 slio-git 用户界面，使其呈现类似 IntelliJ IDEA 的经典 Git 工具窗口布局：顶部工具栏、中间主内容区（变更列表+差异面板）、底部状态栏。核心功能包括：变更文件列表、差异对比面板、提交对话框、分支选择器、冲突解决与自动合并。参考 IntelliJ IDEA 源码 `~/git/intellij-community/plugins/git4idea/` 中的 `GitMergeProvider.java`、`GitMergeUtil.java`、`MultipleFileMergeDialog.kt` 等实现。

## Technical Context

**Language/Version**: Rust 2021+
**Primary Dependencies**: iced 0.13 (UI framework), git2 0.19 (libgit2 bindings), notify 8 (file watching)
**Storage**: N/A (git repositories are file-based)
**Testing**: cargo test (unit), custom integration test framework
**Target Platform**: macOS 11+, Windows 10+, Ubuntu 20.04+
**Project Type**: Desktop application (native UI, pure Rust)
**Performance Goals**: Startup <300ms, common git operations <100ms, memory <80MB
**Constraints**: IntelliJ-compatible behavior (Constitution Principle I), Pure Rust only (no Tauri/WebView)
**Scale/Scope**: 8 user stories, 12 functional requirements, ~15 source files across git-core + src-ui

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. IntelliJ Compatibility | ✅ PASS | Spec references IntelliJ IDEA behavior; conflict resolution references `GitMergeProvider.java`, `MultipleFileMergeDialog.kt` |
| II. Rust + Iced Stack | ✅ PASS | Using pure iced 0.13; no Tauri/WebView |
| III. Library-First Architecture | ✅ PASS | `git-core` crate exists as independent lib imported directly by UI layer |
| IV. Integration Testing | ✅ PASS | Integration tests required for git parity per spec SC-001~SC-009 |
| V. Observability | ✅ PASS | Structured logging implemented via `logging.rs` |

All gates pass. No violations requiring justification.

## Project Structure

### Documentation (this feature)

```text
specs/002-jetbrains-ui-refactor/
├── plan.md              # This file
├── research.md          # Phase 0 output (auto-merge algorithm research)
├── data-model.md        # Phase 1 output (entities, state machines)
├── quickstart.md        # Phase 1 output (dev setup guide)
├── contracts/           # Phase 1 output (UI component contracts)
└── tasks.md             # Phase 2 output (/speckit.tasks - NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
src/
├── git-core/            # Git operations library (library-first per Constitution III)
│   └── src/
│       ├── lib.rs       # Library entry point
│       ├── repository.rs # Repository operations
│       ├── branch.rs    # Branch management
│       ├── commit.rs    # Commit operations
│       ├── diff.rs      # Diff + three-way merge (ThreeWayDiff, ConflictHunk)
│       ├── index.rs     # Staging area
│       ├── remote.rs    # Remote operations
│       ├── stash.rs     # Stash management
│       ├── history.rs   # Commit history
│       ├── rebase.rs    # Rebase operations
│       ├── tag.rs       # Tag operations
│       └── error.rs     # Error types
│
src-ui/                 # Iced UI layer (pure Rust, no Tauri/WebView)
├── src/
│   ├── main.rs          # Application entry, window config, keyboard subscription
│   ├── state.rs         # AppState struct holding all UI state
│   ├── keyboard.rs      # Keyboard shortcuts definitions
│   ├── i18n.rs          # Internationalization (Chinese text)
│   ├── logging.rs       # Structured logging
│   ├── file_watcher.rs  # File system watcher
│   ├── thread_pool.rs   # Async task handling
│   ├── views/           # Main view components
│   │   ├── main_window.rs    # Main window with toolbar + content + statusbar
│   │   ├── commit_dialog.rs   # Commit dialog (FR-006)
│   │   ├── branch_popup.rs   # Branch selector popup (FR-005)
│   │   ├── stash_panel.rs    # Stash management panel
│   │   ├── history_view.rs   # Commit history view
│   │   ├── rebase_editor.rs  # Rebase editor
│   │   ├── remote_dialog.rs  # Remote management
│   │   └── tag_dialog.rs     # Tag management
│   └── widgets/         # Reusable UI components
│       ├── changelist.rs     # File change list with tree view (FR-003)
│       ├── diff_viewer.rs    # Basic diff viewer
│       ├── split_diff_viewer.rs  # Split-screen diff (FR-004)
│       ├── conflict_resolver.rs  # Three-way merge UI (FR-009~FR-012)
│       ├── commit_compare.rs    # Commit comparison
│       ├── button.rs, text_input.rs, scrollable.rs  # Primitives
│       └── mod.rs
│
tests/                  # Integration tests (per Constitution IV)
└── [integration test files]
```

**Structure Decision**: Two-crate architecture (`git-core` + `src-ui`). `git-core` is an independent library that can be tested without the UI. `src-ui` imports `git-core` directly with no IPC layer (per Constitution III).

## Project Structure

### Documentation (this feature)

```text
specs/[###-feature]/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)
<!--
  ACTION REQUIRED: Replace the placeholder tree below with the concrete layout
  for this feature. Delete unused options and expand the chosen structure with
  real paths (e.g., apps/admin, packages/something). The delivered plan must
  not include Option labels.
-->

```text
# [REMOVE IF UNUSED] Option 1: Single project (DEFAULT)
src/
├── models/
├── services/
├── cli/
└── lib/

tests/
├── contract/
├── integration/
└── unit/

# [REMOVE IF UNUSED] Option 2: Web application (when "frontend" + "backend" detected)
backend/
├── src/
│   ├── models/
│   ├── services/
│   └── api/
└── tests/

frontend/
├── src/
│   ├── components/
│   ├── pages/
│   └── services/
└── tests/

# [REMOVE IF UNUSED] Option 3: Mobile + API (when "iOS/Android" detected)
api/
└── [same as backend above]

ios/ or android/
└── [platform-specific structure: feature modules, UI flows, platform tests]
```

**Structure Decision**: [Document the selected structure and reference the real
directories captured above]

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| [e.g., 4th project] | [current need] | [why 3 projects insufficient] |
| [e.g., Repository pattern] | [specific problem] | [why direct DB access insufficient] |
