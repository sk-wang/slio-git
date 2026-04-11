//! Stash panel view.
//!
//! Provides a panel for stash operations.

use crate::i18n::I18n;
use crate::theme::{self, BadgeTone, Surface};
use crate::widgets::{self, button, scrollable, text_input, OptionalPush};
use git_core::{
    stash::{list_stashes, stash_drop, stash_pop, StashInfo},
    Repository,
};
use iced::widget::{text, Button, Column, Container, Row, Text};
use iced::{Alignment, Element, Length};

/// Message types for stash panel.
#[derive(Debug, Clone)]
pub enum StashPanelMessage {
    SetNewStashMessage(String),
    SelectStash(u32),
    SaveStash,
    ApplyStash(u32),
    PopStash(u32),
    DropStash(u32),
    ToggleIncludeUntracked,
    SetKeepIndex(bool),
    TogglePreview(u32),
    ShowUnstashDialog(u32),
    SetUnstashBranchName(String),
    ConfirmUnstashAsBranch,
    CancelUnstashDialog,
    ClearAllStashes,
    Refresh,
    Close,
}

/// State for the stash panel.
#[derive(Debug, Clone)]
pub struct StashPanelState {
    pub stashes: Vec<StashInfo>,
    pub selected_stash: Option<u32>,
    pub new_stash_message: String,
    pub include_untracked: bool,
    pub keep_index: bool,
    pub unstash_branch_name: String,
    pub show_unstash_dialog: Option<u32>,
    pub preview_stash_index: Option<u32>,
    pub preview_diff_text: Option<String>,
    pub is_loading: bool,
    pub error: Option<String>,
    pub success_message: Option<String>,
}

impl StashPanelState {
    pub fn new() -> Self {
        Self {
            stashes: Vec::new(),
            selected_stash: None,
            new_stash_message: String::new(),
            include_untracked: false,
            keep_index: false,
            unstash_branch_name: String::new(),
            show_unstash_dialog: None,
            preview_stash_index: None,
            preview_diff_text: None,
            is_loading: false,
            error: None,
            success_message: None,
        }
    }

    pub fn load_stashes(&mut self, repo: &Repository) {
        self.is_loading = true;
        self.error = None;

        match list_stashes(repo) {
            Ok(stashes) => {
                self.stashes = stashes;
                if self.selected_stash.is_none_or(|selected| {
                    !self.stashes.iter().any(|stash| stash.index == selected)
                }) {
                    self.selected_stash = self.stashes.first().map(|stash| stash.index);
                }
                self.is_loading = false;
            }
            Err(error) => {
                self.error = Some(format!("Failed to load stashes: {error}"));
                self.success_message = None;
                self.is_loading = false;
            }
        }
    }

    pub fn save_stash(&mut self, repo: &Repository) {
        self.is_loading = true;
        self.error = None;
        self.success_message = None;

        let message = if self.new_stash_message.is_empty() {
            None
        } else {
            Some(self.new_stash_message.as_str())
        };

        match git_core::stash_save_with_options(
            repo,
            message,
            self.include_untracked,
            self.keep_index,
        ) {
            Ok(_) => {
                self.success_message = Some(if let Some(message) = message {
                    format!("Stash saved: {message}")
                } else {
                    "Stash saved.".to_string()
                });
                self.new_stash_message.clear();
                self.load_stashes(repo);
            }
            Err(error) => {
                self.error = Some(format!("Failed to save stash: {error}"));
                self.is_loading = false;
            }
        }
    }

    pub fn toggle_preview(&mut self, repo: &Repository, index: u32) {
        if self.preview_stash_index == Some(index) {
            self.preview_stash_index = None;
            self.preview_diff_text = None;
            return;
        }
        match git_core::stash_diff(repo, index) {
            Ok(diff) => {
                self.preview_stash_index = Some(index);
                self.preview_diff_text = Some(diff);
            }
            Err(e) => {
                self.error = Some(format!("Failed to load stash diff: {e}"));
            }
        }
    }

    pub fn apply_stash_keep(&mut self, repo: &Repository, index: u32) {
        self.is_loading = true;
        self.error = None;
        self.success_message = None;

        match git_core::stash_apply(repo, index) {
            Ok(_) => {
                self.success_message = Some(format!("Applied stash@{{{index}}} (kept in list)"));
                self.is_loading = false;
            }
            Err(error) => {
                self.error = Some(format!("Failed to apply stash: {error}"));
                self.is_loading = false;
            }
        }
    }

    pub fn apply_stash(&mut self, repo: &Repository, index: u32) {
        self.is_loading = true;
        self.error = None;
        self.success_message = None;

        match stash_pop(repo, index) {
            Ok(_) => {
                self.success_message = Some(format!("Applied stash@{{{index}}}"));
                self.load_stashes(repo);
            }
            Err(error) => {
                self.error = Some(format!("Failed to apply stash: {error}"));
                self.is_loading = false;
            }
        }
    }

    pub fn drop_stash(&mut self, repo: &Repository, index: u32) {
        self.is_loading = true;
        self.error = None;
        self.success_message = None;

        match stash_drop(repo, index) {
            Ok(_) => {
                if self.selected_stash == Some(index) {
                    self.selected_stash = None;
                }
                self.success_message = Some(format!("Dropped stash@{{{index}}}"));
                self.load_stashes(repo);
            }
            Err(error) => {
                self.error = Some(format!("Failed to drop stash: {error}"));
                self.is_loading = false;
            }
        }
    }
}

impl Default for StashPanelState {
    fn default() -> Self {
        Self::new()
    }
}

fn build_stash_row(stash: &StashInfo, is_selected: bool) -> Element<'_, StashPanelMessage> {
    let mut meta_parts = Vec::new();
    if !stash.branch.is_empty() {
        meta_parts.push(stash.branch.clone());
    }
    if let Some(ts) = stash.timestamp {
        let dt = chrono::DateTime::from_timestamp(ts, 0);
        if let Some(dt) = dt {
            meta_parts.push(dt.format("%m-%d %H:%M").to_string());
        }
    }
    let meta_line = meta_parts.join(" · ");

    let row = Container::new(
        Column::new()
            .spacing(2)
            .push(
                Row::new()
                    .spacing(theme::spacing::XS)
                    .align_y(Alignment::Center)
                    .push(Text::new(format!("stash@{{{}}}", stash.index)).size(12))
                    .push(widgets::info_chip::<StashPanelMessage>(
                        &stash.oid[..stash.oid.len().min(8)],
                        BadgeTone::Neutral,
                    )),
            )
            .push(
                Text::new(&stash.message)
                    .size(11)
                    .width(Length::Fill)
                    .wrapping(text::Wrapping::WordOrGlyph)
                    .color(theme::darcula::TEXT_PRIMARY),
            )
            .push(
                Text::new(meta_line)
                    .size(10)
                    .color(theme::darcula::TEXT_DISABLED),
            ),
    )
    .padding([6, 10])
    .style(theme::panel_style(if is_selected {
        Surface::Selection
    } else {
        Surface::Raised
    }));

    Button::new(row)
        .style(theme::button_style(theme::ButtonTone::Ghost))
        .on_press(StashPanelMessage::SelectStash(stash.index))
        .into()
}

fn build_stashes_list<'a>(state: &'a StashPanelState, i18n: &'a I18n) -> Element<'a, StashPanelMessage> {
    let list = if state.stashes.is_empty() {
        Column::new().push(
            Text::new(i18n.sp_no_stashes)
                .size(12)
                .color(theme::darcula::TEXT_SECONDARY),
        )
    } else {
        state.stashes.iter().fold(
            Column::new().spacing(theme::spacing::XS),
            |column, stash| {
                let is_selected = state.selected_stash == Some(stash.index);
                column.push(build_stash_row(stash, is_selected))
            },
        )
    };

    Container::new(
        Column::new()
            .spacing(theme::spacing::SM)
            .push(
                Row::new()
                    .spacing(theme::spacing::XS)
                    .align_y(Alignment::Center)
                    .push(
                        Text::new(i18n.sp_stash_list)
                            .size(13)
                            .color(theme::darcula::TEXT_SECONDARY),
                    )
                    .push(widgets::info_chip::<StashPanelMessage>(
                        state.stashes.len().to_string(),
                        BadgeTone::Neutral,
                    )),
            )
            .push(
                Text::new(i18n.sp_select_hint)
                    .size(12)
                    .color(theme::darcula::TEXT_SECONDARY),
            )
            .push(scrollable::styled(list).height(Length::Fixed(220.0))),
    )
    .padding([12, 12])
    .style(theme::panel_style(Surface::Panel))
    .into()
}

fn build_stash_input<'a>(state: &'a StashPanelState, i18n: &'a I18n) -> Element<'a, StashPanelMessage> {
    Container::new(
        Column::new()
            .spacing(theme::spacing::SM)
            .push(widgets::section_header(
                i18n.sp_create_section.to_uppercase(),
                i18n.sp_create_title,
                i18n.sp_create_detail,
            ))
            .push(text_input::styled(
                i18n.sp_message_placeholder,
                &state.new_stash_message,
                StashPanelMessage::SetNewStashMessage,
            ))
            .push(
                Row::new()
                    .spacing(theme::spacing::MD)
                    .push(widgets::compact_checkbox(
                        state.include_untracked,
                        i18n.sp_include_untracked,
                        |_| StashPanelMessage::ToggleIncludeUntracked,
                    ))
                    .push(widgets::compact_checkbox(
                        state.keep_index,
                        i18n.sp_keep_index,
                        |_| StashPanelMessage::SetKeepIndex(!state.keep_index),
                    )),
            ),
    )
    .padding([12, 12])
    .style(theme::panel_style(Surface::Panel))
    .into()
}

fn build_action_buttons<'a>(state: &'a StashPanelState, i18n: &'a I18n) -> Element<'a, StashPanelMessage> {
    scrollable::styled_horizontal(
        Row::new()
            .spacing(theme::spacing::XS)
            .push(button::primary(
                i18n.sp_save_stash_btn,
                (!state.is_loading).then_some(StashPanelMessage::SaveStash),
            ))
            .push(button::secondary(
                i18n.sp_pop_btn,
                state.selected_stash.map(StashPanelMessage::PopStash),
            ))
            .push(button::secondary(
                i18n.sp_apply_btn,
                state.selected_stash.map(StashPanelMessage::ApplyStash),
            ))
            .push(button::ghost(
                i18n.sp_apply_to_branch_btn,
                state
                    .selected_stash
                    .map(StashPanelMessage::ShowUnstashDialog),
            ))
            .push(button::ghost(
                i18n.sp_drop_btn,
                state.selected_stash.map(StashPanelMessage::DropStash),
            ))
            .push(button::ghost(
                i18n.sp_clear_all_btn,
                (!state.stashes.is_empty()).then_some(StashPanelMessage::ClearAllStashes),
            )),
    )
    .width(Length::Fill)
    .into()
}

pub fn view<'a>(state: &'a StashPanelState, i18n: &'a I18n) -> Element<'a, StashPanelMessage> {
    // IDEA-style: compact loading indicator when processing
    let status_panel: Option<Element<'_, StashPanelMessage>> = if state.is_loading {
        Some(
            Container::new(
                Row::new()
                    .spacing(theme::spacing::SM)
                    .align_y(Alignment::Center)
                    .push(widgets::loading_spinner::<StashPanelMessage>())
                    .push(
                        Text::new(i18n.sp_loading)
                            .size(12)
                            .color(theme::darcula::TEXT_SECONDARY),
                    ),
            )
            .padding([8, 12])
            .style(theme::panel_style(Surface::Raised))
            .into(),
        )
    } else if let Some(error) = state.error.as_ref() {
        Some(build_status_panel::<StashPanelMessage>(
            i18n.sp_status_failed,
            error,
            BadgeTone::Danger,
        ))
    } else if let Some(message) = state.success_message.as_ref() {
        Some(build_status_panel::<StashPanelMessage>(
            i18n.sp_status_done,
            message,
            BadgeTone::Success,
        ))
    } else if state.stashes.is_empty() {
        Some(build_status_panel::<StashPanelMessage>(
            i18n.sp_status_empty,
            i18n.sp_status_empty_detail,
            BadgeTone::Neutral,
        ))
    } else {
        None
    };

    let toolbar = Container::new(
        Row::new()
            .spacing(theme::spacing::XS)
            .align_y(Alignment::Center)
            .push(Text::new(i18n.sp_title).size(16))
            .push(widgets::info_chip::<StashPanelMessage>(
                i18n.sp_total_count_fmt.replace("{}", &state.stashes.len().to_string()),
                BadgeTone::Neutral,
            ))
            .push_maybe(state.selected_stash.map(|index| {
                widgets::info_chip::<StashPanelMessage>(
                    i18n.sp_selected_fmt.replace("{}", &index.to_string()),
                    BadgeTone::Accent,
                )
            }))
            .push(button::ghost(i18n.refresh, Some(StashPanelMessage::Refresh)))
            .push(button::ghost(i18n.close, Some(StashPanelMessage::Close))),
    )
    .padding([10, 12])
    .style(theme::panel_style(Surface::Panel));

    let content = Column::new()
        .spacing(theme::spacing::MD)
        .push(toolbar)
        .push_maybe(status_panel)
        .push_maybe(
            (state.stashes.is_empty() && !state.is_loading && state.error.is_none()).then(|| {
                widgets::panel_empty_state(
                    i18n.sp_empty_title,
                    i18n.sp_empty_subtitle,
                    i18n.sp_empty_detail,
                    Some(
                        button::primary(
                            i18n.sp_save_stash_btn,
                            (!state.is_loading).then_some(StashPanelMessage::SaveStash),
                        )
                        .into(),
                    ),
                )
            }),
        )
        .push(build_stashes_list(state, i18n))
        .push(build_stash_input(state, i18n))
        .push(build_action_buttons(state, i18n));

    Container::new(scrollable::styled(content).height(Length::Fill))
        .padding([10, 12])
        .width(Length::Fill)
        .height(Length::Fill)
        .style(theme::panel_style(Surface::Panel))
        .into()
}

fn build_status_panel<'a, Message: 'a>(
    label: impl Into<String>,
    detail: impl Into<String>,
    tone: BadgeTone,
) -> Element<'a, Message> {
    widgets::status_banner(label, detail, tone)
}
