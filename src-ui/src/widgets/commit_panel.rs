//! Embedded commit panel widget.
//!
//! Provides a non-modal commit composition UI for use inside the Changes workspace.
//! Includes amend toggle and recent commit message history dropdown.

use crate::i18n::I18n;
use crate::theme::{self, BadgeTone, Surface};
use crate::views::commit_dialog::{CommitDialogMessage, CommitDialogState};
use crate::widgets::{self, button, OptionalPush};
use iced::widget::{text, text_editor, Column, Container, Row, Space, Text};
use iced::{Alignment, Element, Length};

/// Build an embedded commit panel view backed by commit-dialog state.
pub fn view<'a>(
    state: &'a CommitDialogState,
    recent_messages: &'a [String],
    llm_enabled: bool,
    i18n: &'a I18n,
) -> Element<'a, CommitDialogMessage> {
    let status_panel = if state.is_committing {
        Some(build_compact_status(
            i18n.loading,
            i18n.committing_please_wait,
            BadgeTone::Neutral,
        ))
    } else if let Some(error) = state.error.as_ref() {
        Some(build_compact_status(i18n.error, error, BadgeTone::Danger))
    } else {
        state
            .success_message
            .as_ref()
            .map(|message| build_compact_status("OK", message, BadgeTone::Success))
    };

    let commit_label = if state.is_committing {
        i18n.committing
    } else if state.is_amend {
        i18n.amend
    } else {
        i18n.commit
    };
    let commit_enabled =
        state.is_message_valid() && state.has_files_to_commit() && !state.is_committing;

    let editor = text_editor(&state.message_editor)
        .placeholder(i18n.commit_message_placeholder)
        .padding([8, 10])
        .size(theme::typography::BODY_SIZE as f32)
        .height(Length::Fill)
        .style(theme::text_editor_style())
        .on_action(CommitDialogMessage::MessageEdited);

    // Recent message history dropdown
    let history_button: Element<'_, CommitDialogMessage> = if !recent_messages.is_empty() {
        button::toolbar_icon("⏱", Some(CommitDialogMessage::ToggleRecentMessages)).into()
    } else {
        Space::new().width(Length::Shrink).into()
    };

    let amend_checkbox: Element<'_, CommitDialogMessage> = if state.is_committing {
        Space::new().width(Length::Shrink).into()
    } else {
        widgets::compact_checkbox(
            state.is_amend,
            i18n.amend,
            CommitDialogMessage::SetAmendMode,
        )
    };

    let ai_button: Element<'_, CommitDialogMessage> = if llm_enabled {
        if state.is_generating {
            button::secondary(i18n.ai_generating, None::<CommitDialogMessage>).into()
        } else {
            let can_generate = state.has_files_to_commit() && !state.is_committing;
            button::secondary(
                i18n.ai_generate,
                can_generate.then_some(CommitDialogMessage::GenerateCommitMessage),
            )
            .into()
        }
    } else {
        Space::new().width(Length::Shrink).into()
    };

    let actions = Row::new()
        .spacing(theme::spacing::XS)
        .align_y(Alignment::Center)
        .push(ai_button)
        .push(history_button)
        .push(amend_checkbox)
        .push(Space::new().width(Length::Fill))
        .push(button::secondary(
            i18n.commit_and_push,
            commit_enabled.then_some(CommitDialogMessage::CommitAndPushPressed),
        ))
        .push(button::primary(
            commit_label,
            commit_enabled.then_some(CommitDialogMessage::CommitPressed),
        ));

    Column::new()
        .spacing(0)
        .push_maybe(status_panel)
        .push(Container::new(editor).padding([4, 6]).height(Length::Fill))
        .push(
            Container::new(actions)
                .padding([4, 6])
                .style(theme::frame_style(Surface::Toolbar)),
        )
        .into()
}

fn build_compact_status<'a, Message: 'a>(
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
        Row::new()
            .spacing(theme::spacing::XS)
            .align_y(Alignment::Center)
            .push(widgets::compact_chip::<Message>(label.into(), tone))
            .push(
                Text::new(detail.into())
                    .size(theme::typography::CAPTION_SIZE)
                    .line_height(text::LineHeight::Relative(1.0))
                    .font(theme::app_font())
                    .width(Length::Fill)
                    .wrapping(text::Wrapping::WordOrGlyph)
                    .color(theme::darcula::TEXT_SECONDARY),
            ),
    )
    .padding([5, 10])
    .width(Length::Fill)
    .style(theme::panel_style(surface))
    .into()
}
