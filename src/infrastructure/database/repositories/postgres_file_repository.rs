use async_trait::async_trait;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use uuid::Uuid;

use crate::domain::entities::File;
use crate::domain::repositories::{FileRepository, file_repository::FileRepositoryError};
use crate::infrastructure::database::get_connection_from_pool;
use crate::infrastructure::database::models::{FileModel, NewFileModel};
use crate::infrastructure::database::schema::files::dsl::*;

pub struct PostgresFileRepository {
    pool: Pool<ConnectionManager<PgConnection>>,
}

impl PostgresFileRepository {
    pub fn new(pool: Pool<ConnectionManager<PgConnection>>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl FileRepository for PostgresFileRepository {
    async fn save(&self, file: &File) -> Result<Uuid, FileRepositoryError> {
        let mut conn = get_connection_from_pool(&self.pool)
            .map_err(|e| FileRepositoryError::DatabaseError(e.to_string()))?;

        let new_file = NewFileModel::from(file);

        let inserted_file: FileModel = diesel::insert_into(files)
            .values(&new_file)
            .get_result(&mut conn)
            .map_err(|e| FileRepositoryError::DatabaseError(e.to_string()))?;

        Ok(inserted_file.id)
    }

    async fn find_by_id(&self, file_id: Uuid) -> Result<Option<File>, FileRepositoryError> {
        let mut conn = get_connection_from_pool(&self.pool)
            .map_err(|e| FileRepositoryError::DatabaseError(e.to_string()))?;

        let result = files
            .find(file_id)
            .first::<FileModel>(&mut conn)
            .optional()
            .map_err(|e| FileRepositoryError::DatabaseError(e.to_string()))?;

        match result {
            Some(model) => {
                let domain_file =
                    File::try_from(model).map_err(|e| FileRepositoryError::ValidationError(e))?;
                Ok(Some(domain_file))
            }
            None => Ok(None),
        }
    }

    async fn find_by_hash(&self, hash: &str) -> Result<Option<File>, FileRepositoryError> {
        let mut conn = get_connection_from_pool(&self.pool)
            .map_err(|e| FileRepositoryError::DatabaseError(e.to_string()))?;

        let result = files
            .filter(file_hash.eq(hash))
            .first::<FileModel>(&mut conn)
            .optional()
            .map_err(|e| FileRepositoryError::DatabaseError(e.to_string()))?;

        match result {
            Some(model) => {
                let domain_file =
                    File::try_from(model).map_err(|e| FileRepositoryError::ValidationError(e))?;
                Ok(Some(domain_file))
            }
            None => Ok(None),
        }
    }

    async fn find_all(&self, skip: i64, limit: i64) -> Result<Vec<File>, FileRepositoryError> {
        let mut conn = get_connection_from_pool(&self.pool)
            .map_err(|e| FileRepositoryError::DatabaseError(e.to_string()))?;

        let models = files
            .order(created_at.desc())
            .offset(skip)
            .limit(limit)
            .load::<FileModel>(&mut conn)
            .map_err(|e| FileRepositoryError::DatabaseError(e.to_string()))?;

        let mut domain_files = Vec::new();
        for model in models {
            let domain_file =
                File::try_from(model).map_err(|e| FileRepositoryError::ValidationError(e))?;
            domain_files.push(domain_file);
        }

        Ok(domain_files)
    }

    async fn update(&self, file: &File) -> Result<(), FileRepositoryError> {
        let mut conn = get_connection_from_pool(&self.pool)
            .map_err(|e| FileRepositoryError::DatabaseError(e.to_string()))?;

        let update_model = NewFileModel::from(file);

        diesel::update(files.find(file.id()))
            .set(&update_model)
            .execute(&mut conn)
            .map_err(|e| FileRepositoryError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn delete(&self, file_id: Uuid) -> Result<bool, FileRepositoryError> {
        let mut conn = get_connection_from_pool(&self.pool)
            .map_err(|e| FileRepositoryError::DatabaseError(e.to_string()))?;

        let deleted_count = diesel::delete(files.find(file_id))
            .execute(&mut conn)
            .map_err(|e| FileRepositoryError::DatabaseError(e.to_string()))?;

        Ok(deleted_count > 0)
    }

    async fn count(&self) -> Result<i64, FileRepositoryError> {
        let mut conn = get_connection_from_pool(&self.pool)
            .map_err(|e| FileRepositoryError::DatabaseError(e.to_string()))?;

        files
            .count()
            .get_result(&mut conn)
            .map_err(|e| FileRepositoryError::DatabaseError(e.to_string()))
    }
}
