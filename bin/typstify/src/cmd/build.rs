//! Build command - generates the static site

use std::{path::Path, time::Instant};

use color_eyre::eyre::{Result, WrapErr};
use typstify_core::Config;
use typstify_generator::Builder;

use super::check::quick_validate;

/// Run the build command.
///
/// Builds the static site from content files to the output directory.
pub fn run(config_path: &Path, output: &Path, drafts: bool) -> Result<()> {
    let start = Instant::now();
    tracing::info!(?config_path, ?output, drafts, "Starting build");

    // Load configuration
    let mut config = Config::load(config_path).wrap_err("Failed to load configuration")?;

    // Quick validation - print warnings for missing language files
    let warnings = quick_validate(&config);
    if !warnings.is_empty() {
        println!();
        println!("  Warnings:");
        for warn in &warnings {
            println!("  ⚠ {warn}");
        }
        println!();
    }

    // Override output directory if specified
    config.build.output_dir = output.to_string_lossy().to_string();

    // Include drafts if flag is set
    config.build.drafts = drafts;

    tracing::debug!(?config, "Loaded configuration");

    // Create builder with content and output directories
    let content_dir = Path::new("content");
    let builder = Builder::new(config, content_dir, output);
    let stats = builder.build().wrap_err("Build failed")?;

    let duration = start.elapsed();

    // Print build statistics
    println!();
    println!("  Build completed successfully!");
    println!();
    println!("  Pages:      {}", stats.pages);
    println!("  Taxonomies: {}", stats.taxonomy_pages);
    println!("  Auto Pages: {}", stats.auto_pages);
    println!("  Redirects:  {}", stats.redirects);
    println!("  Assets:     {}", stats.assets);
    println!();
    println!("  Duration:   {:.2}s", duration.as_secs_f64());
    println!("  Output:     {}", output.display());
    println!();

    tracing::info!(?stats, ?duration, "Build completed successfully");

    Ok(())
}
