use std::sync::Arc;
use uuid::Uuid;

use crate::application::ports::document_extractor::ExtractionOptions;
use crate::application::services::DocumentProcessorService;
use crate::domain::repositories::{FileRepository, file_repository::FileRepositoryError};

#[derive(Debug)]
pub enum ProcessDocumentError {
    FileNotFound(Uuid),
    RepositoryError(String),
    ProcessingError(String),
    FileNotProcessable(String),
}

impl std::fmt::Display for ProcessDocumentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProcessDocumentError::FileNotFound(id) => write!(f, "File not found: {}", id),
            ProcessDocumentError::RepositoryError(msg) => write!(f, "Repository error: {}", msg),
            ProcessDocumentError::ProcessingError(msg) => write!(f, "Processing error: {}", msg),
            ProcessDocumentError::FileNotProcessable(msg) => {
                write!(f, "File not processable: {}", msg)
            }
        }
    }
}

impl std::error::Error for ProcessDocumentError {}

impl From<FileRepositoryError> for ProcessDocumentError {
    fn from(error: FileRepositoryError) -> Self {
        match error {
            FileRepositoryError::NotFound(id) => ProcessDocumentError::FileNotFound(id),
            _ => ProcessDocumentError::RepositoryError(error.to_string()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProcessDocumentRequest {
    pub file_id: Uuid,
    pub extraction_options: Option<ExtractionOptions>,
}

#[derive(Debug, Clone)]
pub struct ProcessDocumentResponse {
    pub file_id: Uuid,
    pub chunks_created: i32,
    pub embeddings_created: i32,
    pub processing_time_ms: u64,
}

pub struct ProcessDocumentUseCase {
    file_repository: Arc<dyn FileRepository>,
    document_processor: Arc<DocumentProcessorService>,
}

impl ProcessDocumentUseCase {
    pub fn new(
        file_repository: Arc<dyn FileRepository>,
        document_processor: Arc<DocumentProcessorService>,
    ) -> Self {
        Self {
            file_repository,
            document_processor,
        }
    }

    pub async fn execute(
        &self,
        request: ProcessDocumentRequest,
    ) -> Result<ProcessDocumentResponse, ProcessDocumentError> {
        let start_time = std::time::Instant::now();

        // Find the file
        let mut file = self
            .file_repository
            .find_by_id(request.file_id)
            .await?
            .ok_or(ProcessDocumentError::FileNotFound(request.file_id))?;

        // Check if file is processable
        if !file.is_processable() {
            return Err(ProcessDocumentError::FileNotProcessable(format!(
                "File is in {:?} state",
                file.processing_status()
            )));
        }

        // Start processing
        file.start_processing()
            .map_err(|e| ProcessDocumentError::ProcessingError(e))?;

        self.file_repository.update(&file).await?;

        // Process the document
        let processing_result = self
            .document_processor
            .process_file(&file, request.extraction_options.unwrap_or_default())
            .await;

        match processing_result {
            Ok((chunks_created, embeddings_created)) => {
                // Mark as completed
                file.complete_processing()
                    .map_err(|e| ProcessDocumentError::ProcessingError(e))?;

                self.file_repository.update(&file).await?;

                let processing_time = start_time.elapsed().as_millis() as u64;

                Ok(ProcessDocumentResponse {
                    file_id: request.file_id,
                    chunks_created,
                    embeddings_created,
                    processing_time_ms: processing_time,
                })
            }
            Err(e) => {
                // Mark as failed
                file.fail_processing(e.to_string())
                    .map_err(|e| ProcessDocumentError::ProcessingError(e))?;

                self.file_repository.update(&file).await?;

                Err(ProcessDocumentError::ProcessingError(e.to_string()))
            }
        }
    }
}
