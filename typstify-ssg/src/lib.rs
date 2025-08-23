//! Typstify SSG - A Static Site Generator for Markdown and Typst content
//!
//! This library provides a simple way to build static sites from Markdown and Typst files,
//! with modern CSS support via Tailwind CSS and DaisyUI.

pub mod config;
pub mod content;
pub mod content_id;
pub mod mdbook_template;
pub mod metadata;
pub mod renderers;

pub use config::*;
pub use content::*;
pub use content_id::*;
pub use mdbook_template::*;
pub use metadata::*;
pub use renderers::*;

use eyre::Result;
use std::path::{Path, PathBuf};
use tracing::info;

/// Main site builder struct
pub struct Site {
    pub content_dir: PathBuf,
    pub output_dir: PathBuf,
    pub config: AppConfig,
    pub content: Vec<Content>,
}

impl Site {
    /// Create a new Site with the given content and output directories
    pub fn new(content_dir: impl Into<PathBuf>, output_dir: impl Into<PathBuf>) -> Self {
        Self {
            content_dir: content_dir.into(),
            output_dir: output_dir.into(),
            config: AppConfig::default(),
            content: Vec::new(),
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

        info!("Site build complete!");
        Ok(())
    }

    /// Copy static assets to the output directory
    fn copy_assets(&self) -> Result<()> {
        let assets_dir = self.content_dir.join("assets");
        if assets_dir.exists() {
            let output_assets = self.output_dir.join("assets");
            if output_assets.exists() {
                std::fs::remove_dir_all(&output_assets)?;
            }
            copy_dir(&assets_dir, &output_assets)?;
            info!(
                "Copied assets from {} to {}",
                assets_dir.display(),
                output_assets.display()
            );
        }
        Ok(())
    }

    /// Copy style files to the output directory
    fn copy_styles(&self) -> Result<()> {
        let style_dir = self.config.build.style_dir.clone();
        if style_dir.exists() {
            let output_style = self.output_dir.join("style");
            std::fs::create_dir_all(&output_style)?;

            // Copy all CSS files from style directory
            for entry in std::fs::read_dir(&style_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("css") {
                    let file_name = path.file_name().unwrap();
                    let dest_path = output_style.join(file_name);
                    std::fs::copy(&path, &dest_path)?;
                    info!(
                        "Copied style: {} to {}",
                        path.display(),
                        dest_path.display()
                    );
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
