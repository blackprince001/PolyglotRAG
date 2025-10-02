use axum::{
    Router,
    routing::{delete, get},
};
use std::sync::Arc;

use crate::presentation::http::handlers::ChunkHandler;

pub fn chunk_routes(chunk_handler: Arc<ChunkHandler>) -> Router {
    Router::new()
        .route("/chunks/{chunk_id}", get(ChunkHandler::get_chunk))
        .route(
            "/chunks/file/{file_id}",
            get(ChunkHandler::get_chunks_by_file),
        )
        .route(
            "/chunks/file/{file_id}/count",
            get(ChunkHandler::get_chunk_count_by_file),
        )
        .route("/chunks/{chunk_id}", delete(ChunkHandler::delete_chunk))
        .route(
            "/chunks/file/{file_id}",
            delete(ChunkHandler::delete_chunks_by_file),
        )
        .with_state(chunk_handler)
}
