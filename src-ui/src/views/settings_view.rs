//! Git settings view — matches IDEA's Version Control > Git settings panel.

use crate::i18n::I18n;
use crate::theme::{self, Surface};
use crate::widgets;
use crate::widgets::{button, scrollable, text_input};
use iced::widget::{Column, Container, Row, Space, Text};
use iced::{Alignment, Element, Length};

/// Settings messages
#[derive(Debug, Clone)]
pub enum SettingsMessage {
    // Update
    SetUpdateMethod(UpdateMethod),
    ToggleAutoUpdateOnPushReject,
    // Push
    SetProtectedBranches(String),
    TogglePreviewPushOnCommit,
    // Commit
    ToggleSignOffCommit,
    ToggleWarnCrlf,
    ToggleWarnDetachedHead,
    ToggleWarnLargeFiles,
    SetLargeFileLimitMb(String),
    ToggleStagingArea,
    SetEditorFontSize(String),
    // Fetch
    SetFetchTagsMode(FetchTagsMode),
    // LLM
    SetLlmApiUrl(String),
    SetLlmApiKey(String),
    SetLlmModel(String),
    ToggleLlmEnabled,
    // Language
    SetLanguage(Option<String>),
    // Actions
    Close,
    SaveAndClose,
}

/// Update method
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateMethod {
    BranchDefault,
    Merge,
    Rebase,
}

impl UpdateMethod {
    pub fn label(&self) -> &'static str {
        match self {
            Self::BranchDefault => "Branch Default",
            Self::Merge => "Merge",
            Self::Rebase => "Rebase",
        }
    }
}

/// Fetch tags mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FetchTagsMode {
    Default,
    AllTags,
    NoTags,
}

impl FetchTagsMode {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Default => "Default",
            Self::AllTags => "All Tags",
            Self::NoTags => "No Tags",
        }
    }
}

/// Git settings state
#[derive(Debug, Clone)]
pub struct GitSettings {
    pub update_method: UpdateMethod,
    pub auto_update_on_push_reject: bool,
    pub protected_branches: String,
    pub preview_push_on_commit: bool,
    pub sign_off_commit: bool,
    pub warn_crlf: bool,
    pub warn_detached_head: bool,
    pub warn_large_files: bool,
    pub large_file_limit_mb: String,
    pub staging_area_enabled: bool,
    pub fetch_tags_mode: FetchTagsMode,
    // Editor
    pub editor_font_size: String,
    // LLM
    pub llm_enabled: bool,
    pub llm_api_url: String,
    pub llm_api_key: String,
    pub llm_model: String,
    // Language: None = auto-detect, Some("zh-CN"), Some("en")
    pub language: Option<String>,
}

impl Default for GitSettings {
    fn default() -> Self {
        Self {
            update_method: UpdateMethod::Merge,
            auto_update_on_push_reject: false,
            protected_branches: "main, master".to_string(),
            preview_push_on_commit: true,
            sign_off_commit: false,
            warn_crlf: true,
            warn_detached_head: true,
            warn_large_files: true,
            large_file_limit_mb: "50".to_string(),
            staging_area_enabled: false,
            fetch_tags_mode: FetchTagsMode::Default,
            editor_font_size: "13".to_string(),
            llm_enabled: false,
            llm_api_url: "https://api.deepseek.com/v1/chat/completions".to_string(),
            llm_api_key: String::new(),
            llm_model: "deepseek-chat".to_string(),
            language: None,
        }
    }
}

impl GitSettings {
    pub fn apply_message(&mut self, message: &SettingsMessage) {
        match message {
            SettingsMessage::SetUpdateMethod(method) => self.update_method = *method,
            SettingsMessage::ToggleAutoUpdateOnPushReject => {
                self.auto_update_on_push_reject = !self.auto_update_on_push_reject;
            }
            SettingsMessage::SetProtectedBranches(val) => self.protected_branches = val.clone(),
            SettingsMessage::TogglePreviewPushOnCommit => {
                self.preview_push_on_commit = !self.preview_push_on_commit;
            }
            SettingsMessage::ToggleSignOffCommit => {
                self.sign_off_commit = !self.sign_off_commit;
            }
            SettingsMessage::ToggleWarnCrlf => self.warn_crlf = !self.warn_crlf,
            SettingsMessage::ToggleWarnDetachedHead => {
                self.warn_detached_head = !self.warn_detached_head;
            }
            SettingsMessage::ToggleWarnLargeFiles => {
                self.warn_large_files = !self.warn_large_files;
            }
            SettingsMessage::SetLargeFileLimitMb(val) => self.large_file_limit_mb = val.clone(),
            SettingsMessage::ToggleStagingArea => {
                self.staging_area_enabled = !self.staging_area_enabled;
            }
            SettingsMessage::SetEditorFontSize(val) => self.editor_font_size = val.clone(),
            SettingsMessage::SetFetchTagsMode(mode) => self.fetch_tags_mode = *mode,
            SettingsMessage::ToggleLlmEnabled => self.llm_enabled = !self.llm_enabled,
            SettingsMessage::SetLlmApiUrl(val) => self.llm_api_url = val.clone(),
            SettingsMessage::SetLlmApiKey(val) => self.llm_api_key = val.clone(),
            SettingsMessage::SetLlmModel(val) => self.llm_model = val.clone(),
            SettingsMessage::SetLanguage(val) => self.language = val.clone(),
            SettingsMessage::Close | SettingsMessage::SaveAndClose => {}
        }
    }
}

const SETTINGS_FILE: &str = "git-settings-v1.txt";

fn settings_path() -> std::path::PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("slio-git")
        .join(SETTINGS_FILE)
}

impl GitSettings {
    pub fn load() -> Self {
        Self::load_from_path(&settings_path())
    }

    fn load_from_path(path: &std::path::Path) -> Self {
        let Ok(contents) = std::fs::read_to_string(path) else {
            return Self::default();
        };
        Self::parse(&contents)
    }

    fn parse(contents: &str) -> Self {
        let mut s = Self::default();
        for line in contents.lines() {
            let Some((key, value)) = line.split_once('\t') else {
                continue;
            };
            match key {
                "update_method" => {
                    s.update_method = match value {
                        "branch_default" => UpdateMethod::BranchDefault,
                        "rebase" => UpdateMethod::Rebase,
                        _ => UpdateMethod::Merge,
                    };
                }
                "auto_update_on_push_reject" => {
                    s.auto_update_on_push_reject = value == "true";
                }
                "protected_branches" => s.protected_branches = value.to_string(),
                "preview_push_on_commit" => s.preview_push_on_commit = value == "true",
                "sign_off_commit" => s.sign_off_commit = value == "true",
                "warn_crlf" => s.warn_crlf = value == "true",
                "warn_detached_head" => s.warn_detached_head = value == "true",
                "warn_large_files" => s.warn_large_files = value == "true",
                "large_file_limit_mb" => s.large_file_limit_mb = value.to_string(),
                "staging_area_enabled" => s.staging_area_enabled = value == "true",
                "editor_font_size" => s.editor_font_size = value.to_string(),
                "fetch_tags_mode" => {
                    s.fetch_tags_mode = match value {
                        "all" => FetchTagsMode::AllTags,
                        "none" => FetchTagsMode::NoTags,
                        _ => FetchTagsMode::Default,
                    };
                }
                "llm_enabled" => s.llm_enabled = value == "true",
                "llm_api_url" => s.llm_api_url = value.to_string(),
                "llm_api_key" => s.llm_api_key = value.to_string(),
                "llm_model" => s.llm_model = value.to_string(),
                "language" => {
                    s.language = if value == "auto" || value.is_empty() {
                        None
                    } else {
                        Some(value.to_string())
                    };
                }
                _ => {}
            }
        }
        s
    }

    pub fn save(&self) -> std::io::Result<()> {
        self.save_to_path(&settings_path())
    }

    fn save_to_path(&self, path: &std::path::Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, self.serialize())
    }

    fn serialize(&self) -> String {
        let update_method = match self.update_method {
            UpdateMethod::BranchDefault => "branch_default",
            UpdateMethod::Merge => "merge",
            UpdateMethod::Rebase => "rebase",
        };
        let fetch_tags = match self.fetch_tags_mode {
            FetchTagsMode::Default => "default",
            FetchTagsMode::AllTags => "all",
            FetchTagsMode::NoTags => "none",
        };
        format!(
            "update_method\t{update_method}\n\
             auto_update_on_push_reject\t{}\n\
             protected_branches\t{}\n\
             preview_push_on_commit\t{}\n\
             sign_off_commit\t{}\n\
             warn_crlf\t{}\n\
             warn_detached_head\t{}\n\
             warn_large_files\t{}\n\
             large_file_limit_mb\t{}\n\
             staging_area_enabled\t{}\n\
             editor_font_size\t{}\n\
             fetch_tags_mode\t{fetch_tags}\n\
             llm_enabled\t{}\n\
             llm_api_url\t{}\n\
             llm_api_key\t{}\n\
             llm_model\t{}\n\
             language\t{}\n",
            self.auto_update_on_push_reject,
            self.protected_branches,
            self.preview_push_on_commit,
            self.sign_off_commit,
            self.warn_crlf,
            self.warn_detached_head,
            self.warn_large_files,
            self.large_file_limit_mb,
            self.staging_area_enabled,
            self.editor_font_size,
            self.llm_enabled,
            self.llm_api_url,
            self.llm_api_key,
            self.llm_model,
            self.language.as_deref().unwrap_or("auto"),
        )
    }

    pub fn editor_font_size_f32(&self) -> f32 {
        self.editor_font_size
            .parse::<f32>()
            .unwrap_or(13.0)
            .clamp(8.0, 24.0)
    }

    pub fn llm_config(&self) -> git_core::llm::LlmConfig {
        git_core::llm::LlmConfig {
            api_url: self.llm_api_url.clone(),
            api_key: self.llm_api_key.clone(),
            model: self.llm_model.clone(),
        }
    }
}

/// Render the settings panel
pub fn view<'a>(settings: &'a GitSettings, i18n: &'a I18n) -> Element<'a, SettingsMessage> {
    let header = Container::new(
        Row::new()
            .align_y(Alignment::Center)
            .push(
                Text::new(i18n.sv_title)
                    .size(14)
                    .color(theme::darcula::TEXT_PRIMARY),
            )
            .push(Space::new().width(Length::Fill))
            .push(button::compact_ghost(i18n.close, Some(SettingsMessage::Close))),
    )
    .padding([6, 14])
    .width(Length::Fill)
    .style(theme::frame_style(Surface::Toolbar));

    // ── 提交 ──
    let commit_section = settings_section(
        i18n.sv_section_commit,
        vec![
            checkbox_row(
                settings.sign_off_commit,
                i18n.sv_sign_off,
                SettingsMessage::ToggleSignOffCommit,
            ),
            checkbox_row(
                settings.staging_area_enabled,
                i18n.sv_enable_staging,
                SettingsMessage::ToggleStagingArea,
            ),
            checkbox_row(
                settings.warn_crlf,
                i18n.sv_crlf_warning,
                SettingsMessage::ToggleWarnCrlf,
            ),
            checkbox_row(
                settings.warn_detached_head,
                i18n.sv_detached_warning,
                SettingsMessage::ToggleWarnDetachedHead,
            ),
            checkbox_row(
                settings.warn_large_files,
                i18n.sv_large_file_warning,
                SettingsMessage::ToggleWarnLargeFiles,
            ),
        ],
    );

    let large_file_row = Container::new(
        Row::new()
            .spacing(8)
            .align_y(Alignment::Center)
            .push(Space::new().width(Length::Fixed(20.0)))
            .push(
                Text::new(i18n.sv_large_file_threshold)
                    .size(12)
                    .color(theme::darcula::TEXT_SECONDARY),
            )
            .push(
                Container::new(text_input::styled(
                    "50",
                    &settings.large_file_limit_mb,
                    SettingsMessage::SetLargeFileLimitMb,
                ))
                .width(Length::Fixed(60.0)),
            ),
    )
    .padding([2, 14]);

    // ── 编辑器 ──
    let editor_section = Container::new(
        Column::new()
            .spacing(4)
            .push(
                Text::new(i18n.sv_editor)
                    .size(10)
                    .color(theme::darcula::TEXT_DISABLED),
            )
            .push(
                Row::new()
                    .spacing(8)
                    .align_y(Alignment::Center)
                    .push(
                        Text::new(i18n.sv_diff_font_size)
                            .size(12)
                            .color(theme::darcula::TEXT_SECONDARY),
                    )
                    .push(
                        Container::new(text_input::styled(
                            "13",
                            &settings.editor_font_size,
                            SettingsMessage::SetEditorFontSize,
                        ))
                        .width(Length::Fixed(50.0)),
                    )
                    .push(
                        Text::new("px")
                            .size(11)
                            .color(theme::darcula::TEXT_DISABLED),
                    ),
            ),
    )
    .padding([8, 14]);

    // ── 语言 / Language ──
    let lang_auto = settings.language.is_none();
    let lang_zh = settings.language.as_deref() == Some("zh-CN");
    let lang_en = settings.language.as_deref() == Some("en");

    let language_section = Container::new(
        Column::new()
            .spacing(4)
            .push(
                Text::new(i18n.sv_language_label)
                    .size(10)
                    .color(theme::darcula::TEXT_DISABLED),
            )
            .push(
                Row::new()
                    .spacing(12)
                    .push(radio_button(
                        "Auto (System)",
                        lang_auto,
                        SettingsMessage::SetLanguage(None),
                    ))
                    .push(radio_button(
                        i18n.sv_lang_zh,
                        lang_zh,
                        SettingsMessage::SetLanguage(Some("zh-CN".to_string())),
                    ))
                    .push(radio_button(
                        "English",
                        lang_en,
                        SettingsMessage::SetLanguage(Some("en".to_string())),
                    )),
            ),
    )
    .padding([8, 14]);

    // ── 推送 ──
    let push_section = settings_section(
        i18n.sv_section_push,
        vec![
            checkbox_row(
                settings.auto_update_on_push_reject,
                i18n.sv_auto_update_on_reject,
                SettingsMessage::ToggleAutoUpdateOnPushReject,
            ),
            checkbox_row(
                settings.preview_push_on_commit,
                i18n.sv_preview_push,
                SettingsMessage::TogglePreviewPushOnCommit,
            ),
        ],
    );

    let protected_row = Container::new(
        Row::new()
            .spacing(8)
            .align_y(Alignment::Center)
            .push(
                Text::new(i18n.sv_protected_branches)
                    .size(12)
                    .color(theme::darcula::TEXT_SECONDARY),
            )
            .push(
                Container::new(text_input::styled(
                    "main, master",
                    &settings.protected_branches,
                    SettingsMessage::SetProtectedBranches,
                ))
                .width(Length::Fill),
            ),
    )
    .padding([4, 14]);

    // ── 更新 ──
    let update_section = Container::new(
        Column::new()
            .spacing(4)
            .push(
                Text::new(i18n.sv_update_method)
                    .size(10)
                    .color(theme::darcula::TEXT_DISABLED),
            )
            .push(
                Row::new()
                    .spacing(12)
                    .push(radio_button(
                        i18n.sv_branch_default,
                        settings.update_method == UpdateMethod::BranchDefault,
                        SettingsMessage::SetUpdateMethod(UpdateMethod::BranchDefault),
                    ))
                    .push(radio_button(
                        i18n.sv_merge,
                        settings.update_method == UpdateMethod::Merge,
                        SettingsMessage::SetUpdateMethod(UpdateMethod::Merge),
                    ))
                    .push(radio_button(
                        i18n.sv_rebase,
                        settings.update_method == UpdateMethod::Rebase,
                        SettingsMessage::SetUpdateMethod(UpdateMethod::Rebase),
                    )),
            ),
    )
    .padding([8, 14]);

    // ── 获取 ──
    let fetch_section = Container::new(
        Column::new()
            .spacing(4)
            .push(
                Text::new(i18n.sv_fetch_tags)
                    .size(10)
                    .color(theme::darcula::TEXT_DISABLED),
            )
            .push(
                Row::new()
                    .spacing(12)
                    .push(radio_button(
                        i18n.sv_fetch_default,
                        settings.fetch_tags_mode == FetchTagsMode::Default,
                        SettingsMessage::SetFetchTagsMode(FetchTagsMode::Default),
                    ))
                    .push(radio_button(
                        i18n.sv_fetch_all_tags,
                        settings.fetch_tags_mode == FetchTagsMode::AllTags,
                        SettingsMessage::SetFetchTagsMode(FetchTagsMode::AllTags),
                    ))
                    .push(radio_button(
                        i18n.sv_fetch_no_tags,
                        settings.fetch_tags_mode == FetchTagsMode::NoTags,
                        SettingsMessage::SetFetchTagsMode(FetchTagsMode::NoTags),
                    )),
            ),
    )
    .padding([8, 14]);

    // ── AI 提交消息 ──
    let llm_section = Container::new(
        Column::new()
            .spacing(4)
            .push(
                Text::new(i18n.sv_ai_commit)
                    .size(10)
                    .color(theme::darcula::TEXT_DISABLED),
            )
            .push(checkbox_row(
                settings.llm_enabled,
                i18n.sv_ai_enable,
                SettingsMessage::ToggleLlmEnabled,
            ))
            .push(
                Row::new()
                    .spacing(8)
                    .align_y(Alignment::Center)
                    .push(
                        Text::new(i18n.sv_api_url)
                            .size(12)
                            .color(theme::darcula::TEXT_SECONDARY),
                    )
                    .push(
                        Container::new(text_input::styled(
                            "https://api.deepseek.com/v1/chat/completions",
                            &settings.llm_api_url,
                            SettingsMessage::SetLlmApiUrl,
                        ))
                        .width(Length::Fill),
                    ),
            )
            .push(
                Row::new()
                    .spacing(8)
                    .align_y(Alignment::Center)
                    .push(
                        Text::new(i18n.sv_api_key)
                            .size(12)
                            .color(theme::darcula::TEXT_SECONDARY),
                    )
                    .push(
                        Container::new(text_input::styled_password(
                            "sk-...",
                            &settings.llm_api_key,
                            SettingsMessage::SetLlmApiKey,
                        ))
                        .width(Length::Fill),
                    ),
            )
            .push(
                Row::new()
                    .spacing(8)
                    .align_y(Alignment::Center)
                    .push(
                        Text::new(i18n.sv_model_name)
                            .size(12)
                            .color(theme::darcula::TEXT_SECONDARY),
                    )
                    .push(
                        Container::new(text_input::styled(
                            "deepseek-chat",
                            &settings.llm_model,
                            SettingsMessage::SetLlmModel,
                        ))
                        .width(Length::Fixed(200.0)),
                    ),
            ),
    )
    .padding([8, 14]);

    // ── Footer ──
    let footer = Container::new(
        Row::new()
            .spacing(8)
            .align_y(Alignment::Center)
            .push(Space::new().width(Length::Fill))
            .push(button::ghost(i18n.cancel, Some(SettingsMessage::Close)))
            .push(button::primary(i18n.sv_save, Some(SettingsMessage::SaveAndClose))),
    )
    .padding([8, 14])
    .width(Length::Fill)
    .style(theme::frame_style(Surface::Toolbar));

    // ── Assembly ──
    let content = Column::new()
        .spacing(0)
        .width(Length::Fill)
        .push(header)
        .push(iced::widget::rule::horizontal(1))
        .push(commit_section)
        .push(large_file_row)
        .push(iced::widget::rule::horizontal(1))
        .push(editor_section)
        .push(iced::widget::rule::horizontal(1))
        .push(language_section)
        .push(iced::widget::rule::horizontal(1))
        .push(push_section)
        .push(protected_row)
        .push(iced::widget::rule::horizontal(1))
        .push(update_section)
        .push(iced::widget::rule::horizontal(1))
        .push(fetch_section)
        .push(iced::widget::rule::horizontal(1))
        .push(llm_section)
        .push(Space::new().height(Length::Fill))
        .push(iced::widget::rule::horizontal(1))
        .push(footer);

    Container::new(scrollable::styled(content).height(Length::Fill))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(theme::panel_style(Surface::Panel))
        .into()
}

// ── Helpers ──

fn settings_section<'a>(
    title: &'a str,
    items: Vec<Element<'a, SettingsMessage>>,
) -> Container<'a, SettingsMessage> {
    let mut col = Column::new().spacing(4).push(
        Text::new(title)
            .size(10)
            .color(theme::darcula::TEXT_DISABLED),
    );
    for item in items {
        col = col.push(item);
    }
    Container::new(col).padding([8, 14])
}

fn checkbox_row<'a>(
    checked: bool,
    label: &'a str,
    on_toggle: SettingsMessage,
) -> Element<'a, SettingsMessage> {
    widgets::compact_checkbox(checked, label, move |_| on_toggle.clone())
}

fn radio_button<'a>(
    label: &'a str,
    selected: bool,
    on_press: SettingsMessage,
) -> Element<'a, SettingsMessage> {
    let icon = if selected { "◉" } else { "○" };
    let color = if selected {
        theme::darcula::ACCENT
    } else {
        theme::darcula::TEXT_SECONDARY
    };

    iced::widget::Button::new(
        Row::new()
            .spacing(4)
            .align_y(Alignment::Center)
            .push(Text::new(icon).size(12).color(color))
            .push(
                Text::new(label)
                    .size(12)
                    .color(theme::darcula::TEXT_PRIMARY),
            ),
    )
    .style(theme::button_style(theme::ButtonTone::Ghost))
    .padding([2, 4])
    .on_press(on_press)
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_settings_match_idea() {
        let s = GitSettings::default();
        assert_eq!(s.update_method, UpdateMethod::Merge);
        assert!(!s.auto_update_on_push_reject);
        assert!(s.protected_branches.contains("main"));
        assert!(s.protected_branches.contains("master"));
        assert!(!s.sign_off_commit);
        assert!(s.warn_crlf);
        assert!(s.warn_detached_head);
        assert!(s.warn_large_files);
        assert_eq!(s.large_file_limit_mb, "50");
        assert!(!s.staging_area_enabled);
        assert_eq!(s.fetch_tags_mode, FetchTagsMode::Default);
    }

    #[test]
    fn apply_toggle_messages() {
        let mut s = GitSettings::default();
        s.apply_message(&SettingsMessage::ToggleSignOffCommit);
        assert!(s.sign_off_commit);
        s.apply_message(&SettingsMessage::ToggleSignOffCommit);
        assert!(!s.sign_off_commit);
    }

    #[test]
    fn apply_update_method() {
        let mut s = GitSettings::default();
        s.apply_message(&SettingsMessage::SetUpdateMethod(UpdateMethod::Rebase));
        assert_eq!(s.update_method, UpdateMethod::Rebase);
    }

    #[test]
    fn apply_protected_branches() {
        let mut s = GitSettings::default();
        s.apply_message(&SettingsMessage::SetProtectedBranches(
            "main, dev, release/*".to_string(),
        ));
        assert!(s.protected_branches.contains("release"));
    }

    #[test]
    fn settings_roundtrip() {
        let s = GitSettings {
            sign_off_commit: true,
            update_method: UpdateMethod::Rebase,
            fetch_tags_mode: FetchTagsMode::AllTags,
            llm_enabled: true,
            llm_api_key: "sk-test-key".to_string(),
            protected_branches: "main, develop".to_string(),
            large_file_limit_mb: "100".to_string(),
            ..GitSettings::default()
        };

        let serialized = s.serialize();
        let loaded = GitSettings::parse(&serialized);

        assert!(loaded.sign_off_commit);
        assert_eq!(loaded.update_method, UpdateMethod::Rebase);
        assert_eq!(loaded.fetch_tags_mode, FetchTagsMode::AllTags);
        assert!(loaded.llm_enabled);
        assert_eq!(loaded.llm_api_key, "sk-test-key");
        assert_eq!(loaded.protected_branches, "main, develop");
        assert_eq!(loaded.large_file_limit_mb, "100");
    }

    #[test]
    fn settings_save_and_load_from_file() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let path = temp_dir.path().join("settings.txt");

        let s = GitSettings {
            warn_crlf: false,
            staging_area_enabled: true,
            llm_model: "gpt-4".to_string(),
            ..GitSettings::default()
        };

        s.save_to_path(&path).expect("save");
        let loaded = GitSettings::load_from_path(&path);

        assert!(!loaded.warn_crlf);
        assert!(loaded.staging_area_enabled);
        assert_eq!(loaded.llm_model, "gpt-4");
    }

    #[test]
    fn load_missing_file_returns_defaults() {
        let loaded = GitSettings::load_from_path(std::path::Path::new("/nonexistent/path"));
        let default = GitSettings::default();
        assert_eq!(loaded.update_method, default.update_method);
        assert_eq!(loaded.warn_crlf, default.warn_crlf);
    }
}
