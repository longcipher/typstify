//! Content renderers for Markdown and Typst files

use eyre::Result;
use std::path::PathBuf;

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

/// Typst renderer using official typst crate with simplified HTML conversion
pub struct TypstRenderer {
    /// Root path for resolving imports and assets
    #[allow(dead_code)]
    root_path: PathBuf,
}

impl TypstRenderer {
    pub fn new() -> Self {
        Self {
            root_path: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        }
    }

    pub fn with_root_path(root_path: PathBuf) -> Self {
        Self { root_path }
    }
}

impl Default for TypstRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl Renderer for TypstRenderer {
    fn render(&self, content: &str) -> Result<String, RendererError> {
        // For now, use a hybrid approach:
        // 1. Try to parse with typst for validation
        // 2. Use improved text-to-HTML conversion
        self.convert_typst_to_html_improved(content)
    }
}

impl TypstRenderer {
    /// Enhanced Typst to HTML conversion with better syntax support
    fn convert_typst_to_html_improved(&self, content: &str) -> Result<String, RendererError> {
        // First, let's validate the Typst syntax
        if let Err(e) = self.validate_typst_syntax(content) {
            eprintln!("Typst syntax warning: {}", e);
            // Continue with conversion even if validation fails
        }

        let lines: Vec<&str> = content.lines().collect();
        let mut html = String::new();
        let mut in_code_block = false;
        let mut code_language = String::new();
        let mut list_stack: Vec<String> = Vec::new(); // Track nested lists

        html.push_str(r#"<div class="typst-content">"#);

        for (line_num, line) in lines.iter().enumerate() {
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
                let escaped = html_escape(line);
                html.push_str(&escaped);
                html.push('\n');
                continue;
            }

            // Close any open lists if we hit a non-list line
            if !line.trim_start().starts_with("- ") && !list_stack.is_empty() {
                while let Some(list_type) = list_stack.pop() {
                    html.push_str(&format!("</{}>", list_type));
                }
            }

            // Handle headings (Typst style)
            if line.starts_with("====") {
                let text = line.strip_prefix("====").unwrap_or("").trim();
                html.push_str(&format!(
                    "<h4 class=\"typst-heading-4\">{}</h4>\n",
                    self.process_inline_formatting(text)
                ));
            } else if line.starts_with("===") {
                let text = line.strip_prefix("===").unwrap_or("").trim();
                html.push_str(&format!(
                    "<h3 class=\"typst-heading-3\">{}</h3>\n",
                    self.process_inline_formatting(text)
                ));
            } else if line.starts_with("==") {
                let text = line.strip_prefix("==").unwrap_or("").trim();
                html.push_str(&format!(
                    "<h2 class=\"typst-heading-2\">{}</h2>\n",
                    self.process_inline_formatting(text)
                ));
            } else if line.starts_with("=") {
                let text = line.strip_prefix("=").unwrap_or("").trim();
                html.push_str(&format!(
                    "<h1 class=\"typst-heading-1\">{}</h1>\n",
                    self.process_inline_formatting(text)
                ));
            }
            // Handle list items with proper nesting
            else if line.trim_start().starts_with("- ") {
                let indent_level = (line.len() - line.trim_start().len()) / 2;
                let text = line.trim_start().strip_prefix("- ").unwrap_or("").trim();

                // Handle list nesting
                while list_stack.len() > indent_level {
                    if let Some(list_type) = list_stack.pop() {
                        html.push_str(&format!("</{}>", list_type));
                    }
                }

                if list_stack.len() <= indent_level {
                    html.push_str("<ul class=\"typst-list\">");
                    list_stack.push("ul".to_string());
                }

                html.push_str(&format!(
                    "<li class=\"typst-list-item\">{}</li>\n",
                    self.process_inline_formatting(text)
                ));
            }
            // Handle numbered lists (simple)
            else if line.trim_start().matches(char::is_numeric).count() > 0 
                && line.trim_start().contains(". ") 
            {
                if let Some(dot_pos) = line.find(". ") {
                    let text = &line[dot_pos + 2..];
                    html.push_str(&format!(
                        "<ol class=\"typst-ordered-list\"><li class=\"typst-list-item\">{}</li></ol>\n",
                        self.process_inline_formatting(text)
                    ));
                }
            }
            // Handle inline code blocks
            else if line.trim().starts_with("`")
                && line.trim().ends_with("`")
                && line.trim().len() > 1
                && !line.contains("```")
            {
                let code = line
                    .trim()
                    .strip_prefix("`")
                    .unwrap()
                    .strip_suffix("`")
                    .unwrap();
                html.push_str(&format!(
                    "<p><code class=\"typst-inline-code\">{}</code></p>\n",
                    html_escape(code)
                ));
            }
            // Handle blockquotes
            else if line.trim_start().starts_with("> ") {
                let quote_text = line.trim_start().strip_prefix("> ").unwrap_or("");
                html.push_str(&format!(
                    "<blockquote class=\"typst-blockquote\"><p>{}</p></blockquote>\n",
                    self.process_inline_formatting(quote_text)
                ));
            }
            // Handle horizontal rules
            else if line.trim() == "---" || line.trim() == "***" {
                html.push_str("<hr class=\"typst-hr\">\n");
            }
            // Handle empty lines
            else if line.trim().is_empty() {
                // Look ahead to see if this is a paragraph break
                if line_num + 1 < lines.len() && !lines[line_num + 1].trim().is_empty() {
                    html.push_str("<br>\n");
                }
            }
            // Handle regular paragraphs
            else if !line.trim().is_empty() {
                html.push_str(&format!(
                    "<p class=\"typst-paragraph\">{}</p>\n",
                    self.process_inline_formatting(line.trim())
                ));
            }
        }

        // Close any remaining open lists
        while let Some(list_type) = list_stack.pop() {
            html.push_str(&format!("</{}>", list_type));
        }

        html.push_str("</div>");

        Ok(html)
    }

    /// Validate Typst syntax using the official parser
    fn validate_typst_syntax(&self, content: &str) -> Result<(), RendererError> {
        use typst_syntax::{Source, VirtualPath, FileId};

        #[allow(clippy::typos)]
        let path = VirtualPath::new("validation.typ");
        let id = FileId::new(None, path);
        let source = Source::new(id, content.to_string());
        
        // Parse the source to check for syntax errors
        let parsed = typst_syntax::parse(source.text());
        
        // Check for errors in the parsed result
        if parsed.errors().is_empty() {
            Ok(())
        } else {
            let error_messages: Vec<String> = parsed.errors()
                .iter()
                .map(|e| format!("{:?}", e))
                .collect();
            Err(RendererError::TypstError(format!(
                "Syntax errors: {}",
                error_messages.join("; ")
            )))
        }
    }

    /// Enhanced inline formatting processor
    fn process_inline_formatting(&self, text: &str) -> String {
        let mut result = text.to_string();

        // Strong text: *text* -> <strong>text</strong>
        result = regex::Regex::new(r"\*([^*]+)\*")
            .unwrap()
            .replace_all(&result, "<strong class=\"typst-strong\">$1</strong>")
            .to_string();

        // Emphasis: _text_ -> <em>text</em>
        result = regex::Regex::new(r"_([^_]+)_")
            .unwrap()
            .replace_all(&result, "<em class=\"typst-emphasis\">$1</em>")
            .to_string();

        // Inline code: `code` -> <code>code</code>
        result = regex::Regex::new(r"`([^`]+)`")
            .unwrap()
            .replace_all(&result, r#"<code class="typst-inline-code">$1</code>"#)
            .to_string();

        // Links: [text](url) -> <a href="url">text</a>
        result = regex::Regex::new(r"\[([^\]]+)\]\(([^)]+)\)")
            .unwrap()
            .replace_all(&result, r#"<a href="$2" class="typst-link">$1</a>"#)
            .to_string();

        // Math inline: $formula$ -> <span class="math">formula</span>
        result = regex::Regex::new(r"\$([^$]+)\$")
            .unwrap()
            .replace_all(&result, r#"<span class="typst-math-inline">$1</span>"#)
            .to_string();

        result
    }
}

/// Simple HTML escaping function
fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}
