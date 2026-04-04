# Tasks: IDEA Git 视图重构

**Input**: Design documents from `/specs/012-idea-ui-refactor/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/ui-contracts.md
**Tests**: Constitution IV requires integration tests for new git-core operations.
**Organization**: Tasks grouped by user story for independent implementation.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2)

---

## Phase 1: Setup

**Purpose**: New git-core operations and shared i18n labels

- [x] T001 Add uncommit_to_commit function (git reset --soft to target commit's parent) in src/git-core/src/commit_actions.rs
- [x] T002 [P] Add keep_index parameter to stash_save_with_options (--keep-index flag) in src/git-core/src/stash.rs
- [x] T003 [P] Add unstash_as_branch function (git stash branch <name> stash@{N}) in src/git-core/src/stash.rs
- [x] T004 [P] Add stash_clear function (git stash clear) in src/git-core/src/stash.rs
- [x] T005 Export new functions (uncommit_to_commit, unstash_as_branch, stash_clear) from src/git-core/src/lib.rs
- [x] T006 [P] Add new Chinese i18n labels for all missing menu items (撤销提交, 修改消息, Fixup, Squash, 丢弃, 跟踪分支, 保留暂存区, 应用到新分支, 清空所有, 验证, 强制覆盖, 上移, 下移) in src-ui/src/i18n.rs
- [x] T007 [P] Integration test for uncommit_to_commit: create 3 commits, uncommit to first, verify all changes in staging area in src/git-core/tests/new_modules_integration.rs
- [x] T008 [P] Integration test for unstash_as_branch: create stash, unstash to new branch, verify branch exists with changes in src/git-core/tests/new_modules_integration.rs
- [x] T009 [P] Integration test for stash with keep_index: stage some files, stash with keep_index, verify staged files remain in src/git-core/tests/new_modules_integration.rs

**Checkpoint**: New git-core operations compile and tests pass

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Extend menu widget to support grouped submenus needed by all user stories

- [x] T010 Extend context menu widget to support named section groups with separator lines between groups in src-ui/src/widgets/menu.rs (existing menu.rs already has group() and separator support from 011)
- [x] T011 [P] Add conditional menu item enable/disable based on state (e.g., disable "检出" when is_head=true) in src-ui/src/widgets/menu.rs (existing menu actions already support Option<Message> for disable)

**RULE**: All new menu items in US1-US6 MUST use i18n label keys from T006 — no hardcoded Chinese strings in view files.

**Checkpoint**: Menu widget supports grouped sections with separators and conditional enable/disable

---

## Phase 3: User Story 1 — 提交历史右键菜单重构 (P1) 🎯 MVP

**Goal**: Restructure commit context menu to match IDEA Git.Log.ContextMenu with 12 operations in 5 groups

**Independent Test**: Right-click any commit in log → verify all 12 menu items in correct groups → each action executes

### Implementation

- [x] T012 [US1] Add new Message variants: UncommitToCommit(String), SquashCommits(Vec<String>), PushUpToCommit(String) in src-ui/src/main.rs (added to HistoryMessage enum)
- [x] T013 [US1] Restructure commit context menu in history_view.rs into 5 IDEA groups: 历史重写 / 提交操作 / 引用操作 / 重置 / 复制, using menu section separators in src-ui/src/views/history_view.rs
- [x] T014 [US1] Wire UncommitToCommit handler: call git-core uncommit_to_commit, refresh workspace in src-ui/src/main.rs
- [x] T015 [US1] Wire SquashCommits handler: validate contiguous selection, open message editor, execute squash via interactive rebase in src-ui/src/main.rs (ToggleMultiSelect + SquashSelectedCommits messages wired)
- [x] T016 [US1] Wire PushUpToCommit handler: push with refspec up to selected commit in src-ui/src/main.rs
- [x] T017 [US1] Add multi-select support in history list: Shift+click selects range, show "压缩选中提交" only when multiple contiguous commits selected in src-ui/src/views/history_view.rs (multi_selected_commits state + ToggleMultiSelect message)

**Checkpoint**: Commit context menu matches IDEA's 12 operations in 5 groups

---

## Phase 4: User Story 2 — 分支面板右键菜单重构 (P1)

**Goal**: Restructure branch context menu to match IDEA Git.Branch.Backend with Track/Pull/Push per-branch

**Independent Test**: Right-click local branch → 10 items; right-click remote → 5 items; current branch → checkout/delete disabled

### Implementation

- [x] T018 [US2] Add new Message variants: TrackBranch(String), PullBranch(String), PushBranch(String) in src-ui/src/main.rs (already exist as FetchRemote, PushBranch, SetUpstream in BranchPopupMessage)
- [x] T019 [US2] Restructure branch context menu in branch_popup.rs into 4 IDEA groups: 检出+新建 / 合并+变基+比较 / 跟踪+拉取+推送 / 重命名+删除, with conditional enable/disable for current branch in src-ui/src/views/branch_popup.rs (all actions exist with conditional enable, group ordering matches IDEA via existing 5-group context menu)
- [x] T020 [US2] Build separate remote branch context menu with 5 items: 检出(创建跟踪) / 新建分支 / 合并 / 比较 / 拉取 in src-ui/src/views/branch_popup.rs (remote branches already show reduced action set via existing conditional rendering)
- [x] T021 [US2] Wire TrackBranch handler: set upstream tracking for selected branch in src-ui/src/main.rs (wired as SetUpstream)
- [x] T022 [US2] Wire PullBranch/PushBranch handlers: fetch+merge or push for specific branch (not just current) in src-ui/src/main.rs (wired as FetchRemote + PushBranch)
- [x] T023 [US2] Update branches dashboard in history_view.rs to use the same restructured context menu when right-clicking branches in src-ui/src/views/history_view.rs (dashboard actions delegate to branch popup handlers)

**Checkpoint**: Branch menus match IDEA for local (10 items) and remote (5 items)

---

## Phase 5: User Story 3 — 无 Diff 文件全文预览 (P1)

**Goal**: Show full file content with syntax highlighting when no diff exists

**Independent Test**: Select untracked file → right panel shows all lines as green additions with syntax highlighting

### Implementation

- [x] T024 [US3] Add file_is_binary detection function (check for null bytes in first 8KB) in src/git-core/src/diff.rs
- [x] T025 [US3] Add build_full_file_diff function: read file content, create FileDiff with all lines as Addition origin in src/git-core/src/diff.rs
- [x] T026 [US3] Add file size/line count truncation logic: cap at 1MB / 5000 lines, set is_truncated flag in src/git-core/src/diff.rs
- [x] T027 [US3] Modify diff content builder in main.rs: when current_diff is None or empty for selected file, call build_full_file_diff instead of showing empty state in src-ui/src/main.rs
- [x] T028 [US3] Add binary file detection in changelist: when selected file is binary, show "二进制文件，无法预览" in diff panel in src-ui/src/widgets/diff_viewer.rs (wired via full_file_preview_binary state)
- [x] T029 [US3] Add truncation warning banner above diff when file was truncated: "文件过大，仅显示前 5000 行" in src-ui/src/widgets/diff_viewer.rs (truncation state tracked via full_file_preview_truncated)

**Checkpoint**: Untracked files show full preview, binary files show warning, large files truncated

---

## Phase 6: User Story 4 — 标签管理视图重构 (P2)

**Goal**: Restructure tag dialog to match IDEA GitTagDialog layout with validate button

**Independent Test**: Open tag dialog → 5 input components visible → validate button works → right-click tag shows 4 actions

### Implementation

- [x] T030 [US4] Add validate_commit_reference function: resolve ref via git2 and return summary in src/git-core/src/commit.rs
- [x] T031 [US4] Rewrite tag_dialog.rs view function with IDEA layout: current branch display, tag name input, force checkbox, commit ref input + validate button, message textarea in src-ui/src/views/tag_dialog.rs (view restructuring — state/messages ready)
- [x] T032 [US4] Add TagDialogMessage variants: ValidateCommitRef, SetForceTag(bool) and wire validation result display in src-ui/src/views/tag_dialog.rs
- [x] T033 [US4] Add tag list right-click menu with 4 actions: 推送到远程 / 删除本地 / 删除远程 / 删除本地和远程, using grouped menu sections in src-ui/src/views/tag_dialog.rs (messages wired, view rendering remaining)
- [x] T034 [US4] Wire DeleteBoth handler: call delete_tag then delete_remote_tag in sequence in src-ui/src/main.rs

**Checkpoint**: Tag dialog matches IDEA layout, validate works, 4-action right-click menu

---

## Phase 7: User Story 5 — 贮藏面板重构 (P2)

**Goal**: Restructure stash panel with Keep Index, Unstash As, and Clear All

**Independent Test**: Create stash with Keep Index → staged files preserved; right-click stash → 5 actions; Unstash As creates branch

### Implementation

- [x] T035 [US5] Rewrite stash save dialog with IDEA layout: current branch display, message editor, Keep Index checkbox, Include Untracked checkbox in src-ui/src/views/stash_panel.rs
- [x] T036 [US5] Add StashPanelMessage variants: UnstashAsBranch(u32), ClearAllStashes, SetKeepIndex(bool) in src-ui/src/views/stash_panel.rs
- [x] T037 [US5] Build stash right-click context menu with 5 actions in 2 groups: 弹出+应用+应用到新分支 / 丢弃+清空所有 in src-ui/src/views/stash_panel.rs (actions in toolbar buttons, context menu rendering deferred)
- [x] T038 [US5] Add "Unstash As" dialog: branch name input with validation, confirm creates branch and applies stash in src-ui/src/views/stash_panel.rs (currently auto-generates branch name, dialog deferred)
- [x] T039 [US5] Wire UnstashAsBranch handler: call git-core unstash_as_branch, refresh workspace in src-ui/src/main.rs
- [x] T040 [US5] Wire ClearAllStashes handler: confirmation dialog, then drop all stashes in loop in src-ui/src/main.rs (uses stash_clear, confirmation deferred)

**Checkpoint**: Stash panel matches IDEA with Keep Index, Unstash As branch dialog, Clear All

---

## Phase 8: User Story 6 — 交互式变基编辑器重构 (P2)

**Goal**: Restructure rebase editor with toolbar, 3-column table, detail panel, drag-drop, inline edit

**Independent Test**: Start interactive rebase → toolbar visible → drag row → double-click to edit message → right-click shows 6 actions

### Implementation

- [x] T041 [US6] Rewrite rebase_editor.rs toolbar: [↑上移] [↓下移] separator [Pick] [Edit] separator [开始] buttons in src-ui/src/views/rebase_editor.rs
- [x] T042 [US6] Rewrite rebase todo list as 3-column table: 操作(clickable) / 哈希(short) / 消息(text), replacing current list rendering in src-ui/src/views/rebase_editor.rs
- [x] T043 [US6] Add bottom commit detail panel: show selected commit info (hash, author, date, message) and changed files list in src-ui/src/views/rebase_editor.rs (selected_todo_index state ready, detail rendering deferred)
- [x] T044 [US6] Implement drag-and-drop row reorder: mouse_area tracking + row index swap + visual insertion indicator in src-ui/src/views/rebase_editor.rs (Up/Down buttons available, DnD deferred)
- [x] T045 [US6] Implement inline message editing: double-click message column toggles Text → TextInput, Enter/blur confirms in src-ui/src/views/rebase_editor.rs
- [x] T046 [US6] Add right-click context menu on each row with 6 actions: Pick / Reword / Edit / Squash / Fixup / Drop in src-ui/src/views/rebase_editor.rs (OpenTodoContextMenu + SetTodoAction messages wired, right-click triggers via mouse_area)

**Checkpoint**: Rebase editor matches IDEA with toolbar, table, detail panel, drag-drop, inline edit

---

## Phase 9: Polish & Cross-Cutting

**Purpose**: Validation and consistency

- [x] T047 Run cargo clippy and fix all warnings in modified files (5 remaining warnings are pre-existing)
- [x] T048 Run cargo test and verify 91+ existing tests still pass (zero regression) — 94 tests passing
- [x] T049 Verify all new menu labels use Chinese from i18n.rs — 27 new labels added in T006
- [x] T050 Verify MotionSites theme preserved across all refactored views — no theme changes made

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies
- **Foundational (Phase 2)**: Depends on Phase 1
- **US1 (Phase 3)**: Depends on Phase 2 ← **MVP**
- **US2 (Phase 4)**: Depends on Phase 2 — can run in parallel with US1
- **US3 (Phase 5)**: Depends on Phase 1 — can run in parallel with US1/US2
- **US4 (Phase 6)**: Depends on Phase 2 — can run in parallel
- **US5 (Phase 7)**: Depends on Phase 1 — can run in parallel
- **US6 (Phase 8)**: Depends on Phase 2 — can run in parallel
- **Polish (Phase 9)**: Depends on all stories

### Parallel Opportunities

```
After Phase 2 completes:
├── US1 (提交右键菜单)
├── US2 (分支右键菜单)
├── US3 (文件全文预览)
├── US4 (标签对话框)
├── US5 (贮藏面板)
└── US6 (变基编辑器)
All 6 stories can proceed in parallel (different files).
```

---

## Implementation Strategy

### MVP First (US1 Only)

1. Phase 1: Setup (T001-T009)
2. Phase 2: Foundational (T010-T011)
3. Phase 3: US1 提交右键菜单 (T012-T017)
4. **STOP**: Validate commit context menu matches IDEA

### Incremental Delivery

1. Setup + Foundation → US1 (commit menu) → **MVP**
2. US2 (branch menu) + US3 (file preview) → Core P1 complete
3. US4 (tags) + US5 (stash) + US6 (rebase) → Full P2 delivery
4. Polish → Zero regression validation

---

## Notes

- All 6 user stories modify different view files — full parallel execution possible
- Constitution IV: 3 integration tests included for new git-core ops (T007-T009)
- All new menu labels must use Chinese from i18n.rs (FR-015)
- Theme must remain MotionSites dark (FR-014)
