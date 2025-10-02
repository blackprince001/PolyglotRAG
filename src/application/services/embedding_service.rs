use std::sync::Arc;

use crate::application::ports::embedding_provider::{
    BatchEmbeddingRequest, EmbeddingProvider, EmbeddingRequest,
};
use crate::domain::entities::{ContentChunk, Embedding};

#[derive(Debug)]
pub enum EmbeddingServiceError {
    ProviderError(String),
    ValidationError(String),
}

impl std::fmt::Display for EmbeddingServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EmbeddingServiceError::ProviderError(msg) => write!(f, "Provider error: {}", msg),
            EmbeddingServiceError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl std::error::Error for EmbeddingServiceError {}

pub struct EmbeddingService {
    embedding_provider: Arc<dyn EmbeddingProvider>,
}

impl EmbeddingService {
    pub fn new(embedding_provider: Arc<dyn EmbeddingProvider>) -> Self {
        Self { embedding_provider }
    }

    pub async fn generate_embedding_for_chunk(
        &self,
        chunk: &ContentChunk,
    ) -> Result<Embedding, EmbeddingServiceError> {
        if chunk.is_empty() {
            return Err(EmbeddingServiceError::ValidationError(
                "Cannot generate embedding for empty chunk".to_string(),
            ));
        }

        let request = EmbeddingRequest {
            text: chunk.chunk_text().to_string(),
            model_name: None,
            model_version: None,
        };

        let response = self
            .embedding_provider
            .generate_embedding(request)
            .await
            .map_err(|e| EmbeddingServiceError::ProviderError(e.to_string()))?;

        Ok(Embedding::new(
            chunk.id(),
            response.model_name,
            response.model_version,
            None, // Generation parameters
            response.embedding,
        ))
    }

    pub async fn generate_embeddings_for_chunks(
        &self,
        chunks: &[ContentChunk],
    ) -> Result<Vec<Embedding>, EmbeddingServiceError> {
        if chunks.is_empty() {
            return Ok(Vec::new());
        }

        // Filter out empty chunks
        let valid_chunks: Vec<&ContentChunk> =
            chunks.iter().filter(|chunk| !chunk.is_empty()).collect();

        if valid_chunks.is_empty() {
            return Ok(Vec::new());
        }

        let texts: Vec<String> = valid_chunks
            .iter()
            .map(|chunk| chunk.chunk_text().to_string())
            .collect();

        let request = BatchEmbeddingRequest {
            texts,
            model_name: None,
            model_version: None,
        };

        let response = self
            .embedding_provider
            .generate_embeddings(request)
            .await
            .map_err(|e| EmbeddingServiceError::ProviderError(e.to_string()))?;

        let mut embeddings = Vec::new();
        for (chunk, embedding_vector) in valid_chunks.iter().zip(response.embeddings.iter()) {
            let embedding = Embedding::new(
                chunk.id(),
                response.model_name.clone(),
                response.model_version.clone(),
                None, // Generation parameters
                embedding_vector.clone(),
            );
            embeddings.push(embedding);
        }

        Ok(embeddings)
    }

    pub async fn health_check(&self) -> Result<bool, EmbeddingServiceError> {
        self.embedding_provider
            .health_check()
            .await
            .map_err(|e| EmbeddingServiceError::ProviderError(e.to_string()))
    }

    pub fn model_info(&self) -> (String, Option<String>) {
        self.embedding_provider.model_info()
    }

    pub fn embedding_dimension(&self) -> usize {
        self.embedding_provider.embedding_dimension()
    }
}
