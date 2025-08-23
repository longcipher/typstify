use eyre::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ContentMetadata {
    pub title: Option<String>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub date: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub category: Option<String>,
    #[serde(default)]
    pub draft: bool,
    pub slug: Option<String>,
    pub weight: Option<i32>,
    #[serde(default)]
    pub custom: HashMap<String, String>,
}

impl ContentMetadata {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_yaml(yaml: &str) -> Result<Self> {
        let metadata: ContentMetadata = serde_yaml::from_str(yaml)?;
        Ok(metadata)
    }

    pub fn from_toml(toml: &str) -> Result<Self> {
        let metadata: ContentMetadata = toml::from_str(toml)?;
        Ok(metadata)
    }

    pub fn extract_from_markdown(content: &str) -> Result<(Self, String)> {
        if let Some(stripped) = content.strip_prefix("+++") {
            // TOML frontmatter
            if let Some(end) = stripped.find("+++") {
                let frontmatter = &stripped[..end];
                let body = &stripped[end + 3..];
                let metadata = Self::from_toml(frontmatter)?;
                return Ok((metadata, body.trim().to_string()));
            }
        } else if let Some(stripped) = content.strip_prefix("---") {
            // YAML frontmatter
            if let Some(end) = stripped.find("---") {
                let frontmatter = &stripped[..end];
                let body = &stripped[end + 3..];
                let metadata = Self::from_yaml(frontmatter)?;
                return Ok((metadata, body.trim().to_string()));
            }
        }

        // No frontmatter found
        Ok((Self::default(), content.to_string()))
    }

    pub fn extract_from_typst(content: &str) -> Result<(Self, String)> {
        let mut metadata = Self::default();
        let mut body_lines = Vec::new();
        let mut in_metadata = false;
        let mut metadata_content = String::new();

        for line in content.lines() {
            if line.trim() == "#metadata[" {
                in_metadata = true;
                continue;
            } else if in_metadata && line.trim() == "]" {
                in_metadata = false;
                // Parse the metadata content as YAML
                if !metadata_content.trim().is_empty()
                    && let Ok(parsed) = Self::from_yaml(&metadata_content)
                {
                    metadata = parsed;
                }
                continue;
            } else if in_metadata {
                metadata_content.push_str(line);
                metadata_content.push('\n');
                continue;
            } else if line.trim().starts_with("// title:") {
                metadata.title = Some(
                    line.trim()
                        .strip_prefix("// title:")
                        .unwrap()
                        .trim()
                        .to_string(),
                );
                continue;
            } else if line.trim().starts_with("// description:") {
                metadata.description = Some(
                    line.trim()
                        .strip_prefix("// description:")
                        .unwrap()
                        .trim()
                        .to_string(),
                );
                continue;
            } else if line.trim().starts_with("// author:") {
                metadata.author = Some(
                    line.trim()
                        .strip_prefix("// author:")
                        .unwrap()
                        .trim()
                        .to_string(),
                );
                continue;
            } else if line.trim().starts_with("// date:") {
                metadata.date = Some(
                    line.trim()
                        .strip_prefix("// date:")
                        .unwrap()
                        .trim()
                        .to_string(),
                );
                continue;
            } else if line.trim().starts_with("// tags:") {
                let tags_str = line.trim().strip_prefix("// tags:").unwrap().trim();
                metadata.tags = tags_str.split(',').map(|t| t.trim().to_string()).collect();
                continue;
            } else if line.trim().starts_with("// category:") {
                metadata.category = Some(
                    line.trim()
                        .strip_prefix("// category:")
                        .unwrap()
                        .trim()
                        .to_string(),
                );
                continue;
            } else if line.trim().starts_with("// draft:") {
                let draft_str = line.trim().strip_prefix("// draft:").unwrap().trim();
                metadata.draft = draft_str.parse().unwrap_or(false);
                continue;
            }

            body_lines.push(line);
        }

        let body = body_lines.join("\n");
        Ok((metadata, body))
    }

    pub fn get_title(&self) -> String {
        self.title.clone().unwrap_or_else(|| "Untitled".to_string())
    }

    pub fn get_description(&self) -> String {
        self.description.clone().unwrap_or_default()
    }

    pub fn is_draft(&self) -> bool {
        self.draft
    }

    pub fn get_weight(&self) -> i32 {
        self.weight.unwrap_or(0)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.iter().any(|t| t == tag)
    }

    pub fn get_category(&self) -> Option<&str> {
        self.category.as_deref()
    }

    pub fn get_date(&self) -> Option<&str> {
        self.date.as_deref()
    }

    pub fn get_author(&self) -> Option<&str> {
        self.author.as_deref()
    }

    pub fn get_slug(&self) -> Option<&str> {
        self.slug.as_deref()
    }

    pub fn get_summary(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn get_tags(&self) -> Option<&[String]> {
        if self.tags.is_empty() {
            None
        } else {
            Some(&self.tags)
        }
    }

    pub fn get_custom_field(&self, key: &str) -> Option<&str> {
        self.custom.get(key).map(|s| s.as_str())
    }

    pub fn set_custom_field(&mut self, key: String, value: String) {
        self.custom.insert(key, value);
    }
}
