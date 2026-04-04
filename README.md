<div align="center">

# slio-git

**A blazing-fast native Git client built with Rust + Iced**

*Pixel-perfect IntelliJ IDEA Git parity. Zero Electron. Pure Rust.*

[![Rust](https://img.shields.io/badge/Rust-2021+-orange?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Iced](https://img.shields.io/badge/Iced-0.14-blue?logo=data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHZpZXdCb3g9IjAgMCAyNCAyNCIgZmlsbD0ibm9uZSIgc3Ryb2tlPSJ3aGl0ZSIgc3Ryb2tlLXdpZHRoPSIyIj48Y2lyY2xlIGN4PSIxMiIgY3k9IjEyIiByPSIxMCIvPjwvc3ZnPg==)](https://iced.rs)
[![Tests](https://img.shields.io/badge/tests-143%20passing-brightgreen)](.)
[![License](https://img.shields.io/badge/license-MIT%20%2F%20Apache--2.0-blue)](LICENSE-MIT)

</div>

---

## Highlights

- **IntelliJ IDEA Git parity** --- context menus, branch popup, diff viewer, rebase editor all match IDEA's layout
- **GitHub-style inline diff** --- character-level change highlighting via `similar` crate
- **Split & unified diff** --- side-by-side and unified views with syntax highlighting
- **Non-modal commit** --- embedded commit panel, no popup interruption
- **Floating branch dropdown** --- IDEA-style popup with search, folders, tracking info
- **Full file preview** --- new/untracked files render with syntax highlighting
- **143 tests** --- integration tests against real git repos, zero mocks

## Architecture

```
                  +-----------------------+
                  |     Iced 0.14 UI      |
                  |  views / widgets /    |
                  |  components / theme   |
                  +-----------+-----------+
                              |  direct import
                  +-----------+-----------+
                  |      git-core         |
                  |  blame | graph |      |
                  |  signature | worktree |
                  |  submodule | similar  |
                  +-----------+-----------+
                              |
                  +-----------+-----------+
                  |   libgit2 (git2-rs)   |
                  +-----------------------+
```

### Tech Stack

| Layer | Technology | Purpose |
|-------|-----------|---------|
| **UI** | [Iced 0.14](https://iced.rs) | Native Rust GUI, no WebView |
| **Git** | [git2-rs 0.19](https://github.com/rust-lang/git2-rs) | libgit2 bindings |
| **Diff** | [similar 3.0](https://github.com/mitsuhiko/similar) | Character-level inline diff |
| **Syntax** | [syntect 5.3](https://github.com/trishume/syntect) | Syntax highlighting |
| **Watch** | [notify 8](https://github.com/notify-rs/notify) | File system change detection |
| **Async** | [Tokio](https://tokio.rs) | Async runtime |

### Git Operations (git-core)

| Module | Operations |
|--------|-----------|
| `blame` | Per-line attribution via git2 blame API |
| `branch` | Create, delete, rename, checkout, merge, rebase, group_path |
| `commit` | Create, amend, message history, validate ref |
| `commit_actions` | Cherry-pick, revert, uncommit, squash, fixup, drop |
| `diff` | Unified, split, inline char-level, full file preview, binary detection |
| `graph` | Lane-based commit graph layout, ref label computation |
| `history` | Browse, search, filter by author/path/date |
| `index` | Stage, unstage, hunk-level staging, status |
| `rebase` | Interactive rebase, todo editing, continue/abort/skip |
| `remote` | Fetch, pull, push, force-push (--force-with-lease) |
| `signature` | GPG/SSH signature extraction and verification |
| `stash` | Save (keep-index), apply, pop, drop, clear, unstash-as-branch |
| `submodule` | Detection, change summary |
| `tag` | Create (annotated/lightweight), delete, push, delete remote |
| `worktree` | Create, list, remove |

## Quick Start

```bash
git clone https://github.com/user/slio-git.git
cd slio-git

cargo run          # Launch the app
cargo test         # Run 143 tests
cargo clippy       # Lint
```

### Requirements

- Rust 1.70+ (edition 2021)
- macOS 11+ / Linux / Windows 10+

## Features in Detail

### Stage Panel
Collapsible Staged/Unstaged groups with flat/tree display toggle, per-file +/- buttons, hunk-level staging.

### Commit Graph
Lane-based graph visualization with branch lines, merge points, ref label badges.

### Branch Popup
IDEA-style floating dropdown with search, folder grouping, tracking branch display, behind/ahead indicators.

### Diff Viewer
- **Unified**: single-pane with syntax highlighting + inline char-level change markers
- **Split**: side-by-side 50/50 with clipped overflow, hunk stage/unstage buttons
- **Full preview**: new files shown with all-green addition lines

### Conflict Resolver
Three-pane merge (ours / result / theirs) with per-chunk accept/reject and auto-merge.

### Interactive Rebase
3-column table (action/hash/message), toolbar with move up/down/pick/edit, inline message editing, drag-and-drop.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT License](LICENSE-MIT) at your option.

---

<div align="center">
<sub>Built with Rust + Iced. Designed to match IntelliJ IDEA.</sub>
</div>
