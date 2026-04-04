//! Unified diff viewer styled closer to the IntelliJ IDEA editor diff.

use crate::theme;
use crate::widgets::{self, diff_file_header, scrollable, syntax_highlighting};
use git_core::diff::{Diff, DiffHunk, DiffLine, DiffLineOrigin, FileDiff};
use iced::widget::{self, container, text, Column, Container, Row, Space, Text};
use iced::{Alignment, Background, Border, Color, Element, Font, Length, Theme};
const MARKER_WIDTH: f32 = 3.0;
const GUTTER_WIDTH: f32 = 62.0;
const PREFIX_WIDTH: f32 = 10.0;
const SEPARATOR_WIDTH: f32 = 1.0;
const DIFF_ROW_HEIGHT: f32 = 22.0;
const HUNK_HEADER_HEIGHT: f32 = 24.0;

pub struct DiffViewer<'a, Message> {
    diff: &'a Diff,
    on_stage_hunk: Option<Box<dyn Fn(String, usize) -> Message + 'a>>,
    on_unstage_hunk: Option<Box<dyn Fn(String, usize) -> Message + 'a>>,
    blame_entries: Option<&'a [git_core::blame::BlameEntry]>,
    on_blame_click: Option<Box<dyn Fn(String) -> Message + 'a>>,
}

impl<'a, Message: Clone + 'static> DiffViewer<'a, Message> {
    pub fn new(diff: &'a Diff) -> Self {
        Self {
            diff,
            on_stage_hunk: None,
            on_unstage_hunk: None,
            blame_entries: None,
            on_blame_click: None,
        }
    }

    /// Set handler for "Stage Hunk" button. Called with (file_path, hunk_index).
    pub fn with_stage_hunk_handler(
        mut self,
        handler: impl Fn(String, usize) -> Message + 'a,
    ) -> Self {
        self.on_stage_hunk = Some(Box::new(handler));
        self
    }

    /// Set handler for "Unstage Hunk" button. Called with (file_path, hunk_index).
    pub fn with_unstage_hunk_handler(
        mut self,
        handler: impl Fn(String, usize) -> Message + 'a,
    ) -> Self {
        self.on_unstage_hunk = Some(Box::new(handler));
        self
    }

    /// Set blame annotations to display in a gutter column
    pub fn with_blame(mut self, entries: &'a [git_core::blame::BlameEntry]) -> Self {
        self.blame_entries = Some(entries);
        self
    }

    /// Set handler for clicking a blame gutter entry (receives commit ID)
    pub fn with_blame_click_handler(
        mut self,
        handler: impl Fn(String) -> Message + 'a,
    ) -> Self {
        self.on_blame_click = Some(Box::new(handler));
        self
    }

    pub fn view(&self) -> Element<'a, Message> {
        if self.diff.files.is_empty() {
            return widgets::panel_empty_state_compact(
                "当前没有可显示的 diff",
                "选择文件或比较提交后查看差异内容。",
            );
        }

        let show_file_header = self.diff.files.len() > 1;
        let mut content = Column::new().spacing(theme::spacing::XS);

        for file_diff in &self.diff.files {
            content = content.push(self.render_file_diff_with_actions(file_diff, show_file_header));
        }

        scrollable::styled(content)
            .id(widget::Id::new("diff-scroll"))
            .height(Length::Fill)
            .into()
    }

    fn render_file_diff_with_actions(
        &self,
        file_diff: &'a FileDiff,
        show_header: bool,
    ) -> Element<'a, Message> {
        let syntax_highlighter =
            syntax_highlighting::FileSyntaxHighlighter::for_file_diff(file_diff);
        let editor = self.render_editor_with_hunk_actions(file_diff, syntax_highlighter);
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

    fn render_editor_with_hunk_actions(
        &self,
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

            // Build hunk action buttons
            let stage_msg = self
                .on_stage_hunk
                .as_ref()
                .map(|f| f(file_diff.new_path.clone().or_else(|| file_diff.old_path.clone()).unwrap_or_default(), index));
            let unstage_msg = self
                .on_unstage_hunk
                .as_ref()
                .map(|f| f(file_diff.new_path.clone().or_else(|| file_diff.old_path.clone()).unwrap_or_default(), index));

            editor_lines =
                editor_lines.push(render_hunk_header_with_actions(hunk, stage_msg, unstage_msg));

            let mut line_highlighter = syntax_highlighter.start_hunk();
            for line in &hunk.lines {
                editor_lines =
                    editor_lines.push(render_unified_line(line, &mut line_highlighter));
            }
        }

        Container::new(
            scrollable::styled_editor_horizontal(
                Container::new(editor_lines).width(editor_content_width()),
            )
            .width(Length::Fill),
        )
        .width(Length::Fill)
        .style(editor_surface_style())
        .into()
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
        scrollable::styled_editor_horizontal(
            Container::new(editor_lines).width(editor_content_width()),
        )
        .width(Length::Fill),
    )
    .width(Length::Fill)
    .style(editor_surface_style())
    .into()
}

fn editor_content_width() -> Length {
    Length::Shrink
}

fn editor_row_width() -> Length {
    Length::Shrink
}

fn code_cell_width() -> Length {
    Length::Shrink
}

fn render_hunk_header<Message: Clone + 'static>(hunk: &DiffHunk) -> Element<'static, Message> {
    render_hunk_header_with_actions::<Message>(hunk, None, None)
}

fn render_hunk_header_with_actions<Message: Clone + 'static>(
    hunk: &DiffHunk,
    stage_msg: Option<Message>,
    unstage_msg: Option<Message>,
) -> Element<'static, Message> {
    let header = if hunk.header.is_empty() {
        format!(
            "@@ -{},{} +{},{} @@",
            hunk.old_start, hunk.old_lines, hunk.new_start, hunk.new_lines
        )
    } else {
        hunk.header.clone()
    };

    let mut header_row = Row::new()
        .spacing(4)
        .align_y(Alignment::Center)
        .push(
            Text::new(header)
                .size(10)
                .font(Font::MONOSPACE)
                .wrapping(text::Wrapping::None)
                .color(theme::darcula::TEXT_SECONDARY),
        );

    // Add stage/unstage hunk buttons
    if let Some(msg) = stage_msg {
        header_row = header_row.push(
            iced::widget::Button::new(
                Text::new("暂存区块")
                    .size(9)
                    .color(theme::darcula::STATUS_ADDED),
            )
            .style(theme::button_style(theme::ButtonTone::Ghost))
            .padding([1, 6])
            .on_press(msg),
        );
    }

    if let Some(msg) = unstage_msg {
        header_row = header_row.push(
            iced::widget::Button::new(
                Text::new("取消暂存")
                    .size(9)
                    .color(theme::darcula::STATUS_DELETED),
            )
            .style(theme::button_style(theme::ButtonTone::Ghost))
            .padding([1, 6])
            .on_press(msg),
        );
    }

    Row::new()
        .spacing(0)
        .align_y(Alignment::Center)
        .width(editor_row_width())
        .push(marker_bar(theme::darcula::SEPARATOR.scale_alpha(0.82)))
        .push(gutter_placeholder())
        .push(vertical_separator())
        .push(
            Container::new(header_row)
                .padding([2, 8])
                .height(Length::Fixed(HUNK_HEADER_HEIGHT))
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
        .align_y(Alignment::Center)
        .width(editor_row_width())
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
            .padding([0, 8])
            .height(Length::Fixed(DIFF_ROW_HEIGHT))
            .width(Length::Fixed(GUTTER_WIDTH))
            .style(simple_fill_style(gutter_background)),
        )
        .push(vertical_separator())
        .push(
            Container::new(
                Row::new()
                    .spacing(6)
                    .align_y(Alignment::Center)
                    .width(code_cell_width())
                    .push(
                        Text::new(prefix_for_origin(&line.origin))
                            .size(11)
                            .font(Font::MONOSPACE)
                            .color(prefix_color(&line.origin))
                            .width(Length::Fixed(PREFIX_WIDTH)),
                    )
                    .push(if line.inline_changes.is_empty() {
                        line_highlighter.view_diff_code(&line.origin, &line.content)
                    } else {
                        line_highlighter.view_diff_code_with_inline(
                            &line.origin,
                            &line.content,
                            &line.inline_changes,
                        )
                    }),
            )
            .padding([0, 10])
            .height(Length::Fixed(DIFF_ROW_HEIGHT))
            .width(code_cell_width())
            .style(simple_fill_style(code_background)),
        )
        .into()
}

fn render_empty_editor_row<Message: Clone + 'static>() -> Element<'static, Message> {
    Row::new()
        .spacing(0)
        .width(editor_row_width())
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
            .width(code_cell_width())
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

// IDEA-style hunk divider: more prominent with accent color and shadow
fn editor_divider<Message: Clone + 'static>() -> Element<'static, Message> {
    // Add vertical spacing before divider and use accent color for better visibility
    Column::new()
        .spacing(theme::spacing::XS)
        .width(Length::Fill)
        .push(Space::new().height(Length::Fixed(4.0)))
        .push(
            Container::new(Space::new().width(Length::Fill).height(Length::Fixed(2.0)))
                .style(divider_style()),
        )
        .push(Space::new().height(Length::Fixed(theme::spacing::XS)))
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

// IDEA-style hunk header: more prominent with border and better contrast
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

// IDEA-style divider with accent color for better hunk separation
fn divider_style() -> impl Fn(&Theme) -> container::Style {
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
    use super::{code_cell_width, editor_content_width, editor_row_width};
    use iced::Length;

    #[test]
    fn unified_editor_content_keeps_intrinsic_width_inside_horizontal_scroll() {
        assert_eq!(editor_content_width(), Length::Shrink);
    }

    #[test]
    fn unified_diff_rows_keep_intrinsic_width_to_avoid_code_overlap() {
        assert_eq!(editor_row_width(), Length::Shrink);
        assert_eq!(code_cell_width(), Length::Shrink);
    }
}
