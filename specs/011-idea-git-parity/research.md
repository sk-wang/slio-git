# Research: IDEA Git Feature Parity

**Branch**: `011-idea-git-parity` | **Date**: 2026-04-04

## R1: Commit Graph Visualization Algorithm

**Decision**: Use lane-based graph layout algorithm with virtual scrolling

**Rationale**: IDEA uses a lane-assignment algorithm where each branch gets a "lane" (column). Commits are assigned to lanes based on parent-child relationships. This approach:
- Produces clean, non-overlapping branch lines matching IDEA's visual style
- Supports incremental computation (only compute visible viewport + buffer)
- Compatible with virtual scrolling for 10k+ commit repositories

**Algorithm outline**:
1. Walk commits in topological order
2. Assign each branch tip to the leftmost available lane
3. Merge points consume the child's lane; parent continues in its lane
4. Track active lanes at each row for edge rendering
5. Cache computed layout for scroll performance

**Alternatives considered**:
- Full DAG layout (too expensive for large repos, >5s for 10k commits)
- gitk-style text graph (not suitable for graphical rendering in Iced)
- git2 revwalk with manual graph tracking (chosen as basis, extended with lane assignment)

## R2: Git Blame via git2-rs

**Decision**: Use `git2::Repository::blame_file()` API from git2-rs

**Rationale**: git2-rs exposes libgit2's blame API which provides per-line attribution with commit hash, author, date. This matches IDEA's annotate feature exactly. Key considerations:
- `BlameOptions` supports `newest_commit` and `oldest_commit` for range limiting
- Returns `BlameHunk` with `orig_commit_id`, `final_commit_id`, `orig_start_line`, `final_start_line`, `lines_in_hunk`
- Performance: libgit2 blame is O(n*m) where n=lines, m=commits touching file; acceptable for typical files (<5000 lines)

**Alternatives considered**:
- Shelling out to `git blame` (adds process overhead, harder to parse)
- Custom blame implementation (unnecessary, libgit2 coverage is sufficient)

## R3: GPG/SSH Signature Verification

**Decision**: Use git2-rs commit signature extraction + shell out to `gpg`/`ssh-keygen` for verification

**Rationale**: git2-rs provides `Commit::header_field_bytes("gpgsig")` to extract signatures, but does not include verification logic. For display-only verification badges:
- Extract signature from commit header
- Detect signature type (GPG vs SSH) by prefix
- Shell out to `gpg --verify` or `ssh-keygen -Y verify` for actual verification
- Cache verification results per commit hash (immutable)

**Alternatives considered**:
- Pure Rust GPG implementation via `sequoia-openpgp` (heavy dependency, 2MB+ binary increase)
- Skip verification, show "signed" indicator only (insufficient for IDEA parity which shows verified/unverified)

## R4: Working Tree Management via git2-rs

**Decision**: Use git2-rs worktree API where available, shell out for gaps

**Rationale**: git2-rs 0.19 provides:
- `Repository::worktrees()` - list worktree names
- `Repository::find_worktree(name)` - find worktree by name
- `Worktree::path()` - get worktree path
- `Worktree::validate()` - check if worktree is valid

Missing from git2-rs (require shell command):
- `git worktree add <path> <branch>` - create worktree
- `git worktree remove <path>` - remove worktree

**Alternatives considered**:
- Pure git2-rs only (insufficient API coverage for create/remove)
- Pure shell commands only (loses consistency with rest of git-core)

## R5: Submodule Detection

**Decision**: Use `git2::Repository::submodules()` API

**Rationale**: git2-rs provides full submodule enumeration:
- `Repository::submodules()` returns `Vec<Submodule>`
- `Submodule::name()`, `Submodule::path()`, `Submodule::url()`
- `Submodule::head_id()` for current commit, `Submodule::index_id()` for staged
- Detect submodule changes by comparing `head_id` vs `index_id` vs `workdir_id`

**Alternatives considered**:
- Parse `.gitmodules` manually (fragile, misses runtime state)
- Ignore submodules (violates FR-022)

## R6: Iced Virtual Scrolling for Large Lists

**Decision**: Use Iced's `lazy` widget + manual viewport tracking for virtual scrolling

**Rationale**: Iced 0.14 provides:
- `iced::widget::lazy` for deferred rendering
- `Scrollable` with `on_scroll` callback for viewport position tracking
- Combined: render only visible rows + buffer, swap content on scroll

This is critical for:
- Branch popup with 500+ branches (SC-002: <1s render)
- Commit graph with 10k+ commits (SC-003: <2s viewport, 60fps scroll)
- Change list with 1000+ files

**Alternatives considered**:
- Render all items (performance bottleneck at scale)
- Custom Canvas-based rendering (too low-level, loses Iced widget interactivity)

## R7: Drag-and-Drop in Iced

**Decision**: Implement drag-and-drop using Iced mouse events + visual overlay

**Rationale**: Iced 0.14 does not have built-in drag-and-drop support. Implementation approach:
1. Track `mouse::Event::ButtonPressed` on draggable items
2. On mouse move with button held, show a ghost overlay at cursor position
3. Track drop zones (Staged/Unstaged groups) via mouse position + layout bounds
4. On `ButtonReleased`, determine target zone and emit appropriate message

Used for: file staging/unstaging (FR-002), rebase editor reorder (FR-018)

**Alternatives considered**:
- Buttons-only staging (simpler but doesn't match IDEA's drag-and-drop interaction)
- External DnD library (none exist for Iced)

## R8: Context Menu Implementation in Iced

**Decision**: Extend existing `menu.rs` widget with positional popup support

**Rationale**: The project already has a `menu.rs` widget (12KB). It needs to be extended to:
- Open at right-click position (not fixed toolbar position)
- Support submenus (for branch actions like "Merge into Current > [branch list]")
- Support separator items between groups
- Auto-dismiss on click outside or Escape key
- Conditionally enable/disable items based on state

**Alternatives considered**:
- New menu widget from scratch (wasteful, existing widget covers 70% of needs)
- Native OS context menus via `iced::window` (Iced doesn't support this)

## R9: Commit Message History Persistence

**Decision**: Store recent commit messages in `~/.config/slio-git/commit-messages.json`

**Rationale**: Simple JSON array of last 10 commit messages per repository path. Format:
```json
{
  "/path/to/repo": ["msg1", "msg2", ...],
  ...
}
```
- Written after each successful commit
- Read on commit panel initialization
- Maximum 10 messages per repo, FIFO eviction
- File size negligible (<10KB for typical usage)

**Alternatives considered**:
- Read from git reflog (only captures commit hashes, not messages directly; requires extra git operations)
- SQLite database (overkill for 10 strings per repo)

## R10: Multi-Tab Log Design

**Decision**: Tab bar widget above log content area, each tab owns its own filter state and scroll position

**Rationale**: IDEA's log uses tabs where:
- "All" tab is permanent (cannot be closed)
- User creates branch-pinned tabs via "Open in New Tab" action
- Each tab maintains independent filter state, scroll position, and selected commit
- Tabs are closable (except "All") via close button

Implementation: `LogTab` struct holding filter config + scroll state + selected commit. Tab bar widget renders tab list with close buttons and "+" for new tab.

**Alternatives considered**:
- Single view with filter stack/breadcrumbs (doesn't match IDEA's tab UX)
- Browser-style tab bar via external crate (none suitable for Iced)
