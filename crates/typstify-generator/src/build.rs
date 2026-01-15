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
use typstify_core::{Config, Page};
use typstify_search::SimpleSearchIndex;

use crate::{
    assets::{AssetError, AssetManifest, AssetProcessor},
    collector::{CollectorError, ContentCollector, SiteContent, paginate},
    html::{
        HtmlError, HtmlGenerator, list_item_html, pagination_html, shorts_with_separators_html,
    },
    robots::{RobotsError, RobotsGenerator},
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

    /// Robots generation error.
    #[error("robots error: {0}")]
    Robots(#[from] RobotsError),

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

    /// Number of auto-generated index pages (archives, tags index, section indices).
    pub auto_pages: usize,

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

        // 3. Extract sections for dynamic navigation
        let sections: Vec<String> = content.sections.keys().cloned().collect();

        // 4. Generate HTML pages
        stats.pages = self.generate_pages(&content, &sections)?;

        // 5. Generate taxonomy pages
        stats.taxonomy_pages = self.generate_taxonomy_pages(&content, &sections)?;

        // 6. Generate auto-generated index pages (archives, tags index, section indices)
        stats.auto_pages = self.generate_auto_pages(&content, &sections)?;

        // 6. Generate redirects
        stats.redirects = self.generate_redirects(&content)?;

        // 7. Generate RSS feed
        if self.config.rss.enabled {
            self.generate_rss(&content)?;
        }

        // 8. Generate sitemap
        self.generate_sitemap(&content)?;

        // 9. Generate robots.txt
        self.generate_robots()?;

        // 10. Generate search index (per language)
        if self.config.search.enabled {
            self.generate_search_indexes(&content)?;
        }

        // 11. Generate static CSS/JS assets for better caching
        crate::static_assets::generate_static_assets(&self.output_dir)
            .map_err(|e| BuildError::Io(std::io::Error::other(e.to_string())))?;

        // 12. Process user-provided assets
        if let Some(ref static_dir) = self.static_dir {
            let manifest = self.process_assets(static_dir)?;
            stats.assets = manifest.assets().len();
        }

        stats.duration_ms = start.elapsed().as_millis() as u64;

        info!(
            pages = stats.pages,
            taxonomy_pages = stats.taxonomy_pages,
            auto_pages = stats.auto_pages,
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
    fn generate_pages(&self, content: &SiteContent, sections: &[String]) -> Result<usize> {
        let generator = HtmlGenerator::new(self.config.clone()).with_sections(sections.to_vec());
        let pages: Vec<_> = content.pages.values().collect();

        info!(count = pages.len(), "generating HTML pages");

        // Generate pages in parallel
        let results: Vec<_> = pages
            .par_iter()
            .map(|page| {
                // Collect alternate language versions
                let mut alternates = Vec::new();
                if let Some(slugs) = content.translations.get(&page.canonical_id) {
                    for slug in slugs {
                        if let Some(alt_page) = content.pages.get(slug) {
                            alternates.push((alt_page.lang.as_str(), alt_page.url.as_str()));
                        }
                    }
                }

                let html = generator.generate_page(page, &alternates)?;
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
    fn generate_taxonomy_pages(&self, content: &SiteContent, sections: &[String]) -> Result<usize> {
        let generator = HtmlGenerator::new(self.config.clone()).with_sections(sections.to_vec());
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

    /// Generate auto-generated index pages: archives, tags index, categories index, section indices.
    /// Generates per-language versions when multiple languages are configured.
    fn generate_auto_pages(&self, content: &SiteContent, sections: &[String]) -> Result<usize> {
        let generator = HtmlGenerator::new(self.config.clone()).with_sections(sections.to_vec());
        let mut count = 0;

        // Get all languages
        let all_languages = self.config.all_languages();
        let default_lang = &self.config.site.default_language;

        // Generate pages for each language
        for lang in &all_languages {
            let is_default = *lang == default_lang.as_str();
            let lang_prefix = if is_default {
                String::new()
            } else {
                lang.to_string()
            };

            // Filter pages by language
            let lang_pages: Vec<_> = content.pages.values().filter(|p| p.lang == *lang).collect();

            // 1. Generate tags index page (/tags/ or /{lang}/tags/)
            let lang_tags: std::collections::HashMap<String, Vec<String>> = lang_pages
                .iter()
                .flat_map(|p| p.tags.iter().map(|t| (t.clone(), p.url.clone())))
                .fold(std::collections::HashMap::new(), |mut acc, (tag, url)| {
                    acc.entry(tag).or_default().push(url);
                    acc
                });

            if !lang_tags.is_empty() {
                let html = generator.generate_tags_index_page(&lang_tags, lang)?;
                let output_path = if is_default {
                    self.output_dir.join("tags").join("index.html")
                } else {
                    self.output_dir
                        .join(&lang_prefix)
                        .join("tags")
                        .join("index.html")
                };
                if let Some(parent) = output_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::write(&output_path, &html)?;
                count += 1;
                info!(path = %output_path.display(), lang = lang, "generated tags index page");
            }

            // 2. Generate categories index page (/categories/ or /{lang}/categories/)
            let lang_categories: std::collections::HashMap<String, Vec<String>> = lang_pages
                .iter()
                .flat_map(|p| p.categories.iter().map(|c| (c.clone(), p.url.clone())))
                .fold(std::collections::HashMap::new(), |mut acc, (cat, url)| {
                    acc.entry(cat).or_default().push(url);
                    acc
                });

            if !lang_categories.is_empty() {
                let html = generator.generate_categories_index_page(&lang_categories, lang)?;
                let output_path = if is_default {
                    self.output_dir.join("categories").join("index.html")
                } else {
                    self.output_dir
                        .join(&lang_prefix)
                        .join("categories")
                        .join("index.html")
                };
                if let Some(parent) = output_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::write(&output_path, &html)?;
                count += 1;
                info!(path = %output_path.display(), lang = lang, "generated categories index page");
            }

            // 3. Generate archives page (/archives/ or /{lang}/archives/)
            let mut lang_posts: Vec<_> = lang_pages
                .iter()
                .filter(|p| p.date.is_some())
                .copied()
                .collect();
            lang_posts.sort_by(|a, b| b.date.cmp(&a.date));

            if !lang_posts.is_empty() {
                let html = generator.generate_archives_page(&lang_posts, lang)?;
                let output_path = if is_default {
                    self.output_dir.join("archives").join("index.html")
                } else {
                    self.output_dir
                        .join(&lang_prefix)
                        .join("archives")
                        .join("index.html")
                };
                if let Some(parent) = output_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::write(&output_path, &html)?;
                count += 1;
                info!(path = %output_path.display(), lang = lang, "generated archives page");
            }

            // 4. Generate section index pages (e.g., /posts/, /{lang}/posts/)
            // Group pages by section within this language
            let mut sections: std::collections::HashMap<String, Vec<&Page>> =
                std::collections::HashMap::new();
            for page in lang_pages.iter().copied() {
                // Extract section from URL (first path segment after lang prefix if any)
                let url = page.url.trim_start_matches('/');
                let section = if is_default {
                    url.split('/').next().unwrap_or("")
                } else {
                    // For non-default lang, URL starts with /{lang}/section/...
                    url.split('/').nth(1).unwrap_or("")
                };

                if !section.is_empty() && section != "index.html" {
                    sections.entry(section.to_string()).or_default().push(page);
                }
            }

            for (section, mut section_pages) in sections {
                // Sort by date (newest first) or by title
                section_pages.sort_by(|a, b| match (&b.date, &a.date) {
                    (Some(b_date), Some(a_date)) => b_date.cmp(a_date),
                    (Some(_), None) => std::cmp::Ordering::Less,
                    (None, Some(_)) => std::cmp::Ordering::Greater,
                    (None, None) => a.title.cmp(&b.title),
                });

                // Generate paginated section index
                let per_page = self.config.taxonomies.tags.paginate;
                let total_pages = section_pages.len().div_ceil(per_page).max(1);

                // Use shorts-specific template for shorts section
                let is_shorts = section == "shorts";
                let author = self.config.site.author.as_deref().unwrap_or("Author");

                for page_num in 1..=total_pages {
                    let (page_items, _) = paginate(&section_pages, page_num, per_page);

                    // Use appropriate item html based on section type
                    let items_html: String = if is_shorts {
                        shorts_with_separators_html(page_items, author)
                    } else {
                        page_items.iter().map(|p| list_item_html(p)).collect()
                    };

                    let base_url = if is_default {
                        format!("/{section}")
                    } else {
                        format!("/{lang}/{section}")
                    };
                    let pagination = pagination_html(page_num, total_pages, &base_url);

                    // Use shorts template for shorts section
                    let html = if is_shorts {
                        generator.generate_shorts_page(
                            &section,
                            None, // description
                            &items_html,
                            pagination.as_deref(),
                            lang,
                        )?
                    } else {
                        generator.generate_section_page(
                            &section,
                            None, // description
                            &items_html,
                            pagination.as_deref(),
                            lang,
                        )?
                    };

                    let output_path = if page_num == 1 {
                        if is_default {
                            self.output_dir.join(&section).join("index.html")
                        } else {
                            self.output_dir
                                .join(&lang_prefix)
                                .join(&section)
                                .join("index.html")
                        }
                    } else if is_default {
                        self.output_dir
                            .join(&section)
                            .join("page")
                            .join(page_num.to_string())
                            .join("index.html")
                    } else {
                        self.output_dir
                            .join(&lang_prefix)
                            .join(&section)
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

                info!(section = %section, lang = %lang, "generated section index page");
            }
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

        // Generate main RSS feed with all languages
        let xml = generator.generate(&posts)?;
        let output_path = self.output_dir.join("rss.xml");
        fs::write(&output_path, xml)?;
        info!(path = %output_path.display(), "generated RSS feed");

        // Generate language-specific RSS feeds
        let all_languages = self.config.all_languages();
        let default_lang = &self.config.site.default_language;

        for lang in &all_languages {
            // Filter posts by language
            let lang_posts: Vec<_> = posts.iter().filter(|p| p.lang == *lang).copied().collect();

            if lang_posts.is_empty() {
                continue;
            }

            // Generate language-specific feed
            let lang_xml = generator.generate_for_lang(&lang_posts, lang)?;

            // Determine output path
            let lang_output_path = if *lang == default_lang.as_str() {
                // For default language, still put at root but also in lang folder
                self.output_dir.join(lang).join("rss.xml")
            } else {
                self.output_dir.join(lang).join("rss.xml")
            };

            // Create parent directories if needed
            if let Some(parent) = lang_output_path.parent() {
                fs::create_dir_all(parent)?;
            }

            fs::write(&lang_output_path, lang_xml)?;
            info!(path = %lang_output_path.display(), lang = lang, "generated language-specific RSS feed");
        }

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

        // Generate XSLT stylesheet for sitemap
        let xsl = crate::sitemap::generate_sitemap_xsl();
        let xsl_path = self.output_dir.join("sitemap-style.xsl");
        fs::write(&xsl_path, xsl)?;
        info!(path = %xsl_path.display(), "generated sitemap stylesheet");

        Ok(())
    }

    /// Generate robots.txt.
    fn generate_robots(&self) -> Result<()> {
        let generator = RobotsGenerator::new(self.config.clone());
        generator.generate(&self.output_dir)?;
        Ok(())
    }

    /// Generate search indexes per language.
    ///
    /// Creates a `search-index.json` for default language at root,
    /// and `/{lang}/search-index.json` for non-default languages.
    fn generate_search_indexes(&self, content: &SiteContent) -> Result<()> {
        let all_languages = self.config.all_languages();
        let default_lang = &self.config.site.default_language;

        for lang in &all_languages {
            // Filter pages by language
            let lang_pages: Vec<_> = content.pages.values().filter(|p| p.lang == *lang).collect();

            if lang_pages.is_empty() {
                continue;
            }

            // Build simple search index
            let index = SimpleSearchIndex::from_pages(&lang_pages);

            // Determine output path
            let output_path = if *lang == default_lang.as_str() {
                self.output_dir.join("search-index.json")
            } else {
                self.output_dir.join(lang).join("search-index.json")
            };

            // Create parent directories if needed
            if let Some(parent) = output_path.parent() {
                fs::create_dir_all(parent)?;
            }

            // Write the index
            index
                .write_to_file(&output_path)
                .map_err(|e| BuildError::Config(e.to_string()))?;

            info!(
                path = %output_path.display(),
                lang = lang,
                documents = lang_pages.len(),
                "generated search index"
            );
        }

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
    use std::collections::HashMap;

    use tempfile::TempDir;

    use super::*;

    fn test_config() -> Config {
        Config {
            site: typstify_core::config::SiteConfig {
                title: "Test Site".to_string(),
                base_url: "https://example.com".to_string(),
                default_language: "en".to_string(),
                description: None,
                author: None,
            },
            languages: HashMap::new(),
            build: typstify_core::config::BuildConfig::default(),
            search: typstify_core::config::SearchConfig::default(),
            rss: typstify_core::config::RssConfig {
                enabled: true,
                limit: 20,
            },
            robots: typstify_core::config::RobotsConfig::default(),
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
