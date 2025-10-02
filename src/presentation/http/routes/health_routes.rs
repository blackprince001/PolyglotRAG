use axum::{routing::get, Router, Json, http::StatusCode, response::IntoResponse};

use crate::presentation::http::dto::{HealthResponseDto, ApiResponse};

pub fn health_routes() -> Router {
    Router::new()
        .route("/", get(root_handler))
        .route("/health", get(health_handler))
}

async fn root_handler() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(ApiResponse::success("RAG Engine API - Clean Architecture".to_string())),
    )
}

async fn health_handler() -> impl IntoResponse {
    let health_response = HealthResponseDto {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime: "N/A".to_string(), // Could be calculated from start time
    };

    (
        StatusCode::OK,
        Json(ApiResponse::success(health_response)),
    )
}
