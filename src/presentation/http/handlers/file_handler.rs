use axum::{
    Json,
    extract::{Multipart, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::application::use_cases::{
    GetFileChunksUseCase, GetFileUseCase, ListFilesUseCase, ProcessDocumentUseCase,
    UploadFileUseCase, UploadWithProcessingUseCase, get_file::GetFileRequest,
    get_file_chunks::GetFileChunksRequest, list_files::ListFilesRequest,
    process_document::ProcessDocumentRequest, upload_file::UploadFileRequest,
    upload_with_processing::UploadWithProcessingRequest,
};
use crate::domain::repositories::FileRepository;
use crate::presentation::http::dto::content_dto::UploadWithProcessingResponse;
use crate::presentation::http::dto::{
    ApiResponse, PaginationDto, PaginationMetaDto, file_dto::FileChunksResponseDto,
    file_dto::FileDetailResponseDto, file_dto::FileListResponseDto, file_dto::FileResponseDto,
    file_dto::ProcessFileResponseDto, file_dto::UploadResponseDto,
};

pub struct FileHandler {
    upload_use_case: Arc<UploadFileUseCase>,
    upload_with_processing_use_case: Arc<UploadWithProcessingUseCase>,
    list_files_use_case: Arc<ListFilesUseCase>,
    process_document_use_case: Arc<ProcessDocumentUseCase>,
    get_file_use_case: Arc<GetFileUseCase>,
    get_file_chunks_use_case: Arc<GetFileChunksUseCase>,
    file_repository: Arc<dyn FileRepository>,
}

impl FileHandler {
    pub fn new(
        upload_use_case: Arc<UploadFileUseCase>,
        upload_with_processing_use_case: Arc<UploadWithProcessingUseCase>,
        list_files_use_case: Arc<ListFilesUseCase>,
        process_document_use_case: Arc<ProcessDocumentUseCase>,
        get_file_use_case: Arc<GetFileUseCase>,
        get_file_chunks_use_case: Arc<GetFileChunksUseCase>,
        file_repository: Arc<dyn FileRepository>,
    ) -> Self {
        Self {
            upload_use_case,
            upload_with_processing_use_case,
            list_files_use_case,
            process_document_use_case,
            get_file_use_case,
            get_file_chunks_use_case,
            file_repository,
        }
    }

    pub async fn upload_file(
        State(handler): State<Arc<FileHandler>>,
        mut multipart: Multipart,
    ) -> Result<impl IntoResponse, StatusCode> {
        while let Some(field) = multipart
            .next_field()
            .await
            .map_err(|_| StatusCode::BAD_REQUEST)?
        {
            let file_name = field
                .file_name()
                .ok_or(StatusCode::BAD_REQUEST)?
                .to_string();

            let content_type = field.content_type().map(|ct| ct.to_string());

            let data = field
                .bytes()
                .await
                .map_err(|_| StatusCode::BAD_REQUEST)?
                .to_vec();

            let request = UploadFileRequest {
                file_name,
                file_data: data,
                content_type,
                metadata: None,
            };

            match handler.upload_use_case.execute(request).await {
                Ok(response) => {
                    let dto = UploadResponseDto::from(response);
                    return Ok((StatusCode::CREATED, Json(ApiResponse::success(dto))));
                }
                Err(e) => {
                    return Ok((
                        StatusCode::BAD_REQUEST,
                        Json(ApiResponse::error(
                            "UPLOAD_FAILED".to_string(),
                            e.to_string(),
                            None,
                        )),
                    ));
                }
            }
        }

        Ok((
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error(
                "NO_FILE_PROVIDED".to_string(),
                "No file provided in the request".to_string(),
                None,
            )),
        ))
    }

    pub async fn list_files(
        State(handler): State<Arc<FileHandler>>,
        Query(pagination): Query<PaginationDto>,
    ) -> Result<impl IntoResponse, StatusCode> {
        let request = ListFilesRequest {
            skip: pagination.skip,
            limit: pagination.limit,
        };

        match handler.list_files_use_case.execute(request).await {
            Ok(response) => {
                let files: Vec<FileResponseDto> = response
                    .files
                    .into_iter()
                    .map(FileResponseDto::from)
                    .collect();

                let dto = FileListResponseDto {
                    files,
                    meta: PaginationMetaDto {
                        offset: response.skip,
                        limit: response.limit,
                        total: response.total_count,
                    },
                };

                Ok((StatusCode::OK, Json(ApiResponse::success(dto))))
            }
            Err(e) => Ok((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<FileListResponseDto>::error(
                    "LIST_FAILED".to_string(),
                    e.to_string(),
                    None,
                )),
            )),
        }
    }

    pub async fn process_file(
        State(handler): State<Arc<FileHandler>>,
        Path(file_id): Path<Uuid>,
    ) -> Result<impl IntoResponse, StatusCode> {
        let request = ProcessDocumentRequest {
            file_id,
            extraction_options: None,
        };

        match handler.process_document_use_case.execute(request).await {
            Ok(response) => {
                let dto = ProcessFileResponseDto::from(response);
                Ok((StatusCode::OK, Json(ApiResponse::success(dto))))
            }
            Err(e) => Ok((
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<ProcessFileResponseDto>::error(
                    "PROCESSING_FAILED".to_string(),
                    e.to_string(),
                    None,
                )),
            )),
        }
    }

    pub async fn get_file(
        State(handler): State<Arc<FileHandler>>,
        Path(file_id): Path<Uuid>,
    ) -> Result<impl IntoResponse, StatusCode> {
        let request = GetFileRequest { file_id };

        match handler.get_file_use_case.execute(request).await {
            Ok(response) => {
                let dto = FileDetailResponseDto::from(response);
                Ok((StatusCode::OK, Json(ApiResponse::success(dto))))
            }
            Err(e) => Ok((
                StatusCode::NOT_FOUND,
                Json(ApiResponse::error(
                    "FILE_NOT_FOUND".to_string(),
                    e.to_string(),
                    None,
                )),
            )),
        }
    }

    pub async fn get_file_chunks(
        State(handler): State<Arc<FileHandler>>,
        Path(file_id): Path<Uuid>,
        Query(pagination): Query<PaginationDto>,
    ) -> Result<impl IntoResponse, StatusCode> {
        let request = GetFileChunksRequest {
            file_id,
            skip: Some(pagination.skip),
            limit: Some(pagination.limit),
        };

        match handler.get_file_chunks_use_case.execute(request).await {
            Ok(response) => {
                let dto = FileChunksResponseDto::from(response);
                Ok((StatusCode::OK, Json(ApiResponse::success(dto))))
            }
            Err(e) => Ok((
                StatusCode::NOT_FOUND,
                Json(ApiResponse::error(
                    "FILE_CHUNKS_NOT_FOUND".to_string(),
                    e.to_string(),
                    None,
                )),
            )),
        }
    }

    pub async fn update_file(
        State(handler): State<Arc<FileHandler>>,
        Path(file_id): Path<Uuid>,
        Json(_update_request): Json<serde_json::Value>,
    ) -> Result<impl IntoResponse, StatusCode> {
        // Get the current file
        let file = match handler.file_repository.find_by_id(file_id).await {
            Ok(Some(f)) => f,
            Ok(None) => {
                return Ok((
                    StatusCode::NOT_FOUND,
                    Json(ApiResponse::error(
                        "FILE_NOT_FOUND".to_string(),
                        format!("File with ID {} not found", file_id),
                        None,
                    )),
                ));
            }
            Err(e) => {
                return Ok((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::error(
                        "DATABASE_ERROR".to_string(),
                        e.to_string(),
                        None,
                    )),
                ));
            }
        };

        match handler.file_repository.update(&file).await {
            Ok(_) => {
                let response =
                    crate::application::use_cases::get_file::GetFileResponse { file: file.clone() };
                let dto = FileDetailResponseDto::from(response);
                Ok((StatusCode::OK, Json(ApiResponse::success(dto))))
            }
            Err(e) => Ok((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(
                    "UPDATE_FAILED".to_string(),
                    e.to_string(),
                    None,
                )),
            )),
        }
    }

    pub async fn delete_file(
        State(handler): State<Arc<FileHandler>>,
        Path(file_id): Path<Uuid>,
    ) -> Result<impl IntoResponse, StatusCode> {
        match handler.file_repository.delete(file_id).await {
            Ok(true) => Ok((
                StatusCode::OK,
                Json(ApiResponse::success(
                    "File deleted successfully".to_string(),
                )),
            )),
            Ok(false) => Ok((
                StatusCode::NOT_FOUND,
                Json(ApiResponse::error(
                    "FILE_NOT_FOUND".to_string(),
                    format!("File with ID {} not found", file_id),
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

    pub async fn get_file_count(
        State(handler): State<Arc<FileHandler>>,
    ) -> Result<impl IntoResponse, StatusCode> {
        match handler.file_repository.count().await {
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

    pub async fn upload_file_with_processing(
        State(handler): State<Arc<FileHandler>>,
        mut multipart: Multipart,
    ) -> Result<impl IntoResponse, StatusCode> {
        // Parse auto_process parameter (default: true)
        let mut auto_process = true;
        let mut file_data = None;
        let mut file_name = None;
        let mut content_type = None;

        while let Some(field) = multipart
            .next_field()
            .await
            .map_err(|_| StatusCode::BAD_REQUEST)?
        {
            match field.name() {
                Some("file") => {
                    file_name = Some(
                        field
                            .file_name()
                            .ok_or(StatusCode::BAD_REQUEST)?
                            .to_string(),
                    );

                    content_type = field.content_type().map(|ct| ct.to_string());

                    file_data = Some(
                        field
                            .bytes()
                            .await
                            .map_err(|_| StatusCode::BAD_REQUEST)?
                            .to_vec(),
                    );
                }
                Some("auto_process") => {
                    if let Ok(data) = field.bytes().await {
                        if let Ok(value) = String::from_utf8(data.to_vec()) {
                            auto_process = value.parse().unwrap_or(true);
                        }
                    }
                }
                _ => {
                    // Skip unknown fields
                }
            }
        }

        let file_data = file_data.ok_or(StatusCode::BAD_REQUEST)?;
        let file_name = file_name.ok_or(StatusCode::BAD_REQUEST)?;

        let request = UploadWithProcessingRequest {
            file_data,
            file_name,
            content_type,
            auto_process,
            metadata: None,
        };

        match handler
            .upload_with_processing_use_case
            .execute(request)
            .await
        {
            Ok(response) => {
                let dto = UploadWithProcessingResponse::from(response);
                Ok((StatusCode::CREATED, Json(ApiResponse::success(dto))))
            }
            Err(e) => Ok((
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::error(
                    "UPLOAD_WITH_PROCESSING_FAILED".to_string(),
                    e.to_string(),
                    None,
                )),
            )),
        }
    }
}
