use std::path::PathBuf;

use typstify_ssg::{LegacySiteConfig, Site};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Typstify SSG Test");

    let content_dir = PathBuf::from("contents");
    let output_dir = PathBuf::from("site");

    let config = LegacySiteConfig {
        website_title: "My Typstify Site".to_string(),
        website_tagline: "A test site built with Typstify SSG".to_string(),
        base_url: "https://example.com".to_string(),
        author: "Test Author".to_string(),
    };

    let mut site = Site::new(content_dir, output_dir).with_config(config);

    // Scan for content
    site.scan_content()?;

    println!("Found {} content files", site.content.len());
    for content in &site.content {
        println!("- {} ({})", content.title(), content.slug());
    }

    // Build the site
    site.build()?;

    println!("Site built successfully!");
    Ok(())
}
