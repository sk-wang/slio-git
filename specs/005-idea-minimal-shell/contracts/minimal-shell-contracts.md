# UI Contracts: IDEA 风格的极简 Git 工作台

**Feature**: IDEA 风格的极简 Git 工作台  
**Date**: 2026-03-23  
**Branch**: `005-idea-minimal-shell`

## Contract: Single Context Entry

```rust
/// After a repository is opened, the workspace MUST expose exactly one primary
/// context entry that tells the user:
/// - current repository
/// - current branch
/// - whether more branch/actions are available
///
/// The repository workspace MUST NOT persist a separate product-title banner,
/// tagline banner, and chip-row that duplicate the same context.
```

## Contract: Minimal Persistent Chrome

```rust
/// The main workspace chrome should remain sparse.
///
/// Persistent elements allowed:
/// - one context switcher
/// - one compact high-frequency action row (if needed)
/// - the changes tree and diff area
/// - optional compact status info
///
/// Persistent elements discouraged:
/// - product branding in the repository workspace
/// - repeated "current section" chips
/// - repeated "next action" hints
/// - large instructional copy while the user is already in an active repository
```

## Contract: Context Switcher / Branch Panel

```rust
/// The branch/actions popup MUST follow a gradual-disclosure pattern.
///
/// Required sections:
/// - current branch summary
/// - search input (when branch/action volume warrants it)
/// - high-frequency actions
/// - recent branches
/// - local branches
/// - remote branches
///
/// Behavior rules:
/// - opening the panel must not disrupt the main workspace state
/// - branch checkout and action execution remain reachable from the same panel
/// - low-frequency actions may appear deeper than high-frequency actions
```

## Contract: Main Workspace Focus

```rust
/// The dominant visual area of the repository workspace MUST remain dedicated to:
/// - the changes tree/list
/// - the diff preview
/// - conflict editing when applicable
///
/// Secondary UI chrome must not consume enough vertical or horizontal space to
/// demote the changes tree or diff preview from primary focus.
```

## Contract: Feedback Discipline

```rust
/// UI feedback must remain visible but restrained.
///
/// Rules:
/// - success/info feedback should default to ephemeral or compact presentation
/// - warning/error feedback may persist when action is required
/// - stable workspace state should not show explanatory banners by default
/// - user-facing messages remain Chinese
/// - structured logs remain mandatory even when the visual feedback is shortened
```

## Contract: Capability Reachability

```rust
/// Simplification must not remove capability.
///
/// The following capabilities MUST remain reachable:
/// - refresh
/// - stage / unstage / stage all / unstage all
/// - commit
/// - branch operations
/// - stash
/// - history
/// - remote actions
/// - tag actions
/// - conflict resolution
/// - rebase
///
/// Reachability may move from persistent toolbar placement to popup, menu, or
/// contextual entry, but must remain clear and testable.
```
