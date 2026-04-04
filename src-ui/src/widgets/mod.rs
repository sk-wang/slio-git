//! UI widgets module and shared shell primitives.

pub mod button;
pub mod changelist;
pub mod commit_compare;
pub mod commit_panel;
pub mod conflict_resolver;
pub mod diff_file_header;
pub mod diff_viewer;
pub mod file_picker;
pub mod log_tabs;
pub mod menu;
pub mod progress_bar;
pub mod scrollable;
pub mod split_diff_viewer;
pub mod statusbar;
pub mod syntax_highlighting;
pub mod text_input;
pub mod tree_widget;

use crate::theme::{self, BadgeTone, Surface};
use iced::widget::{container, Column, Container, Row, Space, Text};
use iced::{Alignment, Background, Element, Length};

pub trait OptionalPush<'a, Message, Theme, Renderer>: Sized {
    fn push_maybe<E>(self, element: Option<E>) -> Self
    where
        E: Into<Element<'a, Message, Theme, Renderer>>;
}

impl<'a, Message, Theme, Renderer> OptionalPush<'a, Message, Theme, Renderer>
    for Row<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    fn push_maybe<E>(self, element: Option<E>) -> Self
    where
        E: Into<Element<'a, Message, Theme, Renderer>>,
    {
        match element {
            Some(element) => self.push(element),
            None => self,
        }
    }
}

impl<'a, Message, Theme, Renderer> OptionalPush<'a, Message, Theme, Renderer>
    for Column<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    fn push_maybe<E>(self, element: Option<E>) -> Self
    where
        E: Into<Element<'a, Message, Theme, Renderer>>,
    {
        match element {
            Some(element) => self.push(element),
            None => self,
        }
    }
}

pub fn stat_card<'a, Message: 'a>(
    label: &'a str,
    value: String,
    detail: &'a str,
) -> Element<'a, Message> {
    Container::new(
        Column::new()
            .spacing(theme::spacing::SM)
            .push(
                Text::new(label)
                    .size(10)
                    .color(theme::darcula::TEXT_SECONDARY),
            )
            .push(Text::new(value).size(20))
            .push(
                Text::new(detail)
                    .size(11)
                    .color(theme::darcula::TEXT_SECONDARY),
            ),
    )
    .padding([14, 16])
    .style(theme::panel_style(Surface::Raised))
    .into()
}

pub fn info_chip<'a, Message: 'a>(
    label: impl Into<String>,
    tone: BadgeTone,
) -> Element<'a, Message> {
    Container::new(Text::new(label.into()).size(10))
        .padding([3, 8])
        .style(theme::badge_style(tone))
        .into()
}

/// IDEA-style info chip with optional icon prefix
/// e.g., "✓ 3 files staged" or "⚠ 2 conflicts"
pub fn info_chip_with_icon<'a, Message: 'a>(
    icon: &'static str,
    label: impl Into<String>,
    tone: BadgeTone,
) -> Element<'a, Message> {
    let label_text: String = label.into();
    Container::new(
        Row::new()
            .spacing(4)
            .align_y(Alignment::Center)
            .push(Text::new(icon).size(10))
            .push(Text::new(label_text).size(10)),
    )
    .padding([3, 8])
    .style(theme::badge_style(tone))
    .into()
}

pub fn compact_chip<'a, Message: 'a>(
    label: impl Into<String>,
    tone: BadgeTone,
) -> Element<'a, Message> {
    Container::new(Text::new(label.into()).size(10))
        .padding(theme::density::COMPACT_CHIP_PADDING)
        .style(theme::badge_style(tone))
        .into()
}

/// IDEA-style loading spinner with animated dots
/// Shows three cycling dots in a subtle animation
pub fn loading_spinner<'a, Message: 'a>() -> Element<'a, Message> {
    // IDEA-style: subtle pulsing dots indicator
    Container::new(
        Row::new()
            .spacing(2)
            .align_y(Alignment::Center)
            .push(Text::new("◐").size(12).color(theme::darcula::ACCENT))
            .push(
                Text::new("◑")
                    .size(12)
                    .color(theme::darcula::TEXT_SECONDARY),
            ),
    )
    .into()
}

/// IDEA-style inline loading indicator for text fields
/// Shows "Loading…" with pulsing dot
pub fn inline_loading<'a, Message: Clone + 'a>(label: &'a str) -> Element<'a, Message> {
    Container::new(
        Row::new()
            .spacing(6)
            .align_y(Alignment::Center)
            .push(
                Text::new(label)
                    .size(11)
                    .color(theme::darcula::TEXT_SECONDARY),
            )
            .push(Text::new("…").size(11).color(theme::darcula::TEXT_DISABLED)),
    )
    .into()
}

pub fn section_header<'a, Message: 'a>(
    eyebrow: impl Into<String>,
    title: &'a str,
    detail: &'a str,
) -> Element<'a, Message> {
    let eyebrow_text = eyebrow.into();
    Column::new()
        .spacing(theme::spacing::SM)
        .push(
            Container::new(
                Text::new(eyebrow_text)
                    .size(10)
                    .color(theme::darcula::ACCENT),
            )
            .padding([4, 10])
            .style(theme::badge_style(BadgeTone::Accent)),
        )
        .push(Text::new(title).size(18))
        .push(
            Text::new(detail)
                .size(12)
                .color(theme::darcula::TEXT_SECONDARY),
        )
        .into()
}

pub fn status_banner<'a, Message: 'a>(
    label: impl Into<String>,
    detail: impl Into<String>,
    tone: BadgeTone,
) -> Element<'a, Message> {
    let surface = match tone {
        BadgeTone::Neutral => Surface::Raised,
        BadgeTone::Accent => Surface::Accent,
        BadgeTone::Success => Surface::Success,
        BadgeTone::Warning => Surface::Warning,
        BadgeTone::Danger => Surface::Danger,
    };

    Container::new(
        Column::new()
            .spacing(theme::spacing::SM)
            .push(info_chip::<Message>(label, tone))
            .push(
                Text::new(detail.into())
                    .size(11)
                    .width(Length::Fill)
                    .color(theme::darcula::TEXT_SECONDARY),
            ),
    )
    .padding([10, 12])
    .style(theme::panel_style(surface))
    .into()
}

pub fn panel_empty_state<'a, Message: 'a>(
    eyebrow: impl Into<String>,
    title: impl Into<String>,
    detail: impl Into<String>,
    action: Option<Element<'a, Message>>,
) -> Element<'a, Message> {
    let eyebrow_text: String = eyebrow.into();
    let mut column = Column::new()
        .spacing(theme::spacing::SM)
        .align_x(Alignment::Start)
        .push(
            Container::new(
                Text::new(eyebrow_text.to_uppercase())
                    .size(10)
                    .color(theme::darcula::ACCENT),
            )
            .padding([4, 10])
            .style(theme::badge_style(BadgeTone::Accent)),
        )
        .push(Text::new(title.into()).size(18))
        .push(
            Text::new(detail.into())
                .size(12)
                .color(theme::darcula::TEXT_SECONDARY),
        );

    if let Some(action) = action {
        column = column
            .push(Space::new().height(Length::Fixed(theme::spacing::SM)))
            .push(action);
    }

    Container::new(column)
        .padding([20, 20])
        .width(Length::Fill)
        .style(theme::panel_style(Surface::Raised))
        .into()
}

/// Compact empty state for nested panels (e.g. diff viewer).
/// Uses a faint centered text with minimal visual weight, matching IDEA.
pub fn panel_empty_state_compact<'a, Message: 'a>(
    title: impl Into<String>,
    detail: impl Into<String>,
) -> Element<'a, Message> {
    Container::new(
        Column::new()
            .spacing(theme::spacing::XS)
            .align_x(Alignment::Center)
            .push(
                Text::new(title.into())
                    .size(13)
                    .color(theme::darcula::TEXT_SECONDARY),
            )
            .push(
                Text::new(detail.into())
                    .size(10)
                    .color(theme::darcula::TEXT_DISABLED),
            ),
    )
    .padding([12, 12])
    .width(Length::Fill)
    .align_x(iced::alignment::Horizontal::Center)
    .align_y(iced::alignment::Vertical::Center)
    .into()
}

/// IDEA-style separator with optional text caption
/// Used to separate sections like "Recent Branches" or "Actions"
pub fn separator_with_text<'a, Message: Clone + 'a>(
    caption: Option<&'a str>,
) -> Element<'a, Message> {
    match caption {
        Some(text) => {
            // IDEA-style: refined separator with centered text caption
            // Uses a subtle line with uppercase text in disabled color
            Column::new()
                .spacing(theme::spacing::XS)
                .align_x(Alignment::Center)
                .push(
                    Row::new()
                        .spacing(theme::spacing::SM)
                        .align_y(Alignment::Center)
                        .push(
                            Container::new(Space::new())
                                .height(Length::Fixed(1.0))
                                .width(Length::FillPortion(1))
                                .style(|_| container::Style {
                                    background: Some(Background::Color(
                                        theme::darcula::SEPARATOR.scale_alpha(0.5),
                                    )),
                                    ..Default::default()
                                }),
                        )
                        .push(
                            Text::new(text.to_uppercase())
                                .size(10)
                                .color(theme::darcula::TEXT_DISABLED),
                        )
                        .push(
                            Container::new(Space::new())
                                .height(Length::Fixed(1.0))
                                .width(Length::FillPortion(1))
                                .style(|_| container::Style {
                                    background: Some(Background::Color(
                                        theme::darcula::SEPARATOR.scale_alpha(0.5),
                                    )),
                                    ..Default::default()
                                }),
                        ),
                )
                .into()
        }
        None => {
            // Just spacing for empty separator
            Space::new()
                .height(Length::Fixed(theme::spacing::SM))
                .into()
        }
    }
}
