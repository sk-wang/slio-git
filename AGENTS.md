# Codex Instructions

Use `CLAUDE.md` in this repository as the primary project guidance file.

## Codex-specific notes

- Project slash prompts for Codex live in `.codex/prompts/`.
- Those prompt files delegate to `.claude/commands/`, which remains the source of truth.
- When a prompt or script needs an agent target, use `codex` instead of `claude`.

## Active Technologies
- Rust 2021+ + iced 0.13 (native UI), git2 0.19, notify 8, tokio 1, rfd 0.15, log/env_logger, chrono, once_cell (004-ui-usability-refresh)
- N/A（Git 仓库与本地文件系统；UI 状态以内存为主） (004-ui-usability-refresh)
- Rust 2021+ + iced 0.13（native UI）, git2 0.19, notify 8, tokio 1, rfd 0.15, log/env_logger, chrono, once_cell (005-idea-minimal-shell)
- Rust 2021+ + iced 0.13（native UI）, git2 0.19, notify 8, tokio 1, rfd 0.15, log/env_logger, chrono, once_cell, syntect 6 (006-phpstorm-style-polish)
- Rust 2021+ + iced 0.14（native UI）, git2 0.19, notify 8, tokio 1, rfd 0.15, log/env_logger, chrono, once_cell, syntect 5.3 (008-idea-lite-git)
- Git 仓库与本地文件系统；最近项目与工作区记忆使用本地文件持久化 (008-idea-lite-git)
- Rust 2021+ + iced 0.14（native UI）, git2 0.19, notify 8, tokio 1, rfd 0.15, log/env_logger, chrono, syntect 5.3 (009-branch-history-actions)
- Git 仓库与本地文件系统；补丁导出与临时确认结果使用本地文件 / 进程级状态，不引入额外数据库 (009-branch-history-actions)

## Recent Changes
- 004-ui-usability-refresh: Added Rust 2021+ + iced 0.13 (native UI), git2 0.19, notify 8, tokio 1, rfd 0.15, log/env_logger, chrono, once_cell
