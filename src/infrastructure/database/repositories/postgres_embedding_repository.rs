use async_trait::async_trait;
use diesel::prelude::*;
use pgvector::Vector;
use uuid::Uuid;

use crate::domain::entities::Embedding;
use crate::domain::repositories::{EmbeddingRepository, embedding_repository::{EmbeddingRepositoryError, SimilaritySearchResult}};
use crate::infrastructure::database::{DbPool, get_connection_from_pool};
use crate::infrastructure::database::models::{EmbeddingModel, NewEmbeddingModel};
use crate::infrastructure::database::schema::embeddings::dsl::*;

pub struct PostgresEmbeddingRepository {
    pool: DbPool,
}

impl PostgresEmbeddingRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl EmbeddingRepository for PostgresEmbeddingRepository {
    async fn save(&self, embedding_entity: &Embedding) -> Result<(), EmbeddingRepositoryError> {
        let mut conn = get_connection_from_pool(&self.pool)
            .map_err(|e| EmbeddingRepositoryError::DatabaseError(e.to_string()))?;

        let new_embedding = NewEmbeddingModel::from(embedding_entity);

        diesel::insert_into(embeddings)
            .values(&new_embedding)
            .execute(&mut conn)
            .map_err(|e| EmbeddingRepositoryError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn save_batch(&self, embedding_entities: &[Embedding]) -> Result<(), EmbeddingRepositoryError> {
        let mut conn = get_connection_from_pool(&self.pool)
            .map_err(|e| EmbeddingRepositoryError::DatabaseError(e.to_string()))?;

        let new_embeddings: Vec<NewEmbeddingModel> = embedding_entities
            .iter()
            .map(NewEmbeddingModel::from)
            .collect();

        diesel::insert_into(embeddings)
            .values(&new_embeddings)
            .execute(&mut conn)
            .map_err(|e| EmbeddingRepositoryError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn find_by_id(&self, embedding_id: Uuid) -> Result<Option<Embedding>, EmbeddingRepositoryError> {
        let mut conn = get_connection_from_pool(&self.pool)
            .map_err(|e| EmbeddingRepositoryError::DatabaseError(e.to_string()))?;

        let result = embeddings
            .find(embedding_id)
            .first::<EmbeddingModel>(&mut conn)
            .optional()
            .map_err(|e| EmbeddingRepositoryError::DatabaseError(e.to_string()))?;

        match result {
            Some(model) => {
                let domain_embedding = Embedding::try_from(model)
                    .map_err(|e| EmbeddingRepositoryError::ValidationError(e))?;
                Ok(Some(domain_embedding))
            }
            None => Ok(None),
        }
    }

    async fn find_by_chunk_id(&self, chunk_id: Uuid) -> Result<Option<Embedding>, EmbeddingRepositoryError> {
        let mut conn = get_connection_from_pool(&self.pool)
            .map_err(|e| EmbeddingRepositoryError::DatabaseError(e.to_string()))?;

        let result = embeddings
            .filter(content_chunk_id.eq(chunk_id))
            .first::<EmbeddingModel>(&mut conn)
            .optional()
            .map_err(|e| EmbeddingRepositoryError::DatabaseError(e.to_string()))?;

        match result {
            Some(model) => {
                let domain_embedding = Embedding::try_from(model)
                    .map_err(|e| EmbeddingRepositoryError::ValidationError(e))?;
                Ok(Some(domain_embedding))
            }
            None => Ok(None),
        }
    }

    async fn find_by_file_id(&self, _file_id: Uuid) -> Result<Vec<Embedding>, EmbeddingRepositoryError> {
        // This would require joining with content_chunks table
        // For now, return empty vec
        Ok(Vec::new())
    }

    async fn similarity_search(
        &self,
        query_vector: &Vector,
        limit: i32,
        similarity_threshold: Option<f32>,
    ) -> Result<Vec<SimilaritySearchResult>, EmbeddingRepositoryError> {
        let mut conn = get_connection_from_pool(&self.pool)
            .map_err(|e| EmbeddingRepositoryError::DatabaseError(e.to_string()))?;

        // This is a simplified version - in a real implementation, you'd use pgvector's similarity functions
        let models = embeddings
            .filter(embedding.is_not_null())
            .limit(limit.into())
            .load::<EmbeddingModel>(&mut conn)
            .map_err(|e| EmbeddingRepositoryError::DatabaseError(e.to_string()))?;

        let mut results = Vec::new();
        for model in models {
            if let (Some(emb_vector), Some(chunk_id)) = (&model.embedding, model.content_chunk_id) {
                // Calculate cosine similarity (simplified)
                let similarity_score = calculate_cosine_similarity(query_vector, emb_vector);
                
                if let Some(threshold) = similarity_threshold {
                    if similarity_score < threshold {
                        continue;
                    }
                }

                let domain_embedding = Embedding::try_from(model)
                    .map_err(|e| EmbeddingRepositoryError::ValidationError(e))?;

                results.push(SimilaritySearchResult {
                    embedding: domain_embedding,
                    similarity_score,
                    chunk_id,
                });
            }
        }

        // Sort by similarity score (descending)
        results.sort_by(|a, b| b.similarity_score.partial_cmp(&a.similarity_score).unwrap());

        Ok(results)
    }

    async fn similarity_search_by_file(
        &self,
        query_vector: &Vector,
        _file_id: Uuid,
        limit: i32,
        similarity_threshold: Option<f32>,
    ) -> Result<Vec<SimilaritySearchResult>, EmbeddingRepositoryError> {
        // This would require joining with content_chunks table to filter by file_id
        // For now, just call the regular similarity search
        self.similarity_search(query_vector, limit, similarity_threshold).await
    }

    async fn update(&self, embedding_entity: &Embedding) -> Result<(), EmbeddingRepositoryError> {
        let mut conn = get_connection_from_pool(&self.pool)
            .map_err(|e| EmbeddingRepositoryError::DatabaseError(e.to_string()))?;

        let update_model = NewEmbeddingModel::from(embedding_entity);

        diesel::update(embeddings.find(embedding_entity.id()))
            .set(&update_model)
            .execute(&mut conn)
            .map_err(|e| EmbeddingRepositoryError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn delete(&self, embedding_id: Uuid) -> Result<bool, EmbeddingRepositoryError> {
        let mut conn = get_connection_from_pool(&self.pool)
            .map_err(|e| EmbeddingRepositoryError::DatabaseError(e.to_string()))?;

        let deleted_count = diesel::delete(embeddings.find(embedding_id))
            .execute(&mut conn)
            .map_err(|e| EmbeddingRepositoryError::DatabaseError(e.to_string()))?;

        Ok(deleted_count > 0)
    }

    async fn delete_by_chunk_id(&self, chunk_id: Uuid) -> Result<bool, EmbeddingRepositoryError> {
        let mut conn = get_connection_from_pool(&self.pool)
            .map_err(|e| EmbeddingRepositoryError::DatabaseError(e.to_string()))?;

        let deleted_count = diesel::delete(embeddings.filter(content_chunk_id.eq(chunk_id)))
            .execute(&mut conn)
            .map_err(|e| EmbeddingRepositoryError::DatabaseError(e.to_string()))?;

        Ok(deleted_count > 0)
    }

    async fn delete_by_file_id(&self, _file_id: Uuid) -> Result<i64, EmbeddingRepositoryError> {
        // This would require joining with content_chunks table
        // For now, return 0
        Ok(0)
    }

    async fn count(&self) -> Result<i64, EmbeddingRepositoryError> {
        let mut conn = get_connection_from_pool(&self.pool)
            .map_err(|e| EmbeddingRepositoryError::DatabaseError(e.to_string()))?;

        embeddings
            .count()
            .get_result(&mut conn)
            .map_err(|e| EmbeddingRepositoryError::DatabaseError(e.to_string()))
    }

    async fn count_by_model(&self, model_name_param: &str) -> Result<i64, EmbeddingRepositoryError> {
        let mut conn = get_connection_from_pool(&self.pool)
            .map_err(|e| EmbeddingRepositoryError::DatabaseError(e.to_string()))?;

        embeddings
            .filter(model_name.eq(model_name_param))
            .count()
            .get_result(&mut conn)
            .map_err(|e| EmbeddingRepositoryError::DatabaseError(e.to_string()))
    }
}

// Helper function to calculate cosine similarity
fn calculate_cosine_similarity(a: &Vector, b: &Vector) -> f32 {
    let a_slice = a.as_slice();
    let b_slice = b.as_slice();

    if a_slice.len() != b_slice.len() {
        return 0.0;
    }

    let dot_product: f32 = a_slice.iter().zip(b_slice.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a_slice.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b_slice.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    dot_product / (norm_a * norm_b)
}
