//! Typstify Search Library
//!
//! Search index generation with Tantivy for full-text search capabilities.
//!
//! # Features
//!
//! - **Tantivy-based indexing**: Full-text search with language-aware tokenization
//! - **Index chunking**: Split large indexes for efficient browser loading
//! - **Simple index**: Lightweight JSON-based alternative for small sites
//!
//! # Example
//!
//! ```no_run
//! use std::path::Path;
//!
//! use typstify_search::{IndexerConfig, SearchIndexer, SimpleSearchIndex};
//!
//! // Build a Tantivy index
//! let indexer =
//!     SearchIndexer::new(Path::new("./search-index"), IndexerConfig::default()).unwrap();
//! // indexer.index_pages(&pages)?;
//!
//! // Or use simple JSON index for small sites
//! // let simple_index = SimpleSearchIndex::from_pages(&pages);
//! // simple_index.write_to_file(Path::new("search.json"))?;
//! ```

pub mod chunker;
pub mod indexer;
pub mod schema;
pub mod simple;

pub use chunker::{ChunkerConfig, FileManifest, IndexChunker, IndexManifest};
pub use indexer::{IndexStats, IndexerConfig, SearchIndexer};
pub use schema::{SearchFields, create_search_schema, register_tokenizers};
pub use simple::{MAX_SIMPLE_INDEX_SIZE, SimpleDocument, SimpleSearchIndex};
use thiserror::Error;

/// Search-related errors.
#[derive(Debug, Error)]
pub enum SearchError {
    /// I/O error.
    #[error("I/O error: {0}")]
    Io(String),

    /// Index error.
    #[error("Index error: {0}")]
    Index(String),

    /// Serialization error.
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Query error.
    #[error("Query error: {0}")]
    Query(String),
}

/// Result type for search operations.
pub type Result<T> = std::result::Result<T, SearchError>;
