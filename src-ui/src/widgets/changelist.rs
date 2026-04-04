//! Styled change list for the redesigned repository shell.
//!
//! Supports two display modes: flat list and directory tree,
//! with collapsible "Staged" and "Unstaged Changes" groups and
//! per-file stage/unstage icon buttons.

use crate::components::status_icons::FileStatus;
use crate::i18n::I18n;
use crate::state::FileDisplayMode;
use crate::theme::{self, BadgeTone, Surface};
use crate::widgets::{self, scrollable};
use git_core::index::Change;
use iced::widget::{mouse_area, text, Button, Column, Container, Row, Space, Text};
use iced::{mouse, Alignment, Element, Length, Point};
use std::collections::BTreeMap;
use std::path::Path;
use std::rc::Rc;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[allow(dead_code)]
enum ChangeSectionKind {
    // IDEA-style sort order: STAGED > UNSTAGED > UNTRACKED
    Staged,
    Unstaged,
    Untracked,
}

impl ChangeSectionKind {
    #[allow(dead_code)]
    fn context_label(self) -> &'static str {
        match self {
            ChangeSectionKind::Staged => "已暂存",
            ChangeSectionKind::Unstaged => "工作区修改",
            ChangeSectionKind::Untracked => "新文件",
        }
    }
}

pub struct ChangesList<'a, Message> {
    i18n: &'a I18n,
    staged: &'a [Change],
    unstaged: &'a [Change],
    untracked: &'a [Change],
    selected_path: Option<&'a str>,
    display_mode: FileDisplayMode,
    staged_collapsed: bool,
    unstaged_collapsed: bool,
    on_select: Option<Rc<dyn Fn(String) -> Message + 'a>>,
    on_stage: Option<Rc<dyn Fn(String) -> Message + 'a>>,
    on_unstage: Option<Rc<dyn Fn(String) -> Message + 'a>>,
    on_context_menu: Option<Rc<dyn Fn(String) -> Message + 'a>>,
    on_track_cursor: Option<Rc<dyn Fn(Point) -> Message + 'a>>,
    on_toggle_display_mode: Option<Message>,
    on_toggle_staged: Option<Message>,
    on_toggle_unstaged: Option<Message>,
}

impl<'a, Message: Clone + 'a> ChangesList<'a, Message> {
    pub fn new(
        i18n: &'a I18n,
        staged: &'a [Change],
        unstaged: &'a [Change],
        untracked: &'a [Change],
    ) -> Self {
        Self {
            i18n,
            staged,
            unstaged,
            untracked,
            selected_path: None,
            display_mode: FileDisplayMode::Flat,
            staged_collapsed: false,
            unstaged_collapsed: false,
            on_select: None,
            on_stage: None,
            on_unstage: None,
            on_context_menu: None,
            on_track_cursor: None,
            on_toggle_display_mode: None,
            on_toggle_staged: None,
            on_toggle_unstaged: None,
        }
    }

    pub fn with_selected_path(mut self, selected_path: Option<&'a str>) -> Self {
        self.selected_path = selected_path;
        self
    }

    pub fn with_display_mode(mut self, mode: FileDisplayMode) -> Self {
        self.display_mode = mode;
        self
    }

    pub fn with_collapsed_state(mut self, staged_collapsed: bool, unstaged_collapsed: bool) -> Self {
        self.staged_collapsed = staged_collapsed;
        self.unstaged_collapsed = unstaged_collapsed;
        self
    }

    pub fn with_select_handler<F>(mut self, handler: F) -> Self
    where
        F: Fn(String) -> Message + 'a,
    {
        self.on_select = Some(Rc::new(handler));
        self
    }

    pub fn with_stage_handler<F>(mut self, handler: F) -> Self
    where
        F: Fn(String) -> Message + 'a,
    {
        self.on_stage = Some(Rc::new(handler));
        self
    }

    pub fn with_unstage_handler<F>(mut self, handler: F) -> Self
    where
        F: Fn(String) -> Message + 'a,
    {
        self.on_unstage = Some(Rc::new(handler));
        self
    }

    pub fn with_context_menu_handler<F>(mut self, handler: F) -> Self
    where
        F: Fn(String) -> Message + 'a,
    {
        self.on_context_menu = Some(Rc::new(handler));
        self
    }

    pub fn with_track_cursor_handler<F>(mut self, handler: F) -> Self
    where
        F: Fn(Point) -> Message + 'a,
    {
        self.on_track_cursor = Some(Rc::new(handler));
        self
    }

    pub fn with_toggle_display_mode(mut self, msg: Message) -> Self {
        self.on_toggle_display_mode = Some(msg);
        self
    }

    pub fn with_toggle_staged(mut self, msg: Message) -> Self {
        self.on_toggle_staged = Some(msg);
        self
    }

    pub fn with_toggle_unstaged(mut self, msg: Message) -> Self {
        self.on_toggle_unstaged = Some(msg);
        self
    }

    pub fn view(&self) -> Element<'a, Message> {
        let total_changes = self.staged.len() + self.unstaged.len() + self.untracked.len();
        if total_changes == 0 {
            return widgets::panel_empty_state(
                self.i18n.changes,
                self.i18n.clean_workspace,
                self.i18n.clean_workspace_detail,
                None,
            );
        }

        let mut sections = Column::new().spacing(2);

        // Staged group
        if !self.staged.is_empty() {
            sections = sections.push(self.build_collapsible_section(
                self.i18n.staged_changes,
                self.staged,
                ChangeSectionKind::Staged,
                self.staged_collapsed,
                self.on_toggle_staged.clone(),
            ));
        }

        // Unstaged + Untracked group (combined under "Unstaged Changes")
        let unstaged_combined: Vec<&Change> = self
            .unstaged
            .iter()
            .chain(self.untracked.iter())
            .collect();
        if !unstaged_combined.is_empty() {
            sections = sections.push(self.build_collapsible_section_refs(
                self.i18n.unstaged_changes,
                &unstaged_combined,
                ChangeSectionKind::Unstaged,
                self.unstaged_collapsed,
                self.on_toggle_unstaged.clone(),
            ));
        }

        let scrollable = scrollable::styled(sections).height(Length::Fill);

        if let Some(handler) = self.on_track_cursor.as_ref() {
            let handle = handler.clone();
            mouse_area(Container::new(scrollable).width(Length::Fill).height(Length::Fill))
                .on_move(move |point| handle(point))
                .interaction(mouse::Interaction::Pointer)
                .into()
        } else {
            scrollable.into()
        }
    }

    /// Build toolbar with display mode toggle
    pub fn toolbar(&self) -> Element<'a, Message> {
        let mode_icon = match self.display_mode {
            FileDisplayMode::Flat => "≡",
            FileDisplayMode::Tree => "▤",
        };
        let _mode_tooltip = match self.display_mode {
            FileDisplayMode::Flat => self.i18n.tree_view,
            FileDisplayMode::Tree => self.i18n.flat_view,
        };

        Row::new()
            .spacing(theme::spacing::XS)
            .align_y(Alignment::Center)
            .push(Text::new(self.i18n.changes).size(12))
            .push(widgets::info_chip::<Message>(
                (self.staged.len() + self.unstaged.len() + self.untracked.len()).to_string(),
                BadgeTone::Neutral,
            ))
            .push(Space::new().width(Length::Fill))
            .push(
                crate::widgets::button::toolbar_icon(
                    mode_icon,
                    self.on_toggle_display_mode.clone(),
                ),
            )
            .into()
    }

    fn build_collapsible_section(
        &self,
        title: &'a str,
        changes: &'a [Change],
        kind: ChangeSectionKind,
        collapsed: bool,
        on_toggle: Option<Message>,
    ) -> Element<'a, Message> {
        let refs: Vec<&Change> = changes.iter().collect();
        self.build_collapsible_section_refs(title, &refs, kind, collapsed, on_toggle)
    }

    fn build_collapsible_section_refs(
        &self,
        title: &'a str,
        changes: &[&'a Change],
        kind: ChangeSectionKind,
        collapsed: bool,
        on_toggle: Option<Message>,
    ) -> Element<'a, Message> {
        let expand_icon = if collapsed { "▶" } else { "▼" };

        let header_row = Row::new()
            .spacing(theme::spacing::XS)
            .align_y(Alignment::Center)
            .push(
                Text::new(expand_icon)
                    .size(10)
                    .color(theme::darcula::TEXT_SECONDARY),
            )
            .push(
                Text::new(title)
                    .size(11)
                    .color(theme::darcula::TEXT_SECONDARY),
            )
            .push(widgets::info_chip::<Message>(
                changes.len().to_string(),
                Self::section_badge_tone(kind),
            ));

        let header: Element<'a, Message> = if let Some(msg) = on_toggle {
            Button::new(header_row)
                .style(theme::button_style(theme::ButtonTone::Ghost))
                .padding([2, 4])
                .on_press(msg)
                .into()
        } else {
            Container::new(header_row).padding([2, 4]).into()
        };

        let mut section = Column::new().spacing(0).push(header);

        if !collapsed {
            match self.display_mode {
                FileDisplayMode::Flat => {
                    for change in changes {
                        section = section.push(self.build_change_row(change, kind));
                    }
                }
                FileDisplayMode::Tree => {
                    section = self.build_tree_rows(section, changes, kind);
                }
            }
        }

        section.into()
    }

    fn build_tree_rows(
        &self,
        mut section: Column<'a, Message>,
        changes: &[&'a Change],
        kind: ChangeSectionKind,
    ) -> Column<'a, Message> {
        // Group files by directory, collecting indices
        let mut dir_groups: BTreeMap<String, Vec<usize>> = BTreeMap::new();
        for (i, change) in changes.iter().enumerate() {
            let dir = Path::new(&change.path)
                .parent()
                .and_then(|p| p.to_str())
                .unwrap_or("")
                .to_string();
            dir_groups.entry(dir).or_default().push(i);
        }

        for (dir, indices) in &dir_groups {
            if !dir.is_empty() {
                // Directory header
                section = section.push(
                    Container::new(
                        Row::new()
                            .spacing(4)
                            .align_y(Alignment::Center)
                            .push(Space::new().width(Length::Fixed(16.0)))
                            .push(
                                Text::new("📁")
                                    .size(10)
                                    .color(theme::darcula::TEXT_DISABLED),
                            )
                            .push(
                                Text::new(dir.clone())
                                    .size(10)
                                    .color(theme::darcula::TEXT_DISABLED),
                            ),
                    )
                    .padding([1, 4]),
                );
            }

            for &idx in indices {
                section = section.push(self.build_change_row(changes[idx], kind));
            }
        }

        section
    }

    fn build_change_row(
        &self,
        change: &'a Change,
        kind: ChangeSectionKind,
    ) -> Element<'a, Message> {
        let status = FileStatus::from(&change.status);
        let staged = matches!(kind, ChangeSectionKind::Staged);
        let is_selected = self.selected_path == Some(change.path.as_str());
        let (file_name, parent_path) = split_path(&change.path);

        // Build the file info row
        let mut info_row = Row::new()
            .spacing(theme::spacing::XS)
            .align_y(Alignment::Center)
            .push(
                Container::new(Text::new(status.symbol()).size(11).color(status.color()))
                    .width(Length::Fixed(14.0)),
            );

        match self.display_mode {
            FileDisplayMode::Flat => {
                // Show filename + parent path
                let mut name_col = Column::new().spacing(1).width(Length::Fill).push(
                    Text::new(file_name)
                        .size(11)
                        .width(Length::Fill)
                        .wrapping(text::Wrapping::WordOrGlyph),
                );
                if !parent_path.is_empty() {
                    name_col = name_col.push(
                        Text::new(parent_path)
                            .size(9)
                            .width(Length::Fill)
                            .wrapping(text::Wrapping::WordOrGlyph)
                            .color(theme::darcula::TEXT_SECONDARY),
                    );
                }
                info_row = info_row.push(name_col);
            }
            FileDisplayMode::Tree => {
                // Show just filename (directory is shown as group header)
                info_row = info_row.push(
                    Text::new(file_name)
                        .size(11)
                        .width(Length::Fill)
                        .wrapping(text::Wrapping::WordOrGlyph),
                );
            }
        }

        // Submodule indicator
        if change.is_submodule {
            if let Some(summary) = &change.submodule_summary {
                info_row = info_row.push(
                    Text::new(format!("⊞ {}", summary))
                        .size(9)
                        .color(theme::darcula::TEXT_DISABLED),
                );
            }
        }

        // Stage/unstage action button ("+" or "−")
        let action_button: Element<'a, Message> = if staged {
            if let Some(handler) = &self.on_unstage {
                let msg = handler(change.path.clone());
                Button::new(Text::new("−").size(11).color(theme::darcula::STATUS_DELETED))
                    .style(theme::button_style(theme::ButtonTone::Ghost))
                    .padding([0, 4])
                    .on_press(msg)
                    .into()
            } else {
                Space::new().width(Length::Fixed(18.0)).into()
            }
        } else if let Some(handler) = &self.on_stage {
            let msg = handler(change.path.clone());
            Button::new(Text::new("+").size(11).color(theme::darcula::STATUS_ADDED))
                .style(theme::button_style(theme::ButtonTone::Ghost))
                .padding([0, 4])
                .on_press(msg)
                .into()
        } else {
            Space::new().width(Length::Fixed(18.0)).into()
        };

        info_row = info_row.push(action_button);

        let item_panel = Container::new(info_row)
            .padding([2, 4])
            .width(Length::Fill)
            .style(theme::panel_style(if is_selected {
                Surface::ListSelection
            } else {
                Surface::ListRow
            }));

        let selection: Element<'a, Message> = if let Some(select_message) = self
            .on_select
            .as_ref()
            .map(|handler| handler(change.path.clone()))
        {
            let mut area = mouse_area(
                Container::new(
                    Button::new(item_panel)
                        .width(Length::Fill)
                        .style(theme::button_style(theme::ButtonTone::Ghost))
                        .on_press(select_message.clone()),
                )
                .width(Length::Fill),
            )
            .on_double_click(select_message)
            .interaction(mouse::Interaction::Pointer);

            if let Some(context_message) = self
                .on_context_menu
                .as_ref()
                .map(|handler| handler(change.path.clone()))
            {
                area = area.on_right_press(context_message);
            }

            area.into()
        } else {
            item_panel.into()
        };

        Container::new(selection).width(Length::Fill).into()
    }

    fn section_badge_tone(kind: ChangeSectionKind) -> BadgeTone {
        match kind {
            ChangeSectionKind::Staged => BadgeTone::Success,
            ChangeSectionKind::Unstaged => BadgeTone::Accent,
            ChangeSectionKind::Untracked => BadgeTone::Neutral,
        }
    }
}

fn split_path(path: &str) -> (String, String) {
    let parsed = Path::new(path);
    let file_name = parsed
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or(path)
        .to_string();
    let parent = parsed
        .parent()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string())
        .unwrap_or_default();

    (file_name, parent)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn change_section_kind_exposes_compact_context_label() {
        assert_eq!(ChangeSectionKind::Staged.context_label(), "已暂存");
        assert_eq!(ChangeSectionKind::Unstaged.context_label(), "工作区修改");
        assert_eq!(ChangeSectionKind::Untracked.context_label(), "新文件");
    }

    #[test]
    fn split_path_returns_file_name_and_parent_directory() {
        assert_eq!(
            split_path("src/ui/main.rs"),
            ("main.rs".to_string(), "src/ui".to_string())
        );
        assert_eq!(
            split_path("Cargo.toml"),
            ("Cargo.toml".to_string(), String::new())
        );
    }
}
