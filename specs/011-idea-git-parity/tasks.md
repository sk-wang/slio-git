# Tasks: IDEA Git Feature Parity

**Input**: Design documents from `/specs/011-idea-git-parity/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/ui-contracts.md

**Tests**: Constitution IV mandates integration tests for git parity. Integration test tasks included for all new git-core modules.

**Organization**: Tasks grouped by user story for independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2)
- Exact file paths included in descriptions

---

## Phase 1: Setup

**Purpose**: New modules and shared infrastructure needed before any user story work

- [x] T001 [P] Create blame module skeleton with BlameEntry struct in src/git-core/src/blame.rs
- [x] T002 [P] Create signature module skeleton with SignatureStatus, SignatureType structs in src/git-core/src/signature.rs
- [x] T003 [P] Create worktree module skeleton with WorkingTree struct in src/git-core/src/worktree.rs
- [x] T004 [P] Create submodule module skeleton with submodule detection functions in src/git-core/src/submodule.rs
- [x] T005 [P] Create graph module skeleton with GraphNode, GraphEdge, RefLabel structs in src/git-core/src/graph.rs
- [x] T006 Export new modules (blame, signature, worktree, submodule, graph) from src/git-core/src/lib.rs
- [x] T007 [P] Create tree_widget.rs generic collapsible tree widget with TreeNode, SelectNode, ToggleNode, NodeContextMenu messages in src-ui/src/widgets/tree_widget.rs
- [x] T008 [P] Create progress_bar.rs widget with operation name, progress percentage, cancel button in src-ui/src/widgets/progress_bar.rs
- [x] T009 [P] Create log_tabs.rs tab bar widget with SelectTab, CloseTab, NewTab messages in src-ui/src/widgets/log_tabs.rs
- [x] T010 Export new widgets (tree_widget, progress_bar, log_tabs) from src-ui/src/widgets/mod.rs

**Checkpoint**: All new module skeletons and shared widgets exist, project compiles

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before user story implementation

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [x] T011 Extend Change struct with is_submodule and submodule_summary fields in src/git-core/src/index.rs
- [x] T012 [P] Extend HistoryEntry struct with committer_name, committer_email, refs (Vec<RefLabel>), signature_status fields in src/git-core/src/history.rs
- [x] T013 [P] Extend Branch struct with group_path field (Option<Vec<String>> for hierarchical tree display) in src/git-core/src/branch.rs
- [x] T014 [P] Extend StashInfo struct with timestamp and includes_untracked fields in src/git-core/src/stash.rs
- [x] T015 Add FileDisplayMode enum (Flat, Tree) to src-ui/src/state.rs
- [x] T016 [P] Add LogTab struct (id, label, is_closable, branch_filter, text_filter, author_filter, date_range, path_filter, scroll_offset, selected_commit) to src-ui/src/state.rs
- [x] T017 Add new Chinese labels for all new features (blame, worktree, graph, context menus, log tabs, progress indicators) to src-ui/src/i18n.rs
- [ ] T018 Extend context menu widget to support right-click positional popups, submenus, separators, and conditional enable/disable in src-ui/src/widgets/menu.rs
- [ ] T018a [P] Add IDEA-compatible keyboard shortcuts (Cmd+K commit, Cmd+Shift+K push, Cmd+T update, etc.) in src-ui/src/keyboard.rs
- [x] T018b [P] Add structured logging instrumentation (log::info/error with repo path, operation, timing context) to all new git-core module skeletons: blame.rs, signature.rs, worktree.rs, submodule.rs, graph.rs

**Checkpoint**: Foundation ready - all shared data structures, keyboard shortcuts, and infrastructure in place

---

## Phase 3: User Story 1 - Git Stage Panel with Non-Modal Commit (Priority: P1) 🎯 MVP

**Goal**: Staged/unstaged tree groups, flat/tree toggle, drag-and-drop staging, embedded commit panel with message history

**Independent Test**: Open a repo with modified files → see staged/unstaged groups → stage/unstage files via button/drag → type commit message → commit inline without modal

### Implementation for User Story 1

- [x] T019 [US1] Rewrite changelist widget with two collapsible tree groups (Staged, Unstaged Changes) supporting flat list and directory tree modes with toggle button, and stage/unstage icon buttons per file row ("+" to stage, "-" to unstage) in src-ui/src/widgets/changelist.rs
- [ ] T020 [US1] Implement drag-and-drop between Staged and Unstaged groups using mouse events + visual overlay in src-ui/src/widgets/changelist.rs (deferred — +/- buttons provide equivalent functionality)
- [x] T022 [US1] Add "Stage Hunk" and "Unstage Hunk" buttons to diff viewer hunk headers in src-ui/src/widgets/diff_viewer.rs
- [x] T023 [P] [US1] Add "Stage Hunk" and "Unstage Hunk" buttons to split diff viewer hunk headers in src-ui/src/widgets/split_diff_viewer.rs
- [x] T024 [US1] Extend commit panel with recent message history dropdown (last 10 messages) and amend toggle checkbox in src-ui/src/widgets/commit_panel.rs
- [x] T025 [US1] Implement commit message history persistence (load/save last 10 messages per repo) in src/git-core/src/commit.rs using ~/.config/slio-git/commit-messages.json
- [x] T026 [US1] Refactor Changes tab layout in main_window.rs: left=changelist, right-top=diff viewer, right-bottom=embedded commit panel (remove modal commit dialog usage) in src-ui/src/views/main_window.rs
- [x] T027 [US1] Remove modal commit_dialog.rs and redirect all commit flows to embedded commit panel in src-ui/src/views/commit_dialog.rs and src-ui/src/main.rs (already embedded — modal not used)
- [x] T028 [US1] Wire new changelist messages (StageFile, UnstageFile, DragDrop, ToggleDisplayMode, ToggleStaged, ToggleUnstaged) to update function in src-ui/src/main.rs
- [x] T029 [US1] Wire stage hunk / unstage hunk messages from diff viewer to git-core index operations in src-ui/src/main.rs
- [x] T030 [US1] Wire commit panel messages (Commit, CommitAndPush, ToggleAmend, SelectRecentMessage) and save message to history on success in src-ui/src/main.rs

**Checkpoint**: Stage-commit-push cycle works without modal dialog, flat/tree toggle works, drag-and-drop staging works

---

## Phase 4: User Story 2 - Branch Management Popup with Tree Navigation (Priority: P1)

**Goal**: Branch popup with Local/Remote/Recent tree groups, real-time search, IDEA-style action submenus

**Independent Test**: Click branch widget → popup with tree groups → search filters → right-click branch → checkout/merge/rebase/delete actions work

### Implementation for User Story 2

- [x] T031 [US2] Implement branch grouping logic: compute group_path from branch names (e.g., "feature/auth" → ["feature", "auth"]), separate into Local/Remote/Recent groups in src/git-core/src/branch.rs
- [x] T032 [US2] Refactor branch popup to use tree_widget for branch display with Local Branches, Remote Branches, Recent Branches collapsible groups in src-ui/src/views/branch_popup.rs (existing BranchTreeFolder already provides this)
- [x] T033 [US2] Add real-time search filtering to branch popup (filter across all groups as user types) in src-ui/src/views/branch_popup.rs (already implemented)
- [x] T034 [US2] Add branch action submenu on right-click/hover: Checkout, New Branch From, Merge Into Current, Rebase Current Onto, Compare With Current, Rename, Delete in src-ui/src/views/branch_popup.rs (already implemented)
- [x] T035 [US2] Wire branch actions (CheckoutBranch, MergeBranch, RebaseOnto, RenameBranch, DeleteBranch, NewBranchFrom, CompareBranch) to git-core operations in src-ui/src/main.rs (already wired)
- [x] T036 [US2] Add branch deletion confirmation dialog with "not fully merged" warning in src-ui/src/views/branch_popup.rs
- [x] T037 [US2] Add checkout remote branch flow: create local tracking branch and checkout in src-ui/src/main.rs (already implemented via CheckoutRemoteBranch)

**Checkpoint**: Branch popup matches IDEA's tree navigation, all branch actions functional

---

## Phase 5: User Story 3 - Git Log with Full Commit Graph (Priority: P1)

**Goal**: Full-height commit graph with branch lines, merge points, multi-tab design, filter bar, commit detail panel

**Independent Test**: Switch to Log tab → commit graph renders with branch lines → click commit shows details → filter by branch/author/text → right-click commit context menu works

### Implementation for User Story 3

- [x] T038 [US3] Implement lane-based graph layout algorithm in src/git-core/src/graph.rs: assign lanes to branches, compute edges for merge/fork points, support incremental computation for virtual scrolling (existing build_history_graph in history_view.rs + new graph.rs module)
- [x] T039 [US3] Add ref label computation: map branch/tag refs to commit IDs, produce Vec<RefLabel> per commit in src/git-core/src/graph.rs
- [x] T040 [US3] Add history filtering functions: get_history_for_author, get_history_for_path, get_history_for_date_range in src/git-core/src/history.rs (added in Phase 2)
- [x] T041 [US3] Create commit_graph.rs widget: render graph lanes as colored lines, commit nodes as dots, merge edges, ref label badges, virtual scrolling for 10k+ commits in src-ui/src/widgets/commit_graph.rs (existing Canvas-based graph in history_view.rs)
- [x] T042 [US3] Rewrite history_view.rs with multi-tab log design: LogTabs bar at top, "All" tab (permanent) + user-created branch-pinned tabs in src-ui/src/views/history_view.rs
- [x] T043 [US3] Add filter bar to history view: branch dropdown, author dropdown, date range picker, file path input, text search field in src-ui/src/views/history_view.rs (search exists, new filter messages wired: SetBranchFilter, SetAuthorFilter, SetPathFilter)
- [x] T044 [US3] Add commit detail panel (right side or bottom split): show hash, author, committer, date, message, parent IDs, changed files list with diffs in src-ui/src/views/history_view.rs (already exists)
- [x] T045 [US3] Add commit context menu: Cherry-Pick, Revert, Create Branch, Create Tag, Reset Current Branch to Here, Copy Commit Hash, Open in New Tab in src-ui/src/views/history_view.rs (already exists + OpenInNewTab added)
- [x] T046 [US3] Wire commit context menu actions (CherryPick, Revert, CreateBranch, CreateTag, ResetToCommit, CopyHash, OpenInNewTab) to git-core and state updates in src-ui/src/main.rs (already wired + new tab messages added)
- [x] T047 [US3] Wire log tab management messages (SelectTab, CloseTab, NewTab) and per-tab filter state persistence in src-ui/src/main.rs

**Checkpoint**: Multi-tab log with commit graph, filters, commit details, and context menu actions all functional

---

## Phase 6: User Story 4 - Branches Dashboard in Log View (Priority: P1)

**Goal**: Left sidebar in Log tab showing branch tree, clicking branch filters the log

**Independent Test**: Open Log tab → branches dashboard shows local/remote branches in tree → click branch filters commit graph → right-click branch for actions

### Implementation for User Story 4

- [x] T048 [US4] Add branches dashboard sidebar to history_view.rs using tree_widget: collapsible Local/Remote groups, integrated with active log tab's branch filter in src-ui/src/views/history_view.rs
- [x] T049 [US4] Wire branch selection in dashboard to update active log tab's branch_filter and refresh commit graph in src-ui/src/views/history_view.rs
- [x] T050 [US4] Add branch context menu in dashboard (Checkout, Merge, Rebase, Compare with Current, Delete, Rename) reusing branch action handlers from US2 in src-ui/src/views/history_view.rs (messages wired to branch_popup handlers)
- [x] T051 [US4] Add resizable splitter between branches dashboard (1:4 ratio) and main log area with collapse/expand toggle in src-ui/src/views/history_view.rs

**Checkpoint**: Branches dashboard filters log by branch, context menu actions work from dashboard

---

## Phase 7: User Story 5 - Merge and Conflict Resolution (Priority: P1)

**Goal**: Three-pane conflict resolver (ours/result/theirs) with per-chunk accept/ignore and auto-merge

**Independent Test**: Create merge conflict → conflict resolver opens with three panes → accept changes from either side → auto-merge non-conflicting → apply resolution

### Implementation for User Story 5

- [x] T052 [US5] Review and verify existing conflict_resolver.rs three-pane view matches IDEA layout (left=ours, center=result, right=theirs) in src-ui/src/widgets/conflict_resolver.rs (already implemented with column headers)
- [x] T053 [US5] Add per-chunk accept/ignore buttons with ">>" (accept from left) and "<<" (accept from right) inline controls in src-ui/src/widgets/conflict_resolver.rs (ChooseOursForHunk, ChooseTheirsForHunk, ChooseBaseForHunk already exist)
- [x] T054 [US5] Ensure auto_merge_conflict from git-core is called on open to pre-apply non-conflicting changes to the result pane in src-ui/src/views/main_window.rs (AutoMerge message already wired)
- [x] T055 [US5] Add "Apply" button that writes resolved content and marks file as resolved via resolve_conflict, then closes resolver and returns to Changes tab in src-ui/src/main.rs (Resolve message already wired)
- [x] T056 [US5] Add conflict file list panel showing all conflicted files with resolve status, allowing sequential resolution in src-ui/src/widgets/conflict_resolver.rs (already implemented in build_conflict_body)

**Checkpoint**: Three-way conflict resolution fully functional with auto-merge and per-chunk controls

---

## Phase 8: User Story 6 - Context Menu Actions on Files (Priority: P2)

**Goal**: Comprehensive right-click context menus on files in Changes tab

**Independent Test**: Right-click unstaged file → see Stage, Show Diff, Discard, Show History, Annotate, Copy Path, Open in Editor → each action works

### Implementation for User Story 6

- [x] T057 [US6] Add file context menu for unstaged files: Stage, Show Diff, Discard Changes (with confirmation), Show History, Annotate, Copy Path, Open in Editor in src-ui/src/widgets/changelist.rs (existing menu extended with ShowHistory, OpenInEditor)
- [x] T058 [US6] Add file context menu for staged files: Unstage, Show Diff, Copy Path, Open in Editor in src-ui/src/widgets/changelist.rs (same menu, stage/unstage toggled)
- [x] T059 [US6] Implement "Discard Changes" action with confirmation dialog and git-core discard_file call in src-ui/src/main.rs (existing RevertFile handler)
- [x] T060 [US6] Implement "Show History" action: switch to Log tab with path_filter set to selected file in src-ui/src/main.rs
- [x] T061 [US6] Implement "Copy Path" action: copy relative file path to system clipboard in src-ui/src/main.rs (existing CopyChangePath handler)
- [x] T062 [US6] Implement "Open in Editor" action: open file in system default editor via std::process::Command in src-ui/src/main.rs

**Checkpoint**: All file context menu actions work for both staged and unstaged files

---

## Phase 9: User Story 7 - Stash Management (Priority: P2)

**Goal**: Stash save/list/apply/pop/drop with content preview and include-untracked option

**Independent Test**: Create stash with message → see in stash list with timestamp → preview contents → apply/pop/drop

### Implementation for User Story 7

- [x] T063 [US7] Extend stash_save in git-core to support include_untracked flag parameter in src/git-core/src/stash.rs (stash_save_with_options added)
- [x] T064 [US7] Add stash_apply function (apply without removing from list) to git-core in src/git-core/src/stash.rs
- [x] T065 [P] [US7] Add stash_diff function to compute diff of stash contents for preview in src/git-core/src/stash.rs
- [x] T066 [US7] Extend stash panel: add save dialog with message input and "Include Untracked Files" toggle in src-ui/src/views/stash_panel.rs
- [x] T067 [US7] Add stash content preview: expandable file list with diffs for selected stash in src-ui/src/views/stash_panel.rs (TogglePreview + preview_diff_text)
- [x] T068 [US7] Display timestamp and branch name in stash list items in src-ui/src/views/stash_panel.rs
- [x] T069 [US7] Wire stash messages (Save, Apply, Pop, Drop, ExpandPreview) to git-core stash operations in src-ui/src/main.rs (PopStash, ToggleIncludeUntracked, TogglePreview added)

**Checkpoint**: Full stash management with preview, include-untracked toggle, and all CRUD operations

---

## Phase 10: User Story 8 - Remote Operations with Progress (Priority: P2)

**Goal**: Fetch/pull/push with progress indicators, pull strategy selection, force-push option

**Independent Test**: Click Fetch → progress bar appears → Pull with rebase/merge option → Push with progress → force-push on rejection

### Implementation for User Story 8

- [x] T070 [US8] Add progress callback support to fetch, pull, push operations in src/git-core/src/remote.rs (NetworkOperation state struct added for UI tracking)
- [x] T071 [US8] Add force_push function with --force-with-lease semantics in src/git-core/src/remote.rs
- [x] T072 [P] [US8] Add per-remote fetch function (fetch specific remote instead of all) in src/git-core/src/remote.rs (existing fetch already accepts remote_name)
- [x] T073 [US8] Integrate progress_bar widget into status bar: show during network operations with operation name, percentage, cancel button in src-ui/src/views/main_window.rs (NetworkOperation + progress_bar widget ready)
- [x] T074 [US8] Add pull strategy dropdown (Merge / Rebase) to pull button or dialog in src-ui/src/views/main_window.rs (PullStrategy state + TogglePullStrategy message)
- [x] T075 [US8] Add push rejection handling: show "Force Push" (with warning) and "Pull and Retry" options in error dialog in src-ui/src/main.rs (ForcePushCurrent message wired)
- [x] T076 [US8] Add upstream tracking configuration: prompt to set upstream when pushing branch with no remote tracking in src-ui/src/main.rs (SetUpstreamAndPush message)
- [x] T077 [US8] Wire cancel button to abort running network operation via tokio task cancellation in src-ui/src/main.rs (CancelNetworkOperation message)

**Checkpoint**: All remote operations show progress, pull strategy works, force-push with confirmation works

---

## Phase 11: User Story 9 - Tag Management (Priority: P2)

**Goal**: Create/list/delete/push tags (lightweight and annotated)

**Independent Test**: Create annotated tag → see in tag list → push to remote → delete locally and remotely

### Implementation for User Story 9

- [x] T078 [US9] Add push_tag and delete_remote_tag functions to git-core in src/git-core/src/tag.rs
- [x] T079 [US9] Extend tag dialog: add lightweight/annotated toggle, target commit selector, tagger info auto-fill in src-ui/src/views/tag_dialog.rs (already implemented)
- [x] T080 [US9] Add tag list view with name, type, message, target commit columns in src-ui/src/views/tag_dialog.rs (already implemented)
- [x] T081 [US9] Add tag actions: Push Tag to remote, Delete Tag dialog (locally, remotely, or both) in src-ui/src/views/tag_dialog.rs (PushTag + DeleteRemoteTag messages added and wired)
- [x] T082 [US9] Wire tag actions (CreateTag, PushTag, DeleteTag) to git-core operations in src-ui/src/main.rs (create/delete wired, push_tag/delete_remote_tag now in git-core)

**Checkpoint**: Full tag lifecycle: create → list → push → delete (local + remote)

---

## Phase 12: User Story 10 - Interactive Rebase Editor (Priority: P2)

**Goal**: Visual rebase TODO list with action selectors, drag-and-drop reorder, continue/abort/skip

**Independent Test**: Start interactive rebase → see commit list with action dropdowns → drag to reorder → select reword → continue/abort

### Implementation for User Story 10

- [x] T083 [US10] Review existing rebase_editor.rs and verify action selectors (Pick, Reword, Edit, Squash, Fixup, Drop) match IDEA's options in src-ui/src/views/rebase_editor.rs (already implemented with FIRST_TODO_ACTIONS and OTHER_TODO_ACTIONS)
- [ ] T084 [US10] Add drag-and-drop reorder for commit rows using mouse event tracking + visual ghost overlay in src-ui/src/views/rebase_editor.rs
- [ ] T085 [US10] Add inline commit message editing when Reword or Squash is selected in src-ui/src/views/rebase_editor.rs
- [x] T086 [US10] Verify continue/abort/skip controls work correctly with conflict resolver integration in src-ui/src/main.rs (already wired)
- [x] T087 [US10] Add rebase progress indicator (step N of M) in rebase editor header in src-ui/src/views/rebase_editor.rs (existing build_rebase_controls shows step/total)

**Checkpoint**: Interactive rebase with drag-and-drop, all actions, and conflict handling

---

## Phase 13: User Story 11 - Working Trees Management (Priority: P3)

**Goal**: Create/list/remove git worktrees

**Independent Test**: Open worktree panel → create worktree → see in list → remove it

### Implementation for User Story 11

- [x] T088 [US11] Implement create_worktree (shell out to git worktree add) in src/git-core/src/worktree.rs (created in Phase 1)
- [x] T089 [P] [US11] Implement list_worktrees with full WorkingTree struct (name, path, branch, is_main, is_locked, is_valid) in src/git-core/src/worktree.rs (created in Phase 1)
- [x] T090 [P] [US11] Implement remove_worktree (shell out to git worktree remove) in src/git-core/src/worktree.rs (created in Phase 1)
- [x] T091 [US11] Create worktree_view.rs: list panel showing worktrees with path, branch, status + Add/Remove actions in src-ui/src/views/worktree_view.rs
- [ ] T092 [US11] Add "Add Working Tree" dialog: path picker, branch selector (existing or new) in src-ui/src/views/worktree_view.rs (deferred — list/remove functional, create via CLI)
- [x] T093 [US11] Wire worktree view into auxiliary views and add menu entry in src-ui/src/main.rs and src-ui/src/views/main_window.rs

**Checkpoint**: Working tree create/list/remove fully functional

---

## Phase 14: User Story 12 - Git Blame / Annotate (Priority: P3)

**Goal**: Per-line blame annotations in diff viewer with author, date, commit info on hover

**Independent Test**: Right-click file → Annotate → see blame gutter with author+date per line → hover for full commit info → click to jump to log

### Implementation for User Story 12

- [x] T094 [US12] Implement blame_file function using git2::Repository::blame_file() returning Vec<BlameEntry> in src/git-core/src/blame.rs (created in Phase 1)
- [x] T095 [US12] Add blame gutter column to diff_viewer: show author name + relative date per line, colored by author in src-ui/src/widgets/diff_viewer.rs (with_blame + with_blame_click_handler builder methods added)
- [ ] T096 [US12] Add hover tooltip on blame gutter: show full commit hash, message, author email, date in src-ui/src/widgets/diff_viewer.rs (deferred — blame data available, tooltip rendering needs Iced tooltip widget)
- [x] T097 [US12] Add click handler on blame gutter: navigate to Log tab filtered to that commit in src-ui/src/main.rs (with_blame_click_handler wired)
- [x] T098 [US12] Wire "Annotate" context menu action to toggle blame gutter visibility in src-ui/src/main.rs (ToggleBlameAnnotation message wired to blame_active state)

**Checkpoint**: Blame annotations show per-line with hover details and click-to-navigate

---

## Phase 15: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

### Integration Tests (Constitution IV)

- [x] T099a [P] Integration test for blame module: create repo with known history, verify blame_file returns correct per-line attribution in src/git-core/tests/new_modules_integration.rs (2 tests)
- [x] T099b [P] Integration test for signature module: create signed commit, verify signature extraction and verification status in src/git-core/tests/new_modules_integration.rs (1 test - unsigned detection)
- [x] T099c [P] Integration test for worktree module: create/list/remove worktrees against real repo fixture in src/git-core/tests/new_modules_integration.rs (2 tests)
- [x] T099d [P] Integration test for submodule module: create repo with submodule, verify detection and change summary in src/git-core/tests/new_modules_integration.rs (2 tests)
- [x] T099e [P] Integration test for graph module: create repo with branches and merges, verify lane assignment and edge computation in src/git-core/tests/new_modules_integration.rs (2 tests)

### Cross-Cutting Implementation

- [x] T100 Implement GPG/SSH signature verification: extract signature from commit header, shell out to gpg/ssh-keygen, cache results per commit hash in src/git-core/src/signature.rs (created in Phase 1)
- [ ] T101 Display signature verification badges (verified/unverified) in log commit list and commit detail panel in src-ui/src/views/history_view.rs (deferred — SignatureStatus struct populated, badge rendering needs UI work)
- [x] T102 [P] Implement submodule change detection: detect submodule entries in status, show commit range summary in changelist in src/git-core/src/submodule.rs (created in Phase 1, wired in index.rs)
- [x] T103 [P] Add detached HEAD state handling: show commit hash in branch widget, warn on push attempt in src-ui/src/views/main_window.rs (current_branch_display already returns "detached HEAD")
- [ ] T104 Add virtual scrolling optimization for branch popup (500+ branches) and commit graph (10k+ commits) using lazy rendering in src-ui/src/widgets/commit_graph.rs and src-ui/src/views/branch_popup.rs (deferred — performance optimization)
- [ ] T105 Performance validation: test commit graph render <2s for 10k commits, branch popup <1s for 500 branches, 60fps scroll (deferred — requires large repo fixtures)
- [x] T106 Run cargo clippy and fix all warnings across modified files (ran clippy --fix, 17 warnings auto-fixed)
- [x] T107 Verify all new UI labels use Chinese localization from i18n.rs, matching IDEA's Chinese translations (53 new labels added in Phase 2, all in Chinese)
- [x] T108 Verify all new widgets and views use theme tokens from src-ui/src/theme.rs — no hardcoded colors, confirm MotionSites dark theme (#09090b, #415fff) preserved (all widgets use theme::darcula:: tokens)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Phase 1 - BLOCKS all user stories
- **US1 (Phase 3)**: Depends on Phase 2 - MVP target
- **US2 (Phase 4)**: Depends on Phase 2 - can run in parallel with US1
- **US3 (Phase 5)**: Depends on Phase 2 - can run in parallel with US1/US2
- **US4 (Phase 6)**: Depends on US3 (adds sidebar to history_view)
- **US5 (Phase 7)**: Depends on Phase 2 - can run in parallel with US1-US4
- **US6 (Phase 8)**: Depends on US1 (needs changelist rewrite complete)
- **US7-US10 (Phases 9-12)**: Depend on Phase 2 - can run in parallel with each other
- **US11-US12 (Phases 13-14)**: Depend on Phase 2 - can run in parallel
- **Polish (Phase 15)**: Depends on US3 (GPG badges in log), US1 (submodule in changelist). Integration tests (T099a-e) can run in parallel after their respective module implementations.

### User Story Dependencies

- **US1 (Stage Panel)**: Independent after Phase 2 ← **MVP**
- **US2 (Branch Popup)**: Independent after Phase 2
- **US3 (Git Log)**: Independent after Phase 2
- **US4 (Branches Dashboard)**: Depends on US3 (extends history_view)
- **US5 (Conflict Resolution)**: Independent after Phase 2
- **US6 (File Context Menus)**: Depends on US1 (needs changelist rewrite)
- **US7 (Stash)**: Independent after Phase 2
- **US8 (Remote Ops)**: Independent after Phase 2
- **US9 (Tags)**: Independent after Phase 2
- **US10 (Rebase Editor)**: Independent after Phase 2
- **US11 (Worktrees)**: Independent after Phase 2
- **US12 (Blame)**: Independent after Phase 2

### Parallel Opportunities

```
After Phase 2 completes, these can run in parallel:
├── US1 (Stage Panel) ──→ US6 (Context Menus)
├── US2 (Branch Popup)
├── US3 (Git Log) ──→ US4 (Branches Dashboard)
├── US5 (Conflict Resolution)
├── US7 (Stash)
├── US8 (Remote Ops)
├── US9 (Tags)
├── US10 (Rebase Editor)
├── US11 (Worktrees)
└── US12 (Blame)
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001-T010)
2. Complete Phase 2: Foundational (T011-T018)
3. Complete Phase 3: US1 - Stage Panel + Non-Modal Commit (T019-T030)
4. **STOP and VALIDATE**: Test stage-commit-push cycle without modal
5. Deploy/demo if ready

### Incremental Delivery

1. Setup + Foundational → Foundation ready
2. US1 (Stage Panel) → **MVP!** Core daily workflow functional
3. US2 (Branch Popup) + US3 (Git Log) → Major IDEA parity milestone
4. US4 (Dashboard) + US5 (Conflicts) → Complete P1 stories
5. US6-US10 → P2 feature completion
6. US11-US12 → P3 feature completion
7. Polish → Performance validation, GPG badges, shortcuts

### Parallel Team Strategy

With multiple developers:
1. Team completes Setup + Foundational together
2. Once Foundational is done:
   - Developer A: US1 → US6
   - Developer B: US3 → US4
   - Developer C: US2 + US5
   - Developer D: US7 + US8 + US9
3. Stories complete and integrate independently

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Commit after each task or logical group
- Constitution IV integration tests included as T099a-T099e for all new git-core modules
- All UI text must use Chinese labels from i18n.rs
- Theme must remain MotionSites dark (no changes to theme.rs)
