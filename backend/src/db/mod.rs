pub mod models;
pub mod users;
pub mod goals;
pub mod achievements;
pub mod cycles;
pub mod audit;
pub mod backup;

use sqlx::{MySqlPool, mysql::MySqlPoolOptions};
use std::time::Duration;

const MAX_CONNECTIONS: u32 = 50;

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
