//! Query parsing and search execution.
//!
//! Provides query parsing and search functionality for the WASM runtime.

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

/// A search query with parsed terms.
#[derive(Debug, Clone)]
pub struct SearchQuery {
    /// Raw query string.
    pub raw: String,

    /// Parsed and normalized terms.
    pub terms: Vec<String>,

    /// Maximum number of results.
    pub limit: usize,
}

impl SearchQuery {
    /// Parse a query string.
    pub fn parse(query: &str, limit: usize) -> Self {
        let terms = tokenize_query(query);

        Self {
            raw: query.to_string(),
            terms,
            limit,
        }
    }

    /// Check if the query is empty.
    pub fn is_empty(&self) -> bool {
        self.terms.is_empty()
    }
}

/// A single search result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Document URL.
    pub url: String,

    /// Document title.
    pub title: String,

    /// Document summary/description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Relevance score (higher is better).
    pub score: f32,

    /// Highlighted snippet showing matches.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snippet: Option<String>,
}

/// Search results container.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResults {
    /// Query that was executed.
    pub query: String,

    /// Total number of matches.
    pub total: usize,

    /// Result items.
    pub results: Vec<SearchResult>,

    /// Search duration in milliseconds.
    pub duration_ms: u32,
}

impl SearchResults {
    /// Create empty results.
    pub fn empty(query: &str) -> Self {
        Self {
            query: query.to_string(),
            total: 0,
            results: Vec::new(),
            duration_ms: 0,
        }
    }

    /// Convert to JavaScript value.
    pub fn to_js(&self) -> Result<JsValue, JsValue> {
        serde_wasm_bindgen::to_value(self).map_err(|e| JsValue::from_str(&e.to_string()))
    }
}

/// Tokenize a query string into normalized terms.
fn tokenize_query(query: &str) -> Vec<String> {
    query
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| s.len() >= 2) // Skip single characters
        .map(|s| s.to_lowercase())
        .collect()
}

/// Score a document against a query.
///
/// Returns a relevance score based on term frequency and position.
pub fn score_document(query_terms: &[String], title: &str, body_terms: &[String]) -> f32 {
    let title_lower = title.to_lowercase();
    let title_terms: Vec<String> = tokenize_query(&title_lower);

    let mut score = 0.0f32;

    for query_term in query_terms {
        // Title matches are worth more
        for title_term in &title_terms {
            if title_term.contains(query_term) {
                score += 10.0;
            }
            if title_term == query_term {
                score += 5.0; // Exact match bonus
            }
        }

        // Body matches
        for body_term in body_terms {
            if body_term == query_term {
                score += 1.0;
            } else if body_term.contains(query_term) {
                score += 0.5;
            }
        }
    }

    score
}

/// Generate a highlighted snippet for a result.
pub fn generate_snippet(text: &str, query_terms: &[String], max_length: usize) -> Option<String> {
    if text.is_empty() || query_terms.is_empty() {
        return None;
    }

    let text_lower = text.to_lowercase();

    // Find the first occurrence of any query term
    let mut best_pos = None;
    for term in query_terms {
        if let Some(pos) = text_lower.find(term) {
            match best_pos {
                None => best_pos = Some(pos),
                Some(current) if pos < current => best_pos = Some(pos),
                _ => {}
            }
        }
    }

    let start_pos = best_pos.unwrap_or(0);

    // Calculate snippet window
    let snippet_start = if start_pos > 50 {
        // Find word boundary
        text[..start_pos]
            .rfind(char::is_whitespace)
            .map(|p| p + 1)
            .unwrap_or(start_pos.saturating_sub(50))
    } else {
        0
    };

    let snippet_end = (snippet_start + max_length).min(text.len());
    let snippet_end = text[..snippet_end]
        .rfind(char::is_whitespace)
        .unwrap_or(snippet_end);

    let mut snippet = text[snippet_start..snippet_end].to_string();

    // Add ellipsis if needed
    if snippet_start > 0 {
        snippet = format!("...{}", snippet.trim_start());
    }
    if snippet_end < text.len() {
        snippet = format!("{}...", snippet.trim_end());
    }

    Some(snippet)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_query() {
        let query = SearchQuery::parse("hello world", 10);
        assert_eq!(query.terms, vec!["hello", "world"]);
        assert_eq!(query.limit, 10);
    }

    #[test]
    fn test_parse_query_filters_short() {
        let query = SearchQuery::parse("a test b query c", 10);
        // Single letters should be filtered out
        assert_eq!(query.terms, vec!["test", "query"]);
    }

    #[test]
    fn test_empty_query() {
        let query = SearchQuery::parse("", 10);
        assert!(query.is_empty());

        let query = SearchQuery::parse("a b c", 10);
        assert!(query.is_empty()); // All single chars
    }

    #[test]
    fn test_score_document() {
        let query_terms = vec!["rust".to_string()];
        let body_terms = vec!["rust".to_string(), "programming".to_string()];

        // Title match should score higher
        let score_with_title = score_document(&query_terms, "Learning Rust", &body_terms);
        let score_without_title = score_document(&query_terms, "Programming Guide", &body_terms);

        assert!(score_with_title > score_without_title);
    }

    #[test]
    fn test_generate_snippet() {
        let text = "Rust is a systems programming language. It provides memory safety without garbage collection.";
        let terms = vec!["rust".to_string()];

        let snippet = generate_snippet(text, &terms, 50);
        assert!(snippet.is_some());
        assert!(snippet.unwrap().to_lowercase().contains("rust"));
    }

    #[test]
    fn test_search_results_empty() {
        let results = SearchResults::empty("test");
        assert_eq!(results.total, 0);
        assert!(results.results.is_empty());
    }

    #[test]
    fn test_search_result_serialization() {
        let result = SearchResult {
            url: "/test".to_string(),
            title: "Test Page".to_string(),
            description: Some("A test page".to_string()),
            score: 10.5,
            snippet: None,
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("Test Page"));
        assert!(json.contains("10.5"));
    }
}
