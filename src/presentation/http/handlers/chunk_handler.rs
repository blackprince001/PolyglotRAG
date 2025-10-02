use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::application::use_cases::get_file_chunks::GetFileChunksResponse;
use crate::domain::repositories::ChunkRepository;
use crate::presentation::http::dto::{ApiResponse, PaginationDto, file_dto::FileChunksResponseDto};

pub struct ChunkHandler {
    chunk_repository: Arc<dyn ChunkRepository>,
}

impl ChunkHandler {
    pub fn new(chunk_repository: Arc<dyn ChunkRepository>) -> Self {
        Self { chunk_repository }
    }

    pub async fn get_chunk(
        State(handler): State<Arc<ChunkHandler>>,
        Path(chunk_id): Path<Uuid>,
    ) -> Result<impl IntoResponse, StatusCode> {
        match handler.chunk_repository.find_by_id(chunk_id).await {
            Ok(Some(chunk)) => {
                let response = GetFileChunksResponse {
                    file_id: chunk.file_id(),
                    chunks: vec![chunk],
                    total_chunks: 1,
                    skip: 0,
                    limit: 1,
                };
                let dto = FileChunksResponseDto::from(response);
                Ok((StatusCode::OK, Json(ApiResponse::success(dto))))
            }
            Ok(None) => Ok((
                StatusCode::NOT_FOUND,
                Json(ApiResponse::error(
                    "CHUNK_NOT_FOUND".to_string(),
                    format!("Chunk with ID {} not found", chunk_id),
                    None,
                )),
            )),
            Err(e) => Ok((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(
                    "DATABASE_ERROR".to_string(),
                    e.to_string(),
                    None,
                )),
            )),
        }
    }

    pub async fn get_chunks_by_file(
        State(handler): State<Arc<ChunkHandler>>,
        Path(file_id): Path<Uuid>,
        Query(pagination): Query<PaginationDto>,
    ) -> Result<impl IntoResponse, StatusCode> {
        let skip = pagination.skip;
        let limit = pagination.limit;

        match handler
            .chunk_repository
            .find_by_file_id_paginated(file_id, skip, limit)
            .await
        {
            Ok(chunks) => {
                let total_chunks = chunks.len() as i64;
                let response = GetFileChunksResponse {
                    file_id,
                    chunks,
                    total_chunks,
                    skip,
                    limit,
                };
                let dto = FileChunksResponseDto::from(response);
                Ok((StatusCode::OK, Json(ApiResponse::success(dto))))
            }
            Err(e) => Ok((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(
                    "CHUNKS_NOT_FOUND".to_string(),
                    e.to_string(),
                    None,
                )),
            )),
        }
    }

    pub async fn get_chunk_count_by_file(
        State(handler): State<Arc<ChunkHandler>>,
        Path(file_id): Path<Uuid>,
    ) -> Result<impl IntoResponse, StatusCode> {
        match handler.chunk_repository.count_by_file_id(file_id).await {
            Ok(count) => Ok((
                StatusCode::OK,
                Json(ApiResponse::success(serde_json::json!({
                    "file_id": file_id,
                    "chunk_count": count
                }))),
            )),
            Err(e) => Ok((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(
                    "COUNT_FAILED".to_string(),
                    e.to_string(),
                    None,
                )),
            )),
        }
    }

    pub async fn delete_chunk(
        State(handler): State<Arc<ChunkHandler>>,
        Path(chunk_id): Path<Uuid>,
    ) -> Result<impl IntoResponse, StatusCode> {
        match handler.chunk_repository.delete(chunk_id).await {
            Ok(true) => Ok((
                StatusCode::OK,
                Json(ApiResponse::success(
                    "Chunk deleted successfully".to_string(),
                )),
            )),
            Ok(false) => Ok((
                StatusCode::NOT_FOUND,
                Json(ApiResponse::error(
                    "CHUNK_NOT_FOUND".to_string(),
                    format!("Chunk with ID {} not found", chunk_id),
                    None,
                )),
            )),
            Err(e) => Ok((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(
                    "DELETE_FAILED".to_string(),
                    e.to_string(),
                    None,
                )),
            )),
        }
    }

    pub async fn delete_chunks_by_file(
        State(handler): State<Arc<ChunkHandler>>,
        Path(file_id): Path<Uuid>,
    ) -> Result<impl IntoResponse, StatusCode> {
        match handler.chunk_repository.delete_by_file_id(file_id).await {
            Ok(count) => Ok((
                StatusCode::OK,
                Json(ApiResponse::success(serde_json::json!({
                    "file_id": file_id,
                    "deleted_chunks": count
                }))),
            )),
            Err(e) => Ok((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(
                    "DELETE_FAILED".to_string(),
                    e.to_string(),
                    None,
                )),
            )),
        }
    }
}
