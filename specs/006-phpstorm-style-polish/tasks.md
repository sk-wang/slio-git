# Tasks: PhpStorm 风格的轻量化样式收敛

**Input**: Design documents from `/specs/006-phpstorm-style-polish/`  
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: 本 feature 未要求严格 TDD；不单独生成测试先行任务，但必须包含 `cargo test`、`cargo clippy --workspace --all-targets -- -D warnings` 与基于 `quickstart.md` 的人工样式回归任务。  

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g. `US1`, `US2`)
- Include exact file paths in descriptions

## Phase 1: Setup (Shared Baseline)

**Purpose**: 固化用户截图基线、回归路径和 UI 契约，避免实现过程中重新长出“重”样式

- [X] T001 Freeze the PhpStorm visual baseline, density anti-goals, and style decisions in specs/006-phpstorm-style-polish/research.md
- [X] T002 [P] Finalize the screenshot-based walkthrough and acceptance matrix in specs/006-phpstorm-style-polish/quickstart.md
- [X] T003 [P] Lock compact chrome, popup, and status-surface rules in specs/006-phpstorm-style-polish/contracts/phpstorm-style-contracts.md

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: 建立所有后续样式收敛共用的状态、主题、文案、日志和控件基础；完成前不得进入任何用户故事实现

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [X] T004 Refactor compact style state models for CompactChromeProfile, WorkspaceContextStrip, and LightweightStatusSurface in src-ui/src/state.rs
- [X] T005 [P] Rebalance surface hierarchy, radius, spacing, and layout density tokens in src-ui/src/theme.rs
- [X] T006 [P] Normalize compact Chinese labels, status wording, and popup copy in src-ui/src/i18n.rs
- [X] T007 Wire compact chrome, popup lifecycle, and lightweight feedback routing in src-ui/src/main.rs and src-ui/src/state.rs
- [X] T008 [P] Preserve actionable feedback observability for the lighter UI in src-ui/src/logging.rs
- [X] T009 [P] Define shared compact controls and list primitives in src-ui/src/widgets/button.rs, src-ui/src/widgets/text_input.rs, src-ui/src/widgets/scrollable.rs, and src-ui/src/widgets/statusbar.rs

**Checkpoint**: Foundation ready - shared density tokens, compact shell state, localized copy, and observability are aligned for all user stories

---

## Phase 3: User Story 1 - 第一屏像 PhpStorm 一样轻 (Priority: P1) 🎯 MVP

**Goal**: 让仓库工作区首屏更薄、更连续，把主要视觉面积还给改动树和 diff

**Independent Test**: 打开任意仓库后，只看第一屏即可感知主内容连续、顶部更薄、卡片感减弱，改动树和 diff 成为视觉中心

### Implementation for User Story 1

- [X] T010 [US1] Collapse the repository workspace chrome and remove heavy stacked containers in src-ui/src/views/main_window.rs
- [X] T011 [P] [US1] Compress change tree spacing, section framing, and visible density in src-ui/src/widgets/changelist.rs
- [X] T012 [P] [US1] Flatten diff container chrome and reclaim visible workspace area in src-ui/src/widgets/diff_viewer.rs and src-ui/src/widgets/split_diff_viewer.rs
- [X] T013 [US1] Replace the heavy repository/branch summary with one compact context strip in src-ui/src/views/main_window.rs and src-ui/src/widgets/statusbar.rs
- [X] T014 [US1] Keep no-repository and no-changes workspace states lightweight in src-ui/src/views/main_window.rs and src-ui/src/widgets/changelist.rs

**Checkpoint**: User Story 1 delivers a lighter first-screen workspace MVP with thinner chrome and content-first proportions

---

## Phase 4: User Story 2 - 分支和动作弹层像 JetBrains 原生面板 (Priority: P1)

**Goal**: 将当前分支入口重构为更紧凑的 JetBrains 风格列表弹层，承载分支切换和高频 Git 动作

**Independent Test**: 点击当前分支入口后，用户可在一个轻量列表式 popup 中搜索、查看最近/本地/远程分支，并完成刷新、提交、拉取、推送或创建分支

### Implementation for User Story 2

- [X] T015 [US2] Extend popup state for current-branch summary, grouped actions, and minimal metadata density in src-ui/src/state.rs
- [X] T016 [P] [US2] Expose branch tracking and sync hints for compact popup rows in src/git-core/src/repository.rs
- [X] T017 [US2] Rebuild the popup shell into a compact current-branch plus search plus grouped-list layout in src-ui/src/views/branch_popup.rs
- [X] T018 [P] [US2] Route popup trigger, open-close lifecycle, and focus return behavior in src-ui/src/main.rs and src-ui/src/views/main_window.rs
- [X] T019 [US2] Render compact high-frequency action rows and group headers in src-ui/src/views/branch_popup.rs and src-ui/src/widgets/button.rs
- [X] T020 [US2] Bind recent, local, and remote branch rows to minimal metadata and selection behavior in src-ui/src/views/branch_popup.rs and src-ui/src/state.rs

**Checkpoint**: User Story 2 yields a JetBrains-like branch/action popup without reverting to a heavy management page

---

## Phase 5: User Story 3 - 列表、标签和状态元素更克制 (Priority: P2)

**Goal**: 让文件列表、badge、diff 顶栏和状态区统一向“薄、紧、平”的 IDE 控件语言收敛

**Independent Test**: 回归改动列表、diff 顶栏和底部状态区时，能明显感知元素更小、更弱化、更连续，但不影响识别和操作

### Implementation for User Story 3

- [X] T021 [US3] Compress file-row height, group-header spacing, and inline badges in src-ui/src/widgets/changelist.rs
- [X] T022 [P] [US3] Flatten diff header counters, paddings, and mode toggles in src-ui/src/widgets/diff_viewer.rs and src-ui/src/widgets/split_diff_viewer.rs
- [X] T023 [P] [US3] Downshift badge, button, and input emphasis for normal states in src-ui/src/theme.rs, src-ui/src/widgets/button.rs, and src-ui/src/widgets/text_input.rs
- [X] T024 [P] [US3] Make bottom status surfaces and stable-state feedback lower-emphasis in src-ui/src/widgets/statusbar.rs and src-ui/src/views/main_window.rs
- [X] T025 [US3] Shorten success and info copy while keeping stable screens banner-free in src-ui/src/views/branch_popup.rs, src-ui/src/views/commit_dialog.rs, src-ui/src/views/remote_dialog.rs, src-ui/src/views/tag_dialog.rs, and src-ui/src/views/stash_panel.rs

**Checkpoint**: User Story 3 removes the remaining “heavy controls” feeling from lists, badges, and status surfaces

---

## Phase 6: User Story 4 - 更轻的样式下仍保留清晰可读性与可达性 (Priority: P2)

**Goal**: 在持续减重的同时，保住选中态、错误态、长文本、窄窗口和核心 Git 入口的清晰度

**Independent Test**: 在选中文件、切换分支、出现冲突/错误、长名称和窄窗口场景下，用户仍能快速识别焦点和关键动作，且核心 Git 能力不被埋没

### Implementation for User Story 4

- [X] T026 [US4] Rework selected-row and active-branch highlights to stay clear without thick borders in src-ui/src/widgets/changelist.rs, src-ui/src/views/branch_popup.rs, and src-ui/src/theme.rs
- [X] T027 [P] [US4] Keep conflict and rebase error emphasis strong-but-local in src-ui/src/widgets/conflict_resolver.rs and src-ui/src/views/rebase_editor.rs
- [X] T028 [P] [US4] Handle long repository and branch text plus narrow-window overflow in src-ui/src/views/main_window.rs, src-ui/src/views/branch_popup.rs, and src-ui/src/widgets/scrollable.rs
- [X] T029 [US4] Preserve clear reachability for history, remote, tag, stash, and secondary Git actions in src-ui/src/views/main_window.rs, src-ui/src/views/history_view.rs, src-ui/src/views/remote_dialog.rs, src-ui/src/views/tag_dialog.rs, and src-ui/src/views/stash_panel.rs
- [X] T030 [US4] Validate no-repository, no-actions, and failed-remote edge states remain readable in src-ui/src/views/main_window.rs, src-ui/src/views/branch_popup.rs, and src-ui/src/views/remote_dialog.rs

**Checkpoint**: User Story 4 proves the lighter UI still communicates focus, risk, and capability paths clearly

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: 收尾对照、跑质量门禁，并确认成功标准真实落地

- [X] T031 [P] Refresh the screenshot-comparison walkthrough and edge-case matrix in specs/006-phpstorm-style-polish/quickstart.md
- [X] T032 [P] Capture final acceptance notes for SC-001 through SC-007 in specs/006-phpstorm-style-polish/spec.md and specs/006-phpstorm-style-polish/quickstart.md
- [ ] T033 Run workspace quality gates and the manual style walkthrough from Cargo.toml, src-ui/Cargo.toml, and specs/006-phpstorm-style-polish/quickstart.md

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3+)**: All depend on Foundational phase completion
- **Polish (Phase 7)**: Depends on all desired user stories being complete

### User Story Dependencies

- **US1 (P1)**: Starts immediately after Foundational - defines the lighter first-screen MVP
- **US2 (P1)**: Depends on Foundational density/state work and benefits from US1 compact workspace chrome
- **US3 (P2)**: Depends on Foundational styling primitives and can follow once US1 establishes the lighter shell baseline
- **US4 (P2)**: Depends on US1/US2 visual hierarchy so clarity, overflow, and reachability can be validated against the lighter shell

### Within Each User Story

- Shared shell/state/token changes before view-specific style polish
- Popup structure before popup action-row and branch-row refinement
- Normal-state density reduction before edge-state and clarity tuning
- Story checkpoint validation should complete before moving to final polish

### Parallel Opportunities

- **Setup**: T002 and T003 can run in parallel after T001 freezes the visual baseline
- **Foundational**: T005, T006, T008, and T009 can run in parallel after T004 establishes shared compact state
- **US1**: T011 and T012 can run in parallel while T010/T013 reshape the shell
- **US2**: T016 and T018 can run in parallel after T015 defines the popup state contract
- **US3**: T022, T023, and T024 can run in parallel after T021 anchors list density changes
- **US4**: T027 and T028 can run in parallel after T026 defines the new clarity baseline
- **Polish**: T031 and T032 can run in parallel before T033 closes the feature

---

## Parallel Example: User Story 1

```bash
# After the shell density baseline is in place, these can proceed in parallel:
Task: "Compress change tree spacing, section framing, and visible density in src-ui/src/widgets/changelist.rs"
Task: "Flatten diff container chrome and reclaim visible workspace area in src-ui/src/widgets/diff_viewer.rs and src-ui/src/widgets/split_diff_viewer.rs"
```

## Parallel Example: User Story 2

```bash
# Once popup state fields are defined, these can proceed in parallel:
Task: "Expose branch tracking and sync hints for compact popup rows in src/git-core/src/repository.rs"
Task: "Route popup trigger, open-close lifecycle, and focus return behavior in src-ui/src/main.rs and src-ui/src/views/main_window.rs"
```

## Parallel Example: User Story 3

```bash
# After list density starts shrinking, these can proceed in parallel:
Task: "Flatten diff header counters, paddings, and mode toggles in src-ui/src/widgets/diff_viewer.rs and src-ui/src/widgets/split_diff_viewer.rs"
Task: "Downshift badge, button, and input emphasis for normal states in src-ui/src/theme.rs, src-ui/src/widgets/button.rs, and src-ui/src/widgets/text_input.rs"
Task: "Make bottom status surfaces and stable-state feedback lower-emphasis in src-ui/src/widgets/statusbar.rs and src-ui/src/views/main_window.rs"
```

## Parallel Example: User Story 4

```bash
# After the lighter selection language is established, these can proceed in parallel:
Task: "Keep conflict and rebase error emphasis strong-but-local in src-ui/src/widgets/conflict_resolver.rs and src-ui/src/views/rebase_editor.rs"
Task: "Handle long repository and branch text plus narrow-window overflow in src-ui/src/views/main_window.rs, src-ui/src/views/branch_popup.rs, and src-ui/src/widgets/scrollable.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational
3. Complete Phase 3: User Story 1
4. **STOP and VALIDATE**: confirm the first screen is lighter, the top chrome is thinner, and changes + diff dominate the visible area
5. Demo the lighter workspace before rebuilding the popup

### Incremental Delivery

1. Setup + Foundational define compact tokens, shell state, and shared controls
2. Add US1 to deliver the lighter first-screen workspace MVP
3. Add US2 to deliver the JetBrains-style branch/action popup
4. Add US3 to slim down lists, badges, and status surfaces
5. Add US4 to harden clarity, overflow handling, and capability reachability
6. Finish with manual comparison, acceptance capture, and quality gates

### Parallel Team Strategy

With multiple developers:

1. One stream owns shared density/state infrastructure in src-ui/src/state.rs, src-ui/src/theme.rs, src-ui/src/i18n.rs, and src-ui/src/logging.rs
2. One stream owns workspace shell and first-screen density in src-ui/src/views/main_window.rs, src-ui/src/widgets/changelist.rs, src-ui/src/widgets/diff_viewer.rs, and src-ui/src/widgets/split_diff_viewer.rs
3. One stream owns popup behavior and branch metadata in src-ui/src/views/branch_popup.rs, src-ui/src/main.rs, and src/git-core/src/repository.rs
4. One stream owns edge-state clarity and secondary capability surfaces in src-ui/src/widgets/conflict_resolver.rs, src-ui/src/views/history_view.rs, src-ui/src/views/remote_dialog.rs, src-ui/src/views/tag_dialog.rs, src-ui/src/views/stash_panel.rs, and src-ui/src/views/rebase_editor.rs

---

## Notes

- `[P]` tasks are limited to disjoint files or follow a coordinating prerequisite task
- The suggested MVP scope is **User Story 1 only**, with **User Story 2** as the next highest-value increment
- This feature is style/density polish only; it must not change `git-core` ownership boundaries or Git interaction semantics
- Do not close the feature until `quickstart.md` confirms both “更轻” and “仍然清晰可达” goals
