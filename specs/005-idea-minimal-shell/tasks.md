# Tasks: IDEA 风格的极简 Git 工作台

**Input**: Design documents from `/specs/005-idea-minimal-shell/`  
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: 本 feature 未要求严格 TDD；不单独生成测试先行任务，但必须包含基于 constitution 的回归验证、`cargo test` / `cargo clippy` 质量门禁与人工 walkthrough 任务。  

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g. `US1`, `US2`)
- Include exact file paths in descriptions

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: 固化 IDEA 参考、最小化目标和回归检查点，避免实现过程中“越改越回潮”

- [ ] T001 Finalize the chrome-reduction audit and IDEA reference targets in `specs/005-idea-minimal-shell/research.md`
- [ ] T002 [P] Finalize the minimal-shell manual walkthrough and regression matrix in `specs/005-idea-minimal-shell/quickstart.md`
- [ ] T003 [P] Lock the context-switcher, feedback, and reachability rules in `specs/005-idea-minimal-shell/contracts/minimal-shell-contracts.md`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: 建立极简壳层所需的状态、路由、主题、文案和日志基础；完成前不得进入任何用户故事实现

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [ ] T004 Refactor shell state models for `WorkspaceContextSwitcher`, `PrimaryWorkspaceChrome`, and `MinimalFeedbackState` in `src-ui/src/state.rs`
- [ ] T005 [P] Rework shell-level message routing for compact chrome, context switcher lifecycle, and secondary action dispatch in `src-ui/src/main.rs`
- [ ] T006 [P] Rebalance Darcula spacing, toolbar density, and compact surface tokens in `src-ui/src/theme.rs`
- [ ] T007 [P] Normalize concise Chinese labels and action wording for the reduced shell in `src-ui/src/i18n.rs`
- [ ] T008 [P] Add compact feedback logging and context-switcher observability hooks in `src-ui/src/logging.rs`
- [ ] T009 Define shared minimal-shell rendering primitives in `src-ui/src/widgets/button.rs`, `src-ui/src/widgets/scrollable.rs`, and `src-ui/src/widgets/statusbar.rs`

**Checkpoint**: Foundation ready - shell state, compact routing, theme density, and observability are aligned for all user stories

---

## Phase 3: User Story 1 - 更快进入改动处理主线 (Priority: P1) 🎯 MVP

**Goal**: 让仓库工作区第一屏只保留必要上下文，并把视觉重心还给改动列表和差异预览

**Independent Test**: 打开任意仓库后，用户无需阅读多余说明即可识别当前仓库、当前分支、改动列表和差异区，并直接开始处理改动

### Implementation for User Story 1

- [ ] T010 [US1] Remove persistent product branding, tagline, and duplicate chip rows from the repository workspace in `src-ui/src/views/main_window.rs`
- [ ] T011 [P] [US1] Recompose the top workspace chrome around one repository/branch context entry in `src-ui/src/views/main_window.rs` and `src-ui/src/widgets/statusbar.rs`
- [ ] T012 [P] [US1] Rebalance the changes-vs-diff layout so working content dominates the shell in `src-ui/src/views/main_window.rs`, `src-ui/src/widgets/changelist.rs`, and `src-ui/src/widgets/diff_viewer.rs`
- [ ] T013 [US1] Wire repository-open and workspace landing defaults to the minimal shell in `src-ui/src/main.rs` and `src-ui/src/state.rs`

**Checkpoint**: User Story 1 yields a usable minimal workspace MVP with reduced chrome and clear working focus

---

## Phase 4: User Story 2 - 像 IDEA 一样通过上下文切换器完成次要操作 (Priority: P1)

**Goal**: 用一个更克制的上下文切换器承载分支切换和高频 Git 动作，替代常驻按钮堆叠

**Independent Test**: 在打开仓库后，用户可以通过单一上下文入口打开分支与动作面板，并完成刷新、提交、推送、拉取和创建分支

### Implementation for User Story 2

- [ ] T014 [US2] Redesign `branch_popup` into an IDEA-style context switcher with current-branch summary and search in `src-ui/src/views/branch_popup.rs`
- [ ] T015 [P] [US2] Add recent/local/remote grouping and compact branch selection metadata in `src-ui/src/state.rs` and `src-ui/src/views/branch_popup.rs`
- [ ] T016 [P] [US2] Route refresh, commit, pull, push, and new-branch actions through the context switcher in `src-ui/src/main.rs` and `src-ui/src/views/main_window.rs`
- [ ] T017 [P] [US2] Expose compact branch tracking and incoming/outgoing hints for the context switcher in `src/git-core/src/repository.rs` and `src-ui/src/views/branch_popup.rs`
- [ ] T018 [US2] Align context-switcher trigger labels and open/close return behavior in `src-ui/src/main.rs`, `src-ui/src/state.rs`, and `src-ui/src/views/branch_popup.rs`

**Checkpoint**: User Story 2 makes branch operations and high-frequency actions reachable from one IDEA-like context panel

---

## Phase 5: User Story 3 - 减少重复反馈和视觉噪音 (Priority: P2)

**Goal**: 去掉重复 section/next-step/context 表达，把反馈收敛成更短、更克制的状态层

**Independent Test**: 对打开仓库、刷新、暂存、取消暂存、切换分支等高频操作回归时，界面只在需要时显示短时反馈，不再多处重复解释当前状态

### Implementation for User Story 3

- [ ] T019 [US3] Replace persistent section and next-step summaries with compact feedback states in `src-ui/src/state.rs` and `src-ui/src/views/main_window.rs`
- [ ] T020 [P] [US3] Simplify empty and selection messaging in `src-ui/src/widgets/changelist.rs`, `src-ui/src/widgets/diff_viewer.rs`, and `src-ui/src/widgets/split_diff_viewer.rs`
- [ ] T021 [P] [US3] Shorten success and failure presentation across commit, branch, remote, tag, and stash flows in `src-ui/src/views/commit_dialog.rs`, `src-ui/src/views/branch_popup.rs`, `src-ui/src/views/remote_dialog.rs`, `src-ui/src/views/tag_dialog.rs`, and `src-ui/src/views/stash_panel.rs`
- [ ] T022 [US3] Keep only action-required warnings sticky while preserving structured logs in `src-ui/src/main.rs` and `src-ui/src/logging.rs`

**Checkpoint**: User Story 3 removes redundant persistent explanations without losing actionability or observability

---

## Phase 6: User Story 4 - 在精简后仍保留全部核心 Git 可达性 (Priority: P2)

**Goal**: 在最小化主界面的同时，保留历史、标签、储藏、远程、冲突和 rebase 等能力的清晰可达性

**Independent Test**: 从精简后的主工作区出发，用户仍可在一到两步内到达所有现有核心 Git 能力，并完成关键流程

### Implementation for User Story 4

- [ ] T023 [US4] Move secondary Git capabilities behind clear secondary entry points in `src-ui/src/views/main_window.rs` and `src-ui/src/main.rs`
- [ ] T024 [P] [US4] Reorganize history, remote, tag, stash, and rebase entry affordances to match the new hierarchy in `src-ui/src/views/history_view.rs`, `src-ui/src/views/remote_dialog.rs`, `src-ui/src/views/tag_dialog.rs`, `src-ui/src/views/stash_panel.rs`, and `src-ui/src/views/rebase_editor.rs`
- [ ] T025 [P] [US4] Preserve conflict workflow reachability and return-to-workspace behavior in `src-ui/src/widgets/conflict_resolver.rs`, `src-ui/src/main.rs`, and `src-ui/src/state.rs`
- [ ] T026 [US4] Validate one-to-two-step reachability for retained Git capabilities in `specs/005-idea-minimal-shell/quickstart.md` and `src-ui/src/views/main_window.rs`

**Checkpoint**: User Story 4 proves simplification did not remove or bury existing Git capability

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: 收尾回归、补充验证并确认极简目标真正兑现

- [ ] T027 [P] Refresh the IDEA comparison walkthrough and minimal-shell regression matrix in `specs/005-idea-minimal-shell/quickstart.md`
- [ ] T028 [P] Add compact branch-context regression coverage in `src/git-core/tests/workflow_regressions.rs` and `src/git-core/tests/test_helpers.rs`
- [ ] T029 Run workspace quality gates for the minimal shell from `Cargo.toml`, `src-ui/Cargo.toml`, `src/git-core/Cargo.toml`, and `specs/005-idea-minimal-shell/quickstart.md`
- [ ] T030 Verify success criteria and capture final simplification outcomes in `specs/005-idea-minimal-shell/spec.md` and `specs/005-idea-minimal-shell/quickstart.md`

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3+)**: All depend on Foundational phase completion
- **Polish (Phase 7)**: Depends on all desired user stories being complete

### User Story Dependencies

- **US1 (P1)**: Starts immediately after Foundational - defines the minimal workspace MVP
- **US2 (P1)**: Depends on US1 shell simplification and Foundational routing/state work
- **US3 (P2)**: Depends on US1 shell structure; benefits from US2 context-switcher hierarchy but remains independently testable
- **US4 (P2)**: Depends on US1 minimal shell and US2 action hierarchy so retained capabilities can be re-homed cleanly

### Within Each User Story

- Shell-level layout and state changes before view-specific polish
- Popup/panel grouping before secondary reachability adjustments
- Compact feedback rules before per-screen message shortening
- Story checkpoint validation must complete before moving to final polish

### Parallel Opportunities

- **Setup**: T002 and T003 can run in parallel after T001 sets the reference baseline
- **Foundational**: T005, T006, T007, and T008 can run in parallel after T004 defines the minimal-shell state model
- **US1**: T011 and T012 can run in parallel after T010 removes the biggest chrome layers
- **US2**: T015, T016, and T017 can run in parallel after T014 establishes the context-switcher structure
- **US3**: T020 and T021 can run in parallel after T019 defines compact feedback behavior
- **US4**: T024 and T025 can run in parallel after T023 sets the secondary entry hierarchy
- **Polish**: T027 and T028 can run in parallel before T029 and T030 close the feature

---

## Parallel Example: User Story 1

```bash
# After the redundant top chrome is removed, these can proceed in parallel:
Task: "Recompose the top workspace chrome around one repository/branch context entry in src-ui/src/views/main_window.rs and src-ui/src/widgets/statusbar.rs"
Task: "Rebalance the changes-vs-diff layout so working content dominates the shell in src-ui/src/views/main_window.rs, src-ui/src/widgets/changelist.rs, and src-ui/src/widgets/diff_viewer.rs"
```

## Parallel Example: User Story 2

```bash
# Once the context-switcher frame exists, enrich it in parallel:
Task: "Add recent/local/remote grouping and compact branch selection metadata in src-ui/src/state.rs and src-ui/src/views/branch_popup.rs"
Task: "Route refresh, commit, pull, push, and new-branch actions through the context switcher in src-ui/src/main.rs and src-ui/src/views/main_window.rs"
Task: "Expose compact branch tracking and incoming/outgoing hints for the context switcher in src/git-core/src/repository.rs and src-ui/src/views/branch_popup.rs"
```

## Parallel Example: User Story 3

```bash
# After compact feedback rules are established, trim noisy messaging in parallel:
Task: "Simplify empty and selection messaging in src-ui/src/widgets/changelist.rs, src-ui/src/widgets/diff_viewer.rs, and src-ui/src/widgets/split_diff_viewer.rs"
Task: "Shorten success and failure presentation across commit, branch, remote, tag, and stash flows in src-ui/src/views/commit_dialog.rs, src-ui/src/views/branch_popup.rs, src-ui/src/views/remote_dialog.rs, src-ui/src/views/tag_dialog.rs, and src-ui/src/views/stash_panel.rs"
```

## Parallel Example: User Story 4

```bash
# After the new secondary hierarchy is chosen, wire retained capabilities in parallel:
Task: "Reorganize history, remote, tag, stash, and rebase entry affordances to match the new hierarchy in src-ui/src/views/history_view.rs, src-ui/src/views/remote_dialog.rs, src-ui/src/views/tag_dialog.rs, src-ui/src/views/stash_panel.rs, and src-ui/src/views/rebase_editor.rs"
Task: "Preserve conflict workflow reachability and return-to-workspace behavior in src-ui/src/widgets/conflict_resolver.rs, src-ui/src/main.rs, and src-ui/src/state.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational
3. Complete Phase 3: User Story 1
4. **STOP and VALIDATE**: confirm the repository workspace no longer shows redundant branding/chrome and that changes + diff dominate the first screen
5. Demo the minimal shell before introducing the context switcher

### Incremental Delivery

1. Setup + Foundational define the minimal-shell rules and shared state
2. Add US1 to create the stripped-down repository workspace MVP
3. Add US2 to introduce the IDEA-style context switcher for branches and high-frequency actions
4. Add US3 to shrink repetitive feedback and explanation surfaces
5. Add US4 to preserve reachability for all retained Git capabilities
6. Finish with regression walkthrough, parity checks, and quality gates

### Parallel Team Strategy

With multiple developers:

1. One stream owns shell/state/theme foundation (`state.rs`, `main.rs`, `theme.rs`, `i18n.rs`)
2. One stream owns workspace chrome and content focus (`main_window.rs`, `changelist.rs`, `diff_viewer.rs`, `statusbar.rs`)
3. One stream owns context switcher and branch affordances (`branch_popup.rs`, `repository.rs`)
4. One stream owns retained secondary capability surfaces (`history_view.rs`, `remote_dialog.rs`, `tag_dialog.rs`, `stash_panel.rs`, `rebase_editor.rs`, `conflict_resolver.rs`)

---

## Notes

- `[P]` tasks must touch disjoint files or converge only after a coordinating task
- The suggested MVP scope is **User Story 1 only**, with **User Story 2** as the next highest-value increment
- This feature simplifies UI chrome and interaction layering; it must not change `git-core` ownership boundaries or Git operation semantics
- Do not close the feature until `quickstart.md` confirms both “去冗余” and “能力仍可达” goals
