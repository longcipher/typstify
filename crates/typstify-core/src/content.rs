//! Content types and structures.

use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::frontmatter::Frontmatter;

/// Type of content source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ContentType {
    /// Markdown content (.md files).
    Markdown,
    /// Typst content (.typ files).
    Typst,
}

impl ContentType {
    /// Determine content type from file extension.
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "md" | "markdown" => Some(Self::Markdown),
            "typ" | "typst" => Some(Self::Typst),
            _ => None,
        }
    }

    /// Get the file extension for this content type.
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Markdown => "md",
            Self::Typst => "typ",
        }
    }
}

/// Parsed content path with language and slug extraction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentPath {
    /// Original file path.
    pub path: PathBuf,

    /// Extracted language code (if any).
    pub lang: Option<String>,

    /// URL slug derived from the path.
    pub slug: String,

    /// Content type based on extension.
    pub content_type: ContentType,
}

impl ContentPath {
    /// Parse a content path to extract language and slug.
    ///
    /// Supports patterns like:
    /// - `posts/hello.md` → slug: "posts/hello", lang: None
    /// - `posts/hello.zh.md` → slug: "posts/hello", lang: Some("zh")
    /// - `posts/hello/index.md` → slug: "posts/hello", lang: None
    /// - `posts/hello/index.zh.md` → slug: "posts/hello", lang: Some("zh")
    pub fn from_path(path: &Path, default_lang: &str) -> Option<Self> {
        let extension = path.extension()?.to_str()?;
        let content_type = ContentType::from_extension(extension)?;

        let stem = path.file_stem()?.to_str()?;

        // Check for language suffix in filename (e.g., "index.zh" or "hello.zh")
        let (base_stem, lang) = if let Some(dot_pos) = stem.rfind('.') {
            let potential_lang = &stem[dot_pos + 1..];
            // Check if it looks like a language code (2-3 chars, lowercase alpha)
            if potential_lang.len() >= 2
                && potential_lang.len() <= 3
                && potential_lang.chars().all(|c| c.is_ascii_lowercase())
            {
                let lang = if potential_lang == default_lang {
                    None // Don't set lang if it's the default
                } else {
                    Some(potential_lang.to_string())
                };
                (&stem[..dot_pos], lang)
            } else {
                (stem, None)
            }
        } else {
            (stem, None)
        };

        // Build the slug from the path
        let parent = path.parent().unwrap_or(Path::new(""));
        let slug = if base_stem == "index" {
            // For index files, use the parent directory as the slug
            parent.to_string_lossy().to_string()
        } else {
            // For regular files, combine parent and stem
            if parent.as_os_str().is_empty() {
                base_stem.to_string()
            } else {
                format!("{}/{}", parent.display(), base_stem)
            }
        };

        // Normalize slug: remove leading/trailing slashes
        let slug = slug.trim_matches('/').to_string();

        Some(Self {
            path: path.to_path_buf(),
            lang,
            slug,
            content_type,
        })
    }

    /// Get the URL path for this content.
    pub fn url_path(&self) -> String {
        if let Some(ref lang) = self.lang {
            format!("/{}/{}", lang, self.slug)
        } else {
            format!("/{}", self.slug)
        }
    }
}

/// Parsed content with metadata and rendered HTML.
#[derive(Debug, Clone)]
pub struct ParsedContent {
    /// Parsed frontmatter metadata.
    pub frontmatter: Frontmatter,

    /// Rendered HTML content.
    pub html: String,

    /// Raw source content (without frontmatter).
    pub raw: String,

    /// Table of contents extracted from headings.
    pub toc: Vec<TocEntry>,
}

/// Table of contents entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TocEntry {
    /// Heading level (1-6).
    pub level: u8,

    /// Heading text.
    pub text: String,

    /// Anchor ID for linking.
    pub id: String,
}

/// A fully processed page ready for rendering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page {
    /// URL path for this page.
    pub url: String,

    /// Page title.
    pub title: String,

    /// Page description/summary.
    #[serde(default)]
    pub description: Option<String>,

    /// Publication date.
    #[serde(default)]
    pub date: Option<DateTime<Utc>>,

    /// Last updated date.
    #[serde(default)]
    pub updated: Option<DateTime<Utc>>,

    /// Whether this is a draft.
    #[serde(default)]
    pub draft: bool,

    /// Language code.
    #[serde(default)]
    pub lang: Option<String>,

    /// Tags for this page.
    #[serde(default)]
    pub tags: Vec<String>,

    /// Categories for this page.
    #[serde(default)]
    pub categories: Vec<String>,

    /// Rendered HTML content.
    pub content: String,

    /// Summary/excerpt for listings.
    #[serde(default)]
    pub summary: Option<String>,

    /// Reading time in minutes.
    #[serde(default)]
    pub reading_time: Option<u32>,

    /// Word count.
    #[serde(default)]
    pub word_count: Option<u32>,

    /// Table of contents.
    #[serde(default)]
    pub toc: Vec<TocEntry>,

    /// Custom JavaScript files to include.
    #[serde(default)]
    pub custom_js: Vec<String>,

    /// Custom CSS files to include.
    #[serde(default)]
    pub custom_css: Vec<String>,

    /// URL aliases for redirects.
    #[serde(default)]
    pub aliases: Vec<String>,

    /// Template to use for rendering.
    #[serde(default)]
    pub template: Option<String>,

    /// Sort weight for ordering.
    #[serde(default)]
    pub weight: i32,

    /// Source file path.
    #[serde(default)]
    pub source_path: Option<PathBuf>,
}

impl Page {
    /// Create a new page from parsed content and content path.
    pub fn from_parsed(content: ParsedContent, content_path: &ContentPath) -> Self {
        let fm = &content.frontmatter;

        // Calculate word count and reading time
        let word_count = content.raw.split_whitespace().count() as u32;
        let reading_time = (word_count / 200).max(1); // Assume 200 WPM

        // Generate summary if not provided
        let summary = fm.description.clone().or_else(|| {
            // Take first paragraph or first 160 chars
            let plain_text = strip_html(&content.html);
            Some(truncate_at_word_boundary(&plain_text, 160))
        });

        Self {
            url: content_path.url_path(),
            title: fm.title.clone(),
            description: fm.description.clone(),
            date: fm.date,
            updated: fm.updated,
            draft: fm.draft,
            lang: content_path.lang.clone(),
            tags: fm.tags.clone(),
            categories: fm.categories.clone(),
            content: content.html,
            summary,
            reading_time: Some(reading_time),
            word_count: Some(word_count),
            toc: content.toc,
            custom_js: fm.custom_js.clone(),
            custom_css: fm.custom_css.clone(),
            aliases: fm.aliases.clone(),
            template: fm.template.clone(),
            weight: fm.weight,
            source_path: Some(content_path.path.clone()),
        }
    }
}

/// Strip HTML tags from content.
fn strip_html(html: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;

    for c in html.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(c),
            _ => {}
        }
    }

    result
}

/// Truncate text at word boundary.
fn truncate_at_word_boundary(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        return text.to_string();
    }

    let truncated = &text[..max_len];
    if let Some(last_space) = truncated.rfind(' ') {
        format!("{}...", &truncated[..last_space])
    } else {
        format!("{truncated}...")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_type_from_extension() {
        assert_eq!(
            ContentType::from_extension("md"),
            Some(ContentType::Markdown)
        );
        assert_eq!(
            ContentType::from_extension("MD"),
            Some(ContentType::Markdown)
        );
        assert_eq!(ContentType::from_extension("typ"), Some(ContentType::Typst));
        assert_eq!(ContentType::from_extension("txt"), None);
    }

    #[test]
    fn test_content_path_simple() {
        let path = Path::new("posts/hello.md");
        let cp = ContentPath::from_path(path, "en").expect("parse path");

        assert_eq!(cp.slug, "posts/hello");
        assert_eq!(cp.lang, None);
        assert_eq!(cp.content_type, ContentType::Markdown);
        assert_eq!(cp.url_path(), "/posts/hello");
    }

    #[test]
    fn test_content_path_with_language() {
        let path = Path::new("posts/hello.zh.md");
        let cp = ContentPath::from_path(path, "en").expect("parse path");

        assert_eq!(cp.slug, "posts/hello");
        assert_eq!(cp.lang, Some("zh".to_string()));
        assert_eq!(cp.url_path(), "/zh/posts/hello");
    }

    #[test]
    fn test_content_path_default_language() {
        let path = Path::new("posts/hello.en.md");
        let cp = ContentPath::from_path(path, "en").expect("parse path");

        // Default language should not be set explicitly
        assert_eq!(cp.slug, "posts/hello");
        assert_eq!(cp.lang, None);
    }

    #[test]
    fn test_content_path_index_file() {
        let path = Path::new("posts/hello/index.md");
        let cp = ContentPath::from_path(path, "en").expect("parse path");

        assert_eq!(cp.slug, "posts/hello");
        assert_eq!(cp.lang, None);
    }

    #[test]
    fn test_content_path_index_with_lang() {
        let path = Path::new("posts/hello/index.zh.md");
        let cp = ContentPath::from_path(path, "en").expect("parse path");

        assert_eq!(cp.slug, "posts/hello");
        assert_eq!(cp.lang, Some("zh".to_string()));
    }

    #[test]
    fn test_content_path_typst() {
        let path = Path::new("docs/guide.typ");
        let cp = ContentPath::from_path(path, "en").expect("parse path");

        assert_eq!(cp.slug, "docs/guide");
        assert_eq!(cp.content_type, ContentType::Typst);
    }

    #[test]
    fn test_strip_html() {
        assert_eq!(
            strip_html("<p>Hello <strong>World</strong></p>"),
            "Hello World"
        );
        assert_eq!(strip_html("No tags here"), "No tags here");
    }

    #[test]
    fn test_truncate_at_word_boundary() {
        let text = "Hello world this is a test";
        assert_eq!(truncate_at_word_boundary(text, 100), text);
        // max_len=11 gives "Hello world", last space at pos 5, so "Hello..."
        assert_eq!(truncate_at_word_boundary(text, 11), "Hello...");
        assert_eq!(truncate_at_word_boundary(text, 5), "Hello...");
        // max_len=12 gives "Hello world ", last space at pos 11, so "Hello world..."
        assert_eq!(truncate_at_word_boundary(text, 12), "Hello world...");
    }
}
