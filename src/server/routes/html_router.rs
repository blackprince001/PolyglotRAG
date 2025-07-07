use crate::server::errors::AppError;
use crate::server::serializers::{AppState, FileResponse};

use crate::core::html_extractor::page_to_text;
use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use sha2::{Digest, Sha256};
use std::sync::Arc;

use url::Url;
use uuid::Uuid;

use crate::db::{
    get_database_connection,
    models::{self, NewFile},
};

pub async fn parse_html_content(
    State(state): State<Arc<AppState>>,
    Query(url_param): Query<std::collections::HashMap<String, String>>,
) -> Result<impl IntoResponse, AppError> {
    let url_str = url_param
        .get("url")
        .ok_or_else(|| AppError::BadRequest("URL parameter is required".to_string()))?;

    let url =
        Url::parse(url_str).map_err(|e| AppError::BadRequest(format!("Invalid URL: {}", e)))?;

    let file_id = Uuid::new_v4();

    let parsed = page_to_text(url.as_str(), 128_usize).await;

    let file_name = parsed
        .save_to_txt(file_id)
        .map_err(|e| AppError::FileUploadError(format!("Failed to save parsed HTML: {}", e)))?;

    let file_path = state.upload_dir.join(&file_name);

    let data = tokio::fs::read(&file_path)
        .await
        .map_err(|e| AppError::FileUploadError(format!("Failed to read saved file: {}", e)))?;

    let mut hasher = Sha256::new();
    hasher.update(&data);
    let file_hash = format!("{:x}", hasher.finalize());

    let new_file = NewFile {
        file_path: file_path.to_string_lossy().to_string(),
        file_name: file_name.clone(),
        file_size: Some(data.len() as i64),
        file_type: Some("text/plain".to_string()),
        file_hash: Some(file_hash),
        metadata: None,
    };

    let mut conn = get_database_connection()
        .map_err(|e| AppError::DatabaseError(format!("Could not connect to database: {}", e)))
        .unwrap();

    let result = models::File::create_file(&mut conn, new_file)
        .map_err(|e| AppError::DatabaseError(format!("File could not be created details: {}", e)))
        .unwrap();

    let file_type = result.file_type.expect("File does not have a type yet!");

    let _ = state
        .scheduler
        .schedule_file(file_id, result.file_path, file_type.clone())
        .await;

    return Ok((
        StatusCode::CREATED,
        Json(FileResponse {
            id: file_id,
            file_name: result.file_name,
            file_type: Some(file_type),
            file_size: result.file_size,
            file_hash: result.file_hash,
        }),
    ));
}
