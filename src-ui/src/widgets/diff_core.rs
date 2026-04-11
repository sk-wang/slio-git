//! Shared diff rendering core — Meld-style line alignment, colors, and primitives.
//!
//! Both the unified and split diff viewers delegate to this module for consistent
//! rendering. The key abstraction is `AlignedRow`, which pairs left/right sides
//! using Meld's chunk model (Equal, Insert, Delete, Replace).

use crate::theme;
use crate::widgets::syntax_highlighting::{HighlightedSegment, HunkSyntaxHighlighter};
use git_core::diff::{DiffHunk, DiffLine, DiffLineOrigin, InlineChangeSpan};
use iced::widget::{container, text, Column, Container, Row, Space, Text};
use iced::{Alignment, Background, Border, Color, Element, Length, Theme};

// ── Layout constants ──────────────────────────────────────────────────────

pub const MARKER_WIDTH: f32 = 3.0;
pub const UNIFIED_GUTTER_WIDTH: f32 = 62.0;
pub const SPLIT_GUTTER_WIDTH: f32 = 44.0;
pub const PREFIX_WIDTH: f32 = 14.0;
pub const SEPARATOR_WIDTH: f32 = 1.0;
pub const DIFF_CODE_SIZE: u32 = 13;
pub const DIFF_ROW_HEIGHT: f32 = 26.0;
pub const HUNK_HEADER_HEIGHT: f32 = 24.0;

// ── Meld chunk model ──────────────────────────────────────────────────────

/// Chunk tag following Meld's terminology.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChunkTag {
    Equal,
    Insert,
    Delete,
    Replace,
}

/// One side of an aligned diff row.
#[derive(Debug, Clone)]
pub struct SideLine {
    pub line_number: Option<u32>,
    pub content: String,
    pub origin: DiffLineOrigin,
    pub inline_changes: Vec<InlineChangeSpan>,
}

/// A paired row: left (old) and right (new) sides aligned together.
#[derive(Debug, Clone)]
pub struct AlignedRow {
    pub tag: ChunkTag,
    pub left: Option<SideLine>,
    pub right: Option<SideLine>,
}

/// Build aligned rows from a hunk, pairing deletions with additions as Replace.
pub fn build_aligned_rows(hunk: &DiffHunk) -> Vec<AlignedRow> {
    let mut rows = Vec::new();
    let mut pending_deletions: Vec<&DiffLine> = Vec::new();

    for line in &hunk.lines {
        match line.origin {
            DiffLineOrigin::Deletion => {
                pending_deletions.push(line);
            }
            DiffLineOrigin::Addition => {
                if let Some(del_line) = pending_deletions.first() {
                    // Pair deletion + addition → Replace
                    rows.push(AlignedRow {
                        tag: ChunkTag::Replace,
                        left: Some(side_from_line(del_line)),
                        right: Some(side_from_line(line)),
                    });
                    pending_deletions.remove(0);
                } else {
                    // Pure insertion
                    rows.push(AlignedRow {
                        tag: ChunkTag::Insert,
                        left: None,
                        right: Some(side_from_line(line)),
                    });
                }
            }
            _ => {
                // Flush any remaining unpaired deletions
                flush_deletions(&mut rows, &mut pending_deletions);

                // Context / header line
                rows.push(AlignedRow {
                    tag: ChunkTag::Equal,
                    left: Some(SideLine {
                        line_number: line.old_lineno,
                        content: line.content.clone(),
                        origin: line.origin.clone(),
                        inline_changes: Vec::new(),
                    }),
                    right: Some(SideLine {
                        line_number: line.new_lineno,
                        content: line.content.clone(),
                        origin: line.origin.clone(),
                        inline_changes: Vec::new(),
                    }),
                });
            }
        }
    }

    // Flush trailing deletions
    flush_deletions(&mut rows, &mut pending_deletions);

    rows
}

fn flush_deletions(rows: &mut Vec<AlignedRow>, deletions: &mut Vec<&DiffLine>) {
    for del_line in deletions.drain(..) {
        rows.push(AlignedRow {
            tag: ChunkTag::Delete,
            left: Some(side_from_line(del_line)),
            right: None,
        });
    }
}

fn side_from_line(line: &DiffLine) -> SideLine {
    SideLine {
        line_number: match line.origin {
            DiffLineOrigin::Addition => line.new_lineno,
            _ => line.old_lineno,
        },
        content: line.content.clone(),
        origin: line.origin.clone(),
        inline_changes: line.inline_changes.clone(),
    }
}

// ── Meld-adapted dark-theme colors ────────────────────────────────────────
//
// Meld light-theme reference:
//   insert  fill=#d0ffa3  line=#a5ff4c  (green)
//   delete  fill=#d0ffa3  line=#a5ff4c  (same as insert in Meld!)
//   replace fill=#bdddff  line=#65b2ff  (blue)
//   inline  fill=#8ac2ff              (darker blue accent)
//
// Dark-theme adaptation: lower saturation/brightness, higher alpha overlay.

/// Meld insert/delete green adapted for dark background.
const MELD_INSERT_TINT: Color = Color::from_rgb(0.36, 0.72, 0.22);
/// Meld delete — in dark theme we use a distinct red for clarity.
const MELD_DELETE_TINT: Color = Color::from_rgb(0.72, 0.28, 0.25);
/// Meld replace blue adapted for dark background.
const MELD_REPLACE_TINT: Color = Color::from_rgb(0.30, 0.55, 0.85);
/// Meld chunk border — 1px line at chunk boundaries.
const MELD_INSERT_BORDER: Color = Color::from_rgba(0.36, 0.72, 0.22, 0.50);
const MELD_DELETE_BORDER: Color = Color::from_rgba(0.72, 0.28, 0.25, 0.50);
const MELD_REPLACE_BORDER: Color = Color::from_rgba(0.30, 0.55, 0.85, 0.50);

pub fn mix_colors(base: Color, overlay: Color, amount: f32) -> Color {
    let a = amount.clamp(0.0, 1.0);
    Color {
        r: base.r * (1.0 - a) + overlay.r * a,
        g: base.g * (1.0 - a) + overlay.g * a,
        b: base.b * (1.0 - a) + overlay.b * a,
        a: base.a * (1.0 - a) + overlay.a * a,
    }
}

/// Code area fill background (Meld: rectangle background).
pub fn chunk_code_bg(tag: ChunkTag) -> Color {
    match tag {
        ChunkTag::Equal => theme::darcula::BG_EDITOR,
        ChunkTag::Insert => mix_colors(theme::darcula::BG_EDITOR, MELD_INSERT_TINT, 0.15),
        ChunkTag::Delete => mix_colors(theme::darcula::BG_EDITOR, MELD_DELETE_TINT, 0.15),
        ChunkTag::Replace => mix_colors(theme::darcula::BG_EDITOR, MELD_REPLACE_TINT, 0.15),
    }
}

/// Gutter fill background.
pub fn chunk_gutter_bg(tag: ChunkTag) -> Color {
    match tag {
        ChunkTag::Equal => mix_colors(theme::darcula::BG_EDITOR, theme::darcula::BG_RAISED, 0.28),
        ChunkTag::Insert => mix_colors(theme::darcula::BG_RAISED, MELD_INSERT_TINT, 0.18),
        ChunkTag::Delete => mix_colors(theme::darcula::BG_RAISED, MELD_DELETE_TINT, 0.18),
        ChunkTag::Replace => mix_colors(theme::darcula::BG_RAISED, MELD_REPLACE_TINT, 0.14),
    }
}

/// Chunk border color (Meld: 1px stroke at chunk boundaries).
pub fn chunk_border_color(tag: ChunkTag) -> Color {
    match tag {
        ChunkTag::Equal => Color::TRANSPARENT,
        ChunkTag::Insert => MELD_INSERT_BORDER,
        ChunkTag::Delete => MELD_DELETE_BORDER,
        ChunkTag::Replace => MELD_REPLACE_BORDER,
    }
}

/// Empty/padding half background (dimmed, like Meld's grey for absent lines).
pub fn empty_half_bg() -> Color {
    mix_colors(theme::darcula::BG_EDITOR, theme::darcula::BG_PANEL, 0.35)
}

/// Marker bar color (vertical strip on the left edge).
pub fn marker_color(tag: ChunkTag) -> Color {
    match tag {
        ChunkTag::Equal => Color::TRANSPARENT,
        ChunkTag::Insert => MELD_INSERT_TINT,
        ChunkTag::Delete => MELD_DELETE_TINT,
        ChunkTag::Replace => MELD_REPLACE_TINT,
    }
}

/// Prefix character.
pub fn prefix_char(tag: ChunkTag, is_left: bool) -> &'static str {
    match tag {
        ChunkTag::Equal => " ",
        ChunkTag::Insert => "+",
        ChunkTag::Delete => "-",
        ChunkTag::Replace => {
            if is_left {
                "-"
            } else {
                "+"
            }
        }
    }
}

/// Prefix color.
pub fn prefix_color(tag: ChunkTag, is_left: bool) -> Color {
    match tag {
        ChunkTag::Equal => theme::darcula::TEXT_SECONDARY,
        ChunkTag::Insert => MELD_INSERT_TINT,
        ChunkTag::Delete => MELD_DELETE_TINT,
        ChunkTag::Replace => {
            if is_left {
                MELD_DELETE_TINT
            } else {
                MELD_INSERT_TINT
            }
        }
    }
}

// ── Shared rendering primitives ───────────────────────────────────────────

pub fn marker_bar<'a, Message: 'a>(color: Color) -> Container<'a, Message> {
    Container::new(Space::new().width(Length::Fixed(MARKER_WIDTH)))
        .width(Length::Fixed(MARKER_WIDTH))
        .style(fill_style(color))
}

pub fn vertical_separator<'a, Message: 'a>() -> Container<'a, Message> {
    Container::new(Space::new().width(Length::Fixed(SEPARATOR_WIDTH)))
        .width(Length::Fixed(SEPARATOR_WIDTH))
        .style(fill_style(theme::darcula::SEPARATOR.scale_alpha(0.72)))
}

pub fn center_divider<'a, Message: 'a>() -> Container<'a, Message> {
    Container::new(Space::new().width(Length::Fixed(1.0)))
        .width(Length::Fixed(1.0))
        .style(fill_style(theme::darcula::BORDER.scale_alpha(0.6)))
}

pub fn format_line_number(value: Option<u32>) -> String {
    value
        .map(|n| format!("{n:>4}"))
        .unwrap_or_else(|| "    ".to_string())
}

/// Render a code cell with syntax highlighting and optional inline changes.
pub fn render_code<'a, Message: Clone + 'static>(
    side: &SideLine,
    hl: &mut HunkSyntaxHighlighter,
    bg: Color,
) -> Container<'a, Message> {
    let segments = hl.highlight_segments(&side.origin, &side.content);
    let code: Element<'a, Message> = if side.inline_changes.is_empty() {
        HighlightedSegment::render_diff_code(&segments)
    } else {
        let is_add = matches!(side.origin, DiffLineOrigin::Addition);
        HighlightedSegment::render_diff_code_with_inline(&segments, &side.inline_changes, is_add)
    };

    Container::new(
        Row::new()
            .spacing(0)
            .align_y(Alignment::Center)
            .width(Length::Shrink)
            .push(code),
    )
    .padding([0, 6])
    .height(Length::Fixed(DIFF_ROW_HEIGHT))
    .width(Length::Shrink)
    .style(fill_style(bg))
}

/// Render a unified-mode line (marker + dual gutter + separator + prefix + code).
pub fn render_unified_line<'a, Message: Clone + 'static>(
    line: &DiffLine,
    tag: ChunkTag,
    hl: &mut HunkSyntaxHighlighter,
) -> Element<'a, Message> {
    let gutter_bg = chunk_gutter_bg(tag);
    let code_bg = chunk_code_bg(tag);
    let origin = &line.origin;

    // For unified view, use the origin-based colors for insert/delete (more familiar)
    let actual_code_bg = match origin {
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
        _ => code_bg,
    };
    let actual_gutter_bg = match origin {
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
        _ => gutter_bg,
    };

    let segs = hl.highlight_segments(origin, &line.content);
    let code: Element<'a, Message> = if line.inline_changes.is_empty() {
        HighlightedSegment::render_diff_code(&segs)
    } else {
        let is_add = matches!(origin, DiffLineOrigin::Addition);
        HighlightedSegment::render_diff_code_with_inline(&segs, &line.inline_changes, is_add)
    };

    let m_color = match origin {
        DiffLineOrigin::Addition => theme::darcula::SUCCESS,
        DiffLineOrigin::Deletion => theme::darcula::DANGER,
        DiffLineOrigin::Header | DiffLineOrigin::HunkHeader => {
            theme::darcula::ACCENT.scale_alpha(0.64)
        }
        DiffLineOrigin::Context => Color::TRANSPARENT,
    };

    let pfx = match origin {
        DiffLineOrigin::Addition => "+",
        DiffLineOrigin::Deletion => "-",
        _ => " ",
    };
    let pfx_color = match origin {
        DiffLineOrigin::Addition => theme::darcula::SUCCESS,
        DiffLineOrigin::Deletion => theme::darcula::DANGER,
        _ => theme::darcula::TEXT_SECONDARY,
    };

    Row::new()
        .spacing(0)
        .align_y(Alignment::Center)
        .width(Length::Shrink)
        .push(marker_bar(m_color))
        .push(
            Container::new(
                Row::new()
                    .spacing(6)
                    .align_y(Alignment::Center)
                    .push(
                        Text::new(format_line_number(line.old_lineno))
                            .size(theme::typography::MICRO_SIZE)
                            .font(crate::theme::code_font())
                            .color(theme::darcula::TEXT_SECONDARY)
                            .width(Length::Fixed(28.0)),
                    )
                    .push(
                        Text::new(format_line_number(line.new_lineno))
                            .size(theme::typography::MICRO_SIZE)
                            .font(crate::theme::code_font())
                            .color(theme::darcula::TEXT_SECONDARY)
                            .width(Length::Fixed(28.0)),
                    ),
            )
            .padding([0, 8])
            .height(Length::Fixed(DIFF_ROW_HEIGHT))
            .width(Length::Fixed(UNIFIED_GUTTER_WIDTH))
            .style(fill_style(actual_gutter_bg)),
        )
        .push(vertical_separator())
        .push(
            Container::new(
                Row::new()
                    .spacing(6)
                    .align_y(Alignment::Center)
                    .width(Length::Shrink)
                    .push(
                        Text::new(pfx)
                            .size(DIFF_CODE_SIZE)
                            .font(crate::theme::code_font())
                            .color(pfx_color)
                            .width(Length::Fixed(PREFIX_WIDTH)),
                    )
                    .push(code),
            )
            .padding([0, 10])
            .height(Length::Fixed(DIFF_ROW_HEIGHT))
            .width(Length::Shrink)
            .style(fill_style(actual_code_bg)),
        )
        .into()
}

/// Render one half of a split diff row (gutter + prefix + code).
/// Applies Meld-style chunk fill background.
pub fn render_split_half<'a, Message: Clone + 'static>(
    side: &SideLine,
    tag: ChunkTag,
    is_left: bool,
    hl: &mut HunkSyntaxHighlighter,
) -> Container<'a, Message> {
    let gutter_bg = chunk_gutter_bg(tag);
    let code_bg = chunk_code_bg(tag);

    let segs = hl.highlight_segments(&side.origin, &side.content);
    let code: Element<'a, Message> = if side.inline_changes.is_empty() {
        HighlightedSegment::render_diff_code(&segs)
    } else {
        let is_add = matches!(side.origin, DiffLineOrigin::Addition);
        HighlightedSegment::render_diff_code_with_inline(&segs, &side.inline_changes, is_add)
    };

    let pfx = prefix_char(tag, is_left);
    let pfx_color = prefix_color(tag, is_left);

    // Code content — use styled_editor_horizontal (width=3, scroller=2) which
    // reliably clips overflow. Iced's Scrollable with 0-width scrollbar does NOT clip.
    let clipped_code = crate::widgets::scrollable::styled_editor_horizontal(
        Container::new(code).padding([2, 4]).width(Length::Shrink),
    )
    .width(Length::Fill)
    .height(Length::Fixed(DIFF_ROW_HEIGHT));

    Container::new(
        Row::new()
            .spacing(0)
            .align_y(Alignment::Center)
            .push(
                Container::new(
                    Text::new(format_line_number(side.line_number))
                        .size(theme::typography::MICRO_SIZE)
                        .font(crate::theme::code_font())
                        .color(theme::darcula::TEXT_DISABLED),
                )
                .width(Length::Fixed(SPLIT_GUTTER_WIDTH))
                .height(Length::Fixed(DIFF_ROW_HEIGHT))
                .padding([2, 6])
                .style(fill_style(gutter_bg)),
            )
            .push(
                Text::new(pfx)
                    .size(DIFF_CODE_SIZE)
                    .font(crate::theme::code_font())
                    .color(pfx_color)
                    .width(Length::Fixed(PREFIX_WIDTH)),
            )
            .push(clipped_code),
    )
    .height(Length::Fixed(DIFF_ROW_HEIGHT))
    .style(fill_style(code_bg))
}

/// Render an empty padding half (for the side without content).
/// Uses dimmed background with subtle tint matching the chunk type.
pub fn render_empty_half<'a, Message: 'a>(tag: ChunkTag) -> Container<'a, Message> {
    let bg = match tag {
        ChunkTag::Insert => mix_colors(empty_half_bg(), MELD_INSERT_TINT, 0.05),
        ChunkTag::Delete => mix_colors(empty_half_bg(), MELD_DELETE_TINT, 0.05),
        _ => empty_half_bg(),
    };
    Container::new(Space::new())
        .height(Length::Fixed(DIFF_ROW_HEIGHT))
        .style(fill_style(bg))
}

/// Render a 1px chunk boundary line (Meld draws these at chunk top/bottom).
pub fn chunk_boundary<'a, Message: 'a>(tag: ChunkTag) -> Element<'a, Message> {
    let color = chunk_border_color(tag);
    Container::new(Space::new())
        .height(Length::Fixed(1.0))
        .width(Length::Fill)
        .style(fill_style(color))
        .into()
}

/// Hunk header row (shared between both viewers).
pub fn hunk_header<'a, Message: Clone + 'static>(
    hunk: &DiffHunk,
    stage_msg: Option<Message>,
    unstage_msg: Option<Message>,
) -> Element<'a, Message> {
    let header_text = if hunk.header.is_empty() {
        format!(
            "@@ -{},{} +{},{} @@",
            hunk.old_start, hunk.old_lines, hunk.new_start, hunk.new_lines
        )
    } else {
        hunk.header.clone()
    };

    let mut row = Row::new().spacing(4).align_y(Alignment::Center).push(
        Text::new(header_text)
            .size(DIFF_CODE_SIZE)
            .font(crate::theme::code_font())
            .wrapping(text::Wrapping::None)
            .color(theme::darcula::TEXT_SECONDARY),
    );

    if let Some(msg) = stage_msg {
        row = row.push(
            iced::widget::Button::new(
                Text::new("Stage Hunk")
                    .size(theme::typography::MICRO_SIZE)
                    .color(theme::darcula::STATUS_ADDED),
            )
            .style(theme::button_style(theme::ButtonTone::Ghost))
            .padding([1, 6])
            .on_press(msg),
        );
    }

    if let Some(msg) = unstage_msg {
        row = row.push(
            iced::widget::Button::new(
                Text::new("Unstage")
                    .size(theme::typography::MICRO_SIZE)
                    .color(theme::darcula::STATUS_DELETED),
            )
            .style(theme::button_style(theme::ButtonTone::Ghost))
            .padding([1, 6])
            .on_press(msg),
        );
    }

    Container::new(row)
        .padding([2, 8])
        .height(Length::Fixed(HUNK_HEADER_HEIGHT))
        .width(Length::Fill)
        .style(hunk_header_style())
        .into()
}

/// Hunk divider between hunks.
pub fn hunk_divider<'a, Message: 'a>() -> Element<'a, Message> {
    Column::new()
        .spacing(0)
        .width(Length::Fill)
        .push(Space::new().height(Length::Fixed(4.0)))
        .push(
            Container::new(Space::new().width(Length::Fill).height(Length::Fixed(2.0))).style(
                |_| container::Style {
                    background: Some(Background::Color(
                        theme::darcula::ACCENT_WEAK.scale_alpha(0.8),
                    )),
                    shadow: iced::Shadow {
                        color: theme::darcula::ACCENT.scale_alpha(0.15),
                        offset: iced::Vector::new(0.0, 1.0),
                        blur_radius: 4.0,
                    },
                    ..Default::default()
                },
            ),
        )
        .push(Space::new().height(Length::Fixed(4.0)))
        .into()
}

/// Empty editor placeholder.
pub fn empty_editor_row<'a, Message: 'a>(label: &str) -> Element<'a, Message> {
    Container::new(
        Text::new(label.to_string())
            .size(DIFF_CODE_SIZE)
            .color(theme::darcula::TEXT_SECONDARY),
    )
    .padding([8, 10])
    .width(Length::Fill)
    .style(fill_style(theme::darcula::BG_EDITOR))
    .into()
}

// ── Style helpers ─────────────────────────────────────────────────────────

pub fn fill_style(color: Color) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(Background::Color(color)),
        ..Default::default()
    }
}

pub fn editor_surface_style() -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
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
    move |_| container::Style {
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

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use git_core::diff::{DiffHunk, DiffLine, DiffLineOrigin};

    fn make_line(
        origin: DiffLineOrigin,
        content: &str,
        old: Option<u32>,
        new: Option<u32>,
    ) -> DiffLine {
        DiffLine {
            origin,
            content: content.to_string(),
            old_lineno: old,
            new_lineno: new,
            inline_changes: Vec::new(),
        }
    }

    fn test_hunk(lines: Vec<DiffLine>) -> DiffHunk {
        DiffHunk {
            header: String::new(),
            old_start: 1,
            old_lines: 0,
            new_start: 1,
            new_lines: 0,
            lines,
        }
    }

    #[test]
    fn context_lines_become_equal() {
        let hunk = test_hunk(vec![make_line(
            DiffLineOrigin::Context,
            "hello",
            Some(1),
            Some(1),
        )]);
        let rows = build_aligned_rows(&hunk);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].tag, ChunkTag::Equal);
        assert!(rows[0].left.is_some());
        assert!(rows[0].right.is_some());
    }

    #[test]
    fn pure_addition_becomes_insert() {
        let hunk = test_hunk(vec![make_line(
            DiffLineOrigin::Addition,
            "new line",
            None,
            Some(1),
        )]);
        let rows = build_aligned_rows(&hunk);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].tag, ChunkTag::Insert);
        assert!(rows[0].left.is_none());
        assert!(rows[0].right.is_some());
    }

    #[test]
    fn pure_deletion_becomes_delete() {
        let hunk = test_hunk(vec![make_line(
            DiffLineOrigin::Deletion,
            "old line",
            Some(1),
            None,
        )]);
        let rows = build_aligned_rows(&hunk);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].tag, ChunkTag::Delete);
        assert!(rows[0].left.is_some());
        assert!(rows[0].right.is_none());
    }

    #[test]
    fn paired_deletion_addition_becomes_replace() {
        let hunk = test_hunk(vec![
            make_line(DiffLineOrigin::Deletion, "old", Some(1), None),
            make_line(DiffLineOrigin::Addition, "new", None, Some(1)),
        ]);
        let rows = build_aligned_rows(&hunk);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].tag, ChunkTag::Replace);
        assert!(rows[0].left.is_some());
        assert!(rows[0].right.is_some());
    }

    #[test]
    fn uneven_delete_add_produces_replace_plus_insert() {
        let hunk = test_hunk(vec![
            make_line(DiffLineOrigin::Deletion, "old1", Some(1), None),
            make_line(DiffLineOrigin::Addition, "new1", None, Some(1)),
            make_line(DiffLineOrigin::Addition, "new2", None, Some(2)),
        ]);
        let rows = build_aligned_rows(&hunk);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].tag, ChunkTag::Replace);
        assert_eq!(rows[1].tag, ChunkTag::Insert);
    }

    #[test]
    fn uneven_more_deletes_produces_replace_plus_delete() {
        let hunk = test_hunk(vec![
            make_line(DiffLineOrigin::Deletion, "old1", Some(1), None),
            make_line(DiffLineOrigin::Deletion, "old2", Some(2), None),
            make_line(DiffLineOrigin::Addition, "new1", None, Some(1)),
        ]);
        let rows = build_aligned_rows(&hunk);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].tag, ChunkTag::Replace);
        assert_eq!(rows[1].tag, ChunkTag::Delete);
    }

    #[test]
    fn mixed_context_and_changes() {
        let hunk = test_hunk(vec![
            make_line(DiffLineOrigin::Context, "ctx1", Some(1), Some(1)),
            make_line(DiffLineOrigin::Deletion, "old", Some(2), None),
            make_line(DiffLineOrigin::Addition, "new", None, Some(2)),
            make_line(DiffLineOrigin::Context, "ctx2", Some(3), Some(3)),
        ]);
        let rows = build_aligned_rows(&hunk);
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].tag, ChunkTag::Equal);
        assert_eq!(rows[1].tag, ChunkTag::Replace);
        assert_eq!(rows[2].tag, ChunkTag::Equal);
    }
}
