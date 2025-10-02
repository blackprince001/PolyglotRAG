use async_trait::async_trait;
use pgvector::Vector;
use reqwest::{Client, Error as ReqwestError};
use serde::{Deserialize, Serialize};
use std::env;
use std::time::Duration;

use crate::application::ports::embedding_provider::{
    BatchEmbeddingRequest, BatchEmbeddingResponse, EmbeddingProvider, EmbeddingProviderError,
    EmbeddingRequest, EmbeddingResponse,
};

#[derive(Serialize)]
pub struct EmbeddingsRequest {
    pub text: TextInput,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum TextInput {
    Single(String),
    Multiple(Vec<String>),
}

#[derive(Deserialize)]
pub struct EmbeddingsResponse {
    pub success: bool,
    pub input_text: TextInput,
    pub embeddings: Vec<Vector>,
    pub shape: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct EmbeddingsClientConfig {
    pub service_url: String,
    pub max_retries: u32,
    pub timeout_secs: u64,
    pub backoff_factor: f64,
}

impl Default for EmbeddingsClientConfig {
    fn default() -> Self {
        let service_url = env::var("EMBEDDINGS_SERVICE_URL")
            .unwrap_or_else(|_| "https://example.workers.dev".to_string());

        Self {
            service_url,
            max_retries: 3,
            timeout_secs: 30,
            backoff_factor: 1.5,
        }
    }
}

#[derive(Debug)]
pub enum EmbeddingsError {
    RequestError(String),
    ParseError(String),
    MaxRetriesExceeded(String),
}

#[derive(Debug, Clone)]
pub struct InferenceClient {
    client: Client,
    config: EmbeddingsClientConfig,
}

impl InferenceClient {
    pub fn new(config: EmbeddingsClientConfig) -> Result<Self, ReqwestError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()?;

        Ok(Self { client, config })
    }

    pub fn from_env() -> Result<Self, ReqwestError> {
        Self::new(EmbeddingsClientConfig::default())
    }

    pub async fn get_embedding(&self, text: &str) -> Result<EmbeddingsResponse, EmbeddingsError> {
        let request = EmbeddingsRequest {
            text: TextInput::Single(text.to_string()),
        };

        self.send_request(request).await
    }

    pub async fn get_embeddings(
        &self,
        texts: &Vec<String>,
    ) -> Result<EmbeddingsResponse, EmbeddingsError> {
        let request = EmbeddingsRequest {
            text: TextInput::Multiple(texts.to_vec()),
        };

        self.send_request(request).await
    }

    async fn send_request(
        &self,
        request: EmbeddingsRequest,
    ) -> Result<EmbeddingsResponse, EmbeddingsError> {
        let mut attempts = 0;
        let mut last_error = None;

        loop {
            attempts += 1;

            let result = self.execute_request(&request).await;

            match result {
                Ok(response) => return Ok(response),
                Err(e) => {
                    last_error = Some(e);

                    if attempts > self.config.max_retries {
                        break;
                    }

                    let backoff_time = Duration::from_millis(
                        (self.config.backoff_factor.powi(attempts as i32 - 1) * 1000.0) as u64,
                    );

                    tokio::time::sleep(backoff_time).await;
                }
            }
        }

        Err(last_error.unwrap_or(EmbeddingsError::MaxRetriesExceeded(
            "Max retries exceeded".to_string(),
        )))
    }

    async fn execute_request(
        &self,
        request: &EmbeddingsRequest,
    ) -> Result<EmbeddingsResponse, EmbeddingsError> {
        let response = self
            .client
            .post(&self.config.service_url)
            .header("Content-Type", "application/json")
            .json(request)
            .send()
            .await;

        if response.is_ok() {
            let response_data = response
                .unwrap()
                .json::<EmbeddingsResponse>()
                .await
                .map_err(|e| EmbeddingsError::ParseError(e.to_string()))
                .expect("Failed to parse json to embedding_response.");

            return Ok(response_data);
        }

        let error_message = format!("Error: {}", &response.err().unwrap().without_url());

        Err(EmbeddingsError::RequestError(error_message))
    }
}

// Adapter to implement the EmbeddingProvider trait
pub struct InferenceEmbeddingProvider {
    client: InferenceClient,
}

impl InferenceEmbeddingProvider {
    pub fn new(client: InferenceClient) -> Self {
        Self { client }
    }

    pub fn from_env() -> Result<Self, ReqwestError> {
        let client = InferenceClient::from_env()?;
        Ok(Self { client })
    }
}

#[async_trait]
impl EmbeddingProvider for InferenceEmbeddingProvider {
    async fn generate_embedding(
        &self,
        request: EmbeddingRequest,
    ) -> Result<EmbeddingResponse, EmbeddingProviderError> {
        let response = self
            .client
            .get_embedding(&request.text)
            .await
            .map_err(|e| match e {
                EmbeddingsError::RequestError(msg) => EmbeddingProviderError::NetworkError(msg),
                EmbeddingsError::ParseError(msg) => EmbeddingProviderError::ApiError(msg),
                EmbeddingsError::MaxRetriesExceeded(msg) => {
                    EmbeddingProviderError::ServiceUnavailable
                }
            })?;

        if response.embeddings.is_empty() {
            return Err(EmbeddingProviderError::ApiError(
                "No embeddings returned".to_string(),
            ));
        }

        Ok(EmbeddingResponse {
            embedding: response.embeddings[0].clone(),
            model_name: request.model_name.unwrap_or_else(|| "default".to_string()),
            model_version: request.model_version,
            token_count: None, // Not provided by the current API
        })
    }

    async fn generate_embeddings(
        &self,
        request: BatchEmbeddingRequest,
    ) -> Result<BatchEmbeddingResponse, EmbeddingProviderError> {
        let response = self
            .client
            .get_embeddings(&request.texts)
            .await
            .map_err(|e| match e {
                EmbeddingsError::RequestError(msg) => EmbeddingProviderError::NetworkError(msg),
                EmbeddingsError::ParseError(msg) => EmbeddingProviderError::ApiError(msg),
                EmbeddingsError::MaxRetriesExceeded(msg) => {
                    EmbeddingProviderError::ServiceUnavailable
                }
            })?;

        Ok(BatchEmbeddingResponse {
            embeddings: response.embeddings,
            model_name: request.model_name.unwrap_or_else(|| "default".to_string()),
            model_version: request.model_version,
            total_tokens: None, // Not provided by the current API
        })
    }

    async fn health_check(&self) -> Result<bool, EmbeddingProviderError> {
        // Simple health check by trying to embed a test string
        let test_request = EmbeddingRequest {
            text: "health check".to_string(),
            model_name: None,
            model_version: None,
        };

        match self.generate_embedding(test_request).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    fn model_info(&self) -> (String, Option<String>) {
        ("default".to_string(), None)
    }

    fn max_input_length(&self) -> usize {
        8192 // Default max input length
    }

    fn embedding_dimension(&self) -> usize {
        1536 // Default embedding dimension - should be configurable
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_construction() {
        let single_request = EmbeddingsRequest {
            text: TextInput::Single("Hello world".to_string()),
        };

        assert!(matches!(single_request.text, TextInput::Single(_)));

        let multiple_request = EmbeddingsRequest {
            text: TextInput::Multiple(vec!["Hello".to_string(), "World".to_string()]),
        };

        assert!(matches!(multiple_request.text, TextInput::Multiple(_)));
        if let TextInput::Multiple(texts) = multiple_request.text {
            assert_eq!(texts.len(), 2);
            assert_eq!(texts[0], "Hello");
            assert_eq!(texts[1], "World");
        }
    }
}
