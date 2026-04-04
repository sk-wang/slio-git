//! Split diff viewer styled closer to the IntelliJ IDEA editor diff.

use crate::theme;
use crate::widgets::{self, diff_file_header, scrollable, syntax_highlighting};
use git_core::diff::{Diff, DiffHunk, DiffLine, DiffLineOrigin, FileDiff};
use iced::widget::{self, container, text, Column, Container, Row, Space, Text};
use iced::{Alignment, Background, Border, Color, Element, Font, Length, Theme};
const MARKER_WIDTH: f32 = 3.0;
const GUTTER_WIDTH: f32 = 36.0;
const PREFIX_WIDTH: f32 = 10.0;
const SEPARATOR_WIDTH: f32 = 1.0;
const DIFF_ROW_HEIGHT: f32 = 20.0;
const HUNK_HEADER_HEIGHT: f32 = 20.0;

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
            return widgets::panel_empty_state_compact(
                "当前没有可显示的分栏 diff",
                "先选择有差异的文件，或刷新状态后再切换到分栏查看。",
            );
        }

        let show_file_header = self.diff.files.len() > 1;
        let mut content = Column::new().spacing(theme::spacing::XS);

        for file_diff in &self.diff.files {
            content = content.push(render_file_diff(file_diff, show_file_header));
        }

        scrollable::styled(content)
            .id(widget::Id::new("diff-scroll"))
            .height(Length::Fill)
            .into()
    }
}

fn render_file_diff<'a, Message: Clone + 'static>(
    file_diff: &'a FileDiff,
    show_header: bool,
) -> Element<'a, Message> {
    let syntax_highlighter = syntax_highlighting::FileSyntaxHighlighter::for_file_diff(file_diff);
    let editor = render_editor_surface(file_diff, syntax_highlighter);
    let meta = diff_file_header::DiffFileHeaderMeta::from_file_diff(file_diff);

    if show_header {
        Column::new()
            .spacing(theme::spacing::XS)
            .push(diff_file_header::view::<Message>(
                meta,
                file_diff.hunks.len(),
                file_diff.additions,
                file_diff.deletions,
            ))
            .push(editor)
            .into()
    } else {
        editor
    }
}

fn render_editor_surface<'a, Message: Clone + 'static>(
    file_diff: &'a FileDiff,
    syntax_highlighter: syntax_highlighting::FileSyntaxHighlighter,
) -> Element<'a, Message> {
    let mut editor_lines = Column::new().spacing(0).width(Length::Fill);

    if file_diff.hunks.is_empty() {
        editor_lines = editor_lines.push(render_empty_row());
    }

    for (index, hunk) in file_diff.hunks.iter().enumerate() {
        if index > 0 {
            editor_lines = editor_lines.push(editor_divider());
        }

        editor_lines = editor_lines.push(render_hunk_header(hunk));

        let mut line_highlighter = syntax_highlighter.start_hunk();
        for line in &hunk.lines {
            editor_lines = editor_lines.push(render_split_line(line, &mut line_highlighter));
        }
    }

    Container::new(
        scrollable::styled_editor_horizontal(Container::new(editor_lines).width(Length::Fill))
            .width(Length::Fill),
    )
    .width(Length::Fill)
    .style(editor_surface_style())
    .into()
}

fn split_row_width() -> Length {
    Length::Fill
}

fn split_code_cell_width() -> Length {
    Length::Fill
}

fn render_hunk_header<Message: Clone + 'static>(hunk: &DiffHunk) -> Element<'static, Message> {
    let header = if hunk.header.is_empty() {
        format!(
            "@@ -{},{} +{},{} @@",
            hunk.old_start, hunk.old_lines, hunk.new_start, hunk.new_lines
        )
    } else {
        hunk.header.clone()
    };

    Container::new(
        Row::new()
            .spacing(4)
            .align_y(Alignment::Center)
            .push(
                Text::new(header)
                    .size(10)
                    .font(Font::MONOSPACE)
                    .wrapping(text::Wrapping::None)
                    .color(theme::darcula::TEXT_SECONDARY),
            ),
        // Hunk stage/unstage buttons can be added here via builder pattern in future
    )
    .padding([2, 8])
    .height(Length::Fixed(HUNK_HEADER_HEIGHT))
    .width(split_row_width())
    .style(hunk_header_style())
    .into()
}

fn render_split_line<Message: Clone + 'static>(
    line: &DiffLine,
    line_highlighter: &mut syntax_highlighting::HunkSyntaxHighlighter,
) -> Element<'static, Message> {
    match line.origin {
        DiffLineOrigin::Addition => {
            let right = SplitCell {
                line_number: line.new_lineno,
                origin: DiffLineOrigin::Addition,
                segments: line_highlighter.highlight_segments(&line.origin, &line.content),
                inline_changes: line.inline_changes.clone(),
            };
            render_split_row(None, Some(right))
        }
        DiffLineOrigin::Deletion => {
            let left = SplitCell {
                line_number: line.old_lineno,
                origin: DiffLineOrigin::Deletion,
                segments: line_highlighter.highlight_segments(&line.origin, &line.content),
                inline_changes: line.inline_changes.clone(),
            };
            render_split_row(Some(left), None)
        }
        DiffLineOrigin::Context => {
            let segments = line_highlighter.highlight_segments(&line.origin, &line.content);
            let left = SplitCell {
                line_number: line.old_lineno,
                origin: DiffLineOrigin::Context,
                segments: segments.clone(),
                inline_changes: Vec::new(),
            };
            let right = SplitCell {
                line_number: line.new_lineno,
                origin: DiffLineOrigin::Context,
                segments,
                inline_changes: Vec::new(),
            };
            render_split_row(Some(left), Some(right))
        }
        DiffLineOrigin::Header | DiffLineOrigin::HunkHeader => {
            let segments = line_highlighter.highlight_segments(&line.origin, &line.content);
            let left = SplitCell {
                line_number: line.old_lineno,
                origin: DiffLineOrigin::Header,
                segments: segments.clone(),
                inline_changes: Vec::new(),
            };
            let right = SplitCell {
                line_number: line.new_lineno,
                origin: DiffLineOrigin::Header,
                segments,
                inline_changes: Vec::new(),
            };
            render_split_row(Some(left), Some(right))
        }
    }
}

fn render_split_row<Message: Clone + 'static>(
    left: Option<SplitCell>,
    right: Option<SplitCell>,
) -> Element<'static, Message> {
    Row::new()
        .spacing(0)
        .align_y(Alignment::Start)
        .width(Length::Fill)
        .push(Container::new(render_side(left)).width(Length::FillPortion(1)))
        .push(center_divider())
        .push(Container::new(render_side(right)).width(Length::FillPortion(1)))
        .into()
}

fn render_side<Message: Clone + 'static>(cell: Option<SplitCell>) -> Element<'static, Message> {
    match cell {
        Some(cell) => {
            let gutter_background = gutter_background(&cell.origin);
            let code_background = code_background(&cell.origin);

            Row::new()
                .spacing(0)
                .align_y(Alignment::Center)
                .width(split_row_width())
                .push(marker_bar(marker_color(&cell.origin)))
                .push(
                    Container::new(
                        Text::new(format_line_number(cell.line_number))
                            .size(10)
                            .font(Font::MONOSPACE)
                            .color(theme::darcula::TEXT_SECONDARY)
                            .width(Length::Fixed(24.0)),
                    )
                    .padding([0, 4])
                    .height(Length::Fixed(DIFF_ROW_HEIGHT))
                    .width(Length::Fixed(GUTTER_WIDTH))
                    .style(simple_fill_style(gutter_background)),
                )
                .push(vertical_separator())
                .push(
                    Container::new(
                        Row::new()
                            .spacing(3)
                            .align_y(Alignment::Center)
                            .push(
                                Text::new(prefix_for_origin(&cell.origin))
                                    .size(11)
                                    .font(Font::MONOSPACE)
                                    .color(prefix_color(&cell.origin))
                                    .width(Length::Fixed(PREFIX_WIDTH)),
                            )
                            .push(if cell.inline_changes.is_empty() {
                                syntax_highlighting::HighlightedSegment::render_diff_code(
                                    &cell.segments,
                                )
                            } else {
                                let is_add = matches!(cell.origin, DiffLineOrigin::Addition);
                                syntax_highlighting::HighlightedSegment::render_diff_code_with_inline(
                                    &cell.segments,
                                    &cell.inline_changes,
                                    is_add,
                                )
                            }),
                    )
                    .padding([0, 6])
                    .height(Length::Fixed(DIFF_ROW_HEIGHT))
                    .width(split_code_cell_width())
                    .style(simple_fill_style(code_background)),
                )
                .into()
        }
        None => Row::new()
            .spacing(0)
            .align_y(Alignment::Center)
            .width(Length::Fill)
            .push(marker_bar(Color::TRANSPARENT))
            .push(
                Container::new(Space::new().width(Length::Fixed(GUTTER_WIDTH)))
                    .height(Length::Fixed(DIFF_ROW_HEIGHT))
                    .width(Length::Fixed(GUTTER_WIDTH))
                    .style(simple_fill_style(mix_colors(
                        theme::darcula::BG_EDITOR,
                        theme::darcula::BG_RAISED,
                        0.20,
                    ))),
            )
            .push(vertical_separator())
            .push(
                Container::new(Text::new(" "))
                    .padding([0, 6])
                    .height(Length::Fixed(DIFF_ROW_HEIGHT))
                    .width(Length::Fill)
                    .style(simple_fill_style(mix_colors(
                        theme::darcula::BG_EDITOR,
                        theme::darcula::BG_RAISED,
                        0.14,
                    ))),
            )
            .into(),
    }
}

fn render_empty_row<Message: Clone + 'static>() -> Element<'static, Message> {
    Container::new(
        Text::new("当前文件没有文本 diff，可切换其它文件继续查看。")
            .size(11)
            .color(theme::darcula::TEXT_SECONDARY),
    )
    .padding([8, 10])
    .width(split_row_width())
    .style(simple_fill_style(theme::darcula::BG_EDITOR))
    .into()
}

fn prefix_for_origin(origin: &DiffLineOrigin) -> &'static str {
    match origin {
        DiffLineOrigin::Addition => "+",
        DiffLineOrigin::Deletion => "-",
        DiffLineOrigin::Context | DiffLineOrigin::Header | DiffLineOrigin::HunkHeader => " ",
    }
}

fn prefix_color(origin: &DiffLineOrigin) -> Color {
    match origin {
        DiffLineOrigin::Addition => theme::darcula::SUCCESS,
        DiffLineOrigin::Deletion => theme::darcula::DANGER,
        DiffLineOrigin::Context | DiffLineOrigin::Header | DiffLineOrigin::HunkHeader => {
            theme::darcula::TEXT_SECONDARY
        }
    }
}

fn marker_color(origin: &DiffLineOrigin) -> Color {
    match origin {
        DiffLineOrigin::Addition => theme::darcula::SUCCESS,
        DiffLineOrigin::Deletion => theme::darcula::DANGER,
        DiffLineOrigin::Header | DiffLineOrigin::HunkHeader => {
            theme::darcula::ACCENT.scale_alpha(0.64)
        }
        DiffLineOrigin::Context => Color::TRANSPARENT,
    }
}

fn gutter_background(origin: &DiffLineOrigin) -> Color {
    match origin {
        DiffLineOrigin::Addition => mix_colors(
            theme::darcula::BG_RAISED,
            theme::darcula::DIFF_ADDED_BG,
            0.22,
        ),
        DiffLineOrigin::Deletion => mix_colors(
            theme::darcula::BG_RAISED,
            theme::darcula::DIFF_DELETED_BG,
            0.28,
        ),
        DiffLineOrigin::Header | DiffLineOrigin::HunkHeader => {
            mix_colors(theme::darcula::BG_RAISED, theme::darcula::ACCENT_WEAK, 0.34)
        }
        DiffLineOrigin::Context => {
            mix_colors(theme::darcula::BG_EDITOR, theme::darcula::BG_RAISED, 0.28)
        }
    }
}

fn code_background(origin: &DiffLineOrigin) -> Color {
    match origin {
        DiffLineOrigin::Addition => mix_colors(
            theme::darcula::BG_EDITOR,
            theme::darcula::DIFF_ADDED_BG,
            0.44,
        ),
        DiffLineOrigin::Deletion => mix_colors(
            theme::darcula::BG_EDITOR,
            theme::darcula::DIFF_DELETED_BG,
            0.52,
        ),
        DiffLineOrigin::Header | DiffLineOrigin::HunkHeader => {
            mix_colors(theme::darcula::BG_EDITOR, theme::darcula::ACCENT_WEAK, 0.36)
        }
        DiffLineOrigin::Context => theme::darcula::BG_EDITOR,
    }
}

// IDEA-style hunk divider: more prominent with accent color and spacing
fn editor_divider<Message: Clone + 'static>() -> Element<'static, Message> {
    Column::new()
        .spacing(theme::spacing::XS)
        .width(Length::Fill)
        .push(Space::new().height(Length::Fixed(6.0)))
        .push(
            Container::new(Space::new().width(Length::Fill).height(Length::Fixed(2.0)))
                .style(split_divider_style()),
        )
        .push(Space::new().height(Length::Fixed(theme::spacing::XS)))
        .into()
}

fn marker_bar<'a, Message: 'a>(color: Color) -> Container<'a, Message> {
    Container::new(Space::new().width(Length::Fixed(MARKER_WIDTH)))
        .width(Length::Fixed(MARKER_WIDTH))
        .style(simple_fill_style(color))
}

fn vertical_separator<'a, Message: 'a>() -> Container<'a, Message> {
    Container::new(Space::new().width(Length::Fixed(SEPARATOR_WIDTH)))
        .width(Length::Fixed(SEPARATOR_WIDTH))
        .style(simple_fill_style(
            theme::darcula::SEPARATOR.scale_alpha(0.72),
        ))
}

fn center_divider<'a, Message: 'a>() -> Container<'a, Message> {
    Container::new(Space::new().width(Length::Fixed(SEPARATOR_WIDTH)))
        .width(Length::Fixed(SEPARATOR_WIDTH))
        .style(simple_fill_style(theme::darcula::BORDER.scale_alpha(0.92)))
}

fn format_line_number(value: Option<u32>) -> String {
    value
        .map(|number| format!("{number:>4}"))
        .unwrap_or_else(|| "    ".to_string())
}

fn mix_colors(base: Color, overlay: Color, amount: f32) -> Color {
    let amount = amount.clamp(0.0, 1.0);

    Color {
        r: (base.r * (1.0 - amount)) + (overlay.r * amount),
        g: (base.g * (1.0 - amount)) + (overlay.g * amount),
        b: (base.b * (1.0 - amount)) + (overlay.b * amount),
        a: (base.a * (1.0 - amount)) + (overlay.a * amount),
    }
}

fn editor_surface_style() -> impl Fn(&Theme) -> container::Style {
    move |_theme| container::Style {
        background: Some(Background::Color(theme::darcula::BG_EDITOR)),
        border: Border {
            width: 1.0,
            color: theme::darcula::BORDER.scale_alpha(0.84),
            radius: theme::radius::SM.into(),
        },
        ..Default::default()
    }
}

fn hunk_header_style() -> impl Fn(&Theme) -> container::Style {
    move |_theme| container::Style {
        background: Some(Background::Color(mix_colors(
            theme::darcula::BG_EDITOR,
            theme::darcula::ACCENT_WEAK,
            0.45,
        ))),
        border: Border {
            width: 1.0,
            color: theme::darcula::ACCENT.scale_alpha(0.25),
            radius: theme::radius::SM.into(),
        },
        ..Default::default()
    }
}

fn simple_fill_style(color: Color) -> impl Fn(&Theme) -> container::Style {
    move |_theme| container::Style {
        background: Some(Background::Color(color)),
        ..Default::default()
    }
}

// IDEA-style divider for split view
fn split_divider_style() -> impl Fn(&Theme) -> container::Style {
    move |_theme| container::Style {
        background: Some(Background::Color(
            theme::darcula::ACCENT_WEAK.scale_alpha(0.8),
        )),
        border: Border {
            width: 0.0,
            color: Color::TRANSPARENT,
            radius: 0.0.into(),
        },
        shadow: iced::Shadow {
            color: theme::darcula::ACCENT.scale_alpha(0.15),
            offset: iced::Vector::new(0.0, 1.0),
            blur_radius: 4.0,
        },
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::{split_code_cell_width, split_row_width};
    use iced::Length;

    #[test]
    fn split_diff_rows_use_fill_to_split_50_50() {
        assert_eq!(split_row_width(), Length::Fill);
        assert_eq!(split_code_cell_width(), Length::Fill);
    }
}
