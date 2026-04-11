//! Shared Yanqu design tokens and styling helpers.

#![allow(dead_code)]

use iced::widget::{button, checkbox, container, rule, scrollable, text_editor, text_input};
use iced::{border, Background, Border, Color, Shadow, Theme, Vector};

/// Historical module name kept so the rest of the UI can reuse existing imports.
/// Re-themed to MotionSites dark stage palette.
pub mod darcula {
    use super::*;

    // ── MotionSites dark stage backgrounds ───────────────────────────────────
    pub const BG_MAIN: Color = Color::from_rgb(0.035, 0.035, 0.043); // #09090b — stage
    pub const BG_SOFT: Color = Color::from_rgb(0.082, 0.082, 0.098); // #151519 — soft panel
    pub const BG_CARD: Color = Color::from_rgb(0.094, 0.094, 0.114); // #18181d — card
    pub const BG_CARD_2: Color = Color::from_rgb(0.133, 0.133, 0.165); // #22222a — raised
    pub const BG_TOOLBAR: Color = BG_SOFT;
    pub const BG_EDITOR: Color = BG_MAIN;
    pub const BG_NAV: Color = BG_SOFT;
    pub const BG_RAIL: Color = BG_SOFT;
    pub const BG_STATUS: Color = BG_SOFT;
    pub const BG_TAB_ACTIVE: Color = BG_CARD;
    pub const BG_TAB_HOVER: Color = BG_CARD_2;
    // Backward-compatible aliases for legacy code referencing Darcula names
    pub const BG_PANEL: Color = BG_CARD;
    pub const BG_RAISED: Color = BG_CARD_2;

    // ── Text hierarchy ─────────────────────────────────────────────────────────
    pub const TEXT_PRIMARY: Color = Color::from_rgb(0.961, 0.953, 0.937); // #f5f3ef
    pub const TEXT_SECONDARY: Color = Color::from_rgb(0.667, 0.651, 0.698); // #aaa6b2
    pub const TEXT_DISABLED: Color = Color::from_rgb(0.353, 0.337, 0.384); // #5a5662

    // ── Accent — MotionSites electric blue ───────────────────────────────────
    pub const ACCENT: Color = Color::from_rgb(0.255, 0.373, 1.0); // #415fff
    pub const ACCENT_WEAK: Color = Color::from_rgb(0.078, 0.118, 0.325); // #141e53 dark tint
    pub const BRAND: Color = ACCENT;
    pub const BRAND_WEAK: Color = ACCENT_WEAK;
    pub const SUCCESS: Color = Color::from_rgb(0.0, 0.686, 0.376); // #00af60
    pub const WARNING: Color = Color::from_rgb(0.996, 0.596, 0.0); // #fe9800
    pub const DANGER: Color = Color::from_rgb(1.0, 0.322, 0.322); // #ff5252

    // Sync indicators
    pub const INCOMING: Color = ACCENT;
    pub const OUTGOING: Color = SUCCESS;

    pub const STATUS_ADDED: Color = SUCCESS;
    pub const STATUS_MODIFIED: Color = BRAND;
    pub const STATUS_DELETED: Color = DANGER;
    pub const STATUS_RENAMED: Color = Color::from_rgb(0.369, 0.678, 0.831); // #5EACD0
    pub const STATUS_UNVERSIONED: Color = TEXT_SECONDARY;

    // ── Selection / highlight ────────────────────────────────────────────────
    pub const SELECTION_BG: Color = Color::from_rgb(0.12, 0.20, 0.45); // dark blue selection
    pub const SELECTION_INACTIVE: Color = BG_CARD_2;
    pub const HIGHLIGHT_BG: Color = Color::from_rgb(0.08, 0.12, 0.24);

    // ── Borders / separators ─────────────────────────────────────────────────
    pub const BORDER: Color = Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 0.08,
    };
    pub const SEPARATOR: Color = Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 0.05,
    };

    // ── Diff surfaces (dark stage compatible) ────────────────────────────────
    pub const DIFF_ADDED_BG: Color = Color::from_rgb(0.059, 0.165, 0.098);
    pub const DIFF_MODIFIED_BG: Color = Color::from_rgb(0.063, 0.106, 0.208);
    pub const DIFF_DELETED_BG: Color = Color::from_rgb(0.192, 0.055, 0.075);
}

pub mod spacing {
    pub const XS: f32 = 4.0;
    pub const SM: f32 = 8.0;
    pub const MD: f32 = 12.0;
    pub const LG: f32 = 16.0;
}

pub mod density {
    pub const INLINE_GAP: f32 = 5.0;
    pub const CONTROL_GAP: f32 = 4.0;
    pub const TOOLBAR_PADDING: [u16; 2] = [3, 8];
    pub const SECONDARY_BAR_PADDING: [u16; 2] = [3, 8];
    pub const PANE_PADDING: [u16; 2] = [8, 10];
    pub const STATUS_PADDING: [u16; 2] = [2, 8];
    pub const STANDARD_CONTROL_HEIGHT: f32 = 24.0;
    pub const COMPACT_CONTROL_HEIGHT: f32 = 22.0;
    pub const CHECKBOX_SIZE: f32 = 13.0;
    pub const CHECKBOX_SPACING: f32 = 6.0;
    pub const CHIP_HEIGHT: f32 = 18.0;
    pub const COMPACT_CHIP_PADDING: [u16; 2] = [1, 6];
    pub const TOOL_WINDOW_PADDING: [u16; 2] = [6, 8];
}

pub mod radius {
    /// Chips, badges.
    pub const SM: f32 = 6.0;
    /// Buttons, inputs, tabs.
    pub const MD: f32 = 8.0;
    /// Panels, cards, editors.
    pub const LG: f32 = 10.0;
}

/// Semantic text styles — MotionSites-aligned hierarchy.
pub mod typography {
    /// Display / hero text.
    pub const DISPLAY_SIZE: u32 = 14;
    pub const DISPLAY_WEIGHT: iced::font::Weight = iced::font::Weight::Bold;

    /// Title — section headers, dialog headings.
    pub const TITLE_SIZE: u32 = 13;
    pub const TITLE_WEIGHT: iced::font::Weight = iced::font::Weight::Semibold;

    /// Body — primary content, list items, descriptions.
    pub const BODY_SIZE: u32 = 12;
    pub const BODY_WEIGHT: iced::font::Weight = iced::font::Weight::Normal;

    /// Caption — secondary info, timestamps, metadata.
    pub const CAPTION_SIZE: u32 = 11;
    pub const CAPTION_WEIGHT: iced::font::Weight = iced::font::Weight::Normal;

    /// Micro — badges, chips, compact labels.
    pub const MICRO_SIZE: u32 = 10;
    pub const MICRO_WEIGHT: iced::font::Weight = iced::font::Weight::Normal;
}

/// Motion design tokens — transition durations and easing references.
pub mod motion {
    /// Fast micro-interactions: button press, checkbox toggle.
    pub const DURATION_FAST: u32 = 120; // ms
    /// Standard transitions: hover state, panel switch.
    pub const DURATION_NORMAL: u32 = 200; // ms
    /// Emphasized transitions: modal appear, view swap.
    pub const DURATION_EMPHASIZED: u32 = 350; // ms

    /// Standard easing curve (Material 3 equivalent).
    pub const EASING_STANDARD: &str = "cubic-bezier(0.4, 0.0, 0.2, 1)";
    /// Deceleration easing — elements entering screen.
    pub const EASING_DECEL: &str = "cubic-bezier(0.0, 0.0, 0.2, 1)";
    /// Acceleration easing — elements leaving screen.
    pub const EASING_ACCEL: &str = "cubic-bezier(0.4, 0.0, 1.0, 1.0)";
}

pub mod layout {
    pub const WINDOW_DEFAULT_WIDTH: f32 = 1280.0;
    pub const WINDOW_DEFAULT_HEIGHT: f32 = 800.0;
    pub const WINDOW_MIN_WIDTH: f32 = 800.0;
    pub const WINDOW_MIN_HEIGHT: f32 = 600.0;

    pub const SIDEBAR_WIDTH: f32 = 192.0;
    pub const RAIL_WIDTH: f32 = 48.0;
    pub const TOP_BAR_HEIGHT: f32 = 28.0;
    pub const SECONDARY_BAR_HEIGHT: f32 = 24.0;
    pub const STATUS_BAR_HEIGHT: f32 = 22.0;
    pub const CONTROL_HEIGHT: f32 = 24.0;
    pub const SHELL_GAP: f32 = 6.0;
    pub const SHELL_PADDING: f32 = 8.0;
    pub const SECTION_PADDING: f32 = 10.0;
    pub const EDITOR_TAB_HEIGHT: f32 = 28.0;
    pub const TOOL_WINDOW_HEIGHT: f32 = 290.0;
    pub const TOOL_WINDOW_MIN_HEIGHT: f32 = 220.0;
    pub const TOOL_WINDOW_TAB_HEIGHT: f32 = 24.0;
}

pub fn app_font() -> iced::Font {
    #[cfg(target_os = "macos")]
    {
        iced::Font::with_name("PingFang SC")
    }
    #[cfg(target_os = "windows")]
    {
        iced::Font::with_name("Microsoft YaHei")
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        iced::Font::with_name("Noto Sans CJK SC")
    }
}

pub fn code_font() -> iced::Font {
    app_font()
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonTone {
    Primary,
    Secondary,
    Ghost,
    TabActive,
    TabInactive,
    RailActive,
    RailInactive,
    Success,
    Warning,
    Danger,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonChrome {
    Standard,
    Tab,
    Rail,
    ToolbarIcon,
    SplitLeft,
    SplitRight,
}

#[derive(Debug, Clone, Copy)]
pub enum BadgeTone {
    Neutral,
    Accent,
    Success,
    Warning,
    Danger,
}

fn mix(base: Color, overlay: Color, amount: f32) -> Color {
    let amount = amount.clamp(0.0, 1.0);
    Color {
        r: (base.r * (1.0 - amount)) + (overlay.r * amount),
        g: (base.g * (1.0 - amount)) + (overlay.g * amount),
        b: (base.b * (1.0 - amount)) + (overlay.b * amount),
        a: (base.a * (1.0 - amount)) + (overlay.a * amount),
    }
}

fn soft_shadow(alpha: f32, y: f32, blur: f32) -> Shadow {
    Shadow {
        color: Color {
            a: alpha,
            ..Color::BLACK
        },
        offset: Vector::new(0.0, y),
        blur_radius: blur,
    }
}

fn accent_glow(color: Color, alpha: f32, blur: f32) -> Shadow {
    Shadow {
        color: Color { a: alpha, ..color },
        offset: Vector::new(0.0, 1.0),
        blur_radius: blur,
    }
}

fn surface_background(surface: Surface) -> Color {
    match surface {
        Surface::Root => darcula::BG_MAIN,
        Surface::Nav => darcula::BG_NAV,
        Surface::Rail => darcula::BG_RAIL,
        Surface::Toolbar => darcula::BG_TOOLBAR,
        Surface::Status => darcula::BG_STATUS,
        Surface::Panel => darcula::BG_CARD,
        Surface::Raised => darcula::BG_CARD_2,
        Surface::Editor => darcula::BG_EDITOR,
        Surface::Accent => mix(darcula::BG_CARD, darcula::ACCENT, 0.18),
        Surface::Selection => mix(darcula::BG_CARD, darcula::ACCENT, 0.30),
        Surface::Success => mix(darcula::BG_EDITOR, darcula::SUCCESS, 0.10),
        Surface::Warning => mix(darcula::BG_EDITOR, darcula::WARNING, 0.12),
        Surface::Danger => mix(darcula::BG_EDITOR, darcula::DANGER, 0.08),
        Surface::ToolbarField => darcula::BG_CARD_2,
        Surface::ListRow => darcula::BG_CARD,
        Surface::ListSelection => mix(darcula::BG_CARD, darcula::ACCENT, 0.22),
    }
}

/// Create the shared application theme.
pub fn darcula_theme() -> Theme {
    use iced::theme::Palette;

    Theme::custom(
        "MotionSitesStage".to_string(),
        Palette {
            background: darcula::BG_MAIN,
            text: darcula::TEXT_PRIMARY,
            primary: darcula::ACCENT,
            success: darcula::SUCCESS,
            warning: darcula::WARNING,
            danger: darcula::DANGER,
        },
    )
}

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
                darcula::BORDER,
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
                darcula::BG_CARD_2,
                1.0,
                darcula::BORDER,
                radius::MD,
                Shadow::default(),
            ),
            Surface::ListRow => (
                darcula::BG_CARD,
                1.0,
                darcula::SEPARATOR,
                radius::SM,
                Shadow::default(),
            ),
            Surface::ListSelection => (
                mix(darcula::BG_CARD, darcula::ACCENT, 0.18),
                1.0,
                mix(darcula::ACCENT, Color::WHITE, 0.10).scale_alpha(0.35),
                radius::SM,
                Shadow::default(),
            ),
            Surface::Panel => (
                surface_background(surface),
                1.0,
                darcula::BORDER,
                radius::LG,
                soft_shadow(0.14, 4.0, 12.0),
            ),
            Surface::Raised => (
                surface_background(surface),
                1.0,
                darcula::BORDER,
                radius::SM,
                soft_shadow(0.10, 2.0, 6.0),
            ),
            Surface::Accent => (
                surface_background(surface),
                1.0,
                darcula::ACCENT.scale_alpha(0.30),
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
                darcula::SUCCESS.scale_alpha(0.35),
                radius::LG,
                Shadow::default(),
            ),
            Surface::Warning => (
                surface_background(surface),
                1.0,
                darcula::WARNING.scale_alpha(0.35),
                radius::LG,
                Shadow::default(),
            ),
            Surface::Danger => (
                surface_background(surface),
                1.0,
                darcula::DANGER.scale_alpha(0.35),
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

pub fn frame_style(surface: Surface) -> impl Fn(&Theme) -> container::Style {
    move |_theme| {
        let (border_width, border_color, shadow) = match surface {
            Surface::Toolbar => (0.0, Color::TRANSPARENT, Shadow::default()),
            Surface::Nav => (1.0, darcula::BORDER, Shadow::default()),
            Surface::Rail => (1.0, darcula::BORDER, Shadow::default()),
            Surface::Status => (1.0, darcula::BORDER, Shadow::default()),
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

pub fn badge_style(tone: BadgeTone) -> impl Fn(&Theme) -> container::Style {
    move |_theme| {
        let (color, border_color) = match tone {
            BadgeTone::Neutral => (
                mix(darcula::BG_MAIN, darcula::BG_CARD, 0.70),
                darcula::BORDER,
            ),
            BadgeTone::Accent => (
                mix(darcula::BG_CARD, darcula::ACCENT_WEAK, 0.55),
                darcula::ACCENT.scale_alpha(0.25),
            ),
            BadgeTone::Success => (
                mix(darcula::BG_CARD, darcula::SUCCESS, 0.10),
                darcula::SUCCESS.scale_alpha(0.25),
            ),
            BadgeTone::Warning => (
                mix(darcula::BG_CARD, darcula::WARNING, 0.12),
                darcula::WARNING.scale_alpha(0.25),
            ),
            BadgeTone::Danger => (
                mix(darcula::BG_CARD, darcula::DANGER, 0.08),
                darcula::DANGER.scale_alpha(0.25),
            ),
        };

        container::Style {
            background: Some(Background::Color(color)),
            border: Border {
                width: 1.0,
                color: border_color,
                radius: radius::SM.into(),
            },
            ..Default::default()
        }
    }
}

pub fn button_style(tone: ButtonTone) -> impl Fn(&Theme, button::Status) -> button::Style {
    button_style_for(tone, ButtonChrome::Standard)
}

fn subtle_button(tone: ButtonTone) -> bool {
    matches!(
        tone,
        ButtonTone::Ghost | ButtonTone::TabInactive | ButtonTone::RailInactive
    )
}

fn button_radius(chrome: ButtonChrome) -> border::Radius {
    match chrome {
        ButtonChrome::Standard => border::Radius::new(radius::MD),
        ButtonChrome::Tab => border::Radius::new(0.0),
        ButtonChrome::Rail => border::Radius::new(13.0),
        ButtonChrome::ToolbarIcon => border::Radius::new(radius::MD),
        ButtonChrome::SplitLeft => border::Radius::default()
            .top_left(radius::MD)
            .bottom_left(radius::MD),
        ButtonChrome::SplitRight => border::Radius::default()
            .top_right(radius::MD)
            .bottom_right(radius::MD),
    }
}

fn button_border_width(tone: ButtonTone, chrome: ButtonChrome, status: button::Status) -> f32 {
    match status {
        button::Status::Active => {
            if subtle_button(tone) {
                match chrome {
                    ButtonChrome::Tab | ButtonChrome::ToolbarIcon => 1.0,
                    ButtonChrome::SplitLeft | ButtonChrome::SplitRight => 1.0,
                    _ => 0.0,
                }
            } else {
                1.0
            }
        }
        button::Status::Hovered | button::Status::Pressed => 1.0,
        button::Status::Disabled => match tone {
            ButtonTone::TabInactive if chrome == ButtonChrome::Tab => 1.0,
            ButtonTone::Ghost | ButtonTone::RailInactive => 0.0,
            ButtonTone::TabInactive => 0.0,
            _ => 1.0,
        },
    }
}

fn button_shadow(tone: ButtonTone, chrome: ButtonChrome, status: button::Status) -> Shadow {
    match status {
        button::Status::Active => match tone {
            ButtonTone::Primary => soft_shadow(0.20, 2.0, 8.0),
            ButtonTone::Secondary
            | ButtonTone::Success
            | ButtonTone::Warning
            | ButtonTone::Danger => soft_shadow(0.14, 2.0, 6.0),
            _ => Shadow::default(),
        },
        button::Status::Hovered => match tone {
            ButtonTone::Primary => accent_glow(darcula::ACCENT, 0.25, 12.0),
            ButtonTone::Secondary
            | ButtonTone::Success
            | ButtonTone::Warning
            | ButtonTone::Danger => soft_shadow(0.18, 2.0, 8.0),
            ButtonTone::Ghost
                if matches!(
                    chrome,
                    ButtonChrome::ToolbarIcon | ButtonChrome::SplitLeft | ButtonChrome::SplitRight
                ) =>
            {
                soft_shadow(0.10, 1.0, 4.0)
            }
            _ => Shadow::default(),
        },
        button::Status::Pressed => Shadow {
            offset: Vector::new(0.0, 0.0),
            blur_radius: 1.0,
            ..Shadow::default()
        },
        button::Status::Disabled => Shadow::default(),
    }
}

pub fn button_style_for(
    tone: ButtonTone,
    chrome: ButtonChrome,
) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_theme, status| {
        let passive_text = mix(darcula::TEXT_SECONDARY, darcula::TEXT_PRIMARY, 0.30);
        let (base_background, text_color, base_border) = match tone {
            ButtonTone::Primary => (darcula::ACCENT, Color::WHITE, darcula::ACCENT),
            ButtonTone::Secondary => (darcula::BG_CARD_2, darcula::TEXT_PRIMARY, darcula::BORDER),
            ButtonTone::Ghost => (Color::TRANSPARENT, passive_text, Color::TRANSPARENT),
            ButtonTone::TabActive => (
                mix(darcula::BG_CARD, darcula::ACCENT_WEAK, 0.45),
                darcula::TEXT_PRIMARY,
                darcula::ACCENT.scale_alpha(0.35),
            ),
            ButtonTone::TabInactive => (
                if chrome == ButtonChrome::Tab {
                    mix(darcula::BG_CARD, darcula::BG_MAIN, 0.30)
                } else {
                    Color::TRANSPARENT
                },
                passive_text,
                if chrome == ButtonChrome::Tab {
                    darcula::SEPARATOR
                } else {
                    Color::TRANSPARENT
                },
            ),
            ButtonTone::RailActive => (
                mix(darcula::BG_CARD, darcula::ACCENT_WEAK, 0.50),
                darcula::ACCENT,
                darcula::ACCENT.scale_alpha(0.35),
            ),
            ButtonTone::RailInactive => (Color::TRANSPARENT, passive_text, Color::TRANSPARENT),
            ButtonTone::Success => (darcula::SUCCESS, Color::WHITE, darcula::SUCCESS),
            ButtonTone::Warning => (darcula::WARNING, Color::BLACK, darcula::WARNING),
            ButtonTone::Danger => (darcula::DANGER, Color::WHITE, darcula::DANGER),
        };

        let (background, border_color, resolved_text) = match status {
            button::Status::Active => (base_background, base_border, text_color),
            button::Status::Hovered => (
                if subtle_button(tone) {
                    match chrome {
                        ButtonChrome::Tab => mix(darcula::BG_CARD, Color::WHITE, 0.05),
                        ButtonChrome::ToolbarIcon => mix(darcula::BG_CARD, Color::WHITE, 0.04),
                        ButtonChrome::SplitLeft | ButtonChrome::SplitRight => {
                            mix(darcula::BG_CARD, Color::WHITE, 0.04)
                        }
                        ButtonChrome::Rail => mix(darcula::BG_CARD_2, Color::WHITE, 0.05),
                        ButtonChrome::Standard => mix(darcula::BG_CARD_2, Color::WHITE, 0.04),
                    }
                } else {
                    mix(base_background, Color::WHITE, 0.10)
                },
                if subtle_button(tone) {
                    darcula::BORDER.scale_alpha(0.80)
                } else {
                    mix(base_border, Color::WHITE, 0.12)
                },
                if subtle_button(tone) {
                    darcula::TEXT_PRIMARY
                } else {
                    text_color
                },
            ),
            button::Status::Pressed => (
                if subtle_button(tone) {
                    match chrome {
                        ButtonChrome::Tab => mix(darcula::BG_CARD_2, Color::WHITE, 0.08),
                        ButtonChrome::ToolbarIcon => mix(darcula::BG_CARD_2, Color::WHITE, 0.06),
                        ButtonChrome::SplitLeft | ButtonChrome::SplitRight => {
                            mix(darcula::BG_CARD_2, Color::WHITE, 0.06)
                        }
                        ButtonChrome::Rail => mix(darcula::BG_CARD_2, Color::WHITE, 0.10),
                        ButtonChrome::Standard => mix(darcula::BG_CARD_2, Color::WHITE, 0.06),
                    }
                } else {
                    mix(base_background, Color::BLACK, 0.14)
                },
                if subtle_button(tone) {
                    darcula::BORDER.scale_alpha(1.0)
                } else {
                    mix(base_border, Color::WHITE, 0.08)
                },
                if subtle_button(tone) {
                    darcula::TEXT_PRIMARY
                } else {
                    text_color
                },
            ),
            button::Status::Disabled => match tone {
                ButtonTone::Ghost | ButtonTone::RailInactive => (
                    Color::TRANSPARENT,
                    Color::TRANSPARENT,
                    darcula::TEXT_DISABLED,
                ),
                ButtonTone::TabInactive => (
                    if chrome == ButtonChrome::Tab {
                        mix(darcula::BG_CARD, darcula::BG_MAIN, 0.25)
                    } else {
                        Color::TRANSPARENT
                    },
                    if chrome == ButtonChrome::Tab {
                        darcula::SEPARATOR.scale_alpha(0.60)
                    } else {
                        Color::TRANSPARENT
                    },
                    darcula::TEXT_DISABLED,
                ),
                ButtonTone::Primary => (
                    mix(darcula::BG_CARD, darcula::ACCENT_WEAK, 0.35),
                    darcula::BORDER.scale_alpha(0.35),
                    darcula::TEXT_DISABLED,
                ),
                ButtonTone::Secondary => (
                    darcula::BG_CARD,
                    darcula::BORDER.scale_alpha(0.35),
                    darcula::TEXT_DISABLED,
                ),
                ButtonTone::TabActive => (
                    mix(darcula::BG_CARD, darcula::ACCENT_WEAK, 0.30),
                    darcula::BORDER.scale_alpha(0.25),
                    darcula::TEXT_DISABLED,
                ),
                ButtonTone::RailActive => (
                    mix(darcula::BG_CARD, darcula::ACCENT_WEAK, 0.30),
                    darcula::BORDER.scale_alpha(0.25),
                    darcula::TEXT_DISABLED,
                ),
                ButtonTone::Success | ButtonTone::Warning | ButtonTone::Danger => (
                    mix(darcula::BG_CARD, base_background, 0.25),
                    mix(darcula::BORDER, base_border, 0.15),
                    darcula::TEXT_DISABLED,
                ),
            },
        };

        button::Style {
            background: Some(Background::Color(background)),
            border: Border {
                width: button_border_width(tone, chrome, status),
                color: border_color,
                radius: button_radius(chrome),
            },
            shadow: button_shadow(tone, chrome, status),
            text_color: resolved_text,
            ..Default::default()
        }
    }
}

pub fn text_input_style() -> impl Fn(&Theme, text_input::Status) -> text_input::Style {
    move |_theme, status| {
        let field_bg = darcula::BG_CARD_2;
        let (background, border, border_width, value, icon) = match status {
            text_input::Status::Active => (
                field_bg,
                darcula::BORDER,
                1.0,
                darcula::TEXT_PRIMARY,
                darcula::TEXT_SECONDARY,
            ),
            text_input::Status::Hovered => (
                mix(field_bg, Color::WHITE, 0.03),
                darcula::ACCENT.scale_alpha(0.45),
                1.0,
                darcula::TEXT_PRIMARY,
                darcula::TEXT_SECONDARY,
            ),
            text_input::Status::Focused { .. } => (
                mix(field_bg, Color::WHITE, 0.05),
                darcula::ACCENT,
                1.5,
                darcula::TEXT_PRIMARY,
                darcula::ACCENT,
            ),
            text_input::Status::Disabled => (
                darcula::BG_CARD,
                darcula::SEPARATOR.scale_alpha(0.60),
                1.0,
                darcula::TEXT_DISABLED,
                darcula::TEXT_DISABLED,
            ),
        };

        text_input::Style {
            background: Background::Color(background),
            border: Border {
                width: border_width,
                color: border,
                radius: radius::MD.into(),
            },
            icon,
            placeholder: darcula::TEXT_DISABLED,
            value,
            selection: darcula::SELECTION_BG,
        }
    }
}

pub fn text_editor_style() -> impl Fn(&Theme, text_editor::Status) -> text_editor::Style {
    move |_theme, status| {
        let field_bg = darcula::BG_CARD_2;
        let (background, border, value) = match status {
            text_editor::Status::Active => (field_bg, darcula::BORDER, darcula::TEXT_PRIMARY),
            text_editor::Status::Hovered => (
                mix(field_bg, Color::WHITE, 0.03),
                darcula::ACCENT.scale_alpha(0.45),
                darcula::TEXT_PRIMARY,
            ),
            text_editor::Status::Focused { .. } => (
                mix(field_bg, Color::WHITE, 0.05),
                darcula::ACCENT,
                darcula::TEXT_PRIMARY,
            ),
            text_editor::Status::Disabled => (
                darcula::BG_CARD,
                darcula::SEPARATOR.scale_alpha(0.60),
                darcula::TEXT_DISABLED,
            ),
        };

        text_editor::Style {
            background: Background::Color(background),
            border: Border {
                width: 1.0,
                color: border,
                radius: radius::MD.into(),
            },
            placeholder: darcula::TEXT_DISABLED,
            value,
            selection: darcula::SELECTION_BG,
        }
    }
}

pub fn scrollable_style() -> impl Fn(&Theme, scrollable::Status) -> scrollable::Style {
    move |_theme, status| {
        let idle_scroller = darcula::TEXT_SECONDARY.scale_alpha(0.25);
        let (rail_background, rail_border, scroller_color) = match status {
            scrollable::Status::Active { .. } => (
                Background::Color(Color::TRANSPARENT),
                Color::TRANSPARENT,
                idle_scroller,
            ),
            scrollable::Status::Hovered { .. } => (
                Background::Color(Color::TRANSPARENT),
                Color::TRANSPARENT,
                darcula::TEXT_SECONDARY.scale_alpha(0.38),
            ),
            scrollable::Status::Dragged { .. } => (
                Background::Color(Color::TRANSPARENT),
                Color::TRANSPARENT,
                darcula::TEXT_SECONDARY.scale_alpha(0.52),
            ),
        };

        scrollable::Style {
            container: container::Style::default(),
            vertical_rail: scrollable::Rail {
                background: Some(rail_background),
                border: Border {
                    width: 0.0,
                    color: rail_border,
                    radius: radius::SM.into(),
                },
                scroller: scrollable::Scroller {
                    background: Background::Color(scroller_color),
                    border: Border {
                        width: 0.0,
                        color: rail_border,
                        radius: radius::SM.into(),
                    },
                },
            },
            horizontal_rail: scrollable::Rail {
                background: Some(rail_background),
                border: Border {
                    width: 0.0,
                    color: rail_border,
                    radius: radius::SM.into(),
                },
                scroller: scrollable::Scroller {
                    background: Background::Color(scroller_color),
                    border: Border {
                        width: 0.0,
                        color: rail_border,
                        radius: radius::SM.into(),
                    },
                },
            },
            gap: None,
            auto_scroll: scrollable::AutoScroll {
                background: Background::Color(mix(darcula::BG_CARD_2, darcula::ACCENT_WEAK, 0.25)),
                border: Border {
                    width: 1.0,
                    color: darcula::BORDER,
                    radius: radius::MD.into(),
                },
                shadow: soft_shadow(0.14, 1.0, 4.0),
                icon: darcula::TEXT_SECONDARY,
            },
        }
    }
}

pub fn checkbox_style() -> impl Fn(&Theme, checkbox::Status) -> checkbox::Style {
    move |_theme, status| match status {
        checkbox::Status::Active { is_checked } => checkbox_base_style(is_checked, false, false),
        checkbox::Status::Hovered { is_checked } => checkbox_base_style(is_checked, true, false),
        checkbox::Status::Disabled { is_checked } => checkbox_base_style(is_checked, false, true),
    }
}

fn checkbox_base_style(is_checked: bool, hovered: bool, disabled: bool) -> checkbox::Style {
    let checked_bg = darcula::ACCENT;
    let background = if is_checked {
        if disabled {
            mix(checked_bg, darcula::BG_MAIN, 0.50)
        } else if hovered {
            mix(checked_bg, Color::WHITE, 0.12)
        } else {
            checked_bg
        }
    } else if disabled {
        mix(darcula::BG_CARD, darcula::BG_MAIN, 0.50)
    } else if hovered {
        mix(darcula::BG_CARD_2, Color::WHITE, 0.05)
    } else {
        darcula::BG_CARD_2
    };

    let border_color = if is_checked {
        if disabled {
            mix(checked_bg, darcula::BG_MAIN, 0.40)
        } else {
            darcula::ACCENT
        }
    } else {
        darcula::BORDER
    };

    checkbox::Style {
        background: Background::Color(background),
        icon_color: if disabled {
            Color::WHITE.scale_alpha(0.72)
        } else {
            Color::WHITE
        },
        border: Border {
            width: 1.0,
            color: border_color,
            radius: radius::SM.into(),
        },
        text_color: Some(if disabled {
            darcula::TEXT_DISABLED
        } else {
            darcula::TEXT_PRIMARY
        }),
    }
}

/// Get color for file status.
pub fn status_color(status: &str) -> Color {
    match status {
        "added" | "Added" | "Added_zh" => darcula::STATUS_ADDED,
        "modified" | "Modified" | "Modified_zh" => darcula::STATUS_MODIFIED,
        "deleted" | "Deleted" | "Deleted_zh" => darcula::STATUS_DELETED,
        "renamed" | "Renamed" | "Renamed_zh" => darcula::STATUS_RENAMED,
        "unversioned" | "Unversioned" | "Unversioned_zh" => darcula::STATUS_UNVERSIONED,
        _ => darcula::TEXT_SECONDARY,
    }
}

/// Themed horizontal separator using SEPARATOR color instead of default.
pub fn separator_rule_style() -> impl Fn(&Theme) -> rule::Style {
    move |_theme| rule::Style {
        color: darcula::SEPARATOR,
        radius: 0.0.into(),
        fill_mode: rule::FillMode::Full,
        snap: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn homepage_density_tokens_match_spec() {
        assert_eq!(density::TOOLBAR_PADDING, [3, 8]);
        assert_eq!(density::SECONDARY_BAR_PADDING, [3, 8]);
        assert_eq!(density::PANE_PADDING, [8, 10]);
        assert_eq!(density::STATUS_PADDING, [2, 8]);
        assert_eq!(density::STANDARD_CONTROL_HEIGHT, 24.0);
        assert_eq!(density::COMPACT_CONTROL_HEIGHT, 22.0);
        assert_eq!(density::COMPACT_CHIP_PADDING, [1, 6]);
    }

    #[test]
    fn homepage_compact_surfaces_are_available() {
        let theme = darcula_theme();
        let toolbar_field = panel_style(Surface::ToolbarField)(&theme);
        let list_row = panel_style(Surface::ListRow)(&theme);
        let _list_selection = panel_style(Surface::ListSelection)(&theme);

        assert_eq!(toolbar_field.shadow.blur_radius, 0.0);
        assert_eq!(list_row.shadow.blur_radius, 0.0);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn code_font_prefers_cjk_capable_family_on_macos() {
        assert_eq!(code_font(), iced::Font::with_name("PingFang SC"));
        assert_ne!(code_font(), iced::Font::MONOSPACE);
    }
}
