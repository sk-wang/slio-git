use crate::theme::{self, BadgeTone};
use crate::widgets::{self, OptionalPush};
use git_core::diff::FileDiff;
use iced::widget::{text, Column, Container, Row, Text};
use iced::{Alignment, Element, Length};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DiffFileHeaderMeta {
    pub file_name: String,
    pub parent_path: Option<String>,
    pub rename_hint: Option<String>,
    pub status_label: &'static str,
    pub status_tone: BadgeTone,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffFileToolbarSummary {
    pub meta: DiffFileHeaderMeta,
    pub change_summary: String,
}

impl PartialEq for DiffFileHeaderMeta {
    fn eq(&self, other: &Self) -> bool {
        self.file_name == other.file_name
            && self.parent_path == other.parent_path
            && self.rename_hint == other.rename_hint
            && self.status_label == other.status_label
            && std::mem::discriminant(&self.status_tone)
                == std::mem::discriminant(&other.status_tone)
    }
}

impl Eq for DiffFileHeaderMeta {}

impl DiffFileHeaderMeta {
    pub fn describe(old_path: Option<&str>, new_path: Option<&str>) -> Self {
        let full_label = new_path.or(old_path).unwrap_or("未命名文件");
        let file_name = Path::new(full_label)
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or(full_label)
            .to_string();
        let parent_path = Path::new(full_label)
            .parent()
            .and_then(|value| value.to_str())
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        let rename_hint = match (old_path, new_path) {
            (Some(old), Some(new)) if old != new => Some(format!("{old} -> {new}")),
            _ => None,
        };
        let (status_label, status_tone) = match (old_path, new_path) {
            (None, Some(_)) => ("新文件", BadgeTone::Success),
            (Some(_), None) => ("删除", BadgeTone::Danger),
            (Some(old), Some(new)) if old != new => ("重命名", BadgeTone::Accent),
            _ => ("修改", BadgeTone::Accent),
        };

        Self {
            file_name,
            parent_path,
            rename_hint,
            status_label,
            status_tone,
        }
    }

    pub fn from_file_diff(file_diff: &FileDiff) -> Self {
        Self::describe(file_diff.old_path.as_deref(), file_diff.new_path.as_deref())
    }
}

impl DiffFileToolbarSummary {
    pub fn from_file_diff(file_diff: &FileDiff) -> Self {
        Self {
            meta: DiffFileHeaderMeta::from_file_diff(file_diff),
            change_summary: format!("+{} / -{}", file_diff.additions, file_diff.deletions),
        }
    }
}

pub fn view<'a, Message: Clone + 'static>(
    meta: DiffFileHeaderMeta,
    hunks: usize,
    additions: u32,
    deletions: u32,
) -> Element<'a, Message> {
    let DiffFileHeaderMeta {
        file_name,
        parent_path,
        rename_hint,
        status_label,
        status_tone,
    } = meta;

    Container::new(
        Row::new()
            .spacing(theme::spacing::XS)
            .align_y(Alignment::Center)
            .push(
                Column::new()
                    .spacing(1)
                    .width(Length::Fill)
                    .push(Text::new(file_name).size(12))
                    .push_maybe(parent_path.map(|path| {
                        Text::new(path)
                            .size(10)
                            .width(Length::Fill)
                            .wrapping(text::Wrapping::WordOrGlyph)
                            .color(theme::darcula::TEXT_SECONDARY)
                    }))
                    .push_maybe(rename_hint.map(|hint| {
                        Text::new(hint)
                            .size(10)
                            .width(Length::Fill)
                            .wrapping(text::Wrapping::WordOrGlyph)
                            .color(theme::darcula::TEXT_SECONDARY)
                    })),
            )
            .push(widgets::compact_chip::<Message>(status_label, status_tone))
            .push(
                Text::new(format!("{} 区块 · +{} / -{}", hunks, additions, deletions))
                    .size(10)
                    .color(theme::darcula::TEXT_SECONDARY),
            ),
    )
    .padding([4, 6])
    .style(theme::panel_style(theme::Surface::ToolbarField))
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn describe_paths_marks_renames() {
        let meta = DiffFileHeaderMeta::describe(Some("old/src/lib.rs"), Some("new/src/lib.rs"));

        assert_eq!(meta.file_name, "lib.rs");
        assert_eq!(meta.parent_path.as_deref(), Some("new/src"));
        assert_eq!(
            meta.rename_hint.as_deref(),
            Some("old/src/lib.rs -> new/src/lib.rs")
        );
        assert_eq!(meta.status_label, "重命名");
        assert!(matches!(meta.status_tone, BadgeTone::Accent));
    }

    #[test]
    fn describe_paths_handles_new_files() {
        let meta = DiffFileHeaderMeta::describe(None, Some("src/new.rs"));

        assert_eq!(meta.file_name, "new.rs");
        assert_eq!(meta.parent_path.as_deref(), Some("src"));
        assert_eq!(meta.rename_hint, None);
        assert_eq!(meta.status_label, "新文件");
        assert!(matches!(meta.status_tone, BadgeTone::Success));
    }

    #[test]
    fn toolbar_summary_preserves_single_file_status_and_totals() {
        let file_diff = FileDiff {
            old_path: None,
            new_path: Some("src/new.rs".to_string()),
            hunks: Vec::new(),
            additions: 7,
            deletions: 0,
        };

        let summary = DiffFileToolbarSummary::from_file_diff(&file_diff);

        assert_eq!(summary.meta.file_name, "new.rs");
        assert_eq!(summary.meta.parent_path.as_deref(), Some("src"));
        assert_eq!(summary.meta.status_label, "新文件");
        assert!(matches!(summary.meta.status_tone, BadgeTone::Success));
        assert_eq!(summary.change_summary, "+7 / -0");
    }
}
