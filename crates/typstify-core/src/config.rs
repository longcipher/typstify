//! Site configuration management.

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::{CoreError, Result};

/// Main configuration structure for Typstify.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Site-wide settings.
    pub site: SiteConfig,

    /// Build settings.
    #[serde(default)]
    pub build: BuildConfig,

    /// Search settings.
    #[serde(default)]
    pub search: SearchConfig,

    /// RSS feed settings.
    #[serde(default)]
    pub rss: RssConfig,

    /// Taxonomy settings.
    #[serde(default)]
    pub taxonomies: TaxonomyConfig,
}

/// Site-wide configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteConfig {
    /// Site title.
    pub title: String,

    /// Base URL for the site (e.g., "https://example.com").
    pub base_url: String,

    /// Default language code.
    #[serde(default = "default_language")]
    pub default_language: String,

    /// List of supported languages.
    #[serde(default = "default_languages")]
    pub languages: Vec<String>,

    /// Site description for meta tags.
    #[serde(default)]
    pub description: Option<String>,

    /// Site author name.
    #[serde(default)]
    pub author: Option<String>,
}

/// Build configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildConfig {
    /// Output directory for generated site.
    #[serde(default = "default_output_dir")]
    pub output_dir: String,

    /// Whether to minify HTML output.
    #[serde(default)]
    pub minify: bool,

    /// Syntax highlighting theme name.
    #[serde(default = "default_syntax_theme")]
    pub syntax_theme: String,

    /// Whether to generate drafts.
    #[serde(default)]
    pub drafts: bool,
}

/// Search configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConfig {
    /// Whether search is enabled.
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Fields to include in search index.
    #[serde(default = "default_index_fields")]
    pub index_fields: Vec<String>,

    /// Chunk size for index splitting (bytes).
    #[serde(default = "default_chunk_size")]
    pub chunk_size: usize,
}

/// RSS feed configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RssConfig {
    /// Whether RSS feed is enabled.
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Maximum number of items in feed.
    #[serde(default = "default_rss_limit")]
    pub limit: usize,
}

/// Taxonomy configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TaxonomyConfig {
    /// Tags taxonomy settings.
    #[serde(default)]
    pub tags: TaxonomySettings,

    /// Categories taxonomy settings.
    #[serde(default)]
    pub categories: TaxonomySettings,
}

/// Settings for a single taxonomy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxonomySettings {
    /// Number of items per page.
    #[serde(default = "default_paginate")]
    pub paginate: usize,
}

// Default value functions
fn default_language() -> String {
    "en".to_string()
}

fn default_languages() -> Vec<String> {
    vec!["en".to_string()]
}

fn default_output_dir() -> String {
    "public".to_string()
}

fn default_syntax_theme() -> String {
    "base16-ocean.dark".to_string()
}

fn default_true() -> bool {
    true
}

fn default_index_fields() -> Vec<String> {
    vec!["title".to_string(), "body".to_string(), "tags".to_string()]
}

fn default_chunk_size() -> usize {
    65536 // 64KB
}

fn default_rss_limit() -> usize {
    20
}

fn default_paginate() -> usize {
    10
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            output_dir: default_output_dir(),
            minify: false,
            syntax_theme: default_syntax_theme(),
            drafts: false,
        }
    }
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            index_fields: default_index_fields(),
            chunk_size: default_chunk_size(),
        }
    }
}

impl Default for RssConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            limit: default_rss_limit(),
        }
    }
}

impl Default for TaxonomySettings {
    fn default() -> Self {
        Self {
            paginate: default_paginate(),
        }
    }
}

impl Config {
    /// Load configuration from a TOML file.
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Err(CoreError::config(format!(
                "Configuration file not found: {}",
                path.display()
            )));
        }

        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content).map_err(|e| {
            CoreError::config_with_source(
                format!("Failed to parse config file: {}", path.display()),
                e,
            )
        })?;

        config.validate()?;
        Ok(config)
    }

    /// Load configuration using the config crate for more flexibility.
    pub fn load_with_env(path: &Path) -> Result<Self> {
        let settings = config::Config::builder()
            .add_source(config::File::from(path))
            .add_source(config::Environment::with_prefix("TYPSTIFY").separator("__"))
            .build()?;

        let config: Config = settings.try_deserialize()?;
        config.validate()?;
        Ok(config)
    }

    /// Validate the configuration.
    fn validate(&self) -> Result<()> {
        if self.site.title.is_empty() {
            return Err(CoreError::config("site.title cannot be empty"));
        }

        if self.site.base_url.is_empty() {
            return Err(CoreError::config("site.base_url cannot be empty"));
        }

        // Ensure base_url doesn't have trailing slash
        if self.site.base_url.ends_with('/') {
            tracing::warn!("site.base_url should not have a trailing slash");
        }

        Ok(())
    }

    /// Get the full URL for a path.
    pub fn url_for(&self, path: &str) -> String {
        let base = self.site.base_url.trim_end_matches('/');
        let path = path.trim_start_matches('/');
        format!("{base}/{path}")
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;

    fn create_test_config() -> String {
        r#"
[site]
title = "Test Site"
base_url = "https://example.com"
default_language = "en"
languages = ["en", "zh"]

[build]
output_dir = "dist"
minify = true
syntax_theme = "OneHalfDark"

[search]
enabled = true
chunk_size = 32768

[rss]
limit = 15

[taxonomies.tags]
paginate = 20
"#
        .to_string()
    }

    #[test]
    fn test_load_config() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let config_path = dir.path().join("config.toml");
        let mut file = std::fs::File::create(&config_path).expect("create file");
        file.write_all(create_test_config().as_bytes())
            .expect("write");

        let config = Config::load(&config_path).expect("load config");

        assert_eq!(config.site.title, "Test Site");
        assert_eq!(config.site.base_url, "https://example.com");
        assert_eq!(config.site.default_language, "en");
        assert_eq!(config.site.languages, vec!["en", "zh"]);
        assert_eq!(config.build.output_dir, "dist");
        assert!(config.build.minify);
        assert_eq!(config.build.syntax_theme, "OneHalfDark");
        assert!(config.search.enabled);
        assert_eq!(config.search.chunk_size, 32768);
        assert_eq!(config.rss.limit, 15);
        assert_eq!(config.taxonomies.tags.paginate, 20);
    }

    #[test]
    fn test_config_defaults() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let config_path = dir.path().join("config.toml");
        let minimal_config = r#"
[site]
title = "Minimal Site"
base_url = "https://example.com"
"#;
        std::fs::write(&config_path, minimal_config).expect("write");

        let config = Config::load(&config_path).expect("load config");

        assert_eq!(config.site.default_language, "en");
        assert_eq!(config.build.output_dir, "public");
        assert!(!config.build.minify);
        assert!(config.search.enabled);
        assert_eq!(config.search.chunk_size, 65536);
        assert_eq!(config.rss.limit, 20);
    }

    #[test]
    fn test_url_for() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let config_path = dir.path().join("config.toml");
        let config_content = r#"
[site]
title = "Test"
base_url = "https://example.com"
"#;
        std::fs::write(&config_path, config_content).expect("write");

        let config = Config::load(&config_path).expect("load config");

        assert_eq!(
            config.url_for("/posts/hello"),
            "https://example.com/posts/hello"
        );
        assert_eq!(
            config.url_for("posts/hello"),
            "https://example.com/posts/hello"
        );
    }

    #[test]
    fn test_config_validation_empty_title() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let config_path = dir.path().join("config.toml");
        let config_content = r#"
[site]
title = ""
base_url = "https://example.com"
"#;
        std::fs::write(&config_path, config_content).expect("write");

        let result = Config::load(&config_path);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("title cannot be empty")
        );
    }

    #[test]
    fn test_config_not_found() {
        let result = Config::load(Path::new("/nonexistent/config.toml"));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }
}
