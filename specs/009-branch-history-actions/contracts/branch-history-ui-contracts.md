# UI Contracts: 分支视图提交动作补齐

**Feature**: 分支视图提交动作补齐  
**Date**: 2026-03-25  
**Branch**: `009-branch-history-actions`

## Contract: Commit Rows Are First-Class in Branch View

```rust
/// Once a branch is selected, visible commits in the branch view MUST become
/// first-class interaction targets instead of passive text rows.
///
/// Required:
/// - one stable selected commit
/// - one discoverable action entry per visible commit
/// - graph/list rendering consistent with the history view
```

## Contract: Branch Navigator Rows Explain Themselves

```rust
/// Branch navigator rows MUST let the user distinguish current branch, selected
/// branch, and menu-open branch directly from the list row itself.
///
/// Required:
/// - current branch keeps one persistent but low-noise cue
/// - selected branch uses a stable selection treatment
/// - menu-open branch stays identifiable until the menu closes
/// - folder rows reflect expanded/collapsed state in the same compact rhythm
/// - row-end context buttons stay discoverable without bloating row height
```

## Contract: Grouped Action Menu Mirrors PhpStorm Intent

```rust
/// Commit actions MUST be organized in a stable, understandable grouping that
/// keeps high-frequency actions near the top and dangerous actions clearly later.
///
/// Required groups include:
/// - basic info / utility
/// - compare / navigate
/// - branch or tag derivation
/// - current-branch mutation
/// - local rewrite actions
```

## Contract: Branch And Commit Menus Share One Popup Language

```rust
/// Branch action menus and commit action menus MUST feel like members of the
/// same JetBrains-style popup family, even when their actions differ.
///
/// Required:
/// - compact header with the current target summary
/// - stable group order from common to dangerous
/// - Chinese helper text or disabled reasons for non-obvious actions
/// - consistent treatment of close affordance, separators, and danger groups
```

## Contract: Disabled Actions Must Explain Why

```rust
/// If an action is unavailable, the product MUST explain the reason in Chinese.
///
/// Forbidden:
/// - silent disappearance of important actions when context changes
/// - raw git errors as the first explanation for an obviously blocked action
```

## Contract: Dangerous Actions Require Scope Awareness

```rust
/// Reset, revert, push-to-here, and rewrite actions MUST explain their scope
/// before execution.
///
/// Required:
/// - what branch or remote is affected
/// - whether local unpublished commits are involved
/// - whether the action may enter a follow-up state
```

## Contract: Rewrite Flows Stay Inside a Guided Session

```rust
/// Reword, fixup, squash, drop, undo-last-commit, and rebase-from-here MUST be
/// modeled as guided sessions instead of fire-and-forget buttons.
///
/// Required:
/// - clear boundary of affected commits
/// - continue / skip / abort path when paused
/// - refresh timeline after completion
```

## Contract: Push-To-Here Is Current-Upstream Only

```rust
/// "Push to here" MUST stay anchored to the current branch and its configured
/// upstream.
///
/// Required:
/// - resolve one publication target from current branch context
/// - block or explain when no upstream exists
/// - explain non-fast-forward or unsafe publication clearly
```

## Contract: Menu Trigger Rows Stay Traceable

```rust
/// When a context menu opens from a branch row, commit row, or history row,
/// the originating row MUST remain visually traceable until the menu closes or
/// the context switches.
///
/// Required:
/// - opening a menu sets one visible menu-open state on the source row
/// - hover loss alone does not remove that state
/// - closing or retargeting the menu clears the previous source-row cue
```

## Contract: History View Commits Support Basic Right-Click Actions

```rust
/// Commit rows in the standalone history view MUST support a basic right-click
/// menu for frequent non-destructive actions.
///
/// Required:
/// - right-click on any visible commit row opens a menu instead of no-op
/// - copy hash, export patch, and view details are directly available
/// - action completion keeps the user in the history browsing context
```

## Contract: Context Continuity Is Preserved Across Browsing Surfaces

```rust
/// After non-destructive actions and most confirmations, the user MUST remain in
/// the current browsing surface with selection and scroll continuity preserved.
///
/// Discouraged:
/// - jumping back to a repository home screen
/// - clearing commit selection after every successful action
/// - forcing the user into another page to continue work
```

## Contract: Scrollbars Stay Secondary

```rust
/// Nested scrollable regions in the branch-view reading surfaces MUST keep
/// scrollbars visually secondary and structurally unambiguous.
///
/// Required:
/// - scrollbar tracks stay close to container edges
/// - scrollbars do not overlap core text content
/// - one reading region should not show stacked duplicate horizontal scrollbars
///
/// Preferred:
/// - handle long labels with truncation, wrapping, or decoration collapsing
///   before introducing another horizontal scrollbar
```

## Contract: New Files Preview as Content

```rust
/// When a newly added or untracked file is selected in a related reading
/// surface, the preview area MUST render meaningful content instead of falling
/// back to an empty "no changes" state.
///
/// Required:
/// - text files render their whole-file content as an added-file preview
/// - empty files show an explicit empty-file message
/// - binary or unsupported files show a clear Chinese placeholder
```
