use std::sync::Arc;
use uuid::Uuid;

use crate::application::ports::FileStorage;
use crate::domain::entities::File;
use crate::domain::repositories::{FileRepository, file_repository::FileRepositoryError};
use crate::domain::value_objects::{FileHash, FileMetadata};

#[derive(Debug)]
pub enum UploadFileError {
    StorageError(String),
    RepositoryError(String),
    ValidationError(String),
    DuplicateFile(String),
}

impl std::fmt::Display for UploadFileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UploadFileError::StorageError(msg) => write!(f, "Storage error: {}", msg),
            UploadFileError::RepositoryError(msg) => write!(f, "Repository error: {}", msg),
            UploadFileError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            UploadFileError::DuplicateFile(msg) => write!(f, "Duplicate file: {}", msg),
        }
    }
}

impl std::error::Error for UploadFileError {}

impl From<FileRepositoryError> for UploadFileError {
    fn from(error: FileRepositoryError) -> Self {
        UploadFileError::RepositoryError(error.to_string())
    }
}

#[derive(Debug, Clone)]
pub struct UploadFileRequest {
    pub file_name: String,
    pub file_data: Vec<u8>,
    pub content_type: Option<String>,
    pub metadata: Option<FileMetadata>,
}

#[derive(Debug, Clone)]
pub struct UploadFileResponse {
    pub file_id: Uuid,
    pub file_name: String,
    pub file_size: i64,
    pub file_hash: String,
    pub content_type: Option<String>,
}

pub struct UploadFileUseCase {
    file_repository: Arc<dyn FileRepository>,
    file_storage: Arc<dyn FileStorage>,
}

impl UploadFileUseCase {
    pub fn new(
        file_repository: Arc<dyn FileRepository>,
        file_storage: Arc<dyn FileStorage>,
    ) -> Self {
        Self {
            file_repository,
            file_storage,
        }
    }

    pub async fn execute(
        &self,
        request: UploadFileRequest,
    ) -> Result<UploadFileResponse, UploadFileError> {
        // Validate input
        if request.file_name.trim().is_empty() {
            return Err(UploadFileError::ValidationError(
                "File name cannot be empty".to_string(),
            ));
        }

        if request.file_data.is_empty() {
            return Err(UploadFileError::ValidationError(
                "File data cannot be empty".to_string(),
            ));
        }

        // Generate file hash
        let file_hash = FileHash::from_bytes(&request.file_data);

        // Check for duplicate files
        if let Ok(Some(_)) = self.file_repository.find_by_hash(file_hash.as_str()).await {
            return Err(UploadFileError::DuplicateFile(
                "File with this hash already exists".to_string(),
            ));
        }

        // Store file
        let stored_file = self
            .file_storage
            .store_file(
                &request.file_data,
                &request.file_name,
                request.content_type.as_deref(),
            )
            .await
            .map_err(|e| UploadFileError::StorageError(e.to_string()))?;

        // Create domain entity
        let file = File::new(
            stored_file.path,
            request.file_name.clone(),
            Some(request.file_data.len() as i64),
            request.content_type.clone(),
            Some(file_hash.clone()),
            request.metadata,
        );

        // Save to repository
        self.file_repository.save(&file).await?;

        Ok(UploadFileResponse {
            file_id: file.id(),
            file_name: request.file_name,
            file_size: request.file_data.len() as i64,
            file_hash: file_hash.to_string(),
            content_type: request.content_type,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::ports::file_storage::{FileStorageError, StoredFile};
    use crate::domain::repositories::file_repository::FileRepositoryError;
    use async_trait::async_trait;
    use std::collections::HashMap;
    use tokio::sync::Mutex;

    // Mock implementations for testing
    struct MockFileRepository {
        files: Arc<Mutex<HashMap<Uuid, File>>>,
        files_by_hash: Arc<Mutex<HashMap<String, File>>>,
    }

    impl MockFileRepository {
        fn new() -> Self {
            Self {
                files: Arc::new(Mutex::new(HashMap::new())),
                files_by_hash: Arc::new(Mutex::new(HashMap::new())),
            }
        }
    }

    #[async_trait]
    impl FileRepository for MockFileRepository {
        async fn save(&self, file: &File) -> Result<(), FileRepositoryError> {
            let mut files = self.files.lock().await;
            let mut files_by_hash = self.files_by_hash.lock().await;

            files.insert(file.id(), file.clone());
            if let Some(hash) = file.file_hash() {
                files_by_hash.insert(hash.as_str().to_string(), file.clone());
            }
            Ok(())
        }

        async fn find_by_id(&self, id: Uuid) -> Result<Option<File>, FileRepositoryError> {
            let files = self.files.lock().await;
            Ok(files.get(&id).cloned())
        }

        async fn find_by_hash(&self, hash: &str) -> Result<Option<File>, FileRepositoryError> {
            let files_by_hash = self.files_by_hash.lock().await;
            Ok(files_by_hash.get(hash).cloned())
        }

        async fn find_all(
            &self,
            _skip: i64,
            _limit: i64,
        ) -> Result<Vec<File>, FileRepositoryError> {
            unimplemented!()
        }

        async fn update(&self, _file: &File) -> Result<(), FileRepositoryError> {
            unimplemented!()
        }

        async fn delete(&self, _id: Uuid) -> Result<bool, FileRepositoryError> {
            unimplemented!()
        }

        async fn count(&self) -> Result<i64, FileRepositoryError> {
            unimplemented!()
        }

       
    }

    struct MockFileStorage;

    #[async_trait]
    impl FileStorage for MockFileStorage {
        async fn store_file(
            &self,
            data: &[u8],
            file_name: &str,
            content_type: Option<&str>,
        ) -> Result<StoredFile, FileStorageError> {
            Ok(StoredFile {
                id: Uuid::new_v4(),
                path: format!("/tmp/{}", file_name),
                size: data.len() as u64,
                content_type: content_type.map(|s| s.to_string()),
            })
        }

        async fn retrieve_file(&self, _file_id: Uuid) -> Result<Vec<u8>, FileStorageError> {
            unimplemented!()
        }

        async fn retrieve_file_path(&self, _file_id: Uuid) -> Result<String, FileStorageError> {
            unimplemented!()
        }

        async fn delete_file(&self, _file_id: Uuid) -> Result<bool, FileStorageError> {
            unimplemented!()
        }

        async fn file_exists(&self, _file_id: Uuid) -> Result<bool, FileStorageError> {
            unimplemented!()
        }

        async fn get_file_size(&self, _file_id: Uuid) -> Result<u64, FileStorageError> {
            unimplemented!()
        }

        async fn get_storage_info(
            &self,
        ) -> Result<crate::application::ports::file_storage::StorageInfo, FileStorageError> {
            unimplemented!()
        }
    }

    #[tokio::test]
    async fn test_upload_file_success() {
        let file_repo = Arc::new(MockFileRepository::new());
        let file_storage = Arc::new(MockFileStorage);
        let use_case = UploadFileUseCase::new(file_repo, file_storage);

        let request = UploadFileRequest {
            file_name: "test.pdf".to_string(),
            file_data: b"test file content".to_vec(),
            content_type: Some("application/pdf".to_string()),
            metadata: None,
        };

        let result = use_case.execute(request).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.file_name, "test.pdf");
        assert_eq!(response.file_size, 17);
    }

    #[tokio::test]
    async fn test_upload_file_empty_name() {
        let file_repo = Arc::new(MockFileRepository::new());
        let file_storage = Arc::new(MockFileStorage);
        let use_case = UploadFileUseCase::new(file_repo, file_storage);

        let request = UploadFileRequest {
            file_name: "".to_string(),
            file_data: b"test file content".to_vec(),
            content_type: None,
            metadata: None,
        };

        let result = use_case.execute(request).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            UploadFileError::ValidationError(_)
        ));
    }
}
