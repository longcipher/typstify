//! Content handling for Markdown and Typst files

use crate::content_id::ContentId;
use crate::metadata::ContentMetadata;
use crate::renderers::{MarkdownRenderer, Renderer, RendererError, TypstRenderer};
use eyre::Result;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq)]
pub enum ContentType {
    Markdown,
    Typst,
}

impl ContentType {
    pub fn from_extension(extension: &str) -> Option<Self> {
        match extension {
            "md" | "markdown" => Some(Self::Markdown),
            "typ" | "typst" => Some(Self::Typst),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Content {
    pub id: ContentId,
    pub content_type: ContentType,
    pub metadata: ContentMetadata,
    pub raw_content: String,
    pub file_path: PathBuf,
}

impl Content {
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let raw_content = std::fs::read_to_string(path)?;

        let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");

        let content_type = ContentType::from_extension(extension)
            .ok_or_else(|| eyre::eyre!("Unsupported file extension: {}", extension))?;

        let (metadata, content_body) = match content_type {
            ContentType::Markdown => ContentMetadata::extract_from_markdown(&raw_content)?,
            ContentType::Typst => ContentMetadata::extract_from_typst(&raw_content)?,
        };

        let id = ContentId::from_path(path);

        Ok(Content {
            id,
            content_type,
            metadata,
            raw_content: content_body,
            file_path: path.to_path_buf(),
        })
    }

    pub fn scan_directory(dir: impl AsRef<Path>) -> Result<Vec<Self>> {
        let mut content = Vec::new();

        for entry in walkdir::WalkDir::new(dir) {
            let entry = entry?;
            if entry.file_type().is_file() {
                let path = entry.path();

                if let Some(extension) = path.extension().and_then(|e| e.to_str())
                    && ContentType::from_extension(extension).is_some()
                {
                    match Self::from_file(path) {
                        Ok(content_item) => {
                            println!("Loaded: {}", path.display());
                            content.push(content_item);
                        }
                        Err(e) => {
                            eprintln!("Error loading {}: {}", path.display(), e);
                        }
                    }
                }
            }
        }

        // Sort by date (newest first)
        content.sort_by(|a, b| {
            let a_date = a.metadata.get_date().unwrap_or("1970-01-01");
            let b_date = b.metadata.get_date().unwrap_or("1970-01-01");
            b_date.cmp(a_date)
        });

        Ok(content)
    }

    pub fn render(&self) -> Result<String, RendererError> {
        match self.content_type {
            ContentType::Markdown => {
                let renderer = MarkdownRenderer::new();
                renderer.render(&self.raw_content)
            }
            ContentType::Typst => {
                let renderer = TypstRenderer::new();
                renderer.render(&self.raw_content)
            }
        }
    }

    pub fn slug(&self) -> String {
        self.id.as_str().to_string()
    }

    pub fn title(&self) -> String {
        self.metadata.get_title()
    }

    pub fn meta(&self) -> &ContentMetadata {
        &self.metadata
    }
}
