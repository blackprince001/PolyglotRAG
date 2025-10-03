use axum::{
    Router,
    routing::{delete, get, post},
};
use std::sync::Arc;

use crate::presentation::http::handlers::EmbeddingHandler;

pub fn embedding_routes(embedding_handler: Arc<EmbeddingHandler>) -> Router {
    Router::new()
        .route(
            "/embeddings/{embedding_id}",
            get(EmbeddingHandler::get_embedding),
        )
        .route(
            "/chunk-embeddings/{chunk_id}",
            get(EmbeddingHandler::get_embedding_by_chunk),
        )
        .route(
            "/file-embeddings/{file_id}",
            get(EmbeddingHandler::get_embeddings_by_file),
        )
        .route(
            "/similarity-search",
            post(EmbeddingHandler::similarity_search),
        )
        .route(
            "/embeddings/{embedding_id}",
            delete(EmbeddingHandler::delete_embedding),
        )
        .route(
            "/chunk-embeddings/{chunk_id}",
            delete(EmbeddingHandler::delete_embeddings_by_chunk),
        )
        .route(
            "/file-embeddings/{file_id}",
            delete(EmbeddingHandler::delete_embeddings_by_file),
        )
        .route(
            "/embeddings-count",
            get(EmbeddingHandler::get_embedding_count),
        )
        // .route(
        //     "/embeddings/count/model/{model_name}",
        //     get(EmbeddingHandler::get_embedding_count_by_model),
        // )
        .with_state(embedding_handler)
}
