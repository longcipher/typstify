use std::path::PathBuf;

use typstify_ssg::{Site, SiteConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Typstify SSG Test");

    let content_dir = PathBuf::from("contents");
    let output_dir = PathBuf::from("site");

    let config = SiteConfig::new(
        "My Typstify Site",
        "A test site built with Typstify SSG",
        "https://example.com",
        "Test Author",
    );

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
