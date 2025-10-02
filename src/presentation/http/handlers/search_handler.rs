use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use std::sync::Arc;

use crate::application::use_cases::{SearchContentUseCase, search_content::SearchContentRequest};
use crate::presentation::http::dto::{ApiResponse, SearchRequestDto, SearchResponseDto};

pub struct SearchHandler {
    search_use_case: Arc<SearchContentUseCase>,
}

impl SearchHandler {
    pub fn new(search_use_case: Arc<SearchContentUseCase>) -> Self {
        Self { search_use_case }
    }

    pub async fn search_content(
        State(handler): State<Arc<SearchHandler>>,
        Query(search_params): Query<SearchRequestDto>,
    ) -> Result<impl IntoResponse, StatusCode> {
        if search_params.query.trim().is_empty() {
            return Ok((
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::error(
                    "EMPTY_QUERY".to_string(),
                    "Query cannot be empty".to_string(),
                    None,
                )),
            ));
        }

        let request = SearchContentRequest {
            query: search_params.query,
            limit: search_params.limit,
            similarity_threshold: search_params.similarity_threshold,
            file_id_filter: search_params.file_id,
        };

        match handler.search_use_case.execute(request).await {
            Ok(response) => {
                let dto = SearchResponseDto::from(response);
                Ok((
                    StatusCode::OK,
                    Json(ApiResponse::<SearchResponseDto>::success(dto)),
                ))
            }
            Err(e) => Ok((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(
                    "SEARCH_FAILED".to_string(),
                    e.to_string(),
                    None,
                )),
            )),
        }
    }
}
