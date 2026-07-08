//! Shared string utilities for HTML escaping, slugification, and HTML stripping.

/// Escape HTML special characters.
///
/// Converts `&`, `<`, `>`, `"`, and `'` to their HTML entity equivalents.
#[must_use]
pub fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

/// Convert text to a URL-safe slug.
///
/// Lowercases the text, replaces whitespace and special characters with hyphens,
/// and collapses consecutive hyphens.
#[must_use]
pub fn slugify(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c
            } else if c.is_whitespace() || c == '-' || c == '_' {
                '-'
            } else {
                '\0'
            }
        })
        .filter(|c| *c != '\0')
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// Strip HTML tags from content, handling script/style blocks and HTML entities.
///
/// This is the most comprehensive version: it skips `<script>` and `<style>` blocks,
/// decodes common HTML entities, and collapses whitespace.
#[must_use]
pub fn strip_html(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut in_tag = false;
    let mut in_script = false;
    let mut in_style = false;

    let html_lower = html.to_lowercase();
    let chars: Vec<char> = html.chars().collect();
    let chars_lower: Vec<char> = html_lower.chars().collect();

    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];

        // Check for script/style start
        if i + 7 < chars.len() {
            let next_7: String = chars_lower[i..i + 7].iter().collect();
            if next_7 == "<script" {
                in_script = true;
            } else if next_7 == "</scrip" {
                in_script = false;
            }
        }

        if i + 6 < chars.len() {
            let next_6: String = chars_lower[i..i + 6].iter().collect();
            if next_6 == "<style" {
                in_style = true;
            } else if next_6 == "</styl" {
                in_style = false;
            }
        }

        if c == '<' {
            in_tag = true;
        } else if c == '>' {
            in_tag = false;
        } else if !in_tag && !in_script && !in_style {
            result.push(c);
        }

        i += 1;
    }

    // Decode common HTML entities
    result = result
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'");

    // Collapse multiple whitespace
    let mut collapsed = String::with_capacity(result.len());
    let mut prev_space = false;
    for c in result.chars() {
        if c.is_whitespace() {
            if !prev_space {
                collapsed.push(' ');
                prev_space = true;
            }
        } else {
            collapsed.push(c);
            prev_space = false;
        }
    }

    collapsed.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_escape() {
        assert_eq!(html_escape("a & b"), "a &amp; b");
        assert_eq!(html_escape("<div>"), "&lt;div&gt;");
        assert_eq!(html_escape(r#"a "b" c"#), "a &quot;b&quot; c");
        assert_eq!(html_escape("it's"), "it&#x27;s");
    }

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Hello World"), "hello-world");
        assert_eq!(slugify("foo_bar"), "foo-bar");
        assert_eq!(slugify("  multiple   spaces  "), "multiple-spaces");
        assert_eq!(slugify("special!@#chars"), "specialchars");
    }

    #[test]
    fn test_strip_html() {
        assert_eq!(strip_html("<p>Hello</p>"), "Hello");
        assert_eq!(strip_html("<script>alert(1)</script>"), "");
        assert_eq!(
            strip_html("<b>bold</b> and <i>italic</i>"),
            "bold and italic"
        );
        assert_eq!(strip_html("a &amp; b"), "a & b");
        assert_eq!(
            strip_html("  multiple   <br>   spaces  "),
            "multiple spaces"
        );
    }
}
