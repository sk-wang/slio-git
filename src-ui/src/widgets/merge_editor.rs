//! Meld-style 3-column merge editor.
//!
//! Three CodeEditor panes: left (ours) | center (result) | right (theirs)
//! with two link maps and synchronized scrolling.

use crate::theme::{self, BadgeTone, Surface};
use crate::widgets::diff_core;
use crate::widgets::{self, button};
use git_core::diff::{MergeChunkType, MergeEditorModel};
use iced::widget::canvas::{self, Canvas};
use iced::widget::{Column, Container, Row, Space, Stack, Text};
use iced::{mouse, Alignment, Element, Length, Point, Rectangle, Renderer, Size, Theme};
use iced_code_editor::{CodeEditor, Message as EditorMessage};
use std::cell::Cell;
use std::ops::Range;
use std::path::Path;
use std::sync::Arc;

const LINK_MAP_WIDTH: f32 = 32.0;
const OVERVIEW_WIDTH: f32 = 18.0;
const OVERVIEW_PADDING_Y: f32 = 6.0;
const MIN_EMPTY_BLOCK_HEIGHT: f32 = 6.0;
const MIN_OVERVIEW_BLOCK_HEIGHT: f32 = 2.0;

// ═══════════════════════════════════════
// Public types
// ═══════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergePane {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChunkResolution {
    Ours,
    Theirs,
    Base,
}

#[derive(Debug, Clone)]
pub enum MergeEditorEvent {
    Editor {
        pane: MergePane,
        message: EditorMessage,
    },
    AcceptOurs(usize),
    AcceptTheirs(usize),
    AcceptBase(usize),
    AcceptAllOurs,
    AcceptAllTheirs,
    AutoMerge,
    JumpToChunk(usize),
    PrevChunk,
    NextChunk,
    BackToList,
    Apply,
}

// ═══════════════════════════════════════
// Internal data
// ═══════════════════════════════════════

#[derive(Debug, Clone)]
struct PaneDecorations {
    lines: Vec<Option<MergeDecoratedLine>>,
}

#[derive(Debug, Clone)]
struct MergeDecoratedLine {
    chunk_type: MergeChunkType,
    resolved: bool,
}

#[derive(Debug, Clone)]
struct LinkMapBlock {
    chunk_id: usize,
    chunk_type: MergeChunkType,
    resolved: bool,
    left_range: Range<usize>,
    right_range: Range<usize>,
}

#[derive(Debug, Clone)]
struct OverviewBlock {
    chunk_type: MergeChunkType,
    resolved: bool,
    range: Range<usize>,
}

// ═══════════════════════════════════════
// MergeEditorState
// ═══════════════════════════════════════

pub struct MergeEditorState {
    model: MergeEditorModel,
    resolutions: Vec<Option<ChunkResolution>>,

    left: CodeEditor,
    center: CodeEditor,
    right: CodeEditor,

    left_decorations: Arc<PaneDecorations>,
    center_decorations: Arc<PaneDecorations>,
    right_decorations: Arc<PaneDecorations>,
    left_links: Arc<[LinkMapBlock]>,
    right_links: Arc<[LinkMapBlock]>,
    overview_blocks: Arc<[OverviewBlock]>,

    left_line_count: usize,
    center_line_count: usize,
    right_line_count: usize,

    current_chunk: Option<usize>,
    suppress_sync: [bool; 3],
}

impl std::fmt::Debug for MergeEditorState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MergeEditorState")
            .field("path", &self.model.path)
            .field("chunks", &self.model.chunks.len())
            .finish()
    }
}

impl Clone for MergeEditorState {
    fn clone(&self) -> Self {
        Self::new(self.model.clone())
    }
}

impl MergeEditorState {
    pub fn new(model: MergeEditorModel) -> Self {
        let chunk_count = model.chunks.len();
        let mut resolutions: Vec<Option<ChunkResolution>> = vec![None; chunk_count];

        // Auto-merge non-conflicting chunks
        for (i, chunk) in model.chunks.iter().enumerate() {
            resolutions[i] = match chunk.chunk_type {
                MergeChunkType::Equal => Some(ChunkResolution::Base),
                MergeChunkType::OursOnly => Some(ChunkResolution::Ours),
                MergeChunkType::TheirsOnly => Some(ChunkResolution::Theirs),
                MergeChunkType::Conflict => None,
            };
        }

        let center_text = assemble_center_text(&model, &resolutions);
        let path_hint = Some(model.path.as_str());

        let left = build_editor(&model.ours_text, path_hint);
        let center = build_editor(&center_text, path_hint);
        let right = build_editor(&model.theirs_text, path_hint);

        let left_line_count = line_count(&model.ours_text);
        let center_line_count = line_count(&center_text);
        let right_line_count = line_count(&model.theirs_text);

        let (left_decos, center_decos, right_decos) =
            build_all_decorations(&model, &resolutions, center_line_count);
        let (left_links, right_links) =
            build_all_link_blocks(&model, &resolutions, center_line_count);
        let overview_blocks = build_overview(&model, &resolutions);

        let first_conflict = model
            .chunks
            .iter()
            .position(|c| c.chunk_type == MergeChunkType::Conflict);

        Self {
            model,
            resolutions,
            left,
            center,
            right,
            left_decorations: Arc::new(left_decos),
            center_decorations: Arc::new(center_decos),
            right_decorations: Arc::new(right_decos),
            left_links: Arc::from(left_links),
            right_links: Arc::from(right_links),
            overview_blocks: Arc::from(overview_blocks),
            left_line_count,
            center_line_count,
            right_line_count,
            current_chunk: first_conflict.or(Some(0)),
            suppress_sync: [false; 3],
        }
    }

    pub fn resolved_text(&self) -> String {
        assemble_center_text(&self.model, &self.resolutions)
    }

    pub fn all_resolved(&self) -> bool {
        self.resolutions.iter().all(|r| r.is_some())
    }

    pub fn unresolved_count(&self) -> usize {
        self.resolutions.iter().filter(|r| r.is_none()).count()
    }

    pub fn total_chunks(&self) -> usize {
        self.model.chunks.len()
    }

    pub fn conflict_count(&self) -> usize {
        self.model
            .chunks
            .iter()
            .filter(|c| c.chunk_type == MergeChunkType::Conflict)
            .count()
    }

    pub fn update(&mut self, event: MergeEditorEvent) -> iced::Task<MergeEditorEvent> {
        match event {
            MergeEditorEvent::AcceptOurs(id) => {
                self.resolve_chunk(id, ChunkResolution::Ours);
                iced::Task::none()
            }
            MergeEditorEvent::AcceptTheirs(id) => {
                self.resolve_chunk(id, ChunkResolution::Theirs);
                iced::Task::none()
            }
            MergeEditorEvent::AcceptBase(id) => {
                self.resolve_chunk(id, ChunkResolution::Base);
                iced::Task::none()
            }
            MergeEditorEvent::AcceptAllOurs => {
                for i in 0..self.resolutions.len() {
                    self.resolutions[i] = Some(ChunkResolution::Ours);
                }
                self.rebuild_center();
                iced::Task::none()
            }
            MergeEditorEvent::AcceptAllTheirs => {
                for i in 0..self.resolutions.len() {
                    self.resolutions[i] = Some(ChunkResolution::Theirs);
                }
                self.rebuild_center();
                iced::Task::none()
            }
            MergeEditorEvent::AutoMerge => {
                for (i, chunk) in self.model.chunks.iter().enumerate() {
                    if self.resolutions[i].is_none() {
                        self.resolutions[i] = match chunk.chunk_type {
                            MergeChunkType::Equal => Some(ChunkResolution::Base),
                            MergeChunkType::OursOnly => Some(ChunkResolution::Ours),
                            MergeChunkType::TheirsOnly => Some(ChunkResolution::Theirs),
                            MergeChunkType::Conflict => None,
                        };
                    }
                }
                self.rebuild_center();
                iced::Task::none()
            }
            MergeEditorEvent::JumpToChunk(id) => {
                self.current_chunk = Some(id);
                iced::Task::none()
            }
            MergeEditorEvent::PrevChunk => {
                if let Some(current) = self.current_chunk {
                    // Find previous conflict chunk
                    for i in (0..current).rev() {
                        if self.model.chunks[i].chunk_type == MergeChunkType::Conflict {
                            self.current_chunk = Some(i);
                            break;
                        }
                    }
                }
                iced::Task::none()
            }
            MergeEditorEvent::NextChunk => {
                let start = self.current_chunk.map(|c| c + 1).unwrap_or(0);
                for i in start..self.model.chunks.len() {
                    if self.model.chunks[i].chunk_type == MergeChunkType::Conflict {
                        self.current_chunk = Some(i);
                        break;
                    }
                }
                iced::Task::none()
            }
            MergeEditorEvent::Editor { pane, message } => self.handle_editor_event(pane, message),
            MergeEditorEvent::BackToList | MergeEditorEvent::Apply => iced::Task::none(),
        }
    }

    fn resolve_chunk(&mut self, id: usize, resolution: ChunkResolution) {
        if id < self.resolutions.len() {
            self.resolutions[id] = Some(resolution);
            self.rebuild_center();
            // Advance to next unresolved conflict
            for i in (id + 1)..self.model.chunks.len() {
                if self.resolutions[i].is_none()
                    && self.model.chunks[i].chunk_type == MergeChunkType::Conflict
                {
                    self.current_chunk = Some(i);
                    return;
                }
            }
        }
    }

    fn rebuild_center(&mut self) {
        let center_text = assemble_center_text(&self.model, &self.resolutions);
        self.center = build_editor(&center_text, Some(&self.model.path));
        self.center_line_count = line_count(&center_text);

        let (left_decos, center_decos, right_decos) =
            build_all_decorations(&self.model, &self.resolutions, self.center_line_count);
        let (left_links, right_links) =
            build_all_link_blocks(&self.model, &self.resolutions, self.center_line_count);
        let overview_blocks = build_overview(&self.model, &self.resolutions);

        self.left_decorations = Arc::new(left_decos);
        self.center_decorations = Arc::new(center_decos);
        self.right_decorations = Arc::new(right_decos);
        self.left_links = Arc::from(left_links);
        self.right_links = Arc::from(right_links);
        self.overview_blocks = Arc::from(overview_blocks);
    }

    fn handle_editor_event(
        &mut self,
        pane: MergePane,
        message: EditorMessage,
    ) -> iced::Task<MergeEditorEvent> {
        let pane_idx = pane_index(pane);
        let should_skip_sync =
            matches!(message, EditorMessage::Scrolled(_)) && self.suppress_sync[pane_idx];
        if should_skip_sync {
            self.suppress_sync[pane_idx] = false;
        }

        // Block mutations on all panes (read-only)
        if is_mutating(&message) {
            return iced::Task::none();
        }

        let local_task = self
            .editor_mut(pane)
            .update(&message)
            .map(move |m| MergeEditorEvent::Editor { pane, message: m });

        let sync_task = match &message {
            EditorMessage::Scrolled(viewport) if !should_skip_sync => {
                self.synced_scroll_3way(pane, viewport.absolute_offset().y)
            }
            _ => iced::Task::none(),
        };

        iced::Task::batch([local_task, sync_task])
    }

    fn synced_scroll_3way(
        &mut self,
        source_pane: MergePane,
        source_scroll: f32,
    ) -> iced::Task<MergeEditorEvent> {
        let source_editor = self.editor(source_pane);
        let source_lh = source_editor.line_height();
        if source_lh <= 0.0 {
            return iced::Task::none();
        }

        let source_lines = self.pane_line_count(source_pane);
        let source_sp = calc_sync_point(
            source_scroll,
            source_editor.viewport_height(),
            content_height(source_lines, source_lh),
        );
        let source_anchor = anchor_line_for_scroll(
            source_scroll,
            source_sp,
            source_editor.viewport_height(),
            source_lh,
        );

        // Map source anchor to the other two panes
        let targets: [(MergePane, f32); 2] = match source_pane {
            MergePane::Left => {
                let center_anchor = self
                    .map_anchor(source_pane, MergePane::Center, source_anchor)
                    .unwrap_or(scale_anchor(
                        source_anchor,
                        source_lines,
                        self.center_line_count,
                    ));
                let right_anchor = self
                    .map_anchor(MergePane::Center, MergePane::Right, center_anchor)
                    .unwrap_or(scale_anchor(
                        center_anchor,
                        self.center_line_count,
                        self.right_line_count,
                    ));
                [
                    (MergePane::Center, center_anchor),
                    (MergePane::Right, right_anchor),
                ]
            }
            MergePane::Center => {
                let left_anchor = self
                    .map_anchor(source_pane, MergePane::Left, source_anchor)
                    .unwrap_or(scale_anchor(
                        source_anchor,
                        source_lines,
                        self.left_line_count,
                    ));
                let right_anchor = self
                    .map_anchor(source_pane, MergePane::Right, source_anchor)
                    .unwrap_or(scale_anchor(
                        source_anchor,
                        source_lines,
                        self.right_line_count,
                    ));
                [
                    (MergePane::Left, left_anchor),
                    (MergePane::Right, right_anchor),
                ]
            }
            MergePane::Right => {
                let center_anchor = self
                    .map_anchor(source_pane, MergePane::Center, source_anchor)
                    .unwrap_or(scale_anchor(
                        source_anchor,
                        source_lines,
                        self.center_line_count,
                    ));
                let left_anchor = self
                    .map_anchor(MergePane::Center, MergePane::Left, center_anchor)
                    .unwrap_or(scale_anchor(
                        center_anchor,
                        self.center_line_count,
                        self.left_line_count,
                    ));
                [
                    (MergePane::Center, center_anchor),
                    (MergePane::Left, left_anchor),
                ]
            }
        };

        let mut tasks = Vec::new();
        for (target_pane, target_anchor) in targets {
            let target_editor = self.editor(target_pane);
            let target_scroll = scroll_for_anchor_line(
                target_anchor,
                source_sp,
                target_editor.viewport_height(),
                target_editor.line_height(),
                self.pane_line_count(target_pane),
            );
            if (target_editor.viewport_scroll() - target_scroll).abs() > 0.5 {
                self.suppress_sync[pane_index(target_pane)] = true;
                tasks.push(
                    self.editor(target_pane)
                        .scroll_to_offset(None, Some(target_scroll))
                        .map(move |m| MergeEditorEvent::Editor {
                            pane: target_pane,
                            message: m,
                        }),
                );
            }
        }

        iced::Task::batch(tasks)
    }

    fn map_anchor(&self, from: MergePane, to: MergePane, anchor: f32) -> Option<f32> {
        let links = match (from, to) {
            (MergePane::Left, MergePane::Center) | (MergePane::Center, MergePane::Left) => {
                &self.left_links
            }
            (MergePane::Right, MergePane::Center) | (MergePane::Center, MergePane::Right) => {
                &self.right_links
            }
            _ => return None,
        };

        let forward = matches!(
            (from, to),
            (MergePane::Left, MergePane::Center) | (MergePane::Right, MergePane::Center)
        );

        for block in links.iter() {
            let (source_range, target_range) = if forward {
                (&block.left_range, &block.right_range)
            } else {
                (&block.right_range, &block.left_range)
            };

            if line_in_range(source_range, anchor) {
                return Some(interpolate_in_range(anchor, source_range, target_range));
            }
        }

        None
    }

    pub fn view<'b>(&'b self, i18n: &'b crate::i18n::I18n) -> Element<'b, MergeEditorEvent> {
        let unresolved = self.unresolved_count();
        let conflict_total = self.conflict_count();
        let resolved_conflicts = conflict_total - unresolved;

        // ── Toolbar ──
        let toolbar = Container::new(
            Row::new()
                .spacing(theme::spacing::XS)
                .align_y(Alignment::Center)
                .push(button::compact_ghost(
                    i18n.back_to_list,
                    Some(MergeEditorEvent::BackToList),
                ))
                .push(button::compact_ghost(
                    i18n.prev_conflict,
                    Some(MergeEditorEvent::PrevChunk),
                ))
                .push(button::compact_ghost(
                    i18n.next_conflict,
                    Some(MergeEditorEvent::NextChunk),
                ))
                .push(Space::new().width(Length::Fill))
                .push(button::compact_ghost(
                    i18n.auto_merge,
                    Some(MergeEditorEvent::AutoMerge),
                ))
                .push(button::compact_ghost(
                    i18n.accept_all_ours,
                    Some(MergeEditorEvent::AcceptAllOurs),
                ))
                .push(button::compact_ghost(
                    i18n.accept_all_theirs,
                    Some(MergeEditorEvent::AcceptAllTheirs),
                )),
        )
        .padding([4, 10])
        .width(Length::Fill)
        .style(theme::frame_style(Surface::Toolbar));

        // ── Column headers ──
        let headers = Row::new()
            .spacing(0)
            .push(
                Container::new(
                    Text::new(i18n.ours_version)
                        .size(10)
                        .color(merge_pane_color(MergeChunkType::OursOnly)),
                )
                .padding([2, 8])
                .width(Length::FillPortion(5)),
            )
            .push(Space::new().width(Length::Fixed(LINK_MAP_WIDTH)))
            .push(
                Container::new(
                    Text::new(i18n.merge_result)
                        .size(10)
                        .color(iced::Color::from_rgb(0.42, 0.86, 0.50)),
                )
                .padding([2, 8])
                .width(Length::FillPortion(5)),
            )
            .push(Space::new().width(Length::Fixed(LINK_MAP_WIDTH)))
            .push(
                Container::new(
                    Text::new(i18n.theirs_version)
                        .size(10)
                        .color(merge_pane_color(MergeChunkType::TheirsOnly)),
                )
                .padding([2, 8])
                .width(Length::FillPortion(5)),
            )
            .push(Space::new().width(Length::Fixed(OVERVIEW_WIDTH)));

        // ── 3-pane editor ──
        let editor_row = Row::new()
            .spacing(0)
            .width(Length::Fill)
            .height(Length::Fill)
            .push(
                self.pane_view(MergePane::Left)
                    .width(Length::FillPortion(5)),
            )
            .push(self.link_map_view(LinkMapSide::Left))
            .push(diff_core::center_divider())
            .push(
                self.pane_view(MergePane::Center)
                    .width(Length::FillPortion(5)),
            )
            .push(diff_core::center_divider())
            .push(self.link_map_view(LinkMapSide::Right))
            .push(
                self.pane_view(MergePane::Right)
                    .width(Length::FillPortion(5)),
            )
            .push(self.overview_view());

        // ── Footer ──
        let footer = Container::new(
            Row::new()
                .spacing(theme::spacing::XS)
                .align_y(Alignment::Center)
                .push(
                    Text::new(format!(
                        "{} conflicts, resolved {}/{}",
                        conflict_total, resolved_conflicts, conflict_total
                    ))
                    .size(11)
                    .color(theme::darcula::TEXT_SECONDARY),
                )
                .push(Space::new().width(Length::Fill))
                .push(widgets::info_chip::<MergeEditorEvent>(
                    &self.model.path,
                    BadgeTone::Neutral,
                ))
                .push(Space::new().width(Length::Fixed(8.0)))
                .push(button::ghost(i18n.cancel, Some(MergeEditorEvent::BackToList)))
                .push(button::primary(
                    i18n.apply,
                    self.all_resolved().then_some(MergeEditorEvent::Apply),
                )),
        )
        .padding([6, 10])
        .width(Length::Fill)
        .style(theme::frame_style(Surface::Toolbar));

        Container::new(
            Column::new()
                .spacing(0)
                .push(toolbar)
                .push(iced::widget::rule::horizontal(1))
                .push(headers)
                .push(iced::widget::rule::horizontal(1))
                .push(editor_row)
                .push(iced::widget::rule::horizontal(1))
                .push(footer),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .style(theme::panel_style(Surface::Panel))
        .into()
    }

    fn pane_view(&self, pane: MergePane) -> Container<'_, MergeEditorEvent> {
        let editor = self.editor(pane);
        let decorations = match pane {
            MergePane::Left => Arc::clone(&self.left_decorations),
            MergePane::Center => Arc::clone(&self.center_decorations),
            MergePane::Right => Arc::clone(&self.right_decorations),
        };

        let active_range = self
            .current_chunk
            .and_then(|id| self.model.chunks.get(id))
            .map(|chunk| match pane {
                MergePane::Left => chunk.ours_range.clone(),
                MergePane::Right => chunk.theirs_range.clone(),
                MergePane::Center => chunk.base_range.clone(), // approximate
            });

        let background = Canvas::new(MergeDecorationCanvas {
            decorations,
            viewport_scroll: editor.viewport_scroll(),
            line_height: editor.line_height(),
            viewport_height: editor.viewport_height(),
            gutter_width: editor.gutter_width(),
            active_range,
        })
        .width(Length::Fill)
        .height(Length::Fill);

        let editor_view = editor
            .view()
            .map(move |m| MergeEditorEvent::Editor { pane, message: m });

        Container::new(Stack::new().push(background).push(editor_view))
            .width(Length::Fill)
            .height(Length::Fill)
    }

    fn link_map_view(&self, side: LinkMapSide) -> Element<'_, MergeEditorEvent> {
        let blocks = match side {
            LinkMapSide::Left => Arc::clone(&self.left_links),
            LinkMapSide::Right => Arc::clone(&self.right_links),
        };

        let (left_editor, right_editor) = match side {
            LinkMapSide::Left => (&self.left, &self.center),
            LinkMapSide::Right => (&self.center, &self.right),
        };

        Canvas::new(MergeLinkMapCanvas {
            blocks,
            current_chunk: self.current_chunk,
            left_scroll: left_editor.viewport_scroll(),
            right_scroll: right_editor.viewport_scroll(),
            left_line_height: left_editor.line_height(),
            right_line_height: right_editor.line_height(),
        })
        .width(Length::Fixed(LINK_MAP_WIDTH))
        .height(Length::Fill)
        .into()
    }

    fn overview_view(&self) -> Element<'_, MergeEditorEvent> {
        let total_lines = self
            .left_line_count
            .max(self.center_line_count)
            .max(self.right_line_count);
        let editor = &self.left;
        let total_height = content_height(total_lines, editor.line_height()).max(1.0);
        let start = (editor.viewport_scroll() / total_height).clamp(0.0, 1.0);
        let end = ((editor.viewport_scroll() + editor.viewport_height()) / total_height)
            .clamp(start, 1.0);

        Canvas::new(MergeOverviewCanvas {
            blocks: Arc::clone(&self.overview_blocks),
            total_lines,
            viewport_range: start..end,
        })
        .width(Length::Fixed(OVERVIEW_WIDTH))
        .height(Length::Fill)
        .into()
    }

    fn editor(&self, pane: MergePane) -> &CodeEditor {
        match pane {
            MergePane::Left => &self.left,
            MergePane::Center => &self.center,
            MergePane::Right => &self.right,
        }
    }

    fn editor_mut(&mut self, pane: MergePane) -> &mut CodeEditor {
        match pane {
            MergePane::Left => &mut self.left,
            MergePane::Center => &mut self.center,
            MergePane::Right => &mut self.right,
        }
    }

    fn pane_line_count(&self, pane: MergePane) -> usize {
        match pane {
            MergePane::Left => self.left_line_count.max(1),
            MergePane::Center => self.center_line_count.max(1),
            MergePane::Right => self.right_line_count.max(1),
        }
    }
}

// ═══════════════════════════════════════
// Canvas implementations
// ═══════════════════════════════════════

#[derive(Debug, Clone)]
struct MergeDecorationCanvas {
    decorations: Arc<PaneDecorations>,
    viewport_scroll: f32,
    line_height: f32,
    viewport_height: f32,
    gutter_width: f32,
    active_range: Option<Range<usize>>,
}

#[derive(Debug, Default)]
struct MergeDecoCacheState {
    cache: canvas::Cache<Renderer>,
    key: Cell<Option<(i32, i32, i32)>>,
}

impl<Message> canvas::Program<Message> for MergeDecorationCanvas {
    type State = MergeDecoCacheState;

    fn draw(
        &self,
        state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let key = (
            (self.viewport_scroll * 0.5).round() as i32,
            self.active_range
                .as_ref()
                .map(|r| r.start as i32)
                .unwrap_or(-1),
            bounds.width as i32,
        );
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
                    .and_then(|l| l.as_ref())
                else {
                    continue;
                };

                let y = line_index as f32 * self.line_height - self.viewport_scroll;
                let (code_bg, gutter_bg) = merge_block_colors(line.chunk_type, line.resolved);

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

                if self
                    .active_range
                    .as_ref()
                    .is_some_and(|r| line_in_range(r, line_index as f32))
                {
                    frame.fill_rectangle(
                        Point::new(0.0, y),
                        Size::new(bounds.width, self.line_height),
                        theme::darcula::SELECTION_BG.scale_alpha(0.12),
                    );
                }
            }
        });

        vec![geometry]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LinkMapSide {
    Left,
    Right,
}

#[derive(Debug, Clone)]
struct MergeLinkMapCanvas {
    blocks: Arc<[LinkMapBlock]>,
    current_chunk: Option<usize>,
    left_scroll: f32,
    right_scroll: f32,
    left_line_height: f32,
    right_line_height: f32,
}

#[derive(Debug, Default)]
struct MergeLinkCacheState {
    cache: canvas::Cache<Renderer>,
    key: Cell<Option<(i32, i32, Option<usize>)>>,
}

impl<Message> canvas::Program<Message> for MergeLinkMapCanvas {
    type State = MergeLinkCacheState;

    fn draw(
        &self,
        state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let key = (
            (self.left_scroll * 0.5).round() as i32,
            (self.right_scroll * 0.5).round() as i32,
            self.current_chunk,
        );
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

                if left.1 < 0.0
                    || right.1 < 0.0
                    || left.0 > bounds.height
                    || right.0 > bounds.height
                {
                    continue;
                }

                let path = canvas::Path::new(|builder| {
                    builder.move_to(Point::new(0.0, left.0));
                    builder.bezier_curve_to(
                        Point::new(bounds.width * 0.35, left.0),
                        Point::new(bounds.width * 0.65, right.0),
                        Point::new(bounds.width, right.0),
                    );
                    builder.line_to(Point::new(bounds.width, right.1));
                    builder.bezier_curve_to(
                        Point::new(bounds.width * 0.65, right.1),
                        Point::new(bounds.width * 0.35, left.1),
                        Point::new(0.0, left.1),
                    );
                    builder.close();
                });

                let is_active = self.current_chunk == Some(block.chunk_id);
                let fill = merge_link_fill(block.chunk_type, block.resolved, is_active);
                let stroke_color = merge_link_stroke(block.chunk_type, block.resolved);

                frame.fill(&path, fill);
                frame.stroke(
                    &path,
                    canvas::Stroke::default()
                        .with_width(if is_active { 1.3 } else { 0.8 })
                        .with_color(stroke_color),
                );
            }
        });

        vec![geometry]
    }
}

#[derive(Debug, Clone)]
struct MergeOverviewCanvas {
    blocks: Arc<[OverviewBlock]>,
    total_lines: usize,
    viewport_range: Range<f32>,
}

#[derive(Debug, Default)]
struct MergeOverviewCacheState {
    cache: canvas::Cache<Renderer>,
    key: Cell<Option<(usize, i32, i32)>>,
}

impl<Message> canvas::Program<Message> for MergeOverviewCanvas {
    type State = MergeOverviewCacheState;

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

        let key = (
            self.blocks.len(),
            (self.viewport_range.start * 100.0).round() as i32,
            (self.viewport_range.end * 100.0).round() as i32,
        );
        if state.key.get() != Some(key) {
            state.cache.clear();
            state.key.set(Some(key));
        }

        let geometry = state.cache.draw(renderer, bounds.size(), |frame| {
            frame.fill_rectangle(
                Point::ORIGIN,
                bounds.size(),
                theme::darcula::BG_PANEL.scale_alpha(0.94),
            );

            let total = self.total_lines.max(1) as f32;
            for block in self.blocks.iter() {
                let y = block.range.start as f32 / total * drawable_height;
                let height = (((block.range.end.max(block.range.start + 1) - block.range.start)
                    as f32
                    / total)
                    * drawable_height)
                    .max(MIN_OVERVIEW_BLOCK_HEIGHT);
                let color = merge_overview_fill(block.chunk_type, block.resolved);

                frame.fill_rectangle(
                    Point::new(track_x, y + OVERVIEW_PADDING_Y),
                    Size::new(track_width, height),
                    color,
                );
            }

            // Viewport indicator
            let viewport_y = self.viewport_range.start * drawable_height + OVERVIEW_PADDING_Y;
            let viewport_height =
                ((self.viewport_range.end - self.viewport_range.start) * drawable_height).max(8.0);
            let vp_rect = canvas::Path::rectangle(
                Point::new(0.5, viewport_y),
                Size::new(bounds.width - 1.0, viewport_height),
            );
            frame.fill(&vp_rect, theme::darcula::TEXT_PRIMARY.scale_alpha(0.08));
            frame.stroke(
                &vp_rect,
                canvas::Stroke::default()
                    .with_width(1.0)
                    .with_color(theme::darcula::TEXT_SECONDARY.scale_alpha(0.35)),
            );
        });

        vec![geometry]
    }
}

// ═══════════════════════════════════════
// Builder functions
// ═══════════════════════════════════════

fn assemble_center_text(
    model: &MergeEditorModel,
    resolutions: &[Option<ChunkResolution>],
) -> String {
    let mut lines: Vec<String> = Vec::new();

    for (i, chunk) in model.chunks.iter().enumerate() {
        let resolved_lines = match resolutions.get(i).copied().flatten() {
            Some(ChunkResolution::Ours) => &chunk.lines_ours,
            Some(ChunkResolution::Theirs) => &chunk.lines_theirs,
            Some(ChunkResolution::Base) => &chunk.lines_base,
            None => {
                // Unresolved: show conflict markers
                lines.push(format!("<<<<<<< ours"));
                lines.extend(chunk.lines_ours.iter().cloned());
                lines.push("=======".to_string());
                lines.extend(chunk.lines_theirs.iter().cloned());
                lines.push(">>>>>>> theirs".to_string());
                continue;
            }
        };
        lines.extend(resolved_lines.iter().cloned());
    }

    lines.join("\n")
}

fn build_all_decorations(
    model: &MergeEditorModel,
    resolutions: &[Option<ChunkResolution>],
    center_line_count: usize,
) -> (PaneDecorations, PaneDecorations, PaneDecorations) {
    let ours_lines = line_count(&model.ours_text);
    let theirs_lines = line_count(&model.theirs_text);

    let mut left = PaneDecorations {
        lines: vec![None; ours_lines],
    };
    let mut right = PaneDecorations {
        lines: vec![None; theirs_lines],
    };
    let mut center = PaneDecorations {
        lines: vec![None; center_line_count],
    };

    // Decorate left and right based on chunk types
    for (i, chunk) in model.chunks.iter().enumerate() {
        let resolved = resolutions.get(i).copied().flatten().is_some();
        if chunk.chunk_type == MergeChunkType::Equal {
            continue;
        }

        for line_idx in chunk.ours_range.clone() {
            if let Some(slot) = left.lines.get_mut(line_idx) {
                *slot = Some(MergeDecoratedLine {
                    chunk_type: chunk.chunk_type,
                    resolved,
                });
            }
        }

        for line_idx in chunk.theirs_range.clone() {
            if let Some(slot) = right.lines.get_mut(line_idx) {
                *slot = Some(MergeDecoratedLine {
                    chunk_type: chunk.chunk_type,
                    resolved,
                });
            }
        }
    }

    // Decorate center based on resolution state
    let mut center_cursor = 0usize;
    for (i, chunk) in model.chunks.iter().enumerate() {
        let resolution = resolutions.get(i).copied().flatten();
        let resolved = resolution.is_some();
        let num_lines = match resolution {
            Some(ChunkResolution::Ours) => chunk.lines_ours.len(),
            Some(ChunkResolution::Theirs) => chunk.lines_theirs.len(),
            Some(ChunkResolution::Base) => chunk.lines_base.len(),
            None => {
                // Conflict markers: 3 + ours + theirs lines
                chunk.lines_ours.len() + chunk.lines_theirs.len() + 3
            }
        };

        if chunk.chunk_type != MergeChunkType::Equal {
            for line_idx in center_cursor..(center_cursor + num_lines).min(center.lines.len()) {
                if let Some(slot) = center.lines.get_mut(line_idx) {
                    *slot = Some(MergeDecoratedLine {
                        chunk_type: chunk.chunk_type,
                        resolved,
                    });
                }
            }
        }
        center_cursor += num_lines;
    }

    (left, center, right)
}

fn build_all_link_blocks(
    model: &MergeEditorModel,
    resolutions: &[Option<ChunkResolution>],
    _center_line_count: usize,
) -> (Vec<LinkMapBlock>, Vec<LinkMapBlock>) {
    let mut left_blocks = Vec::new();
    let mut right_blocks = Vec::new();
    let mut center_cursor = 0usize;

    for (i, chunk) in model.chunks.iter().enumerate() {
        let resolution = resolutions.get(i).copied().flatten();
        let resolved = resolution.is_some();
        let center_lines = match resolution {
            Some(ChunkResolution::Ours) => chunk.lines_ours.len(),
            Some(ChunkResolution::Theirs) => chunk.lines_theirs.len(),
            Some(ChunkResolution::Base) => chunk.lines_base.len(),
            None => chunk.lines_ours.len() + chunk.lines_theirs.len() + 3,
        };
        let center_range = center_cursor..(center_cursor + center_lines);

        if chunk.chunk_type != MergeChunkType::Equal {
            left_blocks.push(LinkMapBlock {
                chunk_id: chunk.id,
                chunk_type: chunk.chunk_type,
                resolved,
                left_range: chunk.ours_range.clone(),
                right_range: center_range.clone(),
            });

            right_blocks.push(LinkMapBlock {
                chunk_id: chunk.id,
                chunk_type: chunk.chunk_type,
                resolved,
                left_range: center_range.clone(),
                right_range: chunk.theirs_range.clone(),
            });
        }

        center_cursor += center_lines;
    }

    (left_blocks, right_blocks)
}

fn build_overview(
    model: &MergeEditorModel,
    resolutions: &[Option<ChunkResolution>],
) -> Vec<OverviewBlock> {
    model
        .chunks
        .iter()
        .enumerate()
        .filter(|(_, chunk)| chunk.chunk_type != MergeChunkType::Equal)
        .map(|(i, chunk)| {
            let resolved = resolutions.get(i).copied().flatten().is_some();
            let start = chunk
                .ours_range
                .start
                .min(chunk.theirs_range.start)
                .min(chunk.base_range.start);
            let end = chunk
                .ours_range
                .end
                .max(chunk.theirs_range.end)
                .max(chunk.base_range.end)
                .max(start + 1);
            OverviewBlock {
                chunk_type: chunk.chunk_type,
                resolved,
                range: start..end,
            }
        })
        .collect()
}

// ═══════════════════════════════════════
// Helpers
// ═══════════════════════════════════════

const DEFAULT_MERGE_FONT_SIZE: f32 = 13.0;

fn build_editor(content: &str, path_hint: Option<&str>) -> CodeEditor {
    build_editor_sized(content, path_hint, DEFAULT_MERGE_FONT_SIZE)
}

fn build_editor_sized(content: &str, path_hint: Option<&str>, font_size: f32) -> CodeEditor {
    let syntax = path_hint
        .and_then(|p| Path::new(p).extension())
        .and_then(|ext| ext.to_str())
        .unwrap_or("txt");
    let mut editor = CodeEditor::new(content, syntax);
    editor.set_font(theme::code_font());
    editor.set_font_size(font_size, true);
    editor.set_wrap_enabled(false);
    editor.set_line_numbers_enabled(true);
    editor.set_search_replace_enabled(false);
    editor.set_theme(iced_code_editor::theme::Style {
        background: iced::Color::TRANSPARENT,
        text_color: theme::darcula::TEXT_PRIMARY,
        gutter_background: iced::Color::TRANSPARENT,
        gutter_border: iced::Color::TRANSPARENT,
        line_number_color: theme::darcula::TEXT_DISABLED,
        scrollbar_background: theme::darcula::BG_PANEL.scale_alpha(0.65),
        scroller_color: theme::darcula::BORDER.scale_alpha(0.95),
        current_line_highlight: iced::Color::TRANSPARENT,
    });
    editor
}

fn line_count(text: &str) -> usize {
    if text.is_empty() {
        1
    } else {
        text.lines().count().max(1)
    }
}

fn pane_index(pane: MergePane) -> usize {
    match pane {
        MergePane::Left => 0,
        MergePane::Center => 1,
        MergePane::Right => 2,
    }
}

fn is_mutating(message: &EditorMessage) -> bool {
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

// ── Colors ──

fn merge_pane_color(chunk_type: MergeChunkType) -> iced::Color {
    match chunk_type {
        MergeChunkType::Equal => theme::darcula::TEXT_SECONDARY,
        MergeChunkType::OursOnly => iced::Color::from_rgb(0.40, 0.65, 0.95),
        MergeChunkType::TheirsOnly => iced::Color::from_rgb(0.92, 0.44, 0.44),
        MergeChunkType::Conflict => iced::Color::from_rgb(0.75, 0.50, 0.90),
    }
}

fn merge_block_colors(chunk_type: MergeChunkType, resolved: bool) -> (iced::Color, iced::Color) {
    let alpha = if resolved { 0.10 } else { 0.18 };
    let gutter_alpha = if resolved { 0.15 } else { 0.25 };

    match chunk_type {
        MergeChunkType::Equal => (iced::Color::TRANSPARENT, iced::Color::TRANSPARENT),
        MergeChunkType::OursOnly => (
            iced::Color::from_rgba(0.30, 0.55, 0.85, alpha),
            iced::Color::from_rgba(0.30, 0.55, 0.85, gutter_alpha),
        ),
        MergeChunkType::TheirsOnly => (
            iced::Color::from_rgba(0.92, 0.44, 0.44, alpha),
            iced::Color::from_rgba(0.92, 0.44, 0.44, gutter_alpha),
        ),
        MergeChunkType::Conflict => {
            if resolved {
                (
                    iced::Color::from_rgba(0.42, 0.86, 0.50, 0.12),
                    iced::Color::from_rgba(0.42, 0.86, 0.50, 0.18),
                )
            } else {
                (
                    iced::Color::from_rgba(0.75, 0.50, 0.90, alpha),
                    iced::Color::from_rgba(0.75, 0.50, 0.90, gutter_alpha),
                )
            }
        }
    }
}

fn merge_link_fill(chunk_type: MergeChunkType, resolved: bool, active: bool) -> iced::Color {
    let alpha = if active { 0.36 } else { 0.22 };
    if resolved {
        return iced::Color::from_rgba(0.42, 0.86, 0.50, alpha * 0.6);
    }
    match chunk_type {
        MergeChunkType::Equal => iced::Color::TRANSPARENT,
        MergeChunkType::OursOnly => iced::Color::from_rgba(0.30, 0.55, 0.85, alpha),
        MergeChunkType::TheirsOnly => iced::Color::from_rgba(0.92, 0.44, 0.44, alpha),
        MergeChunkType::Conflict => iced::Color::from_rgba(0.75, 0.50, 0.90, alpha),
    }
}

fn merge_link_stroke(chunk_type: MergeChunkType, resolved: bool) -> iced::Color {
    if resolved {
        return iced::Color::from_rgba(0.42, 0.86, 0.50, 0.50);
    }
    match chunk_type {
        MergeChunkType::Equal => iced::Color::TRANSPARENT,
        MergeChunkType::OursOnly => iced::Color::from_rgba(0.30, 0.55, 0.85, 0.70),
        MergeChunkType::TheirsOnly => iced::Color::from_rgba(0.92, 0.44, 0.44, 0.70),
        MergeChunkType::Conflict => iced::Color::from_rgba(0.75, 0.50, 0.90, 0.75),
    }
}

fn merge_overview_fill(chunk_type: MergeChunkType, resolved: bool) -> iced::Color {
    if resolved {
        return iced::Color::from_rgba(0.42, 0.86, 0.50, 0.55);
    }
    match chunk_type {
        MergeChunkType::Equal => iced::Color::TRANSPARENT,
        MergeChunkType::OursOnly => iced::Color::from_rgba(0.30, 0.55, 0.85, 0.65),
        MergeChunkType::TheirsOnly => iced::Color::from_rgba(0.92, 0.44, 0.44, 0.65),
        MergeChunkType::Conflict => iced::Color::from_rgba(0.75, 0.50, 0.90, 0.75),
    }
}

// ── Scroll math ──

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
    let ch = content_height(total_lines, line_height);
    let max_scroll = (ch - viewport_height).max(0.0);
    let anchor_y = anchor_line.max(0.0) * line_height;
    (anchor_y - viewport_height * sync_point).clamp(0.0, max_scroll)
}

fn scale_anchor(anchor: f32, source_total: usize, target_total: usize) -> f32 {
    if source_total <= 1 || target_total <= 1 {
        return 0.0;
    }
    let ratio = anchor / (source_total.saturating_sub(1)) as f32;
    ratio.clamp(0.0, 1.0) * (target_total.saturating_sub(1)) as f32
}

fn content_height(total_lines: usize, line_height: f32) -> f32 {
    total_lines.max(1) as f32 * line_height.max(1.0)
}

fn line_in_range(range: &Range<usize>, line: f32) -> bool {
    if range.is_empty() {
        (line - range.start as f32).abs() < f32::EPSILON
    } else {
        line >= range.start as f32 && line < range.end as f32
    }
}

fn interpolate_in_range(anchor: f32, source: &Range<usize>, target: &Range<usize>) -> f32 {
    if source.is_empty() {
        return target.start as f32;
    }
    if target.is_empty() {
        return target.start as f32;
    }
    let source_len = (source.end - source.start) as f32;
    let target_len = (target.end - target.start) as f32;
    let offset = (anchor - source.start as f32).clamp(0.0, source_len);
    target.start as f32 + offset / source_len.max(1.0) * target_len
}

fn block_visual_bounds(
    range: &Range<usize>,
    line_height: f32,
    scroll: f32,
    minimum_height: f32,
) -> (f32, f32) {
    let start = range.start as f32 * line_height - scroll;
    let height = if range.is_empty() {
        minimum_height
    } else {
        ((range.end - range.start) as f32 * line_height).max(minimum_height)
    };
    (start, start + height)
}
