//! Build orchestration.
//!
//! Coordinates the full site build process.

use std::{
    fs,
    path::{Path, PathBuf},
    time::Instant,
};

use rayon::prelude::*;
use thiserror::Error;
use tracing::{debug, info, warn};
use typstify_core::Config;

use crate::{
    assets::{AssetError, AssetManifest, AssetProcessor},
    collector::{CollectorError, ContentCollector, SiteContent},
    html::{HtmlError, HtmlGenerator, list_item_html, pagination_html},
    rss::{RssError, RssGenerator},
    sitemap::{SitemapError, SitemapGenerator},
};

/// Build errors.
#[derive(Debug, Error)]
pub enum BuildError {
    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Collector error.
    #[error("collector error: {0}")]
    Collector(#[from] CollectorError),

    /// HTML generation error.
    #[error("HTML error: {0}")]
    Html(#[from] HtmlError),

    /// RSS generation error.
    #[error("RSS error: {0}")]
    Rss(#[from] RssError),

    /// Sitemap generation error.
    #[error("sitemap error: {0}")]
    Sitemap(#[from] SitemapError),

    /// Asset error.
    #[error("asset error: {0}")]
    Asset(#[from] AssetError),

    /// Configuration error.
    #[error("config error: {0}")]
    Config(String),
}

/// Result type for build operations.
pub type Result<T> = std::result::Result<T, BuildError>;

/// Build statistics.
#[derive(Debug, Clone, Default)]
pub struct BuildStats {
    /// Number of pages generated.
    pub pages: usize,

    /// Number of taxonomy pages generated.
    pub taxonomy_pages: usize,

    /// Number of redirect pages generated.
    pub redirects: usize,

    /// Number of assets processed.
    pub assets: usize,

    /// Build duration in milliseconds.
    pub duration_ms: u64,
}

/// Site builder that orchestrates the build process.
#[derive(Debug)]
pub struct Builder {
    config: Config,
    content_dir: PathBuf,
    output_dir: PathBuf,
    static_dir: Option<PathBuf>,
}

impl Builder {
    /// Create a new builder.
    #[must_use]
    pub fn new(
        config: Config,
        content_dir: impl Into<PathBuf>,
        output_dir: impl Into<PathBuf>,
    ) -> Self {
        Self {
            config,
            content_dir: content_dir.into(),
            output_dir: output_dir.into(),
            static_dir: None,
        }
    }

    /// Set the static assets directory.
    #[must_use]
    pub fn with_static_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.static_dir = Some(dir.into());
        self
    }

    /// Execute the full build process.
    pub fn build(&self) -> Result<BuildStats> {
        let start = Instant::now();
        let mut stats = BuildStats::default();

        info!(
            content = %self.content_dir.display(),
            output = %self.output_dir.display(),
            "starting build"
        );

        // 1. Clean output directory
        self.clean_output()?;

        // 2. Collect content
        let collector = ContentCollector::new(self.config.clone(), &self.content_dir);
        let content = collector.collect()?;

        // 3. Generate HTML pages
        stats.pages = self.generate_pages(&content)?;

        // 4. Generate taxonomy pages
        stats.taxonomy_pages = self.generate_taxonomy_pages(&content)?;

        // 5. Generate redirects
        stats.redirects = self.generate_redirects(&content)?;

        // 6. Generate RSS feed
        if self.config.rss.enabled {
            self.generate_rss(&content)?;
        }

        // 7. Generate sitemap
        self.generate_sitemap(&content)?;

        // 8. Process assets
        if let Some(ref static_dir) = self.static_dir {
            let manifest = self.process_assets(static_dir)?;
            stats.assets = manifest.assets().len();
        }

        stats.duration_ms = start.elapsed().as_millis() as u64;

        info!(
            pages = stats.pages,
            taxonomy_pages = stats.taxonomy_pages,
            redirects = stats.redirects,
            assets = stats.assets,
            duration_ms = stats.duration_ms,
            "build complete"
        );

        Ok(stats)
    }

    /// Clean the output directory.
    fn clean_output(&self) -> Result<()> {
        if self.output_dir.exists() {
            debug!(dir = %self.output_dir.display(), "cleaning output directory");
            fs::remove_dir_all(&self.output_dir)?;
        }
        fs::create_dir_all(&self.output_dir)?;
        Ok(())
    }

    /// Generate HTML pages for all content.
    fn generate_pages(&self, content: &SiteContent) -> Result<usize> {
        let generator = HtmlGenerator::new(self.config.clone());
        let pages: Vec<_> = content.pages.values().collect();

        info!(count = pages.len(), "generating HTML pages");

        // Generate pages in parallel
        let results: Vec<_> = pages
            .par_iter()
            .map(|page| {
                let html = generator.generate_page(page)?;
                let output_path = generator.output_path(page, &self.output_dir);

                // Write HTML file
                if let Some(parent) = output_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::write(&output_path, &html)?;

                debug!(path = %output_path.display(), "wrote page");
                Ok::<_, BuildError>(())
            })
            .collect();

        // Check for errors
        let mut count = 0;
        for result in results {
            match result {
                Ok(()) => count += 1,
                Err(e) => warn!(error = %e, "failed to generate page"),
            }
        }

        Ok(count)
    }

    /// Generate taxonomy (tag/category) pages.
    fn generate_taxonomy_pages(&self, content: &SiteContent) -> Result<usize> {
        let generator = HtmlGenerator::new(self.config.clone());
        let per_page = self.config.taxonomies.tags.paginate;
        let mut count = 0;

        // Generate tag pages
        for (tag, slugs) in &content.taxonomies.tags {
            let pages: Vec<_> = slugs.iter().filter_map(|s| content.pages.get(s)).collect();
            count += self
                .generate_taxonomy_term_pages(&generator, "Tags", tag, &pages, per_page, "tags")?;
        }

        // Generate category pages
        for (category, slugs) in &content.taxonomies.categories {
            let pages: Vec<_> = slugs.iter().filter_map(|s| content.pages.get(s)).collect();
            count += self.generate_taxonomy_term_pages(
                &generator,
                "Categories",
                category,
                &pages,
                per_page,
                "categories",
            )?;
        }

        Ok(count)
    }

    /// Generate paginated pages for a taxonomy term.
    fn generate_taxonomy_term_pages(
        &self,
        generator: &HtmlGenerator,
        taxonomy_name: &str,
        term: &str,
        pages: &[&typstify_core::Page],
        per_page: usize,
        url_prefix: &str,
    ) -> Result<usize> {
        use crate::collector::paginate;

        let term_slug = term.to_lowercase().replace(' ', "-");
        let base_url = format!("/{url_prefix}/{term_slug}");
        let total_pages = (pages.len() + per_page - 1).max(1) / per_page.max(1);
        let mut count = 0;

        for page_num in 1..=total_pages.max(1) {
            let (page_items, _) = paginate(pages, page_num, per_page);

            let items_html: String = page_items.iter().map(|p| list_item_html(p)).collect();

            let pagination = pagination_html(page_num, total_pages, &base_url);

            let html = generator.generate_taxonomy_page(
                taxonomy_name,
                term,
                &items_html,
                pagination.as_deref(),
            )?;

            // Determine output path
            let output_path = if page_num == 1 {
                self.output_dir
                    .join(url_prefix)
                    .join(&term_slug)
                    .join("index.html")
            } else {
                self.output_dir
                    .join(url_prefix)
                    .join(&term_slug)
                    .join("page")
                    .join(page_num.to_string())
                    .join("index.html")
            };

            if let Some(parent) = output_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&output_path, &html)?;
            count += 1;
        }

        Ok(count)
    }

    /// Generate redirect pages for URL aliases.
    fn generate_redirects(&self, content: &SiteContent) -> Result<usize> {
        let generator = HtmlGenerator::new(self.config.clone());
        let mut count = 0;

        for page in content.pages.values() {
            for alias in &page.aliases {
                let redirect_url = format!("{}{}", self.config.site.base_url, page.url);
                let html = generator.generate_redirect(&redirect_url)?;

                let alias_path = alias.trim_matches('/');
                let output_path = self.output_dir.join(alias_path).join("index.html");

                if let Some(parent) = output_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::write(&output_path, &html)?;
                count += 1;

                debug!(alias = alias, target = %page.url, "generated redirect");
            }
        }

        Ok(count)
    }

    /// Generate RSS feed.
    fn generate_rss(&self, content: &SiteContent) -> Result<()> {
        let generator = RssGenerator::new(self.config.clone());
        let pages = ContentCollector::pages_by_date(content);

        // Filter to only posts (pages with dates)
        let posts: Vec<_> = pages.into_iter().filter(|p| p.date.is_some()).collect();

        let xml = generator.generate(&posts)?;
        let output_path = self.output_dir.join("rss.xml");
        fs::write(&output_path, xml)?;

        info!(path = %output_path.display(), "generated RSS feed");
        Ok(())
    }

    /// Generate sitemap.
    fn generate_sitemap(&self, content: &SiteContent) -> Result<()> {
        let generator = SitemapGenerator::new(self.config.clone());
        let pages: Vec<_> = content.pages.values().collect();

        let xml = generator.generate(&pages)?;
        let output_path = self.output_dir.join("sitemap.xml");
        fs::write(&output_path, xml)?;

        info!(path = %output_path.display(), "generated sitemap");
        Ok(())
    }

    /// Process static assets.
    fn process_assets(&self, static_dir: &Path) -> Result<AssetManifest> {
        let processor = AssetProcessor::new(self.config.build.minify);
        let manifest = processor.process(static_dir, &self.output_dir)?;

        // Write manifest
        let manifest_path = self.output_dir.join("asset-manifest.json");
        fs::write(&manifest_path, manifest.to_json())?;

        Ok(manifest)
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

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
            rss: typstify_core::config::RssConfig {
                enabled: true,
                limit: 20,
            },
            taxonomies: typstify_core::config::TaxonomyConfig::default(),
        }
    }

    #[test]
    fn test_build_empty_site() {
        let content_dir = TempDir::new().unwrap();
        let output_dir = TempDir::new().unwrap();

        let builder = Builder::new(test_config(), content_dir.path(), output_dir.path());

        let stats = builder.build().unwrap();

        assert_eq!(stats.pages, 0);
        assert!(output_dir.path().join("sitemap.xml").exists());
        assert!(output_dir.path().join("rss.xml").exists());
    }

    #[test]
    fn test_build_with_content() {
        let content_dir = TempDir::new().unwrap();
        let output_dir = TempDir::new().unwrap();

        // Create a test markdown file with proper frontmatter
        let post_path = content_dir.path().join("test-post.md");
        fs::write(
            &post_path,
            r#"---
title: "Test Post"
date: 2026-01-14T00:00:00Z
tags:
  - rust
  - web
---

Hello, world!
"#,
        )
        .unwrap();

        // Verify file was created
        assert!(post_path.exists());

        let builder = Builder::new(test_config(), content_dir.path(), output_dir.path());

        let stats = builder.build().unwrap();

        // Check outputs
        let html_path = output_dir.path().join("test-post/index.html");
        let tags_rust = output_dir.path().join("tags/rust/index.html");
        let tags_web = output_dir.path().join("tags/web/index.html");

        // Debug: print what exists
        if html_path.exists() {
            eprintln!("HTML exists at {:?}", html_path);
        } else {
            eprintln!("HTML NOT found at {:?}", html_path);
            // List output dir contents
            if output_dir.path().exists() {
                for entry in std::fs::read_dir(output_dir.path()).unwrap() {
                    eprintln!("  Output contains: {:?}", entry.unwrap().path());
                }
            }
        }

        assert_eq!(stats.pages, 1, "Expected 1 page, got {}", stats.pages);
        assert!(html_path.exists(), "HTML file should exist");
        assert!(tags_rust.exists(), "tags/rust should exist");
        assert!(tags_web.exists(), "tags/web should exist");
    }

    #[test]
    fn test_build_stats() {
        let stats = BuildStats::default();
        assert_eq!(stats.pages, 0);
        assert_eq!(stats.duration_ms, 0);
    }

    #[test]
    fn test_builder_with_static_dir() {
        let content_dir = TempDir::new().unwrap();
        let output_dir = TempDir::new().unwrap();
        let static_dir = TempDir::new().unwrap();

        // Create a static file
        fs::write(static_dir.path().join("style.css"), "body {}").unwrap();

        let builder = Builder::new(test_config(), content_dir.path(), output_dir.path())
            .with_static_dir(static_dir.path());

        let stats = builder.build().unwrap();

        assert_eq!(stats.assets, 1);
        assert!(output_dir.path().join("style.css").exists());
    }
}
