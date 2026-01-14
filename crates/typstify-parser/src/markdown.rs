//! Markdown parser using pulldown-cmark.

use std::path::Path;

use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag, TagEnd};
use thiserror::Error;
use typstify_core::{
    content::{ParsedContent, TocEntry},
    frontmatter::parse_frontmatter,
};

use crate::syntax::SyntaxHighlighter;

/// Markdown parsing errors.
#[derive(Debug, Error)]
pub enum MarkdownError {
    /// Failed to parse frontmatter.
    #[error("frontmatter error: {0}")]
    Frontmatter(#[from] typstify_core::error::CoreError),
}

/// Result type for markdown operations.
pub type Result<T> = std::result::Result<T, MarkdownError>;

/// Markdown parser with syntax highlighting support.
#[derive(Debug)]
pub struct MarkdownParser {
    highlighter: SyntaxHighlighter,
    options: Options,
}

impl Default for MarkdownParser {
    fn default() -> Self {
        Self::new()
    }
}

impl MarkdownParser {
    /// Create a new markdown parser with default options.
    pub fn new() -> Self {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_FOOTNOTES);
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TASKLISTS);
        options.insert(Options::ENABLE_HEADING_ATTRIBUTES);

        Self {
            highlighter: SyntaxHighlighter::default(),
            options,
        }
    }

    /// Create a parser with a custom syntax theme.
    pub fn with_theme(theme: &str) -> Self {
        let mut parser = Self::new();
        parser.highlighter.set_theme(theme);
        parser
    }

    /// Parse markdown content with frontmatter.
    pub fn parse(&self, content: &str, path: &Path) -> Result<ParsedContent> {
        // Split frontmatter from body
        let (frontmatter, body) = parse_frontmatter(content, path)?;

        // Parse the markdown body
        let (html, toc) = self.render_markdown(&body);

        Ok(ParsedContent {
            frontmatter,
            html,
            raw: body,
            toc,
        })
    }

    /// Parse markdown without frontmatter (body only).
    pub fn parse_body(&self, body: &str) -> (String, Vec<TocEntry>) {
        self.render_markdown(body)
    }

    /// Render markdown to HTML with TOC extraction.
    fn render_markdown(&self, content: &str) -> (String, Vec<TocEntry>) {
        let parser = Parser::new_ext(content, self.options);
        let mut toc = Vec::new();
        let mut html = String::new();
        let mut current_heading: Option<(u8, String)> = None;
        let mut code_block_lang: Option<String> = None;
        let mut code_block_content = String::new();

        for event in parser {
            match event {
                // Handle heading start
                Event::Start(Tag::Heading { level, id, .. }) => {
                    let lvl = level as u8;
                    current_heading = Some((lvl, String::new()));
                    let id_attr = id.map(|i| format!(" id=\"{i}\"")).unwrap_or_default();
                    html.push_str(&format!("<h{lvl}{id_attr}>"));
                }

                // Handle heading end
                Event::End(TagEnd::Heading(level)) => {
                    let lvl = level as u8;
                    if let Some((_, ref text)) = current_heading {
                        let id = slugify(text);
                        toc.push(TocEntry {
                            level: lvl,
                            text: text.clone(),
                            id: id.clone(),
                        });
                    }
                    html.push_str(&format!("</h{lvl}>"));
                    current_heading = None;
                }

                // Handle code block start
                Event::Start(Tag::CodeBlock(kind)) => {
                    code_block_lang = match kind {
                        CodeBlockKind::Fenced(lang) => {
                            let lang = lang.to_string();
                            if lang.is_empty() { None } else { Some(lang) }
                        }
                        CodeBlockKind::Indented => None,
                    };
                    code_block_content.clear();
                }

                // Handle code block end
                Event::End(TagEnd::CodeBlock) => {
                    let highlighted = self
                        .highlighter
                        .highlight(&code_block_content, code_block_lang.as_deref());
                    html.push_str(&highlighted);
                    code_block_lang = None;
                    code_block_content.clear();
                }

                // Handle text inside code blocks
                Event::Text(text)
                    if code_block_lang.is_some() || !code_block_content.is_empty() =>
                {
                    code_block_content.push_str(&text);
                }

                // Handle regular text
                Event::Text(text) => {
                    if let Some((_, ref mut heading_text)) = current_heading {
                        heading_text.push_str(&text);
                    }
                    html.push_str(&html_escape(&text));
                }

                // Handle code (inline)
                Event::Code(code) => {
                    if let Some((_, ref mut heading_text)) = current_heading {
                        heading_text.push_str(&code);
                    }
                    html.push_str(&format!("<code>{}</code>", html_escape(&code)));
                }

                // Handle soft breaks
                Event::SoftBreak => {
                    html.push('\n');
                }

                // Handle hard breaks
                Event::HardBreak => {
                    html.push_str("<br />\n");
                }

                // Handle other start tags
                Event::Start(tag) => {
                    html.push_str(&tag_to_html_start(&tag));
                }

                // Handle other end tags
                Event::End(tag) => {
                    html.push_str(&tag_to_html_end(&tag));
                }

                // Handle HTML
                Event::Html(raw) | Event::InlineHtml(raw) => {
                    html.push_str(&raw);
                }

                // Handle footnote references
                Event::FootnoteReference(name) => {
                    html.push_str(&format!(
                        "<sup class=\"footnote-ref\"><a href=\"#fn-{name}\">[{name}]</a></sup>"
                    ));
                }

                // Handle rules
                Event::Rule => {
                    html.push_str("<hr />\n");
                }

                // Handle task list markers
                Event::TaskListMarker(checked) => {
                    let checkbox = if checked {
                        "<input type=\"checkbox\" checked disabled />"
                    } else {
                        "<input type=\"checkbox\" disabled />"
                    };
                    html.push_str(checkbox);
                }

                Event::InlineMath(math) => {
                    html.push_str(&format!("<span class=\"math inline\">\\({math}\\)</span>"));
                }

                Event::DisplayMath(math) => {
                    html.push_str(&format!("<div class=\"math display\">\\[{math}\\]</div>"));
                }
            }
        }

        (html, toc)
    }
}

/// Convert a pulldown-cmark tag to HTML opening tag.
fn tag_to_html_start(tag: &Tag) -> String {
    match tag {
        Tag::Paragraph => "<p>".to_string(),
        Tag::Heading { level, id, .. } => {
            let id_attr = id
                .as_ref()
                .map(|i| format!(" id=\"{i}\""))
                .unwrap_or_default();
            format!("<h{}{id_attr}>", *level as u8)
        }
        Tag::BlockQuote(_) => "<blockquote>".to_string(),
        Tag::CodeBlock(_) => String::new(), // Handled separately
        Tag::List(Some(start)) => format!("<ol start=\"{start}\">"),
        Tag::List(None) => "<ul>".to_string(),
        Tag::Item => "<li>".to_string(),
        Tag::FootnoteDefinition(name) => {
            format!("<div class=\"footnote\" id=\"fn-{name}\">")
        }
        Tag::Table(alignments) => {
            let _ = alignments; // Alignments handled per cell
            "<table>".to_string()
        }
        Tag::TableHead => "<thead><tr>".to_string(),
        Tag::TableRow => "<tr>".to_string(),
        Tag::TableCell => "<td>".to_string(),
        Tag::Emphasis => "<em>".to_string(),
        Tag::Strong => "<strong>".to_string(),
        Tag::Strikethrough => "<del>".to_string(),
        Tag::Link {
            dest_url, title, ..
        } => {
            let title_attr = if title.is_empty() {
                String::new()
            } else {
                format!(" title=\"{}\"", html_escape(title))
            };
            format!("<a href=\"{}\"{}> ", html_escape(dest_url), title_attr)
        }
        Tag::Image {
            dest_url, title, ..
        } => {
            let title_attr = if title.is_empty() {
                String::new()
            } else {
                format!(" title=\"{}\"", html_escape(title))
            };
            format!("<img src=\"{}\"{}", html_escape(dest_url), title_attr)
        }
        Tag::HtmlBlock => String::new(),
        Tag::MetadataBlock(_) => String::new(),
        Tag::DefinitionList => "<dl>".to_string(),
        Tag::DefinitionListTitle => "<dt>".to_string(),
        Tag::DefinitionListDefinition => "<dd>".to_string(),
        Tag::Superscript => "<sup>".to_string(),
        Tag::Subscript => "<sub>".to_string(),
    }
}

/// Convert a pulldown-cmark tag end to HTML closing tag.
fn tag_to_html_end(tag: &TagEnd) -> String {
    match tag {
        TagEnd::Paragraph => "</p>\n".to_string(),
        TagEnd::Heading(level) => format!("</h{}>\n", *level as u8),
        TagEnd::BlockQuote(_) => "</blockquote>\n".to_string(),
        TagEnd::CodeBlock => String::new(), // Handled separately
        TagEnd::List(ordered) => {
            if *ordered {
                "</ol>\n".to_string()
            } else {
                "</ul>\n".to_string()
            }
        }
        TagEnd::Item => "</li>\n".to_string(),
        TagEnd::FootnoteDefinition => "</div>\n".to_string(),
        TagEnd::Table => "</table>\n".to_string(),
        TagEnd::TableHead => "</tr></thead>\n".to_string(),
        TagEnd::TableRow => "</tr>\n".to_string(),
        TagEnd::TableCell => "</td>".to_string(),
        TagEnd::Emphasis => "</em>".to_string(),
        TagEnd::Strong => "</strong>".to_string(),
        TagEnd::Strikethrough => "</del>".to_string(),
        TagEnd::Link => "</a>".to_string(),
        TagEnd::Image => " />".to_string(),
        TagEnd::HtmlBlock => String::new(),
        TagEnd::MetadataBlock(_) => String::new(),
        TagEnd::DefinitionList => "</dl>\n".to_string(),
        TagEnd::DefinitionListTitle => "</dt>\n".to_string(),
        TagEnd::DefinitionListDefinition => "</dd>\n".to_string(),
        TagEnd::Superscript => "</sup>".to_string(),
        TagEnd::Subscript => "</sub>".to_string(),
    }
}

/// Escape HTML special characters.
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_markdown() {
        let parser = MarkdownParser::new();
        let content = r#"---
title: "Test Post"
---

# Hello World

This is a test."#;

        let result = parser.parse(content, Path::new("test.md")).unwrap();

        assert_eq!(result.frontmatter.title, "Test Post");
        assert!(result.html.contains("<h1"));
        assert!(result.html.contains("Hello World"));
        assert!(result.html.contains("<p>"));
    }

    #[test]
    fn test_parse_code_block() {
        let parser = MarkdownParser::new();
        let (html, _) = parser.parse_body(
            r#"```rust
fn main() {
    println!("Hello");
}
```"#,
        );

        assert!(html.contains("fn"));
        assert!(html.contains("main"));
    }

    #[test]
    fn test_toc_extraction() {
        let parser = MarkdownParser::new();
        let (_, toc) = parser.parse_body(
            r#"# Heading 1
## Heading 2
### Heading 3"#,
        );

        assert_eq!(toc.len(), 3);
        assert_eq!(toc[0].level, 1);
        assert_eq!(toc[0].text, "Heading 1");
        assert_eq!(toc[1].level, 2);
        assert_eq!(toc[2].level, 3);
    }

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Hello World"), "hello-world");
        assert_eq!(slugify("Test 123 Post"), "test-123-post");
        assert_eq!(slugify("Multiple   Spaces"), "multiple-spaces");
        assert_eq!(slugify("Special!@#Chars"), "specialchars");
    }

    #[test]
    fn test_table_rendering() {
        let parser = MarkdownParser::new();
        let (html, _) = parser.parse_body(
            r#"| Header 1 | Header 2 |
|----------|----------|
| Cell 1   | Cell 2   |"#,
        );

        assert!(html.contains("<table>"));
        assert!(html.contains("<thead>"));
        assert!(html.contains("<tr>"));
        assert!(html.contains("<td>"));
    }

    #[test]
    fn test_task_list() {
        let parser = MarkdownParser::new();
        let (html, _) = parser.parse_body(
            r#"- [x] Done
- [ ] Not done"#,
        );

        assert!(html.contains("checkbox"));
        assert!(html.contains("checked"));
    }

    #[test]
    fn test_no_frontmatter() {
        let parser = MarkdownParser::new();
        let content = "# Just Content\n\nNo frontmatter here.";
        let result = parser.parse(content, Path::new("test.md")).unwrap();

        assert!(result.frontmatter.title.is_empty());
        assert!(result.html.contains("Just Content"));
    }
}
