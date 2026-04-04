# Tasks: IDEA 式 Git 工作台主线

**Input**: Design documents from `/specs/008-idea-lite-git/`  
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: 本 feature 不要求严格 TDD，但由于 Constitution 要求 IntelliJ parity 与集成回归，任务中包含针对 `src/git-core/tests/workflow_regressions.rs`、`src-ui/src/state.rs` 和 `quickstart.md` 的覆盖扩展与最终质量门禁。  

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g. `US1`, `US2`)
- Include exact file paths in descriptions

## Phase 1: Setup (Shared Baseline)

**Purpose**: 固化 Git-first 工作台的边界、基线和验收路径，避免实现过程中重新退回“功能分散的小工具集合”

- [ ] T001 Freeze the Git-first workspace slices, MVP/V1/V2 framing, and success-criteria checkpoints in `specs/008-idea-lite-git/plan.md`
- [ ] T002 [P] Lock primary-vs-auxiliary workspace decisions and anti-goals in `specs/008-idea-lite-git/research.md`
- [ ] T003 [P] Finalize the workspace walkthrough and UI contracts in `specs/008-idea-lite-git/quickstart.md` and `specs/008-idea-lite-git/contracts/idea-lite-workspace-contracts.md`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: 建立所有用户故事共用的工作台状态、路由、文案、主题与回归基础；完成前不得进入任何用户故事实现

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [ ] T004 Refactor shared workspace-shell, retire standalone repository overview-home routing, and define primary section / auxiliary-peek state in `src-ui/src/state.rs`
- [ ] T005 [P] Wire primary workspace routing, auxiliary peek lifecycle, and return-to-workspace flow in `src-ui/src/main.rs`
- [ ] T006 [P] Normalize Git-first Chinese copy and feedback vocabulary in `src-ui/src/i18n.rs` and `src-ui/src/views/mod.rs`
- [ ] T007 [P] Rebalance shared workspace chrome, context strip, and status-surface primitives in `src-ui/src/theme.rs`, `src-ui/src/widgets/statusbar.rs`, and `src-ui/src/components/rail_icons.rs`
- [ ] T008 [P] Preserve structured logs and observable feedback for workspace, action, and risk flows in `src-ui/src/logging.rs` and `src-ui/src/state.rs`
- [ ] T009 Strengthen shared regression scaffolding for workspace-state transitions and IntelliJ-parity walkthroughs in `src-ui/src/state.rs`, `src/git-core/tests/workflow_regressions.rs`, and `specs/008-idea-lite-git/quickstart.md`

**Checkpoint**: Foundation ready - the app has one coherent Git workspace model, shared chrome primitives, localized copy, and baseline regression coverage

---

## Phase 3: User Story 1 - 打开后立刻进入 Git 工作台 (Priority: P1) 🎯 MVP

**Goal**: 打开仓库后第一眼就是 Git 工作台，而不是分散页面或管理型首页

**Independent Test**: 打开一个带改动的仓库后，用户可在单一工作台中快速识别仓库、分支、变更列表、差异预览和下一步动作

### Implementation for User Story 1

- [ ] T010 [US1] Rebuild the main repository surface around one Git workspace shell in `src-ui/src/views/main_window.rs`
- [ ] T011 [P] [US1] Rework change grouping, selection, and focus rhythm in `src-ui/src/widgets/changelist.rs`
- [ ] T012 [P] [US1] Keep unified/split diff viewers centered on current-file preview and readable empty or binary states in `src-ui/src/widgets/diff_viewer.rs`, `src-ui/src/widgets/split_diff_viewer.rs`, and `src-ui/src/widgets/syntax_highlighting.rs`
- [ ] T013 [US1] Bind repository context, branch context, and sync hints into the primary workspace strip in `src-ui/src/state.rs` and `src-ui/src/main.rs`
- [ ] T014 [US1] Handle no-repository, no-changes, untracked, and conflicted first-screen entry states in `src-ui/src/views/main_window.rs` and `src-ui/src/state.rs`
- [ ] T015 [US1] Add workspace-entry and file-selection state coverage in `src-ui/src/state.rs`
- [ ] T016 [US1] Add diff-viewing parity regression coverage in `src/git-core/tests/workflow_regressions.rs` and `specs/008-idea-lite-git/quickstart.md`

**Checkpoint**: User Story 1 delivers a recognizably IDEA-like Git workspace home screen that is independently usable as the MVP

---

## Phase 4: User Story 2 - 在一个地方完成日常 Git 操作闭环 (Priority: P1)

**Goal**: 在主工作台里完成“审阅改动 → 暂存/取消暂存 → 提交 → 同步 → 切分支”的高频 Git 闭环

**Independent Test**: 用户可以在应用内完成一次常见的小到中等变更集提交并同步远端，不需要回终端

### Implementation for User Story 2

- [ ] T017 [US2] Route stage, unstage, discard, and commit availability from the current review context in `src-ui/src/main.rs`, `src-ui/src/state.rs`, and `src/git-core/src/index.rs`
- [ ] T018 [P] [US2] Rework the commit review flow and action feedback loop in `src-ui/src/views/commit_dialog.rs` and `src-ui/src/views/mod.rs`
- [ ] T019 [P] [US2] Anchor fetch, pull, and push UI flows to the current branch context in `src-ui/src/views/remote_dialog.rs` and `src-ui/src/main.rs`
- [ ] T020 [P] [US2] Tighten branch search, switch, and create workflow in `src-ui/src/views/branch_popup.rs` and `src/git-core/src/branch.rs`
- [ ] T021 [US2] Expose current-branch sync, upstream, and remote-action helpers in `src/git-core/src/repository.rs` and `src/git-core/src/remote.rs`
- [ ] T022 [US2] Extend mainline action regression coverage for stage, commit, sync, and branch workflows in `src/git-core/tests/workflow_regressions.rs`

**Checkpoint**: User Story 2 makes the workspace capable of finishing the default daily Git loop without leaving the app

---

## Phase 5: User Story 3 - 像 IDE 一样连续切换项目和恢复上下文 (Priority: P2)

**Goal**: 提供 IDE 式工作连续性，让最近项目、恢复上下文和快速切换成为稳定体验

**Independent Test**: 打开多个仓库后重启应用，用户可恢复上次仓库并快速切换到其他最近项目继续工作

### Implementation for User Story 3

- [ ] T023 [US3] Expand persisted project memory and workspace-restoration semantics in `src-ui/src/state.rs`
- [ ] T024 [P] [US3] Rework recent-project rail presentation and quick-switch actions in `src-ui/src/views/main_window.rs` and `src-ui/src/components/rail_icons.rs`
- [ ] T025 [P] [US3] Keep file watching and auto-refresh coherent across repository switches in `src-ui/src/file_watcher.rs` and `src-ui/src/main.rs`
- [ ] T026 [US3] Handle invalid paths, moved repositories, and empty-restore fallbacks in `src-ui/src/state.rs` and `src-ui/src/views/main_window.rs`
- [ ] T027 [US3] Add restore-and-switch coverage for repeated-work sessions in `src-ui/src/state.rs`

**Checkpoint**: User Story 3 gives the app IDE-style continuity instead of one-off repository sessions

---

## Phase 6: User Story 4 - 遇到风险状态时也能留在同一工作台处理 (Priority: P2)

**Goal**: 冲突、认证失败、同步异常和进行中的 Git 流程都能在应用内被解释并继续处理

**Independent Test**: 准备一个存在冲突或远端失败的仓库，用户可在应用内看懂当前状态、进入处理流程并确认结果

### Implementation for User Story 4

- [ ] T028 [US4] Promote conflict, merge, rebase, and remote-failure states into workspace-level risk signals in `src-ui/src/state.rs` and `src-ui/src/views/main_window.rs`
- [ ] T029 [P] [US4] Rebuild conflict continuation between the conflict list and resolver workbench in `src-ui/src/main.rs`, `src-ui/src/widgets/conflict_resolver.rs`, and `src/git-core/src/diff.rs`
- [ ] T030 [P] [US4] Normalize actionable Chinese failure copy and next-step hints in `src-ui/src/views/remote_dialog.rs`, `src-ui/src/views/rebase_editor.rs`, and `src-ui/src/views/mod.rs`
- [ ] T031 [P] [US4] Detect detached HEAD, blocked Git flows, and continuation signals from core state in `src/git-core/src/repository.rs`, `src/git-core/src/rebase.rs`, and `src/git-core/src/remote.rs`
- [ ] T032 [US4] Extend risk-state regression coverage for conflicts, rejected remotes, detached HEAD, and in-progress flows in `src/git-core/tests/workflow_regressions.rs` and `src-ui/src/state.rs`

**Checkpoint**: User Story 4 turns dead-end Git failures into understandable continuation paths inside the workspace

---

## Phase 7: User Story 5 - 需要回看历史时也不必离开应用 (Priority: P3)

**Goal**: 历史、标签、储藏和相关上下文以“快速 peek”方式服务主工作流，而不是把用户拉离当前仓库节奏

**Independent Test**: 用户可在应用内快速查看近期提交、标签或储藏，再无缝返回当前 Git 主工作台继续操作

### Implementation for User Story 5

- [ ] T033 [US5] Recast history as a quick context-peek surface with a clear return-to-workspace path in `src-ui/src/views/history_view.rs` and `src-ui/src/main.rs`
- [ ] T034 [P] [US5] Align tag and stash surfaces with the peek-and-return model in `src-ui/src/views/tag_dialog.rs` and `src-ui/src/views/stash_panel.rs`
- [ ] T035 [P] [US5] Keep remote and branch helper surfaces from displacing the main review surface in `src-ui/src/views/remote_dialog.rs`, `src-ui/src/views/branch_popup.rs`, and `src-ui/src/views/main_window.rs`
- [ ] T036 [US5] Preserve selection, scroll, and return targets across auxiliary peeks in `src-ui/src/state.rs` and `src-ui/src/views/main_window.rs`
- [ ] T037 [US5] Expose recent-history, tag, and stash metadata needed for quick judgment in `src/git-core/src/history.rs`, `src/git-core/src/tag.rs`, and `src/git-core/src/stash.rs`
- [ ] T038 [US5] Add quick-peek walkthrough coverage and supporting regressions in `specs/008-idea-lite-git/quickstart.md`, `src/git-core/tests/workflow_regressions.rs`, and `src-ui/src/state.rs`
- [ ] T039 [US5] Add stash-management parity regression coverage for list/apply/pop/drop paths in `src/git-core/tests/workflow_regressions.rs`, `src-ui/src/views/stash_panel.rs`, and `specs/008-idea-lite-git/quickstart.md`

**Checkpoint**: User Story 5 adds supporting context without breaking the Git-first workspace focus

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: 对齐最终体验、收敛文案与图标、跑完整质量门禁并回填验收证据

- [ ] T040 [P] Audit Git-first Chinese copy, icon emphasis, and stable-state feedback across `src-ui/src/views/main_window.rs`, `src-ui/src/views/commit_dialog.rs`, `src-ui/src/views/history_view.rs`, `src-ui/src/views/remote_dialog.rs`, `src-ui/src/views/tag_dialog.rs`, `src-ui/src/views/stash_panel.rs`, and `src-ui/src/views/rebase_editor.rs`
- [ ] T041 [P] Capture final acceptance evidence for SC-001 through SC-009 in `specs/008-idea-lite-git/spec.md` and `specs/008-idea-lite-git/quickstart.md`
- [ ] T042 Run workspace quality gates and the full Git-first walkthrough from `Cargo.toml`, `src-ui/Cargo.toml`, and `specs/008-idea-lite-git/quickstart.md`
- [ ] T043 Validate startup and common Git-operation latency targets in `specs/008-idea-lite-git/quickstart.md`, `src-ui/src/logging.rs`, and `src-ui/src/main.rs`

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3+)**: All depend on Foundational phase completion
- **Polish (Phase 8)**: Depends on all desired user stories being complete

### User Story Dependencies

- **US1 (P1)**: Starts immediately after Foundational - defines the MVP Git workspace shell
- **US2 (P1)**: Starts after Foundational and integrates most naturally on top of the US1 workspace shell, but remains independently testable as a high-frequency action loop
- **US3 (P2)**: Starts after Foundational - extends continuity without depending on US2
- **US4 (P2)**: Starts after Foundational, but benefits from US1/US2 shell and action surfaces to render blocked-state continuation clearly
- **US5 (P3)**: Starts after Foundational and uses the US1 workspace shell as the return target for peek surfaces while carrying stash-management parity coverage

### Within Each User Story

- Shared state/routing updates before the view layers that render them
- Main workspace rendering before story-specific edge cases and walkthrough coverage
- Core action-path wiring before feedback polish and regression expansion
- Story checkpoint validation should complete before moving to final polish

### Parallel Opportunities

- **Setup**: T002 and T003 can run in parallel after T001 confirms the final scope framing
- **Foundational**: T005, T006, T007, and T008 can run in parallel after T004 establishes the shared workspace-state model
- **US1**: T011 and T012 can run in parallel while T010/T013 reshape the main workspace shell
- **US2**: T018, T019, and T020 can run in parallel after T017 anchors actions to the current review context
- **US3**: T024 and T025 can run in parallel after T023 defines the persisted continuity model
- **US4**: T029, T030, and T031 can run in parallel after T028 promotes blocked states into workspace-level signals
- **US5**: T034 and T035 can run in parallel after T033 establishes the peek-and-return pattern
- **Polish**: T040 and T041 can run in parallel before T042 and T043 close the feature

---

## Parallel Example: User Story 1

```bash
# After the workspace shell contract is wired, these can proceed in parallel:
Task: "Rework change grouping, selection, and focus rhythm in src-ui/src/widgets/changelist.rs"
Task: "Keep unified/split diff viewers centered on current-file preview and readable empty or binary states in src-ui/src/widgets/diff_viewer.rs, src-ui/src/widgets/split_diff_viewer.rs, and src-ui/src/widgets/syntax_highlighting.rs"
```

## Parallel Example: User Story 2

```bash
# Once action availability comes from the current review context, these can proceed in parallel:
Task: "Rework the commit review flow and action feedback loop in src-ui/src/views/commit_dialog.rs and src-ui/src/views/mod.rs"
Task: "Anchor fetch, pull, and push UI flows to the current branch context in src-ui/src/views/remote_dialog.rs and src-ui/src/main.rs"
Task: "Tighten branch search, switch, and create workflow in src-ui/src/views/branch_popup.rs and src/git-core/src/branch.rs"
```

## Parallel Example: User Story 4

```bash
# After blocked states are promoted into the workspace, these can proceed in parallel:
Task: "Rebuild conflict continuation between the conflict list and resolver workbench in src-ui/src/main.rs, src-ui/src/widgets/conflict_resolver.rs, and src/git-core/src/diff.rs"
Task: "Normalize actionable Chinese failure copy and next-step hints in src-ui/src/views/remote_dialog.rs, src-ui/src/views/rebase_editor.rs, and src-ui/src/views/mod.rs"
Task: "Detect blocked Git flows and expose continuation signals from core state in src/git-core/src/repository.rs, src/git-core/src/rebase.rs, and src/git-core/src/remote.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational
3. Complete Phase 3: User Story 1
4. **STOP and VALIDATE**: confirm the first screen is a coherent Git workspace with current context, change review, and diff preview
5. Demo the MVP workspace before expanding the action loop

### Incremental Delivery

1. Setup + Foundational define the shared Git workspace shell, chrome, copy, and regression baseline
2. Add US1 to establish the Git-first home screen
3. Add US2 to complete the day-to-day action loop inside the app
4. Add US3 to make the tool feel like an IDE through continuity and recent-project recovery
5. Add US4 to keep users inside the workspace when Git gets blocked
6. Add US5 to supply supporting context without displacing the main review surface and close stash-parity gaps
7. Finish with cross-cutting polish, quality gates, and performance validation

### Parallel Team Strategy

With multiple developers:

1. One stream owns shared workspace-state, routing, and feedback infrastructure in `src-ui/src/state.rs`, `src-ui/src/main.rs`, `src-ui/src/i18n.rs`, `src-ui/src/views/mod.rs`, and `src-ui/src/logging.rs`
2. One stream owns the Git workspace shell and review surfaces in `src-ui/src/views/main_window.rs`, `src-ui/src/widgets/changelist.rs`, `src-ui/src/widgets/diff_viewer.rs`, `src-ui/src/widgets/split_diff_viewer.rs`, and `src-ui/src/widgets/syntax_highlighting.rs`
3. One stream owns the high-frequency action loop in `src-ui/src/views/commit_dialog.rs`, `src-ui/src/views/remote_dialog.rs`, `src-ui/src/views/branch_popup.rs`, `src/git-core/src/index.rs`, `src/git-core/src/branch.rs`, `src/git-core/src/repository.rs`, and `src/git-core/src/remote.rs`
4. One stream owns continuity, risk handling, and context peeks in `src-ui/src/file_watcher.rs`, `src-ui/src/views/history_view.rs`, `src-ui/src/views/tag_dialog.rs`, `src-ui/src/views/stash_panel.rs`, `src-ui/src/views/rebase_editor.rs`, `src-ui/src/widgets/conflict_resolver.rs`, `src/git-core/src/history.rs`, `src/git-core/src/tag.rs`, `src/git-core/src/stash.rs`, and `src/git-core/src/diff.rs`

---

## Notes

- `[P]` tasks are restricted to disjoint files or follow a coordinating prerequisite task
- The suggested MVP scope is **User Story 1 only**, with **User Story 2** as the next highest-value increment
- Regression coverage is intentionally threaded through the stories because Constitution requires IntelliJ-compatible Git parity, not just UI completion
- Do not close the feature until `quickstart.md` confirms both the mainline Git loop and the blocked-state continuation paths
