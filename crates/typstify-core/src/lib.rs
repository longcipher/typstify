//! Typstify Core Library
//!
//! Core types, configuration, and error handling for the Typstify static site generator.

pub mod config;
pub mod content;
pub mod error;
pub mod frontmatter;
pub mod utils;

pub use config::Config;
pub use content::{ContentPath, ContentType, Page, ParsedContent};
pub use error::{CoreError, Result};
pub use frontmatter::Frontmatter;

/// Shared test fixtures for use across workspace test modules.
///
/// Enable the `testing` feature on `typstify-core` in `[dev-dependencies]` to access:
///
/// ```toml
/// [dev-dependencies]
/// typstify-core = { workspace = true, features = ["testing"] }
/// ```
#[cfg(feature = "testing")]
pub mod test_fixtures {
    use std::{collections::HashMap, path::PathBuf};

    use crate::{
        Page,
        config::{
            BuildConfig, Config, RobotsConfig, RssConfig, SearchConfig, SiteConfig, TaxonomyConfig,
        },
    };

    /// Create a standard test configuration.
    pub fn test_config() -> Config {
        Config::from_parts(
            SiteConfig {
                title: "Test Site".to_string(),
                host: "https://example.com".to_string(),
                base_path: String::new(),
                default_language: "en".to_string(),
                description: Some("A test site".to_string()),
                author: Some("Test Author".to_string()),
            },
            BuildConfig::default(),
            SearchConfig::default(),
            RssConfig::default(),
            RobotsConfig::default(),
            TaxonomyConfig::default(),
            HashMap::new(),
        )
    }

    /// Create a standard test page.
    pub fn test_page() -> Page {
        Page {
            url: "/test-page".to_string(),
            title: "Test Page".to_string(),
            description: Some("A test page".to_string()),
            date: None,
            updated: None,
            draft: false,
            lang: "en".to_string(),
            is_default_lang: true,
            canonical_id: "test-page".to_string(),
            tags: vec![],
            categories: vec![],
            content: "<p>Hello, World!</p>".to_string(),
            summary: None,
            reading_time: None,
            word_count: None,
            toc: vec![],
            custom_js: vec![],
            custom_css: vec![],
            aliases: vec![],
            template: None,
            weight: 0,
            source_path: Some(PathBuf::from("test-page.md")),
        }
    }
}
