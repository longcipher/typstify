//! RSS feed generation.
//!
//! Generates RSS 2.0 feeds for site content.

use std::io::Write;

use chrono::Utc;
use rss::{ChannelBuilder, GuidBuilder, Item, ItemBuilder};
use thiserror::Error;
use tracing::debug;
use typstify_core::{Config, Page};

/// RSS generation errors.
#[derive(Debug, Error)]
pub enum RssError {
    /// RSS building error.
    #[error("RSS build error: {0}")]
    Build(String),

    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type for RSS operations.
pub type Result<T> = std::result::Result<T, RssError>;

/// RSS feed generator.
#[derive(Debug)]
pub struct RssGenerator {
    config: Config,
}

impl RssGenerator {
    /// Create a new RSS generator.
    #[must_use]
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Generate RSS feed XML from pages.
    pub fn generate(&self, pages: &[&Page]) -> Result<String> {
        let limit = self.config.rss.limit;
        let pages: Vec<_> = pages.iter().take(limit).collect();

        debug!(count = pages.len(), limit, "generating RSS feed");

        let items: Vec<Item> = pages
            .iter()
            .filter_map(|page| self.page_to_item(page))
            .collect();

        let channel = ChannelBuilder::default()
            .title(&self.config.site.title)
            .link(&self.config.site.base_url)
            .description(
                self.config
                    .site
                    .description
                    .as_deref()
                    .unwrap_or(&self.config.site.title),
            )
            .language(Some(self.config.site.default_language.clone()))
            .last_build_date(Some(Utc::now().to_rfc2822()))
            .items(items)
            .build();

        Ok(channel.to_string())
    }

    /// Generate RSS feed for a specific language.
    pub fn generate_for_lang(&self, pages: &[&Page], lang: &str) -> Result<String> {
        let filtered: Vec<_> = pages
            .iter()
            .filter(|p| p.lang.as_deref() == Some(lang) || p.lang.is_none())
            .copied()
            .collect();

        self.generate(&filtered)
    }

    /// Convert a page to an RSS item.
    fn page_to_item(&self, page: &Page) -> Option<Item> {
        let url = format!("{}{}", self.config.site.base_url, page.url);

        let guid = GuidBuilder::default().value(&url).permalink(true).build();

        let mut builder = ItemBuilder::default();
        builder.title(Some(page.title.clone()));
        builder.link(Some(url.clone()));
        builder.guid(Some(guid));

        // Add publication date
        if let Some(date) = page.date {
            builder.pub_date(Some(date.to_rfc2822()));
        }

        // Add description/summary
        if let Some(desc) = &page.description {
            builder.description(Some(desc.clone()));
        } else if let Some(summary) = &page.summary {
            builder.description(Some(summary.clone()));
        }

        // Add author
        if let Some(author) = &self.config.site.author {
            builder.author(Some(author.clone()));
        }

        // Add categories (tags)
        let categories: Vec<_> = page
            .tags
            .iter()
            .map(|tag| rss::Category {
                name: tag.clone(),
                domain: None,
            })
            .collect();

        if !categories.is_empty() {
            builder.categories(categories);
        }

        Some(builder.build())
    }

    /// Write RSS feed to a writer.
    pub fn write_to<W: Write>(&self, pages: &[&Page], writer: &mut W) -> Result<()> {
        let xml = self.generate(pages)?;
        writer.write_all(xml.as_bytes())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use chrono::{DateTime, Utc};

    use super::*;

    fn test_config() -> Config {
        Config {
            site: typstify_core::config::SiteConfig {
                title: "Test Blog".to_string(),
                base_url: "https://example.com".to_string(),
                default_language: "en".to_string(),
                languages: vec!["en".to_string()],
                description: Some("A test blog".to_string()),
                author: Some("Test Author".to_string()),
            },
            build: typstify_core::config::BuildConfig::default(),
            search: typstify_core::config::SearchConfig::default(),
            rss: typstify_core::config::RssConfig {
                enabled: true,
                limit: 20,
            },
            taxonomies: typstify_core::config::TaxonomyConfig::default(),
        }
    }

    fn test_page(title: &str, date: Option<DateTime<Utc>>) -> Page {
        Page {
            url: format!("/{}", title.to_lowercase().replace(' ', "-")),
            title: title.to_string(),
            description: Some(format!("Description for {}", title)),
            date,
            updated: None,
            draft: false,
            lang: None,
            tags: vec!["rust".to_string(), "web".to_string()],
            categories: vec![],
            content: String::new(),
            summary: None,
            reading_time: None,
            word_count: None,
            toc: vec![],
            custom_js: vec![],
            custom_css: vec![],
            aliases: vec![],
            template: None,
            weight: 0,
            source_path: Some(PathBuf::from("test.md")),
        }
    }

    #[test]
    fn test_generate_rss() {
        let generator = RssGenerator::new(test_config());
        let page1 = test_page("First Post", Some(Utc::now()));
        let page2 = test_page("Second Post", Some(Utc::now()));
        let pages: Vec<&Page> = vec![&page1, &page2];

        let xml = generator.generate(&pages).unwrap();

        assert!(xml.contains("<title>Test Blog</title>"));
        assert!(xml.contains("<link>https://example.com</link>"));
        assert!(xml.contains("First Post"));
        assert!(xml.contains("Second Post"));
        assert!(xml.contains("<category>rust</category>"));
    }

    #[test]
    fn test_rss_limit() {
        let mut config = test_config();
        config.rss.limit = 1;
        let generator = RssGenerator::new(config);

        let page1 = test_page("First Post", Some(Utc::now()));
        let page2 = test_page("Second Post", Some(Utc::now()));
        let pages: Vec<&Page> = vec![&page1, &page2];

        let xml = generator.generate(&pages).unwrap();

        assert!(xml.contains("First Post"));
        assert!(!xml.contains("Second Post"));
    }

    #[test]
    fn test_page_to_item() {
        let generator = RssGenerator::new(test_config());
        let page = test_page("Test Post", Some(Utc::now()));

        let item = generator.page_to_item(&page).unwrap();

        assert_eq!(item.title(), Some("Test Post"));
        assert!(item.link().is_some_and(|l| l.contains("/test-post")));
        assert!(item.pub_date().is_some());
    }
}
