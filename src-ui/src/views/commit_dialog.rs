//! Commit dialog view.
//!
//! Provides a dialog for creating and amending commits.

use crate::components::status_icons::FileStatus;
use crate::i18n::I18n;
use crate::theme::{self, BadgeTone, Surface};
use crate::widgets::{self, button, diff_viewer, scrollable, OptionalPush};
use git_core::commit::CommitInfo;
use git_core::diff::{Diff, FileDiff};
use git_core::index::Change;
use iced::widget::{text, text_editor, Button, Checkbox, Column, Container, Row, Space, Text};
use iced::{Alignment, Element, Length};

/// Message types for commit dialog.
#[derive(Debug, Clone)]
pub enum CommitDialogMessage {
    MessageEdited(text_editor::Action),
    FileToggled(String, bool),
    PreviewFile(String),
    CommitPressed,
    CommitAndPushPressed,
    SetAmendMode(bool),
    CancelPressed,
    ToggleRecentMessages,
    SelectRecentMessage(usize),
    GenerateCommitMessage,
    GenerateCommitMessageResult(Result<String, String>),
}

/// State for the commit dialog.
#[derive(Debug)]
pub struct CommitDialogState {
    pub message: String,
    pub message_editor: text_editor::Content,
    pub is_amend: bool,
    pub commit_to_amend: Option<CommitInfo>,
    pub diff: Diff,
    pub staged_files: Vec<Change>,
    pub selected_files: Vec<String>,
    pub previewed_file: Option<String>,
    pub is_committing: bool,
    pub is_generating: bool,
    pub error: Option<String>,
    pub success_message: Option<String>,
}

impl CommitDialogState {
    pub fn new() -> Self {
        Self {
            message: String::new(),
            message_editor: text_editor::Content::new(),
            is_amend: false,
            commit_to_amend: None,
            diff: Diff {
                files: Vec::new(),
                total_additions: 0,
                total_deletions: 0,
            },
            staged_files: Vec::new(),
            selected_files: Vec::new(),
            previewed_file: None,
            is_committing: false,
            is_generating: false,
            error: None,
            success_message: None,
        }
    }

    pub fn for_new_commit(staged_files: Vec<Change>, diff: &Diff) -> Self {
        let selected_files = staged_files
            .iter()
            .map(|change| change.path.clone())
            .collect::<Vec<_>>();

        Self {
            message: String::new(),
            message_editor: text_editor::Content::new(),
            is_amend: false,
            commit_to_amend: None,
            diff: diff.clone(),
            staged_files,
            selected_files: selected_files.clone(),
            previewed_file: initial_preview_path(&selected_files, diff),
            is_committing: false,
            is_generating: false,
            error: None,
            success_message: None,
        }
    }

    pub fn for_amend(staged_files: Vec<Change>, commit: CommitInfo, diff: &Diff) -> Self {
        let selected_files = staged_files
            .iter()
            .map(|change| change.path.clone())
            .collect::<Vec<_>>();

        Self {
            message: commit.message.clone(),
            message_editor: text_editor::Content::with_text(&commit.message),
            is_amend: true,
            commit_to_amend: Some(commit),
            diff: diff.clone(),
            staged_files,
            selected_files: selected_files.clone(),
            previewed_file: initial_preview_path(&selected_files, diff),
            is_committing: false,
            is_generating: false,
            error: None,
            success_message: None,
        }
    }

    /// Check if the commit message is valid (non-empty after trimming).
    pub fn is_message_valid(&self) -> bool {
        !self.message.trim().is_empty()
    }

    /// Check if there are files to commit.
    pub fn has_files_to_commit(&self) -> bool {
        self.is_amend || !self.staged_files.is_empty()
    }

    /// Toggle file selection.
    pub fn toggle_file(&mut self, path: String) {
        self.success_message = None;

        if let Some(pos) = self.selected_files.iter().position(|p| p == &path) {
            self.selected_files.remove(pos);
        } else {
            self.selected_files.push(path);
        }

        self.ensure_preview_target();
    }

    pub fn preview_file(&mut self, path: String) {
        self.previewed_file = Some(path);
        self.success_message = None;
    }

    pub fn apply_message_edit(&mut self, action: text_editor::Action) {
        self.message_editor.perform(action);
        self.message = normalize_editor_text(self.message_editor.text());
        self.error = None;
        self.success_message = None;
    }

    /// Set error message.
    pub fn set_error(&mut self, error: String) {
        self.error = Some(error);
        self.is_committing = false;
        self.success_message = None;
    }

    /// Clear error message.
    pub fn clear_error(&mut self) {
        self.error = None;
    }

    /// Start committing.
    pub fn start_commit(&mut self) {
        self.is_committing = true;
        self.clear_error();
        self.success_message = None;
    }

    /// Finish committing successfully.
    pub fn commit_success(&mut self) {
        self.is_committing = false;
        self.error = None;
        self.success_message = Some(if self.is_amend {
            "Commit amended.".to_string()
        } else {
            "Commit created.".to_string()
        });
        self.message.clear();
        self.message_editor = text_editor::Content::new();
        self.selected_files.clear();
        self.previewed_file = None;
    }

    pub fn selected_diff_summary(&self) -> (usize, u32, u32) {
        let mut file_count = 0usize;
        let mut additions = 0u32;
        let mut deletions = 0u32;

        for file in &self.diff.files {
            let Some(path) = diff_file_path(file) else {
                continue;
            };

            if self.selected_files.iter().any(|selected| selected == path) {
                file_count += 1;
                additions += file.additions;
                deletions += file.deletions;
            }
        }

        (file_count, additions, deletions)
    }

    pub fn file_diff(&self, path: &str) -> Option<&FileDiff> {
        self.diff
            .files
            .iter()
            .find(|file| diff_file_path(file) == Some(path))
    }

    pub fn enable_amend_mode(&mut self, commit: CommitInfo) {
        self.is_amend = true;
        self.message = commit.message.clone();
        self.message_editor = text_editor::Content::with_text(&commit.message);
        self.commit_to_amend = Some(commit);
        self.error = None;
        self.success_message = None;
        self.ensure_preview_target();
    }

    pub fn disable_amend_mode(&mut self) {
        self.is_amend = false;
        self.commit_to_amend = None;
        self.error = None;
        self.success_message = None;
        self.ensure_preview_target();
    }

    pub fn ensure_preview_target(&mut self) {
        let is_valid = self.previewed_file.as_ref().is_some_and(|path| {
            self.staged_files.iter().any(|change| &change.path == path)
                || self
                    .diff
                    .files
                    .iter()
                    .any(|file| diff_file_path(file) == Some(path.as_str()))
        });

        if !is_valid {
            self.previewed_file = initial_preview_path(&self.selected_files, &self.diff);
        }
    }
}

impl Clone for CommitDialogState {
    fn clone(&self) -> Self {
        Self {
            message: self.message.clone(),
            message_editor: text_editor::Content::with_text(&self.message),
            is_amend: self.is_amend,
            commit_to_amend: self.commit_to_amend.clone(),
            diff: self.diff.clone(),
            staged_files: self.staged_files.clone(),
            selected_files: self.selected_files.clone(),
            previewed_file: self.previewed_file.clone(),
            is_committing: self.is_committing,
            is_generating: self.is_generating,
            error: self.error.clone(),
            success_message: self.success_message.clone(),
        }
    }
}

impl Default for CommitDialogState {
    fn default() -> Self {
        Self::new()
    }
}

/// Build the commit dialog view.
pub fn view<'a>(state: &'a CommitDialogState, i18n: &'a I18n) -> Element<'a, CommitDialogMessage> {
    let files_list: Element<'_, CommitDialogMessage> = if state.staged_files.is_empty() {
        Column::new()
            .push(
                Text::new(i18n.cd_no_staged_files)
                    .size(12)
                    .color(theme::darcula::TEXT_SECONDARY),
            )
            .into()
    } else {
        state
            .staged_files
            .iter()
            .fold(Column::new().spacing(theme::spacing::XS), |column, file| {
                column.push(build_file_row(state, file))
            })
            .into()
    };

    let files_panel = Container::new(
        Column::new()
            .spacing(theme::spacing::XS)
            .height(Length::Fill)
            .push(
                Row::new()
                    .spacing(theme::spacing::XS)
                    .align_y(Alignment::Center)
                    .push(
                        Text::new(i18n.cd_files_to_commit)
                            .size(12)
                            .color(theme::darcula::TEXT_SECONDARY),
                    )
                    .push(widgets::compact_chip::<CommitDialogMessage>(
                        i18n.cd_staged_count_fmt.replace("{}", &state.staged_files.len().to_string()),
                        BadgeTone::Success,
                    ))
                    .push(widgets::compact_chip::<CommitDialogMessage>(
                        i18n.cd_selected_count_fmt.replace("{}", &state.selected_files.len().to_string()),
                        BadgeTone::Accent,
                    )),
            )
            .push(scrollable::styled(files_list).height(Length::Fill)),
    )
    .padding([8, 10])
    .width(Length::FillPortion(2))
    .height(Length::Fill)
    .style(theme::panel_style(Surface::Panel));

    let (selected_file_count, selected_additions, selected_deletions) =
        state.selected_diff_summary();

    let preview_header = Container::new(
        scrollable::styled_horizontal(
            Row::new()
                .spacing(theme::spacing::XS)
                .align_y(Alignment::Center)
                .push(widgets::compact_chip::<CommitDialogMessage>(
                    i18n.cd_file_count_fmt.replace("{}", &selected_file_count.to_string()),
                    BadgeTone::Success,
                ))
                .push(widgets::compact_chip::<CommitDialogMessage>(
                    format!("+{}", selected_additions),
                    BadgeTone::Success,
                ))
                .push(widgets::compact_chip::<CommitDialogMessage>(
                    format!("-{}", selected_deletions),
                    BadgeTone::Danger,
                ))
                .push_maybe(state.previewed_file.as_ref().map(|path| {
                    widgets::compact_chip::<CommitDialogMessage>(
                        i18n.cd_preview_fmt.replace("{}", &path.to_string()),
                        BadgeTone::Accent,
                    )
                })),
        )
        .width(Length::Fill),
    )
    .padding([5, 8])
    .style(theme::panel_style(Surface::Raised));

    let preview_body: Element<'_, CommitDialogMessage> =
        if let Some(path) = state.previewed_file.as_deref() {
            if let Some(file_diff) = state.file_diff(path) {
                Container::new(diff_viewer::file_preview(file_diff))
                    .padding([6, 6])
                    .width(Length::Fill)
                    .style(theme::panel_style(Surface::Raised))
                    .into()
            } else {
                widgets::panel_empty_state(
                    i18n.cd_preview,
                    i18n.cd_no_diff,
                    i18n.cd_no_diff_detail,
                    None,
                )
            }
        } else {
            widgets::panel_empty_state(
                i18n.cd_preview,
                i18n.cd_no_changes,
                i18n.cd_no_changes_detail,
                None,
            )
        };

    let diff_panel = Container::new(
        Column::new()
            .spacing(theme::spacing::XS)
            .height(Length::Fill)
            .push(
                Row::new()
                    .spacing(theme::spacing::XS)
                    .align_y(Alignment::Center)
                    .push(
                        Text::new(i18n.cd_file_changes)
                            .size(12)
                            .color(theme::darcula::TEXT_PRIMARY),
                    )
                    .push(
                        Text::new(i18n.cd_check_preview_hint)
                            .size(10)
                            .color(theme::darcula::TEXT_DISABLED),
                    ),
            )
            .push(preview_header)
            .push(scrollable::styled(preview_body).height(Length::Fill)),
    )
    .padding([8, 10])
    .width(Length::FillPortion(3))
    .height(Length::Fill)
    .style(theme::panel_style(Surface::Panel));

    // Calculate message stats (IDEA shows line/char count)
    let message_lines = state.message.lines().count();
    let message_chars = state.message.len();
    let is_valid = state.is_message_valid();

    let message_hint_text = if state.message.trim().is_empty() {
        i18n.cd_msg_hint_empty
    } else {
        i18n.cd_msg_hint_ready
    };

    let message_panel = Container::new(
        Column::new()
            .spacing(theme::spacing::XS)
            .push(
                Row::new()
                    .spacing(theme::spacing::XS)
                    .align_y(Alignment::Center)
                    .push(
                        Text::new(i18n.cd_commit_msg)
                            .size(12)
                            .color(theme::darcula::TEXT_PRIMARY),
                    )
                    .push(
                        Text::new(i18n.cd_msg_first_line_hint)
                            .size(10)
                            .color(theme::darcula::TEXT_DISABLED),
                    ),
            )
            .push(
                text_editor(&state.message_editor)
                    .placeholder(i18n.cd_msg_placeholder)
                    .padding([8, 10])
                    .size(theme::typography::BODY_SIZE as f32)
                    .height(Length::Fixed(88.0))
                    .style(theme::text_editor_style())
                    .on_action(CommitDialogMessage::MessageEdited),
            )
            .push(
                Row::new()
                    .spacing(theme::spacing::SM)
                    .align_y(Alignment::Center)
                    .push(
                        Text::new(i18n.cd_msg_stats_fmt.replace("{}", &message_lines.to_string()).replacen("{}", &message_chars.to_string(), 1))
                            .size(10)
                            .color(theme::darcula::TEXT_DISABLED),
                    )
                    .push(Space::new().width(Length::Fill))
                    .push(Text::new(message_hint_text).size(11).color(if is_valid {
                        theme::darcula::SUCCESS
                    } else {
                        theme::darcula::TEXT_SECONDARY
                    })),
            )
            .push(
                Text::new(i18n.cd_committer_identity)
                    .size(9)
                    .color(theme::darcula::TEXT_DISABLED),
            ),
    )
    .padding([8, 10])
    .style(theme::panel_style(Surface::Panel));

    let status_panel = if state.is_committing {
        Some(build_compact_commit_status::<CommitDialogMessage>(
            i18n.cd_status_processing,
            i18n.cd_status_processing_detail,
            BadgeTone::Neutral,
        ))
    } else if let Some(error) = state.error.as_ref() {
        Some(build_compact_commit_status::<CommitDialogMessage>(
            i18n.cd_status_failed,
            error,
            BadgeTone::Danger,
        ))
    } else if let Some(message) = state.success_message.as_ref() {
        Some(build_compact_commit_status::<CommitDialogMessage>(
            i18n.cd_status_done,
            message,
            BadgeTone::Success,
        ))
    } else if state.staged_files.is_empty() {
        Some(build_compact_commit_status::<CommitDialogMessage>(
            i18n.cd_status_empty,
            i18n.cd_status_empty_detail,
            BadgeTone::Neutral,
        ))
    } else if !state.is_message_valid() {
        Some(build_compact_commit_status::<CommitDialogMessage>(
            i18n.cd_status_needs_msg,
            i18n.cd_status_needs_msg_detail,
            BadgeTone::Warning,
        ))
    } else {
        None
    };

    let commit_label = if state.is_committing {
        i18n.cd_committing_btn
    } else if state.is_amend {
        i18n.cd_amend_btn
    } else {
        i18n.cd_create_commit_btn
    };
    let commit_enabled =
        state.is_message_valid() && state.has_files_to_commit() && !state.is_committing;

    // IDEA-style action bar with prominent amend checkbox
    let actions = Container::new(
        Row::new()
            .spacing(theme::spacing::MD)
            .align_y(Alignment::Center)
            // IDEA-style amend checkbox (prominent when available)
            .push_maybe((!state.is_committing).then(|| {
                Container::new(
                    Row::new()
                        .spacing(theme::density::CHECKBOX_SPACING)
                        .align_y(Alignment::Center)
                        .push(
                            Checkbox::new(state.is_amend)
                                .size(theme::density::CHECKBOX_SIZE)
                                .style(theme::checkbox_style())
                                .text_line_height(text::LineHeight::Relative(1.0))
                                .on_toggle(amend_checkbox_message),
                        )
                        .push(
                            Text::new(i18n.cd_amend_label)
                                .size(theme::typography::CAPTION_SIZE)
                                .font(theme::app_font())
                                .line_height(text::LineHeight::Relative(1.0))
                                .color(if state.is_amend {
                                    theme::darcula::TEXT_PRIMARY
                                } else {
                                    theme::darcula::TEXT_SECONDARY
                                }),
                        ),
                )
                .height(Length::Fixed(theme::density::STANDARD_CONTROL_HEIGHT))
                .padding([0, 8])
                .center_y(Length::Fill)
                .style(theme::panel_style(if state.is_amend {
                    Surface::Accent
                } else {
                    Surface::ToolbarField
                }))
            }))
            .push(Space::new().width(Length::Fill))
            .push(button::ghost(
                i18n.cancel,
                Some(CommitDialogMessage::CancelPressed),
            ))
            .push(button::primary(
                commit_label,
                commit_enabled.then_some(CommitDialogMessage::CommitPressed),
            )),
    )
    .padding([6, 10])
    .style(theme::frame_style(Surface::Toolbar));

    let toolbar = Container::new(
        Row::new()
            .spacing(theme::spacing::XS)
            .align_y(Alignment::Center)
            .push(Text::new(i18n.cd_title).size(14))
            .push(widgets::compact_chip::<CommitDialogMessage>(
                if state.is_amend { i18n.cd_mode_amend } else { i18n.cd_mode_new },
                BadgeTone::Neutral,
            ))
            .push(widgets::compact_chip::<CommitDialogMessage>(
                i18n.cd_file_count_fmt.replace("{}", &selected_file_count.to_string()),
                BadgeTone::Accent,
            ))
            .push(widgets::compact_chip::<CommitDialogMessage>(
                format!("+{} / -{}", selected_additions, selected_deletions),
                BadgeTone::Success,
            )),
    )
    .padding([6, 10])
    .style(theme::panel_style(Surface::Panel));

    Container::new(
        Column::new()
            .spacing(theme::spacing::SM)
            .height(Length::Fill)
            .push(toolbar)
            .push_maybe(status_panel)
            .push(
                Row::new()
                    .spacing(theme::spacing::SM)
                    .height(Length::Fill)
                    .width(Length::Fill)
                    .push(files_panel)
                    .push(diff_panel),
            )
            .push(widgets::separator_with_text(Some(i18n.cd_commit_msg_separator)))
            .push(message_panel)
            .push(actions),
    )
    .padding([6, 8])
    .width(Length::Fill)
    .height(Length::Fill)
    .style(theme::panel_style(Surface::Panel))
    .into()
}

/// Single-line status strip for commit dialog (saves vertical space vs `status_banner`).
fn build_compact_commit_status<'a, Message: 'a>(
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
                    .size(11)
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

fn build_file_row<'a>(
    state: &'a CommitDialogState,
    file: &'a Change,
) -> Element<'a, CommitDialogMessage> {
    let path = file.path.clone();
    let is_selected = state.selected_files.contains(&path);
    let is_previewed = state.previewed_file.as_deref() == Some(path.as_str());
    let status = FileStatus::from(&file.status);
    let additions = state
        .file_diff(&path)
        .map(|diff| diff.additions)
        .unwrap_or_default();
    let deletions = state
        .file_diff(&path)
        .map(|diff| diff.deletions)
        .unwrap_or_default();

    let preview_button = Button::new(
        Container::new(
            Row::new()
                .spacing(theme::spacing::XS)
                .align_y(Alignment::Center)
                .push(Text::new(status.symbol()).size(11).color(status.color()))
                .push(
                    Text::new(path.clone())
                        .size(12)
                        .width(Length::Fill)
                        .wrapping(text::Wrapping::WordOrGlyph),
                )
                .push_maybe((additions > 0).then(|| {
                    widgets::info_chip::<CommitDialogMessage>(
                        format!("+{}", additions),
                        BadgeTone::Success,
                    )
                }))
                .push_maybe((deletions > 0).then(|| {
                    widgets::info_chip::<CommitDialogMessage>(
                        format!("-{}", deletions),
                        BadgeTone::Danger,
                    )
                }))
                .push_maybe(is_previewed.then(|| {
                    widgets::info_chip::<CommitDialogMessage>("Previewing", BadgeTone::Accent)
                })),
        )
        .padding([4, 6])
        .width(Length::Fill)
        .style(theme::panel_style(if is_previewed {
            Surface::Selection
        } else {
            Surface::Editor
        })),
    )
    .width(Length::Fill)
    .style(theme::button_style(theme::ButtonTone::Ghost))
    .on_press(CommitDialogMessage::PreviewFile(path.clone()));

    Row::new()
        .spacing(theme::spacing::XS)
        .align_y(Alignment::Center)
        .push(
            Checkbox::new(is_selected)
                .size(13)
                .style(theme::checkbox_style())
                .on_toggle(move |checked| CommitDialogMessage::FileToggled(path.clone(), checked)),
        )
        .push(preview_button)
        .into()
}

fn initial_preview_path(selected_files: &[String], diff: &Diff) -> Option<String> {
    selected_files.first().cloned().or_else(|| {
        diff.files
            .iter()
            .find_map(diff_file_path)
            .map(str::to_string)
    })
}

fn diff_file_path(file: &FileDiff) -> Option<&str> {
    file.new_path.as_deref().or(file.old_path.as_deref())
}

fn normalize_editor_text(text: String) -> String {
    text.trim_end_matches('\n').to_string()
}

fn amend_checkbox_message(checked: bool) -> CommitDialogMessage {
    CommitDialogMessage::SetAmendMode(checked)
}

#[cfg(test)]
mod tests {
    use super::{amend_checkbox_message, CommitDialogMessage, CommitDialogState};
    use git_core::commit::CommitInfo;
    use git_core::diff::Diff;
    use git_core::index::{Change, ChangeStatus};

    fn sample_change(path: &str) -> Change {
        Change {
            path: path.to_string(),
            status: ChangeStatus::Modified,
            staged: true,
            unstaged: false,
            old_oid: None,
            new_oid: None,
            is_submodule: false,
            submodule_summary: None,
        }
    }

    fn sample_commit(message: &str) -> CommitInfo {
        CommitInfo {
            id: "abc123".to_string(),
            message: message.to_string(),
            author_name: "Tester".to_string(),
            author_email: "tester@example.com".to_string(),
            author_time: 0,
            committer_name: "Tester".to_string(),
            committer_email: "tester@example.com".to_string(),
            committer_time: 0,
            parent_ids: vec!["parent".to_string()],
        }
    }

    #[test]
    fn unchecking_amend_checkbox_requests_switch_back_to_new_commit_mode() {
        assert!(matches!(
            amend_checkbox_message(false),
            CommitDialogMessage::SetAmendMode(false)
        ));
    }

    #[test]
    fn leaving_amend_mode_keeps_current_selection_and_preview() {
        let staged_files = vec![sample_change("src/main.rs"), sample_change("src/lib.rs")];
        let diff = Diff {
            files: Vec::new(),
            total_additions: 0,
            total_deletions: 0,
        };
        let mut state = CommitDialogState::for_amend(
            staged_files,
            sample_commit("existing amend message"),
            &diff,
        );
        state.selected_files = vec!["src/lib.rs".to_string()];
        state.previewed_file = Some("src/lib.rs".to_string());
        state.message = "draft message".to_string();

        state.disable_amend_mode();

        assert!(!state.is_amend);
        assert!(state.commit_to_amend.is_none());
        assert_eq!(state.selected_files, vec!["src/lib.rs".to_string()]);
        assert_eq!(state.previewed_file.as_deref(), Some("src/lib.rs"));
        assert_eq!(state.message, "draft message");
    }
}
