use async_trait::async_trait;
use std::path::PathBuf;
use tokio::fs;
use uuid::Uuid;

use crate::application::ports::file_storage::{
    FileStorage, FileStorageError, StorageInfo, StoredFile,
};

pub struct LocalFileStorage {
    base_path: PathBuf,
}

impl LocalFileStorage {
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }

    pub async fn ensure_directory_exists(&self) -> Result<(), FileStorageError> {
        fs::create_dir_all(&self.base_path)
            .await
            .map_err(|e| FileStorageError::IoError(e.to_string()))
    }

    fn get_file_path(&self, file_id: Uuid) -> PathBuf {
        self.base_path.join(file_id.to_string())
    }
}

#[async_trait]
impl FileStorage for LocalFileStorage {
    async fn store_file(
        &self,
        data: &[u8],
        file_name: &str,
        content_type: Option<&str>,
    ) -> Result<StoredFile, FileStorageError> {
        self.ensure_directory_exists().await?;

        let file_id = Uuid::new_v4();
        let file_path = self.get_file_path(file_id);

        fs::write(&file_path, data)
            .await
            .map_err(|e| FileStorageError::IoError(e.to_string()))?;

        Ok(StoredFile {
            id: file_id,
            path: file_path.to_string_lossy().to_string(),
            size: data.len() as u64,
            content_type: content_type.map(|s| s.to_string()),
        })
    }

    async fn retrieve_file(&self, file_id: Uuid) -> Result<Vec<u8>, FileStorageError> {
        let file_path = self.get_file_path(file_id);

        if !file_path.exists() {
            return Err(FileStorageError::FileNotFound(file_id.to_string()));
        }

        fs::read(&file_path)
            .await
            .map_err(|e| FileStorageError::IoError(e.to_string()))
    }

    async fn retrieve_file_path(&self, file_id: Uuid) -> Result<String, FileStorageError> {
        let file_path = self.get_file_path(file_id);

        if !file_path.exists() {
            return Err(FileStorageError::FileNotFound(file_id.to_string()));
        }

        Ok(file_path.to_string_lossy().to_string())
    }

    async fn delete_file(&self, file_id: Uuid) -> Result<bool, FileStorageError> {
        let file_path = self.get_file_path(file_id);

        if !file_path.exists() {
            return Ok(false);
        }

        fs::remove_file(&file_path)
            .await
            .map_err(|e| FileStorageError::IoError(e.to_string()))?;

        Ok(true)
    }

    async fn file_exists(&self, file_id: Uuid) -> Result<bool, FileStorageError> {
        let file_path = self.get_file_path(file_id);
        Ok(file_path.exists())
    }

    async fn get_file_size(&self, file_id: Uuid) -> Result<u64, FileStorageError> {
        let file_path = self.get_file_path(file_id);

        if !file_path.exists() {
            return Err(FileStorageError::FileNotFound(file_id.to_string()));
        }

        let metadata = fs::metadata(&file_path)
            .await
            .map_err(|e| FileStorageError::IoError(e.to_string()))?;

        Ok(metadata.len())
    }

    async fn get_storage_info(&self) -> Result<StorageInfo, FileStorageError> {
        // Get directory entries
        let mut entries = fs::read_dir(&self.base_path)
            .await
            .map_err(|e| FileStorageError::IoError(e.to_string()))?;

        let mut file_count = 0u64;
        let mut used_space = 0u64;

        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| FileStorageError::IoError(e.to_string()))?
        {
            if entry
                .file_type()
                .await
                .map_err(|e| FileStorageError::IoError(e.to_string()))?
                .is_file()
            {
                file_count += 1;
                let metadata = entry
                    .metadata()
                    .await
                    .map_err(|e| FileStorageError::IoError(e.to_string()))?;
                used_space += metadata.len();
            }
        }

        let total_space: u64 = 1024 * 1024 * 1024 * 100;
        let available_space = total_space.saturating_sub(used_space);

        Ok(StorageInfo {
            total_space,
            used_space,
            available_space,
            file_count,
        })
    }
}
