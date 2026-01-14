//! Error types for the Typstify core library.

use std::path::PathBuf;

use thiserror::Error;

/// Result type alias using `CoreError`.
pub type Result<T> = std::result::Result<T, CoreError>;

/// Core error types for Typstify.
#[derive(Error, Debug)]
pub enum CoreError {
    /// Configuration loading or parsing error.
    #[error("Configuration error: {message}")]
    Config {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Content parsing error with file location.
    #[error("Parse error in {path}: {message}")]
    Parse { path: PathBuf, message: String },

    /// Frontmatter parsing error.
    #[error("Frontmatter error in {path}: {message}")]
    Frontmatter { path: PathBuf, message: String },

    /// Template rendering error.
    #[error("Template error: {0}")]
    Template(String),

    /// Search index error.
    #[error("Search error: {0}")]
    Search(String),

    /// File system I/O error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// TOML parsing error.
    #[error("TOML parse error: {0}")]
    Toml(#[from] toml::de::Error),

    /// YAML parsing error.
    #[error("YAML parse error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    /// Generic configuration crate error.
    #[error("Config crate error: {0}")]
    ConfigCrate(#[from] config::ConfigError),
}

impl CoreError {
    /// Create a new configuration error with a message.
    pub fn config(message: impl Into<String>) -> Self {
        Self::Config {
            message: message.into(),
            source: None,
        }
    }

    /// Create a new configuration error with source.
    pub fn config_with_source(
        message: impl Into<String>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self::Config {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }

    /// Create a new parse error.
    pub fn parse(path: impl Into<PathBuf>, message: impl Into<String>) -> Self {
        Self::Parse {
            path: path.into(),
            message: message.into(),
        }
    }

    /// Create a new frontmatter error.
    pub fn frontmatter(path: impl Into<PathBuf>, message: impl Into<String>) -> Self {
        Self::Frontmatter {
            path: path.into(),
            message: message.into(),
        }
    }

    /// Create a new template error.
    pub fn template(message: impl Into<String>) -> Self {
        Self::Template(message.into())
    }

    /// Create a new search error.
    pub fn search(message: impl Into<String>) -> Self {
        Self::Search(message.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_error() {
        let err = CoreError::config("missing field");
        assert!(err.to_string().contains("Configuration error"));
        assert!(err.to_string().contains("missing field"));
    }

    #[test]
    fn test_parse_error() {
        let err = CoreError::parse("content/post.md", "invalid syntax");
        assert!(err.to_string().contains("Parse error"));
        assert!(err.to_string().contains("content/post.md"));
    }

    #[test]
    fn test_frontmatter_error() {
        let err = CoreError::frontmatter("content/post.md", "missing title");
        assert!(err.to_string().contains("Frontmatter error"));
        assert!(err.to_string().contains("missing title"));
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err: CoreError = io_err.into();
        assert!(err.to_string().contains("IO error"));
    }
}
