//! Branch popup view.

use std::collections::BTreeMap;

use crate::theme::{self, BadgeTone, Surface};
use crate::widgets::{self, button, diff_viewer, scrollable, text_input, OptionalPush};
use chrono::DateTime;
use git_core::{
    branch::Branch, commit::CommitInfo, diff::Diff, history::HistoryEntry, rebase, remote,
    InProgressCommitAction, InProgressCommitActionKind, PushCurrentBranchTarget, Repository,
};
use iced::widget::{
    container, mouse_area, opaque, stack, text, Button, Column, Container, Row, Space, Text,
};
use iced::{mouse, Alignment, Background, Border, Color, Element, Length, Theme};

#[derive(Debug, Clone)]
pub enum BranchPopupMessage {
    SelectBranch(String),
    ToggleFolder(String),
    OpenBranchContextMenu(String),
    OpenCommitContextMenu(String),
    CloseBranchContextMenu,
    CloseCommitContextMenu,
    SetSearchQuery(String),
    ClearSearch,
    SelectBranchCommit(String),
    SetNewBranchName(String),
    CreateBranch(String),
    DeleteBranch(String),
    CheckoutBranch(String),
    MergeBranch(String),
    PrepareCreateFromSelected(String),
    PrepareRenameBranch(String),
    SetInlineBranchName(String),
    ConfirmInlineAction,
    CancelInlineAction,
    CheckoutRemoteBranch(String),
    CheckoutAndRebase { branch: String, onto: String },
    CompareWithCurrent { selected: String, current: String },
    CompareWithWorktree(String),
    RebaseCurrentOnto(String),
    FetchRemote(String),
    PushBranch { branch: String, remote: String },
    SetUpstream { branch: String, upstream: String },
    PrepareTagFromCommit(String),
    CopyCommitHash(String),
    ExportCommitPatch(String),
    PrepareCherryPickCommit(String),
    PrepareRevertCommit(String),
    PrepareResetCurrentBranchToCommit(String),
    PreparePushCurrentBranchToCommit(String),
    ConfirmPendingCommitAction,
    CancelPendingCommitAction,
    ContinueInProgressCommitAction,
    AbortInProgressCommitAction,
    OpenConflictList,
    ClearPreview,
    OpenCommit,
    OpenPull,
    OpenPush,
    OpenHistory,
    OpenRemotes,
    OpenTags,
    OpenStashes,
    OpenRebase,
    PrepareDeleteBranch(String),
    ConfirmDeleteBranch,
    CancelDeleteBranch,
    Refresh,
    Close,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetadataDensity {
    Minimal,
    Compact,
}

#[derive(Debug, Clone)]
pub enum InlineBranchAction {
    CreateFromSelected { base: String },
    RenameBranch { branch: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PendingCommitActionKind {
    CherryPick,
    Revert,
    ResetCurrentBranch,
    PushCurrentBranchToCommit,
}

#[derive(Debug, Clone)]
pub enum PendingCommitAction {
    CherryPick { commit_id: String },
    Revert { commit_id: String },
    ResetCurrentBranch { commit_id: String },
    PushCurrentBranchToCommit { target: PushCurrentBranchTarget },
}

impl PendingCommitAction {
    pub fn kind(&self) -> PendingCommitActionKind {
        match self {
            Self::CherryPick { .. } => PendingCommitActionKind::CherryPick,
            Self::Revert { .. } => PendingCommitActionKind::Revert,
            Self::ResetCurrentBranch { .. } => PendingCommitActionKind::ResetCurrentBranch,
            Self::PushCurrentBranchToCommit { .. } => {
                PendingCommitActionKind::PushCurrentBranchToCommit
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct CommitActionConfirmation {
    pub action: PendingCommitAction,
    pub title: String,
    pub summary: String,
    pub impact_items: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct BranchPopupState {
    pub local_branches: Vec<Branch>,
    pub remote_branches: Vec<Branch>,
    pub recent_branches: Vec<Branch>,
    pub selected_branch: Option<String>,
    pub search_query: String,
    pub is_loading: bool,
    pub error: Option<String>,
    pub success_message: Option<String>,
    pub new_branch_name: String,
    pub metadata_density: MetadataDensity,
    pub current_branch_sync_hint: Option<String>,
    pub current_branch_state_hint: Option<String>,
    pub inline_action: Option<InlineBranchAction>,
    pub inline_branch_name: String,
    pub comparison_title: Option<String>,
    pub comparison_summary: Option<String>,
    pub comparison_diff: Option<Diff>,
    pub branch_history_entries: Vec<HistoryEntry>,
    pub selected_branch_commit: Option<String>,
    pub selected_branch_commit_info: Option<CommitInfo>,
    pub pending_commit_action: Option<CommitActionConfirmation>,
    pub in_progress_commit_action: Option<InProgressCommitAction>,
    pub folder_expansion: BTreeMap<String, bool>,
    pub context_menu_branch: Option<String>,
    pub context_menu_commit: Option<String>,
    /// Branch name pending deletion confirmation (with merge check)
    pub pending_delete_branch: Option<String>,
    /// Whether the pending delete branch is not fully merged (shows warning)
    pub pending_delete_not_merged: bool,
}

impl BranchPopupState {
    pub fn new() -> Self {
        Self {
            local_branches: Vec::new(),
            remote_branches: Vec::new(),
            recent_branches: Vec::new(),
            selected_branch: None,
            search_query: String::new(),
            is_loading: false,
            error: None,
            success_message: None,
            new_branch_name: String::new(),
            metadata_density: MetadataDensity::Compact,
            current_branch_sync_hint: None,
            current_branch_state_hint: None,
            inline_action: None,
            inline_branch_name: String::new(),
            comparison_title: None,
            comparison_summary: None,
            comparison_diff: None,
            branch_history_entries: Vec::new(),
            selected_branch_commit: None,
            selected_branch_commit_info: None,
            pending_commit_action: None,
            in_progress_commit_action: None,
            folder_expansion: BTreeMap::new(),
            context_menu_branch: None,
            context_menu_commit: None,
            pending_delete_branch: None,
            pending_delete_not_merged: false,
        }
    }

    pub fn load_branches(&mut self, repo: &Repository) {
        self.search_query = normalize_branch_search_text(&self.search_query);
        self.is_loading = true;
        self.error = None;

        match repo.list_branches() {
            Ok(branches) => {
                self.local_branches = branches
                    .iter()
                    .filter(|branch| !branch.is_remote)
                    .cloned()
                    .collect();
                self.remote_branches = branches
                    .iter()
                    .filter(|branch| branch.is_remote)
                    .cloned()
                    .collect();
                self.recent_branches = self
                    .local_branches
                    .iter()
                    .filter(|branch| !branch.is_head)
                    .take(5)
                    .cloned()
                    .collect();
                if self.selected_branch.as_ref().is_none_or(|selected| {
                    !self
                        .local_branches
                        .iter()
                        .chain(self.remote_branches.iter())
                        .any(|branch| &branch.name == selected)
                }) {
                    self.selected_branch = self
                        .local_branches
                        .iter()
                        .find(|branch| branch.is_head)
                        .or_else(|| self.local_branches.first())
                        .or_else(|| self.remote_branches.first())
                        .map(|branch| branch.name.clone());
                }
                self.metadata_density = MetadataDensity::Minimal;
                self.current_branch_sync_hint = repo.sync_status_hint();
                self.current_branch_state_hint = repo.state_hint();
                self.in_progress_commit_action =
                    git_core::get_in_progress_commit_action(repo).unwrap_or(None);
                self.context_menu_branch = None;
                self.context_menu_commit = None;
                if let Some(selected) = self.selected_branch.clone() {
                    self.ensure_branch_visible(&selected);
                }
                self.load_selected_branch_history(repo);
                self.is_loading = false;
            }
            Err(error) => {
                self.error = Some(format!("加载分支失败: {error}"));
                self.success_message = None;
                self.is_loading = false;
            }
        }
    }

    pub fn clear_transient_context(&mut self) {
        self.inline_action = None;
        self.inline_branch_name.clear();
        self.comparison_title = None;
        self.comparison_summary = None;
        self.comparison_diff = None;
        self.pending_commit_action = None;
        self.context_menu_branch = None;
        self.context_menu_commit = None;
    }

    pub fn select_branch(&mut self, branch_name: String) {
        self.selected_branch = Some(branch_name.clone());
        self.ensure_branch_visible(&branch_name);
        self.clear_transient_context();
        self.error = None;
    }

    pub fn toggle_folder(&mut self, path_key: String) {
        let expanded = self.is_folder_expanded(&path_key);
        self.folder_expansion.insert(path_key, !expanded);
        self.context_menu_branch = None;
    }

    pub fn open_context_menu(&mut self, branch_name: String) {
        self.selected_branch = Some(branch_name.clone());
        self.ensure_branch_visible(&branch_name);
        self.inline_action = None;
        self.inline_branch_name.clear();
        self.error = None;
        self.context_menu_commit = None;
        self.context_menu_branch = Some(branch_name);
    }

    pub fn open_commit_context_menu(&mut self, commit_id: String) {
        self.context_menu_branch = None;
        self.pending_commit_action = None;
        self.error = None;
        self.context_menu_commit = Some(commit_id);
    }

    pub fn close_context_menu(&mut self) {
        self.context_menu_branch = None;
        self.context_menu_commit = None;
    }

    pub fn is_context_menu_open_for(&self, branch_name: &str) -> bool {
        self.context_menu_branch
            .as_deref()
            .is_some_and(|branch| branch == branch_name)
    }

    pub fn is_commit_context_menu_open_for(&self, commit_id: &str) -> bool {
        self.context_menu_commit
            .as_deref()
            .is_some_and(|current| current == commit_id)
    }

    fn branch_by_name(&self, branch_name: &str) -> Option<&Branch> {
        self.local_branches
            .iter()
            .chain(self.remote_branches.iter())
            .find(|branch| branch.name == branch_name)
    }

    fn ensure_branch_visible(&mut self, branch_name: &str) {
        let branch_context = self
            .branch_by_name(branch_name)
            .map(|branch| (branch.is_remote, branch.name.clone()));

        if let Some((is_remote, branch_name)) = branch_context {
            let section = if is_remote {
                BranchSection::Remote
            } else {
                BranchSection::Local
            };
            self.expand_branch_path(section, &branch_name);
        }
    }

    fn expand_branch_path(&mut self, section: BranchSection, branch_name: &str) {
        let parts: Vec<&str> = branch_name.split('/').collect();
        if parts.len() <= 1 {
            return;
        }

        let mut current_path = String::new();
        for part in &parts[..parts.len() - 1] {
            if !current_path.is_empty() {
                current_path.push('/');
            }
            current_path.push_str(part);
            self.folder_expansion
                .insert(folder_key(section, &current_path), true);
        }
    }

    fn is_folder_expanded(&self, path_key: &str) -> bool {
        self.folder_expansion
            .get(path_key)
            .copied()
            .unwrap_or_else(|| self.default_folder_expansion(path_key))
    }

    fn default_folder_expansion(&self, path_key: &str) -> bool {
        !self.search_query.trim().is_empty() || folder_depth(path_key) <= 2
    }

    pub fn prepare_create_from_selected(&mut self, base: String) {
        self.inline_action = Some(InlineBranchAction::CreateFromSelected { base: base.clone() });
        self.inline_branch_name = format!("{}-copy", branch_leaf_name(&base));
        self.error = None;
    }

    pub fn prepare_rename_branch(&mut self, branch: String) {
        self.inline_action = Some(InlineBranchAction::RenameBranch {
            branch: branch.clone(),
        });
        self.inline_branch_name = branch;
        self.error = None;
    }

    pub fn confirm_inline_action(&mut self, repo: &Repository) {
        match self.inline_action.clone() {
            Some(InlineBranchAction::CreateFromSelected { base }) => {
                self.create_branch_from_selected(repo, &base, self.inline_branch_name.clone())
            }
            Some(InlineBranchAction::RenameBranch { branch }) => {
                self.rename_branch(repo, branch, self.inline_branch_name.clone())
            }
            None => {
                self.error = Some("没有待执行的分支操作".to_string());
                self.success_message = None;
            }
        }
    }

    pub fn cancel_inline_action(&mut self) {
        self.inline_action = None;
        self.inline_branch_name.clear();
        self.error = None;
    }

    pub fn create_branch(&mut self, repo: &Repository, name: String) {
        let name = name.trim().to_string();
        if name.is_empty() {
            self.error = Some("分支名称不能为空".to_string());
            self.success_message = None;
            return;
        }

        let head_oid = match self
            .local_branches
            .iter()
            .find(|branch| branch.is_head)
            .map(|branch| branch.oid.clone())
        {
            Some(oid) if !oid.is_empty() => oid,
            _ => {
                self.error = Some("当前 HEAD 不可用，无法创建分支。".to_string());
                self.success_message = None;
                return;
            }
        };

        self.is_loading = true;
        self.error = None;
        self.success_message = None;

        match repo.create_branch(&name, &head_oid) {
            Ok(_) => {
                self.selected_branch = Some(name.clone());
                self.new_branch_name.clear();
                self.success_message = Some(format!("已创建 {name}"));
                self.load_branches(repo);
            }
            Err(error) => {
                self.error = Some(format!("创建分支失败: {error}"));
                self.is_loading = false;
            }
        }
    }

    pub fn create_branch_from_selected(&mut self, repo: &Repository, base: &str, name: String) {
        let name = name.trim().to_string();
        if name.is_empty() {
            self.error = Some("新分支名称不能为空".to_string());
            self.success_message = None;
            return;
        }

        self.is_loading = true;
        self.error = None;
        self.success_message = None;

        match repo.create_branch_from_start_point(&name, base) {
            Ok(_) => {
                self.selected_branch = Some(name.clone());
                self.inline_action = None;
                self.inline_branch_name.clear();
                self.success_message = Some(format!("已从 {base} 创建 {name}"));
                self.load_branches(repo);
            }
            Err(error) => {
                self.error = Some(format!("基于 {base} 创建分支失败: {error}"));
                self.is_loading = false;
            }
        }
    }

    pub fn rename_branch(&mut self, repo: &Repository, old_name: String, new_name: String) {
        let new_name = new_name.trim().to_string();
        if new_name.is_empty() {
            self.error = Some("新的分支名称不能为空".to_string());
            self.success_message = None;
            return;
        }

        self.is_loading = true;
        self.error = None;
        self.success_message = None;

        match repo.rename_branch(&old_name, &new_name) {
            Ok(_) => {
                self.selected_branch = Some(new_name.clone());
                self.inline_action = None;
                self.inline_branch_name.clear();
                self.success_message = Some(format!("已将 {old_name} 重命名为 {new_name}"));
                self.load_branches(repo);
            }
            Err(error) => {
                self.error = Some(format!("重命名分支失败: {error}"));
                self.is_loading = false;
            }
        }
    }

    /// Prepare a branch for deletion by checking if it's fully merged.
    /// Sets `pending_delete_branch` and `pending_delete_not_merged` for the confirmation dialog.
    pub fn prepare_delete_branch(&mut self, repo: &Repository, name: String) {
        let can_delete = self
            .local_branches
            .iter()
            .any(|branch| branch.name == name && !branch.is_head);

        if !can_delete {
            self.error = Some("只能删除非当前本地分支".to_string());
            return;
        }

        // Check if branch is fully merged into HEAD
        let is_merged = repo.is_branch_merged(&name).unwrap_or(false);

        self.pending_delete_branch = Some(name);
        self.pending_delete_not_merged = !is_merged;
    }

    pub fn delete_branch(&mut self, repo: &Repository, name: String) {
        let can_delete = self
            .local_branches
            .iter()
            .any(|branch| branch.name == name && !branch.is_head);

        if !can_delete {
            self.error = Some("只能删除非当前本地分支".to_string());
            self.success_message = None;
            return;
        }

        self.is_loading = true;
        self.error = None;
        self.success_message = None;

        match repo.delete_branch(&name) {
            Ok(()) => {
                if self.selected_branch.as_deref() == Some(name.as_str()) {
                    self.selected_branch = None;
                }
                self.success_message = Some(format!("已删除 {name}"));
                self.load_branches(repo);
            }
            Err(error) => {
                self.error = Some(format!("删除分支失败: {error}"));
                self.is_loading = false;
            }
        }
    }

    pub fn checkout_branch(&mut self, repo: &Repository, name: String) {
        if self
            .local_branches
            .iter()
            .any(|branch| branch.name == name && branch.is_head)
        {
            self.error = None;
            self.success_message = Some(format!("{name} 已在当前"));
            return;
        }

        self.is_loading = true;
        self.error = None;
        self.success_message = None;

        match repo.checkout_branch(&name) {
            Ok(()) => {
                self.selected_branch = Some(name.clone());
                self.success_message = Some(format!("已切到 {name}"));
                self.load_branches(repo);
            }
            Err(error) => {
                self.error = Some(format!("切换分支失败: {error}"));
                self.is_loading = false;
            }
        }
    }

    pub fn checkout_remote_branch(&mut self, repo: &Repository, remote_ref: String) {
        if !self
            .remote_branches
            .iter()
            .any(|branch| branch.name == remote_ref)
        {
            self.error = Some("只能从远程分支创建本地跟踪分支".to_string());
            self.success_message = None;
            return;
        }

        self.is_loading = true;
        self.error = None;
        self.success_message = None;

        match repo.checkout_remote_branch(&remote_ref) {
            Ok(local_branch_name) => {
                self.selected_branch = Some(local_branch_name.clone());
                self.success_message = Some(format!(
                    "已基于 {remote_ref} 签出本地分支 {local_branch_name}"
                ));
                self.load_branches(repo);
            }
            Err(error) => {
                self.error = Some(format!("签出远程分支失败: {error}"));
                self.is_loading = false;
            }
        }
    }

    pub fn checkout_and_rebase(&mut self, repo: &Repository, name: &str, onto: &str) {
        self.is_loading = true;
        self.error = None;
        self.success_message = None;

        match repo.checkout_branch(name) {
            Ok(()) => match rebase::rebase_start(repo, onto) {
                Ok(_) => {
                    self.selected_branch = Some(name.to_string());
                    self.inline_action = None;
                    self.inline_branch_name.clear();
                    self.success_message = Some(format!("已切到 {name}，并开始变基到 {onto}"));
                    self.load_branches(repo);
                }
                Err(error) => {
                    self.error = Some(format!("切换后开始变基失败: {error}"));
                    self.is_loading = false;
                }
            },
            Err(error) => {
                self.error = Some(format!("切换分支失败: {error}"));
                self.is_loading = false;
            }
        }
    }

    pub fn rebase_current_onto(&mut self, repo: &Repository, onto: &str) {
        self.is_loading = true;
        self.error = None;
        self.success_message = None;

        match rebase::rebase_start(repo, onto) {
            Ok(_) => {
                self.success_message = Some(format!("已开始将当前分支变基到 {onto}"));
                self.load_branches(repo);
            }
            Err(error) => {
                self.error = Some(format!("开始变基失败: {error}"));
                self.is_loading = false;
            }
        }
    }

    pub fn merge_branch(&mut self, repo: &Repository, name: String) {
        if self
            .local_branches
            .iter()
            .any(|branch| branch.name == name && branch.is_head)
        {
            self.error = Some("当前分支不能合并自身".to_string());
            self.success_message = None;
            return;
        }

        self.is_loading = true;
        self.error = None;
        self.success_message = None;

        match repo.merge_branch(&name) {
            Ok(()) => {
                self.success_message = Some(format!("已合并 {name}"));
                self.load_branches(repo);
            }
            Err(error) => {
                if git_core::index::has_conflicts(repo) {
                    self.error = None;
                    self.success_message =
                        Some(format!("合并 {name} 时产生冲突，请继续处理冲突文件"));
                    self.is_loading = false;
                } else {
                    self.error = Some(format!("合并分支失败: {error}"));
                    self.is_loading = false;
                }
            }
        }
    }

    pub fn fetch_remote(&mut self, repo: &Repository, remote_name: &str) {
        self.is_loading = true;
        self.error = None;
        self.success_message = None;

        match remote::fetch(repo, remote_name, None) {
            Ok(()) => {
                self.success_message = Some(format!("已更新远程 {remote_name}"));
                self.load_branches(repo);
            }
            Err(error) => {
                self.error = Some(format!("更新远程失败: {error}"));
                self.is_loading = false;
            }
        }
    }

    pub fn push_branch_to_remote(
        &mut self,
        repo: &Repository,
        remote_name: &str,
        branch_name: &str,
    ) {
        self.is_loading = true;
        self.error = None;
        self.success_message = None;

        match remote::push(repo, remote_name, branch_name, None) {
            Ok(()) => {
                self.success_message = Some(format!("已推送 {branch_name} -> {remote_name}"));
                self.load_branches(repo);
            }
            Err(error) => {
                self.error = Some(format!("推送分支失败: {error}"));
                self.is_loading = false;
            }
        }
    }

    pub fn set_upstream(&mut self, repo: &Repository, branch_name: &str, upstream: &str) {
        self.is_loading = true;
        self.error = None;
        self.success_message = None;

        match repo.set_branch_upstream(branch_name, upstream) {
            Ok(()) => {
                self.success_message = Some(format!("已让 {branch_name} 跟踪 {upstream}"));
                self.load_branches(repo);
            }
            Err(error) => {
                self.error = Some(format!("设置跟踪分支失败: {error}"));
                self.is_loading = false;
            }
        }
    }

    pub fn compare_refs_preview(&mut self, repo: &Repository, left: &str, right: &str) {
        self.is_loading = true;
        self.error = None;
        self.success_message = None;

        match git_core::diff::diff_refs(repo, left, right) {
            Ok(diff) => {
                self.comparison_summary = Some(format_diff_summary(&diff));
                self.comparison_diff = Some(diff);
                self.success_message = Some(format!("已加载 {left} 与 {right} 的比较结果"));
                self.is_loading = false;
            }
            Err(error) => {
                self.error = Some(format!("加载分支比较失败: {error}"));
                self.is_loading = false;
            }
        }
    }

    pub fn compare_ref_to_workdir_preview(&mut self, repo: &Repository, reference: &str) {
        self.is_loading = true;
        self.error = None;
        self.success_message = None;

        match git_core::diff::diff_ref_to_workdir(repo, reference) {
            Ok(diff) => {
                self.comparison_summary = Some(format_diff_summary(&diff));
                self.comparison_diff = Some(diff);
                self.success_message = Some(format!("已加载 {reference} 与工作树的差异"));
                self.is_loading = false;
            }
            Err(error) => {
                self.error = Some(format!("加载工作树差异失败: {error}"));
                self.is_loading = false;
            }
        }
    }

    pub fn clear_preview(&mut self) {
        self.comparison_title = None;
        self.comparison_summary = None;
        self.comparison_diff = None;
        self.error = None;
    }

    pub fn load_selected_branch_history(&mut self, repo: &Repository) {
        let Some(reference) = self.selected_branch.clone() else {
            self.branch_history_entries.clear();
            self.selected_branch_commit = None;
            self.selected_branch_commit_info = None;
            return;
        };

        match git_core::history::get_history_for_ref(repo, &reference, Some(80)) {
            Ok(entries) => {
                self.error = None;
                let previous_selected = self.selected_branch_commit.clone();
                self.branch_history_entries = entries;

                let next_selected = previous_selected
                    .filter(|id| {
                        self.branch_history_entries
                            .iter()
                            .any(|entry| &entry.id == id)
                    })
                    .or_else(|| {
                        self.branch_history_entries
                            .first()
                            .map(|entry| entry.id.clone())
                    });

                if let Some(commit_id) = next_selected {
                    self.select_branch_commit(repo, commit_id);
                } else {
                    self.selected_branch_commit = None;
                    self.selected_branch_commit_info = None;
                }
            }
            Err(error) => {
                self.branch_history_entries.clear();
                self.selected_branch_commit = None;
                self.selected_branch_commit_info = None;
                self.error = Some(format!("加载分支提交失败: {error}"));
            }
        }
    }

    pub fn select_branch_commit(&mut self, repo: &Repository, commit_id: String) {
        self.selected_branch_commit = Some(commit_id.clone());
        self.pending_commit_action = None;
        self.context_menu_commit = None;

        match git_core::commit::get_commit(repo, &commit_id) {
            Ok(info) => {
                self.error = None;
                self.selected_branch_commit_info = Some(info);
            }
            Err(error) => {
                self.selected_branch_commit_info = None;
                self.error = Some(format!("加载提交详情失败: {error}"));
            }
        }
    }

    pub fn prepare_cherry_pick_commit(&mut self, repo: &Repository, commit_id: String) {
        let current_branch = match repo.current_branch() {
            Ok(Some(branch)) => branch,
            Ok(None) => {
                self.error = Some("当前为 detached HEAD，不能直接摘取到当前分支".to_string());
                self.success_message = None;
                return;
            }
            Err(error) => {
                self.error = Some(format!("读取当前分支失败: {error}"));
                self.success_message = None;
                return;
            }
        };

        let info = match git_core::commit::get_commit(repo, &commit_id) {
            Ok(info) => info,
            Err(error) => {
                self.error = Some(format!("读取提交详情失败: {error}"));
                self.success_message = None;
                return;
            }
        };

        if info.parent_ids.len() > 1 {
            self.error = Some("暂不支持直接摘取 merge 提交".to_string());
            self.success_message = None;
            return;
        }

        self.error = None;
        self.success_message = None;
        self.pending_commit_action = Some(CommitActionConfirmation {
            action: PendingCommitAction::CherryPick {
                commit_id: commit_id.clone(),
            },
            title: "摘取该提交".to_string(),
            summary: format!(
                "会把提交 {} 应用到当前分支 {current_branch}，并生成一条新的提交。",
                short_commit_id(&commit_id)
            ),
            impact_items: vec![
                format!("只会修改当前分支 {current_branch}，不会移动原始提交所在分支"),
                format!("提交标题：{}", commit_subject(&info.message)),
                "若内容冲突，仓库会进入 Cherry-pick 处理中状态".to_string(),
            ],
        });
    }

    pub fn prepare_revert_commit(&mut self, repo: &Repository, commit_id: String) {
        let current_branch = match repo.current_branch() {
            Ok(Some(branch)) => branch,
            Ok(None) => {
                self.error = Some("当前为 detached HEAD，不能直接回退提交".to_string());
                self.success_message = None;
                return;
            }
            Err(error) => {
                self.error = Some(format!("读取当前分支失败: {error}"));
                self.success_message = None;
                return;
            }
        };

        let info = match git_core::commit::get_commit(repo, &commit_id) {
            Ok(info) => info,
            Err(error) => {
                self.error = Some(format!("读取提交详情失败: {error}"));
                self.success_message = None;
                return;
            }
        };

        if info.parent_ids.len() > 1 {
            self.error = Some("暂不支持直接回退 merge 提交".to_string());
            self.success_message = None;
            return;
        }

        self.error = None;
        self.success_message = None;
        self.pending_commit_action = Some(CommitActionConfirmation {
            action: PendingCommitAction::Revert {
                commit_id: commit_id.clone(),
            },
            title: "回退该提交".to_string(),
            summary: format!(
                "会在当前分支 {current_branch} 上生成一条新的反向提交，用来撤销 {} 的影响。",
                short_commit_id(&commit_id)
            ),
            impact_items: vec![
                "原始提交仍然会保留在历史里，只会新增一条回退提交".to_string(),
                format!("提交标题：{}", commit_subject(&info.message)),
                "若内容冲突，仓库会进入回退处理中状态".to_string(),
            ],
        });
    }

    pub fn prepare_reset_current_branch_to_commit(&mut self, repo: &Repository, commit_id: String) {
        let current_branch = match repo.current_branch() {
            Ok(Some(branch)) => branch,
            Ok(None) => {
                self.error = Some("当前为 detached HEAD，无法重置当前分支".to_string());
                self.success_message = None;
                return;
            }
            Err(error) => {
                self.error = Some(format!("读取当前分支失败: {error}"));
                self.success_message = None;
                return;
            }
        };

        self.error = None;
        self.success_message = None;
        self.pending_commit_action = Some(CommitActionConfirmation {
            action: PendingCommitAction::ResetCurrentBranch {
                commit_id: commit_id.clone(),
            },
            title: "重置当前分支到这里".to_string(),
            summary: format!(
                "会把当前分支 {current_branch} 直接移动到提交 {}，并同步更新工作区内容。",
                short_commit_id(&commit_id)
            ),
            impact_items: vec![
                "会丢弃当前分支在该提交之后的本地提交引用".to_string(),
                "工作区与暂存区会一起回到所选提交对应的状态".to_string(),
                "若当前仓库还有未提交改动，操作会被阻止".to_string(),
            ],
        });
    }

    pub fn prepare_push_current_branch_to_commit(&mut self, repo: &Repository, commit_id: String) {
        match git_core::resolve_push_current_branch_target(repo, &commit_id) {
            Ok(target) => {
                let mut impact_items = vec![
                    format!(
                        "只会影响当前分支 {} 的上游 {}",
                        target.local_branch_name, target.upstream_ref
                    ),
                    format!(
                        "远端最终会指向提交 {}",
                        short_commit_id(&target.selected_commit)
                    ),
                ];

                if target.requires_force_with_lease {
                    impact_items.push(
                        "这次发布不是快进推送，会使用 force-with-lease 保护远端最新状态"
                            .to_string(),
                    );
                } else {
                    impact_items.push("这次发布可以按快进方式推进远端分支".to_string());
                }

                self.error = None;
                self.success_message = None;
                self.pending_commit_action = Some(CommitActionConfirmation {
                    action: PendingCommitAction::PushCurrentBranchToCommit {
                        target: target.clone(),
                    },
                    title: "推送当前分支到这里".to_string(),
                    summary: format!(
                        "会把当前分支 {} 的上游 {} 发布到提交 {}。",
                        target.local_branch_name,
                        target.upstream_ref,
                        short_commit_id(&target.selected_commit)
                    ),
                    impact_items,
                });
            }
            Err(error) => {
                self.pending_commit_action = None;
                self.error = Some(format!("无法准备“推送到这里”: {error}"));
                self.success_message = None;
            }
        }
    }

    pub fn confirm_pending_commit_action(
        &mut self,
        repo: &Repository,
    ) -> Option<PendingCommitActionKind> {
        let confirmation = self.pending_commit_action.clone()?;
        let kind = confirmation.action.kind();

        self.is_loading = true;
        self.error = None;
        self.success_message = None;

        let (commit_id, result) = match confirmation.action {
            PendingCommitAction::CherryPick { commit_id } => (
                commit_id.clone(),
                git_core::cherry_pick_commit(repo, &commit_id),
            ),
            PendingCommitAction::Revert { commit_id } => {
                (commit_id.clone(), git_core::revert_commit(repo, &commit_id))
            }
            PendingCommitAction::ResetCurrentBranch { commit_id } => (
                commit_id.clone(),
                git_core::reset_current_branch_to_commit(repo, &commit_id),
            ),
            PendingCommitAction::PushCurrentBranchToCommit { target } => (
                target.selected_commit.clone(),
                git_core::push_current_branch_to_commit(repo, &target),
            ),
        };

        match result {
            Ok(()) => {
                self.pending_commit_action = None;
                self.is_loading = false;
                self.success_message = Some(match kind {
                    PendingCommitActionKind::CherryPick => {
                        format!("已把提交 {} 摘取到当前分支", short_commit_id(&commit_id))
                    }
                    PendingCommitActionKind::Revert => {
                        format!("已回退提交 {}", short_commit_id(&commit_id))
                    }
                    PendingCommitActionKind::ResetCurrentBranch => {
                        format!("当前分支已重置到 {}", short_commit_id(&commit_id))
                    }
                    PendingCommitActionKind::PushCurrentBranchToCommit => {
                        format!("已把当前分支发布到 {}", short_commit_id(&commit_id))
                    }
                });
                Some(kind)
            }
            Err(error) => {
                let requires_follow_up = matches!(
                    kind,
                    PendingCommitActionKind::CherryPick | PendingCommitActionKind::Revert
                ) && (git_core::index::has_conflicts(repo)
                    || matches!(
                        repo.get_state(),
                        git_core::repository::RepositoryState::CherryPick
                            | git_core::repository::RepositoryState::Revert
                    ));

                self.pending_commit_action = None;
                self.is_loading = false;

                if requires_follow_up {
                    self.error = None;
                    self.success_message = Some(match kind {
                        PendingCommitActionKind::CherryPick => format!(
                            "摘取提交 {} 时产生冲突，请先处理冲突文件",
                            short_commit_id(&commit_id)
                        ),
                        PendingCommitActionKind::Revert => format!(
                            "回退提交 {} 时产生冲突，请先处理冲突文件",
                            short_commit_id(&commit_id)
                        ),
                        PendingCommitActionKind::ResetCurrentBranch
                        | PendingCommitActionKind::PushCurrentBranchToCommit => unreachable!(),
                    });
                    Some(kind)
                } else {
                    self.error = Some(format!("提交操作失败: {error}"));
                    None
                }
            }
        }
    }

    pub fn cancel_pending_commit_action(&mut self) {
        self.pending_commit_action = None;
        self.error = None;
    }

    pub fn continue_in_progress_commit_action(&mut self, repo: &Repository) {
        let Some(in_progress) = self.in_progress_commit_action.clone() else {
            self.error = Some("当前没有需要继续的提交级 Git 流程".to_string());
            self.success_message = None;
            return;
        };

        self.is_loading = true;
        self.error = None;
        self.success_message = None;

        match git_core::continue_in_progress_commit_action(repo, in_progress.kind) {
            Ok(()) => {
                self.is_loading = false;
                self.success_message = Some(match in_progress.kind {
                    InProgressCommitActionKind::CherryPick => "已继续 Cherry-pick 流程".to_string(),
                    InProgressCommitActionKind::Revert => "已继续回退提交流程".to_string(),
                });
            }
            Err(error) => {
                self.is_loading = false;
                self.error = Some(format!("继续流程失败: {error}"));
            }
        }
    }

    pub fn abort_in_progress_commit_action(&mut self, repo: &Repository) {
        let Some(in_progress) = self.in_progress_commit_action.clone() else {
            self.error = Some("当前没有需要中止的提交级 Git 流程".to_string());
            self.success_message = None;
            return;
        };

        self.is_loading = true;
        self.error = None;
        self.success_message = None;

        match git_core::abort_in_progress_commit_action(repo, in_progress.kind) {
            Ok(()) => {
                self.is_loading = false;
                self.success_message = Some(match in_progress.kind {
                    InProgressCommitActionKind::CherryPick => "已中止当前 Cherry-pick".to_string(),
                    InProgressCommitActionKind::Revert => "已中止当前回退提交流程".to_string(),
                });
            }
            Err(error) => {
                self.is_loading = false;
                self.error = Some(format!("中止流程失败: {error}"));
            }
        }
    }

    fn current_branch(&self) -> Option<&Branch> {
        self.local_branches.iter().find(|branch| branch.is_head)
    }

    pub fn selected_branch_ref(&self) -> Option<&Branch> {
        self.selected_branch
            .as_deref()
            .and_then(|name| self.branch_by_name(name))
    }

    pub fn set_search_query(&mut self, query: String) -> bool {
        let query = normalize_branch_search_text(&query);
        if self.search_query == query {
            return false;
        }

        self.search_query = query;
        self.error = None;

        let mut candidates = self.visible_branch_candidate_names();
        if candidates.is_empty() {
            return false;
        }

        if self.search_query.trim().is_empty() {
            if let Some(selected) = self.selected_branch.clone() {
                if candidates
                    .iter()
                    .any(|branch_name| branch_name == &selected)
                {
                    self.ensure_branch_visible(&selected);
                    return false;
                }
            }

            let fallback = candidates.remove(0);
            self.selected_branch = Some(fallback.clone());
            self.ensure_branch_visible(&fallback);
            return true;
        }

        let query = self.search_query.trim().to_lowercase();
        let exact_match = candidates
            .iter()
            .find(|branch_name| {
                branch_name.to_lowercase() == query
                    || branch_leaf_name(branch_name).to_lowercase() == query
            })
            .cloned();

        if let Some(selected) = self.selected_branch.clone() {
            if candidates
                .iter()
                .any(|branch_name| branch_name == &selected)
            {
                if let Some(exact) = exact_match.as_ref() {
                    if exact != &selected {
                        self.selected_branch = Some(exact.clone());
                        self.ensure_branch_visible(exact);
                        return true;
                    }
                }
                self.ensure_branch_visible(&selected);
                return false;
            }
        }

        let target = exact_match.or_else(|| candidates.first().cloned());

        if let Some(target) = target {
            self.selected_branch = Some(target.clone());
            self.ensure_branch_visible(&target);
            return true;
        }

        false
    }

    fn visible_local_branches(&self) -> Vec<&Branch> {
        self.filter_branches(&self.local_branches)
    }

    fn visible_remote_branches(&self) -> Vec<&Branch> {
        self.filter_branches(&self.remote_branches)
    }

    fn visible_recent_branches(&self) -> Vec<&Branch> {
        self.filter_branches(&self.recent_branches)
    }

    fn visible_branch_candidate_names(&self) -> Vec<String> {
        let mut branches = self.visible_local_branches();
        branches.extend(self.visible_remote_branches());
        branches
            .into_iter()
            .map(|branch| branch.name.clone())
            .collect()
    }

    fn filter_branches<'a>(&'a self, branches: &'a [Branch]) -> Vec<&'a Branch> {
        let query = self.search_query.trim().to_lowercase();
        if query.is_empty() {
            return branches.iter().collect();
        }

        branches
            .iter()
            .filter(|branch| {
                branch.name.to_lowercase().contains(&query)
                    || branch
                        .upstream
                        .as_ref()
                        .is_some_and(|upstream| upstream.to_lowercase().contains(&query))
            })
            .collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BranchSection {
    Local,
    Remote,
}

impl BranchSection {
    fn storage_key(self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Remote => "remote",
        }
    }
}

#[derive(Debug, Default)]
struct BranchTreeFolder<'a> {
    label: String,
    path: String,
    folders: BTreeMap<String, BranchTreeFolder<'a>>,
    branches: Vec<&'a Branch>,
}

impl<'a> BranchTreeFolder<'a> {
    fn root() -> Self {
        Self::default()
    }

    fn folder(label: String, path: String) -> Self {
        Self {
            label,
            path,
            ..Self::default()
        }
    }

    fn insert(&mut self, branch: &'a Branch) {
        let parts: Vec<&str> = branch.name.split('/').collect();
        if parts.len() <= 1 {
            self.branches.push(branch);
            return;
        }

        let mut node = self;
        let mut current_path = String::new();

        for part in &parts[..parts.len() - 1] {
            let next_path = if current_path.is_empty() {
                (*part).to_string()
            } else {
                format!("{current_path}/{part}")
            };

            node = node.folders.entry((*part).to_string()).or_insert_with(|| {
                BranchTreeFolder::folder((*part).to_string(), next_path.clone())
            });
            current_path = next_path;
        }

        node.branches.push(branch);
    }

    fn branch_count(&self) -> usize {
        self.branches.len()
            + self
                .folders
                .values()
                .map(BranchTreeFolder::branch_count)
                .sum::<usize>()
    }
}

fn build_branch_tree<'a>(branches: &[&'a Branch]) -> BranchTreeFolder<'a> {
    let mut root = BranchTreeFolder::root();
    for branch in branches {
        root.insert(branch);
    }
    root
}

impl Default for BranchPopupState {
    fn default() -> Self {
        Self::new()
    }
}

/// Strip invisible / format characters that commonly sneak into the search field via paste or IME,
/// so they do not zero out the branch list while the input looks empty.
fn normalize_branch_search_text(raw: &str) -> String {
    raw.chars()
        .filter(|c| {
            !matches!(
                c,
                '\u{200b}' | '\u{200c}' | '\u{200d}' | '\u{feff}' | '\u{2060}'
            )
        })
        .collect::<String>()
        .trim()
        .to_string()
}

pub fn view(state: &BranchPopupState) -> Element<'_, BranchPopupMessage> {
    let current_branch = state.current_branch();
    let selected_branch = state.selected_branch_ref();
    let local_branches = state.visible_local_branches();
    let remote_branches = state.visible_remote_branches();
    let recent_branches = state.visible_recent_branches();

    let header = Row::new()
        .spacing(theme::spacing::XS)
        .align_y(Alignment::Center)
        .push(Text::new("分支").size(16))
        .push_maybe(current_branch.map(|branch| {
            widgets::info_chip::<BranchPopupMessage>(
                truncate_branch_name(&branch.name),
                BadgeTone::Accent,
            )
        }))
        .push(Space::new().width(Length::Fill))
        .push(button::ghost("关闭", Some(BranchPopupMessage::Close)));

    let compact_toolbar = Container::new(
        Row::new()
            .spacing(theme::spacing::XS)
            .align_y(Alignment::Center)
            // Keep only branch-focused actions in the top area.
            .push(
                Container::new(text_input::search_with_clear(
                    "搜索分支",
                    &state.search_query,
                    BranchPopupMessage::SetSearchQuery,
                    BranchPopupMessage::ClearSearch,
                ))
                .width(Length::FillPortion(5)),
            )
            .push(
                text_input::styled(
                    "新建分支",
                    &state.new_branch_name,
                    BranchPopupMessage::SetNewBranchName,
                )
                .width(Length::FillPortion(3)),
            )
            .push(button::secondary(
                "创建",
                (!state.new_branch_name.trim().is_empty() && !state.is_loading)
                    .then(|| BranchPopupMessage::CreateBranch(state.new_branch_name.clone())),
            ))
            .push(button::ghost("刷新", Some(BranchPopupMessage::Refresh))),
    )
    .padding([8, 10])
    .style(theme::panel_style(Surface::ToolbarField));

    let branch_workspace = Row::new()
        .spacing(theme::spacing::MD)
        .width(Length::Fill)
        .height(Length::Fixed(520.0))
        .push(
            Container::new(build_branch_navigator(
                state,
                recent_branches,
                local_branches,
                remote_branches,
                current_branch,
            ))
            .width(Length::FillPortion(5))
            .height(Length::Fill),
        )
        .push(
            Container::new(build_selected_branch_panel(state, selected_branch))
                .width(Length::FillPortion(4))
                .height(Length::Fill),
        );

    let content = Column::new()
        .spacing(theme::spacing::SM)
        .push(header)
        .push(compact_toolbar)
        .push_maybe(build_status_panel(state))
        .push(branch_workspace);

    Container::new(scrollable::styled(content).height(Length::Fill))
        .padding([8, 10])
        .width(Length::Fill)
        .height(Length::Fill)
        .style(theme::panel_style(Surface::Panel))
        .into()
}

fn build_status_panel<'a>(state: &'a BranchPopupState) -> Option<Element<'a, BranchPopupMessage>> {
    if state.is_loading {
        // IDEA-style: compact loading indicator in-place of full status banner
        return Some(
            Container::new(
                Row::new()
                    .spacing(theme::spacing::SM)
                    .align_y(Alignment::Center)
                    .push(widgets::loading_spinner::<BranchPopupMessage>())
                    .push(
                        Text::new("正在刷新分支列表...")
                            .size(12)
                            .color(theme::darcula::TEXT_SECONDARY),
                    ),
            )
            .padding([8, 12])
            .style(theme::panel_style(Surface::Raised))
            .into(),
        );
    }

    if let Some(error) = state.error.as_ref() {
        return Some(status_panel("失败", error, BadgeTone::Danger));
    }

    if let Some(message) = state.success_message.as_ref() {
        return Some(status_panel("完成", message, BadgeTone::Success));
    }

    None
}

fn status_panel<'a>(
    label: impl Into<String>,
    detail: impl Into<String>,
    tone: BadgeTone,
) -> Element<'a, BranchPopupMessage> {
    widgets::status_banner(label, detail, tone)
}

fn build_branch_navigator<'a>(
    state: &'a BranchPopupState,
    recent_branches: Vec<&'a Branch>,
    local_branches: Vec<&'a Branch>,
    remote_branches: Vec<&'a Branch>,
    current_branch: Option<&'a Branch>,
) -> Element<'a, BranchPopupMessage> {
    let mut branch_lists = Column::new()
        .spacing(theme::spacing::SM)
        .width(Length::Fill);
    if !recent_branches.is_empty() {
        branch_lists = branch_lists.push(build_flat_branch_section(
            "最近分支",
            recent_branches,
            state,
        ));
        // IDEA-style: add separator between recent and local branches
        if !local_branches.is_empty() {
            branch_lists = branch_lists.push(widgets::separator_with_text(Some("本地分支")));
        }
    }

    branch_lists = branch_lists.push(build_tree_branch_section(
        "本地分支",
        BranchSection::Local,
        local_branches,
        state,
    ));
    // IDEA-style: add separator between local and remote branches
    if !remote_branches.is_empty() {
        branch_lists = branch_lists.push(widgets::separator_with_text(Some("远程分支")));
    }
    branch_lists = branch_lists.push(build_tree_branch_section(
        "远程分支",
        BranchSection::Remote,
        remote_branches,
        state,
    ));

    let navigator = Container::new(scrollable::styled(branch_lists).height(Length::Fill))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(theme::panel_style(Surface::Editor));

    stack([
        navigator.into(),
        build_branch_context_menu_overlay(state, current_branch),
    ])
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn build_flat_branch_section<'a>(
    title: &'a str,
    branches: Vec<&'a Branch>,
    state: &'a BranchPopupState,
) -> Element<'a, BranchPopupMessage> {
    let branch_count = branches.len();

    let mut list = Column::new()
        .spacing(theme::spacing::XS)
        .width(Length::Fill);

    if branches.is_empty() {
        list = list.push(
            Text::new("没有匹配项")
                .size(12)
                .color(theme::darcula::TEXT_SECONDARY),
        );
    } else {
        for branch in branches {
            list = list.push(build_branch_row(branch, &branch.name, 0, state));
        }
    }

    build_branch_section_shell(title, branch_count, list)
}

fn build_tree_branch_section<'a>(
    title: &'a str,
    section: BranchSection,
    branches: Vec<&'a Branch>,
    state: &'a BranchPopupState,
) -> Element<'a, BranchPopupMessage> {
    let branch_count = branches.len();
    let list = if branches.is_empty() {
        Column::new().width(Length::Fill).push(
            Text::new("没有匹配项")
                .size(12)
                .color(theme::darcula::TEXT_SECONDARY),
        )
    } else {
        let tree = build_branch_tree(&branches);
        build_tree_branch_nodes(
            Column::new()
                .spacing(theme::spacing::XS)
                .width(Length::Fill),
            state,
            section,
            &tree,
            0,
        )
    };

    build_branch_section_shell(title, branch_count, list)
}

fn build_branch_section_shell<'a>(
    title: &'a str,
    branch_count: usize,
    list: Column<'a, BranchPopupMessage>,
) -> Element<'a, BranchPopupMessage> {
    Container::new(
        Column::new()
            .spacing(theme::spacing::XS)
            .width(Length::Fill)
            .push(
                Row::new()
                    .spacing(theme::spacing::XS)
                    .align_y(Alignment::Center)
                    .push(
                        Text::new(title.to_uppercase())
                            .size(10)
                            .color(theme::darcula::TEXT_SECONDARY),
                    )
                    .push(widgets::info_chip::<BranchPopupMessage>(
                        branch_count.to_string(),
                        BadgeTone::Neutral,
                    )),
            )
            .push(list),
    )
    .padding([10, 12])
    .width(Length::Fill)
    .style(theme::panel_style(Surface::Panel))
    .into()
}

fn build_tree_branch_nodes<'a>(
    mut column: Column<'a, BranchPopupMessage>,
    state: &'a BranchPopupState,
    section: BranchSection,
    folder: &BranchTreeFolder<'a>,
    depth: usize,
) -> Column<'a, BranchPopupMessage> {
    for child in folder.folders.values() {
        let path_key = folder_key(section, &child.path);
        let expanded = state.is_folder_expanded(&path_key);
        column = column.push(build_folder_row(child, depth, expanded, path_key));

        if expanded {
            column = build_tree_branch_nodes(column, state, section, child, depth + 1);
        }
    }

    let mut branches = folder.branches.clone();
    branches.sort_by_key(|branch| {
        (
            !branch.is_head,
            branch_leaf_name(&branch.name).to_lowercase(),
            branch.name.to_lowercase(),
        )
    });

    for branch in branches {
        column = column.push(build_branch_row(
            branch,
            branch_leaf_name(&branch.name),
            depth,
            state,
        ));
    }

    column
}

fn build_folder_row<'a>(
    folder: &BranchTreeFolder<'_>,
    depth: usize,
    expanded: bool,
    path_key: String,
) -> Element<'a, BranchPopupMessage> {
    let row = Container::new(
        Row::new()
            .spacing(theme::spacing::XS)
            .align_y(Alignment::Center)
            .width(Length::Fill)
            .push(tree_indent(depth))
            .push(branch_row_strip(
                expanded.then_some(theme::darcula::SEPARATOR.scale_alpha(0.72)),
            ))
            .push(
                Container::new(Text::new(if expanded { "▾" } else { "▸" }).size(11).color(
                    if expanded {
                        theme::darcula::TEXT_PRIMARY
                    } else {
                        theme::darcula::TEXT_SECONDARY
                    },
                ))
                .width(Length::Fixed(12.0)),
            )
            .push(
                Text::new(folder.label.clone())
                    .size(12)
                    .width(Length::Fill)
                    .wrapping(text::Wrapping::WordOrGlyph)
                    .color(if expanded {
                        theme::darcula::TEXT_PRIMARY
                    } else {
                        theme::darcula::TEXT_SECONDARY
                    }),
            )
            .push(widgets::info_chip::<BranchPopupMessage>(
                folder.branch_count().to_string(),
                BadgeTone::Neutral,
            )),
    )
    .padding([6, 8])
    .width(Length::Fill);

    Button::new(row)
        .width(Length::Fill)
        .style(branch_folder_row_button_style(expanded))
        .on_press(BranchPopupMessage::ToggleFolder(path_key))
        .into()
}

fn build_branch_row<'a>(
    branch: &'a Branch,
    label: &'a str,
    depth: usize,
    state: &'a BranchPopupState,
) -> Element<'a, BranchPopupMessage> {
    let is_selected = state.selected_branch.as_deref() == Some(branch.name.as_str());
    let is_menu_open = state.is_context_menu_open_for(&branch.name);
    let is_current = branch.is_head;

    // Ensure label is not empty and truncate long names (IDEA-style)
    let display_label = if label.is_empty() {
        truncate_branch_name(&branch.name)
    } else {
        truncate_branch_name(label)
    };

    let label_color = if is_menu_open || is_selected {
        theme::darcula::TEXT_PRIMARY
    } else if is_current {
        blend_color(theme::darcula::TEXT_PRIMARY, theme::darcula::SUCCESS, 0.14)
    } else {
        theme::darcula::TEXT_PRIMARY
    };
    let meta_color = if is_menu_open || is_selected {
        blend_color(
            theme::darcula::TEXT_SECONDARY,
            theme::darcula::TEXT_PRIMARY,
            0.22,
        )
    } else {
        theme::darcula::TEXT_SECONDARY
    };
    let strip_color = if is_menu_open {
        Some(theme::darcula::ACCENT)
    } else if is_selected {
        Some(theme::darcula::ACCENT.scale_alpha(0.84))
    } else if is_current {
        Some(theme::darcula::SUCCESS.scale_alpha(0.82))
    } else {
        None
    };

    let row_content = Container::new(
        Column::new()
            .spacing(2)
            .push(
                Row::new()
                    .spacing(theme::spacing::XS)
                    .align_y(Alignment::Center)
                    .width(Length::Fill)
                    .push(tree_indent(depth))
                    // IDEA-style: branch icon prefix (○ current, ● favorite, ◎ regular)
                    .push(
                        Text::new(if branch.is_head { "●" } else { "○" })
                            .size(10)
                            .color(if branch.is_head {
                                theme::darcula::SUCCESS
                            } else {
                                theme::darcula::TEXT_SECONDARY
                            }),
                    )
                    .push(Text::new(display_label).size(12).color(label_color))
                    // IDEA-style: show incoming/outgoing sync indicators with colored arrows
                    .push_maybe(build_sync_indicators(branch))
                    .push_maybe(branch.is_head.then(|| -> Element<'_, BranchPopupMessage> {
                        Container::new(Text::new("当前").size(10).color(theme::darcula::SUCCESS))
                            .padding([2, 5])
                            .style(|_| container::Style {
                                border: Border {
                                    width: 1.0,
                                    color: theme::darcula::SUCCESS.scale_alpha(0.45),
                                    radius: theme::radius::SM.into(),
                                },
                                ..Default::default()
                            })
                            .into()
                    }))
                    .push_maybe((branch.is_remote && !branch.is_head).then(|| {
                        widgets::info_chip::<BranchPopupMessage>("远程", BadgeTone::Neutral)
                    })),
            )
            .push_maybe(branch_meta_summary(branch).map(|meta| {
                Container::new(
                    Row::new()
                        .spacing(theme::spacing::XS)
                        .width(Length::Fill)
                        .push(tree_indent(depth))
                        .push(Text::new(meta).size(10).color(meta_color)),
                )
            })),
    )
    .padding([4, 6])
    .width(Length::Fill);

    // Strip + button in a nested Row so that strip's height(Fill) is resolved
    // against the button's Shrink height (avoids circular Fill dependency inside
    // the primary Row which previously collapsed the Row to zero height).
    let strip_and_button = Row::new()
        .align_y(Alignment::Center)
        .width(Length::Fill)
        .push(branch_row_strip(strip_color))
        .push(
            Container::new(
                Button::new(row_content)
                    .width(Length::Fill)
                    .style(branch_row_button_style(
                        is_selected,
                        is_menu_open,
                        is_current,
                    ))
                    .on_press(BranchPopupMessage::SelectBranch(branch.name.clone())),
            )
            .width(Length::Fill),
        );

    let row = Row::new()
        .spacing(theme::spacing::XS)
        .align_y(Alignment::Center)
        .width(Length::Fill)
        .push(Container::new(strip_and_button).width(Length::Fill))
        .push(build_branch_context_button(
            branch.name.clone(),
            is_menu_open,
        ));

    mouse_area(Container::new(row).width(Length::Fill))
        .on_right_press(BranchPopupMessage::OpenBranchContextMenu(
            branch.name.clone(),
        ))
        .interaction(mouse::Interaction::Pointer)
        .into()
}

fn build_branch_context_menu_overlay<'a>(
    state: &'a BranchPopupState,
    current_branch: Option<&'a Branch>,
) -> Element<'a, BranchPopupMessage> {
    let Some(selected_branch) = state
        .context_menu_branch
        .as_deref()
        .and_then(|name| state.branch_by_name(name))
    else {
        return Space::new().width(Length::Shrink).into();
    };

    let selected_remote_name = inferred_remote_name(state, selected_branch);
    let upstream_ref = inferred_upstream_ref(state, selected_branch);
    let header = Column::new()
        .spacing(theme::spacing::SM)
        .push(
            Row::new()
                .spacing(theme::spacing::XS)
                .align_y(Alignment::Center)
                .push(
                    Column::new()
                        .spacing(2)
                        .width(Length::Fill)
                        .push(
                            Text::new("分支动作".to_uppercase())
                                .size(10)
                                .color(theme::darcula::TEXT_SECONDARY),
                        )
                        .push(
                            Text::new(truncate_branch_name(&selected_branch.name))
                                .size(14)
                                .width(Length::Fill)
                                .wrapping(text::Wrapping::WordOrGlyph),
                        ),
                )
                .push(button::compact_ghost(
                    "关闭",
                    Some(BranchPopupMessage::CloseBranchContextMenu),
                )),
        )
        .push(
            Row::new()
                .spacing(theme::spacing::XS)
                .align_y(Alignment::Center)
                .push(widgets::info_chip::<BranchPopupMessage>(
                    if selected_branch.is_remote {
                        "远程"
                    } else if selected_branch.is_head {
                        "当前"
                    } else {
                        "本地"
                    },
                    if selected_branch.is_remote {
                        BadgeTone::Neutral
                    } else if selected_branch.is_head {
                        BadgeTone::Success
                    } else {
                        BadgeTone::Accent
                    },
                ))
                .push_maybe(selected_remote_name.as_ref().map(|remote| {
                    widgets::info_chip::<BranchPopupMessage>(
                        format!("remote {remote}"),
                        BadgeTone::Neutral,
                    )
                }))
                .push_maybe(upstream_ref.as_ref().map(|_| {
                    widgets::info_chip::<BranchPopupMessage>("已跟踪", BadgeTone::Accent)
                })),
        )
        .push_maybe(branch_meta_summary(selected_branch).map(|meta| {
            Text::new(meta)
                .size(10)
                .width(Length::Fill)
                .wrapping(text::Wrapping::WordOrGlyph)
                .color(theme::darcula::TEXT_SECONDARY)
        }))
        .push_maybe(
            ((!selected_branch.is_remote)
                && (selected_remote_name.is_some() || upstream_ref.is_some()))
            .then(|| {
                let mut parts = Vec::new();
                if let Some(remote) = selected_remote_name.as_ref() {
                    parts.push(format!("默认远程 {remote}"));
                }
                if let Some(upstream) = upstream_ref.as_ref() {
                    parts.push(format!("跟踪 {upstream}"));
                }
                Text::new(parts.join(" · "))
                    .size(10)
                    .width(Length::Fill)
                    .wrapping(text::Wrapping::WordOrGlyph)
                    .color(theme::darcula::TEXT_SECONDARY)
            }),
        );

    let action_groups = build_branch_action_groups(state, selected_branch, current_branch)
        .into_iter()
        .fold(
            Column::new().spacing(theme::spacing::XS),
            |column, group| column.push(group),
        );

    let menu = Container::new(Column::new().spacing(theme::spacing::SM).push(header).push(
        Container::new(scrollable::styled(action_groups).height(Length::Fixed(360.0))),
    ))
    .padding([8, 9])
    .width(Length::Fixed(374.0))
    .style(widgets::menu::panel_style);

    opaque(
        mouse_area(
            Container::new(
                Row::new()
                    .width(Length::Fill)
                    .push(Space::new().width(Length::Fill))
                    .push(menu),
            )
            .padding([10, 14])
            .width(Length::Fill)
            .height(Length::Fill)
            .style(widgets::menu::scrim_style),
        )
        .on_press(BranchPopupMessage::CloseBranchContextMenu),
    )
}

fn build_selected_branch_panel<'a>(
    state: &'a BranchPopupState,
    selected_branch: Option<&'a Branch>,
) -> Element<'a, BranchPopupMessage> {
    let Some(selected_branch) = selected_branch else {
        return widgets::panel_empty_state(
            "分支操作",
            "先从左侧选择一个分支",
            "选择分支后查看详情、差异预览和快捷操作。",
            None,
        );
    };

    let content = Column::new()
        .spacing(theme::spacing::SM)
        .push(build_selected_branch_summary(state, selected_branch))
        .push_maybe(build_in_progress_commit_action_panel(state))
        .push(build_selected_commit_history_panel(state, selected_branch))
        .push_maybe(build_inline_action_panel(state))
        .push(build_selected_commit_detail_panel(state, selected_branch))
        .push_maybe(build_pending_commit_action_panel(state))
        .push_maybe(build_comparison_panel(state));

    let panel = Container::new(scrollable::styled(content).height(Length::Fill))
        .padding([0, 0])
        .width(Length::Fill)
        .height(Length::Fill)
        .style(theme::panel_style(Surface::Editor));

    stack([
        panel.into(),
        build_commit_context_menu_overlay(state, selected_branch),
    ])
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn build_selected_branch_summary<'a>(
    state: &'a BranchPopupState,
    selected_branch: &'a Branch,
) -> Element<'a, BranchPopupMessage> {
    let selected_remote_name = inferred_remote_name(state, selected_branch);
    let upstream_ref = inferred_upstream_ref(state, selected_branch);

    Container::new(
        Column::new()
            .spacing(theme::spacing::XS)
            .push(
                Row::new()
                    .spacing(theme::spacing::XS)
                    .align_y(Alignment::Center)
                    .push(Text::new(truncate_branch_name(&selected_branch.name)).size(14))
                    .push(widgets::info_chip::<BranchPopupMessage>(
                        if selected_branch.is_remote {
                            "远程"
                        } else if selected_branch.is_head {
                            "当前"
                        } else {
                            "本地"
                        },
                        if selected_branch.is_remote {
                            BadgeTone::Neutral
                        } else if selected_branch.is_head {
                            BadgeTone::Success
                        } else {
                            BadgeTone::Accent
                        },
                    )),
            )
            .push_maybe(branch_meta_summary(selected_branch).map(|meta| {
                Text::new(meta)
                    .size(11)
                    .width(Length::Fill)
                    .wrapping(text::Wrapping::WordOrGlyph)
                    .color(theme::darcula::TEXT_SECONDARY)
            }))
            .push_maybe(selected_remote_name.as_ref().map(|remote| {
                Text::new(format!("默认远程：{remote}"))
                    .size(11)
                    .color(theme::darcula::TEXT_SECONDARY)
            }))
            .push_maybe(upstream_ref.as_ref().map(|upstream| {
                Text::new(format!("跟踪关系：{upstream}"))
                    .size(11)
                    .color(theme::darcula::TEXT_SECONDARY)
            })),
    )
    .padding([6, 8])
    .style(theme::panel_style(Surface::Panel))
    .into()
}

fn build_selected_commit_history_panel<'a>(
    state: &'a BranchPopupState,
    selected_branch: &'a Branch,
) -> Element<'a, BranchPopupMessage> {
    let history_count = state.branch_history_entries.len();

    let history_rows = if state.branch_history_entries.is_empty() {
        Column::new().push(
            Text::new("当前没有可显示的提交历史。")
                .size(12)
                .color(theme::darcula::TEXT_SECONDARY),
        )
    } else {
        state
            .branch_history_entries
            .iter()
            .fold(Column::new().spacing(2), |column, entry| {
                let is_selected =
                    state.selected_branch_commit.as_deref() == Some(entry.id.as_str());
                column.push(build_branch_commit_row(state, entry, is_selected))
            })
    };

    Container::new(
        Column::new()
            .spacing(theme::spacing::XS)
            .push(
                Row::new()
                    .spacing(theme::spacing::XS)
                    .align_y(Alignment::Center)
                    .push(Text::new("提交时间线").size(13))
                    .push(widgets::info_chip::<BranchPopupMessage>(
                        history_count.to_string(),
                        BadgeTone::Neutral,
                    ))
                    .push(Space::new().width(Length::Fill))
                    .push_maybe(state.selected_branch_commit.clone().map(|commit_id| {
                        button::ghost(
                            "提交动作",
                            Some(BranchPopupMessage::OpenCommitContextMenu(commit_id)),
                        )
                    }))
                    .push(button::ghost(
                        "分支动作",
                        Some(BranchPopupMessage::OpenBranchContextMenu(
                            selected_branch.name.clone(),
                        )),
                    )),
            )
            .push(
                Text::new("查看此分支的最近提交记录，帮助判断操作基准点。")
                    .size(10)
                    .width(Length::Fill)
                    .wrapping(text::Wrapping::WordOrGlyph)
                    .color(theme::darcula::TEXT_SECONDARY),
            )
            .push(
                scrollable::styled(Container::new(history_rows).width(Length::Fill))
                    .width(Length::Fill)
                    .height(Length::Fixed(220.0)),
            ),
    )
    .padding([6, 8])
    .style(theme::panel_style(Surface::Panel))
    .into()
}

fn build_selected_commit_detail_panel<'a>(
    state: &'a BranchPopupState,
    selected_branch: &'a Branch,
) -> Element<'a, BranchPopupMessage> {
    let Some(info) = state.selected_branch_commit_info.as_ref() else {
        return widgets::panel_empty_state(
            "提交详情",
            "还没有选中提交",
            "从时间线选择一条提交后查看详情和操作入口。",
            None,
        );
    };

    let current_branch = state.current_branch();
    let can_compare_with_current = current_branch
        .map(|branch| branch.name.clone())
        .filter(|name| name != &selected_branch.name);
    let parent_commit_id = selected_commit_parent_in_history(state);
    let child_commit_ids = selected_commit_children_in_history(state);
    let child_commit_id = (child_commit_ids.len() == 1).then(|| child_commit_ids[0].clone());
    let can_prepare_cherry_pick =
        !state.is_loading && current_branch.is_some() && info.parent_ids.len() <= 1;
    let can_prepare_revert =
        !state.is_loading && current_branch.is_some() && info.parent_ids.len() <= 1;
    let can_reset_current_branch =
        !state.is_loading && selected_branch.is_head && info.id != selected_branch.oid;
    let can_push_current_branch_to_here =
        can_reset_current_branch && current_branch.is_some_and(|branch| branch.upstream.is_some());

    let utility_row = Row::new()
        .spacing(theme::spacing::XS)
        .push(button::secondary(
            "提交动作",
            (!state.is_loading)
                .then_some(BranchPopupMessage::OpenCommitContextMenu(info.id.clone())),
        ))
        .push(button::ghost(
            "复制哈希",
            (!state.is_loading).then_some(BranchPopupMessage::CopyCommitHash(info.id.clone())),
        ))
        .push(button::ghost(
            "导出 Patch",
            (!state.is_loading).then_some(BranchPopupMessage::ExportCommitPatch(info.id.clone())),
        ))
        .push(button::ghost(
            "父提交",
            parent_commit_id
                .clone()
                .map(BranchPopupMessage::SelectBranchCommit),
        ))
        .push(button::ghost(
            "子提交",
            child_commit_id.map(BranchPopupMessage::SelectBranchCommit),
        ));
    let mut branch_row = Row::new()
        .spacing(theme::spacing::XS)
        .push(button::secondary(
            "从该提交建分支",
            Some(BranchPopupMessage::PrepareCreateFromSelected(
                info.id.clone(),
            )),
        ))
        .push(button::ghost(
            "给该提交打标签",
            Some(BranchPopupMessage::PrepareTagFromCommit(info.id.clone())),
        ))
        .push(button::ghost(
            "查看与工作树差异",
            Some(BranchPopupMessage::CompareWithWorktree(info.id.clone())),
        ));

    if let Some(current) = can_compare_with_current {
        branch_row = branch_row.push(button::ghost(
            "与当前分支比较",
            Some(BranchPopupMessage::CompareWithCurrent {
                selected: info.id.clone(),
                current,
            }),
        ));
    }

    let mutation_row = Row::new()
        .spacing(theme::spacing::XS)
        .push(button::ghost(
            "Cherry-pick",
            can_prepare_cherry_pick
                .then_some(BranchPopupMessage::PrepareCherryPickCommit(info.id.clone())),
        ))
        .push(button::ghost(
            "Revert",
            can_prepare_revert.then_some(BranchPopupMessage::PrepareRevertCommit(info.id.clone())),
        ))
        .push(button::warning(
            "重置到这里",
            can_reset_current_branch.then_some(
                BranchPopupMessage::PrepareResetCurrentBranchToCommit(info.id.clone()),
            ),
        ))
        .push(button::warning(
            "推送到这里",
            can_push_current_branch_to_here.then_some(
                BranchPopupMessage::PreparePushCurrentBranchToCommit(info.id.clone()),
            ),
        ));

    let mut action_notes = Vec::new();
    if info.parent_ids.len() > 1 {
        action_notes.push("当前是 merge 提交，暂不支持直接 Cherry-pick / Revert".to_string());
    }
    if parent_commit_id.is_none() && !info.parent_ids.is_empty() {
        action_notes.push("父提交不在当前已加载范围里，可直接点击时间线继续查看".to_string());
    }
    if child_commit_ids.len() > 1 {
        action_notes.push("检测到多个子提交，请直接在时间线里选择目标提交".to_string());
    }
    if selected_branch.is_head && current_branch.is_some_and(|branch| branch.upstream.is_none()) {
        action_notes.push("当前分支还没有上游，暂时不能“推送到这里”".to_string());
    }
    if !selected_branch.is_head {
        action_notes.push("“重置到这里 / 推送到这里”只围绕当前分支启用".to_string());
    }

    Container::new(
        Column::new()
            .spacing(theme::spacing::SM)
            .push(widgets::section_header(
                "提交详情",
                "当前选中提交",
                "可以直接基于这条提交新建分支、打标签，或继续比较差异。",
            ))
            .push(
                Row::new()
                    .spacing(theme::spacing::XS)
                    .align_y(Alignment::Center)
                    .push(widgets::info_chip::<BranchPopupMessage>(
                        format!("提交 {}", short_commit_id(&info.id)),
                        BadgeTone::Accent,
                    ))
                    .push(widgets::info_chip::<BranchPopupMessage>(
                        selected_branch.name.clone(),
                        if selected_branch.is_head {
                            BadgeTone::Success
                        } else {
                            BadgeTone::Neutral
                        },
                    )),
            )
            .push(
                Text::new(commit_subject(&info.message))
                    .size(13)
                    .width(Length::Fill)
                    .wrapping(text::Wrapping::WordOrGlyph),
            )
            .push(
                Text::new(format!(
                    "{} <{}> · {} · 父提交 {} 个",
                    info.author_name,
                    info.author_email,
                    format_timestamp(info.author_time),
                    info.parent_ids.len()
                ))
                .size(11)
                .width(Length::Fill)
                .wrapping(text::Wrapping::WordOrGlyph)
                .color(theme::darcula::TEXT_SECONDARY),
            )
            .push(
                scrollable::styled(
                    Text::new(&info.message)
                        .size(12)
                        .width(Length::Fill)
                        .wrapping(text::Wrapping::WordOrGlyph),
                )
                .height(Length::Fixed(120.0)),
            )
            .push(scrollable::styled_horizontal(utility_row).width(Length::Fill))
            .push(scrollable::styled_horizontal(branch_row).width(Length::Fill))
            .push(scrollable::styled_horizontal(mutation_row).width(Length::Fill))
            .push_maybe((!action_notes.is_empty()).then(|| {
                Text::new(action_notes.join(" · "))
                    .size(10)
                    .width(Length::Fill)
                    .wrapping(text::Wrapping::WordOrGlyph)
                    .color(theme::darcula::TEXT_SECONDARY)
            })),
    )
    .padding([8, 10])
    .style(theme::panel_style(Surface::Panel))
    .into()
}

fn build_pending_commit_action_panel<'a>(
    state: &'a BranchPopupState,
) -> Option<Element<'a, BranchPopupMessage>> {
    let confirmation = state.pending_commit_action.as_ref()?;

    let impact_rows =
        confirmation
            .impact_items
            .iter()
            .fold(Column::new().spacing(4), |column, item| {
                column.push(
                    Row::new()
                        .spacing(theme::spacing::XS)
                        .push(
                            Text::new("•")
                                .size(11)
                                .color(theme::darcula::TEXT_SECONDARY),
                        )
                        .push(
                            Text::new(item)
                                .size(11)
                                .width(Length::Fill)
                                .wrapping(text::Wrapping::WordOrGlyph)
                                .color(theme::darcula::TEXT_SECONDARY),
                        ),
                )
            });

    Some(
        Container::new(
            Column::new()
                .spacing(theme::spacing::SM)
                .push(widgets::section_header(
                    "确认",
                    &confirmation.title,
                    "危险动作先讲清影响范围，再决定是否继续。",
                ))
                .push(
                    Text::new(&confirmation.summary)
                        .size(12)
                        .width(Length::Fill)
                        .wrapping(text::Wrapping::WordOrGlyph),
                )
                .push(impact_rows)
                .push(
                    Row::new()
                        .spacing(theme::spacing::XS)
                        .push(button::warning(
                            "继续执行",
                            (!state.is_loading)
                                .then_some(BranchPopupMessage::ConfirmPendingCommitAction),
                        ))
                        .push(button::ghost(
                            "取消",
                            (!state.is_loading)
                                .then_some(BranchPopupMessage::CancelPendingCommitAction),
                        )),
                ),
        )
        .padding([8, 10])
        .style(theme::panel_style(Surface::Selection))
        .into(),
    )
}

fn build_in_progress_commit_action_panel<'a>(
    state: &'a BranchPopupState,
) -> Option<Element<'a, BranchPopupMessage>> {
    let in_progress = state.in_progress_commit_action.as_ref()?;
    let label = match in_progress.kind {
        InProgressCommitActionKind::CherryPick => "Cherry-pick",
        InProgressCommitActionKind::Revert => "回退提交",
    };
    let summary = match (
        in_progress.commit_id.as_deref(),
        in_progress.subject.as_deref(),
    ) {
        (Some(commit_id), Some(subject)) => format!(
            "{} 正停在提交 {} · {}",
            label,
            short_commit_id(commit_id),
            subject
        ),
        (Some(commit_id), None) => format!("{} 正停在提交 {}", label, short_commit_id(commit_id)),
        (None, _) => format!("{label} 正等待你继续处理当前流程"),
    };
    let conflict_count = in_progress.conflicted_files.len();
    let detail = if conflict_count > 0 {
        format!("还有 {conflict_count} 个冲突文件，先处理冲突再继续。")
    } else {
        "当前看起来已经没有冲突文件了，可以继续完成这个流程。".to_string()
    };

    Some(
        Container::new(
            Column::new()
                .spacing(theme::spacing::SM)
                .push(
                    Column::new()
                        .spacing(2)
                        .push(
                            Text::new("进行中")
                                .size(9)
                                .color(theme::darcula::TEXT_SECONDARY),
                        )
                        .push(Text::new(format!("{label} 暂停")).size(15))
                        .push(
                            Text::new("在界面中直接继续或中止当前 rebase/cherry-pick 操作。")
                                .size(11)
                                .color(theme::darcula::TEXT_SECONDARY),
                        ),
                )
                .push(
                    Row::new()
                        .spacing(theme::spacing::XS)
                        .align_y(Alignment::Center)
                        .push(widgets::info_chip::<BranchPopupMessage>(
                            label,
                            BadgeTone::Warning,
                        ))
                        .push(widgets::info_chip::<BranchPopupMessage>(
                            format!("冲突文件 {}", conflict_count),
                            if conflict_count > 0 {
                                BadgeTone::Danger
                            } else {
                                BadgeTone::Success
                            },
                        )),
                )
                .push(
                    Text::new(summary)
                        .size(12)
                        .width(Length::Fill)
                        .wrapping(text::Wrapping::WordOrGlyph),
                )
                .push(
                    Text::new(detail)
                        .size(11)
                        .width(Length::Fill)
                        .wrapping(text::Wrapping::WordOrGlyph)
                        .color(theme::darcula::TEXT_SECONDARY),
                )
                .push(
                    Row::new()
                        .spacing(theme::spacing::XS)
                        .push(button::secondary(
                            "继续",
                            (!state.is_loading && conflict_count == 0)
                                .then_some(BranchPopupMessage::ContinueInProgressCommitAction),
                        ))
                        .push(button::ghost(
                            "处理冲突",
                            (!state.is_loading && conflict_count > 0)
                                .then_some(BranchPopupMessage::OpenConflictList),
                        ))
                        .push(button::ghost(
                            "中止",
                            (!state.is_loading)
                                .then_some(BranchPopupMessage::AbortInProgressCommitAction),
                        )),
                ),
        )
        .padding([8, 10])
        .style(theme::panel_style(Surface::Selection))
        .into(),
    )
}

fn build_commit_context_menu_overlay<'a>(
    state: &'a BranchPopupState,
    selected_branch: &'a Branch,
) -> Element<'a, BranchPopupMessage> {
    let Some(commit_id) = state.context_menu_commit.as_deref() else {
        return Space::new().width(Length::Shrink).into();
    };
    let Some(info) = state
        .selected_branch_commit_info
        .as_ref()
        .filter(|info| info.id == commit_id)
    else {
        return Space::new().width(Length::Shrink).into();
    };

    let header = Column::new()
        .spacing(theme::spacing::SM)
        .push(
            Row::new()
                .spacing(theme::spacing::XS)
                .align_y(Alignment::Center)
                .push(
                    Column::new()
                        .spacing(2)
                        .width(Length::Fill)
                        .push(
                            Text::new("提交动作")
                                .size(10)
                                .color(theme::darcula::TEXT_SECONDARY),
                        )
                        .push(
                            Text::new(commit_subject(&info.message))
                                .size(14)
                                .width(Length::Fill)
                                .wrapping(text::Wrapping::WordOrGlyph),
                        ),
                )
                .push(button::compact_ghost(
                    "关闭",
                    Some(BranchPopupMessage::CloseCommitContextMenu),
                )),
        )
        .push(
            Row::new()
                .spacing(theme::spacing::XS)
                .align_y(Alignment::Center)
                .push(widgets::info_chip::<BranchPopupMessage>(
                    selected_branch.name.clone(),
                    if selected_branch.is_head {
                        BadgeTone::Success
                    } else {
                        BadgeTone::Accent
                    },
                ))
                .push(widgets::info_chip::<BranchPopupMessage>(
                    short_commit_id(&info.id),
                    BadgeTone::Neutral,
                ))
                .push_maybe((info.parent_ids.len() > 1).then(|| {
                    widgets::info_chip::<BranchPopupMessage>(
                        format!("merge {}", info.parent_ids.len()),
                        BadgeTone::Warning,
                    )
                }))
                .push_maybe(info.parent_ids.is_empty().then(|| {
                    widgets::info_chip::<BranchPopupMessage>("根提交", BadgeTone::Neutral)
                })),
        )
        .push(
            Text::new(format!(
                "{} <{}> · {}",
                info.author_name,
                info.author_email,
                format_timestamp(info.author_time)
            ))
            .size(10)
            .width(Length::Fill)
            .wrapping(text::Wrapping::WordOrGlyph)
            .color(theme::darcula::TEXT_SECONDARY),
        );

    let action_groups = build_commit_action_groups(state, selected_branch, info)
        .into_iter()
        .fold(
            Column::new().spacing(theme::spacing::SM),
            |column, group| column.push(group),
        );

    let menu = Container::new(Column::new().spacing(theme::spacing::SM).push(header).push(
        Container::new(scrollable::styled(action_groups).height(Length::Fixed(360.0))),
    ))
    .padding([9, 10])
    .width(Length::Fixed(374.0))
    .style(widgets::menu::panel_style);

    opaque(
        mouse_area(
            Container::new(
                Row::new()
                    .width(Length::Fill)
                    .push(Space::new().width(Length::Fill))
                    .push(menu),
            )
            .padding([12, 16])
            .width(Length::Fill)
            .height(Length::Fill)
            .style(widgets::menu::scrim_style),
        )
        .on_press(BranchPopupMessage::CloseCommitContextMenu),
    )
}

fn build_commit_action_groups<'a>(
    state: &'a BranchPopupState,
    selected_branch: &'a Branch,
    info: &'a CommitInfo,
) -> Vec<Element<'a, BranchPopupMessage>> {
    let current_branch = state.current_branch();
    let can_compare_with_current = current_branch
        .map(|branch| branch.name.clone())
        .filter(|name| name != &selected_branch.name);
    let parent_commit_id = selected_commit_parent_in_history(state);
    let child_commit_ids = selected_commit_children_in_history(state);
    let child_commit_id = (child_commit_ids.len() == 1).then(|| child_commit_ids[0].clone());
    let can_prepare_cherry_pick =
        !state.is_loading && current_branch.is_some() && info.parent_ids.len() <= 1;
    let can_prepare_revert =
        !state.is_loading && current_branch.is_some() && info.parent_ids.len() <= 1;
    let can_reset_current_branch =
        !state.is_loading && selected_branch.is_head && info.id != selected_branch.oid;
    let can_push_current_branch_to_here =
        can_reset_current_branch && current_branch.is_some_and(|branch| branch.upstream.is_some());

    let compare_with_current_row = commit_menu_action_row(
        Some("<>"),
        "与当前分支比较",
        Some(
            can_compare_with_current
                .as_ref()
                .map(|branch| format!("直接比较当前提交与 {branch} 的差异"))
                .unwrap_or_else(|| "当前已经在这条分支上下文里，无法再和自身比较".to_string()),
        ),
        can_compare_with_current.map(|current| BranchPopupMessage::CompareWithCurrent {
            selected: info.id.clone(),
            current,
        }),
        CommitMenuTone::Accent,
    );
    let parent_row = commit_menu_action_row(
        Some("^"),
        "跳到父提交",
        Some(if parent_commit_id.is_some() {
            "把焦点移到当前提交的父提交".to_string()
        } else if info.parent_ids.is_empty() {
            "当前已经是根提交，没有父提交".to_string()
        } else {
            "父提交不在已加载范围里，请先滚动时间线继续查看".to_string()
        }),
        parent_commit_id.map(BranchPopupMessage::SelectBranchCommit),
        CommitMenuTone::Accent,
    );
    let child_row = commit_menu_action_row(
        Some("v"),
        "跳到子提交",
        Some(if child_commit_id.is_some() {
            "把焦点移到当前提交的直接子提交".to_string()
        } else if child_commit_ids.len() > 1 {
            "检测到多个子提交，请直接在时间线中选择目标提交".to_string()
        } else {
            "当前已是已加载时间线的末端，没有子提交".to_string()
        }),
        child_commit_id.map(BranchPopupMessage::SelectBranchCommit),
        CommitMenuTone::Accent,
    );

    vec![
        build_commit_action_group(
            "常用".to_uppercase(),
            "复制版本号、导出补丁。",
            CommitMenuTone::Neutral,
            vec![
                commit_menu_action_row(
                    Some("#"),
                    "复制哈希",
                    Some("把完整提交哈希复制到系统剪贴板".to_string()),
                    (!state.is_loading)
                        .then_some(BranchPopupMessage::CopyCommitHash(info.id.clone())),
                    CommitMenuTone::Neutral,
                ),
                commit_menu_action_row(
                    Some("PT"),
                    "导出 Patch",
                    Some("用 git format-patch 导出这条提交的补丁文件".to_string()),
                    (!state.is_loading)
                        .then_some(BranchPopupMessage::ExportCommitPatch(info.id.clone())),
                    CommitMenuTone::Neutral,
                ),
            ],
        ),
        build_commit_action_group(
            "比较与定位".to_uppercase(),
            "比较当前上下文，沿着提交前后移动。",
            CommitMenuTone::Accent,
            vec![
                commit_menu_action_row(
                    None,
                    "查看与工作树差异",
                    Some("把这条提交和当前工作区直接做比较".to_string()),
                    (!state.is_loading)
                        .then_some(BranchPopupMessage::CompareWithWorktree(info.id.clone())),
                    CommitMenuTone::Accent,
                ),
                compare_with_current_row,
                parent_row,
                child_row,
            ],
        ),
        build_commit_action_group(
            "派生".to_uppercase(),
            "保留现有历史，基于它继续工作。",
            CommitMenuTone::Neutral,
            vec![
                commit_menu_action_row(
                    None,
                    "从该提交建分支",
                    Some("保留当前分支不动，基于这条提交创建新分支".to_string()),
                    (!state.is_loading).then_some(BranchPopupMessage::PrepareCreateFromSelected(
                        info.id.clone(),
                    )),
                    CommitMenuTone::Neutral,
                ),
                commit_menu_action_row(
                    Some("TG"),
                    "给该提交打标签",
                    Some("在这条提交上创建一个新的标签".to_string()),
                    (!state.is_loading)
                        .then_some(BranchPopupMessage::PrepareTagFromCommit(info.id.clone())),
                    CommitMenuTone::Neutral,
                ),
            ],
        ),
        build_commit_action_group(
            "应用到当前分支".to_uppercase(),
            "会在当前分支生成新的提交。",
            CommitMenuTone::Accent,
            vec![
                commit_menu_action_row(
                    Some("CP"),
                    "Cherry-pick",
                    Some(if info.parent_ids.len() > 1 {
                        "merge 提交暂不支持直接 Cherry-pick".to_string()
                    } else {
                        "把这条提交复制应用到当前分支".to_string()
                    }),
                    can_prepare_cherry_pick
                        .then_some(BranchPopupMessage::PrepareCherryPickCommit(info.id.clone())),
                    CommitMenuTone::Accent,
                ),
                commit_menu_action_row(
                    Some("RV"),
                    "Revert",
                    Some(if info.parent_ids.len() > 1 {
                        "merge 提交暂不支持直接回退".to_string()
                    } else {
                        "生成一条新的反向提交来撤销它".to_string()
                    }),
                    can_prepare_revert
                        .then_some(BranchPopupMessage::PrepareRevertCommit(info.id.clone())),
                    CommitMenuTone::Accent,
                ),
            ],
        ),
        build_commit_action_group(
            "危险动作".to_uppercase(),
            "会移动分支指针或直接发布到当前上游。",
            CommitMenuTone::Danger,
            vec![
                commit_menu_action_row(
                    None,
                    "重置当前分支到这里",
                    Some(if selected_branch.is_head {
                        "把当前分支硬重置到这个祖先提交".to_string()
                    } else {
                        "只对当前分支启用，选中当前分支后再操作".to_string()
                    }),
                    can_reset_current_branch.then_some(
                        BranchPopupMessage::PrepareResetCurrentBranchToCommit(info.id.clone()),
                    ),
                    CommitMenuTone::Danger,
                ),
                commit_menu_action_row(
                    None,
                    "推送当前分支到这里",
                    Some(if !selected_branch.is_head {
                        "只对当前分支启用".to_string()
                    } else if current_branch.is_some_and(|branch| branch.upstream.is_none()) {
                        "当前分支还没有上游，暂时不能推送到这里".to_string()
                    } else {
                        "仅访问当前分支的上游，把远端分支发布到这里".to_string()
                    }),
                    can_push_current_branch_to_here.then_some(
                        BranchPopupMessage::PreparePushCurrentBranchToCommit(info.id.clone()),
                    ),
                    CommitMenuTone::Danger,
                ),
            ],
        ),
    ]
}

fn build_branch_commit_row<'a>(
    state: &'a BranchPopupState,
    entry: &'a HistoryEntry,
    is_selected: bool,
) -> Element<'a, BranchPopupMessage> {
    let is_menu_open = state.is_commit_context_menu_open_for(&entry.id);
    let row = Container::new(
        Column::new()
            .spacing(3)
            .push(
                Row::new()
                    .spacing(theme::spacing::XS)
                    .align_y(Alignment::Center)
                    .push(
                        Text::new(commit_subject(&entry.message))
                            .size(12)
                            .width(Length::Fill)
                            .wrapping(text::Wrapping::WordOrGlyph),
                    )
                    .push(
                        Text::new(short_commit_id(&entry.id))
                            .size(10)
                            .color(theme::darcula::TEXT_DISABLED),
                    ),
            )
            .push(
                Text::new(format!(
                    "{} · {}",
                    entry.author_name,
                    format_timestamp(entry.timestamp)
                ))
                .size(10)
                .width(Length::Fill)
                .wrapping(text::Wrapping::WordOrGlyph)
                .color(theme::darcula::TEXT_SECONDARY),
            ),
    )
    .padding([6, 8])
    .style(theme::panel_style(if is_menu_open {
        Surface::Accent
    } else if is_selected {
        Surface::Selection
    } else {
        Surface::Raised
    }));

    let row = Row::new()
        .spacing(theme::spacing::XS)
        .align_y(Alignment::Center)
        .push(
            Container::new(
                Button::new(row)
                    .width(Length::Fill)
                    .style(widgets::menu::trigger_row_button_style(
                        is_selected,
                        is_menu_open,
                        Some(theme::darcula::ACCENT),
                    ))
                    .on_press(BranchPopupMessage::SelectBranchCommit(entry.id.clone())),
            )
            .width(Length::Fill),
        )
        .push(button::compact_ghost(
            "⋯",
            Some(BranchPopupMessage::OpenCommitContextMenu(entry.id.clone())),
        ));

    mouse_area(Container::new(row).width(Length::Fill))
        .on_right_press(BranchPopupMessage::OpenCommitContextMenu(entry.id.clone()))
        .interaction(mouse::Interaction::Pointer)
        .into()
}

fn build_branch_action_groups<'a>(
    state: &'a BranchPopupState,
    selected_branch: &'a Branch,
    current_branch: Option<&'a Branch>,
) -> Vec<Element<'a, BranchPopupMessage>> {
    let selected_remote_name = inferred_remote_name(state, selected_branch);
    let upstream_ref = inferred_upstream_ref(state, selected_branch);
    let can_checkout = !state.is_loading && !selected_branch.is_head && !selected_branch.is_remote;
    let can_checkout_remote =
        !state.is_loading && !selected_branch.is_head && selected_branch.is_remote;
    let can_rename = !state.is_loading && !selected_branch.is_remote;
    let can_delete = !state.is_loading && !selected_branch.is_remote && !selected_branch.is_head;
    let can_push =
        !state.is_loading && !selected_branch.is_remote && selected_remote_name.is_some();
    let can_fetch = !state.is_loading && selected_remote_name.is_some();
    let can_track = !state.is_loading
        && !selected_branch.is_remote
        && selected_branch.upstream.is_none()
        && upstream_ref.is_some();

    let current_branch_name = current_branch.map(|branch| branch.name.clone());
    let compare_target = current_branch_name
        .clone()
        .filter(|name| name != &selected_branch.name);
    let checkout_and_rebase_target = current_branch_name
        .clone()
        .filter(|_| !selected_branch.is_head && !selected_branch.is_remote);

    vec![
        build_commit_action_group(
            "常用".to_uppercase(),
            "切换到这条分支，或基于它继续开工。",
            CommitMenuTone::Neutral,
            vec![
                commit_menu_action_row(
                    None,
                    if selected_branch.is_remote {
                        "签出为本地分支"
                    } else {
                        "签出"
                    },
                    Some(if selected_branch.is_remote {
                        if can_checkout_remote {
                            "创建同名本地跟踪分支并立即切换过去".to_string()
                        } else {
                            "远程分支当前不可直接签出".to_string()
                        }
                    } else if can_checkout {
                        "切换到这个本地分支".to_string()
                    } else {
                        "当前已经在这个分支上".to_string()
                    }),
                    if selected_branch.is_remote {
                        can_checkout_remote.then(|| {
                            BranchPopupMessage::CheckoutRemoteBranch(selected_branch.name.clone())
                        })
                    } else {
                        can_checkout.then(|| {
                            BranchPopupMessage::CheckoutBranch(selected_branch.name.clone())
                        })
                    },
                    CommitMenuTone::Neutral,
                ),
                commit_menu_action_row(
                    None,
                    format!("从 '{}' 新建分支...", selected_branch.name),
                    Some("保留当前分支不动，基于所选分支创建一个新分支".to_string()),
                    (!state.is_loading).then(|| {
                        BranchPopupMessage::PrepareCreateFromSelected(selected_branch.name.clone())
                    }),
                    CommitMenuTone::Neutral,
                ),
                commit_menu_action_row(
                    None,
                    checkout_and_rebase_target
                        .as_ref()
                        .map(|target| format!("签出并变基到 '{target}'"))
                        .unwrap_or_else(|| "签出并变基".to_string()),
                    Some(if let Some(target) = checkout_and_rebase_target.as_ref() {
                        format!("先切到 {}，再把它变基到 {target}", selected_branch.name)
                    } else if selected_branch.is_head {
                        "当前分支不能对自己执行“签出并变基”".to_string()
                    } else if selected_branch.is_remote {
                        "远程分支请先签出为本地分支，再执行这类操作".to_string()
                    } else {
                        "当前没有可作为目标的本地分支".to_string()
                    }),
                    checkout_and_rebase_target.map(|onto| BranchPopupMessage::CheckoutAndRebase {
                        branch: selected_branch.name.clone(),
                        onto,
                    }),
                    CommitMenuTone::Accent,
                ),
            ],
        ),
        build_commit_action_group(
            "比较".to_uppercase(),
            "直接看它和当前上下文的差异。",
            CommitMenuTone::Accent,
            vec![
                commit_menu_action_row(
                    None,
                    compare_target
                        .as_ref()
                        .map(|target| format!("与 '{target}' 比较"))
                        .unwrap_or_else(|| "与当前分支比较".to_string()),
                    Some(if let Some(target) = compare_target.as_ref() {
                        format!("在右侧直接预览 {} 和 {target} 的差异", selected_branch.name)
                    } else {
                        "当前已经在这条分支上下文里，无法再和自身比较".to_string()
                    }),
                    compare_target.map(|current| BranchPopupMessage::CompareWithCurrent {
                        selected: selected_branch.name.clone(),
                        current,
                    }),
                    CommitMenuTone::Accent,
                ),
                commit_menu_action_row(
                    None,
                    "显示与工作树的差异",
                    Some("预览所选分支与当前工作区（含已暂存改动）的差别".to_string()),
                    (!state.is_loading).then(|| {
                        BranchPopupMessage::CompareWithWorktree(selected_branch.name.clone())
                    }),
                    CommitMenuTone::Accent,
                ),
            ],
        ),
        build_commit_action_group(
            "集成".to_uppercase(),
            "把所选分支并入当前工作线，或让当前工作线基于它重排。",
            CommitMenuTone::Accent,
            vec![
                commit_menu_action_row(
                    None,
                    format!("将当前分支变基到 '{}'", selected_branch.name),
                    Some(if !selected_branch.is_head {
                        "把当前分支移动到所选分支之后，适合保持提交线性".to_string()
                    } else {
                        "当前分支不需要再变基到自己".to_string()
                    }),
                    (!state.is_loading && !selected_branch.is_head).then(|| {
                        BranchPopupMessage::RebaseCurrentOnto(selected_branch.name.clone())
                    }),
                    CommitMenuTone::Accent,
                ),
                commit_menu_action_row(
                    None,
                    current_branch_name
                        .as_ref()
                        .map(|current| {
                            format!("将 '{}' 合并到 '{}' 中", selected_branch.name, current)
                        })
                        .unwrap_or_else(|| "合并所选分支".to_string()),
                    Some(if selected_branch.is_remote {
                        "远程分支请先签出或更新到本地后再合并".to_string()
                    } else if !selected_branch.is_head {
                        "把所选分支的提交合并到当前分支".to_string()
                    } else {
                        "当前分支不能合并自己".to_string()
                    }),
                    (!state.is_loading && !selected_branch.is_head && !selected_branch.is_remote)
                        .then(|| BranchPopupMessage::MergeBranch(selected_branch.name.clone())),
                    CommitMenuTone::Accent,
                ),
            ],
        ),
        build_commit_action_group(
            "远程".to_uppercase(),
            "获取、推送或建立跟踪关系。",
            CommitMenuTone::Accent,
            vec![
                commit_menu_action_row(
                    None,
                    "更新",
                    Some(
                        selected_remote_name
                            .as_ref()
                            .map(|remote| format!("从 {remote} 获取最新远程状态"))
                            .unwrap_or_else(|| "当前分支没有可推断的远程".to_string()),
                    ),
                    if can_fetch {
                        selected_remote_name
                            .clone()
                            .map(BranchPopupMessage::FetchRemote)
                    } else {
                        None
                    },
                    CommitMenuTone::Accent,
                ),
                commit_menu_action_row(
                    None,
                    "推送...",
                    Some(if selected_branch.is_remote {
                        "远程分支不能直接作为推送源".to_string()
                    } else {
                        selected_remote_name
                            .as_ref()
                            .map(|remote| format!("直接把所选本地分支推送到 {remote}"))
                            .unwrap_or_else(|| "当前分支没有可推断的远程".to_string())
                    }),
                    if can_push {
                        selected_remote_name
                            .clone()
                            .map(|remote| BranchPopupMessage::PushBranch {
                                branch: selected_branch.name.clone(),
                                remote,
                            })
                    } else {
                        None
                    },
                    CommitMenuTone::Accent,
                ),
                commit_menu_action_row(
                    None,
                    upstream_ref
                        .as_ref()
                        .map(|upstream| format!("跟踪分支 '{upstream}'"))
                        .unwrap_or_else(|| "设置跟踪分支".to_string()),
                    Some(if selected_branch.is_remote {
                        "远程分支本身不需要设置跟踪关系".to_string()
                    } else if selected_branch.upstream.is_some() {
                        "这个分支已经有跟踪关系".to_string()
                    } else if upstream_ref.is_some() {
                        "让这个本地分支与同名远程分支建立跟踪关系".to_string()
                    } else {
                        "没有匹配到可跟踪的远程分支".to_string()
                    }),
                    if can_track {
                        upstream_ref
                            .clone()
                            .map(|upstream| BranchPopupMessage::SetUpstream {
                                branch: selected_branch.name.clone(),
                                upstream,
                            })
                    } else {
                        None
                    },
                    CommitMenuTone::Accent,
                ),
            ],
        ),
        build_commit_action_group(
            "维护",
            "整理命名，但不直接改写提交历史。",
            CommitMenuTone::Neutral,
            vec![commit_menu_action_row(
                None,
                "重命名...",
                Some(if can_rename {
                    "修改本地分支名称，支持直接编辑完整路径".to_string()
                } else {
                    "远程分支暂不支持直接重命名".to_string()
                }),
                can_rename
                    .then(|| BranchPopupMessage::PrepareRenameBranch(selected_branch.name.clone())),
                CommitMenuTone::Neutral,
            )],
        ),
        build_commit_action_group(
            "危险动作",
            "删除分支前，请确认它不是当前工作分支。",
            CommitMenuTone::Danger,
            vec![commit_menu_action_row(
                None,
                "删除",
                Some(if can_delete {
                    "删除这个本地分支；当前分支不可删除".to_string()
                } else if selected_branch.is_remote {
                    "远程分支请通过远程操作删除".to_string()
                } else {
                    "当前分支不可删除".to_string()
                }),
                can_delete.then(|| BranchPopupMessage::DeleteBranch(selected_branch.name.clone())),
                CommitMenuTone::Danger,
            )],
        ),
    ]
}

fn tree_indent(depth: usize) -> Space {
    const TREE_INDENT: f32 = 14.0;

    Space::new().width(Length::Fixed((depth as f32) * TREE_INDENT))
}

fn folder_key(section: BranchSection, path: &str) -> String {
    format!("{}:{path}", section.storage_key())
}

fn folder_depth(path_key: &str) -> usize {
    path_key
        .split_once(':')
        .map(|(_, path)| path.split('/').count())
        .unwrap_or(0)
}

fn build_branch_context_button<'a>(
    branch_name: String,
    active: bool,
) -> Element<'a, BranchPopupMessage> {
    Button::new(
        Container::new(Text::new("⋯").size(12).color(if active {
            theme::darcula::TEXT_PRIMARY
        } else {
            theme::darcula::TEXT_SECONDARY
        }))
        .center_x(Length::Fixed(24.0))
        .center_y(Length::Fixed(22.0)),
    )
    .style(branch_context_button_style(active))
    .on_press(BranchPopupMessage::OpenBranchContextMenu(branch_name))
    .into()
}

fn branch_row_strip(color: Option<Color>) -> Container<'static, BranchPopupMessage> {
    Container::new(Space::new().width(Length::Fixed(1.0)).height(Length::Fill))
        .width(Length::Fixed(3.0))
        .height(Length::Fill)
        .style(branch_row_strip_style(color))
}

fn branch_row_strip_style(color: Option<Color>) -> impl Fn(&Theme) -> container::Style {
    move |_theme| container::Style {
        background: Some(Background::Color(color.unwrap_or(Color::TRANSPARENT))),
        border: Border {
            width: 0.0,
            color: Color::TRANSPARENT,
            radius: theme::radius::LG.into(),
        },
        ..Default::default()
    }
}

fn branch_row_button_style(
    is_selected: bool,
    is_menu_open: bool,
    is_current: bool,
) -> impl Fn(&Theme, iced::widget::button::Status) -> iced::widget::button::Style {
    widgets::menu::trigger_row_button_style(
        is_selected || is_current,
        is_menu_open,
        Some(if is_current && !is_selected && !is_menu_open {
            theme::darcula::SUCCESS
        } else {
            theme::darcula::ACCENT
        }),
    )
}

fn branch_folder_row_button_style(
    expanded: bool,
) -> impl Fn(&Theme, iced::widget::button::Status) -> iced::widget::button::Style {
    move |_theme, status| {
        let base_background = if expanded {
            blend_color(theme::darcula::BG_PANEL, theme::darcula::BG_RAISED, 0.46)
        } else {
            Color::TRANSPARENT
        };
        let base_border = if expanded {
            theme::darcula::SEPARATOR.scale_alpha(0.60)
        } else {
            Color::TRANSPARENT
        };

        let (background, border_color) = match status {
            iced::widget::button::Status::Active => (base_background, base_border),
            iced::widget::button::Status::Hovered => (
                if expanded {
                    blend_color(base_background, Color::WHITE, 0.04)
                } else {
                    blend_color(theme::darcula::BG_PANEL, theme::darcula::BG_RAISED, 0.58)
                },
                if expanded {
                    blend_color(base_border, Color::WHITE, 0.06)
                } else {
                    theme::darcula::SEPARATOR.scale_alpha(0.64)
                },
            ),
            iced::widget::button::Status::Pressed => (
                if expanded {
                    blend_color(base_background, theme::darcula::BG_MAIN, 0.10)
                } else {
                    blend_color(theme::darcula::BG_PANEL, theme::darcula::BG_RAISED, 0.78)
                },
                if expanded {
                    blend_color(base_border, theme::darcula::BG_MAIN, 0.10)
                } else {
                    theme::darcula::ACCENT.scale_alpha(0.24)
                },
            ),
            iced::widget::button::Status::Disabled => (
                blend_color(theme::darcula::BG_PANEL, base_background, 0.24),
                blend_color(theme::darcula::BORDER, base_border, 0.22),
            ),
        };

        iced::widget::button::Style {
            background: Some(Background::Color(background)),
            border: Border {
                width: 1.0,
                color: border_color,
                radius: theme::radius::LG.into(),
            },
            text_color: if matches!(status, iced::widget::button::Status::Disabled) {
                theme::darcula::TEXT_DISABLED
            } else {
                theme::darcula::TEXT_PRIMARY
            },
            ..Default::default()
        }
    }
}

fn branch_context_button_style(
    active: bool,
) -> impl Fn(&Theme, iced::widget::button::Status) -> iced::widget::button::Style {
    move |_theme, status| {
        let (base_background, base_border) = if active {
            (
                blend_color(theme::darcula::BG_PANEL, theme::darcula::ACCENT_WEAK, 0.84),
                theme::darcula::ACCENT.scale_alpha(0.72),
            )
        } else {
            (Color::TRANSPARENT, Color::TRANSPARENT)
        };

        let (background, border_color) = match status {
            iced::widget::button::Status::Active => (base_background, base_border),
            iced::widget::button::Status::Hovered => (
                if active {
                    blend_color(base_background, Color::WHITE, 0.05)
                } else {
                    blend_color(theme::darcula::BG_PANEL, theme::darcula::BG_RAISED, 0.70)
                },
                if active {
                    blend_color(base_border, Color::WHITE, 0.08)
                } else {
                    theme::darcula::SEPARATOR.scale_alpha(0.70)
                },
            ),
            iced::widget::button::Status::Pressed => (
                if active {
                    blend_color(base_background, theme::darcula::BG_MAIN, 0.12)
                } else {
                    blend_color(theme::darcula::BG_PANEL, theme::darcula::BG_RAISED, 0.90)
                },
                if active {
                    blend_color(base_border, theme::darcula::BG_MAIN, 0.10)
                } else {
                    theme::darcula::ACCENT.scale_alpha(0.30)
                },
            ),
            iced::widget::button::Status::Disabled => (
                blend_color(theme::darcula::BG_PANEL, base_background, 0.28),
                blend_color(theme::darcula::BORDER, base_border, 0.24),
            ),
        };

        iced::widget::button::Style {
            background: Some(Background::Color(background)),
            border: Border {
                width: 1.0,
                color: border_color,
                radius: theme::radius::LG.into(),
            },
            text_color: if matches!(status, iced::widget::button::Status::Disabled) {
                theme::darcula::TEXT_DISABLED
            } else if active
                || matches!(
                    status,
                    iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed
                )
            {
                theme::darcula::TEXT_PRIMARY
            } else {
                theme::darcula::TEXT_SECONDARY
            },
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CommitMenuTone {
    Neutral,
    Accent,
    Danger,
}

fn build_commit_action_group<'a>(
    title: impl Into<String>,
    detail: &'a str,
    tone: CommitMenuTone,
    rows: Vec<Element<'a, BranchPopupMessage>>,
) -> Element<'a, BranchPopupMessage> {
    widgets::menu::group(title, detail, map_commit_menu_tone(tone), rows)
}

fn commit_menu_action_row<'a>(
    icon: Option<&'static str>,
    title: impl Into<String>,
    detail: Option<String>,
    on_press: Option<BranchPopupMessage>,
    tone: CommitMenuTone,
) -> Element<'a, BranchPopupMessage> {
    widgets::menu::action_row(
        icon,
        title,
        detail,
        None,
        on_press,
        map_commit_menu_tone(tone),
    )
}

fn build_inline_action_panel<'a>(
    state: &'a BranchPopupState,
) -> Option<Element<'a, BranchPopupMessage>> {
    let action = state.inline_action.as_ref()?;
    let title = match action {
        InlineBranchAction::CreateFromSelected { base } => format!("从 '{base}' 新建分支"),
        InlineBranchAction::RenameBranch { branch } => format!("重命名 '{branch}'"),
    };

    Some(
        Container::new(
            Column::new()
                .spacing(theme::spacing::SM)
                .push(Text::new(title).size(12))
                .push(
                    Text::new("输入名称后立即执行，不需要再跳到别的面板。")
                        .size(10)
                        .color(theme::darcula::TEXT_SECONDARY),
                )
                .push(text_input::styled(
                    "输入分支名称",
                    &state.inline_branch_name,
                    BranchPopupMessage::SetInlineBranchName,
                ))
                .push(
                    Row::new()
                        .spacing(theme::spacing::XS)
                        .push(button::secondary(
                            "确定",
                            (!state.is_loading && !state.inline_branch_name.trim().is_empty())
                                .then_some(BranchPopupMessage::ConfirmInlineAction),
                        ))
                        .push(button::ghost(
                            "取消",
                            (!state.is_loading).then_some(BranchPopupMessage::CancelInlineAction),
                        )),
                ),
        )
        .padding([10, 12])
        .style(theme::panel_style(Surface::Selection))
        .into(),
    )
}

fn build_comparison_panel<'a>(
    state: &'a BranchPopupState,
) -> Option<Element<'a, BranchPopupMessage>> {
    let diff = state.comparison_diff.as_ref()?;
    let title = state.comparison_title.as_deref().unwrap_or("比较结果");

    Some(
        Container::new(
            Column::new()
                .spacing(theme::spacing::SM)
                .push(
                    Row::new()
                        .spacing(theme::spacing::XS)
                        .align_y(Alignment::Center)
                        .push(Text::new(title).size(12))
                        .push(Space::new().width(Length::Fill))
                        .push(button::compact_ghost(
                            "清空",
                            Some(BranchPopupMessage::ClearPreview),
                        )),
                )
                .push_maybe(state.comparison_summary.as_ref().map(|summary| {
                    Text::new(summary)
                        .size(10)
                        .color(theme::darcula::TEXT_SECONDARY)
                        .width(Length::Fill)
                        .wrapping(text::Wrapping::WordOrGlyph)
                }))
                .push(
                    Container::new(diff_viewer::DiffViewer::new(diff).view())
                        .height(Length::Fixed(280.0)),
                ),
        )
        .padding([10, 12])
        .style(theme::panel_style(Surface::Panel))
        .into(),
    )
}

/// Represents incoming/outgoing sync state for a branch
#[derive(Debug, Clone, Default)]
pub struct BranchSyncState {
    pub incoming: Option<u32>, // Number of commits behind (incoming)
    pub outgoing: Option<u32>, // Number of commits ahead (outgoing)
}

impl BranchSyncState {
    /// Parse sync state from tracking_status string like "3↓" or "↑2" or "↕3/5"
    pub fn from_tracking_status(status: &Option<String>) -> Self {
        let Some(status) = status else {
            return Self::default();
        };

        // Handle diverged case: ↕3/5
        if let Some(after_arrow) = status.strip_prefix('↕') {
            if let Some((ahead, behind)) = after_arrow.split_once('/') {
                let incoming = behind.trim().parse().ok();
                let outgoing = ahead.trim().parse().ok();
                return BranchSyncState { incoming, outgoing };
            }
        }

        // Handle single arrow cases: ↓3 or ↑2
        if let Some(after_arrow) = status.strip_prefix('↓') {
            let count: Option<u32> = after_arrow.trim().parse().ok();
            return BranchSyncState {
                incoming: count,
                outgoing: None,
            };
        }
        if let Some(after_arrow) = status.strip_prefix('↑') {
            let count: Option<u32> = after_arrow.trim().parse().ok();
            return BranchSyncState {
                incoming: None,
                outgoing: count,
            };
        }

        // Handle special cases: ✓ (synced), ? (unknown), or plain text
        Self::default()
    }
}

/// IDEA-style: shrink large commit counts to "99+"
/// Matches GitIncomingOutgoingUi.shrinkTo99 in IDEA
fn shrink_to_99(commits: u32) -> String {
    if commits > 99 {
        "99+".to_string()
    } else {
        commits.to_string()
    }
}

/// IDEA-style: Build sync indicator arrows for branch display
/// Shows colored arrows: blue ↓ for incoming, green ↑ for outgoing
fn build_sync_indicators<'a>(branch: &Branch) -> Option<Element<'a, BranchPopupMessage>> {
    if branch.is_remote {
        return None;
    }

    let sync_state = BranchSyncState::from_tracking_status(&branch.tracking_status);

    if sync_state.incoming.is_none() && sync_state.outgoing.is_none() {
        return None;
    }

    let incoming_indicator: Option<Element<'a, BranchPopupMessage>> =
        sync_state.incoming.map(|count| {
            Container::new(
                Text::new(format!("↓{}", shrink_to_99(count)))
                    .size(10)
                    .color(theme::darcula::INCOMING),
            )
            .into()
        });

    let outgoing_indicator: Option<Element<'a, BranchPopupMessage>> =
        sync_state.outgoing.map(|count| {
            Container::new(
                Text::new(format!("↑{}", shrink_to_99(count)))
                    .size(10)
                    .color(theme::darcula::OUTGOING),
            )
            .into()
        });

    let mut row = Row::new().spacing(2).align_y(Alignment::Center);
    if let Some(elem) = incoming_indicator {
        row = row.push(elem);
    }
    if let Some(elem) = outgoing_indicator {
        row = row.push(elem);
    }

    Some(row.into())
}

fn branch_meta_summary(branch: &Branch) -> Option<String> {
    let mut parts = Vec::new();

    if let Some(sync_hint) = branch.sync_hint.as_ref() {
        parts.push(sync_hint.clone());
    } else if let Some(upstream) = branch.upstream.as_ref() {
        parts.push(format!("跟踪 {upstream}"));
    }

    if let Some(recency_hint) = branch.recency_hint.as_ref() {
        parts.push(recency_hint.clone());
    }

    if parts.is_empty() {
        None
    } else {
        Some(parts.join(" · "))
    }
}

fn inferred_upstream_ref(state: &BranchPopupState, branch: &Branch) -> Option<String> {
    if branch.is_remote {
        return Some(branch.name.clone());
    }

    branch.upstream.clone().or_else(|| {
        matching_remote_branch(state, branch).map(|remote_branch| remote_branch.name.clone())
    })
}

fn inferred_remote_name(state: &BranchPopupState, branch: &Branch) -> Option<String> {
    if branch.is_remote {
        return parse_remote_ref(&branch.name).map(|(remote, _)| remote.to_string());
    }

    branch
        .upstream
        .as_deref()
        .and_then(parse_remote_ref)
        .map(|(remote, _)| remote.to_string())
        .or_else(|| {
            matching_remote_branch(state, branch)
                .and_then(|remote_branch| parse_remote_ref(&remote_branch.name))
                .map(|(remote, _)| remote.to_string())
        })
}

fn matching_remote_branch<'a>(state: &'a BranchPopupState, branch: &Branch) -> Option<&'a Branch> {
    if branch.is_remote {
        return None;
    }

    state.remote_branches.iter().find(|remote_branch| {
        parse_remote_ref(&remote_branch.name)
            .map(|(_, remote_name)| remote_name == branch.name)
            .unwrap_or(false)
    })
}

fn parse_remote_ref(name: &str) -> Option<(&str, &str)> {
    name.split_once('/')
}

fn branch_leaf_name(name: &str) -> &str {
    name.rsplit('/').next().unwrap_or(name)
}

/// IDEA-style branch name truncation at 40 characters
const MAX_BRANCH_NAME_LENGTH: usize = 40;

fn truncate_branch_name(name: &str) -> String {
    if name.chars().count() <= MAX_BRANCH_NAME_LENGTH {
        name.to_string()
    } else {
        // Truncate middle: show start and end
        let half = (MAX_BRANCH_NAME_LENGTH - 3) / 2;
        let prefix: String = name.chars().take(half).collect();
        let suffix: String = name.chars().rev().take(half).collect();
        format!("{}...{}", prefix, suffix.chars().rev().collect::<String>())
    }
}

fn selected_commit_parent_in_history(state: &BranchPopupState) -> Option<String> {
    let entry = selected_history_entry(state)?;
    let parent_id = entry.parent_ids.first()?;
    state
        .branch_history_entries
        .iter()
        .find(|candidate| &candidate.id == parent_id)
        .map(|candidate| candidate.id.clone())
}

fn selected_commit_children_in_history(state: &BranchPopupState) -> Vec<String> {
    let selected_commit = match state.selected_branch_commit.as_deref() {
        Some(commit_id) => commit_id,
        None => return Vec::new(),
    };

    state
        .branch_history_entries
        .iter()
        .filter(|entry| {
            entry
                .parent_ids
                .iter()
                .any(|parent_id| parent_id == selected_commit)
        })
        .map(|entry| entry.id.clone())
        .collect()
}

fn selected_history_entry(state: &BranchPopupState) -> Option<&HistoryEntry> {
    let selected_commit = state.selected_branch_commit.as_deref()?;
    state
        .branch_history_entries
        .iter()
        .find(|entry| entry.id == selected_commit)
}

fn format_timestamp(timestamp: i64) -> String {
    let datetime = DateTime::from_timestamp(timestamp, 0)
        .unwrap_or_else(|| DateTime::from_timestamp(0, 0).unwrap());
    datetime.format("%Y-%m-%d %H:%M:%S").to_string()
}

fn commit_subject(message: &str) -> &str {
    message
        .lines()
        .find(|line| !line.trim().is_empty())
        .unwrap_or(message)
}

fn short_commit_id(id: &str) -> &str {
    &id[..id.len().min(8)]
}

fn map_commit_menu_tone(tone: CommitMenuTone) -> widgets::menu::MenuTone {
    match tone {
        CommitMenuTone::Neutral => widgets::menu::MenuTone::Neutral,
        CommitMenuTone::Accent => widgets::menu::MenuTone::Accent,
        CommitMenuTone::Danger => widgets::menu::MenuTone::Danger,
    }
}

fn blend_color(base: Color, overlay: Color, amount: f32) -> Color {
    let amount = amount.clamp(0.0, 1.0);
    Color {
        r: (base.r * (1.0 - amount)) + (overlay.r * amount),
        g: (base.g * (1.0 - amount)) + (overlay.g * amount),
        b: (base.b * (1.0 - amount)) + (overlay.b * amount),
        a: (base.a * (1.0 - amount)) + (overlay.a * amount),
    }
}

fn format_diff_summary(diff: &Diff) -> String {
    format!(
        "共影响 {} 个文件，+{} / -{} 行。",
        diff.files.len(),
        diff.total_additions,
        diff.total_deletions
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn branch_with_kind(name: &str, is_remote: bool) -> Branch {
        Branch {
            name: name.to_string(),
            oid: String::new(),
            is_remote,
            is_head: false,
            upstream: None,
            tracking_status: None,
            sync_hint: None,
            recency_hint: None,
            last_commit_timestamp: None,
            group_path: None,
        }
    }

    fn branch(name: &str) -> Branch {
        branch_with_kind(name, false)
    }

    #[test]
    fn default_folder_expansion_keeps_two_levels_open() {
        let state = BranchPopupState::new();

        assert!(state.default_folder_expansion("local:feature"));
        assert!(state.default_folder_expansion("local:feature/api"));
        assert!(!state.default_folder_expansion("local:feature/api/login"));
    }

    #[test]
    fn ensure_branch_visible_expands_nested_branch_path() {
        let mut state = BranchPopupState::new();
        state.local_branches = vec![branch("feature/api/login")];

        state.ensure_branch_visible("feature/api/login");

        assert!(state.is_folder_expanded("local:feature"));
        assert!(state.is_folder_expanded("local:feature/api"));
    }

    #[test]
    fn set_search_query_keeps_existing_selection_if_it_still_matches() {
        let mut state = BranchPopupState::new();
        state.local_branches = vec![branch("main"), branch("main-123")];
        state.remote_branches = vec![branch_with_kind("origin/main", true)];
        state.selected_branch = Some("main-123".to_string());

        let changed = state.set_search_query("mai".to_string());

        assert!(!changed);
        assert_eq!(state.selected_branch.as_deref(), Some("main-123"));
    }

    #[test]
    fn set_search_query_prefers_exact_match_when_selection_is_filtered_out() {
        let mut state = BranchPopupState::new();
        state.local_branches = vec![branch("main"), branch("main-123")];
        state.remote_branches = vec![branch_with_kind("origin/main", true)];
        state.selected_branch = Some("main-123".to_string());

        let changed = state.set_search_query("main".to_string());

        assert!(changed);
        assert_eq!(state.selected_branch.as_deref(), Some("main"));
    }

    #[test]
    fn branch_sync_state_parses_unicode_arrow_counts() {
        let outgoing = BranchSyncState::from_tracking_status(&Some("↑2".to_string()));
        assert_eq!(outgoing.incoming, None);
        assert_eq!(outgoing.outgoing, Some(2));

        let incoming = BranchSyncState::from_tracking_status(&Some("↓7".to_string()));
        assert_eq!(incoming.incoming, Some(7));
        assert_eq!(incoming.outgoing, None);
    }

    #[test]
    fn branch_sync_state_maps_diverged_counts_to_outgoing_and_incoming() {
        let state = BranchSyncState::from_tracking_status(&Some("↕3/5".to_string()));

        assert_eq!(state.outgoing, Some(3));
        assert_eq!(state.incoming, Some(5));
    }

    #[test]
    fn branch_popup_view_renders_loaded_state_without_panicking() {
        let mut state = BranchPopupState::new();
        let mut current = branch("feature/very-long-branch-name-for-render-smoke-test");
        current.is_head = true;
        current.tracking_status = Some("↑2".to_string());
        state.local_branches = vec![current.clone(), branch("feature/api/login")];
        state.remote_branches = vec![branch_with_kind("origin/main", true)];
        state.recent_branches = vec![branch("main")];
        state.selected_branch = Some(current.name.clone());

        let _ = view(&state);
    }
}
