//! History view.
//!
//! Provides a view for browsing commit history.

use crate::i18n::I18n;
use crate::theme::{self, BadgeTone, Surface};
use crate::state::FileDisplayMode;
use crate::widgets::{self, button, scrollable, text_input, OptionalPush};
use chrono::DateTime;
use git_core::{
    commit::{get_commit, get_commit_changed_files, CommitChangeStatus, CommitChangedFile},
    history::{get_history, search_history, HistoryEntry},
    Repository,
};
use iced::mouse;
use iced::widget::canvas::{self, Canvas};
use iced::widget::{mouse_area, opaque, stack, text, Button, Column, Container, Row, Space, Text};
use iced::{Alignment, Color, Element, Font, Length, Point, Rectangle, Renderer, Theme};
use std::collections::{BTreeMap, HashSet};

/// Message types for history view.
#[derive(Debug, Clone)]
pub enum HistoryMessage {
    Refresh,
    SelectCommit(String),
    ViewDiff(String),
    ViewCommitFileDiff(String, String),
    ToggleCommitFileDisplayMode,
    CommitFileTreeEvent(crate::widgets::tree_widget::TreeMessage),
    SetSearchQuery(String),
    Search,
    ClearSearch,
    TrackContextMenuCursor(Point),
    OpenCommitContextMenu(String),
    CloseCommitContextMenu,
    CopyCommitHash(String),
    ExportCommitPatch(String),
    CompareWithCurrent(String),
    CompareWithWorktree(String),
    PrepareCreateBranch(String),
    PrepareTagFromCommit(String),
    PrepareCherryPickCommit(String),
    PrepareRevertCommit(String),
    PrepareResetCurrentBranchToCommit(String),
    PreparePushCurrentBranchToCommit(String),
    EditCommitMessage(String),
    FixupCommitToPrevious(String),
    SquashCommitToPrevious(String),
    DropCommitFromHistory(String),
    OpenInteractiveRebaseFromCommit(String),
    // 012: New commit actions matching IDEA Git.Log.ContextMenu
    UncommitToHere(String),
    PushUpToCommit(String),
    // Multi-select for squash
    ToggleMultiSelect(String),
    SquashSelectedCommits,
    // Multi-tab log messages
    SelectLogTab(usize),
    CloseLogTab(usize),
    NewLogTab,
    OpenInNewTab(String),
    // Filter bar messages
    SetBranchFilter(Option<String>),
    SetAuthorFilter(Option<String>),
    SetPathFilter(Option<String>),
    // Branches dashboard messages
    ToggleBranchesDashboard,
    DashboardSelectBranch(String),
    DashboardCheckoutBranch(String),
    DashboardMergeBranch(String),
    DashboardRebaseOnto(String),
    DashboardDeleteBranch(String),
}

/// State for the history view.
#[derive(Debug, Clone)]
pub struct HistoryState {
    pub entries: Vec<HistoryEntry>,
    pub filtered_entries: Vec<HistoryEntry>,
    pub selected_commit: Option<String>,
    pub selected_commit_info: Option<git_core::commit::CommitInfo>,
    pub selected_commit_files: Vec<CommitChangedFile>,
    pub selected_commit_file_display: FileDisplayMode,
    pub selected_commit_file_path: Option<String>,
    pub expanded_commit_file_directories: HashSet<String>,
    pub is_loading: bool,
    pub error: Option<String>,
    pub search_query: String,
    pub is_searching: bool,
    pub multi_selected_commits: Vec<String>,
    pub context_menu_commit: Option<String>,
    pub context_menu_cursor: Point,
    pub context_menu_anchor: Option<Point>,
    pub current_branch_name: Option<String>,
    pub current_upstream_ref: Option<String>,
    pub current_branch_state_hint: Option<String>,
}

impl HistoryState {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            filtered_entries: Vec::new(),
            selected_commit: None,
            selected_commit_info: None,
            selected_commit_files: Vec::new(),
            selected_commit_file_display: FileDisplayMode::default(),
            selected_commit_file_path: None,
            expanded_commit_file_directories: HashSet::new(),
            is_loading: false,
            error: None,
            search_query: String::new(),
            is_searching: false,
            multi_selected_commits: Vec::new(),
            context_menu_commit: None,
            context_menu_cursor: Point::new(0.0, 0.0),
            context_menu_anchor: None,
            current_branch_name: None,
            current_upstream_ref: None,
            current_branch_state_hint: None,
        }
    }

    fn refresh_repo_context(&mut self, repo: &Repository) {
        self.current_branch_name = repo.current_branch().ok().flatten();
        self.current_upstream_ref = repo.current_upstream_ref();
        self.current_branch_state_hint = repo.state_hint();
    }

    pub fn load_history(&mut self, repo: &Repository, i18n: &I18n) {
        self.is_loading = true;
        self.error = None;
        self.refresh_repo_context(repo);

        match get_history(repo, Some(100)) {
            Ok(entries) => {
                self.entries = entries.clone();
                self.filtered_entries = entries;
                self.is_loading = false;
                self.context_menu_commit = None;
                self.context_menu_anchor = None;
            }
            Err(error) => {
                self.error = Some(format!("{}: {error}", i18n.load_history_failed));
                self.is_loading = false;
            }
        }
    }

    pub fn select_commit(&mut self, repo: &Repository, commit_id: String, i18n: &I18n) {
        self.selected_commit = Some(commit_id.clone());
        self.selected_commit_info = None;
        self.selected_commit_files.clear();
        self.selected_commit_file_path = None;
        self.expanded_commit_file_directories.clear();
        self.error = None;
        self.refresh_repo_context(repo);

        match get_commit(repo, &commit_id) {
            Ok(info) => {
                self.selected_commit_info = Some(info);
                match get_commit_changed_files(repo, &commit_id) {
                    Ok(files) => {
                        self.expanded_commit_file_directories =
                            default_expanded_commit_file_directories(&files);
                        self.selected_commit_files = files;
                    }
                    Err(error) => {
                        self.error = Some(format!("{}: {error}", i18n.load_commit_files_failed));
                    }
                }
            }
            Err(error) => {
                self.error = Some(format!("{}: {error}", i18n.load_commit_detail_failed));
            }
        }
    }

    pub fn set_search_query(&mut self, query: String) {
        self.search_query = query;
    }

    pub fn track_context_menu_cursor(&mut self, position: Point) {
        self.context_menu_cursor = position;
    }

    pub fn perform_search(&mut self, repo: &Repository, i18n: &I18n) {
        self.context_menu_commit = None;
        self.context_menu_anchor = None;
        self.refresh_repo_context(repo);
        if self.search_query.trim().is_empty() {
            self.filtered_entries = self.entries.clone();
            self.error = None;
            return;
        }

        self.is_searching = true;
        self.error = None;
        match search_history(repo, &self.search_query, Some(100)) {
            Ok(entries) => {
                self.filtered_entries = entries;
                self.is_searching = false;
            }
            Err(error) => {
                self.error = Some(format!("{}: {error}", i18n.search_failed));
                self.is_searching = false;
            }
        }
    }

    pub fn clear_search(&mut self) {
        self.search_query = String::new();
        self.filtered_entries = self.entries.clone();
        self.error = None;
        self.is_searching = false;
        self.context_menu_commit = None;
        self.context_menu_anchor = None;
    }

    pub fn toggle_commit_file_display_mode(&mut self) {
        self.selected_commit_file_display = match self.selected_commit_file_display {
            FileDisplayMode::Flat => FileDisplayMode::Tree,
            FileDisplayMode::Tree => FileDisplayMode::Flat,
        };
    }

    pub fn toggle_commit_file_tree_node(&mut self, node_id: String) {
        if !self.expanded_commit_file_directories.remove(&node_id) {
            self.expanded_commit_file_directories.insert(node_id);
        }
    }
}

impl Default for HistoryState {
    fn default() -> Self {
        Self::new()
    }
}

/// IDEA-style relative timestamp formatter
/// Shows human-readable relative times like "just now", "5 minutes ago", "yesterday", etc.
fn format_relative_time(timestamp: i64) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    let diff = now.saturating_sub(timestamp);

    if diff < 60 {
        "just now".to_string()
    } else if diff < 3600 {
        let mins = diff / 60;
        format!("{} minute{} ago", mins, if mins == 1 { "" } else { "s" })
    } else if diff < 86400 {
        let hours = diff / 3600;
        format!("{} hour{} ago", hours, if hours == 1 { "" } else { "s" })
    } else if diff < 172800 {
        "yesterday".to_string()
    } else if diff < 604800 {
        let days = diff / 86400;
        format!("{} day{} ago", days, if days == 1 { "" } else { "s" })
    } else if diff < 2592000 {
        let weeks = diff / 604800;
        format!("{} week{} ago", weeks, if weeks == 1 { "" } else { "s" })
    } else if diff < 31536000 {
        let months = diff / 2592000;
        format!("{} month{} ago", months, if months == 1 { "" } else { "s" })
    } else {
        // For older dates, show absolute date
        let datetime = DateTime::from_timestamp(timestamp, 0)
            .unwrap_or_else(|| DateTime::from_timestamp(0, 0).unwrap());
        datetime.format("%Y-%m-%d").to_string()
    }
}

/// Fallback absolute timestamp for older dates
fn format_timestamp(timestamp: i64) -> String {
    let datetime = DateTime::from_timestamp(timestamp, 0)
        .unwrap_or_else(|| DateTime::from_timestamp(0, 0).unwrap());
    datetime.format("%Y-%m-%d %H:%M:%S").to_string()
}

const HISTORY_ROW_HEIGHT: f32 = 22.0;
const HISTORY_CONTEXT_MENU_WIDTH: f32 = 280.0;
const HISTORY_CONTEXT_MENU_ESTIMATED_HEIGHT: f32 = 340.0;
const HISTORY_CONTEXT_MENU_EDGE_PADDING: f32 = 8.0;
const HISTORY_GRAPH_LANE_WIDTH: f32 = 14.0;
const HISTORY_GRAPH_PADDING: f32 = 8.0;
const HISTORY_GRAPH_MIN_WIDTH: f32 = 56.0;
const HISTORY_GRAPH_LINE_WIDTH: f32 = 1.5;
const HISTORY_GRAPH_NODE_RADIUS: f32 = 3.0;

#[derive(Debug, Clone)]
struct LaneState {
    commit_id: String,
    color_index: usize,
}

#[derive(Debug, Clone)]
struct GraphLane {
    lane: usize,
    color_index: usize,
}

#[derive(Debug, Clone)]
struct GraphTransition {
    from_lane: usize,
    to_lane: usize,
    color_index: usize,
}

#[derive(Debug, Clone)]
struct HistoryGraphRow {
    top_lanes: Vec<GraphLane>,
    continuing: Vec<GraphTransition>,
    parent_transitions: Vec<GraphTransition>,
    node_lane: usize,
    node_color_index: usize,
    total_lanes: usize,
}

#[derive(Debug, Clone)]
struct HistoryGraphLayout {
    rows: Vec<HistoryGraphRow>,
    lane_count: usize,
}

#[derive(Debug, Clone)]
struct HistoryGraphCanvas {
    row: HistoryGraphRow,
    is_selected: bool,
}

impl<Message> canvas::Program<Message> for HistoryGraphCanvas {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let center_y = bounds.height / 2.0;
        let bottom_y = bounds.height;

        for lane in &self.row.top_lanes {
            stroke_segment(
                &mut frame,
                lane_center_x(lane.lane),
                0.0,
                lane_center_x(lane.lane),
                center_y,
                history_graph_color(lane.color_index),
            );
        }

        for transition in &self.row.continuing {
            stroke_segment(
                &mut frame,
                lane_center_x(transition.from_lane),
                center_y,
                lane_center_x(transition.to_lane),
                bottom_y,
                history_graph_color(transition.color_index),
            );
        }

        for transition in &self.row.parent_transitions {
            stroke_segment(
                &mut frame,
                lane_center_x(transition.from_lane),
                center_y,
                lane_center_x(transition.to_lane),
                bottom_y,
                history_graph_color(transition.color_index),
            );
        }

        if self.is_selected {
            let halo = canvas::Path::circle(
                Point::new(lane_center_x(self.row.node_lane), center_y),
                HISTORY_GRAPH_NODE_RADIUS + 2.5,
            );
            frame.fill(&halo, theme::darcula::SELECTION_BG.scale_alpha(0.55));
        }

        let node = canvas::Path::circle(
            Point::new(lane_center_x(self.row.node_lane), center_y),
            HISTORY_GRAPH_NODE_RADIUS,
        );
        let node_color = history_graph_color(self.row.node_color_index);
        frame.fill(&node, node_color);
        frame.stroke(
            &node,
            canvas::Stroke::default()
                .with_color(theme::darcula::BG_EDITOR)
                .with_width(1.2),
        );

        vec![frame.into_geometry()]
    }
}

fn build_history_graph(entries: &[HistoryEntry]) -> HistoryGraphLayout {
    let mut active_lanes: Vec<LaneState> = Vec::new();
    let mut rows = Vec::with_capacity(entries.len());
    let mut next_color_index = 0usize;
    let mut max_lane_count = 1usize;

    for entry in entries {
        let incoming = active_lanes.clone();

        let (working_lanes, node_lane, node_color_index) =
            if let Some(position) = incoming.iter().position(|lane| lane.commit_id == entry.id) {
                (incoming.clone(), position, incoming[position].color_index)
            } else {
                let mut lanes = incoming.clone();
                let color_index = next_color_index;
                next_color_index += 1;
                lanes.push(LaneState {
                    commit_id: entry.id.clone(),
                    color_index,
                });
                let node_lane = lanes.len() - 1;
                (lanes, node_lane, color_index)
            };

        let mut after = working_lanes.clone();

        if entry.parent_ids.is_empty() {
            after.remove(node_lane);
        } else {
            let first_parent = &entry.parent_ids[0];
            let existing_first_parent = after
                .iter()
                .enumerate()
                .find(|(index, lane)| *index != node_lane && lane.commit_id == *first_parent)
                .map(|(index, _)| index);

            if existing_first_parent.is_some() {
                after.remove(node_lane);
            } else if let Some(current_lane) = after.get_mut(node_lane) {
                current_lane.commit_id = first_parent.clone();
            }

            let mut insertion_index = (node_lane + 1).min(after.len());
            for parent in entry.parent_ids.iter().skip(1) {
                if after.iter().any(|lane| lane.commit_id == *parent) {
                    continue;
                }

                let color_index = next_color_index;
                next_color_index += 1;
                after.insert(
                    insertion_index,
                    LaneState {
                        commit_id: parent.clone(),
                        color_index,
                    },
                );
                insertion_index += 1;
            }
        }

        let top_lanes = incoming
            .iter()
            .enumerate()
            .map(|(lane, state)| GraphLane {
                lane,
                color_index: state.color_index,
            })
            .collect::<Vec<_>>();

        let continuing = incoming
            .iter()
            .enumerate()
            .filter_map(|(from_lane, lane)| {
                if lane.commit_id == entry.id {
                    return None;
                }

                after
                    .iter()
                    .position(|next_lane| next_lane.commit_id == lane.commit_id)
                    .map(|to_lane| GraphTransition {
                        from_lane,
                        to_lane,
                        color_index: lane.color_index,
                    })
            })
            .collect::<Vec<_>>();

        let mut seen_parent_targets = HashSet::new();
        let parent_transitions = entry
            .parent_ids
            .iter()
            .filter_map(|parent| {
                let target_lane = after.iter().position(|lane| lane.commit_id == *parent)?;

                if !seen_parent_targets.insert(target_lane) {
                    return None;
                }

                Some(GraphTransition {
                    from_lane: node_lane,
                    to_lane: target_lane,
                    color_index: after[target_lane].color_index,
                })
            })
            .collect::<Vec<_>>();

        max_lane_count = max_lane_count
            .max(incoming.len())
            .max(after.len())
            .max(node_lane + 1);

        rows.push(HistoryGraphRow {
            top_lanes,
            continuing,
            parent_transitions,
            node_lane,
            node_color_index,
            total_lanes: 1,
        });

        active_lanes = after;
    }

    for row in &mut rows {
        row.total_lanes = max_lane_count.max(1);
    }

    HistoryGraphLayout {
        rows,
        lane_count: max_lane_count.max(1),
    }
}

fn build_commit_row<'a>(
    entry: &'a HistoryEntry,
    graph_row: &HistoryGraphRow,
    graph_width: f32,
    is_selected: bool,
    is_menu_open: bool,
) -> Element<'a, HistoryMessage> {
    let subject = commit_subject(&entry.message);

    let graph = Canvas::new(HistoryGraphCanvas {
        row: graph_row.clone(),
        is_selected,
    })
    .width(Length::Fixed(graph_width))
    .height(Length::Fixed(HISTORY_ROW_HEIGHT));

    // IDEA-style compact row: graph | hash | message | author | date
    let row = Container::new(
        Row::new()
            .spacing(theme::spacing::SM)
            .align_y(Alignment::Center)
            .push(graph)
            .push(
                Text::new(short_commit_id(&entry.id))
                    .size(11)
                    .font(Font::MONOSPACE)
                    .width(Length::Fixed(60.0))
                    .wrapping(text::Wrapping::None)
                    .color(theme::darcula::TEXT_DISABLED),
            )
            .push(
                Text::new(subject)
                    .size(12)
                    .width(Length::Fill)
                    .wrapping(text::Wrapping::WordOrGlyph),
            )
            .push(
                Text::new(&entry.author_name)
                    .size(11)
                    .width(Length::Fixed(100.0))
                    .wrapping(text::Wrapping::WordOrGlyph)
                    .color(theme::darcula::TEXT_SECONDARY),
            )
            .push(
                Text::new(format_relative_time(entry.timestamp))
                    .size(11)
                    .width(Length::Fixed(80.0))
                    .wrapping(text::Wrapping::None)
                    .color(theme::darcula::TEXT_DISABLED),
            ),
    )
    .padding([2, 6])
    .style(theme::panel_style(if is_menu_open || is_selected {
        Surface::Selection
    } else {
        Surface::Editor
    }));

    mouse_area(
        Container::new(
            Button::new(row)
                .width(Length::Fill)
                .style(widgets::menu::trigger_row_button_style(
                    is_selected,
                    is_menu_open,
                    Some(theme::darcula::ACCENT),
                ))
                .on_press(HistoryMessage::SelectCommit(entry.id.clone())),
        )
        .width(Length::Fill),
    )
    .on_right_press(HistoryMessage::OpenCommitContextMenu(entry.id.clone()))
    .interaction(mouse::Interaction::Pointer)
    .into()
}

fn build_history_list<'a>(state: &'a HistoryState, i18n: &'a I18n) -> Element<'a, HistoryMessage> {
    let entries = &state.filtered_entries;
    let graph = build_history_graph(entries);
    let graph_width = (graph.lane_count as f32 * HISTORY_GRAPH_LANE_WIDTH
        + HISTORY_GRAPH_PADDING * 2.0)
        .max(HISTORY_GRAPH_MIN_WIDTH);

    let list = if entries.is_empty() {
        Column::new().push(
            Text::new(i18n.no_commits_to_display)
                .size(12)
                .color(theme::darcula::TEXT_SECONDARY),
        )
    } else {
        entries.iter().zip(graph.rows.iter()).fold(
            Column::new().spacing(1),
            |column, (entry, graph_row)| {
                let is_selected = state
                    .selected_commit
                    .as_deref()
                    .map(|value| value == entry.id)
                    .unwrap_or(false);
                let is_menu_open = state
                    .context_menu_commit
                    .as_deref()
                    .map(|value| value == entry.id)
                    .unwrap_or(false);
                column.push(build_commit_row(
                    entry,
                    graph_row,
                    graph_width,
                    is_selected,
                    is_menu_open,
                ))
            },
        )
    };

    mouse_area(
        Container::new(
            Column::new()
                .spacing(theme::spacing::SM)
                .push(
                    Row::new()
                        .spacing(theme::spacing::XS)
                        .align_y(Alignment::Center)
                        .push(
                            Text::new(i18n.commits_label)
                                .size(13)
                                .color(theme::darcula::TEXT_SECONDARY),
                        )
                        .push(widgets::info_chip::<HistoryMessage>(
                            entries.len().to_string(),
                            BadgeTone::Neutral,
                        )),
                )
                .push(
                    Container::new(
                        Row::new()
                            .align_y(Alignment::Center)
                            .push(
                                Text::new(i18n.graph_label)
                                    .size(10)
                                    .width(Length::Fixed(graph_width))
                                    .color(theme::darcula::TEXT_DISABLED),
                            )
                            .push(
                                Text::new(i18n.commit)
                                    .size(10)
                                    .width(Length::FillPortion(5))
                                    .color(theme::darcula::TEXT_DISABLED),
                            )
                            .push(
                                Text::new(i18n.author_hash)
                                    .size(10)
                                    .width(Length::FillPortion(3))
                                    .color(theme::darcula::TEXT_DISABLED),
                            )
                            .push(
                                Text::new(i18n.time_label)
                                    .size(10)
                                    .width(Length::FillPortion(2))
                                    .color(theme::darcula::TEXT_DISABLED),
                            ),
                    )
                    .padding([6, 8])
                    .style(theme::panel_style(Surface::ToolbarField)),
                )
                .push(scrollable::styled(list).height(Length::Fill)),
        )
        .padding([8, 8])
        .style(theme::panel_style(Surface::Panel)),
    )
    .on_move(HistoryMessage::TrackContextMenuCursor)
    .into()
}

fn build_commit_context_menu_overlay<'a>(state: &'a HistoryState, i18n: &'a I18n) -> Element<'a, HistoryMessage> {
    let Some(commit_id) = state.context_menu_commit.as_deref() else {
        return Space::new().width(Length::Shrink).into();
    };
    let anchor = state
        .context_menu_anchor
        .unwrap_or(state.context_menu_cursor);
    let Some(entry) = state
        .filtered_entries
        .iter()
        .find(|entry| entry.id == commit_id)
        .or_else(|| state.entries.iter().find(|entry| entry.id == commit_id))
    else {
        return Space::new().width(Length::Shrink).into();
    };

    let selected_info = state
        .selected_commit
        .as_deref()
        .filter(|selected| *selected == entry.id)
        .and(state.selected_commit_info.as_ref());
    let has_current_branch = state.current_branch_name.is_some();
    let has_upstream = state.current_upstream_ref.is_some();
    let commit_detail_ready = selected_info.is_some();
    let is_merge_commit = selected_info.is_some_and(|info| info.parent_ids.len() > 1);
    let is_root_commit = selected_info.is_some_and(|info| info.parent_ids.is_empty());

    let _compare_with_current_detail = if let Some(branch_name) = state.current_branch_name.as_ref()
    {
        i18n.ctx_compare_branch_fmt.replace("{}", branch_name)
    } else {
        i18n.ctx_detached_no_branch_compare.to_string()
    };
    let _compare_with_worktree_detail =
        if let Some(branch_name) = state.current_branch_name.as_ref() {
            i18n.ctx_compare_worktree_fmt.replace("{}", branch_name)
        } else {
            i18n.ctx_detached_worktree_diff.to_string()
        };
    let cherry_pick_detail = if !commit_detail_ready {
        i18n.ctx_commit_detail_not_ready.to_string()
    } else if is_merge_commit {
        i18n.ctx_merge_no_cherry_pick.to_string()
    } else if let Some(branch_name) = state.current_branch_name.as_ref() {
        i18n.ctx_cherry_pick_branch_fmt.replace("{}", branch_name)
    } else {
        i18n.ctx_detached_no_cherry_pick.to_string()
    };
    let revert_detail = if !commit_detail_ready {
        i18n.ctx_commit_detail_not_ready.to_string()
    } else if is_merge_commit {
        i18n.ctx_merge_no_revert.to_string()
    } else if let Some(branch_name) = state.current_branch_name.as_ref() {
        i18n.ctx_revert_branch_fmt.replace("{}", branch_name)
    } else {
        i18n.ctx_detached_no_revert.to_string()
    };
    let reset_detail = if let Some(branch_name) = state.current_branch_name.as_ref() {
        i18n.ctx_reset_branch_fmt.replace("{}", branch_name)
    } else {
        i18n.ctx_detached_no_reset.to_string()
    };
    let push_to_here_detail = if let Some(upstream_ref) = state.current_upstream_ref.as_ref() {
        i18n.ctx_push_upstream_fmt.replace("{}", upstream_ref)
    } else if has_current_branch {
        i18n.ctx_no_upstream_no_push.to_string()
    } else {
        i18n.ctx_detached_no_push_here.to_string()
    };
    let edit_message_detail = if !commit_detail_ready {
        i18n.ctx_commit_detail_not_ready.to_string()
    } else if is_merge_commit {
        i18n.ctx_merge_no_reword.to_string()
    } else if let Some(branch_name) = state.current_branch_name.as_ref() {
        i18n.ctx_reword_branch_fmt.replace("{}", branch_name)
    } else {
        i18n.ctx_detached_no_rewrite.to_string()
    };
    let fixup_detail = if !commit_detail_ready {
        i18n.ctx_commit_detail_not_ready.to_string()
    } else if is_merge_commit {
        i18n.ctx_merge_no_fixup.to_string()
    } else if is_root_commit {
        i18n.ctx_root_no_merge_target.to_string()
    } else {
        i18n.ctx_fixup_hint.to_string()
    };
    let squash_detail = if !commit_detail_ready {
        i18n.ctx_commit_detail_not_ready.to_string()
    } else if is_merge_commit {
        i18n.ctx_merge_no_squash.to_string()
    } else if is_root_commit {
        i18n.ctx_root_no_merge_target.to_string()
    } else {
        i18n.ctx_squash_hint.to_string()
    };
    let drop_detail = if !commit_detail_ready {
        i18n.ctx_commit_detail_not_ready.to_string()
    } else if is_merge_commit {
        i18n.ctx_merge_no_drop.to_string()
    } else {
        i18n.ctx_drop_hint.to_string()
    };
    let interactive_rebase_detail = if has_current_branch {
        i18n.ctx_rebase_hint.to_string()
    } else {
        i18n.ctx_detached_no_rebase.to_string()
    };

    // IDEA Git.Log.ContextMenu — exact order from intellij.vcs.git.xml
    let actions = Column::new()
        .spacing(theme::spacing::XS)
        // Group 1: Reset, Revert, Uncommit (IDEA: first group)
        .push(history_context_group(
            i18n.reset_group,
            "",
            vec![
                history_context_action_row(
                    i18n.reset_to_here,
                    reset_detail,
                    (!state.is_loading && has_current_branch).then_some(
                        HistoryMessage::PrepareResetCurrentBranchToCommit(entry.id.clone()),
                    ),
                    widgets::menu::MenuTone::Danger,
                ),
                history_context_action_row(
                    i18n.revert_commit_label,
                    revert_detail,
                    (!state.is_loading
                        && commit_detail_ready
                        && !is_merge_commit
                        && has_current_branch)
                        .then_some(HistoryMessage::PrepareRevertCommit(entry.id.clone())),
                    widgets::menu::MenuTone::Neutral,
                ),
                history_context_action_row(
                    i18n.uncommit_label,
                    i18n.soft_reset_hint.to_string(),
                    (!state.is_loading && has_current_branch)
                        .then_some(HistoryMessage::UncommitToHere(entry.id.clone())),
                    widgets::menu::MenuTone::Neutral,
                ),
            ],
        ))
        // Group 2: History Rewrite — Reword, Fixup, Squash, Drop, Interactive Rebase
        .push(history_context_group(
            i18n.history_rewrite_group,
            "",
            vec![
                history_context_action_row(
                    i18n.reword_commit,
                    edit_message_detail,
                    (!state.is_loading
                        && commit_detail_ready
                        && !is_merge_commit
                        && has_current_branch)
                        .then_some(HistoryMessage::EditCommitMessage(entry.id.clone())),
                    widgets::menu::MenuTone::Neutral,
                ),
                history_context_action_row(
                    i18n.fixup_to_commit,
                    fixup_detail,
                    (!state.is_loading
                        && commit_detail_ready
                        && !is_merge_commit
                        && !is_root_commit
                        && has_current_branch)
                        .then_some(HistoryMessage::FixupCommitToPrevious(entry.id.clone())),
                    widgets::menu::MenuTone::Neutral,
                ),
                history_context_action_row(
                    i18n.squash_into_commit,
                    squash_detail,
                    (!state.is_loading
                        && commit_detail_ready
                        && !is_merge_commit
                        && !is_root_commit
                        && has_current_branch)
                        .then_some(HistoryMessage::SquashCommitToPrevious(entry.id.clone())),
                    widgets::menu::MenuTone::Neutral,
                ),
                history_context_action_row(
                    i18n.drop_commit,
                    drop_detail,
                    (!state.is_loading
                        && commit_detail_ready
                        && !is_merge_commit
                        && has_current_branch)
                        .then_some(HistoryMessage::DropCommitFromHistory(entry.id.clone())),
                    widgets::menu::MenuTone::Danger,
                ),
                history_context_action_row(
                    i18n.interactive_rebase,
                    interactive_rebase_detail,
                    (!state.is_loading && has_current_branch).then_some(
                        HistoryMessage::OpenInteractiveRebaseFromCommit(entry.id.clone()),
                    ),
                    widgets::menu::MenuTone::Neutral,
                ),
                history_context_action_row(
                    i18n.push_up_to_commit,
                    push_to_here_detail,
                    (!state.is_loading && has_current_branch && has_upstream).then_some(
                        HistoryMessage::PreparePushCurrentBranchToCommit(entry.id.clone()),
                    ),
                    widgets::menu::MenuTone::Neutral,
                ),
            ],
        ))
        // Group 3: Ref Actions — Branch, Tag, Cherry-pick
        .push(history_context_group(
            i18n.ref_actions_group,
            "",
            vec![
                history_context_action_row(
                    i18n.create_branch,
                    i18n.ctx_create_branch_hint.to_string(),
                    (!state.is_loading)
                        .then_some(HistoryMessage::PrepareCreateBranch(entry.id.clone())),
                    widgets::menu::MenuTone::Neutral,
                ),
                history_context_action_row(
                    i18n.create_tag,
                    i18n.ctx_create_tag_hint.to_string(),
                    (!state.is_loading)
                        .then_some(HistoryMessage::PrepareTagFromCommit(entry.id.clone())),
                    widgets::menu::MenuTone::Neutral,
                ),
                history_context_action_row(
                    i18n.cherry_pick,
                    cherry_pick_detail,
                    (!state.is_loading
                        && commit_detail_ready
                        && !is_merge_commit
                        && has_current_branch)
                        .then_some(HistoryMessage::PrepareCherryPickCommit(entry.id.clone())),
                    widgets::menu::MenuTone::Neutral,
                ),
            ],
        ))
        // Group 4: Copy
        .push(history_context_group(
            i18n.copy_group,
            "",
            vec![
                history_context_action_row(
                    i18n.copy_hash,
                    "".to_string(),
                    Some(HistoryMessage::CopyCommitHash(entry.id.clone())),
                    widgets::menu::MenuTone::Neutral,
                ),
                history_context_action_row(
                    i18n.export_patch,
                    "".to_string(),
                    Some(HistoryMessage::ExportCommitPatch(entry.id.clone())),
                    widgets::menu::MenuTone::Neutral,
                ),
            ],
        ));

    // IDEA-style: compact menu with just the action list, no verbose header
    let menu = Container::new(scrollable::styled(actions).height(Length::Shrink))
        .padding([6, 8])
        .width(Length::Fixed(HISTORY_CONTEXT_MENU_WIDTH))
        .style(widgets::menu::panel_style);

    build_history_context_menu_layer(anchor, menu.into())
}

fn history_context_group<'a>(
    title: &str,
    detail: &str,
    rows: Vec<Element<'a, HistoryMessage>>,
) -> Element<'a, HistoryMessage> {
    widgets::menu::group(title, detail, widgets::menu::MenuTone::Neutral, rows)
}

fn history_context_action_row<'a>(
    title: &str,
    detail: String,
    message: Option<HistoryMessage>,
    tone: widgets::menu::MenuTone,
) -> Element<'a, HistoryMessage> {
    widgets::menu::action_row(None, title, Some(detail), None, message, tone)
}

fn build_history_context_menu_layer<'a>(
    anchor: Point,
    menu: Element<'a, HistoryMessage>,
) -> Element<'a, HistoryMessage> {
    let origin = history_context_menu_origin(anchor);

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
        .on_press(HistoryMessage::CloseCommitContextMenu),
    )
}

fn history_context_menu_origin(anchor: Point) -> Point {
    let x = if anchor.x > HISTORY_CONTEXT_MENU_WIDTH * 0.68 {
        (anchor.x - HISTORY_CONTEXT_MENU_WIDTH + 28.0).max(HISTORY_CONTEXT_MENU_EDGE_PADDING)
    } else {
        (anchor.x + 6.0).max(HISTORY_CONTEXT_MENU_EDGE_PADDING)
    };
    let y = if anchor.y > HISTORY_CONTEXT_MENU_ESTIMATED_HEIGHT * 0.52 {
        (anchor.y - HISTORY_CONTEXT_MENU_ESTIMATED_HEIGHT + 18.0)
            .max(HISTORY_CONTEXT_MENU_EDGE_PADDING)
    } else {
        (anchor.y + 6.0).max(HISTORY_CONTEXT_MENU_EDGE_PADDING)
    };

    Point::new(x, y)
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

fn lane_center_x(lane: usize) -> f32 {
    HISTORY_GRAPH_PADDING + lane as f32 * HISTORY_GRAPH_LANE_WIDTH + HISTORY_GRAPH_LANE_WIDTH / 2.0
}

fn stroke_segment(
    frame: &mut canvas::Frame<Renderer>,
    from_x: f32,
    from_y: f32,
    to_x: f32,
    to_y: f32,
    color: Color,
) {
    let path = canvas::Path::line(Point::new(from_x, from_y), Point::new(to_x, to_y));
    frame.stroke(
        &path,
        canvas::Stroke::default()
            .with_color(color)
            .with_width(HISTORY_GRAPH_LINE_WIDTH)
            .with_line_cap(canvas::LineCap::Round)
            .with_line_join(canvas::LineJoin::Round),
    );
}

/// Branch-lane palette — IDEA-style graph colors for Darcula theme.
/// Colors are vivid enough to distinguish lanes on #2B2B2B background.
fn history_graph_color(index: usize) -> Color {
    match index % 8 {
        0 => Color::from_rgb(0.345, 0.616, 0.965), // IDEA blue #589DF6
        1 => Color::from_rgb(0.212, 0.710, 0.361), // IDEA green #369650 brighter
        2 => Color::from_rgb(0.624, 0.471, 0.710), // IDEA purple #9F79B5
        3 => Color::from_rgb(0.369, 0.678, 0.831), // IDEA cyan #5EACD0
        4 => Color::from_rgb(0.851, 0.639, 0.263), // IDEA gold #D9A343
        5 => Color::from_rgb(0.682, 0.588, 0.337), // IDEA tag #AE9656
        6 => Color::from_rgb(0.863, 0.431, 0.478), // rose
        _ => Color::from_rgb(0.682, 0.816, 0.576), // IDEA commit graph #AEB9C0 warm
    }
}

fn build_commit_detail<'a>(
    state: &'a HistoryState,
    info: &'a git_core::commit::CommitInfo,
    i18n: &'a I18n,
) -> Element<'a, HistoryMessage> {
    Container::new(
        Column::new()
            .spacing(theme::spacing::XS)
            .height(Length::Fill)
            .push(build_commit_summary_panel(info, i18n))
            .push(build_commit_files_panel(state, info.id.as_str(), i18n)),
    )
    .height(Length::Fill)
    .into()
}

fn build_commit_summary_panel<'a>(info: &'a git_core::commit::CommitInfo, i18n: &'a I18n) -> Element<'a, HistoryMessage> {
    Container::new(
        Column::new()
            .spacing(theme::spacing::SM)
            .push(
                Row::new()
                    .spacing(theme::spacing::XS)
                    .align_y(Alignment::Center)
                    .push(
                        Text::new(i18n.commit_detail_label)
                            .size(12)
                            .color(theme::darcula::TEXT_PRIMARY),
                    )
                    .push(widgets::compact_chip::<HistoryMessage>(
                        info.id[..8].to_string(),
                        BadgeTone::Accent,
                    ))
                    .push(Space::new().width(Length::Fill)),
            )
            .push(iced::widget::rule::horizontal(1))
            .push(scrollable::styled(Text::new(&info.message).size(13)).height(Length::Fixed(120.0)))
            .push(iced::widget::rule::horizontal(1))
            .push(
                Column::new()
                    .spacing(2)
                    .push(detail_meta_row(
                        i18n.author_label,
                        format!("{} <{}>", info.author_name, info.author_email),
                    ))
                    .push(detail_meta_row(i18n.time_label, format_timestamp(info.author_time)))
                    .push(detail_meta_row(
                        i18n.parent_commits,
                        format!("{}", info.parent_ids.len()),
                    )),
            ),
    )
    .padding([8, 10])
    .style(theme::panel_style(Surface::Panel))
    .into()
}

fn build_commit_files_panel<'a>(
    state: &'a HistoryState,
    commit_id: &'a str,
    i18n: &'a I18n,
) -> Element<'a, HistoryMessage> {
    let file_count = state.selected_commit_files.len();
    let content = if state.selected_commit_files.is_empty() {
        widgets::panel_empty_state_compact(
            i18n.no_file_changes,
            i18n.no_file_changes_hint,
        )
    } else {
        match state.selected_commit_file_display {
            FileDisplayMode::Flat => build_commit_file_flat_list(state, commit_id),
            FileDisplayMode::Tree => build_commit_file_tree(state),
        }
    };

    Container::new(
        Column::new()
            .spacing(theme::spacing::XS)
            .height(Length::Fill)
            .push(
                Row::new()
                    .spacing(theme::spacing::XS)
                    .align_y(Alignment::Center)
                    .push(
                        Text::new(i18n.changed_files)
                            .size(12)
                            .color(theme::darcula::TEXT_PRIMARY),
                    )
                    .push(widgets::info_chip::<HistoryMessage>(
                        i18n.n_files_fmt.replace("{}", &file_count.to_string()),
                        BadgeTone::Neutral,
                    ))
                    .push(Space::new().width(Length::Fill))
                    .push(button::tab(
                        i18n.flat_label,
                        state.selected_commit_file_display == FileDisplayMode::Flat,
                        (state.selected_commit_file_display != FileDisplayMode::Flat)
                            .then_some(HistoryMessage::ToggleCommitFileDisplayMode),
                    ))
                    .push(button::tab(
                        i18n.tree_label,
                        state.selected_commit_file_display == FileDisplayMode::Tree,
                        (state.selected_commit_file_display != FileDisplayMode::Tree)
                            .then_some(HistoryMessage::ToggleCommitFileDisplayMode),
                    )),
            )
            .push(iced::widget::rule::horizontal(1))
            .push(Container::new(scrollable::styled(content).height(Length::Fill)).height(Length::Fill)),
    )
    .padding([8, 10])
    .height(Length::Fill)
    .style(theme::panel_style(Surface::Panel))
    .into()
}

fn build_commit_file_flat_list<'a>(
    state: &'a HistoryState,
    commit_id: &'a str,
) -> Element<'a, HistoryMessage> {
    state
        .selected_commit_files
        .iter()
        .fold(Column::new().spacing(2), |column, file| {
            column.push(build_commit_file_row(
                file,
                commit_id,
                state.selected_commit_file_path.as_deref() == Some(file.path.as_str()),
            ))
        })
        .into()
}

fn build_commit_file_tree<'a>(state: &'a HistoryState) -> Element<'a, HistoryMessage> {
    let mut groups: BTreeMap<String, Vec<&CommitChangedFile>> = BTreeMap::new();
    let mut root_files = Vec::new();

    for file in &state.selected_commit_files {
        let directory = std::path::Path::new(&file.path)
            .parent()
            .and_then(|path| path.to_str())
            .filter(|path| !path.is_empty());
        if let Some(directory) = directory {
            groups.entry(directory.to_string()).or_default().push(file);
        } else {
            root_files.push(file);
        }
    }

    let mut column = Column::new().spacing(2);

    for file in root_files {
        column = column.push(build_commit_file_tree_leaf_row(
            file,
            state.selected_commit_file_path.as_deref() == Some(file.path.as_str()),
        ));
    }

    for (directory, group_files) in groups {
        let node_id = commit_file_directory_node_id(&directory);
        let expanded = state.expanded_commit_file_directories.contains(&node_id);
        column = column.push(
            Button::new(
                Container::new(
                    Row::new()
                        .spacing(4)
                        .align_y(Alignment::Center)
                        .push(
                            Text::new(if expanded { "▼" } else { "▶" })
                                .size(10)
                                .color(theme::darcula::TEXT_SECONDARY),
                        )
                        .push(
                            Text::new("📁")
                                .size(11)
                                .color(theme::darcula::TEXT_DISABLED),
                        )
                        .push(
                            Text::new(directory.clone())
                                .size(11)
                                .color(theme::darcula::TEXT_SECONDARY),
                        )
                        .push(
                            Text::new(format!("({})", group_files.len()))
                                .size(10)
                                .color(theme::darcula::TEXT_DISABLED),
                        ),
                )
                .padding([2, 4])
                .width(Length::Fill),
            )
            .width(Length::Fill)
            .style(theme::button_style(theme::ButtonTone::Ghost))
            .on_press(HistoryMessage::CommitFileTreeEvent(
                crate::widgets::tree_widget::TreeMessage::ToggleNode(node_id.clone()),
            )),
        );

        if expanded {
            for file in group_files {
                column = column.push(build_commit_file_tree_leaf_row(
                    file,
                    state.selected_commit_file_path.as_deref() == Some(file.path.as_str()),
                ));
            }
        }
    }

    column.into()
}

fn build_commit_file_tree_leaf_row<'a>(
    file: &'a CommitChangedFile,
    is_selected: bool,
) -> Element<'a, HistoryMessage> {
    let file_name = std::path::Path::new(&file.path)
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or(&file.path);

    Button::new(
        Container::new(
            Row::new()
                .spacing(theme::spacing::XS)
                .align_y(Alignment::Center)
                .push(Space::new().width(Length::Fixed(18.0)))
                .push(
                    Text::new(commit_change_status_symbol(file.status))
                        .size(12)
                        .color(commit_change_status_color(file.status)),
                )
                .push(
                    Column::new()
                        .spacing(1)
                        .width(Length::Fill)
                        .push(
                            Text::new(file_name)
                                .size(12)
                                .width(Length::Fill)
                                .wrapping(text::Wrapping::WordOrGlyph),
                        )
                        .push_maybe(file.old_path.as_ref().map(|old_path| {
                            Text::new(old_path.as_str())
                                .size(10)
                                .width(Length::Fill)
                                .wrapping(text::Wrapping::WordOrGlyph)
                                .color(theme::darcula::TEXT_SECONDARY)
                        })),
                ),
        )
        .padding([3, 6])
        .width(Length::Fill)
        .style(theme::panel_style(if is_selected {
            Surface::ListSelection
        } else {
            Surface::ListRow
        })),
    )
    .width(Length::Fill)
    .style(theme::button_style(theme::ButtonTone::Ghost))
    .on_press(HistoryMessage::CommitFileTreeEvent(
        crate::widgets::tree_widget::TreeMessage::SelectNode(format!("file:{}", file.path)),
    ))
    .into()
}

fn build_commit_file_row<'a>(
    file: &'a CommitChangedFile,
    commit_id: &'a str,
    is_selected: bool,
) -> Element<'a, HistoryMessage> {
    let path_row = Column::new()
        .spacing(1)
        .width(Length::Fill)
        .push(
            Text::new(file.path.as_str())
                .size(12)
                .width(Length::Fill)
                .wrapping(text::Wrapping::WordOrGlyph),
        )
        .push_maybe(file.old_path.as_ref().map(|old_path| {
            Text::new(format!("{old_path} → {}", file.path))
                .size(10)
                .width(Length::Fill)
                .wrapping(text::Wrapping::WordOrGlyph)
                .color(theme::darcula::TEXT_SECONDARY)
        }));

    Button::new(
        Container::new(
            Row::new()
                .spacing(theme::spacing::XS)
                .align_y(Alignment::Center)
                .push(
                    Text::new(commit_change_status_symbol(file.status))
                        .size(12)
                        .color(commit_change_status_color(file.status)),
                )
                .push(path_row),
        )
        .padding([4, 6])
        .width(Length::Fill)
        .style(theme::panel_style(if is_selected {
            Surface::ListSelection
        } else {
            Surface::ListRow
        })),
    )
    .width(Length::Fill)
    .style(theme::button_style(theme::ButtonTone::Ghost))
    .on_press(HistoryMessage::ViewCommitFileDiff(
        commit_id.to_string(),
        file.path.clone(),
    ))
    .into()
}

fn commit_change_status_symbol(status: CommitChangeStatus) -> &'static str {
    match status {
        CommitChangeStatus::Added => "A",
        CommitChangeStatus::Modified => "M",
        CommitChangeStatus::Deleted => "D",
        CommitChangeStatus::Renamed => "R",
    }
}

fn commit_change_status_color(status: CommitChangeStatus) -> Color {
    match status {
        CommitChangeStatus::Added => theme::darcula::STATUS_ADDED,
        CommitChangeStatus::Modified => theme::darcula::STATUS_MODIFIED,
        CommitChangeStatus::Deleted => theme::darcula::STATUS_DELETED,
        CommitChangeStatus::Renamed => theme::darcula::STATUS_RENAMED,
    }
}

fn commit_file_directory_node_id(directory: &str) -> String {
    format!("dir:{directory}")
}

fn default_expanded_commit_file_directories(
    files: &[CommitChangedFile],
) -> HashSet<String> {
    files.iter()
        .filter_map(|file| {
            std::path::Path::new(&file.path)
                .parent()
                .and_then(|path| path.to_str())
                .filter(|path| !path.is_empty())
                .map(commit_file_directory_node_id)
        })
        .collect()
}

fn detail_meta_row<'a>(label: &'a str, value: impl ToString) -> Element<'a, HistoryMessage> {
    Row::new()
        .spacing(theme::spacing::XS)
        .align_y(Alignment::Center)
        .push(
            Text::new(label)
                .size(10)
                .color(theme::darcula::TEXT_DISABLED)
                .width(Length::Fixed(42.0)),
        )
        .push(
            Text::new(value.to_string())
                .size(11)
                .color(theme::darcula::TEXT_SECONDARY)
                .width(Length::Fill)
                .wrapping(text::Wrapping::WordOrGlyph),
        )
        .into()
}

pub fn view_with_tabs<'a>(
    state: &'a HistoryState,
    log_tabs: &'a [crate::state::LogTab],
    active_tab: usize,
    local_branches: &'a [git_core::branch::Branch],
    remote_branches: &'a [git_core::branch::Branch],
    dashboard_visible: bool,
    i18n: &'a I18n,
) -> Element<'a, HistoryMessage> {
    // Build inline tab bar to avoid lifetime issues with TabDescriptor references
    let mut tab_row = Row::new().spacing(0).align_y(Alignment::End);

    for (i, tab) in log_tabs.iter().enumerate() {
        let is_active = i == active_tab;
        let label_color = if is_active {
            theme::darcula::TEXT_PRIMARY
        } else {
            theme::darcula::TEXT_SECONDARY
        };

        let mut tab_content = Row::new()
            .spacing(4)
            .align_y(Alignment::Center)
            .push(Text::new(tab.label.as_str()).size(12).color(label_color));

        if tab.is_closable {
            tab_content = tab_content.push(
                Button::new(Text::new("×").size(10).color(theme::darcula::TEXT_DISABLED))
                    .style(theme::button_style(theme::ButtonTone::Ghost))
                    .padding(0)
                    .on_press(HistoryMessage::CloseLogTab(i)),
            );
        }

        let _tab_bg = if is_active {
            theme::darcula::BG_RAISED
        } else {
            theme::darcula::BG_SOFT
        };

        let tab_button = Button::new(Container::new(tab_content).padding([6, 12]))
            .style(theme::button_style(theme::ButtonTone::Ghost))
            .padding(0)
            .on_press(HistoryMessage::SelectLogTab(i));

        tab_row = tab_row.push(tab_button);
        tab_row = tab_row.push(Space::new().width(Length::Fixed(1.0)));
    }

    // Add "+" button
    tab_row = tab_row.push(
        Button::new(Text::new("+").size(12).color(theme::darcula::TEXT_DISABLED))
            .style(theme::button_style(theme::ButtonTone::Ghost))
            .padding([6, 10])
            .on_press(HistoryMessage::NewLogTab),
    );

    let main_content = view(state, i18n);

    // Build branches dashboard sidebar
    let content_area: Element<'a, HistoryMessage> = if dashboard_visible {
        let dashboard = build_branches_dashboard(local_branches, remote_branches, i18n);
        Row::new()
            .spacing(0)
            .height(Length::Fill)
            .push(
                Container::new(dashboard)
                    .width(Length::FillPortion(1))
                    .height(Length::Fill)
                    .style(theme::panel_style(Surface::Panel)),
            )
            .push(
                Container::new(main_content)
                    .width(Length::FillPortion(4))
                    .height(Length::Fill),
            )
            .into()
    } else {
        main_content
    };

    // Toggle dashboard button in tab bar
    let dashboard_toggle = Button::new(
        Text::new(if dashboard_visible { "◀" } else { "▶" })
            .size(10)
            .color(theme::darcula::TEXT_SECONDARY),
    )
    .style(theme::button_style(theme::ButtonTone::Ghost))
    .padding([6, 6])
    .on_press(HistoryMessage::ToggleBranchesDashboard);

    let full_tab_row = Row::new()
        .spacing(0)
        .align_y(Alignment::Center)
        .push(dashboard_toggle)
        .push(tab_row);

    Column::new()
        .spacing(0)
        .push(Container::new(full_tab_row).padding([0, 4]))
        .push(content_area)
        .into()
}

/// Build the branches dashboard sidebar for the Log tab
fn build_branches_dashboard<'a>(
    local_branches: &'a [git_core::branch::Branch],
    remote_branches: &'a [git_core::branch::Branch],
    i18n: &'a I18n,
) -> Element<'a, HistoryMessage> {
    let header = Container::new(
        Row::new()
            .spacing(theme::spacing::XS)
            .align_y(Alignment::Center)
            .push(
                Text::new(i18n.branches_label)
                    .size(11)
                    .color(theme::darcula::TEXT_SECONDARY),
            ),
    )
    .padding([6, 8]);

    let mut tree = Column::new().spacing(0);

    // Local branches group
    tree = tree.push(
        Container::new(
            Row::new()
                .spacing(4)
                .align_y(Alignment::Center)
                .push(Text::new("▼").size(9).color(theme::darcula::TEXT_DISABLED))
                .push(
                    Text::new(i18n.local_branches)
                        .size(10)
                        .color(theme::darcula::TEXT_SECONDARY),
                )
                .push(
                    Text::new(format!("({})", local_branches.len()))
                        .size(9)
                        .color(theme::darcula::TEXT_DISABLED),
                ),
        )
        .padding([3, 4]),
    );

    for branch in local_branches {
        let name = branch.name.clone();
        let display = branch.leaf_name().to_string();
        let icon = if branch.is_head { "● " } else { "  " };
        let label_color = if branch.is_head {
            theme::darcula::ACCENT
        } else {
            theme::darcula::TEXT_PRIMARY
        };

        tree = tree.push(
            Button::new(
                Row::new()
                    .spacing(4)
                    .align_y(Alignment::Center)
                    .push(Space::new().width(Length::Fixed(12.0)))
                    .push(Text::new(icon).size(10).color(theme::darcula::ACCENT))
                    .push(Text::new(display).size(11).color(label_color)),
            )
            .style(theme::button_style(theme::ButtonTone::Ghost))
            .padding([2, 4])
            .width(Length::Fill)
            .on_press(HistoryMessage::DashboardSelectBranch(name)),
        );
    }

    // Remote branches group
    tree = tree.push(Space::new().height(Length::Fixed(4.0)));
    tree = tree.push(
        Container::new(
            Row::new()
                .spacing(4)
                .align_y(Alignment::Center)
                .push(Text::new("▶").size(9).color(theme::darcula::TEXT_DISABLED))
                .push(
                    Text::new(i18n.remote_branches)
                        .size(10)
                        .color(theme::darcula::TEXT_SECONDARY),
                )
                .push(
                    Text::new(format!("({})", remote_branches.len()))
                        .size(9)
                        .color(theme::darcula::TEXT_DISABLED),
                ),
        )
        .padding([3, 4]),
    );

    // Show first 20 remote branches (collapsed by default, showing just the header)
    // Full expansion requires toggle state — for now show compact list

    Column::new()
        .spacing(0)
        .push(header)
        .push(iced::widget::rule::horizontal(1))
        .push(
            Container::new(scrollable::styled(tree).height(Length::Fill))
                .padding([4, 4])
                .height(Length::Fill),
        )
        .into()
}

pub fn view<'a>(state: &'a HistoryState, i18n: &'a I18n) -> Element<'a, HistoryMessage> {
    let status_panel = if state.is_loading {
        Some(build_status_panel::<HistoryMessage>(
            i18n.loading_label,
            i18n.loading_history_detail,
            BadgeTone::Neutral,
        ))
    } else if state.is_searching {
        Some(build_status_panel::<HistoryMessage>(
            i18n.searching_label,
            i18n.searching_detail,
            BadgeTone::Neutral,
        ))
    } else if let Some(error) = state.error.as_ref() {
        Some(build_status_panel::<HistoryMessage>(
            i18n.failed_label,
            error,
            BadgeTone::Danger,
        ))
    } else if state.filtered_entries.is_empty() && !state.search_query.trim().is_empty() {
        Some(build_status_panel::<HistoryMessage>(
            i18n.no_match,
            i18n.no_match_fmt.replace("{}", state.search_query.trim()),
            BadgeTone::Warning,
        ))
    } else {
        None
    };

    if state.entries.is_empty() && !state.is_loading && state.error.is_none() {
        return Container::new(
            Column::new()
                .spacing(theme::spacing::XS)
                .align_x(Alignment::Center)
                .push(
                    Text::new(i18n.no_commit_history)
                        .size(13)
                        .color(theme::darcula::TEXT_SECONDARY),
                )
                .push(
                    Text::new(i18n.no_commit_history_hint)
                        .size(10)
                        .color(theme::darcula::TEXT_DISABLED),
                )
                .push(Space::new().height(Length::Fixed(theme::spacing::SM)))
                .push(button::ghost(i18n.refresh, Some(HistoryMessage::Refresh))),
        )
        .width(Length::Fill)
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center)
        .height(Length::Fill)
        .style(theme::panel_style(Surface::Panel))
        .into();
    }

    let detail_panel: Element<'_, HistoryMessage> =
        if let Some(info) = state.selected_commit_info.as_ref() {
            build_commit_detail(state, info, i18n)
        } else if !state.search_query.trim().is_empty() && state.filtered_entries.is_empty() {
            widgets::panel_empty_state_compact(
                i18n.no_match,
                i18n.no_match_fmt.replace("{}", state.search_query.trim()),
            )
        } else {
            widgets::panel_empty_state_compact(
                i18n.no_commit_selected,
                i18n.select_commit_hint,
            )
        };

    let can_search = !state.is_searching && !state.search_query.trim().is_empty();
    let can_clear = !state.is_searching
        && (!state.search_query.trim().is_empty()
            || state.filtered_entries.len() != state.entries.len());

    let search_actions: Element<'_, HistoryMessage> = if state.is_searching {
        widgets::inline_loading(i18n.search)
    } else {
        Row::new()
            .spacing(theme::spacing::XS)
            .align_y(Alignment::Center)
            .push(button::secondary(
                i18n.search,
                can_search.then_some(HistoryMessage::Search),
            ))
            .push(button::ghost(
                i18n.clear,
                can_clear.then_some(HistoryMessage::ClearSearch),
            ))
            .into()
    };

    let toolbar = Container::new(
        Row::new()
            .spacing(theme::spacing::XS)
            .align_y(Alignment::Center)
            .push(Text::new(i18n.commit_history).size(12))
            .push_maybe(state.current_branch_name.as_ref().map(|branch| {
                widgets::info_chip::<HistoryMessage>(
                    format!("{} {branch}", i18n.current_branch),
                    BadgeTone::Accent,
                )
            }))
            .push_maybe(state.current_upstream_ref.as_ref().map(|upstream| {
                widgets::info_chip::<HistoryMessage>(format!("{} {upstream}", i18n.upstream_label), BadgeTone::Neutral)
            }))
            .push(Space::new().width(Length::Fill))
            .push(
                text_input::styled(
                    i18n.search_placeholder,
                    &state.search_query,
                    HistoryMessage::SetSearchQuery,
                )
                .width(Length::Fixed(200.0)),
            )
            .push(search_actions)
            .push(button::ghost(i18n.refresh, Some(HistoryMessage::Refresh))),
    )
    .padding(theme::density::SECONDARY_BAR_PADDING)
    .style(theme::panel_style(Surface::Toolbar));

    let list_area = Container::new(
        stack([
            build_history_list(state, i18n),
            build_commit_context_menu_overlay(state, i18n),
        ])
        .width(Length::Fill)
        .height(Length::Fill),
    )
    .width(Length::FillPortion(6))
    .height(Length::Fill);

    let detail_area = Container::new(detail_panel)
        .width(Length::FillPortion(4))
        .height(Length::Fill);

    Container::new(
        Column::new()
            .spacing(0)
            .push(toolbar)
            .push_maybe(status_panel)
            .push(
                Row::new()
                    .spacing(theme::spacing::XS)
                    .height(Length::Fill)
                    .push(list_area)
                    .push(detail_area),
            ),
    )
    .padding([0, 0])
    .width(Length::Fill)
    .height(Length::Fill)
    .style(theme::frame_style(Surface::Editor))
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

    fn entry(id: &str, parents: &[&str]) -> HistoryEntry {
        HistoryEntry {
            id: id.to_string(),
            message: id.to_string(),
            author_name: "tester".to_string(),
            author_email: "tester@example.com".to_string(),
            timestamp: 0,
            parent_ids: parents.iter().map(|parent| parent.to_string()).collect(),
            committer_name: None,
            committer_email: None,
            refs: Vec::new(),
            signature_status: None,
        }
    }

    #[test]
    fn build_history_graph_keeps_linear_history_on_single_lane() {
        let entries = vec![entry("c3", &["c2"]), entry("c2", &["c1"]), entry("c1", &[])];

        let graph = build_history_graph(&entries);

        assert_eq!(graph.lane_count, 1);
        assert_eq!(graph.rows.len(), 3);
        assert!(graph.rows.iter().all(|row| row.node_lane == 0));
        assert_eq!(graph.rows[0].parent_transitions.len(), 1);
        assert_eq!(graph.rows[0].parent_transitions[0].from_lane, 0);
        assert_eq!(graph.rows[0].parent_transitions[0].to_lane, 0);
        assert_eq!(graph.rows[1].parent_transitions.len(), 1);
        assert_eq!(graph.rows[1].parent_transitions[0].from_lane, 0);
        assert_eq!(graph.rows[1].parent_transitions[0].to_lane, 0);
        assert!(graph.rows[2].parent_transitions.is_empty());
    }

    #[test]
    fn build_history_graph_draws_merge_on_multiple_lanes() {
        let entries = vec![
            entry("merge", &["main", "feature"]),
            entry("main", &["base"]),
            entry("feature", &["base"]),
            entry("base", &[]),
        ];

        let graph = build_history_graph(&entries);

        assert_eq!(graph.lane_count, 2);
        assert_eq!(graph.rows[0].node_lane, 0);
        assert_eq!(graph.rows[0].parent_transitions.len(), 2);
        assert_eq!(graph.rows[0].parent_transitions[0].from_lane, 0);
        assert_eq!(graph.rows[0].parent_transitions[0].to_lane, 0);
        assert_eq!(graph.rows[0].parent_transitions[1].from_lane, 0);
        assert_eq!(graph.rows[0].parent_transitions[1].to_lane, 1);

        assert_eq!(graph.rows[1].continuing.len(), 1);
        assert_eq!(graph.rows[1].continuing[0].from_lane, 1);
        assert_eq!(graph.rows[1].continuing[0].to_lane, 1);

        assert_eq!(graph.rows[2].node_lane, 1);
        assert_eq!(graph.rows[2].parent_transitions.len(), 1);
        assert_eq!(graph.rows[2].parent_transitions[0].from_lane, 1);
        assert_eq!(graph.rows[2].parent_transitions[0].to_lane, 0);
    }
}
