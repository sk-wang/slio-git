//! Editor-backed split diff adapter with Meld-style shell primitives.

use crate::theme;
use crate::widgets::diff_core;
use git_core::diff::{
    EditorDiffBlock, EditorDiffBlockKind, EditorDiffHunk, EditorDiffLine, EditorDiffModel,
    EditorLineMapEntry, InlineChangeSpan,
};
use iced::widget::canvas::{self, Canvas, Frame};
use iced::widget::{Container, Row, Stack};
use iced::{mouse, Element, Length, Point, Rectangle, Renderer, Size, Theme};
use iced_code_editor::{CodeEditor, Message as EditorMessage};
use std::cell::Cell;
use std::ops::Range;
use std::path::Path;
use std::sync::Arc;
use unicode_width::UnicodeWidthChar;

const CODE_PADDING_X: f32 = 5.0;
const LINK_MAP_WIDTH: f32 = 32.0;
const OVERVIEW_WIDTH: f32 = 18.0;
const OVERVIEW_PADDING_Y: f32 = 6.0;
const MIN_EMPTY_BLOCK_HEIGHT: f32 = 6.0;
const MIN_OVERVIEW_BLOCK_HEIGHT: f32 = 2.0;
const HUNK_NAV_SYNC_POINT: f32 = 0.0;
const OVERVIEW_SYNC_POINT: f32 = 0.5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffEditorPane {
    Left,
    Right,
}

#[derive(Debug, Clone)]
pub enum DiffEditorEvent {
    Editor {
        pane: DiffEditorPane,
        message: EditorMessage,
    },
    JumpToOverviewFraction(f32),
    SelectVisibleHunk(usize),
}

pub struct SplitDiffEditorState {
    model: EditorDiffModel,
    left: CodeEditor,
    right: CodeEditor,
    left_decorations: Arc<PaneDecorations>,
    right_decorations: Arc<PaneDecorations>,
    link_blocks: Arc<[LinkMapBlock]>,
    overview_blocks: Arc<[OverviewBlock]>,
    overview_hunks: Arc<[OverviewHunk]>,
    left_line_count: usize,
    right_line_count: usize,
    current_hunk_index: Option<usize>,
    suppress_sync: [bool; 2],
}

impl std::fmt::Debug for SplitDiffEditorState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SplitDiffEditorState")
            .field("old_path", &self.model.old_path)
            .field("new_path", &self.model.new_path)
            .field("hunks", &self.model.hunks.len())
            .finish()
    }
}

impl Clone for SplitDiffEditorState {
    fn clone(&self) -> Self {
        Self::new(self.model.clone())
    }
}

impl SplitDiffEditorState {
    pub fn new(model: EditorDiffModel) -> Self {
        Self::with_font_size(model, DEFAULT_EDITOR_FONT_SIZE)
    }

    pub fn with_font_size(model: EditorDiffModel, font_size: f32) -> Self {
        let left = build_editor_with_font_size(
            &model.left_text,
            model.old_path.as_deref().or(model.new_path.as_deref()),
            font_size,
        );
        let right = build_editor_with_font_size(
            &model.right_text,
            model.new_path.as_deref().or(model.old_path.as_deref()),
            font_size,
        );
        let left_line_count = logical_line_count(&model.left_text);
        let right_line_count = logical_line_count(&model.right_text);
        let (left_decorations, right_decorations) = build_pane_decorations(&model);
        let link_blocks = build_link_blocks(&model);
        let overview_blocks = build_overview_blocks(&link_blocks);
        let overview_hunks = build_overview_hunks(&model);

        Self {
            model,
            left,
            right,
            left_decorations: Arc::new(left_decorations),
            right_decorations: Arc::new(right_decorations),
            link_blocks: Arc::from(link_blocks),
            overview_blocks: Arc::from(overview_blocks),
            overview_hunks: Arc::from(overview_hunks),
            left_line_count,
            right_line_count,
            current_hunk_index: None,
            suppress_sync: [false, false],
        }
    }

    pub fn reset(&mut self, model: EditorDiffModel) -> iced::Task<DiffEditorEvent> {
        *self = Self::new(model);
        iced::Task::none()
    }

    pub fn model(&self) -> &EditorDiffModel {
        &self.model
    }

    pub fn update(
        &mut self,
        event: DiffEditorEvent,
    ) -> (iced::Task<DiffEditorEvent>, Option<usize>) {
        match event {
            DiffEditorEvent::Editor { pane, message } => self.handle_editor_event(pane, message),
            DiffEditorEvent::JumpToOverviewFraction(fraction) => {
                self.handle_overview_jump(fraction.clamp(0.0, 1.0))
            }
            DiffEditorEvent::SelectVisibleHunk(hunk_index) => {
                let changed = self.emit_current_hunk_change(Some(hunk_index));
                (self.scroll_to_hunk(hunk_index), changed)
            }
        }
    }

    pub fn scroll_to_hunk(&mut self, hunk_index: usize) -> iced::Task<DiffEditorEvent> {
        let Some(hunk) = self.model.hunks.get(hunk_index) else {
            return iced::Task::none();
        };
        self.current_hunk_index = Some(hunk_index);

        self.scroll_to_anchor_lines(
            hunk_anchor_line(hunk, DiffEditorPane::Left),
            hunk_anchor_line(hunk, DiffEditorPane::Right),
            HUNK_NAV_SYNC_POINT,
        )
    }

    pub fn view(&self, selected_hunk_index: Option<usize>) -> Element<'_, DiffEditorEvent> {
        let selected_hunk_index = selected_hunk_index.or(self.current_hunk_index);

        Row::new()
            .spacing(0)
            .width(Length::Fill)
            .height(Length::Fill)
            .push(
                self.pane_view(DiffEditorPane::Left, selected_hunk_index)
                    .width(Length::FillPortion(8)),
            )
            .push(self.link_map_view(selected_hunk_index))
            .push(diff_core::center_divider())
            .push(
                self.pane_view(DiffEditorPane::Right, selected_hunk_index)
                    .width(Length::FillPortion(8)),
            )
            .push(self.overview_map_view(selected_hunk_index))
            .into()
    }

    fn handle_editor_event(
        &mut self,
        pane: DiffEditorPane,
        message: EditorMessage,
    ) -> (iced::Task<DiffEditorEvent>, Option<usize>) {
        let pane_idx = pane_index(pane);
        let should_skip_sync =
            matches!(message, EditorMessage::Scrolled(_)) && self.suppress_sync[pane_idx];
        if should_skip_sync {
            self.suppress_sync[pane_idx] = false;
        }

        let local_task = if is_mutating_message(&message) {
            iced::Task::none()
        } else {
            self.editor_mut(pane)
                .update(&message)
                .map(move |editor_message| DiffEditorEvent::Editor {
                    pane,
                    message: editor_message,
                })
        };

        let sync_task = match &message {
            EditorMessage::Scrolled(viewport) if !should_skip_sync => {
                self.synced_scroll_task(pane, viewport.absolute_offset().y)
            }
            _ => iced::Task::none(),
        };

        let current_hunk = match &message {
            EditorMessage::Scrolled(viewport) => self.emit_current_hunk_change(Some(
                self.current_hunk_for_scroll(pane, viewport.absolute_offset().y),
            )),
            _ => None,
        };

        (iced::Task::batch([local_task, sync_task]), current_hunk)
    }

    fn handle_overview_jump(
        &mut self,
        fraction: f32,
    ) -> (iced::Task<DiffEditorEvent>, Option<usize>) {
        let overview_pane = self.overview_pane();
        let source_total_lines = self.line_count(overview_pane);
        if source_total_lines == 0 {
            return (iced::Task::none(), None);
        }

        let source_anchor = anchor_line_for_fraction(fraction, source_total_lines);
        let target_pane = opposite_pane(overview_pane);
        let target_anchor = self
            .map_anchor_between_panes(overview_pane, source_anchor)
            .unwrap_or_else(|| {
                scale_anchor_line(
                    source_anchor,
                    source_total_lines,
                    self.line_count(target_pane),
                )
            });

        let task = match overview_pane {
            DiffEditorPane::Left => {
                self.scroll_to_anchor_lines(source_anchor, target_anchor, OVERVIEW_SYNC_POINT)
            }
            DiffEditorPane::Right => {
                self.scroll_to_anchor_lines(target_anchor, source_anchor, OVERVIEW_SYNC_POINT)
            }
        };

        let current_hunk = self.emit_current_hunk_change(hunk_for_overview_fraction(
            &self.model.hunks,
            overview_pane,
            fraction,
            source_total_lines,
        ));

        (task, current_hunk)
    }

    fn pane_view(
        &self,
        pane: DiffEditorPane,
        selected_hunk_index: Option<usize>,
    ) -> Container<'_, DiffEditorEvent> {
        let editor = self.editor(pane);
        let selected_hunk_index = selected_hunk_index.or(self.current_hunk_index);
        let decorations = match pane {
            DiffEditorPane::Left => Arc::clone(&self.left_decorations),
            DiffEditorPane::Right => Arc::clone(&self.right_decorations),
        };

        let active_range = selected_hunk_index
            .and_then(|index| self.model.hunks.get(index))
            .map(|hunk| hunk_range_for_pane(hunk, pane));

        let background = Canvas::new(DiffDecorationCanvas {
            decorations,
            viewport_scroll: editor.viewport_scroll(),
            horizontal_scroll_offset: editor.horizontal_scroll_offset(),
            line_height: editor.line_height(),
            viewport_height: editor.viewport_height(),
            gutter_width: editor.gutter_width(),
            char_width: editor.char_width(),
            full_char_width: editor.full_char_width(),
            active_range,
        })
        .width(Length::Fill)
        .height(Length::Fill);

        let editor_view = editor
            .view()
            .map(move |editor_message| DiffEditorEvent::Editor {
                pane,
                message: editor_message,
            });

        Container::new(Stack::new().push(background).push(editor_view))
            .width(Length::Fill)
            .height(Length::Fill)
    }

    fn link_map_view(&self, selected_hunk_index: Option<usize>) -> Element<'_, DiffEditorEvent> {
        Canvas::new(LinkMapCanvas {
            blocks: Arc::clone(&self.link_blocks),
            current_hunk: selected_hunk_index,
            left_scroll: self.left.viewport_scroll(),
            right_scroll: self.right.viewport_scroll(),
            left_line_height: self.left.line_height(),
            right_line_height: self.right.line_height(),
        })
        .width(Length::Fixed(LINK_MAP_WIDTH))
        .height(Length::Fill)
        .into()
    }

    fn overview_map_view(
        &self,
        selected_hunk_index: Option<usize>,
    ) -> Element<'_, DiffEditorEvent> {
        Canvas::new(OverviewMapCanvas {
            blocks: Arc::clone(&self.overview_blocks),
            hunks: Arc::clone(&self.overview_hunks),
            current_hunk: selected_hunk_index,
            total_lines: self.line_count(self.overview_pane()),
            viewport_range: self.overview_viewport_range(),
        })
        .width(Length::Fixed(OVERVIEW_WIDTH))
        .height(Length::Fill)
        .into()
    }

    fn synced_scroll_task(
        &mut self,
        pane: DiffEditorPane,
        source_scroll: f32,
    ) -> iced::Task<DiffEditorEvent> {
        let target_pane = opposite_pane(pane);
        let Some(target_scroll) = self.synced_scroll_target(pane, source_scroll) else {
            return iced::Task::none();
        };

        let current_target_scroll = self.editor(target_pane).viewport_scroll();
        if (current_target_scroll - target_scroll).abs() <= 0.5 {
            return iced::Task::none();
        }

        self.suppress_sync[pane_index(target_pane)] = true;
        self.editor(target_pane)
            .scroll_to_offset(None, Some(target_scroll))
            .map(move |editor_message| DiffEditorEvent::Editor {
                pane: target_pane,
                message: editor_message,
            })
    }

    fn synced_scroll_target(&self, pane: DiffEditorPane, source_scroll: f32) -> Option<f32> {
        let source_editor = self.editor(pane);
        let source_line_height = source_editor.line_height();
        if source_line_height <= 0.0 {
            return None;
        }

        let source_total_lines = self.line_count(pane);
        let source_sync_point = calc_sync_point(
            source_scroll,
            source_editor.viewport_height(),
            content_height(source_total_lines, source_line_height),
        );
        let source_anchor = anchor_line_for_scroll(
            source_scroll,
            source_sync_point,
            source_editor.viewport_height(),
            source_line_height,
        );

        let target_pane = opposite_pane(pane);
        let target_anchor = self
            .map_anchor_between_panes(pane, source_anchor)
            .unwrap_or_else(|| {
                scale_anchor_line(
                    source_anchor,
                    source_total_lines,
                    self.line_count(target_pane),
                )
            });

        let target_editor = self.editor(target_pane);
        Some(scroll_for_anchor_line(
            target_anchor,
            source_sync_point,
            target_editor.viewport_height(),
            target_editor.line_height(),
            self.line_count(target_pane),
        ))
    }

    fn map_anchor_between_panes(&self, pane: DiffEditorPane, source_anchor: f32) -> Option<f32> {
        map_anchor_line(&self.model.hunks, pane, source_anchor)
            .or_else(|| map_anchor_line_from_line_map(&self.model.line_map, pane, source_anchor))
    }

    fn scroll_to_anchor_lines(
        &mut self,
        left_anchor: f32,
        right_anchor: f32,
        sync_point: f32,
    ) -> iced::Task<DiffEditorEvent> {
        let left_scroll = scroll_for_anchor_line(
            left_anchor,
            sync_point,
            self.left.viewport_height(),
            self.left.line_height(),
            self.left_line_count,
        );
        let right_scroll = scroll_for_anchor_line(
            right_anchor,
            sync_point,
            self.right.viewport_height(),
            self.right.line_height(),
            self.right_line_count,
        );

        let left_task = if (self.left.viewport_scroll() - left_scroll).abs() > 0.5 {
            self.suppress_sync[pane_index(DiffEditorPane::Left)] = true;
            self.left
                .scroll_to_offset(None, Some(left_scroll))
                .map(|editor_message| DiffEditorEvent::Editor {
                    pane: DiffEditorPane::Left,
                    message: editor_message,
                })
        } else {
            iced::Task::none()
        };
        let right_task = if (self.right.viewport_scroll() - right_scroll).abs() > 0.5 {
            self.suppress_sync[pane_index(DiffEditorPane::Right)] = true;
            self.right
                .scroll_to_offset(None, Some(right_scroll))
                .map(|editor_message| DiffEditorEvent::Editor {
                    pane: DiffEditorPane::Right,
                    message: editor_message,
                })
        } else {
            iced::Task::none()
        };

        iced::Task::batch([left_task, right_task])
    }

    fn current_hunk_for_scroll(&self, pane: DiffEditorPane, scroll: f32) -> usize {
        let editor = self.editor(pane);
        let anchor = anchor_line_for_scroll(
            scroll,
            calc_sync_point(
                scroll,
                editor.viewport_height(),
                content_height(self.line_count(pane), editor.line_height()),
            ),
            editor.viewport_height(),
            editor.line_height().max(1.0),
        );

        current_hunk_from_anchor(&self.model.hunks, pane, anchor).unwrap_or_default()
    }

    fn overview_pane(&self) -> DiffEditorPane {
        if self.right_line_count >= self.left_line_count {
            DiffEditorPane::Right
        } else {
            DiffEditorPane::Left
        }
    }

    fn overview_viewport_range(&self) -> Range<f32> {
        let pane = self.overview_pane();
        let editor = self.editor(pane);
        let total_height = content_height(self.line_count(pane), editor.line_height()).max(1.0);
        let start = (editor.viewport_scroll() / total_height).clamp(0.0, 1.0);
        let end = ((editor.viewport_scroll() + editor.viewport_height()) / total_height)
            .clamp(start, 1.0);
        start..end
    }

    fn line_count(&self, pane: DiffEditorPane) -> usize {
        match pane {
            DiffEditorPane::Left => self.left_line_count.max(1),
            DiffEditorPane::Right => self.right_line_count.max(1),
        }
    }

    fn editor(&self, pane: DiffEditorPane) -> &CodeEditor {
        match pane {
            DiffEditorPane::Left => &self.left,
            DiffEditorPane::Right => &self.right,
        }
    }

    fn editor_mut(&mut self, pane: DiffEditorPane) -> &mut CodeEditor {
        match pane {
            DiffEditorPane::Left => &mut self.left,
            DiffEditorPane::Right => &mut self.right,
        }
    }

    fn emit_current_hunk_change(&mut self, next: Option<usize>) -> Option<usize> {
        if self.current_hunk_index == next {
            None
        } else {
            self.current_hunk_index = next;
            next
        }
    }
}

#[derive(Debug, Clone)]
struct PaneDecorations {
    lines: Vec<Option<DecoratedLine>>,
}

#[derive(Debug, Clone)]
struct DecoratedLine {
    kind: EditorDiffBlockKind,
    content: String,
    inline_changes: Vec<InlineChangeSpan>,
}

#[derive(Debug, Clone)]
struct LinkMapBlock {
    hunk_id: usize,
    kind: EditorDiffBlockKind,
    left_range: Range<usize>,
    right_range: Range<usize>,
}

#[derive(Debug, Clone)]
struct OverviewBlock {
    kind: EditorDiffBlockKind,
    range: Range<usize>,
}

#[derive(Debug, Clone)]
struct OverviewHunk {
    id: usize,
    range: Range<usize>,
}

/// Cache key for decoration canvas — quantized to avoid thrashing on sub-pixel scroll.
#[derive(Debug, Default)]
struct DecorationCacheState {
    cache: canvas::Cache<Renderer>,
    key: Cell<Option<DecorationCacheKey>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DecorationCacheKey {
    scroll_q: i32, // scroll quantized to 2px
    h_scroll_q: i32,
    active_start: i32,
    active_end: i32,
    bounds_w: i32,
    bounds_h: i32,
}

#[derive(Debug, Clone)]
struct DiffDecorationCanvas {
    decorations: Arc<PaneDecorations>,
    viewport_scroll: f32,
    horizontal_scroll_offset: f32,
    line_height: f32,
    viewport_height: f32,
    gutter_width: f32,
    char_width: f32,
    full_char_width: f32,
    active_range: Option<Range<usize>>,
}

impl DiffDecorationCanvas {
    fn cache_key(&self, bounds: Rectangle) -> DecorationCacheKey {
        DecorationCacheKey {
            scroll_q: (self.viewport_scroll * 0.5).round() as i32,
            h_scroll_q: (self.horizontal_scroll_offset * 0.5).round() as i32,
            active_start: self
                .active_range
                .as_ref()
                .map(|r| r.start as i32)
                .unwrap_or(-1),
            active_end: self
                .active_range
                .as_ref()
                .map(|r| r.end as i32)
                .unwrap_or(-1),
            bounds_w: bounds.width as i32,
            bounds_h: bounds.height as i32,
        }
    }
}

impl<Message> canvas::Program<Message> for DiffDecorationCanvas {
    type State = DecorationCacheState;

    fn draw(
        &self,
        state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let key = self.cache_key(bounds);
        if state.key.get() != Some(key) {
            state.cache.clear();
            state.key.set(Some(key));
        }

        let geometry = state.cache.draw(renderer, bounds.size(), |frame| {
            let gutter_width = self.gutter_width.min(bounds.width);
            let code_width = (bounds.width - gutter_width).max(0.0);

            frame.fill_rectangle(
                Point::ORIGIN,
                Size::new(gutter_width, bounds.height),
                diff_core::chunk_gutter_bg(diff_core::ChunkTag::Equal),
            );
            frame.fill_rectangle(
                Point::new(gutter_width, 0.0),
                Size::new(code_width, bounds.height),
                theme::darcula::BG_EDITOR,
            );

            if self.line_height <= 0.0 {
                return;
            }

            let start_line = (self.viewport_scroll / self.line_height).floor().max(0.0) as usize;
            let end_line = ((self.viewport_scroll + self.viewport_height) / self.line_height)
                .ceil()
                .max(0.0) as usize
                + 1;

            for line_index in start_line..end_line.min(self.decorations.lines.len()) {
                let Some(line) = self
                    .decorations
                    .lines
                    .get(line_index)
                    .and_then(|line| line.as_ref())
                else {
                    continue;
                };

                let (code_bg, gutter_bg) = block_colors(line.kind);
                let y = line_index as f32 * self.line_height - self.viewport_scroll;
                let is_active = self
                    .active_range
                    .as_ref()
                    .is_some_and(|range| line_in_range(range, line_index as f32));

                frame.fill_rectangle(
                    Point::new(0.0, y),
                    Size::new(gutter_width, self.line_height),
                    gutter_bg,
                );
                frame.fill_rectangle(
                    Point::new(gutter_width, y),
                    Size::new(code_width, self.line_height),
                    code_bg,
                );

                if is_active {
                    frame.fill_rectangle(
                        Point::new(0.0, y),
                        Size::new(gutter_width, self.line_height),
                        theme::darcula::SELECTION_BG.scale_alpha(0.18),
                    );
                    frame.fill_rectangle(
                        Point::new(gutter_width, y),
                        Size::new(code_width, self.line_height),
                        theme::darcula::SELECTION_BG.scale_alpha(0.10),
                    );
                }

                for inline in line.inline_changes.iter().filter(|span| span.changed) {
                    let end = inline.start + inline.len;
                    let prefix = line.content.get(..inline.start).unwrap_or_default();
                    let changed = line.content.get(inline.start..end).unwrap_or_default();
                    if changed.is_empty() {
                        continue;
                    }

                    let inline_x = gutter_width + CODE_PADDING_X - self.horizontal_scroll_offset
                        + measure_text_width(prefix, self.full_char_width, self.char_width);
                    let inline_width =
                        measure_text_width(changed, self.full_char_width, self.char_width);
                    if inline_width <= 0.0 {
                        continue;
                    }

                    frame.fill_rectangle(
                        Point::new(inline_x, y + 2.0),
                        Size::new(inline_width, (self.line_height - 4.0).max(1.0)),
                        inline_color(line.kind),
                    );
                }
            }
        });

        vec![geometry]
    }
}

#[derive(Debug, Default)]
struct LinkMapCacheState {
    cache: canvas::Cache<Renderer>,
    key: Cell<Option<LinkMapCacheKey>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct LinkMapCacheKey {
    left_scroll_q: i32,
    right_scroll_q: i32,
    current_hunk: Option<usize>,
}

#[derive(Debug, Clone)]
struct LinkMapCanvas {
    blocks: Arc<[LinkMapBlock]>,
    current_hunk: Option<usize>,
    left_scroll: f32,
    right_scroll: f32,
    left_line_height: f32,
    right_line_height: f32,
}

impl<Message> canvas::Program<Message> for LinkMapCanvas {
    type State = LinkMapCacheState;

    fn draw(
        &self,
        state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let key = LinkMapCacheKey {
            left_scroll_q: (self.left_scroll * 0.5).round() as i32,
            right_scroll_q: (self.right_scroll * 0.5).round() as i32,
            current_hunk: self.current_hunk,
        };
        if state.key.get() != Some(key) {
            state.cache.clear();
            state.key.set(Some(key));
        }

        let geometry = state.cache.draw(renderer, bounds.size(), |frame| {
            frame.fill_rectangle(
                Point::ORIGIN,
                bounds.size(),
                theme::darcula::BG_PANEL.scale_alpha(0.92),
            );

            for block in self.blocks.iter() {
                let left = block_visual_bounds(
                    &block.left_range,
                    self.left_line_height,
                    self.left_scroll,
                    MIN_EMPTY_BLOCK_HEIGHT,
                );
                let right = block_visual_bounds(
                    &block.right_range,
                    self.right_line_height,
                    self.right_scroll,
                    MIN_EMPTY_BLOCK_HEIGHT,
                );

                if left.end < 0.0
                    || right.end < 0.0
                    || left.start > bounds.height
                    || right.start > bounds.height
                {
                    continue;
                }

                let path = canvas::Path::new(|builder| {
                    builder.move_to(Point::new(0.0, left.start));
                    builder.bezier_curve_to(
                        Point::new(bounds.width * 0.35, left.start),
                        Point::new(bounds.width * 0.65, right.start),
                        Point::new(bounds.width, right.start),
                    );
                    builder.line_to(Point::new(bounds.width, right.end));
                    builder.bezier_curve_to(
                        Point::new(bounds.width * 0.65, right.end),
                        Point::new(bounds.width * 0.35, left.end),
                        Point::new(0.0, left.end),
                    );
                    builder.close();
                });

                let fill = link_map_fill(block.kind, self.current_hunk == Some(block.hunk_id));
                frame.fill(&path, fill);
                frame.stroke(
                    &path,
                    canvas::Stroke::default()
                        .with_width(if self.current_hunk == Some(block.hunk_id) {
                            1.3
                        } else {
                            0.8
                        })
                        .with_color(link_map_stroke(block.kind)),
                );
            }
        });

        vec![geometry]
    }
}

#[derive(Debug, Default)]
struct OverviewMapState {
    dragging: bool,
    static_cache: canvas::Cache<Renderer>,
    static_key: Cell<Option<(usize, usize, usize)>>,
}

#[derive(Debug, Clone)]
struct OverviewMapCanvas {
    blocks: Arc<[OverviewBlock]>,
    hunks: Arc<[OverviewHunk]>,
    current_hunk: Option<usize>,
    total_lines: usize,
    viewport_range: Range<f32>,
}

impl canvas::Program<DiffEditorEvent> for OverviewMapCanvas {
    type State = OverviewMapState;

    fn update(
        &self,
        state: &mut Self::State,
        event: &canvas::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<DiffEditorEvent>> {
        let cursor_y = cursor.position_in(bounds).map(|position| position.y);

        match event {
            canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let y = cursor_y?;
                state.dragging = true;
                Some(
                    canvas::Action::publish(DiffEditorEvent::JumpToOverviewFraction(
                        overview_fraction(y, bounds.height),
                    ))
                    .and_capture(),
                )
            }
            canvas::Event::Mouse(mouse::Event::CursorMoved { .. }) if state.dragging => {
                let y = cursor_y?;
                Some(
                    canvas::Action::publish(DiffEditorEvent::JumpToOverviewFraction(
                        overview_fraction(y, bounds.height),
                    ))
                    .and_capture(),
                )
            }
            canvas::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
                if state.dragging =>
            {
                state.dragging = false;
                Some(canvas::Action::capture())
            }
            canvas::Event::Mouse(mouse::Event::CursorLeft) if state.dragging => {
                state.dragging = false;
                Some(canvas::Action::capture())
            }
            _ => None,
        }
    }

    fn draw(
        &self,
        state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let drawable_height = (bounds.height - OVERVIEW_PADDING_Y * 2.0).max(1.0);
        let track_x = 2.0;
        let track_width = (bounds.width - 4.0).max(1.0);
        let static_key = (
            self.blocks.as_ptr() as usize,
            self.blocks.len(),
            self.total_lines,
        );
        if state.static_key.get() != Some(static_key) {
            state.static_cache.clear();
            state.static_key.set(Some(static_key));
        }

        let base = state.static_cache.draw(renderer, bounds.size(), |frame| {
            frame.fill_rectangle(
                Point::ORIGIN,
                bounds.size(),
                theme::darcula::BG_PANEL.scale_alpha(0.94),
            );

            for block in self.blocks.iter() {
                let rect = overview_rect(
                    &block.range,
                    self.total_lines,
                    track_x,
                    track_width,
                    drawable_height,
                );
                frame.fill_rectangle(
                    Point::new(rect.x, rect.y + OVERVIEW_PADDING_Y),
                    Size::new(rect.width, rect.height),
                    overview_fill(block.kind, false),
                );
            }
        });

        let mut frame = Frame::new(renderer, bounds.size());

        if let Some(selected) = self.current_hunk {
            if let Some(hunk) = self.hunks.iter().find(|hunk| hunk.id == selected) {
                let rect = overview_rect(
                    &hunk.range,
                    self.total_lines,
                    track_x,
                    track_width,
                    drawable_height,
                );
                let path = canvas::Path::rectangle(
                    Point::new(rect.x - 0.5, rect.y + OVERVIEW_PADDING_Y - 0.5),
                    Size::new(rect.width + 1.0, rect.height + 1.0),
                );
                frame.stroke(
                    &path,
                    canvas::Stroke::default()
                        .with_width(1.0)
                        .with_color(theme::darcula::ACCENT),
                );
            }
        }

        let viewport_y = self.viewport_range.start * drawable_height + OVERVIEW_PADDING_Y;
        let viewport_height =
            ((self.viewport_range.end - self.viewport_range.start) * drawable_height).max(8.0);
        let viewport_rect = canvas::Path::rectangle(
            Point::new(0.5, viewport_y),
            Size::new(bounds.width - 1.0, viewport_height),
        );
        frame.fill(
            &viewport_rect,
            theme::darcula::TEXT_PRIMARY.scale_alpha(0.08),
        );
        frame.stroke(
            &viewport_rect,
            canvas::Stroke::default()
                .with_width(1.0)
                .with_color(theme::darcula::TEXT_SECONDARY.scale_alpha(0.35)),
        );

        vec![base, frame.into_geometry()]
    }

    fn mouse_interaction(
        &self,
        _state: &Self::State,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if cursor.position_in(bounds).is_some() {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct VisualBounds {
    start: f32,
    end: f32,
}

#[derive(Debug, Clone, Copy)]
struct OverviewRect {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

pub fn build_editor_with_font_size(
    content: &str,
    path_hint: Option<&str>,
    font_size: f32,
) -> CodeEditor {
    let syntax = syntax_for_path(path_hint);
    let mut editor = CodeEditor::new(content, syntax);
    editor.set_font(theme::code_font());
    editor.set_font_size(font_size, true);
    editor.set_wrap_enabled(false);
    editor.set_line_numbers_enabled(true);
    editor.set_search_replace_enabled(false);
    editor.set_theme(editor_style());
    editor
}

const DEFAULT_EDITOR_FONT_SIZE: f32 = 13.0;

fn editor_style() -> iced_code_editor::theme::Style {
    iced_code_editor::theme::Style {
        background: iced::Color::TRANSPARENT,
        text_color: theme::darcula::TEXT_PRIMARY,
        gutter_background: iced::Color::TRANSPARENT,
        gutter_border: iced::Color::TRANSPARENT,
        line_number_color: theme::darcula::TEXT_DISABLED,
        scrollbar_background: theme::darcula::BG_PANEL.scale_alpha(0.65),
        scroller_color: theme::darcula::BORDER.scale_alpha(0.95),
        current_line_highlight: iced::Color::TRANSPARENT,
    }
}

fn syntax_for_path(path_hint: Option<&str>) -> &str {
    path_hint
        .and_then(|path| Path::new(path).extension())
        .and_then(|ext| ext.to_str())
        .unwrap_or("txt")
}

fn build_pane_decorations(model: &EditorDiffModel) -> (PaneDecorations, PaneDecorations) {
    let mut left = PaneDecorations {
        lines: vec![None; logical_line_count(&model.left_text)],
    };
    let mut right = PaneDecorations {
        lines: vec![None; logical_line_count(&model.right_text)],
    };

    for hunk in &model.hunks {
        for block in &hunk.blocks {
            for line in &block.old_lines {
                set_decoration(&mut left, line, block.kind);
            }
            for line in &block.new_lines {
                set_decoration(&mut right, line, block.kind);
            }
        }
    }

    (left, right)
}

fn set_decoration(
    decorations: &mut PaneDecorations,
    line: &EditorDiffLine,
    kind: EditorDiffBlockKind,
) {
    if let Some(slot) = decorations.lines.get_mut(line.index) {
        *slot = Some(DecoratedLine {
            kind,
            content: line.content.clone(),
            inline_changes: line.inline_changes.clone(),
        });
    }
}

fn build_link_blocks(model: &EditorDiffModel) -> Vec<LinkMapBlock> {
    model
        .hunks
        .iter()
        .flat_map(|hunk| {
            hunk.blocks
                .iter()
                .filter(|block| block.kind != EditorDiffBlockKind::Equal)
                .map(move |block| LinkMapBlock {
                    hunk_id: hunk.id,
                    kind: block.kind,
                    left_range: block.old_range.clone(),
                    right_range: block.new_range.clone(),
                })
        })
        .collect()
}

fn build_overview_blocks(blocks: &[LinkMapBlock]) -> Vec<OverviewBlock> {
    blocks
        .iter()
        .map(|block| OverviewBlock {
            kind: block.kind,
            range: merged_range(&block.left_range, &block.right_range),
        })
        .collect()
}

fn build_overview_hunks(model: &EditorDiffModel) -> Vec<OverviewHunk> {
    model
        .hunks
        .iter()
        .map(|hunk| OverviewHunk {
            id: hunk.id,
            range: merged_range(&hunk.old_range, &hunk.new_range),
        })
        .collect()
}

fn logical_line_count(text: &str) -> usize {
    if text.is_empty() {
        1
    } else {
        text.lines().count().max(1)
    }
}

fn block_colors(kind: EditorDiffBlockKind) -> (iced::Color, iced::Color) {
    let tag = block_kind_to_chunk(kind);
    (
        diff_core::chunk_code_bg(tag),
        diff_core::chunk_gutter_bg(tag),
    )
}

fn inline_color(kind: EditorDiffBlockKind) -> iced::Color {
    match kind {
        EditorDiffBlockKind::Insert => iced::Color::from_rgba(0.20, 0.62, 0.40, 0.35),
        EditorDiffBlockKind::Delete => iced::Color::from_rgba(0.65, 0.25, 0.25, 0.35),
        EditorDiffBlockKind::Replace => iced::Color::from_rgba(0.30, 0.55, 0.85, 0.28),
        EditorDiffBlockKind::Equal => iced::Color::TRANSPARENT,
    }
}

fn link_map_fill(kind: EditorDiffBlockKind, active: bool) -> iced::Color {
    let alpha = if active { 0.36 } else { 0.24 };
    match kind {
        EditorDiffBlockKind::Insert => iced::Color::from_rgba(0.42, 0.86, 0.50, alpha),
        EditorDiffBlockKind::Delete => iced::Color::from_rgba(0.92, 0.44, 0.44, alpha),
        EditorDiffBlockKind::Replace => iced::Color::from_rgba(0.44, 0.68, 0.96, alpha),
        EditorDiffBlockKind::Equal => iced::Color::TRANSPARENT,
    }
}

fn link_map_stroke(kind: EditorDiffBlockKind) -> iced::Color {
    match kind {
        EditorDiffBlockKind::Insert => iced::Color::from_rgba(0.42, 0.86, 0.50, 0.70),
        EditorDiffBlockKind::Delete => iced::Color::from_rgba(0.92, 0.44, 0.44, 0.70),
        EditorDiffBlockKind::Replace => iced::Color::from_rgba(0.44, 0.68, 0.96, 0.75),
        EditorDiffBlockKind::Equal => iced::Color::TRANSPARENT,
    }
}

fn overview_fill(kind: EditorDiffBlockKind, active: bool) -> iced::Color {
    let alpha = if active { 0.90 } else { 0.68 };
    match kind {
        EditorDiffBlockKind::Insert => iced::Color::from_rgba(0.42, 0.86, 0.50, alpha),
        EditorDiffBlockKind::Delete => iced::Color::from_rgba(0.92, 0.44, 0.44, alpha),
        EditorDiffBlockKind::Replace => iced::Color::from_rgba(0.44, 0.68, 0.96, alpha),
        EditorDiffBlockKind::Equal => iced::Color::TRANSPARENT,
    }
}

fn block_kind_to_chunk(kind: EditorDiffBlockKind) -> diff_core::ChunkTag {
    match kind {
        EditorDiffBlockKind::Equal => diff_core::ChunkTag::Equal,
        EditorDiffBlockKind::Insert => diff_core::ChunkTag::Insert,
        EditorDiffBlockKind::Delete => diff_core::ChunkTag::Delete,
        EditorDiffBlockKind::Replace => diff_core::ChunkTag::Replace,
    }
}

fn pane_index(pane: DiffEditorPane) -> usize {
    match pane {
        DiffEditorPane::Left => 0,
        DiffEditorPane::Right => 1,
    }
}

fn opposite_pane(pane: DiffEditorPane) -> DiffEditorPane {
    match pane {
        DiffEditorPane::Left => DiffEditorPane::Right,
        DiffEditorPane::Right => DiffEditorPane::Left,
    }
}

fn current_hunk_from_anchor(
    hunks: &[EditorDiffHunk],
    pane: DiffEditorPane,
    anchor: f32,
) -> Option<usize> {
    for hunk in hunks {
        let range = hunk_range_for_pane(hunk, pane);
        let end = range.end.max(range.start + 1) as f32;
        if anchor < end {
            return Some(hunk.id);
        }
    }
    hunks.last().map(|hunk| hunk.id)
}

fn hunk_range_for_pane(hunk: &EditorDiffHunk, pane: DiffEditorPane) -> Range<usize> {
    match pane {
        DiffEditorPane::Left if !hunk.old_range.is_empty() => hunk.old_range.clone(),
        DiffEditorPane::Right if !hunk.new_range.is_empty() => hunk.new_range.clone(),
        DiffEditorPane::Left => hunk.new_range.clone(),
        DiffEditorPane::Right => hunk.old_range.clone(),
    }
}

fn block_range_for_pane(block: &EditorDiffBlock, pane: DiffEditorPane) -> Range<usize> {
    match pane {
        DiffEditorPane::Left if !block.old_range.is_empty() => block.old_range.clone(),
        DiffEditorPane::Right if !block.new_range.is_empty() => block.new_range.clone(),
        DiffEditorPane::Left => block.new_range.clone(),
        DiffEditorPane::Right => block.old_range.clone(),
    }
}

fn hunk_anchor_line(hunk: &EditorDiffHunk, pane: DiffEditorPane) -> f32 {
    let range = hunk_range_for_pane(hunk, pane);
    range.start as f32
}

fn map_anchor_line(
    hunks: &[EditorDiffHunk],
    pane: DiffEditorPane,
    anchor_line: f32,
) -> Option<f32> {
    let mut previous: Option<(Range<usize>, Range<usize>)> = None;

    for hunk in hunks {
        let source_hunk = hunk_range_for_pane(hunk, pane);
        let target_hunk = hunk_range_for_pane(hunk, opposite_pane(pane));

        if anchor_line < source_hunk.start as f32 {
            return previous.map(|(prev_source, prev_target)| {
                interpolate_between_ranges(
                    anchor_line,
                    &prev_source,
                    &source_hunk,
                    &prev_target,
                    &target_hunk,
                )
            });
        }

        for block in &hunk.blocks {
            let source_block = block_range_for_pane(block, pane);
            if line_in_range(&source_block, anchor_line) {
                return Some(interpolate_line_between_ranges(
                    anchor_line,
                    &source_block,
                    &block_range_for_pane(block, opposite_pane(pane)),
                ));
            }
        }

        previous = Some((source_hunk, target_hunk));
    }

    None
}

fn map_anchor_line_from_line_map(
    line_map: &[EditorLineMapEntry],
    pane: DiffEditorPane,
    anchor_line: f32,
) -> Option<f32> {
    let source_floor = anchor_line.floor();
    let source_ceil = anchor_line.ceil();
    let source_fraction = anchor_line.fract();

    let mut previous: Option<(f32, f32)> = None;
    let mut next: Option<(f32, f32)> = None;

    for entry in line_map {
        let mapped = match pane {
            DiffEditorPane::Left => entry.old_index.zip(entry.new_index),
            DiffEditorPane::Right => entry.new_index.zip(entry.old_index),
        };
        let Some((source, target)) = mapped else {
            continue;
        };
        let source = source as f32;
        let target = target as f32;

        if source <= source_floor {
            previous = Some((source, target));
        }
        if next.is_none() && source >= source_ceil {
            next = Some((source, target));
        }
    }

    match (previous, next) {
        (Some((prev_source, prev_target)), Some((next_source, next_target)))
            if anchor_line.fract().abs() < f32::EPSILON =>
        {
            if (anchor_line - prev_source) <= (next_source - anchor_line) {
                Some(prev_target)
            } else {
                Some(next_target)
            }
        }
        (Some((prev_source, prev_target)), Some((next_source, next_target)))
            if (next_source - prev_source).abs() > f32::EPSILON =>
        {
            let normalized =
                ((source_floor + source_fraction) - prev_source) / (next_source - prev_source);
            Some(prev_target + normalized * (next_target - prev_target))
        }
        (Some((_, prev_target)), _) => Some(prev_target + source_fraction),
        (_, Some((_, next_target))) => Some((next_target - (1.0 - source_fraction)).max(0.0)),
        _ => None,
    }
}

fn hunk_for_overview_fraction(
    hunks: &[EditorDiffHunk],
    pane: DiffEditorPane,
    fraction: f32,
    total_lines: usize,
) -> Option<usize> {
    if total_lines == 0 {
        return None;
    }
    let anchor = anchor_line_for_fraction(fraction, total_lines);
    current_hunk_from_anchor(hunks, pane, anchor)
}

fn calc_sync_point(viewport_scroll: f32, viewport_height: f32, content_height: f32) -> f32 {
    if viewport_height <= 0.0 || content_height <= viewport_height {
        return 0.5;
    }

    let half_screen = viewport_height / 2.0;
    if half_screen <= 0.0 {
        return 0.5;
    }

    let first_scale = viewport_scroll / half_screen;
    let bottom_val = content_height - 1.5 * viewport_height;
    let last_scale = (viewport_scroll - bottom_val) / half_screen;

    (0.5 * first_scale.min(1.0) + 0.5 * last_scale.max(0.0)).clamp(0.0, 1.0)
}

fn anchor_line_for_scroll(
    viewport_scroll: f32,
    sync_point: f32,
    viewport_height: f32,
    line_height: f32,
) -> f32 {
    if line_height <= 0.0 {
        return 0.0;
    }
    (viewport_scroll + viewport_height * sync_point) / line_height
}

fn scroll_for_anchor_line(
    anchor_line: f32,
    sync_point: f32,
    viewport_height: f32,
    line_height: f32,
    total_lines: usize,
) -> f32 {
    if line_height <= 0.0 {
        return 0.0;
    }

    let content_height = content_height(total_lines, line_height);
    let max_scroll = (content_height - viewport_height).max(0.0);
    let anchor_y = anchor_line.max(0.0) * line_height;

    (anchor_y - viewport_height * sync_point).clamp(0.0, max_scroll)
}

fn anchor_line_for_fraction(fraction: f32, total_lines: usize) -> f32 {
    if total_lines <= 1 {
        0.0
    } else {
        (fraction.clamp(0.0, 1.0) * (total_lines.saturating_sub(1)) as f32)
            .clamp(0.0, (total_lines.saturating_sub(1)) as f32)
    }
}

fn scale_anchor_line(
    anchor_line: f32,
    source_total_lines: usize,
    target_total_lines: usize,
) -> f32 {
    if source_total_lines <= 1 || target_total_lines <= 1 {
        return 0.0;
    }

    let ratio = anchor_line / (source_total_lines.saturating_sub(1)) as f32;
    ratio.clamp(0.0, 1.0) * (target_total_lines.saturating_sub(1)) as f32
}

fn interpolate_line_between_ranges(
    anchor_line: f32,
    source_range: &Range<usize>,
    target_range: &Range<usize>,
) -> f32 {
    if source_range.is_empty() {
        return target_range.start as f32;
    }
    if target_range.is_empty() {
        return target_range.start as f32;
    }

    let source_len = (source_range.end - source_range.start) as f32;
    let target_len = (target_range.end - target_range.start) as f32;
    let offset = (anchor_line - source_range.start as f32).clamp(0.0, source_len);

    target_range.start as f32 + offset / source_len.max(1.0) * target_len
}

fn interpolate_between_ranges(
    anchor_line: f32,
    previous_source: &Range<usize>,
    next_source: &Range<usize>,
    previous_target: &Range<usize>,
    next_target: &Range<usize>,
) -> f32 {
    let source_start = previous_source.end as f32;
    let source_end = next_source.start as f32;
    let target_start = previous_target.end as f32;
    let target_end = next_target.start as f32;

    if (source_end - source_start).abs() <= f32::EPSILON {
        return target_start;
    }

    let fraction = ((anchor_line - source_start) / (source_end - source_start)).clamp(0.0, 1.0);
    target_start + fraction * (target_end - target_start)
}

fn line_in_range(range: &Range<usize>, line: f32) -> bool {
    if range.is_empty() {
        (line - range.start as f32).abs() < f32::EPSILON
    } else {
        line >= range.start as f32 && line < range.end as f32
    }
}

fn merged_range(left: &Range<usize>, right: &Range<usize>) -> Range<usize> {
    let start = left.start.min(right.start);
    let end = left.end.max(right.end).max(start + 1);
    start..end
}

fn block_visual_bounds(
    range: &Range<usize>,
    line_height: f32,
    scroll: f32,
    minimum_height: f32,
) -> VisualBounds {
    let start = range.start as f32 * line_height - scroll;
    let height = if range.is_empty() {
        minimum_height
    } else {
        ((range.end - range.start) as f32 * line_height).max(minimum_height)
    };

    VisualBounds {
        start,
        end: start + height,
    }
}

fn overview_rect(
    range: &Range<usize>,
    total_lines: usize,
    x: f32,
    width: f32,
    drawable_height: f32,
) -> OverviewRect {
    let total = total_lines.max(1) as f32;
    let y = range.start as f32 / total * drawable_height;
    let height = (((range.end.max(range.start + 1) - range.start) as f32 / total)
        * drawable_height)
        .max(MIN_OVERVIEW_BLOCK_HEIGHT);

    OverviewRect {
        x,
        y,
        width,
        height,
    }
}

fn overview_fraction(y: f32, height: f32) -> f32 {
    ((y - OVERVIEW_PADDING_Y) / (height - OVERVIEW_PADDING_Y * 2.0).max(1.0)).clamp(0.0, 1.0)
}

fn content_height(total_lines: usize, line_height: f32) -> f32 {
    total_lines.max(1) as f32 * line_height.max(1.0)
}

fn measure_text_width(text: &str, full_char_width: f32, char_width: f32) -> f32 {
    text.chars()
        .map(|ch| match ch {
            '\t' => char_width * 4.0,
            _ => match ch.width() {
                Some(width) if width > 1 => full_char_width,
                Some(_) => char_width,
                None => 0.0,
            },
        })
        .sum()
}

fn is_mutating_message(message: &EditorMessage) -> bool {
    matches!(
        message,
        EditorMessage::CharacterInput(_)
            | EditorMessage::Backspace
            | EditorMessage::Delete
            | EditorMessage::Enter
            | EditorMessage::Tab
            | EditorMessage::Paste(_)
            | EditorMessage::DeleteSelection
            | EditorMessage::Undo
            | EditorMessage::Redo
            | EditorMessage::OpenSearch
            | EditorMessage::OpenSearchReplace
            | EditorMessage::CloseSearch
            | EditorMessage::SearchQueryChanged(_)
            | EditorMessage::ReplaceQueryChanged(_)
            | EditorMessage::ToggleCaseSensitive
            | EditorMessage::FindNext
            | EditorMessage::FindPrevious
            | EditorMessage::ReplaceNext
            | EditorMessage::ReplaceAll
            | EditorMessage::SearchDialogTab
            | EditorMessage::SearchDialogShiftTab
            | EditorMessage::ImeOpened
            | EditorMessage::ImePreedit(_, _)
            | EditorMessage::ImeCommit(_)
            | EditorMessage::ImeClosed
    )
}

// ═══════════════════════════════════════
// Unified Diff Editor (single CodeEditor pane)
// ═══════════════════════════════════════

/// Line decoration kind for the unified view.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnifiedLineKind {
    Context,
    Addition,
    Deletion,
    HunkHeader,
}

/// State for the unified diff view backed by a single CodeEditor.
pub struct UnifiedDiffEditorState {
    editor: CodeEditor,
    decorations: Arc<PaneDecorations>,
    line_count: usize,
    hunk_start_lines: Arc<[usize]>,
    /// Stored for clone/rebuild
    source_diff: git_core::diff::Diff,
    font_size: f32,
}

impl std::fmt::Debug for UnifiedDiffEditorState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UnifiedDiffEditorState")
            .field("line_count", &self.line_count)
            .finish()
    }
}

impl Clone for UnifiedDiffEditorState {
    fn clone(&self) -> Self {
        Self::from_diff(&self.source_diff, self.font_size)
    }
}

impl UnifiedDiffEditorState {
    /// Build from a `Diff` (the standard unified diff data).
    pub fn from_diff(diff: &git_core::diff::Diff, font_size: f32) -> Self {
        let mut text = String::new();
        let mut line_kinds: Vec<UnifiedLineKind> = Vec::new();
        let mut hunk_start_lines = Vec::new();

        let path_hint = diff
            .files
            .first()
            .and_then(|f| f.new_path.as_deref().or(f.old_path.as_deref()));

        for file in &diff.files {
            for hunk in &file.hunks {
                hunk_start_lines.push(line_kinds.len());
                // Hunk header line — only show the @@ range part, not the trailing context
                let header_display = hunk
                    .header
                    .find(" @@")
                    .map(|pos| &hunk.header[..pos + 3])
                    .unwrap_or(&hunk.header);
                text.push_str(header_display);
                text.push('\n');
                line_kinds.push(UnifiedLineKind::HunkHeader);

                for line in &hunk.lines {
                    // Add +/- prefix like a real unified diff
                    let prefix = match line.origin {
                        git_core::diff::DiffLineOrigin::Addition => "+",
                        git_core::diff::DiffLineOrigin::Deletion => "-",
                        _ => " ",
                    };
                    text.push_str(prefix);
                    text.push_str(&line.content);
                    if !line.content.ends_with('\n') {
                        text.push('\n');
                    }

                    let kind = match line.origin {
                        git_core::diff::DiffLineOrigin::Addition => UnifiedLineKind::Addition,
                        git_core::diff::DiffLineOrigin::Deletion => UnifiedLineKind::Deletion,
                        _ => UnifiedLineKind::Context,
                    };
                    line_kinds.push(kind);
                }
            }
        }

        let line_count = logical_line_count(&text);
        let mut editor = build_editor_with_font_size(&text, path_hint, font_size);
        editor.set_line_numbers_enabled(true);

        let decorations = build_unified_decorations(&line_kinds, line_count);

        Self {
            editor,
            decorations: Arc::new(decorations),
            line_count,
            hunk_start_lines: Arc::from(hunk_start_lines),
            source_diff: diff.clone(),
            font_size,
        }
    }

    pub fn update(&mut self, message: EditorMessage) -> iced::Task<UnifiedDiffEditorEvent> {
        if is_mutating_message(&message) {
            return iced::Task::none();
        }
        self.editor
            .update(&message)
            .map(UnifiedDiffEditorEvent::Editor)
    }

    pub fn scroll_to_hunk(&mut self, hunk_index: usize) -> iced::Task<UnifiedDiffEditorEvent> {
        let Some(line_index) = self.hunk_start_lines.get(hunk_index).copied() else {
            return iced::Task::none();
        };
        self.editor
            .scroll_to_offset(None, Some(line_index as f32 * self.editor.line_height()))
            .map(UnifiedDiffEditorEvent::Editor)
    }

    pub fn view(&self) -> Element<'_, UnifiedDiffEditorEvent> {
        let decorations = Arc::clone(&self.decorations);
        let background = Canvas::new(DiffDecorationCanvas {
            decorations,
            viewport_scroll: self.editor.viewport_scroll(),
            horizontal_scroll_offset: self.editor.horizontal_scroll_offset(),
            line_height: self.editor.line_height(),
            viewport_height: self.editor.viewport_height(),
            gutter_width: self.editor.gutter_width(),
            char_width: self.editor.char_width(),
            full_char_width: self.editor.full_char_width(),
            active_range: None,
        })
        .width(Length::Fill)
        .height(Length::Fill);

        let editor_view = self.editor.view().map(UnifiedDiffEditorEvent::Editor);

        Container::new(Stack::new().push(background).push(editor_view))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

#[derive(Debug, Clone)]
pub enum UnifiedDiffEditorEvent {
    Editor(EditorMessage),
}

fn build_unified_decorations(
    line_kinds: &[UnifiedLineKind],
    total_lines: usize,
) -> PaneDecorations {
    let mut lines: Vec<Option<DecoratedLine>> = vec![None; total_lines];

    for (i, kind) in line_kinds.iter().enumerate() {
        if i >= lines.len() {
            break;
        }
        let block_kind = match kind {
            UnifiedLineKind::Addition => EditorDiffBlockKind::Insert,
            UnifiedLineKind::Deletion => EditorDiffBlockKind::Delete,
            UnifiedLineKind::HunkHeader => EditorDiffBlockKind::Replace,
            UnifiedLineKind::Context => continue,
        };
        lines[i] = Some(DecoratedLine {
            kind: block_kind,
            content: String::new(),
            inline_changes: Vec::new(),
        });
    }

    PaneDecorations { lines }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_corresponding_line_prefers_nearest_mapped_row() {
        let line_map = vec![
            EditorLineMapEntry {
                old_index: Some(0),
                new_index: Some(0),
                kind: EditorDiffBlockKind::Equal,
            },
            EditorLineMapEntry {
                old_index: Some(1),
                new_index: None,
                kind: EditorDiffBlockKind::Delete,
            },
            EditorLineMapEntry {
                old_index: Some(2),
                new_index: Some(1),
                kind: EditorDiffBlockKind::Equal,
            },
        ];

        assert_eq!(
            map_anchor_line_from_line_map(&line_map, DiffEditorPane::Left, 1.0),
            Some(0.0)
        );
    }

    #[test]
    fn current_hunk_falls_back_to_next_hunk_after_gap() {
        let hunks = vec![
            EditorDiffHunk {
                id: 0,
                header: "@@ -1,1 +1,1 @@".to_string(),
                old_range: 2..3,
                new_range: 2..3,
                blocks: vec![EditorDiffBlock {
                    kind: EditorDiffBlockKind::Replace,
                    old_range: 2..3,
                    new_range: 2..3,
                    old_lines: vec![EditorDiffLine {
                        index: 2,
                        line_number: 3,
                        content: "old".to_string(),
                        inline_changes: Vec::new(),
                    }],
                    new_lines: vec![EditorDiffLine {
                        index: 2,
                        line_number: 3,
                        content: "new".to_string(),
                        inline_changes: Vec::new(),
                    }],
                }],
            },
            EditorDiffHunk {
                id: 1,
                header: "@@ -10,1 +10,1 @@".to_string(),
                old_range: 9..10,
                new_range: 9..10,
                blocks: Vec::new(),
            },
        ];

        assert_eq!(
            current_hunk_from_anchor(&hunks, DiffEditorPane::Left, 5.0),
            Some(1)
        );
    }

    #[test]
    fn calc_sync_point_matches_meld_top_middle_bottom_behavior() {
        assert_eq!(calc_sync_point(0.0, 200.0, 1200.0), 0.0);
        assert!((calc_sync_point(250.0, 200.0, 1200.0) - 0.5).abs() < f32::EPSILON);
        assert_eq!(calc_sync_point(1000.0, 200.0, 1200.0), 1.0);
    }

    #[test]
    fn map_anchor_line_interpolates_inside_replace_blocks() {
        let hunk = EditorDiffHunk {
            id: 0,
            header: "@@ -11,2 +11,6 @@".to_string(),
            old_range: 10..12,
            new_range: 10..16,
            blocks: vec![EditorDiffBlock {
                kind: EditorDiffBlockKind::Replace,
                old_range: 10..12,
                new_range: 10..16,
                old_lines: Vec::new(),
                new_lines: Vec::new(),
            }],
        };

        let mapped = map_anchor_line(&[hunk], DiffEditorPane::Right, 13.0).expect("mapped line");
        assert!((mapped - 11.0).abs() < f32::EPSILON);
    }

    #[test]
    fn overview_fraction_selects_hunk_by_document_position() {
        let hunks = vec![
            EditorDiffHunk {
                id: 0,
                header: "@@ -5,3 +5,3 @@".to_string(),
                old_range: 4..7,
                new_range: 4..7,
                blocks: Vec::new(),
            },
            EditorDiffHunk {
                id: 1,
                header: "@@ -80,4 +80,4 @@".to_string(),
                old_range: 79..83,
                new_range: 79..83,
                blocks: Vec::new(),
            },
        ];

        assert_eq!(
            hunk_for_overview_fraction(&hunks, DiffEditorPane::Right, 0.82, 100),
            Some(1)
        );
    }

    #[test]
    fn current_hunk_change_is_only_emitted_when_selection_changes() {
        let mut state = SplitDiffEditorState::new(EditorDiffModel {
            left_text: "one\ntwo\n".to_string(),
            right_text: "one\ntwo\n".to_string(),
            hunks: vec![EditorDiffHunk {
                id: 0,
                header: "@@ -1,1 +1,1 @@".to_string(),
                old_range: 0..1,
                new_range: 0..1,
                blocks: Vec::new(),
            }],
            line_map: Vec::new(),
            old_path: Some("a.txt".to_string()),
            new_path: Some("a.txt".to_string()),
        });

        assert_eq!(state.emit_current_hunk_change(Some(0)), Some(0));
        assert_eq!(state.emit_current_hunk_change(Some(0)), None);
        assert_eq!(state.emit_current_hunk_change(Some(1)), Some(1));
    }
}
