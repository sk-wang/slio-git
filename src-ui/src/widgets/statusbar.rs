//! Shared status bar widget for the main shell.

use crate::i18n::I18n;
use crate::theme::{self, BadgeTone, Surface};
use crate::widgets::OptionalPush;
use iced::widget::{Container, Row, Space, Text};
use iced::{Alignment, Element, Length};

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
        .collect::<String>()
        .trim_start_matches(['/', '\\'])
        .to_string();

    format!("{prefix}…{suffix}")
}

pub struct StatusBar<'a> {
    pub i18n: &'a I18n,
    pub repo_path: Option<String>,
    pub workspace_summary: String,
    pub selected_path: Option<String>,
    pub activity_label: String,
    pub activity_tone: BadgeTone,
    pub detail: Option<String>,
}

impl<'a> StatusBar<'a> {
    pub fn view<Message: 'a>(self) -> Element<'a, Message> {
        let repo_text = truncate_middle(
            &self
                .repo_path
                .unwrap_or_else(|| self.i18n.no_repository.to_string()),
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
            .push(
                Text::new(repo_text)
                    .size(10)
                    .color(theme::darcula::TEXT_SECONDARY)
                    .width(Length::FillPortion(3)),
            )
            .push(
                Text::new(selected_text)
                    .size(10)
                    .color(theme::darcula::TEXT_SECONDARY)
                    .width(Length::FillPortion(2)),
            )
            .push(Space::new().width(Length::Fill))
            .push(
                Text::new(self.workspace_summary)
                    .size(10)
                    .color(theme::darcula::TEXT_SECONDARY),
            )
            .push(Self::separator())
            .push(
                Text::new(self.activity_label)
                    .size(10)
                    .color(Self::tone_color(self.activity_tone)),
            )
            .push_maybe(self.detail.map(|detail| {
                Row::new()
                    .spacing(theme::spacing::XS)
                    .align_y(Alignment::Center)
                    .push(Self::separator())
                    .push(
                        Text::new(truncate_middle(&detail, 28))
                            .size(10)
                            .color(theme::darcula::TEXT_SECONDARY),
                    )
            }));

        Container::new(content)
            .padding(theme::density::STATUS_PADDING)
            .width(Length::Fill)
            .style(theme::frame_style(Surface::Status))
            .into()
    }

    fn separator<'b, Message: 'b>() -> Element<'b, Message> {
        Text::new("|")
            .size(10)
            .color(theme::darcula::TEXT_DISABLED)
            .into()
    }

    fn tone_color(tone: BadgeTone) -> iced::Color {
        match tone {
            BadgeTone::Neutral => theme::darcula::TEXT_SECONDARY,
            BadgeTone::Accent => theme::darcula::ACCENT,
            BadgeTone::Success => theme::darcula::SUCCESS,
            BadgeTone::Warning => theme::darcula::WARNING,
            BadgeTone::Danger => theme::darcula::DANGER,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_middle_keeps_both_ends() {
        assert_eq!(
            truncate_middle("/Users/wanghao/git/slio-git", 18),
            "/Users/w…slio-git"
        );
    }

    #[test]
    fn truncate_middle_leaves_short_strings_untouched() {
        assert_eq!(truncate_middle("src-ui", 18), "src-ui");
    }
}
