//! Rebase editor view.
//!
//! Provides a dialog for editing the rebase todo list.

use crate::theme::{self, BadgeTone, Surface};
use crate::widgets::{self, button, scrollable, text_input, OptionalPush};
use git_core::Repository;
use iced::widget::{text, Column, Container, Row, Text};
use iced::{Alignment, Element, Length};

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
    MoveTodoUp(usize),
    MoveTodoDown(usize),
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

    pub fn load_status(&mut self, repo: &Repository) {
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
                        self.error = Some(format!("读取变基 todo 失败: {error}"));
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
                self.error = Some(format!("获取变基状态失败: {error}"));
                self.success_message = None;
                self.has_conflicts = false;
            }
        }
    }

    pub fn prepare_interactive_rebase(&mut self, repo: &Repository, commit_id: String) {
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
                self.success_message = Some(format!(
                    "已载入从 {} 开始的 {} 条 todo，可先调整动作与顺序。",
                    short_commit_id(&self.base_branch),
                    self.todo_list.len()
                ));
            }
            Err(error) => {
                self.error = Some(format!("准备交互式变基失败: {error}"));
            }
        }

        self.is_loading = false;
    }

    pub fn start_rebase(&mut self, repo: &Repository) {
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
                    self.load_status(repo);
                    self.success_message = Some(if self.is_rebasing {
                        if self.has_conflicts {
                            "交互式变基已启动，但当前有冲突。".to_string()
                        } else {
                            "交互式变基已启动，可继续在这里推进后续步骤。".to_string()
                        }
                    } else {
                        message
                    });
                }
                Err(error) => {
                    self.error = Some(format!("开始交互式变基失败: {error}"));
                }
            }

            self.is_loading = false;
            return;
        }

        let onto_branch = self.onto_branch.trim().to_string();
        if onto_branch.is_empty() {
            self.error = Some("目标分支不能为空".to_string());
            self.is_loading = false;
            return;
        }

        match git_core::rebase::rebase_start(repo, &onto_branch) {
            Ok(_) => {
                self.is_rebasing = true;
                self.load_status(repo);
                self.success_message = Some(if self.has_conflicts {
                    format!("已开始变基到 {onto_branch}，当前有冲突。")
                } else {
                    format!("已开始变基到 {onto_branch}")
                });
            }
            Err(error) => {
                self.error = Some(format!("开始变基失败: {error}"));
            }
        }
        self.is_loading = false;
    }

    pub fn continue_rebase(&mut self, repo: &Repository) {
        if self.has_conflicts {
            self.error = Some("仍有冲突待解决，请先处理冲突后再继续。".to_string());
            self.success_message = None;
            return;
        }

        self.is_loading = true;
        self.error = None;
        self.success_message = None;

        match git_core::rebase::rebase_continue(repo) {
            Ok(result) => {
                self.load_status(repo);
                if result.success {
                    self.success_message = Some(if self.is_rebasing {
                        format!("已推进到第 {}/{} 步", self.current_step, self.total_steps)
                    } else {
                        "变基已完成".to_string()
                    });
                } else {
                    self.error = Some(if result.message.trim().is_empty() {
                        "继续变基失败，请检查当前仓库状态。".to_string()
                    } else {
                        format!("继续变基失败: {}", result.message.trim())
                    });
                }
            }
            Err(error) => {
                self.error = Some(format!("继续变基失败: {error}"));
            }
        }
        self.is_loading = false;
    }

    pub fn skip_commit(&mut self, repo: &Repository) {
        self.is_loading = true;
        self.error = None;
        self.success_message = None;

        match git_core::rebase::rebase_skip(repo) {
            Ok(result) => {
                self.load_status(repo);
                if result.success {
                    self.success_message = Some(if self.is_rebasing {
                        format!(
                            "已跳过当前提交，第 {}/{} 步",
                            self.current_step, self.total_steps
                        )
                    } else {
                        "已跳过最后一个提交".to_string()
                    });
                } else {
                    self.error = Some(if result.message.trim().is_empty() {
                        "跳过提交失败，请检查当前仓库状态。".to_string()
                    } else {
                        format!("跳过提交失败: {}", result.message.trim())
                    });
                }
            }
            Err(error) => {
                self.error = Some(format!("跳过提交失败: {error}"));
            }
        }
        self.is_loading = false;
    }

    pub fn abort_rebase(&mut self, repo: &Repository) {
        self.is_loading = true;
        self.error = None;
        self.success_message = None;

        match git_core::rebase::rebase_abort(repo) {
            Ok(_) => {
                self.is_rebasing = false;
                self.load_status(repo);
                self.success_message = Some("已中止当前变基".to_string());
            }
            Err(error) => {
                self.error = Some(format!("中止变基失败: {error}"));
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

fn build_rebase_controls(state: &RebaseEditorState) -> Element<'_, RebaseEditorMessage> {
    if state.is_rebasing {
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
                            "编辑当前提交消息",
                            Some(RebaseEditorMessage::OpenAmendForCurrentStep),
                        )
                    }),
            )
            .push(button::primary(
                "继续",
                (!state.is_loading && !state.has_conflicts)
                    .then_some(RebaseEditorMessage::ContinueRebase),
            ))
            .push(button::secondary(
                "跳过",
                (!state.is_loading).then_some(RebaseEditorMessage::SkipCommit),
            ))
            .push(button::ghost(
                "中止",
                (!state.is_loading).then_some(RebaseEditorMessage::AbortRebase),
            ));

        scrollable::styled_horizontal(row)
            .width(Length::Fill)
            .into()
    } else if state.has_interactive_draft() {
        scrollable::styled_horizontal(Row::new().spacing(theme::spacing::XS).push(button::primary(
            "开始交互式变基",
            (!state.is_loading).then_some(RebaseEditorMessage::StartRebase),
        )))
        .width(Length::Fill)
        .into()
    } else {
        scrollable::styled_horizontal(
            Row::new().spacing(theme::spacing::XS).push(button::primary(
                "开始变基",
                (!state.is_loading && !state.onto_branch.trim().is_empty())
                    .then_some(RebaseEditorMessage::StartRebase),
            )),
        )
        .width(Length::Fill)
        .into()
    }
}

fn build_todo_list(state: &RebaseEditorState) -> Element<'_, RebaseEditorMessage> {
    if state.todo_list.is_empty() {
        return Column::new()
            .push(
                Text::new(if state.is_rebasing {
                    "当前没有额外的 todo 项可显示。"
                } else {
                    "还没有载入 todo 列表；可先从历史视图选择“从这里进行交互式变基”。"
                })
                .size(12)
                .color(theme::darcula::TEXT_SECONDARY),
            )
            .into();
    }

    state
        .todo_list
        .iter()
        .enumerate()
        .fold(
            Column::new().spacing(theme::spacing::XS),
            |column, (index, item)| {
                let action_chip: Element<'_, RebaseEditorMessage> = if state.todo_is_editable {
                    button::secondary(
                        todo_action_label(&item.action),
                        (!state.is_loading).then_some(RebaseEditorMessage::CycleTodoAction(index)),
                    )
                    .into()
                } else {
                    widgets::info_chip::<RebaseEditorMessage>(
                        todo_action_label(&item.action),
                        todo_action_tone(&item.action),
                    )
                };

                let move_controls = if state.todo_is_editable {
                    Row::new()
                        .spacing(theme::spacing::XS)
                        .push(button::compact_ghost(
                            "↑",
                            (index > 0 && !state.is_loading)
                                .then_some(RebaseEditorMessage::MoveTodoUp(index)),
                        ))
                        .push(button::compact_ghost(
                            "↓",
                            (index + 1 < state.todo_list.len() && !state.is_loading)
                                .then_some(RebaseEditorMessage::MoveTodoDown(index)),
                        ))
                } else {
                    Row::new()
                };

                column.push(
                    Container::new(
                        Row::new()
                            .spacing(theme::spacing::SM)
                            .align_y(Alignment::Center)
                            .push(action_chip)
                            .push(
                                Column::new()
                                    .spacing(2)
                                    .width(Length::Fill)
                                    .push(
                                        Row::new()
                                            .spacing(theme::spacing::XS)
                                            .align_y(Alignment::Center)
                                            .push(
                                                Text::new(short_commit_id(&item.commit))
                                                    .size(11)
                                                    .color(theme::darcula::TEXT_DISABLED),
                                            )
                                            .push(
                                                Text::new(&item.message)
                                                    .size(12)
                                                    .width(Length::Fill)
                                                    .wrapping(text::Wrapping::WordOrGlyph),
                                            ),
                                    )
                                    .push(
                                        Text::new(format!("完整哈希 {}", item.commit))
                                            .size(10)
                                            .width(Length::Fill)
                                            .wrapping(text::Wrapping::WordOrGlyph)
                                            .color(theme::darcula::TEXT_SECONDARY),
                                    ),
                            )
                            .push(move_controls),
                    )
                    .padding([6, 8])
                    .style(theme::panel_style(if state.todo_is_editable {
                        Surface::Raised
                    } else {
                        Surface::Panel
                    })),
                )
            },
        )
        .into()
}

fn build_progress(state: &RebaseEditorState) -> Element<'_, RebaseEditorMessage> {
    if state.is_rebasing {
        let progress_text = format!(
            "变基进度: {}/{} ({:.0}%)",
            state.current_step,
            state.total_steps,
            if state.total_steps > 0 {
                (state.current_step as f32 / state.total_steps as f32) * 100.0
            } else {
                0.0
            }
        );

        Container::new(
            Column::new()
                .spacing(theme::spacing::SM)
                .push(widgets::section_header(
                    "进度".to_uppercase(),
                    "当前变基状态",
                    "继续、跳过或中止前先确认当前步骤和剩余操作。",
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
                    "下一步",
                    if state.has_conflicts {
                        "先在冲突视图解决文件冲突，再返回这里继续 rebase。"
                    } else if state
                        .current_step_item
                        .as_ref()
                        .is_some_and(|item| item.action.eq_ignore_ascii_case("edit"))
                    {
                        "当前步骤要求修改提交说明；先打开提交面板完成 amend，再回来继续 rebase。"
                    } else if state.current_step < state.total_steps {
                        "当前步骤已准备好，可继续推进或根据需要跳过这一提交。"
                    } else {
                        "已经来到最后一步，确认无误后继续即可完成 rebase。"
                    },
                    if state.has_conflicts {
                        BadgeTone::Danger
                    } else {
                        BadgeTone::Accent
                    },
                ))
                .push_maybe(state.current_step_item.as_ref().map(|item| {
                    build_status_panel::<RebaseEditorMessage>(
                        "当前步骤",
                        format!(
                            "{} {} {}",
                            todo_action_label(&item.action),
                            short_commit_id(&item.commit),
                            item.message
                        ),
                        todo_action_tone(&item.action),
                    )
                }))
                .push(scrollable::styled(build_todo_list(state)).height(Length::Fixed(200.0))),
        )
        .padding([12, 12])
        .style(theme::panel_style(Surface::Panel))
        .into()
    } else if state.has_interactive_draft() {
        Container::new(
            Column::new()
                .spacing(theme::spacing::SM)
                .push(widgets::section_header(
                    "待办".to_uppercase(),
                    "交互式变基待办",
                    "点击动作可循环切换 pick / reword / edit / fixup / squash / drop，用上下箭头调整顺序。",
                ))
                .push(build_status_panel::<RebaseEditorMessage>(
                    "编辑规则",
                    "首条 todo 不能是 fixup / squash；当前仅支持当前分支第一父链上的本地未发布提交。",
                    BadgeTone::Accent,
                ))
                .push(scrollable::styled(build_todo_list(state)).height(Length::Fixed(260.0))),
        )
        .padding([12, 12])
        .style(theme::panel_style(Surface::Panel))
        .into()
    } else {
        widgets::panel_empty_state(
            "进度",
            "当前没有进行中的 rebase",
            "输入目标分支后即可开始普通 rebase；若要整理具体提交，请先从历史视图打开交互式变基入口。",
            None,
        )
    }
}

pub fn view(state: &RebaseEditorState) -> Element<'_, RebaseEditorMessage> {
    // IDEA-style: compact loading indicator when processing
    let status_panel: Option<Element<'_, RebaseEditorMessage>> = if state.is_loading {
        Some(
            Container::new(
                Row::new()
                    .spacing(theme::spacing::SM)
                    .align_y(Alignment::Center)
                    .push(widgets::loading_spinner::<RebaseEditorMessage>())
                    .push(
                        Text::new("正在执行 rebase 操作...")
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
            "失败",
            error,
            BadgeTone::Danger,
        ))
    } else if state.is_rebasing && state.has_conflicts {
        Some(build_status_panel::<RebaseEditorMessage>(
            "冲突阻塞",
            "当前 rebase 已暂停；请先解决冲突，再继续后续步骤。",
            BadgeTone::Danger,
        ))
    } else if let Some(message) = state.success_message.as_ref() {
        Some(build_status_panel::<RebaseEditorMessage>(
            "完成",
            message,
            BadgeTone::Success,
        ))
    } else if state.is_rebasing {
        Some(build_status_panel::<RebaseEditorMessage>(
            "进行中",
            "当前仓库正在 rebase。",
            BadgeTone::Warning,
        ))
    } else if state.has_interactive_draft() {
        Some(build_status_panel::<RebaseEditorMessage>(
            "待开始",
            format!(
                "已载入 {} 条 todo，可调整动作与顺序后开始交互式变基。",
                state.todo_list.len()
            ),
            BadgeTone::Accent,
        ))
    } else if state.onto_branch.trim().is_empty() {
        Some(build_status_panel::<RebaseEditorMessage>(
            "待开始",
            "先输入目标分支，再决定是否启动普通 rebase。",
            BadgeTone::Neutral,
        ))
    } else {
        Some(build_status_panel::<RebaseEditorMessage>(
            "准备就绪",
            format!(
                "目标分支为 {}，可以开始本次 rebase。",
                state.onto_branch.trim()
            ),
            BadgeTone::Accent,
        ))
    };

    let branch_inputs: Element<'_, RebaseEditorMessage> =
        if !state.is_rebasing && !state.has_interactive_draft() {
            Container::new(
                Column::new()
                    .spacing(theme::spacing::SM)
                    .push(widgets::section_header(
                        "目标".to_uppercase(),
                        "普通变基选项",
                        "输入 onto 分支，确认要把当前工作流重新整理到哪里。",
                    ))
                    .push(text_input::styled(
                        "目标分支（onto）",
                        &state.onto_branch,
                        RebaseEditorMessage::SetBaseBranch,
                    )),
            )
            .padding([12, 12])
            .style(theme::panel_style(Surface::Panel))
            .into()
        } else {
            Column::new().into()
        };

    let context_panel: Option<Element<'_, RebaseEditorMessage>> = (!state
        .base_branch
        .trim()
        .is_empty())
    .then(|| {
        Container::new(
            Column::new()
                .spacing(theme::spacing::XS)
                .push(widgets::section_header(
                    "上下文".to_uppercase(),
                    "历史整理入口",
                    "编辑待执行的 rebase 步骤，调整动作与顺序后继续。",
                ))
                .push(
                    Row::new()
                        .spacing(theme::spacing::XS)
                        .push(widgets::info_chip::<RebaseEditorMessage>(
                            format!("起点 {}", short_commit_id(&state.base_branch)),
                            BadgeTone::Accent,
                        ))
                        .push(widgets::info_chip::<RebaseEditorMessage>(
                            state.todo_base_ref.as_deref().map_or_else(
                                || "从根提交开始".to_string(),
                                |base| format!("基点 {}", short_commit_id(base)),
                            ),
                            BadgeTone::Neutral,
                        )),
                )
                .push(
                    Text::new(if state.todo_is_editable {
                        "直接编辑 todo 列表：`edit` 停下来改说明，`fixup/squash/drop` 自动继续。"
                    } else {
                        "当前正在执行或查看这次历史整理流程；下方会显示剩余 todo 与下一步操作。"
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
                            .push(Text::new("Rebase 编辑器").size(16))
                            .push(widgets::info_chip::<RebaseEditorMessage>(
                                format!("Todo {}", state.todo_list.len()),
                                BadgeTone::Neutral,
                            ))
                            .push(widgets::info_chip::<RebaseEditorMessage>(
                                if state.is_rebasing {
                                    format!("{}/{}", state.current_step, state.total_steps)
                                } else {
                                    "未开始".to_string()
                                },
                                BadgeTone::Accent,
                            ))
                            .push(button::ghost("刷新", Some(RebaseEditorMessage::Refresh)))
                            .push(button::ghost("关闭", Some(RebaseEditorMessage::Close))),
                    )
                    .padding([10, 12])
                    .style(theme::panel_style(Surface::Panel)),
                )
                .push_maybe(status_panel)
                .push_maybe(context_panel)
                .push(branch_inputs)
                .push(build_progress(state))
                .push(build_rebase_controls(state)),
        )
        .height(Length::Fill),
    )
    .padding([10, 12])
    .width(Length::Fill)
    .height(Length::Fill)
    .style(theme::panel_style(Surface::Panel))
    .into()
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
