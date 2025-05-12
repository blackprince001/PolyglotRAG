use diesel::{ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl};
use pgvector::{Vector, VectorExpressionMethods};
use uuid::Uuid;

use crate::db::models::{Embedding, NewEmbedding};
use crate::db::schema::embeddings::dsl::*;

impl Embedding {
    pub fn create(
        conn: &mut PgConnection,
        new_embedding: NewEmbedding,
    ) -> Result<Embedding, diesel::result::Error> {
        diesel::insert_into(embeddings)
            .values(&new_embedding)
            .get_result(conn)
    }

    pub fn find_by_chunk(
        conn: &mut PgConnection,
        chunk_id: Uuid,
    ) -> Result<Vec<Embedding>, diesel::result::Error> {
        embeddings.filter(content_chunk_id.eq(chunk_id)).load(conn)
    }

    pub fn find_similar(
        conn: &mut PgConnection,
        query_embedding: Vector,
        limit: i64,
    ) -> Result<Vec<Embedding>, diesel::result::Error> {
        embeddings
            .order(embedding.l2_distance(query_embedding))
            .limit(limit)
            .load::<Embedding>(conn)
    }

    pub fn delete(
        conn: &mut PgConnection,
        embedding_id: Uuid,
    ) -> Result<usize, diesel::result::Error> {
        diesel::delete(embeddings.find(embedding_id)).execute(conn)
    }
}
