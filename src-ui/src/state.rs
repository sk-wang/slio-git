//! Application state management

use crate::i18n::I18n;
use crate::theme;
use crate::views::{
    branch_popup::{BranchPopupState, CommitActionConfirmation},
    commit_dialog::CommitDialogState,
    history_view::HistoryState,
    rebase_editor::RebaseEditorState,
    remote_dialog::RemoteDialogState,
    stash_panel::StashPanelState,
    tag_dialog::TagDialogState,
};
use crate::widgets::conflict_resolver::ConflictResolver;
use crate::widgets::diff_editor::{SplitDiffEditorState, UnifiedDiffEditorState};
use git_core::diff::{AutoMergeResult, Diff, EditorDiffModel, ThreeWayDiff};
use git_core::index::Change;
use git_core::remote::RemoteInfo;
use git_core::repository::{Repository, SyncStatus};
use iced::Point;
use log::warn;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// View modes for the main application body.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Welcome,
    Repository,
    ConflictResolver,
}

/// File display mode for the change list
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum FileDisplayMode {
    /// Flat list showing filenames with relative paths
    #[default]
    Flat,
    /// Directory tree grouped by folder hierarchy
    Tree,
}

/// State for a single tab in the multi-tab log view
#[derive(Debug, Clone)]
pub struct LogTab {
    /// Unique tab identifier
    pub id: usize,
    /// Display label (branch name or the "All" tab label)
    pub label: String,
    /// Whether this tab can be closed (false for "All" tab)
    pub is_closable: bool,
    /// Branch to filter by (None = all branches)
    pub branch_filter: Option<String>,
    /// Text search filter
    pub text_filter: String,
    /// Author name filter
    pub author_filter: Option<String>,
    /// Date range filter (start, end) as Unix timestamps
    pub date_range: Option<(i64, i64)>,
    /// File path filter
    pub path_filter: Option<String>,
    /// Vertical scroll position
    pub scroll_offset: f32,
    /// Currently selected commit hash
    pub selected_commit: Option<String>,
}

impl LogTab {
    /// Create the default "All" tab
    pub fn all_with_i18n(i18n: &I18n) -> Self {
        Self {
            id: 0,
            label: i18n.log_tab_all.to_string(),
            is_closable: false,
            branch_filter: None,
            text_filter: String::new(),
            author_filter: None,
            date_range: None,
            path_filter: None,
            scroll_offset: 0.0,
            selected_commit: None,
        }
    }

    /// Create the default "All" tab (fallback without i18n)
    pub fn all() -> Self {
        Self {
            id: 0,
            label: "All".to_string(),
            is_closable: false,
            branch_filter: None,
            text_filter: String::new(),
            author_filter: None,
            date_range: None,
            path_filter: None,
            scroll_offset: 0.0,
            selected_commit: None,
        }
    }

    /// Create a branch-pinned tab
    pub fn for_branch(id: usize, branch: String) -> Self {
        Self {
            id,
            label: branch.clone(),
            is_closable: true,
            branch_filter: Some(branch),
            text_filter: String::new(),
            author_filter: None,
            date_range: None,
            path_filter: None,
            scroll_offset: 0.0,
            selected_commit: None,
        }
    }
}

impl Default for LogTab {
    fn default() -> Self {
        Self::all()
    }
}

/// Diff presentation mode inside the changes workspace.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffPresentation {
    Unified,
    Split,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffSource {
    Workspace,
    HistoryCommit { commit_id: String },
}

#[derive(Debug, Clone)]
pub struct HistoryCommitDiffPopupState {
    pub commit_id: String,
    pub file_path: String,
    pub diff: Diff,
    pub diff_presentation: DiffPresentation,
    pub selected_hunk_index: Option<usize>,
    pub editor_diff: Option<EditorDiffModel>,
    pub split_diff_editor: Option<SplitDiffEditorState>,
    pub unified_diff_editor: Option<UnifiedDiffEditorState>,
}

impl HistoryCommitDiffPopupState {
    pub fn new(
        commit_id: String,
        file_path: String,
        diff: Diff,
        editor_diff: Option<EditorDiffModel>,
        font_size: f32,
    ) -> Self {
        let split_diff_editor = editor_diff
            .clone()
            .map(|model| SplitDiffEditorState::with_font_size(model, font_size));
        let unified_diff_editor = (!diff.files.is_empty())
            .then(|| UnifiedDiffEditorState::from_diff(&diff, font_size));
        let selected_hunk_index = diff
            .files
            .iter()
            .map(|file| file.hunks.len())
            .sum::<usize>()
            .checked_sub(1)
            .map(|_| 0);

        Self {
            commit_id,
            file_path,
            diff,
            diff_presentation: DiffPresentation::Unified,
            selected_hunk_index,
            editor_diff,
            split_diff_editor,
            unified_diff_editor,
        }
    }

    pub fn supports_split_diff(&self) -> bool {
        self.editor_diff.is_some() && self.split_diff_editor.is_some()
    }
}

const MAX_PROJECT_HISTORY: usize = 8;
const WORKSPACE_MEMORY_FILE: &str = "workspace-memory-v1.txt";
const AUTO_REMOTE_CHECK_INTERVAL: Duration = Duration::from_secs(90);
const TOAST_NOTIFICATION_DURATION: Duration = Duration::from_secs(4);

/// Session-scoped project entries rendered in the left rail for quick switching.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectEntry {
    pub name: String,
    pub path: PathBuf,
}

impl ProjectEntry {
    fn from_repository(repo: &Repository) -> Self {
        Self {
            name: repo.name(),
            path: repo.path().to_path_buf(),
        }
    }

    fn from_path(path: PathBuf) -> Self {
        let name = path
            .file_name()
            .and_then(|segment| segment.to_str())
            .filter(|segment| !segment.is_empty())
            .map(str::to_string)
            .unwrap_or_else(|| path.display().to_string());

        Self { name, path }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct PersistedWorkspaceMemory {
    last_open_repository: Option<PathBuf>,
    recent_paths: Vec<PathBuf>,
}

impl PersistedWorkspaceMemory {
    fn load() -> Self {
        Self::load_from_path(&workspace_memory_path())
    }

    fn load_from_path(path: &Path) -> Self {
        let Ok(contents) = fs::read_to_string(path) else {
            return Self::default();
        };

        Self::parse(&contents)
    }

    fn parse(contents: &str) -> Self {
        let mut memory = Self::default();

        for line in contents.lines() {
            if let Some(path) = line.strip_prefix("last\t") {
                let path = path.trim();
                if !path.is_empty() {
                    memory.last_open_repository = Some(PathBuf::from(path));
                }
                continue;
            }

            if let Some(path) = line.strip_prefix("recent\t") {
                let path = path.trim();
                if !path.is_empty() {
                    memory.recent_paths.push(PathBuf::from(path));
                }
            }
        }

        memory.normalize();
        memory
    }

    fn save(&self) -> std::io::Result<()> {
        self.save_to_path(&workspace_memory_path())
    }

    fn save_to_path(&self, path: &Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(path, self.serialize())
    }

    fn serialize(&self) -> String {
        let mut lines = Vec::new();

        if let Some(path) = self.last_open_repository.as_ref() {
            lines.push(format!("last\t{}", path.display()));
        }

        for path in &self.recent_paths {
            lines.push(format!("recent\t{}", path.display()));
        }

        if lines.is_empty() {
            String::new()
        } else {
            format!("{}\n", lines.join("\n"))
        }
    }

    fn normalize(&mut self) {
        let mut normalized = Vec::new();

        if let Some(path) = self.last_open_repository.clone() {
            normalized.push(path);
        }

        for path in self.recent_paths.drain(..) {
            if normalized.iter().any(|existing| existing == &path) {
                continue;
            }
            normalized.push(path);
        }

        normalized.truncate(MAX_PROJECT_HISTORY);
        self.recent_paths = normalized;

        if let Some(last) = self.last_open_repository.as_ref() {
            if !self.recent_paths.iter().any(|path| path == last) {
                self.last_open_repository = None;
            }
        }
    }
}

fn workspace_memory_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("slio-git")
        .join(WORKSPACE_MEMORY_FILE)
}

/// High-level shell sections shown in the navigation rail.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellSection {
    Welcome,
    Changes,
    Conflicts,
}

/// Tabs inside the Git tool-window workspace (IDEA-style).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GitToolWindowTab {
    #[default]
    Changes,
    Log,
}

/// Auxiliary full-screen surfaces that temporarily replace the shell body.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuxiliaryView {
    Commit,
    Branches,
    History,
    Remotes,
    Tags,
    Stashes,
    Rebase,
    Worktrees,
    Settings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolbarRemoteAction {
    Pull,
    Push,
}

#[derive(Debug, Clone)]
pub struct ToolbarRemoteMenuState {
    pub action: ToolbarRemoteAction,
    pub remotes: Vec<RemoteInfo>,
    pub preferred_remote: Option<String>,
}

/// Shared feedback variants rendered in the shell banner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeedbackLevel {
    Info,
    Success,
    Warning,
    Error,
    Loading,
    Empty,
}

/// Shell banner state.
#[derive(Debug, Clone)]
pub struct FeedbackState {
    pub level: FeedbackLevel,
    pub title: String,
    pub detail: Option<String>,
    pub source: &'static str,
    pub compact: bool,
    pub sticky: bool,
}

#[derive(Debug, Clone)]
pub struct ToastNotificationState {
    pub level: FeedbackLevel,
    pub title: String,
    pub detail: Option<String>,
    pub expires_at: Instant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryAction {
    OpenRepository,
    InitRepository,
    Refresh,
    ShowChanges,
    ShowConflicts,
}

#[derive(Debug, Clone)]
pub struct FeedbackNextStep {
    pub title: String,
    pub detail: String,
    pub action: RecoveryAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ChromeElevation {
    #[default]
    Flat,
    Subtle,
    Emphasized,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OverflowBehavior {
    TruncateTail,
    #[default]
    HorizontalScroll,
    SecondaryLine,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StatusSeverity {
    #[default]
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StatusPersistence {
    #[default]
    Ephemeral,
    StickyUntilDismissed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StatusPlacement {
    Banner,
    Inline,
    #[default]
    StatusBar,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StatusEmphasis {
    #[default]
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Default)]
pub struct LightweightStatusSurface {
    pub message: Option<String>,
    pub detail: Option<String>,
    pub severity: StatusSeverity,
    pub persistence: StatusPersistence,
    pub placement: StatusPlacement,
    pub emphasis: StatusEmphasis,
}

/// Shell-level metadata used by the main window.
#[derive(Debug, Clone)]
pub struct AppShellState {
    pub active_section: ShellSection,
    pub git_tool_window_tab: GitToolWindowTab,
    pub title: String,
    pub subtitle: String,
    pub primary_action_label: String,
    pub context_switcher: WorkspaceContextSwitcher,
    pub chrome: PrimaryWorkspaceChrome,
    pub status_surface: LightweightStatusSurface,
}

#[derive(Debug, Clone, Default)]
pub struct WorkspaceContextStrip {
    pub repository_name: String,
    pub repository_path: String,
    pub branch_name: String,
    pub sync_label: String,
    pub sync_hint: Option<String>,
    pub state_hint: Option<String>,
    pub secondary_label: Option<String>,
    pub overflow_behavior: OverflowBehavior,
}

#[derive(Debug, Clone, Default)]
pub struct CompactChromeProfile {
    pub max_visible_top_bars: u8,
    pub toolbar_height: u16,
    pub control_height: u16,
    pub container_radius: u16,
    pub section_gap: u16,
    pub content_padding: u16,
    pub elevation: ChromeElevation,
    pub change_count: usize,
    pub conflict_count: usize,
    pub selected_path: Option<String>,
    pub has_selected_change: bool,
    pub has_staged_changes: bool,
    pub has_secondary_actions: bool,
    pub editor_tab_label: String,
    pub editor_tab_detail: Option<String>,
    pub tool_window_title: Option<String>,
}

pub type WorkspaceContextSwitcher = WorkspaceContextStrip;
pub type PrimaryWorkspaceChrome = CompactChromeProfile;

/// Navigation entry metadata for the sidebar.
#[derive(Debug, Clone)]
pub struct NavigationItem {
    pub section: ShellSection,
    pub badge: Option<String>,
    pub enabled: bool,
}

/// Lightweight defect hook used during the redesign sweep.
#[derive(Debug, Clone)]
pub struct DefectObservation {
    pub area: String,
    pub summary: String,
    pub verified: bool,
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub current_repository: Option<Repository>,
    pub is_loading: bool,
    pub error_message: Option<String>,
    pub view_mode: ViewMode,
    pub auxiliary_view: Option<AuxiliaryView>,
    pub project_history: Vec<ProjectEntry>,
    pub shell: AppShellState,
    pub feedback: Option<FeedbackState>,
    pub toast_notification: Option<ToastNotificationState>,
    pub defect_observations: Vec<DefectObservation>,
    pub staged_changes: Vec<Change>,
    pub unstaged_changes: Vec<Change>,
    pub untracked_files: Vec<Change>,
    pub conflicts_present: bool,
    pub conflict_files: Vec<ThreeWayDiff>,
    pub selected_conflict_index: Option<usize>,
    pub conflict_merge_index: Option<usize>,
    pub auto_merge_result: Option<AutoMergeResult>,
    pub conflict_resolver: Option<ConflictResolver>,
    pub show_diff: bool,
    pub current_diff: Option<Diff>,
    pub diff_presentation: DiffPresentation,
    pub diff_source: DiffSource,
    pub selected_change_path: Option<String>,
    pub selected_hunk_index: Option<usize>,
    pub change_context_menu_path: Option<String>,
    pub change_context_menu_cursor: Point,
    pub change_context_menu_anchor: Option<Point>,
    pub commit_dialog: CommitDialogState,
    pub branch_popup: BranchPopupState,
    /// Top-level pending commit action confirmation (reset/push/cherry-pick/revert)
    pub pending_commit_action: Option<CommitActionConfirmation>,
    pub history_view: HistoryState,
    pub history_commit_diff_popup: Option<HistoryCommitDiffPopupState>,
    pub remote_dialog: RemoteDialogState,
    pub tag_dialog: TagDialogState,
    pub stash_panel: StashPanelState,
    pub rebase_editor: RebaseEditorState,
    pub toolbar_remote_menu: Option<ToolbarRemoteMenuState>,
    /// Whether the project dropdown is visible
    pub show_project_dropdown: bool,
    /// Whether the branch dropdown popup is visible (IDEA-style floating panel)
    pub show_branch_dropdown: bool,
    /// Git settings
    pub git_settings: crate::views::settings_view::GitSettings,
    auto_refresh: AutoRefreshState,
    /// File display mode for the change list (flat vs tree)
    pub file_display_mode: FileDisplayMode,
    /// Multi-tab log state
    pub log_tabs: Vec<LogTab>,
    /// Active log tab index
    pub active_log_tab: usize,
    /// Next log tab ID counter
    pub next_log_tab_id: usize,
    /// Whether the branches dashboard sidebar is visible in the log view
    pub log_branches_dashboard_visible: bool,
    /// Whether blame/annotate gutter is active
    pub blame_active: bool,
    /// Recent commit messages for the current repository (last 10)
    pub recent_commit_messages: Vec<String>,
    /// Meld-style 3-column merge editor
    pub merge_editor: Option<crate::widgets::merge_editor::MergeEditorState>,
    /// Working tree management state
    pub worktree_state: crate::views::worktree_view::WorktreeState,
    /// Full file preview diff (for new/untracked files without diff)
    pub full_file_preview: Option<git_core::diff::FileDiff>,
    /// Whether the full file preview was truncated
    pub full_file_preview_truncated: bool,
    /// Whether the selected file is binary (no preview possible)
    pub full_file_preview_binary: bool,
    /// Editor-oriented split diff model (computed lazily for split view)
    pub editor_diff: Option<EditorDiffModel>,
    /// Runtime state for the editor-backed split diff surface.
    pub split_diff_editor: Option<SplitDiffEditorState>,
    /// Runtime state for the unified diff CodeEditor surface.
    pub unified_diff_editor: Option<crate::widgets::diff_editor::UnifiedDiffEditorState>,
    /// In-progress network operation for progress bar display
    pub network_operation: Option<NetworkOperation>,
    /// Pull strategy preference (merge or rebase)
    pub pull_strategy: PullStrategy,
    /// Available update info from GitHub
    pub available_update: Option<git_core::updater::UpdateInfo>,
}

/// In-progress network operation state for progress indicator
#[derive(Debug, Clone)]
pub struct NetworkOperation {
    pub label: String,
    pub progress: Option<f32>,
    pub status: Option<String>,
}

/// Pull strategy preference
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PullStrategy {
    #[default]
    Merge,
    Rebase,
}

#[derive(Debug, Clone, Default)]
struct AutoRefreshState {
    workspace_refresh_pending: bool,
    last_workspace_refresh_at: Option<Instant>,
    last_remote_check_at: Option<Instant>,
    remote_check_in_flight_for: Option<PathBuf>,
}

impl AppShellState {
    fn new_with_i18n(i18n: &I18n) -> Self {
        Self {
            active_section: ShellSection::Welcome,
            git_tool_window_tab: GitToolWindowTab::Changes,
            title: i18n.app_name.to_string(),
            subtitle: i18n.welcome_subtitle.to_string(),
            primary_action_label: i18n.open_repository.to_string(),
            context_switcher: WorkspaceContextSwitcher::default(),
            chrome: PrimaryWorkspaceChrome::default(),
            status_surface: LightweightStatusSurface::default(),
        }
    }

    fn new() -> Self {
        Self {
            active_section: ShellSection::Welcome,
            git_tool_window_tab: GitToolWindowTab::Changes,
            title: String::new(),
            subtitle: String::new(),
            primary_action_label: String::new(),
            context_switcher: WorkspaceContextSwitcher::default(),
            chrome: PrimaryWorkspaceChrome::default(),
            status_surface: LightweightStatusSurface::default(),
        }
    }
}

impl AppState {
    pub fn new() -> Self {
        Self::new_inner(None)
    }

    pub fn new_with_i18n(i18n: &I18n) -> Self {
        Self::new_inner(Some(i18n))
    }

    fn new_inner(i18n: Option<&I18n>) -> Self {
        let shell = match i18n {
            Some(i18n) => AppShellState::new_with_i18n(i18n),
            None => AppShellState::new(),
        };
        let log_tabs = match i18n {
            Some(i18n) => vec![LogTab::all_with_i18n(i18n)],
            None => vec![LogTab::all()],
        };
        let mut state = Self {
            current_repository: None,
            is_loading: false,
            error_message: None,
            view_mode: ViewMode::Welcome,
            auxiliary_view: None,
            project_history: Vec::new(),
            shell,
            feedback: None,
            toast_notification: None,
            defect_observations: Vec::new(),
            staged_changes: Vec::new(),
            unstaged_changes: Vec::new(),
            untracked_files: Vec::new(),
            conflicts_present: false,
            conflict_files: Vec::new(),
            selected_conflict_index: None,
            conflict_merge_index: None,
            auto_merge_result: None,
            conflict_resolver: None,
            show_diff: false,
            current_diff: None,
            diff_presentation: DiffPresentation::Unified,
            diff_source: DiffSource::Workspace,
            selected_change_path: None,
            selected_hunk_index: None,
            change_context_menu_path: None,
            change_context_menu_cursor: Point::new(0.0, 0.0),
            change_context_menu_anchor: None,
            commit_dialog: CommitDialogState::default(),
            branch_popup: BranchPopupState::default(),
            pending_commit_action: None,
            history_view: HistoryState::default(),
            history_commit_diff_popup: None,
            remote_dialog: RemoteDialogState::default(),
            tag_dialog: TagDialogState::default(),
            stash_panel: StashPanelState::default(),
            rebase_editor: RebaseEditorState::default(),
            toolbar_remote_menu: None,
            show_project_dropdown: false,
            show_branch_dropdown: false,
            git_settings: crate::views::settings_view::GitSettings::load(),
            auto_refresh: AutoRefreshState::default(),
            file_display_mode: FileDisplayMode::default(),
            log_tabs,
            active_log_tab: 0,
            next_log_tab_id: 1,
            log_branches_dashboard_visible: true,
            blame_active: false,
            recent_commit_messages: Vec::new(),
            merge_editor: None,
            worktree_state: Default::default(),
            full_file_preview: None,
            full_file_preview_truncated: false,
            full_file_preview_binary: false,
            editor_diff: None,
            split_diff_editor: None,
            unified_diff_editor: None,
            network_operation: None,
            pull_strategy: PullStrategy::default(),
            available_update: None,
        };

        state.sync_context_feedback(i18n);
        state
    }

    pub fn restore(i18n: &I18n) -> Self {
        let persisted = PersistedWorkspaceMemory::load();
        let mut state = Self::new_with_i18n(i18n);
        state.project_history = persisted
            .recent_paths
            .iter()
            .cloned()
            .map(ProjectEntry::from_path)
            .collect();

        if let Some(last_path) = persisted.last_open_repository {
            match Repository::discover(&last_path) {
                Ok(repo) => {
                    let repo_name = repo.name();
                    state.set_repository(repo, i18n);
                    state.set_info(
                        i18n.repo_restored,
                        Some(i18n.repo_auto_opened_fmt.replace("{}", &repo_name)),
                        "repository.restore",
                    );
                }
                Err(error) => {
                    warn!(
                        "Failed to restore last repository {}: {}",
                        last_path.display(),
                        error
                    );
                    state
                        .project_history
                        .retain(|entry| entry.path.as_path() != last_path.as_path());
                    state.persist_workspace_memory(None);
                    state.clear_feedback();
                }
            }
        }

        state
    }

    pub fn navigation_items(&self) -> Vec<NavigationItem> {
        let has_repo = self.current_repository.is_some();

        vec![
            NavigationItem {
                section: ShellSection::Changes,
                badge: Some(self.workspace_change_count().to_string()),
                enabled: has_repo,
            },
            NavigationItem {
                section: ShellSection::Conflicts,
                badge: if self.has_conflicts() {
                    Some(self.conflict_files.len().max(1).to_string())
                } else {
                    None
                },
                enabled: has_repo && self.has_conflicts(),
            },
        ]
    }

    pub fn workspace_change_count(&self) -> usize {
        self.staged_changes.len() + self.unstaged_changes.len() + self.untracked_files.len()
    }

    pub fn selected_change(&self) -> Option<&Change> {
        let selected = self.selected_change_path.as_deref()?;

        self.staged_changes
            .iter()
            .chain(self.unstaged_changes.iter())
            .chain(self.untracked_files.iter())
            .find(|change| change.path == selected)
    }

    pub fn navigate_to(&mut self, section: ShellSection, i18n: &I18n) {
        self.close_toolbar_remote_menu();

        if self.current_repository.is_none() {
            self.shell.active_section = ShellSection::Welcome;
            self.view_mode = ViewMode::Welcome;
            self.sync_shell_state(Some(i18n));
            self.sync_context_feedback(Some(i18n));
            return;
        }

        match section {
            ShellSection::Conflicts if self.has_conflicts() => {
                self.view_mode = ViewMode::ConflictResolver;
                self.shell.active_section = ShellSection::Conflicts;
            }
            ShellSection::Changes | ShellSection::Welcome => {
                self.view_mode = ViewMode::Repository;
                self.shell.active_section = ShellSection::Changes;
            }
            _ => {
                self.view_mode = ViewMode::Welcome;
                self.shell.active_section = ShellSection::Welcome;
            }
        }

        self.sync_shell_state(Some(i18n));
        self.sync_context_feedback(Some(i18n));
    }

    pub fn active_project_path(&self) -> Option<&Path> {
        self.current_repository.as_ref().map(|repo| repo.path())
    }

    pub fn set_repository(&mut self, repo: Repository, i18n: &I18n) {
        let project_entry = ProjectEntry::from_repository(&repo);
        let repository_changed = self
            .current_repository
            .as_ref()
            .map(|current| current.path().to_path_buf())
            != Some(repo.path().to_path_buf());

        self.current_repository = Some(repo);
        self.remember_project(project_entry);
        // Persist immediately so the project list survives crashes
        let active_path = self
            .current_repository
            .as_ref()
            .map(|r| r.path().to_path_buf());
        self.persist_workspace_memory(active_path.as_deref());
        self.reset_auto_refresh_state();
        self.is_loading = false;
        self.error_message = None;
        self.toast_notification = None;
        if repository_changed {
            self.reset_auxiliary_state();
        }
        self.show_diff = false;
        self.current_diff = None;
        self.diff_source = DiffSource::Workspace;
        self.editor_diff = None;
        self.split_diff_editor = None;
        self.unified_diff_editor = None;
        self.history_commit_diff_popup = None;
        self.selected_hunk_index = None;
        self.change_context_menu_path = None;
        self.change_context_menu_cursor = Point::new(0.0, 0.0);
        self.change_context_menu_anchor = None;
        self.toolbar_remote_menu = None;
        self.conflict_files.clear();
        self.selected_conflict_index = None;
        self.conflict_merge_index = None;
        self.auto_merge_result = None;
        self.conflict_resolver = None;
        self.diff_presentation = DiffPresentation::Unified;
        self.selected_change_path = None;
        self.refresh_changes_with_i18n(i18n);

        if self.has_conflicts() {
            if let Err(error) = self.load_conflicts(i18n) {
                self.set_error_i18n(i18n.load_conflict_failed_fmt.replace("{}", &error), i18n);
            } else {
                self.view_mode = ViewMode::ConflictResolver;
                self.shell.active_section = ShellSection::Conflicts;
                self.sync_context_feedback(Some(i18n));
            }
        } else {
            self.view_mode = ViewMode::Repository;
            self.shell.active_section = ShellSection::Changes;
            self.set_success(
                i18n.repo_opened.to_string(),
                self.current_repository
                    .as_ref()
                    .map(|current| i18n.entered_workspace_fmt.replace("{}", &current.name())),
                "repository.open",
            );
        }

        self.sync_shell_state(Some(i18n));
        if let Some(active_path) = self
            .current_repository
            .as_ref()
            .map(|current| current.path().to_path_buf())
        {
            self.persist_workspace_memory(Some(active_path.as_path()));
        }

        self.mark_workspace_refreshed(Instant::now());
    }

    pub fn switch_to_project(&mut self, path: &Path, i18n: &I18n) -> Result<(), String> {
        if !path.exists() {
            self.project_history
                .retain(|entry| entry.path.as_path() != path);
            let active_path = self
                .current_repository
                .as_ref()
                .map(|current| current.path().to_path_buf());
            self.persist_workspace_memory(active_path.as_deref());
            return Err(i18n.project_dir_not_exist_fmt.replace("{}", &path.display().to_string()));
        }

        let repo = Repository::discover(path).map_err(|error| i18n.cannot_open_project_fmt.replace("{}", &error.to_string()))?;
        self.set_repository(repo, i18n);
        Ok(())
    }

    pub fn clear_repository(&mut self, i18n: &I18n) {
        self.current_repository = None;
        self.staged_changes.clear();
        self.unstaged_changes.clear();
        self.untracked_files.clear();
        self.conflict_files.clear();
        self.selected_conflict_index = None;
        self.conflict_merge_index = None;
        self.auto_merge_result = None;
        self.conflict_resolver = None;
        self.show_diff = false;
        self.current_diff = None;
        self.diff_source = DiffSource::Workspace;
        self.editor_diff = None;
        self.split_diff_editor = None;
        self.unified_diff_editor = None;
        self.history_commit_diff_popup = None;
        self.diff_presentation = DiffPresentation::Unified;
        self.selected_change_path = None;
        self.toolbar_remote_menu = None;
        self.toast_notification = None;
        self.reset_auxiliary_state();
        self.reset_auto_refresh_state();
        self.view_mode = ViewMode::Welcome;
        self.shell.active_section = ShellSection::Welcome;
        self.sync_shell_state(Some(i18n));
        self.sync_context_feedback(Some(i18n));
        self.persist_workspace_memory(None);
    }

    pub fn open_auxiliary_view(&mut self, view: AuxiliaryView, i18n: &I18n) {
        self.close_toolbar_remote_menu();
        self.close_history_commit_diff_popup();
        if view == AuxiliaryView::Branches {
            self.auxiliary_view = None;
            self.show_branch_dropdown = true;
            self.view_mode = ViewMode::Repository;
            self.sync_shell_state(Some(i18n));
            return;
        }

        self.show_branch_dropdown = false;
        self.auxiliary_view = Some(view);
        self.view_mode = ViewMode::Repository;
        self.sync_shell_state(Some(i18n));
    }

    pub fn close_auxiliary_view(&mut self, i18n: &I18n) {
        self.close_toolbar_remote_menu();
        self.close_history_commit_diff_popup();
        self.auxiliary_view = None;
        self.show_branch_dropdown = false;
        self.sync_shell_state(Some(i18n));
    }

    pub fn switch_git_tool_window_tab(&mut self, tab: GitToolWindowTab, i18n: &I18n) {
        self.close_toolbar_remote_menu();
        self.close_history_commit_diff_popup();
        self.auxiliary_view = None;
        self.show_branch_dropdown = false;
        self.shell.git_tool_window_tab = tab;
        self.sync_shell_state(Some(i18n));
    }

    pub fn toggle_toolbar_remote_menu(
        &mut self,
        action: ToolbarRemoteAction,
        i18n: &I18n,
    ) -> Result<(), String> {
        if self
            .toolbar_remote_menu
            .as_ref()
            .is_some_and(|menu| menu.action == action)
        {
            self.toolbar_remote_menu = None;
            return Ok(());
        }

        let repo = self
            .current_repository
            .as_ref()
            .ok_or_else(|| i18n.no_repo_opened.to_string())?;
        let preferred_remote = repo.current_upstream_remote();
        let remotes = git_core::remote::list_remotes(repo)
            .map_err(|error| i18n.load_remote_failed_fmt.replace("{}", &error.to_string()))?
            .into_iter()
            .filter(|remote| {
                preferred_remote
                    .as_deref()
                    .is_some_and(|name| name == remote.name)
            })
            .collect();

        self.toolbar_remote_menu = Some(ToolbarRemoteMenuState {
            action,
            preferred_remote,
            remotes,
        });

        Ok(())
    }

    pub fn close_toolbar_remote_menu(&mut self) {
        self.toolbar_remote_menu = None;
    }

    pub fn set_loading(
        &mut self,
        title: impl Into<String>,
        detail: Option<String>,
        source: &'static str,
    ) {
        self.is_loading = true;
        self.toast_notification = None;
        self.feedback = Some(FeedbackState {
            level: FeedbackLevel::Loading,
            title: title.into(),
            detail,
            source,
            compact: false,
            sticky: true,
        });
        self.sync_status_surface(None);
    }

    pub fn set_info(
        &mut self,
        title: impl Into<String>,
        detail: Option<String>,
        source: &'static str,
    ) {
        self.is_loading = false;
        self.toast_notification = None;
        self.feedback = Some(FeedbackState {
            level: FeedbackLevel::Info,
            title: title.into(),
            detail,
            source,
            compact: true,
            sticky: false,
        });
        self.sync_status_surface(None);
    }

    pub fn set_empty(
        &mut self,
        title: impl Into<String>,
        detail: Option<String>,
        source: &'static str,
    ) {
        self.is_loading = false;
        self.error_message = None;
        self.toast_notification = None;
        self.feedback = Some(FeedbackState {
            level: FeedbackLevel::Empty,
            title: title.into(),
            detail,
            source,
            compact: true,
            sticky: false,
        });
        self.sync_status_surface(None);
    }

    pub fn set_success(
        &mut self,
        title: impl Into<String>,
        detail: Option<String>,
        source: &'static str,
    ) {
        self.is_loading = false;
        self.error_message = None;
        self.feedback = Some(FeedbackState {
            level: FeedbackLevel::Success,
            title: title.into(),
            detail,
            source,
            compact: true,
            sticky: false,
        });
        self.sync_status_surface(None);
    }

    pub fn set_warning(
        &mut self,
        title: impl Into<String>,
        detail: Option<String>,
        source: &'static str,
    ) {
        self.is_loading = false;
        self.toast_notification = None;
        self.feedback = Some(FeedbackState {
            level: FeedbackLevel::Warning,
            title: title.into(),
            detail,
            source,
            compact: false,
            sticky: true,
        });
        self.sync_status_surface(None);
    }

    pub fn set_error(&mut self, error: String) {
        self.error_message = Some(error.clone());
        self.is_loading = false;
        self.toast_notification = None;
        self.feedback = Some(FeedbackState {
            level: FeedbackLevel::Error,
            title: error.clone(),
            detail: Some(error),
            source: "app.error",
            compact: false,
            sticky: true,
        });
        self.sync_status_surface(None);
    }

    pub fn set_error_i18n(&mut self, error: String, i18n: &I18n) {
        self.error_message = Some(error.clone());
        self.is_loading = false;
        self.toast_notification = None;
        self.feedback = Some(FeedbackState {
            level: FeedbackLevel::Error,
            title: i18n.operation_failed.to_string(),
            detail: Some(error),
            source: "app.error",
            compact: false,
            sticky: true,
        });
        self.sync_status_surface(None);
    }

    pub fn set_error_with_source(
        &mut self,
        title: impl Into<String>,
        detail: impl Into<String>,
        source: &'static str,
    ) {
        let detail = detail.into();
        self.error_message = Some(detail.clone());
        self.is_loading = false;
        self.toast_notification = None;
        self.feedback = Some(FeedbackState {
            level: FeedbackLevel::Error,
            title: title.into(),
            detail: Some(detail),
            source,
            compact: false,
            sticky: true,
        });
        self.sync_status_surface(None);
    }

    pub fn clear_feedback(&mut self) {
        self.feedback = None;
        self.sync_status_surface(None);
    }

    pub fn show_toast(
        &mut self,
        level: FeedbackLevel,
        title: impl Into<String>,
        detail: Option<String>,
    ) {
        self.toast_notification = Some(ToastNotificationState {
            level,
            title: title.into(),
            detail,
            expires_at: Instant::now() + TOAST_NOTIFICATION_DURATION,
        });
    }

    pub fn dismiss_toast(&mut self) {
        self.toast_notification = None;
    }

    pub fn toast_has_expired(&self, now: Instant) -> bool {
        self.toast_notification
            .as_ref()
            .is_some_and(|toast| now >= toast.expires_at)
    }

    pub fn recovery_hint_for_source(&self, source: &'static str, i18n: &I18n) -> Option<String> {
        match source {
            "repository.open" => {
                Some(i18n.recovery_hint_open_repo.to_string())
            }
            "repository.init" => {
                Some(i18n.recovery_hint_init_repo.to_string())
            }
            "repository.refresh" => {
                Some(i18n.recovery_hint_refresh.to_string())
            }
            "workspace.select_change" => {
                Some(i18n.recovery_hint_select_change.to_string())
            }
            "workspace.conflicts" | "shell.conflicts" => {
                Some(i18n.recovery_hint_conflicts.to_string())
            }
            "shell.changes" => {
                if self.workspace_change_count() == 0 {
                    Some(i18n.recovery_hint_changes_empty.to_string())
                } else {
                    Some(i18n.recovery_hint_changes_select.to_string())
                }
            }
            "app.error" => {
                if self.current_repository.is_some() {
                    Some(i18n.recovery_hint_error_with_repo.to_string())
                } else {
                    Some(i18n.recovery_hint_error_no_repo.to_string())
                }
            }
            _ => None,
        }
    }

    pub fn feedback_next_step(&self, i18n: &I18n) -> Option<FeedbackNextStep> {
        let feedback = self.feedback.as_ref()?;

        if !matches!(
            feedback.level,
            FeedbackLevel::Error | FeedbackLevel::Warning | FeedbackLevel::Empty
        ) {
            return None;
        }

        let action = match feedback.source {
            "repository.open" => RecoveryAction::OpenRepository,
            "repository.init" => RecoveryAction::InitRepository,
            "repository.refresh" | "workspace.select_change" => RecoveryAction::Refresh,
            "workspace.conflicts" | "shell.conflicts" if self.has_conflicts() => {
                RecoveryAction::ShowConflicts
            }
            "shell.changes" if self.workspace_change_count() > 0 => RecoveryAction::ShowChanges,
            "app.error" if self.current_repository.is_some() => RecoveryAction::Refresh,
            "app.error" => RecoveryAction::OpenRepository,
            _ => match self.shell.active_section {
                ShellSection::Conflicts if self.has_conflicts() => RecoveryAction::ShowConflicts,
                ShellSection::Changes if self.workspace_change_count() > 0 => {
                    RecoveryAction::ShowChanges
                }
                _ if self.current_repository.is_some() => RecoveryAction::Refresh,
                _ => RecoveryAction::OpenRepository,
            },
        };

        Some(FeedbackNextStep {
            title: i18n.next_step_label.to_string(),
            detail: self
                .recovery_hint_for_source(feedback.source, i18n)
                .unwrap_or_else(|| i18n.recovery_hint_default.to_string()),
            action,
        })
    }

    fn sync_context_feedback(&mut self, i18n: Option<&I18n>) {
        if self.is_loading {
            return;
        }

        let preserve_explicit = self
            .feedback
            .as_ref()
            .is_some_and(|feedback| !feedback.source.starts_with("shell."));

        if preserve_explicit {
            return;
        }

        self.feedback = match self.shell.active_section {
            ShellSection::Conflicts if !self.conflict_files.is_empty() => {
                let title = if let Some(i18n) = i18n {
                    i18n.conflicts_pending_fmt.replace("{}", &self.conflict_files.len().to_string())
                } else {
                    format!("{} conflicts pending", self.conflict_files.len())
                };
                let detail = if let Some(i18n) = i18n {
                    i18n.resolve_conflicts_first.to_string()
                } else {
                    "Resolve conflicts before continuing other Git operations.".to_string()
                };
                Some(FeedbackState {
                    level: FeedbackLevel::Warning,
                    title,
                    detail: Some(detail),
                    source: "shell.conflicts",
                    compact: false,
                    sticky: true,
                })
            }
            _ => None,
        };
        self.sync_status_surface(i18n);
    }

    pub fn record_defect_observation(
        &mut self,
        area: impl Into<String>,
        summary: impl Into<String>,
    ) {
        let area = area.into();
        let summary = summary.into();

        if self
            .defect_observations
            .iter()
            .any(|observation| observation.area == area && observation.summary == summary)
        {
            return;
        }

        self.defect_observations.push(DefectObservation {
            area,
            summary,
            verified: false,
        });
    }

    pub fn refresh_changes(&mut self) {
        self.refresh_changes_inner(None);
    }

    pub fn refresh_changes_with_i18n(&mut self, i18n: &I18n) {
        self.refresh_changes_inner(Some(i18n));
    }

    fn refresh_changes_inner(&mut self, i18n: Option<&I18n>) {
        if let Some(repo) = &self.current_repository {
            match git_core::index::get_status(repo) {
                Ok(changes) => {
                    self.staged_changes = changes
                        .iter()
                        .filter(|change| {
                            change.staged
                                && change.status != git_core::index::ChangeStatus::Conflict
                        })
                        .cloned()
                        .collect();

                    self.unstaged_changes = changes
                        .iter()
                        .filter(|change| {
                            change.unstaged
                                && change.status != git_core::index::ChangeStatus::Untracked
                                && change.status != git_core::index::ChangeStatus::Conflict
                        })
                        .cloned()
                        .collect();

                    self.untracked_files = changes
                        .iter()
                        .filter(|change| change.status == git_core::index::ChangeStatus::Untracked)
                        .cloned()
                        .collect();
                    self.conflicts_present = changes
                        .iter()
                        .any(|change| change.status == git_core::index::ChangeStatus::Conflict);

                    if let Some(path) = self.selected_change_path.clone() {
                        if self.diff_source == DiffSource::Workspace {
                            let still_exists = changes.iter().any(|change| change.path == path);
                            if still_exists {
                                if let Err(error) = self.load_diff_for_file_with_i18n(&path, i18n) {
                                    self.set_error(error);
                                }
                            } else {
                                self.selected_change_path = None;
                                self.show_diff = false;
                                self.current_diff = None;
                                self.diff_source = DiffSource::Workspace;
                                self.editor_diff = None;
                                self.split_diff_editor = None;
                                self.unified_diff_editor = None;
                                self.selected_hunk_index = None;
                                self.change_context_menu_path = None;
                                self.change_context_menu_anchor = None;
                            }
                        }
                    }

                    if self.selected_change_path.is_none()
                        && self.auxiliary_view.is_none()
                        && self.view_mode == ViewMode::Repository
                    {
                        if let Some(path) = self.preferred_change_path() {
                            self.selected_change_path = Some(path.clone());
                            if let Err(error) = self.load_diff_for_file_with_i18n(&path, i18n) {
                                self.set_error(error);
                            }
                        }
                    }
                }
                Err(error) => {
                    let msg = i18n.map_or_else(
                        || format!("Failed to get changes: {}", error),
                        |i| i.get_changes_failed_fmt.replace("{}", &error.to_string()),
                    );
                    self.set_error(msg);
                }
            }
        }

        self.sync_commit_dialog_state();
        self.sync_shell_state(i18n);
    }

    fn sync_commit_dialog_state(&mut self) {
        if self.current_repository.is_none() {
            return;
        }

        self.commit_dialog.staged_files = self.staged_changes.clone();
        self.commit_dialog
            .selected_files
            .retain(|path| self.staged_changes.iter().any(|c| &c.path == path));
        if self.commit_dialog.selected_files.is_empty() && !self.staged_changes.is_empty() {
            self.commit_dialog.selected_files =
                self.staged_changes.iter().map(|c| c.path.clone()).collect();
        }
        self.commit_dialog.ensure_preview_target();
    }

    pub fn refresh_current_repository(&mut self, prefer_conflicts: bool, i18n: &I18n) -> Result<(), String> {
        let previous_section = self.shell.active_section;
        let previous_auxiliary = self.auxiliary_view;

        let mut repo = self
            .current_repository
            .clone()
            .ok_or_else(|| i18n.no_repo_opened.to_string())?;

        repo.refresh()
            .map_err(|error| i18n.refresh_repo_state_err_fmt.replace("{}", &error.to_string()))?;
        self.current_repository = Some(repo);
        self.is_loading = false;
        self.error_message = None;

        self.refresh_changes_with_i18n(i18n);

        if self.has_conflicts() {
            self.load_conflicts(i18n)?;
        } else {
            self.conflict_files.clear();
            self.selected_conflict_index = None;
            self.conflict_merge_index = None;
            self.auto_merge_result = None;
            self.conflict_resolver = None;
        }

        if prefer_conflicts && self.has_conflicts() {
            self.close_auxiliary_view(i18n);
            self.open_conflict_resolver(i18n)?;
            self.mark_workspace_refreshed(Instant::now());
            return Ok(());
        }

        let target_section = if previous_section == ShellSection::Conflicts && !self.has_conflicts()
        {
            ShellSection::Changes
        } else {
            previous_section
        };

        self.navigate_to(target_section, i18n);

        if let Some(auxiliary) =
            previous_auxiliary.filter(|_| target_section != ShellSection::Conflicts)
        {
            self.open_auxiliary_view(auxiliary, i18n);
        }

        self.mark_workspace_refreshed(Instant::now());
        Ok(())
    }

    pub fn has_conflicts(&self) -> bool {
        self.conflicts_present
    }

    pub fn load_conflicts(&mut self, i18n: &I18n) -> Result<(), String> {
        if let Some(repo) = &self.current_repository {
            let previous_selected_path = self
                .selected_conflict_index
                .and_then(|index| self.conflict_files.get(index))
                .map(|conflict| conflict.path.clone());
            let previous_merge_path = self
                .conflict_merge_index
                .and_then(|index| self.conflict_files.get(index))
                .map(|conflict| conflict.path.clone());
            let conflict_paths = git_core::index::get_conflicted_files(repo)
                .map_err(|error| i18n.get_conflict_list_failed_fmt.replace("{}", &error.to_string()))?;

            self.conflict_files.clear();

            for path in conflict_paths {
                match git_core::diff::get_conflict_diff(repo, std::path::Path::new(&path)) {
                    Ok(diff) => self.conflict_files.push(diff),
                    Err(error) => log::warn!("Failed to get conflict diff for {}: {}", path, error),
                }
            }

            if !self.conflict_files.is_empty() {
                self.conflicts_present = true;
                self.selected_conflict_index = previous_selected_path
                    .as_ref()
                    .and_then(|path| {
                        self.conflict_files
                            .iter()
                            .position(|conflict| &conflict.path == path)
                    })
                    .or(Some(0));
                self.conflict_merge_index = previous_merge_path.as_ref().and_then(|path| {
                    self.conflict_files
                        .iter()
                        .position(|conflict| &conflict.path == path)
                });
                self.sync_selected_conflict_resolver();
            } else {
                self.conflicts_present = false;
                self.selected_conflict_index = None;
                self.conflict_merge_index = None;
                self.conflict_resolver = None;
            }

            self.sync_shell_state(Some(i18n));
            Ok(())
        } else {
            Err(i18n.no_repo_opened.to_string())
        }
    }

    pub fn open_conflict_resolver(&mut self, i18n: &I18n) -> Result<(), String> {
        if self.has_conflicts() {
            self.load_conflicts(i18n)?;
            self.auxiliary_view = None;
            self.view_mode = ViewMode::ConflictResolver;
            self.shell.active_section = ShellSection::Conflicts;
            self.conflict_merge_index = None;
            self.conflict_resolver = None;
            self.sync_shell_state(Some(i18n));
            self.sync_context_feedback(Some(i18n));
            Ok(())
        } else {
            Err(i18n.no_conflicts_to_resolve.to_string())
        }
    }

    pub fn close_conflict_resolver(&mut self, i18n: &I18n) {
        self.view_mode = ViewMode::Repository;
        self.conflict_files.clear();
        self.selected_conflict_index = None;
        self.conflict_merge_index = None;
        self.auto_merge_result = None;
        self.conflict_resolver = None;
        self.shell.active_section = ShellSection::Changes;
        self.sync_shell_state(Some(i18n));
        self.sync_context_feedback(Some(i18n));
    }

    pub fn select_conflict(&mut self, index: usize) {
        if index < self.conflict_files.len() {
            self.selected_conflict_index = Some(index);
            if self.conflict_merge_index.is_some() {
                self.conflict_merge_index = Some(index);
                self.sync_selected_conflict_resolver();
            }
        }
    }

    pub fn get_selected_conflict(&self) -> Option<&ThreeWayDiff> {
        self.selected_conflict_index
            .and_then(|index| self.conflict_files.get(index))
    }

    pub fn get_active_conflict_merge(&self) -> Option<&ThreeWayDiff> {
        self.conflict_merge_index
            .and_then(|index| self.conflict_files.get(index))
    }

    pub fn open_conflict_merge(&mut self, index: usize, i18n: &I18n) -> Result<(), String> {
        if index >= self.conflict_files.len() {
            return Err(i18n.conflict_file_not_found_state.to_string());
        }

        self.selected_conflict_index = Some(index);
        self.conflict_merge_index = Some(index);
        self.sync_selected_conflict_resolver();
        Ok(())
    }

    pub fn close_conflict_merge(&mut self) {
        self.conflict_merge_index = None;
        self.conflict_resolver = None;
    }

    fn compute_next_selection(&self, path: &str, source_list: &[Change]) -> Option<String> {
        if source_list.len() <= 1 {
            return None;
        }
        let index = source_list.iter().position(|c| c.path == path)?;
        if index + 1 < source_list.len() {
            Some(source_list[index + 1].path.clone())
        } else {
            Some(source_list[index - 1].path.clone())
        }
    }

    pub fn stage_file(&mut self, path: String, i18n: &I18n) -> Result<(), String> {
        if let Some(repo) = &self.current_repository {
            let source_list = if self.unstaged_changes.iter().any(|c| c.path == path) {
                &self.unstaged_changes
            } else if self.untracked_files.iter().any(|c| c.path == path) {
                &self.untracked_files
            } else {
                &[][..]
            };
            let next_path = self.compute_next_selection(&path, source_list);

            git_core::index::stage_file(repo, std::path::Path::new(&path))
                .map_err(|error| i18n.stage_file_err_fmt.replace("{}", &error.to_string()))?;
            self.refresh_changes_with_i18n(i18n);

            if let Some(next) = next_path {
                self.selected_change_path = Some(next.clone());
                let _ = self.load_diff_for_file_with_i18n(&next, Some(i18n));
            }

            self.set_success(i18n.file_staged, Some(path), "workspace.stage_file");
            Ok(())
        } else {
            Err(i18n.no_repo_opened.to_string())
        }
    }

    pub fn unstage_file(&mut self, path: String, i18n: &I18n) -> Result<(), String> {
        if let Some(repo) = &self.current_repository {
            let next_path = self.compute_next_selection(&path, &self.staged_changes);

            git_core::index::unstage_file(repo, std::path::Path::new(&path))
                .map_err(|error| i18n.unstage_err_fmt.replace("{}", &error.to_string()))?;
            self.refresh_changes_with_i18n(i18n);

            if let Some(next) = next_path {
                self.selected_change_path = Some(next.clone());
                let _ = self.load_diff_for_file_with_i18n(&next, Some(i18n));
            }

            self.set_success(i18n.file_unstaged, Some(path), "workspace.unstage_file");
            Ok(())
        } else {
            Err(i18n.no_repo_opened.to_string())
        }
    }

    pub fn stage_all(&mut self, i18n: &I18n) -> Result<(), String> {
        if let Some(repo) = &self.current_repository {
            git_core::index::stage_all(repo).map_err(|error| i18n.stage_all_err_fmt.replace("{}", &error.to_string()))?;
            self.refresh_changes_with_i18n(i18n);
            self.set_success(i18n.all_changes_staged, None, "workspace.stage_all");
            Ok(())
        } else {
            Err(i18n.no_repo_opened.to_string())
        }
    }

    pub fn unstage_all(&mut self, i18n: &I18n) -> Result<(), String> {
        if let Some(repo) = &self.current_repository {
            git_core::index::unstage_all(repo)
                .map_err(|error| i18n.unstage_all_err_fmt.replace("{}", &error.to_string()))?;
            self.refresh_changes_with_i18n(i18n);
            self.set_success(i18n.all_changes_unstaged, None, "workspace.unstage_all");
            Ok(())
        } else {
            Err(i18n.no_repo_opened.to_string())
        }
    }

    pub fn load_diff(&mut self, i18n: &I18n) -> Result<(), String> {
        if let Some(repo) = &self.current_repository {
            match git_core::diff::diff_workdir_to_index(repo) {
                Ok(diff) => {
                    self.current_diff = Some(diff);
                    self.selected_hunk_index = Some(0);
                    self.show_diff = true;
                    Ok(())
                }
                Err(error) => Err(i18n.load_diff_err_fmt.replace("{}", &error.to_string())),
            }
        } else {
            Err(i18n.no_repo_opened.to_string())
        }
    }

    pub fn toggle_diff_presentation(&mut self) {
        self.diff_presentation = match self.diff_presentation {
            DiffPresentation::Unified => DiffPresentation::Split,
            DiffPresentation::Split => DiffPresentation::Unified,
        };
    }

    pub fn open_history_commit_diff_popup(
        &mut self,
        commit_id: String,
        file_path: String,
        diff: Diff,
        editor_diff: Option<EditorDiffModel>,
    ) {
        self.history_view.selected_commit_file_path = Some(file_path.clone());
        self.history_commit_diff_popup = Some(HistoryCommitDiffPopupState::new(
            commit_id,
            file_path,
            diff,
            editor_diff,
            self.git_settings.editor_font_size_f32(),
        ));
    }

    pub fn close_history_commit_diff_popup(&mut self) {
        self.history_commit_diff_popup = None;
    }

    pub fn select_change(&mut self, path: String) -> Result<(), String> {
        self.selected_change_path = Some(path.clone());
        self.diff_source = DiffSource::Workspace;
        self.shell.active_section = ShellSection::Changes;
        self.view_mode = ViewMode::Repository;
        self.full_file_preview = None;
        self.full_file_preview_binary = false;
        self.full_file_preview_truncated = false;
        self.editor_diff = None;
        self.split_diff_editor = None;
        self.unified_diff_editor = None;
        self.sync_shell_state(None::<&I18n>);

        let result = self.load_diff_for_file(&path);

        // If diff is empty/missing, try full file preview (for new/untracked files)
        let diff_empty = self
            .current_diff
            .as_ref()
            .map(|d| d.files.is_empty())
            .unwrap_or(true);

        if diff_empty || result.is_err() {
            if let Some(repo) = &self.current_repository {
                let file_path = std::path::Path::new(&path);
                if let Ok(preview) = git_core::diff::build_full_file_diff(repo, file_path) {
                    self.full_file_preview_binary = preview.is_binary;
                    self.full_file_preview_truncated = preview.is_truncated;
                    if !preview.is_binary {
                        self.full_file_preview = Some(preview.diff);
                    }
                }
            }
        }

        result.or(Ok(()))
    }

    pub fn load_diff_for_file(&mut self, path: &str) -> Result<(), String> {
        self.load_diff_for_file_with_i18n(path, None)
    }

    pub fn load_diff_for_file_with_i18n(&mut self, path: &str, i18n: Option<&I18n>) -> Result<(), String> {
        if let Some(repo) = &self.current_repository {
            let selected_change = self
                .selected_change()
                .cloned()
                .ok_or_else(|| i18n.map_or_else(
                    || "Selected file not found".to_string(),
                    |i| i.file_not_found.to_string(),
                ))?;

            let diff = if selected_change.staged && !selected_change.unstaged {
                git_core::diff::diff_index_to_head(repo, std::path::Path::new(path))
                    .map_err(|error| i18n.map_or_else(
                        || format!("Failed to load staged diff: {}", error),
                        |i| i.load_staged_diff_err_state_fmt.replace("{}", &error.to_string()),
                    ))?
            } else {
                git_core::diff::diff_file_to_index(repo, std::path::Path::new(path))
                    .map_err(|error| i18n.map_or_else(
                        || format!("Failed to load file diff: {}", error),
                        |i| i.load_file_diff_err_state_fmt.replace("{}", &error.to_string()),
                    ))?
            };

            self.current_diff = Some(diff);
            self.diff_source = DiffSource::Workspace;
            self.selected_hunk_index = Some(0);
            self.show_diff = true;
            Ok(())
        } else {
            Err(i18n.map_or_else(
                || "No repository opened".to_string(),
                |i| i.no_repo_opened.to_string(),
            ))
        }
    }

    pub fn navigate_hunk(&mut self, delta: isize) -> Option<f32> {
        let diff = self.current_diff.as_ref()?;
        let total_hunks: usize = diff.files.iter().map(|f| f.hunks.len()).sum();
        if total_hunks == 0 {
            return None;
        }
        let current = self.selected_hunk_index.unwrap_or(0) as isize;
        let next = (current + delta).clamp(0, (total_hunks - 1) as isize) as usize;
        self.selected_hunk_index = Some(next);
        Some(compute_hunk_offset(diff, next))
    }

    pub fn track_change_context_menu_cursor(&mut self, position: Point) {
        self.change_context_menu_cursor = position;
    }

    pub fn open_change_context_menu(&mut self, path: String) {
        self.change_context_menu_path = Some(path);
        self.change_context_menu_anchor = Some(self.change_context_menu_cursor);
    }

    pub fn close_change_context_menu(&mut self) {
        self.change_context_menu_path = None;
        self.change_context_menu_anchor = None;
    }
}

pub(crate) fn compute_hunk_offset(diff: &git_core::diff::Diff, hunk_index: usize) -> f32 {
    const FILE_HEADER_APPROX: f32 = 32.0;
    const FILE_SPACING: f32 = 4.0;
    const HUNK_DIVIDER_HEIGHT: f32 = 18.0;
    const HUNK_HEADER_HEIGHT: f32 = 22.0;
    const ROW_HEIGHT: f32 = 22.0;

    let show_file_header = diff.files.len() > 1;
    let mut offset = 0.0;
    let mut current_index = 0;

    for (file_idx, file) in diff.files.iter().enumerate() {
        if file_idx > 0 {
            offset += FILE_SPACING;
        }
        if show_file_header {
            offset += FILE_HEADER_APPROX + FILE_SPACING;
        }
        for (hunk_idx, hunk) in file.hunks.iter().enumerate() {
            if current_index == hunk_index {
                return offset;
            }
            offset += HUNK_HEADER_HEIGHT;
            offset += hunk.lines.len() as f32 * ROW_HEIGHT;
            if hunk_idx + 1 < file.hunks.len() {
                offset += HUNK_DIVIDER_HEIGHT;
            }
            current_index += 1;
        }
        // If file has no hunks, render_empty_editor_row() consumes roughly a row height.
        if file.hunks.is_empty() {
            offset += ROW_HEIGHT;
        }
    }
    offset
}

impl AppState {
    fn sync_shell_state(&mut self, i18n_ref: Option<&I18n>) {
        if let Some(repo) = &self.current_repository {
            let branch = repo.current_branch_display();
            let repo_name = repo.name();
            let repo_path = repo.path().display().to_string();
            let sync_status = repo.sync_status();
            let sync_label = sync_label(&sync_status);
            let sync_hint = sync_status.hint_text();
            let state_hint = repo.state_hint();
            let preserved_tab = self.shell.git_tool_window_tab;

            let branch_actions_label = i18n_ref.map_or_else(
                || "Branches & Actions".to_string(),
                |i| i.branch_actions.to_string(),
            );
            let conflicts_label = i18n_ref.map_or_else(
                || "Conflicts".to_string(),
                |i| i.conflicts.to_string(),
            );
            let handle_conflicts_label = i18n_ref.map_or_else(
                || "Handle Conflicts".to_string(),
                |i| i.handle_conflicts.to_string(),
            );
            let close_label = i18n_ref.map_or_else(
                || "Close".to_string(),
                |i| i.close.to_string(),
            );
            let current_focus_label = i18n_ref.map_or_else(
                || "Current Focus".to_string(),
                |i| i.current_focus.to_string(),
            );

            let mut shell = match self.shell.active_section {
                ShellSection::Changes | ShellSection::Welcome => AppShellState {
                    active_section: ShellSection::Changes,
                    git_tool_window_tab: preserved_tab,
                    title: repo_name.clone(),
                    subtitle: branch.clone(),
                    primary_action_label: branch_actions_label.clone(),
                    context_switcher: WorkspaceContextSwitcher::default(),
                    chrome: PrimaryWorkspaceChrome::default(),
                    status_surface: LightweightStatusSurface::default(),
                },
                ShellSection::Conflicts => AppShellState {
                    active_section: ShellSection::Conflicts,
                    git_tool_window_tab: preserved_tab,
                    title: repo_name.clone(),
                    subtitle: conflicts_label,
                    primary_action_label: handle_conflicts_label,
                    context_switcher: WorkspaceContextSwitcher::default(),
                    chrome: PrimaryWorkspaceChrome::default(),
                    status_surface: LightweightStatusSurface::default(),
                },
            };

            if let Some(auxiliary) = self.auxiliary_view {
                match auxiliary {
                    AuxiliaryView::Commit
                    | AuxiliaryView::History
                    | AuxiliaryView::Remotes
                    | AuxiliaryView::Tags
                    | AuxiliaryView::Stashes
                    | AuxiliaryView::Rebase
                    | AuxiliaryView::Worktrees
                    | AuxiliaryView::Settings => {}
                    AuxiliaryView::Branches => {
                        shell.title = branch_actions_label;
                        shell.subtitle = branch.clone();
                        shell.primary_action_label = close_label;
                    }
                }
            }

            shell.context_switcher = WorkspaceContextSwitcher {
                repository_name: repo_name.clone(),
                repository_path: repo_path,
                branch_name: branch,
                sync_label,
                sync_hint,
                state_hint,
                secondary_label: self
                    .auxiliary_view
                    .filter(|view| is_docked_auxiliary_view(*view))
                    .map(|v| auxiliary_label(v, i18n_ref))
                    .or_else(|| {
                        self.selected_change_path
                            .as_ref()
                            .map(|_| current_focus_label)
                    }),
                overflow_behavior: if repo_name.len() > 28 || shell.subtitle.len() > 20 {
                    OverflowBehavior::HorizontalScroll
                } else {
                    OverflowBehavior::TruncateTail
                },
            };
            let editor_tab_label = self
                .selected_change_path
                .as_deref()
                .and_then(|path| {
                    std::path::Path::new(path)
                        .file_name()
                        .and_then(|value| value.to_str())
                        .map(str::to_string)
                })
                .unwrap_or_else(|| match self.shell.active_section {
                    ShellSection::Conflicts => "Conflicts".to_string(),
                    _ => "Changes".to_string(),
                });
            let editor_tab_detail = self.selected_change_path.as_ref().and_then(|path| {
                std::path::Path::new(path)
                    .parent()
                    .and_then(|value| value.to_str())
                    .filter(|value| !value.is_empty())
                    .map(str::to_string)
            });
            shell.chrome = PrimaryWorkspaceChrome {
                max_visible_top_bars: 2,
                toolbar_height: theme::layout::TOP_BAR_HEIGHT as u16,
                control_height: theme::layout::CONTROL_HEIGHT as u16,
                container_radius: theme::radius::SM as u16,
                section_gap: theme::layout::SHELL_GAP as u16,
                content_padding: theme::layout::SHELL_PADDING as u16,
                elevation: if self.has_conflicts() {
                    ChromeElevation::Emphasized
                } else {
                    ChromeElevation::Subtle
                },
                change_count: self.workspace_change_count(),
                conflict_count: self.conflict_files.len(),
                selected_path: self.selected_change_path.clone(),
                has_selected_change: self.selected_change_path.is_some(),
                has_staged_changes: !self.staged_changes.is_empty(),
                has_secondary_actions: true,
                editor_tab_label,
                editor_tab_detail,
                tool_window_title: self
                    .auxiliary_view
                    .filter(|view| is_docked_auxiliary_view(*view))
                    .map(|v| auxiliary_label(v, i18n_ref)),
            };

            self.shell = shell;
            self.sync_status_surface(i18n_ref);
        } else {
            self.shell = match i18n_ref {
                Some(i18n) => AppShellState::new_with_i18n(i18n),
                None => AppShellState::new(),
            };
        }
    }

    fn reset_auxiliary_state(&mut self) {
        self.auxiliary_view = None;
        self.show_branch_dropdown = false;
        self.toolbar_remote_menu = None;
        self.toast_notification = None;
        self.commit_dialog = CommitDialogState::default();
        self.branch_popup = BranchPopupState::default();
        self.pending_commit_action = None;
        self.history_view = HistoryState::default();
        self.history_commit_diff_popup = None;
        self.remote_dialog = RemoteDialogState::default();
        self.tag_dialog = TagDialogState::default();
        self.stash_panel = StashPanelState::default();
        self.rebase_editor = RebaseEditorState::default();
    }

    fn sync_selected_conflict_resolver(&mut self) {
        let diff = self
            .conflict_merge_index
            .and_then(|index| self.conflict_files.get(index))
            .cloned();
        self.conflict_resolver = diff.clone().map(ConflictResolver::new);
        self.merge_editor = diff.map(|d| {
            let model = d.to_merge_editor_model();
            crate::widgets::merge_editor::MergeEditorState::new(model)
        });
    }

    fn preferred_change_path(&self) -> Option<String> {
        self.unstaged_changes
            .first()
            .or_else(|| self.untracked_files.first())
            .or_else(|| self.staged_changes.first())
            .map(|change| change.path.clone())
    }

    fn sync_status_surface(&mut self, i18n: Option<&I18n>) {
        self.shell.status_surface = if let Some(feedback) = self.feedback.as_ref() {
            LightweightStatusSurface {
                message: Some(feedback.title.clone()),
                detail: feedback.detail.clone(),
                severity: match feedback.level {
                    FeedbackLevel::Info | FeedbackLevel::Loading | FeedbackLevel::Empty => {
                        StatusSeverity::Info
                    }
                    FeedbackLevel::Success => StatusSeverity::Success,
                    FeedbackLevel::Warning => StatusSeverity::Warning,
                    FeedbackLevel::Error => StatusSeverity::Error,
                },
                persistence: if feedback.sticky {
                    StatusPersistence::StickyUntilDismissed
                } else {
                    StatusPersistence::Ephemeral
                },
                placement: if matches!(
                    feedback.level,
                    FeedbackLevel::Warning | FeedbackLevel::Error | FeedbackLevel::Loading
                ) {
                    StatusPlacement::Banner
                } else {
                    StatusPlacement::StatusBar
                },
                emphasis: match feedback.level {
                    FeedbackLevel::Warning | FeedbackLevel::Loading => StatusEmphasis::Medium,
                    FeedbackLevel::Error => StatusEmphasis::High,
                    _ => StatusEmphasis::Low,
                },
            }
        } else if self.current_repository.is_none() {
            LightweightStatusSurface {
                message: Some(i18n.map_or_else(|| "No repository opened".to_string(), |i| i.no_repo_status.to_string())),
                detail: Some(i18n.map_or_else(|| "Select a repository to enter the workspace.".to_string(), |i| i.no_repo_status_detail.to_string())),
                severity: StatusSeverity::Info,
                persistence: StatusPersistence::Ephemeral,
                placement: StatusPlacement::StatusBar,
                emphasis: StatusEmphasis::Low,
            }
        } else if self.has_conflicts() && self.shell.active_section != ShellSection::Conflicts {
            LightweightStatusSurface {
                message: Some(i18n.map_or_else(|| "Conflicts present".to_string(), |i| i.has_conflicts_status.to_string())),
                detail: Some(i18n.map_or_else(|| "Handle conflicts before other Git operations.".to_string(), |i| i.handle_conflicts_first.to_string())),
                severity: StatusSeverity::Warning,
                persistence: StatusPersistence::StickyUntilDismissed,
                placement: StatusPlacement::StatusBar,
                emphasis: StatusEmphasis::Medium,
            }
        } else if self.workspace_change_count() == 0 {
            LightweightStatusSurface {
                message: Some(i18n.map_or_else(|| "Workspace clean".to_string(), |i| i.workspace_clean.to_string())),
                detail: None,
                severity: StatusSeverity::Info,
                persistence: StatusPersistence::Ephemeral,
                placement: StatusPlacement::StatusBar,
                emphasis: StatusEmphasis::Low,
            }
        } else {
            LightweightStatusSurface {
                message: Some(i18n.map_or_else(
                    || format!("{} changes", self.workspace_change_count()),
                    |i| i.n_changes_count_fmt.replace("{}", &self.workspace_change_count().to_string()),
                )),
                detail: self.selected_change_path.clone(),
                severity: StatusSeverity::Info,
                persistence: StatusPersistence::Ephemeral,
                placement: StatusPlacement::StatusBar,
                emphasis: StatusEmphasis::Low,
            }
        };
    }

    fn remember_project(&mut self, project: ProjectEntry) {
        self.project_history
            .retain(|entry| entry.path != project.path);
        self.project_history.insert(0, project);
        self.project_history.truncate(MAX_PROJECT_HISTORY);
    }

    fn persist_workspace_memory(&self, last_open_repository: Option<&Path>) {
        let mut memory = PersistedWorkspaceMemory {
            last_open_repository: last_open_repository.map(Path::to_path_buf),
            recent_paths: self
                .project_history
                .iter()
                .map(|entry| entry.path.clone())
                .collect(),
        };
        memory.normalize();

        if let Err(error) = memory.save() {
            warn!("Failed to persist workspace memory: {}", error);
        }
    }

    pub fn should_auto_refresh_workspace(&self, now: Instant) -> bool {
        let _ = now;
        self.current_repository.is_some()
            && !self.is_loading
            && !self.auto_refresh_suspended()
            && self.auto_refresh.workspace_refresh_pending
    }

    pub fn should_auto_check_remote(&self, now: Instant) -> bool {
        self.current_repository.is_some()
            && !self.is_loading
            && !self.auto_refresh_suspended()
            && self.auto_refresh.remote_check_in_flight_for.is_none()
            && self
                .auto_refresh
                .last_remote_check_at
                .is_none_or(|last| now.duration_since(last) >= AUTO_REMOTE_CHECK_INTERVAL)
    }

    pub fn auto_refresh_suspended(&self) -> bool {
        self.auxiliary_view.is_some()
            || self.toolbar_remote_menu.is_some()
            || self.view_mode == ViewMode::ConflictResolver
    }

    pub fn mark_workspace_refreshed(&mut self, now: Instant) {
        self.auto_refresh.workspace_refresh_pending = false;
        self.auto_refresh.last_workspace_refresh_at = Some(now);
    }

    pub fn mark_workspace_refresh_pending(&mut self) {
        self.auto_refresh.workspace_refresh_pending = true;
    }

    pub fn begin_auto_remote_check(&mut self, repo_path: &Path, now: Instant) {
        self.auto_refresh.last_remote_check_at = Some(now);
        self.auto_refresh.remote_check_in_flight_for = Some(repo_path.to_path_buf());
    }

    pub fn finish_auto_remote_check(&mut self, repo_path: &Path, now: Instant) {
        if self
            .auto_refresh
            .remote_check_in_flight_for
            .as_deref()
            .is_some_and(|current| current == repo_path)
        {
            self.auto_refresh.last_remote_check_at = Some(now);
            self.auto_refresh.remote_check_in_flight_for = None;
        }
    }

    pub fn auto_remote_check_target_matches(&self, repo_path: &Path) -> bool {
        self.auto_refresh
            .remote_check_in_flight_for
            .as_deref()
            .is_some_and(|current| current == repo_path)
    }

    fn reset_auto_refresh_state(&mut self) {
        self.auto_refresh = AutoRefreshState::default();
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

fn sync_label(status: &SyncStatus) -> String {
    match status {
        SyncStatus::Ahead(count) => format!("↑{count}"),
        SyncStatus::Behind(count) => format!("↓{count}"),
        SyncStatus::Diverged { ahead, behind } => format!("↕{ahead}/{behind}"),
        SyncStatus::Synced => "✓".to_string(),
        SyncStatus::NoUpstream => "○".to_string(),
        SyncStatus::Unknown => "?".to_string(),
    }
}

fn auxiliary_label(view: AuxiliaryView, i18n: Option<&I18n>) -> String {
    match (view, i18n) {
        (AuxiliaryView::Commit, Some(i)) => i.aux_commit.to_string(),
        (AuxiliaryView::Branches, Some(i)) => i.aux_branches.to_string(),
        (AuxiliaryView::History, Some(i)) => i.aux_history.to_string(),
        (AuxiliaryView::Remotes, Some(i)) => i.aux_remotes.to_string(),
        (AuxiliaryView::Tags, Some(i)) => i.aux_tags.to_string(),
        (AuxiliaryView::Stashes, Some(i)) => i.aux_stashes.to_string(),
        (AuxiliaryView::Rebase, _) => "Rebase".to_string(),
        (AuxiliaryView::Worktrees, Some(i)) => i.aux_worktrees.to_string(),
        (AuxiliaryView::Settings, Some(i)) => i.aux_settings.to_string(),
        (AuxiliaryView::Commit, None) => "Commit".to_string(),
        (AuxiliaryView::Branches, None) => "Branches".to_string(),
        (AuxiliaryView::History, None) => "History".to_string(),
        (AuxiliaryView::Remotes, None) => "Remotes".to_string(),
        (AuxiliaryView::Tags, None) => "Tags".to_string(),
        (AuxiliaryView::Stashes, None) => "Stashes".to_string(),
        (AuxiliaryView::Worktrees, None) => "Worktrees".to_string(),
        (AuxiliaryView::Settings, None) => "Settings".to_string(),
    }
}

pub fn is_docked_auxiliary_view(_view: AuxiliaryView) -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::{AppState, PersistedWorkspaceMemory, ProjectEntry, RecoveryAction, ShellSection};
    use crate::i18n::EN;
    use git_core::diff::{Diff, DiffHunk, DiffLine, DiffLineOrigin, FileDiff};
    use std::path::PathBuf;
    use std::time::{Duration, Instant};
    use tempfile::tempdir;

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

    #[test]
    fn remember_project_moves_active_project_to_front() {
        let mut state = AppState::new();

        state.remember_project(ProjectEntry {
            name: "alpha".to_string(),
            path: PathBuf::from("/tmp/alpha"),
        });
        state.remember_project(ProjectEntry {
            name: "beta".to_string(),
            path: PathBuf::from("/tmp/beta"),
        });
        state.remember_project(ProjectEntry {
            name: "alpha".to_string(),
            path: PathBuf::from("/tmp/alpha"),
        });

        assert_eq!(state.project_history.len(), 2);
        assert_eq!(state.project_history[0].name, "alpha");
        assert_eq!(state.project_history[1].name, "beta");
    }

    #[test]
    fn persisted_workspace_memory_roundtrips_last_and_recent_projects() {
        let temp_dir = tempdir().expect("temp dir");
        let state_path = temp_dir.path().join("workspace-memory-v1.txt");

        let original = PersistedWorkspaceMemory {
            last_open_repository: Some(PathBuf::from("/tmp/current")),
            recent_paths: vec![PathBuf::from("/tmp/current"), PathBuf::from("/tmp/other")],
        };

        original.save_to_path(&state_path).expect("save memory");
        let loaded = PersistedWorkspaceMemory::load_from_path(&state_path);

        assert_eq!(loaded, original);
    }

    #[test]
    fn persisted_workspace_memory_normalizes_duplicates() {
        let loaded = PersistedWorkspaceMemory::parse(
            "last\t/tmp/current\nrecent\t/tmp/other\nrecent\t/tmp/current\nrecent\t/tmp/other\n",
        );

        assert_eq!(
            loaded.last_open_repository,
            Some(PathBuf::from("/tmp/current"))
        );
        assert_eq!(
            loaded.recent_paths,
            vec![PathBuf::from("/tmp/current"), PathBuf::from("/tmp/other")]
        );
    }

    #[test]
    fn auto_refresh_intervals_gate_repeated_checks() {
        let mut state = AppState::new();
        let temp_dir = tempdir().expect("temp dir");
        let repo_path = temp_dir.path().to_path_buf();
        let now = Instant::now();

        state.current_repository = None;
        assert!(!state.should_auto_refresh_workspace(now));
        assert!(!state.should_auto_check_remote(now));

        state.auto_refresh.workspace_refresh_pending = true;
        state.auto_refresh.last_remote_check_at = Some(now - Duration::from_secs(91));
        state.current_repository =
            Some(git_core::Repository::init(&repo_path).expect("repository should initialize"));

        assert!(state.should_auto_refresh_workspace(now));
        assert!(state.should_auto_check_remote(now));

        state.auxiliary_view = Some(super::AuxiliaryView::Commit);
        assert!(!state.should_auto_refresh_workspace(now));
        assert!(!state.should_auto_check_remote(now));
        state.auxiliary_view = None;

        state.mark_workspace_refreshed(now);
        state.begin_auto_remote_check(&repo_path, now);

        assert!(!state.should_auto_refresh_workspace(now));
        assert!(!state.should_auto_check_remote(now));
        assert!(state.auto_remote_check_target_matches(&repo_path));

        state.finish_auto_remote_check(&repo_path, now + Duration::from_secs(1));
        assert!(!state.auto_remote_check_target_matches(&repo_path));
    }

    #[test]
    fn repository_navigation_items_keep_changes_as_workspace_home() {
        let temp_dir = tempdir().expect("temp dir");
        let repo =
            git_core::Repository::init(temp_dir.path()).expect("repository should initialize");
        let mut state = AppState::new();

        state.set_repository(repo, &EN);

        let items = state.navigation_items();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].section, ShellSection::Changes);
        assert_eq!(items[1].section, ShellSection::Conflicts);
        assert!(items[0].enabled);
        assert!(!items[1].enabled);
    }

    #[test]
    fn repository_navigation_never_reopens_legacy_overview_home() {
        let temp_dir = tempdir().expect("temp dir");
        let repo =
            git_core::Repository::init(temp_dir.path()).expect("repository should initialize");
        let mut state = AppState::new();

        state.set_repository(repo, &EN);
        state.navigate_to(ShellSection::Welcome, &EN);

        assert_eq!(state.shell.active_section, ShellSection::Changes);
        assert_eq!(state.view_mode, super::ViewMode::Repository);
    }

    #[test]
    fn history_view_keeps_repository_context_in_shell_metadata() {
        let temp_dir = tempdir().expect("temp dir");
        let repo =
            git_core::Repository::init(temp_dir.path()).expect("repository should initialize");
        let expected_title = repo.name();
        let expected_subtitle = repo.current_branch_display();
        let mut state = AppState::new();

        state.set_repository(repo, &EN);
        state.switch_git_tool_window_tab(super::GitToolWindowTab::Log, &EN);

        assert_eq!(state.shell.title, expected_title);
        assert_eq!(state.shell.subtitle, expected_subtitle);
        assert_eq!(state.shell.chrome.tool_window_title, None);
    }

    #[test]
    fn clean_changes_feedback_recovers_with_refresh_instead_of_overview() {
        let temp_dir = tempdir().expect("temp dir");
        let repo =
            git_core::Repository::init(temp_dir.path()).expect("repository should initialize");
        let mut state = AppState::new();

        state.set_repository(repo, &EN);
        state.set_empty("Workspace clean", None, "shell.changes");

        let next_step = state.feedback_next_step(&EN).expect("expected next step");
        assert_eq!(next_step.action, RecoveryAction::Refresh);
    }

    #[test]
    fn git_tool_window_tab_switching_clears_auxiliary_view() {
        let temp_dir = tempdir().expect("temp dir");
        let repo =
            git_core::Repository::init(temp_dir.path()).expect("repository should initialize");
        let mut state = AppState::new();
        state.set_repository(repo, &EN);
        state.auxiliary_view = Some(super::AuxiliaryView::Branches);
        state.switch_git_tool_window_tab(super::GitToolWindowTab::Log, &EN);
        assert!(state.auxiliary_view.is_none());
        assert_eq!(
            state.shell.git_tool_window_tab,
            super::GitToolWindowTab::Log
        );
    }

    #[test]
    fn opening_branches_auxiliary_uses_dropdown_instead_of_legacy_view() {
        let temp_dir = tempdir().expect("temp dir");
        let repo =
            git_core::Repository::init(temp_dir.path()).expect("repository should initialize");
        let mut state = AppState::new();
        state.set_repository(repo, &EN);

        state.open_auxiliary_view(super::AuxiliaryView::Branches, &EN);

        assert!(state.auxiliary_view.is_none());
        assert!(state.show_branch_dropdown);
        assert_eq!(state.view_mode, super::ViewMode::Repository);
    }

    #[test]
    fn close_conflict_resolver_returns_to_repository_and_changes() {
        let temp_dir = tempdir().expect("temp dir");
        let repo =
            git_core::Repository::init(temp_dir.path()).expect("repository should initialize");
        let mut state = AppState::new();
        state.set_repository(repo, &EN);
        state.view_mode = super::ViewMode::ConflictResolver;
        state.shell.active_section = super::ShellSection::Conflicts;
        state.close_conflict_resolver(&EN);
        assert_eq!(state.view_mode, super::ViewMode::Repository);
        assert_eq!(state.shell.active_section, super::ShellSection::Changes);
        assert!(state.conflict_files.is_empty());
    }

    #[test]
    fn commit_action_from_log_switches_to_changes_tab() {
        let temp_dir = tempdir().expect("temp dir");
        let repo =
            git_core::Repository::init(temp_dir.path()).expect("repository should initialize");
        let mut state = AppState::new();
        state.set_repository(repo, &EN);
        state.switch_git_tool_window_tab(super::GitToolWindowTab::Log, &EN);
        // Simulate what the toolbar Commit button handler does.
        state.navigate_to(super::ShellSection::Changes, &EN);
        state.switch_git_tool_window_tab(super::GitToolWindowTab::Changes, &EN);
        assert_eq!(
            state.shell.git_tool_window_tab,
            super::GitToolWindowTab::Changes
        );
        assert_eq!(state.view_mode, super::ViewMode::Repository);
    }

    #[test]
    fn log_tab_all_is_not_closable() {
        let tab = super::LogTab::all();
        assert!(!tab.is_closable);
        assert_eq!(tab.label, "All");
        assert!(tab.branch_filter.is_none());
    }

    #[test]
    fn log_tab_for_branch_is_closable() {
        let tab = super::LogTab::for_branch(1, "main".to_string());
        assert!(tab.is_closable);
        assert_eq!(tab.label, "main");
        assert_eq!(tab.branch_filter, Some("main".to_string()));
    }

    #[test]
    fn file_display_mode_default_is_flat() {
        let mode = super::FileDisplayMode::default();
        assert_eq!(mode, super::FileDisplayMode::Flat);
    }

    #[test]
    fn network_operation_state_can_be_created() {
        let op = super::NetworkOperation {
            label: "Pushing".to_string(),
            progress: Some(0.5),
            status: Some("50%".to_string()),
        };
        assert_eq!(op.progress, Some(0.5));
    }

    #[test]
    fn pull_strategy_default_is_merge() {
        let strategy = super::PullStrategy::default();
        assert_eq!(strategy, super::PullStrategy::Merge);
    }

    #[test]
    fn app_state_new_has_correct_defaults() {
        let state = super::AppState::new();
        assert_eq!(state.file_display_mode, super::FileDisplayMode::Flat);
        assert_eq!(state.log_tabs.len(), 1); // "All" tab
        assert!(!state.log_tabs[0].is_closable);
        assert_eq!(state.active_log_tab, 0);
        assert!(state.log_branches_dashboard_visible);
        assert!(!state.blame_active);
        assert!(state.recent_commit_messages.is_empty());
        assert!(state.network_operation.is_none());
        assert!(state.full_file_preview.is_none());
        assert!(!state.full_file_preview_binary);
        assert!(!state.full_file_preview_truncated);
        assert!(!state.show_project_dropdown);
        assert!(!state.show_branch_dropdown);
    }

    #[test]
    fn dropdowns_default_closed() {
        let state = super::AppState::new();
        assert!(!state.show_project_dropdown);
        assert!(!state.show_branch_dropdown);
    }

    #[test]
    fn log_tab_default_has_empty_filters() {
        let tab = super::LogTab::all();
        assert!(tab.text_filter.is_empty());
        assert!(tab.author_filter.is_none());
        assert!(tab.date_range.is_none());
        assert!(tab.path_filter.is_none());
        assert!(tab.selected_commit.is_none());
        assert_eq!(tab.scroll_offset, 0.0);
    }

    #[test]
    fn opening_auxiliary_view_closes_history_commit_diff_popup() {
        let mut state = AppState::new();
        state.open_history_commit_diff_popup(
            "abc123".to_string(),
            "src/main.rs".to_string(),
            sample_history_file_diff("src/main.rs"),
            None,
        );

        state.open_auxiliary_view(super::AuxiliaryView::Settings, &EN);

        assert!(state.history_commit_diff_popup.is_none());
    }
}
