## ADDED Requirements

### Requirement: Workbench shell SHALL use PhpStorm-like compact density
The repository workbench SHALL render the default shell with compact spacing, control heights, tab heights, and tool-window headers that are visibly denser than the current baseline while preserving clear hover, selected, and disabled states.

#### Scenario: Main shell opens in compact mode
- **WHEN** a user opens a repository into the default changes/diff workspace
- **THEN** the top chrome, editor-style tabs, central work area, and bottom tool window SHALL use the compact density profile
- **THEN** the active repository and branch context SHALL remain visible without expanding the shell height beyond the compact baseline

### Requirement: High-frequency panels SHALL reduce padding and redundant chrome
The changes list, diff file header, history panel, and commit dialog SHALL remove redundant vertical padding, oversized section headers, and multi-line explanatory chrome so that more working information fits in the same viewport.

#### Scenario: Commit dialog stays information-dense
- **WHEN** a user opens the commit dialog with staged files present
- **THEN** the file list, diff preview, message editor, and action row SHALL fit into a denser two-pane layout with compact status and toolbar treatments
- **THEN** the user SHALL still be able to distinguish staged counts, preview state, validation status, and primary commit actions at a glance

### Requirement: Menus and popups SHALL share compact interaction metrics
Branch switching, remote actions, and related contextual menus SHALL use a shared compact row height, grouping rhythm, and submenu/disabled affordances so interaction surfaces feel like one PhpStorm-style system.

#### Scenario: Branch popup and action menus match the same density profile
- **WHEN** a user opens the branch popup or a related repository action menu
- **THEN** each menu SHALL use the shared compact metrics for rows, separators, chips, and hover states
- **THEN** dangerous and disabled actions SHALL remain visually distinguishable without reintroducing oversized card-like spacing

### Requirement: Compact parity SHALL be validated with explicit review evidence
The project SHALL maintain a compact-density checklist or equivalent review evidence that compares the updated workbench against the PhpStorm reference for shell density, list density, dialog density, and bottom tool-window continuity.

#### Scenario: Review evidence covers compactness hotspots
- **WHEN** the change is reviewed before implementation sign-off or merge
- **THEN** the evidence SHALL call out shell chrome, commit dialog, branch/menu surfaces, bottom history tool window, and row density hotspots
- **THEN** reviewers SHALL be able to tell whether the UI is materially closer to PhpStorm compactness rather than only color-matched
