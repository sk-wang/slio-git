# Implementation Plan: IDEA Git Feature Parity

**Branch**: `011-idea-git-parity` | **Date**: 2026-04-04 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/011-idea-git-parity/spec.md`

## Summary

Achieve 1:1 feature parity with IntelliJ IDEA's Git tool window by extending both the git-core library (adding blame, GPG verification, worktree management, submodule detection, graph computation) and the Iced UI layer (refactoring Changes tab to IDEA's stage panel layout, adding multi-tab Log with commit graph, branches dashboard, enhanced branch popup, and comprehensive context menus). The existing MotionSites dark theme is preserved throughout.

## Technical Context

**Language/Version**: Rust (edition 2021+)
**Primary Dependencies**: Iced 0.14 (UI), git2 0.19 (libgit2 bindings), notify 8 (file watching), syntect (syntax highlighting)
**Storage**: File-based (git repositories); commit message history stored in `~/.config/slio-git/`
**Testing**: cargo test (unit), integration tests with real git repository fixtures
**Target Platform**: macOS (primary), Linux, Windows
**Project Type**: Desktop application (native GUI)
**Performance Goals**: 60fps UI, <100ms perceived latency for common git ops, <2s commit graph render for 10k+ commits, <1s branch popup for 500+ branches
**Constraints**: <80MB memory idle, <300ms startup, no WebView/Electron dependencies
**Scale/Scope**: Single-repository workflows, repositories up to 100k commits and 1000+ branches

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
| --------- | ------ | ----- |
| I. IntelliJ Compatibility (NON-NEGOTIABLE) | PASS | Spec explicitly targets 1:1 IDEA parity with 25 FRs mapped to IDEA features |
| II. Rust + Iced Stack | PASS | All implementation in Rust + Iced 0.14, no WebView/Electron |
| III. Library-First Architecture | PASS | New git operations (blame, GPG, worktree, submodule, graph) added to git-core library; UI consumes via direct import |
| IV. Integration Testing for Git Parity | PASS | Plan includes integration tests for all new git-core functions using real repository fixtures |
| V. Observability | PASS | Structured logging for all new git operations with context (repo path, operation, timing) |
| VI. 中文本地化支持 (NON-NEGOTIABLE) | PASS | All new UI labels in Chinese matching IDEA's Chinese localization; PingFang SC on macOS |

## Project Structure

### Documentation (this feature)

```text
specs/011-idea-git-parity/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
│   └── ui-contracts.md  # UI component contracts
└── tasks.md             # Phase 2 output (created by /speckit.tasks)
```

### Source Code (repository root)

```text
src/
└── git-core/
    └── src/
        ├── blame.rs          # NEW: Git blame/annotate operations
        ├── signature.rs      # NEW: GPG/SSH signature verification
        ├── worktree.rs       # NEW: Working tree management
        ├── submodule.rs      # NEW: Submodule detection
        ├── graph.rs          # NEW: Commit graph computation for visualization
        ├── branch.rs         # EXTEND: Branch comparison operations
        ├── history.rs        # EXTEND: History for file path, graph-aware traversal
        ├── remote.rs         # EXTEND: Progress callbacks, force-push, per-remote fetch
        ├── stash.rs          # EXTEND: Include untracked option, stash diff preview
        ├── commit.rs         # EXTEND: Message history persistence
        └── lib.rs            # UPDATE: Export new modules

src-ui/
└── src/
    ├── main.rs              # UPDATE: New message variants, route new views
    ├── state.rs             # UPDATE: New state fields for log tabs, blame, worktrees
    ├── theme.rs             # PRESERVE: No changes (MotionSites theme)
    ├── keyboard.rs          # UPDATE: IDEA-compatible keyboard shortcuts
    ├── i18n.rs              # UPDATE: New Chinese labels for all added features
    ├── views/
    │   ├── main_window.rs   # UPDATE: Changes tab layout to IDEA stage panel
    │   ├── history_view.rs  # REWRITE: Multi-tab log with commit graph, branches dashboard
    │   ├── branch_popup.rs  # REFACTOR: Tree navigation with search, IDEA action submenus
    │   ├── commit_dialog.rs # REMOVE: Replace with embedded commit panel
    │   ├── stash_panel.rs   # EXTEND: Stash content preview, include untracked toggle
    │   ├── tag_dialog.rs    # EXTEND: Push/delete remote tags
    │   ├── rebase_editor.rs # REFINE: Drag-and-drop reorder polish
    │   ├── remote_dialog.rs # EXTEND: Per-remote fetch, progress UI
    │   └── worktree_view.rs # NEW: Working tree management panel
    └── widgets/
        ├── changelist.rs    # REWRITE: Staged/unstaged tree groups, flat/tree toggle, drag-drop
        ├── commit_panel.rs  # EXTEND: Message history dropdown, amend toggle
        ├── diff_viewer.rs   # EXTEND: Stage/unstage hunk buttons, blame gutter
        ├── split_diff_viewer.rs # EXTEND: Stage hunk buttons
        ├── menu.rs          # EXTEND: File context menus, log commit context menus
        ├── commit_graph.rs  # NEW: Visual commit graph renderer (branch lines, merge points)
        ├── tree_widget.rs   # NEW: Generic collapsible tree for branch/file display
        ├── log_tabs.rs      # NEW: Multi-tab bar for log view
        └── progress_bar.rs  # NEW: Network operation progress with cancel
```

**Structure Decision**: Extends the existing two-crate workspace (git-core + src-ui). No new crates needed. 5 new modules added to git-core, 1 new view and 4 new widgets added to src-ui, with significant refactoring of existing views/widgets. Blame/annotate is implemented as a gutter column within diff_viewer.rs rather than a separate view.

## Complexity Tracking

No constitution violations requiring justification. All work fits within the existing two-crate architecture.
