//! Styled scrollable helpers.

use crate::theme;
use iced::widget::{scrollable, Scrollable};
use iced::Element;

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
        ScrollbarRole::Pane => ScrollbarMetrics {
            width: 5,
            scroller_width: 3,
            margin: 1,
        },
        ScrollbarRole::Inline => ScrollbarMetrics {
            width: 3,
            scroller_width: 2,
            margin: 0,
        },
    }
}

fn build_scrollbar(role: ScrollbarRole) -> scrollable::Scrollbar {
    let metrics = scrollbar_metrics(role);
    scrollable::Scrollbar::new()
        .width(u32::from(metrics.width))
        .scroller_width(u32::from(metrics.scroller_width))
        .margin(u32::from(metrics.margin))
}

fn horizontal_direction(role: ScrollbarRole) -> scrollable::Direction {
    scrollable::Direction::Horizontal(build_scrollbar(role))
}

fn editor_horizontal_direction() -> scrollable::Direction {
    horizontal_direction(ScrollbarRole::Inline)
}

pub fn styled<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
) -> Scrollable<'a, Message> {
    Scrollable::new(content)
        .direction(scrollable::Direction::Vertical(build_scrollbar(
            ScrollbarRole::Pane,
        )))
        .style(theme::scrollable_style())
}

pub fn styled_horizontal<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
) -> Scrollable<'a, Message> {
    Scrollable::new(content)
        .direction(horizontal_direction(ScrollbarRole::Pane))
        .style(theme::scrollable_style())
}

pub fn styled_inline_horizontal<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
) -> Scrollable<'a, Message> {
    Scrollable::new(content)
        .direction(horizontal_direction(ScrollbarRole::Inline))
        .style(theme::scrollable_style())
}

pub fn styled_editor_horizontal<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
) -> Scrollable<'a, Message> {
    Scrollable::new(content)
        .direction(editor_horizontal_direction())
        .style(theme::scrollable_style())
}

pub fn styled_both<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
) -> Scrollable<'a, Message> {
    Scrollable::new(content)
        .direction(scrollable::Direction::Both {
            vertical: build_scrollbar(ScrollbarRole::Pane),
            horizontal: build_scrollbar(ScrollbarRole::Pane),
        })
        .style(theme::scrollable_style())
}

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

    #[test]
    fn editor_horizontal_scrollbars_match_inline_scrollbars() {
        assert_eq!(
            editor_horizontal_direction(),
            horizontal_direction(ScrollbarRole::Inline)
        );
        assert_ne!(
            editor_horizontal_direction(),
            horizontal_direction(ScrollbarRole::Pane)
        );
    }
}
