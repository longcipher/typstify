//! End-to-end tests for Typstify.
//!
//! These tests exercise bundled test fixtures and verify core functionality.

use std::{fs, path::Path};

use typstify_core::Config;
use typstify_parser::ParserRegistry;

fn testdata_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata")
}

#[test]
fn test_sample_site_config_loads() {
    let config_path = testdata_dir().join("config.toml");
    let config = Config::load(&config_path).expect("Config should load");
    assert_eq!(config.site.title, "My Typstify Blog");
    assert_eq!(config.site.host, "https://longcipher.github.io");
    assert_eq!(config.site.base_path, "/typstify");
    assert_eq!(config.site.default_language, "en");
    assert!(config.has_language("en"));
    assert!(config.has_language("zh"));
}

#[test]
fn test_sample_site_content_parses() {
    let registry = ParserRegistry::new();

    let md_path = testdata_dir().join("content/posts/hello-world.md");
    let content = fs::read_to_string(&md_path).expect("Failed to read");
    let parsed = registry.parse(&content, &md_path).expect("Should parse");
    assert_eq!(parsed.frontmatter.title, "Hello, World!");
    assert!(!parsed.frontmatter.draft);
}

#[test]
fn test_sample_site_typst_parses() {
    let registry = ParserRegistry::new();

    let typ_path = testdata_dir().join("content/docs/technical-spec.typ");
    let content = fs::read_to_string(&typ_path).expect("Failed to read");
    let parsed = registry.parse(&content, &typ_path).expect("Should parse");
    assert_eq!(parsed.frontmatter.title, "Technical Specification");
}

#[test]
fn test_sample_site_chinese_content() {
    let registry = ParserRegistry::new();

    let zh_path = testdata_dir().join("content/posts/hello-world.zh.md");
    let content = fs::read_to_string(&zh_path).expect("Failed to read");
    let parsed = registry.parse(&content, &zh_path).expect("Should parse");

    assert_eq!(parsed.frontmatter.title, "你好，世界！");
    assert!(parsed.html.contains("Typstify"));
}

#[test]
fn test_sample_site_about_page() {
    let registry = ParserRegistry::new();

    let about_path = testdata_dir().join("content/about.md");
    let content = fs::read_to_string(&about_path).expect("Failed to read");
    let parsed = registry.parse(&content, &about_path).expect("Should parse");

    assert_eq!(parsed.frontmatter.title, "About This Site");
    assert!(!parsed.frontmatter.draft);
}

#[test]
fn test_multiple_posts_parse() {
    let registry = ParserRegistry::new();
    let posts_dir = testdata_dir().join("content/posts");

    let mut parsed_count = 0;
    for entry in fs::read_dir(&posts_dir).expect("Should read dir") {
        let entry = entry.expect("Should get entry");
        let path = entry.path();

        if path.extension().is_some_and(|ext| ext == "md") {
            let content = fs::read_to_string(&path).expect("Should read");
            let parsed = registry.parse(&content, &path).expect("Should parse");

            // All posts should have titles
            assert!(!parsed.frontmatter.title.is_empty());
            parsed_count += 1;
        }
    }

    // Should have parsed at least 3 posts
    assert!(parsed_count >= 3, "Should have at least 3 posts");
}

#[test]
fn test_config_sections() {
    let config_path = testdata_dir().join("config.toml");
    let config = Config::load(&config_path).expect("Config should load");

    // Test site section
    assert_eq!(config.site.title, "My Typstify Blog");
    assert!(config.site.description.is_some());
    assert!(config.site.author.is_some());

    // Test build section
    assert_eq!(config.build.output_dir, "public");
    assert!(!config.build.minify);

    // Test search section
    assert!(config.search.enabled);

    // Test rss section
    assert!(config.rss.enabled);
    assert_eq!(config.rss.limit, 20);
}
