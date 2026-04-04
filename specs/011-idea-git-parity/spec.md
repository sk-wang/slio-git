# Feature Specification: IDEA Git Feature Parity

**Feature Branch**: `011-idea-git-parity`
**Created**: 2026-04-04
**Status**: Draft
**Input**: User description: "继续和~/git/idea对比 一比一还原idea的git功能 包括交互布局和操作git的能力 主题色不要改还是用现在的motionsite"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Git Stage Panel with Non-Modal Commit (Priority: P1)

Users need a staging area panel identical to IDEA's Git Stage, where staged and unstaged changes are shown as two separate collapsible tree groups within the same file list panel. The commit message editor is embedded below the diff preview (not in a modal dialog). Users can drag files between staged/unstaged groups, stage/unstage individual hunks from the diff, and commit directly without any popup interruption.

**Why this priority**: The staging panel is the most-used daily workflow surface. IDEA users expect a seamless stage-commit flow without modal dialogs. This is the core interaction that defines the tool.

**Independent Test**: Open a repository with modified files. Verify staged/unstaged tree groups render correctly, files can be moved between groups, hunks can be staged from diff, and commit executes inline without modal.

**Acceptance Scenarios**:

1. **Given** a repository with modified, added, and deleted files, **When** the user opens the Changes tab, **Then** files are grouped into "Staged" and "Unstaged Changes" collapsible tree sections with file status icons (added=green, modified=blue, deleted=red, renamed=cyan).
2. **Given** an unstaged file is selected, **When** the user clicks the "+" icon or drags it to the Staged group, **Then** the file moves to Staged and the diff updates.
3. **Given** a staged file's diff is shown, **When** the user clicks "Stage Hunk" on a specific hunk, **Then** only that hunk is staged while the rest remains unstaged.
4. **Given** staged files exist and a commit message is typed, **When** the user clicks the Commit button in the embedded panel, **Then** the commit is created without any modal dialog appearing, the staged list clears, and a success toast shows.
5. **Given** the "Amend" checkbox is checked, **When** the user commits, **Then** the previous commit is amended with the new changes and message.

---

### User Story 2 - Branch Management Popup with Tree Navigation (Priority: P1)

Users need a branch popup matching IDEA's GitBranchesPopup: a searchable, filterable popup with tree-structured branch groups (Local, Remote, Recent), and inline actions (checkout, merge, rebase, delete, rename, compare).

**Why this priority**: Branch switching and management is the second most frequent git operation. IDEA's branch popup is a signature interaction that users expect to work identically.

**Independent Test**: Click the branch widget in the toolbar. Verify the popup shows local/remote branches in tree form, search filters work, and all branch actions (checkout, merge, rebase, delete, rename) execute correctly.

**Acceptance Scenarios**:

1. **Given** a repository with multiple local and remote branches, **When** the user clicks the branch name in the toolbar, **Then** a popup appears with "Local Branches", "Remote Branches", and "Recent Branches" tree groups.
2. **Given** the branch popup is open, **When** the user types in the search field, **Then** branches are filtered in real-time across all groups.
3. **Given** a branch is selected in the popup, **When** the user right-clicks or hovers, **Then** a submenu shows: Checkout, New Branch from..., Merge into Current, Rebase Current onto..., Compare with Current, Rename, Delete.
4. **Given** the user selects "Checkout" on a remote branch, **Then** a local tracking branch is created and checked out.
5. **Given** the user selects "Delete" on a local branch, **Then** a confirmation dialog appears; if the branch is not fully merged, a warning is shown.

---

### User Story 3 - Git Log with Full Commit Graph (Priority: P1)

Users need a Log tab identical to IDEA's Git Log: a full-height commit history with graphical branch/merge visualization, commit details panel, search/filter bar, and branches dashboard sidebar. The log must support filtering by branch, author, date, path, and text.

**Why this priority**: The Git Log is the primary tool for understanding project history, reviewing changes, and performing history-based operations. IDEA's log is the industry benchmark for git history visualization.

**Independent Test**: Switch to the Log tab. Verify commit graph renders with branch lines, filtering works, commit details show diff, and context menu operations (cherry-pick, revert, create branch, reset) function correctly.

**Acceptance Scenarios**:

1. **Given** a repository with merge history, **When** the user switches to the Log tab, **Then** a full-height commit graph renders showing branch lines, merge points, and commit nodes with author, date, and message.
2. **Given** the log is displayed, **When** the user clicks a commit, **Then** the right panel shows commit details: hash, author, date, message, parent(s), and a list of changed files with diffs.
3. **Given** the log filter bar, **When** the user enters a search term, **Then** commits are filtered by message text, hash prefix, or author name.
4. **Given** the log filter bar, **When** the user selects a branch filter, **Then** only commits reachable from that branch are shown.
5. **Given** a commit is selected, **When** the user right-clicks, **Then** a context menu shows: Cherry-Pick, Revert, Create Branch, Create Tag, Reset Current Branch to Here, Copy Commit Hash.

---

### User Story 4 - Branches Dashboard in Log View (Priority: P1)

Users need a left sidebar in the Log tab (matching IDEA's "Branches" panel in Git Log) that shows all branches organized in a tree with grouping support. Selecting a branch in the dashboard filters the log to that branch's history.

**Why this priority**: The branches dashboard is IDEA's primary branch navigation surface within the log view, enabling quick branch switching and comparison without leaving the history context.

**Independent Test**: Open the Log tab. Verify the left sidebar shows branches in tree form, clicking a branch filters the log, and branch actions are available via context menu.

**Acceptance Scenarios**:

1. **Given** the Log tab is active, **When** the user opens the branches dashboard sidebar, **Then** all local and remote branches are displayed in a collapsible tree with group separators.
2. **Given** the branches dashboard is visible, **When** the user clicks a branch, **Then** the commit graph filters to show only commits reachable from that branch.
3. **Given** a branch in the dashboard, **When** the user right-clicks, **Then** actions include: Checkout, Merge, Rebase, Compare with Current, Delete, Rename.

---

### User Story 5 - Merge and Conflict Resolution (Priority: P1)

Users need a three-way merge conflict resolution view identical to IDEA's: left (ours), center (result), right (theirs) with apply/ignore buttons per change chunk, syntax highlighting, and automatic non-conflicting merge application.

**Why this priority**: Merge conflicts are the most stressful git interaction. A proper three-way merge tool is essential for developer productivity and is a key differentiator of professional git tools.

**Independent Test**: Create a merge conflict. Verify the conflict resolver opens with three panes, changes can be accepted from either side, the result updates live, and resolving all conflicts marks the file as resolved.

**Acceptance Scenarios**:

1. **Given** a merge with conflicts, **When** the user clicks "Resolve" on a conflicted file, **Then** a three-pane view opens: left (ours), center (merged result), right (theirs).
2. **Given** the conflict resolver is open, **When** the user clicks "Accept" on a left-side change, **Then** that change is applied to the center result pane.
3. **Given** non-conflicting changes exist, **When** the conflict resolver opens, **Then** non-conflicting changes from both sides are auto-applied to the result.
4. **Given** all conflicts are resolved, **When** the user clicks "Apply", **Then** the file is marked resolved and the conflict resolver closes.

---

### User Story 6 - Context Menu Actions on Files (Priority: P2)

Users need comprehensive right-click context menus on files in the Changes tab, matching IDEA's Git file actions: Stage/Unstage, Show Diff, Discard Changes (Rollback), Show History for File, Annotate (Blame), Copy Path, Open in Editor.

**Why this priority**: Context menus are the secondary interaction surface for file operations, providing quick access to less frequent but important actions without toolbar clutter.

**Independent Test**: Right-click a file in the changes list. Verify all expected actions appear and execute correctly.

**Acceptance Scenarios**:

1. **Given** an unstaged file, **When** the user right-clicks it, **Then** the context menu shows: Stage, Show Diff, Discard Changes, Show History, Annotate, Copy Path, Open in Editor.
2. **Given** a staged file, **When** the user right-clicks it, **Then** the context menu shows: Unstage, Show Diff, Copy Path, Open in Editor.
3. **Given** the user selects "Discard Changes", **Then** a confirmation dialog appears; on confirm, the file reverts to the last committed state.
4. **Given** the user selects "Show History", **Then** the Log tab opens filtered to commits that modified this file.

---

### User Story 7 - Stash Management (Priority: P2)

Users need stash operations matching IDEA's Git Stash: Save stash with message, list stashes, apply/pop stash, drop stash, view stash contents (changed files and diffs), and stash including untracked files option.

**Why this priority**: Stash is a frequently used workflow for temporarily shelving work. Full stash management with preview support matches IDEA's capability.

**Independent Test**: Create a stash, verify it appears in the stash list, preview its contents, and apply/pop/drop it.

**Acceptance Scenarios**:

1. **Given** uncommitted changes exist, **When** the user clicks "Stash", **Then** a dialog allows entering a stash message and toggling "Include Untracked Files"; on confirm, changes are stashed and working tree is clean.
2. **Given** stashes exist, **When** the user opens the stash panel, **Then** all stashes are listed with index, message, branch name, and timestamp.
3. **Given** a stash is selected, **When** the user clicks "Apply" or "Pop", **Then** the stash is applied to the working tree; "Pop" also removes it from the stash list.
4. **Given** a stash is selected, **When** the user expands it, **Then** the changed files and their diffs are shown as a preview.

---

### User Story 8 - Remote Operations with Progress (Priority: P2)

Users need remote operations matching IDEA's: Fetch (all remotes / specific remote), Pull (with rebase/merge option), Push (with force push option and upstream tracking), and visible progress indicators for all network operations.

**Why this priority**: Remote synchronization is essential for collaborative work. IDEA provides clear progress feedback and configuration options for all remote operations.

**Independent Test**: Perform fetch, pull, and push operations. Verify progress indicators show, options (rebase vs merge for pull, force push) work, and errors are clearly reported.

**Acceptance Scenarios**:

1. **Given** the user clicks "Fetch", **Then** all remotes are fetched with a progress indicator showing remote name and transfer status.
2. **Given** the user clicks "Pull", **Then** a dropdown or dialog allows choosing "Merge" or "Rebase" strategy; the operation executes with progress indication.
3. **Given** the user clicks "Push", **Then** commits are pushed to the tracking remote; if no upstream is set, the user is prompted to set one.
4. **Given** a push is rejected (non-fast-forward), **When** the error appears, **Then** the user sees options: "Force Push" (with warning) or "Pull and Retry".
5. **Given** any network operation is running, **Then** a progress bar with cancel button is visible in the status bar.

---

### User Story 9 - Tag Management (Priority: P2)

Users need tag operations matching IDEA's: Create tag (lightweight and annotated) on HEAD or specific commit, list tags, delete tags, push tags to remote.

**Why this priority**: Tag management is used for release workflows. Full IDEA parity requires create, list, delete, and push capabilities.

**Independent Test**: Create a tag, verify it appears in the tag list, push it to remote, and delete it.

**Acceptance Scenarios**:

1. **Given** the user selects "Create Tag", **Then** a dialog allows entering tag name, choosing lightweight or annotated, entering a message (for annotated), and selecting target commit.
2. **Given** tags exist, **When** the user opens the tag list, **Then** all tags are shown with name, type, message (if annotated), and target commit.
3. **Given** a tag is selected, **When** the user selects "Push Tag", **Then** the tag is pushed to the default remote.
4. **Given** a tag is selected, **When** the user selects "Delete Tag", **Then** a dialog offers to delete locally, remotely, or both.

---

### User Story 10 - Interactive Rebase Editor (Priority: P2)

Users need an interactive rebase editor matching IDEA's: visual TODO list with pick/reword/edit/squash/fixup/drop actions, drag-and-drop reorder, inline commit message editing for reword/squash, and continue/abort/skip controls.

**Why this priority**: Interactive rebase is an advanced but critical workflow for maintaining clean commit history. IDEA's visual editor makes this complex operation accessible.

**Independent Test**: Start an interactive rebase. Verify the TODO list renders with action selectors, drag-and-drop reorder works, and continue/abort/skip complete the rebase correctly.

**Acceptance Scenarios**:

1. **Given** the user initiates "Interactive Rebase", **Then** a rebase editor opens showing commits with action selectors (Pick, Reword, Edit, Squash, Fixup, Drop).
2. **Given** the rebase editor is open, **When** the user drags a commit row, **Then** the commit order updates.
3. **Given** "Reword" is selected for a commit, **When** the rebase proceeds, **Then** the user can edit that commit's message inline.
4. **Given** the rebase encounters a conflict, **Then** the conflict resolver opens; after resolution, "Continue Rebase" resumes the operation.
5. **Given** a rebase is in progress, **When** the user clicks "Abort", **Then** the rebase is aborted and the branch returns to its pre-rebase state.

---

### User Story 11 - Working Trees Management (Priority: P3)

Users need working tree management matching IDEA's Git Working Trees: create new worktree, list worktrees, remove worktree, and open worktree in a new window.

**Why this priority**: Working trees is an advanced feature used by power users for parallel development. Lower priority as it is less frequently used.

**Independent Test**: Create a worktree, verify it appears in the list, and remove it.

**Acceptance Scenarios**:

1. **Given** the user selects "Manage Working Trees", **Then** a panel shows existing worktrees with path, branch, and status.
2. **Given** the user clicks "Add Working Tree", **Then** a dialog allows specifying path and branch (existing or new).
3. **Given** a worktree is selected, **When** the user clicks "Remove", **Then** the worktree is removed after confirmation.

---

### User Story 12 - Git Blame / Annotate (Priority: P3)

Users need inline blame annotations matching IDEA's Annotate feature: show author, date, and commit hash next to each line in the diff viewer, with hover details showing full commit info.

**Why this priority**: Blame/annotate is an investigative tool used occasionally but is part of complete IDEA git parity.

**Independent Test**: Right-click a file and select "Annotate". Verify blame annotations appear next to each line with author and date info.

**Acceptance Scenarios**:

1. **Given** a file is shown in the diff viewer, **When** the user enables "Annotate", **Then** each line shows the author name and commit date in a gutter column.
2. **Given** annotations are visible, **When** the user hovers over an annotation, **Then** a tooltip shows the full commit hash, message, author email, and date.
3. **Given** annotations are visible, **When** the user clicks an annotation, **Then** the Log view opens filtered to that commit.

---

### Edge Cases

- What happens when the repository is in a detached HEAD state? The branch widget should show the commit hash instead of a branch name, and operations that require a branch (e.g., push) should warn the user.
- How does the system handle a repository with thousands of branches? The branch popup must use virtual scrolling and lazy loading to remain responsive.
- What happens during a long-running network operation if the user closes the app? Operations should be cancellable, and the status bar should indicate when an operation is still running.
- How does the system handle corrupted or missing git objects? Errors should be displayed clearly with guidance on recovery options.
- What happens when staging a file that has been externally modified after the change list was loaded? The system should detect the change and refresh the file status before staging.
- How does the system handle submodules? Submodule changes should appear as a special entry in the change list with summary of submodule diff (commit range).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST display file changes in two collapsible tree groups: "Staged" and "Unstaged Changes", matching IDEA's Git Stage panel layout. Within each group, files MUST be viewable in two modes: flat list (filenames with relative path) and directory tree (grouped by folder hierarchy), togglable via a toolbar button.
- **FR-002**: System MUST support drag-and-drop of files between Staged and Unstaged groups.
- **FR-003**: System MUST support hunk-level staging and unstaging from the diff viewer via "Stage Hunk" / "Unstage Hunk" buttons.
- **FR-004**: System MUST embed the commit message editor below the diff preview within the Changes tab (no modal commit dialog).
- **FR-005**: System MUST provide an "Amend" option that prepopulates the last commit message and amends on commit. The commit message editor MUST include a recent message history dropdown showing the last 10 commit messages for quick reuse.
- **FR-006**: System MUST render a branch popup with tree-structured groups: Local Branches, Remote Branches, Recent Branches, with real-time search filtering.
- **FR-007**: System MUST provide branch actions via context menu: Checkout, New Branch From, Merge Into Current, Rebase Current Onto, Compare With Current, Rename, Delete.
- **FR-008**: System MUST render a commit graph in the Log tab showing branch lines, merge points, and commit nodes with author, date, and message. The Log view MUST support a multi-tab design: a default "All" tab showing all branches, plus user-created tabs that pin a specific branch filter. Users can open multiple tabs to view different branch histories simultaneously.
- **FR-009**: System MUST support log filtering by branch, author, date range, file path, and text search.
- **FR-010**: System MUST provide a branches dashboard sidebar in the Log tab for branch-based log filtering.
- **FR-011**: System MUST provide a three-pane conflict resolver (ours / result / theirs) with per-chunk accept/ignore actions and auto-application of non-conflicting changes.
- **FR-012**: System MUST provide comprehensive right-click context menus on files: Stage/Unstage, Show Diff, Discard Changes (Rollback), Show History, Annotate, Copy Path, Open in Editor.
- **FR-013**: System MUST support stash operations: save (with message, include untracked option), list, apply, pop, drop, and stash content preview.
- **FR-014**: System MUST show progress indicators with cancel support for all network operations (fetch, pull, push).
- **FR-015**: System MUST support pull with merge/rebase strategy selection.
- **FR-016**: System MUST support push with force-push option and upstream tracking configuration.
- **FR-017**: System MUST support tag creation (lightweight and annotated), listing, deletion (local and remote), and pushing.
- **FR-018**: System MUST provide an interactive rebase editor with action selectors (pick/reword/edit/squash/fixup/drop), drag-and-drop reorder, and continue/abort/skip controls.
- **FR-019**: System MUST display commit details (hash, author, date, message, parents, changed files with diffs) when a commit is selected in the Log. Commits with GPG/SSH signatures MUST display a verification badge (verified/unverified) in the log list and commit detail panel.
- **FR-020**: System MUST provide context menu actions on log commits: Cherry-Pick, Revert, Create Branch, Create Tag, Reset Current Branch, Copy Hash.
- **FR-021**: System MUST handle detached HEAD state by showing commit hash in the branch widget and warning on push.
- **FR-022**: System MUST detect and display submodule changes as special entries in the change list.
- **FR-023**: System MUST support Git Blame/Annotate view with per-line author, date, and commit information.
- **FR-024**: System MUST support working tree management: create, list, remove worktrees.
- **FR-025**: System MUST preserve the existing MotionSites dark theme (background #09090b, accent #415fff) across all new UI components.

### Key Entities

- **Change**: A file-level modification with status (added, modified, deleted, renamed, copied), staging state (staged/unstaged), and associated diff hunks.
- **Branch**: A git reference with name, type (local/remote), tracking relationship, ahead/behind count, and last commit metadata.
- **Commit**: A point in history with hash, author, committer, date, message, parent references, and associated file changes.
- **Stash**: A saved work-in-progress state with index, message, source branch, timestamp, and contained changes.
- **Tag**: A named reference to a commit, either lightweight (pointer only) or annotated (with message, tagger, date).
- **Conflict**: A merge conflict state for a file with ours/base/theirs content, resolution status, and applied chunks.
- **Working Tree**: A linked worktree with path, associated branch, and status (clean/dirty).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can complete the stage-commit-push cycle without any modal dialog interruption, in under 30 seconds for a typical 1-5 file change.
- **SC-002**: Branch popup opens and renders 500+ branches in under 1 second, with search filtering responding within 100ms of keystroke.
- **SC-003**: Commit graph for repositories with 10,000+ commits renders the visible viewport in under 2 seconds, with smooth scrolling at 60fps.
- **SC-004**: Three-way conflict resolution enables users to resolve a typical 3-conflict file in under 2 minutes.
- **SC-005**: All 12 core IDEA Git operations (stage, unstage, commit, amend, branch, merge, rebase, cherry-pick, stash, tag, fetch, push) are accessible within 2 clicks or 1 keyboard shortcut from the main view.
- **SC-006**: Network operations (fetch, pull, push) show real-time progress within 500ms of starting and support user-initiated cancellation.
- **SC-007**: Log filtering by any single criterion (branch, author, date, path, text) returns results in under 1 second for repositories with 10,000+ commits.
- **SC-008**: Feature parity coverage: 95%+ of IDEA's Git tool window actions are available in slio-git (measured by action inventory comparison).

## Clarifications

### Session 2026-04-04

- Q: Should the change list support flat list, directory tree, or both display modes? → A: Both flat list and directory tree modes with a toggle button (module grouping excluded).
- Q: Should the Log view use multi-tab or single-tab with filter? → A: Multi-tab: "All" tab + user-created branch-pinned tabs (full IDEA parity).
- Q: Should slio-git implement IDEA's Shelf feature in addition to Git Stash? → A: Git Stash only; Shelf is IDE-specific and out of scope for a git client.
- Q: Should GPG/SSH commit signing UI be included? → A: Display-only: show verification badges in log and commit details; no signing toggle in commit panel.
- Q: Should the commit panel support message history and/or templates? → A: Recent message history dropdown only (last 10 messages); commit templates out of scope.

## Assumptions

- The existing MotionSites dark theme colors and design tokens will be reused without modification.
- The current git-core library already implements most required git operations; this spec focuses on UI parity and interaction design.
- The application targets single-repository workflows (multi-repository support like IDEA's is out of scope for this iteration).
- Keyboard shortcuts will follow IDEA's default keymap where applicable, adapted for macOS conventions.
- The existing Iced 0.14 framework can support all required UI patterns (tree views, drag-and-drop, context menus, three-pane splits).
- Chinese (Simplified) remains the primary UI language, with labels matching IDEA's Chinese localization where possible.
