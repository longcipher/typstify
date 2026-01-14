//! HTTP directory implementation for loading chunked search indexes.
//!
//! Implements a virtual directory that fetches chunks on-demand from an HTTP server.

use std::{collections::HashMap as StdHashMap, sync::Arc};

use gloo_net::http::Request;
use scc::HashMap;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

/// Manifest describing the chunked index structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexManifest {
    /// Manifest format version.
    pub version: u32,

    /// Chunk size used for splitting.
    pub chunk_size: usize,

    /// Total size of all chunks.
    pub total_size: u64,

    /// Files and their chunks.
    pub files: StdHashMap<String, FileManifest>,
}

/// Manifest for a single file's chunks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileManifest {
    /// Original file size.
    pub size: usize,

    /// List of chunk filenames.
    pub chunks: Vec<String>,
}

/// Error type for HTTP directory operations.
#[derive(Debug)]
pub enum DirectoryError {
    /// Network error.
    Network(String),
    /// Parse error.
    Parse(String),
    /// Chunk not found.
    NotFound(String),
}

impl std::fmt::Display for DirectoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DirectoryError::Network(e) => write!(f, "Network error: {e}"),
            DirectoryError::Parse(e) => write!(f, "Parse error: {e}"),
            DirectoryError::NotFound(e) => write!(f, "Not found: {e}"),
        }
    }
}

impl From<DirectoryError> for JsValue {
    fn from(err: DirectoryError) -> Self {
        JsValue::from_str(&err.to_string())
    }
}

/// HTTP directory for loading chunked search indexes.
///
/// Caches fetched chunks in memory to avoid redundant network requests.
#[derive(Clone)]
pub struct HttpDirectory {
    /// Base URL for fetching chunks.
    base_url: String,

    /// Index manifest.
    manifest: Arc<IndexManifest>,

    /// Cache of loaded chunks: chunk_name -> data.
    chunk_cache: Arc<HashMap<String, Vec<u8>>>,
}

impl HttpDirectory {
    /// Create a new HTTP directory by loading the manifest.
    pub async fn new(base_url: &str) -> Result<Self, DirectoryError> {
        let manifest_url = format!("{}/manifest.json", base_url.trim_end_matches('/'));

        let response = Request::get(&manifest_url)
            .send()
            .await
            .map_err(|e| DirectoryError::Network(e.to_string()))?;

        if !response.ok() {
            return Err(DirectoryError::Network(format!(
                "Failed to fetch manifest: HTTP {}",
                response.status()
            )));
        }

        let manifest_text = response
            .text()
            .await
            .map_err(|e| DirectoryError::Network(e.to_string()))?;

        let manifest: IndexManifest = serde_json::from_str(&manifest_text)
            .map_err(|e| DirectoryError::Parse(e.to_string()))?;

        Ok(Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            manifest: Arc::new(manifest),
            chunk_cache: Arc::new(HashMap::new()),
        })
    }

    /// Create a directory with a pre-loaded manifest (for testing).
    pub fn with_manifest(base_url: &str, manifest: IndexManifest) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            manifest: Arc::new(manifest),
            chunk_cache: Arc::new(HashMap::new()),
        }
    }

    /// Get the manifest.
    pub fn manifest(&self) -> &IndexManifest {
        &self.manifest
    }

    /// Load data for a file from the index.
    ///
    /// Fetches required chunks and concatenates them.
    pub async fn load_file(&self, filename: &str) -> Result<Vec<u8>, DirectoryError> {
        let file_manifest =
            self.manifest.files.get(filename).ok_or_else(|| {
                DirectoryError::NotFound(format!("File not in manifest: {filename}"))
            })?;

        let mut data = Vec::with_capacity(file_manifest.size);

        for chunk_name in &file_manifest.chunks {
            let chunk_data = self.load_chunk(chunk_name).await?;
            data.extend(chunk_data);
        }

        // Trim to exact file size (last chunk may have padding)
        data.truncate(file_manifest.size);

        Ok(data)
    }

    /// Load a byte range from a file.
    ///
    /// Calculates which chunks are needed and fetches only those.
    pub async fn load_range(
        &self,
        filename: &str,
        start: usize,
        end: usize,
    ) -> Result<Vec<u8>, DirectoryError> {
        let file_manifest =
            self.manifest.files.get(filename).ok_or_else(|| {
                DirectoryError::NotFound(format!("File not in manifest: {filename}"))
            })?;

        if start >= file_manifest.size || end > file_manifest.size || start >= end {
            return Err(DirectoryError::NotFound(format!(
                "Invalid range: {}-{} for file of size {}",
                start, end, file_manifest.size
            )));
        }

        let chunk_size = self.manifest.chunk_size;
        let start_chunk_idx = start / chunk_size;
        let end_chunk_idx = (end - 1) / chunk_size;

        // Load required chunks
        let mut full_data = Vec::new();
        for idx in start_chunk_idx..=end_chunk_idx {
            if idx < file_manifest.chunks.len() {
                let chunk_name = &file_manifest.chunks[idx];
                let chunk_data = self.load_chunk(chunk_name).await?;
                full_data.extend(chunk_data);
            }
        }

        // Calculate offset within the loaded data
        let data_start = start - (start_chunk_idx * chunk_size);
        let data_end = data_start + (end - start);

        if data_end > full_data.len() {
            return Err(DirectoryError::NotFound(
                "Range exceeds available data".to_string(),
            ));
        }

        Ok(full_data[data_start..data_end].to_vec())
    }

    /// Load a single chunk, using cache if available.
    async fn load_chunk(&self, chunk_name: &str) -> Result<Vec<u8>, DirectoryError> {
        // Check cache first
        if let Some(entry) = self.chunk_cache.get_async(chunk_name).await {
            return Ok(entry.get().clone());
        }

        // Fetch from network
        let chunk_url = format!("{}/{}", self.base_url, chunk_name);

        let response = Request::get(&chunk_url)
            .send()
            .await
            .map_err(|e| DirectoryError::Network(e.to_string()))?;

        if !response.ok() {
            return Err(DirectoryError::Network(format!(
                "Failed to fetch chunk {}: HTTP {}",
                chunk_name,
                response.status()
            )));
        }

        let bytes = response
            .binary()
            .await
            .map_err(|e| DirectoryError::Network(e.to_string()))?;

        // Cache the chunk
        let _ = self
            .chunk_cache
            .insert_async(chunk_name.to_string(), bytes.clone())
            .await;

        Ok(bytes)
    }

    /// Get the number of cached chunks.
    pub fn cached_chunk_count(&self) -> usize {
        self.chunk_cache.len()
    }

    /// Clear the chunk cache.
    pub fn clear_cache(&self) {
        self.chunk_cache.clear_sync();
    }

    /// Get total size of all files.
    pub fn total_size(&self) -> u64 {
        self.manifest.total_size
    }

    /// List all files in the manifest.
    pub fn list_files(&self) -> Vec<&str> {
        self.manifest.files.keys().map(|s| s.as_str()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_manifest() -> IndexManifest {
        let mut files = StdHashMap::new();
        files.insert(
            "test.bin".to_string(),
            FileManifest {
                size: 1000,
                chunks: vec!["chunk_0000.bin".to_string(), "chunk_0001.bin".to_string()],
            },
        );

        IndexManifest {
            version: 1,
            chunk_size: 512,
            total_size: 1000,
            files,
        }
    }

    #[test]
    fn test_manifest_deserialization() {
        let json = r#"{
            "version": 1,
            "chunk_size": 65536,
            "total_size": 100000,
            "files": {
                "data.bin": {
                    "size": 100000,
                    "chunks": ["chunk_0000.bin", "chunk_0001.bin"]
                }
            }
        }"#;

        let manifest: IndexManifest = serde_json::from_str(json).unwrap();
        assert_eq!(manifest.version, 1);
        assert_eq!(manifest.chunk_size, 65536);
        assert!(manifest.files.contains_key("data.bin"));
    }

    #[test]
    fn test_with_manifest() {
        let manifest = create_test_manifest();
        let dir = HttpDirectory::with_manifest("https://example.com/search", manifest);

        assert_eq!(dir.total_size(), 1000);
        assert_eq!(dir.list_files().len(), 1);
        assert!(dir.list_files().contains(&"test.bin"));
    }

    #[test]
    fn test_directory_error_display() {
        let err = DirectoryError::Network("connection refused".to_string());
        assert!(err.to_string().contains("Network error"));

        let err = DirectoryError::NotFound("file.bin".to_string());
        assert!(err.to_string().contains("Not found"));
    }
}
