//! Typstify CLI Library
//!
//! This library provides the core functionality for the Typstify static site generator CLI.
//! It is designed to be used by the binary entry point while also exposing
//! public APIs for documentation and integration purposes.
//!
//! # Modules
//!
//! - [`cmd`] - Command implementations (build, watch, new, check)
//! - [`server`] - Embedded development server with live reload
//!
//! # Example
//!
//! ```no_run
//! use std::path::Path;
//!
//! use typstify::cmd;
//!
//! // Build a static site
//! cmd::build::run(Path::new("config.toml"), Path::new("public"), false).unwrap();
//! ```

pub mod cmd;
pub mod server;

// Re-export core types for convenience
pub use typstify_core::{Config, Page};
pub use typstify_generator::{BuildStats, Builder, ContentCollector, SiteContent};

/// Initialize tracing with the specified verbosity level.
///
/// # Arguments
///
/// * `verbose` - Verbosity level (0 = WARN, 1 = INFO, 2 = DEBUG, 3+ = TRACE)
///
/// # Example
///
/// ```no_run
/// typstify::init_tracing(2); // Enable DEBUG level logging
/// ```
pub fn init_tracing(verbose: u8) {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

    let level = match verbose {
        0 => tracing::Level::WARN,
        1 => tracing::Level::INFO,
        2 => tracing::Level::DEBUG,
        _ => tracing::Level::TRACE,
    };

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env().add_directive(level.into()))
        .init();
}
