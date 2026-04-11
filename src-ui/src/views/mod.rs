//! UI views module and shared render helpers.

pub mod branch_popup;
pub mod commit_dialog;
pub mod history_view;
pub mod main_window;
pub mod rebase_editor;
pub mod remote_dialog;
pub mod settings_view;
pub mod stash_panel;
pub mod tag_dialog;
pub mod worktree_view;

use crate::state::{FeedbackLevel, FeedbackState, ToastNotificationState};
use crate::theme::{self, BadgeTone, Surface};
use crate::widgets::{self, OptionalPush};
use iced::widget::{container, Button, Column, Container, Row, Space, Text};
use iced::{Alignment, Background, Border, Color, Element, Length, Theme};

pub fn render_feedback_banner<'a, Message: Clone + 'a>(
    feedback: &'a FeedbackState,
    on_dismiss: Option<Message>,
) -> Element<'a, Message> {
    let (tone, label) = match feedback.level {
        FeedbackLevel::Info => (BadgeTone::Accent, "Info"),
        FeedbackLevel::Success => (BadgeTone::Success, "Done"),
        FeedbackLevel::Warning => (BadgeTone::Warning, "Warning"),
        FeedbackLevel::Error => (BadgeTone::Danger, "Failed"),
        FeedbackLevel::Loading => (BadgeTone::Neutral, "Processing"),
        FeedbackLevel::Empty => (BadgeTone::Neutral, "Empty"),
    };

    // IDEA-style: compact mode for inline feedback without full banner
    if feedback.compact {
        let content = Row::new()
            .spacing(theme::spacing::SM)
            .align_y(Alignment::Center)
            .push(widgets::compact_chip::<Message>(label, tone))
            .push(
                Text::new(&feedback.title)
                    .size(12)
                    .color(theme::darcula::TEXT_PRIMARY),
            )
            .push_maybe(feedback.detail.as_ref().map(|detail| {
                Text::new(detail)
                    .size(11)
                    .color(theme::darcula::TEXT_SECONDARY)
            }));

        return Container::new(content)
            .padding([6, 10])
            .style(theme::panel_style(match feedback.level {
                FeedbackLevel::Info => Surface::Accent,
                FeedbackLevel::Success => Surface::Success,
                FeedbackLevel::Warning => Surface::Warning,
                FeedbackLevel::Error => Surface::Danger,
                FeedbackLevel::Loading | FeedbackLevel::Empty => Surface::Raised,
            }))
            .into();
    }

    let dismiss_button: Option<Element<'a, Message>> = on_dismiss.map(|message| {
        Button::new(Text::new("Dismiss").size(12))
            .style(theme::button_style(theme::ButtonTone::Ghost))
            .on_press(message)
            .into()
    });

    let content = Row::new()
        .spacing(theme::spacing::MD)
        .align_y(Alignment::Center)
        .push(widgets::info_chip::<Message>(label, tone))
        .push(
            Column::new()
                .spacing(4)
                .width(Length::Fill)
                .push(Text::new(&feedback.title).size(15))
                .push_maybe(feedback.detail.as_ref().map(|detail| {
                    Text::new(detail)
                        .size(12)
                        .color(theme::darcula::TEXT_SECONDARY)
                })),
        )
        .push(Space::new().width(Length::Fill))
        .push_maybe(dismiss_button);

    Container::new(content)
        .padding([14, 16])
        .style(theme::panel_style(match feedback.level {
            FeedbackLevel::Info => Surface::Accent,
            FeedbackLevel::Success => Surface::Success,
            FeedbackLevel::Warning => Surface::Warning,
            FeedbackLevel::Error => Surface::Danger,
            FeedbackLevel::Loading | FeedbackLevel::Empty => Surface::Raised,
        }))
        .into()
}

pub fn render_empty_state<'a, Message: Clone + 'a>(
    eyebrow: &'a str,
    title: &'a str,
    detail: &'a str,
    action: Option<Element<'a, Message>>,
) -> Element<'a, Message> {
    Container::new(widgets::panel_empty_state(eyebrow, title, detail, action))
        .padding([theme::spacing::LG, theme::spacing::LG])
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(theme::frame_style(Surface::Root))
        .into()
}

pub fn render_toast_notification<'a, Message: Clone + 'a>(
    toast: &'a ToastNotificationState,
    on_dismiss: Option<Message>,
) -> Element<'a, Message> {
    let (icon, badge_tone, title_color) = match toast.level {
        FeedbackLevel::Info => ("i", BadgeTone::Accent, theme::darcula::ACCENT),
        FeedbackLevel::Success => ("✓", BadgeTone::Success, theme::darcula::SUCCESS),
        FeedbackLevel::Warning => ("!", BadgeTone::Warning, theme::darcula::WARNING),
        FeedbackLevel::Error => ("×", BadgeTone::Danger, theme::darcula::DANGER),
        FeedbackLevel::Loading => ("…", BadgeTone::Neutral, theme::darcula::TEXT_PRIMARY),
        FeedbackLevel::Empty => ("•", BadgeTone::Neutral, theme::darcula::TEXT_PRIMARY),
    };

    let dismiss_button: Option<Element<'a, Message>> = on_dismiss.map(|message| {
        Button::new(
            Text::new("×")
                .size(14)
                .color(theme::darcula::TEXT_SECONDARY),
        )
        .style(theme::button_style(theme::ButtonTone::Ghost))
        .on_press(message)
        .into()
    });

    let card = Container::new(
        Row::new()
            .spacing(theme::spacing::MD)
            .align_y(Alignment::Start)
            .push(
                Container::new(Text::new(icon).size(14).color(title_color))
                    .padding([6, 10])
                    .style(theme::badge_style(badge_tone)),
            )
            .push(
                Column::new()
                    .spacing(theme::spacing::SM)
                    .width(Length::Fill)
                    .push(Text::new(&toast.title).size(16).color(title_color))
                    .push_maybe(toast.detail.as_ref().map(|detail| {
                        Text::new(detail)
                            .size(12)
                            .width(Length::Fill)
                            .wrapping(iced::widget::text::Wrapping::WordOrGlyph)
                            .color(theme::darcula::TEXT_PRIMARY)
                    })),
            )
            .push_maybe(dismiss_button),
    )
    .padding([14, 16])
    .width(Length::Fixed(420.0))
    .style(toast_card_style);

    Container::new(
        Column::new()
            .push(Space::new().height(Length::Fill))
            .push(Row::new().push(Space::new().width(Length::Fill)).push(card)),
    )
    .padding([0, 18])
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn toast_card_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(theme::darcula::BG_PANEL)),
        border: Border {
            width: 1.0,
            color: theme::darcula::ACCENT.scale_alpha(0.18),
            radius: theme::radius::LG.into(),
        },
        shadow: iced::Shadow {
            color: Color::from_rgba(0.149, 0.149, 0.149, 0.12),
            offset: iced::Vector::new(0.0, 10.0),
            blur_radius: 24.0,
        },
        ..Default::default()
    }
}
