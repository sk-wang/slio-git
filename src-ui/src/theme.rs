//! Shared Yanqu design tokens and styling helpers.

#![allow(dead_code)]

use iced::widget::{button, checkbox, container, scrollable, text_editor, text_input};
use iced::{Background, Border, Color, Shadow, Theme, Vector};

/// Historical module name kept so the rest of the UI can reuse existing imports.
pub mod darcula {
    use super::*;

    pub const BG_MAIN: Color = Color::from_rgb(0.969, 0.973, 0.980); // #F7F8FA
    pub const BG_RAISED: Color = Color::from_rgb(1.0, 1.0, 1.0); // #FFFFFF
    pub const BG_PANEL: Color = Color::from_rgb(1.0, 1.0, 1.0); // #FFFFFF
    pub const BG_TOOLBAR: Color = Color::from_rgb(1.0, 1.0, 1.0); // #FFFFFF
    pub const BG_EDITOR: Color = Color::from_rgb(0.996, 0.998, 0.998); // #FEFEFE
    pub const BG_NAV: Color = Color::from_rgb(0.949, 0.969, 0.965); // #F2F7F6
    pub const BG_RAIL: Color = Color::from_rgb(1.0, 1.0, 1.0); // #FFFFFF
    pub const BG_STATUS: Color = Color::from_rgb(1.0, 1.0, 1.0); // #FFFFFF
    pub const BG_TAB_ACTIVE: Color = Color::from_rgb(0.918, 0.973, 0.957); // #EAF8F4
    pub const BG_TAB_HOVER: Color = Color::from_rgb(0.953, 0.973, 0.969); // #F3F8F7

    pub const TEXT_PRIMARY: Color = Color::from_rgb(0.149, 0.149, 0.149); // #262626
    pub const TEXT_SECONDARY: Color = Color::from_rgb(0.349, 0.349, 0.349); // #595959
    pub const TEXT_DISABLED: Color = Color::from_rgb(0.549, 0.549, 0.549); // #8C8C8C

    pub const ACCENT: Color = Color::from_rgb(0.337, 0.745, 0.698); // #56BEB2
    pub const ACCENT_WEAK: Color = Color::from_rgb(0.906, 0.973, 0.957); // #E7F8F4
    pub const BRAND: Color = Color::from_rgb(0.176, 0.427, 0.992); // #2D6DFD
    pub const BRAND_WEAK: Color = Color::from_rgb(0.918, 0.945, 1.0); // #EAF1FF
    pub const SUCCESS: Color = Color::from_rgb(0.094, 0.718, 0.600); // #18B799
    pub const WARNING: Color = Color::from_rgb(0.875, 0.643, 0.255); // #DFA441
    pub const DANGER: Color = Color::from_rgb(1.0, 0.302, 0.310); // #FF4D4F

    pub const STATUS_ADDED: Color = SUCCESS;
    pub const STATUS_MODIFIED: Color = BRAND;
    pub const STATUS_DELETED: Color = DANGER;
    pub const STATUS_RENAMED: Color = Color::from_rgb(0.153, 0.718, 0.667); // #27B7AA
    pub const STATUS_UNVERSIONED: Color = Color::from_rgb(0.627, 0.627, 0.627); // #A0A0A0

    pub const SELECTION_BG: Color = Color::from_rgb(0.878, 0.961, 0.945); // #E0F5F1
    pub const SELECTION_INACTIVE: Color = Color::from_rgb(0.941, 0.965, 0.961); // #F0F6F5
    pub const HIGHLIGHT_BG: Color = Color::from_rgb(0.918, 0.945, 1.0); // #EAF1FF

    pub const BORDER: Color = Color::from_rgb(0.910, 0.910, 0.910); // #E8E8E8
    pub const SEPARATOR: Color = Color::from_rgb(0.890, 0.910, 0.910); // #E3E8E8

    pub const DIFF_ADDED_BG: Color = Color::from_rgb(0.918, 0.973, 0.957); // #EAF8F4
    pub const DIFF_MODIFIED_BG: Color = Color::from_rgb(0.933, 0.957, 1.0); // #EEF4FF
    pub const DIFF_DELETED_BG: Color = Color::from_rgb(1.0, 0.945, 0.945); // #FFF1F1
}

pub mod spacing {
    pub const XS: f32 = 4.0;
    pub const SM: f32 = 8.0;
    pub const MD: f32 = 16.0;
    pub const LG: f32 = 24.0;
}

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

pub mod radius {
    pub const SM: f32 = 4.0;
    pub const MD: f32 = 8.0;
    pub const LG: f32 = 12.0;
}

pub mod layout {
    pub const WINDOW_DEFAULT_WIDTH: f32 = 1280.0;
    pub const WINDOW_DEFAULT_HEIGHT: f32 = 800.0;
    pub const WINDOW_MIN_WIDTH: f32 = 800.0;
    pub const WINDOW_MIN_HEIGHT: f32 = 600.0;

    pub const SIDEBAR_WIDTH: f32 = 192.0;
    pub const RAIL_WIDTH: f32 = 56.0;
    pub const TOP_BAR_HEIGHT: f32 = 30.0;
    pub const SECONDARY_BAR_HEIGHT: f32 = 28.0;
    pub const STATUS_BAR_HEIGHT: f32 = 24.0;
    pub const CONTROL_HEIGHT: f32 = 26.0;
    pub const SHELL_GAP: f32 = 8.0;
    pub const SHELL_PADDING: f32 = 16.0;
    pub const SECTION_PADDING: f32 = 16.0;
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

fn surface_background(surface: Surface) -> Color {
    match surface {
        Surface::Root => darcula::BG_MAIN,
        Surface::Nav => mix(darcula::BG_MAIN, darcula::ACCENT_WEAK, 0.35),
        Surface::Rail => darcula::BG_RAIL,
        Surface::Toolbar => darcula::BG_TOOLBAR,
        Surface::Status => darcula::BG_STATUS,
        Surface::Panel => darcula::BG_PANEL,
        Surface::Raised => darcula::BG_RAISED,
        Surface::Editor => darcula::BG_EDITOR,
        Surface::Accent => mix(darcula::BG_PANEL, darcula::ACCENT_WEAK, 0.94),
        Surface::Selection => mix(darcula::BG_PANEL, darcula::SELECTION_BG, 0.98),
        Surface::Success => mix(darcula::BG_PANEL, darcula::SUCCESS, 0.12),
        Surface::Warning => mix(darcula::BG_PANEL, darcula::WARNING, 0.16),
        Surface::Danger => mix(darcula::BG_PANEL, darcula::DANGER, 0.10),
        Surface::ToolbarField => mix(darcula::BG_PANEL, darcula::ACCENT_WEAK, 0.18),
        Surface::ListRow => darcula::BG_PANEL,
        Surface::ListSelection => mix(darcula::BG_PANEL, darcula::SELECTION_BG, 0.92),
    }
}

/// Create the shared application theme.
pub fn darcula_theme() -> Theme {
    use iced::theme::Palette;

    Theme::custom(
        "YanquWorkbench".to_string(),
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
                darcula::SEPARATOR.scale_alpha(0.20),
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

pub fn badge_style(tone: BadgeTone) -> impl Fn(&Theme) -> container::Style {
    move |_theme| {
        let (color, border_color) = match tone {
            BadgeTone::Neutral => (
                mix(darcula::BG_MAIN, darcula::BG_PANEL, 0.70),
                darcula::BORDER.scale_alpha(0.92),
            ),
            BadgeTone::Accent => (
                mix(darcula::BG_PANEL, darcula::ACCENT_WEAK, 0.96),
                darcula::ACCENT.scale_alpha(0.24),
            ),
            BadgeTone::Success => (
                mix(darcula::BG_PANEL, darcula::SUCCESS, 0.12),
                darcula::SUCCESS.scale_alpha(0.24),
            ),
            BadgeTone::Warning => (
                mix(darcula::BG_PANEL, darcula::WARNING, 0.16),
                darcula::WARNING.scale_alpha(0.26),
            ),
            BadgeTone::Danger => (
                mix(darcula::BG_PANEL, darcula::DANGER, 0.10),
                darcula::DANGER.scale_alpha(0.24),
            ),
        };

        container::Style {
            background: Some(Background::Color(color)),
            border: Border {
                width: 1.0,
                color: border_color,
                radius: radius::MD.into(),
            },
            ..Default::default()
        }
    }
}

pub fn button_style(tone: ButtonTone) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_theme, status| {
        let passive_text = mix(darcula::TEXT_SECONDARY, darcula::TEXT_PRIMARY, 0.24);
        let (base_background, text_color, base_border) = match tone {
            ButtonTone::Primary => (darcula::ACCENT, Color::WHITE, darcula::ACCENT),
            ButtonTone::Secondary => (
                darcula::BG_PANEL,
                darcula::TEXT_PRIMARY,
                darcula::BORDER.scale_alpha(0.92),
            ),
            ButtonTone::Ghost => (Color::TRANSPARENT, passive_text, Color::TRANSPARENT),
            ButtonTone::TabActive => (
                mix(darcula::BG_PANEL, darcula::ACCENT_WEAK, 0.96),
                darcula::TEXT_PRIMARY,
                darcula::ACCENT.scale_alpha(0.26),
            ),
            ButtonTone::TabInactive => (Color::TRANSPARENT, passive_text, Color::TRANSPARENT),
            ButtonTone::RailActive => (
                mix(darcula::BG_PANEL, darcula::ACCENT_WEAK, 0.98),
                darcula::ACCENT,
                darcula::ACCENT.scale_alpha(0.22),
            ),
            ButtonTone::RailInactive => (Color::TRANSPARENT, passive_text, Color::TRANSPARENT),
            ButtonTone::Success => (darcula::SUCCESS, Color::WHITE, darcula::SUCCESS),
            ButtonTone::Warning => (darcula::WARNING, Color::WHITE, darcula::WARNING),
            ButtonTone::Danger => (darcula::DANGER, Color::WHITE, darcula::DANGER),
        };

        let (background, border_color, resolved_text) = match status {
            button::Status::Active => (base_background, base_border, text_color),
            button::Status::Hovered => (
                if matches!(tone, ButtonTone::Ghost | ButtonTone::TabInactive) {
                    mix(darcula::BG_TAB_HOVER, darcula::ACCENT_WEAK, 0.28)
                } else if tone == ButtonTone::RailInactive {
                    mix(darcula::BG_PANEL, darcula::ACCENT_WEAK, 0.54)
                } else {
                    mix(base_background, Color::BLACK, 0.04)
                },
                if matches!(
                    tone,
                    ButtonTone::Ghost | ButtonTone::TabInactive | ButtonTone::RailInactive
                ) {
                    darcula::ACCENT.scale_alpha(0.22)
                } else {
                    mix(base_border, Color::BLACK, 0.06)
                },
                if matches!(
                    tone,
                    ButtonTone::Ghost | ButtonTone::TabInactive | ButtonTone::RailInactive
                ) {
                    mix(darcula::TEXT_PRIMARY, darcula::ACCENT, 0.40)
                } else {
                    text_color
                },
            ),
            button::Status::Pressed => (
                if matches!(tone, ButtonTone::Ghost | ButtonTone::TabInactive) {
                    mix(darcula::BG_MAIN, darcula::ACCENT_WEAK, 0.60)
                } else if tone == ButtonTone::RailInactive {
                    mix(darcula::BG_PANEL, darcula::ACCENT_WEAK, 0.80)
                } else {
                    mix(base_background, Color::BLACK, 0.10)
                },
                if matches!(
                    tone,
                    ButtonTone::Ghost | ButtonTone::TabInactive | ButtonTone::RailInactive
                ) {
                    darcula::ACCENT
                } else {
                    mix(base_border, Color::BLACK, 0.12)
                },
                if matches!(
                    tone,
                    ButtonTone::Ghost | ButtonTone::TabInactive | ButtonTone::RailInactive
                ) {
                    darcula::ACCENT
                } else {
                    text_color
                },
            ),
            button::Status::Disabled => (
                mix(darcula::BG_MAIN, base_background, 0.32),
                mix(darcula::BORDER, base_border, 0.40),
                darcula::TEXT_DISABLED,
            ),
        };

        button::Style {
            background: Some(Background::Color(background)),
            border: Border {
                width: if matches!(
                    tone,
                    ButtonTone::Ghost | ButtonTone::TabInactive | ButtonTone::RailInactive
                ) && matches!(status, button::Status::Active)
                {
                    0.0
                } else {
                    1.0
                },
                color: border_color,
                radius: radius::LG.into(),
            },
            text_color: resolved_text,
            ..Default::default()
        }
    }
}

pub fn text_input_style() -> impl Fn(&Theme, text_input::Status) -> text_input::Style {
    move |_theme, status| {
        let (background, border, value, icon) = match status {
            text_input::Status::Active => (
                mix(darcula::BG_PANEL, darcula::ACCENT_WEAK, 0.18),
                darcula::BORDER.scale_alpha(0.92),
                darcula::TEXT_PRIMARY,
                darcula::TEXT_SECONDARY,
            ),
            text_input::Status::Hovered => (
                mix(darcula::BG_PANEL, darcula::ACCENT_WEAK, 0.28),
                darcula::ACCENT.scale_alpha(0.22),
                darcula::TEXT_PRIMARY,
                darcula::TEXT_SECONDARY,
            ),
            text_input::Status::Focused { .. } => (
                mix(darcula::BG_PANEL, darcula::ACCENT_WEAK, 0.34),
                darcula::ACCENT,
                darcula::TEXT_PRIMARY,
                darcula::ACCENT,
            ),
            text_input::Status::Disabled => (
                mix(darcula::BG_PANEL, darcula::BG_MAIN, 0.72),
                mix(darcula::BORDER, darcula::BG_MAIN, 0.40),
                darcula::TEXT_DISABLED,
                darcula::TEXT_DISABLED,
            ),
        };

        text_input::Style {
            background: Background::Color(background),
            border: Border {
                width: 1.0,
                color: border,
                radius: radius::LG.into(),
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
        let (background, border, value) = match status {
            text_editor::Status::Active => (
                mix(darcula::BG_PANEL, darcula::ACCENT_WEAK, 0.16),
                darcula::BORDER.scale_alpha(0.92),
                darcula::TEXT_PRIMARY,
            ),
            text_editor::Status::Hovered => (
                mix(darcula::BG_PANEL, darcula::ACCENT_WEAK, 0.24),
                darcula::ACCENT.scale_alpha(0.22),
                darcula::TEXT_PRIMARY,
            ),
            text_editor::Status::Focused { .. } => (
                mix(darcula::BG_PANEL, darcula::ACCENT_WEAK, 0.30),
                darcula::ACCENT,
                darcula::TEXT_PRIMARY,
            ),
            text_editor::Status::Disabled => (
                mix(darcula::BG_PANEL, darcula::BG_MAIN, 0.72),
                mix(darcula::BORDER, darcula::BG_MAIN, 0.40),
                darcula::TEXT_DISABLED,
            ),
        };

        text_editor::Style {
            background: Background::Color(background),
            border: Border {
                width: 1.0,
                color: border,
                radius: radius::LG.into(),
            },
            placeholder: darcula::TEXT_DISABLED,
            value,
            selection: darcula::SELECTION_BG,
        }
    }
}

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
                mix(darcula::TEXT_DISABLED, darcula::ACCENT, 0.16).scale_alpha(0.42),
            ),
            scrollable::Status::Dragged { .. } => (
                Background::Color(Color::TRANSPARENT),
                Color::TRANSPARENT,
                darcula::ACCENT.scale_alpha(0.72),
            ),
        };

        scrollable::Style {
            container: container::Style::default(),
            vertical_rail: scrollable::Rail {
                background: Some(rail_background),
                border: Border {
                    width: 0.0,
                    color: rail_border,
                    radius: radius::LG.into(),
                },
                scroller: scrollable::Scroller {
                    background: Background::Color(scroller_color),
                    border: Border {
                        width: 0.0,
                        color: rail_border,
                        radius: radius::LG.into(),
                    },
                },
            },
            horizontal_rail: scrollable::Rail {
                background: Some(rail_background),
                border: Border {
                    width: 0.0,
                    color: rail_border,
                    radius: radius::LG.into(),
                },
                scroller: scrollable::Scroller {
                    background: Background::Color(scroller_color),
                    border: Border {
                        width: 0.0,
                        color: rail_border,
                        radius: radius::LG.into(),
                    },
                },
            },
            gap: None,
            auto_scroll: scrollable::AutoScroll {
                background: Background::Color(mix(darcula::BG_MAIN, darcula::ACCENT_WEAK, 0.62)),
                border: Border {
                    width: 1.0,
                    color: darcula::ACCENT.scale_alpha(0.18),
                    radius: radius::LG.into(),
                },
                shadow: soft_shadow(0.04, 4.0, 12.0),
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
    let background = if is_checked {
        if disabled {
            mix(darcula::ACCENT, darcula::BG_MAIN, 0.42)
        } else if hovered {
            mix(darcula::ACCENT, Color::WHITE, 0.08)
        } else {
            darcula::ACCENT
        }
    } else if disabled {
        mix(darcula::BG_PANEL, darcula::BG_MAIN, 0.44)
    } else if hovered {
        mix(darcula::BG_PANEL, darcula::ACCENT_WEAK, 0.52)
    } else {
        darcula::BG_PANEL
    };

    let border_color = if is_checked {
        if disabled {
            mix(darcula::ACCENT, darcula::BG_MAIN, 0.40)
        } else {
            darcula::ACCENT
        }
    } else if hovered {
        darcula::ACCENT.scale_alpha(0.54)
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
        "added" | "Added" | "新增" => darcula::STATUS_ADDED,
        "modified" | "Modified" | "已修改" => darcula::STATUS_MODIFIED,
        "deleted" | "Deleted" | "已删除" => darcula::STATUS_DELETED,
        "renamed" | "Renamed" | "已重命名" => darcula::STATUS_RENAMED,
        "unversioned" | "Unversioned" | "未版本控制" => darcula::STATUS_UNVERSIONED,
        _ => darcula::TEXT_SECONDARY,
    }
}

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
