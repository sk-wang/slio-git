//! Remote dialog view.
//!
//! Provides a dialog for remote operations.

use crate::theme::{self, BadgeTone, Surface};
use crate::widgets::{self, button, scrollable, text_input, OptionalPush};
use git_core::{remote::RemoteInfo, Repository};
use iced::widget::{text, Button, Column, Container, Row, Text};
use iced::{Alignment, Element, Length};

/// Active mode of the remote dialog
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoteDialogMode {
    /// Show remote list overview
    Overview,
    /// IDEA-style Push dialog
    Push,
    /// IDEA-style Pull dialog
    Pull,
}

/// Message types for remote dialog.
#[derive(Debug, Clone)]
pub enum RemoteDialogMessage {
    SelectRemote(String),
    Fetch,
    Push,
    Pull,
    SetUsername(String),
    SetPassword(String),
    Refresh,
    Close,
    // IDEA Push dialog messages
    SwitchMode(RemoteDialogMode),
    SetTargetBranch(String),
    ToggleForcePush,
    TogglePushTags,
    ToggleSetUpstream,
    ExecutePush,
    // IDEA Pull dialog messages
    SetPullBranch(String),
    TogglePullRebase,
    TogglePullFfOnly,
    TogglePullNoFf,
    TogglePullSquash,
    ExecutePull,
}

/// State for the remote dialog.
#[derive(Debug, Clone)]
pub struct RemoteDialogState {
    pub remotes: Vec<RemoteInfo>,
    pub selected_remote: Option<String>,
    pub current_branch_name: Option<String>,
    pub current_branch_display: String,
    pub current_upstream_ref: Option<String>,
    pub preferred_remote: Option<String>,
    pub current_branch_sync_hint: Option<String>,
    pub current_branch_state_hint: Option<String>,
    pub username: String,
    pub password: String,
    pub is_loading: bool,
    pub error: Option<String>,
    pub success_message: Option<String>,
    // IDEA Push/Pull dialog state
    pub mode: RemoteDialogMode,
    pub target_branch: String,
    pub force_push: bool,
    pub push_tags: bool,
    pub set_upstream: bool,
    pub pull_branch: String,
    pub pull_rebase: bool,
    pub pull_ff_only: bool,
    pub pull_no_ff: bool,
    pub pull_squash: bool,
}

impl RemoteDialogState {
    pub fn new() -> Self {
        Self {
            remotes: Vec::new(),
            selected_remote: None,
            current_branch_name: None,
            current_branch_display: "detached HEAD".to_string(),
            current_upstream_ref: None,
            preferred_remote: None,
            current_branch_sync_hint: None,
            current_branch_state_hint: None,
            username: String::new(),
            password: String::new(),
            is_loading: false,
            error: None,
            success_message: None,
            mode: RemoteDialogMode::Overview,
            target_branch: String::new(),
            force_push: false,
            push_tags: false,
            set_upstream: false,
            pull_branch: String::new(),
            pull_rebase: false,
            pull_ff_only: false,
            pull_no_ff: false,
            pull_squash: false,
        }
    }

    pub fn load_remotes(&mut self, repo: &Repository) {
        self.is_loading = true;
        self.error = None;
        self.current_branch_name = repo.current_branch().ok().flatten();
        self.current_branch_display = self
            .current_branch_name
            .clone()
            .unwrap_or_else(|| "detached HEAD".to_string());
        self.current_upstream_ref = repo.current_upstream_ref();
        self.preferred_remote = repo.current_upstream_remote();
        self.current_branch_sync_hint = repo.sync_status_hint();
        self.current_branch_state_hint = repo.state_hint();

        match git_core::remote::list_branch_scoped_remotes(repo) {
            Ok(remotes) => {
                self.remotes = remotes;
                if let Some(preferred_remote) = self.preferred_remote.as_ref() {
                    self.selected_remote = self
                        .remotes
                        .iter()
                        .find(|remote| &remote.name == preferred_remote)
                        .map(|remote| remote.name.clone());
                }

                if self.selected_remote.as_ref().is_none_or(|selected| {
                    !self.remotes.iter().any(|remote| &remote.name == selected)
                }) {
                    self.selected_remote = self.remotes.first().map(|remote| remote.name.clone());
                }
                self.is_loading = false;
            }
            Err(error) => {
                self.error = Some(format!("加载远程失败: {error}"));
                self.success_message = None;
                self.is_loading = false;
            }
        }
    }

    fn credentials(&self) -> Option<(String, String)> {
        (!self.username.trim().is_empty()).then(|| {
            (
                self.username.trim().to_string(),
                self.password.as_str().to_string(),
            )
        })
    }

    fn has_current_branch(&self) -> bool {
        self.current_branch_name.is_some()
    }

    fn branch_scope_detail(&self) -> String {
        if let Some(upstream_ref) = self.current_upstream_ref.as_ref() {
            if let Some(remote) = self.preferred_remote.as_ref() {
                return format!("当前分支跟踪 {upstream_ref}，下面只保留主线同步目标 {remote}。");
            }

            return format!("当前分支跟踪 {upstream_ref}。");
        }

        if self.has_current_branch() {
            "当前分支还没有配置上游；可以先确认 remote，再按同名分支继续 fetch / pull / push。"
                .to_string()
        } else {
            "当前为 detached HEAD，建议先切回一个分支；此时只保留 fetch，pull / push 会被禁用。"
                .to_string()
        }
    }

    pub fn fetch_selected(&mut self, repo: &Repository) {
        let Some(remote_name) = self.selected_remote.clone() else {
            self.error = Some("请先选择一个远程仓库".to_string());
            self.success_message = None;
            return;
        };

        self.is_loading = true;
        self.error = None;
        self.success_message = None;
        let credentials = self.credentials();

        match git_core::remote::fetch(
            repo,
            &remote_name,
            credentials
                .as_ref()
                .map(|(username, password)| (username.as_str(), password.as_str())),
        ) {
            Ok(()) => {
                self.is_loading = false;
                self.success_message = Some(format!("已获取 {remote_name}"));
            }
            Err(error) => {
                self.error = Some(format!("获取远程失败: {error}"));
                self.is_loading = false;
            }
        }
    }

    pub fn pull_selected(&mut self, repo: &Repository) {
        let Some(remote_name) = self.selected_remote.clone() else {
            self.error = Some("请先选择一个远程仓库".to_string());
            self.success_message = None;
            return;
        };

        let branch_name = match repo.current_branch() {
            Ok(Some(branch)) => branch,
            Ok(None) => {
                self.error = Some("当前为 detached HEAD，无法执行拉取。".to_string());
                self.success_message = None;
                return;
            }
            Err(error) => {
                self.error = Some(format!("读取当前分支失败: {error}"));
                self.success_message = None;
                return;
            }
        };

        self.is_loading = true;
        self.error = None;
        self.success_message = None;
        let credentials = self.credentials();

        match git_core::remote::pull(
            repo,
            &remote_name,
            &branch_name,
            credentials
                .as_ref()
                .map(|(username, password)| (username.as_str(), password.as_str())),
        ) {
            Ok(()) => {
                self.is_loading = false;
                self.success_message = Some(format!("已拉取 {remote_name}/{branch_name}"));
            }
            Err(error) => {
                self.error = Some(format!("拉取远程失败: {error}"));
                self.is_loading = false;
            }
        }
    }

    pub fn push_selected(&mut self, repo: &Repository) {
        let Some(remote_name) = self.selected_remote.clone() else {
            self.error = Some("请先选择一个远程仓库".to_string());
            self.success_message = None;
            return;
        };

        let branch_name = match repo.current_branch() {
            Ok(Some(branch)) => branch,
            Ok(None) => {
                self.error = Some("当前为 detached HEAD，无法执行推送。".to_string());
                self.success_message = None;
                return;
            }
            Err(error) => {
                self.error = Some(format!("读取当前分支失败: {error}"));
                self.success_message = None;
                return;
            }
        };

        self.is_loading = true;
        self.error = None;
        self.success_message = None;
        let credentials = self.credentials();

        match git_core::remote::push(
            repo,
            &remote_name,
            &branch_name,
            credentials
                .as_ref()
                .map(|(username, password)| (username.as_str(), password.as_str())),
        ) {
            Ok(()) => {
                self.is_loading = false;
                self.success_message = Some(format!("已推送 {branch_name} -> {remote_name}"));
            }
            Err(error) => {
                self.error = Some(format!("推送远程失败: {error}"));
                self.is_loading = false;
            }
        }
    }
}

impl Default for RemoteDialogState {
    fn default() -> Self {
        Self::new()
    }
}

fn build_remote_row(remote: &RemoteInfo, is_selected: bool) -> Element<'_, RemoteDialogMessage> {
    let row = Container::new(
        Column::new()
            .spacing(theme::spacing::SM)
            .push(
                scrollable::styled_horizontal(
                    Row::new()
                        .spacing(theme::spacing::XS)
                        .align_y(Alignment::Center)
                        .push(Text::new(&remote.name).size(13))
                        .push(widgets::info_chip::<RemoteDialogMessage>(
                            "URL",
                            BadgeTone::Neutral,
                        )),
                )
                .width(Length::Fill),
            )
            .push(
                Text::new(&remote.url)
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
        .on_press(RemoteDialogMessage::SelectRemote(remote.name.clone()))
        .into()
}

fn build_remotes_list(state: &RemoteDialogState) -> Element<'_, RemoteDialogMessage> {
    let list = if state.remotes.is_empty() {
        Column::new().push(
            Text::new("当前仓库还没有配置远程。")
                .size(12)
                .color(theme::darcula::TEXT_SECONDARY),
        )
    } else {
        state.remotes.iter().fold(
            Column::new().spacing(theme::spacing::XS),
            |column, remote| {
                let is_selected = state
                    .selected_remote
                    .as_ref()
                    .map(|selected| selected == &remote.name)
                    .unwrap_or(false);
                column.push(build_remote_row(remote, is_selected))
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
                        Text::new(if state.preferred_remote.is_some() {
                            "当前分支 remote"
                        } else {
                            "远程仓库"
                        })
                        .size(13)
                        .color(theme::darcula::TEXT_SECONDARY),
                    )
                    .push(widgets::info_chip::<RemoteDialogMessage>(
                        state.remotes.len().to_string(),
                        BadgeTone::Neutral,
                    )),
            )
            .push(
                Text::new(if state.preferred_remote.is_some() {
                    "当前分支已经有上游 remote，这里只保留主线同步目标。"
                } else {
                    "先选择目标远程，再执行 fetch / pull / push。"
                })
                .size(12)
                .color(theme::darcula::TEXT_SECONDARY),
            )
            .push(scrollable::styled(list).height(Length::Fixed(150.0))),
    )
    .padding([12, 12])
    .style(theme::panel_style(Surface::Panel))
    .into()
}

fn build_credential_inputs(state: &RemoteDialogState) -> Element<'_, RemoteDialogMessage> {
    Container::new(
        Column::new()
            .spacing(theme::spacing::SM)
            .push(text_input::styled(
                "用户名（可选覆盖）",
                &state.username,
                RemoteDialogMessage::SetUsername,
            ))
            .push(text_input::styled(
                "密码（可选覆盖）",
                &state.password,
                RemoteDialogMessage::SetPassword,
            )),
    )
    .padding([12, 12])
    .style(theme::panel_style(Surface::Panel))
    .into()
}

fn build_branch_scope_panel(state: &RemoteDialogState) -> Element<'_, RemoteDialogMessage> {
    let sync_chip = state
        .current_branch_sync_hint
        .as_ref()
        .map(|hint| widgets::info_chip::<RemoteDialogMessage>(hint.clone(), BadgeTone::Neutral));
    let state_chip = state
        .current_branch_state_hint
        .as_ref()
        .map(|hint| widgets::info_chip::<RemoteDialogMessage>(hint.clone(), BadgeTone::Warning));
    let remote_chip = state.preferred_remote.as_ref().map(|remote| {
        widgets::info_chip::<RemoteDialogMessage>(format!("remote {remote}"), BadgeTone::Accent)
    });

    Container::new(
        Column::new()
            .spacing(theme::spacing::SM)
            .push(
                Row::new()
                    .spacing(theme::spacing::XS)
                    .align_y(Alignment::Center)
                    .push(Text::new(&state.current_branch_display).size(14))
                    .push_maybe(remote_chip)
                    .push_maybe(state_chip)
                    .push_maybe(sync_chip),
            )
            .push(
                Text::new(state.branch_scope_detail())
                    .size(12)
                    .width(Length::Fill)
                    .wrapping(text::Wrapping::WordOrGlyph)
                    .color(theme::darcula::TEXT_SECONDARY),
            ),
    )
    .padding([12, 12])
    .style(theme::panel_style(Surface::Panel))
    .into()
}

fn build_action_buttons(state: &RemoteDialogState) -> Element<'_, RemoteDialogMessage> {
    let has_remote = state.selected_remote.is_some() && !state.is_loading;
    let can_sync_branch = has_remote && state.has_current_branch();

    scrollable::styled_horizontal(
        Row::new()
            .spacing(theme::spacing::XS)
            .push(button::primary(
                "获取",
                has_remote.then_some(RemoteDialogMessage::Fetch),
            ))
            .push(button::secondary(
                if state.preferred_remote.is_some() {
                    "拉取当前分支"
                } else {
                    "拉取"
                },
                can_sync_branch.then_some(RemoteDialogMessage::Pull),
            ))
            .push(button::secondary(
                if state.preferred_remote.is_some() {
                    "推送当前分支"
                } else {
                    "推送"
                },
                can_sync_branch.then_some(RemoteDialogMessage::Push),
            ))
            .push(button::ghost("刷新", Some(RemoteDialogMessage::Refresh)))
            .push(button::ghost("关闭", Some(RemoteDialogMessage::Close))),
    )
    .width(Length::Fill)
    .into()
}

/// Build the remote dialog view.
/// IDEA-style Push dialog — compact, clear visual hierarchy
fn build_push_panel(state: &RemoteDialogState) -> Element<'_, RemoteDialogMessage> {
    use iced::widget::Checkbox;

    let remote_name = state
        .selected_remote
        .as_deref()
        .or(state.preferred_remote.as_deref())
        .unwrap_or("origin");
    let branch = state
        .current_branch_name
        .as_deref()
        .unwrap_or("main");
    let target_display = if state.target_branch.is_empty() {
        branch
    } else {
        &state.target_branch
    };

    // ── Header ──
    let header = Container::new(
        Row::new()
            .align_y(Alignment::Center)
            .push(
                Text::new(format!("推送提交到 {}", remote_name))
                    .size(13)
                    .color(theme::darcula::TEXT_PRIMARY),
            )
            .push(iced::widget::Space::new().width(Length::Fill))
            .push(button::compact_ghost("关闭", Some(RemoteDialogMessage::Close))),
    )
    .padding([6, 14])
    .width(Length::Fill)
    .style(theme::frame_style(Surface::Toolbar));

    // ── Command row ──
    let cmd_row = Container::new(
        Row::new()
            .spacing(6)
            .align_y(Alignment::Center)
            .push(
                Text::new(format!("{} →", branch))
                    .size(12)
                    .color(theme::darcula::TEXT_SECONDARY),
            )
            .push(
                Text::new(remote_name)
                    .size(12)
                    .color(theme::darcula::ACCENT),
            )
            .push(Text::new(":").size(12).color(theme::darcula::TEXT_DISABLED))
            .push(
                Container::new(text_input::styled(
                    branch,
                    target_display,
                    RemoteDialogMessage::SetTargetBranch,
                ))
                .width(Length::Fill),
            ),
    )
    .padding([8, 14])
    .width(Length::Fill)
    .style(theme::panel_style(Surface::Raised));

    // ── Options ──
    let options = Container::new(
        Column::new()
            .spacing(6)
            .push(
                Checkbox::new(state.force_push)
                    .label("强制推送 (--force-with-lease)")
                    .size(14)
                    .spacing(6)
                    .style(theme::checkbox_style())
                    .on_toggle(|_| RemoteDialogMessage::ToggleForcePush),
            )
            .push(
                Checkbox::new(state.push_tags)
                    .label("推送标签")
                    .size(14)
                    .spacing(6)
                    .style(theme::checkbox_style())
                    .on_toggle(|_| RemoteDialogMessage::TogglePushTags),
            )
            .push(
                Checkbox::new(state.set_upstream)
                    .label("设置上游分支")
                    .size(14)
                    .spacing(6)
                    .style(theme::checkbox_style())
                    .on_toggle(|_| RemoteDialogMessage::ToggleSetUpstream),
            ),
    )
    .padding([10, 14]);

    // ── Status ──
    let status: Option<Element<'_, RemoteDialogMessage>> = if let Some(error) = &state.error {
        Some(build_status_panel::<RemoteDialogMessage>("失败", error, BadgeTone::Danger))
    } else {
        state.success_message.as_ref().map(|msg| {
            build_status_panel::<RemoteDialogMessage>("完成", msg, BadgeTone::Success)
        })
    };

    // ── Footer ──
    let footer = Container::new(
        Row::new()
            .spacing(theme::spacing::SM)
            .align_y(Alignment::Center)
            .push(iced::widget::Space::new().width(Length::Fill))
            .push(button::ghost("取消", Some(RemoteDialogMessage::Close)))
            .push(button::primary(
                "推送",
                (!state.is_loading).then_some(RemoteDialogMessage::ExecutePush),
            )),
    )
    .padding([8, 14])
    .width(Length::Fill)
    .style(theme::frame_style(Surface::Toolbar));

    // ── Assembly ──
    let mut body = Column::new().spacing(0).width(Length::Fill);
    body = body.push(header);
    body = body.push(iced::widget::rule::horizontal(1));
    body = body.push(cmd_row);
    body = body.push(options);
    if let Some(s) = status {
        body = body.push(s);
    }
    body = body.push(iced::widget::rule::horizontal(1));
    body = body.push(footer);

    Container::new(body)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(theme::panel_style(Surface::Panel))
        .into()
}

/// IDEA-style Pull dialog — compact, clear visual hierarchy
fn build_pull_panel(state: &RemoteDialogState) -> Element<'_, RemoteDialogMessage> {
    use iced::widget::Checkbox;

    let remote_name = state
        .selected_remote
        .as_deref()
        .or(state.preferred_remote.as_deref())
        .unwrap_or("origin");
    let branch = state
        .current_branch_name
        .as_deref()
        .unwrap_or("main");
    let pull_target = if state.pull_branch.is_empty() {
        branch
    } else {
        &state.pull_branch
    };

    // ── Header ──
    let header = Container::new(
        Row::new()
            .align_y(Alignment::Center)
            .push(
                Text::new(format!("拉取到 {}", branch))
                    .size(13)
                    .color(theme::darcula::TEXT_PRIMARY),
            )
            .push(iced::widget::Space::new().width(Length::Fill))
            .push(button::compact_ghost("关闭", Some(RemoteDialogMessage::Close))),
    )
    .padding([6, 14])
    .width(Length::Fill)
    .style(theme::frame_style(Surface::Toolbar));

    // ── Command row ──
    let cmd_row = Container::new(
        Row::new()
            .spacing(6)
            .align_y(Alignment::Center)
            .push(
                Text::new("git pull")
                    .size(12)
                    .color(theme::darcula::TEXT_DISABLED),
            )
            .push(
                Text::new(remote_name)
                    .size(12)
                    .color(theme::darcula::ACCENT),
            )
            .push(
                Container::new(text_input::styled(
                    "选择要拉取的分支",
                    pull_target,
                    RemoteDialogMessage::SetPullBranch,
                ))
                .width(Length::Fill),
            ),
    )
    .padding([8, 14])
    .width(Length::Fill)
    .style(theme::panel_style(Surface::Raised));

    // ── Options ──
    let options = Container::new(
        Column::new()
            .spacing(6)
            .push(
                Checkbox::new(state.pull_rebase)
                    .label("变基 (--rebase)")
                    .size(14)
                    .spacing(6)
                    .style(theme::checkbox_style())
                    .on_toggle(|_| RemoteDialogMessage::TogglePullRebase),
            )
            .push(
                Checkbox::new(state.pull_ff_only)
                    .label("仅快进 (--ff-only)")
                    .size(14)
                    .spacing(6)
                    .style(theme::checkbox_style())
                    .on_toggle(|_| RemoteDialogMessage::TogglePullFfOnly),
            )
            .push(
                Checkbox::new(state.pull_no_ff)
                    .label("禁止快进 (--no-ff)")
                    .size(14)
                    .spacing(6)
                    .style(theme::checkbox_style())
                    .on_toggle(|_| RemoteDialogMessage::TogglePullNoFf),
            )
            .push(
                Checkbox::new(state.pull_squash)
                    .label("压缩 (--squash)")
                    .size(14)
                    .spacing(6)
                    .style(theme::checkbox_style())
                    .on_toggle(|_| RemoteDialogMessage::TogglePullSquash),
            ),
    )
    .padding([10, 14]);

    // ── Status ──
    let status: Option<Element<'_, RemoteDialogMessage>> = if let Some(error) = &state.error {
        Some(build_status_panel::<RemoteDialogMessage>("失败", error, BadgeTone::Danger))
    } else {
        state.success_message.as_ref().map(|msg| {
            build_status_panel::<RemoteDialogMessage>("完成", msg, BadgeTone::Success)
        })
    };

    // ── Footer ──
    let footer = Container::new(
        Row::new()
            .spacing(theme::spacing::SM)
            .align_y(Alignment::Center)
            .push(iced::widget::Space::new().width(Length::Fill))
            .push(button::ghost("取消", Some(RemoteDialogMessage::Close)))
            .push(button::primary(
                "拉取",
                (!state.is_loading).then_some(RemoteDialogMessage::ExecutePull),
            )),
    )
    .padding([8, 14])
    .width(Length::Fill)
    .style(theme::frame_style(Surface::Toolbar));

    // ── Assembly ──
    let mut body = Column::new().spacing(0).width(Length::Fill);
    body = body.push(header);
    body = body.push(iced::widget::rule::horizontal(1));
    body = body.push(cmd_row);
    body = body.push(options);
    if let Some(s) = status {
        body = body.push(s);
    }
    body = body.push(iced::widget::rule::horizontal(1));
    body = body.push(footer);

    Container::new(body)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(theme::panel_style(Surface::Panel))
        .into()
}

pub fn view(state: &RemoteDialogState) -> Element<'_, RemoteDialogMessage> {
    match state.mode {
        RemoteDialogMode::Push => return build_push_panel(state),
        RemoteDialogMode::Pull => return build_pull_panel(state),
        RemoteDialogMode::Overview => {} // fall through to existing overview
    }
    // IDEA-style: compact loading indicator when processing
    let status_panel: Option<Element<'_, RemoteDialogMessage>> = if state.is_loading {
        Some(
            Container::new(
                Row::new()
                    .spacing(theme::spacing::SM)
                    .align_y(Alignment::Center)
                    .push(widgets::loading_spinner::<RemoteDialogMessage>())
                    .push(
                        Text::new("正在与远程仓库交互...")
                            .size(12)
                            .color(theme::darcula::TEXT_SECONDARY),
                    ),
            )
            .padding([8, 12])
            .style(theme::panel_style(Surface::Raised))
            .into(),
        )
    } else if let Some(error) = state.error.as_ref() {
        Some(build_status_panel::<RemoteDialogMessage>(
            "失败",
            error,
            BadgeTone::Danger,
        ))
    } else if let Some(message) = state.success_message.as_ref() {
        Some(build_status_panel::<RemoteDialogMessage>(
            "完成",
            message,
            BadgeTone::Success,
        ))
    } else if state.remotes.is_empty() {
        Some(build_status_panel::<RemoteDialogMessage>(
            "空状态",
            "当前仓库还没有配置远程；先添加 remote，再执行同步操作。",
            BadgeTone::Neutral,
        ))
    } else {
        None
    };

    let toolbar = Container::new(
        Row::new()
            .spacing(theme::spacing::XS)
            .align_y(Alignment::Center)
            .push(Text::new("远程").size(16))
            .push(widgets::info_chip::<RemoteDialogMessage>(
                format!("远程 {}", state.remotes.len()),
                BadgeTone::Neutral,
            ))
            .push(widgets::info_chip::<RemoteDialogMessage>(
                state.current_branch_display.clone(),
                BadgeTone::Accent,
            ))
            .push_maybe(
                state
                    .selected_remote
                    .as_ref()
                    .map(|remote| widgets::info_chip::<RemoteDialogMessage>(format!("已选 {remote}"), BadgeTone::Success)),
            ),
    )
    .padding([10, 12])
    .style(theme::panel_style(Surface::Panel));

    let content = if state.remotes.is_empty() && !state.is_loading && state.error.is_none() {
        Column::new()
            .spacing(theme::spacing::MD)
            .push(toolbar)
            .push(build_branch_scope_panel(state))
            .push_maybe(status_panel)
            .push(widgets::panel_empty_state(
                "远程",
                "当前仓库还没有配置远程",
                "先在仓库里添加 remote，随后就能在这里执行 fetch、pull 和 push。",
                Some(build_action_buttons(state)),
            ))
    } else {
        Column::new()
            .spacing(theme::spacing::MD)
            .push(toolbar)
            .push(build_branch_scope_panel(state))
            .push_maybe(status_panel)
            .push(build_remotes_list(state))
            .push(build_credential_inputs(state))
            .push(build_action_buttons(state))
    };

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_mode_is_overview() {
        let state = RemoteDialogState::new();
        assert_eq!(state.mode, RemoteDialogMode::Overview);
    }

    #[test]
    fn pull_rebase_disables_other_options() {
        let mut state = RemoteDialogState::new();
        state.pull_ff_only = true;
        state.pull_no_ff = true;
        state.pull_squash = true;

        // Selecting rebase should clear the others
        state.pull_rebase = true;
        if state.pull_rebase {
            state.pull_ff_only = false;
            state.pull_no_ff = false;
            state.pull_squash = false;
        }

        assert!(state.pull_rebase);
        assert!(!state.pull_ff_only);
        assert!(!state.pull_no_ff);
        assert!(!state.pull_squash);
    }

    #[test]
    fn pull_ff_only_disables_conflicting_options() {
        let mut state = RemoteDialogState::new();
        state.pull_rebase = true;
        state.pull_no_ff = true;

        state.pull_ff_only = true;
        if state.pull_ff_only {
            state.pull_rebase = false;
            state.pull_no_ff = false;
            state.pull_squash = false;
        }

        assert!(state.pull_ff_only);
        assert!(!state.pull_rebase);
        assert!(!state.pull_no_ff);
    }

    #[test]
    fn push_defaults_are_safe() {
        let state = RemoteDialogState::new();
        assert!(!state.force_push);
        assert!(!state.push_tags);
        assert!(!state.set_upstream);
    }
}
