# slio-git Development Guidelines

Auto-generated from all feature plans. Last updated: 2026-04-04

## Active Technologies
- Rust 2021+ + iced 0.14 (UI framework), git2 0.19 (libgit2 bindings), notify 8 (file watching) (002-jetbrains-ui-refactor)
- N/A (git repositories are file-based) (002-jetbrains-ui-refactor)
- Rust 2021+ + Iced 0.14 (pure Rust UI framework), git2 0.19 (libgit2 bindings), notify 8 (file watching) (010-idea-git-tool-window)
- Rust (edition 2021+) + Iced 0.14 (UI), git2 0.19 (libgit2 bindings), notify 8 (file watching), syntect (syntax highlighting) (011-idea-git-parity)
- File-based (git repositories); commit message history stored in `~/.config/slio-git/` (011-idea-git-parity)

- Rust (edition 2021+) + Pure Iced (native Rust UI), git2-rs (libgit2 bindings), notify (file watching) (001-gitlight-intellij-replica)

## Project Structure

```text
src/
tests/
```

## Commands

cargo test [ONLY COMMANDS FOR ACTIVE TECHNOLOGIES][ONLY COMMANDS FOR ACTIVE TECHNOLOGIES] cargo clippy

## Code Style

Rust (edition 2021+): Follow standard conventions

## Recent Changes
- 013-compact-menus-graph: Added Rust (edition 2021+) + Iced 0.14
- 012-idea-ui-refactor: Added Rust (edition 2021+) + Iced 0.14 (UI), git2 0.19 (libgit2 bindings), notify 8 (file watching), syntect (syntax highlighting)
- 011-idea-git-parity: Added Rust (edition 2021+) + Iced 0.14 (UI), git2 0.19 (libgit2 bindings), notify 8 (file watching), syntect (syntax highlighting)


<!-- MANUAL ADDITIONS START -->
<!-- MANUAL ADDITIONS END -->
