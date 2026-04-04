# UI Contracts: PhpStorm 风格的轻量化样式收敛

**Feature**: PhpStorm 风格的轻量化样式收敛  
**Date**: 2026-03-23  
**Branch**: `006-phpstorm-style-polish`

## Contract: Continuous Workspace Chrome

```rust
/// After a repository is opened, the workspace MUST feel like one continuous
/// working surface.
///
/// Required:
/// - no more than two obvious persistent top bars
/// - top chrome remains thinner than the main content area
/// - changes tree and diff viewer visually dominate the first screen
///
/// Discouraged:
/// - stacked raised cards
/// - heavy rounded containers around every subsection
/// - repeated summary chips that restate the same repository/branch context
```

## Contract: Single Context Strip

```rust
/// The repository workspace MUST expose exactly one primary context strip for:
/// - current repository
/// - current branch
/// - lightweight sync/state hints
///
/// The repository workspace MUST NOT show a separate in-workspace product title
/// like "slio-git" as a primary visual element once a repository is open.
```

## Contract: JetBrains-Style Branch Popup

```rust
/// The branch popup MUST behave like a compact JetBrains-style list popup.
///
/// Required sections:
/// - current branch summary
/// - search input
/// - compact high-frequency actions
/// - recent branches
/// - local branches
/// - remote branches
///
/// Rules:
/// - list rhythm must be denser than the main workspace cards it replaces
/// - item metadata must stay secondary to branch names
/// - explanatory copy must not displace branch/action list area
```

## Contract: Compact List and Badge Language

```rust
/// Changes list rows, group headers, badges, counters, and action buttons MUST
/// use a restrained visual language.
///
/// Rules:
/// - lower control height and padding
/// - flatter badges or text-first indicators for normal states
/// - selected states remain obvious without thick borders
/// - warning/error states may use stronger emphasis than normal states
```

## Contract: Lightweight Status Surfaces

```rust
/// Status, success, warning, and error feedback must remain understandable but
/// visually disciplined.
///
/// Rules:
/// - stable workspace state should not render large instructional banners
/// - success/info feedback should prefer compact or ephemeral presentation
/// - error/conflict states may persist and use stronger contrast
/// - all user-facing feedback remains Chinese
```

## Contract: Capability Reachability

```rust
/// Style polish MUST NOT reduce Git capability reachability.
///
/// The following remain clearly reachable:
/// - refresh
/// - stage / unstage / stage all / unstage all
/// - commit
/// - branch operations
/// - pull / push
/// - stash
/// - history
/// - tag actions
/// - remote actions
/// - conflict resolution
/// - rebase
///
/// Reachability may move between top chrome, popup, and contextual views, but
/// users must still find these paths during regression walkthroughs.
```
