//! Search indexer for building Tantivy indexes.
//!
//! Provides functionality to index pages and build optimized search indexes.

use std::path::Path;

use tantivy::{
    DateTime as TantivyDateTime, Index, IndexWriter, TantivyDocument, directory::MmapDirectory,
};
use tracing::{debug, info};
use typstify_core::Page;

use crate::{
    SearchError,
    schema::{SearchFields, create_search_schema, register_tokenizers},
};

/// Configuration for the search indexer.
#[derive(Debug, Clone)]
pub struct IndexerConfig {
    /// Memory budget for the index writer (in bytes).
    /// Default: 50MB.
    pub memory_budget: usize,

    /// Default language for pages without explicit language.
    pub default_lang: String,
}

impl Default for IndexerConfig {
    fn default() -> Self {
        Self {
            memory_budget: 50_000_000, // 50MB
            default_lang: "en".to_string(),
        }
    }
}

/// Statistics about the built index.
#[derive(Debug, Clone, Default)]
pub struct IndexStats {
    /// Number of documents indexed.
    pub document_count: usize,

    /// Number of segments in the index.
    pub segment_count: usize,

    /// Total index size in bytes.
    pub size_bytes: u64,
}

/// Search indexer for building Tantivy indexes from pages.
#[derive(Debug)]
pub struct SearchIndexer {
    config: IndexerConfig,
    index: Index,
    fields: SearchFields,
}

impl SearchIndexer {
    /// Create a new indexer with index stored at the given path.
    ///
    /// Creates the directory if it doesn't exist.
    pub fn new(index_path: &Path, config: IndexerConfig) -> Result<Self, SearchError> {
        // Create directory if needed
        std::fs::create_dir_all(index_path).map_err(|e| SearchError::Io(e.to_string()))?;

        let (schema, fields) = create_search_schema();
        let directory =
            MmapDirectory::open(index_path).map_err(|e| SearchError::Index(e.to_string()))?;
        let index = Index::open_or_create(directory, schema)
            .map_err(|e| SearchError::Index(e.to_string()))?;

        register_tokenizers(&index);

        Ok(Self {
            config,
            index,
            fields,
        })
    }

    /// Create a new indexer with an in-memory index (for testing).
    pub fn new_in_memory(config: IndexerConfig) -> Result<Self, SearchError> {
        let (schema, fields) = create_search_schema();
        let index = Index::create_in_ram(schema);

        register_tokenizers(&index);

        Ok(Self {
            config,
            index,
            fields,
        })
    }

    /// Index a collection of pages.
    ///
    /// Returns the number of documents indexed.
    pub fn index_pages(&self, pages: &[&Page]) -> Result<usize, SearchError> {
        let mut writer = self
            .index
            .writer(self.config.memory_budget)
            .map_err(|e| SearchError::Index(e.to_string()))?;

        let mut count = 0;
        for page in pages {
            self.index_page(&mut writer, page)?;
            count += 1;
        }

        writer
            .commit()
            .map_err(|e| SearchError::Index(e.to_string()))?;

        info!(count, "Indexed pages");
        Ok(count)
    }

    /// Index a single page.
    fn index_page(&self, writer: &mut IndexWriter, page: &Page) -> Result<(), SearchError> {
        let mut doc = TantivyDocument::new();

        // Add title
        doc.add_text(self.fields.title, &page.title);

        // Add body (strip HTML tags)
        let body_text = strip_html_tags(&page.content);
        doc.add_text(self.fields.body, &body_text);

        // Add URL
        doc.add_text(self.fields.url, &page.url);

        // Add language
        let lang = page.lang.as_deref().unwrap_or(&self.config.default_lang);
        doc.add_text(self.fields.lang, lang);

        // Add tags
        let tags_text = page.tags.join(" ");
        doc.add_text(self.fields.tags, &tags_text);

        // Add date if present
        if let Some(date) = page.date {
            let tantivy_date = TantivyDateTime::from_timestamp_secs(date.timestamp());
            doc.add_date(self.fields.date, tantivy_date);
        }

        writer
            .add_document(doc)
            .map_err(|e| SearchError::Index(e.to_string()))?;

        debug!(url = %page.url, title = %page.title, "Indexed page");
        Ok(())
    }

    /// Optimize the index by merging segments.
    ///
    /// This should be called after all documents are indexed.
    pub fn optimize(&self) -> Result<(), SearchError> {
        let writer: IndexWriter<TantivyDocument> = self
            .index
            .writer(self.config.memory_budget)
            .map_err(|e| SearchError::Index(e.to_string()))?;

        // Wait for merging threads to complete
        writer
            .wait_merging_threads()
            .map_err(|e| SearchError::Index(e.to_string()))?;

        info!("Index optimization complete");
        Ok(())
    }

    /// Get statistics about the index.
    pub fn stats(&self) -> Result<IndexStats, SearchError> {
        let reader = self
            .index
            .reader()
            .map_err(|e| SearchError::Index(e.to_string()))?;

        let searcher = reader.searcher();
        let segment_count = searcher.segment_readers().len();
        let document_count = searcher.num_docs() as usize;

        // Estimate size from segment readers
        let size_bytes = searcher
            .segment_readers()
            .iter()
            .map(|r| r.num_docs() as u64 * 500) // Rough estimate: 500 bytes per doc
            .sum();

        Ok(IndexStats {
            document_count,
            segment_count,
            size_bytes,
        })
    }

    /// Get a reference to the underlying Tantivy index.
    pub fn index(&self) -> &Index {
        &self.index
    }

    /// Get a reference to the search fields.
    pub fn fields(&self) -> &SearchFields {
        &self.fields
    }
}

/// Strip HTML tags from content to get plain text.
///
/// This is a simple implementation that handles common cases.
fn strip_html_tags(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut in_tag = false;
    let mut in_script = false;
    let mut in_style = false;

    let html_lower = html.to_lowercase();
    let chars: Vec<char> = html.chars().collect();
    let chars_lower: Vec<char> = html_lower.chars().collect();

    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];

        // Check for script/style start
        if i + 7 < chars.len() {
            let next_7: String = chars_lower[i..i + 7].iter().collect();
            if next_7 == "<script" {
                in_script = true;
            } else if next_7 == "</scrip" {
                in_script = false;
            }
        }

        if i + 6 < chars.len() {
            let next_6: String = chars_lower[i..i + 6].iter().collect();
            if next_6 == "<style" {
                in_style = true;
            } else if next_6 == "</styl" {
                in_style = false;
            }
        }

        if c == '<' {
            in_tag = true;
        } else if c == '>' {
            in_tag = false;
        } else if !in_tag && !in_script && !in_style {
            result.push(c);
        }

        i += 1;
    }

    // Decode common HTML entities
    result = result
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'");

    // Collapse multiple whitespace
    let mut collapsed = String::with_capacity(result.len());
    let mut prev_space = false;
    for c in result.chars() {
        if c.is_whitespace() {
            if !prev_space {
                collapsed.push(' ');
                prev_space = true;
            }
        } else {
            collapsed.push(c);
            prev_space = false;
        }
    }

    collapsed.trim().to_string()
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;

    fn create_test_page(url: &str, title: &str, content: &str) -> Page {
        Page {
            url: url.to_string(),
            title: title.to_string(),
            description: None,
            date: Some(Utc::now()),
            updated: None,
            draft: false,
            lang: Some("en".to_string()),
            tags: vec!["rust".to_string(), "search".to_string()],
            categories: vec![],
            content: content.to_string(),
            summary: None,
            reading_time: Some(5),
            word_count: Some(500),
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
    fn test_strip_html_tags() {
        let html = "<p>Hello <strong>world</strong>!</p>";
        let text = strip_html_tags(html);
        assert_eq!(text, "Hello world!");
    }

    #[test]
    fn test_strip_html_with_script() {
        let html = "<p>Before</p><script>alert('hi');</script><p>After</p>";
        let text = strip_html_tags(html);
        // Script content is removed, "Before" and "After" end up adjacent
        // The important thing is script content is not included
        assert!(text.contains("Before"));
        assert!(text.contains("After"));
        assert!(!text.contains("alert"));
    }

    #[test]
    fn test_strip_html_entities() {
        let html = "<p>Hello &amp; goodbye &lt;world&gt;</p>";
        let text = strip_html_tags(html);
        assert_eq!(text, "Hello & goodbye <world>");
    }

    #[test]
    fn test_index_single_page() {
        let indexer = SearchIndexer::new_in_memory(IndexerConfig::default()).unwrap();
        let page = create_test_page("/test", "Test Page", "<p>Test content</p>");

        let count = indexer.index_pages(&[&page]).unwrap();
        assert_eq!(count, 1);

        let stats = indexer.stats().unwrap();
        assert_eq!(stats.document_count, 1);
    }

    #[test]
    fn test_index_multiple_pages() {
        let indexer = SearchIndexer::new_in_memory(IndexerConfig::default()).unwrap();
        let page1 = create_test_page("/page1", "Page One", "<p>First page</p>");
        let page2 = create_test_page("/page2", "Page Two", "<p>Second page</p>");
        let page3 = create_test_page("/page3", "Page Three", "<p>Third page</p>");

        let count = indexer.index_pages(&[&page1, &page2, &page3]).unwrap();
        assert_eq!(count, 3);

        let stats = indexer.stats().unwrap();
        assert_eq!(stats.document_count, 3);
    }

    #[test]
    fn test_indexer_config_default() {
        let config = IndexerConfig::default();
        assert_eq!(config.memory_budget, 50_000_000);
        assert_eq!(config.default_lang, "en");
    }
}
