use chrono::{DateTime, Utc};
use diesel::prelude::*;
use pgvector::Vector;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::schema;
use schema::{content_chunks, embeddings, files};

#[derive(Debug, Queryable, Selectable, Serialize, Identifiable)]
#[diesel(table_name = files)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct File {
    pub id: Uuid,
    pub file_path: String,
    pub file_name: String,
    pub file_size: Option<i64>,
    pub file_type: Option<String>,
    pub file_hash: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Insertable, AsChangeset, Deserialize)]
#[diesel(table_name = files)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewFile {
    pub file_path: String,
    pub file_name: String,
    pub file_size: Option<i64>,
    pub file_type: Option<String>,
    pub file_hash: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Queryable, Serialize, Identifiable, Associations, Selectable)]
#[diesel(belongs_to(File))]
#[diesel(table_name = content_chunks)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ContentChunk {
    pub id: Uuid,
    pub file_id: Option<Uuid>,
    pub chunk_text: String,
    pub chunk_index: i32,
    pub token_count: Option<i32>,
    pub page_number: Option<i32>,
    pub section_path: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = content_chunks)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewContentChunk {
    pub file_id: Option<Uuid>,
    pub chunk_text: String,
    pub chunk_index: i32,
    pub token_count: Option<i32>,
    pub page_number: Option<i32>,
    pub section_path: Option<String>,
}

#[derive(Debug, Queryable, Serialize, Identifiable, Associations, Selectable)]
#[diesel(belongs_to(ContentChunk))]
#[diesel(table_name = embeddings)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Embedding {
    pub id: Uuid,
    pub content_chunk_id: Option<Uuid>,
    pub embedding: Option<Vector>,
    pub model_name: String,
    pub model_version: Option<String>,
    pub generated_at: Option<DateTime<Utc>>,
    pub generation_parameters: Option<serde_json::Value>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = embeddings)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewEmbedding {
    pub content_chunk_id: Uuid,
    pub embedding: Option<Vector>,
    pub model_name: String,
    pub model_version: Option<String>,
    pub generation_parameters: Option<serde_json::Value>,
}
