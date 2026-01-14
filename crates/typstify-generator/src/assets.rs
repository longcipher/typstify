//! Asset processing and management.
//!
//! Handles copying static assets and optional fingerprinting for cache busting.

use std::{
    collections::HashMap,
    fs,
    io::Read,
    path::{Path, PathBuf},
};

use thiserror::Error;
use tracing::{debug, info};

/// Asset processing errors.
#[derive(Debug, Error)]
pub enum AssetError {
    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Invalid asset path.
    #[error("invalid asset path: {0}")]
    InvalidPath(PathBuf),
}

/// Result type for asset operations.
pub type Result<T> = std::result::Result<T, AssetError>;

/// Asset manifest for tracking processed assets.
#[derive(Debug, Clone, Default)]
pub struct AssetManifest {
    /// Mapping from original path to fingerprinted path.
    assets: HashMap<String, String>,
}

impl AssetManifest {
    /// Create a new empty manifest.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an asset to the manifest.
    pub fn add(&mut self, original: impl Into<String>, fingerprinted: impl Into<String>) {
        self.assets.insert(original.into(), fingerprinted.into());
    }

    /// Get the fingerprinted path for an asset.
    #[must_use]
    pub fn get(&self, original: &str) -> Option<&str> {
        self.assets.get(original).map(String::as_str)
    }

    /// Get all assets in the manifest.
    #[must_use]
    pub fn assets(&self) -> &HashMap<String, String> {
        &self.assets
    }

    /// Serialize manifest to JSON.
    pub fn to_json(&self) -> String {
        let mut json = String::from("{\n");
        let entries: Vec<_> = self.assets.iter().collect();
        for (i, (orig, fp)) in entries.iter().enumerate() {
            json.push_str(&format!(r#"  "{orig}": "{fp}""#));
            if i < entries.len() - 1 {
                json.push(',');
            }
            json.push('\n');
        }
        json.push('}');
        json
    }
}

/// Asset processor for copying and optionally fingerprinting static files.
#[derive(Debug)]
pub struct AssetProcessor {
    /// Whether to fingerprint assets.
    fingerprint: bool,

    /// File extensions to fingerprint.
    fingerprint_extensions: Vec<String>,
}

impl AssetProcessor {
    /// Create a new asset processor.
    #[must_use]
    pub fn new(fingerprint: bool) -> Self {
        Self {
            fingerprint,
            fingerprint_extensions: vec![
                "css".to_string(),
                "js".to_string(),
                "woff".to_string(),
                "woff2".to_string(),
                "png".to_string(),
                "jpg".to_string(),
                "jpeg".to_string(),
                "gif".to_string(),
                "svg".to_string(),
                "webp".to_string(),
            ],
        }
    }

    /// Set which extensions should be fingerprinted.
    #[must_use]
    pub fn with_fingerprint_extensions(mut self, extensions: Vec<String>) -> Self {
        self.fingerprint_extensions = extensions;
        self
    }

    /// Process all assets from source to destination directory.
    pub fn process(&self, source_dir: &Path, dest_dir: &Path) -> Result<AssetManifest> {
        info!(
            source = %source_dir.display(),
            dest = %dest_dir.display(),
            "processing assets"
        );

        let mut manifest = AssetManifest::new();

        if !source_dir.exists() {
            debug!("source directory does not exist, skipping");
            return Ok(manifest);
        }

        self.process_dir(source_dir, source_dir, dest_dir, &mut manifest)?;

        info!(count = manifest.assets.len(), "assets processed");
        Ok(manifest)
    }

    /// Recursively process a directory.
    fn process_dir(
        &self,
        base_dir: &Path,
        current_dir: &Path,
        dest_base: &Path,
        manifest: &mut AssetManifest,
    ) -> Result<()> {
        for entry in fs::read_dir(current_dir)? {
            let entry = entry?;
            let path = entry.path();

            // Skip hidden files/directories
            if path
                .file_name()
                .is_some_and(|n| n.to_string_lossy().starts_with('.'))
            {
                continue;
            }

            if path.is_dir() {
                self.process_dir(base_dir, &path, dest_base, manifest)?;
            } else if path.is_file() {
                self.process_file(base_dir, &path, dest_base, manifest)?;
            }
        }

        Ok(())
    }

    /// Process a single file.
    fn process_file(
        &self,
        base_dir: &Path,
        file_path: &Path,
        dest_base: &Path,
        manifest: &mut AssetManifest,
    ) -> Result<()> {
        let relative = file_path
            .strip_prefix(base_dir)
            .map_err(|_| AssetError::InvalidPath(file_path.to_path_buf()))?;

        let should_fingerprint = self.fingerprint
            && file_path.extension().is_some_and(|ext| {
                self.fingerprint_extensions
                    .contains(&ext.to_string_lossy().to_string())
            });

        let dest_relative = if should_fingerprint {
            let hash = self.compute_hash(file_path)?;
            let stem = file_path.file_stem().unwrap_or_default().to_string_lossy();
            let ext = file_path.extension().unwrap_or_default().to_string_lossy();

            let fingerprinted_name = format!("{stem}.{hash}.{ext}");
            let parent = relative.parent().unwrap_or(Path::new(""));
            parent.join(&fingerprinted_name)
        } else {
            relative.to_path_buf()
        };

        let dest_path = dest_base.join(&dest_relative);

        // Ensure destination directory exists
        if let Some(parent) = dest_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Copy the file
        fs::copy(file_path, &dest_path)?;

        // Add to manifest
        let orig_path = format!("/{}", relative.display()).replace('\\', "/");
        let dest_path_str = format!("/{}", dest_relative.display()).replace('\\', "/");
        manifest.add(orig_path, dest_path_str);

        debug!(
            src = %file_path.display(),
            dest = %dest_path.display(),
            "copied asset"
        );

        Ok(())
    }

    /// Compute a short hash of file contents for fingerprinting.
    fn compute_hash(&self, path: &Path) -> Result<String> {
        let mut file = fs::File::open(path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        // Simple hash using FNV-1a
        let mut hash: u64 = 0xcbf29ce484222325;
        for byte in &buffer {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x100000001b3);
        }

        // Return first 8 hex characters
        Ok(format!("{hash:016x}")[..8].to_string())
    }

    /// Copy a single file without fingerprinting.
    pub fn copy_file(source: &Path, dest: &Path) -> Result<()> {
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(source, dest)?;
        Ok(())
    }

    /// Create a directory if it doesn't exist.
    pub fn ensure_dir(path: &Path) -> Result<()> {
        if !path.exists() {
            fs::create_dir_all(path)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use tempfile::TempDir;

    use super::*;

    #[test]
    fn test_asset_manifest() {
        let mut manifest = AssetManifest::new();
        manifest.add("/css/style.css", "/css/style.abc12345.css");
        manifest.add("/js/main.js", "/js/main.def67890.js");

        assert_eq!(
            manifest.get("/css/style.css"),
            Some("/css/style.abc12345.css")
        );
        assert_eq!(manifest.get("/js/main.js"), Some("/js/main.def67890.js"));
        assert!(manifest.get("/other.txt").is_none());
    }

    #[test]
    fn test_manifest_to_json() {
        let mut manifest = AssetManifest::new();
        manifest.add("/style.css", "/style.abc.css");

        let json = manifest.to_json();
        assert!(json.contains(r#""/style.css": "/style.abc.css""#));
    }

    #[test]
    fn test_process_assets() {
        let source = TempDir::new().unwrap();
        let dest = TempDir::new().unwrap();

        // Create test files
        let css_path = source.path().join("style.css");
        let mut css_file = fs::File::create(&css_path).unwrap();
        css_file.write_all(b"body { color: red; }").unwrap();

        let txt_path = source.path().join("readme.txt");
        let mut txt_file = fs::File::create(&txt_path).unwrap();
        txt_file.write_all(b"Hello world").unwrap();

        // Process without fingerprinting
        let processor = AssetProcessor::new(false);
        let manifest = processor.process(source.path(), dest.path()).unwrap();

        assert!(dest.path().join("style.css").exists());
        assert!(dest.path().join("readme.txt").exists());
        assert_eq!(manifest.assets().len(), 2);
    }

    #[test]
    fn test_process_with_fingerprinting() {
        let source = TempDir::new().unwrap();
        let dest = TempDir::new().unwrap();

        // Create a CSS file
        let css_path = source.path().join("style.css");
        let mut css_file = fs::File::create(&css_path).unwrap();
        css_file.write_all(b"body { color: blue; }").unwrap();

        // Process with fingerprinting
        let processor = AssetProcessor::new(true);
        let manifest = processor.process(source.path(), dest.path()).unwrap();

        // Original file should map to fingerprinted version
        let fingerprinted = manifest.get("/style.css").unwrap();
        assert!(fingerprinted.starts_with("/style."));
        assert!(fingerprinted.ends_with(".css"));
        assert!(fingerprinted.len() > "/style.css".len());
    }

    #[test]
    fn test_compute_hash_deterministic() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.txt");
        let mut file = fs::File::create(&path).unwrap();
        file.write_all(b"test content").unwrap();
        drop(file);

        let processor = AssetProcessor::new(true);
        let hash1 = processor.compute_hash(&path).unwrap();
        let hash2 = processor.compute_hash(&path).unwrap();

        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 8);
    }

    #[test]
    fn test_ensure_dir() {
        let dir = TempDir::new().unwrap();
        let nested = dir.path().join("a/b/c");

        assert!(!nested.exists());
        AssetProcessor::ensure_dir(&nested).unwrap();
        assert!(nested.exists());
    }
}
