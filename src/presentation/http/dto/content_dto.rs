use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Request DTOs for direct content processing
#[derive(Debug, Deserialize)]
pub struct ProcessUrlRequest {
    pub url: String,
    pub filename: Option<String>,
    pub auto_process: Option<bool>, // Default: true
}

#[derive(Debug, Deserialize)]
pub struct ProcessYoutubeRequest {
    pub url: String,
    pub filename: Option<String>,
    pub extract_timestamps: Option<bool>, // Default: true
    pub language_preference: Option<Vec<String>>, // Default: ["en"]
    pub auto_process: Option<bool>,       // Default: true
}

// Response DTOs
#[derive(Debug, Serialize)]
pub struct ContentProcessingResponse {
    pub job_id: Option<Uuid>,
    pub file_id: Uuid,
    pub source_url: Option<String>,
    pub source_type: String, // "url", "youtube", "file"
    pub filename: String,
    pub status: String,
    pub message: String,
    pub estimated_completion_minutes: Option<u8>,
    pub progress_stream_url: Option<String>,
}

impl From<crate::application::use_cases::process_url_direct::ProcessUrlDirectResponse>
    for ContentProcessingResponse
{
    fn from(
        response: crate::application::use_cases::process_url_direct::ProcessUrlDirectResponse,
    ) -> Self {
        Self {
            job_id: Some(response.job_id),
            file_id: response.file_id,
            source_url: Some(response.url),
            source_type: "url".to_string(),
            filename: response.filename,
            status: response.status,
            message: response.message,
            estimated_completion_minutes: Some(3), // Typical URL processing time
            progress_stream_url: Some(format!("/jobs/{}/stream", response.job_id)),
        }
    }
}

impl From<crate::application::use_cases::process_youtube_direct::ProcessYoutubeDirectResponse>
    for ContentProcessingResponse
{
    fn from(
        response: crate::application::use_cases::process_youtube_direct::ProcessYoutubeDirectResponse,
    ) -> Self {
        Self {
            job_id: Some(response.job_id),
            file_id: response.file_id,
            source_url: Some(response.url),
            source_type: "youtube".to_string(),
            filename: response.filename,
            status: response.status,
            message: response.message,
            estimated_completion_minutes: Some(5), // YouTube processing typically takes longer
            progress_stream_url: Some(format!("/jobs/{}/stream", response.job_id)),
        }
    }
}


#[derive(Debug, Serialize)]
pub struct UploadWithProcessingResponse {
    pub file_id: Uuid,
    pub job_id: Option<Uuid>,
    pub file_name: String,
    pub file_size: i64,
    pub file_hash: String,
    pub content_type: Option<String>,
    pub status: String,
    pub message: String,
    pub progress_stream_url: Option<String>,
}

impl From<crate::application::use_cases::upload_with_processing::UploadWithProcessingResponse>
    for UploadWithProcessingResponse
{
    fn from(
        response: crate::application::use_cases::upload_with_processing::UploadWithProcessingResponse,
    ) -> Self {
        Self {
            file_id: response.file_id,
            job_id: response.job_id,
            file_name: response.file_name,
            file_size: response.file_size,
            file_hash: response.file_hash,
            content_type: response.content_type,
            status: response.status,
            message: response.message,
            progress_stream_url: response.job_id.map(|id| format!("/jobs/{}/stream", id)),
        }
    }
}
