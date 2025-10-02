use std::sync::Arc;
use uuid::Uuid;

use crate::domain::entities::ContentChunk;
use crate::domain::repositories::{
    ChunkRepository, FileRepository, chunk_repository::ChunkRepositoryError,
    file_repository::FileRepositoryError,
};

#[derive(Debug)]
pub enum GetFileChunksError {
    FileNotFound(Uuid),
    RepositoryError(String),
}

impl std::fmt::Display for GetFileChunksError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GetFileChunksError::FileNotFound(id) => write!(f, "File not found: {}", id),
            GetFileChunksError::RepositoryError(msg) => write!(f, "Repository error: {}", msg),
        }
    }
}

impl std::error::Error for GetFileChunksError {}

impl From<FileRepositoryError> for GetFileChunksError {
    fn from(error: FileRepositoryError) -> Self {
        match error {
            FileRepositoryError::NotFound(id) => GetFileChunksError::FileNotFound(id),
            _ => GetFileChunksError::RepositoryError(error.to_string()),
        }
    }
}

impl From<ChunkRepositoryError> for GetFileChunksError {
    fn from(error: ChunkRepositoryError) -> Self {
        GetFileChunksError::RepositoryError(error.to_string())
    }
}

#[derive(Debug, Clone)]
pub struct GetFileChunksRequest {
    pub file_id: Uuid,
    pub skip: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct GetFileChunksResponse {
    pub file_id: Uuid,
    pub chunks: Vec<ContentChunk>,
    pub total_chunks: i64,
    pub skip: i64,
    pub limit: i64,
}

pub struct GetFileChunksUseCase {
    file_repository: Arc<dyn FileRepository>,
    chunk_repository: Arc<dyn ChunkRepository>,
}

impl GetFileChunksUseCase {
    pub fn new(
        file_repository: Arc<dyn FileRepository>,
        chunk_repository: Arc<dyn ChunkRepository>,
    ) -> Self {
        Self {
            file_repository,
            chunk_repository,
        }
    }

    pub async fn execute(
        &self,
        request: GetFileChunksRequest,
    ) -> Result<GetFileChunksResponse, GetFileChunksError> {
        // Verify file exists
        let _file = self
            .file_repository
            .find_by_id(request.file_id)
            .await?
            .ok_or(GetFileChunksError::FileNotFound(request.file_id))?;

        let skip = request.skip.unwrap_or(0);
        let limit = request.limit.unwrap_or(50).min(100); // Cap at 100 chunks per request

        // Get chunks for the file
        let chunks = self
            .chunk_repository
            .find_by_file_id_paginated(request.file_id, skip, limit)
            .await?;

        // Get total count of chunks for this file
        let total_chunks = self
            .chunk_repository
            .count_by_file_id(request.file_id)
            .await?;

        Ok(GetFileChunksResponse {
            file_id: request.file_id,
            chunks,
            total_chunks,
            skip,
            limit,
        })
    }
}
