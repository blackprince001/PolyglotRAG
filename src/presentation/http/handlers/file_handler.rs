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
    UploadFileUseCase, get_file::GetFileRequest, get_file_chunks::GetFileChunksRequest,
    list_files::ListFilesRequest, process_document::ProcessDocumentRequest,
    upload_file::UploadFileRequest,
};
use crate::presentation::http::dto::{
    ApiResponse, PaginationDto, PaginationMetaDto, file_dto::FileChunksResponseDto,
    file_dto::FileDetailResponseDto, file_dto::FileListResponseDto, file_dto::FileResponseDto,
    file_dto::ProcessFileResponseDto, file_dto::UploadResponseDto,
};

pub struct FileHandler {
    upload_use_case: Arc<UploadFileUseCase>,
    list_files_use_case: Arc<ListFilesUseCase>,
    process_document_use_case: Arc<ProcessDocumentUseCase>,
    get_file_use_case: Arc<GetFileUseCase>,
    get_file_chunks_use_case: Arc<GetFileChunksUseCase>,
}

impl FileHandler {
    pub fn new(
        upload_use_case: Arc<UploadFileUseCase>,
        list_files_use_case: Arc<ListFilesUseCase>,
        process_document_use_case: Arc<ProcessDocumentUseCase>,
        get_file_use_case: Arc<GetFileUseCase>,
        get_file_chunks_use_case: Arc<GetFileChunksUseCase>,
    ) -> Self {
        Self {
            upload_use_case,
            list_files_use_case,
            process_document_use_case,
            get_file_use_case,
            get_file_chunks_use_case,
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
}
