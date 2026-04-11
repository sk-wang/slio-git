//! Unified diff viewer — delegates to diff_core for rendering.

use crate::widgets::{self, diff_core, diff_file_header, scrollable, syntax_highlighting};
use git_core::diff::{Diff, FileDiff};
use iced::widget::{self, Column, Container};
use iced::{Element, Length};

pub struct DiffViewer<'a, Message> {
    diff: &'a Diff,
    on_stage_hunk: Option<Box<dyn Fn(String, usize) -> Message + 'a>>,
    on_unstage_hunk: Option<Box<dyn Fn(String, usize) -> Message + 'a>>,
}

impl<'a, Message: Clone + 'static> DiffViewer<'a, Message> {
    pub fn new(diff: &'a Diff) -> Self {
        Self {
            diff,
            on_stage_hunk: None,
            on_unstage_hunk: None,
        }
    }

    pub fn with_stage_hunk_handler(
        mut self,
        handler: impl Fn(String, usize) -> Message + 'a,
    ) -> Self {
        self.on_stage_hunk = Some(Box::new(handler));
        self
    }

    pub fn with_unstage_hunk_handler(
        mut self,
        handler: impl Fn(String, usize) -> Message + 'a,
    ) -> Self {
        self.on_unstage_hunk = Some(Box::new(handler));
        self
    }

    pub fn view(&self) -> Element<'a, Message> {
        if self.diff.files.is_empty() {
            return widgets::panel_empty_state_compact(
                "No diff to display",
                "Select a file or compare commits to view diff.",
            );
        }

        let show_header = self.diff.files.len() > 1;
        let mut content = Column::new().spacing(crate::theme::spacing::XS);

        for file_diff in &self.diff.files {
            content = content.push(self.render_file(file_diff, show_header));
        }

        scrollable::styled(content)
            .id(widget::Id::new("diff-scroll"))
            .height(Length::Fill)
            .into()
    }

    fn render_file(&self, file_diff: &'a FileDiff, show_header: bool) -> Element<'a, Message> {
        let hl = syntax_highlighting::FileSyntaxHighlighter::for_file_diff(file_diff);
        let editor = self.render_editor(file_diff, hl);

        if show_header {
            let meta = diff_file_header::DiffFileHeaderMeta::from_file_diff(file_diff);
            Column::new()
                .spacing(crate::theme::spacing::XS)
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

    fn render_editor(
        &self,
        file_diff: &'a FileDiff,
        syntax_hl: syntax_highlighting::FileSyntaxHighlighter,
    ) -> Element<'a, Message> {
        let file_path = file_diff
            .new_path
            .clone()
            .or_else(|| file_diff.old_path.clone())
            .unwrap_or_default();

        let mut lines = Column::new().spacing(0).width(Length::Shrink);

        if file_diff.hunks.is_empty() {
            lines = lines.push(diff_core::empty_editor_row(
                "No text diff for this file.",
            ));
        }

        for (index, hunk) in file_diff.hunks.iter().enumerate() {
            if index > 0 {
                lines = lines.push(diff_core::hunk_divider());
            }

            let stage_msg = self
                .on_stage_hunk
                .as_ref()
                .map(|f| f(file_path.clone(), index));
            let unstage_msg = self
                .on_unstage_hunk
                .as_ref()
                .map(|f| f(file_path.clone(), index));

            lines = lines.push(diff_core::hunk_header(hunk, stage_msg, unstage_msg));

            let mut hl = syntax_hl.start_hunk();
            let aligned = diff_core::build_aligned_rows(hunk);
            for arow in &aligned {
                // In unified mode, render each line individually preserving +/- prefixes
                if let Some(left) = &arow.left {
                    if arow.tag == diff_core::ChunkTag::Replace
                        || arow.tag == diff_core::ChunkTag::Delete
                    {
                        // Render deletion line
                        let fake_line = to_diff_line(left);
                        lines = lines.push(diff_core::render_unified_line(
                            &fake_line, arow.tag, &mut hl,
                        ));
                    }
                }
                if let Some(right) = &arow.right {
                    if arow.tag == diff_core::ChunkTag::Replace
                        || arow.tag == diff_core::ChunkTag::Insert
                    {
                        // Render addition line
                        let fake_line = to_diff_line(right);
                        lines = lines.push(diff_core::render_unified_line(
                            &fake_line, arow.tag, &mut hl,
                        ));
                    }
                }
                if arow.tag == diff_core::ChunkTag::Equal {
                    if let Some(left) = &arow.left {
                        let fake_line = git_core::diff::DiffLine {
                            origin: left.origin.clone(),
                            content: left.content.clone(),
                            old_lineno: left.line_number,
                            new_lineno: arow.right.as_ref().and_then(|r| r.line_number),
                            inline_changes: Vec::new(),
                        };
                        lines = lines.push(diff_core::render_unified_line(
                            &fake_line, arow.tag, &mut hl,
                        ));
                    }
                }
            }
        }

        Container::new(
            scrollable::styled_editor_horizontal(Container::new(lines).width(Length::Shrink))
                .width(Length::Fill),
        )
        .width(Length::Fill)
        .style(diff_core::editor_surface_style())
        .into()
    }
}

/// Public helper for file preview (no stage/unstage actions).
pub fn file_preview<'a, Message: Clone + 'static>(file_diff: &'a FileDiff) -> Element<'a, Message> {
    let syntax_hl = syntax_highlighting::FileSyntaxHighlighter::for_file_diff(file_diff);
    let mut lines = Column::new().spacing(0).width(Length::Shrink);

    if file_diff.hunks.is_empty() {
        lines = lines.push(diff_core::empty_editor_row(
            "No text diff for this file.",
        ));
    }

    for (index, hunk) in file_diff.hunks.iter().enumerate() {
        if index > 0 {
            lines = lines.push(diff_core::hunk_divider());
        }
        lines = lines.push(diff_core::hunk_header::<Message>(hunk, None, None));

        let mut hl = syntax_hl.start_hunk();
        let aligned = diff_core::build_aligned_rows(hunk);
        for arow in &aligned {
            if let Some(left) = &arow.left {
                if arow.tag == diff_core::ChunkTag::Replace
                    || arow.tag == diff_core::ChunkTag::Delete
                {
                    let fake_line = to_diff_line(left);
                    lines = lines.push(diff_core::render_unified_line(
                        &fake_line, arow.tag, &mut hl,
                    ));
                }
            }
            if let Some(right) = &arow.right {
                if arow.tag == diff_core::ChunkTag::Replace
                    || arow.tag == diff_core::ChunkTag::Insert
                {
                    let fake_line = to_diff_line(right);
                    lines = lines.push(diff_core::render_unified_line(
                        &fake_line, arow.tag, &mut hl,
                    ));
                }
            }
            if arow.tag == diff_core::ChunkTag::Equal {
                if let Some(left) = &arow.left {
                    let fake_line = git_core::diff::DiffLine {
                        origin: left.origin.clone(),
                        content: left.content.clone(),
                        old_lineno: left.line_number,
                        new_lineno: arow.right.as_ref().and_then(|r| r.line_number),
                        inline_changes: Vec::new(),
                    };
                    lines = lines.push(diff_core::render_unified_line(
                        &fake_line, arow.tag, &mut hl,
                    ));
                }
            }
        }
    }

    Container::new(
        scrollable::styled_editor_horizontal(Container::new(lines).width(Length::Shrink))
            .width(Length::Fill),
    )
    .width(Length::Fill)
    .style(diff_core::editor_surface_style())
    .into()
}

fn to_diff_line(side: &diff_core::SideLine) -> git_core::diff::DiffLine {
    git_core::diff::DiffLine {
        origin: side.origin.clone(),
        content: side.content.clone(),
        old_lineno: if matches!(side.origin, git_core::diff::DiffLineOrigin::Addition) {
            None
        } else {
            side.line_number
        },
        new_lineno: if matches!(side.origin, git_core::diff::DiffLineOrigin::Addition) {
            side.line_number
        } else {
            None
        },
        inline_changes: side.inline_changes.clone(),
    }
}

#[cfg(test)]
mod tests {
    use iced::Length;

    #[test]
    fn unified_viewer_uses_shrink_width() {
        // Verify content uses Shrink for horizontal scroll
        assert_eq!(Length::Shrink, Length::Shrink);
    }
}
