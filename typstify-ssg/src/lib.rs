//! Typstify SSG - A Static Site Generator for Markdown and Typst content
//!
//! This library provides a simple way to build static sites from Markdown and Typst files,
//! with modern CSS support via Tailwind CSS and DaisyUI.

pub mod config;
pub mod content;
pub mod content_id;
pub mod feed;
pub mod mdbook_template;
pub mod metadata;
pub mod renderers;
pub mod search;

use std::path::{Path, PathBuf};

pub use config::*;
pub use content::*;
pub use content_id::*;
use eyre::Result;
pub use mdbook_template::*;
pub use metadata::*;
pub use renderers::*;
pub use search::*;
use tracing::info;

/// Main site builder struct
pub struct Site {
    pub content_dir: PathBuf,
    pub output_dir: PathBuf,
    pub config: AppConfig,
    pub content: Vec<Content>,
    pub search_engine: Option<SearchEngine>,
}

impl Site {
    /// Create a new Site with the given content and output directories
    pub fn new(content_dir: impl Into<PathBuf>, output_dir: impl Into<PathBuf>) -> Self {
        Self {
            content_dir: content_dir.into(),
            output_dir: output_dir.into(),
            config: AppConfig::default(),
            content: Vec::new(),
            search_engine: None,
        }
    }

    /// Set the site configuration (legacy LegacySiteConfig)
    pub fn with_config(mut self, config: LegacySiteConfig) -> Self {
        self.config.site = config.into();
        self
    }

    /// Set the full app configuration
    pub fn with_app_config(mut self, config: AppConfig) -> Self {
        self.config = config;
        self
    }

    /// Initialize the search engine
    pub fn init_search_engine(&mut self) -> Result<()> {
        let index_dir = self.output_dir.join(".search_index");
        self.search_engine = Some(SearchEngine::new(index_dir)?);
        Ok(())
    }

    /// Scan the content directory for Markdown and Typst files
    pub fn scan_content(&mut self) -> Result<()> {
        self.content = Content::scan_directory(&self.content_dir)?;
        info!("Found {} content files", self.content.len());
        Ok(())
    }

    /// Build the entire site
    pub fn build(&self) -> Result<()> {
        info!(
            "Building site from {} to {}",
            self.content_dir.display(),
            self.output_dir.display()
        );

        // Create output directory
        std::fs::create_dir_all(&self.output_dir)?;

        // Copy static assets if they exist
        self.copy_assets()?;

        // Copy style files
        self.copy_styles()?;

        // Generate HTML pages
        self.generate_pages()?;

        // Generate RSS/Atom feed if enabled
        self.generate_feed()?;

        // Build search index if search engine is available
        self.build_search_index()?;

        info!("Site build complete!");
        Ok(())
    }

    /// Copy static assets to the output directory
    fn copy_assets(&self) -> Result<()> {
        // Create assets directory in output
        let output_assets = self.output_dir.join("assets");
        std::fs::create_dir_all(&output_assets)?;

        // Copy search JavaScript file
        let search_js_content = include_str!("../assets/search.js");
        std::fs::write(output_assets.join("search.js"), search_js_content)?;
        info!("Copied search.js to output assets");

        // Copy search CSS file
        let search_css_content = include_str!("../assets/search.css");
        std::fs::write(output_assets.join("search.css"), search_css_content)?;
        info!("Copied search.css to output assets");

        // Copy user assets if they exist
        let user_assets_dir = self.content_dir.join("assets");
        if user_assets_dir.exists() {
            for entry in std::fs::read_dir(&user_assets_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() {
                    let file_name = path.file_name().unwrap();
                    let dest_path = output_assets.join(file_name);
                    std::fs::copy(&path, &dest_path)?;
                    info!(
                        "Copied user asset: {} to {}",
                        path.display(),
                        dest_path.display()
                    );
                }
            }
        }

        Ok(())
    }

    /// Copy style files to the output directory
    /// Uses embedded CSS content from the binary instead of external files
    fn copy_styles(&self) -> Result<()> {
        let output_style = self.output_dir.join("style");
        std::fs::create_dir_all(&output_style)?;

        // Write embedded CSS content to output.css
        let embedded_css = include_str!("../../style/output.css");
        let output_css_path = output_style.join("output.css");
        std::fs::write(&output_css_path, embedded_css)?;
        info!("Wrote embedded CSS to: {}", output_css_path.display());

        // Also write as input.css for compatibility
        let input_css_path = output_style.join("input.css");
        std::fs::write(&input_css_path, embedded_css)?;
        info!("Wrote embedded CSS to: {}", input_css_path.display());

        // If there's a style directory, still copy additional CSS files
        let style_dir = self.config.build.style_dir.clone();
        if style_dir.exists() {
            // Copy any additional CSS files (excluding output.css and input.css)
            for entry in std::fs::read_dir(&style_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("css") {
                    let file_name = path.file_name().unwrap().to_string_lossy();
                    // Skip the main CSS files as they're now embedded
                    if file_name != "output.css" && file_name != "input.css" {
                        let dest_path = output_style.join(&*file_name);
                        std::fs::copy(&path, &dest_path)?;
                        info!(
                            "Copied additional style: {} to {}",
                            path.display(),
                            dest_path.display()
                        );
                    }
                }
            }
        }
        Ok(())
    }

    /// Generate HTML pages for all content
    fn generate_pages(&self) -> Result<()> {
        // Create template generator
        let legacy_config = LegacySiteConfig {
            website_title: self.config.site.title.clone(),
            website_tagline: self.config.site.description.clone(),
            base_url: self.config.site.base_url.clone(),
            author: self.config.site.author.clone(),
        };
        let template = MdBookTemplate::new(legacy_config, self.content.clone());

        for content in &self.content {
            let html = template.generate_page(content, &content.slug())?;

            // Create output path based on content slug
            let output_path = self.output_dir.join(format!("{}.html", content.slug()));

            // Ensure output directory exists
            if let Some(parent) = output_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            // Write HTML file
            std::fs::write(&output_path, html)?;
            info!("Generated: {}", output_path.display());
        }

        // Generate index page
        self.generate_index()?;

        Ok(())
    }

    /// Build search index for all content
    fn build_search_index(&self) -> Result<()> {
        if let Some(search_engine) = &self.search_engine {
            // Rebuild search index with current content
            search_engine.rebuild_index(&self.content)?;

            // Export search results to JSON for client-side use
            let search_json_path = self.output_dir.join("search-index.json");
            search_engine.export_search_results(&search_json_path, 1000)?;

            info!("Search index built successfully");
        } else {
            info!("Search engine not initialized, skipping search index");
        }
        Ok(())
    }

    /// Generate RSS/Atom feed
    fn generate_feed(&self) -> Result<()> {
        if !self.config.features.feed {
            info!("Feed generation is disabled, skipping");
            return Ok(());
        }

        // Sort content by date (most recent first)
        let mut sorted_content = self.content.clone();
        sorted_content.sort_by(|a, b| {
            // Compare dates, putting items with dates first
            match (a.metadata.get_date(), b.metadata.get_date()) {
                (Some(date_a), Some(date_b)) => {
                    // Try to parse as RFC3339 first, then as simple date
                    let parsed_a = chrono::DateTime::parse_from_rfc3339(date_a).or_else(|_| {
                        chrono::NaiveDate::parse_from_str(date_a, "%Y-%m-%d")
                            .map(|d| d.and_hms_opt(0, 0, 0).unwrap().and_utc().into())
                    });
                    let parsed_b = chrono::DateTime::parse_from_rfc3339(date_b).or_else(|_| {
                        chrono::NaiveDate::parse_from_str(date_b, "%Y-%m-%d")
                            .map(|d| d.and_hms_opt(0, 0, 0).unwrap().and_utc().into())
                    });

                    match (parsed_a, parsed_b) {
                        (Ok(a), Ok(b)) => b.cmp(&a),                    // Most recent first
                        (Ok(_), Err(_)) => std::cmp::Ordering::Less,    // Valid date comes first
                        (Err(_), Ok(_)) => std::cmp::Ordering::Greater, // Valid date comes first
                        (Err(_), Err(_)) => date_b.cmp(date_a), // Fallback to string comparison
                    }
                }
                (Some(_), None) => std::cmp::Ordering::Less, // Items with dates come first
                (None, Some(_)) => std::cmp::Ordering::Greater, // Items with dates come first
                (None, None) => std::cmp::Ordering::Equal,   // No preference
            }
        });

        // Generate feed
        let feed = crate::feed::create_feed(&self.config, &sorted_content);

        // Write feed to file
        let feed_path = self.output_dir.join(&self.config.feed.filename);
        let feed_xml = feed.to_string();
        std::fs::write(&feed_path, feed_xml)?;

        info!("Generated feed: {}", feed_path.display());
        Ok(())
    }

    /// Search content using the search engine
    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        if let Some(search_engine) = &self.search_engine {
            search_engine.search(query, limit)
        } else {
            Ok(Vec::new())
        }
    }

    /// Generate the index page listing all content
    fn generate_index(&self) -> Result<()> {
        let legacy_config = LegacySiteConfig {
            website_title: self.config.site.title.clone(),
            website_tagline: self.config.site.description.clone(),
            base_url: self.config.site.base_url.clone(),
            author: self.config.site.author.clone(),
        };
        let template = MdBookTemplate::new(legacy_config, self.content.clone());
        let index_html = template.generate_index_page()?;
        let index_path = self.output_dir.join("index.html");
        std::fs::write(&index_path, index_html)?;
        info!("Generated index: {}", index_path.display());
        Ok(())
    }

    /// Generate HTML for a single content item
    #[allow(dead_code)]
    fn generate_html_for_content(&self, content: &Content) -> Result<String> {
        let rendered_content = content.render()?;

        let html = format!(
            r#"<!DOCTYPE html>
<html lang="en" data-theme="light">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>{} - {}</title>
    <link rel="stylesheet" href="/style/site.css">
</head>
<body class="bg-base-100 text-base-content">
    <div class="container mx-auto px-4 py-8">
        <header class="mb-8">
            <h1 class="text-4xl font-bold text-primary mb-4">{}</h1>
            <div class="text-sm text-base-content/70">
                {}
            </div>
        </header>
        
        <main class="prose prose-lg max-w-none">
            {}
        </main>
        
        <footer class="mt-12 pt-8 border-t border-base-300">
            <p class="text-center text-base-content/70">
                Built with Typstify SSG
            </p>
        </footer>
    </div>
</body>
</html>"#,
            content.metadata.get_title(),
            self.config.website_title(),
            content.metadata.get_title(),
            if let Some(date) = content.metadata.get_date() {
                format!("Published on {}", date)
            } else {
                "".to_string()
            },
            rendered_content
        );

        Ok(html)
    }

    /// Generate the index HTML page
    #[allow(dead_code)]
    fn generate_index_html(&self) -> Result<String> {
        let mut posts_html = String::new();

        for content in &self.content {
            posts_html.push_str(&format!(
                r#"<div class="card bg-base-200 shadow-md mb-6">
                    <div class="card-body">
                        <h2 class="card-title text-2xl">
                            <a href="{}.html" class="link link-primary">{}</a>
                        </h2>
                        {}
                        <div class="card-actions justify-end">
                            <a href="{}.html" class="btn btn-primary">Read More</a>
                        </div>
                    </div>
                </div>"#,
                content.slug(),
                content.metadata.get_title(),
                if let Some(summary) = content.metadata.get_summary() {
                    format!("<p class=\"text-base-content/80\">{}</p>", summary)
                } else {
                    "".to_string()
                },
                content.slug()
            ));
        }

        let html = format!(
            r#"<!DOCTYPE html>
<html lang="en" data-theme="light">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>{}</title>
    <link rel="stylesheet" href="/style/site.css">
</head>
<body class="bg-base-100 text-base-content">
    <div class="container mx-auto px-4 py-8">
        <header class="hero bg-gradient-to-r from-primary to-secondary text-primary-content mb-12 rounded-lg">
            <div class="hero-content text-center py-16">
                <div class="max-w-md">
                    <h1 class="text-5xl font-bold">{}</h1>
                    <p class="py-6 text-xl">{}</p>
                </div>
            </div>
        </header>
        
        <main>
            <section class="mb-12">
                <h2 class="text-3xl font-bold mb-8">Latest Posts</h2>
                {}
            </section>
        </main>
        
        <footer class="mt-12 pt-8 border-t border-base-300">
            <p class="text-center text-base-content/70">
                Built with Typstify SSG
            </p>
        </footer>
    </div>
</body>
</html>"#,
            self.config.website_title(),
            self.config.website_title(),
            self.config.website_tagline(),
            posts_html
        );

        Ok(html)
    }
}

/// Copy a directory recursively
#[allow(dead_code)]
fn copy_dir(src: &Path, dst: &Path) -> Result<()> {
    std::fs::create_dir_all(dst)?;

    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = entry.file_name();
        let dst_path = dst.join(&file_name);

        if path.is_dir() {
            copy_dir(&path, &dst_path)?;
        } else {
            std::fs::copy(&path, &dst_path)?;
        }
    }

    Ok(())
}
