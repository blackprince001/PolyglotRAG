use chrono::{DateTime, Utc};
use diesel::prelude::*;
use pgvector::Vector;
use serde::Serialize;
use uuid::Uuid;

use crate::domain::entities::Embedding as DomainEmbedding;
use crate::infrastructure::database::schema::embeddings;

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Identifiable, Associations)]
#[diesel(belongs_to(super::ContentChunkModel, foreign_key = content_chunk_id))]
#[diesel(table_name = embeddings)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct EmbeddingModel {
    pub id: Uuid,
    pub content_chunk_id: Option<Uuid>,
    pub model_name: String,
    pub model_version: Option<String>,
    pub generated_at: Option<DateTime<Utc>>,
    pub generation_parameters: Option<serde_json::Value>,
    pub embedding: Option<Vector>,
}

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = embeddings)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewEmbeddingModel {
    pub id: Option<Uuid>,
    pub content_chunk_id: Uuid,
    pub model_name: String,
    pub model_version: Option<String>,
    pub generated_at: Option<DateTime<Utc>>,
    pub generation_parameters: Option<serde_json::Value>,
    pub embedding: Option<Vector>,
}

impl From<&DomainEmbedding> for NewEmbeddingModel {
    fn from(domain_embedding: &DomainEmbedding) -> Self {
        Self {
            id: Some(domain_embedding.id()),
            content_chunk_id: domain_embedding.content_chunk_id(),
            model_name: domain_embedding.model_name().to_string(),
            model_version: domain_embedding.model_version().map(|s| s.to_string()),
            generated_at: Some(domain_embedding.generated_at()),
            generation_parameters: domain_embedding.generation_parameters().cloned(),
            embedding: Some(domain_embedding.embedding().clone()),
        }
    }
}

impl TryFrom<EmbeddingModel> for DomainEmbedding {
    type Error = String;

    fn try_from(model: EmbeddingModel) -> Result<Self, Self::Error> {
        let embedding_vector = model.embedding.ok_or("Embedding vector is required")?;

        Ok(DomainEmbedding::new(
            model
                .content_chunk_id
                .ok_or("Content chunk ID is required")?,
            model.model_name,
            model.model_version,
            model.generation_parameters,
            embedding_vector,
        ))
    }
}
