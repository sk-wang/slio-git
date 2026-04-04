//! Syntax highlighting helpers shared by diff viewers.

use crate::theme;
use git_core::diff::{DiffLineOrigin, FileDiff};
use iced::widget::{rich_text, span, text};
use iced::{Color, Element, Font, Length};
use once_cell::sync::Lazy;
use std::path::Path;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Color as SyntectColor, Theme, ThemeSet};
use syntect::parsing::{SyntaxReference, SyntaxSet};

static SYNTAX_SET: Lazy<SyntaxSet> = Lazy::new(SyntaxSet::load_defaults_nonewlines);
static SYNTAX_THEME: Lazy<Theme> = Lazy::new(|| {
    let themes = ThemeSet::load_defaults();

    [
        "base16-eighties.dark",
        "base16-ocean.dark",
        "Solarized (dark)",
    ]
    .into_iter()
    .find_map(|name| themes.themes.get(name).cloned())
    .or_else(|| themes.themes.values().next().cloned())
    .expect("syntect should provide at least one default theme")
});

pub struct FileSyntaxHighlighter {
    syntax: Option<&'static SyntaxReference>,
}

pub struct CodeSyntaxHighlighter {
    syntax: Option<&'static SyntaxReference>,
}

#[derive(Debug, Clone)]
pub struct HighlightedSegment {
    text: String,
    color: Color,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct HighlightRenderConfig {
    width: Length,
    wrapping: text::Wrapping,
}

pub struct HunkSyntaxHighlighter {
    old: Option<HighlightLines<'static>>,
    new: Option<HighlightLines<'static>>,
}

pub struct CodeLineHighlighter {
    inner: Option<HighlightLines<'static>>,
}

impl FileSyntaxHighlighter {
    pub fn for_file_diff(file_diff: &FileDiff) -> Self {
        let syntax = file_diff
            .new_path
            .as_deref()
            .or(file_diff.old_path.as_deref())
            .and_then(resolve_syntax_for_path);

        Self { syntax }
    }

    pub fn start_hunk(&self) -> HunkSyntaxHighlighter {
        let old = self
            .syntax
            .map(|syntax| HighlightLines::new(syntax, &SYNTAX_THEME));
        let new = self
            .syntax
            .map(|syntax| HighlightLines::new(syntax, &SYNTAX_THEME));

        HunkSyntaxHighlighter { old, new }
    }
}

impl CodeSyntaxHighlighter {
    pub fn for_path(path: &str) -> Self {
        Self {
            syntax: resolve_syntax_for_path(path),
        }
    }

    pub fn start(&self) -> CodeLineHighlighter {
        CodeLineHighlighter {
            inner: self
                .syntax
                .map(|syntax| HighlightLines::new(syntax, &SYNTAX_THEME)),
        }
    }
}

impl HunkSyntaxHighlighter {
    pub fn highlight_segments(
        &mut self,
        origin: &DiffLineOrigin,
        content: &str,
    ) -> Vec<HighlightedSegment> {
        let sanitized = sanitize_content(content);
        self.highlight_spans(origin, &sanitized)
            .into_iter()
            .map(|span| HighlightedSegment {
                text: span.content,
                color: span.color,
            })
            .collect()
    }
}

impl CodeLineHighlighter {
    pub fn highlight_segments(&mut self, content: &str) -> Vec<HighlightedSegment> {
        let sanitized = sanitize_content(content);
        highlight_plain_spans(self.inner.as_mut(), &sanitized)
            .into_iter()
            .map(|span| HighlightedSegment {
                text: span.content,
                color: span.color,
            })
            .collect()
    }
}

impl HighlightedSegment {
    pub fn render<Message: Clone + 'static>(segments: &[Self]) -> Element<'static, Message> {
        render_segments(segments, default_render_config())
    }

    pub fn render_diff_code<Message: Clone + 'static>(
        segments: &[Self],
    ) -> Element<'static, Message> {
        render_segments(segments, diff_code_render_config())
    }

    /// Render with inline change highlighting — changed characters get a brighter background.
    /// This produces GitHub-style character-level diff highlighting.
    pub fn render_diff_code_with_inline<Message: Clone + 'static>(
        segments: &[Self],
        inline_changes: &[git_core::diff::InlineChangeSpan],
        is_addition: bool,
    ) -> Element<'static, Message> {
        if inline_changes.is_empty() {
            return render_segments(segments, diff_code_render_config());
        }

        let change_bg = if is_addition {
            Color::from_rgba(0.0, 0.68, 0.37, 0.30) // green tint for additions
        } else {
            Color::from_rgba(1.0, 0.32, 0.32, 0.30) // red tint for deletions
        };

        // Build a position→color lookup from syntax segments
        let mut color_map: Vec<(usize, usize, Color)> = Vec::new();
        let mut pos = 0;
        for seg in segments {
            let end = pos + seg.text.len();
            color_map.push((pos, end, seg.color));
            pos = end;
        }

        let full_text: String = segments.iter().map(|s| s.text.as_str()).collect();
        let mut result_spans: Vec<text::Span<'static, Message, Font>> = Vec::new();

        for inline in inline_changes {
            let end = (inline.start + inline.len).min(full_text.len());
            let text_slice = full_text.get(inline.start..end).unwrap_or("").to_string();
            if text_slice.is_empty() {
                continue;
            }

            // Find syntax color at this position
            let color = color_map.iter()
                .find(|(s, e, _)| *s <= inline.start && *e > inline.start)
                .map(|(_, _, c)| *c)
                .unwrap_or(crate::theme::darcula::TEXT_PRIMARY);

            let mut s = span(text_slice).color(color);
            if inline.changed {
                s = s.background(iced::Background::Color(change_bg));
            }
            result_spans.push(s);
        }

        if result_spans.is_empty() {
            return render_segments(segments, diff_code_render_config());
        }

        rich_text(result_spans)
            .size(11)
            .line_height(text::LineHeight::Relative(1.30))
            .font(Font::MONOSPACE)
            .wrapping(text::Wrapping::None)
            .width(Length::Shrink)
            .into()
    }
}

fn render_segments<Message: Clone + 'static>(
    segments: &[HighlightedSegment],
    config: HighlightRenderConfig,
) -> Element<'static, Message> {
    let spans: Vec<text::Span<'static, Message, Font>> = segments
        .iter()
        .map(|segment| span(segment.text.clone()).color(segment.color))
        .collect();

    rich_text(spans)
        .size(11)
        .line_height(text::LineHeight::Relative(1.30))
        .font(Font::MONOSPACE)
        .wrapping(config.wrapping)
        .width(config.width)
        .into()
}

fn default_render_config() -> HighlightRenderConfig {
    HighlightRenderConfig {
        width: Length::Fill,
        wrapping: text::Wrapping::WordOrGlyph,
    }
}

fn diff_code_render_config() -> HighlightRenderConfig {
    HighlightRenderConfig {
        width: Length::Shrink,
        wrapping: text::Wrapping::None,
    }
}

#[derive(Debug, Clone)]
struct OwnedSpan {
    content: String,
    color: Color,
}

impl HunkSyntaxHighlighter {
    pub fn view<Message: Clone + 'static>(
        &mut self,
        origin: &DiffLineOrigin,
        content: &str,
    ) -> Element<'static, Message> {
        let segments = self.highlight_segments(origin, content);
        HighlightedSegment::render(&segments)
    }

    pub fn view_diff_code<Message: Clone + 'static>(
        &mut self,
        origin: &DiffLineOrigin,
        content: &str,
    ) -> Element<'static, Message> {
        let segments = self.highlight_segments(origin, content);
        HighlightedSegment::render_diff_code(&segments)
    }

    /// Render code with inline character-level change highlighting (GitHub-style).
    pub fn view_diff_code_with_inline<Message: Clone + 'static>(
        &mut self,
        origin: &DiffLineOrigin,
        content: &str,
        inline_changes: &[git_core::diff::InlineChangeSpan],
    ) -> Element<'static, Message> {
        let segments = self.highlight_segments(origin, content);
        let is_addition = matches!(origin, DiffLineOrigin::Addition);
        HighlightedSegment::render_diff_code_with_inline(&segments, inline_changes, is_addition)
    }
}

impl HunkSyntaxHighlighter {
    fn highlight_spans(&mut self, origin: &DiffLineOrigin, content: &str) -> Vec<OwnedSpan> {
        let highlighted = match origin {
            DiffLineOrigin::Context => {
                let highlighted = self
                    .old
                    .as_mut()
                    .and_then(|highlighter| highlight_line(highlighter, content));

                if let Some(highlighter) = self.new.as_mut() {
                    let _ = highlight_line(highlighter, content);
                }

                highlighted
            }
            DiffLineOrigin::Deletion => self
                .old
                .as_mut()
                .and_then(|highlighter| highlight_line(highlighter, content)),
            DiffLineOrigin::Addition => self
                .new
                .as_mut()
                .and_then(|highlighter| highlight_line(highlighter, content)),
            DiffLineOrigin::Header | DiffLineOrigin::HunkHeader => None,
        };

        highlighted
            .map(|ranges| {
                ranges
                    .into_iter()
                    .map(|(style, text)| OwnedSpan {
                        content: text.to_string(),
                        color: adjust_highlight_color(style.foreground),
                    })
                    .collect()
            })
            .unwrap_or_else(|| {
                vec![OwnedSpan {
                    content: content.to_string(),
                    color: theme::darcula::TEXT_PRIMARY,
                }]
            })
    }
}

fn resolve_syntax_for_path(path: &str) -> Option<&'static SyntaxReference> {
    let syntax_set = &*SYNTAX_SET;
    let file_name = Path::new(path)
        .file_name()
        .and_then(|segment| segment.to_str())
        .unwrap_or(path);
    let lowered_name = file_name.to_ascii_lowercase();

    match lowered_name.as_str() {
        "dockerfile" | "containerfile" => syntax_set.find_syntax_by_token("Dockerfile"),
        "makefile" | "gnumakefile" => syntax_set
            .find_syntax_by_name("Makefile")
            .or_else(|| syntax_set.find_syntax_by_token("Makefile")),
        "cmakelists.txt" => syntax_set
            .find_syntax_by_extension("cmake")
            .or_else(|| syntax_set.find_syntax_by_name("CMake")),
        ".bashrc" | ".bash_profile" | ".zshrc" | ".zprofile" | ".zshenv" | ".profile" => syntax_set
            .find_syntax_by_extension("sh")
            .or_else(|| syntax_set.find_syntax_by_name("Bourne Again Shell (bash)")),
        _ => {
            let extension = Path::new(file_name)
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.to_ascii_lowercase());

            extension
                .as_deref()
                .and_then(|ext| syntax_for_extension(syntax_set, ext))
                .or_else(|| syntax_set.find_syntax_by_token(file_name))
        }
    }
}

fn syntax_for_extension<'a>(
    syntax_set: &'a SyntaxSet,
    extension: &str,
) -> Option<&'a SyntaxReference> {
    syntax_set
        .find_syntax_by_extension(extension)
        .or_else(|| match extension {
            "tsx" => syntax_set
                .find_syntax_by_name("TypeScriptReact")
                .or_else(|| syntax_set.find_syntax_by_name("TypeScript React")),
            "jsx" => syntax_set
                .find_syntax_by_name("JavaScriptReact")
                .or_else(|| syntax_set.find_syntax_by_name("JavaScript React")),
            "kt" | "kts" => syntax_set.find_syntax_by_name("Kotlin"),
            "yml" => syntax_set.find_syntax_by_extension("yaml"),
            _ => None,
        })
}

fn highlight_line<'a>(
    highlighter: &mut HighlightLines<'static>,
    content: &'a str,
) -> Option<Vec<(syntect::highlighting::Style, &'a str)>> {
    highlighter.highlight_line(content, &SYNTAX_SET).ok()
}

fn highlight_plain_spans(
    highlighter: Option<&mut HighlightLines<'static>>,
    content: &str,
) -> Vec<OwnedSpan> {
    highlighter
        .and_then(|highlighter| highlight_line(highlighter, content))
        .map(|ranges| {
            ranges
                .into_iter()
                .map(|(style, text)| OwnedSpan {
                    content: text.to_string(),
                    color: adjust_highlight_color(style.foreground),
                })
                .collect()
        })
        .unwrap_or_else(|| {
            vec![OwnedSpan {
                content: content.to_string(),
                color: theme::darcula::TEXT_PRIMARY,
            }]
        })
}

fn sanitize_content(content: &str) -> String {
    if content.is_empty() {
        return " ".to_string();
    }

    content.replace('\t', "    ")
}

fn adjust_highlight_color(color: SyntectColor) -> Color {
    let color = Color::from_rgba8(color.r, color.g, color.b, (color.a as f32) / 255.0);
    mix_colors(color, theme::darcula::TEXT_PRIMARY, 0.18)
}

fn mix_colors(base: Color, overlay: Color, amount: f32) -> Color {
    let amount = amount.clamp(0.0, 1.0);

    Color {
        r: (base.r * (1.0 - amount)) + (overlay.r * amount),
        g: (base.g * (1.0 - amount)) + (overlay.g * amount),
        b: (base.b * (1.0 - amount)) + (overlay.b * amount),
        a: (base.a * (1.0 - amount)) + (overlay.a * amount),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        diff_code_render_config, resolve_syntax_for_path, sanitize_content, FileSyntaxHighlighter,
        HighlightRenderConfig,
    };
    use git_core::diff::FileDiff;
    use iced::{widget::text, Length};

    #[test]
    fn resolve_syntax_for_common_extensions() {
        assert!(resolve_syntax_for_path("src/main.rs").is_some());
        assert!(resolve_syntax_for_path("app/intent/__init__.py").is_some());
    }

    #[test]
    fn sanitize_content_keeps_empty_lines_visible() {
        assert_eq!(sanitize_content(""), " ");
        assert_eq!(sanitize_content("\tlet value = 1;"), "    let value = 1;");
    }

    #[test]
    fn file_highlighter_detects_path_from_diff() {
        let file_diff = FileDiff {
            old_path: Some("src/main.rs".to_string()),
            new_path: Some("src/main.rs".to_string()),
            hunks: Vec::new(),
            additions: 0,
            deletions: 0,
        };

        let highlighter = FileSyntaxHighlighter::for_file_diff(&file_diff);
        let mut hunk = highlighter.start_hunk();
        let _ = hunk.view::<()>(
            &git_core::diff::DiffLineOrigin::Context,
            "fn main() { println!(\"hi\"); }",
        );
    }

    #[test]
    fn diff_code_render_config_disables_wrapping_for_scrollable_editors() {
        assert_eq!(
            diff_code_render_config(),
            HighlightRenderConfig {
                width: Length::Shrink,
                wrapping: text::Wrapping::None,
            }
        );
    }
}
