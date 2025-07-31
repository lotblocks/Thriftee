use sqlx::{PgPool, Pool, Postgres};
use std::time::Duration;
use crate::error::AppError;

#[derive(Clone)]
pub struct Database {
    pool: PgPool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self, AppError> {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(20)
            .acquire_timeout(Duration::from_secs(30))
            .connect(database_url)
            .await?;

        Ok(Database { pool })
    }

    pub async fn migrate(&self) -> Result<(), AppError> {
        sqlx::migrate!("./migrations")
            .run(&self.pool)
            .await?;
        Ok(())
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}