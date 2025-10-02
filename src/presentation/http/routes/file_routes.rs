use axum::{
    Router,
    routing::{delete, get, post, put},
};
use std::sync::Arc;

use crate::presentation::http::handlers::FileHandler;

pub fn file_routes(file_handler: Arc<FileHandler>) -> Router {
    Router::new()
        .route("/upload", post(FileHandler::upload_file))
        .route(
            "/upload-and-process",
            post(FileHandler::upload_file_with_processing),
        )
        .route("/files", get(FileHandler::list_files))
        .route("/files/count", get(FileHandler::get_file_count))
        .route("/files/{file_id}/chunks", get(FileHandler::get_file_chunks))
        .route("/files/{file_id}", get(FileHandler::get_file))
        .route("/files/{file_id}", put(FileHandler::update_file))
        .route("/files/{file_id}", delete(FileHandler::delete_file))
        .route("/process/{id}", post(FileHandler::process_file))
        .with_state(file_handler)
}
