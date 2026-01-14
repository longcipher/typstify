//! Typstify UI Components
//!
//! Leptos components for the Typstify frontend.
//!
//! # Components
//!
//! ## Search
//! - [`SearchBox`] - Text input with debounced search
//! - [`SearchResults`] - List of search results
//! - [`SearchModal`] - Modal dialog for search (Cmd/Ctrl+K)
//! - [`SearchShortcut`] - Global keyboard shortcut handler
//!
//! ## Article
//! - [`Article`] - Renders HTML content with custom CSS/JS
//! - [`ArticleMeta`] - Article metadata (date, reading time, tags)
//! - [`Prose`] - Styled prose wrapper
//!
//! ## Navigation
//! - [`Navigation`] - Main site navigation
//! - [`TableOfContents`] - Article table of contents
//! - [`Breadcrumbs`] - Breadcrumb navigation
//!
//! # Example
//!
//! ```ignore
//! use leptos::prelude::*;
//! use typstify_ui::{SearchBox, SearchResults, SearchResultItem};
//!
//! #[component]
//! fn App() -> impl IntoView {
//!     let query = RwSignal::new(String::new());
//!     let results = Signal::derive(|| vec![]);
//!     let loading = Signal::derive(|| false);
//!
//!     view! {
//!         <SearchBox query=query loading=loading />
//!         <SearchResults results=results />
//!     }
//! }
//! ```

pub mod article;
pub mod navigation;
pub mod search;

pub use article::{Article, ArticleData, ArticleMeta, Prose};
pub use navigation::{Breadcrumbs, NavItem, Navigation, TableOfContents, TocEntry};
pub use search::{SearchBox, SearchModal, SearchResultItem, SearchResults, SearchShortcut};
