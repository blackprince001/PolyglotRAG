use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::application::use_cases::{
    CancelJobUseCase, GetJobStatusUseCase, QueueProcessingJobUseCase, cancel_job::CancelJobRequest,
    get_job_status::GetJobStatusRequest,
};
use crate::presentation::http::dto::{
    ApiResponse, CancelJobResponseDto, JobStatusDto, ProcessUrlRequestDto,
    ProcessYoutubeRequestDto, QueueJobResponseDto,
};

pub struct JobHandler {
    queue_job_use_case: Arc<QueueProcessingJobUseCase>,
    get_job_status_use_case: Arc<GetJobStatusUseCase>,
    cancel_job_use_case: Arc<CancelJobUseCase>,
}

impl JobHandler {
    pub fn new(
        queue_job_use_case: Arc<QueueProcessingJobUseCase>,
        get_job_status_use_case: Arc<GetJobStatusUseCase>,
        cancel_job_use_case: Arc<CancelJobUseCase>,
    ) -> Self {
        Self {
            queue_job_use_case,
            get_job_status_use_case,
            cancel_job_use_case,
        }
    }

    // Queue file processing job
    pub async fn queue_file_processing(
        State(handler): State<Arc<JobHandler>>,
        Path(file_id): Path<Uuid>,
    ) -> Result<impl IntoResponse, StatusCode> {
        match handler
            .queue_job_use_case
            .queue_file_processing(file_id)
            .await
        {
            Ok(response) => {
                let dto = QueueJobResponseDto::from(response);
                Ok((StatusCode::ACCEPTED, Json(ApiResponse::success(dto))))
            }
            Err(e) => Ok((
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::error(
                    "QUEUE_FAILED".to_string(),
                    e.to_string(),
                    None,
                )),
            )),
        }
    }

    // Queue URL extraction job
    pub async fn queue_url_extraction(
        State(handler): State<Arc<JobHandler>>,
        Path(file_id): Path<Uuid>,
        Json(request): Json<ProcessUrlRequestDto>,
    ) -> Result<impl IntoResponse, StatusCode> {
        match handler
            .queue_job_use_case
            .queue_url_extraction(file_id, request.url)
            .await
        {
            Ok(response) => {
                let dto = QueueJobResponseDto::from(response);
                Ok((StatusCode::ACCEPTED, Json(ApiResponse::success(dto))))
            }
            Err(e) => Ok((
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::error(
                    "QUEUE_FAILED".to_string(),
                    e.to_string(),
                    None,
                )),
            )),
        }
    }

    // Queue YouTube extraction job
    pub async fn queue_youtube_extraction(
        State(handler): State<Arc<JobHandler>>,
        Path(file_id): Path<Uuid>,
        Json(request): Json<ProcessYoutubeRequestDto>,
    ) -> Result<impl IntoResponse, StatusCode> {
        match handler
            .queue_job_use_case
            .queue_youtube_extraction(file_id, request.url)
            .await
        {
            Ok(response) => {
                let dto = QueueJobResponseDto::from(response);
                Ok((StatusCode::ACCEPTED, Json(ApiResponse::success(dto))))
            }
            Err(e) => Ok((
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::error(
                    "QUEUE_FAILED".to_string(),
                    e.to_string(),
                    None,
                )),
            )),
        }
    }

    // Get job status
    pub async fn get_job_status(
        State(handler): State<Arc<JobHandler>>,
        Path(job_id): Path<Uuid>,
    ) -> Result<impl IntoResponse, StatusCode> {
        let request = GetJobStatusRequest { job_id };

        match handler.get_job_status_use_case.execute(request).await {
            Ok(response) => {
                let dto = JobStatusDto::from(response);
                Ok((StatusCode::OK, Json(ApiResponse::success(dto))))
            }
            Err(e) => Ok((
                StatusCode::NOT_FOUND,
                Json(ApiResponse::error(
                    "JOB_NOT_FOUND".to_string(),
                    e.to_string(),
                    None,
                )),
            )),
        }
    }

    // Get jobs for a specific file
    pub async fn get_file_jobs(
        State(handler): State<Arc<JobHandler>>,
        Path(file_id): Path<Uuid>,
    ) -> Result<impl IntoResponse, StatusCode> {
        match handler
            .get_job_status_use_case
            .get_jobs_for_file(file_id)
            .await
        {
            Ok(jobs) => {
                let dtos: Vec<JobStatusDto> =
                    jobs.into_iter().map(JobStatusDto::from_job).collect();
                Ok((StatusCode::OK, Json(ApiResponse::success(dtos))))
            }
            Err(e) => Ok((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(
                    "FETCH_FAILED".to_string(),
                    e.to_string(),
                    None,
                )),
            )),
        }
    }

    // Get all active jobs
    pub async fn get_active_jobs(
        State(handler): State<Arc<JobHandler>>,
    ) -> Result<impl IntoResponse, StatusCode> {
        match handler.get_job_status_use_case.get_active_jobs().await {
            Ok(jobs) => {
                let dtos: Vec<JobStatusDto> =
                    jobs.into_iter().map(JobStatusDto::from_job).collect();
                Ok((StatusCode::OK, Json(ApiResponse::success(dtos))))
            }
            Err(e) => Ok((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(
                    "FETCH_FAILED".to_string(),
                    e.to_string(),
                    None,
                )),
            )),
        }
    }

    // Cancel job
    pub async fn cancel_job(
        State(handler): State<Arc<JobHandler>>,
        Path(job_id): Path<Uuid>,
    ) -> Result<impl IntoResponse, StatusCode> {
        let request = CancelJobRequest { job_id };

        match handler.cancel_job_use_case.execute(request).await {
            Ok(response) => {
                let dto = CancelJobResponseDto::from(response);
                Ok((StatusCode::OK, Json(ApiResponse::success(dto))))
            }
            Err(e) => Ok((
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::error(
                    "CANCEL_FAILED".to_string(),
                    e.to_string(),
                    None,
                )),
            )),
        }
    }
}
