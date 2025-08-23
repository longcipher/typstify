//! Content renderers for Markdown and Typst files

use eyre::Result;

#[derive(Debug, thiserror::Error)]
pub enum RendererError {
    #[error("Markdown rendering error: {0}")]
    MarkdownError(String),

    #[error("Typst rendering error: {0}")]
    TypstError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Common trait for content renderers
pub trait Renderer {
    fn render(&self, content: &str) -> Result<String, RendererError>;
}

/// Markdown renderer with Tailwind CSS and DaisyUI class integration
pub struct MarkdownRenderer {
    options: pulldown_cmark::Options,
}

impl MarkdownRenderer {
    pub fn new() -> Self {
        let mut options = pulldown_cmark::Options::empty();
        options.insert(pulldown_cmark::Options::ENABLE_TABLES);
        options.insert(pulldown_cmark::Options::ENABLE_FOOTNOTES);
        options.insert(pulldown_cmark::Options::ENABLE_STRIKETHROUGH);
        options.insert(pulldown_cmark::Options::ENABLE_TASKLISTS);
        options.insert(pulldown_cmark::Options::ENABLE_SMART_PUNCTUATION);

        Self { options }
    }
}

impl Default for MarkdownRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl Renderer for MarkdownRenderer {
    fn render(&self, content: &str) -> Result<String, RendererError> {
        let parser = pulldown_cmark::Parser::new_ext(content, self.options);
        let mut html_output = String::new();
        pulldown_cmark::html::push_html(&mut html_output, parser);

        // Post-process to add syntax highlighting classes
        let processed = html_output
            .replace("<pre><code>", "<pre><code class=\"language-text\">")
            .replace("<code>", "<code class=\"inline-code\">");

        Ok(processed)
    }
}

impl MarkdownRenderer {
    // Placeholder for future Tailwind/DaisyUI styling
}

/// Typst renderer
pub struct TypstRenderer {}

impl TypstRenderer {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for TypstRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl Renderer for TypstRenderer {
    fn render(&self, content: &str) -> Result<String, RendererError> {
        // Basic Typst to HTML converter
        // This is a simplified implementation that handles common Typst syntax
        let html = self.convert_typst_to_html(content)?;
        Ok(html)
    }
}

impl TypstRenderer {
    /// Convert Typst syntax to HTML
    fn convert_typst_to_html(&self, content: &str) -> Result<String, RendererError> {
        let lines: Vec<&str> = content.lines().collect();
        let mut html = String::new();
        let mut in_code_block = false;
        let mut code_language = String::new();

        for line in lines {
            // Skip comment lines (metadata)
            if line.trim_start().starts_with("//") {
                continue;
            }

            // Handle code blocks
            if line.trim().starts_with("```") {
                if in_code_block {
                    // End code block
                    html.push_str("</code></pre>\n");
                    in_code_block = false;
                    code_language.clear();
                } else {
                    // Start code block
                    let lang = line.trim().strip_prefix("```").unwrap_or("").trim();
                    code_language = if lang.is_empty() {
                        "text".to_string()
                    } else {
                        lang.to_string()
                    };
                    html.push_str(&format!(
                        r#"<pre><code class="language-{}">"#,
                        code_language
                    ));
                    in_code_block = true;
                }
                continue;
            }

            if in_code_block {
                // Inside code block, escape HTML and preserve formatting
                let escaped = line
                    .replace('&', "&amp;")
                    .replace('<', "&lt;")
                    .replace('>', "&gt;");
                html.push_str(&escaped);
                html.push('\n');
                continue;
            }

            // Handle headings
            if line.starts_with("====") {
                let text = line.strip_prefix("====").unwrap_or("").trim();
                html.push_str(&format!(
                    "<h4>{}</h4>\n",
                    self.process_inline_formatting(text)
                ));
            } else if line.starts_with("===") {
                let text = line.strip_prefix("===").unwrap_or("").trim();
                html.push_str(&format!(
                    "<h3>{}</h3>\n",
                    self.process_inline_formatting(text)
                ));
            } else if line.starts_with("==") {
                let text = line.strip_prefix("==").unwrap_or("").trim();
                html.push_str(&format!(
                    "<h2>{}</h2>\n",
                    self.process_inline_formatting(text)
                ));
            } else if line.starts_with("=") {
                let text = line.strip_prefix("=").unwrap_or("").trim();
                html.push_str(&format!(
                    "<h1>{}</h1>\n",
                    self.process_inline_formatting(text)
                ));
            }
            // Handle list items
            else if line.trim_start().starts_with("- ") {
                let _indent_level = (line.len() - line.trim_start().len()) / 2; // Assuming 2 spaces per indent
                let text = line.trim_start().strip_prefix("- ").unwrap_or("").trim();

                // For simplicity, just use <ul><li> without nested handling for now
                html.push_str(&format!(
                    "<ul><li>{}</li></ul>\n",
                    self.process_inline_formatting(text)
                ));
            }
            // Handle inline code
            else if line.trim().starts_with("`")
                && line.trim().ends_with("`")
                && line.trim().len() > 1
            {
                let code = line
                    .trim()
                    .strip_prefix("`")
                    .unwrap()
                    .strip_suffix("`")
                    .unwrap();
                html.push_str(&format!(
                    "<p><code class=\"inline-code\">{}</code></p>\n",
                    code
                ));
            }
            // Handle empty lines
            else if line.trim().is_empty() {
                html.push_str("<br>\n");
            }
            // Handle regular paragraphs
            else if !line.trim().is_empty() {
                html.push_str(&format!(
                    "<p>{}</p>\n",
                    self.process_inline_formatting(line.trim())
                ));
            }
        }

        Ok(html)
    }

    /// Process inline formatting like *bold*, _italic_, etc.
    fn process_inline_formatting(&self, text: &str) -> String {
        let mut result = text.to_string();

        // Bold text: *text* -> <strong>text</strong>
        result = regex::Regex::new(r"\*([^*]+)\*")
            .unwrap()
            .replace_all(&result, "<strong>$1</strong>")
            .to_string();

        // Italic text: _text_ -> <em>text</em>
        result = regex::Regex::new(r"_([^_]+)_")
            .unwrap()
            .replace_all(&result, "<em>$1</em>")
            .to_string();

        // Inline code: `code` -> <code>code</code>
        result = regex::Regex::new(r"`([^`]+)`")
            .unwrap()
            .replace_all(&result, r#"<code class="inline-code">$1</code>"#)
            .to_string();

        // Links: [text](url) -> <a href="url">text</a>
        result = regex::Regex::new(r"\[([^\]]+)\]\(([^)]+)\)")
            .unwrap()
            .replace_all(&result, r#"<a href="$2">$1</a>"#)
            .to_string();

        result
    }
}
