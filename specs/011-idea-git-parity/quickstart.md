# Quickstart: IDEA Git Feature Parity

**Branch**: `011-idea-git-parity` | **Date**: 2026-04-04

## Prerequisites

- Rust toolchain (edition 2021+, stable channel)
- macOS 11+ (primary), Linux, or Windows 10+
- Git 2.30+ installed system-wide
- GPG or SSH agent configured (for signature verification features)

## Build & Run

```bash
# Clone and switch to feature branch
cd /Users/wanghao/git/slio-git
git checkout 011-idea-git-parity

# Build
cargo build

# Run (opens GUI pointing to current directory's git repo)
cargo run

# Run pointing to a specific repository
cargo run -- /path/to/git/repo
```

## Test

```bash
# Run all tests
cargo test

# Run git-core library tests only
cargo test -p git-core

# Run UI tests only
cargo test -p slio-git-ui

# Run with logging
RUST_LOG=debug cargo test
```

## Lint

```bash
cargo clippy
```

## Key Development Workflows

### Adding a new git-core operation

1. Add public function in appropriate `src/git-core/src/<module>.rs`
2. Re-export in `src/git-core/src/lib.rs`
3. Add integration test in `src/git-core/tests/`
4. Add structured logging with `log::info!` / `log::error!`

### Adding a new UI widget

1. Create widget file in `src-ui/src/widgets/<name>.rs`
2. Export in `src-ui/src/widgets/mod.rs`
3. Define message variants in `src-ui/src/main.rs`
4. Add state fields in `src-ui/src/state.rs`
5. Add Chinese labels in `src-ui/src/i18n.rs`

### Adding a new view

1. Create view file in `src-ui/src/views/<name>.rs`
2. Export in `src-ui/src/views/mod.rs`
3. Add routing in `src-ui/src/main.rs` update function
4. Wire up in `src-ui/src/views/main_window.rs` layout

## Architecture Overview

```
┌─────────────────────────────────────┐
│           Iced Application          │
│  ┌─────────┐  ┌──────────────────┐  │
│  │  Views   │  │    Widgets       │  │
│  │ (layout) │  │ (interactive)    │  │
│  └────┬─────┘  └────┬────────────┘  │
│       │              │               │
│  ┌────▼──────────────▼────┐          │
│  │     AppState            │          │
│  │  (state.rs / main.rs)   │          │
│  └────────────┬────────────┘          │
│               │ direct import         │
│  ┌────────────▼────────────┐          │
│  │      git-core           │          │
│  │  (pure Rust library)    │          │
│  └─────────────────────────┘          │
└─────────────────────────────────────┘
```

No IPC, no RPC, no WebView. Direct function calls from UI to git-core.
