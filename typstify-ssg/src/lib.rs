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

use std::path::{Path, PathBuf};

pub use config::*;
pub use content::*;
pub use content_id::*;
use eyre::Result;
pub use mdbook_template::*;
pub use metadata::*;
pub use renderers::*;
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

    /// Set the site configuration
    pub fn with_site_config(mut self, config: SiteConfig) -> Self {
        self.config.site = config;
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
                    let file_name = path.file_name().expect("Path should have a filename");
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
        let template = MdBookTemplate::new(self.config.site.clone(), self.content.clone());

        for content in &self.content {
            let html = template.generate_page(content, &content.slug())?;

            // Create output path based on content slug with proper directory structure
            let output_path = if content.slug().contains('/') {
                // For nested paths like "getting-started/installation"
                self.output_dir.join(format!("{}.html", content.slug()))
            } else {
                // For root level files
                self.output_dir.join(format!("{}.html", content.slug()))
            };

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
        let template = MdBookTemplate::new(self.config.site.clone(), self.content.clone());
        let index_html = template.generate_index_page()?;
        let index_path = self.output_dir.join("index.html");
        std::fs::write(&index_path, index_html)?;
        info!("Generated index: {}", index_path.display());
        Ok(())
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
