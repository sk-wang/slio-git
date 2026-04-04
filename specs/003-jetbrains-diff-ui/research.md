# Research: JetBrains-Styled Diff File List Panel

**Feature**: 003-jetbrains-diff-ui
**Date**: 2026-03-22

## Research Questions

### RQ1: IntelliJ IDEA Diff Panel UI Structure and Darcula Theme Colors

**Finding**: IntelliJ IDEA's git diff panel uses the following structure and colors:

**File Status Colors**:
| Status | Hex Code |
|--------|----------|
| Added | `#629755` (green) |
| Modified | `#6897BB` (light blue) |
| Deleted | `#6C6C6C` (gray) |
| Renamed | `#3A8484` (teal) |
| Unversioned | `#D1675A` (orange) |

**Background Colors**:
| Element | Hex Code |
|---------|----------|
| Main Background | `#2B2B2B` |
| Editor Background | `#1E1E1E` |
| Panel Background | `#313335` |
| Toolbar Background | `#2B2B2B` |

**Text Colors**:
| Element | Hex Code |
|---------|----------|
| Primary Text | `#BDBDBD` |
| Secondary Text | `#808080` |
| Disabled Text | `#555555` |

**Selection/Highlight**:
| Element | Hex Code |
|---------|----------|
| Selection Background | `#214283` |
| Selection Inactive | `#3A3A3A` |
| Highlight Background | `#213B5C` |

**Diff Highlighting**:
| Change Type | Hex Code |
|------------|----------|
| Added lines bg | `#2B6742` |
| Modified lines bg | `#365880` |
| Deleted lines bg | `#3B3B3B` |

**Alternatives considered**: Light theme (rejected per user request for Darcula)

---

### RQ2: iced Framework Theme Implementation

**Finding**: Iced 0.13 supports custom themes via:

1. **Custom Palette**: Define base colors using `Palette` struct with 5 core colors (background, text, primary, success, danger)

2. **Extended Palette**: For more detailed control, use `Extended::generate()` from base palette

3. **Custom Theme Creation**:
```rust
let theme = Theme::custom("Darcula".to_string(), Palette {
    background: Color::from_rgb(0.169, 0.173, 0.188),  // #2B2B2B
    text: Color::from_rgb(0.737, 0.741, 0.749),        // #BDBDBD
    primary: Color::from_rgb(0.408, 0.592, 0.733),     // #6897BB
    success: Color::from_rgb(0.384, 0.580, 0.333),     // #629755
    danger: Color::from_rgb(0.424, 0.424, 0.424),       // #6C6C6C
});
```

4. **Component Styling**: Use closure-based `.style()` methods on widgets for per-component styling

5. **Font Configuration**: Use `iced::Font::with_name()` for system fonts; CJK fonts already configured in existing codebase

**Alternatives considered**: Using built-in Dracula theme (close but not exact IDEA match)

---

## Consolidated Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Darcula Color Palette | Custom implementation | IDEA-specific colors differ from standard Dracula |
| Theme Implementation | Custom Palette + Component styling | Gives fine-grained control over all UI elements |
| File Status Colors | IDEA official colors | Ensures IntelliJ compatibility |
| Background | `#2B2B2B` main, `#313335` panels | Matches IDEA tool windows |
| Selection Highlight | `#214283` | IDEA selection color |

## Technical Notes

1. **iced 0.13 limitations**: No native split pane widget - need to implement custom layout using `Row`/`Column`
2. **Scrollable styling**: Use `Rail` styling for custom scrollbar appearance
3. **Chinese fonts**: Already configured in existing codebase via platform-specific Font::with_name()

## References

- JetBrains Help: File Status Highlights
- JetBrains Help: Comparing Files
- JetBrains Help: Differences Viewer
- JetBrains Help: Version Control Tool Window
- iced 0.13.0 Theme Documentation
- iced 0.13.0 Palette Struct
- iced 0.13.0 Custom Theme
