# slio-git

A native desktop Git client built with Rust and [Iced](https://iced.rs).

![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)

## Features

- **Repository management** - Open, browse, and manage Git repositories
- **Change list** - Stage, unstage, and discard changes with a JetBrains-style workbench layout
- **Diff viewer** - Side-by-side diff with syntax highlighting (via syntect)
- **Branch operations** - Create, switch, merge, and rebase branches
- **History view** - Browse commit history with full graph visualization
- **Commit dialog** - Write commit messages with a dedicated editor
- **Rebase editor** - Interactive rebase with drag-and-drop todo reordering
- **Stash management** - Create, apply, and pop stashes
- **Tag support** - Create and manage tags
- **Remote dialog** - Configure and manage remotes
- **Conflict resolver** - Visual merge conflict resolution
- **File watching** - Auto-refresh on filesystem changes (via notify)
- **Keyboard shortcuts** - Full keyboard-driven workflow
- **Dark theme** - Custom dark UI with a compact, JetBrains-inspired layout

## Architecture

```
slio-git/
├── src/
│   └── git-core/        # Core Git operations library (git2-rs)
├── src-ui/              # Iced-based desktop UI
│   └── src/
│       ├── components/  # Reusable UI components (status icons, rail)
│       ├── views/       # Top-level views (main window, popups, dialogs)
│       └── widgets/     # Custom Iced widgets (diff, scrollable, buttons)
└── specs/               # Feature specifications
```

**Tech stack:**

| Layer | Technology |
|-------|-----------|
| UI Framework | [Iced 0.14](https://iced.rs) (native Rust GUI) |
| Git Operations | [git2-rs 0.19](https://github.com/rust-lang/git2-rs) (libgit2 bindings) |
| File Watching | [notify 8](https://github.com/notify-rs/notify) |
| Async Runtime | [Tokio](https://tokio.rs) |
| Syntax Highlighting | [syntect](https://github.com/trishume/syntect) |

## Getting Started

### Prerequisites

- Rust 1.70+ (edition 2021)
- macOS (primary target)
- libgit2 (bundled via git2-rs)

### Build & Run

```bash
# Clone
git clone https://github.com/your-org/slio-git.git
cd slio-git

# Build and run
cargo run -p src-ui

# Run tests
cargo test

# Lint
cargo clippy
```

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT License](LICENSE-MIT) at your option.
