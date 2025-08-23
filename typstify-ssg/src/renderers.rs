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
        let mut in_list = false;
        let mut current_list_items = Vec::new();

        for (i, line) in lines.iter().enumerate() {
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

            let line_trimmed = line.trim();

            // Handle list items
            if line_trimmed.starts_with("- ") {
                let text = line_trimmed.strip_prefix("- ").unwrap_or("").trim();
                current_list_items.push(text.to_string());
                in_list = true;
                continue;
            } else if in_list {
                // End of list, output all items
                html.push_str("<ul>");
                for item in &current_list_items {
                    html.push_str(&format!(
                        "<li>{}</li>",
                        self.process_inline_formatting(item)
                    ));
                }
                html.push_str("</ul>\n");
                current_list_items.clear();
                in_list = false;
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
            // Handle empty lines - only add spacing between content blocks
            else if line_trimmed.is_empty() {
                // Look ahead to see if next line has content
                let next_line_has_content = lines
                    .get(i + 1)
                    .map(|next_line| {
                        !next_line.trim().is_empty() && !next_line.trim_start().starts_with("//")
                    })
                    .unwrap_or(false);

                // Only add spacing if there's content after this empty line
                if next_line_has_content && !html.ends_with("\n\n") {
                    html.push('\n');
                }
            }
            // Handle inline code blocks (single line with backticks)
            else if line_trimmed.starts_with("`")
                && line_trimmed.ends_with("`")
                && line_trimmed.len() > 1
            {
                let code = line_trimmed
                    .strip_prefix("`")
                    .unwrap()
                    .strip_suffix("`")
                    .unwrap();
                html.push_str(&format!(
                    "<p><code class=\"inline-code\">{}</code></p>\n",
                    code
                ));
            }
            // Handle regular paragraphs
            else if !line_trimmed.is_empty() {
                html.push_str(&format!(
                    "<p>{}</p>\n",
                    self.process_inline_formatting(line_trimmed)
                ));
            }
        }

        // Handle any remaining list items
        if in_list && !current_list_items.is_empty() {
            html.push_str("<ul>");
            for item in &current_list_items {
                html.push_str(&format!(
                    "<li>{}</li>",
                    self.process_inline_formatting(item)
                ));
            }
            html.push_str("</ul>\n");
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
