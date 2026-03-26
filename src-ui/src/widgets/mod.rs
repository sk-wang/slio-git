//! UI widgets module and shared shell primitives.

pub mod button;
pub mod changelist;
pub mod commit_compare;
pub mod conflict_resolver;
pub mod diff_file_header;
pub mod diff_viewer;
pub mod file_picker;
pub mod scrollable;
pub mod split_diff_viewer;
pub mod statusbar;
pub mod syntax_highlighting;
pub mod text_input;

use crate::theme::{self, BadgeTone, Surface};
use iced::widget::{Column, Container, Row, Space, Text};
use iced::{Alignment, Element, Length};

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
        .padding([5, 10])
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

pub fn section_header<'a, Message: 'a>(
    eyebrow: &'a str,
    title: &'a str,
    detail: &'a str,
) -> Element<'a, Message> {
    Column::new()
        .spacing(theme::spacing::SM)
        .push(
            Container::new(Text::new(eyebrow).size(10).color(theme::darcula::ACCENT))
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
                    .size(12)
                    .width(Length::Fill)
                    .color(theme::darcula::TEXT_SECONDARY),
            ),
    )
    .padding([14, 16])
    .style(theme::panel_style(surface))
    .into()
}

pub fn panel_empty_state<'a, Message: 'a>(
    eyebrow: impl Into<String>,
    title: impl Into<String>,
    detail: impl Into<String>,
    action: Option<Element<'a, Message>>,
) -> Element<'a, Message> {
    let mut column = Column::new()
        .spacing(theme::spacing::SM)
        .align_x(Alignment::Start)
        .push(
            Container::new(
                Text::new(eyebrow.into())
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
