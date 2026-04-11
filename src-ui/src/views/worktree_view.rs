//! Working tree management view.

use crate::i18n::I18n;
use crate::theme::{self, BadgeTone, Surface};
use crate::widgets::{self, button, scrollable};
use git_core::worktree::{self, WorkingTree};
use git_core::Repository;
use iced::widget::{Column, Container, Row, Space, Text};
use iced::{Alignment, Element, Length};

#[derive(Debug, Clone)]
pub enum WorktreeMessage {
    Refresh,
    Remove(String),
    Close,
}

#[derive(Debug, Clone, Default)]
pub struct WorktreeState {
    pub worktrees: Vec<WorkingTree>,
    pub is_loading: bool,
    pub error: Option<String>,
    pub success_message: Option<String>,
}

impl WorktreeState {
    pub fn load_worktrees(&mut self, repo: &Repository) {
        self.is_loading = true;
        self.error = None;

        match worktree::list_worktrees(repo) {
            Ok(wts) => {
                self.worktrees = wts;
                self.is_loading = false;
            }
            Err(e) => {
                self.error = Some(format!("Failed to load worktrees: {e}"));
                self.is_loading = false;
            }
        }
    }

    pub fn remove_worktree(&mut self, repo: &Repository, path: String) {
        self.error = None;
        self.success_message = None;
        let p = std::path::Path::new(&path);
        match worktree::remove_worktree(repo, p) {
            Ok(()) => {
                self.success_message = Some(format!("Removed worktree: {path}"));
                self.load_worktrees(repo);
            }
            Err(e) => {
                self.error = Some(format!("Failed to remove worktree: {e}"));
            }
        }
    }
}

pub fn view<'a>(state: &'a WorktreeState, i18n: &'a I18n) -> Element<'a, WorktreeMessage> {
    let header = Row::new()
        .spacing(theme::spacing::XS)
        .align_y(Alignment::Center)
        .push(Text::new(i18n.wt_title).size(14))
        .push(widgets::info_chip::<WorktreeMessage>(
            state.worktrees.len().to_string(),
            BadgeTone::Neutral,
        ))
        .push(Space::new().width(Length::Fill))
        .push(button::ghost(i18n.refresh, Some(WorktreeMessage::Refresh)))
        .push(button::ghost(i18n.close, Some(WorktreeMessage::Close)));

    let status = if let Some(error) = &state.error {
        Some(widgets::status_banner::<WorktreeMessage>(
            i18n.wt_error,
            error.as_str(),
            BadgeTone::Danger,
        ))
    } else {
        state.success_message.as_ref().map(|msg| {
            widgets::status_banner::<WorktreeMessage>(i18n.wt_done, msg.as_str(), BadgeTone::Success)
        })
    };

    let mut list = Column::new().spacing(theme::spacing::XS);

    if state.worktrees.is_empty() {
        list = list.push(
            Text::new(i18n.wt_no_worktrees)
                .size(12)
                .color(theme::darcula::TEXT_SECONDARY),
        );
    }

    for wt in &state.worktrees {
        let branch_label = wt.branch.as_deref().unwrap_or("(detached)");
        let status_label = if wt.is_main {
            i18n.wt_main
        } else if wt.is_locked {
            i18n.wt_locked
        } else if !wt.is_valid {
            i18n.wt_invalid
        } else {
            i18n.wt_normal
        };

        let mut row = Row::new()
            .spacing(theme::spacing::SM)
            .align_y(Alignment::Center)
            .push(
                Column::new()
                    .spacing(1)
                    .width(Length::Fill)
                    .push(Text::new(&wt.name).size(12))
                    .push(
                        Text::new(wt.path.to_string_lossy().to_string())
                            .size(10)
                            .color(theme::darcula::TEXT_DISABLED),
                    )
                    .push(
                        Row::new()
                            .spacing(theme::spacing::XS)
                            .push(widgets::compact_chip::<WorktreeMessage>(
                                branch_label.to_string(),
                                BadgeTone::Accent,
                            ))
                            .push(widgets::compact_chip::<WorktreeMessage>(
                                status_label,
                                if wt.is_valid {
                                    BadgeTone::Neutral
                                } else {
                                    BadgeTone::Warning
                                },
                            )),
                    ),
            );

        if !wt.is_main {
            row = row.push(button::ghost(
                i18n.wt_remove_btn,
                Some(WorktreeMessage::Remove(
                    wt.path.to_string_lossy().to_string(),
                )),
            ));
        }

        list = list.push(
            Container::new(row)
                .padding([8, 10])
                .style(theme::panel_style(Surface::Raised)),
        );
    }

    let mut content = Column::new().spacing(theme::spacing::SM).push(
        Container::new(header)
            .padding(theme::density::SECONDARY_BAR_PADDING)
            .style(theme::panel_style(Surface::Toolbar)),
    );

    if let Some(s) = status {
        content = content.push(s);
    }

    content = content.push(
        Container::new(scrollable::styled(list).height(Length::Fill))
            .padding([8, 8])
            .height(Length::Fill),
    );

    Container::new(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(theme::frame_style(Surface::Editor))
        .into()
}
