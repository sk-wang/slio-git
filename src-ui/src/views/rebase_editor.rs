//! Rebase editor view.
//!
//! Provides a dialog for editing the rebase todo list.

use crate::i18n::I18n;
use crate::theme::{self, BadgeTone, Surface};
use crate::widgets::{self, button, scrollable, text_input, OptionalPush};
use git_core::Repository;
use iced::widget::{mouse_area, text, Button, Column, Container, Row, Text};
use iced::{Alignment, Color, Element, Length};

const FIRST_TODO_ACTIONS: [&str; 4] = ["pick", "reword", "edit", "drop"];
const OTHER_TODO_ACTIONS: [&str; 6] = ["pick", "reword", "edit", "fixup", "squash", "drop"];

/// Message types for rebase editor.
#[derive(Debug, Clone)]
pub enum RebaseEditorMessage {
    SetBaseBranch(String),
    StartRebase,
    ContinueRebase,
    SkipCommit,
    AbortRebase,
    OpenAmendForCurrentStep,
    CycleTodoAction(usize),
    SetTodoAction(usize, String),
    MoveTodoUp(usize),
    MoveTodoDown(usize),
    // Inline message editing (T045)
    StartInlineEdit(usize),
    InlineEditChanged(String),
    ConfirmInlineEdit,
    CancelInlineEdit,
    // Right-click context menu (T046)
    OpenTodoContextMenu(usize),
    CloseTodoContextMenu,
    Refresh,
    Close,
}

/// A single rebase todo item.
#[derive(Debug, Clone)]
pub struct RebaseTodoItem {
    pub action: String,
    pub commit: String,
    pub message: String,
}

/// State for the rebase editor.
#[derive(Debug, Clone)]
pub struct RebaseEditorState {
    pub base_branch: String,
    pub onto_branch: String,
    pub current_step: u32,
    pub total_steps: u32,
    pub is_rebasing: bool,
    pub has_conflicts: bool,
    pub todo_list: Vec<RebaseTodoItem>,
    pub todo_base_ref: Option<String>,
    pub todo_is_editable: bool,
    pub current_step_item: Option<RebaseTodoItem>,
    pub is_loading: bool,
    pub error: Option<String>,
    pub success_message: Option<String>,
    /// Index of the todo item currently being inline-edited (T045)
    pub inline_edit_index: Option<usize>,
    /// Current inline edit text
    pub inline_edit_text: String,
    /// Index of the todo item with open context menu (T046)
    pub context_menu_index: Option<usize>,
    /// Selected todo item index for detail panel (T043)
    pub selected_todo_index: Option<usize>,
}

impl RebaseEditorState {
    pub fn new() -> Self {
        Self {
            base_branch: String::new(),
            onto_branch: String::new(),
            current_step: 0,
            total_steps: 0,
            is_rebasing: false,
            has_conflicts: false,
            todo_list: Vec::new(),
            todo_base_ref: None,
            todo_is_editable: false,
            current_step_item: None,
            is_loading: false,
            error: None,
            success_message: None,
            inline_edit_index: None,
            inline_edit_text: String::new(),
            context_menu_index: None,
            selected_todo_index: None,
        }
    }

    pub fn clear_draft_context(&mut self) {
        self.base_branch.clear();
        self.todo_base_ref = None;
        self.todo_list.clear();
        self.todo_is_editable = false;
        self.current_step_item = None;
        self.onto_branch.clear();
    }

    pub fn has_interactive_draft(&self) -> bool {
        self.todo_is_editable && !self.todo_list.is_empty() && !self.base_branch.trim().is_empty()
    }

    pub fn load_status(&mut self, repo: &Repository, i18n: &I18n) {
        match git_core::rebase::get_rebase_status(repo) {
            Ok(Some(status)) => {
                self.is_rebasing = true;
                self.current_step = status.current_step;
                self.total_steps = status.total_steps;
                self.has_conflicts = git_core::rebase::has_rebase_conflicts(repo).unwrap_or(false);
                self.todo_is_editable = false;
                self.current_step_item = git_core::rebase::get_current_rebase_step(repo)
                    .ok()
                    .flatten()
                    .map(|item| RebaseTodoItem {
                        action: item.action,
                        commit: item.commit,
                        message: item.message,
                    });
                match git_core::rebase::get_rebase_todo(repo) {
                    Ok(todo_list) => {
                        self.todo_list = todo_list
                            .into_iter()
                            .map(|item| RebaseTodoItem {
                                action: item.action,
                                commit: item.commit,
                                message: item.message,
                            })
                            .collect();
                    }
                    Err(error) => {
                        self.error = Some(i18n.re_read_todo_failed_fmt.replace("{}", &error.to_string()));
                        self.success_message = None;
                    }
                }
            }
            Ok(None) => {
                self.is_rebasing = false;
                self.current_step = 0;
                self.total_steps = 0;
                self.has_conflicts = false;
                self.current_step_item = None;
                if !self.todo_is_editable {
                    self.base_branch.clear();
                    self.todo_base_ref = None;
                    self.todo_list.clear();
                }
            }
            Err(error) => {
                self.error = Some(i18n.re_get_status_failed_fmt.replace("{}", &error.to_string()));
                self.success_message = None;
                self.has_conflicts = false;
            }
        }
    }

    pub fn prepare_interactive_rebase(&mut self, repo: &Repository, commit_id: String, i18n: &I18n) {
        self.is_loading = true;
        self.error = None;
        self.success_message = None;

        match git_core::rebase::prepare_interactive_rebase_plan(repo, &commit_id) {
            Ok(plan) => {
                self.base_branch = plan.start_commit;
                self.todo_base_ref = plan.base_ref;
                self.todo_list = plan
                    .entries
                    .into_iter()
                    .map(|item| RebaseTodoItem {
                        action: item.action,
                        commit: item.commit,
                        message: item.message,
                    })
                    .collect();
                self.todo_is_editable = true;
                self.onto_branch.clear();
                self.is_rebasing = false;
                self.current_step = 0;
                self.total_steps = self.todo_list.len() as u32;
                self.has_conflicts = false;
                self.current_step_item = None;
                self.success_message = Some(
                    i18n.re_loaded_todo_fmt
                        .replace("{}", short_commit_id(&self.base_branch))
                        .replacen("{}", &self.todo_list.len().to_string(), 1)
                );
            }
            Err(error) => {
                self.error = Some(i18n.re_prepare_failed_fmt.replace("{}", &error.to_string()));
            }
        }

        self.is_loading = false;
    }

    pub fn start_rebase(&mut self, repo: &Repository, i18n: &I18n) {
        self.is_loading = true;
        self.error = None;
        self.success_message = None;

        if self.has_interactive_draft() {
            let entries = self
                .todo_list
                .iter()
                .map(|item| git_core::rebase::RebaseTodoEntry {
                    action: item.action.clone(),
                    commit: item.commit.clone(),
                    message: item.message.clone(),
                })
                .collect::<Vec<_>>();

            match git_core::rebase::start_interactive_rebase(
                repo,
                self.todo_base_ref.as_deref(),
                &entries,
            ) {
                Ok(message) => {
                    self.todo_is_editable = false;
                    self.load_status(repo, i18n);
                    self.success_message = Some(if self.is_rebasing {
                        if self.has_conflicts {
                            i18n.re_started_with_conflicts.to_string()
                        } else {
                            i18n.re_started_continue.to_string()
                        }
                    } else {
                        message
                    });
                }
                Err(error) => {
                    self.error = Some(i18n.re_start_interactive_failed_fmt.replace("{}", &error.to_string()));
                }
            }

            self.is_loading = false;
            return;
        }

        let onto_branch = self.onto_branch.trim().to_string();
        if onto_branch.is_empty() {
            self.error = Some(i18n.re_onto_branch_empty.to_string());
            self.is_loading = false;
            return;
        }

        match git_core::rebase::rebase_start(repo, &onto_branch) {
            Ok(_) => {
                self.is_rebasing = true;
                self.load_status(repo, i18n);
                self.success_message = Some(if self.has_conflicts {
                    i18n.re_started_onto_conflict_fmt.replace("{}", &onto_branch.to_string())
                } else {
                    i18n.re_started_onto_fmt.replace("{}", &onto_branch.to_string())
                });
            }
            Err(error) => {
                self.error = Some(i18n.re_start_failed_fmt.replace("{}", &error.to_string()));
            }
        }
        self.is_loading = false;
    }

    pub fn continue_rebase(&mut self, repo: &Repository, i18n: &I18n) {
        if self.has_conflicts {
            self.error = Some(i18n.re_conflicts_pending.to_string());
            self.success_message = None;
            return;
        }

        self.is_loading = true;
        self.error = None;
        self.success_message = None;

        match git_core::rebase::rebase_continue(repo) {
            Ok(result) => {
                self.load_status(repo, i18n);
                if result.success {
                    self.success_message = Some(if self.is_rebasing {
                        i18n.re_advanced_step_fmt.replace("{}", &self.current_step.to_string()).replacen("{}", &self.total_steps.to_string(), 1)
                    } else {
                        i18n.re_rebase_complete.to_string()
                    });
                } else {
                    self.error = Some(if result.message.trim().is_empty() {
                        i18n.re_continue_failed_check.to_string()
                    } else {
                        i18n.re_continue_failed_fmt.replace("{}", result.message.trim())
                    });
                }
            }
            Err(error) => {
                self.error = Some(i18n.re_continue_error_fmt.replace("{}", &error.to_string()));
            }
        }
        self.is_loading = false;
    }

    pub fn skip_commit(&mut self, repo: &Repository, i18n: &I18n) {
        self.is_loading = true;
        self.error = None;
        self.success_message = None;

        match git_core::rebase::rebase_skip(repo) {
            Ok(result) => {
                self.load_status(repo, i18n);
                if result.success {
                    self.success_message = Some(if self.is_rebasing {
                        i18n.re_skipped_step_fmt.replace("{}", &self.current_step.to_string()).replacen("{}", &self.total_steps.to_string(), 1)
                    } else {
                        i18n.re_skipped_last.to_string()
                    });
                } else {
                    self.error = Some(if result.message.trim().is_empty() {
                        i18n.re_skip_failed_check.to_string()
                    } else {
                        i18n.re_skip_failed_fmt.replace("{}", result.message.trim())
                    });
                }
            }
            Err(error) => {
                self.error = Some(i18n.re_skip_error_fmt.replace("{}", &error.to_string()));
            }
        }
        self.is_loading = false;
    }

    pub fn abort_rebase(&mut self, repo: &Repository, i18n: &I18n) {
        self.is_loading = true;
        self.error = None;
        self.success_message = None;

        match git_core::rebase::rebase_abort(repo) {
            Ok(_) => {
                self.is_rebasing = false;
                self.load_status(repo, i18n);
                self.success_message = Some(i18n.re_aborted.to_string());
            }
            Err(error) => {
                self.error = Some(i18n.re_abort_failed_fmt.replace("{}", &error.to_string()));
            }
        }
        self.is_loading = false;
    }

    pub fn cycle_todo_action(&mut self, index: usize) {
        if !self.todo_is_editable || index >= self.todo_list.len() {
            return;
        }

        let actions = allowed_todo_actions(index);
        let current = self.todo_list[index].action.to_lowercase();
        let next_index = actions
            .iter()
            .position(|action| *action == current)
            .map(|position| (position + 1) % actions.len())
            .unwrap_or(0);
        self.todo_list[index].action = actions[next_index].to_string();
        self.error = None;
    }

    pub fn move_todo_up(&mut self, index: usize) {
        if !self.todo_is_editable || index == 0 || index >= self.todo_list.len() {
            return;
        }

        self.todo_list.swap(index, index - 1);
        self.normalize_todo_constraints();
        self.error = None;
    }

    pub fn move_todo_down(&mut self, index: usize) {
        if !self.todo_is_editable || index + 1 >= self.todo_list.len() {
            return;
        }

        self.todo_list.swap(index, index + 1);
        self.normalize_todo_constraints();
        self.error = None;
    }

    /// Set a specific action for a todo item (T046 context menu)
    pub fn set_todo_action(&mut self, index: usize, action: String) {
        if !self.todo_is_editable || index >= self.todo_list.len() {
            return;
        }
        self.todo_list[index].action = action;
        self.normalize_todo_constraints();
        self.error = None;
        self.context_menu_index = None;
    }

    /// Start inline editing of a todo item's message (T045)
    pub fn start_inline_edit(&mut self, index: usize) {
        if index < self.todo_list.len() {
            self.inline_edit_text = self.todo_list[index].message.clone();
            self.inline_edit_index = Some(index);
        }
    }

    /// Confirm inline edit and update the message
    pub fn confirm_inline_edit(&mut self) {
        if let Some(index) = self.inline_edit_index {
            if index < self.todo_list.len() {
                self.todo_list[index].message = self.inline_edit_text.clone();
            }
        }
        self.inline_edit_index = None;
        self.inline_edit_text.clear();
    }

    /// Cancel inline edit
    pub fn cancel_inline_edit(&mut self) {
        self.inline_edit_index = None;
        self.inline_edit_text.clear();
    }

    fn normalize_todo_constraints(&mut self) {
        if let Some(first) = self.todo_list.first_mut() {
            let action = first.action.to_lowercase();
            if action == "fixup" || action == "squash" {
                first.action = "pick".to_string();
            }
        }
    }
}

impl Default for RebaseEditorState {
    fn default() -> Self {
        Self::new()
    }
}

fn build_rebase_controls<'a>(state: &'a RebaseEditorState, i18n: &'a I18n) -> Element<'a, RebaseEditorMessage> {
    if state.is_rebasing {
        // In-progress rebase: continue/skip/abort
        let row = Row::new()
            .spacing(theme::spacing::XS)
            .push_maybe(
                state
                    .current_step_item
                    .as_ref()
                    .filter(|item| item.action.eq_ignore_ascii_case("edit"))
                    .and((!state.is_loading && !state.has_conflicts).then_some(()))
                    .map(|_| {
                        button::warning(
                            i18n.re_edit_commit_message,
                            Some(RebaseEditorMessage::OpenAmendForCurrentStep),
                        )
                    }),
            )
            .push(button::primary(
                i18n.re_continue_btn,
                (!state.is_loading && !state.has_conflicts)
                    .then_some(RebaseEditorMessage::ContinueRebase),
            ))
            .push(button::secondary(
                i18n.re_skip_btn,
                (!state.is_loading).then_some(RebaseEditorMessage::SkipCommit),
            ))
            .push(button::ghost(
                i18n.re_abort_btn,
                (!state.is_loading).then_some(RebaseEditorMessage::AbortRebase),
            ));

        scrollable::styled_horizontal(row)
            .width(Length::Fill)
            .into()
    } else if state.has_interactive_draft() {
        // IDEA-style toolbar: [↑上移] [↓下移] | [Pick] [Edit] | [开始] [重置]
        let selected = state.selected_todo_index;
        let has_selected = selected.is_some();
        let can_move_up = selected.is_some_and(|i| i > 0);
        let can_move_down = selected.is_some_and(|i| i + 1 < state.todo_list.len());

        scrollable::styled_horizontal(
            Row::new()
                .spacing(theme::spacing::XS)
                .push(button::toolbar_icon(
                    "↑",
                    can_move_up.then(|| RebaseEditorMessage::MoveTodoUp(selected.unwrap())),
                ))
                .push(button::toolbar_icon(
                    "↓",
                    can_move_down.then(|| RebaseEditorMessage::MoveTodoDown(selected.unwrap())),
                ))
                .push(Text::new("│").size(12).color(theme::darcula::SEPARATOR))
                .push(button::toolbar_icon(
                    "Pick",
                    (has_selected && state.todo_is_editable).then(|| {
                        RebaseEditorMessage::SetTodoAction(selected.unwrap(), "pick".to_string())
                    }),
                ))
                .push(button::toolbar_icon(
                    "Edit",
                    (has_selected && state.todo_is_editable).then(|| {
                        RebaseEditorMessage::SetTodoAction(selected.unwrap(), "edit".to_string())
                    }),
                ))
                .push(Text::new("│").size(12).color(theme::darcula::SEPARATOR))
                .push(button::primary(
                    i18n.re_start_interactive_btn,
                    (!state.is_loading).then_some(RebaseEditorMessage::StartRebase),
                )),
        )
        .width(Length::Fill)
        .into()
    } else {
        scrollable::styled_horizontal(
            Row::new().spacing(theme::spacing::XS).push(button::primary(
                i18n.re_start_rebase_btn,
                (!state.is_loading && !state.onto_branch.trim().is_empty())
                    .then_some(RebaseEditorMessage::StartRebase),
            )),
        )
        .width(Length::Fill)
        .into()
    }
}

fn build_todo_list<'a>(state: &'a RebaseEditorState, i18n: &'a I18n) -> Element<'a, RebaseEditorMessage> {
    if state.todo_list.is_empty() {
        return Column::new()
            .push(
                Text::new(if state.is_rebasing {
                    i18n.re_no_todo_rebasing
                } else {
                    i18n.re_no_todo_idle
                })
                .size(12)
                .color(theme::darcula::TEXT_SECONDARY),
            )
            .into();
    }

    // IDEA-style 3-column table: 操作 | 哈希 | 消息
    let header = Row::new()
        .spacing(0)
        .push(
            Container::new(
                Text::new(i18n.re_col_action)
                    .size(10)
                    .color(theme::darcula::TEXT_DISABLED),
            )
            .width(Length::Fixed(80.0))
            .padding([2, 8]),
        )
        .push(
            Container::new(
                Text::new(i18n.re_col_hash)
                    .size(10)
                    .color(theme::darcula::TEXT_DISABLED),
            )
            .width(Length::Fixed(80.0))
            .padding([2, 4]),
        )
        .push(
            Container::new(
                Text::new(i18n.re_col_message)
                    .size(10)
                    .color(theme::darcula::TEXT_DISABLED),
            )
            .width(Length::Fill)
            .padding([2, 4]),
        );

    let mut table = Column::new().spacing(0).push(header);

    for (index, item) in state.todo_list.iter().enumerate() {
        let is_selected = state.selected_todo_index == Some(index);
        let is_editing = state.inline_edit_index == Some(index);

        // Column 1: Action (clickable to cycle)
        let action_cell: Element<'_, RebaseEditorMessage> = if state.todo_is_editable {
            Button::new(
                Text::new(todo_action_label(&item.action))
                    .size(11)
                    .color(todo_action_color(&item.action)),
            )
            .style(theme::button_style(theme::ButtonTone::Ghost))
            .padding([2, 4])
            .on_press(RebaseEditorMessage::CycleTodoAction(index))
            .into()
        } else {
            Text::new(todo_action_label(&item.action))
                .size(11)
                .color(todo_action_color(&item.action))
                .into()
        };

        // Column 2: Short hash
        let hash_cell = Text::new(short_commit_id(&item.commit))
            .size(11)
            .color(theme::darcula::TEXT_DISABLED);

        // Column 3: Message (inline editable on double-click)
        let message_cell: Element<'_, RebaseEditorMessage> = if is_editing {
            text_input::styled(
                i18n.re_commit_msg_placeholder,
                &state.inline_edit_text,
                RebaseEditorMessage::InlineEditChanged,
            )
            .into()
        } else {
            // Double-click to start editing
            mouse_area(
                Text::new(&item.message)
                    .size(11)
                    .width(Length::Fill)
                    .wrapping(text::Wrapping::WordOrGlyph),
            )
            .on_double_click(RebaseEditorMessage::StartInlineEdit(index))
            .interaction(iced::mouse::Interaction::Text)
            .into()
        };

        let row = Row::new()
            .spacing(0)
            .align_y(Alignment::Center)
            .push(
                Container::new(action_cell)
                    .width(Length::Fixed(80.0))
                    .padding([3, 8]),
            )
            .push(
                Container::new(hash_cell)
                    .width(Length::Fixed(80.0))
                    .padding([3, 4]),
            )
            .push(
                Container::new(message_cell)
                    .width(Length::Fill)
                    .padding([3, 4]),
            );

        let row_surface = if is_selected {
            Surface::ListSelection
        } else {
            Surface::ListRow
        };

        // Right-click context menu support
        let row_area = mouse_area(
            Container::new(row)
                .width(Length::Fill)
                .style(theme::panel_style(row_surface)),
        )
        .on_right_press(RebaseEditorMessage::OpenTodoContextMenu(index))
        .interaction(iced::mouse::Interaction::Pointer);

        table = table.push(row_area);
    }

    table.into()
}

/// Get color for a rebase action label
fn todo_action_color(action: &str) -> Color {
    match action.to_lowercase().as_str() {
        "pick" => theme::darcula::TEXT_PRIMARY,
        "reword" => theme::darcula::ACCENT,
        "edit" => theme::darcula::WARNING,
        "squash" | "fixup" => theme::darcula::STATUS_MODIFIED,
        "drop" => theme::darcula::STATUS_DELETED,
        _ => theme::darcula::TEXT_SECONDARY,
    }
}

fn build_progress<'a>(state: &'a RebaseEditorState, i18n: &'a I18n) -> Element<'a, RebaseEditorMessage> {
    if state.is_rebasing {
        let pct = if state.total_steps > 0 {
            (state.current_step as f32 / state.total_steps as f32) * 100.0
        } else {
            0.0
        };
        let progress_text = i18n.re_progress_fmt
            .replace("{}", &state.current_step.to_string())
            .replacen("{}", &state.total_steps.to_string(), 1)
            .replacen("{}", &format!("{:.0}", pct), 1);

        Container::new(
            Column::new()
                .spacing(theme::spacing::SM)
                .push(widgets::section_header(
                    i18n.re_section_progress.to_uppercase(),
                    i18n.re_progress_title,
                    i18n.re_progress_detail,
                ))
                .push(widgets::info_chip::<RebaseEditorMessage>(
                    progress_text,
                    if state.has_conflicts {
                        BadgeTone::Danger
                    } else {
                        BadgeTone::Warning
                    },
                ))
                .push(build_status_panel::<RebaseEditorMessage>(
                    i18n.re_next_step,
                    if state.has_conflicts {
                        i18n.re_next_resolve_conflicts
                    } else if state
                        .current_step_item
                        .as_ref()
                        .is_some_and(|item| item.action.eq_ignore_ascii_case("edit"))
                    {
                        i18n.re_next_edit_amend
                    } else if state.current_step < state.total_steps {
                        i18n.re_next_ready
                    } else {
                        i18n.re_next_last_step
                    },
                    if state.has_conflicts {
                        BadgeTone::Danger
                    } else {
                        BadgeTone::Accent
                    },
                ))
                .push_maybe(state.current_step_item.as_ref().map(|item| {
                    build_status_panel::<RebaseEditorMessage>(
                        i18n.re_current_step,
                        format!(
                            "{} {} {}",
                            todo_action_label(&item.action),
                            short_commit_id(&item.commit),
                            item.message
                        ),
                        todo_action_tone(&item.action),
                    )
                }))
                .push(scrollable::styled(build_todo_list(state, i18n)).height(Length::Fixed(200.0))),
        )
        .padding([12, 12])
        .style(theme::panel_style(Surface::Panel))
        .into()
    } else if state.has_interactive_draft() {
        Container::new(
            Column::new()
                .spacing(theme::spacing::SM)
                .push(widgets::section_header(
                    i18n.re_section_todo.to_uppercase(),
                    i18n.re_todo_title,
                    i18n.re_todo_detail,
                ))
                .push(build_status_panel::<RebaseEditorMessage>(
                    i18n.re_edit_rules,
                    i18n.re_edit_rules_detail,
                    BadgeTone::Accent,
                ))
                .push(scrollable::styled(build_todo_list(state, i18n)).height(Length::Fixed(260.0))),
        )
        .padding([12, 12])
        .style(theme::panel_style(Surface::Panel))
        .into()
    } else {
        // Idle state: no separate progress panel — branch input area is enough
        Column::new().into()
    }
}

pub fn view<'a>(state: &'a RebaseEditorState, i18n: &'a I18n) -> Element<'a, RebaseEditorMessage> {
    // IDEA-style: compact loading indicator when processing
    let status_panel: Option<Element<'_, RebaseEditorMessage>> = if state.is_loading {
        Some(
            Container::new(
                Row::new()
                    .spacing(theme::spacing::SM)
                    .align_y(Alignment::Center)
                    .push(widgets::loading_spinner::<RebaseEditorMessage>())
                    .push(
                        Text::new(i18n.re_loading)
                            .size(12)
                            .color(theme::darcula::TEXT_SECONDARY),
                    ),
            )
            .padding([8, 12])
            .style(theme::panel_style(Surface::Raised))
            .into(),
        )
    } else if let Some(error) = state.error.as_ref() {
        Some(build_status_panel::<RebaseEditorMessage>(
            i18n.re_status_failed,
            error,
            BadgeTone::Danger,
        ))
    } else if state.is_rebasing && state.has_conflicts {
        Some(build_status_panel::<RebaseEditorMessage>(
            i18n.re_conflict_blocked,
            i18n.re_conflict_blocked_detail,
            BadgeTone::Danger,
        ))
    } else if let Some(message) = state.success_message.as_ref() {
        Some(build_status_panel::<RebaseEditorMessage>(
            i18n.re_status_done,
            message,
            BadgeTone::Success,
        ))
    } else if state.is_rebasing {
        Some(build_status_panel::<RebaseEditorMessage>(
            i18n.re_status_in_progress,
            i18n.re_in_progress_detail,
            BadgeTone::Warning,
        ))
    } else if state.has_interactive_draft() {
        Some(build_status_panel::<RebaseEditorMessage>(
            i18n.re_status_pending,
            i18n.re_pending_detail_fmt.replace("{}", &state.todo_list.len().to_string()),
            BadgeTone::Accent,
        ))
    } else {
        // Idle: no status panel needed — the branch input makes the state obvious
        None
    };

    let branch_inputs: Element<'_, RebaseEditorMessage> =
        if !state.is_rebasing && !state.has_interactive_draft() {
            // Compact: input + hint in one block, no separate "目标" section header
            Container::new(
                Column::new()
                    .spacing(theme::spacing::SM)
                    .push(
                        Text::new(i18n.re_target_branch)
                            .size(11)
                            .color(theme::darcula::TEXT_SECONDARY),
                    )
                    .push(text_input::styled(
                        i18n.re_onto_placeholder,
                        &state.onto_branch,
                        RebaseEditorMessage::SetBaseBranch,
                    ))
                    .push(
                        Text::new(i18n.re_from_history_hint)
                            .size(10)
                            .color(theme::darcula::TEXT_SECONDARY)
                            .wrapping(text::Wrapping::WordOrGlyph),
                    ),
            )
            .padding([12, 12])
            .style(theme::panel_style(Surface::Panel))
            .into()
        } else {
            Column::new().into()
        };

    let context_panel: Option<Element<'_, RebaseEditorMessage>> =
        (!state.base_branch.trim().is_empty()).then(|| {
            Container::new(
            Column::new()
                .spacing(theme::spacing::XS)
                .push(widgets::section_header(
                    i18n.re_section_context.to_uppercase(),
                    i18n.re_context_title,
                    i18n.re_context_detail,
                ))
                .push(
                    Row::new()
                        .spacing(theme::spacing::XS)
                        .push(widgets::info_chip::<RebaseEditorMessage>(
                            i18n.re_start_point_fmt.replace("{}", short_commit_id(&state.base_branch)),
                            BadgeTone::Accent,
                        ))
                        .push(widgets::info_chip::<RebaseEditorMessage>(
                            state.todo_base_ref.as_deref().map_or_else(
                                || i18n.re_from_root.to_string(),
                                |base| i18n.re_base_point_fmt.replace("{}", short_commit_id(base)),
                            ),
                            BadgeTone::Neutral,
                        )),
                )
                .push(
                    Text::new(if state.todo_is_editable {
                        i18n.re_editable_hint
                    } else {
                        i18n.re_viewing_hint
                    })
                    .size(11)
                    .width(Length::Fill)
                    .wrapping(text::Wrapping::WordOrGlyph)
                    .color(theme::darcula::TEXT_SECONDARY),
                ),
        )
        .padding([12, 12])
        .style(theme::panel_style(Surface::Panel))
        .into()
        });

    Container::new(
        scrollable::styled(
            Column::new()
                .spacing(theme::spacing::MD)
                .push(
                    Container::new(
                        Row::new()
                            .spacing(theme::spacing::XS)
                            .align_y(Alignment::Center)
                            .push(Text::new(i18n.re_editor_title).size(16))
                            .push(widgets::info_chip::<RebaseEditorMessage>(
                                format!("Todo {}", state.todo_list.len()),
                                BadgeTone::Neutral,
                            ))
                            .push(widgets::info_chip::<RebaseEditorMessage>(
                                if state.is_rebasing {
                                    format!("{}/{}", state.current_step, state.total_steps)
                                } else {
                                    i18n.re_not_started.to_string()
                                },
                                BadgeTone::Accent,
                            ))
                            .push(button::ghost(i18n.refresh, Some(RebaseEditorMessage::Refresh)))
                            .push(button::ghost(i18n.close, Some(RebaseEditorMessage::Close))),
                    )
                    .padding([10, 12])
                    .style(theme::panel_style(Surface::Panel)),
                )
                .push_maybe(status_panel)
                .push_maybe(context_panel)
                .push(branch_inputs)
                .push(build_progress(state, i18n))
                .push(build_rebase_controls(state, i18n))
                .push_maybe(build_selected_todo_detail(state, i18n)),
        )
        .height(Length::Fill),
    )
    .padding([10, 12])
    .width(Length::Fill)
    .height(Length::Fill)
    .style(theme::panel_style(Surface::Panel))
    .into()
}

/// Build detail panel for the selected todo item (T043)
fn build_selected_todo_detail<'a>(
    state: &'a RebaseEditorState, i18n: &'a I18n,
) -> Option<Element<'a, RebaseEditorMessage>> {
    let index = state.selected_todo_index?;
    let item = state.todo_list.get(index)?;

    Some(
        Container::new(
            Column::new()
                .spacing(theme::spacing::XS)
                .push(
                    Text::new(i18n.re_commit_detail)
                        .size(12)
                        .color(theme::darcula::TEXT_SECONDARY),
                )
                .push(
                    Row::new()
                        .spacing(theme::spacing::SM)
                        .push(
                            Text::new(short_commit_id(&item.commit))
                                .size(11)
                                .color(theme::darcula::TEXT_DISABLED),
                        )
                        .push(
                            Text::new(todo_action_label(&item.action))
                                .size(11)
                                .color(todo_action_color(&item.action)),
                        ),
                )
                .push(
                    Text::new(&item.message)
                        .size(12)
                        .width(Length::Fill)
                        .wrapping(text::Wrapping::WordOrGlyph),
                )
                .push(
                    Text::new(i18n.re_full_hash_fmt.replace("{}", &item.commit))
                        .size(10)
                        .color(theme::darcula::TEXT_DISABLED),
                ),
        )
        .padding([8, 12])
        .style(theme::panel_style(Surface::Raised))
        .into(),
    )
}

fn allowed_todo_actions(index: usize) -> &'static [&'static str] {
    if index == 0 {
        &FIRST_TODO_ACTIONS
    } else {
        &OTHER_TODO_ACTIONS
    }
}

fn todo_action_label(action: &str) -> String {
    action.trim().to_lowercase()
}

fn todo_action_tone(action: &str) -> BadgeTone {
    match action.trim().to_lowercase().as_str() {
        "pick" => BadgeTone::Neutral,
        "reword" | "edit" => BadgeTone::Accent,
        "fixup" | "squash" => BadgeTone::Warning,
        "drop" => BadgeTone::Danger,
        _ => BadgeTone::Neutral,
    }
}

fn short_commit_id(id: &str) -> &str {
    &id[..id.len().min(8)]
}

fn build_status_panel<'a, Message: 'a>(
    label: impl Into<String>,
    detail: impl Into<String>,
    tone: BadgeTone,
) -> Element<'a, Message> {
    widgets::status_banner(label, detail, tone)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_todo(action: &str, commit: &str, message: &str) -> RebaseTodoItem {
        RebaseTodoItem {
            action: action.to_string(),
            commit: commit.to_string(),
            message: message.to_string(),
        }
    }

    #[test]
    fn cycle_todo_action_rotates_through_actions() {
        let mut state = RebaseEditorState::new();
        state.todo_is_editable = true;
        state.todo_list = vec![
            make_todo("pick", "abc123", "first"),
            make_todo("pick", "def456", "second"),
        ];

        // First item: pick → reword → edit → drop → pick
        state.cycle_todo_action(0);
        assert_eq!(state.todo_list[0].action, "reword");

        state.cycle_todo_action(0);
        assert_eq!(state.todo_list[0].action, "edit");

        state.cycle_todo_action(0);
        assert_eq!(state.todo_list[0].action, "drop");

        state.cycle_todo_action(0);
        assert_eq!(state.todo_list[0].action, "pick");
    }

    #[test]
    fn second_item_can_use_fixup_and_squash() {
        let mut state = RebaseEditorState::new();
        state.todo_is_editable = true;
        state.todo_list = vec![
            make_todo("pick", "abc", "first"),
            make_todo("pick", "def", "second"),
        ];

        // Second item has more options: pick → reword → edit → fixup → squash → drop
        state.cycle_todo_action(1); // reword
        state.cycle_todo_action(1); // edit
        state.cycle_todo_action(1); // fixup
        assert_eq!(state.todo_list[1].action, "fixup");

        state.cycle_todo_action(1); // squash
        assert_eq!(state.todo_list[1].action, "squash");
    }

    #[test]
    fn move_todo_up_swaps_items() {
        let mut state = RebaseEditorState::new();
        state.todo_is_editable = true;
        state.todo_list = vec![
            make_todo("pick", "aaa", "first"),
            make_todo("pick", "bbb", "second"),
            make_todo("pick", "ccc", "third"),
        ];

        state.move_todo_up(2);
        assert_eq!(state.todo_list[1].commit, "ccc");
        assert_eq!(state.todo_list[2].commit, "bbb");
    }

    #[test]
    fn move_todo_down_swaps_items() {
        let mut state = RebaseEditorState::new();
        state.todo_is_editable = true;
        state.todo_list = vec![
            make_todo("pick", "aaa", "first"),
            make_todo("pick", "bbb", "second"),
        ];

        state.move_todo_down(0);
        assert_eq!(state.todo_list[0].commit, "bbb");
        assert_eq!(state.todo_list[1].commit, "aaa");
    }

    #[test]
    fn move_todo_up_at_zero_does_nothing() {
        let mut state = RebaseEditorState::new();
        state.todo_is_editable = true;
        state.todo_list = vec![make_todo("pick", "aaa", "first")];

        state.move_todo_up(0); // should not panic
        assert_eq!(state.todo_list[0].commit, "aaa");
    }

    #[test]
    fn set_todo_action_changes_action() {
        let mut state = RebaseEditorState::new();
        state.todo_is_editable = true;
        state.todo_list = vec![
            make_todo("pick", "aaa", "first"), // need 2 items so squash is valid
            make_todo("pick", "abc", "test"),
        ];

        state.set_todo_action(1, "squash".to_string());
        assert_eq!(state.todo_list[1].action, "squash");
    }

    #[test]
    fn inline_edit_lifecycle() {
        let mut state = RebaseEditorState::new();
        state.todo_list = vec![make_todo("pick", "abc", "original message")];

        // Start editing
        state.start_inline_edit(0);
        assert_eq!(state.inline_edit_index, Some(0));
        assert_eq!(state.inline_edit_text, "original message");

        // Change text
        state.inline_edit_text = "new message".to_string();

        // Confirm
        state.confirm_inline_edit();
        assert_eq!(state.todo_list[0].message, "new message");
        assert_eq!(state.inline_edit_index, None);
    }

    #[test]
    fn inline_edit_cancel_preserves_original() {
        let mut state = RebaseEditorState::new();
        state.todo_list = vec![make_todo("pick", "abc", "original")];

        state.start_inline_edit(0);
        state.inline_edit_text = "changed but cancelled".to_string();
        state.cancel_inline_edit();

        assert_eq!(state.todo_list[0].message, "original");
        assert_eq!(state.inline_edit_index, None);
    }

    #[test]
    fn non_editable_state_ignores_mutations() {
        let mut state = RebaseEditorState::new();
        state.todo_is_editable = false;
        state.todo_list = vec![make_todo("pick", "abc", "test")];

        state.cycle_todo_action(0);
        assert_eq!(
            state.todo_list[0].action, "pick",
            "should not change when not editable"
        );

        state.move_todo_up(0);
        state.move_todo_down(0);
        state.set_todo_action(0, "drop".to_string());
        assert_eq!(state.todo_list[0].action, "pick");
    }
}
