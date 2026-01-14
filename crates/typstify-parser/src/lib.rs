//! Typstify Parser Library
//!
//! Content parsers for Markdown and Typst formats.

pub mod markdown;
pub mod syntax;
pub mod typst_parser;

use std::path::Path;

pub use markdown::MarkdownParser;
pub use syntax::SyntaxHighlighter;
use thiserror::Error;
pub use typst_parser::TypstParser;
use typstify_core::content::{ContentType, ParsedContent};

/// Parser errors.
#[derive(Debug, Error)]
pub enum ParserError {
    /// Markdown parsing error.
    #[error("markdown error: {0}")]
    Markdown(#[from] markdown::MarkdownError),

    /// Typst parsing error.
    #[error("typst error: {0}")]
    Typst(#[from] typst_parser::TypstError),

    /// Unsupported content type.
    #[error("unsupported content type: {0:?}")]
    UnsupportedType(ContentType),

    /// Unknown file extension.
    #[error("unknown file extension: {0}")]
    UnknownExtension(String),
}

/// Result type for parser operations.
pub type Result<T> = std::result::Result<T, ParserError>;

/// Trait for content parsers.
pub trait ContentParser {
    /// Parse content from a string and file path.
    fn parse(&self, content: &str, path: &Path) -> Result<ParsedContent>;
}

impl ContentParser for MarkdownParser {
    fn parse(&self, content: &str, path: &Path) -> Result<ParsedContent> {
        Ok(self.parse(content, path)?)
    }
}

impl ContentParser for TypstParser {
    fn parse(&self, content: &str, path: &Path) -> Result<ParsedContent> {
        Ok(self.parse(content, path)?)
    }
}

/// Registry for content parsers with auto-detection.
#[derive(Debug)]
pub struct ParserRegistry {
    markdown: MarkdownParser,
    typst: TypstParser,
}

impl Default for ParserRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ParserRegistry {
    /// Create a new parser registry with default parsers.
    pub fn new() -> Self {
        Self {
            markdown: MarkdownParser::new(),
            typst: TypstParser::new(),
        }
    }

    /// Create a parser registry with a custom syntax theme.
    pub fn with_theme(theme: &str) -> Self {
        Self {
            markdown: MarkdownParser::with_theme(theme),
            typst: TypstParser::new(),
        }
    }

    /// Parse content, auto-detecting the parser from file extension.
    pub fn parse(&self, content: &str, path: &Path) -> Result<ParsedContent> {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .ok_or_else(|| ParserError::UnknownExtension("(none)".to_string()))?;

        match ContentType::from_extension(ext) {
            Some(ContentType::Markdown) => Ok(self.markdown.parse(content, path)?),
            Some(ContentType::Typst) => Ok(self.typst.parse(content, path)?),
            None => Err(ParserError::UnknownExtension(ext.to_string())),
        }
    }

    /// Get the markdown parser.
    pub fn markdown(&self) -> &MarkdownParser {
        &self.markdown
    }

    /// Get the typst parser.
    pub fn typst(&self) -> &TypstParser {
        &self.typst
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_markdown() {
        let registry = ParserRegistry::new();
        let content = r#"---
title: "Test"
---

# Hello"#;

        let result = registry.parse(content, Path::new("test.md")).unwrap();
        assert_eq!(result.frontmatter.title, "Test");
    }

    #[test]
    fn test_registry_unknown_extension() {
        let registry = ParserRegistry::new();
        let result = registry.parse("content", Path::new("test.xyz"));

        assert!(matches!(result, Err(ParserError::UnknownExtension(_))));
    }

    #[test]
    fn test_content_parser_trait() {
        let parser = MarkdownParser::new();
        let content = r#"---
title: "Trait Test"
---

Content"#;

        let result: Result<ParsedContent> =
            ContentParser::parse(&parser, content, Path::new("test.md"));
        assert!(result.is_ok());
    }
}
