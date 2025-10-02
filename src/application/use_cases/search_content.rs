use std::sync::Arc;
use pgvector::Vector;

use crate::application::ports::EmbeddingProvider;
use crate::application::services::SearchService;
use crate::domain::entities::{SearchQuery, ContentChunk};
use crate::domain::repositories::{EmbeddingRepository, ChunkRepository};

#[derive(Debug)]
pub enum SearchContentError {
    EmbeddingError(String),
    RepositoryError(String),
    ValidationError(String),
}

impl std::fmt::Display for SearchContentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SearchContentError::EmbeddingError(msg) => write!(f, "Embedding error: {}", msg),
            SearchContentError::RepositoryError(msg) => write!(f, "Repository error: {}", msg),
            SearchContentError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl std::error::Error for SearchContentError {}

#[derive(Debug, Clone)]
pub struct SearchContentRequest {
    pub query: String,
    pub limit: Option<i32>,
    pub similarity_threshold: Option<f32>,
    pub file_id_filter: Option<uuid::Uuid>,
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub chunk: ContentChunk,
    pub similarity_score: f32,
    pub file_id: uuid::Uuid,
}

#[derive(Debug, Clone)]
pub struct SearchContentResponse {
    pub query: String,
    pub results: Vec<SearchResult>,
    pub total_results: i32,
    pub search_time_ms: u64,
}

pub struct SearchContentUseCase {
    search_service: Arc<SearchService>,
}

impl SearchContentUseCase {
    pub fn new(search_service: Arc<SearchService>) -> Self {
        Self { search_service }
    }

    pub async fn execute(&self, request: SearchContentRequest) -> Result<SearchContentResponse, SearchContentError> {
        let start_time = std::time::Instant::now();

        // Validate input
        if request.query.trim().is_empty() {
            return Err(SearchContentError::ValidationError("Query cannot be empty".to_string()));
        }

        let limit = request.limit.unwrap_or(10);
        if limit <= 0 || limit > 100 {
            return Err(SearchContentError::ValidationError("Limit must be between 1 and 100".to_string()));
        }

        // Perform search
        let results = self.search_service
            .search_content(
                &request.query,
                limit,
                request.similarity_threshold,
                request.file_id_filter,
            )
            .await
            .map_err(|e| SearchContentError::RepositoryError(e.to_string()))?;

        let search_time = start_time.elapsed().as_millis() as u64;

        Ok(SearchContentResponse {
            query: request.query,
            total_results: results.len() as i32,
            results,
            search_time_ms: search_time,
        })
    }
}
