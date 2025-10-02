use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use pgvector::Vector;
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::repositories::EmbeddingRepository;
use crate::presentation::http::dto::ApiResponse;

#[derive(serde::Deserialize)]
pub struct SimilaritySearchRequest {
    pub query_vector: Vec<f32>,
    pub limit: Option<i32>,
    pub similarity_threshold: Option<f32>,
    pub file_id: Option<Uuid>,
}

#[derive(serde::Serialize)]
pub struct SimilaritySearchResponse {
    pub results: Vec<SimilaritySearchResultDto>,
    pub total_results: usize,
}

#[derive(serde::Serialize)]
pub struct SimilaritySearchResultDto {
    pub similarity_score: f32,
    pub chunk_id: Uuid,
    pub file_id: Uuid,
}

pub struct EmbeddingHandler {
    embedding_repository: Arc<dyn EmbeddingRepository>,
}

impl EmbeddingHandler {
    pub fn new(embedding_repository: Arc<dyn EmbeddingRepository>) -> Self {
        Self {
            embedding_repository,
        }
    }

    pub async fn get_embedding(
        State(handler): State<Arc<EmbeddingHandler>>,
        Path(embedding_id): Path<Uuid>,
    ) -> Result<impl IntoResponse, StatusCode> {
        match handler.embedding_repository.find_by_id(embedding_id).await {
            Ok(Some(embedding)) => Ok((
                StatusCode::OK,
                Json(ApiResponse::success(serde_json::json!({
                    "id": embedding.id(),
                    "chunk_id": embedding.content_chunk_id(),
                    "model_name": embedding.model_name(),
                    "model_version": embedding.model_version(),
                    "vector_dimension": embedding.embedding().as_slice().len(),
                    "created_at": embedding.generated_at().to_rfc3339()
                }))),
            )),
            Ok(None) => Ok((
                StatusCode::NOT_FOUND,
                Json(ApiResponse::error(
                    "EMBEDDING_NOT_FOUND".to_string(),
                    format!("Embedding with ID {} not found", embedding_id),
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

    pub async fn get_embedding_by_chunk(
        State(handler): State<Arc<EmbeddingHandler>>,
        Path(chunk_id): Path<Uuid>,
    ) -> Result<impl IntoResponse, StatusCode> {
        match handler
            .embedding_repository
            .find_by_chunk_id(chunk_id)
            .await
        {
            Ok(Some(embedding)) => Ok((
                StatusCode::OK,
                Json(ApiResponse::success(serde_json::json!({
                    "id": embedding.id(),
                    "chunk_id": embedding.content_chunk_id(),
                    "model_name": embedding.model_name(),
                    "model_version": embedding.model_version(),
                    "vector_dimension": embedding.embedding().as_slice().len(),
                    "created_at": embedding.generated_at().to_rfc3339()
                }))),
            )),
            Ok(None) => Ok((
                StatusCode::NOT_FOUND,
                Json(ApiResponse::error(
                    "EMBEDDING_NOT_FOUND".to_string(),
                    format!("No embedding found for chunk ID {}", chunk_id),
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

    pub async fn get_embeddings_by_file(
        State(handler): State<Arc<EmbeddingHandler>>,
        Path(file_id): Path<Uuid>,
    ) -> Result<impl IntoResponse, StatusCode> {
        match handler.embedding_repository.find_by_file_id(file_id).await {
            Ok(embeddings) => {
                let embeddings_dto: Vec<serde_json::Value> = embeddings
                    .into_iter()
                    .map(|e| {
                        serde_json::json!({
                            "id": e.id(),
                            "chunk_id": e.content_chunk_id(),
                            "model_name": e.model_name(),
                            "model_version": e.model_version(),
                            "vector_dimension": e.embedding().as_slice().len(),
                            "created_at": e.generated_at().to_rfc3339()
                        })
                    })
                    .collect();

                Ok((
                    StatusCode::OK,
                    Json(ApiResponse::success(serde_json::json!({
                        "file_id": file_id,
                        "embeddings": embeddings_dto,
                        "count": embeddings_dto.len()
                    }))),
                ))
            }
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

    pub async fn similarity_search(
        State(handler): State<Arc<EmbeddingHandler>>,
        Json(request): Json<SimilaritySearchRequest>,
    ) -> Result<impl IntoResponse, StatusCode> {
        let limit = request.limit.unwrap_or(10);
        let query_vector = Vector::from(request.query_vector);

        let results = if let Some(file_id) = request.file_id {
            // Search within specific file
            match handler
                .embedding_repository
                .similarity_search_by_file(
                    &query_vector,
                    file_id,
                    limit,
                    request.similarity_threshold,
                )
                .await
            {
                Ok(results) => results,
                Err(e) => {
                    return Ok((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ApiResponse::error(
                            "SEARCH_FAILED".to_string(),
                            e.to_string(),
                            None,
                        )),
                    ));
                }
            }
        } else {
            // Global search
            match handler
                .embedding_repository
                .similarity_search(&query_vector, limit, request.similarity_threshold)
                .await
            {
                Ok(results) => results,
                Err(e) => {
                    return Ok((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ApiResponse::error(
                            "SEARCH_FAILED".to_string(),
                            e.to_string(),
                            None,
                        )),
                    ));
                }
            }
        };

        let results_dto: Vec<SimilaritySearchResultDto> = results
            .into_iter()
            .map(|r| SimilaritySearchResultDto {
                similarity_score: r.similarity_score,
                chunk_id: r.chunk_id,
                file_id: Uuid::new_v4(), // TODO: Get file_id from chunk_id
            })
            .collect();

        let response = SimilaritySearchResponse {
            total_results: results_dto.len(),
            results: results_dto,
        };

        Ok((StatusCode::OK, Json(ApiResponse::success(response))))
    }

    pub async fn delete_embedding(
        State(handler): State<Arc<EmbeddingHandler>>,
        Path(embedding_id): Path<Uuid>,
    ) -> Result<impl IntoResponse, StatusCode> {
        match handler.embedding_repository.delete(embedding_id).await {
            Ok(true) => Ok((
                StatusCode::OK,
                Json(ApiResponse::success(
                    "Embedding deleted successfully".to_string(),
                )),
            )),
            Ok(false) => Ok((
                StatusCode::NOT_FOUND,
                Json(ApiResponse::error(
                    "EMBEDDING_NOT_FOUND".to_string(),
                    format!("Embedding with ID {} not found", embedding_id),
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

    pub async fn delete_embeddings_by_chunk(
        State(handler): State<Arc<EmbeddingHandler>>,
        Path(chunk_id): Path<Uuid>,
    ) -> Result<impl IntoResponse, StatusCode> {
        match handler
            .embedding_repository
            .delete_by_chunk_id(chunk_id)
            .await
        {
            Ok(true) => Ok((
                StatusCode::OK,
                Json(ApiResponse::success(
                    "Embeddings deleted successfully".to_string(),
                )),
            )),
            Ok(false) => Ok((
                StatusCode::NOT_FOUND,
                Json(ApiResponse::error(
                    "EMBEDDING_NOT_FOUND".to_string(),
                    format!("No embeddings found for chunk ID {}", chunk_id),
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

    pub async fn delete_embeddings_by_file(
        State(handler): State<Arc<EmbeddingHandler>>,
        Path(file_id): Path<Uuid>,
    ) -> Result<impl IntoResponse, StatusCode> {
        match handler
            .embedding_repository
            .delete_by_file_id(file_id)
            .await
        {
            Ok(count) => Ok((
                StatusCode::OK,
                Json(ApiResponse::success(serde_json::json!({
                    "file_id": file_id,
                    "deleted_embeddings": count
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

    pub async fn get_embedding_count(
        State(handler): State<Arc<EmbeddingHandler>>,
    ) -> Result<impl IntoResponse, StatusCode> {
        match handler.embedding_repository.count().await {
            Ok(count) => Ok((
                StatusCode::OK,
                Json(ApiResponse::success(serde_json::json!({
                    "count": count
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

    pub async fn get_embedding_count_by_model(
        State(handler): State<Arc<EmbeddingHandler>>,
        Path(model_name): Path<String>,
    ) -> Result<impl IntoResponse, StatusCode> {
        match handler
            .embedding_repository
            .count_by_model(&model_name)
            .await
        {
            Ok(count) => Ok((
                StatusCode::OK,
                Json(ApiResponse::success(serde_json::json!({
                    "model_name": model_name,
                    "count": count
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
}
