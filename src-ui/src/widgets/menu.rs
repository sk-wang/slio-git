//! Shared dense menu primitives for PhpStorm-style action surfaces.

use crate::theme::{self, BadgeTone};
// OptionalPush removed — no longer needed after compact menu refactor
use iced::widget::{button, container, Button, Column, Container, Row, Text};
use iced::{Alignment, Background, Border, Color, Element, Length, Theme, Vector};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuTone {
    Neutral,
    Accent,
    Danger,
}

pub fn blend_color(base: Color, overlay: Color, amount: f32) -> Color {
    let amount = amount.clamp(0.0, 1.0);
    Color {
        r: (base.r * (1.0 - amount)) + (overlay.r * amount),
        g: (base.g * (1.0 - amount)) + (overlay.g * amount),
        b: (base.b * (1.0 - amount)) + (overlay.b * amount),
        a: (base.a * (1.0 - amount)) + (overlay.a * amount),
    }
}

pub fn panel_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(blend_color(
            theme::darcula::BG_PANEL,
            theme::darcula::BG_RAISED,
            0.94,
        ))),
        border: Border {
            width: 1.0,
            color: theme::darcula::SEPARATOR.scale_alpha(0.92),
            radius: theme::radius::SM.into(),
        },
        shadow: iced::Shadow {
            color: Color {
                a: 0.18,
                ..theme::darcula::BG_MAIN
            },
            offset: Vector::new(0.0, 4.0),
            blur_radius: 8.0,
        },
        ..Default::default()
    }
}

pub fn scrim_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.5))),
        border: Border::default(),
        ..Default::default()
    }
}

pub fn group<'a, Message: 'a>(
    title: impl Into<String>,
    detail: impl Into<String>,
    tone: MenuTone,
    rows: Vec<Element<'a, Message>>,
) -> Element<'a, Message> {
    let title_str: String = title.into();
    let detail_str: String = detail.into();
    let rows = rows
        .into_iter()
        .fold(Column::new().spacing(0), |column, row| column.push(row));

    // IDEA-style: if title and detail are empty, render separator-only group
    let mut content = Column::new().spacing(2);

    if !title_str.is_empty() {
        let mut header = Column::new().spacing(2);
        header = header.push(Text::new(title_str).size(10).color(group_title_color(tone)));
        if !detail_str.is_empty() {
            header = header.push(
                Text::new(detail_str)
                    .size(10)
                    .width(Length::Fill)
                    .wrapping(iced::widget::text::Wrapping::WordOrGlyph)
                    .color(theme::darcula::TEXT_SECONDARY),
            );
        }
        content = content.push(header);
    }

    content = content.push(rows);

    Container::new(content)
        .padding([4, 8])
        .style(group_style(tone))
        .into()
}

pub fn action_row<'a, Message: Clone + 'a>(
    icon: Option<&'static str>,
    title: impl Into<String>,
    _detail: Option<String>,
    badge: Option<(String, BadgeTone)>,
    on_press: Option<Message>,
    tone: MenuTone,
) -> Element<'a, Message> {
    let enabled = on_press.is_some();
    let title_color = if enabled {
        match tone {
            MenuTone::Danger => {
                blend_color(theme::darcula::TEXT_PRIMARY, theme::darcula::DANGER, 0.16)
            }
            _ => theme::darcula::TEXT_PRIMARY,
        }
    } else {
        theme::darcula::TEXT_DISABLED
    };

    // IDEA-style: no leading icon placeholder, no trailing arrow — compact flat list
    let mut row = Row::new()
        .spacing(theme::spacing::XS)
        .align_y(Alignment::Center);

    if let Some(icon) = icon {
        row = row.push(icon_badge(icon, tone, enabled));
    }

    row = row.push(
        Text::new(title.into())
            .size(12)
            .width(Length::Fill)
            .color(title_color),
    );

    if let Some((label, badge_tone)) = badge {
        row = row.push(crate::widgets::compact_chip::<Message>(label, badge_tone));
    }

    let button = Button::new(Container::new(row).padding([4, 8]).width(Length::Fill))
        .width(Length::Fill)
        .style(action_button_style(tone, enabled));

    if let Some(message) = on_press {
        button.on_press(message).into()
    } else {
        button.into()
    }
}

pub fn trigger_row_button_style(
    is_selected: bool,
    is_menu_open: bool,
    accent: Option<Color>,
) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_theme, status| {
        let accent = accent.unwrap_or(theme::darcula::ACCENT);
        let (base_background, base_border) = if is_menu_open {
            (
                blend_color(theme::darcula::BG_PANEL, accent, 0.20),
                accent.scale_alpha(0.72),
            )
        } else if is_selected {
            (
                blend_color(theme::darcula::BG_PANEL, accent, 0.12),
                accent.scale_alpha(0.24),
            )
        } else {
            (Color::TRANSPARENT, Color::TRANSPARENT)
        };

        let (background, border_color) = match status {
            button::Status::Active => (base_background, base_border),
            button::Status::Hovered => (
                if is_menu_open || is_selected {
                    blend_color(base_background, Color::WHITE, 0.05)
                } else {
                    blend_color(theme::darcula::BG_PANEL, theme::darcula::BG_RAISED, 0.58)
                },
                if is_menu_open || is_selected {
                    blend_color(base_border, Color::WHITE, 0.08)
                } else {
                    theme::darcula::SEPARATOR.scale_alpha(0.64)
                },
            ),
            button::Status::Pressed => (
                if is_menu_open || is_selected {
                    blend_color(base_background, theme::darcula::BG_MAIN, 0.10)
                } else {
                    blend_color(theme::darcula::BG_PANEL, theme::darcula::BG_RAISED, 0.78)
                },
                if is_menu_open || is_selected {
                    blend_color(base_border, theme::darcula::BG_MAIN, 0.10)
                } else {
                    accent.scale_alpha(0.28)
                },
            ),
            button::Status::Disabled => (
                blend_color(theme::darcula::BG_PANEL, base_background, 0.24),
                blend_color(theme::darcula::BORDER, base_border, 0.22),
            ),
        };

        button::Style {
            background: Some(Background::Color(background)),
            border: Border {
                width: 1.0,
                color: border_color,
                radius: theme::radius::SM.into(),
            },
            text_color: if matches!(status, button::Status::Disabled) {
                theme::darcula::TEXT_DISABLED
            } else {
                theme::darcula::TEXT_PRIMARY
            },
            ..Default::default()
        }
    }
}

fn group_style(tone: MenuTone) -> impl Fn(&Theme) -> container::Style {
    move |_theme| {
        let (background, border_color) = match tone {
            MenuTone::Neutral => (
                blend_color(theme::darcula::BG_PANEL, theme::darcula::BG_RAISED, 0.76),
                theme::darcula::BORDER.scale_alpha(0.74),
            ),
            MenuTone::Accent => (
                blend_color(theme::darcula::BG_PANEL, theme::darcula::ACCENT_WEAK, 0.70),
                theme::darcula::ACCENT.scale_alpha(0.20),
            ),
            MenuTone::Danger => (
                blend_color(theme::darcula::BG_PANEL, theme::darcula::DANGER, 0.10),
                theme::darcula::DANGER.scale_alpha(0.24),
            ),
        };

        container::Style {
            background: Some(Background::Color(background)),
            border: Border {
                width: 1.0,
                color: border_color,
                radius: theme::radius::SM.into(),
            },
            ..Default::default()
        }
    }
}

fn action_button_style(
    tone: MenuTone,
    enabled: bool,
) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_theme, status| {
        let interaction_color = match tone {
            MenuTone::Neutral => theme::darcula::BG_TAB_HOVER,
            MenuTone::Accent => theme::darcula::ACCENT,
            MenuTone::Danger => theme::darcula::DANGER,
        };

        let (background, border_color) = if enabled {
            match status {
                button::Status::Active => (
                    blend_color(theme::darcula::BG_PANEL, interaction_color, 0.07),
                    interaction_color.scale_alpha(0.14),
                ),
                button::Status::Hovered => (
                    blend_color(theme::darcula::BG_PANEL, interaction_color, 0.18),
                    interaction_color.scale_alpha(0.30),
                ),
                button::Status::Pressed => (
                    blend_color(theme::darcula::BG_PANEL, interaction_color, 0.26),
                    interaction_color.scale_alpha(0.40),
                ),
                button::Status::Disabled => (
                    blend_color(theme::darcula::BG_PANEL, theme::darcula::BG_MAIN, 0.22),
                    theme::darcula::SEPARATOR.scale_alpha(0.26),
                ),
            }
        } else {
            (
                blend_color(theme::darcula::BG_PANEL, theme::darcula::BG_MAIN, 0.18),
                theme::darcula::SEPARATOR.scale_alpha(0.22),
            )
        };

        button::Style {
            background: Some(Background::Color(background)),
            border: Border {
                width: 1.0,
                color: border_color,
                radius: theme::radius::SM.into(),
            },
            text_color: if enabled {
                theme::darcula::TEXT_PRIMARY
            } else {
                theme::darcula::TEXT_DISABLED
            },
            ..Default::default()
        }
    }
}

fn icon_badge<'a, Message: 'a>(
    icon: &'static str,
    tone: MenuTone,
    enabled: bool,
) -> Element<'a, Message> {
    let badge_tone = if enabled {
        match tone {
            MenuTone::Neutral => BadgeTone::Neutral,
            MenuTone::Accent => BadgeTone::Accent,
            MenuTone::Danger => BadgeTone::Danger,
        }
    } else {
        BadgeTone::Neutral
    };

    Container::new(
        Text::new(icon)
            .size(if icon.len() > 1 { 8 } else { 10 })
            .color(if enabled {
                theme::darcula::TEXT_PRIMARY
            } else {
                theme::darcula::TEXT_DISABLED
            }),
    )
    .width(Length::Fixed(24.0))
    .height(Length::Fixed(18.0))
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .style(theme::badge_style(badge_tone))
    .into()
}

fn group_title_color(tone: MenuTone) -> Color {
    match tone {
        MenuTone::Neutral => theme::darcula::TEXT_SECONDARY,
        MenuTone::Accent => theme::darcula::ACCENT,
        MenuTone::Danger => blend_color(theme::darcula::WARNING, theme::darcula::DANGER, 0.48),
    }
}
