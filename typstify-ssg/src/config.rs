//! Configuration for the typstify SSG

use std::path::PathBuf;

use config::{Config, File};
use eyre::Result;
use serde::{Deserialize, Serialize};

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub site: SiteConfig,
    pub build: BuildConfig,
    pub rendering: RenderingConfig,
    pub features: FeaturesConfig,
    pub feed: FeedConfig,
    pub dev: DevConfig,
}

/// Site configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteConfig {
    pub title: String,
    pub description: String,
    pub base_url: String,
    pub author: String,
}

/// Build configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildConfig {
    pub content_dir: PathBuf,
    pub output_dir: PathBuf,
    pub style_dir: PathBuf,
    pub assets_dir: PathBuf,
}

/// Rendering configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderingConfig {
    pub syntax_highlighting: bool,
    pub code_theme: String,
    pub generate_toc: bool,
    pub toc_depth: u8,
}

/// Features configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeaturesConfig {
    pub feed: bool,
    pub sitemap: bool,
    pub search: bool,
    pub opengraph: bool,
}

/// Feed configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedConfig {
    pub filename: String,
    pub max_items: usize,
    pub language: String,
}

/// Development configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevConfig {
    pub port: u16,
    pub watch: bool,
    pub reload_port: u16,
}

impl Default for SiteConfig {
    fn default() -> Self {
        Self {
            title: "Typstify Documentation".to_string(),
            description: "A static site generator that supports both Markdown and Typst content"
                .to_string(),
            base_url: "https://typstify.dev".to_string(),
            author: "Typstify Team".to_string(),
        }
    }
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            content_dir: PathBuf::from("contents"),
            output_dir: PathBuf::from("site"),
            style_dir: PathBuf::from("style"),
            assets_dir: PathBuf::from("assets"),
        }
    }
}

impl Default for RenderingConfig {
    fn default() -> Self {
        Self {
            syntax_highlighting: true,
            code_theme: "dracula".to_string(),
            generate_toc: true,
            toc_depth: 3,
        }
    }
}

impl Default for FeaturesConfig {
    fn default() -> Self {
        Self {
            feed: true,
            sitemap: true,
            search: false,
            opengraph: true,
        }
    }
}

impl Default for FeedConfig {
    fn default() -> Self {
        Self {
            filename: "feed.xml".to_string(),
            max_items: 20,
            language: "en".to_string(),
        }
    }
}

impl Default for DevConfig {
    fn default() -> Self {
        Self {
            port: 5173,
            watch: true,
            reload_port: 3002,
        }
    }
}

impl AppConfig {
    /// Load configuration from file
    /// Supports TOML, YAML, and JSON formats
    pub fn from_file(path: &str) -> Result<Self> {
        let builder = Config::builder()
            .add_source(File::with_name(path))
            .build()?;

        let config = builder.try_deserialize::<AppConfig>()?;
        Ok(config)
    }

    /// Load configuration with optional file override
    /// Falls back to default if file doesn't exist
    pub fn load_or_default(config_path: Option<&str>) -> Result<Self> {
        match config_path {
            Some(path) if std::path::Path::new(path).exists() => Self::from_file(path),
            Some(path) => {
                tracing::warn!("Config file {} not found, using defaults", path);
                Ok(Self::default())
            }
            None => {
                // Try to find config file in common locations
                for path in &["config.toml", "config.yaml", "config.yml", "config.json"] {
                    if std::path::Path::new(path).exists() {
                        return Self::from_file(path);
                    }
                }
                tracing::info!("No config file found, using defaults");
                Ok(Self::default())
            }
        }
    }

    /// Get website title (for backward compatibility)
    pub fn website_title(&self) -> &str {
        &self.site.title
    }

    /// Get website tagline (for backward compatibility)
    pub fn website_tagline(&self) -> &str {
        &self.site.description
    }

    /// Get base URL (for backward compatibility)
    pub fn base_url(&self) -> &str {
        &self.site.base_url
    }

    /// Get author (for backward compatibility)
    pub fn author(&self) -> &str {
        &self.site.author
    }
}

/// Legacy SiteConfig for backward compatibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacySiteConfig {
    pub website_title: String,
    pub website_tagline: String,
    pub base_url: String,
    pub author: String,
}

impl Default for LegacySiteConfig {
    fn default() -> Self {
        let app_config = AppConfig::default();
        Self {
            website_title: app_config.site.title,
            website_tagline: app_config.site.description,
            base_url: app_config.site.base_url,
            author: app_config.site.author,
        }
    }
}

impl From<AppConfig> for LegacySiteConfig {
    fn from(app_config: AppConfig) -> Self {
        Self {
            website_title: app_config.site.title,
            website_tagline: app_config.site.description,
            base_url: app_config.site.base_url,
            author: app_config.site.author,
        }
    }
}

impl From<LegacySiteConfig> for SiteConfig {
    fn from(legacy: LegacySiteConfig) -> Self {
        Self {
            title: legacy.website_title,
            description: legacy.website_tagline,
            base_url: legacy.base_url,
            author: legacy.author,
        }
    }
}
