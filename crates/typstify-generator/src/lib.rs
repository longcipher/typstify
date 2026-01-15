//! Typstify Generator Library
//!
//! Static site generation engine for Typstify.
//!
//! # Modules
//!
//! - [`template`] - HTML template system with variable interpolation
//! - [`html`] - HTML generation from parsed content
//! - [`collector`] - Content collection and organization
//! - [`rss`] - RSS feed generation
//! - [`sitemap`] - XML sitemap generation
//! - [`assets`] - Static asset processing with optional fingerprinting
//! - [`build`] - Build orchestration

pub mod assets;
pub mod build;
pub mod collector;
pub mod html;
pub mod robots;
pub mod rss;
pub mod sitemap;
pub mod static_assets;
pub mod template;

pub use assets::{AssetManifest, AssetProcessor};
pub use build::{BuildStats, Builder};
pub use collector::{ContentCollector, SiteContent, TaxonomyIndex};
pub use html::HtmlGenerator;
pub use robots::RobotsGenerator;
pub use rss::RssGenerator;
pub use sitemap::SitemapGenerator;
pub use static_assets::generate_static_assets;
pub use template::{Template, TemplateContext, TemplateRegistry};
