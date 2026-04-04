//! Symbolic SVG icons used by the compact shell chrome.

use iced::widget::svg;
use iced::{Color, Element, Length};
use once_cell::sync::Lazy;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RailIcon {
    Repository,
    OpenRepository,
    Branch,
    Overview,
    Changes,
    Conflicts,
    History,
    Remotes,
    Tags,
    Stashes,
    Rebase,
}

pub fn view<'a, Message: 'a>(
    icon: RailIcon,
    color: Color,
    hover_color: Color,
    size: f32,
) -> Element<'a, Message> {
    svg::Svg::new(handle(icon))
        .width(Length::Fixed(size))
        .height(Length::Fixed(size))
        .style(move |_theme, status| svg::Style {
            color: Some(match status {
                svg::Status::Hovered => hover_color,
                svg::Status::Idle => color,
            }),
        })
        .into()
}

fn handle(icon: RailIcon) -> svg::Handle {
    match icon {
        RailIcon::Repository => REPOSITORY_ICON.clone(),
        RailIcon::OpenRepository => OPEN_REPOSITORY_ICON.clone(),
        RailIcon::Branch => BRANCH_ICON.clone(),
        RailIcon::Overview => OVERVIEW_ICON.clone(),
        RailIcon::Changes => CHANGES_ICON.clone(),
        RailIcon::Conflicts => CONFLICTS_ICON.clone(),
        RailIcon::History => HISTORY_ICON.clone(),
        RailIcon::Remotes => REMOTES_ICON.clone(),
        RailIcon::Tags => TAGS_ICON.clone(),
        RailIcon::Stashes => STASHES_ICON.clone(),
        RailIcon::Rebase => REBASE_ICON.clone(),
    }
}

static REPOSITORY_ICON: Lazy<svg::Handle> = Lazy::new(|| svg::Handle::from_memory(REPOSITORY_SVG));
static OPEN_REPOSITORY_ICON: Lazy<svg::Handle> =
    Lazy::new(|| svg::Handle::from_memory(OPEN_REPOSITORY_SVG));
static BRANCH_ICON: Lazy<svg::Handle> = Lazy::new(|| svg::Handle::from_memory(BRANCH_SVG));
static OVERVIEW_ICON: Lazy<svg::Handle> = Lazy::new(|| svg::Handle::from_memory(OVERVIEW_SVG));
static CHANGES_ICON: Lazy<svg::Handle> = Lazy::new(|| svg::Handle::from_memory(CHANGES_SVG));
static CONFLICTS_ICON: Lazy<svg::Handle> = Lazy::new(|| svg::Handle::from_memory(CONFLICTS_SVG));
static HISTORY_ICON: Lazy<svg::Handle> = Lazy::new(|| svg::Handle::from_memory(HISTORY_SVG));
static REMOTES_ICON: Lazy<svg::Handle> = Lazy::new(|| svg::Handle::from_memory(REMOTES_SVG));
static TAGS_ICON: Lazy<svg::Handle> = Lazy::new(|| svg::Handle::from_memory(TAGS_SVG));
static STASHES_ICON: Lazy<svg::Handle> = Lazy::new(|| svg::Handle::from_memory(STASHES_SVG));
static REBASE_ICON: Lazy<svg::Handle> = Lazy::new(|| svg::Handle::from_memory(REBASE_SVG));

// Lucide-style polished icons — rounded strokes, balanced proportions
const REPOSITORY_SVG: &[u8] = br#"
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
  <path d="M4 20h16a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.93a2 2 0 0 1-1.66-.9l-.82-1.2A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13c0 1.1.9 2 2 2Z"/>
  <circle cx="12" cy="13" r="1" fill="currentColor" stroke="none"/>
</svg>
"#;

const OPEN_REPOSITORY_SVG: &[u8] = br#"
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
  <path d="M4 20h16a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.93a2 2 0 0 1-1.66-.9l-.82-1.2A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13c0 1.1.9 2 2 2Z"/>
  <line x1="12" y1="10" x2="12" y2="16"/>
  <line x1="9" y1="13" x2="15" y2="13"/>
</svg>
"#;

const BRANCH_SVG: &[u8] = br#"
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
  <line x1="6" y1="3" x2="6" y2="15"/>
  <circle cx="18" cy="6" r="3"/>
  <circle cx="6" cy="18" r="3"/>
  <path d="M18 9a9 9 0 0 1-9 9"/>
</svg>
"#;

const OVERVIEW_SVG: &[u8] = br#"
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
  <rect width="7" height="7" x="3" y="3" rx="1"/>
  <rect width="7" height="7" x="14" y="3" rx="1"/>
  <rect width="7" height="7" x="14" y="14" rx="1"/>
  <rect width="7" height="7" x="3" y="14" rx="1"/>
</svg>
"#;

const CHANGES_SVG: &[u8] = br#"
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
  <path d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z"/>
  <polyline points="14 2 14 8 20 8"/>
  <line x1="9" y1="15" x2="15" y2="15"/>
</svg>
"#;

const CONFLICTS_SVG: &[u8] = br#"
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
  <path d="m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3Z"/>
  <line x1="12" y1="9" x2="12" y2="13"/>
  <line x1="12" y1="17" x2="12.01" y2="17"/>
</svg>
"#;

const HISTORY_SVG: &[u8] = br#"
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
  <circle cx="12" cy="12" r="10"/>
  <polyline points="12 6 12 12 16 14"/>
</svg>
"#;

const REMOTES_SVG: &[u8] = br#"
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
  <line x1="7" y1="17" x2="7" y2="7"/>
  <polyline points="4 10 7 7 10 10"/>
  <line x1="17" y1="7" x2="17" y2="17"/>
  <polyline points="14 14 17 17 20 14"/>
</svg>
"#;

const TAGS_SVG: &[u8] = br#"
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
  <path d="M12 2H2v10l9.29 9.29c.94.94 2.48.94 3.42 0l6.58-6.58c.94-.94.94-2.48 0-3.42L12 2Z"/>
  <path d="M7 7h.01"/>
</svg>
"#;

const STASHES_SVG: &[u8] = br#"
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
  <path d="m20 7-8-4-8 4"/>
  <path d="M4 7v10l8 4 8-4V7"/>
  <line x1="12" y1="21" x2="12" y2="11"/>
  <line x1="20" y1="7" x2="12" y2="11"/>
  <line x1="4" y1="7" x2="12" y2="11"/>
</svg>
"#;

const REBASE_SVG: &[u8] = br#"
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
  <circle cx="5" cy="6" r="3"/>
  <circle cx="19" cy="6" r="3"/>
  <circle cx="12" cy="18" r="3"/>
  <path d="M5 9v3a4 4 0 0 0 4 4h2"/>
  <path d="M19 9v3a4 4 0 0 1-4 4h-2"/>
</svg>
"#;
