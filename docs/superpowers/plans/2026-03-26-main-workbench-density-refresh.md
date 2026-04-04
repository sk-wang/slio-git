# Main Workbench Density Refresh Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Tighten the `slio-git` homepage into a denser, calmer, editor-first Git workbench that keeps the Yanqu light palette while moving layout rhythm and control discipline closer to PhpStorm / IDEA.

**Architecture:** Keep the existing homepage structure and behavior, but introduce homepage-specific density tokens plus a small set of reusable compact helpers for buttons, scrollbars, chips, and diff headers. Apply those shared helpers first, then consume them in the main workbench shell so the top chrome, rail, change list, diff pane, and status bar all read as one coordinated editor surface.

**Tech Stack:** Rust 2021, `iced` 0.14, `git2` 0.19, `notify` 8, `tokio` 1, `rfd` 0.15, `syntect` 5.3.

---

## File Structure

- `src-ui/src/theme.rs:53-620` - add homepage-specific density tokens, compact surface roles, and quieter frame/scroll styling without changing the global light palette.
- `src-ui/src/widgets/button.rs:1-139` - centralize button metrics so toolbar buttons, tabs, icon buttons, and split-button segments share height and padding rules.
- `src-ui/src/widgets/scrollable.rs:1-39` - split pane scrollbars from inline scrollbars so only true long-content regions keep visible scrolling affordances.
- `src-ui/src/widgets/mod.rs:82-176` - expose compact chip helpers and export any new shared homepage widget modules.
- `src-ui/src/views/main_window.rs:232-966` - compact the top chrome, badge selection, remote split button, and rail so the shell behaves like an editor toolbar instead of stacked cards.
- `src-ui/src/main.rs:3237-3446` - tighten the homepage pane shells, commit footer, and diff header.
- `src-ui/src/widgets/changelist.rs:13-320` - restyle the change summary and rows into a structured editor list with flatter selection and no chip-heavy metadata lines.
- `src-ui/src/widgets/diff_file_header.rs` - new shared compact diff file header metadata/view helper consumed by both unified and split diff viewers.
- `src-ui/src/widgets/diff_viewer.rs:24-237` - remove the extra summary strip and switch file headers/editor framing to the compact shared style.
- `src-ui/src/widgets/split_diff_viewer.rs:31-233` - mirror the same compact header/framing rules in split mode so both diff presentations stay visually aligned.
- `src-ui/src/widgets/statusbar.rs:10-105` - turn the status bar into a thin utility strip with truncated inline text instead of multiple horizontal scroll containers.

### Implementation Notes

- Do not rename or repurpose the existing Yanqu colors in `theme::darcula`; introduce homepage density tokens next to them so auxiliary views do not get a surprise global spacing shift.
- Keep behavior intact: no new Git actions, no new routing, no new data flow. The plan only changes layout density, surface hierarchy, and helper reuse.
- Keep the homepage shell in `src-ui/src/main.rs` and `src-ui/src/views/main_window.rs`; do not start a broad refactor of unrelated panels while doing this pass.
- `split_diff_viewer` is in scope even though the spec only calls out one homepage, because the diff mode toggle lives in the homepage header and both presentations must look consistent.

### Verification Commands Used Throughout

- Focused tests: `cargo test -p src-ui <test_filter> -- --exact`
- Package build check: `cargo check -p src-ui`
- Full UI crate tests: `cargo test -p src-ui`
- Manual visual pass: `cargo run -p src-ui`

---

### Task 1: Add Homepage Density Tokens And Compact Surface Roles

**Files:**
- Modify: `src-ui/src/theme.rs:53-121`
- Modify: `src-ui/src/theme.rs:179-620`
- Modify: `src-ui/src/widgets/mod.rs:82-176`
- Test: `src-ui/src/theme.rs`

- [ ] **Step 1: Write the failing token/surface tests in `src-ui/src/theme.rs`**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn homepage_density_tokens_match_spec() {
        assert_eq!(density::TOOLBAR_PADDING, [6, 12]);
        assert_eq!(density::SECONDARY_BAR_PADDING, [5, 12]);
        assert_eq!(density::PANE_PADDING, [12, 12]);
        assert_eq!(density::STATUS_PADDING, [4, 12]);
        assert_eq!(density::STANDARD_CONTROL_HEIGHT, 32.0);
        assert_eq!(density::COMPACT_CONTROL_HEIGHT, 28.0);
        assert_eq!(density::COMPACT_CHIP_PADDING, [3, 8]);
    }

    #[test]
    fn homepage_compact_surfaces_are_available() {
        let theme = darcula_theme();
        let toolbar_field = panel_style(Surface::ToolbarField)(&theme);
        let list_row = panel_style(Surface::ListRow)(&theme);
        let list_selection = panel_style(Surface::ListSelection)(&theme);

        assert_eq!(toolbar_field.shadow.blur_radius, 0.0);
        assert_eq!(list_row.shadow.blur_radius, 0.0);
        assert!(list_selection.border.color.a > list_row.border.color.a);
    }
}
```

- [ ] **Step 2: Run the focused tests to confirm the new tokens do not exist yet**

Run: `cargo test -p src-ui homepage_density_tokens_match_spec -- --exact`
Expected: FAIL with unresolved `density` module and unknown `Surface::ToolbarField` / `Surface::ListRow` variants.

- [ ] **Step 3: Implement homepage-specific density tokens and compact surfaces in `src-ui/src/theme.rs`**

```rust
pub mod density {
    pub const INLINE_GAP: f32 = 8.0;
    pub const CONTROL_GAP: f32 = 12.0;
    pub const TOOLBAR_PADDING: [u16; 2] = [6, 12];
    pub const SECONDARY_BAR_PADDING: [u16; 2] = [5, 12];
    pub const PANE_PADDING: [u16; 2] = [12, 12];
    pub const STATUS_PADDING: [u16; 2] = [4, 12];
    pub const STANDARD_CONTROL_HEIGHT: f32 = 32.0;
    pub const COMPACT_CONTROL_HEIGHT: f32 = 28.0;
    pub const COMPACT_CHIP_PADDING: [u16; 2] = [3, 8];
}

#[derive(Debug, Clone, Copy)]
pub enum Surface {
    Root,
    Nav,
    Rail,
    Toolbar,
    Status,
    Panel,
    Raised,
    Editor,
    Accent,
    Selection,
    Success,
    Warning,
    Danger,
    ToolbarField,
    ListRow,
    ListSelection,
}
```

```rust
pub fn panel_style(surface: Surface) -> impl Fn(&Theme) -> container::Style {
    move |_theme| {
        let (background, border_width, border_color, radius, shadow) = match surface {
            Surface::Root => (
                surface_background(surface),
                0.0,
                Color::TRANSPARENT,
                0.0,
                Shadow::default(),
            ),
            Surface::Editor => (
                surface_background(surface),
                1.0,
                darcula::BORDER.scale_alpha(0.88),
                radius::LG,
                Shadow::default(),
            ),
            Surface::Toolbar | Surface::Nav | Surface::Rail | Surface::Status => (
                surface_background(surface),
                0.0,
                Color::TRANSPARENT,
                0.0,
                Shadow::default(),
            ),
            Surface::ToolbarField => (
                mix(darcula::BG_PANEL, darcula::ACCENT_WEAK, 0.18),
                1.0,
                darcula::SEPARATOR.scale_alpha(0.84),
                radius::MD,
                Shadow::default(),
            ),
            Surface::ListRow => (
                darcula::BG_PANEL,
                1.0,
                darcula::SEPARATOR.scale_alpha(0.72),
                radius::MD,
                Shadow::default(),
            ),
            Surface::ListSelection => (
                mix(darcula::BG_PANEL, darcula::SELECTION_BG, 0.92),
                1.0,
                darcula::ACCENT.scale_alpha(0.26),
                radius::MD,
                Shadow::default(),
            ),
            Surface::Panel => (
                surface_background(surface),
                1.0,
                darcula::BORDER.scale_alpha(0.90),
                radius::LG,
                soft_shadow(0.03, 4.0, 12.0),
            ),
            Surface::Raised => (
                surface_background(surface),
                1.0,
                darcula::SEPARATOR.scale_alpha(0.82),
                radius::MD,
                Shadow::default(),
            ),
            Surface::Accent => (
                surface_background(surface),
                1.0,
                darcula::ACCENT.scale_alpha(0.28),
                radius::LG,
                Shadow::default(),
            ),
            Surface::Selection => (
                surface_background(surface),
                1.0,
                darcula::ACCENT.scale_alpha(0.40),
                radius::LG,
                Shadow::default(),
            ),
            Surface::Success => (
                surface_background(surface),
                1.0,
                darcula::SUCCESS.scale_alpha(0.26),
                radius::LG,
                Shadow::default(),
            ),
            Surface::Warning => (
                surface_background(surface),
                1.0,
                darcula::WARNING.scale_alpha(0.28),
                radius::LG,
                Shadow::default(),
            ),
            Surface::Danger => (
                surface_background(surface),
                1.0,
                darcula::DANGER.scale_alpha(0.24),
                radius::LG,
                Shadow::default(),
            ),
        };

        container::Style {
            background: Some(Background::Color(background)),
            border: Border {
                width: border_width,
                color: border_color,
                radius: radius.into(),
            },
            shadow,
            ..Default::default()
        }
    }
}
```

```rust
pub fn frame_style(surface: Surface) -> impl Fn(&Theme) -> container::Style {
    move |_theme| {
        let (border_width, border_color, shadow) = match surface {
            Surface::Toolbar => (0.0, Color::TRANSPARENT, soft_shadow(0.03, 3.0, 10.0)),
            Surface::Nav => (1.0, darcula::SEPARATOR.scale_alpha(0.60), Shadow::default()),
            Surface::Rail => (1.0, darcula::SEPARATOR.scale_alpha(0.72), Shadow::default()),
            Surface::Status => (1.0, darcula::SEPARATOR.scale_alpha(0.60), Shadow::default()),
            _ => (0.0, Color::TRANSPARENT, Shadow::default()),
        };

        container::Style {
            background: Some(Background::Color(surface_background(surface))),
            border: Border {
                width: border_width,
                color: border_color,
                radius: 0.0.into(),
            },
            shadow,
            ..Default::default()
        }
    }
}
```

- [ ] **Step 4: Add a homepage-only compact chip helper in `src-ui/src/widgets/mod.rs`**

```rust
pub fn compact_chip<'a, Message: 'a>(
    label: impl Into<String>,
    tone: BadgeTone,
) -> Element<'a, Message> {
    Container::new(Text::new(label.into()).size(10))
        .padding(theme::density::COMPACT_CHIP_PADDING)
        .style(theme::badge_style(tone))
        .into()
}
```

- [ ] **Step 5: Run the theme tests and crate build**

Run: `cargo test -p src-ui homepage_density_tokens_match_spec -- --exact`
Expected: PASS

Run: `cargo test -p src-ui homepage_compact_surfaces_are_available -- --exact`
Expected: PASS

Run: `cargo check -p src-ui`
Expected: PASS with `Finished` for `src-ui`.

- [ ] **Step 6: Commit the shared density baseline**

```bash
git add src-ui/src/theme.rs src-ui/src/widgets/mod.rs
git commit -m "refactor: add homepage density tokens"
```

### Task 2: Normalize Button Metrics And Scrollbar Presets

**Files:**
- Modify: `src-ui/src/widgets/button.rs:1-139`
- Modify: `src-ui/src/widgets/scrollable.rs:1-39`
- Modify: `src-ui/src/theme.rs:327-620`
- Test: `src-ui/src/widgets/button.rs`
- Test: `src-ui/src/widgets/scrollable.rs`

- [ ] **Step 1: Write the failing metrics tests in `src-ui/src/widgets/button.rs` and `src-ui/src/widgets/scrollable.rs`**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toolbar_split_segments_share_same_vertical_padding() {
        let main = metrics(ButtonRole::ToolbarSplitMain);
        let chevron = metrics(ButtonRole::ToolbarSplitChevron);

        assert_eq!(main.padding[0], chevron.padding[0]);
        assert_eq!(main.height, chevron.height);
        assert_eq!(main.height, 32);
    }

    #[test]
    fn tabs_share_the_standard_control_height() {
        assert_eq!(metrics(ButtonRole::Tab).height, metrics(ButtonRole::Standard).height);
    }
}
```

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inline_scrollbars_are_quieter_than_pane_scrollbars() {
        let pane = scrollbar_metrics(ScrollbarRole::Pane);
        let inline = scrollbar_metrics(ScrollbarRole::Inline);

        assert!(inline.width < pane.width);
        assert!(inline.scroller_width <= pane.scroller_width);
    }
}
```

- [ ] **Step 2: Run the focused tests to capture the missing metrics layer**

Run: `cargo test -p src-ui toolbar_split_segments_share_same_vertical_padding -- --exact`
Expected: FAIL with unresolved `ButtonRole` / `metrics` items.

Run: `cargo test -p src-ui inline_scrollbars_are_quieter_than_pane_scrollbars -- --exact`
Expected: FAIL with unresolved `ScrollbarRole` / `scrollbar_metrics` items.

- [ ] **Step 3: Add reusable button metrics and split-button helpers in `src-ui/src/widgets/button.rs`**

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ButtonRole {
    Standard,
    Compact,
    Tab,
    Rail,
    ToolbarIcon,
    ToolbarSplitMain,
    ToolbarSplitChevron,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ButtonMetrics {
    text_size: u16,
    padding: [u16; 2],
    height: u16,
    width: Option<f32>,
}

fn metrics(role: ButtonRole) -> ButtonMetrics {
    match role {
        ButtonRole::Standard => ButtonMetrics { text_size: 11, padding: [6, 12], height: 32, width: None },
        ButtonRole::Compact => ButtonMetrics { text_size: 10, padding: [4, 8], height: 28, width: None },
        ButtonRole::Tab => ButtonMetrics { text_size: 11, padding: [5, 10], height: 32, width: None },
        ButtonRole::Rail => ButtonMetrics { text_size: 11, padding: [6, 0], height: 30, width: None },
        ButtonRole::ToolbarIcon => ButtonMetrics { text_size: 13, padding: [4, 0], height: 28, width: Some(24.0) },
        ButtonRole::ToolbarSplitMain => ButtonMetrics { text_size: 11, padding: [6, 12], height: 32, width: None },
        ButtonRole::ToolbarSplitChevron => ButtonMetrics { text_size: 10, padding: [6, 0], height: 32, width: Some(22.0) },
    }
}
```

```rust
pub fn toolbar_split_main<'a, Message: Clone + 'a>(
    content: impl Into<Element<'a, Message>>,
    tone: ButtonTone,
    on_press: Option<Message>,
) -> Button<'a, Message> {
    let metrics = metrics(ButtonRole::ToolbarSplitMain);
    let button = Button::new(content)
        .padding(metrics.padding)
        .style(theme::button_style(tone))
        .height(Length::Fixed(metrics.height as f32));

    if let Some(message) = on_press {
        button.on_press(message)
    } else {
        button
    }
}

pub fn toolbar_split_chevron<'a, Message: Clone + 'a>(
    label: impl Into<String>,
    tone: ButtonTone,
    on_press: Option<Message>,
) -> Button<'a, Message> {
    let metrics = metrics(ButtonRole::ToolbarSplitChevron);
    let button = Button::new(Text::new(label.into()).size(u32::from(metrics.text_size)))
        .padding(metrics.padding)
        .width(Length::Fixed(metrics.width.expect("split chevron width")))
        .height(Length::Fixed(metrics.height as f32))
        .style(theme::button_style(tone));

    if let Some(message) = on_press {
        button.on_press(message)
    } else {
        button
    }
}
```

- [ ] **Step 4: Add pane-vs-inline scrollbar presets in `src-ui/src/widgets/scrollable.rs` and soften the theme scroller in `src-ui/src/theme.rs`**

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScrollbarRole {
    Pane,
    Inline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ScrollbarMetrics {
    width: u16,
    scroller_width: u16,
    margin: u16,
}

fn scrollbar_metrics(role: ScrollbarRole) -> ScrollbarMetrics {
    match role {
        ScrollbarRole::Pane => ScrollbarMetrics { width: 6, scroller_width: 4, margin: 1 },
        ScrollbarRole::Inline => ScrollbarMetrics { width: 4, scroller_width: 3, margin: 0 },
    }
}

fn build_scrollbar(role: ScrollbarRole) -> scrollable::Scrollbar {
    let metrics = scrollbar_metrics(role);
    scrollable::Scrollbar::new()
        .width(metrics.width)
        .scroller_width(metrics.scroller_width)
        .margin(metrics.margin)
}

pub fn styled_inline_horizontal<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
) -> Scrollable<'a, Message> {
    Scrollable::new(content)
        .direction(scrollable::Direction::Horizontal(build_scrollbar(ScrollbarRole::Inline)))
        .style(theme::scrollable_style())
}
```

```rust
pub fn scrollable_style() -> impl Fn(&Theme, scrollable::Status) -> scrollable::Style {
    move |_theme, status| {
        let (rail_background, rail_border, scroller_color) = match status {
            scrollable::Status::Active { .. } => (
                Background::Color(Color::TRANSPARENT),
                Color::TRANSPARENT,
                Color::TRANSPARENT,
            ),
            scrollable::Status::Hovered { .. } => (
                Background::Color(Color::TRANSPARENT),
                Color::TRANSPARENT,
                mix(darcula::TEXT_DISABLED, darcula::ACCENT, 0.12).scale_alpha(0.28),
            ),
            scrollable::Status::Dragged { .. } => (
                Background::Color(Color::TRANSPARENT),
                Color::TRANSPARENT,
                darcula::ACCENT.scale_alpha(0.52),
            ),
        };

        scrollable::Style {
            container: container::Style::default(),
            vertical_rail: scrollable::Rail {
                background: Some(rail_background),
                border: Border {
                    width: 0.0,
                    color: rail_border,
                    radius: radius::MD.into(),
                },
                scroller: scrollable::Scroller {
                    background: Background::Color(scroller_color),
                    border: Border {
                        width: 0.0,
                        color: rail_border,
                        radius: radius::MD.into(),
                    },
                },
            },
            horizontal_rail: scrollable::Rail {
                background: Some(rail_background),
                border: Border {
                    width: 0.0,
                    color: rail_border,
                    radius: radius::MD.into(),
                },
                scroller: scrollable::Scroller {
                    background: Background::Color(scroller_color),
                    border: Border {
                        width: 0.0,
                        color: rail_border,
                        radius: radius::MD.into(),
                    },
                },
            },
            gap: None,
            auto_scroll: scrollable::AutoScroll {
                background: Background::Color(mix(darcula::BG_MAIN, darcula::ACCENT_WEAK, 0.52)),
                border: Border {
                    width: 1.0,
                    color: darcula::ACCENT.scale_alpha(0.14),
                    radius: radius::MD.into(),
                },
                shadow: soft_shadow(0.03, 3.0, 8.0),
                icon: darcula::TEXT_SECONDARY,
            },
        }
    }
}
```

- [ ] **Step 5: Re-run focused tests plus the UI crate build**

Run: `cargo test -p src-ui toolbar_split_segments_share_same_vertical_padding -- --exact`
Expected: PASS

Run: `cargo test -p src-ui inline_scrollbars_are_quieter_than_pane_scrollbars -- --exact`
Expected: PASS

Run: `cargo check -p src-ui`
Expected: PASS with no new warnings from `src-ui`.

- [ ] **Step 6: Commit the shared control-metrics pass**

```bash
git add src-ui/src/widgets/button.rs src-ui/src/widgets/scrollable.rs src-ui/src/theme.rs
git commit -m "refactor: normalize workbench control metrics"
```

### Task 3: Compact The Top Chrome And Left Rail

**Files:**
- Modify: `src-ui/src/views/main_window.rs:232-708`
- Modify: `src-ui/src/views/main_window.rs:748-967`
- Modify: `src-ui/src/widgets/mod.rs:82-176`
- Test: `src-ui/src/views/main_window.rs`

- [ ] **Step 1: Write the failing badge-priority tests in `src-ui/src/views/main_window.rs`**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pick_branch_badges_prefers_state_hint_over_secondary_label() {
        let badges = MainWindow::<()>::pick_branch_badges(
            Some("跟踪 origin/main"),
            Some("有冲突"),
            Some("ahead 1"),
            "✓",
        );

        assert_eq!(badges.branch_badge, Some(("有冲突".to_string(), BadgeTone::Warning)));
        assert_eq!(badges.sync_badge, None);
    }

    #[test]
    fn show_sync_chip_hides_synced_and_no_upstream_states() {
        assert!(!MainWindow::<()>::show_sync_chip("✓"));
        assert!(!MainWindow::<()>::show_sync_chip("○"));
        assert!(MainWindow::<()>::show_sync_chip("↑2"));
        assert!(MainWindow::<()>::show_sync_chip("↕1/1"));
    }
}
```

- [ ] **Step 2: Run the focused main-window tests before changing the shell layout**

Run: `cargo test -p src-ui pick_branch_badges_prefers_state_hint_over_secondary_label -- --exact`
Expected: FAIL with missing `pick_branch_badges` helper.

Run: `cargo test -p src-ui show_sync_chip_hides_synced_and_no_upstream_states -- --exact`
Expected: FAIL until the badge-selection helper is added.

- [ ] **Step 3: Add badge-priority selection and switch the top chrome to compact field surfaces in `src-ui/src/views/main_window.rs`**

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
struct ChromeBadges {
    branch_badge: Option<(String, BadgeTone)>,
    sync_badge: Option<(String, BadgeTone)>,
}

fn pick_branch_badges(
    secondary_label: Option<&str>,
    state_hint: Option<&str>,
    sync_hint: Option<&str>,
    sync_label: &str,
) -> ChromeBadges {
    let branch_badge = state_hint
        .map(|label| (label.to_string(), BadgeTone::Warning))
        .or_else(|| secondary_label.map(|label| (label.to_string(), BadgeTone::Accent)))
        .or_else(|| sync_hint.map(|label| (label.to_string(), BadgeTone::Neutral)));

    let sync_badge = Self::show_sync_chip(sync_label)
        .then(|| (sync_label.to_string(), Self::sync_tone(sync_label)));

    ChromeBadges {
        branch_badge,
        sync_badge,
    }
}
```

```rust
let badges = Self::pick_branch_badges(
    context.secondary_label.as_deref(),
    context.state_hint.as_deref(),
    context.sync_hint.as_deref(),
    &context.sync_label,
);

let repo_switcher = Button::new(
    Container::new(
        Row::new()
            .spacing(theme::spacing::SM)
            .align_y(Alignment::Center)
            .push(
                Container::new(Self::inline_icon(
                    RailIcon::Repository,
                    theme::darcula::ACCENT,
                    13.0,
                ))
                .padding([4, 6])
                .style(theme::panel_style(Surface::Accent)),
            )
            .push(
                Column::new()
                    .spacing(1)
                    .push(
                        Text::new(&context.repository_name)
                            .size(12)
                            .width(Length::Fill)
                            .wrapping(text::Wrapping::WordOrGlyph),
                    )
                    .push(
                        Text::new(&context.repository_path)
                            .size(10)
                            .color(theme::darcula::TEXT_SECONDARY)
                            .width(Length::Fill)
                            .wrapping(text::Wrapping::WordOrGlyph),
                    ),
            ),
    )
        .padding(theme::density::TOOLBAR_PADDING)
        .width(Length::Fill)
        .style(theme::panel_style(Surface::ToolbarField)),
)
.style(theme::button_style(ButtonTone::Ghost))
.width(Length::FillPortion(4))
.on_press(on_show_branches.clone());

let branch_switcher = Button::new(
    Container::new(
        Row::new()
            .spacing(theme::spacing::XS)
            .align_y(Alignment::Center)
            .push(Self::inline_icon(RailIcon::Branch, theme::darcula::BRAND, 12.0))
            .push(Text::new(&context.branch_name).size(11))
            .push_maybe(
                badges.branch_badge.as_ref().map(|(label, tone)| {
                    widgets::compact_chip::<Message>(label.clone(), *tone)
                }),
            ),
    )
    .padding(theme::density::TOOLBAR_PADDING)
    .style(theme::panel_style(Surface::ToolbarField)),
)
.style(theme::button_style(ButtonTone::Ghost))
.on_press(on_show_branches.clone());
```

- [ ] **Step 4: Remove unnecessary horizontal scroll wrappers from the chrome and tighten the rail**

```rust
let tabs = Row::new()
    .spacing(theme::spacing::XS)
    .push(Self::nav_button(i18n.changes, state.shell.active_section == ShellSection::Changes, Some(on_show_changes.clone())))
    .push(Self::nav_button(i18n.conflicts, state.shell.active_section == ShellSection::Conflicts, state.has_conflicts().then_some(on_show_conflicts.clone())));

let quick_actions = Row::new()
    .spacing(theme::spacing::XS)
    .push(button::ghost(i18n.refresh, Some(on_refresh.clone())))
    .push(Self::toolbar_remote_split_button(
        i18n.pull,
        ToolbarRemoteAction::Pull,
        false,
        Some(on_pull.clone()),
        Some(on_toggle_remote_menu(ToolbarRemoteAction::Pull)),
        state
            .toolbar_remote_menu
            .as_ref()
            .is_some_and(|menu| menu.action == ToolbarRemoteAction::Pull),
    ))
    .push(Self::toolbar_remote_split_button(
        i18n.push,
        ToolbarRemoteAction::Push,
        true,
        Some(on_push.clone()),
        Some(on_toggle_remote_menu(ToolbarRemoteAction::Push)),
        state
            .toolbar_remote_menu
            .as_ref()
            .is_some_and(|menu| menu.action == ToolbarRemoteAction::Push),
    ))
    .push(button::secondary(i18n.commit, state.shell.chrome.has_staged_changes.then_some(on_commit.clone())));

let secondary_actions = Row::new()
    .spacing(theme::spacing::XS)
    .push(Self::utility_button("历史", state.auxiliary_view == Some(AuxiliaryView::History), Some(on_show_history.clone())))
    .push(Self::utility_button("远程", state.auxiliary_view == Some(AuxiliaryView::Remotes), Some(on_show_remotes.clone())))
    .push(Self::utility_button("标签", state.auxiliary_view == Some(AuxiliaryView::Tags), Some(on_show_tags.clone())))
    .push(Self::utility_button("储藏", state.auxiliary_view == Some(AuxiliaryView::Stashes), Some(on_show_stashes.clone())))
    .push(Self::utility_button("Rebase", state.auxiliary_view == Some(AuxiliaryView::Rebase), Some(on_show_rebase.clone())));
```

```rust
Container::new(navigation)
    .padding([12, 6])
    .width(Length::Fixed(theme::layout::RAIL_WIDTH))
    .height(Length::Fill)
    .style(theme::frame_style(Surface::Rail))
```

Use `button::toolbar_split_main` and `button::toolbar_split_chevron` inside `toolbar_remote_split_button()` so the pull/push split buttons stop drifting in height.

- [ ] **Step 5: Run the focused tests and a build check**

Run: `cargo test -p src-ui pick_branch_badges_prefers_state_hint_over_secondary_label -- --exact`
Expected: PASS

Run: `cargo test -p src-ui show_sync_chip_hides_synced_and_no_upstream_states -- --exact`
Expected: PASS

Run: `cargo check -p src-ui`
Expected: PASS.

- [ ] **Step 6: Commit the shell-density pass**

```bash
git add src-ui/src/views/main_window.rs src-ui/src/widgets/mod.rs
git commit -m "feat: compact workbench chrome and rail"
```

### Task 4: Reshape The Change List Pane And Commit Footer

**Files:**
- Modify: `src-ui/src/widgets/changelist.rs:13-320`
- Modify: `src-ui/src/main.rs:3237-3377`
- Modify: `src-ui/src/widgets/mod.rs:82-176`
- Test: `src-ui/src/widgets/changelist.rs`

- [ ] **Step 1: Write the failing change-row metadata tests in `src-ui/src/widgets/changelist.rs`**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn change_section_kind_exposes_compact_context_label() {
        assert_eq!(ChangeSectionKind::Staged.context_label(), "已暂存");
        assert_eq!(ChangeSectionKind::Unstaged.context_label(), "工作区修改");
        assert_eq!(ChangeSectionKind::Untracked.context_label(), "新文件");
    }

    #[test]
    fn split_path_returns_file_name_and_parent_directory() {
        assert_eq!(split_path("src/ui/main.rs"), ("main.rs".to_string(), "src/ui".to_string()));
        assert_eq!(split_path("Cargo.toml"), ("Cargo.toml".to_string(), String::new()));
    }
}
```

- [ ] **Step 2: Run the focused change-list tests to capture the missing helper method**

Run: `cargo test -p src-ui change_section_kind_exposes_compact_context_label -- --exact`
Expected: FAIL with missing `context_label()` on `ChangeSectionKind`.

- [ ] **Step 3: Replace the chip-heavy summary band and row cards in `src-ui/src/widgets/changelist.rs`**

```rust
impl ChangeSectionKind {
    fn context_label(self) -> &'static str {
        match self {
            ChangeSectionKind::Staged => "已暂存",
            ChangeSectionKind::Unstaged => "工作区修改",
            ChangeSectionKind::Untracked => "新文件",
        }
    }
}
```

```rust
let summary = Container::new(
    Row::new()
        .spacing(theme::spacing::XS)
        .align_y(Alignment::Center)
        .push(Text::new(format!(
            "{} 工作区修改 · {} 新文件 · {} 待提交",
            self.unstaged.len(),
            self.untracked.len(),
            self.staged.len(),
        ))
        .size(10)
        .color(theme::darcula::TEXT_SECONDARY))
        .push_maybe(self.selected_path.map(|path| {
            Text::new(format!("当前查看：{path}"))
                .size(10)
                .color(theme::darcula::TEXT_SECONDARY)
        }))
)
.padding([6, 0]);
```

```rust
let meta_line = [
    (!parent_path.is_empty()).then_some(parent_path.clone()),
    Some(status.label().to_string()),
    Some(kind.context_label().to_string()),
    is_selected.then_some("当前查看".to_string()),
]
.into_iter()
.flatten()
.collect::<Vec<_>>()
.join(" · ");

let item_panel = Container::new(
    Row::new()
        .spacing(theme::spacing::XS)
        .align_y(Alignment::Start)
        .push(Container::new(Text::new(status.symbol()).size(11).color(status.color())).width(Length::Fixed(14.0)))
        .push(
            Column::new()
                .spacing(1)
                .width(Length::Fill)
                .push(Text::new(file_name).size(12).width(Length::Fill).wrapping(text::Wrapping::WordOrGlyph))
                .push(Text::new(meta_line).size(10).color(theme::darcula::TEXT_SECONDARY)),
        ),
)
.padding([6, 8])
.width(Length::Fill)
.style(theme::panel_style(if is_selected {
    Surface::ListSelection
} else {
    Surface::ListRow
}));
```

- [ ] **Step 4: Turn the commit footer in `src-ui/src/main.rs` into a compact one-line action bar**

```rust
Container::new(
    Row::new()
        .spacing(theme::spacing::XS)
        .align_y(Alignment::Center)
        .push(Text::new("提交准备").size(11))
        .push(
            Text::new(status_text)
                .size(10)
                .color(theme::darcula::TEXT_SECONDARY),
        )
        .push(Space::new().width(Length::Fill))
        .push(button::primary(i18n.commit, can_commit.then_some(Message::Commit))),
)
.padding([6, 0])
.style(theme::frame_style(theme::Surface::Nav))
```

Also tighten the pane shell paddings in `src-ui/src/main.rs` from `[6, 8]` to `theme::density::PANE_PADDING` so the left pane stops wasting vertical space.

- [ ] **Step 5: Re-run the change-list tests and the UI build**

Run: `cargo test -p src-ui change_section_kind_exposes_compact_context_label -- --exact`
Expected: PASS

Run: `cargo test -p src-ui split_path_returns_file_name_and_parent_directory -- --exact`
Expected: PASS

Run: `cargo check -p src-ui`
Expected: PASS.

- [ ] **Step 6: Commit the denser change-list pane**

```bash
git add src-ui/src/widgets/changelist.rs src-ui/src/main.rs
git commit -m "feat: densify change list pane"
```

### Task 5: Compact The Diff Header And Both Diff Presentations

**Files:**
- Create: `src-ui/src/widgets/diff_file_header.rs`
- Modify: `src-ui/src/widgets/mod.rs:1-18`
- Modify: `src-ui/src/widgets/diff_viewer.rs:24-237`
- Modify: `src-ui/src/widgets/split_diff_viewer.rs:31-233`
- Modify: `src-ui/src/main.rs:3380-3465`
- Test: `src-ui/src/widgets/diff_file_header.rs`

- [ ] **Step 1: Write the failing shared diff-header tests in `src-ui/src/widgets/diff_file_header.rs`**

```rust
use crate::theme::BadgeTone;

#[derive(Debug, PartialEq, Eq)]
struct DiffFileHeaderMeta {
    file_name: String,
    parent_path: Option<String>,
    rename_hint: Option<String>,
    status_label: &'static str,
    status_tone: BadgeTone,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn describe_paths_marks_renames() {
        let meta = DiffFileHeaderMeta::describe(Some("old/src/lib.rs"), Some("new/src/lib.rs"));

        assert_eq!(meta.file_name, "lib.rs");
        assert_eq!(meta.parent_path.as_deref(), Some("new/src"));
        assert_eq!(meta.rename_hint.as_deref(), Some("old/src/lib.rs -> new/src/lib.rs"));
        assert_eq!(meta.status_label, "重命名");
        assert_eq!(meta.status_tone, BadgeTone::Accent);
    }

    #[test]
    fn describe_paths_handles_new_files() {
        let meta = DiffFileHeaderMeta::describe(None, Some("src/new.rs"));

        assert_eq!(meta.file_name, "new.rs");
        assert_eq!(meta.parent_path.as_deref(), Some("src"));
        assert_eq!(meta.rename_hint, None);
        assert_eq!(meta.status_label, "新文件");
        assert_eq!(meta.status_tone, BadgeTone::Success);
    }
}
```

- [ ] **Step 2: Run the focused diff-header tests before extracting the helper**

Run: `cargo test -p src-ui describe_paths_marks_renames -- --exact`
Expected: FAIL because `src-ui/src/widgets/diff_file_header.rs` does not exist yet.

- [ ] **Step 3: Create `src-ui/src/widgets/diff_file_header.rs` and export it from `src-ui/src/widgets/mod.rs`**

```rust
use crate::theme::{self, BadgeTone};
use crate::widgets;
use git_core::diff::FileDiff;
use iced::widget::{text, Column, Container, Row, Text};
use iced::{Alignment, Element, Length};
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffFileHeaderMeta {
    pub file_name: String,
    pub parent_path: Option<String>,
    pub rename_hint: Option<String>,
    pub status_label: &'static str,
    pub status_tone: BadgeTone,
}

impl DiffFileHeaderMeta {
    pub fn describe(old_path: Option<&str>, new_path: Option<&str>) -> Self {
        let full_label = new_path.or(old_path).unwrap_or("未命名文件");
        let file_name = Path::new(full_label)
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or(full_label)
            .to_string();
        let parent_path = Path::new(full_label)
            .parent()
            .and_then(|value| value.to_str())
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        let rename_hint = match (old_path, new_path) {
            (Some(old), Some(new)) if old != new => Some(format!("{old} -> {new}")),
            _ => None,
        };
        let (status_label, status_tone) = match (old_path, new_path) {
            (None, Some(_)) => ("新文件", BadgeTone::Success),
            (Some(_), None) => ("删除", BadgeTone::Danger),
            (Some(old), Some(new)) if old != new => ("重命名", BadgeTone::Accent),
            _ => ("修改", BadgeTone::Accent),
        };

        Self {
            file_name,
            parent_path,
            rename_hint,
            status_label,
            status_tone,
        }
    }

    pub fn from_file_diff(file_diff: &FileDiff) -> Self {
        Self::describe(file_diff.old_path.as_deref(), file_diff.new_path.as_deref())
    }
}

pub fn view<'a, Message: Clone + 'static>(
    meta: &'a DiffFileHeaderMeta,
    hunks: usize,
    additions: usize,
    deletions: usize,
) -> Element<'a, Message> {
    Container::new(
        Row::new()
            .spacing(theme::spacing::XS)
            .align_y(Alignment::Center)
            .push(
                Column::new()
                    .spacing(1)
                    .width(Length::Fill)
                    .push(Text::new(&meta.file_name).size(12))
                    .push_maybe(meta.parent_path.as_ref().map(|path| {
                        Text::new(path.clone())
                            .size(10)
                            .width(Length::Fill)
                            .wrapping(text::Wrapping::WordOrGlyph)
                            .color(theme::darcula::TEXT_SECONDARY)
                    }))
                    .push_maybe(meta.rename_hint.as_ref().map(|hint| {
                        Text::new(hint.clone())
                            .size(10)
                            .width(Length::Fill)
                            .wrapping(text::Wrapping::WordOrGlyph)
                            .color(theme::darcula::TEXT_SECONDARY)
                    })),
            )
            .push(widgets::compact_chip::<Message>(meta.status_label, meta.status_tone))
            .push(
                Text::new(format!("{} 区块 · +{} / -{}", hunks, additions, deletions))
                    .size(10)
                    .color(theme::darcula::TEXT_SECONDARY),
            ),
    )
    .padding([5, 8])
    .style(theme::panel_style(theme::Surface::ToolbarField))
    .into()
}
```

```rust
pub mod diff_file_header;
```

- [ ] **Step 4: Move diff summary into the compact header and remove the extra body summary strip in `src-ui/src/main.rs`, `src-ui/src/widgets/diff_viewer.rs`, and `src-ui/src/widgets/split_diff_viewer.rs`**

```rust
let total_hunks = state
    .current_diff
    .as_ref()
    .map(|diff| diff.files.iter().map(|file| file.hunks.len()).sum::<usize>());

Container::new(
    Row::new()
        .spacing(theme::spacing::XS)
        .align_y(Alignment::Center)
        .push(button::tab(title, true, None::<Message>))
        .push_maybe(path_hint.map(|hint| {
            Text::new(hint)
                .size(10)
                .color(theme::darcula::TEXT_SECONDARY)
        }))
        .push_maybe(state.current_diff.as_ref().map(|diff| {
            widgets::compact_chip::<Message>(format!("{} 文件", diff.files.len()), BadgeTone::Neutral)
        }))
        .push_maybe(total_hunks.map(|count| {
            widgets::compact_chip::<Message>(format!("{} 区块", count), BadgeTone::Accent)
        }))
        .push(Space::new().width(Length::Fill))
        .push(button::tab(
            "统一",
            state.diff_presentation == DiffPresentation::Unified,
            (state.show_diff
                && state.current_diff.is_some()
                && state.diff_presentation != DiffPresentation::Unified)
                .then_some(Message::ToggleDiffPresentation),
        ))
        .push(button::tab(
            "分栏",
            state.diff_presentation == DiffPresentation::Split,
            (state.show_diff
                && state.current_diff.is_some()
                && state.diff_presentation != DiffPresentation::Split)
                .then_some(Message::ToggleDiffPresentation),
        ))
        .push(button::compact_ghost("上个", Some(Message::NavigatePrevFile)))
        .push(button::compact_ghost("下个", Some(Message::NavigateNextFile))),
)
.padding(theme::density::SECONDARY_BAR_PADDING)
.style(theme::frame_style(theme::Surface::Toolbar))
```

```rust
pub fn view<Message: Clone + 'static>(&self) -> Element<'a, Message> {
    if self.diff.files.is_empty() {
        return widgets::panel_empty_state(
            "差异",
            "当前没有可显示的 diff",
            "切换文件、刷新状态，或比较其它提交后再查看这里。",
            None,
        );
    }

    let show_file_header = self.diff.files.len() > 1;
    let mut content = Column::new().spacing(theme::spacing::XS);

    for file_diff in &self.diff.files {
        content = content.push(render_file_diff(file_diff, show_file_header));
    }

    scrollable::styled(content).height(Length::Fill).into()
}
```

```rust
let meta = diff_file_header::DiffFileHeaderMeta::from_file_diff(file_diff);

Column::new()
    .spacing(theme::spacing::XS)
    .push(diff_file_header::view::<Message>(
        &meta,
        file_diff.hunks.len(),
        file_diff.additions,
        file_diff.deletions,
    ))
    .push(editor)
```

Apply the same shared header helper in both unified and split viewers so the mode switch does not bounce between two unrelated visual styles.

- [ ] **Step 5: Run the diff-header tests and crate build**

Run: `cargo test -p src-ui describe_paths_marks_renames -- --exact`
Expected: PASS

Run: `cargo test -p src-ui describe_paths_handles_new_files -- --exact`
Expected: PASS

Run: `cargo check -p src-ui`
Expected: PASS.

- [ ] **Step 6: Commit the compact diff workspace pass**

```bash
git add src-ui/src/widgets/diff_file_header.rs src-ui/src/widgets/mod.rs src-ui/src/widgets/diff_viewer.rs src-ui/src/widgets/split_diff_viewer.rs src-ui/src/main.rs
git commit -m "feat: compact workbench diff presentation"
```

### Task 6: Slim The Status Bar And Run The Homepage Consistency Sweep

**Files:**
- Modify: `src-ui/src/widgets/statusbar.rs:10-105`
- Modify: `src-ui/src/views/main_window.rs:126-174`
- Modify: `src-ui/src/main.rs:3243-3307`
- Test: `src-ui/src/widgets/statusbar.rs`

- [ ] **Step 1: Write the failing truncation tests in `src-ui/src/widgets/statusbar.rs`**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_middle_keeps_both_ends() {
        assert_eq!(truncate_middle("/Users/wanghao/git/slio-git", 18), "/Users/w…slio-git");
    }

    #[test]
    fn truncate_middle_leaves_short_strings_untouched() {
        assert_eq!(truncate_middle("src-ui", 18), "src-ui");
    }
}
```

- [ ] **Step 2: Run the focused status-bar test before removing the horizontal scroll wrappers**

Run: `cargo test -p src-ui truncate_middle_keeps_both_ends -- --exact`
Expected: FAIL with missing `truncate_middle()` helper.

- [ ] **Step 3: Replace scrollable status cells with truncated inline text in `src-ui/src/widgets/statusbar.rs`**

```rust
fn truncate_middle(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_string();
    }

    let head = (max_chars.saturating_sub(1)) / 2;
    let tail = max_chars.saturating_sub(head + 1);
    let prefix = value.chars().take(head).collect::<String>();
    let suffix = value
        .chars()
        .rev()
        .take(tail)
        .collect::<String>()
        .chars()
        .rev()
        .collect::<String>();

    format!("{prefix}…{suffix}")
}
```

```rust
let repo_text = truncate_middle(
    &self.repo_path.unwrap_or_else(|| self.i18n.no_repository.to_string()),
    32,
);
let selected_text = truncate_middle(
    &self
        .selected_path
        .map(|path| format!("{}: {path}", self.i18n.selected_file))
        .unwrap_or_else(|| format!("{}: 未选择", self.i18n.selected_file)),
    28,
);

let content = Row::new()
    .spacing(theme::spacing::XS)
    .align_y(Alignment::Center)
    .push(Text::new(repo_text).size(10).color(theme::darcula::TEXT_SECONDARY).width(Length::FillPortion(3)))
    .push(Text::new(selected_text).size(10).color(theme::darcula::TEXT_SECONDARY).width(Length::FillPortion(2)))
    .push(Space::new().width(Length::Fill))
    .push(
        Text::new(self.workspace_summary)
            .size(10)
            .color(theme::darcula::TEXT_SECONDARY),
    )
    .push(Self::separator())
    .push(Text::new(self.activity_label).size(10).color(Self::tone_color(self.activity_tone)))
    .push_maybe(self.detail.map(|detail| {
        Row::new()
            .spacing(theme::spacing::XS)
            .align_y(Alignment::Center)
            .push(Self::separator())
            .push(Text::new(truncate_middle(&detail, 28)).size(10).color(theme::darcula::TEXT_SECONDARY))
    }));
```

```rust
Container::new(content)
    .padding(theme::density::STATUS_PADDING)
    .width(Length::Fill)
    .style(theme::frame_style(Surface::Status))
```

- [ ] **Step 4: Run the final homepage density sweep in `src-ui/src/views/main_window.rs` and `src-ui/src/main.rs`**

```rust
let workspace = Row::new()
    .height(Length::Fill)
    .push(Self::navigation_rail(
        state,
        &on_open_repo,
        on_switch_project.as_ref(),
        &on_show_changes,
        &on_show_conflicts,
        &on_show_history,
        &on_show_remotes,
        &on_show_tags,
        &on_show_stashes,
        &on_show_rebase,
    ))
    .push(
        Column::new()
            .spacing(0)
            .width(Length::Fill)
            .height(Length::Fill)
            .push_maybe(banner)
            .push(
                Container::new(body)
                    .padding(theme::density::PANE_PADDING)
                    .width(Length::Fill)
                    .height(Length::Fill),
            ),
    );
```

```rust
let changes_panel = Container::new(
    Column::new()
        .spacing(0)
        .push(
            Container::new(
                Row::new()
                    .spacing(theme::spacing::XS)
                    .align_y(Alignment::Center)
                    .push(button::tab("提交", true, None::<Message>))
                    .push(button::tab("搁置", false, None::<Message>))
                    .push(button::tab("储藏", false, None::<Message>))
                    .push(Space::new().width(Length::Fill)),
            )
            .padding(theme::density::SECONDARY_BAR_PADDING)
            .style(theme::frame_style(theme::Surface::Toolbar)),
        )
        .push(iced::widget::rule::horizontal(1))
        .push(
            Container::new(
                Row::new()
                    .spacing(theme::spacing::XS)
                    .align_y(Alignment::Center)
                    .push(button::toolbar_icon("⟳", Some(Message::Refresh)))
                    .push(button::toolbar_icon("✓", can_stage_all.then_some(Message::StageAll)))
                    .push(button::toolbar_icon("↶", can_unstage_all.then_some(Message::UnstageAll)))
                    .push(Space::new().width(Length::Fill)),
            )
            .padding(theme::density::SECONDARY_BAR_PADDING)
            .style(theme::frame_style(theme::Surface::Nav)),
        )
        .push(iced::widget::rule::horizontal(1))
        .push(
            Container::new(changes_content)
                .padding(theme::density::PANE_PADDING)
                .height(Length::Fill),
        ),
)
    .width(Length::FillPortion(5))
    .style(theme::panel_style(theme::Surface::Panel));

let diff_panel = Container::new(
    Column::new()
        .spacing(0)
        .push(build_diff_header(state))
        .push(
            Container::new(build_diff_content(state, i18n))
                .padding([0, 0])
                .height(Length::Fill),
        ),
)
    .width(Length::FillPortion(8))
    .style(theme::panel_style(theme::Surface::Panel));

Row::new()
    .spacing(theme::spacing::XS)
    .height(Length::Fill)
    .push(changes_panel)
    .push(diff_panel)
```

The final sweep is done when all of the following are true in code:
- top chrome uses plain rows instead of horizontal scroll wrappers
- change list summary uses plain text or compact chips only
- diff viewers no longer render the extra explanatory summary strip
- status bar no longer uses horizontal scrollables for repo and selected-file labels
- left/right pane interior padding is `12`, not `16`

- [ ] **Step 5: Run the full UI verification set**

Run: `cargo test -p src-ui`
Expected: PASS

Run: `cargo check -p src-ui`
Expected: PASS

Run: `cargo run -p src-ui`
Expected: application launches.

Manual visual checks inside the running app:
- synced branch hides the sync chip (`✓` does not render as a badge)
- ahead / behind / diverged states still surface in the top chrome
- top chrome buttons all share the same vertical rhythm
- left rail inactive icons look quieter than the active icon
- the change list shows more rows in the same viewport than before
- the diff pane header is one compact line and the code area starts higher
- the status bar stays one line tall and does not show resting scrollbars

- [ ] **Step 6: Commit the final homepage consistency sweep**

```bash
git add src-ui/src/widgets/statusbar.rs src-ui/src/views/main_window.rs src-ui/src/main.rs
git commit -m "feat: finalize compact homepage workbench"
```

---

## Spec Coverage Check

- Spec §9.2 primary toolbar → Task 3 compacts repository/branch switchers, sync chip gating, and split-button alignment.
- Spec §9.3 secondary navigation strip → Task 3 removes horizontal scroll wrappers and normalizes tab height.
- Spec §9.4 left rail → Task 3 reduces rail padding and visual weight.
- Spec §9.5 left change list pane → Task 4 replaces chip-heavy summary, flattens rows, and compresses metadata.
- Spec §9.6 right diff pane header/canvas → Task 5 removes extra summary strips and aligns both diff modes to one compact header system.
- Spec §9.7 bottom status bar → Task 6 turns the status bar into a thin utility strip.
- Spec §10-11 visual tokens / buttons / chips / scroll containers / inputs → Tasks 1 and 2 create the shared density, surface, button, chip, and scrollbar rules consumed by the homepage.
- Spec §15-16 validation / acceptance criteria → Task 6 ends with full `src-ui` verification plus the manual visual checklist from the spec states.

## Placeholder Scan

- No deferred-work markers remain.
- Every code-changing step includes a concrete helper, function, enum, or layout block to add or replace.
- Every task includes exact file paths, commands, expected results, and a commit boundary.

## Execution Notes

- Keep each task self-contained; do not batch Tasks 1-6 into one mega-commit.
- If a manual visual check fails, fix it inside the current task before moving on.
- Do not widen the scope to history, conflict, remote, or branch panels in this pass unless a shared helper change automatically improves them without adding local one-off styling.
