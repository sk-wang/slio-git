//! Main application shell.

use crate::components::rail_icons::{self, RailIcon};
use crate::i18n::I18n;
use crate::state::{
    AppState, AuxiliaryView, FeedbackLevel, GitToolWindowTab, ProjectEntry, ShellSection,
    StatusSeverity, ToolbarRemoteAction, ToolbarRemoteMenuState,
};
use crate::theme::{self, BadgeTone, ButtonTone, Surface};
use crate::views;
use crate::widgets::{self, button, scrollable, OptionalPush};
use git_core::remote::RemoteInfo;
use iced::widget::{container, rule, stack, text, Button, Column, Container, Row, Space, Text};
use iced::{Alignment, Color, Element, Length};
use std::path::PathBuf;

const MAX_RAIL_PROJECTS: usize = 5;

#[derive(Debug, Clone)]
struct ChromeBadges {
    branch_badge: Option<(String, BadgeTone)>,
    sync_badge: Option<(String, BadgeTone)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ChromeContextWidths {
    repo: u16,
    branch: u16,
}

#[derive(Debug, Clone)]
struct StatusBarContent {
    repo_path: String,
    workspace_summary: String,
    selected_path: Option<String>,
    activity_label: String,
    activity_tone: BadgeTone,
    detail: Option<String>,
}

pub struct MainWindow<'a, Message> {
    pub i18n: &'a I18n,
    pub state: &'a AppState,
    pub body: Element<'a, Message>,
    pub bottom_tool_window: Option<Element<'a, Message>>,
    pub on_open_repo: Message,
    pub on_switch_project: Box<dyn Fn(PathBuf) -> Message + 'a>,
    pub on_init_repo: Message,
    pub on_refresh: Message,
    pub on_commit: Message,
    pub on_pull: Message,
    pub on_push: Message,
    pub on_toggle_remote_menu: Box<dyn Fn(ToolbarRemoteAction) -> Message + 'a>,
    pub on_toolbar_remote_action: Box<dyn Fn(ToolbarRemoteAction, String) -> Message + 'a>,
    pub on_close_toolbar_remote_menu: Message,
    pub on_show_branches: Message,
    pub on_show_changes: Message,
    pub on_show_conflicts: Message,
    pub on_show_history: Message,
    pub on_show_remotes: Message,
    pub on_show_tags: Message,
    pub on_show_stashes: Message,
    pub on_show_rebase: Message,
    pub on_close_auxiliary: Message,
    pub on_switch_git_tool_window_tab: Box<dyn Fn(GitToolWindowTab) -> Message + 'a>,
    pub on_dismiss_feedback: Message,
    pub on_dismiss_toast: Message,
}

impl<'a, Message: Clone + 'a> MainWindow<'a, Message> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        i18n: &'a I18n,
        state: &'a AppState,
        body: Element<'a, Message>,
        bottom_tool_window: Option<Element<'a, Message>>,
        on_open_repo: Message,
        on_switch_project: impl Fn(PathBuf) -> Message + 'a,
        on_init_repo: Message,
        on_refresh: Message,
        on_commit: Message,
        on_pull: Message,
        on_push: Message,
        on_toggle_remote_menu: impl Fn(ToolbarRemoteAction) -> Message + 'a,
        on_toolbar_remote_action: impl Fn(ToolbarRemoteAction, String) -> Message + 'a,
        on_close_toolbar_remote_menu: Message,
        on_show_branches: Message,
        on_show_changes: Message,
        on_show_conflicts: Message,
        on_show_history: Message,
        on_show_remotes: Message,
        on_show_tags: Message,
        on_show_stashes: Message,
        on_show_rebase: Message,
        on_close_auxiliary: Message,
        on_switch_git_tool_window_tab: impl Fn(GitToolWindowTab) -> Message + 'a,
        on_dismiss_feedback: Message,
        on_dismiss_toast: Message,
    ) -> Self {
        Self {
            i18n,
            state,
            body,
            bottom_tool_window,
            on_open_repo,
            on_switch_project: Box::new(on_switch_project),
            on_init_repo,
            on_refresh,
            on_commit,
            on_pull,
            on_push,
            on_toggle_remote_menu: Box::new(on_toggle_remote_menu),
            on_toolbar_remote_action: Box::new(on_toolbar_remote_action),
            on_close_toolbar_remote_menu,
            on_show_branches,
            on_show_changes,
            on_show_conflicts,
            on_show_history,
            on_show_remotes,
            on_show_tags,
            on_show_stashes,
            on_show_rebase,
            on_close_auxiliary,
            on_switch_git_tool_window_tab: Box::new(on_switch_git_tool_window_tab),
            on_dismiss_feedback,
            on_dismiss_toast,
        }
    }

    pub fn view(self) -> Element<'a, Message> {
        let MainWindow {
            i18n,
            state,
            body,
            bottom_tool_window,
            on_open_repo,
            on_switch_project,
            on_init_repo,
            on_refresh,
            on_commit,
            on_pull,
            on_push,
            on_toggle_remote_menu,
            on_toolbar_remote_action,
            on_close_toolbar_remote_menu,
            on_show_branches,
            on_show_changes,
            on_show_conflicts,
            on_show_history,
            on_show_remotes,
            on_show_tags,
            on_show_stashes,
            on_show_rebase,
            on_close_auxiliary,
            on_switch_git_tool_window_tab,
            on_dismiss_feedback,
            on_dismiss_toast,
        } = self;

        let banner = state
            .feedback
            .as_ref()
            .filter(|feedback| Self::should_render_feedback_banner(feedback))
            .map(|feedback| {
                views::render_feedback_banner(feedback, Some(on_dismiss_feedback.clone()))
            });

        let content = if state.current_repository.is_some() {
            let workspace_body = Column::new()
                .spacing(0)
                .width(Length::Fill)
                .height(Length::Fill)
                .push(Self::editor_tab_strip(
                    state,
                    on_switch_git_tool_window_tab.as_ref(),
                ))
                .push(rule::horizontal(1).style(theme::separator_rule_style()))
                .push(
                    Container::new(body)
                        .width(Length::Fill)
                        .height(Length::Fill),
                );

            let workspace_column = Column::new()
                .spacing(0)
                .width(Length::Fill)
                .height(Length::Fill)
                .push_maybe(banner)
                .push(
                    Container::new(workspace_body)
                        .padding([0, 0])
                        .width(Length::Fill)
                        .height(Length::Fill),
                )
                .push_maybe(bottom_tool_window.map(|panel| {
                    Self::bottom_tool_window_panel(
                        state,
                        panel,
                        &on_show_history,
                        &on_close_auxiliary,
                    )
                }));

            let workspace = Row::new()
                .height(Length::Fill)
                .push(Self::navigation_rail(
                    state,
                    &on_open_repo,
                    on_switch_project.as_ref(),
                    &on_show_changes,
                    &on_show_conflicts,
                    &on_show_history,
                    &on_show_remotes,
                    &on_show_tags,
                    &on_show_stashes,
                    &on_show_rebase,
                ))
                .push(workspace_column);

            Column::new()
                .spacing(0)
                .push(Self::workspace_top_chrome(
                    i18n,
                    state,
                    &on_refresh,
                    &on_commit,
                    &on_pull,
                    &on_push,
                    on_toggle_remote_menu.as_ref(),
                    on_toolbar_remote_action.as_ref(),
                    &on_close_toolbar_remote_menu,
                    &on_show_branches,
                    &on_open_repo,
                ))
                .push(rule::horizontal(1).style(theme::separator_rule_style()))
                .push(workspace)
                .push(rule::horizontal(1).style(theme::separator_rule_style()))
                .push(Self::status_bar(i18n, state))
        } else {
            Column::new()
                .spacing(0)
                .push(
                    Container::new(body)
                        .width(Length::Fill)
                        .height(Length::Fill),
                )
                .push(rule::horizontal(1).style(theme::separator_rule_style()))
                .push(Self::welcome_status_bar(
                    i18n,
                    state,
                    &on_open_repo,
                    &on_init_repo,
                ))
        };

        let content: Element<'a, Message> = Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(theme::frame_style(Surface::Root))
            .into();

        if let Some(toast) = state.toast_notification.as_ref() {
            stack([
                content,
                views::render_toast_notification(toast, Some(on_dismiss_toast)),
            ])
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
        } else {
            content
        }
    }

    fn should_render_feedback_banner(feedback: &crate::state::FeedbackState) -> bool {
        feedback.sticky
            || matches!(
                feedback.level,
                FeedbackLevel::Error | FeedbackLevel::Warning | FeedbackLevel::Loading
            )
    }

    fn workspace_top_chrome(
        i18n: &'a I18n,
        state: &'a AppState,
        on_refresh: &Message,
        on_commit: &Message,
        on_pull: &Message,
        on_push: &Message,
        on_toggle_remote_menu: &dyn Fn(ToolbarRemoteAction) -> Message,
        on_toolbar_remote_action: &dyn Fn(ToolbarRemoteAction, String) -> Message,
        on_close_toolbar_remote_menu: &Message,
        on_show_branches: &Message,
        on_open_repo: &Message,
    ) -> Element<'a, Message> {
        let context = &state.shell.context_switcher;
        let badges = Self::pick_branch_badges(
            context.secondary_label.as_deref(),
            context.state_hint.as_deref(),
            context.sync_hint.as_deref(),
            &context.sync_label,
        );
        let context_widths = Self::chrome_context_widths();

        // Project selector — name + ▾, opens project list (not branches)
        let repo_switcher = Button::new(
            Row::new()
                .spacing(4)
                .align_y(Alignment::Center)
                .push(
                    Self::inline_icon(RailIcon::Repository, theme::darcula::TEXT_SECONDARY, 12.0),
                )
                .push(Text::new(&context.repository_name).size(12))
                .push(Text::new("▾").size(9).color(theme::darcula::TEXT_DISABLED)),
        )
        .style(theme::button_style(ButtonTone::Ghost))
        .padding([4, 8])
        .on_press(on_open_repo.clone()); // wired to ToggleProjectDropdown in main view()

        let branch_switcher = Button::new(
            Container::new(
                Row::new()
                    .spacing(theme::spacing::XS)
                    .align_y(Alignment::Center)
                    .width(Length::Fill)
                    .push(Self::inline_icon(
                        RailIcon::Branch,
                        theme::darcula::BRAND,
                        12.0,
                    ))
                    .push(
                        Text::new(&context.branch_name)
                            .size(11)
                            .width(Length::Fill)
                            .wrapping(text::Wrapping::WordOrGlyph),
                    )
                    .push_maybe(badges.branch_badge.as_ref().map(|(label, tone)| {
                        widgets::compact_chip::<Message>(label.clone(), *tone)
                    })),
            )
            .padding(theme::density::TOOLBAR_PADDING)
            .width(Length::Fill)
            .style(theme::panel_style(Surface::ToolbarField)),
        )
        .style(theme::button_style(ButtonTone::Ghost))
        .on_press(on_show_branches.clone());

        let context_switchers = Row::new()
            .spacing(theme::spacing::SM)
            .push(repo_switcher)
            .push(branch_switcher);

        let quick_actions = Row::new()
            .spacing(theme::spacing::XS)
            .push(button::ghost(i18n.refresh, Some(on_refresh.clone())))
            .push(Self::toolbar_remote_split_button(
                i18n.pull,
                ToolbarRemoteAction::Pull,
                false,
                Some(on_pull.clone()),
                Some(on_toggle_remote_menu(ToolbarRemoteAction::Pull)),
                state
                    .toolbar_remote_menu
                    .as_ref()
                    .is_some_and(|menu| menu.action == ToolbarRemoteAction::Pull),
            ))
            .push(Self::toolbar_remote_split_button(
                i18n.push,
                ToolbarRemoteAction::Push,
                true,
                Some(on_push.clone()),
                Some(on_toggle_remote_menu(ToolbarRemoteAction::Push)),
                state
                    .toolbar_remote_menu
                    .as_ref()
                    .is_some_and(|menu| menu.action == ToolbarRemoteAction::Push),
            ))
            .push(button::primary(
                i18n.commit,
                state
                    .shell
                    .chrome
                    .has_staged_changes
                    .then_some(on_commit.clone()),
            ));

        let primary_bar = Container::new(
            Row::new()
                .spacing(theme::spacing::SM)
                .align_y(Alignment::Center)
                .push(context_switchers)
                .push_maybe(badges.sync_badge.as_ref().map(|(label, tone)| {
                    widgets::compact_chip::<Message>(label.clone(), *tone)
                }))
                .push(Space::new().width(Length::Fill))
                .push(quick_actions),
        )
        .padding([10, 16])
        .style(theme::frame_style(Surface::Toolbar));

        let remote_menu: Option<Element<'a, Message>> =
            state.toolbar_remote_menu.as_ref().map(|menu| {
                Container::new(Row::new().push(Space::new().width(Length::Fill)).push(
                    Self::toolbar_remote_menu(
                        state,
                        menu,
                        on_toolbar_remote_action,
                        on_close_toolbar_remote_menu,
                    ),
                ))
                .padding([8, 16])
                .width(Length::Fill)
                .style(theme::frame_style(Surface::Toolbar))
                .into()
            });

        Column::new()
            .spacing(0)
            .push(primary_bar)
            .push_maybe(remote_menu)
            .into()
    }

    fn toolbar_remote_split_button(
        label: &'a str,
        action: ToolbarRemoteAction,
        emphasized: bool,
        on_primary: Option<Message>,
        on_toggle: Option<Message>,
        menu_open: bool,
    ) -> Element<'a, Message> {
        let tone = if menu_open {
            ButtonTone::TabActive
        } else if emphasized {
            ButtonTone::Secondary
        } else {
            ButtonTone::Ghost
        };
        let label_color = match tone {
            ButtonTone::Primary
            | ButtonTone::Success
            | ButtonTone::Warning
            | ButtonTone::Danger => iced::Color::WHITE,
            ButtonTone::Ghost => theme::darcula::TEXT_SECONDARY,
            _ => theme::darcula::TEXT_PRIMARY,
        };

        let main_button = {
            let button = button::toolbar_split_main(
                Row::new()
                    .spacing(4)
                    .align_y(Alignment::Center)
                    .push(
                        Text::new(Self::toolbar_remote_action_symbol(action))
                            .size(11)
                            .color(label_color),
                    )
                    .push(Text::new(label).size(11).color(label_color)),
                tone,
                None,
            );

            if let Some(message) = on_primary {
                button.on_press(message)
            } else {
                button
            }
        };

        let chevron_button = {
            let button =
                button::toolbar_split_chevron(if menu_open { "▴" } else { "▾" }, tone, None);

            if let Some(message) = on_toggle {
                button.on_press(message)
            } else {
                button
            }
        };

        Row::new()
            .spacing(0)
            .align_y(Alignment::Center)
            .push(main_button)
            .push(chevron_button)
            .into()
    }

    fn toolbar_remote_menu(
        state: &'a AppState,
        menu: &'a ToolbarRemoteMenuState,
        on_toolbar_remote_action: &dyn Fn(ToolbarRemoteAction, String) -> Message,
        on_close_toolbar_remote_menu: &Message,
    ) -> Element<'a, Message> {
        let remotes = menu.remotes.iter().collect::<Vec<_>>();

        let summary = menu
            .preferred_remote
            .as_ref()
            .map(|remote| {
                format!(
                    "当前分支：{} · 上游 remote：{remote}",
                    state.shell.context_switcher.branch_name
                )
            })
            .unwrap_or_else(|| format!("当前分支：{}", state.shell.context_switcher.branch_name));

        let remote_list = if remotes.is_empty() {
            Column::new().push(
                Text::new("当前分支还没有配置上游 remote。可以先点主按钮打开远程面板。")
                    .size(11)
                    .width(Length::Fill)
                    .wrapping(text::Wrapping::WordOrGlyph)
                    .color(theme::darcula::TEXT_SECONDARY),
            )
        } else {
            remotes.into_iter().fold(
                Column::new().spacing(theme::spacing::XS),
                |column, remote| {
                    column.push(Self::toolbar_remote_menu_item(
                        menu.action,
                        remote,
                        menu.preferred_remote.as_deref(),
                        on_toolbar_remote_action,
                    ))
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
                            Text::new(format!(
                                "{}当前分支 remote",
                                Self::toolbar_remote_action_label(menu.action)
                            ))
                            .size(12),
                        )
                        .push_maybe(
                            menu.preferred_remote
                                .as_ref()
                                .map(|_| widgets::info_chip::<Message>("上游", BadgeTone::Accent)),
                        )
                        .push(Space::new().width(Length::Fill))
                        .push(button::compact_ghost(
                            "收起",
                            Some(on_close_toolbar_remote_menu.clone()),
                        )),
                )
                .push(
                    Text::new(summary)
                        .size(10)
                        .width(Length::Fill)
                        .wrapping(text::Wrapping::WordOrGlyph)
                        .color(theme::darcula::TEXT_SECONDARY),
                )
                .push(
                    Container::new(scrollable::styled(remote_list).height(Length::Shrink))
                        .width(Length::Fill),
                ),
        )
        .padding(theme::density::TOOLBAR_PADDING)
        .width(Length::Fixed(340.0))
        .style(widgets::menu::panel_style)
        .into()
    }

    fn toolbar_remote_menu_item(
        action: ToolbarRemoteAction,
        remote: &'a RemoteInfo,
        preferred_remote: Option<&str>,
        on_toolbar_remote_action: &dyn Fn(ToolbarRemoteAction, String) -> Message,
    ) -> Element<'a, Message> {
        let is_preferred = preferred_remote.is_some_and(|name| name == remote.name);

        widgets::menu::action_row(
            Some(Self::toolbar_remote_action_symbol(action)),
            &remote.name,
            Some(remote.url.clone()),
            is_preferred.then(|| ("默认".to_string(), BadgeTone::Accent)),
            Some(on_toolbar_remote_action(action, remote.name.clone())),
            widgets::menu::MenuTone::Accent,
        )
    }

    fn toolbar_remote_action_label(action: ToolbarRemoteAction) -> &'static str {
        match action {
            ToolbarRemoteAction::Pull => "拉取",
            ToolbarRemoteAction::Push => "推送",
        }
    }

    fn toolbar_remote_action_symbol(action: ToolbarRemoteAction) -> &'static str {
        match action {
            ToolbarRemoteAction::Pull => "↓",
            ToolbarRemoteAction::Push => "↑",
        }
    }

    fn sync_tone(sync_label: &str) -> BadgeTone {
        // IDEA-style sync status colors:
        // ↑ (ahead/outgoing) = green = need to push
        // ↓ (behind/incoming) = blue/accent = need to pull
        // ↕ (diverged) = accent + warning mixed = need to pull and push
        // ? (unknown) = neutral
        if sync_label.starts_with('↑') {
            BadgeTone::Success
        } else if sync_label.starts_with('↓') {
            BadgeTone::Accent
        } else if sync_label.starts_with('↕') {
            BadgeTone::Warning
        } else if sync_label.starts_with('?') {
            BadgeTone::Neutral
        } else {
            BadgeTone::Neutral
        }
    }

    fn pick_branch_badges(
        secondary_label: Option<&str>,
        state_hint: Option<&str>,
        sync_hint: Option<&str>,
        sync_label: &str,
    ) -> ChromeBadges {
        let branch_badge = state_hint
            .map(|label| (label.to_string(), BadgeTone::Warning))
            .or_else(|| secondary_label.map(|label| (label.to_string(), BadgeTone::Accent)))
            .or_else(|| sync_hint.map(|label| (label.to_string(), BadgeTone::Neutral)));

        let sync_badge = Self::show_sync_chip(sync_label)
            .then(|| (sync_label.to_string(), Self::sync_tone(sync_label)));

        ChromeBadges {
            branch_badge,
            sync_badge,
        }
    }

    fn chrome_context_widths() -> ChromeContextWidths {
        ChromeContextWidths { repo: 3, branch: 2 }
    }

    fn editor_tab_strip(
        state: &'a AppState,
        on_switch_git_tool_window_tab: &dyn Fn(GitToolWindowTab) -> Message,
    ) -> Element<'a, Message> {
        let tabs: Element<'a, Message> = if state.shell.active_section == ShellSection::Conflicts {
            Row::new()
                .spacing(theme::spacing::XS)
                .align_y(Alignment::Center)
                .push(button::tab("冲突".to_string(), true, None::<Message>))
                .into()
        } else {
            Row::new()
                .spacing(theme::spacing::XS)
                .align_y(Alignment::Center)
                .push(button::tab(
                    "变更".to_string(),
                    state.shell.git_tool_window_tab == GitToolWindowTab::Changes,
                    Some(on_switch_git_tool_window_tab(GitToolWindowTab::Changes)),
                ))
                .push(button::tab(
                    "日志".to_string(),
                    state.shell.git_tool_window_tab == GitToolWindowTab::Log,
                    Some(on_switch_git_tool_window_tab(GitToolWindowTab::Log)),
                ))
                .into()
        };

        Container::new(
            Row::new()
                .spacing(theme::spacing::XS)
                .align_y(Alignment::Center)
                .push(tabs)
                .push(Space::new().width(Length::Fill)),
        )
        .padding(theme::density::SECONDARY_BAR_PADDING)
        .style(theme::frame_style(Surface::Toolbar))
        .into()
    }

    fn bottom_tool_window_panel(
        state: &'a AppState,
        panel: Element<'a, Message>,
        on_show_history: &Message,
        on_close_auxiliary: &Message,
    ) -> Element<'a, Message> {
        let title = state
            .shell
            .chrome
            .tool_window_title
            .clone()
            .unwrap_or_else(|| "日志".to_string());

        Container::new(
            Column::new()
                .spacing(0)
                .push(
                    Container::new(
                        Row::new()
                            .spacing(theme::spacing::XS)
                            .align_y(Alignment::Center)
                            .push(button::tab("Git", false, Some(on_close_auxiliary.clone())))
                            .push(button::tab(title, true, Some(on_show_history.clone())))
                            .push(Space::new().width(Length::Fill))
                            .push(button::compact_ghost(
                                "收起",
                                Some(on_close_auxiliary.clone()),
                            )),
                    )
                    .padding(theme::density::SECONDARY_BAR_PADDING)
                    .style(theme::frame_style(Surface::Nav)),
                )
                .push(rule::horizontal(1).style(theme::separator_rule_style()))
                .push(
                    Container::new(panel)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .padding(theme::density::TOOL_WINDOW_PADDING),
                ),
        )
        .height(Length::Fixed(theme::layout::TOOL_WINDOW_HEIGHT))
        .width(Length::Fill)
        .style(theme::panel_style(Surface::Panel))
        .into()
    }

    fn show_sync_chip(sync_label: &str) -> bool {
        !matches!(sync_label, "✓" | "○")
    }

    fn build_status_bar_content(state: &'a AppState) -> StatusBarContent {
        let status = &state.shell.status_surface;
        let selected_path = state.selected_change_path.clone();
        let workspace_summary = format!(
            "{} 个改动{}",
            state.shell.chrome.change_count,
            if state.shell.chrome.conflict_count > 0 {
                format!(" · {} 个冲突", state.shell.chrome.conflict_count)
            } else {
                String::new()
            }
        );
        let default_workspace_status = format!("{} 项变更", state.workspace_change_count());
        let is_common_workspace_status = state.workspace_change_count() > 0
            && status.severity == StatusSeverity::Info
            && status.message.as_deref() == Some(default_workspace_status.as_str())
            && match (status.detail.as_deref(), selected_path.as_deref()) {
                (Some(detail), Some(selected)) => detail == selected,
                (None, _) => true,
                _ => false,
            };

        let (activity_label, activity_tone, detail) = if is_common_workspace_status {
            ("就绪".to_string(), BadgeTone::Neutral, None)
        } else {
            (
                status.message.clone().unwrap_or_else(|| "就绪".to_string()),
                Self::status_bar_tone(state, status),
                status.detail.clone(),
            )
        };

        StatusBarContent {
            repo_path: state.shell.context_switcher.repository_path.clone(),
            workspace_summary,
            selected_path,
            activity_label,
            activity_tone,
            detail,
        }
    }

    fn status_bar_tone(
        state: &AppState,
        status: &crate::state::LightweightStatusSurface,
    ) -> BadgeTone {
        match status.severity {
            StatusSeverity::Success => BadgeTone::Success,
            StatusSeverity::Warning => BadgeTone::Warning,
            StatusSeverity::Error => BadgeTone::Danger,
            StatusSeverity::Info => {
                if state.is_loading {
                    BadgeTone::Neutral
                } else {
                    BadgeTone::Accent
                }
            }
        }
    }

    fn status_bar(i18n: &'a I18n, state: &'a AppState) -> Element<'a, Message> {
        let StatusBarContent {
            repo_path,
            workspace_summary,
            selected_path,
            activity_label,
            activity_tone,
            detail,
        } = Self::build_status_bar_content(state);

        crate::widgets::statusbar::StatusBar {
            i18n,
            repo_path: Some(repo_path),
            workspace_summary,
            selected_path,
            activity_label,
            activity_tone,
            detail,
        }
        .view()
    }

    #[allow(clippy::too_many_arguments)]
    fn navigation_rail(
        state: &'a AppState,
        on_open_repo: &Message,
        on_switch_project: &dyn Fn(PathBuf) -> Message,
        on_show_changes: &Message,
        on_show_conflicts: &Message,
        _on_show_history: &Message,
        on_show_remotes: &Message,
        on_show_tags: &Message,
        on_show_stashes: &Message,
        on_show_rebase: &Message,
    ) -> Element<'a, Message> {
        // Project switcher moved to top toolbar dropdown — rail only has navigation icons
        let navigation = state
            .navigation_items()
            .into_iter()
            .fold(
                Column::new()
                    .spacing(theme::spacing::XS)
                    .align_x(Alignment::Center),
                |column, item| {
                    let icon = Self::rail_label(item.section);
                    let message = match item.section {
                        ShellSection::Changes => item.enabled.then_some(on_show_changes.clone()),
                        ShellSection::Conflicts => {
                            item.enabled.then_some(on_show_conflicts.clone())
                        }
                        ShellSection::Welcome => None,
                    };

                    let cell: Element<'a, Message> = Container::new(button::rail_icon(
                        Self::rail_icon(icon, state.shell.active_section == item.section, 14.0),
                        state.shell.active_section == item.section,
                        message,
                    ))
                    .width(Length::Fill)
                    .center_x(Length::Fill)
                    .into();

                    column.push(cell)
                },
            )
            .push(Space::new().height(Length::Fill))
            .push(Self::rail_aux_button(
                Self::auxiliary_rail_icon(AuxiliaryView::Remotes),
                state.auxiliary_view == Some(AuxiliaryView::Remotes),
                Some(on_show_remotes.clone()),
            ))
            .push(Self::rail_aux_button(
                Self::auxiliary_rail_icon(AuxiliaryView::Tags),
                state.auxiliary_view == Some(AuxiliaryView::Tags),
                Some(on_show_tags.clone()),
            ))
            .push(Self::rail_aux_button(
                Self::auxiliary_rail_icon(AuxiliaryView::Stashes),
                state.auxiliary_view == Some(AuxiliaryView::Stashes),
                Some(on_show_stashes.clone()),
            ))
            .push(Self::rail_aux_button(
                Self::auxiliary_rail_icon(AuxiliaryView::Rebase),
                state.auxiliary_view == Some(AuxiliaryView::Rebase),
                Some(on_show_rebase.clone()),
            ));

        Container::new(navigation)
            .padding([12, 6])
            .width(Length::Fixed(theme::layout::RAIL_WIDTH))
            .height(Length::Fill)
            .style(theme::frame_style(Surface::Rail))
            .into()
    }

    fn project_switch_button(
        project: &ProjectEntry,
        active: bool,
        on_press: Option<Message>,
    ) -> Element<'a, Message> {
        // Generate a colored monogram avatar (like GitHub/GitLab default)
        let monogram = Self::project_monogram(&project.name);
        let bg_color = Self::project_color(&project.name);

        let avatar = Container::new(
            Text::new(monogram)
                .size(10)
                .color(Color::from_rgba(1.0, 1.0, 1.0, 0.85))
                .width(Length::Fill),
        )
        .width(Length::Fixed(22.0))
        .height(Length::Fixed(22.0))
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(move |_: &_| container::Style {
            background: Some(iced::Background::Color(bg_color)),
            border: iced::Border {
                radius: 6.0.into(),
                width: 1.0,
                color: Color::from_rgba(1.0, 1.0, 1.0, 0.08),
            },
            ..Default::default()
        });

        let btn = Button::new(avatar)
            .style(|_, _| iced::widget::button::Style::default())
            .padding(0);

        let btn = if let Some(msg) = on_press {
            btn.on_press(msg)
        } else {
            btn
        };

        Container::new(btn)
            .width(Length::Fill)
            .center_x(Length::Fill)
            .into()
    }

    fn rail_aux_button(
        icon: RailIcon,
        active: bool,
        on_press: Option<Message>,
    ) -> Element<'a, Message> {
        Container::new(button::rail_icon(
            Self::rail_icon(icon, active, 14.0),
            active,
            on_press,
        ))
        .width(Length::Fill)
        .center_x(Length::Fill)
        .into()
    }

    fn rail_label(section: ShellSection) -> RailIcon {
        match section {
            ShellSection::Changes => RailIcon::Changes,
            ShellSection::Conflicts => RailIcon::Conflicts,
            ShellSection::Welcome => RailIcon::Repository,
        }
    }

    fn auxiliary_rail_icon(view: AuxiliaryView) -> RailIcon {
        match view {
            AuxiliaryView::Commit => RailIcon::Repository,
            AuxiliaryView::Branches => RailIcon::Branch,
            AuxiliaryView::History => RailIcon::History,
            AuxiliaryView::Remotes => RailIcon::Remotes,
            AuxiliaryView::Tags => RailIcon::Tags,
            AuxiliaryView::Stashes => RailIcon::Stashes,
            AuxiliaryView::Rebase => RailIcon::Rebase,
            AuxiliaryView::Worktrees => RailIcon::Repository,
        }
    }

    fn rail_icon(icon: RailIcon, active: bool, size: f32) -> Element<'a, Message> {
        let color = if active {
            theme::darcula::ACCENT
        } else {
            theme::darcula::TEXT_SECONDARY
        };

        rail_icons::view(icon, color, theme::darcula::TEXT_PRIMARY, size)
    }

    fn inline_icon(icon: RailIcon, color: iced::Color, size: f32) -> Element<'a, Message> {
        rail_icons::view(icon, color, theme::darcula::TEXT_PRIMARY, size)
    }

    /// Generate a consistent muted color from a project name
    fn project_color(name: &str) -> Color {
        let hash: u32 = name.bytes().fold(0u32, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u32));
        // Muted dark-mode palette — low saturation, medium brightness
        const PALETTE: &[(f32, f32, f32)] = &[
            (0.22, 0.33, 0.52), // dark blue
            (0.20, 0.42, 0.33), // dark green
            (0.40, 0.25, 0.45), // dark purple
            (0.50, 0.33, 0.22), // dark brown
            (0.30, 0.38, 0.48), // dark slate
            (0.25, 0.42, 0.42), // dark teal
            (0.45, 0.30, 0.35), // dark rose
            (0.35, 0.35, 0.25), // dark olive
        ];
        let (r, g, b) = PALETTE[(hash as usize) % PALETTE.len()];
        Color::from_rgb(r, g, b)
    }

    fn project_monogram(name: &str) -> String {
        // Single letter — cleaner in small icon
        name.chars()
            .find(|c| c.is_alphanumeric())
            .map(|c| c.to_uppercase().to_string())
            .unwrap_or_else(|| "G".to_string())
    }
    fn welcome_status_bar(
        i18n: &'a I18n,
        state: &'a AppState,
        on_open_repo: &Message,
        on_init_repo: &Message,
    ) -> Element<'a, Message> {
        Container::new(
            Row::new()
                .spacing(theme::spacing::SM)
                .align_y(Alignment::Center)
                .push(
                    Text::new(i18n.app_tagline)
                        .size(11)
                        .color(theme::darcula::TEXT_SECONDARY),
                )
                .push(Space::new().width(Length::Fill))
                .push(button::ghost(
                    i18n.open_repository,
                    Some(on_open_repo.clone()),
                ))
                .push(button::secondary(
                    i18n.init_repository,
                    Some(on_init_repo.clone()),
                ))
                .push_maybe(state.feedback.as_ref().and_then(|feedback| {
                    feedback.compact.then(|| {
                        widgets::info_chip::<Message>(feedback.title.clone(), BadgeTone::Neutral)
                    })
                })),
        )
        .padding([8, 16])
        .style(theme::frame_style(Surface::Toolbar))
        .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::LightweightStatusSurface;
    use git_core::index::{Change, ChangeStatus};

    #[test]
    fn chrome_context_widths_leave_room_for_actions() {
        assert_eq!(
            MainWindow::<()>::chrome_context_widths(),
            ChromeContextWidths { repo: 3, branch: 2 }
        );
    }

    #[test]
    fn pick_branch_badges_prefers_state_hint_over_secondary_label() {
        let badges = MainWindow::<()>::pick_branch_badges(
            Some("跟踪 origin/main"),
            Some("有冲突"),
            Some("ahead 1"),
            "✓",
        );

        assert!(matches!(
            badges.branch_badge.as_ref(),
            Some((label, BadgeTone::Warning)) if label == "有冲突"
        ));
        assert!(badges.sync_badge.is_none());
    }

    #[test]
    fn show_sync_chip_hides_synced_and_no_upstream_states() {
        assert!(!MainWindow::<()>::show_sync_chip("✓"));
        assert!(!MainWindow::<()>::show_sync_chip("○"));
        assert!(MainWindow::<()>::show_sync_chip("↑2"));
        assert!(MainWindow::<()>::show_sync_chip("↕1/1"));
    }

    #[test]
    fn status_bar_content_suppresses_duplicate_workspace_status_signal() {
        let mut state = AppState::default();
        state.shell.context_switcher.repository_path = "/Users/wanghao/git/slio-git".to_string();
        state.shell.chrome.change_count = 3;
        state.shell.chrome.conflict_count = 1;
        state.selected_change_path = Some("src-ui/src/main.rs".to_string());
        state.unstaged_changes = vec![
            Change {
                path: "src-ui/src/main.rs".to_string(),
                status: ChangeStatus::Modified,
                staged: false,
                unstaged: true,
                old_oid: None,
                new_oid: None,
                is_submodule: false,
                submodule_summary: None,
            },
            Change {
                path: "src-ui/src/views/main_window.rs".to_string(),
                status: ChangeStatus::Modified,
                staged: false,
                unstaged: true,
                old_oid: None,
                new_oid: None,
                is_submodule: false,
                submodule_summary: None,
            },
            Change {
                path: "src-ui/src/widgets/statusbar.rs".to_string(),
                status: ChangeStatus::Modified,
                staged: false,
                unstaged: true,
                old_oid: None,
                new_oid: None,
                is_submodule: false,
                submodule_summary: None,
            },
        ];
        state.shell.status_surface = LightweightStatusSurface {
            message: Some("3 项变更".to_string()),
            detail: Some("src-ui/src/main.rs".to_string()),
            severity: StatusSeverity::Info,
            ..LightweightStatusSurface::default()
        };

        let content = MainWindow::<()>::build_status_bar_content(&state);

        assert_eq!(content.workspace_summary, "3 个改动 · 1 个冲突");
        assert_eq!(content.selected_path.as_deref(), Some("src-ui/src/main.rs"));
        assert_eq!(content.activity_label, "就绪");
        assert!(matches!(content.activity_tone, BadgeTone::Neutral));
        assert_eq!(content.detail, None);
    }

    #[test]
    fn status_bar_content_keeps_long_detail_for_widget_truncation() {
        let mut state = AppState::default();
        let long_detail = "origin/main 比本地领先 12 次提交，建议先拉取后再继续推送。".to_string();
        state.shell.status_surface = LightweightStatusSurface {
            message: Some("远程状态".to_string()),
            detail: Some(long_detail.clone()),
            severity: StatusSeverity::Warning,
            ..LightweightStatusSurface::default()
        };

        let content = MainWindow::<()>::build_status_bar_content(&state);

        assert_eq!(content.activity_label, "远程状态");
        assert!(matches!(content.activity_tone, BadgeTone::Warning));
        assert_eq!(content.detail, Some(long_detail));
    }
}
