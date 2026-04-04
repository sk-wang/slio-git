//! Network operation progress bar widget with cancel button

use crate::theme;
use iced::widget::{button, container, Container, Row, Space, Text};
use iced::{Alignment, Background, Color, Element, Length};

/// Messages emitted by the progress bar
#[derive(Debug, Clone)]
pub enum ProgressMessage {
    /// User clicked cancel button
    Cancel,
}

/// Render a progress bar with operation info and cancel button
pub fn progress_bar_view<'a, Message: Clone + 'a>(
    operation: &'a str,
    progress: Option<f32>,
    status_text: Option<&'a str>,
    on_message: impl Fn(ProgressMessage) -> Message + 'a,
) -> Element<'a, Message> {
    let progress_pct = progress.unwrap_or(0.0);
    let progress_text = if progress.is_some() {
        format!("{}%", (progress_pct * 100.0) as u32)
    } else {
        "…".to_string()
    };

    let display_text = match status_text {
        Some(status) => format!("{} — {}", operation, status),
        None => operation.to_string(),
    };

    let bar_width = 120.0;
    let filled_width = bar_width * progress_pct;

    let bar = Container::new(
        Container::new(Space::new())
            .width(Length::Fixed(filled_width))
            .height(Length::Fixed(3.0))
            .style(|_| container::Style {
                background: Some(Background::Color(theme::darcula::ACCENT)),
                ..Default::default()
            }),
    )
    .width(Length::Fixed(bar_width))
    .height(Length::Fixed(3.0))
    .style(|_| container::Style {
        background: Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.1))),
        ..Default::default()
    });

    let row = Row::new()
        .spacing(theme::spacing::SM)
        .align_y(Alignment::Center)
        .push(
            Text::new(display_text)
                .size(11)
                .color(theme::darcula::TEXT_SECONDARY),
        )
        .push(bar)
        .push(
            Text::new(progress_text)
                .size(10)
                .color(theme::darcula::TEXT_DISABLED),
        )
        .push(
            button(
                Text::new("✕")
                    .size(10)
                    .color(theme::darcula::TEXT_SECONDARY),
            )
            .style(|_, _| button::Style::default())
            .padding([2, 6])
            .on_press(on_message(ProgressMessage::Cancel)),
        );

    Container::new(row)
        .padding([4, 8])
        .width(Length::Shrink)
        .into()
}
