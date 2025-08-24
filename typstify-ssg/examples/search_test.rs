use std::path::PathBuf;

use eyre::Result;
use typstify_ssg::{AppConfig, Site};

fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Create a test site with search functionality
    let content_dir = PathBuf::from("contents");
    let output_dir = PathBuf::from("test_output");

    // Create site
    let mut site = Site::new(content_dir, output_dir);

    // Set up configuration
    let mut config = AppConfig::default();
    config.site.title = "Test Site with Search".to_string();
    config.site.description = "Testing Tantivy search functionality".to_string();
    site = site.with_app_config(config);

    // Scan content
    site.scan_content()?;
    println!("Found {} content files", site.content.len());

    // Initialize search engine
    site.init_search_engine()?;
    println!("Search engine initialized");

    // Test search functionality
    if !site.content.is_empty() {
        let results = site.search("rust", 5)?;
        println!("Search results for 'rust': {} matches", results.len());

        for result in results {
            println!("- {}: {:.2}", result.entry.title, result.score);
        }
    }

    // Build the site
    site.build()?;
    println!("Site built successfully with search functionality!");

    Ok(())
}
