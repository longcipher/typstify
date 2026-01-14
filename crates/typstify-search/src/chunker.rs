//! Index chunking for efficient client-side loading.
//!
//! Splits large search indexes into smaller chunks that can be loaded incrementally
//! in the browser.

use std::{collections::HashMap, fs, path::Path};

use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use crate::SearchError;

/// Default chunk size (64KB).
pub const DEFAULT_CHUNK_SIZE: usize = 64 * 1024;

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
    pub files: HashMap<String, FileManifest>,
}

impl IndexManifest {
    /// Create a new empty manifest.
    pub fn new(chunk_size: usize) -> Self {
        Self {
            version: 1,
            chunk_size,
            total_size: 0,
            files: HashMap::new(),
        }
    }

    /// Serialize the manifest to JSON.
    pub fn to_json(&self) -> Result<String, SearchError> {
        serde_json::to_string_pretty(self).map_err(|e| SearchError::Serialization(e.to_string()))
    }

    /// Deserialize a manifest from JSON.
    pub fn from_json(json: &str) -> Result<Self, SearchError> {
        serde_json::from_str(json).map_err(|e| SearchError::Serialization(e.to_string()))
    }
}

/// Manifest for a single file's chunks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileManifest {
    /// Original file size.
    pub size: usize,

    /// List of chunk filenames.
    pub chunks: Vec<String>,
}

/// Configuration for the index chunker.
#[derive(Debug, Clone)]
pub struct ChunkerConfig {
    /// Maximum size of each chunk in bytes.
    pub chunk_size: usize,

    /// Prefix for chunk filenames.
    pub chunk_prefix: String,
}

impl Default for ChunkerConfig {
    fn default() -> Self {
        Self {
            chunk_size: DEFAULT_CHUNK_SIZE,
            chunk_prefix: "chunk".to_string(),
        }
    }
}

/// Index chunker for splitting large index files.
#[derive(Debug)]
pub struct IndexChunker {
    config: ChunkerConfig,
}

impl IndexChunker {
    /// Create a new chunker with the given configuration.
    pub fn new(config: ChunkerConfig) -> Self {
        Self { config }
    }

    /// Create a chunker with default configuration.
    pub fn with_defaults() -> Self {
        Self::new(ChunkerConfig::default())
    }

    /// Chunk all files in the source directory and write to output directory.
    ///
    /// Returns a manifest describing the chunked files.
    pub fn chunk_directory(
        &self,
        source_dir: &Path,
        output_dir: &Path,
    ) -> Result<IndexManifest, SearchError> {
        // Create output directory
        fs::create_dir_all(output_dir).map_err(|e| SearchError::Io(e.to_string()))?;

        let mut manifest = IndexManifest::new(self.config.chunk_size);
        let mut chunk_counter = 0;

        // Process all files in the source directory
        let entries = fs::read_dir(source_dir).map_err(|e| SearchError::Io(e.to_string()))?;

        for entry in entries {
            let entry = entry.map_err(|e| SearchError::Io(e.to_string()))?;
            let path = entry.path();

            if path.is_file() {
                let filename = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .ok_or_else(|| SearchError::Io("Invalid filename".to_string()))?
                    .to_string();

                let file_manifest = self.chunk_file(&path, output_dir, &mut chunk_counter)?;

                manifest.total_size += file_manifest.size as u64;
                manifest.files.insert(filename, file_manifest);
            }
        }

        info!(
            files = manifest.files.len(),
            total_size = manifest.total_size,
            "Chunked index files"
        );

        Ok(manifest)
    }

    /// Chunk a single file into smaller pieces.
    fn chunk_file(
        &self,
        source: &Path,
        output_dir: &Path,
        chunk_counter: &mut usize,
    ) -> Result<FileManifest, SearchError> {
        let data = fs::read(source).map_err(|e| SearchError::Io(e.to_string()))?;
        let size = data.len();

        // If file is small enough, don't chunk it
        if size <= self.config.chunk_size {
            let chunk_name = format!("{}_{:04}.bin", self.config.chunk_prefix, chunk_counter);
            *chunk_counter += 1;

            let chunk_path = output_dir.join(&chunk_name);
            fs::write(&chunk_path, &data).map_err(|e| SearchError::Io(e.to_string()))?;

            debug!(file = ?source, chunk = %chunk_name, size, "File not chunked (small)");

            return Ok(FileManifest {
                size,
                chunks: vec![chunk_name],
            });
        }

        // Split into chunks
        let mut chunks = Vec::new();
        let mut offset = 0;

        while offset < size {
            let end = (offset + self.config.chunk_size).min(size);
            let chunk_data = &data[offset..end];

            let chunk_name = format!("{}_{:04}.bin", self.config.chunk_prefix, chunk_counter);
            *chunk_counter += 1;

            let chunk_path = output_dir.join(&chunk_name);
            fs::write(&chunk_path, chunk_data).map_err(|e| SearchError::Io(e.to_string()))?;

            debug!(
                file = ?source,
                chunk = %chunk_name,
                offset,
                size = chunk_data.len(),
                "Wrote chunk"
            );

            chunks.push(chunk_name);
            offset = end;
        }

        info!(
            file = ?source,
            original_size = size,
            chunks = chunks.len(),
            "Chunked file"
        );

        Ok(FileManifest { size, chunks })
    }

    /// Write the manifest to a JSON file.
    pub fn write_manifest(manifest: &IndexManifest, output_path: &Path) -> Result<(), SearchError> {
        let json = manifest.to_json()?;
        fs::write(output_path, json).map_err(|e| SearchError::Io(e.to_string()))?;
        Ok(())
    }
}

/// Reassemble chunked files back into the original.
///
/// Useful for testing and verification.
pub fn reassemble_chunks(
    manifest: &IndexManifest,
    chunks_dir: &Path,
    file_name: &str,
) -> Result<Vec<u8>, SearchError> {
    let file_manifest = manifest
        .files
        .get(file_name)
        .ok_or_else(|| SearchError::Io(format!("File not found in manifest: {file_name}")))?;

    let mut data = Vec::with_capacity(file_manifest.size);

    for chunk_name in &file_manifest.chunks {
        let chunk_path = chunks_dir.join(chunk_name);
        let chunk_data = fs::read(&chunk_path).map_err(|e| SearchError::Io(e.to_string()))?;
        data.extend(chunk_data);
    }

    Ok(data)
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    #[test]
    fn test_chunk_small_file() {
        let temp_dir = TempDir::new().unwrap();
        let source_dir = temp_dir.path().join("source");
        let output_dir = temp_dir.path().join("output");
        fs::create_dir_all(&source_dir).unwrap();

        // Create a small file (smaller than chunk size)
        let small_data = b"Hello, world!";
        fs::write(source_dir.join("small.txt"), small_data).unwrap();

        let chunker = IndexChunker::with_defaults();
        let manifest = chunker.chunk_directory(&source_dir, &output_dir).unwrap();

        assert_eq!(manifest.files.len(), 1);
        let file_manifest = manifest.files.get("small.txt").unwrap();
        assert_eq!(file_manifest.size, small_data.len());
        assert_eq!(file_manifest.chunks.len(), 1);
    }

    #[test]
    fn test_chunk_large_file() {
        let temp_dir = TempDir::new().unwrap();
        let source_dir = temp_dir.path().join("source");
        let output_dir = temp_dir.path().join("output");
        fs::create_dir_all(&source_dir).unwrap();

        // Create a file larger than chunk size
        let chunk_size = 1024;
        let large_data: Vec<u8> = (0..3000).map(|i| (i % 256) as u8).collect();
        fs::write(source_dir.join("large.bin"), &large_data).unwrap();

        let chunker = IndexChunker::new(ChunkerConfig {
            chunk_size,
            chunk_prefix: "test".to_string(),
        });
        let manifest = chunker.chunk_directory(&source_dir, &output_dir).unwrap();

        assert_eq!(manifest.files.len(), 1);
        let file_manifest = manifest.files.get("large.bin").unwrap();
        assert_eq!(file_manifest.size, large_data.len());
        // 3000 bytes / 1024 chunk size = 3 chunks
        assert_eq!(file_manifest.chunks.len(), 3);
    }

    #[test]
    fn test_reassemble_chunks() {
        let temp_dir = TempDir::new().unwrap();
        let source_dir = temp_dir.path().join("source");
        let output_dir = temp_dir.path().join("output");
        fs::create_dir_all(&source_dir).unwrap();

        // Create test data
        let original_data: Vec<u8> = (0..5000).map(|i| (i % 256) as u8).collect();
        fs::write(source_dir.join("data.bin"), &original_data).unwrap();

        let chunker = IndexChunker::new(ChunkerConfig {
            chunk_size: 1024,
            chunk_prefix: "chunk".to_string(),
        });
        let manifest = chunker.chunk_directory(&source_dir, &output_dir).unwrap();

        // Reassemble and verify
        let reassembled = reassemble_chunks(&manifest, &output_dir, "data.bin").unwrap();
        assert_eq!(reassembled, original_data);
    }

    #[test]
    fn test_manifest_serialization() {
        let mut manifest = IndexManifest::new(64 * 1024);
        manifest.files.insert(
            "test.bin".to_string(),
            FileManifest {
                size: 1000,
                chunks: vec!["chunk_0000.bin".to_string()],
            },
        );
        manifest.total_size = 1000;

        let json = manifest.to_json().unwrap();
        let parsed = IndexManifest::from_json(&json).unwrap();

        assert_eq!(parsed.version, manifest.version);
        assert_eq!(parsed.chunk_size, manifest.chunk_size);
        assert_eq!(parsed.total_size, manifest.total_size);
        assert_eq!(parsed.files.len(), 1);
    }

    #[test]
    fn test_chunk_multiple_files() {
        let temp_dir = TempDir::new().unwrap();
        let source_dir = temp_dir.path().join("source");
        let output_dir = temp_dir.path().join("output");
        fs::create_dir_all(&source_dir).unwrap();

        // Create multiple files
        fs::write(source_dir.join("file1.txt"), b"Content 1").unwrap();
        fs::write(source_dir.join("file2.txt"), b"Content 2").unwrap();
        fs::write(source_dir.join("file3.txt"), b"Content 3").unwrap();

        let chunker = IndexChunker::with_defaults();
        let manifest = chunker.chunk_directory(&source_dir, &output_dir).unwrap();

        assert_eq!(manifest.files.len(), 3);
        assert!(manifest.files.contains_key("file1.txt"));
        assert!(manifest.files.contains_key("file2.txt"));
        assert!(manifest.files.contains_key("file3.txt"));
    }
}
