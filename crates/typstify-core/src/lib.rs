//! Typstify Core Library
//!
//! Core types, configuration, and error handling for the Typstify static site generator.

pub mod config;
pub mod content;
pub mod error;
pub mod frontmatter;

pub use config::Config;
pub use content::{ContentPath, ContentType, Page, ParsedContent};
pub use error::{CoreError, Result};
pub use frontmatter::Frontmatter;
