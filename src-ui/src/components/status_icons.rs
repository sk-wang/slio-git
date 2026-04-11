//! Status icon helpers for file changes and shell badges.

use crate::theme::{darcula, BadgeTone};
use iced::widget::{container, Container, Text};
use iced::{Background, Border, Color, Element};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileStatus {
    Added,
    Modified,
    Deleted,
    Renamed,
    Unversioned,
    Conflict,
    Ignored,
}

impl FileStatus {
    pub fn color(self) -> Color {
        match self {
            FileStatus::Added => darcula::STATUS_ADDED,
            FileStatus::Modified => darcula::STATUS_MODIFIED,
            FileStatus::Deleted => darcula::STATUS_DELETED,
            FileStatus::Renamed => darcula::STATUS_RENAMED,
            FileStatus::Unversioned => darcula::STATUS_UNVERSIONED,
            FileStatus::Conflict => darcula::DANGER,
            FileStatus::Ignored => darcula::TEXT_DISABLED,
        }
    }

    /// IDEA-style status badge with colored border and text
    pub fn badge<'a, Message: 'a>(self) -> Element<'a, Message> {
        let (symbol, _label) = self.symbol_and_label();
        let _tone = self.badge_tone();

        Container::new(Text::new(symbol).size(10).color(self.color()))
            .padding([2, 6])
            .style(badge_container_style(self))
            .into()
    }

    fn symbol_and_label(self) -> (&'static str, &'static str) {
        match self {
            FileStatus::Added => ("A", "Added"),
            FileStatus::Modified => ("M", "Modified"),
            FileStatus::Deleted => ("D", "Deleted"),
            FileStatus::Renamed => ("R", "Renamed"),
            FileStatus::Unversioned => ("U", "Untracked"),
            FileStatus::Conflict => ("!", "Conflict"),
            FileStatus::Ignored => ("I", "Ignored"),
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            FileStatus::Added => "Added File",
            FileStatus::Modified => "Modified File",
            FileStatus::Deleted => "Deleted File",
            FileStatus::Renamed => "Renamed",
            FileStatus::Unversioned => "Untracked",
            FileStatus::Conflict => "Has Conflicts",
            FileStatus::Ignored => "Ignored",
        }
    }

    pub fn badge_tone(self) -> BadgeTone {
        match self {
            FileStatus::Added => BadgeTone::Success,
            FileStatus::Modified => BadgeTone::Accent,
            FileStatus::Renamed => BadgeTone::Accent,
            FileStatus::Deleted => BadgeTone::Danger,
            FileStatus::Conflict => BadgeTone::Danger,
            FileStatus::Unversioned | FileStatus::Ignored => BadgeTone::Neutral,
        }
    }

    /// IDEA-style short symbol (single letter)
    pub fn symbol(self) -> &'static str {
        match self {
            FileStatus::Added => "A",
            FileStatus::Modified => "M",
            FileStatus::Deleted => "D",
            FileStatus::Renamed => "R",
            FileStatus::Unversioned => "U",
            FileStatus::Conflict => "!",
            FileStatus::Ignored => "I",
        }
    }
}

/// IDEA-style badge container with colored border matching status
fn badge_container_style(status: FileStatus) -> impl Fn(&iced::Theme) -> container::Style {
    move |_theme| container::Style {
        background: Some(Background::Color(background_color(status))),
        border: Border {
            width: 1.0,
            color: status.color().scale_alpha(0.7),
            radius: 3.0.into(),
        },
        ..Default::default()
    }
}

fn background_color(status: FileStatus) -> Color {
    match status {
        FileStatus::Added => darcula::DIFF_ADDED_BG,
        FileStatus::Modified => darcula::DIFF_MODIFIED_BG,
        FileStatus::Deleted => darcula::DIFF_DELETED_BG,
        FileStatus::Renamed => darcula::BG_RAISED,
        FileStatus::Unversioned => darcula::BG_RAISED,
        FileStatus::Conflict => Color::from_rgb(0.18, 0.06, 0.08),
        FileStatus::Ignored => darcula::BG_RAISED,
    }
}

impl From<&git_core::index::ChangeStatus> for FileStatus {
    fn from(status: &git_core::index::ChangeStatus) -> Self {
        match status {
            git_core::index::ChangeStatus::Added => FileStatus::Added,
            git_core::index::ChangeStatus::Modified => FileStatus::Modified,
            git_core::index::ChangeStatus::Deleted => FileStatus::Deleted,
            git_core::index::ChangeStatus::Renamed => FileStatus::Renamed,
            git_core::index::ChangeStatus::Untracked => FileStatus::Unversioned,
            git_core::index::ChangeStatus::Conflict => FileStatus::Conflict,
            git_core::index::ChangeStatus::Ignored => FileStatus::Ignored,
        }
    }
}
