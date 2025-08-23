use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ContentId(String);

impl ContentId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn from_path(path: &Path) -> Self {
        let path_str = path.to_string_lossy();

        // Remove file extension
        let without_ext = if let Some(stem) = path.file_stem() {
            stem.to_string_lossy().to_string()
        } else {
            path_str.to_string()
        };

        // Convert to slug format
        let slug = Self::to_slug(&without_ext);
        Self(slug)
    }

    pub fn from_relative_path(base_dir: &Path, full_path: &Path) -> Self {
        if let Ok(relative) = full_path.strip_prefix(base_dir) {
            let path_str = relative.to_string_lossy();

            // Remove file extension
            let without_ext = if let Some(dot_pos) = path_str.rfind('.') {
                &path_str[..dot_pos]
            } else {
                &path_str
            };

            // Keep path separators as slashes for URL paths
            let normalized = without_ext.replace('\\', "/");
            Self(normalized)
        } else {
            Self::from_path(full_path)
        }
    }

    pub fn from_frontmatter_slug(slug: &str) -> Self {
        Self(Self::to_slug(slug))
    }

    fn to_slug(input: &str) -> String {
        input
            .to_lowercase()
            .chars()
            .map(|c| {
                if c.is_alphanumeric() {
                    c
                } else if c.is_whitespace() || c == '-' || c == '_' {
                    '-'
                } else {
                    // Skip other characters
                    '\0'
                }
            })
            .filter(|&c| c != '\0')
            .collect::<String>()
            .split('-')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("-")
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn to_url_path(&self) -> String {
        if self.0.starts_with('/') {
            self.0.clone()
        } else {
            format!("/{}", self.0)
        }
    }

    pub fn to_file_name(&self, extension: &str) -> String {
        format!("{}.{}", self.0, extension)
    }

    /// Generate a content ID that's suitable for use in URLs
    pub fn to_url_safe(&self) -> String {
        self.0.clone()
    }

    /// Check if the content ID is valid (non-empty, URL-safe)
    pub fn is_valid(&self) -> bool {
        !self.0.is_empty() && self.0.chars().all(|c| c.is_alphanumeric() || c == '-')
    }
}

impl std::fmt::Display for ContentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for ContentId {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<&str> for ContentId {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl AsRef<str> for ContentId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn test_from_path() {
        let path = PathBuf::from("content/blog/my-first-post.md");
        let id = ContentId::from_path(&path);
        assert_eq!(id.as_str(), "my-first-post");
    }

    #[test]
    fn test_from_relative_path() {
        let base = PathBuf::from("content");
        let full = PathBuf::from("content/blog/getting-started/installation.md");
        let id = ContentId::from_relative_path(&base, &full);
        assert_eq!(id.as_str(), "blog-getting-started-installation");
    }

    #[test]
    fn test_to_slug() {
        assert_eq!(ContentId::to_slug("Hello World!"), "hello-world");
        assert_eq!(ContentId::to_slug("My_Cool-Post"), "my-cool-post");
        assert_eq!(
            ContentId::to_slug("Special@#$Characters"),
            "specialcharacters"
        );
        assert_eq!(ContentId::to_slug("Multiple   Spaces"), "multiple-spaces");
    }

    #[test]
    fn test_url_path() {
        let id = ContentId::new("my-post");
        assert_eq!(id.to_url_path(), "/my-post");
    }

    #[test]
    fn test_is_valid() {
        assert!(ContentId::new("valid-post").is_valid());
        assert!(ContentId::new("123-valid").is_valid());
        assert!(!ContentId::new("").is_valid());
        assert!(!ContentId::new("invalid@post").is_valid());
    }
}
