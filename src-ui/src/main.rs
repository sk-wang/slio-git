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
use crate::state::{
    is_docked_auxiliary_view, AppState, AuxiliaryView, DiffPresentation, GitToolWindowTab,
    ShellSection, ToolbarRemoteAction,
};
use iced::widget::operation::{scroll_to, AbsoluteOffset};
use iced::widget::Id;
use crate::theme::BadgeTone;
use crate::views::main_window::MainWindow;
use crate::views::{
    branch_popup::{self, BranchPopupMessage},
    commit_dialog::CommitDialogMessage,
    history_view::{self, HistoryMessage},
    rebase_editor::{self, RebaseEditorMessage},
    remote_dialog::{self, RemoteDialogMessage},
    stash_panel::{self, StashPanelMessage},
    tag_dialog::{self, TagDialogMessage},
};
use crate::widgets::conflict_resolver::{ConflictResolverMessage, ResolutionOption};
use crate::widgets::{button, commit_panel, file_picker, scrollable, OptionalPush};
use git_core::index::Change;
use git_core::{
    diff::{ConflictHunk, ConflictHunkType, ConflictLineType, ConflictResolution, ThreeWayDiff},
    Repository,
};
use iced::widget::{container, mouse_area, opaque, stack, text, Button, Column, Container, Row, Space, Text};
use iced::{
    time, Alignment, Background, Border, Color, Element, Length, Point, Subscription, Task, Theme,
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

    // Load embedded window icon (64x64 RGBA)
    let icon_data = include_bytes!("../assets/icon_64.rgba");
    let window_icon = iced::window::icon::from_rgba(icon_data.to_vec(), 64, 64).ok();

    let mut window_settings = iced::window::Settings {
        size: iced::Size::new(
            theme::layout::WINDOW_DEFAULT_WIDTH,
            theme::layout::WINDOW_DEFAULT_HEIGHT,
        ),
        min_size: Some(iced::Size::new(
            theme::layout::WINDOW_MIN_WIDTH,
            theme::layout::WINDOW_MIN_HEIGHT,
        )),
        ..Default::default()
    };
    if let Some(icon) = window_icon {
        window_settings.icon = Some(icon);
    }

    iced::application(|| (AppState::restore(), Task::none()), update, view)
        .title("slio-git")
        .default_font(cjk_font)
        .theme(app_theme)
        .window(window_settings)
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
            state.show_project_dropdown = false;
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
            if state.selected_change_path.as_deref() != Some(&path) {
                let _ = state.select_change(path.clone());
            }
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
            if state.selected_change_path.as_deref() != Some(&path) {
                let _ = state.select_change(path.clone());
            }
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
        Message::ToggleProjectDropdown => {
            state.show_project_dropdown = !state.show_project_dropdown;
            state.show_branch_dropdown = false; // close branch if open
        }
        Message::ToggleFileDisplayMode => {
            state.file_display_mode = match state.file_display_mode {
                state::FileDisplayMode::Flat => state::FileDisplayMode::Tree,
                state::FileDisplayMode::Tree => state::FileDisplayMode::Flat,
            };
        }
        Message::ToggleStagedCollapsed => {
            // Toggled via AppState; stored in shell for now
        }
        Message::ToggleUnstagedCollapsed => {
            // Toggled via AppState; stored in shell for now
        }
        Message::StageHunk(path, hunk_index) => {
            if let Some(repo) = &state.current_repository {
                let file_path = std::path::Path::new(&path);
                if let Err(e) = git_core::stage_hunk(repo, file_path, hunk_index) {
                    report_async_failure(
                        state,
                        "暂存区块失败",
                        e.to_string(),
                        "workspace.stage_hunk",
                        "workspace.stage_hunk",
                    );
                } else {
                    return update(state, Message::Refresh);
                }
            }
        }
        Message::UnstageHunk(path, hunk_index) => {
            if let Some(repo) = &state.current_repository {
                let file_path = std::path::Path::new(&path);
                if let Err(e) = git_core::unstage_hunk(repo, file_path, hunk_index) {
                    report_async_failure(
                        state,
                        "取消暂存区块失败",
                        e.to_string(),
                        "workspace.unstage_hunk",
                        "workspace.unstage_hunk",
                    );
                } else {
                    return update(state, Message::Refresh);
                }
            }
        }
        Message::ShowFileHistory(path) => {
            // Switch to Log tab with path filter set
            state.shell.git_tool_window_tab = state::GitToolWindowTab::Log;
            if let Some(tab) = state.log_tabs.get_mut(state.active_log_tab) {
                tab.path_filter = Some(path);
            }
            state.change_context_menu_path = None;
            if let Some(repo) = state.current_repository.clone() {
                state.history_view.load_history(&repo);
            }
        }
        Message::ToggleBlameAnnotation => {
            state.blame_active = !state.blame_active;
        }
        Message::CancelNetworkOperation => {
            state.network_operation = None;
        }
        Message::TogglePullStrategy => {
            state.pull_strategy = match state.pull_strategy {
                state::PullStrategy::Merge => state::PullStrategy::Rebase,
                state::PullStrategy::Rebase => state::PullStrategy::Merge,
            };
        }
        Message::ForcePushCurrent => {
            if let Some(repo) = &state.current_repository {
                if let Ok(Some(branch)) = repo.current_branch() {
                    let remote = repo
                        .current_upstream_remote()
                        .unwrap_or_else(|| "origin".to_string());
                    state.network_operation = Some(state::NetworkOperation {
                        label: format!("强制推送 {} → {}", branch, remote),
                        progress: None,
                        status: Some("--force-with-lease".to_string()),
                    });
                    match git_core::force_push(repo, &remote, &branch) {
                        Ok(()) => {
                            state.network_operation = None;
                            state.set_success(
                                "强制推送成功",
                                Some(format!("{branch} → {remote}")),
                                "workspace.push.force",
                            );
                        }
                        Err(e) => {
                            state.network_operation = None;
                            report_async_failure(
                                state,
                                "强制推送失败",
                                e.to_string(),
                                "workspace.push.force",
                                "workspace.push.force",
                            );
                        }
                    }
                }
            }
        }
        Message::SetUpstreamAndPush { branch, remote } => {
            if let Some(repo) = &state.current_repository {
                let upstream = format!("{remote}/{branch}");
                if let Err(e) = repo.set_branch_upstream(&branch, &upstream) {
                    report_async_failure(
                        state,
                        "设置上游失败",
                        e.to_string(),
                        "workspace.push.upstream",
                        "workspace.push.upstream",
                    );
                } else {
                    return update(state, Message::Push);
                }
            }
        }
        Message::ShowWorktrees => {
            if let Ok(repo) = require_repository(state) {
                state.worktree_state.load_worktrees(&repo);
                state.open_auxiliary_view(state::AuxiliaryView::Worktrees);
            }
        }
        Message::WorktreeMessage(msg) => {
            use views::worktree_view::WorktreeMessage;
            match msg {
                WorktreeMessage::Refresh => {
                    if let Ok(repo) = require_repository(state) {
                        state.worktree_state.load_worktrees(&repo);
                    }
                }
                WorktreeMessage::Remove(path) => {
                    if let Ok(repo) = require_repository(state) {
                        state.worktree_state.remove_worktree(&repo, path);
                    }
                }
                WorktreeMessage::Close => state.close_auxiliary_view(),
            }
        }
        Message::OpenInEditor(path) => {
            state.change_context_menu_path = None;
            if let Some(repo) = &state.current_repository {
                let full_path = repo.path().join(&path);
                if let Err(e) = std::process::Command::new("open").arg(&full_path).spawn() {
                    log::warn!("Failed to open file in editor: {}", e);
                }
            }
        }
        Message::SwitchProject(path) => {
            state.show_project_dropdown = false;
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
        Message::PrevHunk => return navigate_hunk(state, -1),
        Message::NextHunk => return navigate_hunk(state, 1),
        Message::TrackChangeContextMenuCursor(position) => {
            state.track_change_context_menu_cursor(position);
        }
        Message::OpenChangeContextMenu(path) => {
            state.open_change_context_menu(path);
        }
        Message::CloseChangeContextMenu => {
            state.close_change_context_menu();
        }
        Message::RevertFile(path) => {
            if let Some(repo) = state.current_repository.as_ref() {
                if let Err(error) = git_core::index::discard_file(repo, std::path::Path::new(&path)) {
                    report_async_failure(
                        state,
                        "回滚文件失败",
                        error.to_string(),
                        "workspace.revert",
                        "workspace.revert",
                    );
                } else {
                    state.refresh_changes();
                    state.close_change_context_menu();
                    state.set_success("文件已回滚", Some(path), "workspace.revert");
                }
            }
        }
        Message::CopyChangePath(path) => {
            if let Err(error) = copy_text_to_clipboard(&path) {
                report_async_failure(
                    state,
                    "复制路径失败",
                    error,
                    "workspace.copy_path",
                    "workspace.copy_path",
                );
            } else {
                state.set_success("路径已复制到剪贴板", Some(path), "workspace.copy_path");
                state.close_change_context_menu();
            }
        }
        Message::KeyboardShortcut(action) => match action {
            ShortcutAction::StageFile => {
                if let Some(path) = state.selected_change_path.clone() {
                    let can_stage = state.unstaged_changes.iter().any(|c| c.path == path)
                        || state.untracked_files.iter().any(|c| c.path == path);
                    if can_stage {
                        if let Err(error) = state.stage_file(path) {
                            report_async_failure(
                                state,
                                "暂存文件失败",
                                error,
                                "keyboard.stage",
                                "keyboard.stage",
                            );
                        }
                    } else {
                        state.set_info("文件已暂存或无法暂存", None, "keyboard.stage");
                    }
                } else {
                    state.set_info("请先选择一个文件", None, "keyboard.stage");
                }
            }
            ShortcutAction::UnstageFile => {
                if let Some(path) = state.selected_change_path.clone() {
                    let can_unstage = state.staged_changes.iter().any(|c| c.path == path);
                    if can_unstage {
                        if let Err(error) = state.unstage_file(path) {
                            report_async_failure(
                                state,
                                "取消暂存失败",
                                error,
                                "keyboard.unstage",
                                "keyboard.unstage",
                            );
                        }
                    } else {
                        state.set_info("文件未暂存", None, "keyboard.unstage");
                    }
                } else {
                    state.set_info("请先选择一个文件", None, "keyboard.unstage");
                }
            }
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
            ShortcutAction::OpenCommitDialog => {
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
            ShortcutAction::ToggleAmendCommitMode => {
                if let Err(error) = toggle_commit_dialog_amend_mode(state) {
                    report_async_failure(
                        state,
                        "无法切换 amend 模式",
                        error,
                        "workspace.commit",
                        "workspace.commit.amend",
                    );
                }
            }
            ShortcutAction::OpenPushDialog => {
                if let Err(error) = open_remote_dialog(state) {
                    report_async_failure(
                        state,
                        "无法打开远程面板",
                        error,
                        "workspace.push",
                        "workspace.push",
                    );
                } else {
                    state.set_info("已打开远程面板", None, "keyboard.shortcut");
                }
            }
            ShortcutAction::ShowFileDiff => {
                if let Some(path) = state.selected_change_path.clone() {
                    if let Err(error) = state.load_diff_for_file(&path) {
                        report_async_failure(
                            state,
                            "加载差异失败",
                            error,
                            "keyboard.diff",
                            "keyboard.diff",
                        );
                    }
                } else {
                    state.set_info("请先选择一个文件以查看差异", None, "keyboard.diff");
                }
            }
            ShortcutAction::NavigatePrevFile => select_relative_file(state, -1),
            ShortcutAction::NavigateNextFile => select_relative_file(state, 1),
            ShortcutAction::PrevHunk => return update(state, Message::PrevHunk),
            ShortcutAction::NextHunk => return update(state, Message::NextHunk),
            ShortcutAction::Commit => {
                if let Err(error) = submit_commit_dialog(state) {
                    report_async_failure(
                        state,
                        "提交失败",
                        error,
                        "workspace.commit",
                        "workspace.commit",
                    );
                }
            }
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
                // Switch to IDEA-style Pull dialog
                state.remote_dialog.mode = views::remote_dialog::RemoteDialogMode::Pull;
                state.set_info(
                    "已打开拉取面板",
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
                // Switch to IDEA-style Push dialog
                state.remote_dialog.mode = views::remote_dialog::RemoteDialogMode::Push;
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
        Message::CloseAuxiliary => state.close_auxiliary_view(),
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
            // Toggle IDEA-style floating branch dropdown
            state.show_branch_dropdown = !state.show_branch_dropdown;
            if state.show_branch_dropdown {
                // Load branches when opening
                if let Some(repo) = state.current_repository.clone() {
                    state.branch_popup.load_branches(&repo);
                }
            }
        }
        Message::ShowHistory => {
            state.switch_git_tool_window_tab(GitToolWindowTab::Log);
            if let Some(repo) = state.current_repository.clone() {
                state.history_view.load_history(&repo);
                // Also load branches for the dashboard sidebar
                if state.branch_popup.local_branches.is_empty() {
                    state.branch_popup.load_branches(&repo);
                }
            }
        }
        Message::SwitchGitToolWindowTab(tab) => {
            state.switch_git_tool_window_tab(tab);
            if tab == GitToolWindowTab::Log {
                if let Some(repo) = state.current_repository.clone() {
                    state.history_view.load_history(&repo);
                    if state.branch_popup.local_branches.is_empty() {
                        state.branch_popup.load_branches(&repo);
                    }
                }
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
            CommitDialogMessage::SetAmendMode(enabled) => {
                let result = if enabled {
                    switch_commit_dialog_to_amend(state)
                } else {
                    switch_commit_dialog_to_new_commit_mode(state)
                };

                if let Err(error) = result {
                    state.commit_dialog.set_error(error.clone());
                    report_async_failure(
                        state,
                        "无法切换提交模式",
                        error,
                        "workspace.commit",
                        "workspace.commit.amend",
                    );
                }
            }
            CommitDialogMessage::CancelPressed => state.close_auxiliary_view(),
            CommitDialogMessage::CommitAndPushPressed => {
                // Commit first, then push
                if let Err(error) = submit_commit_dialog(state) {
                    state.commit_dialog.set_error(error.clone());
                    report_async_failure(
                        state,
                        "提交失败",
                        error,
                        "workspace.commit_and_push",
                        "workspace.commit_and_push",
                    );
                } else {
                    // Commit succeeded, now push
                    return update(state, Message::Push);
                }
            }
            CommitDialogMessage::ToggleRecentMessages => {
                // Load recent messages from history file
                if let Some(repo) = &state.current_repository {
                    state.recent_commit_messages =
                        git_core::load_recent_messages(repo.path());
                }
            }
            CommitDialogMessage::SelectRecentMessage(index) => {
                // Insert selected message into the commit editor
                if let Some(msg) = state.recent_commit_messages.get(index).cloned() {
                    state.commit_dialog.message = msg.clone();
                    state.commit_dialog.message_editor =
                        iced::widget::text_editor::Content::with_text(&msg);
                }
            }
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
                    let selection_changed = state.branch_popup.set_search_query(query);
                    if selection_changed {
                        if let Ok(repo) = require_repository(state) {
                            state.branch_popup.load_selected_branch_history(&repo);
                        }
                    }
                }
                BranchPopupMessage::ClearSearch => {
                    let selection_changed = state.branch_popup.set_search_query(String::new());
                    if selection_changed {
                        if let Ok(repo) = require_repository(state) {
                            state.branch_popup.load_selected_branch_history(&repo);
                        }
                    }
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
                BranchPopupMessage::PrepareDeleteBranch(name) => {
                    if let Ok(repo) = require_repository(state) {
                        state.branch_popup.prepare_delete_branch(&repo, name);
                    }
                }
                BranchPopupMessage::ConfirmDeleteBranch => {
                    if let Some(name) = state.branch_popup.pending_delete_branch.take() {
                        state.branch_popup.pending_delete_not_merged = false;
                        return update(
                            state,
                            Message::BranchPopupMessage(BranchPopupMessage::DeleteBranch(name)),
                        );
                    }
                }
                BranchPopupMessage::CancelDeleteBranch => {
                    state.branch_popup.pending_delete_branch = None;
                    state.branch_popup.pending_delete_not_merged = false;
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
                BranchPopupMessage::Close => {
                    state.show_branch_dropdown = false;
                    state.close_auxiliary_view();
                }
            }
        }
        Message::HistoryMessage(message) => match message {
            HistoryMessage::Refresh => {
                if let Ok(repo) = require_repository(state) {
                    state.history_view.load_history(&repo);
                    state.history_view.context_menu_commit = None;
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
            HistoryMessage::TrackContextMenuCursor(position) => {
                state.history_view.track_context_menu_cursor(position);
            }
            HistoryMessage::OpenCommitContextMenu(commit_id) => {
                if let Ok(repo) = require_repository(state) {
                    state.history_view.select_commit(&repo, commit_id.clone());
                    state.history_view.context_menu_commit = Some(commit_id);
                    state.history_view.context_menu_anchor =
                        Some(state.history_view.context_menu_cursor);
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
                state.history_view.context_menu_anchor = None;
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
            HistoryMessage::ToggleMultiSelect(commit_id) => {
                let pos = state
                    .history_view
                    .multi_selected_commits
                    .iter()
                    .position(|id| id == &commit_id);
                if let Some(pos) = pos {
                    state.history_view.multi_selected_commits.remove(pos);
                } else {
                    state.history_view.multi_selected_commits.push(commit_id);
                }
            }
            HistoryMessage::SquashSelectedCommits => {
                let selected = &state.history_view.multi_selected_commits;
                if selected.len() < 2 {
                    state.set_error_with_source(
                        "无法压缩",
                        "请先选中至少两个连续提交",
                        "workspace.squash",
                    );
                } else {
                    // Validate contiguous: check all selected commits are adjacent in the list
                    let entry_ids: Vec<&str> = state
                        .history_view
                        .filtered_entries
                        .iter()
                        .map(|e| e.id.as_str())
                        .collect();
                    let positions: Vec<usize> = selected
                        .iter()
                        .filter_map(|id| entry_ids.iter().position(|e| *e == id.as_str()))
                        .collect();
                    let mut sorted = positions.clone();
                    sorted.sort();
                    let is_contiguous = sorted.len() >= 2
                        && sorted.windows(2).all(|w| w[1] == w[0] + 1);

                    if !is_contiguous {
                        state.set_error_with_source(
                            "无法压缩",
                            "仅支持连续提交的压缩",
                            "workspace.squash",
                        );
                    } else {
                        // Use the oldest selected commit as the squash target
                        let oldest_id = sorted
                            .last()
                            .and_then(|&i| entry_ids.get(i))
                            .map(|s| s.to_string());
                        if let Some(target) = oldest_id {
                            // Open interactive rebase from the oldest commit
                            return update(
                                state,
                                Message::HistoryMessage(
                                    HistoryMessage::OpenInteractiveRebaseFromCommit(target),
                                ),
                            );
                        }
                    }
                }
            }
            HistoryMessage::UncommitToHere(commit_id) => {
                if let Ok(repo) = require_repository(state) {
                    match git_core::uncommit_to_commit(&repo, &commit_id) {
                        Ok(()) => {
                            let _ = refresh_repository_after_action(state, &repo, false);
                            state.set_success(
                                "提交已撤销",
                                Some("改动已返回暂存区".to_string()),
                                "workspace.uncommit",
                            );
                        }
                        Err(e) => {
                            report_async_failure(
                                state,
                                "撤销提交失败",
                                e.to_string(),
                                "workspace.uncommit",
                                "workspace.uncommit",
                            );
                        }
                    }
                }
            }
            HistoryMessage::PushUpToCommit(commit_id) => {
                if let Ok(repo) = require_repository(state) {
                    if let Ok(Some(branch)) = repo.current_branch() {
                        let remote = repo
                            .current_upstream_remote()
                            .unwrap_or_else(|| "origin".to_string());
                        let refspec = format!("{}:refs/heads/{}", commit_id, branch);
                        match git_core::push(&repo, &remote, &refspec, None) {
                            Ok(()) => {
                                state.set_success(
                                    "推送成功",
                                    Some(format!("已推送到 {} 的 {}", &commit_id[..7.min(commit_id.len())], remote)),
                                    "workspace.push.up_to",
                                );
                            }
                            Err(e) => {
                                report_async_failure(
                                    state,
                                    "推送到此提交失败",
                                    e.to_string(),
                                    "workspace.push.up_to",
                                    "workspace.push.up_to",
                                );
                            }
                        }
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
            HistoryMessage::SelectLogTab(index) => {
                if index < state.log_tabs.len() {
                    state.active_log_tab = index;
                }
            }
            HistoryMessage::CloseLogTab(index) => {
                if index < state.log_tabs.len() && state.log_tabs[index].is_closable {
                    state.log_tabs.remove(index);
                    if state.active_log_tab >= state.log_tabs.len() {
                        state.active_log_tab = state.log_tabs.len().saturating_sub(1);
                    }
                }
            }
            HistoryMessage::NewLogTab => {
                let id = state.next_log_tab_id;
                state.next_log_tab_id += 1;
                state.log_tabs.push(state::LogTab {
                    id,
                    label: format!("标签页 {}", id),
                    is_closable: true,
                    branch_filter: None,
                    text_filter: String::new(),
                    author_filter: None,
                    date_range: None,
                    path_filter: None,
                    scroll_offset: 0.0,
                    selected_commit: None,
                });
                state.active_log_tab = state.log_tabs.len() - 1;
            }
            HistoryMessage::OpenInNewTab(branch) => {
                let id = state.next_log_tab_id;
                state.next_log_tab_id += 1;
                state.log_tabs.push(state::LogTab::for_branch(id, branch));
                state.active_log_tab = state.log_tabs.len() - 1;
            }
            HistoryMessage::SetBranchFilter(branch) => {
                if let Some(tab) = state.log_tabs.get_mut(state.active_log_tab) {
                    tab.branch_filter = branch;
                }
            }
            HistoryMessage::SetAuthorFilter(author) => {
                if let Some(tab) = state.log_tabs.get_mut(state.active_log_tab) {
                    tab.author_filter = author;
                }
            }
            HistoryMessage::SetPathFilter(path) => {
                if let Some(tab) = state.log_tabs.get_mut(state.active_log_tab) {
                    tab.path_filter = path;
                }
            }
            HistoryMessage::ToggleBranchesDashboard => {
                state.log_branches_dashboard_visible = !state.log_branches_dashboard_visible;
            }
            HistoryMessage::DashboardSelectBranch(branch) => {
                // Filter log to this branch
                if let Some(tab) = state.log_tabs.get_mut(state.active_log_tab) {
                    tab.branch_filter = Some(branch);
                }
            }
            HistoryMessage::DashboardCheckoutBranch(name) => {
                return update(
                    state,
                    Message::BranchPopupMessage(
                        crate::views::branch_popup::BranchPopupMessage::CheckoutBranch(name),
                    ),
                );
            }
            HistoryMessage::DashboardMergeBranch(name) => {
                return update(
                    state,
                    Message::BranchPopupMessage(
                        crate::views::branch_popup::BranchPopupMessage::MergeBranch(name),
                    ),
                );
            }
            HistoryMessage::DashboardRebaseOnto(name) => {
                return update(
                    state,
                    Message::BranchPopupMessage(
                        crate::views::branch_popup::BranchPopupMessage::RebaseCurrentOnto(name),
                    ),
                );
            }
            HistoryMessage::DashboardDeleteBranch(name) => {
                return update(
                    state,
                    Message::BranchPopupMessage(
                        crate::views::branch_popup::BranchPopupMessage::PrepareDeleteBranch(name),
                    ),
                );
            }
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
            RemoteDialogMessage::Close => {
                state.remote_dialog.mode = views::remote_dialog::RemoteDialogMode::Overview;
                state.close_auxiliary_view();
            }
            RemoteDialogMessage::SwitchMode(mode) => {
                state.remote_dialog.mode = mode;
            }
            RemoteDialogMessage::SetTargetBranch(branch) => {
                state.remote_dialog.target_branch = branch;
            }
            RemoteDialogMessage::ToggleForcePush => {
                state.remote_dialog.force_push = !state.remote_dialog.force_push;
            }
            RemoteDialogMessage::TogglePushTags => {
                state.remote_dialog.push_tags = !state.remote_dialog.push_tags;
            }
            RemoteDialogMessage::ToggleSetUpstream => {
                state.remote_dialog.set_upstream = !state.remote_dialog.set_upstream;
            }
            RemoteDialogMessage::ExecutePush => {
                if let Ok(repo) = require_repository(state) {
                    let remote = state.remote_dialog.selected_remote.clone()
                        .or_else(|| state.remote_dialog.preferred_remote.clone())
                        .unwrap_or_else(|| "origin".to_string());
                    let branch = state.remote_dialog.current_branch_name.clone()
                        .unwrap_or_else(|| "main".to_string());

                    state.remote_dialog.is_loading = true;
                    state.remote_dialog.error = None;

                    let result = if state.remote_dialog.force_push {
                        git_core::force_push(&repo, &remote, &branch)
                    } else {
                        git_core::push(&repo, &remote, &branch, None)
                    };

                    state.remote_dialog.is_loading = false;
                    match result {
                        Ok(()) => {
                            state.remote_dialog.success_message = Some(format!(
                                "已推送 {} → {}/{}",
                                branch, remote, branch
                            ));
                            let _ = refresh_repository_after_action(state, &repo, false);
                        }
                        Err(e) => {
                            state.remote_dialog.error = Some(e.to_string());
                        }
                    }
                }
            }
            RemoteDialogMessage::SetPullBranch(branch) => {
                state.remote_dialog.pull_branch = branch;
            }
            RemoteDialogMessage::TogglePullRebase => {
                state.remote_dialog.pull_rebase = !state.remote_dialog.pull_rebase;
                if state.remote_dialog.pull_rebase {
                    state.remote_dialog.pull_ff_only = false;
                    state.remote_dialog.pull_no_ff = false;
                    state.remote_dialog.pull_squash = false;
                }
            }
            RemoteDialogMessage::TogglePullFfOnly => {
                state.remote_dialog.pull_ff_only = !state.remote_dialog.pull_ff_only;
                if state.remote_dialog.pull_ff_only {
                    state.remote_dialog.pull_rebase = false;
                    state.remote_dialog.pull_no_ff = false;
                    state.remote_dialog.pull_squash = false;
                }
            }
            RemoteDialogMessage::TogglePullNoFf => {
                state.remote_dialog.pull_no_ff = !state.remote_dialog.pull_no_ff;
                if state.remote_dialog.pull_no_ff {
                    state.remote_dialog.pull_rebase = false;
                    state.remote_dialog.pull_ff_only = false;
                    state.remote_dialog.pull_squash = false;
                }
            }
            RemoteDialogMessage::TogglePullSquash => {
                state.remote_dialog.pull_squash = !state.remote_dialog.pull_squash;
                if state.remote_dialog.pull_squash {
                    state.remote_dialog.pull_rebase = false;
                    state.remote_dialog.pull_ff_only = false;
                    state.remote_dialog.pull_no_ff = false;
                }
            }
            RemoteDialogMessage::ExecutePull => {
                if let Ok(repo) = require_repository(state) {
                    let remote = state.remote_dialog.selected_remote.clone()
                        .or_else(|| state.remote_dialog.preferred_remote.clone())
                        .unwrap_or_else(|| "origin".to_string());
                    let branch = state.remote_dialog.current_branch_name.clone()
                        .unwrap_or_else(|| "main".to_string());

                    state.remote_dialog.is_loading = true;
                    state.remote_dialog.error = None;

                    let result = git_core::pull(&repo, &remote, &branch, None);

                    state.remote_dialog.is_loading = false;
                    match result {
                        Ok(()) => {
                            state.remote_dialog.success_message = Some(format!(
                                "已拉取 {}/{} → {}",
                                remote, branch, branch
                            ));
                            let _ = refresh_repository_after_action(state, &repo, false);
                        }
                        Err(e) => {
                            state.remote_dialog.error = Some(e.to_string());
                        }
                    }
                }
            }
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
            TagDialogMessage::PushTag(name) => {
                if let Ok(repo) = require_repository(state) {
                    let remote = repo
                        .current_upstream_remote()
                        .unwrap_or_else(|| "origin".to_string());
                    match git_core::push_tag(&repo, &name, &remote) {
                        Ok(()) => {
                            state.tag_dialog.success_message =
                                Some(format!("标签 {name} 已推送到 {remote}"));
                        }
                        Err(e) => {
                            state.tag_dialog.error = Some(format!("推送标签失败: {e}"));
                        }
                    }
                }
            }
            TagDialogMessage::DeleteRemoteTag(name) => {
                if let Ok(repo) = require_repository(state) {
                    let remote = repo
                        .current_upstream_remote()
                        .unwrap_or_else(|| "origin".to_string());
                    match git_core::delete_remote_tag(&repo, &name, &remote) {
                        Ok(()) => {
                            state.tag_dialog.success_message =
                                Some(format!("远程标签 {name} 已从 {remote} 删除"));
                        }
                        Err(e) => {
                            state.tag_dialog.error = Some(format!("删除远程标签失败: {e}"));
                        }
                    }
                }
            }
            TagDialogMessage::SetForceTag(value) => state.tag_dialog.is_force = value,
            TagDialogMessage::ValidateCommitRef => {
                if let Ok(repo) = require_repository(state) {
                    match git_core::validate_commit_ref(&repo, &state.tag_dialog.target) {
                        Ok((hash, summary)) => {
                            state.tag_dialog.validation_result =
                                Some(format!("✓ {} — {}", &hash[..8.min(hash.len())], summary));
                            state.tag_dialog.error = None;
                        }
                        Err(_) => {
                            state.tag_dialog.validation_result =
                                Some("✗ 无效的提交引用".to_string());
                        }
                    }
                }
            }
            TagDialogMessage::DeleteLocalAndRemote(name) => {
                if let Ok(repo) = require_repository(state) {
                    // Delete local first
                    if let Err(e) = git_core::delete_tag(&repo, &name) {
                        state.tag_dialog.error = Some(format!("删除本地标签失败: {e}"));
                    } else {
                        let remote = repo
                            .current_upstream_remote()
                            .unwrap_or_else(|| "origin".to_string());
                        if let Err(e) = git_core::delete_remote_tag(&repo, &name, &remote) {
                            state.tag_dialog.error = Some(format!("删除远程标签失败: {e}"));
                        } else {
                            state.tag_dialog.success_message =
                                Some(format!("标签 {name} 已从本地和远程删除"));
                            if let Some(current) = state.current_repository.clone() {
                                state.tag_dialog.load_tags(&current);
                            }
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
            StashPanelMessage::PopStash(index) => {
                if let Ok(repo) = require_repository(state) {
                    state.stash_panel.apply_stash(&repo, index);
                    if state.stash_panel.error.is_none() {
                        let _ = refresh_repository_after_action(state, &repo, false);
                        if let Some(current) = state.current_repository.clone() {
                            state.stash_panel.load_stashes(&current);
                        }
                    }
                }
            }
            StashPanelMessage::ToggleIncludeUntracked => {
                state.stash_panel.include_untracked = !state.stash_panel.include_untracked;
            }
            StashPanelMessage::SetKeepIndex(value) => {
                state.stash_panel.keep_index = value;
            }
            StashPanelMessage::ShowUnstashDialog(index) => {
                state.stash_panel.show_unstash_dialog = Some(index);
                state.stash_panel.unstash_branch_name = format!("stash-branch-{}", index);
            }
            StashPanelMessage::SetUnstashBranchName(name) => {
                state.stash_panel.unstash_branch_name = name;
            }
            StashPanelMessage::ConfirmUnstashAsBranch => {
                if let Some(index) = state.stash_panel.show_unstash_dialog.take() {
                    let branch_name = state.stash_panel.unstash_branch_name.clone();
                    if branch_name.trim().is_empty() {
                        state.stash_panel.error = Some("分支名不能为空".to_string());
                    } else if let Ok(repo) = require_repository(state) {
                        match git_core::unstash_as_branch(&repo, index, &branch_name) {
                            Ok(()) => {
                                let _ = refresh_repository_after_action(state, &repo, false);
                                state.stash_panel.success_message =
                                    Some(format!("已应用到新分支 {branch_name}"));
                                if let Some(current) = state.current_repository.clone() {
                                    state.stash_panel.load_stashes(&current);
                                }
                            }
                            Err(e) => {
                                state.stash_panel.error =
                                    Some(format!("应用到新分支失败: {e}"));
                            }
                        }
                    }
                }
            }
            StashPanelMessage::CancelUnstashDialog => {
                state.stash_panel.show_unstash_dialog = None;
            }
            StashPanelMessage::ClearAllStashes => {
                if let Ok(repo) = require_repository(state) {
                    match git_core::stash_clear(&repo) {
                        Ok(()) => {
                            state.stash_panel.success_message = Some("所有储藏已清空".to_string());
                            state.stash_panel.stashes.clear();
                            state.stash_panel.selected_stash = None;
                        }
                        Err(e) => {
                            state.stash_panel.error = Some(format!("清空储藏失败: {e}"));
                        }
                    }
                }
            }
            StashPanelMessage::TogglePreview(index) => {
                if let Ok(repo) = require_repository(state) {
                    state.stash_panel.toggle_preview(&repo, index);
                }
            }
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
            RebaseEditorMessage::SetTodoAction(index, action) => {
                state.rebase_editor.set_todo_action(index, action);
            }
            RebaseEditorMessage::MoveTodoUp(index) => {
                state.rebase_editor.move_todo_up(index);
            }
            RebaseEditorMessage::MoveTodoDown(index) => {
                state.rebase_editor.move_todo_down(index);
            }
            RebaseEditorMessage::StartInlineEdit(index) => {
                state.rebase_editor.start_inline_edit(index);
            }
            RebaseEditorMessage::InlineEditChanged(text) => {
                state.rebase_editor.inline_edit_text = text;
            }
            RebaseEditorMessage::ConfirmInlineEdit => {
                state.rebase_editor.confirm_inline_edit();
            }
            RebaseEditorMessage::CancelInlineEdit => {
                state.rebase_editor.cancel_inline_edit();
            }
            RebaseEditorMessage::OpenTodoContextMenu(index) => {
                state.rebase_editor.context_menu_index = Some(index);
            }
            RebaseEditorMessage::CloseTodoContextMenu => {
                state.rebase_editor.context_menu_index = None;
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
        Some(AuxiliaryView::Remotes) => state.remote_dialog.load_remotes(&repo),
        Some(AuxiliaryView::Tags) => state.tag_dialog.load_tags(&repo),
        Some(AuxiliaryView::Stashes) => state.stash_panel.load_stashes(&repo),
        Some(AuxiliaryView::Rebase) => state.rebase_editor.load_status(&repo),
        Some(AuxiliaryView::Worktrees) => state.worktree_state.load_worktrees(&repo),
        Some(AuxiliaryView::Commit)
        | Some(AuxiliaryView::History)
        | None => {}
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
    require_repository(state)?;

    state.navigate_to(ShellSection::Changes);
    state.switch_git_tool_window_tab(GitToolWindowTab::Changes);
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

    state.commit_dialog.diff = diff;
    state.commit_dialog.staged_files = state.staged_changes.clone();
    state.commit_dialog.enable_amend_mode(commit);
    state.set_info(
        "已切换到 amend 模式",
        Some("你可以修改提交说明，或只保留部分暂存文件后更新最近一次提交。".to_string()),
        "workspace.commit",
    );
    Ok(())
}

fn switch_commit_dialog_to_new_commit_mode(state: &mut AppState) -> Result<(), String> {
    if state.commit_dialog.is_amend {
        state.commit_dialog.diff =
            build_staged_diff(&require_repository(state)?, &state.staged_changes)?;
        state.commit_dialog.staged_files = state.staged_changes.clone();
        state.commit_dialog.disable_amend_mode();
        state.set_info(
            "已切回普通提交模式",
            Some("当前提交面板会按暂存文件创建新的提交。".to_string()),
            "workspace.commit",
        );
        Ok(())
    } else {
        open_commit_dialog(state)
    }
}

fn toggle_commit_dialog_amend_mode(state: &mut AppState) -> Result<(), String> {
    if state.commit_dialog.is_amend {
        switch_commit_dialog_to_new_commit_mode(state)
    } else {
        switch_commit_dialog_to_amend(state)
    }
}

fn submit_commit_dialog(state: &mut AppState) -> Result<(), String> {
    let repo = require_repository(state)?;
    state.commit_dialog.start_commit();

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
            | BranchPopupMessage::ClearSearch
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
            | BranchPopupMessage::PrepareDeleteBranch(_)
            | BranchPopupMessage::ConfirmDeleteBranch
            | BranchPopupMessage::CancelDeleteBranch
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

fn navigate_hunk(state: &mut AppState, delta: isize) -> Task<Message> {
    if let Some(offset) = state.navigate_hunk(delta) {
        scroll_to(
            Id::new("diff-scroll"),
            AbsoluteOffset { x: 0.0, y: offset },
        )
    } else {
        Task::none()
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
    let bottom_tool_window = build_docked_tool_window(state);

    let main_window = MainWindow::new(
        i18n,
        state,
        body,
        bottom_tool_window,
        Message::ToggleProjectDropdown,
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
        Message::CloseAuxiliary,
        Message::SwitchGitToolWindowTab,
        Message::DismissFeedback,
        Message::DismissToast,
    );
    let main_view = main_window.view();

    // Overlay: project dropdown
    if state.show_project_dropdown {
        let mut project_list = Column::new().spacing(0).width(Length::Fill);

        // "Open project" button
        project_list = project_list.push(
            Button::new(
                Row::new()
                    .spacing(8)
                    .align_y(Alignment::Center)
                    .push(Text::new("📂").size(12))
                    .push(Text::new("打开项目...").size(12)),
            )
            .style(theme::button_style(theme::ButtonTone::Ghost))
            .padding([6, 12])
            .width(Length::Fill)
            .on_press(Message::OpenRepository),
        );

        project_list = project_list.push(
            iced::widget::rule::horizontal(1).style(theme::separator_rule_style()),
        );

        // Recent projects header
        if !state.project_history.is_empty() {
            project_list = project_list.push(
                Container::new(
                    Text::new("最近的项目")
                        .size(10)
                        .color(theme::darcula::TEXT_DISABLED),
                )
                .padding([6, 12]),
            );
        }

        // Project list
        for project in state.project_history.iter().take(10) {
            let is_active = state
                .active_project_path()
                .map(|p| p == project.path.as_path())
                .unwrap_or(false);
            let path = project.path.clone();
            let name_color = if is_active {
                theme::darcula::ACCENT
            } else {
                theme::darcula::TEXT_PRIMARY
            };

            project_list = project_list.push(
                Button::new(
                    Column::new()
                        .spacing(1)
                        .push(Text::new(&project.name).size(12).color(name_color))
                        .push(
                            Text::new(project.path.to_string_lossy().to_string())
                                .size(10)
                                .color(theme::darcula::TEXT_DISABLED),
                        ),
                )
                .style(theme::button_style(theme::ButtonTone::Ghost))
                .padding([4, 12])
                .width(Length::Fill)
                .on_press(Message::SwitchProject(path)),
            );
        }

        let dropdown = Container::new(
            widgets::scrollable::styled(project_list).height(Length::Shrink),
        )
        .width(Length::Fixed(320.0))
        .max_height(400.0)
        .style(|_| iced::widget::container::Style {
            background: Some(iced::Background::Color(theme::darcula::BG_PANEL)),
            border: iced::Border {
                width: 1.0,
                color: theme::darcula::BORDER,
                radius: 8.0.into(),
            },
            shadow: iced::Shadow {
                color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.4),
                offset: iced::Vector::new(0.0, 4.0),
                blur_radius: 16.0,
            },
            ..Default::default()
        });

        let overlay = Container::new(
            Column::new()
                .push(iced::widget::Space::new().height(Length::Fixed(46.0)))
                .push(
                    Row::new()
                        .push(iced::widget::Space::new().width(Length::Fixed(10.0)))
                        .push(dropdown),
                ),
        )
        .width(Length::Fill)
        .height(Length::Fill);

        let backdrop = iced::widget::mouse_area(
            Container::new(iced::widget::Space::new())
                .width(Length::Fill)
                .height(Length::Fill),
        )
        .on_press(Message::ToggleProjectDropdown);

        return iced::widget::stack([main_view, backdrop.into(), overlay.into()])
            .width(Length::Fill)
            .height(Length::Fill)
            .into();
    }

    // Overlay: IDEA-style floating branch dropdown
    if state.show_branch_dropdown {
        let dropdown = Container::new(
            branch_popup::view(&state.branch_popup).map(Message::BranchPopupMessage),
        )
        .width(Length::Fixed(620.0))
        .height(Length::Fixed(520.0))
        .style(|_| iced::widget::container::Style {
            background: Some(iced::Background::Color(theme::darcula::BG_PANEL)),
            border: iced::Border {
                width: 1.0,
                color: theme::darcula::BORDER,
                radius: 8.0.into(),
            },
            shadow: iced::Shadow {
                color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.4),
                offset: iced::Vector::new(0.0, 4.0),
                blur_radius: 16.0,
            },
            ..Default::default()
        });

        // Position the dropdown below the toolbar, left-aligned with some margin
        let overlay = Container::new(
            Column::new()
                .push(iced::widget::Space::new().height(Length::Fixed(46.0))) // toolbar height
                .push(
                    Row::new()
                        .push(iced::widget::Space::new().width(Length::Fixed(60.0))) // left offset
                        .push(dropdown),
                ),
        )
        .width(Length::Fill)
        .height(Length::Fill);

        // Clickable backdrop to close
        let backdrop = iced::widget::mouse_area(
            Container::new(iced::widget::Space::new())
                .width(Length::Fill)
                .height(Length::Fill),
        )
        .on_press(Message::ShowBranches); // toggle off

        return iced::widget::stack([
            main_view,
            backdrop.into(),
            overlay.into(),
        ])
        .width(Length::Fill)
        .height(Length::Fill)
        .into();
    }

    main_view
}

fn build_body<'a>(state: &'a AppState, i18n: &'a i18n::I18n) -> Element<'a, Message> {
    if state.current_repository.is_none() {
        return build_welcome_body(i18n);
    }

    if let Some(auxiliary) = state
        .auxiliary_view
        .filter(|view| !is_docked_auxiliary_view(*view))
    {
        return match auxiliary {
            AuxiliaryView::Branches => {
                branch_popup::view(&state.branch_popup).map(Message::BranchPopupMessage)
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
            AuxiliaryView::Worktrees => {
                views::worktree_view::view(&state.worktree_state)
                    .map(Message::WorktreeMessage)
            }
            AuxiliaryView::Commit => build_changes_body(state, i18n),
            AuxiliaryView::History => build_log_body(state),
        };
    }

    match state.shell.active_section {
        ShellSection::Changes => match state.shell.git_tool_window_tab {
            GitToolWindowTab::Changes => build_changes_body(state, i18n),
            GitToolWindowTab::Log => build_log_body(state),
        },
        ShellSection::Conflicts => build_conflict_body(state),
        ShellSection::Welcome => build_welcome_body(i18n),
    }
}

fn build_docked_tool_window<'a>(_state: &'a AppState) -> Option<Element<'a, Message>> {
    None
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

fn build_log_body<'a>(state: &'a AppState) -> Element<'a, Message> {
    history_view::view_with_tabs(
        &state.history_view,
        &state.log_tabs,
        state.active_log_tab,
        &state.branch_popup.local_branches,
        &state.branch_popup.remote_branches,
        state.log_branches_dashboard_visible,
    )
    .map(Message::HistoryMessage)
}

fn build_changes_body<'a>(state: &'a AppState, i18n: &'a i18n::I18n) -> Element<'a, Message> {
    let can_stage_all = !state.unstaged_changes.is_empty() || !state.untracked_files.is_empty();
    let can_unstage_all = !state.staged_changes.is_empty();
    let changes_content = Column::new()
        .spacing(theme::spacing::XS)
        .height(Length::Fill)
        .push(Container::new(build_change_sections(state, i18n)).height(Length::Fill));

    let display_mode_icon = match state.file_display_mode {
        state::FileDisplayMode::Flat => "≡",
        state::FileDisplayMode::Tree => "▤",
    };

    let changes_panel = Container::new(
        Column::new()
            .spacing(0)
            .push(
                Container::new(
                    Row::new()
                        .spacing(theme::spacing::XS)
                        .align_y(Alignment::Center)
                        .push(Text::new(i18n.changes).size(12))
                        .push(widgets::info_chip::<Message>(
                            state.workspace_change_count().to_string(),
                            BadgeTone::Neutral,
                        ))
                        .push(Space::new().width(Length::Fill))
                        .push(button::toolbar_icon(
                            display_mode_icon,
                            Some(Message::ToggleFileDisplayMode),
                        ))
                        .push(button::toolbar_icon("⟳", Some(Message::Refresh)))
                        .push(button::toolbar_icon(
                            "✓",
                            can_stage_all.then_some(Message::StageAll),
                        ))
                        .push(button::toolbar_icon(
                            "↶",
                            can_unstage_all.then_some(Message::UnstageAll),
                        )),
                )
                .padding(theme::density::SECONDARY_BAR_PADDING)
                .style(theme::frame_style(theme::Surface::Toolbar)),
            )
            .push(iced::widget::rule::horizontal(1))
            .push(
                Container::new(changes_content)
                    .padding([4, 4])
                    .height(Length::Fill),
            ),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .style(theme::panel_style(theme::Surface::Panel));

    let changes_stack = Container::new(
        stack([
            changes_panel.into(),
            build_change_context_menu_overlay(state),
        ])
        .width(Length::Fill)
        .height(Length::Fill),
    )
    .width(Length::FillPortion(5))
    .height(Length::Fill);

    let diff_panel = Container::new(
        Column::new()
            .spacing(0)
            .push(build_diff_header(state))
            .push(
                Container::new(build_diff_content(state, i18n))
                    .padding([0, 0])
                    .height(Length::Fill),
            ),
    )
    .height(Length::Fill)
    .style(theme::panel_style(theme::Surface::Panel));

    let commit_panel = commit_panel::view(&state.commit_dialog, &state.recent_commit_messages).map(Message::CommitDialogMessage);

    let right_panel = Column::new()
        .spacing(0)
        .height(Length::Fill)
        .push(diff_panel.height(Length::FillPortion(10)))
        .push(iced::widget::rule::horizontal(1))
        .push(
            Container::new(commit_panel)
                .height(Length::FillPortion(2)),
        );

    Row::new()
        .spacing(theme::spacing::XS)
        .height(Length::Fill)
        .push(changes_stack)
        .push(right_panel.width(Length::FillPortion(8)))
        .into()
}

fn build_change_sections<'a>(state: &'a AppState, i18n: &'a i18n::I18n) -> Element<'a, Message> {
    if state.workspace_change_count() == 0 {
        return Container::new(
            Column::new()
                .spacing(theme::spacing::XS)
                .align_x(Alignment::Center)
                .push(
                    Text::new(i18n.clean_workspace)
                        .size(13)
                        .color(theme::darcula::TEXT_SECONDARY),
                )
                .push(
                    Text::new(i18n.clean_workspace_detail)
                        .size(10)
                        .color(theme::darcula::TEXT_DISABLED),
                )
                .push(Space::new().height(Length::Fixed(theme::spacing::SM)))
                .push(
                    Row::new()
                        .spacing(theme::spacing::XS)
                        .push(button::secondary(i18n.refresh, Some(Message::Refresh)))
                        .push(button::ghost("分支", Some(Message::ShowBranches)))
                        .push(button::ghost("历史", Some(Message::ShowHistory))),
                ),
        )
        .width(Length::Fill)
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center)
        .into();
    }

    widgets::changelist::ChangesList::new(
        i18n,
        &state.staged_changes,
        &state.unstaged_changes,
        &state.untracked_files,
    )
    .with_selected_path(state.selected_change_path.as_deref())
    .with_display_mode(state.file_display_mode)
    .with_select_handler(Message::SelectChange)
    .with_stage_handler(Message::StageFile)
    .with_unstage_handler(Message::UnstageFile)
    .with_context_menu_handler(Message::OpenChangeContextMenu)
    .with_track_cursor_handler(Message::TrackChangeContextMenuCursor)
    .with_toggle_display_mode(Message::ToggleFileDisplayMode)
    .with_toggle_staged(Message::ToggleStagedCollapsed)
    .with_toggle_unstaged(Message::ToggleUnstagedCollapsed)
    .view()
}

const CHANGE_CONTEXT_MENU_WIDTH: f32 = 180.0;
const CHANGE_CONTEXT_MENU_ESTIMATED_HEIGHT: f32 = 180.0;
const CHANGE_CONTEXT_MENU_EDGE_PADDING: f32 = 8.0;

fn build_change_context_menu_overlay<'a>(state: &'a AppState) -> Element<'a, Message> {
    let Some(path) = state.change_context_menu_path.as_deref() else {
        return Space::new().width(Length::Shrink).into();
    };
    let anchor = state
        .change_context_menu_anchor
        .unwrap_or(state.change_context_menu_cursor);

    let is_staged = state.staged_changes.iter().any(|c| c.path == path);
    let _is_unstaged = state.unstaged_changes.iter().any(|c| c.path == path);

    let stage_label = if is_staged { "取消暂存" } else { "暂存" };
    let stage_message = if is_staged {
        Some(Message::UnstageFile(path.to_string()))
    } else {
        Some(Message::StageFile(path.to_string()))
    };

    let show_diff_enabled = state.selected_change_path.as_deref() != Some(path);

    // IDEA-style compact file context menu — no subtitles, flat list
    let actions = Column::new()
        .spacing(0)
        .push(change_context_action_row(
            stage_label,
            String::new(),
            stage_message,
            widgets::menu::MenuTone::Neutral,
        ))
        .push(change_context_action_row(
            "查看差异",
            String::new(),
            show_diff_enabled.then_some(Message::SelectChange(path.to_string())),
            widgets::menu::MenuTone::Neutral,
        ))
        .push(change_context_action_row(
            "放弃更改",
            String::new(),
            Some(Message::RevertFile(path.to_string())),
            widgets::menu::MenuTone::Danger,
        ))
        .push(change_context_action_row(
            "显示历史",
            String::new(),
            Some(Message::ShowFileHistory(path.to_string())),
            widgets::menu::MenuTone::Neutral,
        ))
        .push(change_context_action_row(
            "复制路径",
            String::new(),
            Some(Message::CopyChangePath(path.to_string())),
            widgets::menu::MenuTone::Neutral,
        ))
        .push(change_context_action_row(
            "在编辑器中打开",
            String::new(),
            Some(Message::OpenInEditor(path.to_string())),
            widgets::menu::MenuTone::Neutral,
        ));

    let menu = Container::new(actions)
        .padding([4, 6])
        .width(Length::Fixed(CHANGE_CONTEXT_MENU_WIDTH))
        .style(widgets::menu::panel_style);

    build_change_context_menu_layer(anchor, menu.into())
}

fn change_context_action_row<'a>(
    title: &'static str,
    detail: String,
    message: Option<Message>,
    tone: widgets::menu::MenuTone,
) -> Element<'a, Message> {
    widgets::menu::action_row(None, title, Some(detail), None, message, tone)
}

fn build_change_context_menu_layer<'a>(
    anchor: Point,
    menu: Element<'a, Message>,
) -> Element<'a, Message> {
    let origin = change_context_menu_origin(anchor);

    opaque(
        mouse_area(
            Container::new(
                Column::new()
                    .push(Space::new().height(Length::Fixed(origin.y)))
                    .push(
                        Row::new()
                            .width(Length::Fill)
                            .push(Space::new().width(Length::Fixed(origin.x)))
                            .push(menu)
                            .push(Space::new().width(Length::Fill)),
                    )
                    .push(Space::new().height(Length::Fill)),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .style(widgets::menu::scrim_style),
        )
        .on_press(Message::CloseChangeContextMenu),
    )
}

fn change_context_menu_origin(anchor: Point) -> Point {
    let x = if anchor.x > CHANGE_CONTEXT_MENU_WIDTH * 0.68 {
        (anchor.x - CHANGE_CONTEXT_MENU_WIDTH + 28.0).max(CHANGE_CONTEXT_MENU_EDGE_PADDING)
    } else {
        (anchor.x + 6.0).max(CHANGE_CONTEXT_MENU_EDGE_PADDING)
    };
    let y = if anchor.y > CHANGE_CONTEXT_MENU_ESTIMATED_HEIGHT * 0.52 {
        (anchor.y - CHANGE_CONTEXT_MENU_ESTIMATED_HEIGHT + 18.0)
            .max(CHANGE_CONTEXT_MENU_EDGE_PADDING)
    } else {
        (anchor.y + 6.0).max(CHANGE_CONTEXT_MENU_EDGE_PADDING)
    };

    Point::new(x, y)
}

fn build_diff_header<'a>(state: &'a AppState) -> Element<'a, Message> {
    let file_name = state
        .selected_change_path
        .as_ref()
        .and_then(|path| std::path::Path::new(path).file_name()?.to_str())
        .unwrap_or("差异");

    let path_hint = state.selected_change_path.as_ref().and_then(|path| {
        std::path::Path::new(path)
            .parent()
            .and_then(|p| p.to_str())
            .filter(|p| !p.is_empty())
    });

    let summary = state
        .current_diff
        .as_ref()
        .and_then(|diff| diff.files.first().map(|f| (f.additions, f.deletions)));

    let file_position = state.current_diff.as_ref().and_then(|diff| {
        (diff.files.len() > 1).then(|| {
            state.selected_change_path.as_ref().and_then(|selected| {
                diff.files
                    .iter()
                    .position(|f| f.new_path.as_deref().or(f.old_path.as_deref()) == Some(selected))
                    .map(|idx| (idx + 1, diff.files.len()))
            })
        })?
    });

    Container::new(
        Row::new()
            .spacing(theme::spacing::XS)
            .align_y(Alignment::Center)
            .push(
                Column::new()
                    .spacing(1)
                    .width(Length::Fill)
                    .push(Text::new(file_name).size(11))
                    .push_maybe(path_hint.map(|hint| {
                        Text::new(hint)
                            .size(9)
                            .color(theme::darcula::TEXT_SECONDARY)
                    })),
            )
            .push_maybe(summary.map(|(add, del)| {
                widgets::compact_chip::<Message>(format!("+{add} / -{del}"), BadgeTone::Neutral)
            }))
            .push_maybe(file_position.map(|(cur, tot)| {
                widgets::compact_chip::<Message>(format!("{cur} / {tot}"), BadgeTone::Accent)
            }))
            .push(Space::new().width(Length::Fill))
            .push(button::tab(
                "统一",
                state.diff_presentation == DiffPresentation::Unified,
                state.current_diff.is_some().then_some(Message::ToggleDiffPresentation),
            ))
            .push(button::tab(
                "分栏",
                state.diff_presentation == DiffPresentation::Split,
                state.current_diff.is_some().then_some(Message::ToggleDiffPresentation),
            ))
            .push_maybe(state.current_diff.as_ref().and_then(|diff| {
                let total_hunks: usize = diff.files.iter().map(|f| f.hunks.len()).sum();
                (total_hunks > 1).then(|| {
                    let nav: Element<'_, Message> = Row::new()
                        .spacing(theme::spacing::XS)
                        .push(button::compact_ghost("▲", Some(Message::PrevHunk)))
                        .push(button::compact_ghost("▼", Some(Message::NextHunk)))
                        .into();
                    nav
                })
            })),
    )
    .padding(theme::density::SECONDARY_BAR_PADDING)
    .style(theme::frame_style(theme::Surface::Toolbar))
    .into()
}

fn build_diff_content<'a>(state: &'a AppState, i18n: &'a i18n::I18n) -> Element<'a, Message> {
    // Check full file preview first (for new/untracked/binary files)
    if state.full_file_preview_binary {
        return widgets::panel_empty_state_compact(
            i18n.binary_file_no_preview,
            state.selected_change_path.as_deref().unwrap_or(""),
        );
    }

    if let Some(preview_diff) = &state.full_file_preview {
        return widgets::diff_viewer::file_preview(preview_diff);
    }

    if !state.show_diff || state.current_diff.is_none() {
        return widgets::panel_empty_state_compact(
            i18n.diff_empty,
            i18n.diff_empty_detail,
        );
    }

    let diff = state.current_diff.as_ref().expect("diff checked");
    if diff.files.is_empty() {
        return widgets::panel_empty_state_compact(
            i18n.no_changes,
            i18n.diff_empty_detail,
        );
    }

    // Determine if selected file is staged or unstaged for hunk action buttons
    let selected_is_staged = state
        .selected_change_path
        .as_ref()
        .map(|p| state.staged_changes.iter().any(|c| &c.path == p))
        .unwrap_or(false);

    match state.diff_presentation {
        DiffPresentation::Unified => {
            let mut viewer = widgets::diff_viewer::DiffViewer::new(diff);
            if selected_is_staged {
                viewer = viewer
                    .with_unstage_hunk_handler(Message::UnstageHunk);
            } else {
                viewer = viewer
                    .with_stage_hunk_handler(Message::StageHunk);
            }
            viewer.view()
        }
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
                "先决定整文件取舍，需要精细处理时进入三栏合并。",
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
    CloseAuxiliary,
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
    SwitchGitToolWindowTab(GitToolWindowTab),
    SwitchProject(PathBuf),
    SelectChange(String),
    ToggleDiffPresentation,
    NavigatePrevFile,
    NavigateNextFile,
    PrevHunk,
    NextHunk,
    TrackChangeContextMenuCursor(Point),
    OpenChangeContextMenu(String),
    CloseChangeContextMenu,
    RevertFile(String),
    CopyChangePath(String),
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
    ToggleProjectDropdown,
    ToggleFileDisplayMode,
    ToggleStagedCollapsed,
    ToggleUnstagedCollapsed,
    StageHunk(String, usize),
    UnstageHunk(String, usize),
    ShowFileHistory(String),
    OpenInEditor(String),
    ShowWorktrees,
    WorktreeMessage(views::worktree_view::WorktreeMessage),
    ToggleBlameAnnotation,
    CancelNetworkOperation,
    TogglePullStrategy,
    ForcePushCurrent,
    SetUpstreamAndPush { branch: String, remote: String },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::ViewMode;

    #[test]
    fn open_commit_dialog_navigates_to_changes_tab() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let repo = git_core::Repository::init(temp_dir.path())
            .expect("repository should initialize");
        let mut state = AppState::new();
        state.set_repository(repo);
        state.switch_git_tool_window_tab(GitToolWindowTab::Log);
        open_commit_dialog(&mut state).expect("should open commit dialog");
        assert_eq!(state.shell.git_tool_window_tab, GitToolWindowTab::Changes);
        assert_eq!(state.view_mode, ViewMode::Repository);
    }
}
