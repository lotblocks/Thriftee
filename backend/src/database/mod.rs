use sqlx::{PgPool, Postgres, Transaction};
use std::time::Duration;
use crate::error::AppError;

pub mod connection;
pub mod migrations;
pub mod transaction_manager;

pub use connection::DatabaseConnection;
pub use transaction_manager::TransactionManager;

/// Database configuration
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout: Duration,
    pub idle_timeout: Duration,
    pub max_lifetime: Duration,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost:5432/raffle_platform".to_string()),
            max_connections: 20,
            min_connections: 5,
            connect_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(600), // 10 minutes
            max_lifetime: Duration::from_secs(1800), // 30 minutes
        }
    }
}

/// Database instance with connection pooling
pub struct Database {
    pool: PgPool,
    config: DatabaseConfig,
}

impl Database {
    /// Create a new database instance with connection pooling
    pub async fn new(config: DatabaseConfig) -> Result<Self, AppError> {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(config.max_connections)
            .min_connections(config.min_connections)
            .acquire_timeout(config.connect_timeout)
            .idle_timeout(config.idle_timeout)
            .max_lifetime(config.max_lifetime)
            .connect(&config.url)
            .await
            .map_err(|e| AppError::Database(format!("Failed to connect to database: {}", e)))?;

        Ok(Self { pool, config })
    }

    /// Get the connection pool
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Get database configuration
    pub fn config(&self) -> &DatabaseConfig {
        &self.config
    }

    /// Run database migrations
    pub async fn migrate(&self) -> Result<(), AppError> {
        sqlx::migrate!("./migrations")
            .run(&self.pool)
            .await
            .map_err(|e| AppError::Database(format!("Migration failed: {}", e)))?;

        Ok(())
    }

    /// Check database health
    pub async fn health_check(&self) -> Result<DatabaseHealth, AppError> {
        let start = std::time::Instant::now();
        
        // Simple query to check connectivity
        let result = sqlx::query_scalar!("SELECT 1")
            .fetch_one(&self.pool)
            .await;

        let response_time = start.elapsed();

        match result {
            Ok(_) => {
                let pool_status = self.pool.size();
                Ok(DatabaseHealth {
                    is_healthy: true,
                    response_time,
                    active_connections: pool_status,
                    idle_connections: self.config.max_connections - pool_status,
                    error: None,
                })
            }
            Err(e) => Ok(DatabaseHealth {
                is_healthy: false,
                response_time,
                active_connections: 0,
                idle_connections: 0,
                error: Some(e.to_string()),
            }),
        }
    }

    /// Begin a new transaction
    pub async fn begin_transaction(&self) -> Result<Transaction<'_, Postgres>, AppError> {
        self.pool
            .begin()
            .await
            .map_err(|e| AppError::Database(format!("Failed to begin transaction: {}", e)))
    }

    /// Execute a function within a transaction
    pub async fn with_transaction<F, R>(&self, f: F) -> Result<R, AppError>
    where
        F: for<'c> FnOnce(&mut Transaction<'c, Postgres>) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<R, AppError>> + Send + 'c>>,
    {
        let mut tx = self.begin_transaction().await?;
        
        match f(&mut tx).await {
            Ok(result) => {
                tx.commit()
                    .await
                    .map_err(|e| AppError::Database(format!("Failed to commit transaction: {}", e)))?;
                Ok(result)
            }
            Err(e) => {
                tx.rollback()
                    .await
                    .map_err(|rollback_err| AppError::Database(format!("Failed to rollback transaction: {}", rollback_err)))?;
                Err(e)
            }
        }
    }

    /// Close the database connection pool
    pub async fn close(&self) {
        self.pool.close().await;
    }

    /// Get connection pool statistics
    pub fn get_pool_stats(&self) -> PoolStatistics {
        PoolStatistics {
            size: self.pool.size(),
            idle: self.pool.num_idle(),
            max_connections: self.config.max_connections,
            min_connections: self.config.min_connections,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DatabaseHealth {
    pub is_healthy: bool,
    pub response_time: Duration,
    pub active_connections: u32,
    pub idle_connections: u32,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PoolStatistics {
    pub size: u32,
    pub idle: usize,
    pub max_connections: u32,
    pub min_connections: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_database_connection() {
        let config = DatabaseConfig {
            url: std::env::var("TEST_DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost:5432/raffle_platform_test".to_string()),
            ..Default::default()
        };

        let db = Database::new(config).await.expect("Failed to create database");
        
        // Test health check
        let health = db.health_check().await.expect("Health check failed");
        assert!(health.is_healthy);

        // Test pool statistics
        let stats = db.get_pool_stats();
        assert!(stats.size <= stats.max_connections);
    }

    #[tokio::test]
    async fn test_transaction() {
        let config = DatabaseConfig {
            url: std::env::var("TEST_DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost:5432/raffle_platform_test".to_string()),
            ..Default::default()
        };

        let db = Database::new(config).await.expect("Failed to create database");
        
        // Test transaction
        let result = db.with_transaction(|tx| {
            Box::pin(async move {
                // Simple test query
                sqlx::query!("SELECT 1")
                    .execute(&mut **tx)
                    .await
                    .map_err(|e| AppError::Database(e.to_string()))?;
                
                Ok(42)
            })
        }).await;

        assert_eq!(result.unwrap(), 42);
    }
}