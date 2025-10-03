use axum::{Json, Router, http::StatusCode, response::IntoResponse, routing::get};

use crate::presentation::http::dto::{ApiResponse, HealthResponseDto};

pub fn health_routes() -> Router {
    Router::new()
        .route("/", get(root_handler))
        .route("/health", get(health_handler))
}

async fn root_handler() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(ApiResponse::success("PolyglotRAG".to_string())),
    )
}

async fn health_handler() -> impl IntoResponse {
    let health_response = HealthResponseDto {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    };

    (StatusCode::OK, Json(ApiResponse::success(health_response)))
}
