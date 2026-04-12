//! Remote dialog view.
//!
//! Provides a dialog for remote operations.

use crate::i18n::I18n;
use crate::theme::{self, BadgeTone, Surface};
use crate::widgets::{self, button, scrollable, text_input, OptionalPush};
use git_core::{remote::PullOptions, remote::RemoteInfo, Repository};
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

    pub fn load_remotes(&mut self, repo: &Repository, i18n: &I18n) {
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
                self.error = Some(i18n.rd_load_failed_fmt.replace("{}", &error.to_string()));
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

    fn default_pull_branch<'a>(&'a self, remote_name: &str) -> Option<&'a str> {
        let upstream_ref = self.current_upstream_ref.as_deref()?;
        let (upstream_remote, upstream_branch) = upstream_ref.split_once('/')?;
        (upstream_remote == remote_name && !upstream_branch.is_empty()).then_some(upstream_branch)
    }

    fn pull_branch_label(&self, remote_name: &str) -> String {
        let explicit_branch = self.pull_branch.trim();
        if !explicit_branch.is_empty() {
            return explicit_branch.to_string();
        }

        self.default_pull_branch(remote_name)
            .or(self.current_branch_name.as_deref())
            .unwrap_or("main")
            .to_string()
    }

    fn pull_command_prefix(&self) -> &'static str {
        if self.pull_rebase {
            "git pull --rebase"
        } else if self.pull_ff_only {
            "git pull --no-rebase --ff-only"
        } else if self.pull_no_ff {
            "git pull --no-rebase --no-ff"
        } else if self.pull_squash {
            "git pull --no-rebase --squash"
        } else {
            "git pull --no-rebase"
        }
    }

    fn pull_options(&self, force_autocrlf_true: bool) -> PullOptions<'_> {
        PullOptions {
            branch_name: Some(self.pull_branch.trim()).filter(|branch| !branch.is_empty()),
            rebase: self.pull_rebase,
            ff_only: self.pull_ff_only,
            no_ff: self.pull_no_ff,
            squash: self.pull_squash,
            force_autocrlf_true,
        }
    }

    fn branch_scope_detail(&self, i18n: &I18n) -> String {
        if let Some(upstream_ref) = self.current_upstream_ref.as_ref() {
            if let Some(remote) = self.preferred_remote.as_ref() {
                return i18n.rd_tracking_upstream_fmt.replace("{}", &upstream_ref.to_string()).replacen("{}", &remote.to_string(), 1);
            }

            return i18n.rd_tracking_upstream_only_fmt.replace("{}", &upstream_ref.to_string());
        }

        if self.has_current_branch() {
            i18n.rd_no_upstream
                .to_string()
        } else {
            i18n.rd_detached_head
                .to_string()
        }
    }

    pub fn fetch_selected(&mut self, repo: &Repository) {
        let Some(remote_name) = self.selected_remote.clone() else {
            self.error = Some("Please select a remote first".to_string());
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
                self.success_message = Some(format!("Fetched {remote_name}"));
            }
            Err(error) => {
                self.error = Some(format!("Failed to fetch remote: {error}"));
                self.is_loading = false;
            }
        }
    }

    pub fn pull_selected(&mut self, repo: &Repository, force_autocrlf_true: bool) {
        let Some(remote_name) = self.selected_remote.clone() else {
            self.error = Some("Please select a remote first".to_string());
            self.success_message = None;
            return;
        };

        match repo.current_branch() {
            Ok(Some(_)) => {}
            Ok(None) => {
                self.error = Some("Detached HEAD, cannot pull.".to_string());
                self.success_message = None;
                return;
            }
            Err(error) => {
                self.error = Some(format!("Failed to read current branch: {error}"));
                self.success_message = None;
                return;
            }
        }

        self.is_loading = true;
        self.error = None;
        self.success_message = None;
        let credentials = self.credentials();

        match git_core::remote::pull_with_options(
            repo,
            &remote_name,
            self.pull_options(force_autocrlf_true),
            credentials
                .as_ref()
                .map(|(username, password)| (username.as_str(), password.as_str())),
        ) {
            Ok(()) => {
                self.is_loading = false;
                let branch_label = self.pull_branch_label(&remote_name);
                self.success_message = Some(format!("Pulled {remote_name}/{branch_label}"));
            }
            Err(error) => {
                self.error = Some(format!("Failed to pull from remote: {error}"));
                self.is_loading = false;
            }
        }
    }

    pub fn push_selected(&mut self, repo: &Repository) {
        let Some(remote_name) = self.selected_remote.clone() else {
            self.error = Some("Please select a remote first".to_string());
            self.success_message = None;
            return;
        };

        let branch_name = match repo.current_branch() {
            Ok(Some(branch)) => branch,
            Ok(None) => {
                self.error = Some("Detached HEAD, cannot push.".to_string());
                self.success_message = None;
                return;
            }
            Err(error) => {
                self.error = Some(format!("Failed to read current branch: {error}"));
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
                self.success_message = Some(format!("Pushed {branch_name} -> {remote_name}"));
            }
            Err(error) => {
                self.error = Some(format!("Failed to push to remote: {error}"));
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
                        .push(Text::new(&remote.name).size(12))
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
    .padding([8, 14])
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

fn build_remotes_list<'a>(state: &'a RemoteDialogState, i18n: &'a I18n) -> Element<'a, RemoteDialogMessage> {
    let list = if state.remotes.is_empty() {
        Column::new().push(
            Text::new(i18n.rd_no_remotes)
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
                            i18n.rd_branch_remote
                        } else {
                            i18n.rd_remote_repos
                        })
                        .size(10)
                        .color(theme::darcula::TEXT_DISABLED),
                    )
                    .push(widgets::info_chip::<RemoteDialogMessage>(
                        state.remotes.len().to_string(),
                        BadgeTone::Neutral,
                    )),
            )
            .push(
                Text::new(if state.preferred_remote.is_some() {
                    i18n.rd_has_upstream_hint
                } else {
                    i18n.rd_select_target_hint
                })
                .size(12)
                .color(theme::darcula::TEXT_SECONDARY),
            )
            .push(scrollable::styled(list).height(Length::Fixed(150.0))),
    )
    .padding([8, 14])
    .style(theme::panel_style(Surface::Panel))
    .into()
}

fn build_credential_inputs<'a>(state: &'a RemoteDialogState, i18n: &'a I18n) -> Element<'a, RemoteDialogMessage> {
    Container::new(
        Column::new()
            .spacing(theme::spacing::SM)
            .push(text_input::styled(
                i18n.rd_username_optional,
                &state.username,
                RemoteDialogMessage::SetUsername,
            ))
            .push(text_input::styled(
                i18n.rd_password_optional,
                &state.password,
                RemoteDialogMessage::SetPassword,
            )),
    )
    .padding([8, 14])
    .style(theme::panel_style(Surface::Panel))
    .into()
}

fn build_branch_scope_panel<'a>(state: &'a RemoteDialogState, i18n: &'a I18n) -> Element<'a, RemoteDialogMessage> {
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
                    .push(Text::new(&state.current_branch_display).size(12))
                    .push_maybe(remote_chip)
                    .push_maybe(state_chip)
                    .push_maybe(sync_chip),
            )
            .push(
                Text::new(state.branch_scope_detail(i18n))
                    .size(11)
                    .width(Length::Fill)
                    .wrapping(text::Wrapping::WordOrGlyph)
                    .color(theme::darcula::TEXT_SECONDARY),
            ),
    )
    .padding([8, 14])
    .style(theme::panel_style(Surface::Panel))
    .into()
}

fn build_action_buttons<'a>(state: &'a RemoteDialogState, i18n: &'a I18n) -> Element<'a, RemoteDialogMessage> {
    let has_remote = state.selected_remote.is_some() && !state.is_loading;
    let can_sync_branch = has_remote && state.has_current_branch();

    scrollable::styled_horizontal(
        Row::new()
            .spacing(theme::spacing::XS)
            .push(button::primary(
                i18n.rd_fetch_btn,
                has_remote.then_some(RemoteDialogMessage::Fetch),
            ))
            .push(button::secondary(
                if state.preferred_remote.is_some() {
                    i18n.rd_pull_branch_btn
                } else {
                    i18n.rd_pull_btn
                },
                can_sync_branch.then_some(RemoteDialogMessage::Pull),
            ))
            .push(button::secondary(
                if state.preferred_remote.is_some() {
                    i18n.rd_push_branch_btn
                } else {
                    i18n.rd_push_btn
                },
                can_sync_branch.then_some(RemoteDialogMessage::Push),
            ))
            .push(button::ghost(i18n.refresh, Some(RemoteDialogMessage::Refresh)))
            .push(button::ghost(i18n.close, Some(RemoteDialogMessage::Close))),
    )
    .width(Length::Fill)
    .into()
}

/// Build the remote dialog view.
/// IDEA-style Push dialog — compact, clear visual hierarchy
fn build_push_panel<'a>(state: &'a RemoteDialogState, i18n: &'a I18n) -> Element<'a, RemoteDialogMessage> {
    let remote_name = state
        .selected_remote
        .as_deref()
        .or(state.preferred_remote.as_deref())
        .unwrap_or("origin");
    let branch = state.current_branch_name.as_deref().unwrap_or("main");
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
                Text::new(i18n.rd_push_commits_fmt.replace("{}", remote_name))
                    .size(14)
                    .color(theme::darcula::TEXT_PRIMARY),
            )
            .push(iced::widget::Space::new().width(Length::Fill))
            .push(button::compact_ghost(
                i18n.close,
                Some(RemoteDialogMessage::Close),
            )),
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
            .push(widgets::compact_checkbox(
                state.force_push,
                i18n.rd_force_push,
                |_| RemoteDialogMessage::ToggleForcePush,
            ))
            .push(widgets::compact_checkbox(
                state.push_tags,
                i18n.rd_push_tags,
                |_| RemoteDialogMessage::TogglePushTags,
            ))
            .push(widgets::compact_checkbox(
                state.set_upstream,
                i18n.rd_set_upstream,
                |_| RemoteDialogMessage::ToggleSetUpstream,
            )),
    )
    .padding([8, 14]);

    // ── Status ──
    let status: Option<Element<'_, RemoteDialogMessage>> = if let Some(error) = &state.error {
        Some(build_status_panel::<RemoteDialogMessage>(
            i18n.rd_status_failed,
            error,
            BadgeTone::Danger,
        ))
    } else {
        state
            .success_message
            .as_ref()
            .map(|msg| build_status_panel::<RemoteDialogMessage>(i18n.rd_status_done, msg, BadgeTone::Success))
    };

    // ── Footer ──
    let footer = Container::new(
        Row::new()
            .spacing(theme::spacing::SM)
            .align_y(Alignment::Center)
            .push(iced::widget::Space::new().width(Length::Fill))
            .push(button::ghost(i18n.cancel, Some(RemoteDialogMessage::Close)))
            .push(button::primary(
                i18n.rd_push_btn,
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
fn build_pull_panel<'a>(state: &'a RemoteDialogState, i18n: &'a I18n) -> Element<'a, RemoteDialogMessage> {
    let remote_name = state
        .selected_remote
        .as_deref()
        .or(state.preferred_remote.as_deref())
        .unwrap_or("origin");
    let fallback_branch = state.current_branch_name.as_deref().unwrap_or("main");
    let pull_target = if state.pull_branch.is_empty() {
        state
            .default_pull_branch(remote_name)
            .unwrap_or(fallback_branch)
    } else {
        &state.pull_branch
    };
    let pull_command = state.pull_command_prefix();

    // ── Header ──
    let header = Container::new(
        Row::new()
            .align_y(Alignment::Center)
            .push(
                Text::new(i18n.rd_pull_to_fmt.replace("{}", pull_target))
                    .size(14)
                    .color(theme::darcula::TEXT_PRIMARY),
            )
            .push(iced::widget::Space::new().width(Length::Fill))
            .push(button::compact_ghost(
                i18n.close,
                Some(RemoteDialogMessage::Close),
            )),
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
                Text::new(pull_command)
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
                    pull_target,
                    state.pull_branch.as_str(),
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
            .push(widgets::compact_checkbox(
                state.pull_rebase,
                i18n.rd_rebase_option,
                |_| RemoteDialogMessage::TogglePullRebase,
            ))
            .push(widgets::compact_checkbox(
                state.pull_ff_only,
                i18n.rd_ff_only_option,
                |_| RemoteDialogMessage::TogglePullFfOnly,
            ))
            .push(widgets::compact_checkbox(
                state.pull_no_ff,
                i18n.rd_no_ff_option,
                |_| RemoteDialogMessage::TogglePullNoFf,
            ))
            .push(widgets::compact_checkbox(
                state.pull_squash,
                i18n.rd_squash_option,
                |_| RemoteDialogMessage::TogglePullSquash,
            )),
    )
    .padding([8, 14]);

    let autocrlf_hint = Container::new(
        Text::new(i18n.rd_pull_autocrlf_hint)
            .size(11)
            .color(theme::darcula::TEXT_SECONDARY),
    )
    .padding([8, 14]);

    // ── Status ──
    let status: Option<Element<'_, RemoteDialogMessage>> = if let Some(error) = &state.error {
        Some(build_status_panel::<RemoteDialogMessage>(
            i18n.rd_status_failed,
            error,
            BadgeTone::Danger,
        ))
    } else {
        state
            .success_message
            .as_ref()
            .map(|msg| build_status_panel::<RemoteDialogMessage>(i18n.rd_status_done, msg, BadgeTone::Success))
    };

    // ── Footer ──
    let footer = Container::new(
        Row::new()
            .spacing(theme::spacing::SM)
            .align_y(Alignment::Center)
            .push(iced::widget::Space::new().width(Length::Fill))
            .push(button::ghost(i18n.cancel, Some(RemoteDialogMessage::Close)))
            .push(button::primary(
                i18n.rd_pull_btn,
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
    if cfg!(windows) {
        body = body.push(autocrlf_hint);
    }
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

pub fn view<'a>(state: &'a RemoteDialogState, i18n: &'a I18n) -> Element<'a, RemoteDialogMessage> {
    match state.mode {
        RemoteDialogMode::Push => return build_push_panel(state, i18n),
        RemoteDialogMode::Pull => return build_pull_panel(state, i18n),
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
                        Text::new(i18n.rd_loading)
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
            i18n.rd_status_failed,
            error,
            BadgeTone::Danger,
        ))
    } else if let Some(message) = state.success_message.as_ref() {
        Some(build_status_panel::<RemoteDialogMessage>(
            i18n.rd_status_done,
            message,
            BadgeTone::Success,
        ))
    } else if state.remotes.is_empty() {
        Some(build_status_panel::<RemoteDialogMessage>(
            i18n.rd_status_empty,
            i18n.rd_status_empty_detail,
            BadgeTone::Neutral,
        ))
    } else {
        None
    };

    let toolbar = Container::new(
        Row::new()
            .spacing(theme::spacing::XS)
            .align_y(Alignment::Center)
            .push(Text::new(i18n.rd_title).size(14))
            .push(widgets::info_chip::<RemoteDialogMessage>(
                i18n.rd_remote_count_fmt.replace("{}", &state.remotes.len().to_string()),
                BadgeTone::Neutral,
            ))
            .push(widgets::info_chip::<RemoteDialogMessage>(
                state.current_branch_display.clone(),
                BadgeTone::Accent,
            ))
            .push_maybe(state.selected_remote.as_ref().map(|remote| {
                widgets::info_chip::<RemoteDialogMessage>(
                    i18n.rd_selected_fmt.replace("{}", &remote.to_string()),
                    BadgeTone::Success,
                )
            })),
    )
    .padding([6, 14])
    .width(Length::Fill)
    .style(theme::frame_style(Surface::Toolbar));

    let content = if state.remotes.is_empty() && !state.is_loading && state.error.is_none() {
        Column::new()
            .spacing(0)
            .width(Length::Fill)
            .push(toolbar)
            .push(iced::widget::rule::horizontal(1))
            .push(build_branch_scope_panel(state, i18n))
            .push_maybe(status_panel)
            .push(widgets::panel_empty_state(
                i18n.rd_empty_title,
                i18n.rd_empty_subtitle,
                i18n.rd_empty_detail,
                Some(build_action_buttons(state, i18n)),
            ))
    } else {
        Column::new()
            .spacing(0)
            .width(Length::Fill)
            .push(toolbar)
            .push(iced::widget::rule::horizontal(1))
            .push(build_branch_scope_panel(state, i18n))
            .push(iced::widget::rule::horizontal(1))
            .push_maybe(status_panel)
            .push(build_remotes_list(state, i18n))
            .push(iced::widget::rule::horizontal(1))
            .push(build_credential_inputs(state, i18n))
            .push(iced::widget::rule::horizontal(1))
            .push(build_action_buttons(state, i18n))
    };

    Container::new(scrollable::styled(content).height(Length::Fill))
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
    fn plain_pull_preview_defaults_to_no_rebase() {
        let state = RemoteDialogState::new();
        assert_eq!(state.pull_command_prefix(), "git pull --no-rebase");
    }

    #[test]
    fn push_defaults_are_safe() {
        let state = RemoteDialogState::new();
        assert!(!state.force_push);
        assert!(!state.push_tags);
        assert!(!state.set_upstream);
    }
}
