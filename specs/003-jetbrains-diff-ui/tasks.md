# Tasks: JetBrains-Styled Diff File List Panel

**Input**: Design documents from `/specs/003-jetbrains-diff-ui/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic structure

- [x] T001 [P] Create src/ui/ directory structure with mod.rs, theme.rs, diff_panel.rs, diff_view.rs, and components/ subdirectory
- [x] T002 [P] Create src/git_core/ directory structure with mod.rs, status.rs, and diff.rs
- [x] T003 [P] Configure Cargo.toml with iced 0.13, git2 0.19, and notify 8 dependencies
- [x] T004 [P] Create src/ui/theme.rs with Darcula color palette constants (background #2B2B2B, text #BDBDBD, primary #6897BB)
- [x] T005 Create src/lib.rs to export git_core and ui modules
- [x] T006 Create src/main.rs entry point with CJK font configuration (PingFang SC for macOS, Microsoft YaHei for Windows)

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [x] T007 Create src/git_core/mod.rs with library exports
- [x] T008 Create src/git_core/status.rs with get_changed_files(repo_path: &Path) -> Result<Vec<ChangedFile>, GitError> function
- [x] T009 Create src/git_core/diff.rs with get_file_diff(repo_path: &Path, file_path: &str) -> Result<DiffContent, GitError> function
- [x] T010 Create src/ui/mod.rs with UI module exports
- [x] T011 [P] Create src/ui/components/mod.rs for reusable components
- [x] T012 [P] Implement FileStatus enum in src/ui/components/status_icons.rs with colors: Modified=#6897BB, Added=#629755, Deleted=#6C6C6C, Renamed=#3A8484
- [x] T013 Create DiffPanelState struct in src/ui/diff_panel.rs with fields: files, selected_index, filter, sort_order

**Checkpoint**: Foundation ready - user story implementation can now begin in parallel

---

## Phase 3: User Story 1 - View Changed Files List (Priority: P1) 🎯 MVP

**Goal**: Display a list of all files with uncommitted changes (modified, added, deleted) in the repository

**Independent Test**: Open a repository with changes and verify the changed files appear in the panel with correct status indicators

### Implementation for User Story 1

- [x] T014 [P] [US1] Create ChangedFile struct in src/ui/diff_panel.rs with fields: path, status, selection_state, old_path
- [x] T015 [P] [US1] Create FileStatus enum in src/ui/diff_panel.rs with variants: Modified, Added, Deleted, Renamed
- [x] T016 [US1] Implement DiffPanel widget in src/ui/diff_panel.rs that displays list of ChangedFile items
- [x] T017 [US1] Implement file path display with status indicator for each item in DiffPanel
- [x] T018 [US1] Implement empty state display when repository has no changes (Chinese: "无更改")
- [x] T019 [US1] Connect DiffPanel to git_core status module to fetch changed files
- [x] T020 [US1] Add scroll handling when list exceeds visible area
- [x] T021 [US1] Add logging for changed files panel operations (Chinese labels)

**Checkpoint**: At this point, User Story 1 should be fully functional and testable independently

---

## Phase 4: User Story 2 - JetBrains Visual Theme (Priority: P1)

**Goal**: Apply Darcula dark theme styling to ALL UI components in the application

**Independent Test**: Compare application's color palette, typography, and UI elements against IntelliJ IDEA's appearance

### Implementation for User Story 2

- [x] T022 [P] [US2] Extend src/ui/theme.rs with full Darcula palette: background=#2B2B2B, panel=#313335, editor=#1E1E1E
- [x] T023 [P] [US2] Implement container styling in theme.rs for panels with border color #3B3B3B
- [x] T024 [US2] Apply Darcula theme to main application window in src/main.rs
- [x] T025 [US2] Apply Darcula theme to DiffPanel container styling
- [x] T026 [US2] Implement selection highlighting with color #214283
- [x] T028 [US2] Add border styling (#3B3B3B) to all containers and panels
- [x] T029 [US2] Apply text colors: primary=#BDBDBD, secondary=#808080

**Checkpoint**: All UI components should now use Darcula theme matching IDEA's appearance

---

## Phase 5: User Story 3 - Interactive File Selection (Priority: P2)

**Goal**: Allow users to click on a file in the diff list to select it and open the diff view

**Independent Test**: Click on files in the list and verify selection state is visually indicated and diff view opens

### Implementation for User Story 3

- [x] T030 [P] [US3] Implement click handler on DiffPanel items in src/ui/diff_panel.rs
- [x] T031 [P] [US3] Create DiffView widget in src/ui/diff_view.rs for displaying file diff content
- [x] T032 [US3] Implement selection state management in DiffPanelState (selection_state toggle)
- [x] T033 [US3] Connect file selection to DiffView to display diff when file is clicked
- [x] T034 [US3] Implement diff content rendering in DiffView with old/new content panels
- [x] T035 [US3] Add navigation between files (previous/next) in toolbar
- [x] T036 [US3] Add Chinese labels: "上一个文件", "下一个文件", "文件差异"

**Checkpoint**: User can select files and view their diffs

---

## Phase 6: User Story 4 - File Status Icons (Priority: P2)

**Goal**: Display intuitive icons indicating the type of change for each file

**Independent Test**: Verify correct icons appear for modified, added, and deleted files

### Implementation for User Story 4

- [ ] T037 [P] [US4] Implement status icon rendering in src/ui/components/status_icons.rs
- [ ] T038 [P] [US4] Create icon shapes: Modified (blue dot), Added (green plus), Deleted (red minus), Renamed (teal arrow)
- [ ] T039 [US4] Integrate status icons into DiffPanel item rendering
- [ ] T040 [US4] Apply correct status colors from research.md: Modified=#6897BB, Added=#629755, Deleted=#6C6C6C
- [ ] T041 [US4] Add tooltip on hover showing file status in Chinese (已修改, 已添加, 已删除, 已重命名)

**Checkpoint**: All file status indicators correctly displayed

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

- [x] T042 [P] Add Chinese localization to all UI text (panels, buttons, labels)
- [x] T043 [P] Run cargo clippy to check for code quality issues
- [x] T044 [P] Run cargo test to verify all tests pass
- [ ] T045 Verify SC-001: Changed files display within 1 second of opening panel
- [ ] T046 Verify SC-002: Color palette matches JetBrains within 95% accuracy
- [ ] T047 Verify SC-005: Empty state displays when repository has no changes
- [ ] T048 Add integration test for full diff workflow in tests/integration/
- [ ] T049 Run quickstart.md validation to ensure implementation matches design

**Note**: T027 (scrollbar styling) is not achievable with iced 0.13's current theming API limitations. The native scrollbar appearance is used as fallback.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3-6)**: All depend on Foundational phase completion
  - User stories can then proceed in parallel (if staffed)
  - Or sequentially in priority order (P1 → P2)
- **Polish (Phase 7)**: Depends on all desired user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) - No dependencies on other stories
- **User Story 2 (P1)**: Can start after Foundational (Phase 2) - No dependencies on other stories, can run parallel with US1
- **User Story 3 (P2)**: Depends on US1 (DiffPanel) and US2 (theme) - Needs file list and styling
- **User Story 4 (P2)**: Can start after Foundational (Phase 2) - Can run parallel with US3

### Within Each User Story

- Foundational tasks must complete first (Phase 2)
- Models before widgets
- Core implementation before styling
- Story complete before moving to polish

### Parallel Opportunities

- All Setup tasks marked [P] can run in parallel
- All Foundational tasks marked [P] can run in parallel (within Phase 2)
- US1 and US2 can start in parallel after Foundational completes
- US3 depends on US1, US4 can start after Foundational
- All tasks marked [P] within a phase can run in parallel

---

## Parallel Example

```bash
# After Foundational completes, launch US1 and US2 in parallel:
Task: "[US1] Implement DiffPanel widget in src/ui/diff_panel.rs"
Task: "[US2] Extend theme.rs with full Darcula palette"

# Then launch US3 and US4 in parallel:
Task: "[US3] Implement click handler on DiffPanel items"
Task: "[US4] Implement status icon rendering in status_icons.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL - blocks all stories)
3. Complete Phase 3: User Story 1
4. **STOP and VALIDATE**: Test User Story 1 independently
5. Deploy/demo if ready

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready
2. Add User Story 1 → Test independently → Deploy/Demo (MVP!)
3. Add User Story 2 → Apply Darcula theme
4. Add User Story 3 → Add interactivity
5. Add User Story 4 → Add status icons
6. Polish → Final validation

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together
2. Once Foundational is done:
   - Developer A: User Story 1 (DiffPanel)
   - Developer B: User Story 2 (Theme)
3. Then:
   - Developer A: User Story 3 (Selection + DiffView)
   - Developer B: User Story 4 (Status Icons)

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Avoid: vague tasks, same file conflicts, cross-story dependencies that break independence
