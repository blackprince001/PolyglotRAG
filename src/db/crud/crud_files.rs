use crate::db::schema::files::dsl::*;
use diesel::{ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl};

use uuid::Uuid;

use crate::db::models::{File, NewFile};

impl File {
    pub fn create_file(
        conn: &mut PgConnection,
        new_file: NewFile,
    ) -> Result<File, diesel::result::Error> {
        diesel::insert_into(files)
            .values(&new_file)
            .get_result(conn)
    }

    pub fn find_file(
        conn: &mut PgConnection,
        file_id: Uuid,
    ) -> Result<File, diesel::result::Error> {
        files.find(file_id).first(conn)
    }

    pub fn find_files(
        conn: &mut PgConnection,
        skip: i64,
        limit: i64,
    ) -> Result<Vec<File>, diesel::result::Error> {
        files
            .order(created_at.desc())
            .offset(skip)
            .limit(limit)
            .load::<File>(conn)
    }

    pub fn update_file(
        conn: &mut PgConnection,
        file_id: Uuid,
        changes: NewFile,
    ) -> Result<File, diesel::result::Error> {
        diesel::update(files.find(file_id))
            .set(&changes)
            .get_result(conn)
    }

    pub fn delete_file(
        conn: &mut PgConnection,
        file_id: Uuid,
    ) -> Result<usize, diesel::result::Error> {
        diesel::delete(files.find(file_id)).execute(conn)
    }
}
