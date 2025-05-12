use diesel::{ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl};
use uuid::Uuid;

use crate::db::schema::content_chunks::dsl::*;

use crate::db::models::{ContentChunk, NewContentChunk};

impl ContentChunk {
    pub fn create(
        conn: &mut PgConnection,
        new_chunk: NewContentChunk,
    ) -> Result<ContentChunk, diesel::result::Error> {
        diesel::insert_into(content_chunks)
            .values(&new_chunk)
            .get_result(conn)
    }

    pub fn find_by_file_id(
        conn: &mut PgConnection,
        fr_file_id: Uuid,
    ) -> Result<Vec<ContentChunk>, diesel::result::Error> {
        content_chunks.filter(file_id.eq(fr_file_id)).load(conn)
    }

    pub fn find_by_id(
        conn: &mut PgConnection,
        chunk_id: Uuid,
    ) -> Result<ContentChunk, diesel::result::Error> {
        content_chunks.find(chunk_id).first(conn)
    }

    pub fn delete(conn: &mut PgConnection, chunk_id: Uuid) -> Result<usize, diesel::result::Error> {
        diesel::delete(content_chunks.find(chunk_id)).execute(conn)
    }
}
