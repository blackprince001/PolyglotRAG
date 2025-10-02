use async_trait::async_trait;
use std::path::Path;

use crate::domain::value_objects::FileMetadata;

#[derive(Debug)]
pub enum DocumentExtractionError {
    UnsupportedFormat(String),
    CorruptedFile(String),
    ExtractionFailed(String),
    IoError(String),
}

impl std::fmt::Display for DocumentExtractionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DocumentExtractionError::UnsupportedFormat(format) => write!(f, "Unsupported format: {}", format),
            DocumentExtractionError::CorruptedFile(msg) => write!(f, "Corrupted file: {}", msg),
            DocumentExtractionError::ExtractionFailed(msg) => write!(f, "Extraction failed: {}", msg),
            DocumentExtractionError::IoError(msg) => write!(f, "IO error: {}", msg),
        }
    }
}

impl std::error::Error for DocumentExtractionError {}

#[derive(Debug, Clone)]
pub struct ExtractedContent {
    pub text: String,
    pub metadata: FileMetadata,
    pub page_count: Option<i32>,
    pub language: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ExtractionOptions {
    pub extract_metadata: bool,
    pub preserve_formatting: bool,
    pub include_images: bool,
    pub max_pages: Option<i32>,
}

impl Default for ExtractionOptions {
    fn default() -> Self {
        Self {
            extract_metadata: true,
            preserve_formatting: false,
            include_images: false,
            max_pages: None,
        }
    }
}

#[async_trait]
pub trait DocumentExtractor: Send + Sync {
    async fn extract_text(
        &self,
        file_path: &Path,
        options: ExtractionOptions,
    ) -> Result<ExtractedContent, DocumentExtractionError>;
    
    async fn extract_text_from_bytes(
        &self,
        data: &[u8],
        file_type: &str,
        options: ExtractionOptions,
    ) -> Result<ExtractedContent, DocumentExtractionError>;
    
    fn supported_formats(&self) -> Vec<String>;
    
    fn can_extract(&self, file_type: &str) -> bool;
    
    fn max_file_size(&self) -> Option<usize>;
}
