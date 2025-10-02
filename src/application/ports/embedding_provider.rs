use async_trait::async_trait;
use pgvector::Vector;

#[derive(Debug)]
pub enum EmbeddingProviderError {
    NetworkError(String),
    ApiError(String),
    InvalidInput(String),
    RateLimitExceeded,
    ServiceUnavailable,
}

impl std::fmt::Display for EmbeddingProviderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EmbeddingProviderError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            EmbeddingProviderError::ApiError(msg) => write!(f, "API error: {}", msg),
            EmbeddingProviderError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            EmbeddingProviderError::RateLimitExceeded => write!(f, "Rate limit exceeded"),
            EmbeddingProviderError::ServiceUnavailable => write!(f, "Service unavailable"),
        }
    }
}

impl std::error::Error for EmbeddingProviderError {}

#[derive(Debug, Clone)]
pub struct EmbeddingRequest {
    pub text: String,
    pub model_name: Option<String>,
    pub model_version: Option<String>,
}

#[derive(Debug, Clone)]
pub struct EmbeddingResponse {
    pub embedding: Vector,
    pub model_name: String,
    pub model_version: Option<String>,
    pub token_count: Option<i32>,
}

#[derive(Debug, Clone)]
pub struct BatchEmbeddingRequest {
    pub texts: Vec<String>,
    pub model_name: Option<String>,
    pub model_version: Option<String>,
}

#[derive(Debug, Clone)]
pub struct BatchEmbeddingResponse {
    pub embeddings: Vec<Vector>,
    pub model_name: String,
    pub model_version: Option<String>,
    pub total_tokens: Option<i32>,
}

#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    async fn generate_embedding(
        &self,
        request: EmbeddingRequest,
    ) -> Result<EmbeddingResponse, EmbeddingProviderError>;

    async fn generate_embeddings(
        &self,
        request: BatchEmbeddingRequest,
    ) -> Result<BatchEmbeddingResponse, EmbeddingProviderError>;

    async fn health_check(&self) -> Result<bool, EmbeddingProviderError>;

    fn model_info(&self) -> (String, Option<String>);

    fn max_input_length(&self) -> usize;

    fn embedding_dimension(&self) -> usize;
}
