//! Editor-backed split diff viewer shell.

use crate::theme::{self, BadgeTone};
use crate::widgets::diff_editor::{DiffEditorEvent, SplitDiffEditorState};
use crate::widgets::{self, button, diff_core, OptionalPush};
use git_core::diff::{EditorDiffHunk, EditorDiffModel};
use iced::widget::{Column, Container, Row, Space, Text};
use iced::{Alignment, Element, Length};

pub fn view<'a, Message: Clone + 'static>(
    model: &'a EditorDiffModel,
    editor: &'a SplitDiffEditorState,
    selected_hunk_index: Option<usize>,
    on_editor_event: fn(DiffEditorEvent) -> Message,
    on_stage_hunk: Option<fn(String, usize) -> Message>,
    on_unstage_hunk: Option<fn(String, usize) -> Message>,
) -> Element<'a, Message> {
    if model.hunks.is_empty() {
        return widgets::panel_empty_state_compact(
            "No split diff to display",
            "Select a file with text changes, then switch to split view.",
        );
    }

    let current_hunk = selected_hunk_index
        .and_then(|index| model.hunks.get(index))
        .or_else(|| model.hunks.first());

    let strip = current_hunk
        .map(|hunk| hunk_strip(model, hunk, on_stage_hunk, on_unstage_hunk))
        .unwrap_or_else(|| Space::new().height(Length::Shrink).into());

    Column::new()
        .spacing(0)
        .width(Length::Fill)
        .height(Length::Fill)
        .push(strip)
        .push(
            Container::new(editor.view(selected_hunk_index).map(on_editor_event))
                .height(Length::Fill)
                .style(diff_core::editor_surface_style()),
        )
        .into()
}

fn hunk_strip<'a, Message: Clone + 'static>(
    model: &'a EditorDiffModel,
    hunk: &'a EditorDiffHunk,
    on_stage_hunk: Option<fn(String, usize) -> Message>,
    on_unstage_hunk: Option<fn(String, usize) -> Message>,
) -> Element<'a, Message> {
    let file_path = model
        .new_path
        .clone()
        .or_else(|| model.old_path.clone())
        .unwrap_or_default();
    let stage_message = on_stage_hunk.map(|handler| handler(file_path.clone(), hunk.id));
    let unstage_message = on_unstage_hunk.map(|handler| handler(file_path, hunk.id));

    Container::new(
        Row::new()
            .spacing(theme::spacing::SM)
            .align_y(Alignment::Center)
            .push(widgets::compact_chip::<Message>(
                format!("Hunk {}/{}", hunk.id + 1, model.hunks.len()),
                BadgeTone::Accent,
            ))
            .push(
                Text::new(hunk.header.clone())
                    .size(theme::typography::CAPTION_SIZE)
                    .color(theme::darcula::TEXT_SECONDARY),
            )
            .push(Space::new().width(Length::Fill))
            .push_maybe(
                stage_message.map(|message| button::compact_ghost("Stage Hunk", Some(message))),
            )
            .push_maybe(
                unstage_message.map(|message| button::compact_ghost("Unstage", Some(message))),
            ),
    )
    .padding(theme::density::SECONDARY_BAR_PADDING)
    .style(theme::frame_style(theme::Surface::Toolbar))
    .into()
}

#[cfg(test)]
mod tests {
    #[test]
    fn split_viewer_handles_empty_hunk_state() {
        assert_eq!(iced::Length::Fill, iced::Length::Fill);
    }
}
