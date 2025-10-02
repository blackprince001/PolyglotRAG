use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct SearchRequestDto {
    pub query: String,
    #[serde(default = "default_limit")]
    pub limit: Option<i32>,
    pub similarity_threshold: Option<f32>,
    pub file_id: Option<Uuid>,
}

fn default_limit() -> Option<i32> {
    Some(10)
}

#[derive(Debug, Serialize)]
pub struct SearchResponseDto {
    pub query: String,
    pub results: Vec<SearchResultDto>,
    pub total_results: i32,
    pub search_time_ms: u64,
}

#[derive(Debug, Serialize)]
pub struct SearchResultDto {
    pub chunk_id: Uuid,
    pub file_id: Uuid,
    pub chunk_text: String,
    pub similarity_score: f32,
    pub chunk_index: i32,
    pub page_number: Option<i32>,
    pub section_path: Option<String>,
}

impl From<crate::application::use_cases::search_content::SearchContentResponse> for SearchResponseDto {
    fn from(response: crate::application::use_cases::search_content::SearchContentResponse) -> Self {
        Self {
            query: response.query,
            results: response.results.into_iter().map(SearchResultDto::from).collect(),
            total_results: response.total_results,
            search_time_ms: response.search_time_ms,
        }
    }
}

impl From<crate::application::use_cases::search_content::SearchResult> for SearchResultDto {
    fn from(result: crate::application::use_cases::search_content::SearchResult) -> Self {
        Self {
            chunk_id: result.chunk.id(),
            file_id: result.file_id,
            chunk_text: result.chunk.chunk_text().to_string(),
            similarity_score: result.similarity_score,
            chunk_index: result.chunk.chunk_index(),
            page_number: result.chunk.page_number(),
            section_path: result.chunk.section_path().map(|s| s.to_string()),
        }
    }
}
