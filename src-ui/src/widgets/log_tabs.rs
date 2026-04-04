//! Multi-tab bar widget for the log view

use crate::theme;
use iced::widget::{button, container, Container, Row, Space, Text};
use iced::{Alignment, Background, Element, Length};

/// Messages emitted by the log tabs widget
#[derive(Debug, Clone)]
pub enum LogTabsMessage {
    /// Switch to a tab by index
    SelectTab(usize),
    /// Close a tab by index (not emitted for permanent tabs)
    CloseTab(usize),
    /// Create a new empty tab
    NewTab,
}

/// A single tab descriptor
pub struct TabDescriptor<'a> {
    pub label: &'a str,
    pub is_active: bool,
    pub is_closable: bool,
}

/// Render the log tab bar
pub fn log_tabs_view<'a, Message: Clone + 'a>(
    tabs: &'a [TabDescriptor<'a>],
    on_message: impl Fn(LogTabsMessage) -> Message + Clone + 'a,
) -> Element<'a, Message> {
    let mut row = Row::new().spacing(0).align_y(Alignment::End);

    for (i, tab) in tabs.iter().enumerate() {
        let is_active = tab.is_active;
        let on_msg = on_message.clone();

        let label = Text::new(tab.label)
            .size(12)
            .color(if is_active {
                theme::darcula::TEXT_PRIMARY
            } else {
                theme::darcula::TEXT_SECONDARY
            });

        let mut tab_content = Row::new()
            .spacing(4)
            .align_y(Alignment::Center)
            .push(label);

        if tab.is_closable {
            let close_idx = i;
            let on_msg_close = on_message.clone();
            tab_content = tab_content.push(
                button(
                    Text::new("×")
                        .size(10)
                        .color(theme::darcula::TEXT_DISABLED),
                )
                .style(|_, _| button::Style::default())
                .padding(0)
                .on_press(on_msg_close(LogTabsMessage::CloseTab(close_idx))),
            );
        }

        let tab_style = if is_active {
            move |_: &_| container::Style {
                background: Some(Background::Color(theme::darcula::BG_RAISED)),
                border: iced::Border {
                    color: theme::darcula::ACCENT,
                    width: 0.0,
                    radius: iced::border::Radius::new(4.0),
                },
                ..Default::default()
            }
        } else {
            move |_: &_| container::Style {
                background: Some(Background::Color(theme::darcula::BG_SOFT)),
                border: iced::Border {
                    color: theme::darcula::SEPARATOR,
                    width: 0.0,
                    radius: iced::border::Radius::new(4.0),
                },
                ..Default::default()
            }
        };

        let tab_button = button(
            Container::new(tab_content)
                .padding([6, 12])
                .style(tab_style),
        )
        .style(|_, _| button::Style::default())
        .padding(0)
        .on_press(on_msg(LogTabsMessage::SelectTab(i)));

        row = row.push(tab_button);
        row = row.push(Space::new().width(Length::Fixed(1.0)));
    }

    // Add "+" button for new tab
    let on_msg_new = on_message.clone();
    row = row.push(
        button(
            Text::new("+")
                .size(12)
                .color(theme::darcula::TEXT_DISABLED),
        )
        .style(|_, _| button::Style::default())
        .padding([6, 10])
        .on_press(on_msg_new(LogTabsMessage::NewTab)),
    );

    Container::new(row).width(Length::Fill).into()
}
