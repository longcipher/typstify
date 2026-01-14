//! Typstify Search WASM Runtime
//!
//! Browser-side search using WebAssembly.
//!
//! # Features
//!
//! - **SimpleSearchEngine**: Lightweight JSON-based search for small sites (<500KB)
//! - **SearchEngine**: Full chunked index support for larger sites (coming soon)
//! - **Chunk caching**: Efficient network usage with `scc::HashMap`
//!
//! # Example (JavaScript)
//!
//! ```javascript
//! import { SimpleSearchEngine } from 'typstify-search-wasm';
//!
//! // Load index
//! const engine = await new SimpleSearchEngine('/search.json');
//!
//! // Search
//! const results = engine.search('rust programming', 10);
//! console.log(results);
//! ```

pub mod directory;
pub mod query;
pub mod simple;

pub use directory::{DirectoryError, FileManifest, HttpDirectory, IndexManifest};
pub use query::{SearchQuery, SearchResult, SearchResults};
pub use simple::{SimpleDocument, SimpleSearchEngine, SimpleSearchIndex};
use wasm_bindgen::prelude::*;

/// Initialize the WASM module.
///
/// Sets up panic hook for better error messages in the console.
#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

/// Get the version of the search library.
#[wasm_bindgen(js_name = getVersion)]
pub fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Check if the library is ready.
#[wasm_bindgen(js_name = isReady)]
pub fn is_ready() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_version() {
        let version = get_version();
        assert!(!version.is_empty());
        assert!(version.starts_with("0."));
    }

    #[test]
    fn test_is_ready() {
        assert!(is_ready());
    }
}
