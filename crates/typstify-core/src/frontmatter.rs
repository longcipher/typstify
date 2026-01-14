//! Frontmatter parsing for content files.

use std::path::Path;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::{CoreError, Result};

/// Frontmatter metadata for content files.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Frontmatter {
    /// Page title (required).
    pub title: String,

    /// Publication date.
    #[serde(default)]
    pub date: Option<DateTime<Utc>>,

    /// Last updated date.
    #[serde(default)]
    pub updated: Option<DateTime<Utc>>,

    /// Whether this is a draft.
    #[serde(default)]
    pub draft: bool,

    /// Page description for meta tags and summaries.
    #[serde(default)]
    pub description: Option<String>,

    /// Tags for the page.
    #[serde(default)]
    pub tags: Vec<String>,

    /// Categories for the page.
    #[serde(default)]
    pub categories: Vec<String>,

    /// URL aliases for redirects.
    #[serde(default)]
    pub aliases: Vec<String>,

    /// Custom JavaScript files to include.
    #[serde(default)]
    pub custom_js: Vec<String>,

    /// Custom CSS files to include.
    #[serde(default)]
    pub custom_css: Vec<String>,

    /// Template to use for rendering.
    #[serde(default)]
    pub template: Option<String>,

    /// Sort weight for ordering.
    #[serde(default)]
    pub weight: i32,

    /// Custom extra fields (for extensibility).
    #[serde(default, flatten)]
    pub extra: std::collections::HashMap<String, serde_yaml::Value>,
}

/// Delimiter types for frontmatter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrontmatterFormat {
    /// YAML frontmatter delimited by `---`.
    Yaml,
    /// TOML frontmatter delimited by `+++`.
    Toml,
}

impl FrontmatterFormat {
    /// Get the delimiter string for this format.
    pub fn delimiter(&self) -> &'static str {
        match self {
            Self::Yaml => "---",
            Self::Toml => "+++",
        }
    }
}

/// Split content into frontmatter and body.
pub fn split_frontmatter(content: &str) -> Option<(FrontmatterFormat, &str, &str)> {
    let content = content.trim_start();

    // Detect format based on opening delimiter
    let format = if content.starts_with("---") {
        FrontmatterFormat::Yaml
    } else if content.starts_with("+++") {
        FrontmatterFormat::Toml
    } else {
        return None;
    };

    let delimiter = format.delimiter();

    // Find the closing delimiter
    let after_first = &content[delimiter.len()..];
    let closing_pos = after_first.find(delimiter)?;

    let frontmatter = after_first[..closing_pos].trim();
    let body = after_first[closing_pos + delimiter.len()..].trim_start();

    Some((format, frontmatter, body))
}

/// Parse frontmatter from a string.
pub fn parse_frontmatter(content: &str, path: &Path) -> Result<(Frontmatter, String)> {
    let Some((format, fm_str, body)) = split_frontmatter(content) else {
        // No frontmatter found, return default with full content
        return Ok((Frontmatter::default(), content.to_string()));
    };

    let frontmatter: Frontmatter = match format {
        FrontmatterFormat::Yaml => {
            serde_yaml::from_str(fm_str).map_err(|e| CoreError::frontmatter(path, e.to_string()))?
        }
        FrontmatterFormat::Toml => {
            toml::from_str(fm_str).map_err(|e| CoreError::frontmatter(path, e.to_string()))?
        }
    };

    Ok((frontmatter, body.to_string()))
}

/// Parse frontmatter from Typst-style comments.
///
/// Typst frontmatter uses comments at the start of the file:
/// ```typst
/// // typstify:frontmatter
/// // title: "My Document"
/// // date: 2024-01-14
/// // tags: [rust, typst]
/// ```
pub fn parse_typst_frontmatter(content: &str, path: &Path) -> Result<(Frontmatter, String)> {
    let mut fm_lines = Vec::new();
    let mut body_start = 0;
    let mut in_frontmatter = false;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed == "// typstify:frontmatter" {
            in_frontmatter = true;
            body_start += line.len() + 1; // +1 for newline
            continue;
        }

        if in_frontmatter {
            if let Some(stripped) = trimmed.strip_prefix("// ") {
                fm_lines.push(stripped);
                body_start += line.len() + 1;
            } else if trimmed.starts_with("//") && trimmed.len() == 2 {
                // Empty comment line
                body_start += line.len() + 1;
            } else {
                // End of frontmatter
                break;
            }
        } else {
            break;
        }
    }

    if fm_lines.is_empty() {
        return Ok((Frontmatter::default(), content.to_string()));
    }

    let fm_str = fm_lines.join("\n");
    let frontmatter: Frontmatter =
        serde_yaml::from_str(&fm_str).map_err(|e| CoreError::frontmatter(path, e.to_string()))?;

    let body = if body_start < content.len() {
        content[body_start..].trim_start().to_string()
    } else {
        String::new()
    };

    Ok((frontmatter, body))
}

impl Frontmatter {
    /// Validate required fields.
    pub fn validate(&self, path: &Path) -> Result<()> {
        if self.title.is_empty() {
            return Err(CoreError::frontmatter(path, "title is required"));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_yaml_frontmatter() {
        let content = r#"---
title: "Hello World"
date: 2024-01-14
---

This is the body content."#;

        let (format, fm, body) = split_frontmatter(content).expect("split");
        assert_eq!(format, FrontmatterFormat::Yaml);
        assert!(fm.contains("title:"));
        assert!(body.starts_with("This is the body"));
    }

    #[test]
    fn test_split_toml_frontmatter() {
        let content = r#"+++
title = "Hello World"
date = 2024-01-14
+++

This is the body content."#;

        let (format, fm, body) = split_frontmatter(content).expect("split");
        assert_eq!(format, FrontmatterFormat::Toml);
        assert!(fm.contains("title ="));
        assert!(body.starts_with("This is the body"));
    }

    #[test]
    fn test_no_frontmatter() {
        let content = "Just some content without frontmatter.";
        assert!(split_frontmatter(content).is_none());
    }

    #[test]
    fn test_parse_yaml_frontmatter() {
        let content = r#"---
title: "Test Post"
date: 2024-01-14T10:00:00Z
draft: false
tags:
  - rust
  - test
---

Content here."#;

        let (fm, body) = parse_frontmatter(content, Path::new("test.md")).expect("parse");

        assert_eq!(fm.title, "Test Post");
        assert!(fm.date.is_some());
        assert!(!fm.draft);
        assert_eq!(fm.tags, vec!["rust", "test"]);
        assert_eq!(body, "Content here.");
    }

    #[test]
    fn test_parse_toml_frontmatter() {
        let content = r#"+++
title = "Test Post"
draft = true
tags = ["rust", "test"]
+++

Content here."#;

        let (fm, body) = parse_frontmatter(content, Path::new("test.md")).expect("parse");

        assert_eq!(fm.title, "Test Post");
        assert!(fm.draft);
        assert_eq!(fm.tags, vec!["rust", "test"]);
        assert_eq!(body, "Content here.");
    }

    #[test]
    fn test_parse_typst_frontmatter() {
        let content = r#"// typstify:frontmatter
// title: "My Typst Document"
// date: "2024-01-14T00:00:00Z"
// tags: [typst, docs]

= Heading

Some typst content."#;

        let (fm, body) = parse_typst_frontmatter(content, Path::new("test.typ")).expect("parse");

        assert_eq!(fm.title, "My Typst Document");
        assert_eq!(fm.tags, vec!["typst", "docs"]);
        assert!(body.starts_with("= Heading"));
    }

    #[test]
    fn test_frontmatter_with_extra_fields() {
        let content = r#"---
title: "Test"
custom_field: "custom value"
---

Body"#;

        let (fm, _body) = parse_frontmatter(content, Path::new("test.md")).expect("parse");

        assert_eq!(fm.title, "Test");
        assert!(fm.extra.contains_key("custom_field"));
    }

    #[test]
    fn test_frontmatter_defaults() {
        let content = r#"---
title: "Minimal"
---

Body"#;

        let (fm, _body) = parse_frontmatter(content, Path::new("test.md")).expect("parse");

        assert_eq!(fm.title, "Minimal");
        assert!(!fm.draft);
        assert!(fm.tags.is_empty());
        assert!(fm.date.is_none());
    }

    #[test]
    fn test_validate_missing_title() {
        let fm = Frontmatter::default();
        let result = fm.validate(Path::new("test.md"));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("title"));
    }
}
