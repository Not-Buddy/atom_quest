pub mod models;
pub mod student;
pub mod backup;
pub mod import;
pub mod faculty;

use sqlx::{MySqlPool, mysql::MySqlPoolOptions};
use std::time::Duration;

const MAX_CONNECTIONS : u32 =50;
// Type alias for database pool
pub type DbPool = MySqlPool;

#[derive(Clone)]
pub struct Database {
    pub pool: MySqlPool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = MySqlPoolOptions::new()
            .max_connections(MAX_CONNECTIONS)
            .acquire_timeout(Duration::from_secs(30))
            .connect(database_url)
            .await?;

        Ok(Database { pool })
    }

}

// Re-export commonly used functions
#[allow(unused_imports)]
pub use student::{find_student_by_email, find_student_by_id, update_student_links, get_student_by_ra, get_all_students};
