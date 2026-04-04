# Tasks: 主界面可用性与视觉改造

**Input**: Design documents from `/specs/004-ui-usability-refresh/`  
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: 本 feature 未要求严格 TDD，但依据 constitution 与计划要求，必须包含回归验证、`cargo test` / `cargo clippy` 和 defect sweep 验证任务。  

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g. `US1`, `US2`)
- Include exact file paths in descriptions

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: 建立全应用改造与 defect sweep 的执行基线

- [X] T001 Create screen-by-screen redesign tracker in `specs/004-ui-usability-refresh/ui-surface-inventory.md`
- [X] T002 [P] Create repository-wide defect sweep ledger in `specs/004-ui-usability-refresh/defect-matrix.md`
- [X] T003 [P] Extend validation checkpoints and screen walkthrough notes in `specs/004-ui-usability-refresh/quickstart.md`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: 壳层、状态、主题和共享反馈基础设施；完成前不得进入任何用户故事实现

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [X] T004 Refactor shell state models in `src-ui/src/state.rs` to add `AppShellState`, navigation metadata, unified feedback state, and defect-tracking hooks
- [X] T005 [P] Centralize shell routing and primary-action message flow in `src-ui/src/main.rs`
- [X] T006 [P] Expand shared Darcula design tokens for shell, dialogs, panels, and feedback states in `src-ui/src/theme.rs`
- [X] T007 [P] Refactor shared primitive styling in `src-ui/src/widgets/button.rs`, `src-ui/src/widgets/text_input.rs`, `src-ui/src/widgets/scrollable.rs`, and `src-ui/src/components/status_icons.rs`
- [X] T008 [P] Normalize cross-screen Chinese copy, navigation labels, and action wording in `src-ui/src/i18n.rs`
- [X] T009 Create reusable loading/error/success logging and rendering hooks in `src-ui/src/logging.rs`, `src-ui/src/views/mod.rs`, and `src-ui/src/widgets/mod.rs`
- [X] T010 Define defect severity, ownership, and verification fields in `specs/004-ui-usability-refresh/defect-matrix.md`

**Checkpoint**: Foundation ready - shell state, theme tokens, feedback model, and defect ledger are in place for all user stories

---

## Phase 3: User Story 1 - 快速进入核心工作流 (Priority: P1) 🎯 MVP

**Goal**: 重做欢迎态、应用壳层和仓库主工作区，让用户不依赖旧版导航也能迅速进入核心流程

**Independent Test**: 启动应用后，不阅读额外说明，用户可以在 30 秒内找到主入口、打开仓库并进入可操作的仓库工作区

### Implementation for User Story 1

- [X] T011 [US1] Rebuild the global shell layout and primary entry structure in `src-ui/src/views/main_window.rs`
- [X] T012 [P] [US1] Implement redesigned welcome state and onboarding actions in `src-ui/src/views/main_window.rs` and `src-ui/src/widgets/file_picker.rs`
- [X] T013 [P] [US1] Recompose repository workspace landing sections and contextual summary in `src-ui/src/views/main_window.rs` and `src-ui/src/widgets/statusbar.rs`
- [X] T014 [P] [US1] Refactor changed-file and diff entry flow for the new shell in `src-ui/src/widgets/changelist.rs`, `src-ui/src/widgets/diff_viewer.rs`, and `src-ui/src/widgets/split_diff_viewer.rs`
- [X] T015 [US1] Wire repository open/init/workspace transitions into the new shell flow in `src-ui/src/main.rs` and `src-ui/src/state.rs`

**Checkpoint**: User Story 1 is independently usable as the MVP shell and workspace entry flow

---

## Phase 4: User Story 2 - 更清晰的视觉层级与信息密度 (Priority: P1)

**Goal**: 用统一 Darcula 视觉语言重做全部现有 UI 界面，形成一致的层级、留白和控件密度

**Independent Test**: 在标准桌面窗口尺寸下遍历所有现有界面，标题、主操作、次级操作、边界层次和控件风格保持一致

### Implementation for User Story 2

- [X] T016 [P] [US2] Apply shared Darcula hierarchy and density to workspace surfaces in `src-ui/src/views/main_window.rs`, `src-ui/src/widgets/changelist.rs`, `src-ui/src/widgets/diff_viewer.rs`, `src-ui/src/widgets/split_diff_viewer.rs`, and `src-ui/src/widgets/statusbar.rs`
- [X] T017 [P] [US2] Restyle dialog-oriented screens in `src-ui/src/views/commit_dialog.rs`, `src-ui/src/views/branch_popup.rs`, `src-ui/src/views/remote_dialog.rs`, and `src-ui/src/views/tag_dialog.rs`
- [X] T018 [P] [US2] Restyle panel/editor surfaces in `src-ui/src/views/stash_panel.rs`, `src-ui/src/views/history_view.rs`, `src-ui/src/views/rebase_editor.rs`, `src-ui/src/widgets/conflict_resolver.rs`, and `src-ui/src/widgets/commit_compare.rs`
- [X] T019 [US2] Harmonize iconography, separators, borders, and dense layout rules in `src-ui/src/components/status_icons.rs`, `src-ui/src/widgets/button.rs`, `src-ui/src/widgets/text_input.rs`, and `src-ui/src/widgets/scrollable.rs`

**Checkpoint**: User Story 2 yields a visually coherent Darcula redesign across every current UI surface

---

## Phase 5: User Story 3 - 更明确的交互反馈与状态引导 (Priority: P1)

**Goal**: 为加载、失败、成功、选中、禁用和异步操作建立全应用一致的反馈机制

**Independent Test**: 对主界面和常用操作执行点击、切换、刷新和打开动作时，用户都能看到明确且一致的反馈

### Implementation for User Story 3

- [X] T020 [US3] Add global loading/success/error/empty feedback orchestration in `src-ui/src/state.rs` and `src-ui/src/main.rs`
- [X] T021 [P] [US3] Surface actionable selection, disabled, and operation feedback in `src-ui/src/views/main_window.rs`, `src-ui/src/widgets/changelist.rs`, and `src-ui/src/widgets/statusbar.rs`
- [X] T022 [P] [US3] Surface per-screen loading/error/success states in `src-ui/src/views/commit_dialog.rs`, `src-ui/src/views/branch_popup.rs`, `src-ui/src/views/remote_dialog.rs`, `src-ui/src/views/tag_dialog.rs`, `src-ui/src/views/stash_panel.rs`, and `src-ui/src/views/history_view.rs`
- [X] T023 [P] [US3] Surface conflict/rebase specific feedback and next-step guidance in `src-ui/src/views/rebase_editor.rs` and `src-ui/src/widgets/conflict_resolver.rs`
- [X] T024 [US3] Extend structured navigation and async failure logging in `src-ui/src/logging.rs` and `src-ui/src/main.rs`

**Checkpoint**: User Story 3 makes state changes and failures understandable across the application

---

## Phase 6: User Story 4 - 在常见窗口尺寸下保持可用 (Priority: P2)

**Goal**: 让重构后的壳层、列表、diff、弹窗和面板在最小支持尺寸与默认尺寸下都保持稳定可用

**Independent Test**: 在 `800x600` 和 `1280x800` 窗口尺寸下，用户都能完成打开仓库、浏览内容和执行主操作

### Implementation for User Story 4

- [X] T025 [US4] Implement minimum-size-aware shell layout and panel resizing rules in `src-ui/src/views/main_window.rs` and `src-ui/src/widgets/scrollable.rs`
- [X] T026 [P] [US4] Harden long-path, dense-list, and diff overflow handling in `src-ui/src/widgets/changelist.rs`, `src-ui/src/widgets/diff_viewer.rs`, `src-ui/src/widgets/split_diff_viewer.rs`, and `src-ui/src/widgets/statusbar.rs`
- [X] T027 [P] [US4] Harden dialog and panel sizing/scroll behavior in `src-ui/src/views/commit_dialog.rs`, `src-ui/src/views/branch_popup.rs`, `src-ui/src/views/remote_dialog.rs`, `src-ui/src/views/tag_dialog.rs`, `src-ui/src/views/stash_panel.rs`, `src-ui/src/views/history_view.rs`, and `src-ui/src/views/rebase_editor.rs`
- [X] T028 [US4] Verify and adjust window sizing constraints and shell spacing scales in `src-ui/src/main.rs` and `src-ui/src/theme.rs`

**Checkpoint**: User Story 4 keeps the redesigned UI stable under realistic desktop window sizes and content loads

---

## Phase 7: User Story 5 - 空状态与异常状态也保持专业体验 (Priority: P2)

**Goal**: 为无仓库、无变更、空列表、无选择和失败场景提供清晰、可执行的说明与下一步操作

**Independent Test**: 模拟无仓库、无变更、数据缺失和加载失败场景时，应用不会出现无法解释的大块空白

### Implementation for User Story 5

- [X] T029 [US5] Implement clear no-repository and shell-level empty states in `src-ui/src/views/main_window.rs` and `src-ui/src/i18n.rs`
- [X] T030 [P] [US5] Implement screen-specific empty states in `src-ui/src/views/history_view.rs`, `src-ui/src/views/stash_panel.rs`, `src-ui/src/views/remote_dialog.rs`, `src-ui/src/views/tag_dialog.rs`, and `src-ui/src/views/rebase_editor.rs`
- [X] T031 [P] [US5] Implement no-selection/no-diff/no-changes states in `src-ui/src/widgets/changelist.rs`, `src-ui/src/widgets/diff_viewer.rs`, `src-ui/src/widgets/split_diff_viewer.rs`, and `src-ui/src/widgets/commit_compare.rs`
- [X] T032 [US5] Implement retry and next-step affordances for repository and operation failures in `src-ui/src/main.rs`, `src-ui/src/state.rs`, and `src-ui/src/views/main_window.rs`

**Checkpoint**: User Story 5 removes broken-looking blank states and gives the user a path forward from every empty/error scenario

---

## Phase 8: User Story 6 - 同步修复发现的现有功能问题 (Priority: P2)

**Goal**: 把本次改造过程中在整个仓库内发现的问题统一记录、修复并回归验证，不让“只换皮”成为交付结果

**Independent Test**: 在 defect ledger 中记录的每个问题都有复现路径、修复动作和验证结果，且相关功能路径可重新完成

### Implementation for User Story 6

- [X] T033 [US6] Audit repository-wide defects and capture reproduction paths in `specs/004-ui-usability-refresh/defect-matrix.md`
- [X] T034 [P] [US6] Fix repository open/refresh/selection regressions in `src-ui/src/main.rs`, `src-ui/src/state.rs`, `src-ui/src/views/main_window.rs`, and `src-ui/src/widgets/changelist.rs`
- [X] T035 [P] [US6] Fix toolbar and dialog action gaps in `src-ui/src/main.rs`, `src-ui/src/views/commit_dialog.rs`, `src-ui/src/views/branch_popup.rs`, `src-ui/src/views/remote_dialog.rs`, `src-ui/src/views/tag_dialog.rs`, and `src-ui/src/views/stash_panel.rs`
- [X] T036 [P] [US6] Fix diff/conflict/rebase behavior gaps in `src-ui/src/widgets/conflict_resolver.rs`, `src-ui/src/widgets/diff_viewer.rs`, `src-ui/src/widgets/split_diff_viewer.rs`, `src-ui/src/views/rebase_editor.rs`, and `src/git-core/src/diff.rs`
- [X] T037 [P] [US6] Fix repository-wide git-core defects discovered during the sweep in `src/git-core/src/repository.rs`, `src/git-core/src/index.rs`, `src/git-core/src/commit.rs`, `src/git-core/src/remote.rs`, `src/git-core/src/branch.rs`, `src/git-core/src/stash.rs`, and `src/git-core/src/history.rs`
- [X] T038 [US6] Re-verify every recorded defect and update resolution status in `specs/004-ui-usability-refresh/defect-matrix.md`

**Checkpoint**: User Story 6 closes the defect sweep with verified fixes rather than an open-ended bug list

---

## Phase 9: Polish & Cross-Cutting Concerns

**Purpose**: 回归验证、质量门禁和跨故事收尾

- [X] T039 [P] Add git-core regression coverage for discovered workflow fixes in `src/git-core/tests/workflow_regressions.rs` and `src/git-core/tests/test_helpers.rs`
- [X] T040 [P] Finalize manual regression matrix and screen walkthrough notes in `specs/004-ui-usability-refresh/quickstart.md` and `specs/004-ui-usability-refresh/defect-matrix.md`
- [X] T041 Run workspace quality gates from `Cargo.toml`, `src-ui/Cargo.toml`, and `src/git-core/Cargo.toml`, recording follow-up fixes in `specs/004-ui-usability-refresh/defect-matrix.md`
- [X] T042 Verify all success criteria and close remaining defects in `specs/004-ui-usability-refresh/spec.md` and `specs/004-ui-usability-refresh/defect-matrix.md`

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1: Setup**: No dependencies - can start immediately
- **Phase 2: Foundational**: Depends on Phase 1 completion - BLOCKS all user stories
- **Phase 3-8: User Stories**: Depend on Phase 2 completion
- **Phase 9: Polish**: Depends on all desired user stories being complete

### User Story Dependencies

- **US1 (P1)**: Starts immediately after Foundational - defines the new shell and MVP workflow
- **US2 (P1)**: Starts after Foundational and can run partly in parallel with US1, but final harmonization depends on the new shell surfaces landing
- **US3 (P1)**: Depends on US1 shell flow and Foundational feedback primitives; benefits from US2 tokens but remains independently testable
- **US4 (P2)**: Depends on US1 shell layout and US2 styling baselines
- **US5 (P2)**: Depends on US1 shell structure and US3 feedback primitives
- **US6 (P2)**: Defect discovery can start early after Foundational, but final closure depends on the redesigned surfaces from US1-US5 stabilizing

### Within Each User Story

- Shared state and theme updates before screen-specific wiring
- Screen groups marked `[P]` can run in parallel when they touch disjoint files
- Defect fixes must be recorded before they are marked verified
- Story checkpoint validation must complete before moving to final polish

### Parallel Opportunities

- **Setup**: T002 and T003 can run in parallel after T001 starts the feature tracker
- **Foundational**: T005, T006, T007, and T008 can run in parallel after T004 establishes the shell state model
- **US1**: T012, T013, and T014 can run in parallel after shell direction from T011 is set
- **US2**: T016, T017, and T018 are parallel by screen group; T019 closes the shared primitives
- **US3**: T021, T022, and T023 are parallel by screen group after T020 establishes the feedback model
- **US4**: T026 and T027 can run in parallel after T025 defines shell sizing behavior
- **US5**: T030 and T031 can run in parallel after T029 defines shell-level empty states
- **US6**: T034, T035, T036, and T037 can run in parallel once T033 captures the defect backlog

---

## Parallel Example: User Story 2

```bash
# Restyle independent screen groups in parallel after the Darcula shell baseline is ready:
Task: "Apply shared Darcula hierarchy and density to workspace surfaces in src-ui/src/views/main_window.rs, src-ui/src/widgets/changelist.rs, src-ui/src/widgets/diff_viewer.rs, src-ui/src/widgets/split_diff_viewer.rs, and src-ui/src/widgets/statusbar.rs"
Task: "Restyle dialog-oriented screens in src-ui/src/views/commit_dialog.rs, src-ui/src/views/branch_popup.rs, src-ui/src/views/remote_dialog.rs, and src-ui/src/views/tag_dialog.rs"
Task: "Restyle panel/editor surfaces in src-ui/src/views/stash_panel.rs, src-ui/src/views/history_view.rs, src-ui/src/views/rebase_editor.rs, src-ui/src/widgets/conflict_resolver.rs, and src-ui/src/widgets/commit_compare.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational
3. Complete Phase 3: User Story 1
4. **STOP and VALIDATE**: confirm users can discover the primary entry, open a repository, and land in a readable workspace
5. Demo the new shell before expanding the redesign to all screens

### Incremental Delivery

1. Setup + Foundational establish shell, state, theme, and defect tracking
2. Add US1 to create the new navigable MVP shell
3. Add US2 to spread visual consistency across all screens
4. Add US3 to make states and feedback understandable
5. Add US4 and US5 to harden layout stability and empty/error paths
6. Add US6 to close the defect sweep with verified fixes
7. Finish with regression coverage, quality gates, and success-criteria verification

### Parallel Team Strategy

With multiple developers:

1. One stream owns shell/state/theme foundation (`state.rs`, `main.rs`, `theme.rs`)
2. One stream owns workspace and shared widgets (`main_window.rs`, `changelist.rs`, `diff_viewer.rs`, `statusbar.rs`)
3. One stream owns secondary screens (`commit_dialog.rs`, `branch_popup.rs`, `remote_dialog.rs`, `tag_dialog.rs`, `stash_panel.rs`, `history_view.rs`, `rebase_editor.rs`, `conflict_resolver.rs`)
4. One stream owns defect sweep and `git-core` regressions (`src/git-core/src/*.rs`, `src/git-core/tests/*.rs`, `defect-matrix.md`)

---

## Notes

- `[P]` tasks must touch disjoint files or only converge after a coordinating task
- `US6` is intentionally broad because the spec explicitly includes repository-wide defect fixes; the defect matrix is the mechanism that keeps it executable
- Do not mark the feature done until `defect-matrix.md` shows every discovered issue as verified or explicitly blocked with rationale
- Keep Git behavior inside `git-core`; UI refactors may change flow and naming, but not the underlying Git parity requirements
