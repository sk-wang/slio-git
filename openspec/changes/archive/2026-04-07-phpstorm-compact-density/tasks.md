## 1. Tighten shared density tokens

- [x] 1.1 Add a compact density profile in `/Users/wanghao/git/slio-git/src-ui/src/theme.rs` for spacing, control heights, tabs, panel padding, and section captions
- [x] 1.2 Update shared widgets in `/Users/wanghao/git/slio-git/src-ui/src/widgets/button.rs`, `/Users/wanghao/git/slio-git/src-ui/src/widgets/text_input.rs`, `/Users/wanghao/git/slio-git/src-ui/src/widgets/scrollable.rs`, and `/Users/wanghao/git/slio-git/src-ui/src/widgets/statusbar.rs` to consume the tighter metrics consistently

## 2. Compress the core workbench surfaces

- [x] 2.1 Refine `/Users/wanghao/git/slio-git/src-ui/src/views/main_window.rs` so the top chrome, tabs, and bottom tool-window header match the compact PhpStorm-like baseline
- [x] 2.2 Tighten `/Users/wanghao/git/slio-git/src-ui/src/widgets/changelist.rs`, `/Users/wanghao/git/slio-git/src-ui/src/widgets/diff_file_header.rs`, and `/Users/wanghao/git/slio-git/src-ui/src/widgets/diff_viewer.rs` so more change-review content fits per viewport without losing state clarity
- [x] 2.3 Adjust `/Users/wanghao/git/slio-git/src-ui/src/views/history_view.rs` and related shell state so the bottom history/log tool window remains continuous while using denser headers and row treatments

## 3. Align compactness across dialogs and menus

- [x] 3.1 Rework `/Users/wanghao/git/slio-git/src-ui/src/views/commit_dialog.rs` to reduce redundant section chrome and fit file list, diff preview, message editor, and action row into a denser workflow
- [x] 3.2 Update `/Users/wanghao/git/slio-git/src-ui/src/views/branch_popup.rs` and related repository action menus so compact row height, grouping rhythm, and hover/disabled states stay consistent across popups

## 4. Capture parity evidence

- [x] 4.1 Refresh `/Users/wanghao/git/slio-git/docs/phpstorm-parity-checklist.md` or equivalent review notes with compact-density checkpoints for shell chrome, commit dialog, menus, and bottom tool window
- [x] 4.2 Run targeted UI/regression verification for repository open, change review, commit, branch switching, and history navigation after the density changes land
