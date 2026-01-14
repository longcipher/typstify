//! Syntax highlighting for code blocks.

use syntect::{highlighting::ThemeSet, html::highlighted_html_for_string, parsing::SyntaxSet};
use thiserror::Error;

/// Syntax highlighting errors.
#[derive(Debug, Error)]
pub enum SyntaxError {
    /// Failed to highlight code.
    #[error("syntax highlighting failed: {0}")]
    Highlight(String),
}

/// Syntax highlighter using syntect.
#[derive(Debug)]
pub struct SyntaxHighlighter {
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
    default_theme: String,
}

impl Default for SyntaxHighlighter {
    fn default() -> Self {
        Self::new("base16-ocean.dark")
    }
}

impl SyntaxHighlighter {
    /// Create a new syntax highlighter with the specified theme.
    pub fn new(theme: &str) -> Self {
        Self {
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
            default_theme: theme.to_string(),
        }
    }

    /// Get available theme names.
    pub fn available_themes(&self) -> Vec<&str> {
        self.theme_set.themes.keys().map(|s| s.as_str()).collect()
    }

    /// Highlight code with the given language.
    ///
    /// If the language is not recognized, returns the code wrapped in a `<pre><code>` block.
    pub fn highlight(&self, code: &str, lang: Option<&str>) -> String {
        let syntax = lang
            .and_then(|l| self.syntax_set.find_syntax_by_token(l))
            .or_else(|| self.syntax_set.find_syntax_by_extension("txt"));

        let theme = self
            .theme_set
            .themes
            .get(&self.default_theme)
            .or_else(|| self.theme_set.themes.values().next());

        match (syntax, theme) {
            (Some(syntax), Some(theme)) => {
                match highlighted_html_for_string(code, &self.syntax_set, syntax, theme) {
                    Ok(html) => html,
                    Err(_) => self.fallback_highlight(code, lang),
                }
            }
            _ => self.fallback_highlight(code, lang),
        }
    }

    /// Fallback highlighting when syntect fails.
    fn fallback_highlight(&self, code: &str, lang: Option<&str>) -> String {
        let escaped = html_escape(code);
        let lang_class = lang
            .map(|l| format!(" class=\"language-{l}\""))
            .unwrap_or_default();
        format!("<pre><code{lang_class}>{escaped}</code></pre>")
    }

    /// Set the default theme.
    pub fn set_theme(&mut self, theme: &str) {
        if self.theme_set.themes.contains_key(theme) {
            self.default_theme = theme.to_string();
        }
    }
}

/// Escape HTML special characters.
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_highlight_rust() {
        let highlighter = SyntaxHighlighter::default();
        let code = "fn main() {\n    println!(\"Hello\");\n}";
        let html = highlighter.highlight(code, Some("rust"));

        assert!(html.contains("<pre"));
        assert!(html.contains("fn"));
    }

    #[test]
    fn test_highlight_unknown_language() {
        let highlighter = SyntaxHighlighter::default();
        let code = "some code";
        let html = highlighter.highlight(code, Some("unknown_lang_xyz"));

        // Should fall back gracefully
        assert!(html.contains("some code"));
    }

    #[test]
    fn test_highlight_no_language() {
        let highlighter = SyntaxHighlighter::default();
        let code = "plain text";
        let html = highlighter.highlight(code, None);

        assert!(html.contains("plain text"));
    }

    #[test]
    fn test_html_escape() {
        assert_eq!(html_escape("<script>"), "&lt;script&gt;");
        assert_eq!(html_escape("a & b"), "a &amp; b");
    }

    #[test]
    fn test_available_themes() {
        let highlighter = SyntaxHighlighter::default();
        let themes = highlighter.available_themes();

        assert!(!themes.is_empty());
        assert!(themes.contains(&"base16-ocean.dark"));
    }
}
