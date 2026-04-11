//! Conflict resolver widget.
//!
//! Provides a PhpStorm-style three-column merge editor for conflicted files.

use crate::theme::{self, BadgeTone, Surface};
use crate::widgets::syntax_highlighting::{
    CodeLineHighlighter, CodeSyntaxHighlighter, HighlightedSegment,
};
use crate::widgets::{self, button, scrollable, OptionalPush};
use git_core::diff::{ConflictHunk, ConflictHunkType, ConflictLineType, ThreeWayDiff};
use iced::widget::{container, Button, Column, Container, Row, Space, Text};
use iced::{Alignment, Background, Border, Color, Element, Length, Theme};

/// Message types for conflict resolver.
#[derive(Debug, Clone)]
pub enum ConflictResolverMessage {
    BackToList,
    Refresh,
    Resolve,
    SelectHunk(usize),
    SelectPrevHunk,
    SelectNextHunk,
    ChooseOursForHunk(usize),
    ChooseTheirsForHunk(usize),
    ChooseBaseForHunk(usize),
    AcceptOursAll,
    AcceptTheirsAll,
    AutoMerge,
}

/// Resolution options.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolutionOption {
    Ours,
    Theirs,
    Base,
}

/// Per-hunk resolution state.
#[derive(Debug, Clone, Default)]
pub struct HunkResolution {
    pub hunk_index: usize,
    pub resolution: Option<ResolutionOption>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PaneTone {
    Neutral,
    Ours,
    Theirs,
    Base,
    Marker,
    Empty,
}

#[derive(Debug, Clone)]
struct PaneLine {
    number: Option<usize>,
    text: String,
    tone: PaneTone,
}

/// A widget for displaying and resolving merge conflicts.
#[derive(Debug, Clone)]
pub struct ConflictResolver {
    pub diff: ThreeWayDiff,
    pub selected_hunk: Option<usize>,
    pub hunk_resolutions: Vec<HunkResolution>,
    pub preview_content: Option<String>,
    pub is_auto_merged: bool,
}

impl ConflictResolver {
    pub fn new(diff: ThreeWayDiff) -> Self {
        let hunk_count = diff.hunks.len();
        Self {
            diff,
            selected_hunk: (hunk_count > 0).then_some(0),
            hunk_resolutions: (0..hunk_count)
                .map(|index| HunkResolution {
                    hunk_index: index,
                    resolution: None,
                })
                .collect(),
            preview_content: None,
            is_auto_merged: false,
        }
    }

    pub fn select_hunk(&mut self, index: usize) {
        if index < self.diff.hunks.len() {
            self.selected_hunk = Some(index);
        }
    }

    pub fn select_previous_hunk(&mut self) {
        let Some(current) = self.selected_hunk else {
            if !self.diff.hunks.is_empty() {
                self.selected_hunk = Some(0);
            }
            return;
        };

        if current > 0 {
            self.selected_hunk = Some(current - 1);
        }
    }

    pub fn select_next_hunk(&mut self) {
        let Some(current) = self.selected_hunk else {
            if !self.diff.hunks.is_empty() {
                self.selected_hunk = Some(0);
            }
            return;
        };

        if current + 1 < self.diff.hunks.len() {
            self.selected_hunk = Some(current + 1);
        }
    }

    pub fn resolve_hunk(&mut self, index: usize, option: ResolutionOption) {
        if index < self.hunk_resolutions.len() {
            self.hunk_resolutions[index].resolution = Some(option);
            self.selected_hunk = Some(index);
            self.preview_content = Some(self.get_preview_content());
            self.is_auto_merged = false;
        }
    }

    pub fn accept_all(&mut self, option: ResolutionOption) {
        for resolution in &mut self.hunk_resolutions {
            resolution.resolution = Some(option);
        }
        self.preview_content = Some(self.get_preview_content());
        self.is_auto_merged = false;
    }

    pub fn auto_merge(&mut self) {
        for (index, hunk) in self.diff.hunks.iter().enumerate() {
            self.hunk_resolutions[index].resolution = match classify_hunk(hunk) {
                ConflictHunkType::OursOnly => Some(ResolutionOption::Ours),
                ConflictHunkType::TheirsOnly => Some(ResolutionOption::Theirs),
                ConflictHunkType::Unchanged => Some(ResolutionOption::Base),
                ConflictHunkType::Modified => None,
            };
        }

        self.selected_hunk = self.first_unresolved_hunk().or(self.selected_hunk);
        self.is_auto_merged = true;
        self.preview_content = Some(self.get_preview_content());
    }

    pub fn get_preview_content(&self) -> String {
        let base_lines: Vec<&str> = self.diff.base_content.lines().collect();
        let ours_lines: Vec<&str> = self.diff.ours_content.lines().collect();
        let theirs_lines: Vec<&str> = self.diff.theirs_content.lines().collect();
        let max_lines = base_lines
            .len()
            .max(ours_lines.len())
            .max(theirs_lines.len());

        let mut output = Vec::new();
        let mut cursor = 0usize;

        for (index, hunk) in self.diff.hunks.iter().enumerate() {
            let start = hunk.base_start.min(hunk.ours_start).min(hunk.theirs_start) as usize;

            while cursor < start && cursor < max_lines {
                if let Some(line) = default_line_at(&base_lines, &ours_lines, &theirs_lines, cursor)
                {
                    output.push(line);
                }
                cursor += 1;
            }

            output.extend(self.get_hunk_lines(index));
            cursor = cursor.max(start + hunk.lines.len());
        }

        while cursor < max_lines {
            if let Some(line) = default_line_at(&base_lines, &ours_lines, &theirs_lines, cursor) {
                output.push(line);
            }
            cursor += 1;
        }

        let mut result = output.join("\n");
        if !result.is_empty()
            && (self.diff.base_content.ends_with('\n')
                || self.diff.ours_content.ends_with('\n')
                || self.diff.theirs_content.ends_with('\n'))
        {
            result.push('\n');
        }
        result
    }

    pub fn view(&self) -> Element<'_, ConflictResolverMessage> {
        let resolved_count = self.resolved_count();
        let total_count = self.diff.hunks.len();
        let unresolved_count = total_count.saturating_sub(resolved_count);
        let selected_hunk = self
            .selected_hunk
            .and_then(|index| self.diff.hunks.get(index));
        let selected_index = self.selected_hunk;

        let next_step = if total_count == 0 {
            "No remaining conflict hunks. Return to the list to continue.".to_string()
        } else if unresolved_count == 0 {
            "All conflict hunks resolved. Review the result, then click Apply.".to_string()
        } else if let Some(index) = selected_index {
            format!(
                "Focused on conflict hunk {}. Accept left, right, or base version.",
                index + 1
            )
        } else {
            "Select a conflict hunk below first.".to_string()
        };

        let instruction_bar = build_inline_status(
            "Next Step",
            next_step,
            if unresolved_count == 0 {
                BadgeTone::Success
            } else {
                BadgeTone::Accent
            },
        );

        let toolbar = build_merge_toolbar(selected_index, total_count);

        let stats = Row::new()
            .spacing(theme::spacing::XS)
            .push(widgets::info_chip::<ConflictResolverMessage>(
                format!("File {}", self.diff.path),
                BadgeTone::Accent,
            ))
            .push(widgets::info_chip::<ConflictResolverMessage>(
                format!("Hunks {total_count}"),
                BadgeTone::Neutral,
            ))
            .push(widgets::info_chip::<ConflictResolverMessage>(
                format!("Resolved {resolved_count}/{total_count}"),
                if unresolved_count == 0 {
                    BadgeTone::Success
                } else {
                    BadgeTone::Warning
                },
            ))
            .push_maybe(self.is_auto_merged.then(|| {
                widgets::info_chip::<ConflictResolverMessage>("Auto merge executed", BadgeTone::Neutral)
            }))
            .push_maybe(selected_index.map(|index| {
                widgets::info_chip::<ConflictResolverMessage>(
                    format!("Hunk {}", index + 1),
                    BadgeTone::Accent,
                )
            }));

        let hunk_navigator = build_hunk_navigator(self);

        let headers = Row::new()
            .spacing(theme::spacing::SM)
            .push(build_column_header(
                "Yours",
                "From current branch",
                "<< Accept Left",
                EditorColumnKind::Ours,
                BadgeTone::Accent,
            ))
            .push(build_column_header(
                "Merge Result",
                "Final result",
                "<> Current Result",
                EditorColumnKind::Result,
                BadgeTone::Success,
            ))
            .push(build_column_header(
                "Theirs",
                "From incoming branch",
                ">> Accept Right",
                EditorColumnKind::Theirs,
                BadgeTone::Danger,
            ));

        let editor_body = self.build_merge_body();

        let footer = Row::new()
            .spacing(theme::spacing::XS)
            .align_y(Alignment::Center)
            .push(Text::new(format!("{} hunks to review", unresolved_count)).size(11))
            .push(Space::new().width(Length::Fill))
            .push(button::ghost(
                "Cancel",
                Some(ConflictResolverMessage::BackToList),
            ))
            .push(button::primary(
                "Apply",
                (total_count > 0 && unresolved_count == 0)
                    .then_some(ConflictResolverMessage::Resolve),
            ));

        let merge_hint = selected_hunk
            .map(|hunk| build_selected_hunk_hint(selected_index.unwrap_or(0), hunk, self))
            .unwrap_or_else(|| {
                build_inline_status(
                    "Status",
                    "No editable conflict hunks.".to_string(),
                    BadgeTone::Neutral,
                )
            });

        Container::new(
            Column::new()
                .spacing(theme::spacing::MD)
                .push(widgets::section_header(
                    "Conflicts",
                    "Three-Way Merge",
                    "Left: current, Right: incoming, Center: final result.",
                ))
                .push(stats)
                .push(instruction_bar)
                .push(merge_hint)
                .push(toolbar)
                .push(hunk_navigator)
                .push(headers)
                .push(editor_body)
                .push(footer),
        )
        .padding([16, 18])
        .style(theme::panel_style(Surface::Panel))
        .into()
    }

    fn build_merge_body(&self) -> Element<'static, ConflictResolverMessage> {
        let syntax = CodeSyntaxHighlighter::for_path(&self.diff.path);
        let mut ours_highlighter = syntax.start();
        let mut result_highlighter = syntax.start();
        let mut theirs_highlighter = syntax.start();

        let content = self.diff.hunks.iter().enumerate().fold(
            Column::new().spacing(theme::spacing::SM),
            |column, (index, hunk)| {
                column.push(self.build_hunk_row(
                    index,
                    hunk,
                    &mut ours_highlighter,
                    &mut result_highlighter,
                    &mut theirs_highlighter,
                ))
            },
        );

        Container::new(scrollable::styled_both(content).height(Length::Fill))
            .height(Length::Fill)
            .style(theme::panel_style(Surface::Editor))
            .into()
    }

    fn build_hunk_row(
        &self,
        index: usize,
        hunk: &ConflictHunk,
        ours_highlighter: &mut CodeLineHighlighter,
        result_highlighter: &mut CodeLineHighlighter,
        theirs_highlighter: &mut CodeLineHighlighter,
    ) -> Element<'static, ConflictResolverMessage> {
        let is_selected = self.selected_hunk == Some(index);
        let resolution = self.effective_resolution(index);
        let resolution_label = match resolution {
            Some(ResolutionOption::Ours) => "Accepted Left",
            Some(ResolutionOption::Theirs) => "Accepted Right",
            Some(ResolutionOption::Base) => "Accepted Base",
            None => "Unresolved",
        };
        let conflict_type = classify_hunk(hunk);
        let resolution_tone = match resolution {
            Some(ResolutionOption::Ours) => BadgeTone::Accent,
            Some(ResolutionOption::Theirs) => BadgeTone::Danger,
            Some(ResolutionOption::Base) => BadgeTone::Neutral,
            None => BadgeTone::Warning,
        };

        let header = Row::new()
            .push(
                Container::new(Space::new().width(Length::Fixed(3.0)))
                    .width(Length::Fixed(3.0))
                    .height(Length::Fill)
                    .style(selection_strip_style(is_selected, resolution)),
            )
            .push(
                Container::new(
                    Column::new()
                        .spacing(theme::spacing::XS)
                        .push(
                            Row::new()
                                .spacing(theme::spacing::XS)
                                .align_y(Alignment::Center)
                                .push(widgets::info_chip::<ConflictResolverMessage>(
                                    format!("Conflict {}", index + 1),
                                    if is_selected {
                                        BadgeTone::Accent
                                    } else {
                                        BadgeTone::Neutral
                                    },
                                ))
                                .push(widgets::info_chip::<ConflictResolverMessage>(
                                    match conflict_type {
                                        ConflictHunkType::Modified => "Needs manual resolution",
                                        ConflictHunkType::OursOnly => "Left side can be accepted",
                                        ConflictHunkType::TheirsOnly => "Right side can be accepted",
                                        ConflictHunkType::Unchanged => "Base version unchanged",
                                    },
                                    match conflict_type {
                                        ConflictHunkType::Modified => BadgeTone::Warning,
                                        ConflictHunkType::OursOnly => BadgeTone::Accent,
                                        ConflictHunkType::TheirsOnly => BadgeTone::Danger,
                                        ConflictHunkType::Unchanged => BadgeTone::Neutral,
                                    },
                                ))
                                .push(widgets::info_chip::<ConflictResolverMessage>(
                                    resolution_label,
                                    resolution_tone,
                                ))
                                .push(
                                    Text::new(format!(
                                        "{} lines, base {}, ours {}, theirs {}",
                                        hunk.lines.len(),
                                        hunk.base_start + 1,
                                        hunk.ours_start + 1,
                                        hunk.theirs_start + 1
                                    ))
                                    .size(11)
                                    .color(theme::darcula::TEXT_SECONDARY),
                                )
                                .push(Space::new().width(Length::Fill))
                                .push(button::compact_ghost(
                                    "Locate",
                                    Some(ConflictResolverMessage::SelectHunk(index)),
                                )),
                        )
                        .push(
                            Row::new()
                                .spacing(theme::spacing::XS)
                                .push(build_header_action(
                                    "<<",
                                    "Accept Left",
                                    ButtonFlavor::Ours,
                                    Some(ConflictResolverMessage::ChooseOursForHunk(index)),
                                ))
                                .push(build_header_action(
                                    "=",
                                    "Keep Base",
                                    ButtonFlavor::Base,
                                    Some(ConflictResolverMessage::ChooseBaseForHunk(index)),
                                ))
                                .push(build_header_action(
                                    ">>",
                                    "Accept Right",
                                    ButtonFlavor::Theirs,
                                    Some(ConflictResolverMessage::ChooseTheirsForHunk(index)),
                                )),
                        ),
                )
                .padding([10, 12])
                .width(Length::Fill)
                .style(hunk_header_style(is_selected, resolution)),
            );

        let ours_lines = build_side_lines(hunk, ConflictSide::Ours);
        let result_lines = build_result_lines(hunk, resolution);
        let theirs_lines = build_side_lines(hunk, ConflictSide::Theirs);

        let panels = Row::new()
            .spacing(theme::spacing::SM)
            .push(build_editor_column(
                ours_lines,
                ours_highlighter,
                EditorColumnKind::Ours,
                is_selected,
            ))
            .push(build_editor_column(
                result_lines,
                result_highlighter,
                EditorColumnKind::Result,
                is_selected,
            ))
            .push(build_editor_column(
                theirs_lines,
                theirs_highlighter,
                EditorColumnKind::Theirs,
                is_selected,
            ));

        Container::new(
            Column::new()
                .spacing(theme::spacing::XS)
                .push(header)
                .push(panels),
        )
        .padding([12, 12])
        .style(hunk_card_style(is_selected))
        .into()
    }

    fn first_unresolved_hunk(&self) -> Option<usize> {
        self.diff
            .hunks
            .iter()
            .enumerate()
            .find_map(|(index, _)| (!self.is_hunk_resolved(index)).then_some(index))
    }

    fn resolved_count(&self) -> usize {
        self.diff
            .hunks
            .iter()
            .enumerate()
            .filter(|(index, _)| self.is_hunk_resolved(*index))
            .count()
    }

    fn is_hunk_resolved(&self, index: usize) -> bool {
        self.effective_resolution(index).is_some()
    }

    fn effective_resolution(&self, index: usize) -> Option<ResolutionOption> {
        self.hunk_resolutions
            .get(index)
            .and_then(|resolution| resolution.resolution)
            .or_else(|| {
                self.diff
                    .hunks
                    .get(index)
                    .and_then(|hunk| match classify_hunk(hunk) {
                        ConflictHunkType::OursOnly => Some(ResolutionOption::Ours),
                        ConflictHunkType::TheirsOnly => Some(ResolutionOption::Theirs),
                        ConflictHunkType::Unchanged => Some(ResolutionOption::Base),
                        ConflictHunkType::Modified => None,
                    })
            })
    }

    fn get_hunk_lines(&self, index: usize) -> Vec<String> {
        let Some(hunk) = self.diff.hunks.get(index) else {
            return Vec::new();
        };

        if let Some(resolution) = self.effective_resolution(index) {
            return self.apply_resolution(hunk, resolution);
        }

        self.render_unresolved_hunk(hunk)
    }

    fn apply_resolution(&self, hunk: &ConflictHunk, option: ResolutionOption) -> Vec<String> {
        hunk.lines
            .iter()
            .filter_map(|line| select_line_for_resolution(line, option))
            .collect()
    }

    fn render_unresolved_hunk(&self, hunk: &ConflictHunk) -> Vec<String> {
        let ours_lines: Vec<String> = hunk
            .lines
            .iter()
            .filter_map(|line| line.ours_line.clone())
            .collect();
        let theirs_lines: Vec<String> = hunk
            .lines
            .iter()
            .filter_map(|line| line.theirs_line.clone())
            .collect();

        // IDEA-style: no raw markers, just show the content
        let mut result = Vec::with_capacity(ours_lines.len() + theirs_lines.len());
        result.extend(ours_lines);
        result.extend(theirs_lines);
        result
    }
}

#[derive(Debug, Clone, Copy)]
enum ConflictSide {
    Ours,
    Theirs,
}

#[derive(Debug, Clone, Copy)]
enum EditorColumnKind {
    Ours,
    Result,
    Theirs,
}

fn build_side_lines(hunk: &ConflictHunk, side: ConflictSide) -> Vec<PaneLine> {
    let mut line_number = match side {
        ConflictSide::Ours => hunk.ours_start as usize + 1,
        ConflictSide::Theirs => hunk.theirs_start as usize + 1,
    };

    hunk.lines
        .iter()
        .map(|line| {
            let content = match side {
                ConflictSide::Ours => line.ours_line.as_deref(),
                ConflictSide::Theirs => line.theirs_line.as_deref(),
            };

            let number = content.map(|_| {
                let current = line_number;
                line_number += 1;
                current
            });

            let tone = match (side, line.line_type.clone()) {
                (ConflictSide::Ours, ConflictLineType::OursOnly)
                | (ConflictSide::Ours, ConflictLineType::Modified) => PaneTone::Ours,
                (ConflictSide::Theirs, ConflictLineType::TheirsOnly)
                | (ConflictSide::Theirs, ConflictLineType::Modified) => PaneTone::Theirs,
                (_, ConflictLineType::Unchanged) => PaneTone::Neutral,
                (_, ConflictLineType::Empty) => PaneTone::Empty,
                (_, ConflictLineType::ConflictMarker) => PaneTone::Marker,
                _ => PaneTone::Neutral,
            };

            PaneLine {
                number,
                text: content.unwrap_or("").to_string(),
                tone: if content.is_none() {
                    PaneTone::Empty
                } else {
                    tone
                },
            }
        })
        .collect()
}

fn build_result_lines(hunk: &ConflictHunk, resolution: Option<ResolutionOption>) -> Vec<PaneLine> {
    let mut line_number = hunk.base_start.min(hunk.ours_start).min(hunk.theirs_start) as usize + 1;

    match resolution {
        Some(choice) => hunk
            .lines
            .iter()
            .filter_map(|line| select_line_for_resolution(line, choice))
            .map(|text| {
                let current = line_number;
                line_number += 1;
                PaneLine {
                    number: Some(current),
                    text,
                    tone: match choice {
                        ResolutionOption::Ours => PaneTone::Ours,
                        ResolutionOption::Theirs => PaneTone::Theirs,
                        ResolutionOption::Base => PaneTone::Base,
                    },
                }
            })
            .collect(),
        None => {
            let ours_lines: Vec<String> = hunk
                .lines
                .iter()
                .filter_map(|line| line.ours_line.clone())
                .collect();
            let theirs_lines: Vec<String> = hunk
                .lines
                .iter()
                .filter_map(|line| line.theirs_line.clone())
                .collect();
            let mut result = Vec::new();
            // IDEA-style: show "Unresolved Conflict" instead of raw markers when unresolved
            result.push(PaneLine {
                number: None,
                text: "── Unresolved Conflict ──".to_string(),
                tone: PaneTone::Marker,
            });
            for text in ours_lines {
                let current = line_number;
                line_number += 1;
                result.push(PaneLine {
                    number: Some(current),
                    text,
                    tone: PaneTone::Ours,
                });
            }
            for text in theirs_lines {
                let current = line_number;
                line_number += 1;
                result.push(PaneLine {
                    number: Some(current),
                    text,
                    tone: PaneTone::Theirs,
                });
            }
            result
        }
    }
}

fn build_editor_column(
    lines: Vec<PaneLine>,
    highlighter: &mut CodeLineHighlighter,
    kind: EditorColumnKind,
    selected: bool,
) -> Element<'static, ConflictResolverMessage> {
    let body = lines
        .into_iter()
        .fold(Column::new().spacing(1), |column, line| {
            let segments = highlighter.highlight_segments(&line.text);
            let code = HighlightedSegment::render::<ConflictResolverMessage>(&segments);
            let line_number = line
                .number
                .map(|number| number.to_string())
                .unwrap_or_else(|| "".to_string());

            column.push(
                Container::new(
                    Row::new()
                        .spacing(theme::spacing::XS)
                        .align_y(Alignment::Start)
                        .push(
                            Text::new(line_number)
                                .size(10)
                                .width(Length::Fixed(34.0))
                                .color(theme::darcula::TEXT_SECONDARY),
                        )
                        .push(Container::new(code).width(Length::Fill)),
                )
                .padding([1, 4])
                .style(conflict_line_style(line.tone, selected)),
            )
        });

    Container::new(body)
        .width(Length::FillPortion(1))
        .padding([10, 10])
        .style(editor_column_style(kind, selected))
        .into()
}

fn build_column_header(
    title: impl Into<String>,
    detail: impl Into<String>,
    action_hint: impl Into<String>,
    kind: EditorColumnKind,
    tone: BadgeTone,
) -> Element<'static, ConflictResolverMessage> {
    Container::new(
        Row::new()
            .spacing(theme::spacing::XS)
            .align_y(Alignment::Center)
            .push(
                Container::new(
                    Text::new(match kind {
                        // IDEA-style: more descriptive column labels
                        EditorColumnKind::Ours => "Yours",
                        EditorColumnKind::Result => "Result",
                        EditorColumnKind::Theirs => "Theirs",
                    })
                    .size(11)
                    .color(theme::darcula::TEXT_PRIMARY),
                )
                .width(Length::Fixed(26.0))
                .height(Length::Fixed(20.0))
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .style(column_icon_style(kind)),
            )
            .push(
                Column::new()
                    .spacing(2)
                    .width(Length::Fill)
                    .push(Text::new(title.into()).size(13))
                    .push(
                        Row::new()
                            .spacing(theme::spacing::XS)
                            .align_y(Alignment::Center)
                            .push(widgets::info_chip::<ConflictResolverMessage>(detail, tone))
                            .push(
                                Text::new(action_hint.into())
                                    .size(10)
                                    .color(theme::darcula::TEXT_SECONDARY),
                            ),
                    ),
            ),
    )
    .width(Length::FillPortion(1))
    .padding([10, 12])
    .style(column_header_style(kind))
    .into()
}

fn build_selected_hunk_hint(
    index: usize,
    hunk: &ConflictHunk,
    resolver: &ConflictResolver,
) -> Element<'static, ConflictResolverMessage> {
    let conflict_type = classify_hunk(hunk);
    let detail = match resolver.effective_resolution(index) {
        Some(ResolutionOption::Ours) => {
            "Hunk accepted left. Jump to next or apply all.".to_string()
        }
        Some(ResolutionOption::Theirs) => {
            "Hunk accepted right. Jump to next or apply all.".to_string()
        }
        Some(ResolutionOption::Base) => {
            "Hunk accepted base version. Verify the result.".to_string()
        }
        None => match conflict_type {
            ConflictHunkType::Modified => {
                "Real conflict. Choose between left, right, or base.".to_string()
            }
            ConflictHunkType::OursOnly => {
                "Only left changed. Auto-merge or accept left is safe.".to_string()
            }
            ConflictHunkType::TheirsOnly => {
                "Only right changed. Auto-merge or accept right is safe.".to_string()
            }
            ConflictHunkType::Unchanged => "Content identical. Keep base.".to_string(),
        },
    };

    build_inline_status(
        format!("Hunk {}", index + 1),
        detail,
        match resolver.effective_resolution(index) {
            Some(ResolutionOption::Ours) => BadgeTone::Accent,
            Some(ResolutionOption::Theirs) => BadgeTone::Danger,
            Some(ResolutionOption::Base) => BadgeTone::Neutral,
            None => BadgeTone::Warning,
        },
    )
}

fn build_inline_status(
    label: impl Into<String>,
    detail: impl Into<String>,
    tone: BadgeTone,
) -> Element<'static, ConflictResolverMessage> {
    widgets::status_banner(label, detail, tone)
}

#[derive(Debug, Clone, Copy)]
enum ButtonFlavor {
    Ours,
    Base,
    Theirs,
}

fn build_merge_toolbar(
    selected_index: Option<usize>,
    total_count: usize,
) -> Element<'static, ConflictResolverMessage> {
    let navigator_group = Container::new(
        Row::new()
            .spacing(theme::spacing::XS)
            .align_y(Alignment::Center)
            .push(button::compact_ghost(
                "Back to List",
                Some(ConflictResolverMessage::BackToList),
            ))
            .push(button::compact_ghost(
                "Previous",
                selected_index
                    .filter(|index| *index > 0)
                    .map(|_| ConflictResolverMessage::SelectPrevHunk),
            ))
            .push(button::compact_ghost(
                "Next",
                selected_index
                    .filter(|index| index + 1 < total_count)
                    .map(|_| ConflictResolverMessage::SelectNextHunk),
            ))
            .push_maybe(selected_index.map(|index| {
                widgets::info_chip::<ConflictResolverMessage>(
                    format!("Go to {}", index + 1),
                    BadgeTone::Accent,
                )
            })),
    )
    .padding([10, 12])
    .style(theme::panel_style(Surface::Raised));

    let action_group = Container::new(
        Row::new()
            .spacing(theme::spacing::XS)
            .align_y(Alignment::Center)
            .push(build_toolbar_action(
                "<<",
                "Left",
                ButtonFlavor::Ours,
                selected_index.map(ConflictResolverMessage::ChooseOursForHunk),
            ))
            .push(build_toolbar_action(
                "=",
                "Base",
                ButtonFlavor::Base,
                selected_index.map(ConflictResolverMessage::ChooseBaseForHunk),
            ))
            .push(build_toolbar_action(
                ">>",
                "Right",
                ButtonFlavor::Theirs,
                selected_index.map(ConflictResolverMessage::ChooseTheirsForHunk),
            )),
    )
    .padding([10, 12])
    .style(theme::panel_style(Surface::Accent));

    let batch_group = Container::new(
        Row::new()
            .spacing(theme::spacing::XS)
            .align_y(Alignment::Center)
            .push(button::compact_ghost(
                "Auto Merge",
                (total_count > 0).then_some(ConflictResolverMessage::AutoMerge),
            ))
            .push(button::compact_ghost(
                "All Left",
                (total_count > 0).then_some(ConflictResolverMessage::AcceptOursAll),
            ))
            .push(button::compact_ghost(
                "All Right",
                (total_count > 0).then_some(ConflictResolverMessage::AcceptTheirsAll),
            ))
            .push(button::compact_ghost(
                "Refresh",
                Some(ConflictResolverMessage::Refresh),
            )),
    )
    .padding([10, 12])
    .style(theme::panel_style(Surface::Raised));

    Row::new()
        .spacing(theme::spacing::SM)
        .push(navigator_group)
        .push(action_group)
        .push(Space::new().width(Length::Fill))
        .push(batch_group)
        .into()
}

fn build_hunk_navigator(resolver: &ConflictResolver) -> Element<'static, ConflictResolverMessage> {
    let row = resolver.diff.hunks.iter().enumerate().fold(
        Row::new().spacing(theme::spacing::XS),
        |row, (index, hunk)| {
            let is_selected = resolver.selected_hunk == Some(index);
            let resolution = resolver.effective_resolution(index);
            let tone = match resolution {
                Some(ResolutionOption::Ours) => BadgeTone::Accent,
                Some(ResolutionOption::Theirs) => BadgeTone::Danger,
                Some(ResolutionOption::Base) => BadgeTone::Neutral,
                None => match classify_hunk(hunk) {
                    ConflictHunkType::Modified => BadgeTone::Warning,
                    ConflictHunkType::OursOnly => BadgeTone::Accent,
                    ConflictHunkType::TheirsOnly => BadgeTone::Danger,
                    ConflictHunkType::Unchanged => BadgeTone::Neutral,
                },
            };

            row.push(
                Button::new(
                    Row::new()
                        .spacing(theme::spacing::XS)
                        .align_y(Alignment::Center)
                        .push(
                            Text::new(format!("{}", index + 1))
                                .size(10)
                                .color(theme::darcula::TEXT_PRIMARY),
                        )
                        .push(widgets::info_chip::<ConflictResolverMessage>(
                            match resolution {
                                Some(ResolutionOption::Ours) => "Left",
                                Some(ResolutionOption::Theirs) => "Right",
                                Some(ResolutionOption::Base) => "Base",
                                None => "Pending",
                            },
                            tone,
                        )),
                )
                .padding([4, 7])
                .style(theme::button_style(if is_selected {
                    theme::ButtonTone::TabActive
                } else {
                    theme::ButtonTone::TabInactive
                }))
                .on_press(ConflictResolverMessage::SelectHunk(index)),
            )
        },
    );

    Container::new(scrollable::styled_horizontal(row))
        .padding([8, 10])
        .style(theme::panel_style(Surface::Raised))
        .into()
}

fn build_toolbar_action(
    icon: &'static str,
    label: &'static str,
    flavor: ButtonFlavor,
    message: Option<ConflictResolverMessage>,
) -> Element<'static, ConflictResolverMessage> {
    let button = Button::new(
        Row::new()
            .spacing(theme::spacing::XS)
            .align_y(Alignment::Center)
            .push(Text::new(icon).size(10).color(theme::darcula::TEXT_PRIMARY))
            .push(Text::new(label).size(10)),
    )
    .padding([6, 10])
    .style(toolbar_action_style(flavor));

    if let Some(message) = message {
        button.on_press(message).into()
    } else {
        button.into()
    }
}

fn build_header_action(
    icon: &'static str,
    label: &'static str,
    flavor: ButtonFlavor,
    message: Option<ConflictResolverMessage>,
) -> Element<'static, ConflictResolverMessage> {
    let button = Button::new(
        Row::new()
            .spacing(theme::spacing::XS)
            .align_y(Alignment::Center)
            .push(Text::new(icon).size(10).color(theme::darcula::TEXT_PRIMARY))
            .push(Text::new(label).size(10)),
    )
    .padding([6, 10])
    .style(header_action_style(flavor));

    if let Some(message) = message {
        button.on_press(message).into()
    } else {
        button.into()
    }
}

fn classify_hunk(hunk: &ConflictHunk) -> ConflictHunkType {
    let mut ours_only_count = 0;
    let mut theirs_only_count = 0;
    let mut modified_count = 0;
    let mut unchanged_count = 0;

    for line in &hunk.lines {
        match line.line_type {
            ConflictLineType::OursOnly => ours_only_count += 1,
            ConflictLineType::TheirsOnly => theirs_only_count += 1,
            ConflictLineType::Modified => modified_count += 1,
            ConflictLineType::Unchanged => unchanged_count += 1,
            ConflictLineType::Empty | ConflictLineType::ConflictMarker => {}
        }
    }

    if modified_count > 0 {
        ConflictHunkType::Modified
    } else if ours_only_count > 0 && theirs_only_count == 0 {
        ConflictHunkType::OursOnly
    } else if theirs_only_count > 0 && ours_only_count == 0 {
        ConflictHunkType::TheirsOnly
    } else if unchanged_count > 0 && ours_only_count == 0 && theirs_only_count == 0 {
        ConflictHunkType::Unchanged
    } else {
        ConflictHunkType::Modified
    }
}

fn default_line_at(
    base_lines: &[&str],
    ours_lines: &[&str],
    theirs_lines: &[&str],
    index: usize,
) -> Option<String> {
    base_lines
        .get(index)
        .or_else(|| ours_lines.get(index))
        .or_else(|| theirs_lines.get(index))
        .map(|line| (*line).to_string())
}

fn select_line_for_resolution(
    line: &git_core::diff::ConflictLine,
    option: ResolutionOption,
) -> Option<String> {
    let preferred = match option {
        ResolutionOption::Ours => line.ours_line.as_ref(),
        ResolutionOption::Theirs => line.theirs_line.as_ref(),
        ResolutionOption::Base => line.base_line.as_ref(),
    };

    preferred
        .or(line.base_line.as_ref())
        .or(line.ours_line.as_ref())
        .or(line.theirs_line.as_ref())
        .cloned()
}

fn hunk_card_style(selected: bool) -> impl Fn(&Theme) -> container::Style {
    move |_theme| container::Style {
        background: Some(Background::Color(if selected {
            blend(theme::darcula::BG_PANEL, theme::darcula::ACCENT_WEAK, 0.54)
        } else {
            theme::darcula::BG_RAISED
        })),
        border: Border {
            width: 1.0,
            color: if selected {
                theme::darcula::ACCENT.scale_alpha(0.48)
            } else {
                theme::darcula::BORDER.scale_alpha(0.72)
            },
            radius: theme::radius::LG.into(),
        },
        shadow: iced::Shadow {
            color: Color::from_rgba(0.149, 0.149, 0.149, 0.06),
            offset: iced::Vector::new(0.0, 8.0),
            blur_radius: 18.0,
        },
        ..Default::default()
    }
}

fn hunk_header_style(
    selected: bool,
    resolution: Option<ResolutionOption>,
) -> impl Fn(&Theme) -> container::Style {
    move |_theme| {
        let accent = match resolution {
            Some(ResolutionOption::Ours) => theme::darcula::ACCENT,
            Some(ResolutionOption::Theirs) => theme::darcula::DANGER,
            Some(ResolutionOption::Base) => theme::darcula::WARNING,
            None => theme::darcula::SELECTION_BG,
        };

        container::Style {
            background: Some(Background::Color(if selected {
                blend(theme::darcula::BG_PANEL, accent, 0.24)
            } else {
                blend(theme::darcula::BG_RAISED, accent, 0.10)
            })),
            border: Border {
                width: 1.0,
                color: if selected {
                    accent.scale_alpha(0.44)
                } else {
                    theme::darcula::SEPARATOR.scale_alpha(0.72)
                },
                radius: theme::radius::LG.into(),
            },
            ..Default::default()
        }
    }
}

fn selection_strip_style(
    selected: bool,
    resolution: Option<ResolutionOption>,
) -> impl Fn(&Theme) -> container::Style {
    move |_theme| {
        let color = match resolution {
            Some(ResolutionOption::Ours) => theme::darcula::ACCENT,
            Some(ResolutionOption::Theirs) => theme::darcula::DANGER,
            Some(ResolutionOption::Base) => theme::darcula::WARNING,
            None if selected => theme::darcula::ACCENT.scale_alpha(0.82),
            None => theme::darcula::BG_PANEL,
        };

        container::Style {
            background: Some(Background::Color(color)),
            border: Border {
                width: 0.0,
                color: Color::TRANSPARENT,
                radius: theme::radius::SM.into(),
            },
            ..Default::default()
        }
    }
}

fn editor_column_style(
    kind: EditorColumnKind,
    selected: bool,
) -> impl Fn(&Theme) -> container::Style {
    move |_theme| {
        let tint = match kind {
            EditorColumnKind::Ours => theme::darcula::ACCENT,
            EditorColumnKind::Result => theme::darcula::SELECTION_BG,
            EditorColumnKind::Theirs => theme::darcula::DANGER,
        };

        container::Style {
            background: Some(Background::Color(blend(
                theme::darcula::BG_RAISED,
                tint,
                if selected { 0.10 } else { 0.04 },
            ))),
            border: Border {
                width: 1.0,
                color: blend(
                    theme::darcula::BORDER,
                    tint,
                    if selected { 0.22 } else { 0.10 },
                ),
                radius: theme::radius::LG.into(),
            },
            ..Default::default()
        }
    }
}

fn column_header_style(kind: EditorColumnKind) -> impl Fn(&Theme) -> container::Style {
    move |_theme| {
        let tint = match kind {
            EditorColumnKind::Ours => theme::darcula::ACCENT,
            EditorColumnKind::Result => theme::darcula::SUCCESS,
            EditorColumnKind::Theirs => theme::darcula::DANGER,
        };

        container::Style {
            background: Some(Background::Color(blend(
                theme::darcula::BG_RAISED,
                tint,
                0.12,
            ))),
            border: Border {
                width: 1.0,
                color: blend(theme::darcula::SEPARATOR, tint, 0.18),
                radius: theme::radius::LG.into(),
            },
            ..Default::default()
        }
    }
}

fn column_icon_style(kind: EditorColumnKind) -> impl Fn(&Theme) -> container::Style {
    move |_theme| {
        let tint = match kind {
            EditorColumnKind::Ours => theme::darcula::ACCENT,
            EditorColumnKind::Result => theme::darcula::SUCCESS,
            EditorColumnKind::Theirs => theme::darcula::DANGER,
        };

        container::Style {
            background: Some(Background::Color(blend(
                theme::darcula::BG_PANEL,
                tint,
                0.22,
            ))),
            border: Border {
                width: 1.0,
                color: tint.scale_alpha(0.40),
                radius: theme::radius::LG.into(),
            },
            ..Default::default()
        }
    }
}

fn toolbar_action_style(
    flavor: ButtonFlavor,
) -> impl Fn(&Theme, iced::widget::button::Status) -> iced::widget::button::Style {
    move |_theme, status| {
        let (background, border, text) = match flavor {
            ButtonFlavor::Ours => (
                blend(theme::darcula::BG_PANEL, theme::darcula::ACCENT, 0.14),
                theme::darcula::ACCENT.scale_alpha(0.42),
                theme::darcula::TEXT_PRIMARY,
            ),
            ButtonFlavor::Base => (
                blend(theme::darcula::BG_PANEL, theme::darcula::WARNING, 0.12),
                theme::darcula::WARNING.scale_alpha(0.38),
                theme::darcula::TEXT_PRIMARY,
            ),
            ButtonFlavor::Theirs => (
                blend(theme::darcula::BG_PANEL, theme::darcula::DANGER, 0.14),
                theme::darcula::DANGER.scale_alpha(0.42),
                theme::darcula::TEXT_PRIMARY,
            ),
        };

        let hovered = matches!(status, iced::widget::button::Status::Hovered);
        let disabled = matches!(status, iced::widget::button::Status::Disabled);
        iced::widget::button::Style {
            background: Some(Background::Color(if disabled {
                blend(background, theme::darcula::BG_MAIN, 0.40)
            } else if hovered {
                blend(background, Color::WHITE, 0.06)
            } else {
                background
            })),
            border: Border {
                width: 1.0,
                color: if disabled {
                    theme::darcula::TEXT_DISABLED
                } else {
                    border
                },
                radius: theme::radius::LG.into(),
            },
            text_color: if disabled {
                theme::darcula::TEXT_DISABLED
            } else {
                text
            },
            ..Default::default()
        }
    }
}

fn header_action_style(
    flavor: ButtonFlavor,
) -> impl Fn(&Theme, iced::widget::button::Status) -> iced::widget::button::Style {
    move |_theme, status| {
        let (background, border) = match flavor {
            ButtonFlavor::Ours => (
                blend(theme::darcula::BG_RAISED, theme::darcula::ACCENT, 0.14),
                theme::darcula::ACCENT.scale_alpha(0.40),
            ),
            ButtonFlavor::Base => (
                blend(theme::darcula::BG_RAISED, theme::darcula::WARNING, 0.12),
                theme::darcula::WARNING.scale_alpha(0.36),
            ),
            ButtonFlavor::Theirs => (
                blend(theme::darcula::BG_RAISED, theme::darcula::DANGER, 0.14),
                theme::darcula::DANGER.scale_alpha(0.40),
            ),
        };

        let hovered = matches!(status, iced::widget::button::Status::Hovered);
        let disabled = matches!(status, iced::widget::button::Status::Disabled);
        iced::widget::button::Style {
            background: Some(Background::Color(if disabled {
                blend(background, theme::darcula::BG_MAIN, 0.35)
            } else if hovered {
                blend(background, Color::WHITE, 0.05)
            } else {
                background
            })),
            border: Border {
                width: 1.0,
                color: if disabled {
                    theme::darcula::TEXT_DISABLED
                } else {
                    border
                },
                radius: theme::radius::LG.into(),
            },
            text_color: if disabled {
                theme::darcula::TEXT_DISABLED
            } else {
                theme::darcula::TEXT_PRIMARY
            },
            ..Default::default()
        }
    }
}

fn conflict_line_style(tone: PaneTone, selected: bool) -> impl Fn(&Theme) -> container::Style {
    move |_theme| container::Style {
        background: Some(Background::Color(match tone {
            PaneTone::Neutral => Color::TRANSPARENT,
            PaneTone::Ours => blend(
                theme::darcula::BG_EDITOR,
                theme::darcula::ACCENT,
                if selected { 0.16 } else { 0.10 },
            ),
            PaneTone::Theirs => blend(
                theme::darcula::BG_EDITOR,
                theme::darcula::DANGER,
                if selected { 0.14 } else { 0.10 },
            ),
            PaneTone::Base => blend(theme::darcula::BG_EDITOR, theme::darcula::WARNING, 0.08),
            PaneTone::Marker => blend(
                theme::darcula::BG_EDITOR,
                theme::darcula::SELECTION_BG,
                0.20,
            ),
            PaneTone::Empty => Color::TRANSPARENT,
        })),
        border: Border {
            width: 0.0,
            color: Color::TRANSPARENT,
            radius: theme::radius::MD.into(),
        },
        ..Default::default()
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
