//! HTML template system for page generation.
//!
//! Provides a lightweight template system using string interpolation rather than
//! heavy template engines like Tera or Handlebars.

use std::collections::HashMap;

use thiserror::Error;

/// Template rendering errors.
#[derive(Debug, Error)]
pub enum TemplateError {
    /// Missing required variable.
    #[error("missing required variable: {0}")]
    MissingVariable(String),

    /// Template not found.
    #[error("template not found: {0}")]
    NotFound(String),

    /// Invalid template syntax.
    #[error("invalid template syntax: {0}")]
    InvalidSyntax(String),
}

/// Result type for template operations.
pub type Result<T> = std::result::Result<T, TemplateError>;

/// Template context with variables for interpolation.
#[derive(Debug, Clone, Default)]
pub struct TemplateContext {
    variables: HashMap<String, String>,
}

impl TemplateContext {
    /// Create a new empty context.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a variable into the context.
    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.variables.insert(key.into(), value.into());
    }

    /// Create context with initial variables.
    pub fn with_var(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.insert(key, value);
        self
    }

    /// Get a variable value.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&str> {
        self.variables.get(key).map(String::as_str)
    }

    /// Check if a variable exists.
    #[must_use]
    pub fn contains(&self, key: &str) -> bool {
        self.variables.contains_key(key)
    }
}

/// A simple template that supports variable interpolation.
///
/// Variables are specified as `{{ variable_name }}` in the template string.
#[derive(Debug, Clone)]
pub struct Template {
    name: String,
    content: String,
}

impl Template {
    /// Create a new template with the given name and content.
    #[must_use]
    pub fn new(name: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            content: content.into(),
        }
    }

    /// Get the template name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Render the template with the given context.
    ///
    /// Replaces all `{{ variable }}` placeholders with values from context.
    pub fn render(&self, context: &TemplateContext) -> Result<String> {
        let mut result = self.content.clone();
        let mut pos = 0;

        while let Some(start) = result[pos..].find("{{") {
            let start = pos + start;
            let end = result[start..]
                .find("}}")
                .ok_or_else(|| TemplateError::InvalidSyntax("unclosed {{ delimiter".to_string()))?;
            let end = start + end + 2;

            let var_name = result[start + 2..end - 2].trim();

            // Check for optional variable syntax: {{ variable? }}
            let (var_name, optional) = if let Some(stripped) = var_name.strip_suffix('?') {
                (stripped, true)
            } else {
                (var_name, false)
            };

            let value = match context.get(var_name) {
                Some(v) => v.to_string(),
                None if optional => String::new(),
                None => return Err(TemplateError::MissingVariable(var_name.to_string())),
            };

            result.replace_range(start..end, &value);
            pos = start + value.len();
        }

        Ok(result)
    }
}

/// Registry of templates.
#[derive(Debug, Clone, Default)]
pub struct TemplateRegistry {
    templates: HashMap<String, Template>,
}

impl TemplateRegistry {
    /// Create a new registry with default templates.
    #[must_use]
    pub fn new() -> Self {
        let mut registry = Self::default();
        registry.register_defaults();
        registry
    }

    /// Register default built-in templates.
    fn register_defaults(&mut self) {
        self.register(Template::new("base", DEFAULT_BASE_TEMPLATE));
        self.register(Template::new("page", DEFAULT_PAGE_TEMPLATE));
        self.register(Template::new("post", DEFAULT_POST_TEMPLATE));
        self.register(Template::new("short", DEFAULT_SHORT_TEMPLATE));
        self.register(Template::new("list", DEFAULT_LIST_TEMPLATE));
        self.register(Template::new("taxonomy", DEFAULT_TAXONOMY_TEMPLATE));
        self.register(Template::new("redirect", DEFAULT_REDIRECT_TEMPLATE));
        self.register(Template::new("tags_index", DEFAULT_TAGS_INDEX_TEMPLATE));
        self.register(Template::new(
            "categories_index",
            DEFAULT_CATEGORIES_INDEX_TEMPLATE,
        ));
        self.register(Template::new("archives", DEFAULT_ARCHIVES_TEMPLATE));
        self.register(Template::new("section", DEFAULT_SECTION_TEMPLATE));
        self.register(Template::new("shorts", DEFAULT_SHORTS_SECTION_TEMPLATE));
    }

    /// Register a template.
    pub fn register(&mut self, template: Template) {
        self.templates.insert(template.name.clone(), template);
    }

    /// Get a template by name.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&Template> {
        self.templates.get(name)
    }

    /// Render a named template with the given context.
    pub fn render(&self, name: &str, context: &TemplateContext) -> Result<String> {
        let template = self
            .get(name)
            .ok_or_else(|| TemplateError::NotFound(name.to_string()))?;
        template.render(context)
    }
}

/// Default base HTML template.
/// Uses external CSS and JS files for better caching and smaller HTML files.
pub const DEFAULT_BASE_TEMPLATE: &str = r##"<!DOCTYPE html>
<html lang="{{ lang }}" class="scroll-smooth">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{{ title }}{{ site_title_suffix? }}</title>
    <meta name="description" content="{{ description? }}">
    <meta name="author" content="{{ author? }}">
    <link rel="canonical" href="{{ canonical_url }}">
    {{ hreflang? }}
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
    <link href="https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500;600;700&display=swap" rel="stylesheet">
    <link rel="stylesheet" href="{{ base_path }}/assets/style.css">
    {{ custom_css? }}
    <script>
        // Inline critical JS to prevent FOUC (Flash of Unstyled Content)
        (function() {
            const saved = localStorage.getItem('theme');
            const theme = saved || (window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light');
            document.documentElement.setAttribute('data-theme', theme);
        })();
    </script>
</head>
<body>
    <header>
        <div class="container">
            <nav>
                <a href="{{ nav_home_url }}" class="site-title">{{ site_title }}</a>
                <div class="nav-links">
                    {{ section_nav? }}
                    <a href="{{ nav_archives_url }}">Archives</a>
                    <a href="{{ nav_tags_url }}">Tags</a>
                    <a href="{{ nav_about_url }}">About</a>
                    <div class="nav-actions">
                        <div class="search-wrapper" id="searchWrapper">
                            <input type="text" class="search-input" id="searchInput" placeholder="Search..." autocomplete="off">
                            <button class="search-btn" id="searchBtn" aria-label="Search" type="button">
                                <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                                    <path stroke-linecap="round" stroke-linejoin="round" d="M21 21l-5.197-5.197m0 0A7.5 7.5 0 105.196 5.196a7.5 7.5 0 0010.607 10.607z" />
                                </svg>
                            </button>
                            <div class="search-results" id="searchResults"></div>
                        </div>
                        {{ lang_switcher? }}
                        <button class="theme-toggle" aria-label="Toggle theme" type="button">
                            <svg class="icon-sun" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                                <path stroke-linecap="round" stroke-linejoin="round" d="M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707M16 12a4 4 0 11-8 0 4 4 0 018 0z" />
                            </svg>
                            <svg class="icon-moon" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                                <path stroke-linecap="round" stroke-linejoin="round" d="M20.354 15.354A9 9 0 018.646 3.646 9.003 9.003 0 0012 21a9.003 9.003 0 008.354-5.646z" />
                            </svg>
                        </button>
                    </div>
                </div>
            </nav>
        </div>
    </header>
    <main>
        <div class="container">
            {{ content }}
        </div>
    </main>
    <footer>
        <div class="container">
            <p>&copy; {{ year }} {{ site_title }}. Built with <a href="https://github.com/longcipher/typstify">Typstify</a>.</p>
        </div>
    </footer>
    <script src="{{ base_path }}/assets/main.js" defer></script>
    {{ custom_js? }}
</body>
</html>"##;

/// Default page template (for standalone pages).
pub const DEFAULT_PAGE_TEMPLATE: &str = r#"<article class="page">
    <h1>{{ title }}</h1>
    <div class="content">
        {{ content }}
    </div>
</article>"#;

/// Default post template (for blog posts with metadata).
pub const DEFAULT_POST_TEMPLATE: &str = r#"<article class="post">
    <header>
        <h1>{{ title }}</h1>
        <time datetime="{{ date_iso }}">{{ date_formatted }}</time>
        {{ tags_html? }}
    </header>
    <div class="content">
        {{ content }}
    </div>
</article>"#;

/// Default list template (for index pages).
pub const DEFAULT_LIST_TEMPLATE: &str = r#"<section class="post-list">
    <h1>{{ title }}</h1>
    <ul>
        {{ items }}
    </ul>
    <div class="pagination">{{ pagination? }}</div>
</section>"#;

/// Default taxonomy term template (for tag/category pages).
pub const DEFAULT_TAXONOMY_TEMPLATE: &str = r#"<section class="taxonomy post-list">
    <h1>{{ taxonomy_name }}: <span>{{ term }}</span></h1>
    <ul>
        {{ items }}
    </ul>
    <div class="pagination">{{ pagination? }}</div>
</section>"#;

/// Default redirect template for URL aliases.
pub const DEFAULT_REDIRECT_TEMPLATE: &str = r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <meta http-equiv="refresh" content="0; url={{ redirect_url }}">
    <link rel="canonical" href="{{ redirect_url }}">
    <title>Redirecting...</title>
</head>
<body>
    <p>Redirecting to <a href="{{ redirect_url }}">{{ redirect_url }}</a></p>
</body>
</html>"#;

/// Default tags index template (lists all tags with counts).
pub const DEFAULT_TAGS_INDEX_TEMPLATE: &str = r#"<section class="taxonomy-index">
    <h1>Tags</h1>
    <div class="tags-cloud">
        {{ items }}
    </div>
</section>"#;

/// Default categories index template (lists all categories with counts).
pub const DEFAULT_CATEGORIES_INDEX_TEMPLATE: &str = r#"<section class="taxonomy-index">
    <h1>Categories</h1>
    <ul class="categories-list">
        {{ items }}
    </ul>
</section>"#;

/// Default archives template (lists all posts grouped by year).
pub const DEFAULT_ARCHIVES_TEMPLATE: &str = r#"<section class="archives">
    <h1>Archives</h1>
    {{ items }}
</section>"#;

/// Default section template (lists all posts in a section).
pub const DEFAULT_SECTION_TEMPLATE: &str = r#"<section class="section-list post-list">
    <h1>{{ title }}</h1>
    <p class="section-description">{{ description? }}</p>
    <ul>
        {{ items }}
    </ul>
    <div class="pagination">{{ pagination? }}</div>
</section>"#;

/// Default short template (minimalist layout).
pub const DEFAULT_SHORT_TEMPLATE: &str = r#"<div class="short-item">
    <time class="short-date" datetime="{{ date_iso }}">{{ date_formatted }}</time>
    <div class="short-content">
        {{ content }}
    </div>
</div>"#;

/// Default shorts section template (minimalist layout).
pub const DEFAULT_SHORTS_SECTION_TEMPLATE: &str = r#"<section class="shorts-section">
    <h1>{{ title }}</h1>
    <p class="section-description">{{ description? }}</p>
    <div class="short-list">
        {{ items }}
    </div>
    <div class="pagination">{{ pagination? }}</div>
</section>"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_simple_render() {
        let template = Template::new("test", "Hello, {{ name }}!");
        let mut ctx = TemplateContext::new();
        ctx.insert("name", "World");

        let result = template.render(&ctx).unwrap();
        assert_eq!(result, "Hello, World!");
    }

    #[test]
    fn test_template_multiple_variables() {
        let template = Template::new(
            "test",
            "{{ greeting }}, {{ name }}! Welcome to {{ place }}.",
        );
        let ctx = TemplateContext::new()
            .with_var("greeting", "Hello")
            .with_var("name", "User")
            .with_var("place", "Typstify");

        let result = template.render(&ctx).unwrap();
        assert_eq!(result, "Hello, User! Welcome to Typstify.");
    }

    #[test]
    fn test_template_optional_variable() {
        let template = Template::new("test", "Hello{{ suffix? }}!");
        let ctx = TemplateContext::new();

        let result = template.render(&ctx).unwrap();
        assert_eq!(result, "Hello!");

        let ctx = TemplateContext::new().with_var("suffix", ", World");
        let result = template.render(&ctx).unwrap();
        assert_eq!(result, "Hello, World!");
    }

    #[test]
    fn test_template_missing_required_variable() {
        let template = Template::new("test", "Hello, {{ name }}!");
        let ctx = TemplateContext::new();

        let result = template.render(&ctx);
        assert!(matches!(result, Err(TemplateError::MissingVariable(_))));
    }

    #[test]
    fn test_template_registry() {
        let registry = TemplateRegistry::new();

        assert!(registry.get("base").is_some());
        assert!(registry.get("page").is_some());
        assert!(registry.get("post").is_some());
        assert!(registry.get("list").is_some());
        assert!(registry.get("nonexistent").is_none());
    }

    #[test]
    fn test_render_base_template() {
        let registry = TemplateRegistry::new();
        let ctx = TemplateContext::new()
            .with_var("lang", "en")
            .with_var("title", "My Page")
            .with_var("base_path", "")
            .with_var("canonical_url", "https://example.com/my-page")
            .with_var("content", "<p>Hello!</p>")
            .with_var("site_title", "My Site")
            .with_var("year", "2026")
            // Navigation URLs
            .with_var("nav_home_url", "/")
            .with_var("section_nav", r#"<a href="/posts">Posts</a>"#)
            .with_var("nav_archives_url", "/archives")
            .with_var("nav_tags_url", "/tags")
            .with_var("nav_about_url", "/about");

        let result = registry.render("base", &ctx).unwrap();
        assert!(result.contains("<!DOCTYPE html>"));
        assert!(result.contains("<title>My Page</title>"));
        assert!(result.contains("<p>Hello!</p>"));
    }
}
