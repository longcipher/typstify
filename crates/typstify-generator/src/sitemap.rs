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
        // Add XSLT stylesheet reference for browser rendering
        xml.push_str(r#"<?xml-stylesheet type="text/xsl" href="/sitemap-style.xsl"?>"#);
        xml.push('\n');
        xml.push_str(r#"<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9""#);

        // Add xhtml namespace if we have multiple languages
        let all_languages = self.config.all_languages();
        if all_languages.len() > 1 {
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
        let loc = format!("{}{}", self.config.base_url(), page.url);

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
        let all_languages = self.config.all_languages();
        let alternates = if all_languages.len() > 1 {
            all_languages
                .iter()
                .map(|lang| {
                    let href = if *lang == self.config.site.default_language {
                        format!("{}/{}", self.config.base_url(), slug)
                    } else {
                        format!("{}/{}/{}", self.config.base_url(), lang, slug)
                    };
                    AlternateLink {
                        hreflang: lang.to_string(),
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
                self.config.base_url(),
                sitemap
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

/// Generate XSLT stylesheet for sitemap rendering in browsers.
///
/// This creates a modern, clean stylesheet with light/dark mode support
/// that renders the sitemap as an HTML table.
#[must_use]
pub fn generate_sitemap_xsl() -> String {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<xsl:stylesheet version="2.0"
    xmlns:xsl="http://www.w3.org/1999/XSL/Transform"
    xmlns:sitemap="http://www.sitemaps.org/schemas/sitemap/0.9"
    xmlns:xhtml="http://www.w3.org/1999/xhtml">

<xsl:output method="html" version="1.0" encoding="UTF-8" indent="yes"/>

<xsl:template match="/">
<html lang="en">
<head>
    <meta charset="UTF-8"/>
    <meta name="viewport" content="width=device-width, initial-scale=1.0"/>
    <title>Sitemap</title>
    <style>
        :root {
            --bg-primary: #ffffff;
            --bg-secondary: #f8fafc;
            --bg-tertiary: #f1f5f9;
            --text-primary: #0f172a;
            --text-secondary: #475569;
            --text-muted: #94a3b8;
            --border-color: #e2e8f0;
            --accent-color: #3b82f6;
            --accent-hover: #2563eb;
            --priority-high: #22c55e;
            --priority-medium: #eab308;
            --priority-low: #94a3b8;
        }

        @media (prefers-color-scheme: dark) {
            :root {
                --bg-primary: #0f172a;
                --bg-secondary: #1e293b;
                --bg-tertiary: #334155;
                --text-primary: #f1f5f9;
                --text-secondary: #cbd5e1;
                --text-muted: #64748b;
                --border-color: #334155;
                --accent-color: #60a5fa;
                --accent-hover: #93c5fd;
            }
        }

        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }

        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
            background-color: var(--bg-primary);
            color: var(--text-primary);
            line-height: 1.6;
            padding: 2rem;
        }

        .container {
            max-width: 1200px;
            margin: 0 auto;
        }

        header {
            margin-bottom: 2rem;
            padding-bottom: 1rem;
            border-bottom: 1px solid var(--border-color);
        }

        h1 {
            font-size: 1.875rem;
            font-weight: 700;
            margin-bottom: 0.5rem;
        }

        .subtitle {
            color: var(--text-secondary);
            font-size: 0.875rem;
        }

        .stats {
            display: flex;
            gap: 2rem;
            margin-top: 1rem;
            flex-wrap: wrap;
        }

        .stat {
            background: var(--bg-secondary);
            padding: 0.75rem 1.25rem;
            border-radius: 0.5rem;
            border: 1px solid var(--border-color);
        }

        .stat-label {
            font-size: 0.75rem;
            text-transform: uppercase;
            letter-spacing: 0.05em;
            color: var(--text-muted);
        }

        .stat-value {
            font-size: 1.25rem;
            font-weight: 600;
            color: var(--accent-color);
        }

        table {
            width: 100%;
            border-collapse: collapse;
            margin-top: 1.5rem;
            background: var(--bg-secondary);
            border-radius: 0.5rem;
            overflow: hidden;
            border: 1px solid var(--border-color);
        }

        thead {
            background: var(--bg-tertiary);
        }

        th {
            padding: 0.875rem 1rem;
            text-align: left;
            font-weight: 600;
            font-size: 0.75rem;
            text-transform: uppercase;
            letter-spacing: 0.05em;
            color: var(--text-secondary);
            border-bottom: 1px solid var(--border-color);
        }

        td {
            padding: 0.875rem 1rem;
            border-bottom: 1px solid var(--border-color);
            font-size: 0.875rem;
        }

        tbody tr:hover {
            background: var(--bg-tertiary);
        }

        tbody tr:last-child td {
            border-bottom: none;
        }

        a {
            color: var(--accent-color);
            text-decoration: none;
            word-break: break-all;
        }

        a:hover {
            color: var(--accent-hover);
            text-decoration: underline;
        }

        .priority {
            display: inline-flex;
            align-items: center;
            gap: 0.375rem;
        }

        .priority-dot {
            width: 0.5rem;
            height: 0.5rem;
            border-radius: 50%;
        }

        .priority-high .priority-dot {
            background: var(--priority-high);
        }

        .priority-medium .priority-dot {
            background: var(--priority-medium);
        }

        .priority-low .priority-dot {
            background: var(--priority-low);
        }

        .changefreq {
            display: inline-block;
            padding: 0.25rem 0.5rem;
            background: var(--bg-tertiary);
            border-radius: 0.25rem;
            font-size: 0.75rem;
            color: var(--text-secondary);
        }

        .date {
            color: var(--text-muted);
            font-size: 0.8125rem;
        }

        footer {
            margin-top: 2rem;
            padding-top: 1rem;
            border-top: 1px solid var(--border-color);
            text-align: center;
            color: var(--text-muted);
            font-size: 0.75rem;
        }

        @media (max-width: 768px) {
            body {
                padding: 1rem;
            }

            .stats {
                gap: 1rem;
            }

            th, td {
                padding: 0.625rem 0.5rem;
            }

            .hide-mobile {
                display: none;
            }
        }
    </style>
</head>
<body>
    <div class="container">
        <header>
            <h1>🗺️ Sitemap</h1>
            <p class="subtitle">This sitemap contains all pages available on this website.</p>
            <div class="stats">
                <div class="stat">
                    <div class="stat-label">Total URLs</div>
                    <div class="stat-value"><xsl:value-of select="count(sitemap:urlset/sitemap:url)"/></div>
                </div>
            </div>
        </header>

        <table>
            <thead>
                <tr>
                    <th>URL</th>
                    <th class="hide-mobile">Priority</th>
                    <th class="hide-mobile">Change Frequency</th>
                    <th class="hide-mobile">Last Modified</th>
                </tr>
            </thead>
            <tbody>
                <xsl:for-each select="sitemap:urlset/sitemap:url">
                    <xsl:sort select="sitemap:priority" order="descending"/>
                    <tr>
                        <td>
                            <a href="{sitemap:loc}"><xsl:value-of select="sitemap:loc"/></a>
                        </td>
                        <td class="hide-mobile">
                            <xsl:choose>
                                <xsl:when test="sitemap:priority &gt;= 0.8">
                                    <span class="priority priority-high">
                                        <span class="priority-dot"></span>
                                        <xsl:value-of select="sitemap:priority"/>
                                    </span>
                                </xsl:when>
                                <xsl:when test="sitemap:priority &gt;= 0.5">
                                    <span class="priority priority-medium">
                                        <span class="priority-dot"></span>
                                        <xsl:value-of select="sitemap:priority"/>
                                    </span>
                                </xsl:when>
                                <xsl:otherwise>
                                    <span class="priority priority-low">
                                        <span class="priority-dot"></span>
                                        <xsl:value-of select="sitemap:priority"/>
                                    </span>
                                </xsl:otherwise>
                            </xsl:choose>
                        </td>
                        <td class="hide-mobile">
                            <xsl:if test="sitemap:changefreq">
                                <span class="changefreq"><xsl:value-of select="sitemap:changefreq"/></span>
                            </xsl:if>
                        </td>
                        <td class="hide-mobile">
                            <xsl:if test="sitemap:lastmod">
                                <span class="date"><xsl:value-of select="sitemap:lastmod"/></span>
                            </xsl:if>
                        </td>
                    </tr>
                </xsl:for-each>
            </tbody>
        </table>

        <footer>
            <p>Generated by Typstify • XML Sitemap Protocol</p>
        </footer>
    </div>
</body>
</html>
</xsl:template>

</xsl:stylesheet>"#.to_string()
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, path::PathBuf};

    use typstify_core::config::LanguageConfig;

    use super::*;

    fn test_config() -> Config {
        Config {
            site: typstify_core::config::SiteConfig {
                title: "Test Site".to_string(),
                host: "https://example.com".to_string(),
                base_path: String::new(),
                default_language: "en".to_string(),
                description: None,
                author: None,
            },
            languages: HashMap::new(),
            build: typstify_core::config::BuildConfig::default(),
            search: typstify_core::config::SearchConfig::default(),
            rss: typstify_core::config::RssConfig::default(),
            robots: typstify_core::config::RobotsConfig::default(),
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
            lang: "en".to_string(),
            is_default_lang: true,
            canonical_id: slug.to_string(),
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
        config.languages.insert(
            "en".to_string(),
            LanguageConfig {
                name: Some("English".to_string()),
                title: None,
                description: None,
            },
        );
        config.languages.insert(
            "zh".to_string(),
            LanguageConfig {
                name: Some("中文".to_string()),
                title: None,
                description: None,
            },
        );
        let generator = SitemapGenerator::new(config);

        let page = test_page("about", None);
        let pages: Vec<&Page> = vec![&page];

        let xml = generator.generate(&pages).unwrap();

        assert!(xml.contains("xmlns:xhtml"));
        assert!(xml.contains(r#"hreflang="en""#));
        assert!(xml.contains(r#"hreflang="zh""#));
    }
}
