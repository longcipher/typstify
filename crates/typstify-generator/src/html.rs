//! HTML generation from parsed content.
//!
//! Converts parsed content into final HTML pages using templates.

use std::path::{Path, PathBuf};

use chrono::{Datelike, Utc};
use thiserror::Error;
use tracing::debug;
use typstify_core::{Config, Page};

use crate::template::{Template, TemplateContext, TemplateError, TemplateRegistry};

/// HTML generation errors.
#[derive(Debug, Error)]
pub enum HtmlError {
    /// Template error.
    #[error("template error: {0}")]
    Template(#[from] TemplateError),

    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Invalid page data.
    #[error("invalid page data: {0}")]
    InvalidPage(String),
}

/// Result type for HTML generation.
pub type Result<T> = std::result::Result<T, HtmlError>;

/// HTML page generator.
#[derive(Debug)]
pub struct HtmlGenerator {
    templates: TemplateRegistry,
    config: Config,
}

impl HtmlGenerator {
    /// Create a new HTML generator with the given configuration.
    #[must_use]
    pub fn new(config: Config) -> Self {
        Self {
            templates: TemplateRegistry::new(),
            config,
        }
    }

    /// Create a generator with custom templates.
    #[must_use]
    pub fn with_templates(config: Config, templates: TemplateRegistry) -> Self {
        Self { templates, config }
    }

    /// Register a custom template.
    pub fn register_template(&mut self, template: Template) {
        self.templates.register(template);
    }

    /// Generate HTML for a page.
    pub fn generate_page(&self, page: &Page) -> Result<String> {
        debug!(url = %page.url, "generating HTML for page");

        // Determine which template to use
        let template_name =
            page.template
                .as_deref()
                .unwrap_or(if page.date.is_some() { "post" } else { "page" });

        // Build inner content context
        let inner_ctx = self.build_page_context(page)?;
        let inner_html = self.templates.render(template_name, &inner_ctx)?;

        // Build outer (base) context
        let base_ctx = self.build_base_context(page, &inner_html)?;
        Ok(self.templates.render("base", &base_ctx)?)
    }

    /// Generate redirect HTML for URL aliases.
    pub fn generate_redirect(&self, redirect_url: &str) -> Result<String> {
        let ctx = TemplateContext::new().with_var("redirect_url", redirect_url);
        self.templates
            .render("redirect", &ctx)
            .map_err(HtmlError::from)
    }

    /// Generate a list page HTML.
    pub fn generate_list_page(
        &self,
        title: &str,
        items_html: &str,
        pagination_html: Option<&str>,
    ) -> Result<String> {
        let mut ctx = TemplateContext::new()
            .with_var("title", title)
            .with_var("items", items_html);

        if let Some(pagination) = pagination_html {
            ctx.insert("pagination", pagination);
        }

        let inner_html = self.templates.render("list", &ctx)?;

        // Wrap in base template
        let base_ctx = TemplateContext::new()
            .with_var("lang", &self.config.site.default_language)
            .with_var("title", title)
            .with_var(
                "site_title_suffix",
                format!(" | {}", self.config.site.title),
            )
            .with_var("canonical_url", &self.config.site.base_url)
            .with_var("content", &inner_html)
            .with_var("site_title", &self.config.site.title)
            .with_var("year", Utc::now().year().to_string());

        Ok(self.templates.render("base", &base_ctx)?)
    }

    /// Generate a taxonomy term page HTML.
    pub fn generate_taxonomy_page(
        &self,
        taxonomy_name: &str,
        term: &str,
        items_html: &str,
        pagination_html: Option<&str>,
    ) -> Result<String> {
        let mut ctx = TemplateContext::new()
            .with_var("taxonomy_name", taxonomy_name)
            .with_var("term", term)
            .with_var("items", items_html);

        if let Some(pagination) = pagination_html {
            ctx.insert("pagination", pagination);
        }

        let inner_html = self.templates.render("taxonomy", &ctx)?;
        let title = format!("{taxonomy_name}: {term}");

        // Wrap in base template
        let base_ctx = TemplateContext::new()
            .with_var("lang", &self.config.site.default_language)
            .with_var("title", &title)
            .with_var(
                "site_title_suffix",
                format!(" | {}", self.config.site.title),
            )
            .with_var(
                "canonical_url",
                format!(
                    "{}/{}/{}",
                    self.config.site.base_url,
                    taxonomy_name.to_lowercase(),
                    term
                ),
            )
            .with_var("content", &inner_html)
            .with_var("site_title", &self.config.site.title)
            .with_var("year", Utc::now().year().to_string());

        Ok(self.templates.render("base", &base_ctx)?)
    }

    /// Build template context for page content.
    fn build_page_context(&self, page: &Page) -> Result<TemplateContext> {
        let mut ctx = TemplateContext::new()
            .with_var("title", &page.title)
            .with_var("content", &page.content);

        // Add date if present
        if let Some(date) = page.date {
            ctx.insert("date_iso", date.format("%Y-%m-%d").to_string());
            ctx.insert("date_formatted", date.format("%B %d, %Y").to_string());
        }

        // Add tags HTML if present
        if !page.tags.is_empty() {
            let tags_html = page
                .tags
                .iter()
                .map(|tag| {
                    format!(
                        r#"<a href="/tags/{}" rel="tag">{}</a>"#,
                        slug_from_str(tag),
                        tag
                    )
                })
                .collect::<Vec<_>>()
                .join(" ");
            ctx.insert(
                "tags_html",
                format!(r#"<div class="tags">{tags_html}</div>"#),
            );
        }

        Ok(ctx)
    }

    /// Build template context for base HTML wrapper.
    fn build_base_context(&self, page: &Page, inner_html: &str) -> Result<TemplateContext> {
        let lang = page
            .lang
            .as_deref()
            .unwrap_or(&self.config.site.default_language);

        let mut ctx = TemplateContext::new()
            .with_var("lang", lang)
            .with_var("title", &page.title)
            .with_var(
                "site_title_suffix",
                format!(" | {}", self.config.site.title),
            )
            .with_var(
                "canonical_url",
                format!("{}{}", self.config.site.base_url, page.url),
            )
            .with_var("content", inner_html)
            .with_var("site_title", &self.config.site.title)
            .with_var("year", Utc::now().year().to_string());

        // Add description if present
        if let Some(desc) = &page.description {
            ctx.insert("description", desc);
        } else if let Some(site_desc) = &self.config.site.description {
            ctx.insert("description", site_desc);
        }

        // Add author if present
        if let Some(author) = &self.config.site.author {
            ctx.insert("author", author);
        }

        // Add custom CSS
        if !page.custom_css.is_empty() {
            let css_links = page
                .custom_css
                .iter()
                .map(|href| format!(r#"<link rel="stylesheet" href="{href}">"#))
                .collect::<Vec<_>>()
                .join("\n");
            ctx.insert("custom_css", css_links);
        }

        // Add custom JS
        if !page.custom_js.is_empty() {
            let js_scripts = page
                .custom_js
                .iter()
                .map(|src| format!(r#"<script src="{src}"></script>"#))
                .collect::<Vec<_>>()
                .join("\n");
            ctx.insert("custom_js", js_scripts);
        }

        Ok(ctx)
    }

    /// Get the output path for a page.
    #[must_use]
    pub fn output_path(&self, page: &Page, output_dir: &Path) -> PathBuf {
        let relative = page.url.trim_start_matches('/');

        if relative.is_empty() {
            output_dir.join("index.html")
        } else {
            output_dir.join(relative).join("index.html")
        }
    }
}

/// Generate a URL-safe slug from a string.
fn slug_from_str(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// Generate HTML for a list item (used in list pages).
pub fn list_item_html(page: &Page) -> String {
    let date_html = page
        .date
        .map(|d| {
            format!(
                r#"<time datetime="{}">{}</time>"#,
                d.format("%Y-%m-%d"),
                d.format("%Y-%m-%d")
            )
        })
        .unwrap_or_default();

    format!(
        r#"<li><a href="{}">{}</a> {}</li>"#,
        page.url, page.title, date_html
    )
}

/// Generate pagination HTML.
pub fn pagination_html(current: usize, total: usize, base_url: &str) -> Option<String> {
    if total <= 1 {
        return None;
    }

    let mut parts = Vec::new();

    if current > 1 {
        let prev_url = if current == 2 {
            base_url.to_string()
        } else {
            format!("{}/page/{}", base_url, current - 1)
        };
        parts.push(format!(r#"<a href="{prev_url}" rel="prev">← Previous</a>"#));
    }

    parts.push(format!("Page {current} of {total}"));

    if current < total {
        parts.push(format!(
            r#"<a href="{}/page/{}" rel="next">Next →</a>"#,
            base_url,
            current + 1
        ));
    }

    Some(format!(
        r#"<nav class="pagination">{}</nav>"#,
        parts.join(" ")
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> Config {
        Config {
            site: typstify_core::config::SiteConfig {
                title: "Test Site".to_string(),
                base_url: "https://example.com".to_string(),
                default_language: "en".to_string(),
                languages: vec!["en".to_string()],
                description: Some("A test site".to_string()),
                author: Some("Test Author".to_string()),
            },
            build: typstify_core::config::BuildConfig::default(),
            search: typstify_core::config::SearchConfig::default(),
            rss: typstify_core::config::RssConfig::default(),
            taxonomies: typstify_core::config::TaxonomyConfig::default(),
        }
    }

    fn test_page() -> Page {
        Page {
            url: "/test-page".to_string(),
            title: "Test Page".to_string(),
            description: Some("A test page".to_string()),
            date: None,
            updated: None,
            draft: false,
            lang: None,
            tags: vec![],
            categories: vec![],
            content: "<p>Hello, World!</p>".to_string(),
            summary: None,
            reading_time: None,
            word_count: None,
            toc: vec![],
            custom_js: vec![],
            custom_css: vec![],
            aliases: vec![],
            template: None,
            weight: 0,
            source_path: Some(PathBuf::from("test-page.md")),
        }
    }

    #[test]
    fn test_generate_page() {
        let generator = HtmlGenerator::new(test_config());
        let page = test_page();

        let html = generator.generate_page(&page).unwrap();

        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("<title>Test Page | Test Site</title>"));
        assert!(html.contains("<p>Hello, World!</p>"));
        assert!(html.contains("Test Site"));
    }

    #[test]
    fn test_generate_redirect() {
        let generator = HtmlGenerator::new(test_config());

        let html = generator
            .generate_redirect("https://example.com/new-url")
            .unwrap();

        assert!(html.contains("Redirecting"));
        assert!(html.contains("https://example.com/new-url"));
        assert!(html.contains(r#"http-equiv="refresh""#));
    }

    #[test]
    fn test_slug_from_str() {
        assert_eq!(slug_from_str("Hello World"), "hello-world");
        assert_eq!(slug_from_str("Rust & Go"), "rust-go");
        assert_eq!(slug_from_str("  multiple   spaces  "), "multiple-spaces");
        assert_eq!(slug_from_str("CamelCase"), "camelcase");
    }

    #[test]
    fn test_list_item_html() {
        let page = test_page();
        let html = list_item_html(&page);

        assert!(html.contains("<li>"));
        assert!(html.contains("Test Page"));
        assert!(html.contains("/test-page"));
    }

    #[test]
    fn test_pagination_html() {
        // Single page - no pagination
        assert!(pagination_html(1, 1, "/blog").is_none());

        // First page of many
        let html = pagination_html(1, 5, "/blog").unwrap();
        assert!(html.contains("Page 1 of 5"));
        assert!(html.contains("Next →"));
        assert!(!html.contains("Previous"));

        // Middle page
        let html = pagination_html(3, 5, "/blog").unwrap();
        assert!(html.contains("Page 3 of 5"));
        assert!(html.contains("Previous"));
        assert!(html.contains("Next →"));

        // Last page
        let html = pagination_html(5, 5, "/blog").unwrap();
        assert!(html.contains("Page 5 of 5"));
        assert!(html.contains("Previous"));
        assert!(!html.contains("Next →"));
    }

    #[test]
    fn test_output_path() {
        let generator = HtmlGenerator::new(test_config());
        let output_dir = Path::new("public");

        let page = test_page();
        let path = generator.output_path(&page, output_dir);
        assert_eq!(path, PathBuf::from("public/test-page/index.html"));

        // Root page
        let mut root_page = test_page();
        root_page.url = "/".to_string();
        let path = generator.output_path(&root_page, output_dir);
        assert_eq!(path, PathBuf::from("public/index.html"));
    }
}
