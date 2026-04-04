# Quickstart: JetBrains-Styled Diff File List Panel

**Feature**: 003-jetbrains-diff-ui
**Date**: 2026-03-22

## Building the Feature

### Prerequisites

- Rust 2021+ with `cargo`
- Platform: macOS 11+, Windows 10+, or Ubuntu 20.04+

### Build Commands

```bash
# Build the project
cargo build

# Run tests
cargo test

# Run clippy for linting
cargo clippy

# Run with logging
RUST_LOG=debug cargo run
```

### Project Structure

```
src/
├── main.rs              # Entry point with CJK font setup
├── lib.rs               # Library exports
├── ui/                  # Iced UI layer
│   ├── mod.rs
│   ├── theme.rs         # Darcula theme (KEY FILE)
│   ├── diff_panel.rs    # Diff file list panel
│   ├── diff_view.rs     # File diff viewer
│   └── components/      # Reusable components
└── git_core/           # Git operations library
    ├── mod.rs
    ├── status.rs       # Get changed files
    └── diff.rs         # Generate diff content
```

## Key Implementation Files

### 1. theme.rs - Darcula Theme Definition

**Location**: `src/ui/theme.rs`

**Purpose**: Defines the Darcula color palette and theme for the entire application.

**Key Colors**:
```rust
// File status colors
const STATUS_ADDED: Color = Color::from_rgb(0.384, 0.580, 0.333);      // #629755
const STATUS_MODIFIED: Color = Color::from_rgb(0.408, 0.592, 0.733);   // #6897BB
const STATUS_DELETED: Color = Color::from_rgb(0.424, 0.424, 0.424);    // #6C6C6C
const STATUS_RENAMED: Color = Color::from_rgb(0.220, 0.518, 0.518);     // #3A8484

// Background colors
const BG_MAIN: Color = Color::from_rgb(0.169, 0.173, 0.188);            // #2B2B2B
const BG_PANEL: Color = Color::from_rgb(0.192, 0.196, 0.207);           // #313335
const BG_EDITOR: Color = Color::from_rgb(0.118, 0.118, 0.118);         // #1E1E1E

// Text colors
const TEXT_PRIMARY: Color = Color::from_rgb(0.737, 0.741, 0.749);       // #BDBDBD
const TEXT_SECONDARY: Color = Color::from_rgb(0.502, 0.502, 0.502);     // #808080

// Selection
const SELECTION_BG: Color = Color::from_rgb(0.129, 0.259, 0.514);       // #214283
```

### 2. diff_panel.rs - Diff File List Panel

**Location**: `src/ui/diff_panel.rs`

**Purpose**: Displays list of changed files with status indicators.

**Key Types**:
- `DiffPanel` - Main panel widget
- `ChangedFileRow` - Individual file entry with status icon
- `FileStatusIcon` - Visual indicator for file status

### 3. diff_view.rs - File Diff Viewer

**Location**: `src/ui/diff_view.rs`

**Purpose**: Shows the diff content for the selected file.

**Key Types**:
- `DiffView` - Split pane diff viewer
- `DiffHunk` - Grouped changes
- `DiffLine` - Individual line with highlighting

### 4. git_core/status.rs - Git Status

**Location**: `src/git_core/status.rs`

**Purpose**: Retrieves list of changed files from git repository.

**Key Function**:
```rust
pub fn get_changed_files(repo_path: &Path) -> Result<Vec<ChangedFile>, GitError>
```

### 5. git_core/diff.rs - Git Diff Generation

**Location**: `src/git_core/diff.rs`

**Purpose**: Generates diff content for a specific file.

**Key Function**:
```rust
pub fn get_file_diff(repo_path: &Path, file_path: &str) -> Result<DiffContent, GitError>
```

## Theme Application

To apply the Darcula theme to your application in `main.rs`:

```rust
use iced::{Application, Theme, Settings};
use iced::theme::Palette;

fn main() -> iced::Result {
    MyApp::run(Settings {
        default_theme: Theme::custom("Darcula".to_string(), Palette {
            background: Color::from_rgb(0.169, 0.173, 0.188),  // #2B2B2B
            text: Color::from_rgb(0.737, 0.741, 0.749),        // #BDBDBD
            primary: Color::from_rgb(0.408, 0.592, 0.733),     // #6897BB
            success: Color::from_rgb(0.384, 0.580, 0.333),     // #629755
            danger: Color::from_rgb(0.424, 0.424, 0.424),      // #6C6C6C
        }),
        ..Default::default()
    })
}
```

## Testing

```bash
# Run unit tests for git_core
cargo test git_core

# Run UI component tests
cargo test ui

# Run all tests with coverage
cargo test
```

## Development Workflow

1. **Implement git_core library** - Pure git operations without UI
2. **Create theme.rs** - Darcula color definitions
3. **Build diff_panel.rs** - File list with status icons
4. **Build diff_view.rs** - Diff content viewer
5. **Integrate in main_window.rs** - Put panels together
6. **Add Chinese localization** - All UI text in Chinese
7. **Integration testing** - Verify IDEA parity
