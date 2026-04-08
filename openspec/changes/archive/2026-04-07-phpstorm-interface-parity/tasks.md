## 1. Rebuild the workbench shell

- [x] 1.1 Define shared PhpStorm parity tokens for spacing, separators, hover/selected states, tabs, and scrollbars in `theme.rs` and related widget helpers
- [x] 1.2 Refactor `main_window.rs` to present a stable JetBrains-style shell with compact top chrome, editor-like tabs, central changes/diff content, and a docked bottom tool-window region
- [x] 1.3 Align `state.rs` shell metadata so branch/history/log surfaces preserve current repository, branch, and selection context across view switches

## 2. Unify popup and menu interactions

- [x] 2.1 Create shared dense list/menu primitives for grouped actions, submenu indicators, disabled reasons, and trigger-row highlighting
- [x] 2.2 Rework `branch_popup.rs` to use the shared primitives for search-first branch switching and action grouping that matches the PhpStorm cadence
- [x] 2.3 Apply the same interaction language to commit, history, and remote action menus so branch/commit surfaces behave consistently

## 3. Match the core workspace surfaces

- [x] 3.1 Tighten the changes list, diff viewer, file headers, status bar, and low-noise scrollbars to the new parity profile
- [x] 3.2 Turn the history/log area into a docked bottom tool window that preserves focus and feels continuous with the main workbench
- [x] 3.3 Verify the submit, stage/unstage, refresh, compare, pull/push, and branch-switch flows remain reachable inside the new shell

## 4. Validate parity and regressions

- [x] 4.1 Add a PhpStorm parity checklist covering default layout, branch popup, context menus, bottom log window, row density, and focus states
- [x] 4.2 Capture updated screenshots or equivalent review evidence and compare them against the provided reference image
- [x] 4.3 Run targeted regression checks for repository open, change review, commit, branch switching, and history navigation before merging
