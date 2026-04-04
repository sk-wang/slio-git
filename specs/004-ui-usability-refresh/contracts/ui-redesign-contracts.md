# UI Redesign Contracts: 主界面可用性与视觉改造

**Feature**: 主界面可用性与视觉改造  
**Date**: 2026-03-22  
**Branch**: `004-ui-usability-refresh`

## Contract: Global App Shell

```rust
/// Global shell requirements for all existing UI screens
///
/// Required outcomes:
/// - Every existing screen in src-ui/views/ must render inside one consistent Darcula shell
/// - The shell may redesign navigation, entry labels, and flow grouping
/// - Chinese labels and CJK-capable fonts remain mandatory
/// - A user must always understand: current state, next action, and current feedback
///
/// Shell responsibilities:
/// - Global navigation or entry grouping
/// - Shared header / title / action placement rules
/// - Shared loading / empty / error / success surfaces
/// - Shared status presentation for repository context
```

## Contract: Navigation & Reachability

```rust
/// Navigation may be fully redesigned, but all existing capabilities must remain reachable.
///
/// Reachability rules:
/// - Open repository and initialize repository remain primary entry points from Welcome
/// - Core repository workspace remains reachable immediately after repository selection
/// - Existing views/dialogs/panels remain discoverable through the new navigation structure:
///   - Commit
///   - Branch
///   - Stash
///   - History
///   - Remote
///   - Tag
///   - Rebase
///   - Conflict Resolver
///
/// Usability rules:
/// - Primary actions must be visually distinct from secondary actions
/// - Disabled actions must communicate why they are unavailable
/// - Navigation labels may change, but meaning must remain clear in Chinese
```

## Contract: Visual Language

```rust
/// Darcula visual system contract
///
/// Theme rules:
/// - Use one shared Darcula token family across all existing UI screens
/// - Keep typography, spacing, borders, selection states, and feedback states consistent
/// - Avoid mixed themes or ad-hoc per-screen overrides
///
/// Required states:
/// - Default
/// - Hover / focus (where applicable in Iced)
/// - Selected
/// - Disabled
/// - Loading
/// - Empty
/// - Error / warning
/// - Success
```

## Contract: Feedback & Async State

```rust
/// Every user-triggered operation must surface one of the supported feedback states.
///
/// Supported feedback kinds:
/// - Loading: operation accepted and in progress
/// - Success: operation completed
/// - Error: operation failed with actionable summary
/// - Empty: there is no current content to show
/// - Warning: non-blocking issue or risk
///
/// Rules:
/// - No large unexplained blank regions
/// - No silent failures for existing actions
/// - Loading state must appear before long-running work completes
/// - Error text must be user-facing Chinese, not raw git internals
```

## Contract: Screen Modernization Scope

```rust
/// Scope covers ALL current UI surfaces in src-ui:
/// - main_window
/// - commit_dialog
/// - branch_popup
/// - stash_panel
/// - history_view
/// - remote_dialog
/// - tag_dialog
/// - rebase_editor
/// - conflict_resolver
/// - reusable widgets used by these screens
///
/// Modernization rules:
/// - Each screen must adopt the shared shell conventions
/// - Each screen must retain or improve access to its pre-existing functional behavior
/// - Screen-level refactors may change layout and entry path, but not remove capability
```

## Contract: Defect Sweep

```rust
/// Defect sweep is part of this feature, not a side task.
///
/// Rules:
/// - Any discovered repository defect must be logged to the feature task stream
/// - Fixed defects must have a reproducible verification path
/// - UI refactors may not close visually while leaving the discovered behavior broken
/// - Regression checks must cover both touched UI surfaces and touched git-core behavior
```
