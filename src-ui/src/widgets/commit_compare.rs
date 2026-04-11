//! Commit comparison widget.
//!
//! Provides functionality to compare two commits.

use crate::theme::{self, BadgeTone, Surface};
use crate::widgets::{self, button, diff_viewer, scrollable, OptionalPush};
use git_core::diff::{diff_commits, Diff};
use git_core::history::{get_history, HistoryEntry};
use git_core::repository::Repository;
use iced::widget::{Column, Container, PickList, Row, Text};
use iced::{Element, Length};

/// Message types for commit comparison.
#[derive(Debug, Clone)]
pub enum CommitCompareMessage {
    SetLeftCommit(String),
    SetRightCommit(String),
    SwapCommits,
    Compare,
    Refresh,
}

/// State for commit comparison.
#[derive(Debug, Clone)]
pub struct CommitCompareState {
    pub entries: Vec<HistoryEntry>,
    pub left_commit: Option<String>,
    pub right_commit: Option<String>,
    pub diff: Option<Diff>,
    pub is_loading: bool,
    pub error: Option<String>,
}

impl CommitCompareState {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            left_commit: None,
            right_commit: None,
            diff: None,
            is_loading: false,
            error: None,
        }
    }

    pub fn load_entries(&mut self, repo: &Repository) {
        self.is_loading = true;
        match get_history(repo, Some(50)) {
            Ok(entries) => {
                if entries.len() >= 2 {
                    self.right_commit = Some(entries[0].id.clone());
                    self.left_commit = Some(entries[1].id.clone());
                }
                self.entries = entries;
                self.is_loading = false;
            }
            Err(error) => {
                self.error = Some(format!("Failed to load commit history: {error}"));
                self.is_loading = false;
            }
        }
    }

    pub fn set_left_commit(&mut self, commit_id: String) {
        self.left_commit = Some(commit_id);
        self.diff = None;
    }

    pub fn set_right_commit(&mut self, commit_id: String) {
        self.right_commit = Some(commit_id);
        self.diff = None;
    }

    pub fn swap_commits(&mut self) {
        std::mem::swap(&mut self.left_commit, &mut self.right_commit);
        self.diff = None;
    }

    pub fn compare(&mut self, repo: &Repository) {
        let left = match &self.left_commit {
            Some(id) => id.as_str(),
            None => {
                self.error = Some("Please select a left commit".to_string());
                return;
            }
        };

        let right = match &self.right_commit {
            Some(id) => id.as_str(),
            None => {
                self.error = Some("Please select a right commit".to_string());
                return;
            }
        };

        self.is_loading = true;
        self.error = None;

        match diff_commits(repo, left, right) {
            Ok(diff) => {
                self.diff = Some(diff);
                self.is_loading = false;
            }
            Err(error) => {
                self.error = Some(format!("Comparison failed: {error}"));
                self.is_loading = false;
            }
        }
    }
}

impl Default for CommitCompareState {
    fn default() -> Self {
        Self::new()
    }
}

fn build_commit_selector<'a>(
    entries: &'a [HistoryEntry],
    selected: Option<&'a str>,
    label: &'a str,
    on_select: impl Fn(String) -> CommitCompareMessage + 'a,
) -> Element<'a, CommitCompareMessage> {
    let options: Vec<&str> = entries.iter().map(|entry| entry.id.as_str()).collect();
    let selected_str = selected.unwrap_or("");

    Container::new(
        Column::new()
            .spacing(theme::spacing::XS)
            .push(
                Text::new(label)
                    .size(12)
                    .color(theme::darcula::TEXT_SECONDARY),
            )
            .push(PickList::new(options, Some(selected_str), move |value| {
                on_select(value.to_string())
            })),
    )
    .padding(12)
    .style(theme::panel_style(Surface::Panel))
    .width(Length::FillPortion(1))
    .into()
}

pub fn view(state: &CommitCompareState) -> Element<'_, CommitCompareMessage> {
    if state.entries.is_empty() && !state.is_loading && state.error.is_none() {
        return Container::new(
            Column::new()
                .spacing(theme::spacing::MD)
                .push(widgets::section_header(
                    "Compare",
                    "Commit Comparison",
                    "Select left and right commits, then view the unified diff.",
                ))
                .push(widgets::panel_empty_state(
                    "Result",
                    "No commit history to compare",
                    "Create commits first, or refresh the history list.",
                    Some(button::ghost("Refresh", Some(CommitCompareMessage::Refresh)).into()),
                )),
        )
        .padding(20)
        .style(theme::panel_style(Surface::Panel))
        .into();
    }

    let status_panel = if state.is_loading {
        Some(build_status_panel::<CommitCompareMessage>(
            "Loading",
            "Comparing the two commits.",
            BadgeTone::Neutral,
        ))
    } else if let Some(error) = state.error.as_ref() {
        Some(build_status_panel::<CommitCompareMessage>(
            "Failed",
            error,
            BadgeTone::Danger,
        ))
    } else if let Some(diff) = state.diff.as_ref() {
        Some(build_status_panel::<CommitCompareMessage>(
            "Comparison complete",
            format!(
                "{} files affected, +{} / -{} lines.",
                diff.files.len(),
                diff.total_additions,
                diff.total_deletions
            ),
            BadgeTone::Success,
        ))
    } else {
        Some(build_status_panel::<CommitCompareMessage>(
            "Pending",
            "Select commits from both sides to compare.",
            BadgeTone::Accent,
        ))
    };

    let diff_panel: Element<'_, CommitCompareMessage> = if let Some(diff) = state.diff.as_ref() {
        Container::new(diff_viewer::DiffViewer::new(diff).view())
            .padding(14)
            .style(theme::panel_style(Surface::Panel))
            .into()
    } else {
        widgets::panel_empty_state(
            "Result",
            if state.left_commit.is_none() || state.right_commit.is_none() {
                "Select left and right commits first"
            } else {
                "No comparison results yet"
            },
            if state.left_commit.is_none() || state.right_commit.is_none() {
                "Select a commit on each side, then click Compare."
            } else {
                "Re-compare or switch commits to update."
            },
            Some(button::primary("Compare", Some(CommitCompareMessage::Compare)).into()),
        )
    };

    Container::new(
        Column::new()
            .spacing(theme::spacing::MD)
            .push(widgets::section_header(
                "Compare",
                "Commit Comparison",
                "Select left and right commits, then view the unified diff.",
            ))
            .push(
                scrollable::styled_horizontal(
                    Row::new()
                        .spacing(theme::spacing::XS)
                        .push(widgets::info_chip::<CommitCompareMessage>(
                            format!("Candidates {}", state.entries.len()),
                            BadgeTone::Neutral,
                        ))
                        .push_maybe(state.left_commit.as_ref().map(|left| {
                            widgets::info_chip::<CommitCompareMessage>(
                                format!("Left {}", &left[..left.len().min(8)]),
                                BadgeTone::Accent,
                            )
                        }))
                        .push_maybe(state.right_commit.as_ref().map(|right| {
                            widgets::info_chip::<CommitCompareMessage>(
                                format!("Right {}", &right[..right.len().min(8)]),
                                BadgeTone::Warning,
                            )
                        })),
                )
                .width(Length::Fill),
            )
            .push_maybe(status_panel)
            .push(
                Column::new()
                    .spacing(theme::spacing::SM)
                    .push(build_commit_selector(
                        &state.entries,
                        state.left_commit.as_deref(),
                        "Left Commit (Old)",
                        CommitCompareMessage::SetLeftCommit,
                    ))
                    .push(
                        scrollable::styled_horizontal(
                            Row::new()
                                .spacing(theme::spacing::XS)
                                .push(button::ghost(
                                    "Swap",
                                    Some(CommitCompareMessage::SwapCommits),
                                ))
                                .push(button::primary("Compare", Some(CommitCompareMessage::Compare)))
                                .push(button::ghost("Refresh", Some(CommitCompareMessage::Refresh))),
                        )
                        .width(Length::Fill),
                    )
                    .push(build_commit_selector(
                        &state.entries,
                        state.right_commit.as_deref(),
                        "Right Commit (New)",
                        CommitCompareMessage::SetRightCommit,
                    )),
            )
            .push(diff_panel),
    )
    .padding(20)
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
