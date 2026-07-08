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
pub struct RobotsGenerator<'a> {
    config: &'a Config,
}

impl<'a> RobotsGenerator<'a> {
    /// Create a new robots generator.
    #[must_use]
    pub fn new(config: &'a Config) -> Self {
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
        let sitemap_url = format!("{}/sitemap.xml", self.config.base_url());
        writeln!(file, "Sitemap: {sitemap_url}")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, fs};

    use typstify_core::test_fixtures::test_config;

    use super::*;

    #[test]
    fn test_disabled_produces_no_file() {
        let mut config = test_config();
        config.robots.enabled = false;

        let dir = tempfile::tempdir().expect("create temp dir");
        let generator = RobotsGenerator::new(&config);
        generator.generate(dir.path()).unwrap();

        assert!(!dir.path().join("robots.txt").exists());
    }

    #[test]
    fn test_enabled_with_disallow_allow_paths() {
        let mut config = test_config();
        config.robots.disallow = vec!["/private/".to_string(), "/admin/".to_string()];
        config.robots.allow = vec!["/public/".to_string()];

        let dir = tempfile::tempdir().expect("create temp dir");
        let generator = RobotsGenerator::new(&config);
        generator.generate(dir.path()).unwrap();

        let content = fs::read_to_string(dir.path().join("robots.txt")).unwrap();

        assert!(content.starts_with("User-agent: *\n"));
        assert!(content.contains("Disallow: /private/\n"));
        assert!(content.contains("Disallow: /admin/\n"));
        assert!(content.contains("Allow: /public/\n"));
        assert!(content.contains("Sitemap: https://example.com/sitemap.xml\n"));
    }

    #[test]
    fn test_sitemap_includes_base_path() {
        let config = Config::from_parts(
            typstify_core::config::SiteConfig {
                title: "Test Site".to_string(),
                host: "https://example.com".to_string(),
                base_path: "/typstify".to_string(),
                default_language: "en".to_string(),
                description: None,
                author: None,
            },
            typstify_core::config::BuildConfig::default(),
            typstify_core::config::SearchConfig::default(),
            typstify_core::config::RssConfig::default(),
            typstify_core::config::RobotsConfig::default(),
            typstify_core::config::TaxonomyConfig::default(),
            HashMap::new(),
        );

        let dir = tempfile::tempdir().expect("create temp dir");
        let generator = RobotsGenerator::new(&config);
        generator.generate(dir.path()).unwrap();

        let content = fs::read_to_string(dir.path().join("robots.txt")).unwrap();

        assert!(content.contains("Sitemap: https://example.com/typstify/sitemap.xml\n"));
    }
}
