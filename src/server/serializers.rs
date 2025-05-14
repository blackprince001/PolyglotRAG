use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct FileResponse {
    pub id: Uuid,
    pub file_name: String,
    pub file_type: Option<String>,
    pub file_size: Option<i64>,
    pub file_hash: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Pagination {
    pub skip: i64,
    pub limit: i64,
}

pub struct AppState {
    pub upload_dir: PathBuf,
}
