use axum::{
    Json, Router,
    extract::{Multipart, Path as QueryPath, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    serve,
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{net::SocketAddr, path::PathBuf, sync::Arc};
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;
use tower_http::cors::{Any, CorsLayer};
use uuid::Uuid;

use crate::db::{
    get_database_connection,
    models::{self, NewFile},
};

#[derive(Debug, Serialize)]
pub struct FileResponse {
    pub id: Uuid,
    pub file_name: String,
    pub file_type: Option<String>,
    pub file_size: Option<i64>,
}

struct AppState {
    upload_dir: PathBuf,
}

#[derive(Debug)]
enum AppError {
    FileUploadError(String),
    DatabaseError(String),
    NotFoundError(String),
    FileProcessingError(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::FileUploadError(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::DatabaseError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AppError::NotFoundError(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::FileProcessingError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}

pub async fn run() {
    let upload_dir =
        PathBuf::from(std::env::var("UPLOAD_DIR").unwrap_or_else(|_| "./uploads".to_string()));

    fs::create_dir_all(&upload_dir)
        .await
        .expect("Failed to create upload directory");

    // Initialize state
    let state = Arc::new(AppState { upload_dir });

    // Set up CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build our application router
    let app = Router::new()
        .route("/", get(|| async { "Document Processing API" }))
        .route("/upload", post(upload_file))
        .route("/files", get(list_files))
        .route("/process/:id", post(process_file))
        .with_state(state)
        .layer(cors);

    // Run the server
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    serve(listener, app).await.unwrap();
}

// Handle file upload
async fn upload_file(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    // Process the multipart form
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::FileUploadError(format!("Failed to process form: {}", e)))?
    {
        let file_name = field
            .file_name()
            .ok_or(AppError::FileUploadError(
                "File name not provided".to_string(),
            ))?
            .to_string();

        let content_type = field.content_type().map(|ct| ct.to_string());

        let data = field
            .bytes()
            .await
            .map_err(|e| AppError::FileUploadError(format!("Failed to read file data: {}", e)))?;

        let file_id = Uuid::new_v4();
        let file_path = state.upload_dir.join(file_id.to_string());

        // Save file to disk
        let mut file = File::create(&file_path)
            .await
            .map_err(|e| AppError::FileUploadError(format!("Failed to create file: {}", e)))?;

        file.write_all(&data)
            .await
            .map_err(|e| AppError::FileUploadError(format!("Failed to write file: {}", e)))?;

        // Calculate hash for the file
        let mut hasher = Sha256::new();
        hasher.update(&data);
        let file_hash = format!("{:x}", hasher.finalize());

        // Save file metadata to database
        let new_file = NewFile {
            file_path: file_path.to_string_lossy().to_string(),
            file_name,
            file_size: Some(data.len() as i64),
            file_type: content_type,
            file_hash: Some(file_hash),
            metadata: None,
        };

        let mut conn = get_database_connection()
            .map_err(|e| AppError::DatabaseError(format!("Could not connect to database: {}", e)))
            .unwrap();

        let result = models::File::create_file(&mut conn, new_file)
            .map_err(|e| {
                AppError::DatabaseError(format!("File could not be created details: {}", e))
            })
            .unwrap();

        return Ok((
            StatusCode::CREATED,
            Json(FileResponse {
                id: file_id,
                file_name: result.file_name,
                file_type: result.file_type,
                file_size: result.file_size,
            }),
        ));
    }

    Err(AppError::FileUploadError("No file provided".to_string()))
}

async fn list_files(
    State(state): State<Arc<AppState>>,
    QueryPath(skip): QueryPath<i64>,
    QueryPath(limit): QueryPath<i64>,
) -> Result<impl IntoResponse, AppError> {
    let mut conn = get_database_connection()
        .map_err(|e| AppError::DatabaseError(format!("Could not connect to database: {}", e)))
        .unwrap();

    let results = models::File::find_files(&mut conn, skip, limit)
        .map_err(|e| AppError::NotFoundError(format!("{}", e)))
        .unwrap();

    let response: Vec<FileResponse> = results
        .into_iter()
        .map(|file| FileResponse {
            id: file.id,
            file_name: file.file_name,
            file_type: file.file_type,
            file_size: file.file_size,
        })
        .collect();

    return Ok((StatusCode::OK, Json(response)));
}

// Process a previously uploaded file
async fn process_file(
    State(state): State<Arc<AppState>>,
    QueryPath(id): QueryPath<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let mut conn = get_database_connection()
        .map_err(|e| AppError::DatabaseError(format!("Could not connect to database: {}", e)))
        .unwrap();

    let file = models::File::find_file(&mut conn, id)
        .map_err(|e| AppError::DatabaseError(format!("Failed to find file: {}", e)))?;

    // Here you would call your existing logic to process the file
    // This is where you'd implement the PDF extraction, chunking, and DB storage
    // process_document(&file).await?;

    Ok(Json(serde_json::json!({
        "message": "File processing started",
        "id": file.id
    })))
}

// // Document processing function - replace with your actual processing logic
// async fn process_document(file: &File) -> Result<(), AppError> {
//     // This function would implement or call your existing document processing logic
//     // For example:
//     // 1. Read the file from disk
//     // 2. Determine file type and use appropriate extractor
//     // 3. Extract text from PDF or other documents
//     // 4. Chunk the text
//     // 5. Create vectors
//     // 6. Store in database

//     // For now, let's just log that we would process the file
//     println!("Processing file: {} ({})", file.file_name, file.id);

//     // Check if file exists
//     if !Path::new(&file.file_path).exists() {
//         return Err(AppError::FileProcessingError(format!(
//             "File not found: {}",
//             file.file_path
//         )));
//     }

//     // This is where you'd call your existing logic for:
//     // - PDF text extraction
//     // - Document chunking
//     // - Vector creation
//     // - Database storage

//     // For now we'll just simulate processing
//     tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

//     println!("Finished processing file: {}", file.id);

//     Ok(())
// }
