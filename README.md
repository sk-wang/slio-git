<div align="center">

# slio-git

**A blazing-fast native Git client built with Rust + Iced**

*Pixel-perfect IntelliJ IDEA Git parity. Meld-quality diff. Zero Electron. Pure Rust.*

[![Rust](https://img.shields.io/badge/Rust-2021+-orange?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Iced](https://img.shields.io/badge/Iced-0.14-blue?logo=data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHZpZXdCb3g9IjAgMCAyNCAyNCIgZmlsbD0ibm9uZSIgc3Ryb2tlPSJ3aGl0ZSIgc3Ryb2tlLXdpZHRoPSIyIj48Y2lyY2xlIGN4PSIxMiIgY3k9IjEyIiByPSIxMCIvPjwvc3ZnPg==)](https://iced.rs)
[![Release](https://img.shields.io/badge/release-v0.0.1-brightgreen)](https://github.com/nicx-next/slio-git/releases/tag/v0.0.1)
[![License](https://img.shields.io/badge/license-MIT%20%2F%20Apache--2.0-blue)](LICENSE-MIT)

</div>

<p align="center">
  <img src="docs/assets/screenshot.png" alt="slio-git screenshot" width="960" />
</p>

---

## Highlights

- **IntelliJ IDEA Git parity** --- context menus, branch popup, commit panel, push/pull dialogs all match IDEA's layout
- **Meld-quality diff engine** --- Meld-style line alignment (Equal/Insert/Delete/Replace) with chunk boundary lines, 3-char kmer inline filtering, 20K character threshold
- **Split & unified diff** --- side-by-side with clipped overflow + unified view, both with syntax highlighting and hunk-level staging
- **AI commit message** --- LLM-powered commit message generation via OpenAI-compatible API (DeepSeek default), considers branch name + recent git log + staged diff
- **Smart checkout** --- IDEA-style dialog when switching branches with uncommitted changes: Smart Checkout (stash/checkout/unstash), Force Checkout, or Cancel
- **Floating branch dropdown** --- IDEA-style popup with search, folder grouping, tracking info, behind/ahead indicators
- **Git settings panel** --- IDEA-style configuration for commit, push, update, fetch, and LLM settings
- **Non-modal commit** --- embedded commit panel with amend toggle, message history, AI generate button

## Install

### macOS (Apple Silicon / Intel)

Download **slio-git-v0.0.1.dmg** from the [Releases](https://github.com/nicx-next/slio-git/releases/tag/v0.0.1) page.

### Build from source

```bash
git clone https://github.com/nicx-next/slio-git.git
cd slio-git
cargo build --release
# Binary at target/release/src-ui
```

### Requirements

- macOS 12+ / Linux / Windows 10+
- Rust 1.70+ (edition 2021) for building from source

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
                  |  blame | graph | llm  |
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
| **Diff** | [similar 3.0](https://github.com/mitsuhiko/similar) | Meld-style character-level inline diff |
| **Syntax** | [syntect 5.3](https://github.com/trishume/syntect) | Syntax highlighting |
| **HTTP** | [reqwest 0.12](https://github.com/seanmonstar/reqwest) | LLM API calls |
| **Watch** | [notify 8](https://github.com/notify-rs/notify) | File system change detection |
| **Async** | [Tokio](https://tokio.rs) | Async runtime |

### Git Operations (git-core)

| Module | Operations |
|--------|-----------|
| `blame` | Per-line attribution via git2 blame API |
| `branch` | Create, delete, rename, checkout, smart checkout, force checkout, merge, rebase |
| `commit` | Create, amend, message history, validate ref |
| `commit_actions` | Cherry-pick, revert, uncommit, squash, fixup, drop |
| `diff` | Meld-style unified/split, inline char-level (3-char kmer, 20K limit), full file preview |
| `graph` | Lane-based commit graph layout, ref label computation |
| `history` | Browse, search, filter by author/path/date |
| `index` | Stage, unstage, hunk-level staging, status |
| `llm` | OpenAI-compatible commit message generation (DeepSeek, GPT, etc.) |
| `rebase` | Interactive rebase, todo editing, continue/abort/skip |
| `remote` | Fetch, pull, push, force-push (--force-with-lease) |
| `signature` | GPG/SSH signature extraction and verification |
| `stash` | Save (keep-index), apply, pop, drop, clear, unstash-as-branch |
| `submodule` | Detection, change summary |
| `tag` | Create (annotated/lightweight), delete, push, delete remote |
| `worktree` | Create, list, remove |

## Features

### Diff Viewer (Meld-style)
- **Unified**: single-pane with syntax highlighting + inline char-level change markers
- **Split**: side-by-side 50/50 with Meld-style line alignment, chunk boundary lines, clipped overflow
- **Replace chunks**: paired deletions/additions shown in blue with character-level inline accent
- **Algorithms**: Myers O(NP) via `similar`, 3-char minimum match filter, 20K char threshold

### Smart Checkout
When switching branches with uncommitted changes, shows an IDEA-style dialog:
- **Smart Checkout**: stash changes, checkout, unstash (preserves your work)
- **Force Checkout**: discard all changes and switch
- **Don't Checkout**: cancel the operation

### AI Commit Message
- Configure OpenAI-compatible LLM in Settings (default: DeepSeek)
- Considers: current branch name, recent 15 git log entries, staged diff
- Generates conventional commit format matching your project's style

### Branch Popup
IDEA-style floating dropdown with search, folder grouping, tracking branch display, behind/ahead indicators.

### Push / Pull Dialogs
IDEA-style push/pull panels with force-push, tags, upstream options, rebase/ff-only/no-ff/squash modes.

### Settings
IDEA-style configuration panel: commit, push, update, fetch settings + LLM API configuration.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT License](LICENSE-MIT) at your option.

---

<div align="center">
<sub>Built with Rust + Iced. Designed to match IntelliJ IDEA. Diff engine inspired by Meld.</sub>
</div>
