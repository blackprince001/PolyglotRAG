use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::entities::File;

#[derive(Debug)]
pub enum FileRepositoryError {
    NotFound(Uuid),
    DatabaseError(String),
    ValidationError(String),
    DuplicateError(String),
}

impl std::fmt::Display for FileRepositoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileRepositoryError::NotFound(id) => write!(f, "File not found: {}", id),
            FileRepositoryError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            FileRepositoryError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            FileRepositoryError::DuplicateError(msg) => write!(f, "Duplicate error: {}", msg),
        }
    }
}

impl std::error::Error for FileRepositoryError {}

#[async_trait]
pub trait FileRepository: Send + Sync {
    async fn save(&self, file: &File) -> Result<(), FileRepositoryError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<File>, FileRepositoryError>;
    async fn find_by_hash(&self, hash: &str) -> Result<Option<File>, FileRepositoryError>;
    async fn find_all(&self, skip: i64, limit: i64) -> Result<Vec<File>, FileRepositoryError>;
    async fn update(&self, file: &File) -> Result<(), FileRepositoryError>;
    async fn delete(&self, id: Uuid) -> Result<bool, FileRepositoryError>;
    async fn count(&self) -> Result<i64, FileRepositoryError>;
}
