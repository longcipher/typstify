//! Sitemap generation.
//!
//! Generates XML sitemaps for search engine optimization.

use std::io::Write;

use chrono::{DateTime, Utc};
use thiserror::Error;
use tracing::debug;
use typstify_core::{Config, Page};

/// Sitemap generation errors.
#[derive(Debug, Error)]
pub enum SitemapError {
    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// XML encoding error.
    #[error("XML encoding error: {0}")]
    Xml(String),
}

/// Result type for sitemap operations.
pub type Result<T> = std::result::Result<T, SitemapError>;

/// Change frequency for sitemap entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeFreq {
    Always,
    Hourly,
    Daily,
    Weekly,
    Monthly,
    Yearly,
    Never,
}

impl ChangeFreq {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Always => "always",
            Self::Hourly => "hourly",
            Self::Daily => "daily",
            Self::Weekly => "weekly",
            Self::Monthly => "monthly",
            Self::Yearly => "yearly",
            Self::Never => "never",
        }
    }
}

/// A sitemap URL entry.
#[derive(Debug, Clone)]
pub struct SitemapUrl {
    /// URL location.
    pub loc: String,

    /// Last modification date.
    pub lastmod: Option<DateTime<Utc>>,

    /// Change frequency.
    pub changefreq: Option<ChangeFreq>,

    /// Priority (0.0 to 1.0).
    pub priority: Option<f32>,

    /// Alternate language versions.
    pub alternates: Vec<AlternateLink>,
}

/// Alternate language link for a URL.
#[derive(Debug, Clone)]
pub struct AlternateLink {
    /// Language code (e.g., "en", "zh").
    pub hreflang: String,

    /// URL for this language version.
    pub href: String,
}

/// Sitemap generator.
#[derive(Debug)]
pub struct SitemapGenerator {
    config: Config,
}

impl SitemapGenerator {
    /// Create a new sitemap generator.
    #[must_use]
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Generate sitemap XML from pages.
    pub fn generate(&self, pages: &[&Page]) -> Result<String> {
        debug!(count = pages.len(), "generating sitemap");

        let mut xml = String::from(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
        xml.push('\n');
        xml.push_str(r#"<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9""#);

        // Add xhtml namespace if we have multiple languages
        if self.config.site.languages.len() > 1 {
            xml.push_str(r#" xmlns:xhtml="http://www.w3.org/1999/xhtml""#);
        }
        xml.push_str(">\n");

        for page in pages {
            let url = self.page_to_url(page);
            xml.push_str(&self.url_to_xml(&url));
        }

        xml.push_str("</urlset>\n");

        Ok(xml)
    }

    /// Convert a page to a sitemap URL entry.
    fn page_to_url(&self, page: &Page) -> SitemapUrl {
        let loc = format!("{}{}", self.config.site.base_url, page.url);

        // Determine lastmod from page date or updated date
        let lastmod = page.updated.or(page.date);

        // Determine change frequency and priority based on content type
        let (changefreq, priority) = if page.url == "/" || page.url.is_empty() {
            // Home page
            (Some(ChangeFreq::Daily), Some(1.0))
        } else if page.date.is_some() {
            // Blog posts
            (Some(ChangeFreq::Monthly), Some(0.8))
        } else {
            // Static pages
            (Some(ChangeFreq::Yearly), Some(0.5))
        };

        // Build alternate links for multi-language sites
        let slug = page.url.trim_start_matches('/');
        let alternates = if self.config.site.languages.len() > 1 {
            self.config
                .site
                .languages
                .iter()
                .map(|lang| {
                    let href = if lang == &self.config.site.default_language {
                        format!("{}/{}", self.config.site.base_url, slug)
                    } else {
                        format!("{}/{}/{}", self.config.site.base_url, lang, slug)
                    };
                    AlternateLink {
                        hreflang: lang.clone(),
                        href,
                    }
                })
                .collect()
        } else {
            Vec::new()
        };

        SitemapUrl {
            loc,
            lastmod,
            changefreq,
            priority,
            alternates,
        }
    }

    /// Convert a URL entry to XML.
    fn url_to_xml(&self, url: &SitemapUrl) -> String {
        let mut xml = String::from("  <url>\n");

        xml.push_str(&format!("    <loc>{}</loc>\n", escape_xml(&url.loc)));

        if let Some(lastmod) = &url.lastmod {
            xml.push_str(&format!(
                "    <lastmod>{}</lastmod>\n",
                lastmod.format("%Y-%m-%d")
            ));
        }

        if let Some(changefreq) = &url.changefreq {
            xml.push_str(&format!(
                "    <changefreq>{}</changefreq>\n",
                changefreq.as_str()
            ));
        }

        if let Some(priority) = &url.priority {
            xml.push_str(&format!("    <priority>{priority:.1}</priority>\n"));
        }

        // Add alternate language links
        for alt in &url.alternates {
            xml.push_str(&format!(
                r#"    <xhtml:link rel="alternate" hreflang="{}" href="{}" />"#,
                alt.hreflang,
                escape_xml(&alt.href)
            ));
            xml.push('\n');
        }

        xml.push_str("  </url>\n");
        xml
    }

    /// Write sitemap to a writer.
    pub fn write_to<W: Write>(&self, pages: &[&Page], writer: &mut W) -> Result<()> {
        let xml = self.generate(pages)?;
        writer.write_all(xml.as_bytes())?;
        Ok(())
    }

    /// Generate sitemap index for multiple sitemaps.
    pub fn generate_index(&self, sitemaps: &[&str]) -> String {
        let mut xml = String::from(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
        xml.push('\n');
        xml.push_str(r#"<sitemapindex xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">"#);
        xml.push('\n');

        let now = Utc::now().format("%Y-%m-%d").to_string();

        for sitemap in sitemaps {
            xml.push_str("  <sitemap>\n");
            xml.push_str(&format!(
                "    <loc>{}/{}</loc>\n",
                self.config.site.base_url, sitemap
            ));
            xml.push_str(&format!("    <lastmod>{now}</lastmod>\n"));
            xml.push_str("  </sitemap>\n");
        }

        xml.push_str("</sitemapindex>\n");
        xml
    }
}

/// Escape special XML characters.
fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn test_config() -> Config {
        Config {
            site: typstify_core::config::SiteConfig {
                title: "Test Site".to_string(),
                base_url: "https://example.com".to_string(),
                default_language: "en".to_string(),
                languages: vec!["en".to_string()],
                description: None,
                author: None,
            },
            build: typstify_core::config::BuildConfig::default(),
            search: typstify_core::config::SearchConfig::default(),
            rss: typstify_core::config::RssConfig::default(),
            taxonomies: typstify_core::config::TaxonomyConfig::default(),
        }
    }

    fn test_page(slug: &str, date: Option<DateTime<Utc>>) -> Page {
        Page {
            url: format!("/{}", slug),
            title: slug.to_string(),
            description: None,
            date,
            updated: None,
            draft: false,
            lang: None,
            tags: vec![],
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
    fn test_generate_sitemap() {
        let generator = SitemapGenerator::new(test_config());
        let page1 = test_page("about", None);
        let page2 = test_page("blog/post-1", Some(Utc::now()));
        let pages: Vec<&Page> = vec![&page1, &page2];

        let xml = generator.generate(&pages).unwrap();

        assert!(xml.contains(r#"<?xml version="1.0""#));
        assert!(xml.contains("<urlset"));
        assert!(xml.contains("<loc>https://example.com/about</loc>"));
        assert!(xml.contains("<loc>https://example.com/blog/post-1</loc>"));
        assert!(xml.contains("<changefreq>"));
        assert!(xml.contains("<priority>"));
    }

    #[test]
    fn test_escape_xml() {
        assert_eq!(escape_xml("a & b"), "a &amp; b");
        assert_eq!(escape_xml("<tag>"), "&lt;tag&gt;");
        assert_eq!(escape_xml("\"quoted\""), "&quot;quoted&quot;");
    }

    #[test]
    fn test_home_page_priority() {
        let generator = SitemapGenerator::new(test_config());
        let mut home = test_page("", None);
        home.url = "/".to_string();

        let url = generator.page_to_url(&home);

        assert_eq!(url.priority, Some(1.0));
        assert_eq!(url.changefreq, Some(ChangeFreq::Daily));
    }

    #[test]
    fn test_generate_index() {
        let generator = SitemapGenerator::new(test_config());
        let sitemaps = vec!["sitemap-posts.xml", "sitemap-pages.xml"];

        let xml = generator.generate_index(&sitemaps);

        assert!(xml.contains("<sitemapindex"));
        assert!(xml.contains("sitemap-posts.xml"));
        assert!(xml.contains("sitemap-pages.xml"));
    }

    #[test]
    fn test_multilang_sitemap() {
        let mut config = test_config();
        config.site.languages = vec!["en".to_string(), "zh".to_string()];
        let generator = SitemapGenerator::new(config);

        let page = test_page("about", None);
        let pages: Vec<&Page> = vec![&page];

        let xml = generator.generate(&pages).unwrap();

        assert!(xml.contains("xmlns:xhtml"));
        assert!(xml.contains(r#"hreflang="en""#));
        assert!(xml.contains(r#"hreflang="zh""#));
    }
}
