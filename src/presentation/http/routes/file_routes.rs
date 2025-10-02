use axum::{Router, routing::get, routing::post};
use std::sync::Arc;

use crate::presentation::http::handlers::FileHandler;

pub fn file_routes(file_handler: Arc<FileHandler>) -> Router {
    Router::new()
        .route("/upload", post(FileHandler::upload_file))
        .route("/files", get(FileHandler::list_files))
        .route("/files/{file_id}", get(FileHandler::get_file))
        .route("/files/{file_id}/chunks", get(FileHandler::get_file_chunks))
        .route("/process/{id}", post(FileHandler::process_file))
        .with_state(file_handler)
}
