//! Styled change list for the redesigned repository shell.

use crate::components::status_icons::FileStatus;
use crate::i18n::I18n;
use crate::theme::{self, BadgeTone, Surface};
use crate::widgets::{self, scrollable, OptionalPush};
use git_core::index::Change;
use iced::widget::{text, Button, Checkbox, Column, Container, Row, Text};
use iced::{Alignment, Element, Length};
use std::path::Path;
use std::rc::Rc;

#[derive(Clone, Copy)]
enum ChangeSectionKind {
    Staged,
    Unstaged,
    Untracked,
}

impl ChangeSectionKind {
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
    on_select: Option<Rc<dyn Fn(String) -> Message + 'a>>,
    on_stage: Option<Rc<dyn Fn(String) -> Message + 'a>>,
    on_unstage: Option<Rc<dyn Fn(String) -> Message + 'a>>,
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
            on_select: None,
            on_stage: None,
            on_unstage: None,
        }
    }

    pub fn with_selected_path(mut self, selected_path: Option<&'a str>) -> Self {
        self.selected_path = selected_path;
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

        let summary = Container::new(
            Column::new()
                .spacing(2)
                .push(
                    Text::new(format!(
                        "{} 工作区修改 · {} 新文件 · {} 待提交",
                        self.unstaged.len(),
                        self.untracked.len(),
                        self.staged.len(),
                    ))
                    .size(10)
                    .width(Length::Fill)
                    .wrapping(text::Wrapping::WordOrGlyph)
                    .color(theme::darcula::TEXT_SECONDARY),
                )
                .push_maybe(self.selected_path.map(|path| {
                    Text::new(format!("当前查看：{path}"))
                        .size(10)
                        .width(Length::Fill)
                        .wrapping(text::Wrapping::WordOrGlyph)
                        .color(theme::darcula::TEXT_SECONDARY)
                })),
        )
        .width(Length::Fill)
        .padding([6, 0]);

        let mut sections = Column::new().spacing(theme::spacing::SM).push(summary);

        if !self.staged.is_empty() {
            sections = sections.push(self.build_section(
                self.i18n.staged_changes,
                self.staged,
                ChangeSectionKind::Staged,
            ));
        }

        if !self.unstaged.is_empty() {
            sections = sections.push(self.build_section(
                self.i18n.unstaged_changes,
                self.unstaged,
                ChangeSectionKind::Unstaged,
            ));
        }

        if !self.untracked.is_empty() {
            sections = sections.push(self.build_section(
                self.i18n.untracked_files,
                self.untracked,
                ChangeSectionKind::Untracked,
            ));
        }

        scrollable::styled(sections).height(Length::Fill).into()
    }

    fn build_section(
        &self,
        title: &'a str,
        changes: &'a [Change],
        kind: ChangeSectionKind,
    ) -> Element<'a, Message> {
        let mut section = Column::new()
            .spacing(theme::spacing::XS)
            .push(
                Column::new().spacing(2).push(
                    Row::new()
                        .spacing(theme::spacing::XS)
                        .align_y(Alignment::Center)
                        .push(
                            Text::new(title)
                                .size(11)
                                .color(theme::darcula::TEXT_SECONDARY),
                        )
                        .push(widgets::info_chip::<Message>(
                            changes.len().to_string(),
                            Self::section_badge_tone(kind),
                        )),
                ),
            )
            .push(iced::widget::rule::horizontal(1));

        for change in changes {
            section = section.push(self.build_change_row(change, kind));
        }

        section.into()
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

        let meta_line = [
            (!parent_path.is_empty()).then_some(parent_path.clone()),
            Some(status.label().to_string()),
            Some(kind.context_label().to_string()),
            is_selected.then_some("当前查看".to_string()),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join(" · ");

        let item_panel = Container::new(
            Row::new()
                .spacing(theme::spacing::XS)
                .align_y(Alignment::Start)
                .push(
                    Container::new(Text::new(status.symbol()).size(11).color(status.color()))
                        .width(Length::Fixed(14.0)),
                )
                .push(
                    Column::new()
                        .spacing(1)
                        .width(Length::Fill)
                        .push(
                            Text::new(file_name)
                                .size(12)
                                .width(Length::Fill)
                                .wrapping(text::Wrapping::WordOrGlyph),
                        )
                        .push(
                            Text::new(meta_line)
                                .size(10)
                                .width(Length::Fill)
                                .wrapping(text::Wrapping::WordOrGlyph)
                                .color(theme::darcula::TEXT_SECONDARY),
                        ),
                ),
        )
        .padding([6, 8])
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
            Button::new(item_panel)
                .width(Length::Fill)
                .style(theme::button_style(theme::ButtonTone::Ghost))
                .on_press(select_message)
                .into()
        } else {
            item_panel.into()
        };

        Row::new()
            .spacing(theme::spacing::XS)
            .align_y(Alignment::Center)
            .push(self.stage_checkbox(change.path.clone(), staged))
            .push(Container::new(selection).width(Length::Fill))
            .into()
    }

    fn section_badge_tone(kind: ChangeSectionKind) -> BadgeTone {
        match kind {
            ChangeSectionKind::Staged => BadgeTone::Success,
            ChangeSectionKind::Unstaged => BadgeTone::Accent,
            ChangeSectionKind::Untracked => BadgeTone::Neutral,
        }
    }

    fn stage_checkbox(&self, path: String, staged: bool) -> Element<'a, Message> {
        let on_stage = self.on_stage.clone();
        let on_unstage = self.on_unstage.clone();

        let checkbox = Checkbox::new(staged)
            .size(14)
            .spacing(0)
            .width(Length::Fixed(18.0))
            .style(theme::checkbox_style());

        if on_stage.is_none() && on_unstage.is_none() {
            return checkbox.into();
        }

        checkbox
            .on_toggle(move |checked| {
                if checked {
                    on_stage
                        .as_ref()
                        .expect("stage handler should exist when checkbox is interactive")(
                        path.clone(),
                    )
                } else {
                    on_unstage
                        .as_ref()
                        .expect("unstage handler should exist when checkbox is interactive")(
                        path.clone(),
                    )
                }
            })
            .into()
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
