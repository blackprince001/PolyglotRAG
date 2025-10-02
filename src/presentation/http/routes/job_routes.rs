use axum::{Router, routing::delete, routing::get, routing::post};
use std::sync::Arc;

use crate::presentation::http::handlers::{JobHandler, SseHandler};

pub fn job_routes(job_handler: Arc<JobHandler>, sse_handler: Arc<SseHandler>) -> Router {
    Router::new()
        // Job management endpoints
        .route(
            "/jobs/process/file/{file_id}",
            post(JobHandler::queue_file_processing),
        )
        .route(
            "/jobs/process/url/{file_id}",
            post(JobHandler::queue_url_extraction),
        )
        .route(
            "/jobs/process/youtube/{file_id}",
            post(JobHandler::queue_youtube_extraction),
        )
        .route("/jobs/{job_id}", get(JobHandler::get_job_status))
        .route("/jobs/{job_id}/cancel", delete(JobHandler::cancel_job))
        .route("/jobs/file/{file_id}", get(JobHandler::get_file_jobs))
        .route("/jobs/active", get(JobHandler::get_active_jobs))
        .nest(
            "/jobs",
            Router::new()
                .route("/{job_id}/stream", get(SseHandler::job_progress_stream))
                .route("/stream", get(SseHandler::multiple_jobs_stream))
                .with_state(sse_handler),
        )
        .with_state(job_handler)
}
