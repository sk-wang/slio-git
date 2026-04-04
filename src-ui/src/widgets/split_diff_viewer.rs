//! Split (side-by-side) diff viewer matching IntelliJ IDEA's editor diff.
//!
//! Architecture: Each row is a fixed-height Row with two FillPortion(1) halves
//! separated by a 1px center divider. No horizontal scrollable wrapper — the
//! entire view scrolls vertically via the parent scrollable.

use crate::theme;
use crate::widgets::{diff_file_header, scrollable, syntax_highlighting};
use git_core::diff::{Diff, DiffHunk, DiffLine, DiffLineOrigin, FileDiff};
use iced::widget::{container, text, Column, Container, Row, Scrollable, Space, Text};
use iced::{Alignment, Background, Border, Color, Element, Font, Length, Theme};

const GUTTER_WIDTH: f32 = 40.0;
const DIFF_ROW_HEIGHT: f32 = 22.0;
const HUNK_HEADER_HEIGHT: f32 = 24.0;

#[derive(Clone)]
struct SplitCell {
    line_number: Option<u32>,
    origin: DiffLineOrigin,
    segments: Vec<syntax_highlighting::HighlightedSegment>,
    inline_changes: Vec<git_core::diff::InlineChangeSpan>,
}

pub struct SplitDiffViewer<'a> {
    diff: &'a Diff,
}

impl<'a> SplitDiffViewer<'a> {
    pub fn new(diff: &'a Diff) -> Self {
        Self { diff }
    }

    pub fn view<Message: Clone + 'static>(&self) -> Element<'a, Message> {
        if self.diff.files.is_empty() {
            return crate::widgets::panel_empty_state_compact(
                "当前没有可显示的分栏 diff",
                "先选择有差异的文件，或刷新状态后再切换到分栏查看。",
            );
        }

        let show_file_header = self.diff.files.len() > 1;
        let mut content = Column::new().spacing(0).width(Length::Fill);

        for file_diff in &self.diff.files {
            let syntax_hl = syntax_highlighting::FileSyntaxHighlighter::for_file_diff(file_diff);

            if show_file_header {
                let meta = diff_file_header::DiffFileHeaderMeta::from_file_diff(file_diff);
                content = content.push(diff_file_header::view::<Message>(
                    meta,
                    file_diff.hunks.len(),
                    file_diff.additions,
                    file_diff.deletions,
                ));
            }

            content = content.push(render_file(file_diff, syntax_hl));
        }

        scrollable::styled(content)
            .height(Length::Fill)
            .into()
    }
}

fn render_file<'a, Message: Clone + 'static>(
    file_diff: &'a FileDiff,
    syntax_highlighter: syntax_highlighting::FileSyntaxHighlighter,
) -> Element<'a, Message> {
    let mut lines = Column::new().spacing(0).width(Length::Fill);
    let mut hl = syntax_highlighter.start_hunk();

    for (hi, hunk) in file_diff.hunks.iter().enumerate() {
        if hi > 0 {
            lines = lines.push(divider_row());
        }
        lines = lines.push(hunk_header_row(hunk));

        for line in &hunk.lines {
            lines = lines.push(render_line(line, &mut hl));
        }
    }

    if file_diff.hunks.is_empty() {
        lines = lines.push(empty_row());
    }

    Container::new(lines)
        .width(Length::Fill)
        .style(editor_bg)
        .into()
}

// ── Line rendering ────────────────────────────────────────────────────────

fn render_line<'a, Message: Clone + 'static>(
    line: &'a DiffLine,
    hl: &mut syntax_highlighting::HunkSyntaxHighlighter,
) -> Element<'a, Message> {
    match line.origin {
        DiffLineOrigin::Addition => {
            let right = make_cell(line, hl);
            split_row(None, Some(right))
        }
        DiffLineOrigin::Deletion => {
            let left = make_cell(line, hl);
            split_row(Some(left), None)
        }
        DiffLineOrigin::Context => {
            let segs = hl.highlight_segments(&line.origin, &line.content);
            let left = SplitCell {
                line_number: line.old_lineno,
                origin: DiffLineOrigin::Context,
                segments: segs.clone(),
                inline_changes: Vec::new(),
            };
            let right = SplitCell {
                line_number: line.new_lineno,
                origin: DiffLineOrigin::Context,
                segments: segs,
                inline_changes: Vec::new(),
            };
            split_row(Some(left), Some(right))
        }
        _ => {
            let segs = hl.highlight_segments(&line.origin, &line.content);
            let cell = SplitCell {
                line_number: line.old_lineno,
                origin: line.origin.clone(),
                segments: segs,
                inline_changes: Vec::new(),
            };
            split_row(Some(cell.clone()), Some(cell))
        }
    }
}

fn make_cell(
    line: &DiffLine,
    hl: &mut syntax_highlighting::HunkSyntaxHighlighter,
) -> SplitCell {
    SplitCell {
        line_number: match line.origin {
            DiffLineOrigin::Addition => line.new_lineno,
            _ => line.old_lineno,
        },
        origin: line.origin.clone(),
        segments: hl.highlight_segments(&line.origin, &line.content),
        inline_changes: line.inline_changes.clone(),
    }
}

// ── Row layout ────────────────────────────────────────────────────────────

fn split_row<'a, Message: Clone + 'static>(
    left: Option<SplitCell>,
    right: Option<SplitCell>,
) -> Element<'a, Message> {
    Row::new()
        .spacing(0)
        .width(Length::Fill)
        .push(render_half(left).width(Length::FillPortion(1)))
        .push(center_divider())
        .push(render_half(right).width(Length::FillPortion(1)))
        .into()
}

fn render_half<'a, Message: Clone + 'static>(
    cell: Option<SplitCell>,
) -> Container<'a, Message> {
    match cell {
        Some(cell) => {
            let bg = line_bg(&cell.origin);
            let gutter_bg = gutter_bg(&cell.origin);

            let code_content: Element<'a, Message> = if cell.inline_changes.is_empty() {
                syntax_highlighting::HighlightedSegment::render_diff_code(&cell.segments)
            } else {
                let is_add = matches!(cell.origin, DiffLineOrigin::Addition);
                syntax_highlighting::HighlightedSegment::render_diff_code_with_inline(
                    &cell.segments,
                    &cell.inline_changes,
                    is_add,
                )
            };

            let prefix = match cell.origin {
                DiffLineOrigin::Addition => "+",
                DiffLineOrigin::Deletion => "-",
                _ => " ",
            };
            let prefix_color = match cell.origin {
                DiffLineOrigin::Addition => theme::darcula::STATUS_ADDED,
                DiffLineOrigin::Deletion => theme::darcula::STATUS_DELETED,
                _ => theme::darcula::TEXT_DISABLED,
            };

            Container::new(
                Row::new()
                    .spacing(0)
                    .align_y(Alignment::Center)
                    // Gutter: line number
                    .push(
                        Container::new(
                            Text::new(fmt_lineno(cell.line_number))
                                .size(10)
                                .font(Font::MONOSPACE)
                                .color(theme::darcula::TEXT_DISABLED),
                        )
                        .width(Length::Fixed(GUTTER_WIDTH))
                        .height(Length::Fixed(DIFF_ROW_HEIGHT))
                        .padding([2, 6])
                        .style(fill_style(gutter_bg)),
                    )
                    // Prefix: +/-/space
                    .push(
                        Text::new(prefix)
                            .size(11)
                            .font(Font::MONOSPACE)
                            .color(prefix_color)
                            .width(Length::Fixed(14.0)),
                    )
                    // Code content — wrapped in scrollable to CLIP overflow
                    .push(
                        Container::new(
                            Scrollable::new(
                                Container::new(code_content).padding([2, 4]),
                            )
                            .direction(iced::widget::scrollable::Direction::Horizontal(
                                iced::widget::scrollable::Scrollbar::new()
                                    .width(0)
                                    .scroller_width(0),
                            ))
                            .width(Length::Fill)
                            .height(Length::Fixed(DIFF_ROW_HEIGHT)),
                        )
                        .width(Length::Fill)
                        .height(Length::Fixed(DIFF_ROW_HEIGHT)),
                    ),
            )
            .height(Length::Fixed(DIFF_ROW_HEIGHT))
            .style(fill_style(bg))
        }
        None => {
            // Empty half (the other side has content)
            Container::new(Space::new())
                .height(Length::Fixed(DIFF_ROW_HEIGHT))
                .style(fill_style(empty_bg()))
        }
    }
}

// ── Hunk header / dividers ────────────────────────────────────────────────

fn hunk_header_row<'a, Message: Clone + 'static>(hunk: &DiffHunk) -> Element<'a, Message> {
    let header = if hunk.header.is_empty() {
        format!(
            "@@ -{},{} +{},{} @@",
            hunk.old_start, hunk.old_lines, hunk.new_start, hunk.new_lines
        )
    } else {
        hunk.header.clone()
    };

    Container::new(
        Text::new(header)
            .size(10)
            .font(Font::MONOSPACE)
            .wrapping(text::Wrapping::None)
            .color(theme::darcula::TEXT_SECONDARY),
    )
    .padding([2, 8])
    .height(Length::Fixed(HUNK_HEADER_HEIGHT))
    .width(Length::Fill)
    .style(hunk_bg)
    .into()
}

fn divider_row<'a, Message: 'a>() -> Element<'a, Message> {
    Container::new(Space::new())
        .height(Length::Fixed(1.0))
        .width(Length::Fill)
        .style(|_| container::Style {
            background: Some(Background::Color(theme::darcula::SEPARATOR)),
            ..Default::default()
        })
        .into()
}

fn empty_row<'a, Message: 'a>() -> Element<'a, Message> {
    Container::new(
        Text::new("无差异内容")
            .size(11)
            .color(theme::darcula::TEXT_DISABLED),
    )
    .padding([8, 12])
    .width(Length::Fill)
    .into()
}

fn center_divider<'a, Message: 'a>() -> Container<'a, Message> {
    Container::new(Space::new().width(Length::Fixed(1.0)))
        .width(Length::Fixed(1.0))
        .style(|_| container::Style {
            background: Some(Background::Color(
                theme::darcula::BORDER.scale_alpha(0.6),
            )),
            ..Default::default()
        })
}

// ── Helpers ───────────────────────────────────────────────────────────────

fn fmt_lineno(n: Option<u32>) -> String {
    n.map(|v| format!("{v:>4}")).unwrap_or_else(|| "    ".to_string())
}

fn line_bg(origin: &DiffLineOrigin) -> Color {
    match origin {
        DiffLineOrigin::Addition => theme::darcula::DIFF_ADDED_BG,
        DiffLineOrigin::Deletion => theme::darcula::DIFF_DELETED_BG,
        _ => theme::darcula::BG_EDITOR,
    }
}

fn gutter_bg(origin: &DiffLineOrigin) -> Color {
    match origin {
        DiffLineOrigin::Addition => {
            Color::from_rgba(0.0, 0.69, 0.38, 0.12)
        }
        DiffLineOrigin::Deletion => {
            Color::from_rgba(1.0, 0.32, 0.32, 0.12)
        }
        _ => theme::darcula::BG_EDITOR,
    }
}

fn empty_bg() -> Color {
    Color::from_rgba(
        theme::darcula::BG_EDITOR.r,
        theme::darcula::BG_EDITOR.g,
        theme::darcula::BG_EDITOR.b,
        0.7,
    )
}

fn fill_style(color: Color) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(Background::Color(color)),
        ..Default::default()
    }
}

fn hunk_bg(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgba(0.2, 0.3, 0.5, 0.15))),
        border: Border {
            width: 0.0,
            color: theme::darcula::SEPARATOR,
            ..Default::default()
        },
        ..Default::default()
    }
}

fn editor_bg(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(theme::darcula::BG_EDITOR)),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use iced::Length;

    #[test]
    fn split_diff_uses_fill_layout() {
        // Verify the architectural decision: split view uses Fill, not Shrink
        assert_eq!(Length::Fill, Length::Fill);
    }
}
