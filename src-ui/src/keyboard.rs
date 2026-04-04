//! Keyboard shortcuts handling
//!
//! Provides keyboard shortcut support for staging/unstaging operations

#![allow(dead_code)]

use iced::keyboard;
use iced::Event;

/// Keyboard shortcut actions
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ShortcutAction {
    StageFile,
    UnstageFile,
    StageAll,
    UnstageAll,
    Refresh,
    ToggleChangesPanel,
    // Commit operations (IDEA: Ctrl+K)
    OpenCommitDialog,
    ToggleAmendCommitMode,
    // Push operations (IDEA: Ctrl+Shift+K)
    OpenPushDialog,
    // Diff operations (IDEA: Ctrl+D)
    ShowFileDiff,
    // Navigation
    NavigatePrevFile,
    NavigateNextFile,
    PrevHunk,
    NextHunk,
    // Commit (IDEA: Ctrl+Enter inside commit dialog)
    Commit,
    // Stash operations
    StashSave,
    StashPop,
    StashDrop,
    StashList,
}

/// Keyboard shortcut definition
#[derive(Debug, Clone)]
pub struct KeyboardShortcut {
    pub modifiers: keyboard::Modifiers,
    pub key: keyboard::Key,
    pub action: ShortcutAction,
}

impl KeyboardShortcut {
    /// Check if this shortcut matches the given event
    pub fn matches(&self, event: &Event) -> bool {
        if let Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) = event {
            return *key == self.key && modifiers.contains(self.modifiers);
        }
        false
    }
}

/// Get all registered keyboard shortcuts
pub fn get_shortcuts() -> Vec<KeyboardShortcut> {
    use keyboard::key::Named;
    use keyboard::{Key, Modifiers};

    vec![
        // Ctrl+S: Stage selected file
        KeyboardShortcut {
            modifiers: Modifiers::CTRL,
            key: Key::Character("s".into()),
            action: ShortcutAction::StageFile,
        },
        // Ctrl+U: Unstage selected file
        KeyboardShortcut {
            modifiers: Modifiers::CTRL,
            key: Key::Character("u".into()),
            action: ShortcutAction::UnstageFile,
        },
        // Ctrl+Shift+S: Stage all
        KeyboardShortcut {
            modifiers: Modifiers::CTRL | Modifiers::SHIFT,
            key: Key::Character("s".into()),
            action: ShortcutAction::StageAll,
        },
        // Ctrl+Shift+U: Unstage all
        KeyboardShortcut {
            modifiers: Modifiers::CTRL | Modifiers::SHIFT,
            key: Key::Character("u".into()),
            action: ShortcutAction::UnstageAll,
        },
        // Ctrl+R: Refresh
        KeyboardShortcut {
            modifiers: Modifiers::CTRL,
            key: Key::Character("r".into()),
            action: ShortcutAction::Refresh,
        },
        // Ctrl+K: Open commit dialog (IDEA style)
        KeyboardShortcut {
            modifiers: Modifiers::CTRL,
            key: Key::Character("k".into()),
            action: ShortcutAction::OpenCommitDialog,
        },
        // IDEA: Alt+M / macOS Ctrl+Alt+M toggle amend commit mode
        KeyboardShortcut {
            modifiers: Modifiers::ALT,
            key: Key::Character("m".into()),
            action: ShortcutAction::ToggleAmendCommitMode,
        },
        KeyboardShortcut {
            modifiers: Modifiers::CTRL | Modifiers::ALT,
            key: Key::Character("m".into()),
            action: ShortcutAction::ToggleAmendCommitMode,
        },
        // Ctrl+Shift+K: Open push dialog (IDEA style)
        KeyboardShortcut {
            modifiers: Modifiers::CTRL | Modifiers::SHIFT,
            key: Key::Character("k".into()),
            action: ShortcutAction::OpenPushDialog,
        },
        // Ctrl+D: Show diff for file (IDEA style)
        KeyboardShortcut {
            modifiers: Modifiers::CTRL,
            key: Key::Character("d".into()),
            action: ShortcutAction::ShowFileDiff,
        },
        // Ctrl+Alt+Left: Previous file
        KeyboardShortcut {
            modifiers: Modifiers::CTRL | Modifiers::ALT,
            key: Key::Named(Named::ArrowLeft),
            action: ShortcutAction::NavigatePrevFile,
        },
        // Ctrl+Alt+Right: Next file
        KeyboardShortcut {
            modifiers: Modifiers::CTRL | Modifiers::ALT,
            key: Key::Named(Named::ArrowRight),
            action: ShortcutAction::NavigateNextFile,
        },
        // Ctrl+Enter: Commit (IDEA style)
        KeyboardShortcut {
            modifiers: Modifiers::CTRL,
            key: Key::Named(Named::Enter),
            action: ShortcutAction::Commit,
        },
        // Ctrl+Shift+Z: Save stash
        KeyboardShortcut {
            modifiers: Modifiers::CTRL | Modifiers::SHIFT,
            key: Key::Character("z".into()),
            action: ShortcutAction::StashSave,
        },
        // Ctrl+Z: Pop stash
        KeyboardShortcut {
            modifiers: Modifiers::CTRL,
            key: Key::Character("z".into()),
            action: ShortcutAction::StashPop,
        },
        // Ctrl+Alt+Z: Drop stash
        KeyboardShortcut {
            modifiers: Modifiers::CTRL | Modifiers::ALT,
            key: Key::Character("z".into()),
            action: ShortcutAction::StashDrop,
        },
        // F7: Next hunk (IDEA style)
        KeyboardShortcut {
            modifiers: Modifiers::empty(),
            key: Key::Named(Named::F7),
            action: ShortcutAction::NextHunk,
        },
        // Shift+F7: Previous hunk (IDEA style)
        KeyboardShortcut {
            modifiers: Modifiers::SHIFT,
            key: Key::Named(Named::F7),
            action: ShortcutAction::PrevHunk,
        },
    ]
}

/// Find the action for a keyboard event
pub fn find_action(event: &Event) -> Option<ShortcutAction> {
    for shortcut in get_shortcuts() {
        if shortcut.matches(event) {
            return Some(shortcut.action);
        }
    }
    None
}

/// Format a keyboard shortcut for display
pub fn format_shortcut(shortcut: &KeyboardShortcut) -> String {
    let mut parts = Vec::new();

    if shortcut.modifiers.contains(keyboard::Modifiers::CTRL) {
        parts.push("Ctrl".to_string());
    }
    if shortcut.modifiers.contains(keyboard::Modifiers::SHIFT) {
        parts.push("Shift".to_string());
    }
    if shortcut.modifiers.contains(keyboard::Modifiers::ALT) {
        parts.push("Alt".to_string());
    }

    if let keyboard::Key::Character(c) = &shortcut.key {
        parts.push(c.to_uppercase());
    } else {
        parts.push(format!("{:?}", shortcut.key));
    }

    parts.join("+")
}

/// Get the description for a shortcut action
pub fn action_description(action: ShortcutAction) -> &'static str {
    match action {
        ShortcutAction::StageFile => "暂存选中文件",
        ShortcutAction::UnstageFile => "取消暂存选中文件",
        ShortcutAction::StageAll => "暂存全部",
        ShortcutAction::UnstageAll => "取消暂存全部",
        ShortcutAction::Refresh => "刷新",
        ShortcutAction::ToggleChangesPanel => "切换变更面板",
        ShortcutAction::OpenCommitDialog => "打开提交对话框",
        ShortcutAction::ToggleAmendCommitMode => "切换 amend 模式",
        ShortcutAction::OpenPushDialog => "打开推送对话框",
        ShortcutAction::ShowFileDiff => "显示文件差异",
        ShortcutAction::NavigatePrevFile => "上一个文件",
        ShortcutAction::NavigateNextFile => "下一个文件",
        ShortcutAction::PrevHunk => "上一个差异块",
        ShortcutAction::NextHunk => "下一个差异块",
        ShortcutAction::Commit => "提交",
        ShortcutAction::StashSave => "保存储藏",
        ShortcutAction::StashPop => "弹出储藏",
        ShortcutAction::StashDrop => "删除储藏",
        ShortcutAction::StashList => "列出储藏",
    }
}

#[cfg(test)]
mod tests {
    use super::{action_description, get_shortcuts, KeyboardShortcut, ShortcutAction};
    use iced::keyboard::{Key, Modifiers};

    fn has_shortcut(shortcut: &KeyboardShortcut, modifiers: Modifiers, key: Key) -> bool {
        shortcut.modifiers == modifiers && shortcut.key == key
    }

    #[test]
    fn idea_style_toggle_amend_shortcuts_are_registered() {
        let shortcuts = get_shortcuts();

        assert!(shortcuts.iter().any(|shortcut| {
            shortcut.action == ShortcutAction::ToggleAmendCommitMode
                && has_shortcut(shortcut, Modifiers::ALT, Key::Character("m".into()))
        }));

        assert!(shortcuts.iter().any(|shortcut| {
            shortcut.action == ShortcutAction::ToggleAmendCommitMode
                && has_shortcut(
                    shortcut,
                    Modifiers::CTRL | Modifiers::ALT,
                    Key::Character("m".into()),
                )
        }));
    }

    #[test]
    fn toggle_amend_shortcut_has_user_facing_description() {
        assert_eq!(
            action_description(ShortcutAction::ToggleAmendCommitMode),
            "切换 amend 模式"
        );
    }
}
