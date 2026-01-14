//! Typst parser for converting Typst documents to HTML.
//!
//! This module provides Typst document parsing with frontmatter extraction
//! and TOC generation. The actual Typst compilation requires setting up
//! a proper TypstWorld which is deferred to the generator phase.

use std::path::Path;

use thiserror::Error;
use typstify_core::{
    content::{ParsedContent, TocEntry},
    frontmatter::parse_typst_frontmatter,
};

/// Typst parsing errors.
#[derive(Debug, Error)]
pub enum TypstError {
    /// Failed to parse frontmatter.
    #[error("frontmatter error: {0}")]
    Frontmatter(#[from] typstify_core::error::CoreError),

    /// Typst compilation error.
    #[error("typst compilation failed: {0}")]
    Compilation(String),

    /// SVG rendering error.
    #[error("SVG rendering failed: {0}")]
    Render(String),
}

/// Result type for Typst operations.
pub type Result<T> = std::result::Result<T, TypstError>;

/// Typst parser that extracts frontmatter and prepares content for compilation.
///
/// Note: Full Typst compilation requires a proper World implementation with
/// file system access and font loading. This parser focuses on:
/// - Extracting frontmatter from Typst comment syntax
/// - Extracting TOC from heading patterns
/// - Preparing content for later compilation
#[derive(Debug)]
pub struct TypstParser {
    /// Whether to extract TOC from headings.
    extract_toc: bool,
}

impl Default for TypstParser {
    fn default() -> Self {
        Self::new()
    }
}

impl TypstParser {
    /// Create a new Typst parser.
    pub fn new() -> Self {
        Self { extract_toc: true }
    }

    /// Parse a Typst document with frontmatter.
    ///
    /// This extracts frontmatter and TOC but does not perform full compilation.
    /// The HTML field will contain the raw Typst source wrapped in a code block
    /// for preview, or can be compiled later with a proper World implementation.
    pub fn parse(&self, content: &str, path: &Path) -> Result<ParsedContent> {
        // Parse frontmatter from Typst comments
        let (frontmatter, body) = parse_typst_frontmatter(content, path)?;

        // Extract TOC from source
        let toc = if self.extract_toc {
            self.extract_toc_from_source(&body)
        } else {
            Vec::new()
        };

        // For now, wrap the Typst source in a placeholder
        // Full compilation will be done in the generator with proper World setup
        let html = format!(
            "<div class=\"typst-source\" data-path=\"{}\">\n<pre><code class=\"language-typst\">{}</code></pre>\n</div>",
            path.display(),
            html_escape(&body)
        );

        Ok(ParsedContent {
            frontmatter,
            html,
            raw: body,
            toc,
        })
    }

    /// Extract TOC entries from Typst source (simple heuristic).
    fn extract_toc_from_source(&self, content: &str) -> Vec<TocEntry> {
        let mut toc = Vec::new();

        for line in content.lines() {
            let trimmed = line.trim();

            // Match Typst headings: = Title, == Subtitle, etc.
            if let Some(heading) = parse_typst_heading(trimmed) {
                toc.push(heading);
            }
        }

        toc
    }
}

/// Parse a Typst heading line into a TocEntry.
fn parse_typst_heading(line: &str) -> Option<TocEntry> {
    if !line.starts_with('=') {
        return None;
    }

    // Count the number of = at the start
    let level = line.chars().take_while(|c| *c == '=').count();
    if level == 0 || level > 6 {
        return None;
    }

    // Extract the heading text
    let text = line[level..].trim().to_string();
    if text.is_empty() {
        return None;
    }

    // Generate a slug from the text
    let id = slugify(&text);

    Some(TocEntry {
        level: level as u8,
        text,
        id,
    })
}

/// Convert text to a URL-safe slug.
fn slugify(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c
            } else if c.is_whitespace() || c == '-' || c == '_' {
                '-'
            } else {
                '\0'
            }
        })
        .filter(|c| *c != '\0')
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// Escape HTML special characters.
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_typst_heading() {
        let h1 = parse_typst_heading("= Introduction").unwrap();
        assert_eq!(h1.level, 1);
        assert_eq!(h1.text, "Introduction");

        let h2 = parse_typst_heading("== Sub Section").unwrap();
        assert_eq!(h2.level, 2);
        assert_eq!(h2.text, "Sub Section");

        assert!(parse_typst_heading("Not a heading").is_none());
        assert!(parse_typst_heading("=").is_none()); // Empty heading
    }

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Hello World"), "hello-world");
        assert_eq!(slugify("Test 123"), "test-123");
    }

    #[test]
    fn test_extract_toc() {
        let parser = TypstParser::new();
        let content = r#"= Main Title
== Section One
=== Subsection
== Section Two"#;

        let toc = parser.extract_toc_from_source(content);

        assert_eq!(toc.len(), 4);
        assert_eq!(toc[0].level, 1);
        assert_eq!(toc[0].text, "Main Title");
        assert_eq!(toc[1].level, 2);
        assert_eq!(toc[2].level, 3);
    }

    #[test]
    fn test_parse_with_frontmatter() {
        let parser = TypstParser::new();
        let content = r#"// typstify:frontmatter
// title: "Test Document"

= Hello Typst

This is a test document."#;

        let result = parser.parse(content, Path::new("test.typ")).unwrap();

        assert_eq!(result.frontmatter.title, "Test Document");
        assert!(!result.toc.is_empty());
        assert!(result.html.contains("typst-source"));
    }

    #[test]
    fn test_html_escape() {
        assert_eq!(html_escape("<script>"), "&lt;script&gt;");
        assert_eq!(html_escape("a & b"), "a &amp; b");
    }
}
