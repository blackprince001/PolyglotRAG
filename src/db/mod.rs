pub mod crud;
pub mod models;
pub mod schema;

use diesel::{Connection, pg::PgConnection};
use std::env;

pub fn get_database_connection() -> Result<PgConnection, diesel::result::ConnectionError> {
    let db_url = env::var("DATABASE_URL").map_err(|_| {
        diesel::result::ConnectionError::BadConnection("DATABASE_URL not set".into())
    })?;
    PgConnection::establish(&db_url)
}
