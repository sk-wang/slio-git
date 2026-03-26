//! slio-git UI - Pure Iced desktop application.

mod i18n;
mod keyboard;
mod logging;
mod state;
mod theme;

pub mod components;
pub mod views;
pub mod widgets;

use crate::components::status_icons::FileStatus;
use crate::keyboard::{get_shortcuts, ShortcutAction};
use crate::state::{AppState, AuxiliaryView, DiffPresentation, ShellSection, ToolbarRemoteAction};
use crate::theme::BadgeTone;
use crate::views::main_window::MainWindow;
use crate::views::{
    branch_popup::{self, BranchPopupMessage},
    commit_dialog::{self, CommitDialogMessage},
    history_view::{self, HistoryMessage},
    rebase_editor::{self, RebaseEditorMessage},
    remote_dialog::{self, RemoteDialogMessage},
    stash_panel::{self, StashPanelMessage},
    tag_dialog::{self, TagDialogMessage},
};
use crate::widgets::conflict_resolver::{ConflictResolverMessage, ResolutionOption};
use crate::widgets::{button, file_picker, scrollable, OptionalPush};
use git_core::index::Change;
use git_core::{
    diff::{ConflictHunk, ConflictHunkType, ConflictLineType, ConflictResolution, ThreeWayDiff},
    Repository,
};
use iced::widget::{container, text, Button, Column, Container, Row, Space, Text};
use iced::{
    time, Alignment, Background, Border, Color, Element, Length, Subscription, Task, Theme,
};
use log::{info, warn};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const AUTO_REFRESH_TICK_INTERVAL: Duration = Duration::from_secs(2);
const TOAST_TICK_INTERVAL: Duration = Duration::from_millis(250);

fn app_theme(_: &AppState) -> Theme {
    theme::darcula_theme()
}

pub fn main() -> iced::Result {
    if let Err(error) = logging::LogManager::init(None) {
        eprintln!("Failed to initialize logging: {}", error);
    }
    info!("Starting slio-git UI");

    #[cfg(target_os = "macos")]
    let cjk_font = iced::Font::with_name("PingFang SC");
    #[cfg(target_os = "windows")]
    let cjk_font = iced::Font::with_name("Microsoft YaHei");
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    let cjk_font = iced::Font::with_name("Noto Sans CJK SC");

    iced::application(|| (AppState::restore(), Task::none()), update, view)
        .title("slio-git")
        .default_font(cjk_font)
        .theme(app_theme)
        .window(iced::window::Settings {
            size: iced::Size::new(
                theme::layout::WINDOW_DEFAULT_WIDTH,
                theme::layout::WINDOW_DEFAULT_HEIGHT,
            ),
            min_size: Some(iced::Size::new(
                theme::layout::WINDOW_MIN_WIDTH,
                theme::layout::WINDOW_MIN_HEIGHT,
            )),
            ..Default::default()
        })
        .subscription(app_subscription)
        .run()
}

fn app_subscription(state: &AppState) -> Subscription<Message> {
    use iced::keyboard;

    let keyboard = keyboard::listen().filter_map(|event| match event {
        keyboard::Event::KeyPressed { key, modifiers, .. } => get_shortcuts()
            .into_iter()
            .find(|shortcut| key == shortcut.key && modifiers == shortcut.modifiers)
            .map(|shortcut| Message::KeyboardShortcut(shortcut.action)),
        _ => None,
    });

    let mut subscriptions = vec![keyboard];

    if state.current_repository.is_some() && !state.auto_refresh_suspended() {
        subscriptions.push(time::every(AUTO_REFRESH_TICK_INTERVAL).map(Message::AutoRefreshTick));
    }

    if state.toast_notification.is_some() {
        subscriptions.push(time::every(TOAST_TICK_INTERVAL).map(Message::ToastTick));
    }

    Subscription::batch(subscriptions)
}

fn update(state: &mut AppState, message: Message) -> Task<Message> {
    if !matches!(
        &message,
        Message::AutoRefreshTick(_)
            | Message::AutoRemoteCheckFinished(_)
            | Message::ToastTick(_)
            | Message::ToggleToolbarRemoteMenu(_)
            | Message::CloseToolbarRemoteMenu
            | Message::DismissToast
            | Message::ToolbarRemoteActionSelected { .. }
    ) {
        state.close_toolbar_remote_menu();
    }

    match message {
        Message::OpenRepository => {
            state.set_loading(
                "正在打开仓库",
                Some("选择本地 Git 目录后会直接进入变更工作台。".to_string()),
                "repository.open",
            );

            if let Some(path) = file_picker::pick_folder() {
                match Repository::discover(&path) {
                    Ok(repo) => {
                        logging::LogManager::log_repo_operation(
                            "open",
                            &path.display().to_string(),
                            true,
                        );
                        state.set_repository(repo);
                    }
                    Err(error) => {
                        logging::LogManager::log_repo_operation(
                            "open",
                            &path.display().to_string(),
                            false,
                        );
                        report_async_failure(
                            state,
                            "无法打开仓库",
                            format!("无法打开仓库: {}", error),
                            "repository.open",
                            "repository.open",
                        );
                    }
                }
            } else {
                state.set_empty(
                    "未选择仓库目录",
                    Some("你可以重新打开选择器，或先初始化一个新的 Git 仓库。".to_string()),
                    "repository.open",
                );
            }
        }
        Message::InitRepository => {
            state.set_loading(
                "正在初始化仓库",
                Some("选择一个目录后会立即创建 Git 仓库。".to_string()),
                "repository.init",
            );

            if let Some(path) = file_picker::pick_folder() {
                match Repository::init(&path) {
                    Ok(repo) => {
                        logging::LogManager::log_repo_operation(
                            "init",
                            &path.display().to_string(),
                            true,
                        );
                        state.set_repository(repo);
                    }
                    Err(error) => {
                        logging::LogManager::log_repo_operation(
                            "init",
                            &path.display().to_string(),
                            false,
                        );
                        report_async_failure(
                            state,
                            "无法初始化仓库",
                            format!("无法初始化仓库: {}", error),
                            "repository.init",
                            "repository.init",
                        );
                    }
                }
            } else {
                state.set_empty(
                    "未选择初始化目录",
                    Some("请选择一个目录来创建新的 Git 仓库。".to_string()),
                    "repository.init",
                );
            }
        }
        Message::Refresh => {
            let previous_section = state.shell.active_section;
            state.set_loading(
                "正在刷新工作区",
                Some("重新读取仓库状态和变更列表。".to_string()),
                "repository.refresh",
            );

            if state.current_repository.is_some() {
                match state.refresh_current_repository(false) {
                    Ok(()) => {
                        if previous_section != ShellSection::Conflicts || !state.has_conflicts() {
                            state.navigate_to(previous_section);
                        }
                        refresh_open_auxiliary_view(state);
                        state.set_success("仓库状态已刷新", None, "repository.refresh");
                    }
                    Err(error) => report_async_failure(
                        state,
                        "刷新失败",
                        format!("刷新失败: {}", error),
                        "repository.refresh",
                        "repository.refresh",
                    ),
                }
            } else {
                state.set_warning(
                    "还没有打开仓库",
                    Some("先从欢迎页打开一个仓库，再刷新状态。".to_string()),
                    "repository.refresh",
                );
            }
        }
        Message::AutoRefreshTick(now) => {
            let should_refresh_workspace = state.should_auto_refresh_workspace(now);
            if should_refresh_workspace {
                if let Err(error) = state.refresh_current_repository(false) {
                    warn!("Auto refresh workspace failed: {}", error);
                } else {
                    refresh_open_auxiliary_view(state);
                }
            }

            if state.should_auto_check_remote(now) {
                if let Some(repo_path) = state.active_project_path().map(Path::to_path_buf) {
                    state.begin_auto_remote_check(&repo_path, now);
                    return Task::perform(
                        async move { auto_refresh_remote_status(repo_path) },
                        Message::AutoRemoteCheckFinished,
                    );
                }
            }
        }
        Message::AutoRemoteCheckFinished(result) => {
            state.finish_auto_remote_check(&result.repo_path, Instant::now());

            if state.auto_refresh_suspended() {
                return Task::none();
            }

            match result.outcome {
                AutoRemoteCheckOutcome::SkippedNoUpstream => {}
                AutoRemoteCheckOutcome::Fetched { remote_name } => {
                    if state
                        .active_project_path()
                        .is_some_and(|path| path == result.repo_path.as_path())
                    {
                        if let Err(error) = state.refresh_current_repository(false) {
                            warn!(
                                "Auto refresh after remote fetch failed for {}: {}",
                                result.repo_path.display(),
                                error
                            );
                        } else {
                            refresh_open_auxiliary_view(state);
                        }
                    }

                    info!(
                        "Auto refreshed remote status from '{}' for {}",
                        remote_name,
                        result.repo_path.display()
                    );
                }
                AutoRemoteCheckOutcome::Failed { remote_name, error } => {
                    let remote_label = remote_name.unwrap_or_else(|| "upstream".to_string());
                    warn!(
                        "Auto remote status refresh failed for {} ({}): {}",
                        result.repo_path.display(),
                        remote_label,
                        error
                    );
                }
            }
        }
        Message::ToastTick(now) => {
            if state.toast_has_expired(now) {
                state.dismiss_toast();
            }
        }
        Message::DismissToast => state.dismiss_toast(),
        Message::CloseRepository => {
            state.clear_repository();
            state.set_empty(
                "仓库已关闭",
                Some("可以重新打开其他仓库，或初始化一个新的 Git 仓库。".to_string()),
                "repository.close",
            );
        }
        Message::ShowChanges => {
            let previous_section = state.shell.active_section;
            state.close_auxiliary_view();
            state.navigate_to(ShellSection::Changes);
            log_shell_navigation(
                previous_section,
                state.shell.active_section,
                "switch from shell rail",
            );
        }
        Message::ShowConflicts => {
            let previous_section = state.shell.active_section;
            state.close_auxiliary_view();
            if let Err(error) = state.open_conflict_resolver() {
                report_async_failure(
                    state,
                    "无法打开冲突视图",
                    error,
                    "workspace.conflicts",
                    "workspace.conflicts",
                );
            } else {
                log_shell_navigation(
                    previous_section,
                    state.shell.active_section,
                    "switch from shell rail",
                );
            }
        }
        Message::DismissFeedback => state.clear_feedback(),
        Message::StageFile(path) => {
            if let Err(error) = state.stage_file(path) {
                report_async_failure(
                    state,
                    "暂存文件失败",
                    error,
                    "workspace.stage_file",
                    "workspace.stage_file",
                );
            }
        }
        Message::UnstageFile(path) => {
            if let Err(error) = state.unstage_file(path) {
                report_async_failure(
                    state,
                    "取消暂存失败",
                    error,
                    "workspace.unstage_file",
                    "workspace.unstage_file",
                );
            }
        }
        Message::StageAll => {
            if let Err(error) = state.stage_all() {
                report_async_failure(
                    state,
                    "暂存全部失败",
                    error,
                    "workspace.stage_all",
                    "workspace.stage_all",
                );
            }
        }
        Message::UnstageAll => {
            if let Err(error) = state.unstage_all() {
                report_async_failure(
                    state,
                    "取消暂存全部失败",
                    error,
                    "workspace.unstage_all",
                    "workspace.unstage_all",
                );
            }
        }
        Message::SelectChange(path) => {
            if let Err(error) = state.select_change(path) {
                report_async_failure(
                    state,
                    "加载文件差异失败",
                    error,
                    "workspace.select_change",
                    "workspace.select_change",
                );
            }
        }
        Message::ToggleDiffPresentation => state.toggle_diff_presentation(),
        Message::SwitchProject(path) => {
            if let Err(error) = state.switch_to_project(&path) {
                report_async_failure(
                    state,
                    "切换项目失败",
                    error,
                    "workspace.project-switch",
                    "workspace.project-switch",
                );
            }
        }
        Message::NavigatePrevFile => select_relative_file(state, -1),
        Message::NavigateNextFile => select_relative_file(state, 1),
        Message::KeyboardShortcut(action) => match action {
            ShortcutAction::StageAll => {
                if let Err(error) = state.stage_all() {
                    report_async_failure(
                        state,
                        "暂存全部失败",
                        error,
                        "workspace.stage_all",
                        "workspace.stage_all",
                    );
                }
            }
            ShortcutAction::UnstageAll => {
                if let Err(error) = state.unstage_all() {
                    report_async_failure(
                        state,
                        "取消暂存全部失败",
                        error,
                        "workspace.unstage_all",
                        "workspace.unstage_all",
                    );
                }
            }
            ShortcutAction::Refresh => return update(state, Message::Refresh),
            _ => {}
        },
        Message::Commit => {
            if let Err(error) = open_commit_dialog(state) {
                report_async_failure(
                    state,
                    "无法打开提交面板",
                    error,
                    "workspace.commit",
                    "workspace.commit",
                );
            }
        }
        Message::Pull => {
            if let Err(error) = open_remote_dialog(state) {
                report_async_failure(
                    state,
                    "无法打开远程面板",
                    error,
                    "workspace.pull",
                    "workspace.pull",
                );
            } else {
                state.set_info(
                    "已打开远程面板",
                    state
                        .current_repository
                        .as_ref()
                        .map(|repo| remote_panel_hint(repo, "拉取")),
                    "workspace.pull",
                );
            }
        }
        Message::Push => {
            if let Err(error) = open_remote_dialog(state) {
                report_async_failure(
                    state,
                    "无法打开远程面板",
                    error,
                    "workspace.push",
                    "workspace.push",
                );
            } else {
                state.set_info(
                    "已打开远程面板",
                    state
                        .current_repository
                        .as_ref()
                        .map(|repo| remote_panel_hint(repo, "推送")),
                    "workspace.push",
                );
            }
        }
        Message::ToggleToolbarRemoteMenu(action) => {
            if let Err(error) = state.toggle_toolbar_remote_menu(action) {
                report_async_failure(
                    state,
                    "无法加载远程列表",
                    error,
                    "workspace.remote.toolbar-menu",
                    "workspace.remote.toolbar-menu",
                );
            }
        }
        Message::CloseToolbarRemoteMenu => state.close_toolbar_remote_menu(),
        Message::ToolbarRemoteActionSelected { action, remote } => {
            if let Err(error) = run_toolbar_remote_action(state, action, remote) {
                let (title, source) = match action {
                    ToolbarRemoteAction::Pull => ("拉取远程失败", "workspace.remote.toolbar.pull"),
                    ToolbarRemoteAction::Push => ("推送远程失败", "workspace.remote.toolbar.push"),
                };
                report_async_failure(state, title, error, source, source);
            }
        }
        Message::Stash => {
            if let Err(error) = open_stash_panel(state) {
                report_async_failure(
                    state,
                    "无法打开储藏面板",
                    error,
                    "workspace.stash",
                    "workspace.stash",
                );
            }
        }
        Message::ShowBranches => {
            if let Err(error) = open_branch_popup(state) {
                report_async_failure(
                    state,
                    "无法打开分支面板",
                    error,
                    "workspace.branches",
                    "workspace.branches",
                );
            }
        }
        Message::ShowHistory => {
            if let Err(error) = open_history_view(state) {
                report_async_failure(
                    state,
                    "无法打开历史视图",
                    error,
                    "workspace.history",
                    "workspace.history",
                );
            }
        }
        Message::ShowRemotes => {
            if let Err(error) = open_remote_dialog(state) {
                report_async_failure(
                    state,
                    "无法打开远程面板",
                    error,
                    "workspace.remote",
                    "workspace.remote",
                );
            }
        }
        Message::ShowTags => {
            if let Err(error) = open_tag_dialog(state) {
                report_async_failure(
                    state,
                    "无法打开标签面板",
                    error,
                    "workspace.tags",
                    "workspace.tags",
                );
            }
        }
        Message::ShowRebase => {
            if let Err(error) = open_rebase_editor(state) {
                report_async_failure(
                    state,
                    "无法打开 Rebase 面板",
                    error,
                    "workspace.rebase",
                    "workspace.rebase",
                );
            }
        }
        Message::OpenConflictResolver => {
            state.close_auxiliary_view();
            if let Err(error) = state.open_conflict_resolver() {
                report_async_failure(
                    state,
                    "无法打开冲突视图",
                    error,
                    "workspace.conflicts",
                    "workspace.conflicts",
                );
            }
        }
        Message::CloseConflictResolver => {
            let previous_section = state.shell.active_section;
            state.close_conflict_resolver();
            state.navigate_to(ShellSection::Changes);
            log_shell_navigation(
                previous_section,
                state.shell.active_section,
                "close conflict resolver",
            );
        }
        Message::SelectConflict(index) => state.select_conflict(index),
        Message::OpenConflictMerge(index) => {
            if let Err(error) = state.open_conflict_merge(index) {
                report_async_failure(
                    state,
                    "无法打开三栏合并视图",
                    error,
                    "workspace.conflicts",
                    "workspace.conflicts.merge",
                );
            }
        }
        Message::ResolveConflictWithOurs(index) => {
            if let Err(error) = resolve_conflict_with_side(
                state,
                index,
                ConflictResolution::Ours,
                "已接受您的更改",
                "workspace.conflicts.accept_ours",
            ) {
                report_async_failure(
                    state,
                    "接受您的更改失败",
                    error,
                    "workspace.conflicts",
                    "workspace.conflicts.accept_ours",
                );
            }
        }
        Message::ResolveConflictWithTheirs(index) => {
            if let Err(error) = resolve_conflict_with_side(
                state,
                index,
                ConflictResolution::Theirs,
                "已接受他们的更改",
                "workspace.conflicts.accept_theirs",
            ) {
                report_async_failure(
                    state,
                    "接受他们的更改失败",
                    error,
                    "workspace.conflicts",
                    "workspace.conflicts.accept_theirs",
                );
            }
        }
        Message::ConflictResolverMessage(message) => match message {
            ConflictResolverMessage::BackToList => state.close_conflict_merge(),
            ConflictResolverMessage::Refresh => {
                if let Err(error) = state.load_conflicts() {
                    report_async_failure(
                        state,
                        "刷新冲突失败",
                        format!("刷新冲突失败: {}", error),
                        "workspace.conflicts",
                        "workspace.conflicts.refresh",
                    );
                }
            }
            ConflictResolverMessage::SelectPrevHunk => {
                if let Some(resolver) = state.conflict_resolver.as_mut() {
                    resolver.select_previous_hunk();
                }
            }
            ConflictResolverMessage::SelectNextHunk => {
                if let Some(resolver) = state.conflict_resolver.as_mut() {
                    resolver.select_next_hunk();
                }
            }
            ConflictResolverMessage::Resolve => {
                if let Err(error) = resolve_selected_conflict(state) {
                    report_async_failure(
                        state,
                        "应用冲突解决方案失败",
                        error,
                        "workspace.conflicts",
                        "workspace.conflicts.resolve",
                    );
                }
            }
            ConflictResolverMessage::SelectHunk(index) => {
                if let Some(resolver) = state.conflict_resolver.as_mut() {
                    resolver.select_hunk(index);
                }
            }
            ConflictResolverMessage::ChooseOursForHunk(index) => {
                if let Some(resolver) = state.conflict_resolver.as_mut() {
                    resolver.resolve_hunk(index, ResolutionOption::Ours);
                }
            }
            ConflictResolverMessage::ChooseTheirsForHunk(index) => {
                if let Some(resolver) = state.conflict_resolver.as_mut() {
                    resolver.resolve_hunk(index, ResolutionOption::Theirs);
                }
            }
            ConflictResolverMessage::ChooseBaseForHunk(index) => {
                if let Some(resolver) = state.conflict_resolver.as_mut() {
                    resolver.resolve_hunk(index, ResolutionOption::Base);
                }
            }
            ConflictResolverMessage::AcceptOursAll => {
                if let Some(resolver) = state.conflict_resolver.as_mut() {
                    resolver.accept_all(ResolutionOption::Ours);
                }
            }
            ConflictResolverMessage::AcceptTheirsAll => {
                if let Some(resolver) = state.conflict_resolver.as_mut() {
                    resolver.accept_all(ResolutionOption::Theirs);
                }
            }
            ConflictResolverMessage::AutoMerge => {
                if let Some(resolver) = state.conflict_resolver.as_mut() {
                    resolver.auto_merge();
                }
            }
        },
        Message::CommitDialogMessage(message) => match message {
            CommitDialogMessage::MessageEdited(action) => {
                state.commit_dialog.apply_message_edit(action);
            }
            CommitDialogMessage::FileToggled(path, checked) => {
                let already_selected = state
                    .commit_dialog
                    .selected_files
                    .iter()
                    .any(|value| value == &path);

                if checked != already_selected {
                    state.commit_dialog.toggle_file(path);
                } else {
                    state.commit_dialog.clear_error();
                }
            }
            CommitDialogMessage::PreviewFile(path) => state.commit_dialog.preview_file(path),
            CommitDialogMessage::CommitPressed => {
                if let Err(error) = submit_commit_dialog(state) {
                    state.commit_dialog.set_error(error.clone());
                    report_async_failure(
                        state,
                        "提交失败",
                        error,
                        "workspace.commit",
                        "workspace.commit.submit",
                    );
                }
            }
            CommitDialogMessage::AmendPressed => {
                if let Err(error) = switch_commit_dialog_to_amend(state) {
                    state.commit_dialog.set_error(error.clone());
                    report_async_failure(
                        state,
                        "无法切换到 amend 模式",
                        error,
                        "workspace.commit",
                        "workspace.commit.amend",
                    );
                }
            }
            CommitDialogMessage::CancelPressed => state.close_auxiliary_view(),
        },
        Message::BranchPopupMessage(message) => {
            if branch_popup_message_closes_context_menu(&message) {
                state.branch_popup.close_context_menu();
            }

            match message {
                BranchPopupMessage::SelectBranch(name) => {
                    state.branch_popup.select_branch(name);
                    if let Ok(repo) = require_repository(state) {
                        state.branch_popup.load_selected_branch_history(&repo);
                    }
                }
                BranchPopupMessage::ToggleFolder(path_key) => {
                    state.branch_popup.toggle_folder(path_key);
                }
                BranchPopupMessage::OpenBranchContextMenu(name) => {
                    state.branch_popup.open_context_menu(name);
                    if let Ok(repo) = require_repository(state) {
                        state.branch_popup.load_selected_branch_history(&repo);
                    }
                }
                BranchPopupMessage::OpenCommitContextMenu(commit_id) => {
                    if let Ok(repo) = require_repository(state) {
                        state
                            .branch_popup
                            .select_branch_commit(&repo, commit_id.clone());
                        state.branch_popup.open_commit_context_menu(commit_id);
                        state.open_auxiliary_view(AuxiliaryView::Branches);
                        if let Some(error) = state.branch_popup.error.clone() {
                            report_async_failure(
                                state,
                                "打开提交动作失败",
                                error,
                                "workspace.branches",
                                "workspace.branches.commit_menu",
                            );
                        }
                    }
                }
                BranchPopupMessage::CloseBranchContextMenu => {
                    state.branch_popup.close_context_menu();
                }
                BranchPopupMessage::CloseCommitContextMenu => {
                    state.branch_popup.close_context_menu();
                }
                BranchPopupMessage::SetSearchQuery(query) => {
                    state.branch_popup.search_query = query;
                    state.branch_popup.error = None;
                }
                BranchPopupMessage::SetNewBranchName(name) => {
                    state.branch_popup.new_branch_name = name;
                }
                BranchPopupMessage::PrepareCreateFromSelected(name) => {
                    state.branch_popup.prepare_create_from_selected(name);
                }
                BranchPopupMessage::PrepareRenameBranch(name) => {
                    state.branch_popup.prepare_rename_branch(name);
                }
                BranchPopupMessage::SelectBranchCommit(commit_id) => {
                    if let Ok(repo) = require_repository(state) {
                        state.branch_popup.select_branch_commit(&repo, commit_id);
                        state.open_auxiliary_view(AuxiliaryView::Branches);
                        if let Some(error) = state.branch_popup.error.clone() {
                            report_async_failure(
                                state,
                                "加载提交详情失败",
                                error,
                                "workspace.branches",
                                "workspace.branches.select_commit",
                            );
                        }
                    }
                }
                BranchPopupMessage::SetInlineBranchName(name) => {
                    state.branch_popup.inline_branch_name = name;
                    state.branch_popup.error = None;
                }
                BranchPopupMessage::ConfirmInlineAction => {
                    if let Ok(repo) = require_repository(state) {
                        state.branch_popup.confirm_inline_action(&repo);
                        if let Some(error) = state.branch_popup.error.clone() {
                            report_async_failure(
                                state,
                                "执行分支操作失败",
                                error,
                                "workspace.branches",
                                "workspace.branches.inline",
                            );
                        } else {
                            let _ = refresh_repository_after_action(state, &repo, false);
                            if let Some(current) = state.current_repository.clone() {
                                state.branch_popup.load_branches(&current);
                            }
                            state.open_auxiliary_view(AuxiliaryView::Branches);
                            if let Some(message) = state.branch_popup.success_message.clone() {
                                state.set_success(message, None, "workspace.branches");
                            }
                        }
                    }
                }
                BranchPopupMessage::CancelInlineAction => state.branch_popup.cancel_inline_action(),
                BranchPopupMessage::CreateBranch(name) => {
                    if let Ok(repo) = require_repository(state) {
                        state.branch_popup.new_branch_name = name.clone();
                        state.branch_popup.create_branch(&repo, name);
                        if let Some(error) = state.branch_popup.error.clone() {
                            report_async_failure(
                                state,
                                "创建分支失败",
                                error,
                                "workspace.branches",
                                "workspace.branches.create",
                            );
                        } else {
                            let _ = refresh_repository_after_action(state, &repo, false);
                            if let Some(current) = state.current_repository.clone() {
                                state.branch_popup.load_branches(&current);
                            }
                            state.open_auxiliary_view(AuxiliaryView::Branches);
                            if let Some(message) = state.branch_popup.success_message.clone() {
                                state.set_success(message, None, "workspace.branches");
                            }
                        }
                    }
                }
                BranchPopupMessage::DeleteBranch(name) => {
                    if let Ok(repo) = require_repository(state) {
                        state.branch_popup.delete_branch(&repo, name);
                        if let Some(error) = state.branch_popup.error.clone() {
                            report_async_failure(
                                state,
                                "删除分支失败",
                                error,
                                "workspace.branches",
                                "workspace.branches.delete",
                            );
                        } else {
                            let _ = refresh_repository_after_action(state, &repo, false);
                            if let Some(current) = state.current_repository.clone() {
                                state.branch_popup.load_branches(&current);
                            }
                            state.open_auxiliary_view(AuxiliaryView::Branches);
                            if let Some(message) = state.branch_popup.success_message.clone() {
                                state.set_success(message, None, "workspace.branches");
                            }
                        }
                    }
                }
                BranchPopupMessage::CheckoutBranch(name) => {
                    if let Ok(repo) = require_repository(state) {
                        state.branch_popup.checkout_branch(&repo, name);
                        if let Some(error) = state.branch_popup.error.clone() {
                            report_async_failure(
                                state,
                                "切换分支失败",
                                error,
                                "workspace.branches",
                                "workspace.branches.checkout",
                            );
                        } else if let Err(error) =
                            refresh_repository_after_action(state, &repo, false)
                        {
                            report_async_failure(
                                state,
                                "刷新仓库状态失败",
                                error,
                                "workspace.branches",
                                "workspace.branches.checkout",
                            );
                        } else if let Some(current) = state.current_repository.clone() {
                            state.branch_popup.load_branches(&current);
                            state.open_auxiliary_view(AuxiliaryView::Branches);
                            if let Some(message) = state.branch_popup.success_message.clone() {
                                state.set_success(message, None, "workspace.branches");
                            }
                        }
                    }
                }
                BranchPopupMessage::CheckoutRemoteBranch(remote_ref) => {
                    if let Ok(repo) = require_repository(state) {
                        state.branch_popup.checkout_remote_branch(&repo, remote_ref);
                        if let Some(error) = state.branch_popup.error.clone() {
                            report_async_failure(
                                state,
                                "签出远程分支失败",
                                error,
                                "workspace.branches",
                                "workspace.branches.checkout_remote",
                            );
                        } else if let Err(error) =
                            refresh_repository_after_action(state, &repo, false)
                        {
                            report_async_failure(
                                state,
                                "刷新仓库状态失败",
                                error,
                                "workspace.branches",
                                "workspace.branches.checkout_remote",
                            );
                        } else if let Some(current) = state.current_repository.clone() {
                            state.branch_popup.load_branches(&current);
                            state.open_auxiliary_view(AuxiliaryView::Branches);
                            if let Some(message) = state.branch_popup.success_message.clone() {
                                state.set_success(message, None, "workspace.branches");
                            }
                        }
                    }
                }
                BranchPopupMessage::MergeBranch(name) => {
                    if let Ok(repo) = require_repository(state) {
                        let branch_name = name.clone();
                        state.branch_popup.merge_branch(&repo, name);
                        if let Some(error) = state.branch_popup.error.clone() {
                            report_async_failure(
                                state,
                                "合并分支失败",
                                error,
                                "workspace.branches",
                                "workspace.branches.merge",
                            );
                        } else if let Err(error) =
                            refresh_repository_after_action(state, &repo, true)
                        {
                            report_async_failure(
                                state,
                                "刷新仓库状态失败",
                                error,
                                "workspace.branches",
                                "workspace.branches.merge",
                            );
                        } else if state.has_conflicts() {
                            state.set_warning(
                                "合并产生冲突",
                                Some(format!(
                                    "与分支 {branch_name} 的合并已进入冲突状态，请先处理冲突文件。"
                                )),
                                "workspace.branches.merge",
                            );
                        } else if !state.has_conflicts() {
                            if let Some(current) = state.current_repository.clone() {
                                state.branch_popup.load_branches(&current);
                            }
                            state.open_auxiliary_view(AuxiliaryView::Branches);
                            if let Some(message) = state.branch_popup.success_message.clone() {
                                state.set_success(message, None, "workspace.branches");
                            }
                        }
                    }
                }
                BranchPopupMessage::CheckoutAndRebase { branch, onto } => {
                    if let Ok(repo) = require_repository(state) {
                        state
                            .branch_popup
                            .checkout_and_rebase(&repo, &branch, &onto);
                        if let Some(error) = state.branch_popup.error.clone() {
                            report_async_failure(
                                state,
                                "签出并变基失败",
                                error,
                                "workspace.branches",
                                "workspace.branches.checkout_rebase",
                            );
                        } else if let Err(error) =
                            refresh_repository_after_action(state, &repo, true)
                        {
                            report_async_failure(
                                state,
                                "刷新仓库状态失败",
                                error,
                                "workspace.branches",
                                "workspace.branches.checkout_rebase",
                            );
                        } else if !state.has_conflicts() {
                            if let Some(current) = state.current_repository.clone() {
                                state.branch_popup.load_branches(&current);
                            }
                            state.open_auxiliary_view(AuxiliaryView::Branches);
                            if let Some(message) = state.branch_popup.success_message.clone() {
                                state.set_success(message, None, "workspace.branches");
                            }
                        }
                    }
                }
                BranchPopupMessage::CompareWithCurrent { selected, current } => {
                    if let Ok(repo) = require_repository(state) {
                        state.branch_popup.comparison_title =
                            Some(format!("{selected} ↔ {current}"));
                        state
                            .branch_popup
                            .compare_refs_preview(&repo, &selected, &current);
                        state.open_auxiliary_view(AuxiliaryView::Branches);
                        if let Some(error) = state.branch_popup.error.clone() {
                            report_async_failure(
                                state,
                                "加载分支比较失败",
                                error,
                                "workspace.branches",
                                "workspace.branches.compare",
                            );
                        }
                    }
                }
                BranchPopupMessage::CompareWithWorktree(reference) => {
                    if let Ok(repo) = require_repository(state) {
                        state.branch_popup.comparison_title = Some(format!("{reference} ↔ 工作树"));
                        state
                            .branch_popup
                            .compare_ref_to_workdir_preview(&repo, &reference);
                        state.open_auxiliary_view(AuxiliaryView::Branches);
                        if let Some(error) = state.branch_popup.error.clone() {
                            report_async_failure(
                                state,
                                "加载工作树差异失败",
                                error,
                                "workspace.branches",
                                "workspace.branches.worktree_diff",
                            );
                        }
                    }
                }
                BranchPopupMessage::RebaseCurrentOnto(onto) => {
                    if let Ok(repo) = require_repository(state) {
                        state.branch_popup.rebase_current_onto(&repo, &onto);
                        if let Some(error) = state.branch_popup.error.clone() {
                            report_async_failure(
                                state,
                                "开始变基失败",
                                error,
                                "workspace.branches",
                                "workspace.branches.rebase",
                            );
                        } else if let Err(error) =
                            refresh_repository_after_action(state, &repo, true)
                        {
                            report_async_failure(
                                state,
                                "刷新仓库状态失败",
                                error,
                                "workspace.branches",
                                "workspace.branches.rebase",
                            );
                        } else if !state.has_conflicts() {
                            if let Some(current) = state.current_repository.clone() {
                                state.branch_popup.load_branches(&current);
                            }
                            state.open_auxiliary_view(AuxiliaryView::Branches);
                            if let Some(message) = state.branch_popup.success_message.clone() {
                                state.set_success(message, None, "workspace.branches");
                            }
                        }
                    }
                }
                BranchPopupMessage::FetchRemote(remote_name) => {
                    if let Ok(repo) = require_repository(state) {
                        state.branch_popup.fetch_remote(&repo, &remote_name);
                        if let Some(error) = state.branch_popup.error.clone() {
                            report_async_failure(
                                state,
                                "更新远程失败",
                                error,
                                "workspace.branches",
                                "workspace.branches.fetch",
                            );
                        } else {
                            let _ = refresh_repository_after_action(state, &repo, false);
                            if let Some(current) = state.current_repository.clone() {
                                state.branch_popup.load_branches(&current);
                            }
                            state.open_auxiliary_view(AuxiliaryView::Branches);
                            if let Some(message) = state.branch_popup.success_message.clone() {
                                state.set_success(message, None, "workspace.branches");
                            }
                        }
                    }
                }
                BranchPopupMessage::PushBranch { branch, remote } => {
                    if let Ok(repo) = require_repository(state) {
                        state
                            .branch_popup
                            .push_branch_to_remote(&repo, &remote, &branch);
                        if let Some(error) = state.branch_popup.error.clone() {
                            report_async_failure(
                                state,
                                "推送分支失败",
                                error,
                                "workspace.branches",
                                "workspace.branches.push",
                            );
                        } else {
                            let _ = refresh_repository_after_action(state, &repo, false);
                            if let Some(current) = state.current_repository.clone() {
                                state.branch_popup.load_branches(&current);
                            }
                            state.open_auxiliary_view(AuxiliaryView::Branches);
                            if let Some(message) = state.branch_popup.success_message.clone() {
                                state.set_success(message, None, "workspace.branches");
                            }
                        }
                    }
                }
                BranchPopupMessage::SetUpstream { branch, upstream } => {
                    if let Ok(repo) = require_repository(state) {
                        state.branch_popup.set_upstream(&repo, &branch, &upstream);
                        if let Some(error) = state.branch_popup.error.clone() {
                            report_async_failure(
                                state,
                                "设置跟踪分支失败",
                                error,
                                "workspace.branches",
                                "workspace.branches.upstream",
                            );
                        } else {
                            let _ = refresh_repository_after_action(state, &repo, false);
                            if let Some(current) = state.current_repository.clone() {
                                state.branch_popup.load_branches(&current);
                            }
                            state.open_auxiliary_view(AuxiliaryView::Branches);
                            if let Some(message) = state.branch_popup.success_message.clone() {
                                state.set_success(message, None, "workspace.branches");
                            }
                        }
                    }
                }
                BranchPopupMessage::PrepareTagFromCommit(commit_id) => {
                    state.tag_dialog.tag_name.clear();
                    state.tag_dialog.target = commit_id;
                    state.tag_dialog.message.clear();
                    state.tag_dialog.is_lightweight = false;
                    if let Err(error) = open_tag_dialog(state) {
                        report_async_failure(
                            state,
                            "打开标签面板失败",
                            error,
                            "workspace.tags",
                            "workspace.tags.open",
                        );
                    }
                }
                BranchPopupMessage::CopyCommitHash(commit_id) => {
                    if let Err(error) = copy_text_to_clipboard(&commit_id) {
                        report_async_failure(
                            state,
                            "复制提交哈希失败",
                            error,
                            "workspace.branches",
                            "workspace.branches.copy_commit",
                        );
                    } else {
                        state.open_auxiliary_view(AuxiliaryView::Branches);
                        state.show_toast(
                            crate::state::FeedbackLevel::Success,
                            "已复制提交哈希",
                            Some(short_commit_id(&commit_id).to_string()),
                        );
                    }
                }
                BranchPopupMessage::ExportCommitPatch(commit_id) => {
                    if let Ok(repo) = require_repository(state) {
                        let default_name = default_patch_file_name(&repo, &commit_id);
                        if let Some(path) = file_picker::save_file(&default_name) {
                            match git_core::export_commit_patch(&repo, &commit_id, &path) {
                                Ok(()) => {
                                    state.open_auxiliary_view(AuxiliaryView::Branches);
                                    state.set_success(
                                        "已导出补丁",
                                        Some(path.display().to_string()),
                                        "workspace.branches.patch",
                                    );
                                }
                                Err(error) => report_async_failure(
                                    state,
                                    "导出补丁失败",
                                    format!("导出补丁失败: {error}"),
                                    "workspace.branches",
                                    "workspace.branches.patch",
                                ),
                            }
                        }
                    }
                }
                BranchPopupMessage::PrepareCherryPickCommit(commit_id) => {
                    if let Ok(repo) = require_repository(state) {
                        state
                            .branch_popup
                            .prepare_cherry_pick_commit(&repo, commit_id);
                        state.open_auxiliary_view(AuxiliaryView::Branches);
                        if let Some(error) = state.branch_popup.error.clone() {
                            report_async_failure(
                                state,
                                "无法准备 Cherry-pick",
                                error,
                                "workspace.branches",
                                "workspace.branches.cherry_pick.prepare",
                            );
                        }
                    }
                }
                BranchPopupMessage::PrepareRevertCommit(commit_id) => {
                    if let Ok(repo) = require_repository(state) {
                        state.branch_popup.prepare_revert_commit(&repo, commit_id);
                        state.open_auxiliary_view(AuxiliaryView::Branches);
                        if let Some(error) = state.branch_popup.error.clone() {
                            report_async_failure(
                                state,
                                "无法准备回退提交",
                                error,
                                "workspace.branches",
                                "workspace.branches.revert.prepare",
                            );
                        }
                    }
                }
                BranchPopupMessage::PrepareResetCurrentBranchToCommit(commit_id) => {
                    if let Ok(repo) = require_repository(state) {
                        state
                            .branch_popup
                            .prepare_reset_current_branch_to_commit(&repo, commit_id);
                        state.open_auxiliary_view(AuxiliaryView::Branches);
                        if let Some(error) = state.branch_popup.error.clone() {
                            report_async_failure(
                                state,
                                "无法准备重置当前分支",
                                error,
                                "workspace.branches",
                                "workspace.branches.reset.prepare",
                            );
                        }
                    }
                }
                BranchPopupMessage::PreparePushCurrentBranchToCommit(commit_id) => {
                    if let Ok(repo) = require_repository(state) {
                        state
                            .branch_popup
                            .prepare_push_current_branch_to_commit(&repo, commit_id);
                        state.open_auxiliary_view(AuxiliaryView::Branches);
                        if let Some(error) = state.branch_popup.error.clone() {
                            report_async_failure(
                                state,
                                "无法准备“推送到这里”",
                                error,
                                "workspace.branches",
                                "workspace.branches.push_to_here.prepare",
                            );
                        }
                    }
                }
                BranchPopupMessage::ContinueInProgressCommitAction => {
                    if let Ok(repo) = require_repository(state) {
                        state.branch_popup.continue_in_progress_commit_action(&repo);
                        if let Some(error) = state.branch_popup.error.clone() {
                            report_async_failure(
                                state,
                                "继续提交流程失败",
                                error,
                                "workspace.branches",
                                "workspace.branches.follow_up.continue",
                            );
                        } else if let Err(error) =
                            refresh_repository_after_action(state, &repo, true)
                        {
                            report_async_failure(
                                state,
                                "刷新仓库状态失败",
                                error,
                                "workspace.branches",
                                "workspace.branches.follow_up.continue",
                            );
                        } else if state.has_conflicts() {
                            state.open_auxiliary_view(AuxiliaryView::Branches);
                            state.set_warning(
                                "提交流程仍有冲突",
                                Some("还有冲突文件未解决，请继续处理后再点继续。".to_string()),
                                "workspace.branches",
                            );
                        } else {
                            if let Some(current) = state.current_repository.clone() {
                                state.branch_popup.load_branches(&current);
                            }
                            state.open_auxiliary_view(AuxiliaryView::Branches);
                            if let Some(message) = state.branch_popup.success_message.clone() {
                                state.set_success(message, None, "workspace.branches");
                            }
                        }
                    }
                }
                BranchPopupMessage::AbortInProgressCommitAction => {
                    if let Ok(repo) = require_repository(state) {
                        state.branch_popup.abort_in_progress_commit_action(&repo);
                        if let Some(error) = state.branch_popup.error.clone() {
                            report_async_failure(
                                state,
                                "中止提交流程失败",
                                error,
                                "workspace.branches",
                                "workspace.branches.follow_up.abort",
                            );
                        } else if let Err(error) =
                            refresh_repository_after_action(state, &repo, false)
                        {
                            report_async_failure(
                                state,
                                "刷新仓库状态失败",
                                error,
                                "workspace.branches",
                                "workspace.branches.follow_up.abort",
                            );
                        } else {
                            if let Some(current) = state.current_repository.clone() {
                                state.branch_popup.load_branches(&current);
                            }
                            state.open_auxiliary_view(AuxiliaryView::Branches);
                            if let Some(message) = state.branch_popup.success_message.clone() {
                                state.set_success(message, None, "workspace.branches");
                            }
                        }
                    }
                }
                BranchPopupMessage::OpenConflictList => {
                    state.close_auxiliary_view();
                    state.navigate_to(ShellSection::Conflicts);
                    state.set_info(
                        "已切到冲突列表",
                        Some("先处理完冲突文件，再回到分支视图继续当前流程。".to_string()),
                        "workspace.conflicts",
                    );
                }
                BranchPopupMessage::ConfirmPendingCommitAction => {
                    if let Ok(repo) = require_repository(state) {
                        if let Some(action_kind) = state
                            .branch_popup
                            .pending_commit_action
                            .as_ref()
                            .map(|confirmation| confirmation.action.kind())
                        {
                            state.branch_popup.confirm_pending_commit_action(&repo);
                            if let Some(error) = state.branch_popup.error.clone() {
                                report_async_failure(
                                    state,
                                    "执行提交操作失败",
                                    error,
                                    "workspace.branches",
                                    "workspace.branches.commit_action",
                                );
                            } else if let Err(error) = refresh_repository_after_action(
                                state,
                                &repo,
                                matches!(
                                    action_kind,
                                    branch_popup::PendingCommitActionKind::CherryPick
                                        | branch_popup::PendingCommitActionKind::Revert
                                ),
                            ) {
                                report_async_failure(
                                    state,
                                    "刷新仓库状态失败",
                                    error,
                                    "workspace.branches",
                                    "workspace.branches.commit_action",
                                );
                            } else if state.has_conflicts() {
                                let detail = match action_kind {
                                    branch_popup::PendingCommitActionKind::CherryPick => {
                                        "Cherry-pick 已进入冲突状态，请先处理冲突文件。".to_string()
                                    }
                                    branch_popup::PendingCommitActionKind::Revert => {
                                        "回退提交已进入冲突状态，请先处理冲突文件。".to_string()
                                    }
                                    branch_popup::PendingCommitActionKind::ResetCurrentBranch
                                    | branch_popup::PendingCommitActionKind::PushCurrentBranchToCommit => {
                                        "当前仓库进入了需要继续处理的状态，请先处理冲突文件。".to_string()
                                    }
                                };
                                state.open_auxiliary_view(AuxiliaryView::Branches);
                                state.set_warning(
                                    "提交操作产生冲突",
                                    Some(detail),
                                    "workspace.branches",
                                );
                            } else {
                                if let Some(current) = state.current_repository.clone() {
                                    state.branch_popup.load_branches(&current);
                                }
                                state.open_auxiliary_view(AuxiliaryView::Branches);
                                if let Some(message) = state.branch_popup.success_message.clone() {
                                    state.set_success(message, None, "workspace.branches");
                                }
                            }
                        }
                    }
                }
                BranchPopupMessage::CancelPendingCommitAction => {
                    state.branch_popup.cancel_pending_commit_action();
                    state.open_auxiliary_view(AuxiliaryView::Branches);
                }
                BranchPopupMessage::ClearPreview => state.branch_popup.clear_preview(),
                BranchPopupMessage::Refresh => {
                    if let Ok(repo) = require_repository(state) {
                        logging::LogManager::log_context_switcher("refresh", &repo.name());
                        state.branch_popup.load_branches(&repo);
                        state.open_auxiliary_view(AuxiliaryView::Branches);
                        if let Some(error) = state.branch_popup.error.clone() {
                            report_async_failure(
                                state,
                                "刷新分支列表失败",
                                error,
                                "workspace.branches",
                                "workspace.branches.refresh",
                            );
                        }
                    }
                }
                BranchPopupMessage::OpenCommit => {
                    state.close_auxiliary_view();
                    return update(state, Message::Commit);
                }
                BranchPopupMessage::OpenPull => {
                    state.close_auxiliary_view();
                    return update(state, Message::Pull);
                }
                BranchPopupMessage::OpenPush => {
                    state.close_auxiliary_view();
                    return update(state, Message::Push);
                }
                BranchPopupMessage::OpenHistory => {
                    state.close_auxiliary_view();
                    return update(state, Message::ShowHistory);
                }
                BranchPopupMessage::OpenRemotes => {
                    state.close_auxiliary_view();
                    return update(state, Message::ShowRemotes);
                }
                BranchPopupMessage::OpenTags => {
                    state.close_auxiliary_view();
                    return update(state, Message::ShowTags);
                }
                BranchPopupMessage::OpenStashes => {
                    state.close_auxiliary_view();
                    return update(state, Message::Stash);
                }
                BranchPopupMessage::OpenRebase => {
                    state.close_auxiliary_view();
                    return update(state, Message::ShowRebase);
                }
                BranchPopupMessage::Close => state.close_auxiliary_view(),
            }
        }
        Message::HistoryMessage(message) => match message {
            HistoryMessage::Refresh => {
                if let Ok(repo) = require_repository(state) {
                    state.history_view.load_history(&repo);
                    state.history_view.context_menu_commit = None;
                    state.open_auxiliary_view(AuxiliaryView::History);
                    if let Some(error) = state.history_view.error.clone() {
                        report_async_failure(
                            state,
                            "刷新提交历史失败",
                            error,
                            "workspace.history",
                            "workspace.history.refresh",
                        );
                    }
                }
            }
            HistoryMessage::SelectCommit(commit_id) => {
                if let Ok(repo) = require_repository(state) {
                    state.history_view.context_menu_commit = None;
                    state.history_view.select_commit(&repo, commit_id);
                    if let Some(error) = state.history_view.error.clone() {
                        report_async_failure(
                            state,
                            "加载提交详情失败",
                            error,
                            "workspace.history",
                            "workspace.history.select",
                        );
                    }
                }
            }
            HistoryMessage::ViewDiff(_) => {
                state.set_info(
                    "提交详情已加载",
                    Some("当前可直接查看作者、时间与完整提交消息。".to_string()),
                    "workspace.history",
                );
            }
            HistoryMessage::OpenCommitContextMenu(commit_id) => {
                if let Ok(repo) = require_repository(state) {
                    state.history_view.select_commit(&repo, commit_id.clone());
                    state.history_view.context_menu_commit = Some(commit_id);
                    state.open_auxiliary_view(AuxiliaryView::History);
                    if let Some(error) = state.history_view.error.clone() {
                        report_async_failure(
                            state,
                            "打开提交动作失败",
                            error,
                            "workspace.history",
                            "workspace.history.context_menu",
                        );
                    }
                }
            }
            HistoryMessage::CloseCommitContextMenu => {
                state.history_view.context_menu_commit = None;
            }
            HistoryMessage::CopyCommitHash(commit_id) => {
                if let Err(error) = copy_text_to_clipboard(&commit_id) {
                    report_async_failure(
                        state,
                        "复制提交哈希失败",
                        error,
                        "workspace.history",
                        "workspace.history.copy_commit",
                    );
                } else {
                    state.history_view.context_menu_commit = None;
                    state.open_auxiliary_view(AuxiliaryView::History);
                    state.show_toast(
                        crate::state::FeedbackLevel::Success,
                        "已复制提交哈希",
                        Some(short_commit_id(&commit_id).to_string()),
                    );
                }
            }
            HistoryMessage::ExportCommitPatch(commit_id) => {
                if let Ok(repo) = require_repository(state) {
                    let default_name = default_patch_file_name(&repo, &commit_id);
                    if let Some(path) = file_picker::save_file(&default_name) {
                        match git_core::export_commit_patch(&repo, &commit_id, &path) {
                            Ok(()) => {
                                state.history_view.context_menu_commit = None;
                                state.open_auxiliary_view(AuxiliaryView::History);
                                state.set_success(
                                    "已导出补丁",
                                    Some(path.display().to_string()),
                                    "workspace.history.patch",
                                );
                            }
                            Err(error) => report_async_failure(
                                state,
                                "导出补丁失败",
                                format!("导出补丁失败: {error}"),
                                "workspace.history",
                                "workspace.history.patch",
                            ),
                        }
                    }
                }
            }
            HistoryMessage::CompareWithCurrent(commit_id) => {
                let Some(current_branch) = state.history_view.current_branch_name.clone() else {
                    report_async_failure(
                        state,
                        "无法比较当前分支",
                        "当前为 detached HEAD，不能直接与当前分支比较".to_string(),
                        "workspace.history",
                        "workspace.history.compare_current",
                    );
                    return iced::Task::none();
                };
                if let Ok(repo) = require_repository(state) {
                    state.history_view.context_menu_commit = None;
                    state.branch_popup.load_branches(&repo);
                }
                return update(
                    state,
                    Message::BranchPopupMessage(BranchPopupMessage::CompareWithCurrent {
                        selected: commit_id,
                        current: current_branch,
                    }),
                );
            }
            HistoryMessage::CompareWithWorktree(commit_id) => {
                if let Ok(repo) = require_repository(state) {
                    state.history_view.context_menu_commit = None;
                    state.branch_popup.load_branches(&repo);
                }
                return update(
                    state,
                    Message::BranchPopupMessage(BranchPopupMessage::CompareWithWorktree(commit_id)),
                );
            }
            HistoryMessage::PrepareCreateBranch(commit_id) => {
                if let Ok(repo) = require_repository(state) {
                    state.history_view.context_menu_commit = None;
                    state.branch_popup.load_branches(&repo);
                }
                return update(
                    state,
                    Message::BranchPopupMessage(BranchPopupMessage::PrepareCreateFromSelected(
                        commit_id,
                    )),
                );
            }
            HistoryMessage::PrepareTagFromCommit(commit_id) => {
                if let Ok(repo) = require_repository(state) {
                    state.history_view.context_menu_commit = None;
                    state.branch_popup.load_branches(&repo);
                }
                return update(
                    state,
                    Message::BranchPopupMessage(BranchPopupMessage::PrepareTagFromCommit(
                        commit_id,
                    )),
                );
            }
            HistoryMessage::PrepareCherryPickCommit(commit_id) => {
                if let Ok(repo) = require_repository(state) {
                    state.history_view.context_menu_commit = None;
                    state.branch_popup.load_branches(&repo);
                }
                return update(
                    state,
                    Message::BranchPopupMessage(BranchPopupMessage::PrepareCherryPickCommit(
                        commit_id,
                    )),
                );
            }
            HistoryMessage::PrepareRevertCommit(commit_id) => {
                if let Ok(repo) = require_repository(state) {
                    state.history_view.context_menu_commit = None;
                    state.branch_popup.load_branches(&repo);
                }
                return update(
                    state,
                    Message::BranchPopupMessage(BranchPopupMessage::PrepareRevertCommit(commit_id)),
                );
            }
            HistoryMessage::PrepareResetCurrentBranchToCommit(commit_id) => {
                if let Ok(repo) = require_repository(state) {
                    state.history_view.context_menu_commit = None;
                    state.branch_popup.load_branches(&repo);
                }
                return update(
                    state,
                    Message::BranchPopupMessage(
                        BranchPopupMessage::PrepareResetCurrentBranchToCommit(commit_id),
                    ),
                );
            }
            HistoryMessage::PreparePushCurrentBranchToCommit(commit_id) => {
                if let Ok(repo) = require_repository(state) {
                    state.history_view.context_menu_commit = None;
                    state.branch_popup.load_branches(&repo);
                }
                return update(
                    state,
                    Message::BranchPopupMessage(
                        BranchPopupMessage::PreparePushCurrentBranchToCommit(commit_id),
                    ),
                );
            }
            HistoryMessage::EditCommitMessage(commit_id) => {
                if let Ok(repo) = require_repository(state) {
                    match git_core::edit_commit_message(&repo, &commit_id) {
                        Ok(execution) => {
                            state.history_view.context_menu_commit = None;
                            if let Err(error) = refresh_repository_after_action(state, &repo, true)
                            {
                                report_async_failure(
                                    state,
                                    "刷新仓库状态失败",
                                    error,
                                    "workspace.history",
                                    "workspace.history.reword",
                                );
                            } else if state.has_conflicts() {
                                open_rebase_session_with_context(state, Some(&commit_id));
                                state.set_warning(
                                    "改说明流程产生冲突",
                                    Some(
                                        "请先解决冲突，再回到 Rebase 面板继续整理历史。"
                                            .to_string(),
                                    ),
                                    "workspace.history.reword",
                                );
                            } else if matches!(execution, git_core::RewriteExecution::InProgress) {
                                open_rebase_session_with_context(state, Some(&commit_id));
                                if let Err(error) = switch_commit_dialog_to_amend(state) {
                                    report_async_failure(
                                        state,
                                        "无法打开提交说明编辑面板",
                                        error,
                                        "workspace.history",
                                        "workspace.history.reword",
                                    );
                                } else {
                                    state.set_info(
                                        "已停在这条提交",
                                        Some(
                                            "修改提交说明后提交，再到 Rebase 面板继续后续整理。"
                                                .to_string(),
                                        ),
                                        "workspace.history.reword",
                                    );
                                }
                            } else {
                                state.open_auxiliary_view(AuxiliaryView::History);
                                state.set_success(
                                    format!("已准备修改提交 {}", short_commit_id(&commit_id)),
                                    Some("当前历史已刷新，可继续查看后续状态。".to_string()),
                                    "workspace.history.reword",
                                );
                            }
                        }
                        Err(error) => report_async_failure(
                            state,
                            "启动改说明流程失败",
                            error.to_string(),
                            "workspace.history",
                            "workspace.history.reword",
                        ),
                    }
                }
            }
            HistoryMessage::FixupCommitToPrevious(commit_id) => {
                if let Ok(repo) = require_repository(state) {
                    match git_core::fixup_commit_to_previous(&repo, &commit_id) {
                        Ok(execution) => {
                            state.history_view.context_menu_commit = None;
                            if let Err(error) = refresh_repository_after_action(state, &repo, true)
                            {
                                report_async_failure(
                                    state,
                                    "刷新仓库状态失败",
                                    error,
                                    "workspace.history",
                                    "workspace.history.fixup",
                                );
                            } else if state.has_conflicts() {
                                open_rebase_session_with_context(state, Some(&commit_id));
                                state.set_warning(
                                    "Fixup 进入冲突状态",
                                    Some(
                                        "请先解决冲突，再返回 Rebase 面板继续或跳过当前步骤。"
                                            .to_string(),
                                    ),
                                    "workspace.history.fixup",
                                );
                            } else if matches!(execution, git_core::RewriteExecution::InProgress) {
                                open_rebase_session_with_context(state, Some(&commit_id));
                                state.set_info(
                                    "Fixup 已进入整理流程",
                                    Some(
                                        "当前 rewrite 还没结束，可在 Rebase 面板继续、跳过或中止。"
                                            .to_string(),
                                    ),
                                    "workspace.history.fixup",
                                );
                            } else {
                                state.open_auxiliary_view(AuxiliaryView::History);
                                state.set_success(
                                    format!("已完成 Fixup {}", short_commit_id(&commit_id)),
                                    Some(
                                        "当前历史已刷新，可继续浏览或执行下一步整理。".to_string(),
                                    ),
                                    "workspace.history.fixup",
                                );
                            }
                        }
                        Err(error) => report_async_failure(
                            state,
                            "Fixup 失败",
                            error.to_string(),
                            "workspace.history",
                            "workspace.history.fixup",
                        ),
                    }
                }
            }
            HistoryMessage::SquashCommitToPrevious(commit_id) => {
                if let Ok(repo) = require_repository(state) {
                    match git_core::squash_commit_to_previous(&repo, &commit_id) {
                        Ok(execution) => {
                            state.history_view.context_menu_commit = None;
                            if let Err(error) = refresh_repository_after_action(state, &repo, true)
                            {
                                report_async_failure(
                                    state,
                                    "刷新仓库状态失败",
                                    error,
                                    "workspace.history",
                                    "workspace.history.squash",
                                );
                            } else if state.has_conflicts() {
                                open_rebase_session_with_context(state, Some(&commit_id));
                                state.set_warning(
                                    "压缩提交进入冲突状态",
                                    Some(
                                        "请先解决冲突，再返回 Rebase 面板继续当前整理流程。"
                                            .to_string(),
                                    ),
                                    "workspace.history.squash",
                                );
                            } else if matches!(execution, git_core::RewriteExecution::InProgress) {
                                open_rebase_session_with_context(state, Some(&commit_id));
                                state.set_info(
                                    "压缩提交流程已启动",
                                    Some(
                                        "当前 rewrite 还没结束，可在 Rebase 面板继续、跳过或中止。"
                                            .to_string(),
                                    ),
                                    "workspace.history.squash",
                                );
                            } else {
                                state.open_auxiliary_view(AuxiliaryView::History);
                                state.set_success(
                                    format!("已压缩提交 {}", short_commit_id(&commit_id)),
                                    Some(
                                        "当前历史已刷新，可继续浏览或执行下一步整理。".to_string(),
                                    ),
                                    "workspace.history.squash",
                                );
                            }
                        }
                        Err(error) => report_async_failure(
                            state,
                            "压缩提交失败",
                            error.to_string(),
                            "workspace.history",
                            "workspace.history.squash",
                        ),
                    }
                }
            }
            HistoryMessage::DropCommitFromHistory(commit_id) => {
                if let Ok(repo) = require_repository(state) {
                    match git_core::drop_commit_from_history(&repo, &commit_id) {
                        Ok(execution) => {
                            state.history_view.context_menu_commit = None;
                            if let Err(error) = refresh_repository_after_action(state, &repo, true)
                            {
                                report_async_failure(
                                    state,
                                    "刷新仓库状态失败",
                                    error,
                                    "workspace.history",
                                    "workspace.history.drop",
                                );
                            } else if state.has_conflicts() {
                                open_rebase_session_with_context(state, Some(&commit_id));
                                state.set_warning(
                                    "删除提交进入冲突状态",
                                    Some(
                                        "请先解决冲突，再返回 Rebase 面板继续当前整理流程。"
                                            .to_string(),
                                    ),
                                    "workspace.history.drop",
                                );
                            } else if matches!(execution, git_core::RewriteExecution::InProgress) {
                                open_rebase_session_with_context(state, Some(&commit_id));
                                state.set_info(
                                    "删除提交流程已启动",
                                    Some(
                                        "当前 rewrite 还没结束，可在 Rebase 面板继续、跳过或中止。"
                                            .to_string(),
                                    ),
                                    "workspace.history.drop",
                                );
                            } else {
                                state.open_auxiliary_view(AuxiliaryView::History);
                                state.set_success(
                                    format!("已删除提交 {}", short_commit_id(&commit_id)),
                                    Some(
                                        "当前历史已刷新，可继续浏览或执行下一步整理。".to_string(),
                                    ),
                                    "workspace.history.drop",
                                );
                            }
                        }
                        Err(error) => report_async_failure(
                            state,
                            "删除提交失败",
                            error.to_string(),
                            "workspace.history",
                            "workspace.history.drop",
                        ),
                    }
                }
            }
            HistoryMessage::OpenInteractiveRebaseFromCommit(commit_id) => {
                if let Ok(repo) = require_repository(state) {
                    state.history_view.context_menu_commit = None;
                    state
                        .rebase_editor
                        .prepare_interactive_rebase(&repo, commit_id);
                    state.open_auxiliary_view(AuxiliaryView::Rebase);
                    if let Some(error) = state.rebase_editor.error.clone() {
                        report_async_failure(
                            state,
                            "打开交互式变基失败",
                            error,
                            "workspace.history",
                            "workspace.history.rebase_from_here",
                        );
                    } else if let Some(message) = state.rebase_editor.success_message.clone() {
                        state.set_info(message, None, "workspace.history.rebase_from_here");
                    }
                }
            }
            HistoryMessage::SetSearchQuery(query) => state.history_view.set_search_query(query),
            HistoryMessage::Search => {
                if let Ok(repo) = require_repository(state) {
                    state.history_view.perform_search(&repo);
                    if let Some(error) = state.history_view.error.clone() {
                        report_async_failure(
                            state,
                            "搜索提交历史失败",
                            error,
                            "workspace.history",
                            "workspace.history.search",
                        );
                    }
                }
            }
            HistoryMessage::ClearSearch => state.history_view.clear_search(),
        },
        Message::RemoteDialogMessage(message) => match message {
            RemoteDialogMessage::SelectRemote(name) => {
                state.remote_dialog.selected_remote = Some(name);
                state.remote_dialog.error = None;
            }
            RemoteDialogMessage::Fetch => {
                if let Ok(repo) = require_repository(state) {
                    state.remote_dialog.fetch_selected(&repo);
                    if let Some(error) = state.remote_dialog.error.clone() {
                        report_async_failure(
                            state,
                            "获取远程失败",
                            error,
                            "workspace.remote",
                            "workspace.remote.fetch",
                        );
                    } else {
                        let _ = refresh_repository_after_action(state, &repo, false);
                        if let Some(current) = state.current_repository.clone() {
                            state.remote_dialog.load_remotes(&current);
                        }
                        state.open_auxiliary_view(AuxiliaryView::Remotes);
                        if let Some(message) = state.remote_dialog.success_message.clone() {
                            state.set_success(message, None, "workspace.remote");
                        }
                    }
                }
            }
            RemoteDialogMessage::Push => {
                if let Ok(repo) = require_repository(state) {
                    state.remote_dialog.push_selected(&repo);
                    if let Some(error) = state.remote_dialog.error.clone() {
                        report_async_failure(
                            state,
                            "推送远程失败",
                            error,
                            "workspace.remote",
                            "workspace.remote.push",
                        );
                    } else {
                        let _ = refresh_repository_after_action(state, &repo, false);
                        if let Some(current) = state.current_repository.clone() {
                            state.remote_dialog.load_remotes(&current);
                        }
                        state.open_auxiliary_view(AuxiliaryView::Remotes);
                        if let Some(message) = state.remote_dialog.success_message.clone() {
                            state.set_success(message, None, "workspace.remote");
                            state.show_toast(
                                crate::state::FeedbackLevel::Success,
                                "推送成功",
                                Some("当前分支的提交已经推送到远端。".to_string()),
                            );
                        }
                    }
                }
            }
            RemoteDialogMessage::Pull => {
                if let Ok(repo) = require_repository(state) {
                    state.remote_dialog.pull_selected(&repo);
                    if let Some(error) = state.remote_dialog.error.clone() {
                        report_async_failure(
                            state,
                            "拉取远程失败",
                            error,
                            "workspace.remote",
                            "workspace.remote.pull",
                        );
                    } else if let Err(error) = refresh_repository_after_action(state, &repo, true) {
                        report_async_failure(
                            state,
                            "刷新仓库状态失败",
                            error,
                            "workspace.remote",
                            "workspace.remote.pull",
                        );
                    } else if !state.has_conflicts() {
                        if let Some(current) = state.current_repository.clone() {
                            state.remote_dialog.load_remotes(&current);
                        }
                        state.open_auxiliary_view(AuxiliaryView::Remotes);
                        if let Some(message) = state.remote_dialog.success_message.clone() {
                            state.set_success(message, None, "workspace.remote");
                            state.show_toast(
                                crate::state::FeedbackLevel::Success,
                                "拉取成功",
                                Some("远程改动已同步到当前分支。".to_string()),
                            );
                        }
                    }
                }
            }
            RemoteDialogMessage::SetUsername(value) => state.remote_dialog.username = value,
            RemoteDialogMessage::SetPassword(value) => state.remote_dialog.password = value,
            RemoteDialogMessage::Refresh => {
                if let Ok(repo) = require_repository(state) {
                    state.remote_dialog.load_remotes(&repo);
                    state.open_auxiliary_view(AuxiliaryView::Remotes);
                    if let Some(error) = state.remote_dialog.error.clone() {
                        report_async_failure(
                            state,
                            "刷新远程列表失败",
                            error,
                            "workspace.remote",
                            "workspace.remote.refresh",
                        );
                    }
                }
            }
            RemoteDialogMessage::Close => state.close_auxiliary_view(),
        },
        Message::TagDialogMessage(message) => match message {
            TagDialogMessage::SelectTag(name) => {
                state.tag_dialog.selected_tag = Some(name);
                state.tag_dialog.error = None;
            }
            TagDialogMessage::CreateTag(name, target, lightweight) => {
                if let Ok(repo) = require_repository(state) {
                    state.tag_dialog.tag_name = name;
                    state.tag_dialog.target = target;
                    state.tag_dialog.is_lightweight = lightweight;
                    state.tag_dialog.create_tag(&repo);
                    if let Some(error) = state.tag_dialog.error.clone() {
                        report_async_failure(
                            state,
                            "创建标签失败",
                            error,
                            "workspace.tags",
                            "workspace.tags.create",
                        );
                    } else {
                        let _ = refresh_repository_after_action(state, &repo, false);
                        if let Some(current) = state.current_repository.clone() {
                            state.tag_dialog.load_tags(&current);
                        }
                        state.open_auxiliary_view(AuxiliaryView::Tags);
                        if let Some(message) = state.tag_dialog.success_message.clone() {
                            state.set_success(message, None, "workspace.tags");
                        }
                    }
                }
            }
            TagDialogMessage::DeleteTag(name) => {
                if let Ok(repo) = require_repository(state) {
                    state.tag_dialog.delete_tag(&repo, name);
                    if let Some(error) = state.tag_dialog.error.clone() {
                        report_async_failure(
                            state,
                            "删除标签失败",
                            error,
                            "workspace.tags",
                            "workspace.tags.delete",
                        );
                    } else {
                        let _ = refresh_repository_after_action(state, &repo, false);
                        if let Some(current) = state.current_repository.clone() {
                            state.tag_dialog.load_tags(&current);
                        }
                        state.open_auxiliary_view(AuxiliaryView::Tags);
                        if let Some(message) = state.tag_dialog.success_message.clone() {
                            state.set_success(message, None, "workspace.tags");
                        }
                    }
                }
            }
            TagDialogMessage::SetTagName(value) => state.tag_dialog.tag_name = value,
            TagDialogMessage::SetTarget(value) => state.tag_dialog.target = value,
            TagDialogMessage::SetMessage(value) => state.tag_dialog.message = value,
            TagDialogMessage::SetLightweight(value) => state.tag_dialog.is_lightweight = value,
            TagDialogMessage::Refresh => {
                if let Ok(repo) = require_repository(state) {
                    state.tag_dialog.load_tags(&repo);
                    state.open_auxiliary_view(AuxiliaryView::Tags);
                    if let Some(error) = state.tag_dialog.error.clone() {
                        report_async_failure(
                            state,
                            "刷新标签列表失败",
                            error,
                            "workspace.tags",
                            "workspace.tags.refresh",
                        );
                    }
                }
            }
            TagDialogMessage::Close => state.close_auxiliary_view(),
        },
        Message::StashPanelMessage(message) => match message {
            StashPanelMessage::SetNewStashMessage(value) => {
                state.stash_panel.new_stash_message = value
            }
            StashPanelMessage::SelectStash(index) => {
                state.stash_panel.selected_stash = Some(index);
                state.stash_panel.error = None;
            }
            StashPanelMessage::SaveStash => {
                if let Ok(repo) = require_repository(state) {
                    state.stash_panel.save_stash(&repo);
                    if let Some(error) = state.stash_panel.error.clone() {
                        report_async_failure(
                            state,
                            "保存储藏失败",
                            error,
                            "workspace.stash",
                            "workspace.stash.save",
                        );
                    } else {
                        let _ = refresh_repository_after_action(state, &repo, false);
                        if let Some(current) = state.current_repository.clone() {
                            state.stash_panel.load_stashes(&current);
                        }
                        state.open_auxiliary_view(AuxiliaryView::Stashes);
                        if let Some(message) = state.stash_panel.success_message.clone() {
                            state.set_success(message, None, "workspace.stash");
                        }
                    }
                }
            }
            StashPanelMessage::ApplyStash(index) => {
                if let Ok(repo) = require_repository(state) {
                    state.stash_panel.apply_stash(&repo, index);
                    if let Some(error) = state.stash_panel.error.clone() {
                        report_async_failure(
                            state,
                            "应用储藏失败",
                            error,
                            "workspace.stash",
                            "workspace.stash.apply",
                        );
                    } else if let Err(error) = refresh_repository_after_action(state, &repo, true) {
                        report_async_failure(
                            state,
                            "刷新仓库状态失败",
                            error,
                            "workspace.stash",
                            "workspace.stash.apply",
                        );
                    } else if !state.has_conflicts() {
                        if let Some(current) = state.current_repository.clone() {
                            state.stash_panel.load_stashes(&current);
                        }
                        state.open_auxiliary_view(AuxiliaryView::Stashes);
                        if let Some(message) = state.stash_panel.success_message.clone() {
                            state.set_success(message, None, "workspace.stash");
                        }
                    }
                }
            }
            StashPanelMessage::DropStash(index) => {
                if let Ok(repo) = require_repository(state) {
                    state.stash_panel.drop_stash(&repo, index);
                    if let Some(error) = state.stash_panel.error.clone() {
                        report_async_failure(
                            state,
                            "删除储藏失败",
                            error,
                            "workspace.stash",
                            "workspace.stash.drop",
                        );
                    } else {
                        let _ = refresh_repository_after_action(state, &repo, false);
                        if let Some(current) = state.current_repository.clone() {
                            state.stash_panel.load_stashes(&current);
                        }
                        state.open_auxiliary_view(AuxiliaryView::Stashes);
                        if let Some(message) = state.stash_panel.success_message.clone() {
                            state.set_success(message, None, "workspace.stash");
                        }
                    }
                }
            }
            StashPanelMessage::Refresh => {
                if let Ok(repo) = require_repository(state) {
                    state.stash_panel.load_stashes(&repo);
                    state.open_auxiliary_view(AuxiliaryView::Stashes);
                    if let Some(error) = state.stash_panel.error.clone() {
                        report_async_failure(
                            state,
                            "刷新储藏列表失败",
                            error,
                            "workspace.stash",
                            "workspace.stash.refresh",
                        );
                    }
                }
            }
            StashPanelMessage::Close => state.close_auxiliary_view(),
        },
        Message::RebaseEditorMessage(message) => match message {
            RebaseEditorMessage::SetBaseBranch(value) => state.rebase_editor.onto_branch = value,
            RebaseEditorMessage::OpenAmendForCurrentStep => {
                if let Err(error) = switch_commit_dialog_to_amend(state) {
                    report_async_failure(
                        state,
                        "无法打开提交说明编辑面板",
                        error,
                        "workspace.rebase",
                        "workspace.rebase.edit_current",
                    );
                }
            }
            RebaseEditorMessage::CycleTodoAction(index) => {
                state.rebase_editor.cycle_todo_action(index);
            }
            RebaseEditorMessage::MoveTodoUp(index) => {
                state.rebase_editor.move_todo_up(index);
            }
            RebaseEditorMessage::MoveTodoDown(index) => {
                state.rebase_editor.move_todo_down(index);
            }
            RebaseEditorMessage::StartRebase => {
                if let Ok(repo) = require_repository(state) {
                    state.rebase_editor.start_rebase(&repo);
                    if let Some(error) = state.rebase_editor.error.clone() {
                        report_async_failure(
                            state,
                            "开始变基失败",
                            error,
                            "workspace.rebase",
                            "workspace.rebase.start",
                        );
                    } else if let Err(error) = refresh_repository_after_action(state, &repo, true) {
                        report_async_failure(
                            state,
                            "刷新仓库状态失败",
                            error,
                            "workspace.rebase",
                            "workspace.rebase.start",
                        );
                    } else {
                        if let Some(current) = state.current_repository.clone() {
                            state.rebase_editor.load_status(&current);
                        }
                        state.open_auxiliary_view(AuxiliaryView::Rebase);
                        if state.has_conflicts() {
                            state.set_warning(
                                "交互式变基出现冲突",
                                Some(
                                    "请先解决冲突，再回到 Rebase 面板继续或跳过当前步骤。"
                                        .to_string(),
                                ),
                                "workspace.rebase",
                            );
                        } else if let Some(message) = state.rebase_editor.success_message.clone() {
                            state.set_success(message, None, "workspace.rebase");
                        }
                    }
                }
            }
            RebaseEditorMessage::ContinueRebase => {
                if let Ok(repo) = require_repository(state) {
                    state.rebase_editor.continue_rebase(&repo);
                    if let Some(error) = state.rebase_editor.error.clone() {
                        report_async_failure(
                            state,
                            "继续变基失败",
                            error,
                            "workspace.rebase",
                            "workspace.rebase.continue",
                        );
                    } else if let Err(error) = refresh_repository_after_action(state, &repo, true) {
                        report_async_failure(
                            state,
                            "刷新仓库状态失败",
                            error,
                            "workspace.rebase",
                            "workspace.rebase.continue",
                        );
                    } else if !state.has_conflicts() {
                        if let Some(current) = state.current_repository.clone() {
                            state.rebase_editor.load_status(&current);
                        }
                        state.open_auxiliary_view(AuxiliaryView::Rebase);
                        if let Some(message) = state.rebase_editor.success_message.clone() {
                            state.set_success(message, None, "workspace.rebase");
                        }
                    }
                }
            }
            RebaseEditorMessage::SkipCommit => {
                if let Ok(repo) = require_repository(state) {
                    state.rebase_editor.skip_commit(&repo);
                    if let Some(error) = state.rebase_editor.error.clone() {
                        report_async_failure(
                            state,
                            "跳过提交失败",
                            error,
                            "workspace.rebase",
                            "workspace.rebase.skip",
                        );
                    } else if let Err(error) = refresh_repository_after_action(state, &repo, true) {
                        report_async_failure(
                            state,
                            "刷新仓库状态失败",
                            error,
                            "workspace.rebase",
                            "workspace.rebase.skip",
                        );
                    } else if !state.has_conflicts() {
                        if let Some(current) = state.current_repository.clone() {
                            state.rebase_editor.load_status(&current);
                        }
                        state.open_auxiliary_view(AuxiliaryView::Rebase);
                        if let Some(message) = state.rebase_editor.success_message.clone() {
                            state.set_success(message, None, "workspace.rebase");
                        }
                    }
                }
            }
            RebaseEditorMessage::AbortRebase => {
                if let Ok(repo) = require_repository(state) {
                    state.rebase_editor.abort_rebase(&repo);
                    if let Some(error) = state.rebase_editor.error.clone() {
                        report_async_failure(
                            state,
                            "中止变基失败",
                            error,
                            "workspace.rebase",
                            "workspace.rebase.abort",
                        );
                    } else if let Err(error) = refresh_repository_after_action(state, &repo, false)
                    {
                        report_async_failure(
                            state,
                            "刷新仓库状态失败",
                            error,
                            "workspace.rebase",
                            "workspace.rebase.abort",
                        );
                    } else {
                        if let Some(current) = state.current_repository.clone() {
                            state.rebase_editor.load_status(&current);
                        }
                        state.open_auxiliary_view(AuxiliaryView::Rebase);
                        if let Some(message) = state.rebase_editor.success_message.clone() {
                            state.set_success(message, None, "workspace.rebase");
                        }
                    }
                }
            }
            RebaseEditorMessage::Refresh => {
                if let Ok(repo) = require_repository(state) {
                    state.rebase_editor.load_status(&repo);
                    state.open_auxiliary_view(AuxiliaryView::Rebase);
                    if let Some(error) = state.rebase_editor.error.clone() {
                        report_async_failure(
                            state,
                            "刷新变基状态失败",
                            error,
                            "workspace.rebase",
                            "workspace.rebase.refresh",
                        );
                    }
                }
            }
            RebaseEditorMessage::Close => state.close_auxiliary_view(),
        },
    }

    if let Some(feedback) = state.feedback.as_ref() {
        logging::LogManager::log_feedback(
            match feedback.level {
                crate::state::FeedbackLevel::Info => "info",
                crate::state::FeedbackLevel::Success => "success",
                crate::state::FeedbackLevel::Warning => "warning",
                crate::state::FeedbackLevel::Error => "error",
                crate::state::FeedbackLevel::Loading => "loading",
                crate::state::FeedbackLevel::Empty => "empty",
            },
            &feedback.title,
            feedback.detail.as_deref(),
        );
    }

    Task::none()
}

fn require_repository(state: &AppState) -> Result<Repository, String> {
    state
        .current_repository
        .clone()
        .ok_or_else(|| "没有打开的仓库".to_string())
}

fn build_staged_diff(
    repo: &Repository,
    staged_changes: &[Change],
) -> Result<git_core::diff::Diff, String> {
    let mut diff = git_core::diff::Diff {
        files: Vec::new(),
        total_additions: 0,
        total_deletions: 0,
    };

    for change in staged_changes {
        let file_diff = git_core::diff::diff_index_to_head(repo, Path::new(&change.path))
            .or_else(|_| git_core::diff::diff_file_to_index(repo, Path::new(&change.path)))
            .map_err(|error| format!("加载暂存差异失败: {}", error))?;

        diff.total_additions += file_diff.total_additions;
        diff.total_deletions += file_diff.total_deletions;
        diff.files.extend(file_diff.files);
    }

    Ok(diff)
}

fn refresh_repository_after_action(
    state: &mut AppState,
    repo: &Repository,
    prefer_conflicts: bool,
) -> Result<(), String> {
    let _ = repo;
    state.refresh_current_repository(prefer_conflicts)?;
    refresh_open_auxiliary_view(state);
    Ok(())
}

fn refresh_open_auxiliary_view(state: &mut AppState) {
    let Some(repo) = state.current_repository.clone() else {
        return;
    };

    match state.auxiliary_view {
        Some(AuxiliaryView::Branches) => state.branch_popup.load_branches(&repo),
        Some(AuxiliaryView::History) => state.history_view.load_history(&repo),
        Some(AuxiliaryView::Remotes) => state.remote_dialog.load_remotes(&repo),
        Some(AuxiliaryView::Tags) => state.tag_dialog.load_tags(&repo),
        Some(AuxiliaryView::Stashes) => state.stash_panel.load_stashes(&repo),
        Some(AuxiliaryView::Rebase) => state.rebase_editor.load_status(&repo),
        Some(AuxiliaryView::Commit) | None => {}
    }
}

fn open_rebase_session_with_context(state: &mut AppState, context_commit_id: Option<&str>) {
    state.rebase_editor.todo_is_editable = false;
    state.rebase_editor.todo_base_ref = None;
    state.rebase_editor.onto_branch.clear();

    if let Some(commit_id) = context_commit_id {
        state.rebase_editor.base_branch = commit_id.to_string();
    }

    if let Some(repo) = state.current_repository.clone() {
        state.rebase_editor.load_status(&repo);
    }

    state.open_auxiliary_view(AuxiliaryView::Rebase);
}

#[derive(Debug, Clone)]
pub struct AutoRemoteCheckResult {
    repo_path: PathBuf,
    outcome: AutoRemoteCheckOutcome,
}

#[derive(Debug, Clone)]
pub enum AutoRemoteCheckOutcome {
    SkippedNoUpstream,
    Fetched {
        remote_name: String,
    },
    Failed {
        remote_name: Option<String>,
        error: String,
    },
}

fn auto_refresh_remote_status(repo_path: PathBuf) -> AutoRemoteCheckResult {
    let outcome = match Repository::discover(&repo_path) {
        Ok(repo) => {
            let Some(remote_name) = repo.current_upstream_remote() else {
                return AutoRemoteCheckResult {
                    repo_path,
                    outcome: AutoRemoteCheckOutcome::SkippedNoUpstream,
                };
            };

            match git_core::remote::fetch(&repo, &remote_name, None) {
                Ok(()) => AutoRemoteCheckOutcome::Fetched { remote_name },
                Err(error) => AutoRemoteCheckOutcome::Failed {
                    remote_name: Some(remote_name),
                    error: error.to_string(),
                },
            }
        }
        Err(error) => AutoRemoteCheckOutcome::Failed {
            remote_name: None,
            error: error.to_string(),
        },
    };

    AutoRemoteCheckResult { repo_path, outcome }
}

fn run_toolbar_remote_action(
    state: &mut AppState,
    action: ToolbarRemoteAction,
    remote_name: String,
) -> Result<(), String> {
    state.close_toolbar_remote_menu();

    let repo = require_repository(state)?;
    let branch_name = match repo.current_branch() {
        Ok(Some(branch)) => branch,
        Ok(None) => {
            return Err(match action {
                ToolbarRemoteAction::Pull => "当前为 detached HEAD，无法执行拉取。".to_string(),
                ToolbarRemoteAction::Push => "当前为 detached HEAD，无法执行推送。".to_string(),
            });
        }
        Err(error) => return Err(format!("读取当前分支失败: {error}")),
    };

    match action {
        ToolbarRemoteAction::Pull => {
            git_core::remote::pull(&repo, &remote_name, &branch_name, None)
                .map_err(|error| format!("拉取远程失败: {error}"))?;
            refresh_repository_after_action(state, &repo, true)?;

            if state.has_conflicts() {
                state.set_warning(
                    format!("已拉取 {remote_name}/{branch_name}"),
                    Some("发现合并冲突，已切到冲突视图。".to_string()),
                    "workspace.remote.toolbar.pull",
                );
            } else {
                state.set_success(
                    format!("已拉取 {remote_name}/{branch_name}"),
                    Some("仓库状态已刷新。".to_string()),
                    "workspace.remote.toolbar.pull",
                );
                state.show_toast(
                    crate::state::FeedbackLevel::Success,
                    "拉取成功",
                    Some(format!("已从 {remote_name} 拉取并刷新当前分支。")),
                );
            }
        }
        ToolbarRemoteAction::Push => {
            git_core::remote::push(&repo, &remote_name, &branch_name, None)
                .map_err(|error| format!("推送远程失败: {error}"))?;
            refresh_repository_after_action(state, &repo, false)?;
            state.set_success(
                format!("已推送 {branch_name} -> {remote_name}"),
                Some("仓库状态已刷新。".to_string()),
                "workspace.remote.toolbar.push",
            );
            state.show_toast(
                crate::state::FeedbackLevel::Success,
                "推送成功",
                Some(format!("已将 {branch_name} 推送到 {remote_name}。")),
            );
        }
    }

    Ok(())
}

fn open_commit_dialog(state: &mut AppState) -> Result<(), String> {
    let repo = require_repository(state)?;
    let diff = build_staged_diff(&repo, &state.staged_changes)?;

    state.navigate_to(ShellSection::Changes);
    state.commit_dialog =
        commit_dialog::CommitDialogState::for_new_commit(state.staged_changes.clone(), &diff);
    state.open_auxiliary_view(AuxiliaryView::Commit);
    state.set_info(
        "已打开提交面板",
        Some("确认暂存文件与提交说明后，即可创建提交。".to_string()),
        "workspace.commit",
    );
    Ok(())
}

fn switch_commit_dialog_to_amend(state: &mut AppState) -> Result<(), String> {
    let repo = require_repository(state)?;
    let diff = build_staged_diff(&repo, &state.staged_changes)?;
    let head_commit = git_core::history::get_history(&repo, Some(1))
        .map_err(|error| format!("读取最近提交失败: {}", error))?
        .into_iter()
        .next()
        .ok_or_else(|| "当前仓库还没有提交历史，无法进入 amend 模式。".to_string())?;
    let commit = git_core::commit::get_commit(&repo, &head_commit.id)
        .map_err(|error| format!("加载提交详情失败: {}", error))?;

    state.commit_dialog =
        commit_dialog::CommitDialogState::for_amend(state.staged_changes.clone(), commit, &diff);
    state.open_auxiliary_view(AuxiliaryView::Commit);
    state.set_info(
        "已切换到 amend 模式",
        Some("你可以修改提交说明，或只保留部分暂存文件后更新最近一次提交。".to_string()),
        "workspace.commit",
    );
    Ok(())
}

fn submit_commit_dialog(state: &mut AppState) -> Result<(), String> {
    let repo = require_repository(state)?;
    state.commit_dialog.start_commit();

    if !state.commit_dialog.is_amend {
        let selected_files = state.commit_dialog.selected_files.clone();
        for change in state
            .staged_changes
            .iter()
            .filter(|change| !selected_files.contains(&change.path))
        {
            git_core::index::unstage_file(&repo, Path::new(&change.path))
                .map_err(|error| format!("更新待提交文件集合失败: {}", error))?;
        }
    }

    let commit_id = if state.commit_dialog.is_amend {
        let commit_to_amend = state
            .commit_dialog
            .commit_to_amend
            .as_ref()
            .ok_or_else(|| "缺少待修改的提交上下文。".to_string())?;
        git_core::commit::amend_commit(&repo, &commit_to_amend.id, &state.commit_dialog.message)
            .map_err(|error| format!("更新提交失败: {}", error))?
    } else {
        git_core::commit::create_commit(&repo, &state.commit_dialog.message, "", "")
            .map_err(|error| format!("创建提交失败: {}", error))?
    };

    state.commit_dialog.commit_success();
    refresh_repository_after_action(state, &repo, false)?;

    let still_rebasing = state.current_repository.as_ref().is_some_and(|current| {
        current.get_state() == git_core::repository::RepositoryState::Rebasing
    });
    if still_rebasing && state.commit_dialog.is_amend {
        if let Some(current) = state.current_repository.clone() {
            state.rebase_editor.load_status(&current);
        }
        state.open_auxiliary_view(AuxiliaryView::Rebase);
    } else {
        state.close_auxiliary_view();
        state.navigate_to(ShellSection::Changes);
    }

    let short_id = &commit_id[..commit_id.len().min(8)];
    state.set_success(
        if state.commit_dialog.is_amend && still_rebasing {
            format!("已更新提交 {}，可继续 rebase", short_id)
        } else if state.commit_dialog.is_amend {
            format!("已更新提交 {}", short_id)
        } else {
            format!("已创建提交 {}", short_id)
        },
        Some(if state.commit_dialog.is_amend && still_rebasing {
            "当前仍在交互式变基中，可在 Rebase 面板继续、跳过或中止。".to_string()
        } else {
            "仓库状态已刷新，可继续查看历史、分支或剩余变更。".to_string()
        }),
        "workspace.commit",
    );
    Ok(())
}

fn open_branch_popup(state: &mut AppState) -> Result<(), String> {
    let repo = require_repository(state)?;
    state.branch_popup.load_branches(&repo);
    if let Some(error) = state.branch_popup.error.clone() {
        return Err(error);
    }
    state.open_auxiliary_view(AuxiliaryView::Branches);
    logging::LogManager::log_context_switcher("open", &repo.name());
    Ok(())
}

fn branch_popup_message_closes_context_menu(message: &BranchPopupMessage) -> bool {
    matches!(
        message,
        BranchPopupMessage::SelectBranch(_)
            | BranchPopupMessage::SelectBranchCommit(_)
            | BranchPopupMessage::ToggleFolder(_)
            | BranchPopupMessage::SetSearchQuery(_)
            | BranchPopupMessage::CloseCommitContextMenu
            | BranchPopupMessage::PrepareCreateFromSelected(_)
            | BranchPopupMessage::PrepareRenameBranch(_)
            | BranchPopupMessage::PrepareTagFromCommit(_)
            | BranchPopupMessage::CopyCommitHash(_)
            | BranchPopupMessage::ExportCommitPatch(_)
            | BranchPopupMessage::PrepareCherryPickCommit(_)
            | BranchPopupMessage::PrepareRevertCommit(_)
            | BranchPopupMessage::PrepareResetCurrentBranchToCommit(_)
            | BranchPopupMessage::PreparePushCurrentBranchToCommit(_)
            | BranchPopupMessage::ConfirmInlineAction
            | BranchPopupMessage::CancelInlineAction
            | BranchPopupMessage::ConfirmPendingCommitAction
            | BranchPopupMessage::CancelPendingCommitAction
            | BranchPopupMessage::ContinueInProgressCommitAction
            | BranchPopupMessage::AbortInProgressCommitAction
            | BranchPopupMessage::OpenConflictList
            | BranchPopupMessage::DeleteBranch(_)
            | BranchPopupMessage::CheckoutBranch(_)
            | BranchPopupMessage::CheckoutRemoteBranch(_)
            | BranchPopupMessage::MergeBranch(_)
            | BranchPopupMessage::CheckoutAndRebase { .. }
            | BranchPopupMessage::CompareWithCurrent { .. }
            | BranchPopupMessage::CompareWithWorktree(_)
            | BranchPopupMessage::RebaseCurrentOnto(_)
            | BranchPopupMessage::FetchRemote(_)
            | BranchPopupMessage::PushBranch { .. }
            | BranchPopupMessage::SetUpstream { .. }
            | BranchPopupMessage::Refresh
            | BranchPopupMessage::OpenCommit
            | BranchPopupMessage::OpenPull
            | BranchPopupMessage::OpenPush
            | BranchPopupMessage::OpenHistory
            | BranchPopupMessage::OpenRemotes
            | BranchPopupMessage::OpenTags
            | BranchPopupMessage::OpenStashes
            | BranchPopupMessage::OpenRebase
            | BranchPopupMessage::Close
    )
}

fn open_history_view(state: &mut AppState) -> Result<(), String> {
    let repo = require_repository(state)?;
    state.history_view.load_history(&repo);
    if let Some(error) = state.history_view.error.clone() {
        return Err(error);
    }
    state.open_auxiliary_view(AuxiliaryView::History);
    state.set_info("已打开历史", None, "workspace.history");
    Ok(())
}

fn open_remote_dialog(state: &mut AppState) -> Result<(), String> {
    let repo = require_repository(state)?;
    state.remote_dialog.load_remotes(&repo);
    if let Some(error) = state.remote_dialog.error.clone() {
        return Err(error);
    }
    state.open_auxiliary_view(AuxiliaryView::Remotes);
    state.set_info("已打开远程", None, "workspace.remote");
    Ok(())
}

fn open_tag_dialog(state: &mut AppState) -> Result<(), String> {
    let repo = require_repository(state)?;
    state.tag_dialog.load_tags(&repo);
    if let Some(error) = state.tag_dialog.error.clone() {
        return Err(error);
    }
    state.open_auxiliary_view(AuxiliaryView::Tags);
    state.set_info("已打开标签", None, "workspace.tags");
    Ok(())
}

fn open_stash_panel(state: &mut AppState) -> Result<(), String> {
    let repo = require_repository(state)?;
    state.stash_panel.load_stashes(&repo);
    if let Some(error) = state.stash_panel.error.clone() {
        return Err(error);
    }
    state.open_auxiliary_view(AuxiliaryView::Stashes);
    state.set_info("已打开储藏", None, "workspace.stash");
    Ok(())
}

fn open_rebase_editor(state: &mut AppState) -> Result<(), String> {
    let repo = require_repository(state)?;
    state.rebase_editor.clear_draft_context();
    state.rebase_editor.load_status(&repo);
    if let Some(error) = state.rebase_editor.error.clone() {
        return Err(error);
    }
    state.open_auxiliary_view(AuxiliaryView::Rebase);
    state.set_info("已打开 Rebase", None, "workspace.rebase");
    Ok(())
}

fn resolve_selected_conflict(state: &mut AppState) -> Result<(), String> {
    let repo = require_repository(state)?;
    let resolver = state
        .conflict_resolver
        .clone()
        .ok_or_else(|| "当前没有可写回的冲突解析器状态。".to_string())?;
    let resolved_content = resolver
        .preview_content
        .clone()
        .unwrap_or_else(|| resolver.get_preview_content());

    git_core::diff::resolve_conflict(
        &repo,
        Path::new(&resolver.diff.path),
        ConflictResolution::Custom(resolved_content),
    )
    .map_err(|error| format!("写回冲突解决结果失败: {}", error))?;

    refresh_repository_after_action(state, &repo, true)?;
    if state.has_conflicts() {
        state.set_success(
            "冲突文件已写回",
            Some(format!(
                "{} 已处理完毕，请继续解决剩余冲突。",
                resolver.diff.path
            )),
            "workspace.conflicts",
        );
    } else {
        state.navigate_to(ShellSection::Changes);
        state.set_success(
            "冲突已解决",
            Some(format!(
                "{} 已写回工作区并重新加入索引。",
                resolver.diff.path
            )),
            "workspace.conflicts",
        );
    }
    Ok(())
}

fn resolve_conflict_with_side(
    state: &mut AppState,
    index: usize,
    resolution: ConflictResolution,
    success_title: &str,
    source: &'static str,
) -> Result<(), String> {
    let repo = require_repository(state)?;
    let conflict = state
        .conflict_files
        .get(index)
        .cloned()
        .ok_or_else(|| "未找到所选冲突文件".to_string())?;
    let path = conflict.path.clone();

    git_core::diff::resolve_conflict(&repo, Path::new(&path), resolution)
        .map_err(|error| format!("写回冲突解决结果失败: {}", error))?;

    refresh_repository_after_action(state, &repo, true)?;

    if state.has_conflicts() {
        state.set_success(
            success_title.to_string(),
            Some(format!("{path} 已处理，继续解决剩余冲突。")),
            source,
        );
    } else {
        state.navigate_to(ShellSection::Changes);
        state.set_success(
            success_title.to_string(),
            Some(format!("{path} 已写回工作区并重新加入索引。")),
            source,
        );
    }

    Ok(())
}

fn select_relative_file(state: &mut AppState, delta: isize) {
    let all_changes: Vec<&Change> = state
        .staged_changes
        .iter()
        .chain(state.unstaged_changes.iter())
        .chain(state.untracked_files.iter())
        .collect();

    if all_changes.is_empty() {
        state.set_warning(
            "当前没有可浏览的文件",
            Some("工作区没有变更时，导航键不会执行任何动作。".to_string()),
            "workspace.navigation",
        );
        return;
    }

    let next_index = if let Some(current) = &state.selected_change_path {
        let current_index = all_changes
            .iter()
            .position(|change| &change.path == current)
            .unwrap_or(0) as isize;
        (current_index + delta).clamp(0, (all_changes.len() - 1) as isize) as usize
    } else if delta >= 0 {
        0
    } else {
        all_changes.len() - 1
    };

    if let Some(path) = all_changes
        .get(next_index)
        .map(|change| change.path.clone())
    {
        if let Err(error) = state.select_change(path) {
            report_async_failure(
                state,
                "加载文件差异失败",
                error,
                "workspace.select_change",
                "workspace.select_change",
            );
        }
    }
}

fn report_async_failure(
    state: &mut AppState,
    title: impl Into<String>,
    detail: impl Into<String>,
    source: &'static str,
    operation: &str,
) {
    let title = title.into();
    let detail = detail.into();
    let detail = state
        .recovery_hint_for_source(source)
        .map(|hint| format!("{detail} {hint}"))
        .unwrap_or(detail);
    logging::LogManager::log_async_failure(operation, source, &detail);
    state.set_error_with_source(title, detail, source);
}

fn copy_text_to_clipboard(value: &str) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        return pipe_command_stdin("pbcopy", &[], value);
    }

    #[cfg(target_os = "windows")]
    {
        return pipe_command_stdin("cmd", &["/C", "clip"], value);
    }

    #[cfg(target_os = "linux")]
    {
        for (program, args) in [
            ("wl-copy", vec![]),
            ("xclip", vec!["-selection", "clipboard"]),
            ("xsel", vec!["--clipboard", "--input"]),
        ] {
            if pipe_command_stdin(program, &args, value).is_ok() {
                return Ok(());
            }
        }

        return Err("当前系统没有可用的剪贴板命令（wl-copy / xclip / xsel）".to_string());
    }

    #[allow(unreachable_code)]
    Err("当前平台暂不支持复制到系统剪贴板".to_string())
}

fn pipe_command_stdin(command: &str, args: &[&str], value: &str) -> Result<(), String> {
    let mut child = Command::new(command)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("无法启动剪贴板命令 {command}: {error}"))?;

    if let Some(stdin) = child.stdin.as_mut() {
        stdin
            .write_all(value.as_bytes())
            .map_err(|error| format!("写入系统剪贴板失败: {error}"))?;
    }

    let output = child
        .wait_with_output()
        .map_err(|error| format!("等待剪贴板命令完成失败: {error}"))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

fn default_patch_file_name(repo: &Repository, commit_id: &str) -> String {
    let short_id = short_commit_id(commit_id);
    let subject = git_core::get_commit(repo, commit_id)
        .ok()
        .map(|info| commit_subject_line(&info.message).to_string())
        .unwrap_or_default();
    let sanitized_subject = sanitize_file_name(&subject);

    if sanitized_subject.is_empty() {
        format!("{short_id}.patch")
    } else {
        format!("{short_id}-{sanitized_subject}.patch")
    }
}

fn sanitize_file_name(value: &str) -> String {
    let mut sanitized = String::new();
    let mut previous_dash = false;

    for ch in value.chars() {
        let normalized = if ch.is_ascii_alphanumeric() {
            previous_dash = false;
            Some(ch.to_ascii_lowercase())
        } else if ch == '-' || ch == '_' || ch.is_ascii_whitespace() {
            if previous_dash {
                None
            } else {
                previous_dash = true;
                Some('-')
            }
        } else {
            None
        };

        if let Some(ch) = normalized {
            sanitized.push(ch);
        }
    }

    sanitized.trim_matches('-').chars().take(48).collect()
}

fn commit_subject_line(message: &str) -> &str {
    message
        .lines()
        .find(|line| !line.trim().is_empty())
        .unwrap_or(message)
}

fn short_commit_id(id: &str) -> &str {
    &id[..id.len().min(8)]
}

fn log_shell_navigation(from: ShellSection, to: ShellSection, detail: &str) {
    logging::LogManager::log_navigation_transition(
        shell_section_name(from),
        shell_section_name(to),
        detail,
    );
}

fn shell_section_name(section: ShellSection) -> &'static str {
    match section {
        ShellSection::Welcome => "welcome",
        ShellSection::Changes => "changes",
        ShellSection::Conflicts => "conflicts",
    }
}

fn remote_panel_hint(repo: &Repository, action_label: &str) -> String {
    if let Some(upstream_ref) = repo.current_upstream_ref() {
        format!(
            "将围绕当前分支 {} 与上游 {upstream_ref} 继续执行{action_label}。",
            repo.current_branch_display()
        )
    } else if repo.current_branch().ok().flatten().is_some() {
        format!(
            "当前分支是 {}，请确认 remote 后继续执行{action_label}。",
            repo.current_branch_display()
        )
    } else {
        format!("当前为 detached HEAD，只能先查看远程信息，暂不建议直接{action_label}。")
    }
}

fn view(state: &AppState) -> Element<'_, Message> {
    let i18n = &i18n::ZH_CN;
    let body = build_body(state, i18n);

    MainWindow::new(
        i18n,
        state,
        body,
        Message::OpenRepository,
        Message::SwitchProject,
        Message::InitRepository,
        Message::Refresh,
        Message::Commit,
        Message::Pull,
        Message::Push,
        Message::ToggleToolbarRemoteMenu,
        |action, remote| Message::ToolbarRemoteActionSelected { action, remote },
        Message::CloseToolbarRemoteMenu,
        Message::ShowBranches,
        Message::ShowChanges,
        Message::ShowConflicts,
        Message::ShowHistory,
        Message::ShowRemotes,
        Message::ShowTags,
        Message::Stash,
        Message::ShowRebase,
        Message::DismissFeedback,
        Message::DismissToast,
    )
    .view()
}

fn build_body<'a>(state: &'a AppState, i18n: &'a i18n::I18n) -> Element<'a, Message> {
    if state.current_repository.is_none() {
        return build_welcome_body(i18n);
    }

    if let Some(auxiliary) = state.auxiliary_view {
        return match auxiliary {
            AuxiliaryView::Commit => {
                commit_dialog::view(&state.commit_dialog).map(Message::CommitDialogMessage)
            }
            AuxiliaryView::Branches => {
                branch_popup::view(&state.branch_popup).map(Message::BranchPopupMessage)
            }
            AuxiliaryView::History => {
                history_view::view(&state.history_view).map(Message::HistoryMessage)
            }
            AuxiliaryView::Remotes => {
                remote_dialog::view(&state.remote_dialog).map(Message::RemoteDialogMessage)
            }
            AuxiliaryView::Tags => {
                tag_dialog::view(&state.tag_dialog).map(Message::TagDialogMessage)
            }
            AuxiliaryView::Stashes => {
                stash_panel::view(&state.stash_panel).map(Message::StashPanelMessage)
            }
            AuxiliaryView::Rebase => {
                rebase_editor::view(&state.rebase_editor).map(Message::RebaseEditorMessage)
            }
        };
    }

    match state.shell.active_section {
        ShellSection::Changes => build_changes_body(state, i18n),
        ShellSection::Conflicts => build_conflict_body(state),
        ShellSection::Welcome => build_welcome_body(i18n),
    }
}

fn build_welcome_body<'a>(i18n: &'a i18n::I18n) -> Element<'a, Message> {
    let action_row = Row::new()
        .spacing(theme::spacing::SM)
        .push(button::primary(
            i18n.open_repository,
            Some(Message::OpenRepository),
        ))
        .push(button::secondary(
            i18n.init_repository,
            Some(Message::InitRepository),
        ));

    views::render_empty_state(
        i18n.app_tagline,
        i18n.welcome,
        i18n.open_repo_hint,
        Some(action_row.into()),
    )
}

fn build_changes_body<'a>(state: &'a AppState, i18n: &'a i18n::I18n) -> Element<'a, Message> {
    let can_stage_all = !state.unstaged_changes.is_empty() || !state.untracked_files.is_empty();
    let can_unstage_all = !state.staged_changes.is_empty();
    let mut changes_content = Column::new()
        .spacing(theme::spacing::XS)
        .height(Length::Fill)
        .push(
            Row::new()
                .spacing(theme::spacing::XS)
                .align_y(Alignment::Center)
                .push(Text::new(i18n.changes).size(13))
                .push(widgets::info_chip::<Message>(
                    state.workspace_change_count().to_string(),
                    BadgeTone::Neutral,
                )),
        )
        .push(Container::new(build_change_sections(state, i18n)).height(Length::Fill));

    if state.workspace_change_count() > 0 {
        changes_content = changes_content
            .push(iced::widget::rule::horizontal(1))
            .push(build_commit_footer(state, i18n));
    }

    let changes_panel = Container::new(
        Column::new()
            .spacing(0)
            .push(
                Container::new(
                    Row::new()
                        .spacing(theme::spacing::XS)
                        .align_y(Alignment::Center)
                        .push(button::tab("提交", true, None::<Message>))
                        .push(button::tab("搁置", false, None::<Message>))
                        .push(button::tab("储藏", false, None::<Message>))
                        .push(Space::new().width(Length::Fill)),
                )
                .padding([4, 6])
                .style(theme::frame_style(theme::Surface::Toolbar)),
            )
            .push(iced::widget::rule::horizontal(1))
            .push(
                Container::new(
                    Row::new()
                        .spacing(theme::spacing::XS)
                        .align_y(Alignment::Center)
                        .push(button::toolbar_icon("⟳", Some(Message::Refresh)))
                        .push(button::toolbar_icon(
                            "✓",
                            can_stage_all.then_some(Message::StageAll),
                        ))
                        .push(button::toolbar_icon(
                            "↶",
                            can_unstage_all.then_some(Message::UnstageAll),
                        ))
                        .push(Space::new().width(Length::Fill)),
                )
                .padding([4, 6])
                .style(theme::frame_style(theme::Surface::Nav)),
            )
            .push(iced::widget::rule::horizontal(1))
            .push(
                Container::new(changes_content)
                    .padding(theme::density::PANE_PADDING)
                    .height(Length::Fill),
            ),
    )
    .width(Length::FillPortion(5))
    .style(theme::panel_style(theme::Surface::Panel));

    let diff_panel = Container::new(
        Column::new()
            .spacing(0)
            .push(build_diff_header(state))
            .push(
                Container::new(build_diff_content(state, i18n))
                    .padding([6, 0])
                    .height(Length::Fill),
            ),
    )
    .width(Length::FillPortion(8))
    .style(theme::panel_style(theme::Surface::Panel));

    Row::new()
        .spacing(theme::spacing::SM)
        .height(Length::Fill)
        .push(changes_panel)
        .push(diff_panel)
        .into()
}

fn build_change_sections<'a>(state: &'a AppState, i18n: &'a i18n::I18n) -> Element<'a, Message> {
    if state.workspace_change_count() == 0 {
        let action_row = Row::new()
            .spacing(theme::spacing::XS)
            .push(button::secondary(i18n.refresh, Some(Message::Refresh)))
            .push(button::ghost("分支", Some(Message::ShowBranches)))
            .push(button::ghost("历史", Some(Message::ShowHistory)));

        return widgets::panel_empty_state(
            i18n.changes,
            i18n.clean_workspace,
            i18n.clean_workspace_detail,
            Some(action_row.into()),
        );
    }

    widgets::changelist::ChangesList::new(
        i18n,
        &state.staged_changes,
        &state.unstaged_changes,
        &state.untracked_files,
    )
    .with_selected_path(state.selected_change_path.as_deref())
    .with_select_handler(Message::SelectChange)
    .with_stage_handler(Message::StageFile)
    .with_unstage_handler(Message::UnstageFile)
    .view()
}

fn build_commit_footer<'a>(state: &'a AppState, i18n: &'a i18n::I18n) -> Element<'a, Message> {
    let staged_count = state.staged_changes.len();
    let can_commit = staged_count > 0;
    let status_text = if can_commit {
        format!("已暂存 {staged_count} 个文件，直接打开提交面板继续填写说明。")
    } else {
        "先勾选要提交的文件，提交按钮就会可用。".to_string()
    };

    Container::new(
        Row::new()
            .spacing(theme::spacing::XS)
            .align_y(Alignment::Center)
            .push(Text::new("提交准备").size(11))
            .push(
                Container::new(
                    Text::new(status_text)
                        .size(10)
                        .width(Length::Fill)
                        .wrapping(text::Wrapping::WordOrGlyph)
                        .color(theme::darcula::TEXT_SECONDARY),
                )
                .width(Length::Fill),
            )
            .push(button::primary(
                i18n.commit,
                can_commit.then_some(Message::Commit),
            )),
    )
    .padding([6, 0])
    .style(theme::frame_style(theme::Surface::Nav))
    .into()
}

fn build_diff_header<'a>(state: &'a AppState) -> Element<'a, Message> {
    let title = state
        .selected_change_path
        .as_ref()
        .map(|path| {
            std::path::Path::new(path)
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or(path.as_str())
                .to_string()
        })
        .unwrap_or_else(|| "差异".to_string());

    let path_hint = state.selected_change_path.as_ref().and_then(|path| {
        std::path::Path::new(path)
            .parent()
            .and_then(|value| value.to_str())
            .filter(|value| !value.is_empty())
            .map(|value| value.to_string())
    });
    let total_hunks = state.current_diff.as_ref().map(|diff| {
        diff.files
            .iter()
            .map(|file| file.hunks.len())
            .sum::<usize>()
    });

    Container::new(
        Row::new()
            .spacing(theme::spacing::XS)
            .align_y(Alignment::Center)
            .push(button::tab(title, true, None::<Message>))
            .push_maybe(path_hint.map(|hint| {
                Text::new(hint)
                    .size(10)
                    .color(theme::darcula::TEXT_SECONDARY)
            }))
            .push_maybe(state.current_diff.as_ref().map(|diff| {
                widgets::compact_chip::<Message>(
                    format!("{} 文件", diff.files.len()),
                    BadgeTone::Neutral,
                )
            }))
            .push_maybe(total_hunks.map(|count| {
                widgets::compact_chip::<Message>(format!("{} 区块", count), BadgeTone::Accent)
            }))
            .push(Space::new().width(Length::Fill))
            .push(button::tab(
                "统一",
                state.diff_presentation == DiffPresentation::Unified,
                (state.show_diff
                    && state.current_diff.is_some()
                    && state.diff_presentation != DiffPresentation::Unified)
                    .then_some(Message::ToggleDiffPresentation),
            ))
            .push(button::tab(
                "分栏",
                state.diff_presentation == DiffPresentation::Split,
                (state.show_diff
                    && state.current_diff.is_some()
                    && state.diff_presentation != DiffPresentation::Split)
                    .then_some(Message::ToggleDiffPresentation),
            ))
            .push(button::compact_ghost(
                "上个",
                Some(Message::NavigatePrevFile),
            ))
            .push(button::compact_ghost(
                "下个",
                Some(Message::NavigateNextFile),
            )),
    )
    .padding(theme::density::SECONDARY_BAR_PADDING)
    .style(theme::frame_style(theme::Surface::Toolbar))
    .into()
}

fn build_diff_content<'a>(state: &'a AppState, i18n: &'a i18n::I18n) -> Element<'a, Message> {
    if !state.show_diff || state.current_diff.is_none() {
        return widgets::panel_empty_state(
            i18n.diff,
            i18n.diff_empty,
            i18n.diff_empty_detail,
            Some(button::secondary("查看变更列表", Some(Message::ShowChanges)).into()),
        );
    }

    let diff = state.current_diff.as_ref().expect("diff checked");
    if diff.files.is_empty() {
        return widgets::panel_empty_state(
            i18n.diff,
            i18n.no_changes,
            i18n.diff_empty_detail,
            Some(button::secondary(i18n.refresh, Some(Message::Refresh)).into()),
        );
    }

    match state.diff_presentation {
        DiffPresentation::Unified => widgets::diff_viewer::DiffViewer::new(diff).view(),
        DiffPresentation::Split => widgets::split_diff_viewer::SplitDiffViewer::new(diff).view(),
    }
}

fn build_conflict_body<'a>(state: &'a AppState) -> Element<'a, Message> {
    if state.conflict_files.is_empty() {
        return views::render_empty_state(
            "冲突",
            "当前没有冲突文件",
            "如果仓库仍处于异常状态，先刷新一次仓库状态。",
            Some(button::secondary("返回变更", Some(Message::ShowChanges)).into()),
        );
    }

    if state.conflict_merge_index.is_some() {
        return if let Some(resolver) = state.conflict_resolver.as_ref() {
            Container::new(resolver.view().map(Message::ConflictResolverMessage))
                .height(Length::Fill)
                .style(theme::panel_style(theme::Surface::Editor))
                .into()
        } else {
            widgets::panel_empty_state(
                "冲突",
                "暂时无法打开三栏合并视图",
                "请返回冲突列表后重新选择文件。",
                Some(button::secondary("返回列表", Some(Message::ShowConflicts)).into()),
            )
        };
    }

    let selected_index = state
        .selected_conflict_index
        .or_else(|| (!state.conflict_files.is_empty()).then_some(0));
    let total_hunks = state
        .conflict_files
        .iter()
        .map(|conflict| conflict.hunks.len())
        .sum::<usize>();
    let total_manual_conflicts = state
        .conflict_files
        .iter()
        .map(summarize_conflict)
        .map(|summary| summary.manual_conflicts)
        .sum::<usize>();
    let total_auto_resolvable = state
        .conflict_files
        .iter()
        .map(summarize_conflict)
        .map(|summary| summary.auto_resolvable)
        .sum::<usize>();

    let summary_bar = Container::new(
        Row::new()
            .spacing(theme::spacing::SM)
            .align_y(Alignment::Center)
            .push(widgets::info_chip::<Message>(
                format!("冲突文件 {}", state.conflict_files.len()),
                BadgeTone::Warning,
            ))
            .push(widgets::info_chip::<Message>(
                format!("冲突块 {}", total_hunks),
                BadgeTone::Neutral,
            ))
            .push(widgets::info_chip::<Message>(
                format!("需手工合并 {}", total_manual_conflicts),
                if total_manual_conflicts > 0 {
                    BadgeTone::Warning
                } else {
                    BadgeTone::Success
                },
            ))
            .push(widgets::info_chip::<Message>(
                format!("可直接处理 {}", total_auto_resolvable),
                BadgeTone::Accent,
            ))
            .push(Space::new().width(Length::Fill))
            .push(
                Text::new("列表页先做整文件判断；真正复杂的文件再进入三栏合并。")
                    .size(11)
                    .color(theme::darcula::TEXT_SECONDARY),
            ),
    )
    .padding([8, 10])
    .style(theme::panel_style(theme::Surface::Raised));

    let table_header = Container::new(
        Row::new()
            .spacing(theme::spacing::SM)
            .align_y(Alignment::Center)
            .push(
                Text::new("名称")
                    .size(10)
                    .width(Length::FillPortion(6))
                    .color(theme::darcula::TEXT_SECONDARY),
            )
            .push(
                Text::new("您的更改")
                    .size(10)
                    .width(Length::FillPortion(2))
                    .color(theme::darcula::TEXT_SECONDARY),
            )
            .push(
                Text::new("他们的更改")
                    .size(10)
                    .width(Length::FillPortion(2))
                    .color(theme::darcula::TEXT_SECONDARY),
            )
            .push(
                Text::new("状态")
                    .size(10)
                    .width(Length::FillPortion(2))
                    .color(theme::darcula::TEXT_SECONDARY),
            ),
    )
    .padding([6, 8])
    .style(theme::panel_style(theme::Surface::Raised));

    let file_rows = state.conflict_files.iter().enumerate().fold(
        Column::new().spacing(2).width(Length::Fill),
        |column, (index, conflict)| {
            column.push(build_conflict_list_row(
                index,
                conflict,
                state.selected_conflict_index == Some(index),
            ))
        },
    );

    let list_panel = Container::new(
        Column::new()
            .spacing(theme::spacing::XS)
            .push(table_header)
            .push(
                scrollable::styled(Container::new(file_rows).width(Length::Fill))
                    .width(Length::Fill)
                    .height(Length::Fill),
            ),
    )
    .padding([10, 10])
    .width(Length::FillPortion(3))
    .height(Length::Fill)
    .style(theme::panel_style(theme::Surface::Panel));

    let action_panel = selected_index
        .and_then(|index| {
            state
                .conflict_files
                .get(index)
                .map(|conflict| build_conflict_action_panel(index, conflict))
        })
        .unwrap_or_else(|| {
            widgets::panel_empty_state(
                "冲突",
                "还没有选中冲突文件",
                "从左侧列表选择一个文件后，可直接接受一侧版本或进入三栏合并。",
                None,
            )
        });

    let footer = Row::new()
        .spacing(theme::spacing::XS)
        .align_y(Alignment::Center)
        .push(button::ghost(
            "返回变更",
            Some(Message::CloseConflictResolver),
        ))
        .push(button::secondary(
            "刷新",
            Some(Message::ConflictResolverMessage(
                ConflictResolverMessage::Refresh,
            )),
        ))
        .push(Space::new().width(Length::Fill))
        .push(
            Text::new("选中一行后，右侧操作面板会显示与 PhpStorm 类似的快捷入口。")
                .size(10)
                .color(theme::darcula::TEXT_SECONDARY),
        );

    Container::new(
        Column::new()
            .spacing(theme::spacing::MD)
            .height(Length::Fill)
            .push(widgets::section_header(
                "冲突",
                "解决冲突文件",
                "列表页用于快速决定整文件取舍，三栏页用于逐块确认最终结果。",
            ))
            .push(summary_bar)
            .push(
                Row::new()
                    .spacing(theme::spacing::SM)
                    .height(Length::Fill)
                    .push(list_panel)
                    .push(
                        Container::new(action_panel)
                            .width(Length::FillPortion(2))
                            .height(Length::Fill)
                            .style(theme::panel_style(theme::Surface::Editor)),
                    ),
            )
            .push(footer),
    )
    .padding([10, 12])
    .height(Length::Fill)
    .into()
}

#[derive(Debug, Clone, Copy, Default)]
struct ConflictListSummary {
    hunk_count: usize,
    manual_conflicts: usize,
    auto_resolvable: usize,
    ours_changed: usize,
    theirs_changed: usize,
}

fn build_conflict_list_row<'a>(
    index: usize,
    conflict: &'a ThreeWayDiff,
    is_selected: bool,
) -> Element<'a, Message> {
    let summary = summarize_conflict(conflict);
    let file_status = FileStatus::Conflict;
    let (file_name, parent_path) = split_workspace_path(&conflict.path);
    let status_label = if summary.manual_conflicts > 0 {
        format!("需合并 {}", summary.manual_conflicts)
    } else {
        "可直接处理".to_string()
    };

    let content = Row::new()
        .spacing(theme::spacing::SM)
        .align_y(Alignment::Center)
        .push(
            Row::new()
                .spacing(theme::spacing::SM)
                .width(Length::FillPortion(6))
                .align_y(Alignment::Center)
                .push(build_conflict_file_icon(file_status))
                .push(
                    Column::new()
                        .spacing(2)
                        .width(Length::Fill)
                        .push(
                            Row::new()
                                .spacing(theme::spacing::XS)
                                .align_y(Alignment::Center)
                                .push(
                                    Text::new(file_name)
                                        .size(12)
                                        .width(Length::Shrink)
                                        .wrapping(text::Wrapping::WordOrGlyph),
                                )
                                .push_maybe(is_selected.then(|| {
                                    widgets::info_chip::<Message>("当前", BadgeTone::Accent)
                                })),
                        )
                        .push(
                            Text::new(parent_path)
                                .size(10)
                                .color(theme::darcula::TEXT_SECONDARY)
                                .wrapping(text::Wrapping::None),
                        ),
                ),
        )
        .push(build_conflict_status_cell(
            "当前分支",
            if summary.ours_changed > 0 {
                format!("已修改 · {} 块", summary.ours_changed)
            } else {
                "无差异".to_string()
            },
            BadgeTone::Accent,
        ))
        .push(build_conflict_status_cell(
            "传入分支",
            if summary.theirs_changed > 0 {
                format!("已修改 · {} 块", summary.theirs_changed)
            } else {
                "无差异".to_string()
            },
            BadgeTone::Danger,
        ))
        .push(
            Column::new()
                .spacing(2)
                .width(Length::FillPortion(2))
                .push(widgets::info_chip::<Message>(
                    status_label,
                    if summary.manual_conflicts > 0 {
                        BadgeTone::Warning
                    } else {
                        BadgeTone::Success
                    },
                ))
                .push(
                    Text::new(format!("{} 个冲突块", summary.hunk_count))
                        .size(10)
                        .color(theme::darcula::TEXT_SECONDARY),
                ),
        );

    Button::new(
        Row::new()
            .push(
                Container::new(Space::new().width(Length::Fixed(3.0)))
                    .width(Length::Fixed(3.0))
                    .height(Length::Fill)
                    .style(conflict_row_strip_style(is_selected)),
            )
            .push(
                Container::new(content)
                    .padding([8, 10])
                    .width(Length::Fill)
                    .style(theme::panel_style(if is_selected {
                        theme::Surface::Selection
                    } else {
                        theme::Surface::Panel
                    })),
            ),
    )
    .width(Length::Fill)
    .style(theme::button_style(theme::ButtonTone::Ghost))
    .on_press(Message::SelectConflict(index))
    .into()
}

fn build_conflict_action_panel<'a>(
    index: usize,
    conflict: &'a ThreeWayDiff,
) -> Element<'a, Message> {
    let summary = summarize_conflict(conflict);
    let (file_name, parent_path) = split_workspace_path(&conflict.path);

    Container::new(
        Column::new()
            .spacing(theme::spacing::SM)
            .push(widgets::section_header(
                "当前文件",
                "快速解决",
                "和 JetBrains 一样，先在这里决定整文件取舍；需要精细处理时再进入三栏合并。",
            ))
            .push(
                Column::new()
                    .spacing(2)
                    .push(Text::new(file_name).size(15))
                    .push(
                        Text::new(parent_path)
                            .size(10)
                            .color(theme::darcula::TEXT_SECONDARY)
                            .wrapping(text::Wrapping::WordOrGlyph),
                    ),
            )
            .push(
                Row::new()
                    .spacing(theme::spacing::XS)
                    .push(widgets::info_chip::<Message>(
                        format!("冲突块 {}", summary.hunk_count),
                        BadgeTone::Warning,
                    ))
                    .push(widgets::info_chip::<Message>(
                        format!("需手工合并 {}", summary.manual_conflicts),
                        if summary.manual_conflicts > 0 {
                            BadgeTone::Warning
                        } else {
                            BadgeTone::Success
                        },
                    ))
                    .push(widgets::info_chip::<Message>(
                        format!("自动可处理 {}", summary.auto_resolvable),
                        BadgeTone::Neutral,
                    )),
            )
            .push(
                Row::new()
                    .spacing(theme::spacing::SM)
                    .push(build_conflict_stat_card(
                        "当前分支",
                        summary.ours_changed.to_string(),
                        "发生变化的块",
                        BadgeTone::Accent,
                    ))
                    .push(build_conflict_stat_card(
                        "传入分支",
                        summary.theirs_changed.to_string(),
                        "发生变化的块",
                        BadgeTone::Danger,
                    )),
            )
            .push(iced::widget::rule::horizontal(1))
            .push(build_conflict_action_button(
                "<<",
                "接受您的更改",
                "直接采用当前分支版本并将该文件标记为已解决。",
                theme::ButtonTone::Secondary,
                Message::ResolveConflictWithOurs(index),
            ))
            .push(
                build_conflict_action_button(
                    ">>",
                    "接受他们的更改",
                    "直接采用传入分支版本，适合明确以对方内容为准的文件。",
                    theme::ButtonTone::Danger,
                    Message::ResolveConflictWithTheirs(index),
                ),
            )
            .push(build_conflict_action_button(
                "<>",
                "合并...",
                "打开三栏合并编辑器，逐块确认最终结果。",
                theme::ButtonTone::Primary,
                Message::OpenConflictMerge(index),
            ))
            .push(iced::widget::rule::horizontal(1))
            .push(
                Text::new(
                    "建议：配置文件、锁文件等单方可信内容可直接接受；业务代码优先进入“三栏合并”逐块确认。",
                )
                .size(11)
                .color(theme::darcula::TEXT_SECONDARY)
                .wrapping(text::Wrapping::WordOrGlyph),
            ),
    )
    .padding([12, 12])
    .width(Length::Fill)
    .style(theme::panel_style(theme::Surface::Panel))
    .into()
}

fn build_conflict_file_icon<'a>(status: FileStatus) -> Element<'a, Message> {
    Container::new(
        Text::new(status.symbol())
            .size(11)
            .color(theme::darcula::TEXT_PRIMARY),
    )
    .width(Length::Fixed(18.0))
    .height(Length::Fixed(18.0))
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .style(theme::panel_style(theme::Surface::Danger))
    .into()
}

fn build_conflict_status_cell<'a>(
    title: &'a str,
    label: String,
    tone: BadgeTone,
) -> Element<'a, Message> {
    Container::new(
        Column::new()
            .spacing(2)
            .push(widgets::info_chip::<Message>(label, tone))
            .push(
                Text::new(title)
                    .size(9)
                    .color(theme::darcula::TEXT_SECONDARY),
            ),
    )
    .width(Length::FillPortion(2))
    .into()
}

fn build_conflict_stat_card<'a>(
    title: &'a str,
    value: String,
    detail: &'a str,
    tone: BadgeTone,
) -> Element<'a, Message> {
    Container::new(
        Column::new()
            .spacing(2)
            .push(
                Text::new(title)
                    .size(10)
                    .color(theme::darcula::TEXT_SECONDARY),
            )
            .push(Text::new(value).size(18))
            .push(widgets::info_chip::<Message>(detail, tone)),
    )
    .width(Length::FillPortion(1))
    .padding([8, 10])
    .style(theme::panel_style(theme::Surface::Raised))
    .into()
}

fn build_conflict_action_button<'a>(
    icon: &'a str,
    title: &'a str,
    detail: &'a str,
    tone: theme::ButtonTone,
    message: Message,
) -> Element<'a, Message> {
    Button::new(
        Container::new(
            Row::new()
                .spacing(theme::spacing::SM)
                .align_y(Alignment::Center)
                .push(
                    Container::new(Text::new(icon).size(11).color(theme::darcula::TEXT_PRIMARY))
                        .width(Length::Fixed(28.0))
                        .height(Length::Fixed(22.0))
                        .center_x(Length::Fill)
                        .center_y(Length::Fill)
                        .style(conflict_action_icon_style(tone)),
                )
                .push(
                    Column::new()
                        .spacing(1)
                        .width(Length::Fill)
                        .push(Text::new(title).size(12))
                        .push(
                            Text::new(detail)
                                .size(10)
                                .color(theme::darcula::TEXT_SECONDARY)
                                .wrapping(text::Wrapping::WordOrGlyph),
                        ),
                )
                .push(
                    Text::new(">")
                        .size(11)
                        .color(theme::darcula::TEXT_SECONDARY),
                ),
        )
        .padding([7, 9])
        .width(Length::Fill),
    )
    .width(Length::Fill)
    .style(theme::button_style(tone))
    .on_press(message)
    .into()
}

fn summarize_conflict(conflict: &ThreeWayDiff) -> ConflictListSummary {
    let mut summary = ConflictListSummary {
        hunk_count: conflict.hunks.len(),
        ..ConflictListSummary::default()
    };

    for hunk in &conflict.hunks {
        match summarize_conflict_hunk(hunk) {
            ConflictHunkType::Modified => summary.manual_conflicts += 1,
            ConflictHunkType::OursOnly
            | ConflictHunkType::TheirsOnly
            | ConflictHunkType::Unchanged => summary.auto_resolvable += 1,
        }

        let mut ours_changed = false;
        let mut theirs_changed = false;

        for line in &hunk.lines {
            match line.line_type {
                ConflictLineType::OursOnly => ours_changed = true,
                ConflictLineType::TheirsOnly => theirs_changed = true,
                ConflictLineType::Modified => {
                    ours_changed = true;
                    theirs_changed = true;
                }
                ConflictLineType::Unchanged
                | ConflictLineType::Empty
                | ConflictLineType::ConflictMarker => {}
            }
        }

        if ours_changed {
            summary.ours_changed += 1;
        }
        if theirs_changed {
            summary.theirs_changed += 1;
        }
    }

    summary
}

fn summarize_conflict_hunk(hunk: &ConflictHunk) -> ConflictHunkType {
    let mut ours_only = 0usize;
    let mut theirs_only = 0usize;
    let mut modified = 0usize;
    let mut unchanged = 0usize;

    for line in &hunk.lines {
        match line.line_type {
            ConflictLineType::OursOnly => ours_only += 1,
            ConflictLineType::TheirsOnly => theirs_only += 1,
            ConflictLineType::Modified => modified += 1,
            ConflictLineType::Unchanged => unchanged += 1,
            ConflictLineType::Empty | ConflictLineType::ConflictMarker => {}
        }
    }

    if modified > 0 {
        ConflictHunkType::Modified
    } else if ours_only > 0 && theirs_only == 0 {
        ConflictHunkType::OursOnly
    } else if theirs_only > 0 && ours_only == 0 {
        ConflictHunkType::TheirsOnly
    } else if unchanged > 0 && ours_only == 0 && theirs_only == 0 {
        ConflictHunkType::Unchanged
    } else {
        ConflictHunkType::Modified
    }
}

fn split_workspace_path(path: &str) -> (String, String) {
    let path = Path::new(path);
    let file_name = path
        .file_name()
        .and_then(|segment| segment.to_str())
        .filter(|segment| !segment.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| path.display().to_string());

    let parent = path
        .parent()
        .and_then(|segment| {
            let rendered = segment.display().to_string();
            (!rendered.is_empty() && rendered != ".").then_some(rendered)
        })
        .unwrap_or_else(|| "仓库根目录".to_string());

    (file_name, parent)
}

fn conflict_row_strip_style(selected: bool) -> impl Fn(&Theme) -> container::Style {
    move |_theme| container::Style {
        background: Some(Background::Color(if selected {
            theme::darcula::ACCENT
        } else {
            theme::darcula::BG_PANEL
        })),
        border: Border {
            width: 0.0,
            color: Color::TRANSPARENT,
            radius: 0.0.into(),
        },
        ..Default::default()
    }
}

fn conflict_action_icon_style(tone: theme::ButtonTone) -> impl Fn(&Theme) -> container::Style {
    move |_theme| {
        let (background, border) = match tone {
            theme::ButtonTone::Primary => (
                blend(theme::darcula::BG_PANEL, theme::darcula::ACCENT, 0.32),
                theme::darcula::ACCENT.scale_alpha(0.72),
            ),
            theme::ButtonTone::Danger => (
                blend(theme::darcula::BG_PANEL, theme::darcula::DANGER, 0.28),
                theme::darcula::DANGER.scale_alpha(0.68),
            ),
            _ => (
                blend(theme::darcula::BG_PANEL, theme::darcula::BG_RAISED, 0.82),
                theme::darcula::BORDER.scale_alpha(0.76),
            ),
        };

        container::Style {
            background: Some(Background::Color(background)),
            border: Border {
                width: 1.0,
                color: border,
                radius: 3.0.into(),
            },
            ..Default::default()
        }
    }
}

fn blend(base: Color, overlay: Color, amount: f32) -> Color {
    let amount = amount.clamp(0.0, 1.0);
    Color {
        r: (base.r * (1.0 - amount)) + (overlay.r * amount),
        g: (base.g * (1.0 - amount)) + (overlay.g * amount),
        b: (base.b * (1.0 - amount)) + (overlay.b * amount),
        a: (base.a * (1.0 - amount)) + (overlay.a * amount),
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    OpenRepository,
    InitRepository,
    Refresh,
    AutoRefreshTick(Instant),
    AutoRemoteCheckFinished(AutoRemoteCheckResult),
    ToastTick(Instant),
    CloseRepository,
    ShowChanges,
    ShowConflicts,
    DismissFeedback,
    DismissToast,
    StageFile(String),
    UnstageFile(String),
    StageAll,
    UnstageAll,
    KeyboardShortcut(ShortcutAction),
    Commit,
    Pull,
    Push,
    ToggleToolbarRemoteMenu(ToolbarRemoteAction),
    CloseToolbarRemoteMenu,
    ToolbarRemoteActionSelected {
        action: ToolbarRemoteAction,
        remote: String,
    },
    Stash,
    ShowBranches,
    ShowHistory,
    ShowRemotes,
    ShowTags,
    ShowRebase,
    SwitchProject(PathBuf),
    SelectChange(String),
    ToggleDiffPresentation,
    NavigatePrevFile,
    NavigateNextFile,
    OpenConflictResolver,
    CloseConflictResolver,
    SelectConflict(usize),
    OpenConflictMerge(usize),
    ResolveConflictWithOurs(usize),
    ResolveConflictWithTheirs(usize),
    ConflictResolverMessage(ConflictResolverMessage),
    CommitDialogMessage(CommitDialogMessage),
    BranchPopupMessage(BranchPopupMessage),
    HistoryMessage(HistoryMessage),
    RemoteDialogMessage(RemoteDialogMessage),
    TagDialogMessage(TagDialogMessage),
    StashPanelMessage(StashPanelMessage),
    RebaseEditorMessage(RebaseEditorMessage),
}
