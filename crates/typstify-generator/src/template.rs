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
        self.register(Template::new("list", DEFAULT_LIST_TEMPLATE));
        self.register(Template::new("taxonomy", DEFAULT_TAXONOMY_TEMPLATE));
        self.register(Template::new("redirect", DEFAULT_REDIRECT_TEMPLATE));
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
pub const DEFAULT_BASE_TEMPLATE: &str = r##"<!DOCTYPE html>
<html lang="{{ lang }}" class="scroll-smooth">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{{ title }}{{ site_title_suffix? }}</title>
    <meta name="description" content="{{ description? }}">
    <meta name="author" content="{{ author? }}">
    <link rel="canonical" href="{{ canonical_url }}">
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
    <link href="https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500;600;700&display=swap" rel="stylesheet">
    {{ custom_css? }}
    <style>
        /* CSS Variables for Light/Dark Themes */
        :root {
            --color-primary: #3B82F6;
            --color-primary-hover: #2563EB;
            --color-secondary: #60A5FA;
            --color-cta: #F97316;
            --color-cta-hover: #EA580C;
            --color-bg: #F8FAFC;
            --color-bg-secondary: #FFFFFF;
            --color-text: #1E293B;
            --color-text-secondary: #475569;
            --color-text-muted: #64748B;
            --color-border: #E2E8F0;
            --color-code-bg: #F1F5F9;
            --shadow-sm: 0 1px 2px 0 rgb(0 0 0 / 0.05);
            --shadow-md: 0 4px 6px -1px rgb(0 0 0 / 0.1), 0 2px 4px -2px rgb(0 0 0 / 0.1);
            color-scheme: light;
        }

        [data-theme="dark"] {
            --color-primary: #60A5FA;
            --color-primary-hover: #93C5FD;
            --color-secondary: #3B82F6;
            --color-cta: #FB923C;
            --color-cta-hover: #FDBA74;
            --color-bg: #0F172A;
            --color-bg-secondary: #1E293B;
            --color-text: #F1F5F9;
            --color-text-secondary: #CBD5E1;
            --color-text-muted: #94A3B8;
            --color-border: #334155;
            --color-code-bg: #1E293B;
            --shadow-sm: 0 1px 2px 0 rgb(0 0 0 / 0.3);
            --shadow-md: 0 4px 6px -1px rgb(0 0 0 / 0.4), 0 2px 4px -2px rgb(0 0 0 / 0.3);
            color-scheme: dark;
        }

        @media (prefers-color-scheme: dark) {
            :root:not([data-theme="light"]) {
                --color-primary: #60A5FA;
                --color-primary-hover: #93C5FD;
                --color-secondary: #3B82F6;
                --color-cta: #FB923C;
                --color-cta-hover: #FDBA74;
                --color-bg: #0F172A;
                --color-bg-secondary: #1E293B;
                --color-text: #F1F5F9;
                --color-text-secondary: #CBD5E1;
                --color-text-muted: #94A3B8;
                --color-border: #334155;
                --color-code-bg: #1E293B;
                --shadow-sm: 0 1px 2px 0 rgb(0 0 0 / 0.3);
                --shadow-md: 0 4px 6px -1px rgb(0 0 0 / 0.4), 0 2px 4px -2px rgb(0 0 0 / 0.3);
                color-scheme: dark;
            }
        }

        /* Reset & Base */
        *, *::before, *::after { box-sizing: border-box; }
        * { margin: 0; padding: 0; }

        html {
            font-size: 16px;
            -webkit-font-smoothing: antialiased;
            -moz-osx-font-smoothing: grayscale;
        }

        body {
            font-family: 'Inter', system-ui, -apple-system, sans-serif;
            font-weight: 400;
            line-height: 1.7;
            color: var(--color-text);
            background-color: var(--color-bg);
            min-height: 100vh;
            display: flex;
            flex-direction: column;
            transition: background-color 0.2s ease, color 0.2s ease;
        }

        /* Layout */
        .container {
            width: 100%;
            max-width: 720px;
            margin: 0 auto;
            padding: 0 1.5rem;
        }

        /* Header */
        header {
            position: sticky;
            top: 0;
            z-index: 50;
            background-color: var(--color-bg);
            border-bottom: 1px solid var(--color-border);
            backdrop-filter: blur(8px);
            -webkit-backdrop-filter: blur(8px);
            background-color: rgba(248, 250, 252, 0.9);
        }

        [data-theme="dark"] header {
            background-color: rgba(15, 23, 42, 0.9);
        }

        @media (prefers-color-scheme: dark) {
            :root:not([data-theme="light"]) header {
                background-color: rgba(15, 23, 42, 0.9);
            }
        }

        header nav {
            display: flex;
            align-items: center;
            justify-content: space-between;
            padding: 1rem 0;
        }

        .site-title {
            font-size: 1.125rem;
            font-weight: 600;
            color: var(--color-text);
            text-decoration: none;
            letter-spacing: -0.025em;
            transition: color 0.2s ease;
        }

        .site-title:hover {
            color: var(--color-primary);
        }

        .nav-links {
            display: flex;
            align-items: center;
            gap: 1.5rem;
        }

        .nav-links a {
            font-size: 0.875rem;
            font-weight: 500;
            color: var(--color-text-secondary);
            text-decoration: none;
            transition: color 0.2s ease;
        }

        .nav-links a:hover {
            color: var(--color-primary);
        }

        /* Theme Toggle Button */
        .theme-toggle {
            display: flex;
            align-items: center;
            justify-content: center;
            width: 2.25rem;
            height: 2.25rem;
            border-radius: 0.5rem;
            border: 1px solid var(--color-border);
            background-color: var(--color-bg-secondary);
            cursor: pointer;
            transition: all 0.2s ease;
        }

        .theme-toggle:hover {
            border-color: var(--color-primary);
            background-color: var(--color-bg);
        }

        .theme-toggle svg {
            width: 1.125rem;
            height: 1.125rem;
            color: var(--color-text-secondary);
        }

        .theme-toggle .icon-sun { display: none; }
        .theme-toggle .icon-moon { display: block; }

        [data-theme="dark"] .theme-toggle .icon-sun { display: block; }
        [data-theme="dark"] .theme-toggle .icon-moon { display: none; }

        @media (prefers-color-scheme: dark) {
            :root:not([data-theme="light"]) .theme-toggle .icon-sun { display: block; }
            :root:not([data-theme="light"]) .theme-toggle .icon-moon { display: none; }
        }

        /* Main Content */
        main {
            flex: 1;
            padding: 3rem 0;
        }

        /* Typography */
        h1, h2, h3, h4, h5, h6 {
            font-weight: 600;
            line-height: 1.3;
            letter-spacing: -0.025em;
            color: var(--color-text);
        }

        h1 { font-size: 2rem; margin-bottom: 1rem; }
        h2 { font-size: 1.5rem; margin: 2rem 0 0.75rem; }
        h3 { font-size: 1.25rem; margin: 1.5rem 0 0.5rem; }
        h4 { font-size: 1.125rem; margin: 1.25rem 0 0.5rem; }

        p { margin-bottom: 1.25rem; }

        a {
            color: var(--color-primary);
            text-decoration: none;
            transition: color 0.15s ease;
        }

        a:hover {
            color: var(--color-primary-hover);
            text-decoration: underline;
        }

        /* Lists */
        ul, ol {
            padding-left: 1.5rem;
            margin-bottom: 1.25rem;
        }

        li { margin-bottom: 0.375rem; }
        li::marker { color: var(--color-text-muted); }

        /* Code */
        code {
            font-family: 'SF Mono', ui-monospace, 'Cascadia Code', Menlo, Consolas, monospace;
            font-size: 0.875em;
            background-color: var(--color-code-bg);
            padding: 0.125rem 0.375rem;
            border-radius: 0.25rem;
        }

        pre {
            background-color: var(--color-code-bg);
            padding: 1rem;
            border-radius: 0.5rem;
            overflow-x: auto;
            margin-bottom: 1.5rem;
            border: 1px solid var(--color-border);
        }

        pre code {
            background: none;
            padding: 0;
            font-size: 0.8125rem;
            line-height: 1.6;
        }

        /* Blockquote */
        blockquote {
            border-left: 3px solid var(--color-primary);
            padding-left: 1rem;
            margin: 1.5rem 0;
            color: var(--color-text-secondary);
            font-style: italic;
        }

        /* Images */
        img {
            max-width: 100%;
            height: auto;
            border-radius: 0.5rem;
        }

        /* Tables */
        table {
            width: 100%;
            border-collapse: collapse;
            margin: 1.5rem 0;
            font-size: 0.875rem;
        }

        th, td {
            padding: 0.75rem;
            text-align: left;
            border-bottom: 1px solid var(--color-border);
        }

        th {
            font-weight: 600;
            background-color: var(--color-bg-secondary);
        }

        /* Horizontal Rule */
        hr {
            border: none;
            border-top: 1px solid var(--color-border);
            margin: 2rem 0;
        }

        /* Footer */
        footer {
            border-top: 1px solid var(--color-border);
            padding: 2rem 0;
            margin-top: auto;
        }

        footer p {
            font-size: 0.875rem;
            color: var(--color-text-muted);
            text-align: center;
            margin: 0;
        }

        /* Article Styles */
        article header {
            position: static;
            background: none;
            border: none;
            backdrop-filter: none;
            padding: 0;
            margin-bottom: 2rem;
        }

        article header h1 {
            margin-bottom: 0.75rem;
        }

        article time {
            display: block;
            font-size: 0.875rem;
            color: var(--color-text-muted);
            margin-bottom: 0.5rem;
        }

        /* Tags */
        .tags {
            display: flex;
            flex-wrap: wrap;
            gap: 0.5rem;
            margin-top: 0.75rem;
        }

        .tags a {
            display: inline-flex;
            align-items: center;
            padding: 0.25rem 0.75rem;
            font-size: 0.75rem;
            font-weight: 500;
            color: var(--color-primary);
            background-color: var(--color-code-bg);
            border-radius: 9999px;
            text-decoration: none;
            transition: all 0.15s ease;
        }

        .tags a:hover {
            background-color: var(--color-primary);
            color: white;
            text-decoration: none;
        }

        /* Post List */
        .post-list ul {
            list-style: none;
            padding: 0;
        }

        .post-list li {
            display: flex;
            justify-content: space-between;
            align-items: baseline;
            gap: 1rem;
            padding: 1rem 0;
            border-bottom: 1px solid var(--color-border);
        }

        .post-list li:first-child {
            padding-top: 0;
        }

        .post-list li a {
            font-weight: 500;
            color: var(--color-text);
            text-decoration: none;
            transition: color 0.15s ease;
        }

        .post-list li a:hover {
            color: var(--color-primary);
        }

        .post-list time {
            flex-shrink: 0;
            font-size: 0.8125rem;
            color: var(--color-text-muted);
            font-variant-numeric: tabular-nums;
        }

        /* Pagination */
        .pagination {
            display: flex;
            justify-content: center;
            align-items: center;
            gap: 1rem;
            margin-top: 2rem;
            font-size: 0.875rem;
        }

        .pagination a {
            font-weight: 500;
        }

        /* Taxonomy */
        .taxonomy h1 {
            display: flex;
            align-items: center;
            gap: 0.5rem;
        }

        .taxonomy h1 span {
            color: var(--color-text-muted);
            font-weight: 400;
        }

        /* Responsive */
        @media (max-width: 640px) {
            html { font-size: 15px; }
            h1 { font-size: 1.75rem; }
            h2 { font-size: 1.375rem; }
            .container { padding: 0 1rem; }
            main { padding: 2rem 0; }
            .nav-links { gap: 1rem; }
            .post-list li { flex-direction: column; gap: 0.25rem; }
        }

        /* Reduced Motion */
        @media (prefers-reduced-motion: reduce) {
            *, *::before, *::after {
                animation-duration: 0.01ms !important;
                animation-iteration-count: 1 !important;
                transition-duration: 0.01ms !important;
            }
        }
    </style>
</head>
<body>
    <header>
        <div class="container">
            <nav>
                <a href="/" class="site-title">{{ site_title }}</a>
                <div class="nav-links">
                    <a href="/about">About</a>
                    <a href="/tags">Tags</a>
                    <button class="theme-toggle" aria-label="Toggle theme" type="button">
                        <svg class="icon-sun" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707M16 12a4 4 0 11-8 0 4 4 0 018 0z" />
                        </svg>
                        <svg class="icon-moon" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M20.354 15.354A9 9 0 018.646 3.646 9.003 9.003 0 0012 21a9.003 9.003 0 008.354-5.646z" />
                        </svg>
                    </button>
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
    <script>
        (function() {
            const toggle = document.querySelector('.theme-toggle');
            const html = document.documentElement;

            // Get saved theme or use system preference
            function getTheme() {
                const saved = localStorage.getItem('theme');
                if (saved) return saved;
                return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
            }

            // Apply theme
            function setTheme(theme) {
                html.setAttribute('data-theme', theme);
                localStorage.setItem('theme', theme);
            }

            // Initialize
            setTheme(getTheme());

            // Toggle on click
            toggle.addEventListener('click', () => {
                const current = html.getAttribute('data-theme') || getTheme();
                setTheme(current === 'dark' ? 'light' : 'dark');
            });

            // Listen for system changes
            window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', (e) => {
                if (!localStorage.getItem('theme')) {
                    setTheme(e.matches ? 'dark' : 'light');
                }
            });
        })();
    </script>
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
            .with_var("canonical_url", "https://example.com/my-page")
            .with_var("content", "<p>Hello!</p>")
            .with_var("site_title", "My Site")
            .with_var("year", "2026");

        let result = registry.render("base", &ctx).unwrap();
        assert!(result.contains("<!DOCTYPE html>"));
        assert!(result.contains("<title>My Page</title>"));
        assert!(result.contains("<p>Hello!</p>"));
    }
}
