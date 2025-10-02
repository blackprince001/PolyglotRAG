use async_trait::async_trait;
use uuid::Uuid;

#[derive(Debug)]
pub enum FileStorageError {
    FileNotFound(String),
    PermissionDenied(String),
    StorageFull,
    IoError(String),
    InvalidPath(String),
}

impl std::fmt::Display for FileStorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileStorageError::FileNotFound(path) => write!(f, "File not found: {}", path),
            FileStorageError::PermissionDenied(msg) => write!(f, "Permission denied: {}", msg),
            FileStorageError::StorageFull => write!(f, "Storage full"),
            FileStorageError::IoError(msg) => write!(f, "IO error: {}", msg),
            FileStorageError::InvalidPath(path) => write!(f, "Invalid path: {}", path),
        }
    }
}

impl std::error::Error for FileStorageError {}

#[derive(Debug, Clone)]
pub struct StoredFile {
    pub id: Uuid,
    pub path: String,
    pub size: u64,
    pub content_type: Option<String>,
}

#[async_trait]
pub trait FileStorage: Send + Sync {
    async fn store_file(
        &self,
        data: &[u8],
        file_name: &str,
        content_type: Option<&str>,
    ) -> Result<StoredFile, FileStorageError>;
    
    async fn retrieve_file(&self, file_id: Uuid) -> Result<Vec<u8>, FileStorageError>;
    
    async fn retrieve_file_path(&self, file_id: Uuid) -> Result<String, FileStorageError>;
    
    async fn delete_file(&self, file_id: Uuid) -> Result<bool, FileStorageError>;
    
    async fn file_exists(&self, file_id: Uuid) -> Result<bool, FileStorageError>;
    
    async fn get_file_size(&self, file_id: Uuid) -> Result<u64, FileStorageError>;
    
    async fn get_storage_info(&self) -> Result<StorageInfo, FileStorageError>;
}

#[derive(Debug, Clone)]
pub struct StorageInfo {
    pub total_space: u64,
    pub used_space: u64,
    pub available_space: u64,
    pub file_count: u64,
}
