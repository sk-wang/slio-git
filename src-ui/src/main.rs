//! slio-git UI - Pure Iced desktop application.

mod file_watcher;
mod i18n;
mod keyboard;
mod logging;
mod state;
mod theme;

pub mod components;
pub mod views;
pub mod widgets;

use crate::components::status_icons::FileStatus;
use crate::file_watcher::RepositoryWatchEvent;
use crate::i18n::I18n;
use crate::keyboard::{get_shortcuts, ShortcutAction};
use crate::state::{
    is_docked_auxiliary_view, AppState, AuxiliaryView, DiffPresentation, GitToolWindowTab,
    ShellSection, ToolbarRemoteAction,
};
use crate::theme::BadgeTone;
use crate::views::main_window::MainWindow;
use crate::views::{
    branch_popup::{self, BranchPopupMessage, PendingCommitAction},
    commit_dialog::CommitDialogMessage,
    history_view::{self, HistoryMessage},
    rebase_editor::{self, RebaseEditorMessage},
    remote_dialog::{self, RemoteDialogMessage},
    stash_panel::{self, StashPanelMessage},
    tag_dialog::{self, TagDialogMessage},
};
use crate::widgets::conflict_resolver::{ConflictResolverMessage, ResolutionOption};
use crate::widgets::diff_editor::DiffEditorEvent;
use crate::widgets::{button, commit_panel, file_picker, scrollable, OptionalPush};
use git_core::index::Change;
use git_core::{
    diff::{ConflictHunk, ConflictHunkType, ConflictLineType, ConflictResolution, ThreeWayDiff},
    Repository,
};
use iced::widget::operation::{scroll_to, AbsoluteOffset};
use iced::widget::Id;
use iced::widget::{
    container, mouse_area, opaque, stack, text, Button, Column, Container, Row, Space, Text,
};
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

    iced::application(
        || {
            let startup_task = Task::perform(
                git_core::updater::check_for_update(env!("CARGO_PKG_VERSION").to_string()),
                |result| Message::UpdateCheckResult(result.ok().flatten()),
            );
            let init_i18n = i18n::locale(None);
            (AppState::restore(init_i18n), startup_task)
        },
        update,
        view,
    )
    .title("slio-git")
    .default_font(theme::app_font())
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

    if let Some(repo_path) = state.active_project_path().map(Path::to_path_buf) {
        subscriptions
            .push(file_watcher::subscription(repo_path).map(Message::RepositoryWatchEvent));
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
            | Message::RepositoryWatchEvent(_)
            | Message::AutoRemoteCheckFinished(_)
            | Message::ToastTick(_)
            | Message::ToggleToolbarRemoteMenu(_)
            | Message::CloseToolbarRemoteMenu
            | Message::DismissToast
            | Message::ToolbarRemoteActionSelected { .. }
    ) {
        state.close_toolbar_remote_menu();
    }

    let i18n = i18n::locale(state.git_settings.language.as_deref());

    match message {
        Message::OpenRepository => {
            state.show_project_dropdown = false;
            state.set_loading(
                i18n.opening_repo,
                Some(i18n.opening_repo_detail.to_string()),
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
                        state.set_repository(repo, i18n);
                    }
                    Err(error) => {
                        logging::LogManager::log_repo_operation(
                            "open",
                            &path.display().to_string(),
                            false,
                        );
                        report_async_failure(
                            state,
                            i18n.cannot_open_repo,
                            i18n.cannot_open_repo_fmt.replace("{}", &error.to_string()),
                            "repository.open",
                            "repository.open",
                        i18n,
                        );
                    }
                }
            } else {
                state.set_empty(
                    i18n.no_repo_selected,
                    Some(i18n.no_repo_selected_detail.to_string()),
                    "repository.open",
                );
            }
        }
        Message::InitRepository => {
            state.set_loading(
                i18n.initializing_repo,
                Some(i18n.initializing_repo_detail.to_string()),
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
                        state.set_repository(repo, i18n);
                    }
                    Err(error) => {
                        logging::LogManager::log_repo_operation(
                            "init",
                            &path.display().to_string(),
                            false,
                        );
                        report_async_failure(
                            state,
                            i18n.cannot_init_repo,
                            i18n.cannot_init_repo_fmt.replace("{}", &error.to_string()),
                            "repository.init",
                            "repository.init",
                        i18n,
                        );
                    }
                }
            } else {
                state.set_empty(
                    i18n.no_init_dir_selected,
                    Some(i18n.no_init_dir_selected_detail.to_string()),
                    "repository.init",
                );
            }
        }
        Message::Refresh => {
            let previous_section = state.shell.active_section;
            state.set_loading(
                i18n.refreshing_workspace,
                Some(i18n.refreshing_workspace_detail.to_string()),
                "repository.refresh",
            );

            if state.current_repository.is_some() {
                match state.refresh_current_repository(false, i18n) {
                    Ok(()) => {
                        if previous_section != ShellSection::Conflicts || !state.has_conflicts() {
                            state.navigate_to(previous_section, i18n);
                        }
                        refresh_workspace_views(state);
                        state.set_success(i18n.repo_refreshed, None, "repository.refresh");
                    }
                    Err(error) => report_async_failure(
                        state,
                        i18n.refresh_failed,
                        i18n.refresh_failed_fmt.replace("{}", &error.to_string()),
                        "repository.refresh",
                        "repository.refresh",
                        i18n,
                    ),
                }
            } else {
                state.set_warning(
                    i18n.no_repo_open,
                    Some(i18n.no_repo_open_detail.to_string()),
                    "repository.refresh",
                );
            }
        }
        Message::AutoRefreshTick(now) => {
            let should_refresh_workspace = state.should_auto_refresh_workspace(now);
            if should_refresh_workspace {
                if let Err(error) = state.refresh_current_repository(false, i18n) {
                    warn!("Auto refresh workspace failed: {}", error);
                } else {
                    refresh_workspace_views(state);
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
                        if let Err(error) = state.refresh_current_repository(false, i18n) {
                            warn!(
                                "Auto refresh after remote fetch failed for {}: {}",
                                result.repo_path.display(),
                                error
                            );
                        } else {
                            refresh_workspace_views(state);
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
        Message::RepositoryWatchEvent(event) => {
            if state
                .active_project_path()
                .is_some_and(|path| path == event.repo_path.as_path())
            {
                state.mark_workspace_refresh_pending();

                if !state.auto_refresh_suspended() {
                    if let Err(error) = state.refresh_current_repository(false, i18n) {
                        warn!(
                            "Repository watcher refresh failed for {}: {}",
                            event.repo_path.display(),
                            error
                        );
                    } else {
                        refresh_workspace_views(state);
                    }
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
            state.clear_repository(i18n);
            state.set_empty(
                i18n.repo_closed,
                Some(i18n.repo_closed_detail.to_string()),
                "repository.close",
            );
        }
        Message::ShowChanges => {
            let previous_section = state.shell.active_section;
            state.close_auxiliary_view(i18n);
            state.navigate_to(ShellSection::Changes, i18n);
            log_shell_navigation(
                previous_section,
                state.shell.active_section,
                "switch from shell rail",
            );
        }
        Message::ShowConflicts => {
            let previous_section = state.shell.active_section;
            state.close_auxiliary_view(i18n);
            if let Err(error) = state.open_conflict_resolver(i18n) {
                report_async_failure(
                    state,
                    i18n.cannot_open_conflict_view,
                    error,
                    "workspace.conflicts",
                    "workspace.conflicts",
                i18n,
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
            if let Err(error) = state.stage_file(path, i18n) {
                report_async_failure(
                    state,
                    i18n.stage_file_failed,
                    error,
                    "workspace.stage_file",
                    "workspace.stage_file",
                i18n,
                );
            }
        }
        Message::UnstageFile(path) => {
            if state.selected_change_path.as_deref() != Some(&path) {
                let _ = state.select_change(path.clone());
            }
            if let Err(error) = state.unstage_file(path, i18n) {
                report_async_failure(
                    state,
                    i18n.unstage_file_failed,
                    error,
                    "workspace.unstage_file",
                    "workspace.unstage_file",
                i18n,
                );
            }
        }
        Message::StageAll => {
            if let Err(error) = state.stage_all(i18n) {
                report_async_failure(
                    state,
                    i18n.stage_all_failed,
                    error,
                    "workspace.stage_all",
                    "workspace.stage_all",
                i18n,
                );
            }
        }
        Message::UnstageAll => {
            if let Err(error) = state.unstage_all(i18n) {
                report_async_failure(
                    state,
                    i18n.unstage_all_failed,
                    error,
                    "workspace.unstage_all",
                    "workspace.unstage_all",
                i18n,
                );
            }
        }
        Message::SelectChange(path) => {
            if let Err(error) = state.select_change(path) {
                report_async_failure(
                    state,
                    i18n.load_file_diff_failed,
                    error,
                    "workspace.select_change",
                    "workspace.select_change",
                i18n,
                );
            }
            update_editor_diff_model(state);
        }
        Message::ToggleDiffPresentation => {
            if let Some(popup) = state.history_commit_diff_popup.as_mut() {
                if !popup.supports_split_diff() {
                    return iced::Task::none();
                }
                popup.diff_presentation = match popup.diff_presentation {
                    DiffPresentation::Unified => DiffPresentation::Split,
                    DiffPresentation::Split => DiffPresentation::Unified,
                };
                return iced::Task::none();
            }
            if matches!(state.diff_source, state::DiffSource::HistoryCommit { .. }) {
                return iced::Task::none();
            }
            state.toggle_diff_presentation();
            update_editor_diff_model(state);
        }
        Message::UnifiedDiffEditorEvent(event) => {
            if let Some(popup) = state.history_commit_diff_popup.as_mut() {
                if let Some(editor) = popup.unified_diff_editor.as_mut() {
                    let task = editor.update(match event {
                        widgets::diff_editor::UnifiedDiffEditorEvent::Editor(m) => m,
                    });
                    return task.map(Message::UnifiedDiffEditorEvent);
                }
                return Task::none();
            }
            if let Some(editor) = state.unified_diff_editor.as_mut() {
                let task = editor.update(match event {
                    widgets::diff_editor::UnifiedDiffEditorEvent::Editor(m) => m,
                });
                return task.map(Message::UnifiedDiffEditorEvent);
            }
        }
        Message::SplitDiffEditorEvent(event) => {
            if let Some(popup) = state.history_commit_diff_popup.as_mut() {
                if let Some(editor) = popup.split_diff_editor.as_mut() {
                    let (task, current_hunk) = editor.update(event);
                    if let Some(hunk_index) = current_hunk {
                        if popup.selected_hunk_index != Some(hunk_index) {
                            popup.selected_hunk_index = Some(hunk_index);
                        }
                    }
                    return task.map(Message::SplitDiffEditorEvent);
                }
                return Task::none();
            }
            if let Some(editor) = state.split_diff_editor.as_mut() {
                let (task, current_hunk) = editor.update(event);
                if let Some(hunk_index) = current_hunk {
                    if state.selected_hunk_index != Some(hunk_index) {
                        state.selected_hunk_index = Some(hunk_index);
                    }
                }
                return task.map(Message::SplitDiffEditorEvent);
            }
        }
        Message::ShowSettings => {
            state.open_auxiliary_view(state::AuxiliaryView::Settings, i18n);
        }
        Message::SettingsMessage(msg) => {
            use views::settings_view::SettingsMessage;
            match &msg {
                SettingsMessage::Close => state.close_auxiliary_view(i18n),
                SettingsMessage::SaveAndClose => {
                    state.git_settings.apply_message(&msg);
                    if let Err(e) = state.git_settings.save() {
                        log::warn!("Failed to persist git settings: {}", e);
                    }
                    state.close_auxiliary_view(i18n);
                    state.set_success(i18n.settings_saved, None, "settings.save");
                }
                other => state.git_settings.apply_message(other),
            }
        }
        Message::MergeEditorMessage(event) => {
            use widgets::merge_editor::MergeEditorEvent;
            match &event {
                MergeEditorEvent::BackToList => {
                    state.close_conflict_merge();
                    state.merge_editor = None;
                }
                MergeEditorEvent::Apply => {
                    if let Some(editor) = &state.merge_editor {
                        if editor.all_resolved() {
                            let resolved_text = editor.resolved_text();
                            if let Some(repo) = &state.current_repository {
                                if let Some(conflict_index) = state.conflict_merge_index {
                                    if let Some(conflict) = state.conflict_files.get(conflict_index)
                                    {
                                        let path = std::path::Path::new(&conflict.path);
                                        match git_core::diff::resolve_conflict(
                                            repo,
                                            path,
                                            git_core::diff::ConflictResolution::Custom(
                                                resolved_text,
                                            ),
                                        ) {
                                            Ok(()) => {
                                                state.set_success(
                                                    i18n.conflict_resolved,
                                                    Some(conflict.path.clone()),
                                                    "merge_editor.apply",
                                                );
                                                state.close_conflict_merge();
                                                state.merge_editor = None;
                                                let _ = state.refresh_current_repository(true, i18n);
                                            }
                                            Err(e) => {
                                                state.set_error(i18n.write_conflict_result_failed_fmt.replace("{}", &e.to_string()));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                _ => {
                    if let Some(editor) = &mut state.merge_editor {
                        let _task = editor.update(event);
                    }
                }
            }
        }
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
                        i18n.stage_hunk_failed,
                        e.to_string(),
                        "workspace.stage_hunk",
                        "workspace.stage_hunk",
                    i18n,
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
                        i18n.unstage_hunk_failed,
                        e.to_string(),
                        "workspace.unstage_hunk",
                        "workspace.unstage_hunk",
                    i18n,
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
                let i18n = i18n::locale(state.git_settings.language.as_deref());
                state.history_view.load_history(&repo, i18n);
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
                        label: i18n.force_push_label_fmt.replace("{}", &branch).replacen("{}", &remote, 1),
                        progress: None,
                        status: Some("--force-with-lease".to_string()),
                    });
                    match git_core::force_push(repo, &remote, &branch) {
                        Ok(()) => {
                            state.network_operation = None;
                            state.set_success(
                                i18n.force_push_success,
                                Some(format!("{branch} → {remote}")),
                                "workspace.push.force",
                            );
                        }
                        Err(e) => {
                            state.network_operation = None;
                            report_async_failure(
                                state,
                                i18n.force_push_failed,
                                e.to_string(),
                                "workspace.push.force",
                                "workspace.push.force",
                            i18n,
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
                        i18n.set_upstream_failed,
                        e.to_string(),
                        "workspace.push.upstream",
                        "workspace.push.upstream",
                    i18n,
                    );
                } else {
                    return update(state, Message::Push);
                }
            }
        }
        Message::UpdateCheckResult(update_info) => {
            state.available_update = update_info;
        }
        Message::OpenUpdateUrl => {
            if let Some(ref update) = state.available_update {
                let url = update
                    .download_url
                    .as_deref()
                    .unwrap_or(&update.release_url);
                let _ = open::that(url);
            }
        }
        Message::DismissUpdate => {
            state.available_update = None;
        }
        Message::ShowWorktrees => {
            if let Ok(repo) = require_repository(state) {
                state.worktree_state.load_worktrees(&repo);
                state.open_auxiliary_view(state::AuxiliaryView::Worktrees, i18n);
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
                WorktreeMessage::Close => state.close_auxiliary_view(i18n),
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
            if let Err(error) = state.switch_to_project(&path, i18n) {
                report_async_failure(
                    state,
                    i18n.switch_project_failed,
                    error,
                    "workspace.project-switch",
                    "workspace.project-switch",
                i18n,
                );
            }
        }
        Message::NavigatePrevFile => select_relative_file(state, -1),
        Message::NavigateNextFile => select_relative_file(state, 1),
        Message::PrevHunk => {
            if state.history_commit_diff_popup.is_some() {
                return navigate_history_commit_diff_popup_hunk(state, -1);
            }
            return navigate_hunk(state, -1);
        }
        Message::NextHunk => {
            if state.history_commit_diff_popup.is_some() {
                return navigate_history_commit_diff_popup_hunk(state, 1);
            }
            return navigate_hunk(state, 1);
        }
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
                if let Err(error) = git_core::index::discard_file(repo, std::path::Path::new(&path))
                {
                    report_async_failure(
                        state,
                        i18n.revert_file_failed,
                        error.to_string(),
                        "workspace.revert",
                        "workspace.revert",
                    i18n,
                    );
                } else {
                    state.refresh_changes();
                    state.close_change_context_menu();
                    state.set_success(i18n.file_reverted, Some(path), "workspace.revert");
                }
            }
        }
        Message::CopyChangePath(path) => {
            if let Err(error) = copy_text_to_clipboard(&path) {
                report_async_failure(
                    state,
                    i18n.copy_path_failed,
                    error,
                    "workspace.copy_path",
                    "workspace.copy_path",
                i18n,
                );
            } else {
                state.set_success(i18n.path_copied, Some(path), "workspace.copy_path");
                state.close_change_context_menu();
            }
        }
        Message::KeyboardShortcut(action) => match action {
            ShortcutAction::StageFile => {
                if let Some(path) = state.selected_change_path.clone() {
                    let can_stage = state.unstaged_changes.iter().any(|c| c.path == path)
                        || state.untracked_files.iter().any(|c| c.path == path);
                    if can_stage {
                        if let Err(error) = state.stage_file(path, i18n) {
                            report_async_failure(
                                state,
                                i18n.stage_file_failed,
                                error,
                                "keyboard.stage",
                                "keyboard.stage",
                            i18n,
                            );
                        }
                    } else {
                        state.set_info(i18n.file_already_staged, None, "keyboard.stage");
                    }
                } else {
                    state.set_info(i18n.select_file_first, None, "keyboard.stage");
                }
            }
            ShortcutAction::UnstageFile => {
                if let Some(path) = state.selected_change_path.clone() {
                    let can_unstage = state.staged_changes.iter().any(|c| c.path == path);
                    if can_unstage {
                        if let Err(error) = state.unstage_file(path, i18n) {
                            report_async_failure(
                                state,
                                i18n.unstage_file_failed,
                                error,
                                "keyboard.unstage",
                                "keyboard.unstage",
                            i18n,
                            );
                        }
                    } else {
                        state.set_info(i18n.file_not_staged, None, "keyboard.unstage");
                    }
                } else {
                    state.set_info(i18n.select_file_first, None, "keyboard.unstage");
                }
            }
            ShortcutAction::StageAll => {
                if let Err(error) = state.stage_all(i18n) {
                    report_async_failure(
                        state,
                        i18n.stage_all_failed,
                        error,
                        "workspace.stage_all",
                        "workspace.stage_all",
                    i18n,
                    );
                }
            }
            ShortcutAction::UnstageAll => {
                if let Err(error) = state.unstage_all(i18n) {
                    report_async_failure(
                        state,
                        i18n.unstage_all_failed,
                        error,
                        "workspace.unstage_all",
                        "workspace.unstage_all",
                    i18n,
                    );
                }
            }
            ShortcutAction::Refresh => return update(state, Message::Refresh),
            ShortcutAction::OpenCommitDialog => {
                if let Err(error) = open_commit_dialog(state) {
                    report_async_failure(
                        state,
                        i18n.cannot_open_commit_panel,
                        error,
                        "workspace.commit",
                        "workspace.commit",
                    i18n,
                    );
                }
            }
            ShortcutAction::ToggleAmendCommitMode => {
                if let Err(error) = toggle_commit_dialog_amend_mode(state) {
                    report_async_failure(
                        state,
                        i18n.cannot_toggle_amend,
                        error,
                        "workspace.commit",
                        "workspace.commit.amend",
                    i18n,
                    );
                }
            }
            ShortcutAction::OpenPushDialog => {
                if let Err(error) = open_remote_dialog(state) {
                    report_async_failure(
                        state,
                        i18n.cannot_open_remote_panel,
                        error,
                        "workspace.push",
                        "workspace.push",
                    i18n,
                    );
                } else {
                    state.set_info(i18n.remote_panel_opened, None, "keyboard.shortcut");
                }
            }
            ShortcutAction::ShowFileDiff => {
                if let Some(path) = state.selected_change_path.clone() {
                    if let Err(error) = state.load_diff_for_file(&path) {
                        report_async_failure(
                            state,
                            i18n.load_diff_failed,
                            error,
                            "keyboard.diff",
                            "keyboard.diff",
                        i18n,
                        );
                    }
                } else {
                    state.set_info(i18n.select_file_for_diff, None, "keyboard.diff");
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
                        i18n.commit_failed,
                        error,
                        "workspace.commit",
                        "workspace.commit",
                    i18n,
                    );
                }
            }
            ShortcutAction::SwitchToLogTab => {
                return update(state, Message::SwitchGitToolWindowTab(GitToolWindowTab::Log));
            }
            ShortcutAction::SwitchToChangesTab => {
                return update(
                    state,
                    Message::SwitchGitToolWindowTab(GitToolWindowTab::Changes),
                );
            }
            _ => {}
        },
        Message::Commit => {
            if let Err(error) = open_commit_dialog(state) {
                report_async_failure(
                    state,
                    i18n.cannot_open_commit_panel,
                    error,
                    "workspace.commit",
                    "workspace.commit",
                i18n,
                );
            }
        }
        Message::Pull => {
            if let Err(error) = open_remote_dialog(state) {
                report_async_failure(
                    state,
                    i18n.cannot_open_remote_panel,
                    error,
                    "workspace.pull",
                    "workspace.pull",
                i18n,
                );
            } else {
                // Switch to IDEA-style Pull dialog
                state.remote_dialog.mode = views::remote_dialog::RemoteDialogMode::Pull;
                state.set_info(
                    i18n.pull_panel_opened,
                    state
                        .current_repository
                        .as_ref()
                        .map(|repo| remote_panel_hint(repo, i18n.pull, i18n)),
                    "workspace.pull",
                );
            }
        }
        Message::Push => {
            if let Err(error) = open_remote_dialog(state) {
                report_async_failure(
                    state,
                    i18n.cannot_open_remote_panel,
                    error,
                    "workspace.push",
                    "workspace.push",
                i18n,
                );
            } else {
                // Switch to IDEA-style Push dialog
                state.remote_dialog.mode = views::remote_dialog::RemoteDialogMode::Push;
                state.set_info(
                    i18n.push_panel_opened,
                    state
                        .current_repository
                        .as_ref()
                        .map(|repo| remote_panel_hint(repo, i18n.push, i18n)),
                    "workspace.push",
                );
            }
        }
        Message::ToggleToolbarRemoteMenu(action) => {
            if let Err(error) = state.toggle_toolbar_remote_menu(action, i18n) {
                report_async_failure(
                    state,
                    i18n.cannot_load_remote_list,
                    error,
                    "workspace.remote.toolbar-menu",
                    "workspace.remote.toolbar-menu",
                i18n,
                );
            }
        }
        Message::CloseToolbarRemoteMenu => state.close_toolbar_remote_menu(),
        Message::CloseHistoryCommitDiffPopup => state.close_history_commit_diff_popup(),
        Message::CloseAuxiliary => state.close_auxiliary_view(i18n),
        Message::ToolbarRemoteActionSelected { action, remote } => {
            if let Err(error) = run_toolbar_remote_action(state, action, remote) {
                let (title, source) = match action {
                    ToolbarRemoteAction::Pull => (i18n.pull_remote_failed, "workspace.remote.toolbar.pull"),
                    ToolbarRemoteAction::Push => (i18n.push_remote_failed, "workspace.remote.toolbar.push"),
                };
                report_async_failure(state, title, error, source, source, i18n);
            }
        }
        Message::Stash => {
            if let Err(error) = open_stash_panel(state) {
                report_async_failure(
                    state,
                    i18n.cannot_open_stash_panel,
                    error,
                    "workspace.stash",
                    "workspace.stash",
                i18n,
                );
            }
        }
        Message::ShowBranches => {
            state.close_history_commit_diff_popup();
            // Toggle IDEA-style floating branch dropdown
            state.show_branch_dropdown = !state.show_branch_dropdown;
            if state.show_branch_dropdown {
                // Load branches when opening
                if let Some(repo) = state.current_repository.clone() {
                    let i18n = i18n::locale(state.git_settings.language.as_deref());
                    state.branch_popup.load_branches(&repo, i18n);
                }
            }
        }
        Message::ShowHistory => {
            state.switch_git_tool_window_tab(GitToolWindowTab::Log, i18n);
            if let Some(repo) = state.current_repository.clone() {
                let i18n = i18n::locale(state.git_settings.language.as_deref());
                state.history_view.load_history(&repo, i18n);
                // Also load branches for the dashboard sidebar
                if state.branch_popup.local_branches.is_empty() {
                    state.branch_popup.load_branches(&repo, i18n);
                }
            }
        }
        Message::SwitchGitToolWindowTab(tab) => {
            state.switch_git_tool_window_tab(tab, i18n);
            if tab == GitToolWindowTab::Log {
                if let Some(repo) = state.current_repository.clone() {
                    let i18n = i18n::locale(state.git_settings.language.as_deref());
                    state.history_view.load_history(&repo, i18n);
                    if state.branch_popup.local_branches.is_empty() {
                        state.branch_popup.load_branches(&repo, i18n);
                    }
                }
            }
        }
        Message::ShowRemotes => {
            if let Err(error) = open_remote_dialog(state) {
                report_async_failure(
                    state,
                    i18n.cannot_open_remote_panel,
                    error,
                    "workspace.remote",
                    "workspace.remote",
                i18n,
                );
            }
        }
        Message::ShowTags => {
            if let Err(error) = open_tag_dialog(state) {
                report_async_failure(
                    state,
                    i18n.cannot_open_tag_panel,
                    error,
                    "workspace.tags",
                    "workspace.tags",
                i18n,
                );
            }
        }
        Message::ShowRebase => {
            if let Err(error) = open_rebase_editor(state) {
                report_async_failure(
                    state,
                    i18n.cannot_open_rebase_panel,
                    error,
                    "workspace.rebase",
                    "workspace.rebase",
                i18n,
                );
            }
        }
        Message::OpenConflictResolver => {
            state.close_auxiliary_view(i18n);
            if let Err(error) = state.open_conflict_resolver(i18n) {
                report_async_failure(
                    state,
                    i18n.cannot_open_conflict_view,
                    error,
                    "workspace.conflicts",
                    "workspace.conflicts",
                i18n,
                );
            }
        }
        Message::CloseConflictResolver => {
            let previous_section = state.shell.active_section;
            state.close_conflict_resolver(i18n);
            state.navigate_to(ShellSection::Changes, i18n);
            log_shell_navigation(
                previous_section,
                state.shell.active_section,
                "close conflict resolver",
            );
        }
        Message::SelectConflict(index) => state.select_conflict(index),
        Message::OpenConflictMerge(index) => {
            if let Err(error) = state.open_conflict_merge(index, i18n) {
                report_async_failure(
                    state,
                    i18n.cannot_open_merge_view,
                    error,
                    "workspace.conflicts",
                    "workspace.conflicts.merge",
                i18n,
                );
            }
        }
        Message::ResolveConflictWithOurs(index) => {
            if let Err(error) = resolve_conflict_with_side(
                state,
                index,
                ConflictResolution::Ours,
                i18n.accepted_ours,
                "workspace.conflicts.accept_ours",
            ) {
                report_async_failure(
                    state,
                    i18n.accept_ours_failed,
                    error,
                    "workspace.conflicts",
                    "workspace.conflicts.accept_ours",
                i18n,
                );
            }
        }
        Message::ResolveConflictWithTheirs(index) => {
            if let Err(error) = resolve_conflict_with_side(
                state,
                index,
                ConflictResolution::Theirs,
                i18n.accepted_theirs,
                "workspace.conflicts.accept_theirs",
            ) {
                report_async_failure(
                    state,
                    i18n.accept_theirs_failed,
                    error,
                    "workspace.conflicts",
                    "workspace.conflicts.accept_theirs",
                i18n,
                );
            }
        }
        Message::ConflictResolverMessage(message) => match message {
            ConflictResolverMessage::BackToList => state.close_conflict_merge(),
            ConflictResolverMessage::Refresh => {
                if let Err(error) = state.load_conflicts(i18n) {
                    report_async_failure(
                        state,
                        i18n.refresh_conflicts_failed,
                        i18n.refresh_conflicts_failed_fmt.replace("{}", &error.to_string()),
                        "workspace.conflicts",
                        "workspace.conflicts.refresh",
                    i18n,
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
                        i18n.apply_conflict_resolution_failed,
                        error,
                        "workspace.conflicts",
                        "workspace.conflicts.resolve",
                    i18n,
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
                        i18n.commit_failed,
                        error,
                        "workspace.commit",
                        "workspace.commit.submit",
                    i18n,
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
                        i18n.cannot_switch_commit_mode,
                        error,
                        "workspace.commit",
                        "workspace.commit.amend",
                    i18n,
                    );
                }
            }
            CommitDialogMessage::CancelPressed => state.close_auxiliary_view(i18n),
            CommitDialogMessage::CommitAndPushPressed => {
                // Commit first, then push
                if let Err(error) = submit_commit_dialog(state) {
                    state.commit_dialog.set_error(error.clone());
                    report_async_failure(
                        state,
                        i18n.commit_failed,
                        error,
                        "workspace.commit_and_push",
                        "workspace.commit_and_push",
                    i18n,
                    );
                } else {
                    // Commit succeeded, now push
                    return update(state, Message::Push);
                }
            }
            CommitDialogMessage::ToggleRecentMessages => {
                // Load recent messages from history file
                if let Some(repo) = &state.current_repository {
                    state.recent_commit_messages = git_core::load_recent_messages(repo.path());
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
            CommitDialogMessage::GenerateCommitMessage => {
                if !state.git_settings.llm_enabled {
                    state
                        .commit_dialog
                        .set_error(i18n.enable_ai_first.to_string());
                    return Task::none();
                }
                state.commit_dialog.is_generating = true;
                state.commit_dialog.error = None;

                // Gather diff summary from staged files
                let diff_summary = state
                    .commit_dialog
                    .diff
                    .files
                    .iter()
                    .map(|f| {
                        let path = f
                            .new_path
                            .as_deref()
                            .or(f.old_path.as_deref())
                            .unwrap_or("?");
                        let stats = format!("+{} -{}", f.additions, f.deletions);
                        let hunks: String = f
                            .hunks
                            .iter()
                            .flat_map(|h| h.lines.iter())
                            .take(80)
                            .map(|l| {
                                let prefix = match l.origin {
                                    git_core::diff::DiffLineOrigin::Addition => "+",
                                    git_core::diff::DiffLineOrigin::Deletion => "-",
                                    _ => " ",
                                };
                                format!("{}{}\n", prefix, l.content)
                            })
                            .collect();
                        format!("--- {path} ({stats})\n{hunks}")
                    })
                    .collect::<Vec<_>>()
                    .join("\n");

                // Branch name for context
                let branch_name = state.shell.context_switcher.branch_name.clone();

                // Gather recent git log messages (actual commits, not just saved messages)
                let recent_logs: Vec<String> = if let Some(repo) = &state.current_repository {
                    git_core::get_history(repo, Some(15))
                        .unwrap_or_default()
                        .into_iter()
                        .map(|entry| entry.message)
                        .collect()
                } else {
                    state.recent_commit_messages.clone()
                };

                let config = state.git_settings.llm_config();

                return Task::perform(
                    async move {
                        git_core::llm::generate_commit_message(
                            &config,
                            &branch_name,
                            &diff_summary,
                            &recent_logs,
                        )
                        .await
                    },
                    |result| {
                        Message::CommitDialogMessage(
                            CommitDialogMessage::GenerateCommitMessageResult(result),
                        )
                    },
                );
            }
            CommitDialogMessage::GenerateCommitMessageResult(result) => {
                state.commit_dialog.is_generating = false;
                match result {
                    Ok(msg) => {
                        state.commit_dialog.message = msg.clone();
                        state.commit_dialog.message_editor =
                            iced::widget::text_editor::Content::with_text(&msg);
                    }
                    Err(err) => {
                        state.commit_dialog.set_error(err);
                    }
                }
            }
        },
        Message::BranchPopupMessage(message) => {
            let i18n = i18n::locale(state.git_settings.language.as_deref());
            if branch_popup_message_closes_context_menu(&message) {
                state.branch_popup.close_context_menu();
            }

            match message {
                BranchPopupMessage::SelectBranch(name) => {
                    state.branch_popup.select_branch(name);
                    if let Ok(repo) = require_repository(state) {
                        state.branch_popup.load_selected_branch_history(&repo, i18n);
                    }
                }
                BranchPopupMessage::ToggleFolder(path_key) => {
                    state.branch_popup.toggle_folder(path_key);
                }
                BranchPopupMessage::OpenBranchContextMenu(name) => {
                    state.branch_popup.open_context_menu(name);
                    if let Ok(repo) = require_repository(state) {
                        state.branch_popup.load_selected_branch_history(&repo, i18n);
                    }
                }
                BranchPopupMessage::OpenCommitContextMenu(commit_id) => {
                    if let Ok(repo) = require_repository(state) {
                        state
                            .branch_popup
                            .select_branch_commit(&repo, commit_id.clone(), i18n);
                        state.branch_popup.open_commit_context_menu(commit_id);
                        if let Some(error) = state.branch_popup.error.clone() {
                            report_async_failure(
                                state,
                                i18n.open_commit_action_failed_main,
                                error,
                                "workspace.branches",
                                "workspace.branches.commit_menu",
                            i18n,
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
                            state.branch_popup.load_selected_branch_history(&repo, i18n);
                        }
                    }
                }
                BranchPopupMessage::ClearSearch => {
                    let selection_changed = state.branch_popup.set_search_query(String::new());
                    if selection_changed {
                        if let Ok(repo) = require_repository(state) {
                            state.branch_popup.load_selected_branch_history(&repo, i18n);
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
                        state.branch_popup.select_branch_commit(&repo, commit_id, i18n);
                        state.open_auxiliary_view(AuxiliaryView::Branches, i18n);
                        if let Some(error) = state.branch_popup.error.clone() {
                            report_async_failure(
                                state,
                                i18n.load_commit_detail_failed_main,
                                error,
                                "workspace.branches",
                                "workspace.branches.select_commit",
                            i18n,
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
                        state.branch_popup.confirm_inline_action(&repo, i18n);
                        if let Some(error) = state.branch_popup.error.clone() {
                            report_async_failure(
                                state,
                                i18n.execute_branch_action_failed,
                                error,
                                "workspace.branches",
                                "workspace.branches.inline",
                            i18n,
                            );
                        } else {
                            let _ = refresh_repository_after_action(state, &repo, false, i18n);
                            if let Some(current) = state.current_repository.clone() {
                                state.branch_popup.load_branches(&current, i18n);
                            }
                            state.open_auxiliary_view(AuxiliaryView::Branches, i18n);
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
                        state.branch_popup.create_branch(&repo, name, i18n);
                        if let Some(error) = state.branch_popup.error.clone() {
                            report_async_failure(
                                state,
                                i18n.create_branch_failed,
                                error,
                                "workspace.branches",
                                "workspace.branches.create",
                            i18n,
                            );
                        } else {
                            let _ = refresh_repository_after_action(state, &repo, false, i18n);
                            if let Some(current) = state.current_repository.clone() {
                                state.branch_popup.load_branches(&current, i18n);
                            }
                            state.open_auxiliary_view(AuxiliaryView::Branches, i18n);
                            if let Some(message) = state.branch_popup.success_message.clone() {
                                state.set_success(message, None, "workspace.branches");
                            }
                        }
                    }
                }
                BranchPopupMessage::DeleteBranch(name) => {
                    if let Ok(repo) = require_repository(state) {
                        state.branch_popup.delete_branch(&repo, name, i18n);
                        if let Some(error) = state.branch_popup.error.clone() {
                            report_async_failure(
                                state,
                                i18n.delete_branch_failed,
                                error,
                                "workspace.branches",
                                "workspace.branches.delete",
                            i18n,
                            );
                        } else {
                            let _ = refresh_repository_after_action(state, &repo, false, i18n);
                            if let Some(current) = state.current_repository.clone() {
                                state.branch_popup.load_branches(&current, i18n);
                            }
                            state.open_auxiliary_view(AuxiliaryView::Branches, i18n);
                            if let Some(message) = state.branch_popup.success_message.clone() {
                                state.set_success(message, None, "workspace.branches");
                            }
                        }
                    }
                }
                BranchPopupMessage::PrepareDeleteBranch(name) => {
                    if let Ok(repo) = require_repository(state) {
                        state.branch_popup.prepare_delete_branch(&repo, name, i18n);
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
                        state.branch_popup.checkout_branch(&repo, name.clone(), i18n);
                        if let Some(error) = state.branch_popup.error.clone() {
                            // Detect uncommitted changes conflict → show smart checkout dialog
                            if error.contains("would be overwritten")
                                || error.contains("please commit your changes or stash")
                                || error.contains("local changes")
                            {
                                state.branch_popup.error = None;
                                state.branch_popup.smart_checkout_affected_files =
                                    repo.list_uncommitted_files();
                                state.branch_popup.smart_checkout_branch = Some(name);
                                state.branch_popup.smart_checkout_is_remote = false;
                            } else {
                                report_async_failure(
                                    state,
                                    i18n.checkout_failed,
                                    error,
                                    "workspace.branches",
                                    "workspace.branches.checkout",
                                i18n,
                                );
                            }
                        } else if let Err(error) =
                            refresh_repository_after_action(state, &repo, false, i18n)
                        {
                            report_async_failure(
                                state,
                                i18n.refresh_repo_state_failed,
                                error,
                                "workspace.branches",
                                "workspace.branches.checkout",
                            i18n,
                            );
                        } else if let Some(current) = state.current_repository.clone() {
                            state.branch_popup.load_branches(&current, i18n);
                            state.open_auxiliary_view(AuxiliaryView::Branches, i18n);
                            if let Some(message) = state.branch_popup.success_message.clone() {
                                state.set_success(message, None, "workspace.branches");
                            }
                        }
                    }
                }
                BranchPopupMessage::CheckoutRemoteBranch(remote_ref) => {
                    if let Ok(repo) = require_repository(state) {
                        state
                            .branch_popup
                            .checkout_remote_branch(&repo, remote_ref.clone(), i18n);
                        if let Some(error) = state.branch_popup.error.clone() {
                            if error.contains("would be overwritten")
                                || error.contains("please commit your changes or stash")
                                || error.contains("local changes")
                            {
                                state.branch_popup.error = None;
                                state.branch_popup.smart_checkout_affected_files =
                                    repo.list_uncommitted_files();
                                state.branch_popup.smart_checkout_branch = Some(remote_ref);
                                state.branch_popup.smart_checkout_is_remote = true;
                            } else {
                                report_async_failure(
                                    state,
                                    i18n.checkout_remote_failed,
                                    error,
                                    "workspace.branches",
                                    "workspace.branches.checkout_remote",
                                i18n,
                                );
                            }
                        } else if let Err(error) =
                            refresh_repository_after_action(state, &repo, false, i18n)
                        {
                            report_async_failure(
                                state,
                                i18n.refresh_repo_state_failed,
                                error,
                                "workspace.branches",
                                "workspace.branches.checkout_remote",
                            i18n,
                            );
                        } else if let Some(current) = state.current_repository.clone() {
                            state.branch_popup.load_branches(&current, i18n);
                            state.open_auxiliary_view(AuxiliaryView::Branches, i18n);
                            if let Some(message) = state.branch_popup.success_message.clone() {
                                state.set_success(message, None, "workspace.branches");
                            }
                        }
                    }
                }
                BranchPopupMessage::SmartCheckout(name) => {
                    state.branch_popup.smart_checkout_branch = None;
                    state.branch_popup.smart_checkout_affected_files.clear();
                    if let Ok(repo) = require_repository(state) {
                        state.branch_popup.is_loading = true;
                        match repo.smart_checkout_branch(&name) {
                            Ok(()) => {
                                state.branch_popup.is_loading = false;
                                state.branch_popup.selected_branch = Some(name.clone());
                                state.branch_popup.success_message =
                                    Some(i18n.smart_checkout_msg_fmt.replace("{}", &name));
                                let _ = refresh_repository_after_action(state, &repo, true, i18n);
                                if let Some(current) = state.current_repository.clone() {
                                    state.branch_popup.load_branches(&current, i18n);
                                }
                                state.set_success(
                                    i18n.smart_checkout_done_fmt.replace("{}", &name),
                                    None,
                                    "workspace.branches",
                                );
                            }
                            Err(error) => {
                                state.branch_popup.is_loading = false;
                                let msg = i18n.smart_checkout_failed_fmt.replace("{}", &error.to_string());
                                state.branch_popup.error = Some(msg.clone());
                                report_async_failure(
                                    state,
                                    i18n.smart_checkout_failed,
                                    msg,
                                    "workspace.branches",
                                    "workspace.branches.smart_checkout",
                                i18n,
                                );
                            }
                        }
                    }
                }
                BranchPopupMessage::ForceCheckout(name) => {
                    state.branch_popup.smart_checkout_branch = None;
                    state.branch_popup.smart_checkout_affected_files.clear();
                    if let Ok(repo) = require_repository(state) {
                        state.branch_popup.is_loading = true;
                        match repo.force_checkout_branch(&name) {
                            Ok(()) => {
                                state.branch_popup.is_loading = false;
                                state.branch_popup.selected_branch = Some(name.clone());
                                state.branch_popup.success_message =
                                    Some(i18n.force_checkout_msg_fmt.replace("{}", &name));
                                let _ = refresh_repository_after_action(state, &repo, false, i18n);
                                if let Some(current) = state.current_repository.clone() {
                                    state.branch_popup.load_branches(&current, i18n);
                                }
                                state.set_success(
                                    i18n.force_checkout_done_fmt.replace("{}", &name),
                                    None,
                                    "workspace.branches",
                                );
                            }
                            Err(error) => {
                                state.branch_popup.is_loading = false;
                                let msg = i18n.force_checkout_failed_fmt.replace("{}", &error.to_string());
                                state.branch_popup.error = Some(msg.clone());
                                report_async_failure(
                                    state,
                                    i18n.force_checkout_failed,
                                    msg,
                                    "workspace.branches",
                                    "workspace.branches.force_checkout",
                                i18n,
                                );
                            }
                        }
                    }
                }
                BranchPopupMessage::CancelSmartCheckout => {
                    state.branch_popup.smart_checkout_branch = None;
                    state.branch_popup.smart_checkout_affected_files.clear();
                }
                BranchPopupMessage::MergeBranch(name) => {
                    if let Ok(repo) = require_repository(state) {
                        let branch_name = name.clone();
                        state.branch_popup.merge_branch(&repo, name, i18n);
                        if let Some(error) = state.branch_popup.error.clone() {
                            report_async_failure(
                                state,
                                i18n.merge_branch_failed,
                                error,
                                "workspace.branches",
                                "workspace.branches.merge",
                            i18n,
                            );
                        } else if let Err(error) =
                            refresh_repository_after_action(state, &repo, true, i18n)
                        {
                            report_async_failure(
                                state,
                                i18n.refresh_repo_state_failed,
                                error,
                                "workspace.branches",
                                "workspace.branches.merge",
                            i18n,
                            );
                        } else if state.has_conflicts() {
                            state.set_warning(
                                i18n.merge_conflict_warning,
                                Some(i18n.merge_conflict_detail_fmt.replace("{}", &branch_name)),
                                "workspace.branches.merge",
                            );
                        } else if !state.has_conflicts() {
                            if let Some(current) = state.current_repository.clone() {
                                state.branch_popup.load_branches(&current, i18n);
                            }
                            state.open_auxiliary_view(AuxiliaryView::Branches, i18n);
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
                            .checkout_and_rebase(&repo, &branch, &onto, i18n);
                        if let Some(error) = state.branch_popup.error.clone() {
                            report_async_failure(
                                state,
                                i18n.checkout_rebase_failed,
                                error,
                                "workspace.branches",
                                "workspace.branches.checkout_rebase",
                            i18n,
                            );
                        } else if let Err(error) =
                            refresh_repository_after_action(state, &repo, true, i18n)
                        {
                            report_async_failure(
                                state,
                                i18n.refresh_repo_state_failed,
                                error,
                                "workspace.branches",
                                "workspace.branches.checkout_rebase",
                            i18n,
                            );
                        } else if !state.has_conflicts() {
                            if let Some(current) = state.current_repository.clone() {
                                state.branch_popup.load_branches(&current, i18n);
                            }
                            state.open_auxiliary_view(AuxiliaryView::Branches, i18n);
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
                            .compare_refs_preview(&repo, &selected, &current, i18n);
                        state.open_auxiliary_view(AuxiliaryView::Branches, i18n);
                        if let Some(error) = state.branch_popup.error.clone() {
                            report_async_failure(
                                state,
                                i18n.compare_branch_failed,
                                error,
                                "workspace.branches",
                                "workspace.branches.compare",
                            i18n,
                            );
                        }
                    }
                }
                BranchPopupMessage::CompareWithWorktree(reference) => {
                    if let Ok(repo) = require_repository(state) {
                        state.branch_popup.comparison_title = Some(i18n.worktree_label_fmt.replace("{}", &reference));
                        state
                            .branch_popup
                            .compare_ref_to_workdir_preview(&repo, &reference, i18n);
                        state.open_auxiliary_view(AuxiliaryView::Branches, i18n);
                        if let Some(error) = state.branch_popup.error.clone() {
                            report_async_failure(
                                state,
                                i18n.worktree_diff_failed,
                                error,
                                "workspace.branches",
                                "workspace.branches.worktree_diff",
                            i18n,
                            );
                        }
                    }
                }
                BranchPopupMessage::RebaseCurrentOnto(onto) => {
                    if let Ok(repo) = require_repository(state) {
                        state.branch_popup.rebase_current_onto(&repo, &onto, i18n);
                        if let Some(error) = state.branch_popup.error.clone() {
                            report_async_failure(
                                state,
                                i18n.start_rebase_failed,
                                error,
                                "workspace.branches",
                                "workspace.branches.rebase",
                            i18n,
                            );
                        } else if let Err(error) =
                            refresh_repository_after_action(state, &repo, true, i18n)
                        {
                            report_async_failure(
                                state,
                                i18n.refresh_repo_state_failed,
                                error,
                                "workspace.branches",
                                "workspace.branches.rebase",
                            i18n,
                            );
                        } else if !state.has_conflicts() {
                            if let Some(current) = state.current_repository.clone() {
                                state.branch_popup.load_branches(&current, i18n);
                            }
                            state.open_auxiliary_view(AuxiliaryView::Branches, i18n);
                            if let Some(message) = state.branch_popup.success_message.clone() {
                                state.set_success(message, None, "workspace.branches");
                            }
                        }
                    }
                }
                BranchPopupMessage::FetchRemote(remote_name) => {
                    if let Ok(repo) = require_repository(state) {
                        state.branch_popup.fetch_remote(&repo, &remote_name, i18n);
                        if let Some(error) = state.branch_popup.error.clone() {
                            report_async_failure(
                                state,
                                i18n.update_remote_failed,
                                error,
                                "workspace.branches",
                                "workspace.branches.fetch",
                            i18n,
                            );
                        } else {
                            let _ = refresh_repository_after_action(state, &repo, false, i18n);
                            if let Some(current) = state.current_repository.clone() {
                                state.branch_popup.load_branches(&current, i18n);
                            }
                            state.open_auxiliary_view(AuxiliaryView::Branches, i18n);
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
                            .push_branch_to_remote(&repo, &remote, &branch, i18n);
                        if let Some(error) = state.branch_popup.error.clone() {
                            report_async_failure(
                                state,
                                i18n.push_branch_failed,
                                error,
                                "workspace.branches",
                                "workspace.branches.push",
                            i18n,
                            );
                        } else {
                            let _ = refresh_repository_after_action(state, &repo, false, i18n);
                            if let Some(current) = state.current_repository.clone() {
                                state.branch_popup.load_branches(&current, i18n);
                            }
                            state.open_auxiliary_view(AuxiliaryView::Branches, i18n);
                            if let Some(message) = state.branch_popup.success_message.clone() {
                                state.set_success(message, None, "workspace.branches");
                            }
                        }
                    }
                }
                BranchPopupMessage::SetUpstream { branch, upstream } => {
                    if let Ok(repo) = require_repository(state) {
                        state.branch_popup.set_upstream(&repo, &branch, &upstream, i18n);
                        if let Some(error) = state.branch_popup.error.clone() {
                            report_async_failure(
                                state,
                                i18n.set_tracking_failed,
                                error,
                                "workspace.branches",
                                "workspace.branches.upstream",
                            i18n,
                            );
                        } else {
                            let _ = refresh_repository_after_action(state, &repo, false, i18n);
                            if let Some(current) = state.current_repository.clone() {
                                state.branch_popup.load_branches(&current, i18n);
                            }
                            state.open_auxiliary_view(AuxiliaryView::Branches, i18n);
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
                            i18n.open_tag_panel_failed,
                            error,
                            "workspace.tags",
                            "workspace.tags.open",
                        i18n,
                        );
                    }
                }
                BranchPopupMessage::CopyCommitHash(commit_id) => {
                    if let Err(error) = copy_text_to_clipboard(&commit_id) {
                        report_async_failure(
                            state,
                            i18n.copy_commit_hash_failed,
                            error,
                            "workspace.branches",
                            "workspace.branches.copy_commit",
                        i18n,
                        );
                    } else {
                        state.open_auxiliary_view(AuxiliaryView::Branches, i18n);
                        state.show_toast(
                            crate::state::FeedbackLevel::Success,
                            i18n.commit_hash_copied,
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
                                    state.open_auxiliary_view(AuxiliaryView::Branches, i18n);
                                    state.set_success(
                                        i18n.patch_exported,
                                        Some(path.display().to_string()),
                                        "workspace.branches.patch",
                                    );
                                }
                                Err(error) => report_async_failure(
                                    state,
                                    i18n.export_patch_failed,
                                    i18n.export_patch_failed_fmt.replace("{}", &error.to_string()),
                                    "workspace.branches",
                                    "workspace.branches.patch",
                                i18n,
                                ),
                            }
                        }
                    }
                }
                BranchPopupMessage::PrepareCherryPickCommit(commit_id) => {
                    if let Ok(repo) = require_repository(state) {
                        state.pending_commit_action = state
                            .branch_popup
                            .prepare_cherry_pick_commit(&repo, commit_id, i18n);
                        if let Some(error) = state.branch_popup.error.clone() {
                            report_async_failure(
                                state,
                                i18n.cannot_prepare_cherry_pick,
                                error,
                                "workspace.branches",
                                "workspace.branches.cherry_pick.prepare",
                            i18n,
                            );
                        }
                    }
                }
                BranchPopupMessage::PrepareRevertCommit(commit_id) => {
                    if let Ok(repo) = require_repository(state) {
                        state.pending_commit_action =
                            state.branch_popup.prepare_revert_commit(&repo, commit_id, i18n);
                        if let Some(error) = state.branch_popup.error.clone() {
                            report_async_failure(
                                state,
                                i18n.cannot_prepare_revert,
                                error,
                                "workspace.branches",
                                "workspace.branches.revert.prepare",
                            i18n,
                            );
                        }
                    }
                }
                BranchPopupMessage::PrepareResetCurrentBranchToCommit(commit_id) => {
                    if let Ok(repo) = require_repository(state) {
                        state.pending_commit_action = state
                            .branch_popup
                            .prepare_reset_current_branch_to_commit(&repo, commit_id, i18n);
                        if let Some(error) = state.branch_popup.error.clone() {
                            report_async_failure(
                                state,
                                i18n.cannot_prepare_reset_branch,
                                error,
                                "workspace.branches",
                                "workspace.branches.reset.prepare",
                            i18n,
                            );
                        }
                    }
                }
                BranchPopupMessage::PreparePushCurrentBranchToCommit(commit_id) => {
                    if let Ok(repo) = require_repository(state) {
                        state.pending_commit_action = state
                            .branch_popup
                            .prepare_push_current_branch_to_commit(&repo, commit_id, i18n);
                        if let Some(error) = state.branch_popup.error.clone() {
                            report_async_failure(
                                state,
                                i18n.cannot_prepare_push_here,
                                error,
                                "workspace.branches",
                                "workspace.branches.push_to_here.prepare",
                            i18n,
                            );
                        }
                    }
                }
                BranchPopupMessage::ContinueInProgressCommitAction => {
                    if let Ok(repo) = require_repository(state) {
                        state.branch_popup.continue_in_progress_commit_action(&repo, i18n);
                        if let Some(error) = state.branch_popup.error.clone() {
                            report_async_failure(
                                state,
                                i18n.continue_commit_flow_failed,
                                error,
                                "workspace.branches",
                                "workspace.branches.follow_up.continue",
                            i18n,
                            );
                        } else if let Err(error) =
                            refresh_repository_after_action(state, &repo, true, i18n)
                        {
                            report_async_failure(
                                state,
                                i18n.refresh_repo_state_failed,
                                error,
                                "workspace.branches",
                                "workspace.branches.follow_up.continue",
                            i18n,
                            );
                        } else if state.has_conflicts() {
                            state.open_auxiliary_view(AuxiliaryView::Branches, i18n);
                            state.set_warning(
                                i18n.commit_flow_has_conflicts,
                                Some(i18n.commit_flow_conflicts_detail.to_string()),
                                "workspace.branches",
                            );
                        } else {
                            if let Some(current) = state.current_repository.clone() {
                                state.branch_popup.load_branches(&current, i18n);
                            }
                            state.open_auxiliary_view(AuxiliaryView::Branches, i18n);
                            if let Some(message) = state.branch_popup.success_message.clone() {
                                state.set_success(message, None, "workspace.branches");
                            }
                        }
                    }
                }
                BranchPopupMessage::AbortInProgressCommitAction => {
                    if let Ok(repo) = require_repository(state) {
                        state.branch_popup.abort_in_progress_commit_action(&repo, i18n);
                        if let Some(error) = state.branch_popup.error.clone() {
                            report_async_failure(
                                state,
                                i18n.abort_commit_flow_failed,
                                error,
                                "workspace.branches",
                                "workspace.branches.follow_up.abort",
                            i18n,
                            );
                        } else if let Err(error) =
                            refresh_repository_after_action(state, &repo, false, i18n)
                        {
                            report_async_failure(
                                state,
                                i18n.refresh_repo_state_failed,
                                error,
                                "workspace.branches",
                                "workspace.branches.follow_up.abort",
                            i18n,
                            );
                        } else {
                            if let Some(current) = state.current_repository.clone() {
                                state.branch_popup.load_branches(&current, i18n);
                            }
                            state.open_auxiliary_view(AuxiliaryView::Branches, i18n);
                            if let Some(message) = state.branch_popup.success_message.clone() {
                                state.set_success(message, None, "workspace.branches");
                            }
                        }
                    }
                }
                BranchPopupMessage::OpenConflictList => {
                    state.close_auxiliary_view(i18n);
                    state.navigate_to(ShellSection::Conflicts, i18n);
                    state.set_info(
                        i18n.switched_to_conflicts,
                        Some(i18n.switched_to_conflicts_detail.to_string()),
                        "workspace.conflicts",
                    );
                }
                BranchPopupMessage::ConfirmPendingCommitAction => {
                    if let Ok(repo) = require_repository(state) {
                        if let Some(confirmation) = state.pending_commit_action.take() {
                            let action_kind = confirmation.action.kind();
                            let keep_branch_popup_open =
                                state.auxiliary_view == Some(AuxiliaryView::Branches);
                            let keep_branch_dropdown_open = state.show_branch_dropdown;
                            state
                                .branch_popup
                                .confirm_pending_commit_action(&repo, confirmation, i18n);
                            if let Some(error) = state.branch_popup.error.clone() {
                                report_async_failure(
                                    state,
                                    i18n.execute_commit_action_failed,
                                    error,
                                    "workspace.branches",
                                    "workspace.branches.commit_action",
                                i18n,
                                );
                            } else if let Err(error) = refresh_repository_after_action(
                                state,
                                &repo,
                                matches!(
                                    action_kind,
                                    branch_popup::PendingCommitActionKind::CherryPick
                                        | branch_popup::PendingCommitActionKind::Revert
                                ),
                                i18n,
                            ) {
                                report_async_failure(
                                    state,
                                    i18n.refresh_repo_state_failed,
                                    error,
                                    "workspace.branches",
                                    "workspace.branches.commit_action",
                                i18n,
                                );
                            } else if state.has_conflicts() {
                                let detail = match action_kind {
                                    branch_popup::PendingCommitActionKind::CherryPick => {
                                        i18n.cherry_pick_conflict_detail.to_string()
                                    }
                                    branch_popup::PendingCommitActionKind::Revert => {
                                        i18n.revert_conflict_detail.to_string()
                                    }
                                    branch_popup::PendingCommitActionKind::ResetCurrentBranch
                                    | branch_popup::PendingCommitActionKind::PushCurrentBranchToCommit => {
                                        i18n.generic_conflict_detail.to_string()
                                    }
                                };
                                if keep_branch_popup_open {
                                    state.open_auxiliary_view(AuxiliaryView::Branches, i18n);
                                }
                                state.set_warning(
                                    i18n.commit_action_conflict,
                                    Some(detail),
                                    "workspace.branches",
                                );
                            } else {
                                if keep_branch_popup_open || keep_branch_dropdown_open {
                                    if let Some(current) = state.current_repository.clone() {
                                        state.branch_popup.load_branches(&current, i18n);
                                    }
                                }
                                if keep_branch_popup_open {
                                    state.open_auxiliary_view(AuxiliaryView::Branches, i18n);
                                }
                                if let Some(message) = state.branch_popup.success_message.clone() {
                                    state.set_success(message, None, "workspace.branches");
                                }
                            }
                        }
                    }
                }
                BranchPopupMessage::CancelPendingCommitAction => {
                    state.pending_commit_action = None;
                    state.branch_popup.cancel_pending_commit_action();
                }
                BranchPopupMessage::SetResetMode(mode) => {
                    if let Some(ref mut confirmation) = state.pending_commit_action {
                        if let PendingCommitAction::ResetCurrentBranch {
                            ref mut reset_mode, ..
                        } = confirmation.action
                        {
                            *reset_mode = mode;
                        }
                    }
                }
                BranchPopupMessage::ClearPreview => state.branch_popup.clear_preview(),
                BranchPopupMessage::Refresh => {
                    if let Ok(repo) = require_repository(state) {
                        logging::LogManager::log_context_switcher("refresh", &repo.name());
                        state.branch_popup.load_branches(&repo, i18n);
                        state.open_auxiliary_view(AuxiliaryView::Branches, i18n);
                        if let Some(error) = state.branch_popup.error.clone() {
                            report_async_failure(
                                state,
                                i18n.refresh_branch_list_failed,
                                error,
                                "workspace.branches",
                                "workspace.branches.refresh",
                            i18n,
                            );
                        }
                    }
                }
                BranchPopupMessage::OpenCommit => {
                    state.close_auxiliary_view(i18n);
                    return update(state, Message::Commit);
                }
                BranchPopupMessage::OpenPull => {
                    state.close_auxiliary_view(i18n);
                    return update(state, Message::Pull);
                }
                BranchPopupMessage::OpenPush => {
                    state.close_auxiliary_view(i18n);
                    return update(state, Message::Push);
                }
                BranchPopupMessage::OpenHistory => {
                    state.close_auxiliary_view(i18n);
                    return update(state, Message::ShowHistory);
                }
                BranchPopupMessage::OpenRemotes => {
                    state.close_auxiliary_view(i18n);
                    return update(state, Message::ShowRemotes);
                }
                BranchPopupMessage::OpenTags => {
                    state.close_auxiliary_view(i18n);
                    return update(state, Message::ShowTags);
                }
                BranchPopupMessage::OpenStashes => {
                    state.close_auxiliary_view(i18n);
                    return update(state, Message::Stash);
                }
                BranchPopupMessage::OpenRebase => {
                    state.close_auxiliary_view(i18n);
                    return update(state, Message::ShowRebase);
                }
                BranchPopupMessage::Close => {
                    state.show_branch_dropdown = false;
                    state.close_auxiliary_view(i18n);
                }
            }
        }
        Message::HistoryMessage(message) => { let i18n = i18n::locale(state.git_settings.language.as_deref()); match message {
            HistoryMessage::Refresh => {
                if let Ok(repo) = require_repository(state) {
                    let i18n = i18n::locale(state.git_settings.language.as_deref());
                    state.history_view.load_history(&repo, i18n);
                    state.history_view.context_menu_commit = None;
                    if let Some(error) = state.history_view.error.clone() {
                        report_async_failure(
                            state,
                            i18n.load_history_failed,
                            error,
                            "workspace.history",
                            "workspace.history.refresh",
                        i18n,
                        );
                    }
                }
            }
            HistoryMessage::SelectCommit(commit_id) => {
                if let Ok(repo) = require_repository(state) {
                    let i18n = i18n::locale(state.git_settings.language.as_deref());
                    state.history_view.context_menu_commit = None;
                    state.history_view.select_commit(&repo, commit_id, i18n);
                    if let Some(error) = state.history_view.error.clone() {
                        report_async_failure(
                            state,
                            i18n.load_commit_detail_failed,
                            error,
                            "workspace.history",
                            "workspace.history.select",
                        i18n,
                        );
                    }
                }
            }
            HistoryMessage::ViewDiff(_) => {
                let i18n = i18n::locale(state.git_settings.language.as_deref());
                state.set_info(
                    i18n.commit_detail_loaded,
                    Some(i18n.commit_detail_loaded_hint.to_string()),
                    "workspace.history",
                );
            }
            HistoryMessage::ViewCommitFileDiff(commit_id, file_path) => {
                if let Ok(repo) = require_repository(state) {
                    match load_history_commit_file_diff(&repo, &commit_id, &file_path) {
                        Ok(history_diff) => {
                            state.history_view.context_menu_commit = None;
                            state.history_view.context_menu_anchor = None;
                            show_history_commit_file_diff(
                                state,
                                commit_id,
                                file_path,
                                history_diff.diff,
                                history_diff.editor_diff,
                            );
                        }
                        Err(error) => {
                            let i18n = i18n::locale(state.git_settings.language.as_deref());
                            report_async_failure(
                                state,
                                i18n.load_commit_file_diff_failed,
                                error,
                                "workspace.history",
                                "workspace.history.file_diff",
                            i18n,
                            );
                        }
                    }
                }
            }
            HistoryMessage::ToggleCommitFileDisplayMode => {
                state.history_view.toggle_commit_file_display_mode();
            }
            HistoryMessage::CommitFileTreeEvent(tree_message) => match tree_message {
                crate::widgets::tree_widget::TreeMessage::SelectNode(node_id) => {
                    if let Some(path) = node_id.strip_prefix("file:") {
                        let Some(commit_id) = state.history_view.selected_commit.clone() else {
                            return iced::Task::none();
                        };
                        return update(
                            state,
                            Message::HistoryMessage(HistoryMessage::ViewCommitFileDiff(
                                commit_id,
                                path.to_string(),
                            )),
                        );
                    }
                    state.history_view.toggle_commit_file_tree_node(node_id);
                }
                crate::widgets::tree_widget::TreeMessage::ToggleNode(node_id) => {
                    state.history_view.toggle_commit_file_tree_node(node_id);
                }
                crate::widgets::tree_widget::TreeMessage::NodeContextMenu(_, _) => {}
            },
            HistoryMessage::TrackContextMenuCursor(position) => {
                state.history_view.track_context_menu_cursor(position);
            }
            HistoryMessage::OpenCommitContextMenu(commit_id) => {
                if let Ok(repo) = require_repository(state) {
                    let i18n = i18n::locale(state.git_settings.language.as_deref());
                    state.history_view.select_commit(&repo, commit_id.clone(), i18n);
                    state.history_view.context_menu_commit = Some(commit_id);
                    state.history_view.context_menu_anchor =
                        Some(state.history_view.context_menu_cursor);
                    if let Some(error) = state.history_view.error.clone() {
                        report_async_failure(
                            state,
                            i18n.open_commit_action_failed,
                            error,
                            "workspace.history",
                            "workspace.history.context_menu",
                        i18n,
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
                        i18n.copy_commit_hash_failed,
                        error,
                        "workspace.history",
                        "workspace.history.copy_commit",
                    i18n,
                    );
                } else {
                    state.history_view.context_menu_commit = None;
                    state.show_toast(
                        crate::state::FeedbackLevel::Success,
                        i18n.commit_hash_copied,
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
                                    i18n.patch_exported,
                                    Some(path.display().to_string()),
                                    "workspace.history.patch",
                                );
                            }
                            Err(error) => report_async_failure(
                                state,
                                i18n.export_patch_failed,
                                i18n.export_patch_failed_fmt.replace("{}", &error.to_string()),
                                "workspace.history",
                                "workspace.history.patch",
                            i18n,
                            ),
                        }
                    }
                }
            }
            HistoryMessage::CompareWithCurrent(commit_id) => {
                let Some(current_branch) = state.history_view.current_branch_name.clone() else {
                    report_async_failure(
                        state,
                        i18n.cannot_compare_branch,
                        i18n.detached_no_branch_compare.to_string(),
                        "workspace.history",
                        "workspace.history.compare_current",
                    i18n,
                    );
                    return iced::Task::none();
                };
                if let Ok(repo) = require_repository(state) {
                    state.history_view.context_menu_commit = None;
                    state.branch_popup.load_branches(&repo, i18n);
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
                    state.branch_popup.load_branches(&repo, i18n);
                }
                return update(
                    state,
                    Message::BranchPopupMessage(BranchPopupMessage::CompareWithWorktree(commit_id)),
                );
            }
            HistoryMessage::PrepareCreateBranch(commit_id) => {
                state.history_view.context_menu_commit = None;
                if let Ok(repo) = require_repository(state) {
                    state.branch_popup.load_branches(&repo, i18n);
                    state.branch_popup.prepare_create_from_selected(commit_id);
                    state.open_auxiliary_view(AuxiliaryView::Branches, i18n);
                }
            }
            HistoryMessage::PrepareTagFromCommit(commit_id) => {
                state.history_view.context_menu_commit = None;
                if let Ok(repo) = require_repository(state) {
                    state.branch_popup.load_branches(&repo, i18n);
                }
                return update(
                    state,
                    Message::BranchPopupMessage(BranchPopupMessage::PrepareTagFromCommit(
                        commit_id,
                    )),
                );
            }
            HistoryMessage::PrepareCherryPickCommit(commit_id) => {
                state.history_view.context_menu_commit = None;
                if let Ok(repo) = require_repository(state) {
                    // Check for merge commit
                    match git_core::commit::get_commit(&repo, &commit_id) {
                        Ok(info) if info.parent_ids.len() > 1 => {
                            state.set_error(i18n.merge_no_cherry_pick.to_string());
                            return iced::Task::none();
                        }
                        Err(e) => {
                            state.set_error(i18n.read_commit_detail_failed_fmt.replace("{}", &e.to_string()));
                            return iced::Task::none();
                        }
                        _ => {}
                    }
                    // Execute cherry-pick directly (IDEA-style, no confirmation)
                    match git_core::cherry_pick_commit(&repo, &commit_id) {
                        Ok(()) => {
                            state.show_toast(
                                crate::state::FeedbackLevel::Success,
                                i18n.cherry_picked_fmt.replace("{}", short_commit_id(&commit_id)),
                                None,
                            );
                            if let Err(e) = refresh_repository_after_action(state, &repo, true, i18n) {
                                state.set_error(i18n.refresh_repo_state_failed_fmt.replace("{}", &e.to_string()));
                            }
                        }
                        Err(e) => {
                            state.set_error(i18n.cherry_pick_failed_fmt.replace("{}", &e.to_string()));
                        }
                    }
                }
            }
            HistoryMessage::PrepareRevertCommit(commit_id) => {
                state.history_view.context_menu_commit = None;
                if let Ok(repo) = require_repository(state) {
                    // Check for merge commit first
                    match git_core::commit::get_commit(&repo, &commit_id) {
                        Ok(info) if info.parent_ids.len() > 1 => {
                            state.set_error(i18n.no_merge_revert.to_string());
                            return iced::Task::none();
                        }
                        Err(e) => {
                            state.set_error(i18n.read_commit_detail_failed_fmt.replace("{}", &e.to_string()));
                            return iced::Task::none();
                        }
                        _ => {}
                    }
                    // Execute revert directly (IDEA-style, no confirmation)
                    match git_core::revert_commit(&repo, &commit_id) {
                        Ok(()) => {
                            state.show_toast(
                                crate::state::FeedbackLevel::Success,
                                i18n.reverted_commit_fmt.replace("{}", short_commit_id(&commit_id)),
                                None,
                            );
                            if let Err(e) = refresh_repository_after_action(state, &repo, true, i18n) {
                                state.set_error(i18n.refresh_repo_state_failed_fmt.replace("{}", &e.to_string()));
                            }
                        }
                        Err(e) => {
                            state.set_error(i18n.revert_commit_failed_fmt.replace("{}", &e.to_string()));
                        }
                    }
                }
            }
            HistoryMessage::PrepareResetCurrentBranchToCommit(commit_id) => {
                state.history_view.context_menu_commit = None;
                if let Ok(repo) = require_repository(state) {
                    state.pending_commit_action = state
                        .branch_popup
                        .prepare_reset_current_branch_to_commit(&repo, commit_id, i18n);
                    if let Some(error) = state.branch_popup.error.clone() {
                        state.set_error(error);
                    }
                }
            }
            HistoryMessage::PreparePushCurrentBranchToCommit(commit_id) => {
                state.history_view.context_menu_commit = None;
                if let Ok(repo) = require_repository(state) {
                    state.pending_commit_action = state
                        .branch_popup
                        .prepare_push_current_branch_to_commit(&repo, commit_id, i18n);
                    if let Some(error) = state.branch_popup.error.clone() {
                        state.set_error(error);
                    }
                }
            }
            HistoryMessage::EditCommitMessage(commit_id) => {
                if let Ok(repo) = require_repository(state) {
                    match git_core::edit_commit_message(&repo, &commit_id) {
                        Ok(execution) => {
                            state.history_view.context_menu_commit = None;
                            if let Err(error) = refresh_repository_after_action(state, &repo, true, i18n)
                            {
                                report_async_failure(
                                    state,
                                    i18n.refresh_repo_state_failed,
                                    error,
                                    "workspace.history",
                                    "workspace.history.reword",
                                i18n,
                                );
                            } else if state.has_conflicts() {
                                open_rebase_session_with_context(state, Some(&commit_id), i18n);
                                state.set_warning(
                                    i18n.reword_conflict,
                                    Some(
                                        i18n.reword_conflict_detail
                                            .to_string(),
                                    ),
                                    "workspace.history.reword",
                                );
                            } else if matches!(execution, git_core::RewriteExecution::InProgress) {
                                open_rebase_session_with_context(state, Some(&commit_id), i18n);
                                if let Err(error) = switch_commit_dialog_to_amend(state) {
                                    report_async_failure(
                                        state,
                                        i18n.cannot_open_commit_edit_panel,
                                        error,
                                        "workspace.history",
                                        "workspace.history.reword",
                                    i18n,
                                    );
                                } else {
                                    state.set_info(
                                        i18n.stopped_at_commit,
                                        Some(
                                            i18n.reword_hint_detail
                                                .to_string(),
                                        ),
                                        "workspace.history.reword",
                                    );
                                }
                            } else {
                                state.set_success(
                                    i18n.prepared_reword_fmt.replace("{}", short_commit_id(&commit_id)),
                                    Some(i18n.history_refreshed_detail.to_string()),
                                    "workspace.history.reword",
                                );
                            }
                        }
                        Err(error) => report_async_failure(
                            state,
                            i18n.start_reword_failed,
                            error.to_string(),
                            "workspace.history",
                            "workspace.history.reword",
                        i18n,
                        ),
                    }
                }
            }
            HistoryMessage::FixupCommitToPrevious(commit_id) => {
                if let Ok(repo) = require_repository(state) {
                    match git_core::fixup_commit_to_previous(&repo, &commit_id) {
                        Ok(execution) => {
                            state.history_view.context_menu_commit = None;
                            if let Err(error) = refresh_repository_after_action(state, &repo, true, i18n)
                            {
                                report_async_failure(
                                    state,
                                    i18n.refresh_repo_state_failed,
                                    error,
                                    "workspace.history",
                                    "workspace.history.fixup",
                                i18n,
                                );
                            } else if state.has_conflicts() {
                                open_rebase_session_with_context(state, Some(&commit_id), i18n);
                                state.set_warning(
                                    i18n.fixup_conflict,
                                    Some(
                                        i18n.fixup_conflict_detail
                                            .to_string(),
                                    ),
                                    "workspace.history.fixup",
                                );
                            } else if matches!(execution, git_core::RewriteExecution::InProgress) {
                                open_rebase_session_with_context(state, Some(&commit_id), i18n);
                                state.set_info(
                                    i18n.fixup_in_progress,
                                    Some(
                                        i18n.rewrite_in_progress_detail
                                            .to_string(),
                                    ),
                                    "workspace.history.fixup",
                                );
                            } else {
                                state.set_success(
                                    i18n.fixup_done_fmt.replace("{}", short_commit_id(&commit_id)),
                                    Some(
                                        i18n.history_refreshed_continue.to_string(),
                                    ),
                                    "workspace.history.fixup",
                                );
                            }
                        }
                        Err(error) => report_async_failure(
                            state,
                            i18n.fixup_failed,
                            error.to_string(),
                            "workspace.history",
                            "workspace.history.fixup",
                        i18n,
                        ),
                    }
                }
            }
            HistoryMessage::SquashCommitToPrevious(commit_id) => {
                if let Ok(repo) = require_repository(state) {
                    match git_core::squash_commit_to_previous(&repo, &commit_id) {
                        Ok(execution) => {
                            state.history_view.context_menu_commit = None;
                            if let Err(error) = refresh_repository_after_action(state, &repo, true, i18n)
                            {
                                report_async_failure(
                                    state,
                                    i18n.refresh_repo_state_failed,
                                    error,
                                    "workspace.history",
                                    "workspace.history.squash",
                                i18n,
                                );
                            } else if state.has_conflicts() {
                                open_rebase_session_with_context(state, Some(&commit_id), i18n);
                                state.set_warning(
                                    i18n.squash_conflict,
                                    Some(
                                        i18n.squash_conflict_detail
                                            .to_string(),
                                    ),
                                    "workspace.history.squash",
                                );
                            } else if matches!(execution, git_core::RewriteExecution::InProgress) {
                                open_rebase_session_with_context(state, Some(&commit_id), i18n);
                                state.set_info(
                                    i18n.squash_in_progress,
                                    Some(
                                        i18n.rewrite_in_progress_detail
                                            .to_string(),
                                    ),
                                    "workspace.history.squash",
                                );
                            } else {
                                state.set_success(
                                    i18n.squash_done_fmt.replace("{}", short_commit_id(&commit_id)),
                                    Some(
                                        i18n.history_refreshed_continue.to_string(),
                                    ),
                                    "workspace.history.squash",
                                );
                            }
                        }
                        Err(error) => report_async_failure(
                            state,
                            i18n.squash_failed,
                            error.to_string(),
                            "workspace.history",
                            "workspace.history.squash",
                        i18n,
                        ),
                    }
                }
            }
            HistoryMessage::DropCommitFromHistory(commit_id) => {
                if let Ok(repo) = require_repository(state) {
                    match git_core::drop_commit_from_history(&repo, &commit_id) {
                        Ok(execution) => {
                            state.history_view.context_menu_commit = None;
                            if let Err(error) = refresh_repository_after_action(state, &repo, true, i18n)
                            {
                                report_async_failure(
                                    state,
                                    i18n.refresh_repo_state_failed,
                                    error,
                                    "workspace.history",
                                    "workspace.history.drop",
                                i18n,
                                );
                            } else if state.has_conflicts() {
                                open_rebase_session_with_context(state, Some(&commit_id), i18n);
                                state.set_warning(
                                    i18n.drop_conflict,
                                    Some(
                                        i18n.drop_conflict_detail
                                            .to_string(),
                                    ),
                                    "workspace.history.drop",
                                );
                            } else if matches!(execution, git_core::RewriteExecution::InProgress) {
                                open_rebase_session_with_context(state, Some(&commit_id), i18n);
                                state.set_info(
                                    i18n.drop_in_progress,
                                    Some(
                                        i18n.rewrite_in_progress_detail
                                            .to_string(),
                                    ),
                                    "workspace.history.drop",
                                );
                            } else {
                                state.set_success(
                                    i18n.drop_done_fmt.replace("{}", short_commit_id(&commit_id)),
                                    Some(
                                        i18n.history_refreshed_continue.to_string(),
                                    ),
                                    "workspace.history.drop",
                                );
                            }
                        }
                        Err(error) => report_async_failure(
                            state,
                            i18n.drop_failed,
                            error.to_string(),
                            "workspace.history",
                            "workspace.history.drop",
                        i18n,
                        ),
                    }
                }
            }
            HistoryMessage::OpenInteractiveRebaseFromCommit(commit_id) => {
                if let Ok(repo) = require_repository(state) {
                    state.history_view.context_menu_commit = None;
                    state
                        .rebase_editor
                        .prepare_interactive_rebase(&repo, commit_id, i18n);
                    state.open_auxiliary_view(AuxiliaryView::Rebase, i18n);
                    if let Some(error) = state.rebase_editor.error.clone() {
                        report_async_failure(
                            state,
                            i18n.open_rebase_from_here_failed,
                            error,
                            "workspace.history",
                            "workspace.history.rebase_from_here",
                        i18n,
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
                        i18n.cannot_squash,
                        i18n.select_two_contiguous,
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
                    let is_contiguous =
                        sorted.len() >= 2 && sorted.windows(2).all(|w| w[1] == w[0] + 1);

                    if !is_contiguous {
                        state.set_error_with_source(
                            i18n.cannot_squash,
                            i18n.only_contiguous_squash,
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
                            let _ = refresh_repository_after_action(state, &repo, false, i18n);
                            state.set_success(
                                i18n.commits_uncommitted,
                                Some(i18n.changes_returned_to_staging.to_string()),
                                "workspace.uncommit",
                            );
                        }
                        Err(e) => {
                            report_async_failure(
                                state,
                                i18n.uncommit_failed,
                                e.to_string(),
                                "workspace.uncommit",
                                "workspace.uncommit",
                            i18n,
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
                                    i18n.push_success,
                                    Some(i18n.pushed_to_fmt.replace("{}", &commit_id[..7.min(commit_id.len())]).replacen("{}", &remote, 1)),
                                    "workspace.push.up_to",
                                );
                            }
                            Err(e) => {
                                report_async_failure(
                                    state,
                                    i18n.push_to_commit_failed,
                                    e.to_string(),
                                    "workspace.push.up_to",
                                    "workspace.push.up_to",
                                i18n,
                                );
                            }
                        }
                    }
                }
            }
            HistoryMessage::SetSearchQuery(query) => state.history_view.set_search_query(query),
            HistoryMessage::Search => {
                if let Ok(repo) = require_repository(state) {
                    let i18n = i18n::locale(state.git_settings.language.as_deref());
                    state.history_view.perform_search(&repo, i18n);
                    if let Some(error) = state.history_view.error.clone() {
                        report_async_failure(
                            state,
                            i18n.search_failed,
                            error,
                            "workspace.history",
                            "workspace.history.search",
                        i18n,
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
                    label: i18n.log_tab_fmt.replace("{}", &id.to_string()),
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
        } },
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
                            i18n.fetch_remote_failed,
                            error,
                            "workspace.remote",
                            "workspace.remote.fetch",
                        i18n,
                        );
                    } else {
                        let _ = refresh_repository_after_action(state, &repo, false, i18n);
                        if let Some(current) = state.current_repository.clone() {
                            state.remote_dialog.load_remotes(&current, i18n);
                        }
                        state.open_auxiliary_view(AuxiliaryView::Remotes, i18n);
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
                            i18n.push_remote_dialog_failed,
                            error,
                            "workspace.remote",
                            "workspace.remote.push",
                        i18n,
                        );
                    } else {
                        let _ = refresh_repository_after_action(state, &repo, false, i18n);
                        if let Some(current) = state.current_repository.clone() {
                            state.remote_dialog.load_remotes(&current, i18n);
                        }
                        state.open_auxiliary_view(AuxiliaryView::Remotes, i18n);
                        if let Some(message) = state.remote_dialog.success_message.clone() {
                            state.set_success(message, None, "workspace.remote");
                            state.show_toast(
                                crate::state::FeedbackLevel::Success,
                                i18n.push_toast_success,
                                Some(i18n.push_toast_detail.to_string()),
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
                            i18n.pull_remote_dialog_failed,
                            error,
                            "workspace.remote",
                            "workspace.remote.pull",
                        i18n,
                        );
                    } else if let Err(error) = refresh_repository_after_action(state, &repo, true, i18n) {
                        report_async_failure(
                            state,
                            i18n.refresh_repo_state_failed,
                            error,
                            "workspace.remote",
                            "workspace.remote.pull",
                        i18n,
                        );
                    } else if !state.has_conflicts() {
                        if let Some(current) = state.current_repository.clone() {
                            state.remote_dialog.load_remotes(&current, i18n);
                        }
                        state.open_auxiliary_view(AuxiliaryView::Remotes, i18n);
                        if let Some(message) = state.remote_dialog.success_message.clone() {
                            state.set_success(message, None, "workspace.remote");
                            state.show_toast(
                                crate::state::FeedbackLevel::Success,
                                i18n.pull_toast_success,
                                Some(i18n.pull_toast_detail.to_string()),
                            );
                        }
                    }
                }
            }
            RemoteDialogMessage::SetUsername(value) => state.remote_dialog.username = value,
            RemoteDialogMessage::SetPassword(value) => state.remote_dialog.password = value,
            RemoteDialogMessage::Refresh => {
                if let Ok(repo) = require_repository(state) {
                    state.remote_dialog.load_remotes(&repo, i18n);
                    state.open_auxiliary_view(AuxiliaryView::Remotes, i18n);
                    if let Some(error) = state.remote_dialog.error.clone() {
                        report_async_failure(
                            state,
                            i18n.refresh_remote_list_failed,
                            error,
                            "workspace.remote",
                            "workspace.remote.refresh",
                        i18n,
                        );
                    }
                }
            }
            RemoteDialogMessage::Close => {
                state.remote_dialog.mode = views::remote_dialog::RemoteDialogMode::Overview;
                state.close_auxiliary_view(i18n);
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
                    let remote = state
                        .remote_dialog
                        .selected_remote
                        .clone()
                        .or_else(|| state.remote_dialog.preferred_remote.clone())
                        .unwrap_or_else(|| "origin".to_string());
                    let branch = state
                        .remote_dialog
                        .current_branch_name
                        .clone()
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
                            state.remote_dialog.success_message =
                                Some(i18n.pushed_fmt.replace("{}", &branch).replacen("{}", &remote, 1).replacen("{}", &branch, 1));
                            let _ = refresh_repository_after_action(state, &repo, false, i18n);
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
                    let remote = state
                        .remote_dialog
                        .selected_remote
                        .clone()
                        .or_else(|| state.remote_dialog.preferred_remote.clone())
                        .unwrap_or_else(|| "origin".to_string());
                    let branch = state
                        .remote_dialog
                        .current_branch_name
                        .clone()
                        .unwrap_or_else(|| "main".to_string());

                    state.remote_dialog.is_loading = true;
                    state.remote_dialog.error = None;

                    let result = git_core::pull(&repo, &remote, &branch, None);

                    state.remote_dialog.is_loading = false;
                    match result {
                        Ok(()) => {
                            state.remote_dialog.success_message =
                                Some(i18n.pulled_fmt.replace("{}", &remote).replacen("{}", &branch, 1).replacen("{}", &branch, 1));
                            let _ = refresh_repository_after_action(state, &repo, false, i18n);
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
                            i18n.create_tag_failed,
                            error,
                            "workspace.tags",
                            "workspace.tags.create",
                        i18n,
                        );
                    } else {
                        let _ = refresh_repository_after_action(state, &repo, false, i18n);
                        if let Some(current) = state.current_repository.clone() {
                            state.tag_dialog.load_tags(&current);
                        }
                        state.open_auxiliary_view(AuxiliaryView::Tags, i18n);
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
                            i18n.delete_tag_failed,
                            error,
                            "workspace.tags",
                            "workspace.tags.delete",
                        i18n,
                        );
                    } else {
                        let _ = refresh_repository_after_action(state, &repo, false, i18n);
                        if let Some(current) = state.current_repository.clone() {
                            state.tag_dialog.load_tags(&current);
                        }
                        state.open_auxiliary_view(AuxiliaryView::Tags, i18n);
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
                                Some(i18n.tag_pushed_fmt.replace("{}", &name).replacen("{}", &remote, 1));
                        }
                        Err(e) => {
                            state.tag_dialog.error = Some(i18n.push_tag_failed_fmt.replace("{}", &e.to_string()));
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
                                Some(i18n.remote_tag_deleted_fmt.replace("{}", &name).replacen("{}", &remote, 1));
                        }
                        Err(e) => {
                            state.tag_dialog.error = Some(i18n.delete_remote_tag_failed_fmt.replace("{}", &e.to_string()));
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
                                Some(i18n.invalid_commit_ref.to_string());
                        }
                    }
                }
            }
            TagDialogMessage::DeleteLocalAndRemote(name) => {
                if let Ok(repo) = require_repository(state) {
                    // Delete local first
                    if let Err(e) = git_core::delete_tag(&repo, &name) {
                        state.tag_dialog.error = Some(i18n.delete_local_tag_failed_fmt.replace("{}", &e.to_string()));
                    } else {
                        let remote = repo
                            .current_upstream_remote()
                            .unwrap_or_else(|| "origin".to_string());
                        if let Err(e) = git_core::delete_remote_tag(&repo, &name, &remote) {
                            state.tag_dialog.error = Some(i18n.delete_remote_tag_failed_fmt.replace("{}", &e.to_string()));
                        } else {
                            state.tag_dialog.success_message =
                                Some(i18n.tag_deleted_local_remote_fmt.replace("{}", &name));
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
                    state.open_auxiliary_view(AuxiliaryView::Tags, i18n);
                    if let Some(error) = state.tag_dialog.error.clone() {
                        report_async_failure(
                            state,
                            i18n.refresh_tag_list_failed,
                            error,
                            "workspace.tags",
                            "workspace.tags.refresh",
                        i18n,
                        );
                    }
                }
            }
            TagDialogMessage::Close => state.close_auxiliary_view(i18n),
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
                            i18n.save_stash_failed,
                            error,
                            "workspace.stash",
                            "workspace.stash.save",
                        i18n,
                        );
                    } else {
                        let _ = refresh_repository_after_action(state, &repo, false, i18n);
                        if let Some(current) = state.current_repository.clone() {
                            state.stash_panel.load_stashes(&current);
                        }
                        state.open_auxiliary_view(AuxiliaryView::Stashes, i18n);
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
                            i18n.apply_stash_failed,
                            error,
                            "workspace.stash",
                            "workspace.stash.apply",
                        i18n,
                        );
                    } else if let Err(error) = refresh_repository_after_action(state, &repo, true, i18n) {
                        report_async_failure(
                            state,
                            i18n.refresh_repo_state_failed,
                            error,
                            "workspace.stash",
                            "workspace.stash.apply",
                        i18n,
                        );
                    } else if !state.has_conflicts() {
                        if let Some(current) = state.current_repository.clone() {
                            state.stash_panel.load_stashes(&current);
                        }
                        state.open_auxiliary_view(AuxiliaryView::Stashes, i18n);
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
                            i18n.drop_stash_failed,
                            error,
                            "workspace.stash",
                            "workspace.stash.drop",
                        i18n,
                        );
                    } else {
                        let _ = refresh_repository_after_action(state, &repo, false, i18n);
                        if let Some(current) = state.current_repository.clone() {
                            state.stash_panel.load_stashes(&current);
                        }
                        state.open_auxiliary_view(AuxiliaryView::Stashes, i18n);
                        if let Some(message) = state.stash_panel.success_message.clone() {
                            state.set_success(message, None, "workspace.stash");
                        }
                    }
                }
            }
            StashPanelMessage::Refresh => {
                if let Ok(repo) = require_repository(state) {
                    state.stash_panel.load_stashes(&repo);
                    state.open_auxiliary_view(AuxiliaryView::Stashes, i18n);
                    if let Some(error) = state.stash_panel.error.clone() {
                        report_async_failure(
                            state,
                            i18n.refresh_stash_list_failed,
                            error,
                            "workspace.stash",
                            "workspace.stash.refresh",
                        i18n,
                        );
                    }
                }
            }
            StashPanelMessage::Close => state.close_auxiliary_view(i18n),
            StashPanelMessage::PopStash(index) => {
                if let Ok(repo) = require_repository(state) {
                    state.stash_panel.apply_stash(&repo, index);
                    if state.stash_panel.error.is_none() {
                        let _ = refresh_repository_after_action(state, &repo, false, i18n);
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
                        state.stash_panel.error = Some(i18n.branch_name_empty.to_string());
                    } else if let Ok(repo) = require_repository(state) {
                        match git_core::unstash_as_branch(&repo, index, &branch_name) {
                            Ok(()) => {
                                let _ = refresh_repository_after_action(state, &repo, false, i18n);
                                state.stash_panel.success_message =
                                    Some(i18n.applied_to_branch_fmt.replace("{}", &branch_name));
                                if let Some(current) = state.current_repository.clone() {
                                    state.stash_panel.load_stashes(&current);
                                }
                            }
                            Err(e) => {
                                state.stash_panel.error = Some(i18n.apply_to_branch_failed_fmt.replace("{}", &e.to_string()));
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
                            state.stash_panel.success_message = Some(i18n.all_stashes_cleared.to_string());
                            state.stash_panel.stashes.clear();
                            state.stash_panel.selected_stash = None;
                        }
                        Err(e) => {
                            state.stash_panel.error = Some(i18n.clear_stashes_failed_fmt.replace("{}", &e.to_string()));
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
                        i18n.cannot_open_commit_edit_panel,
                        error,
                        "workspace.rebase",
                        "workspace.rebase.edit_current",
                    i18n,
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
                    state.rebase_editor.start_rebase(&repo, i18n);
                    if let Some(error) = state.rebase_editor.error.clone() {
                        report_async_failure(
                            state,
                            i18n.start_rebase_panel_failed,
                            error,
                            "workspace.rebase",
                            "workspace.rebase.start",
                        i18n,
                        );
                    } else if let Err(error) = refresh_repository_after_action(state, &repo, true, i18n) {
                        report_async_failure(
                            state,
                            i18n.refresh_repo_state_failed,
                            error,
                            "workspace.rebase",
                            "workspace.rebase.start",
                        i18n,
                        );
                    } else {
                        if let Some(current) = state.current_repository.clone() {
                            state.rebase_editor.load_status(&current, i18n);
                        }
                        state.open_auxiliary_view(AuxiliaryView::Rebase, i18n);
                        if state.has_conflicts() {
                            state.set_warning(
                                i18n.rebase_conflict_warning,
                                Some(
                                    i18n.rebase_conflict_detail
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
                    state.rebase_editor.continue_rebase(&repo, i18n);
                    if let Some(error) = state.rebase_editor.error.clone() {
                        report_async_failure(
                            state,
                            i18n.continue_rebase_failed,
                            error,
                            "workspace.rebase",
                            "workspace.rebase.continue",
                        i18n,
                        );
                    } else if let Err(error) = refresh_repository_after_action(state, &repo, true, i18n) {
                        report_async_failure(
                            state,
                            i18n.refresh_repo_state_failed,
                            error,
                            "workspace.rebase",
                            "workspace.rebase.continue",
                        i18n,
                        );
                    } else if !state.has_conflicts() {
                        if let Some(current) = state.current_repository.clone() {
                            state.rebase_editor.load_status(&current, i18n);
                        }
                        state.open_auxiliary_view(AuxiliaryView::Rebase, i18n);
                        if let Some(message) = state.rebase_editor.success_message.clone() {
                            state.set_success(message, None, "workspace.rebase");
                        }
                    }
                }
            }
            RebaseEditorMessage::SkipCommit => {
                if let Ok(repo) = require_repository(state) {
                    state.rebase_editor.skip_commit(&repo, i18n);
                    if let Some(error) = state.rebase_editor.error.clone() {
                        report_async_failure(
                            state,
                            i18n.skip_commit_failed,
                            error,
                            "workspace.rebase",
                            "workspace.rebase.skip",
                        i18n,
                        );
                    } else if let Err(error) = refresh_repository_after_action(state, &repo, true, i18n) {
                        report_async_failure(
                            state,
                            i18n.refresh_repo_state_failed,
                            error,
                            "workspace.rebase",
                            "workspace.rebase.skip",
                        i18n,
                        );
                    } else if !state.has_conflicts() {
                        if let Some(current) = state.current_repository.clone() {
                            state.rebase_editor.load_status(&current, i18n);
                        }
                        state.open_auxiliary_view(AuxiliaryView::Rebase, i18n);
                        if let Some(message) = state.rebase_editor.success_message.clone() {
                            state.set_success(message, None, "workspace.rebase");
                        }
                    }
                }
            }
            RebaseEditorMessage::AbortRebase => {
                if let Ok(repo) = require_repository(state) {
                    state.rebase_editor.abort_rebase(&repo, i18n);
                    if let Some(error) = state.rebase_editor.error.clone() {
                        report_async_failure(
                            state,
                            i18n.abort_rebase_failed,
                            error,
                            "workspace.rebase",
                            "workspace.rebase.abort",
                        i18n,
                        );
                    } else if let Err(error) = refresh_repository_after_action(state, &repo, false, i18n)
                    {
                        report_async_failure(
                            state,
                            i18n.refresh_repo_state_failed,
                            error,
                            "workspace.rebase",
                            "workspace.rebase.abort",
                        i18n,
                        );
                    } else {
                        if let Some(current) = state.current_repository.clone() {
                            state.rebase_editor.load_status(&current, i18n);
                        }
                        state.open_auxiliary_view(AuxiliaryView::Rebase, i18n);
                        if let Some(message) = state.rebase_editor.success_message.clone() {
                            state.set_success(message, None, "workspace.rebase");
                        }
                    }
                }
            }
            RebaseEditorMessage::Refresh => {
                if let Ok(repo) = require_repository(state) {
                    state.rebase_editor.load_status(&repo, i18n);
                    state.open_auxiliary_view(AuxiliaryView::Rebase, i18n);
                    if let Some(error) = state.rebase_editor.error.clone() {
                        report_async_failure(
                            state,
                            i18n.refresh_rebase_status_failed,
                            error,
                            "workspace.rebase",
                            "workspace.rebase.refresh",
                        i18n,
                        );
                    }
                }
            }
            RebaseEditorMessage::Close => state.close_auxiliary_view(i18n),
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

/// Compute the editor-oriented split diff model for the currently selected file.
fn update_editor_diff_model(state: &mut AppState) {
    state.editor_diff = None;
    state.split_diff_editor = None;
    state.unified_diff_editor = None;

    // Build unified editor for unified mode
    if state.diff_presentation == DiffPresentation::Unified {
        if let Some(diff) = &state.current_diff {
            if !diff.files.is_empty() {
                state.unified_diff_editor =
                    Some(widgets::diff_editor::UnifiedDiffEditorState::from_diff(
                        diff,
                        state.git_settings.editor_font_size_f32(),
                    ));
            }
        }
        return;
    }

    if state.full_file_preview.is_some() || state.full_file_preview_binary {
        return;
    }

    let (Some(repo), Some(path)) = (&state.current_repository, &state.selected_change_path) else {
        return;
    };

    let is_staged = state
        .staged_changes
        .iter()
        .any(|change| &change.path == path && change.staged && !change.unstaged);

    match git_core::diff::build_editor_diff_model(repo, path, is_staged) {
        Ok(Some(model)) => {
            state.editor_diff = Some(model.clone());
            state.split_diff_editor =
                Some(widgets::diff_editor::SplitDiffEditorState::with_font_size(
                    model,
                    state.git_settings.editor_font_size_f32(),
                ));
            if state.selected_hunk_index.is_none() {
                state.selected_hunk_index = Some(0);
            }
        }
        Ok(None) => {}
        Err(error) => {
            warn!("Failed to build editor diff model for {}: {}", path, error);
        }
    }
}

fn require_repository(state: &AppState) -> Result<Repository, String> {
    let i18n = i18n::locale(state.git_settings.language.as_deref());
    state
        .current_repository
        .clone()
        .ok_or_else(|| i18n.no_repo_opened.to_string())
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
            .map_err(|error| format!("{}: {}", "load staged diff failed", error))?;

        diff.total_additions += file_diff.total_additions;
        diff.total_deletions += file_diff.total_deletions;
        diff.files.extend(file_diff.files);
    }

    Ok(diff)
}

struct HistoryCommitFileDiffData {
    diff: git_core::diff::Diff,
    editor_diff: Option<git_core::diff::EditorDiffModel>,
}

fn load_history_commit_file_diff(
    repo: &Repository,
    commit_id: &str,
    file_path: &str,
) -> Result<HistoryCommitFileDiffData, String> {
    let commit = git_core::commit::get_commit(repo, commit_id)
        .map_err(|error| format!("read commit detail failed: {error}"))?;

    let diff = if commit.parent_ids.is_empty() {
        git_core::diff::diff_commit_against_parent(repo, commit_id)
            .map_err(|error| format!("load root commit diff failed: {error}"))?
    } else {
        git_core::diff::diff_refs(repo, &format!("{commit_id}^"), commit_id)
            .or_else(|_| git_core::diff::diff_commit_against_parent(repo, commit_id))
            .map_err(|error| format!("load commit diff failed: {error}"))?
    };

    let files = diff
        .files
        .into_iter()
        .filter(|file| {
            file.new_path.as_deref() == Some(file_path) || file.old_path.as_deref() == Some(file_path)
        })
        .collect::<Vec<_>>();

    if files.is_empty() {
        return Err(format!("no diff found for {file_path} in this commit"));
    }

    let total_additions = files.iter().map(|file| file.additions).sum();
    let total_deletions = files.iter().map(|file| file.deletions).sum();
    let diff = git_core::diff::Diff {
        files,
        total_additions,
        total_deletions,
    };
    let editor_diff = if let Some(file_diff) = diff.files.first() {
        let old_bytes = match (file_diff.old_path.as_deref(), commit.parent_ids.first()) {
            (Some(path), Some(parent_id)) => {
                git_core::diff::read_file_bytes_at_commit(repo, parent_id, Path::new(path))
                    .map_err(|error| format!("read old version failed: {error}"))?
                    .unwrap_or_default()
            }
            _ => Vec::new(),
        };
        let new_bytes = if let Some(path) = file_diff.new_path.as_deref() {
            git_core::diff::read_file_bytes_at_commit(repo, commit_id, Path::new(path))
                .map_err(|error| format!("read new version failed: {error}"))?
                .unwrap_or_default()
        } else {
            Vec::new()
        };

        git_core::diff::build_editor_diff_model_from_file_contents(
            file_diff,
            &old_bytes,
            &new_bytes,
        )
    } else {
        None
    };

    Ok(HistoryCommitFileDiffData {
        diff,
        editor_diff,
    })
}

fn show_history_commit_file_diff(
    state: &mut AppState,
    commit_id: String,
    file_path: String,
    diff: git_core::diff::Diff,
    editor_diff: Option<git_core::diff::EditorDiffModel>,
) {
    state.open_history_commit_diff_popup(commit_id, file_path, diff, editor_diff);
}

fn refresh_repository_after_action(
    state: &mut AppState,
    repo: &Repository,
    prefer_conflicts: bool,
    i18n: &I18n,
) -> Result<(), String> {
    let _ = repo;
    state.refresh_current_repository(prefer_conflicts, i18n)?;
    refresh_workspace_views(state);
    Ok(())
}

fn refresh_workspace_views(state: &mut AppState) {
    update_editor_diff_model(state);
    refresh_open_auxiliary_view(state);
}

fn refresh_open_auxiliary_view(state: &mut AppState) {
    let Some(repo) = state.current_repository.clone() else {
        return;
    };

    match state.auxiliary_view {
        Some(AuxiliaryView::Branches) => {
            let i18n = i18n::locale(state.git_settings.language.as_deref());
            state.branch_popup.load_branches(&repo, i18n);
        }
        Some(AuxiliaryView::Remotes) => {
            let i18n = i18n::locale(state.git_settings.language.as_deref());
            state.remote_dialog.load_remotes(&repo, i18n);
        }
        Some(AuxiliaryView::Tags) => state.tag_dialog.load_tags(&repo),
        Some(AuxiliaryView::Stashes) => state.stash_panel.load_stashes(&repo),
        Some(AuxiliaryView::Rebase) => {
            let i18n = i18n::locale(state.git_settings.language.as_deref());
            state.rebase_editor.load_status(&repo, i18n);
        }
        Some(AuxiliaryView::Worktrees) => state.worktree_state.load_worktrees(&repo),
        Some(AuxiliaryView::Settings)
        | Some(AuxiliaryView::Commit)
        | Some(AuxiliaryView::History)
        | None => {}
    }
}

fn open_rebase_session_with_context(state: &mut AppState, context_commit_id: Option<&str>, i18n: &I18n) {
    state.rebase_editor.todo_is_editable = false;
    state.rebase_editor.todo_base_ref = None;
    state.rebase_editor.onto_branch.clear();

    if let Some(commit_id) = context_commit_id {
        state.rebase_editor.base_branch = commit_id.to_string();
    }

    if let Some(repo) = state.current_repository.clone() {
        state.rebase_editor.load_status(&repo, i18n);
    }

    state.open_auxiliary_view(AuxiliaryView::Rebase, i18n);
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
    let i18n = i18n::locale(state.git_settings.language.as_deref());

    let repo = require_repository(state)?;
    let branch_name = match repo.current_branch() {
        Ok(Some(branch)) => branch,
        Ok(None) => {
            return Err(match action {
                ToolbarRemoteAction::Pull => i18n.detached_no_pull.to_string(),
                ToolbarRemoteAction::Push => i18n.detached_no_push.to_string(),
            });
        }
        Err(error) => return Err(i18n.read_branch_failed_fmt.replace("{}", &error.to_string())),
    };

    match action {
        ToolbarRemoteAction::Pull => {
            git_core::remote::pull(&repo, &remote_name, &branch_name, None)
                .map_err(|error| i18n.pull_remote_failed_fmt.replace("{}", &error.to_string()))?;
            refresh_repository_after_action(state, &repo, true, i18n)?;

            if state.has_conflicts() {
                state.set_warning(
                    i18n.pulled_remote_fmt.replace("{}", &remote_name).replacen("{}", &branch_name, 1),
                    Some(i18n.merge_conflict_found_detail.to_string()),
                    "workspace.remote.toolbar.pull",
                );
            } else {
                state.set_success(
                    i18n.pulled_remote_fmt.replace("{}", &remote_name).replacen("{}", &branch_name, 1),
                    Some(i18n.repo_state_refreshed.to_string()),
                    "workspace.remote.toolbar.pull",
                );
                state.show_toast(
                    crate::state::FeedbackLevel::Success,
                    i18n.pull_toast_success,
                    Some(i18n.pull_toast_detail_fmt.replace("{}", &remote_name)),
                );
            }
        }
        ToolbarRemoteAction::Push => {
            git_core::remote::push(&repo, &remote_name, &branch_name, None)
                .map_err(|error| i18n.push_remote_failed_fmt.replace("{}", &error.to_string()))?;
            refresh_repository_after_action(state, &repo, false, i18n)?;
            state.set_success(
                i18n.pushed_remote_fmt.replace("{}", &branch_name).replacen("{}", &remote_name, 1),
                Some(i18n.repo_state_refreshed.to_string()),
                "workspace.remote.toolbar.push",
            );
            state.show_toast(
                crate::state::FeedbackLevel::Success,
                i18n.push_toast_success,
                Some(i18n.push_toast_detail_fmt.replace("{}", &branch_name).replacen("{}", &remote_name, 1)),
            );
        }
    }

    Ok(())
}

fn open_commit_dialog(state: &mut AppState) -> Result<(), String> {
    require_repository(state)?;
    let i18n = i18n::locale(state.git_settings.language.as_deref());

    state.navigate_to(ShellSection::Changes, i18n);
    state.switch_git_tool_window_tab(GitToolWindowTab::Changes, i18n);
    state.set_info(
        i18n.commit_panel_opened,
        Some(i18n.commit_panel_opened_detail.to_string()),
        "workspace.commit",
    );
    Ok(())
}

fn switch_commit_dialog_to_amend(state: &mut AppState) -> Result<(), String> {
    let repo = require_repository(state)?;
    let i18n = i18n::locale(state.git_settings.language.as_deref());
    let diff = build_staged_diff(&repo, &state.staged_changes)?;
    let head_commit = git_core::history::get_history(&repo, Some(1))
        .map_err(|error| i18n.read_recent_commit_failed_fmt.replace("{}", &error.to_string()))?
        .into_iter()
        .next()
        .ok_or_else(|| i18n.no_commit_history_amend.to_string())?;
    let commit = git_core::commit::get_commit(&repo, &head_commit.id)
        .map_err(|error| i18n.load_commit_detail_err_fmt.replace("{}", &error.to_string()))?;

    state.commit_dialog.diff = diff;
    state.commit_dialog.staged_files = state.staged_changes.clone();
    state.commit_dialog.enable_amend_mode(commit);
    state.set_info(
        i18n.switched_to_amend,
        Some(i18n.switched_to_amend_detail.to_string()),
        "workspace.commit",
    );
    Ok(())
}

fn switch_commit_dialog_to_new_commit_mode(state: &mut AppState) -> Result<(), String> {
    let i18n = i18n::locale(state.git_settings.language.as_deref());
    if state.commit_dialog.is_amend {
        state.commit_dialog.diff =
            build_staged_diff(&require_repository(state)?, &state.staged_changes)?;
        state.commit_dialog.staged_files = state.staged_changes.clone();
        state.commit_dialog.disable_amend_mode();
        state.set_info(
            i18n.switched_to_normal_commit,
            Some(i18n.switched_to_normal_commit_detail.to_string()),
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
    let i18n = i18n::locale(state.git_settings.language.as_deref());
    state.commit_dialog.start_commit();

    let commit_id = if state.commit_dialog.is_amend {
        let commit_to_amend = state
            .commit_dialog
            .commit_to_amend
            .as_ref()
            .ok_or_else(|| i18n.missing_amend_context.to_string())?;
        git_core::commit::amend_commit(&repo, &commit_to_amend.id, &state.commit_dialog.message)
            .map_err(|error| i18n.amend_commit_failed_fmt.replace("{}", &error.to_string()))?
    } else {
        git_core::commit::create_commit(&repo, &state.commit_dialog.message, "", "")
            .map_err(|error| i18n.create_commit_failed_fmt.replace("{}", &error.to_string()))?
    };

    state.commit_dialog.commit_success();
    refresh_repository_after_action(state, &repo, false, i18n)?;

    let still_rebasing = state.current_repository.as_ref().is_some_and(|current| {
        current.get_state() == git_core::repository::RepositoryState::Rebasing
    });
    if still_rebasing && state.commit_dialog.is_amend {
        if let Some(current) = state.current_repository.clone() {
            state.rebase_editor.load_status(&current, i18n);
        }
        state.open_auxiliary_view(AuxiliaryView::Rebase, i18n);
    } else {
        state.close_auxiliary_view(i18n);
        state.navigate_to(ShellSection::Changes, i18n);
    }

    let short_id = &commit_id[..commit_id.len().min(8)];
    state.set_success(
        if state.commit_dialog.is_amend && still_rebasing {
            i18n.updated_commit_rebase_fmt.replace("{}", short_id)
        } else if state.commit_dialog.is_amend {
            i18n.updated_commit_fmt.replace("{}", short_id)
        } else {
            i18n.created_commit_fmt.replace("{}", short_id)
        },
        Some(if state.commit_dialog.is_amend && still_rebasing {
            i18n.still_rebasing_detail.to_string()
        } else {
            i18n.commit_done_detail.to_string()
        }),
        "workspace.commit",
    );
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
            | BranchPopupMessage::SetResetMode(_)
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
    let i18n = i18n::locale(state.git_settings.language.as_deref());
    state.remote_dialog.load_remotes(&repo, i18n);
    if let Some(error) = state.remote_dialog.error.clone() {
        return Err(error);
    }
    state.open_auxiliary_view(AuxiliaryView::Remotes, i18n);
    state.set_info(i18n.remote_opened, None, "workspace.remote");
    Ok(())
}

fn open_tag_dialog(state: &mut AppState) -> Result<(), String> {
    let repo = require_repository(state)?;
    let i18n = i18n::locale(state.git_settings.language.as_deref());
    state.tag_dialog.load_tags(&repo);
    if let Some(error) = state.tag_dialog.error.clone() {
        return Err(error);
    }
    state.open_auxiliary_view(AuxiliaryView::Tags, i18n);
    state.set_info(i18n.tags_opened, None, "workspace.tags");
    Ok(())
}

fn open_stash_panel(state: &mut AppState) -> Result<(), String> {
    let repo = require_repository(state)?;
    let i18n = i18n::locale(state.git_settings.language.as_deref());
    state.stash_panel.load_stashes(&repo);
    if let Some(error) = state.stash_panel.error.clone() {
        return Err(error);
    }
    state.open_auxiliary_view(AuxiliaryView::Stashes, i18n);
    state.set_info(i18n.stash_opened, None, "workspace.stash");
    Ok(())
}

fn open_rebase_editor(state: &mut AppState) -> Result<(), String> {
    let repo = require_repository(state)?;
    let i18n = i18n::locale(state.git_settings.language.as_deref());
    state.rebase_editor.clear_draft_context();
    state.rebase_editor.load_status(&repo, i18n);
    if let Some(error) = state.rebase_editor.error.clone() {
        return Err(error);
    }
    state.open_auxiliary_view(AuxiliaryView::Rebase, i18n);
    state.set_info(i18n.rebase_opened, None, "workspace.rebase");
    Ok(())
}

fn resolve_selected_conflict(state: &mut AppState) -> Result<(), String> {
    let repo = require_repository(state)?;
    let i18n = i18n::locale(state.git_settings.language.as_deref());
    let resolver = state
        .conflict_resolver
        .clone()
        .ok_or_else(|| i18n.no_conflict_resolver_state.to_string())?;
    let resolved_content = resolver
        .preview_content
        .clone()
        .unwrap_or_else(|| resolver.get_preview_content());

    git_core::diff::resolve_conflict(
        &repo,
        Path::new(&resolver.diff.path),
        ConflictResolution::Custom(resolved_content),
    )
    .map_err(|error| i18n.write_conflict_failed_fmt.replace("{}", &error.to_string()))?;

    refresh_repository_after_action(state, &repo, true, i18n)?;
    if state.has_conflicts() {
        state.set_success(
            i18n.conflict_file_written_back,
            Some(i18n.conflict_file_continue_fmt.replace("{}", &resolver.diff.path)),
            "workspace.conflicts",
        );
    } else {
        state.navigate_to(ShellSection::Changes, i18n);
        state.set_success(
            i18n.all_conflicts_resolved,
            Some(i18n.conflict_file_indexed_fmt.replace("{}", &resolver.diff.path)),
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
    let i18n = i18n::locale(state.git_settings.language.as_deref());
    let conflict = state
        .conflict_files
        .get(index)
        .cloned()
        .ok_or_else(|| i18n.conflict_file_not_found.to_string())?;
    let path = conflict.path.clone();

    git_core::diff::resolve_conflict(&repo, Path::new(&path), resolution)
        .map_err(|error| i18n.write_conflict_failed_fmt.replace("{}", &error.to_string()))?;

    refresh_repository_after_action(state, &repo, true, i18n)?;

    if state.has_conflicts() {
        state.set_success(
            success_title.to_string(),
            Some(i18n.conflict_path_continue_fmt.replace("{}", &path)),
            source,
        );
    } else {
        state.navigate_to(ShellSection::Changes, i18n);
        state.set_success(
            success_title.to_string(),
            Some(i18n.conflict_path_indexed_fmt.replace("{}", &path)),
            source,
        );
    }

    Ok(())
}

fn select_relative_file(state: &mut AppState, delta: isize) {
    let i18n = i18n::locale(state.git_settings.language.as_deref());
    let all_changes: Vec<&Change> = state
        .staged_changes
        .iter()
        .chain(state.unstaged_changes.iter())
        .chain(state.untracked_files.iter())
        .collect();

    if all_changes.is_empty() {
        state.set_warning(
            i18n.no_files_to_browse,
            Some(i18n.no_changes_no_navigation.to_string()),
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
                i18n.load_file_diff_failed,
                error,
                "workspace.select_change",
                "workspace.select_change",
            i18n,
            );
        }
    }
}

fn navigate_hunk(state: &mut AppState, delta: isize) -> Task<Message> {
    if state.diff_presentation == DiffPresentation::Split {
        let Some(model) = state.editor_diff.as_ref() else {
            return Task::none();
        };
        let total_hunks = model.hunks.len();
        if total_hunks == 0 {
            return Task::none();
        }

        let current = state.selected_hunk_index.unwrap_or(0) as isize;
        let next = (current + delta).clamp(0, (total_hunks - 1) as isize) as usize;
        state.selected_hunk_index = Some(next);

        if let Some(editor) = state.split_diff_editor.as_mut() {
            return editor
                .scroll_to_hunk(next)
                .map(Message::SplitDiffEditorEvent);
        }

        return Task::none();
    }

    let Some(diff) = state.current_diff.as_ref() else {
        return Task::none();
    };
    let total_hunks: usize = diff.files.iter().map(|file| file.hunks.len()).sum();
    if total_hunks == 0 {
        return Task::none();
    }

    let current = state.selected_hunk_index.unwrap_or(0) as isize;
    let next = (current + delta).clamp(0, (total_hunks - 1) as isize) as usize;
    state.selected_hunk_index = Some(next);

    if let Some(editor) = state.unified_diff_editor.as_mut() {
        return editor.scroll_to_hunk(next).map(Message::UnifiedDiffEditorEvent);
    }

    scroll_to(Id::new("diff-scroll"), AbsoluteOffset {
        x: 0.0,
        y: state::compute_hunk_offset(diff, next),
    })
}

fn navigate_history_commit_diff_popup_hunk(state: &mut AppState, delta: isize) -> Task<Message> {
    let Some(popup) = state.history_commit_diff_popup.as_mut() else {
        return Task::none();
    };

    if popup.diff_presentation == DiffPresentation::Split {
        let Some(model) = popup.editor_diff.as_ref() else {
            return Task::none();
        };
        let total_hunks = model.hunks.len();
        if total_hunks == 0 {
            return Task::none();
        }

        let current = popup.selected_hunk_index.unwrap_or(0) as isize;
        let next = (current + delta).clamp(0, (total_hunks - 1) as isize) as usize;
        popup.selected_hunk_index = Some(next);

        if let Some(editor) = popup.split_diff_editor.as_mut() {
            return editor.scroll_to_hunk(next).map(Message::SplitDiffEditorEvent);
        }

        return Task::none();
    }

    let total_hunks: usize = popup.diff.files.iter().map(|file| file.hunks.len()).sum();
    if total_hunks == 0 {
        return Task::none();
    }

    let current = popup.selected_hunk_index.unwrap_or(0) as isize;
    let next = (current + delta).clamp(0, (total_hunks - 1) as isize) as usize;
    popup.selected_hunk_index = Some(next);

    if let Some(editor) = popup.unified_diff_editor.as_mut() {
        return editor.scroll_to_hunk(next).map(Message::UnifiedDiffEditorEvent);
    }

    Task::none()
}

fn report_async_failure(
    state: &mut AppState,
    title: impl Into<String>,
    detail: impl Into<String>,
    source: &'static str,
    operation: &str,
    i18n: &I18n,
) {
    let title = title.into();
    let detail = detail.into();
    let detail = state
        .recovery_hint_for_source(source, i18n)
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

fn remote_panel_hint(repo: &Repository, action_label: &str, i18n: &i18n::I18n) -> String {
    if let Some(upstream_ref) = repo.current_upstream_ref() {
        i18n.remote_hint_upstream_fmt
            .replace("{}", &repo.current_branch_display())
            .replacen("{}", &upstream_ref, 1)
            .replacen("{}", action_label, 1)
    } else if repo.current_branch().ok().flatten().is_some() {
        i18n.remote_hint_branch_fmt
            .replace("{}", &repo.current_branch_display())
            .replacen("{}", action_label, 1)
    } else {
        i18n.remote_hint_detached_fmt
            .replace("{}", action_label)
    }
}

fn view(state: &AppState) -> Element<'_, Message> {
    let i18n = i18n::locale(state.git_settings.language.as_deref());
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
        Message::ShowSettings,
    );
    let mut layered = main_window.view();

    if state.show_project_dropdown {
        let mut project_list = Column::new().spacing(0).width(Length::Fill);

        // "Open project" button
        project_list = project_list.push(
            Button::new(
                Row::new()
                    .spacing(8)
                    .align_y(Alignment::Center)
                    .push(Text::new("📂").size(12))
                    .push(Text::new(i18n.open_project).size(12)),
            )
            .style(theme::button_style(theme::ButtonTone::Ghost))
            .padding([6, 12])
            .width(Length::Fill)
            .on_press(Message::OpenRepository),
        );

        project_list = project_list
            .push(iced::widget::rule::horizontal(1).style(theme::separator_rule_style()));

        // Recent projects header
        if !state.project_history.is_empty() {
            project_list = project_list.push(
                Container::new(
                    Text::new(i18n.recent_projects)
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

        let dropdown =
            Container::new(widgets::scrollable::styled(project_list).height(Length::Shrink))
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

        layered = iced::widget::stack([layered, backdrop.into(), overlay.into()])
            .width(Length::Fill)
            .height(Length::Fill)
            .into();
    }

    if state.show_branch_dropdown {
        let dropdown = Container::new(
            branch_popup::view(&state.branch_popup, i18n).map(Message::BranchPopupMessage),
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

        layered = iced::widget::stack([layered, backdrop.into(), overlay.into()])
            .width(Length::Fill)
            .height(Length::Fill)
            .into();
    }

    if let Some(ref update) = state.available_update {
        let banner = Container::new(
            Row::new()
                .spacing(8)
                .align_y(Alignment::Center)
                .push(
                    Text::new(i18n.new_version_fmt.replace("{}", &update.latest_version))
                        .size(11)
                        .color(theme::darcula::TEXT_PRIMARY),
                )
                .push(
                    Button::new(Text::new(i18n.download_update).size(11))
                        .style(theme::button_style(theme::ButtonTone::Primary))
                        .padding([3, 10])
                        .on_press(Message::OpenUpdateUrl),
                )
                .push(
                    Button::new(Text::new(i18n.ignore_update).size(11))
                        .style(theme::button_style(theme::ButtonTone::Ghost))
                        .padding([3, 6])
                        .on_press(Message::DismissUpdate),
                ),
        )
        .padding([6, 12])
        .style(|_: &Theme| iced::widget::container::Style {
            background: Some(Background::Color(theme::darcula::BG_PANEL)),
            border: Border {
                color: theme::darcula::ACCENT,
                width: 1.0,
                radius: 6.0.into(),
            },
            shadow: iced::Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.3),
                offset: iced::Vector::new(0.0, 2.0),
                blur_radius: 8.0,
            },
            ..Default::default()
        });

        let positioned = Container::new(banner)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Alignment::End)
            .align_y(Alignment::End)
            .padding([32, 16]);

        layered = iced::widget::stack([layered, positioned.into()]).into();
    }

    let layered = wrap_with_history_commit_diff_popup(state, i18n, layered);
    wrap_with_pending_commit_action_dialog(state, layered)
}

fn wrap_with_pending_commit_action_dialog<'a>(
    state: &'a AppState,
    base: Element<'a, Message>,
) -> Element<'a, Message> {
    use crate::views::branch_popup;

    let i18n = i18n::locale(state.git_settings.language.as_deref());
    let Some(dialog) = branch_popup::build_pending_commit_action_dialog(
        state.pending_commit_action.as_ref(),
        state.branch_popup.is_loading,
        i18n,
    ) else {
        return base;
    };

    stack![
        base,
        opaque(
            mouse_area(
                Container::new(dialog.map(Message::BranchPopupMessage))
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center_x(Length::Fill)
                    .center_y(Length::Fill)
                    .style(|_: &Theme| iced::widget::container::Style {
                        background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.5,))),
                        ..Default::default()
                    }),
            )
            .on_press(Message::BranchPopupMessage(
                branch_popup::BranchPopupMessage::CancelPendingCommitAction,
            )),
        )
    ]
    .into()
}

fn wrap_with_history_commit_diff_popup<'a>(
    state: &'a AppState,
    i18n: &'a i18n::I18n,
    base: Element<'a, Message>,
) -> Element<'a, Message> {
    let Some(popup) = state.history_commit_diff_popup.as_ref() else {
        return base;
    };

    let surface = history_diff_popup_surface(popup);
    let commit_label = popup.commit_id.chars().take(7).collect::<String>();
    let title_bar = Container::new(
        Row::new()
            .spacing(theme::spacing::SM)
            .align_y(Alignment::Center)
            .push(
                Column::new()
                    .spacing(2)
                    .width(Length::Fill)
                    .push(Text::new(i18n.commit_file_diff).size(13))
                    .push(
                        Text::new(&popup.file_path)
                            .size(10)
                            .color(theme::darcula::TEXT_SECONDARY),
                    ),
            )
            .push(widgets::compact_chip::<Message>(
                i18n.commit_label_fmt.replace("{}", &commit_label),
                BadgeTone::Neutral,
            ))
            .push(button::compact_ghost(
                i18n.close,
                Some(Message::CloseHistoryCommitDiffPopup),
            )),
    )
    .padding(theme::density::SECONDARY_BAR_PADDING)
    .style(theme::frame_style(theme::Surface::Toolbar));

    let popup_card = Container::new(
        Column::new()
            .spacing(0)
            .height(Length::Fill)
            .push(title_bar)
            .push(iced::widget::rule::horizontal(1))
            .push(build_read_only_diff_header(&surface, i18n))
            .push(
                Container::new(build_read_only_diff_content(
                    &surface,
                    i18n,
                    DiffSurfaceHunkActions {
                        stage: None,
                        unstage: None,
                    },
                ))
                .height(Length::Fill),
            ),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .max_width(980.0)
    .max_height(680.0)
    .style(theme::panel_style(theme::Surface::Panel));

    stack![
        base,
        opaque(
            mouse_area(
                Container::new(Space::new())
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .style(|_: &Theme| iced::widget::container::Style {
                        background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.5))),
                        ..Default::default()
                    }),
            )
            .on_press(Message::CloseHistoryCommitDiffPopup),
        ),
        Container::new(popup_card)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .padding([32, 40]),
    ]
    .into()
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
            AuxiliaryView::Branches => build_changes_body(state, i18n),
            AuxiliaryView::Remotes => {
                remote_dialog::view(&state.remote_dialog, i18n).map(Message::RemoteDialogMessage)
            }
            AuxiliaryView::Tags => {
                tag_dialog::view(&state.tag_dialog, i18n).map(Message::TagDialogMessage)
            }
            AuxiliaryView::Stashes => {
                stash_panel::view(&state.stash_panel, i18n).map(Message::StashPanelMessage)
            }
            AuxiliaryView::Rebase => {
                rebase_editor::view(&state.rebase_editor, i18n).map(Message::RebaseEditorMessage)
            }
            AuxiliaryView::Worktrees => {
                views::worktree_view::view(&state.worktree_state, i18n).map(Message::WorktreeMessage)
            }
            AuxiliaryView::Settings => {
                views::settings_view::view(&state.git_settings, i18n).map(Message::SettingsMessage)
            }
            AuxiliaryView::Commit => build_changes_body(state, i18n),
            AuxiliaryView::History => build_log_body(state, i18n),
        };
    }

    match state.shell.active_section {
        ShellSection::Changes => match state.shell.git_tool_window_tab {
            GitToolWindowTab::Changes => build_changes_body(state, i18n),
            GitToolWindowTab::Log => build_log_body(state, i18n),
        },
        ShellSection::Conflicts => build_conflict_body(state, i18n),
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

fn build_log_body<'a>(state: &'a AppState, i18n: &'a i18n::I18n) -> Element<'a, Message> {
    history_view::view_with_tabs(
        &state.history_view,
        &state.log_tabs,
        state.active_log_tab,
        &state.branch_popup.local_branches,
        &state.branch_popup.remote_branches,
        state.log_branches_dashboard_visible,
        i18n,
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
            build_change_context_menu_overlay(state, i18n),
        ])
        .width(Length::Fill)
        .height(Length::Fill),
    )
    .width(Length::FillPortion(5))
    .height(Length::Fill);

    let diff_panel = Container::new(
        Column::new()
            .spacing(0)
            .push(build_diff_header(state, i18n))
            .push(
                Container::new(build_diff_content(state, i18n))
                    .padding([0, 0])
                    .height(Length::Fill),
            ),
    )
    .height(Length::Fill)
    .style(theme::panel_style(theme::Surface::Panel));

    let commit_panel = commit_panel::view(
        &state.commit_dialog,
        &state.recent_commit_messages,
        state.git_settings.llm_enabled,
        i18n,
    )
    .map(Message::CommitDialogMessage);

    let right_panel = Column::new()
        .spacing(0)
        .height(Length::Fill)
        .push(diff_panel.height(Length::FillPortion(10)))
        .push(iced::widget::rule::horizontal(1))
        .push(Container::new(commit_panel).height(Length::FillPortion(2)));

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
                        .push(button::ghost(i18n.branches_btn, Some(Message::ShowBranches)))
                        .push(button::ghost(i18n.history_btn, Some(Message::ShowHistory))),
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

fn build_change_context_menu_overlay<'a>(state: &'a AppState, i18n: &'a i18n::I18n) -> Element<'a, Message> {
    let Some(path) = state.change_context_menu_path.as_deref() else {
        return Space::new().width(Length::Shrink).into();
    };
    let anchor = state
        .change_context_menu_anchor
        .unwrap_or(state.change_context_menu_cursor);

    let is_staged = state.staged_changes.iter().any(|c| c.path == path);
    let _is_unstaged = state.unstaged_changes.iter().any(|c| c.path == path);

    let stage_label = if is_staged { i18n.unstage_file } else { i18n.stage_file };
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
            i18n.show_diff_ctx,
            String::new(),
            show_diff_enabled.then_some(Message::SelectChange(path.to_string())),
            widgets::menu::MenuTone::Neutral,
        ))
        .push(change_context_action_row(
            i18n.discard_changes_ctx,
            String::new(),
            Some(Message::RevertFile(path.to_string())),
            widgets::menu::MenuTone::Danger,
        ))
        .push(change_context_action_row(
            i18n.show_history_ctx,
            String::new(),
            Some(Message::ShowFileHistory(path.to_string())),
            widgets::menu::MenuTone::Neutral,
        ))
        .push(change_context_action_row(
            i18n.copy_path_ctx,
            String::new(),
            Some(Message::CopyChangePath(path.to_string())),
            widgets::menu::MenuTone::Neutral,
        ))
        .push(change_context_action_row(
            i18n.open_in_editor_ctx,
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

#[derive(Clone, Copy)]
struct DiffSurfaceHunkActions {
    stage: Option<fn(String, usize) -> Message>,
    unstage: Option<fn(String, usize) -> Message>,
}

struct ReadOnlyDiffSurface<'a> {
    full_file_preview_binary: bool,
    full_file_preview: Option<&'a git_core::diff::FileDiff>,
    show_diff: bool,
    diff: Option<&'a git_core::diff::Diff>,
    diff_presentation: DiffPresentation,
    selected_path: Option<&'a str>,
    selected_hunk_index: Option<usize>,
    editor_diff: Option<&'a git_core::diff::EditorDiffModel>,
    split_diff_editor: Option<&'a widgets::diff_editor::SplitDiffEditorState>,
    unified_diff_editor: Option<&'a widgets::diff_editor::UnifiedDiffEditorState>,
    supports_split_diff: bool,
}

fn build_diff_header<'a>(state: &'a AppState, i18n: &'a i18n::I18n) -> Element<'a, Message> {
    let (surface, _) = workspace_diff_surface(state);
    build_read_only_diff_header(&surface, i18n)
}

fn build_diff_content<'a>(state: &'a AppState, i18n: &'a i18n::I18n) -> Element<'a, Message> {
    let (surface, hunk_actions) = workspace_diff_surface(state);
    build_read_only_diff_content(&surface, i18n, hunk_actions)
}

fn workspace_diff_surface<'a>(state: &'a AppState) -> (ReadOnlyDiffSurface<'a>, DiffSurfaceHunkActions) {
    let selected_is_staged = state
        .selected_change_path
        .as_ref()
        .map(|path| state.staged_changes.iter().any(|change| &change.path == path))
        .unwrap_or(false);
    let hunk_actions = if selected_is_staged {
        DiffSurfaceHunkActions {
            stage: None,
            unstage: Some(Message::UnstageHunk),
        }
    } else {
        DiffSurfaceHunkActions {
            stage: Some(Message::StageHunk),
            unstage: None,
        }
    };

    (
        ReadOnlyDiffSurface {
            full_file_preview_binary: state.full_file_preview_binary,
            full_file_preview: state.full_file_preview.as_ref(),
            show_diff: state.show_diff,
            diff: state.current_diff.as_ref(),
            diff_presentation: state.diff_presentation,
            selected_path: state.selected_change_path.as_deref(),
            selected_hunk_index: state.selected_hunk_index,
            editor_diff: state.editor_diff.as_ref(),
            split_diff_editor: state.split_diff_editor.as_ref(),
            unified_diff_editor: state.unified_diff_editor.as_ref(),
            supports_split_diff: !matches!(
                state.diff_source,
                state::DiffSource::HistoryCommit { .. }
            ),
        },
        hunk_actions,
    )
}

fn history_diff_popup_surface<'a>(
    popup: &'a state::HistoryCommitDiffPopupState,
) -> ReadOnlyDiffSurface<'a> {
    ReadOnlyDiffSurface {
        full_file_preview_binary: false,
        full_file_preview: None,
        show_diff: true,
        diff: Some(&popup.diff),
        diff_presentation: popup.diff_presentation,
        selected_path: Some(popup.file_path.as_str()),
        selected_hunk_index: popup.selected_hunk_index,
        editor_diff: popup.editor_diff.as_ref(),
        split_diff_editor: popup.split_diff_editor.as_ref(),
        unified_diff_editor: popup.unified_diff_editor.as_ref(),
        supports_split_diff: popup.supports_split_diff(),
    }
}

fn build_read_only_diff_header<'a>(surface: &ReadOnlyDiffSurface<'a>, i18n: &'a i18n::I18n) -> Element<'a, Message> {
    let file_name = surface
        .selected_path
        .and_then(|path| std::path::Path::new(path).file_name()?.to_str())
        .unwrap_or(i18n.diff);

    let path_hint = surface.selected_path.and_then(|path| {
        std::path::Path::new(path)
            .parent()
            .and_then(|p| p.to_str())
            .filter(|p| !p.is_empty())
    });

    let summary = surface
        .diff
        .and_then(|diff| diff.files.first().map(|f| (f.additions, f.deletions)));

    let file_position = surface.diff.and_then(|diff| {
        (diff.files.len() > 1).then(|| {
            surface.selected_path.and_then(|selected| {
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
                i18n.unified_view,
                surface.diff_presentation == DiffPresentation::Unified,
                (surface.diff.is_some() && surface.supports_split_diff)
                    .then_some(Message::ToggleDiffPresentation),
            ))
            .push(button::tab(
                i18n.split_view,
                surface.diff_presentation == DiffPresentation::Split,
                (surface.diff.is_some() && surface.supports_split_diff)
                    .then_some(Message::ToggleDiffPresentation),
            ))
            .push_maybe(surface.diff.and_then(|diff| {
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

fn build_read_only_diff_content<'a>(
    surface: &ReadOnlyDiffSurface<'a>,
    i18n: &'a i18n::I18n,
    hunk_actions: DiffSurfaceHunkActions,
) -> Element<'a, Message> {
    if surface.full_file_preview_binary {
        return widgets::panel_empty_state_compact(
            i18n.binary_file_no_preview,
            surface.selected_path.unwrap_or(""),
        );
    }

    if let Some(preview_diff) = surface.full_file_preview {
        return widgets::diff_viewer::file_preview(preview_diff);
    }

    if !surface.show_diff || surface.diff.is_none() {
        return widgets::panel_empty_state_compact(i18n.diff_empty, i18n.diff_empty_detail);
    }

    let diff = surface.diff.expect("diff checked");
    if diff.files.is_empty() {
        return widgets::panel_empty_state_compact(i18n.no_changes, i18n.diff_empty_detail);
    }

    match surface.diff_presentation {
        DiffPresentation::Unified => {
            if let Some(editor) = surface.unified_diff_editor {
                editor.view().map(Message::UnifiedDiffEditorEvent)
            } else {
                let mut viewer = widgets::diff_viewer::DiffViewer::new(diff);
                if let Some(handler) = hunk_actions.stage {
                    viewer = viewer.with_stage_hunk_handler(handler);
                }
                if let Some(handler) = hunk_actions.unstage {
                    viewer = viewer.with_unstage_hunk_handler(handler);
                }
                viewer.view()
            }
        }
        DiffPresentation::Split => {
            let (Some(model), Some(editor)) = (surface.editor_diff, surface.split_diff_editor) else {
                return widgets::panel_empty_state_compact(
                    i18n.split_view_unavailable,
                    i18n.split_view_unavailable_detail,
                );
            };

            widgets::split_diff_viewer::view(
                model,
                editor,
                surface.selected_hunk_index,
                Message::SplitDiffEditorEvent,
                hunk_actions.stage,
                hunk_actions.unstage,
            )
        }
    }
}

fn build_conflict_body<'a>(state: &'a AppState, i18n: &'a i18n::I18n) -> Element<'a, Message> {
    if state.conflict_files.is_empty() {
        return views::render_empty_state(
            i18n.resolve_conflict_header,
            i18n.conflict_no_files,
            i18n.conflict_no_files_detail,
            Some(button::secondary(i18n.back_to_changes, Some(Message::ShowChanges)).into()),
        );
    }

    if state.conflict_merge_index.is_some() {
        // Prefer Meld-style 3-column merge editor
        if let Some(editor) = state.merge_editor.as_ref() {
            return Container::new(editor.view(i18n).map(Message::MergeEditorMessage))
                .height(Length::Fill)
                .style(theme::panel_style(theme::Surface::Editor))
                .into();
        }
        // Fallback to old conflict resolver
        return if let Some(resolver) = state.conflict_resolver.as_ref() {
            Container::new(resolver.view().map(Message::ConflictResolverMessage))
                .height(Length::Fill)
                .style(theme::panel_style(theme::Surface::Editor))
                .into()
        } else {
            widgets::panel_empty_state(
                i18n.resolve_conflict_header,
                i18n.conflict_merge_unavailable,
                i18n.conflict_merge_unavailable_detail,
                Some(button::secondary(i18n.back_to_list, Some(Message::ShowConflicts)).into()),
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
                i18n.conflict_files_count_fmt.replace("{}", &state.conflict_files.len().to_string()),
                BadgeTone::Warning,
            ))
            .push(widgets::info_chip::<Message>(
                i18n.conflict_hunks_fmt.replace("{}", &total_hunks.to_string()),
                BadgeTone::Neutral,
            ))
            .push(widgets::info_chip::<Message>(
                i18n.manual_merge_fmt.replace("{}", &total_manual_conflicts.to_string()),
                if total_manual_conflicts > 0 {
                    BadgeTone::Warning
                } else {
                    BadgeTone::Success
                },
            ))
            .push(widgets::info_chip::<Message>(
                i18n.auto_resolvable_fmt.replace("{}", &total_auto_resolvable.to_string()),
                BadgeTone::Accent,
            ))
            .push(Space::new().width(Length::Fill))
            .push(
                Text::new(i18n.conflict_list_hint)
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
                Text::new(i18n.name_column)
                    .size(10)
                    .width(Length::FillPortion(6))
                    .color(theme::darcula::TEXT_SECONDARY),
            )
            .push(
                Text::new(i18n.your_changes_column)
                    .size(10)
                    .width(Length::FillPortion(2))
                    .color(theme::darcula::TEXT_SECONDARY),
            )
            .push(
                Text::new(i18n.their_changes_column)
                    .size(10)
                    .width(Length::FillPortion(2))
                    .color(theme::darcula::TEXT_SECONDARY),
            )
            .push(
                Text::new(i18n.status_column)
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
                i18n,
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
                .map(|conflict| build_conflict_action_panel(index, conflict, i18n))
        })
        .unwrap_or_else(|| {
            widgets::panel_empty_state(
                i18n.resolve_conflict_header,
                i18n.conflict_no_selection,
                i18n.conflict_no_selection_detail,
                None,
            )
        });

    let footer = Row::new()
        .spacing(theme::spacing::XS)
        .align_y(Alignment::Center)
        .push(button::ghost(
            i18n.back_changes_btn,
            Some(Message::CloseConflictResolver),
        ))
        .push(button::secondary(
            i18n.refresh,
            Some(Message::ConflictResolverMessage(
                ConflictResolverMessage::Refresh,
            )),
        ))
        .push(Space::new().width(Length::Fill))
        .push(
            Text::new(i18n.conflict_footer_hint)
                .size(10)
                .color(theme::darcula::TEXT_SECONDARY),
        );

    Container::new(
        Column::new()
            .spacing(theme::spacing::MD)
            .height(Length::Fill)
            .push(widgets::section_header(
                i18n.resolve_conflict_header,
                i18n.resolve_conflict_title,
                i18n.resolve_conflict_detail,
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
    i18n: &'a i18n::I18n,
) -> Element<'a, Message> {
    let summary = summarize_conflict(conflict);
    let file_status = FileStatus::Conflict;
    let (file_name, parent_path) = split_workspace_path(&conflict.path, i18n);
    let status_label = if summary.manual_conflicts > 0 {
        i18n.needs_merge_fmt.replace("{}", &summary.manual_conflicts.to_string())
    } else {
        i18n.auto_resolvable_label.to_string()
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
                                    widgets::info_chip::<Message>(i18n.current_label, BadgeTone::Accent)
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
            i18n.current_branch_label,
            if summary.ours_changed > 0 {
                i18n.modified_hunks_fmt.replace("{}", &summary.ours_changed.to_string())
            } else {
                i18n.no_diff_label.to_string()
            },
            BadgeTone::Accent,
        ))
        .push(build_conflict_status_cell(
            i18n.incoming_branch_label,
            if summary.theirs_changed > 0 {
                i18n.modified_hunks_fmt.replace("{}", &summary.theirs_changed.to_string())
            } else {
                i18n.no_diff_label.to_string()
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
                    Text::new(i18n.conflict_hunks_count_fmt.replace("{}", &summary.hunk_count.to_string()))
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
    i18n: &'a i18n::I18n,
) -> Element<'a, Message> {
    let summary = summarize_conflict(conflict);
    let (file_name, parent_path) = split_workspace_path(&conflict.path, i18n);

    // IDEA-style: compact file info + toolbar action buttons
    Container::new(
        Column::new()
            .spacing(theme::spacing::MD)
            // ── File info ──
            .push(
                Column::new()
                    .spacing(4)
                    .push(
                        Row::new()
                            .spacing(theme::spacing::XS)
                            .align_y(Alignment::Center)
                            .push(Text::new("U").size(10).color(theme::darcula::DANGER))
                            .push(Text::new(file_name).size(14).color(theme::darcula::DANGER)),
                    )
                    .push(
                        Text::new(parent_path)
                            .size(10)
                            .color(theme::darcula::TEXT_SECONDARY)
                            .wrapping(text::Wrapping::WordOrGlyph),
                    ),
            )
            // ── Conflict stats (inline chips) ──
            .push(
                Row::new()
                    .spacing(theme::spacing::XS)
                    .push(widgets::info_chip::<Message>(
                        i18n.conflict_count_fmt.replace("{}", &summary.hunk_count.to_string()),
                        BadgeTone::Warning,
                    ))
                    .push(widgets::info_chip::<Message>(
                        i18n.manual_count_fmt.replace("{}", &summary.manual_conflicts.to_string()),
                        if summary.manual_conflicts > 0 {
                            BadgeTone::Danger
                        } else {
                            BadgeTone::Success
                        },
                    ))
                    .push(widgets::info_chip::<Message>(
                        i18n.auto_count_fmt.replace("{}", &summary.auto_resolvable.to_string()),
                        BadgeTone::Neutral,
                    )),
            )
            // ── Branch stat row (compact) ──
            .push(
                Row::new()
                    .spacing(theme::spacing::SM)
                    .push(build_conflict_stat_card(
                        i18n.current_branch_label,
                        summary.ours_changed.to_string(),
                        i18n.changed_hunks_label,
                        BadgeTone::Accent,
                    ))
                    .push(build_conflict_stat_card(
                        i18n.incoming_branch_label,
                        summary.theirs_changed.to_string(),
                        i18n.changed_hunks_label,
                        BadgeTone::Danger,
                    )),
            )
            .push(iced::widget::rule::horizontal(1))
            // ── Action buttons: IDEA-style compact row ──
            .push(
                Column::new()
                    .spacing(theme::spacing::XS)
                    .push(
                        button::primary(i18n.merge_btn, Some(Message::OpenConflictMerge(index)))
                            .width(Length::Fill),
                    )
                    .push(
                        Row::new()
                            .spacing(theme::spacing::XS)
                            .push(
                                button::secondary(
                                    i18n.accept_yours_btn,
                                    Some(Message::ResolveConflictWithOurs(index)),
                                )
                                .width(Length::FillPortion(1)),
                            )
                            .push(
                                button::secondary(
                                    i18n.accept_theirs_btn,
                                    Some(Message::ResolveConflictWithTheirs(index)),
                                )
                                .width(Length::FillPortion(1)),
                            ),
                    ),
            )
            .push(
                Text::new(i18n.conflict_action_hint)
                    .size(10)
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

fn split_workspace_path<'a>(path: &str, i18n: &'a i18n::I18n) -> (String, String) {
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
        .unwrap_or_else(|| i18n.repo_root.to_string());

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

#[derive(Debug, Clone)]
pub enum Message {
    OpenRepository,
    InitRepository,
    Refresh,
    AutoRefreshTick(Instant),
    RepositoryWatchEvent(RepositoryWatchEvent),
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
    CloseHistoryCommitDiffPopup,
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
    UnifiedDiffEditorEvent(widgets::diff_editor::UnifiedDiffEditorEvent),
    SplitDiffEditorEvent(DiffEditorEvent),
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
    ShowSettings,
    SettingsMessage(views::settings_view::SettingsMessage),
    MergeEditorMessage(widgets::merge_editor::MergeEditorEvent),
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
    SetUpstreamAndPush {
        branch: String,
        remote: String,
    },
    UpdateCheckResult(Option<git_core::updater::UpdateInfo>),
    OpenUpdateUrl,
    DismissUpdate,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::EN;
    use crate::state::ViewMode;
    use git_core::diff::{Diff, DiffHunk, DiffLine, DiffLineOrigin, FileDiff};

    fn sample_history_file_diff(path: &str) -> Diff {
        Diff {
            files: vec![FileDiff {
                old_path: Some(path.to_string()),
                new_path: Some(path.to_string()),
                hunks: vec![DiffHunk {
                    header: "@@ -1 +1 @@".to_string(),
                    lines: vec![
                        DiffLine {
                            content: "old line\n".to_string(),
                            origin: DiffLineOrigin::Deletion,
                            old_lineno: Some(1),
                            new_lineno: None,
                            inline_changes: Vec::new(),
                        },
                        DiffLine {
                            content: "new line\n".to_string(),
                            origin: DiffLineOrigin::Addition,
                            old_lineno: None,
                            new_lineno: Some(1),
                            inline_changes: Vec::new(),
                        },
                    ],
                    old_start: 1,
                    old_lines: 1,
                    new_start: 1,
                    new_lines: 1,
                }],
                additions: 1,
                deletions: 1,
            }],
            total_additions: 1,
            total_deletions: 1,
        }
    }

    fn sample_history_editor_diff(path: &str) -> git_core::diff::EditorDiffModel {
        let diff = sample_history_file_diff(path);
        git_core::diff::build_editor_diff_model_from_file_contents(
            diff.files.first().expect("file diff"),
            b"old line\n",
            b"new line\n",
        )
        .expect("editor diff")
    }

    #[test]
    fn open_commit_dialog_navigates_to_changes_tab() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let repo =
            git_core::Repository::init(temp_dir.path()).expect("repository should initialize");
        let mut state = AppState::new();
        state.set_repository(repo, &EN);
        state.switch_git_tool_window_tab(GitToolWindowTab::Log, &EN);
        open_commit_dialog(&mut state).expect("should open commit dialog");
        assert_eq!(state.shell.git_tool_window_tab, GitToolWindowTab::Changes);
        assert_eq!(state.view_mode, ViewMode::Repository);
    }

    #[test]
    fn show_history_commit_file_diff_keeps_log_context() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let repo =
            git_core::Repository::init(temp_dir.path()).expect("repository should initialize");
        let mut state = AppState::new();
        state.set_repository(repo, &EN);
        state.switch_git_tool_window_tab(GitToolWindowTab::Log, &EN);
        state.selected_change_path = Some("workspace-file.rs".to_string());

        show_history_commit_file_diff(
            &mut state,
            "abc123".to_string(),
            "src/main.rs".to_string(),
            sample_history_file_diff("src/main.rs"),
            None,
        );

        assert_eq!(state.shell.git_tool_window_tab, GitToolWindowTab::Log);
        assert_eq!(state.selected_change_path.as_deref(), Some("workspace-file.rs"));
        assert!(state.current_diff.is_none());
        assert_eq!(
            state.history_view.selected_commit_file_path.as_deref(),
            Some("src/main.rs")
        );
        assert!(state.history_commit_diff_popup.is_some());
    }

    #[test]
    fn closing_history_commit_diff_popup_preserves_history_selection() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let repo =
            git_core::Repository::init(temp_dir.path()).expect("repository should initialize");
        let mut state = AppState::new();
        state.set_repository(repo, &EN);
        state.switch_git_tool_window_tab(GitToolWindowTab::Log, &EN);
        state.history_view.selected_commit = Some("abc123".to_string());

        show_history_commit_file_diff(
            &mut state,
            "abc123".to_string(),
            "src/main.rs".to_string(),
            sample_history_file_diff("src/main.rs"),
            None,
        );
        let _ = update(&mut state, Message::CloseHistoryCommitDiffPopup);

        assert_eq!(state.shell.git_tool_window_tab, GitToolWindowTab::Log);
        assert_eq!(state.history_view.selected_commit.as_deref(), Some("abc123"));
        assert_eq!(
            state.history_view.selected_commit_file_path.as_deref(),
            Some("src/main.rs")
        );
        assert!(state.history_commit_diff_popup.is_none());
    }

    #[test]
    fn toggling_presentation_with_history_popup_keeps_log_context() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let repo =
            git_core::Repository::init(temp_dir.path()).expect("repository should initialize");
        let mut state = AppState::new();
        state.set_repository(repo, &EN);
        state.switch_git_tool_window_tab(GitToolWindowTab::Log, &EN);

        show_history_commit_file_diff(
            &mut state,
            "abc123".to_string(),
            "src/main.rs".to_string(),
            sample_history_file_diff("src/main.rs"),
            Some(sample_history_editor_diff("src/main.rs")),
        );
        let _ = update(&mut state, Message::ToggleDiffPresentation);

        let popup = state
            .history_commit_diff_popup
            .as_ref()
            .expect("history diff popup");
        assert_eq!(state.shell.git_tool_window_tab, GitToolWindowTab::Log);
        assert_eq!(popup.diff_presentation, DiffPresentation::Split);
    }
}
