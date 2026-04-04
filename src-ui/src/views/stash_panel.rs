//! Stash panel view.
//!
//! Provides a panel for stash operations.

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
                self.error = Some(format!("加载储藏失败: {error}"));
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

        match git_core::stash_save_with_options(repo, message, self.include_untracked, self.keep_index) {
            Ok(_) => {
                self.success_message = Some(if let Some(message) = message {
                    format!("已保存储藏：{message}")
                } else {
                    "已保存储藏。".to_string()
                });
                self.new_stash_message.clear();
                self.load_stashes(repo);
            }
            Err(error) => {
                self.error = Some(format!("保存储藏失败: {error}"));
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
                self.error = Some(format!("加载储藏差异失败: {e}"));
            }
        }
    }

    pub fn apply_stash_keep(&mut self, repo: &Repository, index: u32) {
        self.is_loading = true;
        self.error = None;
        self.success_message = None;

        match git_core::stash_apply(repo, index) {
            Ok(_) => {
                self.success_message = Some(format!("已应用 stash@{{{index}}}（保留在列表中）"));
                self.is_loading = false;
            }
            Err(error) => {
                self.error = Some(format!("应用储藏失败: {error}"));
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
                self.success_message = Some(format!("已应用 stash@{{{index}}}"));
                self.load_stashes(repo);
            }
            Err(error) => {
                self.error = Some(format!("应用储藏失败: {error}"));
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
                self.success_message = Some(format!("已删除 stash@{{{index}}}"));
                self.load_stashes(repo);
            }
            Err(error) => {
                self.error = Some(format!("删除储藏失败: {error}"));
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

fn build_stashes_list(state: &StashPanelState) -> Element<'_, StashPanelMessage> {
    let list = if state.stashes.is_empty() {
        Column::new().push(
            Text::new("当前没有 stash 记录。")
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
                        Text::new("储藏列表")
                            .size(13)
                            .color(theme::darcula::TEXT_SECONDARY),
                    )
                    .push(widgets::info_chip::<StashPanelMessage>(
                        state.stashes.len().to_string(),
                        BadgeTone::Neutral,
                    )),
            )
            .push(
                Text::new("选择一条 stash 后可直接应用或删除。")
                    .size(12)
                    .color(theme::darcula::TEXT_SECONDARY),
            )
            .push(scrollable::styled(list).height(Length::Fixed(220.0))),
    )
    .padding([12, 12])
    .style(theme::panel_style(Surface::Panel))
    .into()
}

fn build_stash_input(state: &StashPanelState) -> Element<'_, StashPanelMessage> {
    use iced::widget::Checkbox;
    Container::new(
        Column::new()
            .spacing(theme::spacing::SM)
            .push(widgets::section_header(
                "创建".to_uppercase(),
                "新建储藏",
                "储藏当前工作区修改，便于先切换任务再回来继续。",
            ))
            .push(text_input::styled(
                "储藏消息（可选）",
                &state.new_stash_message,
                StashPanelMessage::SetNewStashMessage,
            ))
            .push(
                Row::new()
                    .spacing(theme::spacing::MD)
                    .push(
                        Checkbox::new(state.include_untracked)
                            .label("包含未跟踪文件")
                            .size(14)
                            .style(crate::theme::checkbox_style())
                            .on_toggle(|_| StashPanelMessage::ToggleIncludeUntracked),
                    )
                    .push(
                        Checkbox::new(state.keep_index)
                            .label("保留暂存区")
                            .size(14)
                            .style(crate::theme::checkbox_style())
                            .on_toggle(|_| StashPanelMessage::SetKeepIndex(!state.keep_index)),
                    ),
            ),
    )
    .padding([12, 12])
    .style(theme::panel_style(Surface::Panel))
    .into()
}

fn build_action_buttons(state: &StashPanelState) -> Element<'_, StashPanelMessage> {
    scrollable::styled_horizontal(
        Row::new()
            .spacing(theme::spacing::XS)
            .push(button::primary(
                "保存储藏",
                (!state.is_loading).then_some(StashPanelMessage::SaveStash),
            ))
            .push(button::secondary(
                "弹出",
                state.selected_stash.map(StashPanelMessage::PopStash),
            ))
            .push(button::secondary(
                "应用",
                state.selected_stash.map(StashPanelMessage::ApplyStash),
            ))
            .push(button::ghost(
                "应用到新分支",
                state.selected_stash.map(StashPanelMessage::ShowUnstashDialog),
            ))
            .push(button::ghost(
                "丢弃",
                state.selected_stash.map(StashPanelMessage::DropStash),
            ))
            .push(button::ghost(
                "清空所有",
                (!state.stashes.is_empty()).then_some(StashPanelMessage::ClearAllStashes),
            )),
    )
    .width(Length::Fill)
    .into()
}

pub fn view(state: &StashPanelState) -> Element<'_, StashPanelMessage> {
    // IDEA-style: compact loading indicator when processing
    let status_panel: Option<Element<'_, StashPanelMessage>> = if state.is_loading {
        Some(
            Container::new(
                Row::new()
                    .spacing(theme::spacing::SM)
                    .align_y(Alignment::Center)
                    .push(widgets::loading_spinner::<StashPanelMessage>())
                    .push(
                        Text::new("正在刷新 stash 列表...")
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
            "失败",
            error,
            BadgeTone::Danger,
        ))
    } else if let Some(message) = state.success_message.as_ref() {
        Some(build_status_panel::<StashPanelMessage>(
            "完成",
            message,
            BadgeTone::Success,
        ))
    } else if state.stashes.is_empty() {
        Some(build_status_panel::<StashPanelMessage>(
            "空状态",
            "当前没有 stash 记录；可以先保存一组工作区修改再回来处理。",
            BadgeTone::Neutral,
        ))
    } else {
        None
    };

    let toolbar = Container::new(
        Row::new()
            .spacing(theme::spacing::XS)
            .align_y(Alignment::Center)
            .push(Text::new("储藏").size(16))
            .push(widgets::info_chip::<StashPanelMessage>(
                format!("总数 {}", state.stashes.len()),
                BadgeTone::Neutral,
            ))
            .push_maybe(state.selected_stash.map(|index| {
                widgets::info_chip::<StashPanelMessage>(
                    format!("已选 stash@{{{index}}}"),
                    BadgeTone::Accent,
                )
            }))
            .push(button::ghost("刷新", Some(StashPanelMessage::Refresh)))
            .push(button::ghost("关闭", Some(StashPanelMessage::Close))),
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
                    "储藏",
                    "当前还没有 stash 记录",
                    "如果你想暂存当前工作区修改，可以先填写一条消息并点击“保存储藏”。",
                    Some(
                        button::primary(
                            "保存储藏",
                            (!state.is_loading).then_some(StashPanelMessage::SaveStash),
                        )
                        .into(),
                    ),
                )
            }),
        )
        .push(build_stashes_list(state))
        .push(build_stash_input(state))
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
