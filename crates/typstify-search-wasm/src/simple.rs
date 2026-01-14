//! Simple search implementation for small indices.
//!
//! Provides a lightweight search engine that loads the entire index into memory.
//! Suitable for sites with fewer than a few hundred pages.

use std::collections::HashMap;

use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::query::{SearchQuery, SearchResult, SearchResults, generate_snippet, score_document};

/// A simple search index document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleDocument {
    /// Document URL.
    pub url: String,

    /// Document title.
    pub title: String,

    /// Document description/summary.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Language code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,

    /// Tags.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,

    /// Publication date as ISO string.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,

    /// Pre-tokenized terms from title and body.
    pub terms: Vec<String>,
}

/// A simple JSON-based search index.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleSearchIndex {
    /// Index format version.
    pub version: u32,

    /// All indexed documents.
    pub documents: Vec<SimpleDocument>,

    /// Inverted index: term -> document indices.
    pub index: HashMap<String, Vec<usize>>,
}

impl SimpleSearchIndex {
    /// Create an empty index.
    pub fn empty() -> Self {
        Self {
            version: 1,
            documents: Vec::new(),
            index: HashMap::new(),
        }
    }

    /// Parse index from JSON string.
    pub fn from_json(json: &str) -> Result<Self, String> {
        serde_json::from_str(json).map_err(|e| e.to_string())
    }

    /// Search the index.
    pub fn search(&self, query: &SearchQuery) -> SearchResults {
        #[cfg(target_arch = "wasm32")]
        let start = js_sys::Date::now();

        if query.is_empty() {
            return SearchResults::empty(&query.raw);
        }

        // Find documents containing any query term
        let mut doc_scores: HashMap<usize, f32> = HashMap::new();

        for term in &query.terms {
            if let Some(postings) = self.index.get(term) {
                for &doc_idx in postings {
                    let doc = &self.documents[doc_idx];
                    let score = score_document(&query.terms, &doc.title, &doc.terms);
                    let entry = doc_scores.entry(doc_idx).or_insert(0.0);
                    *entry = entry.max(score);
                }
            }
        }

        // Sort by score
        let mut scored: Vec<_> = doc_scores.into_iter().collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Take top results
        let results: Vec<SearchResult> = scored
            .into_iter()
            .take(query.limit)
            .map(|(doc_idx, score)| {
                let doc = &self.documents[doc_idx];
                let snippet = doc
                    .description
                    .as_ref()
                    .and_then(|d| generate_snippet(d, &query.terms, 150));

                SearchResult {
                    url: doc.url.clone(),
                    title: doc.title.clone(),
                    description: doc.description.clone(),
                    score,
                    snippet,
                }
            })
            .collect();

        #[cfg(target_arch = "wasm32")]
        let duration_ms = (js_sys::Date::now() - start) as u32;
        #[cfg(not(target_arch = "wasm32"))]
        let duration_ms = 0u32;

        SearchResults {
            query: query.raw.clone(),
            total: results.len(),
            results,
            duration_ms,
        }
    }

    /// Get document count.
    pub fn document_count(&self) -> usize {
        self.documents.len()
    }

    /// Get term count.
    pub fn term_count(&self) -> usize {
        self.index.len()
    }
}

/// Simple search engine for WASM.
#[wasm_bindgen]
pub struct SimpleSearchEngine {
    index: SimpleSearchIndex,
}

#[wasm_bindgen]
impl SimpleSearchEngine {
    /// Load a simple search index from a URL.
    #[wasm_bindgen(js_name = load)]
    pub async fn load(index_url: &str) -> Result<SimpleSearchEngine, JsValue> {
        let response = Request::get(index_url)
            .send()
            .await
            .map_err(|e| JsValue::from_str(&format!("Network error: {e}")))?;

        if !response.ok() {
            return Err(JsValue::from_str(&format!(
                "Failed to load index: HTTP {}",
                response.status()
            )));
        }

        let json = response
            .text()
            .await
            .map_err(|e| JsValue::from_str(&format!("Failed to read response: {e}")))?;

        let index = SimpleSearchIndex::from_json(&json)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse index: {e}")))?;

        Ok(Self { index })
    }

    /// Create from a JSON string (for testing).
    #[wasm_bindgen(js_name = fromJson)]
    pub fn from_json(json: &str) -> Result<SimpleSearchEngine, JsValue> {
        let index = SimpleSearchIndex::from_json(json)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse index: {e}")))?;

        Ok(Self { index })
    }

    /// Search the index.
    pub fn search(&self, query: &str, limit: Option<usize>) -> Result<JsValue, JsValue> {
        let limit = limit.unwrap_or(10);
        let parsed_query = SearchQuery::parse(query, limit);
        let results = self.index.search(&parsed_query);
        results.to_js()
    }

    /// Get the number of indexed documents.
    #[wasm_bindgen(js_name = documentCount)]
    pub fn document_count(&self) -> usize {
        self.index.document_count()
    }

    /// Get the number of indexed terms.
    #[wasm_bindgen(js_name = termCount)]
    pub fn term_count(&self) -> usize {
        self.index.term_count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_index() -> SimpleSearchIndex {
        let documents = vec![
            SimpleDocument {
                url: "/rust".to_string(),
                title: "Learning Rust".to_string(),
                description: Some("A guide to Rust programming".to_string()),
                lang: Some("en".to_string()),
                tags: vec!["rust".to_string()],
                date: None,
                terms: vec![
                    "learning".to_string(),
                    "rust".to_string(),
                    "programming".to_string(),
                ],
            },
            SimpleDocument {
                url: "/go".to_string(),
                title: "Learning Go".to_string(),
                description: Some("A guide to Go programming".to_string()),
                lang: Some("en".to_string()),
                tags: vec!["go".to_string()],
                date: None,
                terms: vec![
                    "learning".to_string(),
                    "go".to_string(),
                    "programming".to_string(),
                ],
            },
        ];

        let mut index = HashMap::new();
        index.insert("learning".to_string(), vec![0, 1]);
        index.insert("rust".to_string(), vec![0]);
        index.insert("go".to_string(), vec![1]);
        index.insert("programming".to_string(), vec![0, 1]);

        SimpleSearchIndex {
            version: 1,
            documents,
            index,
        }
    }

    #[test]
    fn test_simple_search() {
        let index = create_test_index();
        let query = SearchQuery::parse("rust", 10);
        let results = index.search(&query);

        assert_eq!(results.total, 1);
        assert_eq!(results.results[0].url, "/rust");
    }

    #[test]
    fn test_simple_search_multiple_results() {
        let index = create_test_index();
        let query = SearchQuery::parse("programming", 10);
        let results = index.search(&query);

        assert_eq!(results.total, 2);
    }

    #[test]
    fn test_simple_search_no_results() {
        let index = create_test_index();
        let query = SearchQuery::parse("python", 10);
        let results = index.search(&query);

        assert_eq!(results.total, 0);
    }

    #[test]
    fn test_simple_search_empty_query() {
        let index = create_test_index();
        let query = SearchQuery::parse("", 10);
        let results = index.search(&query);

        assert_eq!(results.total, 0);
    }

    #[test]
    fn test_index_from_json() {
        let json = r#"{
            "version": 1,
            "documents": [{
                "url": "/test",
                "title": "Test",
                "terms": ["test"]
            }],
            "index": {"test": [0]}
        }"#;

        let index = SimpleSearchIndex::from_json(json).unwrap();
        assert_eq!(index.documents.len(), 1);
        assert_eq!(index.index.len(), 1);
    }

    #[test]
    fn test_document_and_term_count() {
        let index = create_test_index();
        assert_eq!(index.document_count(), 2);
        assert_eq!(index.term_count(), 4);
    }
}
