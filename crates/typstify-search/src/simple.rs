//! Simple JSON-based search index for small sites.
//!
//! Provides a lightweight alternative to Tantivy for sites with fewer pages.
//! The entire index is loaded into memory in the browser.

use std::{collections::HashMap, fs, path::Path};

use serde::{Deserialize, Serialize};
use tracing::info;
use typstify_core::Page;

use crate::SearchError;

/// Maximum recommended size for simple index (500KB).
pub const MAX_SIMPLE_INDEX_SIZE: usize = 500 * 1024;

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
    /// Create a new empty index.
    pub fn new() -> Self {
        Self {
            version: 1,
            documents: Vec::new(),
            index: HashMap::new(),
        }
    }

    /// Build an index from a collection of pages.
    pub fn from_pages(pages: &[&Page]) -> Self {
        let mut index = Self::new();

        for page in pages {
            index.add_page(page);
        }

        index.build_inverted_index();
        index
    }

    /// Add a page to the index.
    pub fn add_page(&mut self, page: &Page) {
        let terms = tokenize_content(&page.title, &page.content, &page.tags);

        let doc = SimpleDocument {
            url: page.url.clone(),
            title: page.title.clone(),
            description: page.description.clone().or(page.summary.clone()),
            lang: Some(page.lang.clone()),
            tags: page.tags.clone(),
            date: page.date.map(|d| d.to_rfc3339()),
            terms,
        };

        self.documents.push(doc);
    }

    /// Build the inverted index from documents.
    fn build_inverted_index(&mut self) {
        self.index.clear();

        for (doc_idx, doc) in self.documents.iter().enumerate() {
            for term in &doc.terms {
                self.index.entry(term.clone()).or_default().push(doc_idx);
            }
        }

        // Deduplicate posting lists
        for postings in self.index.values_mut() {
            postings.sort_unstable();
            postings.dedup();
        }

        info!(
            documents = self.documents.len(),
            terms = self.index.len(),
            "Built simple search index"
        );
    }

    /// Search the index for matching documents.
    ///
    /// Returns documents matching all query terms (AND search).
    pub fn search(&self, query: &str) -> Vec<&SimpleDocument> {
        let query_terms = tokenize_query(query);

        if query_terms.is_empty() {
            return Vec::new();
        }

        // Find documents containing all query terms
        let mut result_indices: Option<Vec<usize>> = None;

        for term in &query_terms {
            if let Some(postings) = self.index.get(term) {
                match &mut result_indices {
                    None => {
                        result_indices = Some(postings.clone());
                    }
                    Some(indices) => {
                        // Intersect with existing results
                        indices.retain(|idx| postings.contains(idx));
                    }
                }
            } else {
                // Term not found, no results
                return Vec::new();
            }
        }

        result_indices
            .unwrap_or_default()
            .iter()
            .filter_map(|&idx| self.documents.get(idx))
            .collect()
    }

    /// Serialize the index to JSON.
    pub fn to_json(&self) -> Result<String, SearchError> {
        serde_json::to_string(self).map_err(|e| SearchError::Serialization(e.to_string()))
    }

    /// Serialize the index to pretty-printed JSON.
    pub fn to_json_pretty(&self) -> Result<String, SearchError> {
        serde_json::to_string_pretty(self).map_err(|e| SearchError::Serialization(e.to_string()))
    }

    /// Deserialize an index from JSON.
    pub fn from_json(json: &str) -> Result<Self, SearchError> {
        serde_json::from_str(json).map_err(|e| SearchError::Serialization(e.to_string()))
    }

    /// Write the index to a file.
    pub fn write_to_file(&self, path: &Path) -> Result<(), SearchError> {
        let json = self.to_json()?;

        // Warn if index is too large
        if json.len() > MAX_SIMPLE_INDEX_SIZE {
            tracing::warn!(
                size = json.len(),
                max = MAX_SIMPLE_INDEX_SIZE,
                "Simple search index exceeds recommended size"
            );
        }

        fs::write(path, json).map_err(|e| SearchError::Io(e.to_string()))?;
        Ok(())
    }

    /// Get the estimated size of the serialized index.
    pub fn estimated_size(&self) -> usize {
        // Rough estimate: JSON overhead + document data
        self.documents
            .iter()
            .map(|d| {
                d.url.len()
                    + d.title.len()
                    + d.description.as_ref().map(|s| s.len()).unwrap_or(0)
                    + d.terms.iter().map(|t| t.len() + 3).sum::<usize>()
                    + 100 // JSON overhead
            })
            .sum()
    }

    /// Check if the index is within the recommended size limit.
    pub fn is_within_size_limit(&self) -> bool {
        self.estimated_size() <= MAX_SIMPLE_INDEX_SIZE
    }
}

impl Default for SimpleSearchIndex {
    fn default() -> Self {
        Self::new()
    }
}

/// Tokenize content for indexing.
///
/// Extracts terms from title, body content, and tags.
fn tokenize_content(title: &str, content: &str, tags: &[String]) -> Vec<String> {
    let mut terms = Vec::new();

    // Tokenize title (higher weight, keep as-is)
    for term in tokenize_text(title) {
        terms.push(term);
    }

    // Tokenize body content
    let body_text = strip_html(content);
    for term in tokenize_text(&body_text) {
        terms.push(term);
    }

    // Add tags
    for tag in tags {
        terms.push(normalize_term(tag));
    }

    // Deduplicate
    terms.sort();
    terms.dedup();

    terms
}

/// Tokenize a query string.
fn tokenize_query(query: &str) -> Vec<String> {
    tokenize_text(query)
}

/// Tokenize text into normalized terms.
/// Supports both space-separated languages (English) and CJK languages (Chinese, Japanese, Korean).
fn tokenize_text(text: &str) -> Vec<String> {
    let mut terms = Vec::new();

    // First, extract word-based terms (for English and other space-separated languages)
    for word in text.split(|c: char| !c.is_alphanumeric()) {
        if word.len() >= 2 {
            terms.push(normalize_term(word));
        }
    }

    // Then, extract CJK characters (Chinese, Japanese, Korean)
    // CJK characters are meaningful individually or in small groups
    let cjk_text: String = text.chars().filter(|c| is_cjk_char(*c)).collect();

    if !cjk_text.is_empty() {
        // Add individual CJK characters
        for c in cjk_text.chars() {
            terms.push(c.to_string());
        }

        // Add bigrams (2-character combinations) for better matching
        let chars: Vec<char> = cjk_text.chars().collect();
        for i in 0..chars.len().saturating_sub(1) {
            terms.push(format!("{}{}", chars[i], chars[i + 1]));
        }

        // Add the full CJK text if it's short enough to be meaningful
        if cjk_text.len() <= 20 && cjk_text.chars().count() >= 2 {
            terms.push(cjk_text.to_lowercase());
        }
    }

    terms
}

/// Check if a character is a CJK (Chinese, Japanese, Korean) character.
fn is_cjk_char(c: char) -> bool {
    matches!(c,
        '\u{4E00}'..='\u{9FFF}' |      // CJK Unified Ideographs
        '\u{3400}'..='\u{4DBF}' |      // CJK Unified Ideographs Extension A
        '\u{20000}'..='\u{2A6DF}' |    // CJK Unified Ideographs Extension B
        '\u{2A700}'..='\u{2B73F}' |    // CJK Unified Ideographs Extension C
        '\u{2B740}'..='\u{2B81F}' |    // CJK Unified Ideographs Extension D
        '\u{2B820}'..='\u{2CEAF}' |    // CJK Unified Ideographs Extension E
        '\u{2CEB0}'..='\u{2EBEF}' |    // CJK Unified Ideographs Extension F
        '\u{30000}'..='\u{3134F}' |    // CJK Unified Ideographs Extension G
        '\u{F900}'..='\u{FAFF}' |      // CJK Compatibility Ideographs
        '\u{2F800}'..='\u{2FA1F}' |    // CJK Compatibility Ideographs Supplement
        '\u{3040}'..='\u{309F}' |      // Hiragana
        '\u{30A0}'..='\u{30FF}' |      // Katakana
        '\u{AC00}'..='\u{D7AF}'        // Korean Hangul Syllables
    )
}

/// Normalize a term (lowercase, trim).
fn normalize_term(term: &str) -> String {
    term.to_lowercase().trim().to_string()
}

/// Strip HTML tags from content.
fn strip_html(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut in_tag = false;

    for c in html.chars() {
        if c == '<' {
            in_tag = true;
        } else if c == '>' {
            in_tag = false;
            result.push(' ');
        } else if !in_tag {
            result.push(c);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;

    fn create_test_page(url: &str, title: &str, content: &str, tags: Vec<String>) -> Page {
        Page {
            url: url.to_string(),
            title: title.to_string(),
            description: Some(format!("Description of {}", title)),
            date: Some(Utc::now()),
            updated: None,
            draft: false,
            lang: "en".to_string(),
            is_default_lang: true,
            canonical_id: url.trim_start_matches('/').to_string(),
            tags,
            categories: vec![],
            content: content.to_string(),
            summary: None,
            reading_time: Some(5),
            word_count: Some(100),
            source_path: None,
            aliases: vec![],
            toc: vec![],
            custom_js: vec![],
            custom_css: vec![],
            template: None,
            weight: 0,
        }
    }

    #[test]
    fn test_tokenize_text() {
        let terms = tokenize_text("Hello World! This is a test.");
        assert!(terms.contains(&"hello".to_string()));
        assert!(terms.contains(&"world".to_string()));
        assert!(terms.contains(&"test".to_string()));
        // Single character "a" should be filtered out
        assert!(!terms.contains(&"a".to_string()));
    }

    #[test]
    fn test_tokenize_chinese() {
        let terms = tokenize_text("你好世界");
        // Should contain individual characters
        assert!(terms.contains(&"你".to_string()));
        assert!(terms.contains(&"好".to_string()));
        assert!(terms.contains(&"世".to_string()));
        assert!(terms.contains(&"界".to_string()));
        // Should contain bigrams
        assert!(terms.contains(&"你好".to_string()));
        assert!(terms.contains(&"世界".to_string()));
    }

    #[test]
    fn test_is_cjk_char() {
        // Chinese
        assert!(is_cjk_char('你'));
        assert!(is_cjk_char('好'));
        // Japanese
        assert!(is_cjk_char('あ')); // Hiragana
        assert!(is_cjk_char('ア')); // Katakana
        // Korean
        assert!(is_cjk_char('한')); // Hangul
        // Not CJK
        assert!(!is_cjk_char('a'));
        assert!(!is_cjk_char('1'));
    }

    #[test]
    fn test_strip_html() {
        let html = "<p>Hello <strong>world</strong>!</p>";
        let text = strip_html(html);
        assert!(text.contains("Hello"));
        assert!(text.contains("world"));
        assert!(!text.contains("<p>"));
    }

    #[test]
    fn test_simple_index_from_pages() {
        let page1 = create_test_page(
            "/post1",
            "Introduction to Rust",
            "<p>Rust is a systems programming language.</p>",
            vec!["rust".to_string(), "programming".to_string()],
        );
        let page2 = create_test_page(
            "/post2",
            "Learning Go",
            "<p>Go is a great language for servers.</p>",
            vec!["go".to_string(), "programming".to_string()],
        );

        let index = SimpleSearchIndex::from_pages(&[&page1, &page2]);

        assert_eq!(index.documents.len(), 2);
        assert!(!index.index.is_empty());

        // Check term indexing
        assert!(index.index.contains_key("rust"));
        assert!(index.index.contains_key("programming"));
    }

    #[test]
    fn test_simple_index_search() {
        let page1 = create_test_page(
            "/rust",
            "Learning Rust",
            "<p>Rust programming tutorial.</p>",
            vec!["rust".to_string()],
        );
        let page2 = create_test_page(
            "/go",
            "Learning Go",
            "<p>Go programming tutorial.</p>",
            vec!["go".to_string()],
        );

        let index = SimpleSearchIndex::from_pages(&[&page1, &page2]);

        // Search for Rust
        let results = index.search("rust");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].url, "/rust");

        // Search for programming (should match both)
        let results = index.search("programming");
        assert_eq!(results.len(), 2);

        // Search for non-existent term
        let results = index.search("python");
        assert!(results.is_empty());
    }

    #[test]
    fn test_simple_index_serialization() {
        let page = create_test_page(
            "/test",
            "Test Page",
            "<p>Test content</p>",
            vec!["test".to_string()],
        );

        let index = SimpleSearchIndex::from_pages(&[&page]);
        let json = index.to_json().unwrap();
        let parsed = SimpleSearchIndex::from_json(&json).unwrap();

        assert_eq!(parsed.documents.len(), 1);
        assert_eq!(parsed.documents[0].url, "/test");
    }

    #[test]
    fn test_simple_index_multi_term_search() {
        let page1 = create_test_page(
            "/post1",
            "Rust Programming Guide",
            "<p>Learn systems programming with Rust.</p>",
            vec!["rust".to_string()],
        );
        let page2 = create_test_page(
            "/post2",
            "Python Programming",
            "<p>Learn scripting with Python.</p>",
            vec!["python".to_string()],
        );

        let index = SimpleSearchIndex::from_pages(&[&page1, &page2]);

        // Search for "rust programming" should only match post1
        let results = index.search("rust systems");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].url, "/post1");
    }

    #[test]
    fn test_estimated_size() {
        let page = create_test_page(
            "/test",
            "Test Page",
            "<p>Test content</p>",
            vec!["test".to_string()],
        );

        let index = SimpleSearchIndex::from_pages(&[&page]);
        let estimated = index.estimated_size();

        // Should have some reasonable size
        assert!(estimated > 0);
        assert!(estimated < MAX_SIMPLE_INDEX_SIZE);
        assert!(index.is_within_size_limit());
    }
}
