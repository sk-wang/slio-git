//! Main application shell.

use crate::components::rail_icons::{self, RailIcon};
use crate::i18n::I18n;
use crate::state::{
    AppState, AuxiliaryView, FeedbackLevel, ProjectEntry, ShellSection, StatusSeverity,
    ToolbarRemoteAction, ToolbarRemoteMenuState,
};
use crate::theme::{self, BadgeTone, ButtonTone, Surface};
use crate::views;
use crate::widgets::{self, button, scrollable, OptionalPush};
use git_core::remote::RemoteInfo;
use iced::widget::{stack, text, Button, Column, Container, Row, Space, Text};
use iced::{Alignment, Element, Length};
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
    pub on_dismiss_feedback: Message,
    pub on_dismiss_toast: Message,
}

impl<'a, Message: Clone + 'a> MainWindow<'a, Message> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        i18n: &'a I18n,
        state: &'a AppState,
        body: Element<'a, Message>,
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
        on_dismiss_feedback: Message,
        on_dismiss_toast: Message,
    ) -> Self {
        Self {
            i18n,
            state,
            body,
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
            on_dismiss_feedback,
            on_dismiss_toast,
        }
    }

    pub fn view(self) -> Element<'a, Message> {
        let MainWindow {
            i18n,
            state,
            body,
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
                .push(
                    Column::new()
                        .spacing(0)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .push_maybe(banner)
                        .push(
                            Container::new(body)
                                .padding(theme::density::PANE_PADDING)
                                .width(Length::Fill)
                                .height(Length::Fill),
                        ),
                );

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
                    &on_show_changes,
                    &on_show_conflicts,
                    &on_show_history,
                    &on_show_remotes,
                    &on_show_tags,
                    &on_show_stashes,
                    &on_show_rebase,
                ))
                .push(iced::widget::rule::horizontal(1))
                .push(workspace)
                .push(iced::widget::rule::horizontal(1))
                .push(Self::status_bar(i18n, state))
        } else {
            Column::new()
                .spacing(0)
                .push(
                    Container::new(body)
                        .width(Length::Fill)
                        .height(Length::Fill),
                )
                .push(iced::widget::rule::horizontal(1))
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

    #[allow(clippy::too_many_arguments)]
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
        on_show_changes: &Message,
        on_show_conflicts: &Message,
        on_show_history: &Message,
        on_show_remotes: &Message,
        on_show_tags: &Message,
        on_show_stashes: &Message,
        on_show_rebase: &Message,
    ) -> Element<'a, Message> {
        let context = &state.shell.context_switcher;
        let badges = Self::pick_branch_badges(
            context.secondary_label.as_deref(),
            context.state_hint.as_deref(),
            context.sync_hint.as_deref(),
            &context.sync_label,
        );
        let context_widths = Self::chrome_context_widths();

        let repo_switcher = Button::new(
            Container::new(
                Row::new()
                    .spacing(theme::spacing::SM)
                    .align_y(Alignment::Center)
                    .width(Length::Fill)
                    .push(
                        Container::new(Self::inline_icon(
                            RailIcon::Repository,
                            theme::darcula::ACCENT,
                            13.0,
                        ))
                        .padding([4, 6])
                        .style(theme::panel_style(Surface::Accent)),
                    )
                    .push(
                        Column::new()
                            .spacing(1)
                            .width(Length::Fill)
                            .push(
                                Text::new(&context.repository_name)
                                    .size(12)
                                    .width(Length::Fill)
                                    .wrapping(text::Wrapping::WordOrGlyph),
                            )
                            .push(
                                Text::new(&context.repository_path)
                                    .size(10)
                                    .color(theme::darcula::TEXT_SECONDARY)
                                    .width(Length::Fill)
                                    .wrapping(text::Wrapping::WordOrGlyph),
                            ),
                    ),
            )
            .padding(theme::density::TOOLBAR_PADDING)
            .width(Length::Fill)
            .style(theme::panel_style(Surface::ToolbarField)),
        )
        .style(theme::button_style(ButtonTone::Ghost))
        .width(Length::FillPortion(context_widths.repo))
        .on_press(on_show_branches.clone());

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
        .width(Length::FillPortion(context_widths.branch))
        .on_press(on_show_branches.clone());

        let context_switchers = Row::new()
            .spacing(theme::spacing::SM)
            .width(Length::Fill)
            .push(repo_switcher)
            .push(branch_switcher);

        let tabs = Row::new()
            .spacing(theme::spacing::XS)
            .push(Self::nav_button(
                i18n.changes,
                state.shell.active_section == ShellSection::Changes,
                Some(on_show_changes.clone()),
            ))
            .push(Self::nav_button(
                i18n.conflicts,
                state.shell.active_section == ShellSection::Conflicts,
                state.has_conflicts().then_some(on_show_conflicts.clone()),
            ));

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
            .push(button::secondary(
                i18n.commit,
                state
                    .shell
                    .chrome
                    .has_staged_changes
                    .then_some(on_commit.clone()),
            ));

        let secondary_actions = Row::new()
            .spacing(theme::spacing::XS)
            .push(Self::utility_button(
                "历史",
                state.auxiliary_view == Some(AuxiliaryView::History),
                Some(on_show_history.clone()),
            ))
            .push(Self::utility_button(
                "远程",
                state.auxiliary_view == Some(AuxiliaryView::Remotes),
                Some(on_show_remotes.clone()),
            ))
            .push(Self::utility_button(
                "标签",
                state.auxiliary_view == Some(AuxiliaryView::Tags),
                Some(on_show_tags.clone()),
            ))
            .push(Self::utility_button(
                "储藏",
                state.auxiliary_view == Some(AuxiliaryView::Stashes),
                Some(on_show_stashes.clone()),
            ))
            .push(Self::utility_button(
                "Rebase",
                state.auxiliary_view == Some(AuxiliaryView::Rebase),
                Some(on_show_rebase.clone()),
            ));

        let primary_bar =
            Container::new(
                Row::new()
                    .spacing(theme::spacing::SM)
                    .align_y(Alignment::Center)
                    .push(context_switchers)
                    .push_maybe(badges.sync_badge.as_ref().map(|(label, tone)| {
                        widgets::compact_chip::<Message>(label.clone(), *tone)
                    }))
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

        let secondary_bar = Container::new(
            Row::new()
                .spacing(theme::spacing::SM)
                .align_y(Alignment::Center)
                .push(tabs)
                .push(Space::new().width(Length::Fill))
                .push_maybe(
                    state
                        .shell
                        .chrome
                        .has_secondary_actions
                        .then_some(secondary_actions),
                ),
        )
        .padding([8, 16])
        .style(theme::frame_style(Surface::Nav));

        Column::new()
            .spacing(0)
            .push(primary_bar)
            .push_maybe(remote_menu)
            .push(secondary_bar)
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
        let label_color = if menu_open || emphasized {
            theme::darcula::TEXT_PRIMARY
        } else {
            theme::darcula::TEXT_SECONDARY
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
            .spacing(1)
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
        .padding([16, 16])
        .width(Length::Fixed(340.0))
        .style(theme::panel_style(Surface::Raised))
        .into()
    }

    fn toolbar_remote_menu_item(
        action: ToolbarRemoteAction,
        remote: &'a RemoteInfo,
        preferred_remote: Option<&str>,
        on_toolbar_remote_action: &dyn Fn(ToolbarRemoteAction, String) -> Message,
    ) -> Element<'a, Message> {
        let is_preferred = preferred_remote.is_some_and(|name| name == remote.name);

        let content = Container::new(
            Row::new()
                .spacing(theme::spacing::SM)
                .align_y(Alignment::Center)
                .push(
                    Column::new()
                        .spacing(2)
                        .width(Length::Fill)
                        .push(
                            Row::new()
                                .spacing(theme::spacing::XS)
                                .align_y(Alignment::Center)
                                .push(Text::new(&remote.name).size(12))
                                .push_maybe(is_preferred.then(|| {
                                    widgets::info_chip::<Message>("默认", BadgeTone::Accent)
                                })),
                        )
                        .push(
                            Text::new(&remote.url)
                                .size(10)
                                .width(Length::Fill)
                                .wrapping(text::Wrapping::WordOrGlyph)
                                .color(theme::darcula::TEXT_SECONDARY),
                        ),
                )
                .push(
                    Text::new(Self::toolbar_remote_action_symbol(action))
                        .size(12)
                        .color(theme::darcula::TEXT_SECONDARY),
                ),
        )
        .padding([5, 8])
        .style(theme::panel_style(if is_preferred {
            Surface::Accent
        } else {
            Surface::Panel
        }));

        Button::new(content)
            .style(theme::button_style(ButtonTone::Ghost))
            .on_press(on_toolbar_remote_action(action, remote.name.clone()))
            .into()
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

    fn nav_button(label: &str, active: bool, on_press: Option<Message>) -> Element<'a, Message> {
        button::tab(label.to_string(), active, on_press).into()
    }

    fn utility_button(
        label: &str,
        active: bool,
        on_press: Option<Message>,
    ) -> Element<'a, Message> {
        button::tab(label.to_string(), active, on_press).into()
    }

    fn sync_tone(sync_label: &str) -> BadgeTone {
        if sync_label.starts_with('↑') {
            BadgeTone::Success
        } else if sync_label.starts_with('↓') {
            BadgeTone::Warning
        } else if sync_label.starts_with('↕') || sync_label.starts_with('?') {
            BadgeTone::Danger
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
        on_show_history: &Message,
        on_show_remotes: &Message,
        on_show_tags: &Message,
        on_show_stashes: &Message,
        on_show_rebase: &Message,
    ) -> Element<'a, Message> {
        let project_switcher = state
            .project_history
            .iter()
            .take(MAX_RAIL_PROJECTS)
            .fold(
                Column::new()
                    .spacing(theme::spacing::XS)
                    .align_x(Alignment::Center),
                |column, project| {
                    let is_active = state
                        .active_project_path()
                        .map(|path| path == project.path.as_path())
                        .unwrap_or(false);
                    let on_press = (!is_active).then(|| on_switch_project(project.path.clone()));
                    column.push(Self::project_switch_button(project, is_active, on_press))
                },
            )
            .push(Self::rail_aux_button(
                RailIcon::OpenRepository,
                false,
                Some(on_open_repo.clone()),
            ));

        let navigation = state
            .navigation_items()
            .into_iter()
            .fold(
                Column::new()
                    .spacing(theme::spacing::XS)
                    .align_x(Alignment::Center)
                    .push(project_switcher)
                    .push(Space::new().height(Length::Fixed(8.0)))
                    .push(Container::new(iced::widget::rule::horizontal(1)).width(Length::Fill))
                    .push(Space::new().height(Length::Fixed(6.0))),
                |column, item| {
                    let icon = Self::rail_label(item.section);
                    let message = match item.section {
                        ShellSection::Changes => item.enabled.then_some(on_show_changes.clone()),
                        ShellSection::Conflicts => {
                            item.enabled.then_some(on_show_conflicts.clone())
                        }
                        ShellSection::Welcome => None,
                    };

                    let cell: Element<'a, Message> = Container::new(
                        button::rail_icon(
                            Self::rail_icon(icon, state.shell.active_section == item.section, 14.0),
                            state.shell.active_section == item.section,
                            message,
                        )
                        .width(Length::Fill),
                    )
                    .width(Length::Fill)
                    .into();

                    column.push(cell)
                },
            )
            .push(Space::new().height(Length::Fill))
            .push(Self::rail_aux_button(
                Self::auxiliary_rail_icon(AuxiliaryView::History),
                state.auxiliary_view == Some(AuxiliaryView::History),
                Some(on_show_history.clone()),
            ))
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
        Container::new(
            button::rail(Self::project_monogram(&project.name), active, on_press)
                .width(Length::Fill),
        )
        .width(Length::Fill)
        .into()
    }

    fn rail_aux_button(
        icon: RailIcon,
        active: bool,
        on_press: Option<Message>,
    ) -> Element<'a, Message> {
        button::rail_icon(Self::rail_icon(icon, active, 14.0), active, on_press)
            .width(Length::Fill)
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

    fn project_monogram(name: &str) -> String {
        let mut parts = name
            .split(['-', '_', ' '])
            .filter(|part| !part.is_empty())
            .filter_map(|part| part.chars().next())
            .take(2)
            .collect::<String>()
            .to_uppercase();

        if parts.is_empty() {
            parts = name
                .chars()
                .filter(|ch| ch.is_alphanumeric())
                .take(2)
                .collect::<String>()
                .to_uppercase();
        }

        if parts.is_empty() {
            "G".to_string()
        } else {
            parts
        }
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
    use git_core::index::{Change, ChangeStatus};
    use crate::state::LightweightStatusSurface;

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
            },
            Change {
                path: "src-ui/src/views/main_window.rs".to_string(),
                status: ChangeStatus::Modified,
                staged: false,
                unstaged: true,
                old_oid: None,
                new_oid: None,
            },
            Change {
                path: "src-ui/src/widgets/statusbar.rs".to_string(),
                status: ChangeStatus::Modified,
                staged: false,
                unstaged: true,
                old_oid: None,
                new_oid: None,
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
        let long_detail =
            "origin/main 比本地领先 12 次提交，建议先拉取后再继续推送。".to_string();
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
