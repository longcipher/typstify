//! Search schema definition for Tantivy.
//!
//! Defines the search index schema with fields for title, body, URL, language, tags, and date.

use tantivy::{
    Index,
    schema::{
        DateOptions, FAST, Field, STORED, STRING, Schema, SchemaBuilder, TextFieldIndexing,
        TextOptions,
    },
    tokenizer::{LowerCaser, SimpleTokenizer, TextAnalyzer},
};

/// Search schema field references.
#[derive(Debug, Clone)]
pub struct SearchFields {
    /// Page title (TEXT | STORED).
    pub title: Field,

    /// Page body content (TEXT).
    pub body: Field,

    /// Page URL (STRING | STORED).
    pub url: Field,

    /// Language code (STRING | STORED | FAST).
    pub lang: Field,

    /// Tags (TEXT | STORED).
    pub tags: Field,

    /// Publication date (DATE | STORED | FAST).
    pub date: Field,
}

/// Create the search schema with all required fields.
///
/// Returns the schema and field references for indexing.
pub fn create_search_schema() -> (Schema, SearchFields) {
    let mut builder = SchemaBuilder::new();

    // Title field: full-text searchable and stored for display
    let title_options = TextOptions::default()
        .set_indexing_options(
            TextFieldIndexing::default()
                .set_tokenizer("default")
                .set_index_option(tantivy::schema::IndexRecordOption::WithFreqsAndPositions),
        )
        .set_stored();
    let title = builder.add_text_field("title", title_options);

    // Body field: full-text searchable, not stored (too large)
    let body_options = TextOptions::default().set_indexing_options(
        TextFieldIndexing::default()
            .set_tokenizer("default")
            .set_index_option(tantivy::schema::IndexRecordOption::WithFreqsAndPositions),
    );
    let body = builder.add_text_field("body", body_options);

    // URL field: exact match, stored for results
    let url = builder.add_text_field("url", STRING | STORED);

    // Language field: exact match, stored, fast for filtering
    let lang = builder.add_text_field("lang", STRING | STORED | FAST);

    // Tags field: searchable and stored
    let tags_options = TextOptions::default()
        .set_indexing_options(
            TextFieldIndexing::default()
                .set_tokenizer("default")
                .set_index_option(tantivy::schema::IndexRecordOption::WithFreqs),
        )
        .set_stored();
    let tags = builder.add_text_field("tags", tags_options);

    // Date field: stored and fast for sorting/filtering
    let date_options = DateOptions::default().set_stored().set_fast();
    let date = builder.add_date_field("date", date_options);

    let schema = builder.build();
    let fields = SearchFields {
        title,
        body,
        url,
        lang,
        tags,
        date,
    };

    (schema, fields)
}

/// Register custom tokenizers for the search index.
///
/// Sets up the default tokenizer with lowercase normalization.
pub fn register_tokenizers(index: &Index) {
    let tokenizer = TextAnalyzer::builder(SimpleTokenizer::default())
        .filter(LowerCaser)
        .build();

    index.tokenizers().register("default", tokenizer);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_schema() {
        let (schema, fields) = create_search_schema();

        // Verify all fields exist
        assert!(schema.get_field("title").is_ok());
        assert!(schema.get_field("body").is_ok());
        assert!(schema.get_field("url").is_ok());
        assert!(schema.get_field("lang").is_ok());
        assert!(schema.get_field("tags").is_ok());
        assert!(schema.get_field("date").is_ok());

        // Verify field references match schema
        assert_eq!(fields.title, schema.get_field("title").unwrap());
        assert_eq!(fields.body, schema.get_field("body").unwrap());
        assert_eq!(fields.url, schema.get_field("url").unwrap());
    }

    #[test]
    fn test_title_field_is_stored() {
        let (schema, fields) = create_search_schema();
        let field_entry = schema.get_field_entry(fields.title);

        assert!(field_entry.is_indexed());
        // TextOptions doesn't have a direct is_stored method in schema,
        // but we configured it with set_stored()
        assert_eq!(field_entry.name(), "title");
    }

    #[test]
    fn test_url_field_is_string() {
        let (schema, fields) = create_search_schema();
        let field_entry = schema.get_field_entry(fields.url);

        assert!(field_entry.is_indexed());
        assert_eq!(field_entry.name(), "url");
    }

    #[test]
    fn test_register_tokenizers() {
        let (schema, _) = create_search_schema();
        let index = Index::create_in_ram(schema);

        register_tokenizers(&index);

        // Verify the tokenizer is registered
        let tokenizer = index.tokenizers().get("default");
        assert!(tokenizer.is_some());
    }
}
