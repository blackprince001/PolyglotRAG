use std::sync::Arc;
use uuid::Uuid;

use crate::application::ports::{
    DocumentExtractor, EmbeddingProvider,
    document_extractor::{ExtractedContent, ExtractionOptions},
    embedding_provider::BatchEmbeddingRequest,
};
use crate::domain::entities::{ContentChunk, Embedding, File};
use crate::domain::repositories::{ChunkRepository, EmbeddingRepository, FileRepository};

#[derive(Debug)]
pub enum DocumentProcessingError {
    ExtractionError(String),
    EmbeddingError(String),
    RepositoryError(String),
}

impl std::fmt::Display for DocumentProcessingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DocumentProcessingError::ExtractionError(msg) => write!(f, "Extraction error: {}", msg),
            DocumentProcessingError::EmbeddingError(msg) => write!(f, "Embedding error: {}", msg),
            DocumentProcessingError::RepositoryError(msg) => write!(f, "Repository error: {}", msg),
        }
    }
}

impl std::error::Error for DocumentProcessingError {}

pub struct DocumentProcessorService {
    document_extractor: Arc<dyn DocumentExtractor>,
    embedding_provider: Arc<dyn EmbeddingProvider>,
    chunk_repository: Arc<dyn ChunkRepository>,
    embedding_repository: Arc<dyn EmbeddingRepository>,
    file_repository: Arc<dyn FileRepository>,
    chunk_size: usize,
    chunk_overlap: usize,
}

impl DocumentProcessorService {
    pub fn new(
        document_extractor: Arc<dyn DocumentExtractor>,
        embedding_provider: Arc<dyn EmbeddingProvider>,
        chunk_repository: Arc<dyn ChunkRepository>,
        embedding_repository: Arc<dyn EmbeddingRepository>,
        file_repository: Arc<dyn FileRepository>,
    ) -> Self {
        Self {
            document_extractor,
            embedding_provider,
            chunk_repository,
            embedding_repository,
            file_repository,
            chunk_size: 578,
            chunk_overlap: 102,
        }
    }

    pub async fn process_file(
        &self,
        file: &File,
        extraction_options: ExtractionOptions,
    ) -> Result<(i32, i32), DocumentProcessingError> {
        let extracted_content = self
            .extract_text_from_file(file, extraction_options)
            .await?;

        let chunks = self.create_chunks(file.id(), &extracted_content.text)?;

        match self.file_repository.find_by_id(file.id()).await {
            Ok(Some(_verified_file)) => {}
            Ok(None) => {
                return Err(DocumentProcessingError::RepositoryError(format!(
                    "File {} disappeared from database before saving chunks",
                    file.id()
                )));
            }
            Err(e) => {
                return Err(DocumentProcessingError::RepositoryError(format!(
                    "Failed to verify file exists before saving chunks: {}",
                    e
                )));
            }
        }

        self.chunk_repository
            .save_batch(&chunks)
            .await
            .map_err(|e| DocumentProcessingError::RepositoryError(e.to_string()))?;

        let embeddings = self.generate_embeddings_for_chunks(&chunks).await?;

        self.embedding_repository
            .save_batch(&embeddings)
            .await
            .map_err(|e| DocumentProcessingError::RepositoryError(e.to_string()))?;

        Ok((chunks.len() as i32, embeddings.len() as i32))
    }

    async fn extract_text_from_file(
        &self,
        file: &File,
        extraction_options: ExtractionOptions,
    ) -> Result<ExtractedContent, DocumentProcessingError> {
        self.document_extractor
            .extract_text(file, extraction_options)
            .await
            .map_err(|e| DocumentProcessingError::ExtractionError(e.to_string()))
    }

    fn create_chunks(
        &self,
        file_id: Uuid,
        text: &str,
    ) -> Result<Vec<ContentChunk>, DocumentProcessingError> {
        let mut chunks = Vec::new();
        let words: Vec<&str> = text.split_whitespace().collect();

        if words.is_empty() {
            return Ok(chunks);
        }

        let mut start = 0;
        let mut chunk_index = 0;

        while start < words.len() {
            // Calculate end position for this chunk
            let end = std::cmp::min(start + self.chunk_size, words.len());

            // Create chunk text
            let chunk_text = words[start..end].join(" ");

            // Skip empty or very small chunks
            if chunk_text.trim().len() < 10 {
                break;
            }

            // Create chunk entity
            let chunk = ContentChunk::new(
                file_id,
                chunk_text,
                chunk_index,
                Some(end as i32 - start as i32), // Approximate token count
                None,                            // Page number - could be extracted from metadata
                None, // Section path - could be extracted from document structure
            );

            chunks.push(chunk);
            chunk_index += 1;

            // Move start position with overlap
            start = if end >= words.len() {
                break;
            } else {
                std::cmp::max(start + self.chunk_size - self.chunk_overlap, start + 1)
            };
        }

        Ok(chunks)
    }

    async fn generate_embeddings_for_chunks(
        &self,
        chunks: &[ContentChunk],
    ) -> Result<Vec<Embedding>, DocumentProcessingError> {
        let mut embeddings = Vec::new();
        let (model_name, model_version) = self.embedding_provider.model_info();

        const BATCH_SIZE: usize = 10;

        for chunk_batch in chunks.chunks(BATCH_SIZE) {
            let texts: Vec<String> = chunk_batch
                .iter()
                .map(|chunk| chunk.chunk_text().to_string())
                .collect();

            let batch_request = BatchEmbeddingRequest {
                texts,
                model_name: Some(model_name.clone()),
                model_version: model_version.clone(),
            };

            let batch_response = self
                .embedding_provider
                .generate_embeddings(batch_request)
                .await
                .map_err(|e| DocumentProcessingError::EmbeddingError(e.to_string()))?;

            for (chunk, embedding_vector) in
                chunk_batch.iter().zip(batch_response.embeddings.iter())
            {
                let embedding = Embedding::new(
                    chunk.id(),
                    batch_response.model_name.clone(),
                    batch_response.model_version.clone(),
                    None,
                    embedding_vector.clone(),
                );

                embeddings.push(embedding);
            }
        }

        Ok(embeddings)
    }
}
