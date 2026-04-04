//! Tag dialog view.
//!
//! Provides a dialog for tag operations.

use crate::theme::{self, BadgeTone, Surface};
use crate::widgets::{self, button, scrollable, text_input, OptionalPush};
use git_core::{
    tag::{create_lightweight_tag, create_tag, delete_tag, list_tags, TagInfo},
    Repository,
};
use iced::widget::{text, Button, Checkbox, Column, Container, Row, Text};
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
                self.error = Some(format!("加载标签失败: {error}"));
                self.success_message = None;
                self.is_loading = false;
            }
        }
    }

    pub fn create_tag(&mut self, repo: &Repository) {
        if self.tag_name.is_empty() || self.target.is_empty() {
            self.error = Some("标签名称和目标不能为空".to_string());
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
                self.success_message = Some(format!("已创建标签 {tag_name}"));
                self.load_tags(repo);
            }
            Err(error) => {
                self.error = Some(format!("创建标签失败: {error}"));
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
                self.success_message = Some(format!("已删除标签 {name}"));
                self.load_tags(repo);
            }
            Err(error) => {
                self.error = Some(format!("删除标签失败: {error}"));
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

fn build_tag_row(tag: &TagInfo, is_selected: bool) -> Element<'_, TagDialogMessage> {
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
                                "注释"
                            } else {
                                "轻量"
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
                    "目标待解析"
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

fn build_tags_list(state: &TagDialogState) -> Element<'_, TagDialogMessage> {
    let list = if state.tags.is_empty() {
        Column::new().push(
            Text::new("当前仓库还没有任何标签。")
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
                column.push(build_tag_row(tag, is_selected))
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
                        Text::new("标签列表")
                            .size(13)
                            .color(theme::darcula::TEXT_SECONDARY),
                    )
                    .push(widgets::info_chip::<TagDialogMessage>(
                        state.tags.len().to_string(),
                        BadgeTone::Neutral,
                    )),
            )
            .push(
                Text::new("选择现有标签后可直接删除，也可在下方表单创建新标签。")
                    .size(12)
                    .color(theme::darcula::TEXT_SECONDARY),
            )
            .push(scrollable::styled(list).height(Length::Fixed(150.0))),
    )
    .padding([12, 12])
    .style(theme::panel_style(Surface::Panel))
    .into()
}

fn build_tag_form(state: &TagDialogState) -> Element<'_, TagDialogMessage> {
    // IDEA-style tag dialog layout: name / force / commit+validate / message
    let mut form = Column::new()
        .spacing(theme::spacing::SM)
        .push(widgets::section_header(
            "创建".to_uppercase(),
            "新建标签",
            "支持轻量标签与注释标签两种模式。",
        ))
        .push(text_input::styled(
            "标签名称",
            &state.tag_name,
            TagDialogMessage::SetTagName,
        ))
        .push(
            Checkbox::new(state.is_force)
                .label("强制覆盖已有标签")
                .size(13)
                .style(theme::checkbox_style())
                .on_toggle(TagDialogMessage::SetForceTag),
        )
        .push(
            Row::new()
                .spacing(theme::spacing::XS)
                .align_y(Alignment::Center)
                .push(
                    Container::new(text_input::styled(
                        "目标 commit（HEAD 或 hash）",
                        &state.target,
                        TagDialogMessage::SetTarget,
                    ))
                    .width(Length::Fill),
                )
                .push(button::secondary(
                    "验证",
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
            "标签消息（仅注释标签会使用）",
            &state.message,
            TagDialogMessage::SetMessage,
        ))
        .push(
            Checkbox::new(state.is_lightweight)
                .label("创建轻量标签")
                .size(13)
                .style(theme::checkbox_style())
                .on_toggle(TagDialogMessage::SetLightweight),
        );

    Container::new(form)
        .padding([12, 12])
        .style(theme::panel_style(Surface::Panel))
        .into()
}

fn build_action_buttons(state: &TagDialogState) -> Element<'_, TagDialogMessage> {
    let can_create =
        !state.is_loading && !state.tag_name.trim().is_empty() && !state.target.trim().is_empty();

    scrollable::styled_horizontal(
        Row::new()
            .spacing(theme::spacing::XS)
            .push(button::primary(
                "创建标签",
                can_create.then(|| {
                    TagDialogMessage::CreateTag(
                        state.tag_name.clone(),
                        state.target.clone(),
                        state.is_lightweight,
                    )
                }),
            ))
            .push(button::secondary(
                "推送到远程",
                state.selected_tag.clone().map(TagDialogMessage::PushTag),
            ))
            .push(button::ghost(
                "删除本地",
                state.selected_tag.clone().map(TagDialogMessage::DeleteTag),
            ))
            .push(button::ghost(
                "删除远程",
                state
                    .selected_tag
                    .clone()
                    .map(TagDialogMessage::DeleteRemoteTag),
            ))
            .push(button::ghost(
                "删除本地和远程",
                state
                    .selected_tag
                    .clone()
                    .map(TagDialogMessage::DeleteLocalAndRemote),
            ))
            .push(button::ghost("刷新", Some(TagDialogMessage::Refresh)))
            .push(button::ghost("关闭", Some(TagDialogMessage::Close))),
    )
    .width(Length::Fill)
    .into()
}

/// Build the tag dialog view.
pub fn view(state: &TagDialogState) -> Element<'_, TagDialogMessage> {
    // IDEA-style: compact loading indicator when processing
    let status_panel: Option<Element<'_, TagDialogMessage>> = if state.is_loading {
        Some(
            Container::new(
                Row::new()
                    .spacing(theme::spacing::SM)
                    .align_y(Alignment::Center)
                    .push(widgets::loading_spinner::<TagDialogMessage>())
                    .push(
                        Text::new("正在刷新标签列表...")
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
            "失败",
            error,
            BadgeTone::Danger,
        ))
    } else if let Some(message) = state.success_message.as_ref() {
        Some(build_status_panel::<TagDialogMessage>(
            "完成",
            message,
            BadgeTone::Success,
        ))
    } else if state.tags.is_empty() {
        Some(build_status_panel::<TagDialogMessage>(
            "空状态",
            "当前仓库还没有标签；填写名称与目标后即可创建第一条标签。",
            BadgeTone::Neutral,
        ))
    } else {
        None
    };

    let toolbar = Container::new(
        Row::new()
            .spacing(theme::spacing::XS)
            .align_y(Alignment::Center)
            .push(Text::new("标签").size(16))
            .push(widgets::info_chip::<TagDialogMessage>(
                format!("总数 {}", state.tags.len()),
                BadgeTone::Neutral,
            ))
            .push_maybe(
                state
                    .selected_tag
                    .as_ref()
                    .map(|tag| widgets::info_chip::<TagDialogMessage>(format!("已选 {tag}"), BadgeTone::Accent)),
            )
            .push(button::ghost("刷新", Some(TagDialogMessage::Refresh)))
            .push(button::ghost("关闭", Some(TagDialogMessage::Close))),
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
                    "标签",
                    "当前仓库还没有标签",
                    "填写名称和目标后即可创建第一条标签；创建后也可以回到上方列表直接删除。",
                    Some(
                        button::primary(
                            "创建标签",
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
        .push(build_tags_list(state))
        .push(build_tag_form(state))
        .push(build_action_buttons(state));

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
