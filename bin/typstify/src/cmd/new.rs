//! New command - create new content from template

use std::{fs, path::Path};

use chrono::Utc;
use color_eyre::eyre::{Result, WrapErr};

/// Run the new command.
///
/// Creates a new content file with boilerplate frontmatter.
pub fn run(path: &Path, template: &str) -> Result<()> {
    tracing::info!(?path, template, "Creating new content");

    let content_dir = Path::new("content");
    let full_path = content_dir.join(path);

    // Determine extension based on template
    let (ext, frontmatter) = match template {
        "typst" => ("typ", generate_typst_frontmatter(path)),
        _ => ("md", generate_markdown_frontmatter(path)),
    };

    let file_path = if full_path.extension().is_some() {
        full_path
    } else {
        full_path.with_extension(ext)
    };

    // Create parent directories
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent).wrap_err("Failed to create directories")?;
    }

    fs::write(&file_path, frontmatter).wrap_err("Failed to write file")?;

    tracing::info!(?file_path, "Created new content file");
    println!("Created: {}", file_path.display());

    Ok(())
}

fn generate_markdown_frontmatter(path: &Path) -> String {
    let title = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Untitled")
        .replace('-', " ");

    let date = Utc::now().format("%Y-%m-%d").to_string();

    format!(
        r#"---
title: "{title}"
date: {date}
draft: true
tags: []
---

Write your content here.
"#
    )
}

fn generate_typst_frontmatter(path: &Path) -> String {
    let title = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Untitled")
        .replace('-', " ");

    let date = Utc::now().format("%Y-%m-%d").to_string();

    format!(
        r#"// typstify:frontmatter
// title: "{title}"
// date: {date}
// draft: true
// tags: []

= {title}

Write your content here.
"#
    )
}
