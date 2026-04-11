//! Tag dialog view.
//!
//! Provides a dialog for tag operations.

use crate::i18n::I18n;
use crate::theme::{self, BadgeTone, Surface};
use crate::widgets::{self, button, scrollable, text_input, OptionalPush};
use git_core::{
    tag::{create_lightweight_tag, create_tag, delete_tag, list_tags, TagInfo},
    Repository,
};
use iced::widget::{text, Button, Column, Container, Row, Text};
use iced::{Alignment, Element, Length};

/// Message types for tag dialog.
#[derive(Debug, Clone)]
pub enum TagDialogMessage {
    SelectTag(String),
    CreateTag(String, String, bool),
    DeleteTag(String),
    DeleteLocalAndRemote(String),
    PushTag(String),
    DeleteRemoteTag(String),
    SetTagName(String),
    SetTarget(String),
    SetMessage(String),
    SetLightweight(bool),
    SetForceTag(bool),
    ValidateCommitRef,
    Refresh,
    Close,
}

/// State for the tag dialog.
#[derive(Debug, Clone)]
pub struct TagDialogState {
    pub tags: Vec<TagInfo>,
    pub selected_tag: Option<String>,
    pub tag_name: String,
    pub target: String,
    pub message: String,
    pub is_lightweight: bool,
    pub is_force: bool,
    pub is_loading: bool,
    pub error: Option<String>,
    pub success_message: Option<String>,
    pub validation_result: Option<String>,
}

impl TagDialogState {
    pub fn new() -> Self {
        Self {
            tags: Vec::new(),
            selected_tag: None,
            tag_name: String::new(),
            target: String::new(),
            message: String::new(),
            is_lightweight: false,
            is_force: false,
            is_loading: false,
            error: None,
            success_message: None,
            validation_result: None,
        }
    }

    pub fn load_tags(&mut self, repo: &Repository) {
        self.is_loading = true;
        self.error = None;

        match list_tags(repo) {
            Ok(tags) => {
                self.tags = tags;
                if self
                    .selected_tag
                    .as_ref()
                    .is_none_or(|selected| !self.tags.iter().any(|tag| &tag.name == selected))
                {
                    self.selected_tag = self.tags.first().map(|tag| tag.name.clone());
                }
                self.is_loading = false;
            }
            Err(error) => {
                self.error = Some(format!("Failed to load tags: {error}"));
                self.success_message = None;
                self.is_loading = false;
            }
        }
    }

    pub fn create_tag(&mut self, repo: &Repository) {
        if self.tag_name.is_empty() || self.target.is_empty() {
            self.error = Some("Tag name and target cannot be empty".to_string());
            self.success_message = None;
            return;
        }

        self.is_loading = true;
        self.error = None;
        self.success_message = None;
        let tag_name = self.tag_name.clone();

        let result = if self.is_lightweight {
            create_lightweight_tag(repo, &self.tag_name, &self.target)
        } else {
            create_tag(
                repo,
                &self.tag_name,
                &self.target,
                &self.message,
                "User",
                "user@example.com",
            )
        };

        match result {
            Ok(_) => {
                self.tag_name.clear();
                self.target.clear();
                self.message.clear();
                self.selected_tag = Some(tag_name.clone());
                self.success_message = Some(format!("Created tag {tag_name}"));
                self.load_tags(repo);
            }
            Err(error) => {
                self.error = Some(format!("Failed to create tag: {error}"));
                self.is_loading = false;
            }
        }
    }

    pub fn delete_tag(&mut self, repo: &Repository, name: String) {
        self.is_loading = true;
        self.error = None;
        self.success_message = None;

        match delete_tag(repo, &name) {
            Ok(_) => {
                if self.selected_tag.as_deref() == Some(name.as_str()) {
                    self.selected_tag = None;
                }
                self.success_message = Some(format!("Deleted tag {name}"));
                self.load_tags(repo);
            }
            Err(error) => {
                self.error = Some(format!("Failed to delete tag: {error}"));
                self.is_loading = false;
            }
        }
    }
}

impl Default for TagDialogState {
    fn default() -> Self {
        Self::new()
    }
}

fn build_tag_row<'a>(tag: &'a TagInfo, is_selected: bool, i18n: &'a I18n) -> Element<'a, TagDialogMessage> {
    let row = Container::new(
        Column::new()
            .spacing(4)
            .push(
                scrollable::styled_horizontal(
                    Row::new()
                        .spacing(theme::spacing::XS)
                        .align_y(Alignment::Center)
                        .push(Text::new(&tag.name).size(13))
                        .push(widgets::info_chip::<TagDialogMessage>(
                            if tag.message.is_some() {
                                i18n.td_annotated
                            } else {
                                i18n.td_lightweight
                            },
                            if tag.message.is_some() {
                                BadgeTone::Accent
                            } else {
                                BadgeTone::Neutral
                            },
                        )),
                )
                .width(Length::Fill),
            )
            .push(
                Text::new(if tag.target.is_empty() {
                    i18n.td_target_pending
                } else {
                    &tag.target
                })
                .size(11)
                .width(Length::Fill)
                .wrapping(text::Wrapping::WordOrGlyph)
                .color(theme::darcula::TEXT_SECONDARY),
            ),
    )
    .padding([10, 12])
    .style(theme::panel_style(if is_selected {
        Surface::Selection
    } else {
        Surface::Raised
    }));

    Button::new(row)
        .style(theme::button_style(theme::ButtonTone::Ghost))
        .on_press(TagDialogMessage::SelectTag(tag.name.clone()))
        .into()
}

fn build_tags_list<'a>(state: &'a TagDialogState, i18n: &'a I18n) -> Element<'a, TagDialogMessage> {
    let list = if state.tags.is_empty() {
        Column::new().push(
            Text::new(i18n.td_no_tags)
                .size(12)
                .color(theme::darcula::TEXT_SECONDARY),
        )
    } else {
        state
            .tags
            .iter()
            .fold(Column::new().spacing(theme::spacing::XS), |column, tag| {
                let is_selected = state
                    .selected_tag
                    .as_ref()
                    .map(|selected| selected == &tag.name)
                    .unwrap_or(false);
                column.push(build_tag_row(tag, is_selected, i18n))
            })
    };

    Container::new(
        Column::new()
            .spacing(theme::spacing::SM)
            .push(
                Row::new()
                    .spacing(theme::spacing::XS)
                    .align_y(Alignment::Center)
                    .push(
                        Text::new(i18n.td_tag_list)
                            .size(13)
                            .color(theme::darcula::TEXT_SECONDARY),
                    )
                    .push(widgets::info_chip::<TagDialogMessage>(
                        state.tags.len().to_string(),
                        BadgeTone::Neutral,
                    )),
            )
            .push(
                Text::new(i18n.td_select_hint)
                    .size(12)
                    .color(theme::darcula::TEXT_SECONDARY),
            )
            .push(scrollable::styled(list).height(Length::Fixed(150.0))),
    )
    .padding([12, 12])
    .style(theme::panel_style(Surface::Panel))
    .into()
}

fn build_tag_form<'a>(state: &'a TagDialogState, i18n: &'a I18n) -> Element<'a, TagDialogMessage> {
    // IDEA-style tag dialog layout: name / force / commit+validate / message
    let mut form = Column::new()
        .spacing(theme::spacing::SM)
        .push(widgets::section_header(
            i18n.td_create_section.to_uppercase(),
            i18n.td_create_title,
            i18n.td_create_detail,
        ))
        .push(text_input::styled(
            i18n.td_tag_name,
            &state.tag_name,
            TagDialogMessage::SetTagName,
        ))
        .push(widgets::compact_checkbox(
            state.is_force,
            i18n.force_overwrite,
            TagDialogMessage::SetForceTag,
        ))
        .push(
            Row::new()
                .spacing(theme::spacing::XS)
                .align_y(Alignment::Center)
                .push(
                    Container::new(text_input::styled(
                        i18n.td_target_placeholder,
                        &state.target,
                        TagDialogMessage::SetTarget,
                    ))
                    .width(Length::Fill),
                )
                .push(button::secondary(
                    i18n.validate_ref,
                    (!state.target.trim().is_empty())
                        .then_some(TagDialogMessage::ValidateCommitRef),
                )),
        );

    // Show validation result
    if let Some(result) = &state.validation_result {
        form = form.push(
            Text::new(result.as_str())
                .size(11)
                .color(if result.starts_with('✓') {
                    theme::darcula::SUCCESS
                } else {
                    theme::darcula::DANGER
                }),
        );
    }

    form = form
        .push(text_input::styled(
            i18n.td_tag_message,
            &state.message,
            TagDialogMessage::SetMessage,
        ))
        .push(widgets::compact_checkbox(
            state.is_lightweight,
            i18n.td_create_lightweight_btn,
            TagDialogMessage::SetLightweight,
        ));

    Container::new(form)
        .padding([12, 12])
        .style(theme::panel_style(Surface::Panel))
        .into()
}

fn build_action_buttons<'a>(state: &'a TagDialogState, i18n: &'a I18n) -> Element<'a, TagDialogMessage> {
    let can_create =
        !state.is_loading && !state.tag_name.trim().is_empty() && !state.target.trim().is_empty();

    scrollable::styled_horizontal(
        Row::new()
            .spacing(theme::spacing::XS)
            .push(button::primary(
                i18n.td_create_tag_btn,
                can_create.then(|| {
                    TagDialogMessage::CreateTag(
                        state.tag_name.clone(),
                        state.target.clone(),
                        state.is_lightweight,
                    )
                }),
            ))
            .push(button::secondary(
                i18n.td_push_remote_btn,
                state.selected_tag.clone().map(TagDialogMessage::PushTag),
            ))
            .push(button::ghost(
                i18n.td_delete_local_btn,
                state.selected_tag.clone().map(TagDialogMessage::DeleteTag),
            ))
            .push(button::ghost(
                i18n.td_delete_remote_btn,
                state
                    .selected_tag
                    .clone()
                    .map(TagDialogMessage::DeleteRemoteTag),
            ))
            .push(button::ghost(
                i18n.delete_local_and_remote,
                state
                    .selected_tag
                    .clone()
                    .map(TagDialogMessage::DeleteLocalAndRemote),
            ))
            .push(button::ghost(i18n.refresh, Some(TagDialogMessage::Refresh)))
            .push(button::ghost(i18n.close, Some(TagDialogMessage::Close))),
    )
    .width(Length::Fill)
    .into()
}

/// Build the tag dialog view.
pub fn view<'a>(state: &'a TagDialogState, i18n: &'a I18n) -> Element<'a, TagDialogMessage> {
    // IDEA-style: compact loading indicator when processing
    let status_panel: Option<Element<'_, TagDialogMessage>> = if state.is_loading {
        Some(
            Container::new(
                Row::new()
                    .spacing(theme::spacing::SM)
                    .align_y(Alignment::Center)
                    .push(widgets::loading_spinner::<TagDialogMessage>())
                    .push(
                        Text::new(i18n.td_loading)
                            .size(12)
                            .color(theme::darcula::TEXT_SECONDARY),
                    ),
            )
            .padding([8, 12])
            .style(theme::panel_style(Surface::Raised))
            .into(),
        )
    } else if let Some(error) = state.error.as_ref() {
        Some(build_status_panel::<TagDialogMessage>(
            i18n.td_status_failed,
            error,
            BadgeTone::Danger,
        ))
    } else if let Some(message) = state.success_message.as_ref() {
        Some(build_status_panel::<TagDialogMessage>(
            i18n.td_status_done,
            message,
            BadgeTone::Success,
        ))
    } else if state.tags.is_empty() {
        Some(build_status_panel::<TagDialogMessage>(
            i18n.td_status_empty,
            i18n.td_status_empty_detail,
            BadgeTone::Neutral,
        ))
    } else {
        None
    };

    let toolbar = Container::new(
        Row::new()
            .spacing(theme::spacing::XS)
            .align_y(Alignment::Center)
            .push(Text::new(i18n.td_title).size(16))
            .push(widgets::info_chip::<TagDialogMessage>(
                i18n.td_total_count_fmt.replace("{}", &state.tags.len().to_string()),
                BadgeTone::Neutral,
            ))
            .push_maybe(state.selected_tag.as_ref().map(|tag| {
                widgets::info_chip::<TagDialogMessage>(i18n.td_selected_fmt.replace("{}", tag), BadgeTone::Accent)
            }))
            .push(button::ghost(i18n.refresh, Some(TagDialogMessage::Refresh)))
            .push(button::ghost(i18n.close, Some(TagDialogMessage::Close))),
    )
    .padding([10, 12])
    .style(theme::panel_style(Surface::Panel));

    let content = Column::new()
        .spacing(theme::spacing::MD)
        .push(toolbar)
        .push_maybe(status_panel)
        .push_maybe(
            (state.tags.is_empty() && !state.is_loading && state.error.is_none()).then(|| {
                widgets::panel_empty_state(
                    i18n.td_empty_title,
                    i18n.td_empty_subtitle,
                    i18n.td_empty_detail,
                    Some(
                        button::primary(
                            i18n.td_create_tag_btn,
                            (!state.is_loading
                                && !state.tag_name.trim().is_empty()
                                && !state.target.trim().is_empty())
                            .then(|| {
                                TagDialogMessage::CreateTag(
                                    state.tag_name.clone(),
                                    state.target.clone(),
                                    state.is_lightweight,
                                )
                            }),
                        )
                        .into(),
                    ),
                )
            }),
        )
        .push(build_tags_list(state, i18n))
        .push(build_tag_form(state, i18n))
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
