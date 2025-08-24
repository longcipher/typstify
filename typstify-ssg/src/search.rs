use std::{
    fs,
    path::{Path, PathBuf},
};

use eyre::{Context, Result};
use serde::{Deserialize, Serialize};
use tantivy::{
    collector::TopDocs,
    doc,
    query::QueryParser,
    schema::{Field, Schema, Value, STORED, TEXT},
    Index, IndexWriter, ReloadPolicy,
};
use tracing::{debug, info};

use crate::content::Content;

/// A search entry that will be indexed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchEntry {
    pub id: String,
    pub title: String,
    pub description: String,
    pub content: String,
    pub url: String,
    pub tags: Vec<String>,
    pub category: Option<String>,
    pub date: Option<String>,
}

/// Search result with score and highlights
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub entry: SearchEntry,
    pub score: f32,
    pub snippet: String,
}

/// Tantivy-based search engine
pub struct SearchEngine {
    index: Index,
    #[allow(dead_code)]
    schema: Schema,
    title_field: Field,
    description_field: Field,
    content_field: Field,
    url_field: Field,
    tags_field: Field,
    category_field: Field,
    date_field: Field,
    id_field: Field,
    #[allow(dead_code)]
    index_dir: PathBuf,
}

impl SearchEngine {
    /// Create a new search engine with the given index directory
    pub fn new(index_dir: PathBuf) -> Result<Self> {
        // Create schema
        let mut schema_builder = Schema::builder();

        let id_field = schema_builder.add_text_field("id", STORED);
        let title_field = schema_builder.add_text_field("title", TEXT | STORED);
        let description_field = schema_builder.add_text_field("description", TEXT | STORED);
        let content_field = schema_builder.add_text_field("content", TEXT);
        let url_field = schema_builder.add_text_field("url", STORED);
        let tags_field = schema_builder.add_text_field("tags", TEXT | STORED);
        let category_field = schema_builder.add_text_field("category", TEXT | STORED);
        let date_field = schema_builder.add_text_field("date", STORED);

        let schema = schema_builder.build();

        // Create or open index
        fs::create_dir_all(&index_dir).with_context(|| {
            format!("Failed to create index directory: {}", index_dir.display())
        })?;

        let index = if index_dir.join("meta.json").exists() {
            debug!("Opening existing index at {}", index_dir.display());
            Index::open_in_dir(&index_dir).with_context(|| {
                format!("Failed to open existing index at {}", index_dir.display())
            })?
        } else {
            debug!("Creating new index at {}", index_dir.display());
            Index::create_in_dir(&index_dir, schema.clone())
                .with_context(|| format!("Failed to create index at {}", index_dir.display()))?
        };

        Ok(SearchEngine {
            index,
            schema,
            title_field,
            description_field,
            content_field,
            url_field,
            tags_field,
            category_field,
            date_field,
            id_field,
            index_dir,
        })
    }

    /// Clear and rebuild the search index from content
    pub fn rebuild_index(&self, contents: &[Content]) -> Result<()> {
        info!("Rebuilding search index with {} entries", contents.len());

        // Get index writer
        let mut index_writer = self
            .index
            .writer(50_000_000)
            .context("Failed to create index writer")?;

        // Clear existing documents
        index_writer
            .delete_all_documents()
            .context("Failed to clear existing documents")?;

        // Add all content to index
        for content in contents {
            if !content.meta().is_draft() {
                self.add_content_to_writer(&mut index_writer, content)?;
            }
        }

        // Commit changes
        index_writer
            .commit()
            .context("Failed to commit index changes")?;

        info!("Search index rebuilt successfully");
        Ok(())
    }

    /// Add a single content item to the index writer
    fn add_content_to_writer(&self, writer: &mut IndexWriter, content: &Content) -> Result<()> {
        // Render content and strip HTML
        let rendered_content = content.render().unwrap_or_default();
        let plain_content = strip_html(&rendered_content);

        // Create document
        let doc = doc!(
            self.id_field => content.slug(),
            self.title_field => content.meta().get_title(),
            self.description_field => content.meta().get_description(),
            self.content_field => plain_content,
            self.url_field => format!("/{}.html", content.slug()),
            self.tags_field => content.meta().tags.join(" "),
            self.category_field => content.meta().category.clone().unwrap_or_default(),
            self.date_field => content.meta().date.clone().unwrap_or_default(),
        );

        writer
            .add_document(doc)
            .context("Failed to add document to index")?;

        debug!("Added content to index: {}", content.slug());
        Ok(())
    }

    /// Search the index and return results
    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        if query.trim().is_empty() {
            return Ok(Vec::new());
        }

        let reader = self
            .index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommitWithDelay)
            .try_into()
            .context("Failed to create index reader")?;

        let searcher = reader.searcher();

        // Create query parser for multiple fields
        let query_parser = QueryParser::for_index(
            &self.index,
            vec![
                self.title_field,
                self.description_field,
                self.content_field,
                self.tags_field,
            ],
        );

        // Parse query
        let parsed_query = query_parser
            .parse_query(query)
            .context("Failed to parse search query")?;

        // Search
        let top_docs = searcher
            .search(&parsed_query, &TopDocs::with_limit(limit))
            .context("Failed to execute search")?;

        // Convert results
        let mut results = Vec::new();
        for (score, doc_address) in top_docs {
            let retrieved_doc = searcher
                .doc(doc_address)
                .context("Failed to retrieve document")?;

            if let Some(search_result) = self.doc_to_search_result(&retrieved_doc, score)? {
                results.push(search_result);
            }
        }

        debug!("Search for '{}' returned {} results", query, results.len());
        Ok(results)
    }

    /// Convert a Tantivy document to a SearchResult
    fn doc_to_search_result(
        &self,
        doc: &tantivy::TantivyDocument,
        score: f32,
    ) -> Result<Option<SearchResult>> {
        let id = doc
            .get_first(self.id_field)
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let title = doc
            .get_first(self.title_field)
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let description = doc
            .get_first(self.description_field)
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let content = doc
            .get_first(self.content_field)
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let url = doc
            .get_first(self.url_field)
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let tags_str = doc
            .get_first(self.tags_field)
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let tags: Vec<String> = if tags_str.is_empty() {
            Vec::new()
        } else {
            tags_str.split_whitespace().map(|s| s.to_string()).collect()
        };

        let category = doc
            .get_first(self.category_field)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .filter(|s| !s.is_empty());

        let date = doc
            .get_first(self.date_field)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .filter(|s| !s.is_empty());

        // Generate snippet from content
        let snippet = generate_snippet(&content, &description, 200);

        let entry = SearchEntry {
            id,
            title,
            description,
            content,
            url,
            tags,
            category,
            date,
        };

        Ok(Some(SearchResult {
            entry,
            score,
            snippet,
        }))
    }

    /// Export search results to JSON for client-side use
    pub fn export_search_results(&self, output_path: &Path, max_results: usize) -> Result<()> {
        // For now, we'll create a simple export of all indexed content
        let reader = self
            .index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommitWithDelay)
            .try_into()
            .context("Failed to create index reader")?;

        let searcher = reader.searcher();

        // Get all documents using a match-all query
        let query_parser = QueryParser::for_index(&self.index, vec![self.title_field]);
        let query = query_parser
            .parse_query("*")
            .or_else(|_| {
                // If wildcard doesn't work, try to get all documents by searching for common words
                query_parser.parse_query(
                    "the OR a OR to OR and OR of OR in OR is OR for OR as OR with OR that OR this",
                )
            })
            .context("Failed to parse search-all query")?;

        let top_docs = searcher
            .search(&query, &TopDocs::with_limit(max_results))
            .context("Failed to search all documents")?;

        let mut entries = Vec::new();
        for (_score, doc_address) in top_docs {
            let retrieved_doc = searcher
                .doc(doc_address)
                .context("Failed to retrieve document")?;

            if let Some(result) = self.doc_to_search_result(&retrieved_doc, 1.0)? {
                entries.push(result.entry);
            }
        }

        // Export to JSON
        let json = serde_json::to_string_pretty(&entries)
            .context("Failed to serialize search entries to JSON")?;

        std::fs::write(output_path, json).with_context(|| {
            format!("Failed to write search index to {}", output_path.display())
        })?;

        info!(
            "Exported {} search entries to {}",
            entries.len(),
            output_path.display()
        );
        Ok(())
    }
}

/// Strip HTML tags from content
fn strip_html(html: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;
    let mut chars = html.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '<' => {
                in_tag = true;
                // Add space before tags to separate words
                if !result.is_empty() && !result.ends_with(' ') {
                    result.push(' ');
                }
            }
            '>' => {
                in_tag = false;
                // Add space after tags to separate words
                if let Some(&next_ch) = chars.peek() {
                    if next_ch.is_alphanumeric() && !result.ends_with(' ') {
                        result.push(' ');
                    }
                }
            }
            _ if !in_tag => {
                result.push(ch);
            }
            _ => {} // Skip content inside tags
        }
    }

    // Clean up multiple spaces
    result.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Generate a snippet from content for search results
fn generate_snippet(content: &str, description: &str, max_length: usize) -> String {
    // Prefer description if it's substantial
    if !description.is_empty() && description.len() >= 50 {
        if description.len() <= max_length {
            return description.to_string();
        } else {
            return format!("{}...", &description[..max_length.saturating_sub(3)]);
        }
    }

    // Otherwise use content
    if content.len() <= max_length {
        content.to_string()
    } else {
        format!("{}...", &content[..max_length.saturating_sub(3)])
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    #[test]
    fn test_strip_html() {
        assert_eq!(strip_html("<p>Hello <b>world</b></p>"), "Hello world");
        assert_eq!(strip_html("Plain text"), "Plain text");
        assert_eq!(
            strip_html("<div>Test</div><span>Content</span>"),
            "Test Content"
        );
    }

    #[test]
    fn test_generate_snippet() {
        let content = "This is a long piece of content that should be truncated";
        let description = "Short desc";

        assert_eq!(generate_snippet(content, description, 100), content);
        assert_eq!(
            generate_snippet(content, description, 20),
            "This is a long pi..."
        );

        let long_desc = "This is a very long description that is longer than the content";
        assert_eq!(generate_snippet("short", long_desc, 100), long_desc);
    }

    #[test]
    fn test_search_engine_creation() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let engine = SearchEngine::new(temp_dir.path().to_path_buf())?;

        // Should create index files
        assert!(temp_dir.path().join("meta.json").exists());

        Ok(())
    }
}
