//! Unified diff viewer styled closer to the IntelliJ IDEA editor diff.

use crate::theme;
use crate::widgets::{self, diff_file_header, scrollable, syntax_highlighting};
use git_core::diff::{Diff, DiffHunk, DiffLine, DiffLineOrigin, FileDiff};
use iced::widget::{container, text, Column, Container, Row, Space, Text};
use iced::{Alignment, Background, Border, Color, Element, Font, Length, Theme};
const MARKER_WIDTH: f32 = 4.0;
const GUTTER_WIDTH: f32 = 82.0;
const PREFIX_WIDTH: f32 = 12.0;
const SEPARATOR_WIDTH: f32 = 1.0;

pub struct DiffViewer<'a> {
    diff: &'a Diff,
}

impl<'a> DiffViewer<'a> {
    pub fn new(diff: &'a Diff) -> Self {
        Self { diff }
    }

    pub fn view<Message: Clone + 'static>(&self) -> Element<'a, Message> {
        if self.diff.files.is_empty() {
            return widgets::panel_empty_state(
                "差异",
                "当前没有可显示的 diff",
                "切换文件、刷新状态，或比较其它提交后再查看这里。",
                None,
            );
        }

        let show_file_header = self.diff.files.len() > 1;
        let mut content = Column::new().spacing(theme::spacing::XS);

        for file_diff in &self.diff.files {
            content = content.push(render_file_diff(file_diff, show_file_header));
        }

        scrollable::styled(content).height(Length::Fill).into()
    }
}

pub fn file_preview<'a, Message: Clone + 'static>(file_diff: &'a FileDiff) -> Element<'a, Message> {
    render_file_diff(file_diff, false)
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
    let mut editor_lines = Column::new().spacing(0).width(Length::Shrink);

    if file_diff.hunks.is_empty() {
        editor_lines = editor_lines.push(render_empty_editor_row());
    }

    for (index, hunk) in file_diff.hunks.iter().enumerate() {
        if index > 0 {
            editor_lines = editor_lines.push(editor_divider());
        }

        editor_lines = editor_lines.push(render_hunk_header(hunk));

        let mut line_highlighter = syntax_highlighter.start_hunk();
        for line in &hunk.lines {
            editor_lines = editor_lines.push(render_unified_line(line, &mut line_highlighter));
        }
    }

    Container::new(
        scrollable::styled_horizontal(Container::new(editor_lines).width(Length::Shrink))
            .width(Length::Fill),
    )
    .width(Length::Fill)
    .style(editor_surface_style())
    .into()
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

    Row::new()
        .spacing(0)
        .push(marker_bar(theme::darcula::SEPARATOR.scale_alpha(0.82)))
        .push(gutter_placeholder())
        .push(vertical_separator())
        .push(
            Container::new(
                Text::new(header)
                    .size(10)
                    .font(Font::MONOSPACE)
                    .wrapping(text::Wrapping::None)
                    .color(theme::darcula::TEXT_SECONDARY),
            )
            .padding([4, 10])
            .style(hunk_header_style()),
        )
        .into()
}

fn render_unified_line<Message: Clone + 'static>(
    line: &DiffLine,
    line_highlighter: &mut syntax_highlighting::HunkSyntaxHighlighter,
) -> Element<'static, Message> {
    let gutter_background = gutter_background(&line.origin);
    let code_background = code_background(&line.origin);

    Row::new()
        .spacing(0)
        .push(marker_bar(marker_color(&line.origin)))
        .push(
            Container::new(
                Row::new()
                    .spacing(6)
                    .align_y(Alignment::Center)
                    .push(
                        Text::new(format_line_number(line.old_lineno))
                            .size(10)
                            .font(Font::MONOSPACE)
                            .color(theme::darcula::TEXT_SECONDARY)
                            .width(Length::Fixed(28.0)),
                    )
                    .push(
                        Text::new(format_line_number(line.new_lineno))
                            .size(10)
                            .font(Font::MONOSPACE)
                            .color(theme::darcula::TEXT_SECONDARY)
                            .width(Length::Fixed(28.0)),
                    ),
            )
            .padding([1, 8])
            .width(Length::Fixed(GUTTER_WIDTH))
            .style(simple_fill_style(gutter_background)),
        )
        .push(vertical_separator())
        .push(
            Container::new(
                Row::new()
                    .spacing(6)
                    .align_y(Alignment::Start)
                    .push(
                        Text::new(prefix_for_origin(&line.origin))
                            .size(11)
                            .font(Font::MONOSPACE)
                            .color(prefix_color(&line.origin))
                            .width(Length::Fixed(PREFIX_WIDTH)),
                    )
                    .push(line_highlighter.view(&line.origin, &line.content)),
            )
            .padding([1, 10])
            .style(simple_fill_style(code_background)),
        )
        .into()
}

fn render_empty_editor_row<Message: Clone + 'static>() -> Element<'static, Message> {
    Row::new()
        .spacing(0)
        .push(marker_bar(Color::TRANSPARENT))
        .push(gutter_placeholder())
        .push(vertical_separator())
        .push(
            Container::new(
                Text::new("当前文件没有文本 diff，可切换其它文件继续查看。")
                    .size(11)
                    .color(theme::darcula::TEXT_SECONDARY),
            )
            .padding([8, 10])
            .style(simple_fill_style(theme::darcula::BG_EDITOR)),
        )
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

fn editor_divider<Message: Clone + 'static>() -> Element<'static, Message> {
    Container::new(Space::new().width(Length::Fill).height(Length::Fixed(1.0)))
        .style(simple_fill_style(
            theme::darcula::SEPARATOR.scale_alpha(0.72),
        ))
        .into()
}

fn marker_bar<'a, Message: 'a>(color: Color) -> Container<'a, Message> {
    Container::new(Space::new().width(Length::Fixed(MARKER_WIDTH)))
        .width(Length::Fixed(MARKER_WIDTH))
        .style(simple_fill_style(color))
}

fn gutter_placeholder<'a, Message: 'a>() -> Container<'a, Message> {
    Container::new(Space::new().width(Length::Fixed(GUTTER_WIDTH)))
        .width(Length::Fixed(GUTTER_WIDTH))
        .style(simple_fill_style(mix_colors(
            theme::darcula::BG_EDITOR,
            theme::darcula::BG_RAISED,
            0.28,
        )))
}

fn vertical_separator<'a, Message: 'a>() -> Container<'a, Message> {
    Container::new(Space::new().width(Length::Fixed(SEPARATOR_WIDTH)))
        .width(Length::Fixed(SEPARATOR_WIDTH))
        .style(simple_fill_style(
            theme::darcula::SEPARATOR.scale_alpha(0.72),
        ))
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
            0.36,
        ))),
        ..Default::default()
    }
}

fn simple_fill_style(color: Color) -> impl Fn(&Theme) -> container::Style {
    move |_theme| container::Style {
        background: Some(Background::Color(color)),
        ..Default::default()
    }
}
