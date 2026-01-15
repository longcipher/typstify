//! Content collection and organization.
//!
//! Walks the content directory and collects all pages into a structured hierarchy.

use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use rayon::prelude::*;
use thiserror::Error;
use tracing::{debug, info, warn};
use typstify_core::{Config, ContentPath, ContentType, Page};
use typstify_parser::ParserRegistry;

/// Content collection errors.
#[derive(Debug, Error)]
pub enum CollectorError {
    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Parser error.
    #[error("parse error in {path}: {message}")]
    Parse { path: PathBuf, message: String },

    /// Invalid content path.
    #[error("invalid content path: {0}")]
    InvalidPath(PathBuf),
}

/// Result type for collector operations.
pub type Result<T> = std::result::Result<T, CollectorError>;

/// Collected site content.
#[derive(Debug, Default)]
pub struct SiteContent {
    /// All pages indexed by slug.
    pub pages: HashMap<String, Page>,

    /// Pages organized by section (first path component).
    pub sections: HashMap<String, Vec<String>>,

    /// Taxonomy term to page slugs mapping.
    pub taxonomies: TaxonomyIndex,

    /// Translation groups (canonical_id -> [slugs]).
    pub translations: HashMap<String, Vec<String>>,
}

/// Index of taxonomy terms.
#[derive(Debug, Default)]
pub struct TaxonomyIndex {
    /// Tag -> page slugs.
    pub tags: HashMap<String, Vec<String>>,

    /// Category -> page slugs.
    pub categories: HashMap<String, Vec<String>>,
}

/// Content collector that walks directories and parses files.
#[derive(Debug)]
pub struct ContentCollector {
    config: Config,
    parser: ParserRegistry,
    content_dir: PathBuf,
}

impl ContentCollector {
    /// Create a new content collector.
    #[must_use]
    pub fn new(config: Config, content_dir: impl Into<PathBuf>) -> Self {
        Self {
            config,
            parser: ParserRegistry::new(),
            content_dir: content_dir.into(),
        }
    }

    /// Collect all content from the content directory.
    pub fn collect(&self) -> Result<SiteContent> {
        info!(dir = %self.content_dir.display(), "collecting content");

        // Find all content files
        let files = self.find_content_files()?;
        info!(count = files.len(), "found content files");

        // Parse files in parallel
        let pages: Vec<_> = files
            .par_iter()
            .filter_map(|path| {
                match self.parse_file(path) {
                    Ok(page) => {
                        // Filter drafts unless configured to include them
                        if page.draft && !self.config.build.drafts {
                            debug!(url = %page.url, "skipping draft");
                            None
                        } else {
                            Some(page)
                        }
                    }
                    Err(e) => {
                        warn!(path = %path.display(), error = %e, "failed to parse file");
                        None
                    }
                }
            })
            .collect();

        // Build site content structure
        let mut content = SiteContent::default();

        for page in pages {
            let url = page.url.clone();
            let slug = url.trim_start_matches('/').to_string();

            // Add to sections
            let section = slug.split('/').next().unwrap_or("").to_string();
            if !section.is_empty() {
                content
                    .sections
                    .entry(section)
                    .or_default()
                    .push(url.clone());
            }

            // Index taxonomies
            for tag in &page.tags {
                content
                    .taxonomies
                    .tags
                    .entry(tag.clone())
                    .or_default()
                    .push(url.clone());
            }
            for category in &page.categories {
                content
                    .taxonomies
                    .categories
                    .entry(category.clone())
                    .or_default()
                    .push(url.clone());
            }

            // Index translations
            if !page.canonical_id.is_empty() {
                content
                    .translations
                    .entry(page.canonical_id.clone())
                    .or_default()
                    .push(url.clone());
            }

            content.pages.insert(url, page);
        }

        info!(
            pages = content.pages.len(),
            sections = content.sections.len(),
            tags = content.taxonomies.tags.len(),
            categories = content.taxonomies.categories.len(),
            "content collection complete"
        );

        Ok(content)
    }

    /// Find all content files recursively.
    fn find_content_files(&self) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        self.walk_dir(&self.content_dir, &mut files)?;
        Ok(files)
    }

    /// Recursively walk a directory for content files.
    fn walk_dir(&self, dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
        if !dir.exists() {
            return Ok(());
        }

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                // Skip hidden directories
                if path
                    .file_name()
                    .is_some_and(|n| n.to_string_lossy().starts_with('.'))
                {
                    continue;
                }
                self.walk_dir(&path, files)?;
            } else if path.is_file() {
                // Check if it's a content file
                if let Some(ext) = path.extension()
                    && ContentType::from_extension(&ext.to_string_lossy()).is_some()
                {
                    files.push(path);
                }
            }
        }

        Ok(())
    }

    /// Parse a single content file into a Page.
    fn parse_file(&self, path: &Path) -> Result<Page> {
        debug!(path = %path.display(), "parsing file");

        // Read file content
        let content = fs::read_to_string(path)?;

        // Parse content path to extract slug and language
        let relative_path = path.strip_prefix(&self.content_dir).unwrap_or(path);
        let content_path =
            ContentPath::from_path(relative_path, &self.config.site.default_language)
                .ok_or_else(|| CollectorError::InvalidPath(path.to_path_buf()))?;

        // Parse content using appropriate parser
        let parsed = self
            .parser
            .parse(&content, path)
            .map_err(|e| CollectorError::Parse {
                path: path.to_path_buf(),
                message: e.to_string(),
            })?;

        Ok(Page::from_parsed(parsed, &content_path))
    }

    /// Get pages sorted by date (newest first).
    pub fn pages_by_date(content: &SiteContent) -> Vec<&Page> {
        let mut pages: Vec<_> = content.pages.values().collect();
        pages.sort_by(|a, b| match (&b.date, &a.date) {
            (Some(b_date), Some(a_date)) => b_date.cmp(a_date),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => a.title.cmp(&b.title),
        });
        pages
    }

    /// Get pages for a specific section, sorted by date.
    pub fn section_pages<'a>(content: &'a SiteContent, section: &str) -> Vec<&'a Page> {
        let mut pages: Vec<_> = content
            .sections
            .get(section)
            .map(|urls| urls.iter().filter_map(|u| content.pages.get(u)).collect())
            .unwrap_or_default();

        pages.sort_by(|a, b| match (&b.date, &a.date) {
            (Some(b_date), Some(a_date)) => b_date.cmp(a_date),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => a.title.cmp(&b.title),
        });
        pages
    }

    /// Get pages for a taxonomy term, sorted by date.
    pub fn taxonomy_pages<'a>(
        content: &'a SiteContent,
        taxonomy: &str,
        term: &str,
    ) -> Vec<&'a Page> {
        let urls = match taxonomy {
            "tags" => content.taxonomies.tags.get(term),
            "categories" => content.taxonomies.categories.get(term),
            _ => None,
        };

        let mut pages: Vec<_> = urls
            .map(|u| u.iter().filter_map(|url| content.pages.get(url)).collect())
            .unwrap_or_default();

        pages.sort_by(|a, b| match (&b.date, &a.date) {
            (Some(b_date), Some(a_date)) => b_date.cmp(a_date),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => a.title.cmp(&b.title),
        });
        pages
    }
}

/// Paginate a slice of items.
pub fn paginate<T>(items: &[T], page: usize, per_page: usize) -> (&[T], usize) {
    let total_pages = items.len().div_ceil(per_page);
    let start = (page - 1) * per_page;
    let end = (start + per_page).min(items.len());

    if start >= items.len() {
        (&[], total_pages)
    } else {
        (&items[start..end], total_pages)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[allow(dead_code)]
    fn test_config() -> Config {
        Config {
            site: typstify_core::config::SiteConfig {
                title: "Test Site".to_string(),
                host: "https://example.com".to_string(),
                base_path: String::new(),
                default_language: "en".to_string(),
                description: None,
                author: None,
            },
            languages: HashMap::new(),
            build: typstify_core::config::BuildConfig {
                drafts: false,
                ..Default::default()
            },
            search: typstify_core::config::SearchConfig::default(),
            rss: typstify_core::config::RssConfig::default(),
            robots: typstify_core::config::RobotsConfig::default(),
            taxonomies: typstify_core::config::TaxonomyConfig::default(),
        }
    }

    #[test]
    fn test_paginate() {
        let items = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

        let (page1, total) = paginate(&items, 1, 3);
        assert_eq!(page1, &[1, 2, 3]);
        assert_eq!(total, 4);

        let (page2, _) = paginate(&items, 2, 3);
        assert_eq!(page2, &[4, 5, 6]);

        let (page4, _) = paginate(&items, 4, 3);
        assert_eq!(page4, &[10]);

        let (page5, _) = paginate(&items, 5, 3);
        assert!(page5.is_empty());
    }

    #[test]
    fn test_taxonomy_index() {
        let mut index = TaxonomyIndex::default();
        index.tags.insert(
            "rust".to_string(),
            vec!["post1".to_string(), "post2".to_string()],
        );
        index
            .tags
            .insert("web".to_string(), vec!["post2".to_string()]);

        assert_eq!(index.tags.get("rust").unwrap().len(), 2);
        assert_eq!(index.tags.get("web").unwrap().len(), 1);
        assert!(!index.tags.contains_key("python"));
    }

    #[test]
    fn test_site_content_default() {
        let content = SiteContent::default();
        assert!(content.pages.is_empty());
        assert!(content.sections.is_empty());
        assert!(content.taxonomies.tags.is_empty());
    }
}
