//! Robots.txt generation.
//!
//! Generates the robots.txt file for search engine crawlers.

use std::{fs::File, io::Write, path::Path};

use thiserror::Error;
use tracing::info;
use typstify_core::Config;

/// Robots generation errors.
#[derive(Debug, Error)]
pub enum RobotsError {
    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type for robots generation.
pub type Result<T> = std::result::Result<T, RobotsError>;

/// Robots.txt generator.
#[derive(Debug)]
pub struct RobotsGenerator {
    config: Config,
}

impl RobotsGenerator {
    /// Create a new robots generator.
    #[must_use]
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Generate robots.txt.
    pub fn generate(&self, output_dir: &Path) -> Result<()> {
        if !self.config.robots.enabled {
            return Ok(());
        }

        info!("generating robots.txt");

        let path = output_dir.join("robots.txt");
        let mut file = File::create(path)?;

        writeln!(file, "User-agent: *")?;

        for path in &self.config.robots.disallow {
            writeln!(file, "Disallow: {path}")?;
        }

        for path in &self.config.robots.allow {
            writeln!(file, "Allow: {path}")?;
        }

        // Add sitemap reference if configured (defaulting to sitemap.xml in root)
        let sitemap_url = format!("{}/sitemap.xml", self.config.site.base_url);
        writeln!(file, "Sitemap: {sitemap_url}")?;

        Ok(())
    }
}
