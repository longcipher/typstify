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
    pub fn generate_page(&self, page: &Page, alternates: &[(&str, &str)]) -> Result<String> {
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
        let base_ctx = self.build_base_context(page, &inner_html, alternates)?;
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
            .with_var("year", Utc::now().year().to_string())
            // Navigation URLs
            .with_var("nav_home_url", "/")
            .with_var("nav_posts_url", "/posts")
            .with_var("nav_archives_url", "/archives")
            .with_var("nav_tags_url", "/tags")
            .with_var("nav_about_url", "/about");

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
            .with_var("year", Utc::now().year().to_string())
            // Navigation URLs
            .with_var("nav_home_url", "/")
            .with_var("nav_posts_url", "/posts")
            .with_var("nav_archives_url", "/archives")
            .with_var("nav_tags_url", "/tags")
            .with_var("nav_about_url", "/about");

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
    fn build_base_context(
        &self,
        page: &Page,
        inner_html: &str,
        alternates: &[(&str, &str)],
    ) -> Result<TemplateContext> {
        // Determine language prefix for URLs
        let lang_prefix = if page.is_default_lang {
            String::new()
        } else {
            format!("/{}", page.lang)
        };

        let mut ctx = TemplateContext::new()
            .with_var("lang", &page.lang)
            .with_var("title", &page.title)
            .with_var(
                "site_title_suffix",
                format!(" | {}", self.config.title_for_language(&page.lang)),
            )
            .with_var(
                "canonical_url",
                format!("{}{}", self.config.site.base_url, page.url),
            )
            .with_var("content", inner_html)
            .with_var("site_title", self.config.title_for_language(&page.lang))
            .with_var("year", Utc::now().year().to_string())
            // Navigation URLs with language prefix
            .with_var("nav_home_url", format!("{lang_prefix}/"))
            .with_var("nav_posts_url", format!("{lang_prefix}/posts"))
            .with_var("nav_archives_url", format!("{lang_prefix}/archives"))
            .with_var("nav_tags_url", format!("{lang_prefix}/tags"))
            .with_var("nav_about_url", format!("{lang_prefix}/about"));

        // Add description if present
        if let Some(desc) = &page.description {
            ctx.insert("description", desc);
        } else if let Some(site_desc) = self.config.description_for_language(&page.lang) {
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

        // Generate language switcher HTML
        let lang_switcher = self.generate_lang_switcher(&page.lang, &page.canonical_id);
        if !lang_switcher.is_empty() {
            ctx.insert("lang_switcher", lang_switcher);
        }

        // Add hreflang tags
        if !alternates.is_empty() {
            let hreflang = alternates
                .iter()
                .map(|(lang, url)| {
                    format!(
                        r#"<link rel="alternate" hreflang="{}" href="{}{}" />"#,
                        lang, self.config.site.base_url, url
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");
            ctx.insert("hreflang", hreflang);
        }

        Ok(ctx)
    }

    /// Generate language switcher HTML dropdown.
    fn generate_lang_switcher(&self, current_lang: &str, canonical_id: &str) -> String {
        let all_langs = self.config.all_languages();
        if all_langs.len() <= 1 {
            return String::new();
        }

        let mut options = Vec::new();

        for lang in &all_langs {
            let name = self.config.language_name(lang);
            let url = if *lang == self.config.site.default_language {
                // Default language: no prefix
                if canonical_id.is_empty() {
                    "/".to_string()
                } else {
                    format!("/{canonical_id}")
                }
            } else {
                // Non-default language: add prefix
                if canonical_id.is_empty() {
                    format!("/{lang}/")
                } else {
                    format!("/{lang}/{canonical_id}")
                }
            };

            let selected_class = if *lang == current_lang { " active" } else { "" };
            options.push(format!(
                r#"<a href="{url}" class="lang-option{selected_class}">{name}</a>"#,
            ));
        }

        // Get the language code for display (uppercase, max 2 chars)
        let display_code = current_lang
            .chars()
            .take(2)
            .collect::<String>()
            .to_uppercase();

        format!(
            r#"<div class="lang-switcher" tabindex="0" role="button" aria-label="Switch language" aria-haspopup="true">
    <span class="lang-code">{}</span>
    <div class="lang-dropdown">{}</div>
</div>"#,
            display_code,
            options.join("\n        ")
        )
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

    /// Generate a tags index page listing all tags with their counts.
    pub fn generate_tags_index_page(
        &self,
        tags: &std::collections::HashMap<String, Vec<String>>,
        lang: &str,
    ) -> Result<String> {
        let is_default_lang = lang == self.config.site.default_language;
        let lang_prefix = if is_default_lang {
            String::new()
        } else {
            format!("/{lang}")
        };

        let mut items: Vec<_> = tags.iter().collect();
        items.sort_by(|a, b| b.1.len().cmp(&a.1.len())); // Sort by count descending

        let items_html: String = items
            .iter()
            .map(|(tag, pages)| {
                format!(
                    r#"<a href="{}/tags/{}" class="tag-item"><span class="tag-name">{}</span><span class="tag-count">{}</span></a>"#,
                    lang_prefix,
                    slug_from_str(tag),
                    tag,
                    pages.len()
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        let ctx = TemplateContext::new().with_var("items", &items_html);
        let inner_html = self.templates.render("tags_index", &ctx)?;

        let mut base_ctx = TemplateContext::new()
            .with_var("lang", lang)
            .with_var("title", "Tags")
            .with_var(
                "site_title_suffix",
                format!(" | {}", self.config.title_for_language(lang)),
            )
            .with_var(
                "canonical_url",
                format!("{}{}/tags", self.config.site.base_url, lang_prefix),
            )
            .with_var("content", &inner_html)
            .with_var("site_title", self.config.title_for_language(lang))
            .with_var("year", Utc::now().year().to_string())
            // Navigation URLs
            .with_var("nav_home_url", format!("{lang_prefix}/"))
            .with_var("nav_posts_url", format!("{lang_prefix}/posts"))
            .with_var("nav_archives_url", format!("{lang_prefix}/archives"))
            .with_var("nav_tags_url", format!("{lang_prefix}/tags"))
            .with_var("nav_about_url", format!("{lang_prefix}/about"));

        // Generate language switcher
        let lang_switcher = self.generate_lang_switcher(lang, "tags");
        if !lang_switcher.is_empty() {
            base_ctx.insert("lang_switcher", lang_switcher);
        }

        Ok(self.templates.render("base", &base_ctx)?)
    }

    /// Generate a categories index page listing all categories with their counts.
    pub fn generate_categories_index_page(
        &self,
        categories: &std::collections::HashMap<String, Vec<String>>,
        lang: &str,
    ) -> Result<String> {
        let is_default_lang = lang == self.config.site.default_language;
        let lang_prefix = if is_default_lang {
            String::new()
        } else {
            format!("/{lang}")
        };

        let mut items: Vec<_> = categories.iter().collect();
        items.sort_by(|a, b| a.0.cmp(b.0)); // Sort alphabetically

        let items_html: String = items
            .iter()
            .map(|(category, pages)| {
                format!(
                    r#"<li><a href="{}/categories/{}">{}</a> <span class="count">({})</span></li>"#,
                    lang_prefix,
                    slug_from_str(category),
                    category,
                    pages.len()
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        let ctx = TemplateContext::new().with_var("items", &items_html);
        let inner_html = self.templates.render("categories_index", &ctx)?;

        let mut base_ctx = TemplateContext::new()
            .with_var("lang", lang)
            .with_var("title", "Categories")
            .with_var(
                "site_title_suffix",
                format!(" | {}", self.config.title_for_language(lang)),
            )
            .with_var(
                "canonical_url",
                format!("{}{}/categories", self.config.site.base_url, lang_prefix),
            )
            .with_var("content", &inner_html)
            .with_var("site_title", self.config.title_for_language(lang))
            .with_var("year", Utc::now().year().to_string())
            // Navigation URLs
            .with_var("nav_home_url", format!("{lang_prefix}/"))
            .with_var("nav_posts_url", format!("{lang_prefix}/posts"))
            .with_var("nav_archives_url", format!("{lang_prefix}/archives"))
            .with_var("nav_tags_url", format!("{lang_prefix}/tags"))
            .with_var("nav_about_url", format!("{lang_prefix}/about"));

        // Generate language switcher
        let lang_switcher = self.generate_lang_switcher(lang, "categories");
        if !lang_switcher.is_empty() {
            base_ctx.insert("lang_switcher", lang_switcher);
        }

        Ok(self.templates.render("base", &base_ctx)?)
    }

    /// Generate an archives page listing all posts grouped by year.
    pub fn generate_archives_page(&self, pages: &[&Page], lang: &str) -> Result<String> {
        use std::collections::BTreeMap;

        let is_default_lang = lang == self.config.site.default_language;
        let lang_prefix = if is_default_lang {
            String::new()
        } else {
            format!("/{lang}")
        };

        // Group pages by year
        let mut by_year: BTreeMap<i32, Vec<&Page>> = BTreeMap::new();
        for page in pages {
            if let Some(date) = page.date {
                by_year.entry(date.year()).or_default().push(page);
            }
        }

        // Sort pages within each year by date (newest first)
        for pages in by_year.values_mut() {
            pages.sort_by(|a, b| b.date.cmp(&a.date));
        }

        // Generate HTML (years in descending order)
        let items_html: String = by_year
            .iter()
            .rev()
            .map(|(year, year_pages)| {
                let posts_html: String = year_pages
                    .iter()
                    .map(|p| {
                        let date_str = p
                            .date
                            .map(|d| d.format("%m-%d").to_string())
                            .unwrap_or_default();
                        format!(
                            r#"<li><span class="archive-date">{}</span><a href="{}">{}</a></li>"#,
                            date_str, p.url, p.title
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n");

                format!(r#"<div class="archive-year"><h2>{year}</h2><ul>{posts_html}</ul></div>"#,)
            })
            .collect::<Vec<_>>()
            .join("\n");

        let ctx = TemplateContext::new().with_var("items", &items_html);
        let inner_html = self.templates.render("archives", &ctx)?;

        let mut base_ctx = TemplateContext::new()
            .with_var("lang", lang)
            .with_var("title", "Archives")
            .with_var(
                "site_title_suffix",
                format!(" | {}", self.config.title_for_language(lang)),
            )
            .with_var(
                "canonical_url",
                format!("{}{}/archives", self.config.site.base_url, lang_prefix),
            )
            .with_var("content", &inner_html)
            .with_var("site_title", self.config.title_for_language(lang))
            .with_var("year", Utc::now().year().to_string())
            // Navigation URLs
            .with_var("nav_home_url", format!("{lang_prefix}/"))
            .with_var("nav_posts_url", format!("{lang_prefix}/posts"))
            .with_var("nav_archives_url", format!("{lang_prefix}/archives"))
            .with_var("nav_tags_url", format!("{lang_prefix}/tags"))
            .with_var("nav_about_url", format!("{lang_prefix}/about"));

        // Generate language switcher
        let lang_switcher = self.generate_lang_switcher(lang, "archives");
        if !lang_switcher.is_empty() {
            base_ctx.insert("lang_switcher", lang_switcher);
        }

        Ok(self.templates.render("base", &base_ctx)?)
    }

    /// Generate a section index page (e.g., /posts/).
    pub fn generate_section_page(
        &self,
        section: &str,
        description: Option<&str>,
        items_html: &str,
        pagination_html: Option<&str>,
        lang: &str,
    ) -> Result<String> {
        let is_default_lang = lang == self.config.site.default_language;
        let lang_prefix = if is_default_lang {
            String::new()
        } else {
            format!("/{lang}")
        };

        // Convert section name to title case
        let title = section
            .chars()
            .next()
            .map(|c| c.to_uppercase().collect::<String>() + &section[1..])
            .unwrap_or_else(|| section.to_string());

        let mut ctx = TemplateContext::new()
            .with_var("title", &title)
            .with_var("items", items_html);

        if let Some(desc) = description {
            ctx.insert("description", desc);
        }

        if let Some(pagination) = pagination_html {
            ctx.insert("pagination", pagination);
        }

        let inner_html = self.templates.render("section", &ctx)?;

        let mut base_ctx = TemplateContext::new()
            .with_var("lang", lang)
            .with_var("title", &title)
            .with_var(
                "site_title_suffix",
                format!(" | {}", self.config.title_for_language(lang)),
            )
            .with_var(
                "canonical_url",
                format!("{}{}/{}", self.config.site.base_url, lang_prefix, section),
            )
            .with_var("content", &inner_html)
            .with_var("site_title", self.config.title_for_language(lang))
            .with_var("year", Utc::now().year().to_string())
            // Navigation URLs
            .with_var("nav_home_url", format!("{lang_prefix}/"))
            .with_var("nav_posts_url", format!("{lang_prefix}/posts"))
            .with_var("nav_archives_url", format!("{lang_prefix}/archives"))
            .with_var("nav_tags_url", format!("{lang_prefix}/tags"))
            .with_var("nav_about_url", format!("{lang_prefix}/about"));

        // Generate language switcher
        let lang_switcher = self.generate_lang_switcher(lang, section);
        if !lang_switcher.is_empty() {
            base_ctx.insert("lang_switcher", lang_switcher);
        }

        Ok(self.templates.render("base", &base_ctx)?)
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

    let description_html = page
        .description
        .as_ref()
        .filter(|d| !d.is_empty())
        .map(|d| format!(r#"<p class="post-description">{d}</p>"#))
        .unwrap_or_default();

    format!(
        r#"<li class="post-item">
    <div class="post-item-header">
        <a href="{}" class="post-title">{}</a>
        {}
    </div>
    {}
</li>"#,
        page.url, page.title, date_html, description_html
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
    use std::collections::HashMap;

    use super::*;

    fn test_config() -> Config {
        Config {
            site: typstify_core::config::SiteConfig {
                title: "Test Site".to_string(),
                base_url: "https://example.com".to_string(),
                default_language: "en".to_string(),
                description: Some("A test site".to_string()),
                author: Some("Test Author".to_string()),
            },
            languages: HashMap::new(),
            build: typstify_core::config::BuildConfig::default(),
            search: typstify_core::config::SearchConfig::default(),
            rss: typstify_core::config::RssConfig::default(),
            robots: typstify_core::config::RobotsConfig::default(),
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
            lang: "en".to_string(),
            is_default_lang: true,
            canonical_id: "test-page".to_string(),
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

        let html = generator.generate_page(&page, &[]).unwrap();

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

        assert!(html.contains(r#"<li class="post-item">"#));
        assert!(html.contains("post-title"));
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
