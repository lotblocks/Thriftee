use sqlx::{PgPool, Row};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use crate::error::AppError;

/// Database connection manager with health monitoring
#[derive(Clone)]
pub struct DatabaseConnection {
    pool: Arc<PgPool>,
    health_status: Arc<RwLock<ConnectionHealth>>,
    config: ConnectionConfig,
}

#[derive(Debug, Clone)]
pub struct ConnectionConfig {
    pub health_check_interval: Duration,
    pub max_retries: u32,
    pub retry_delay: Duration,
    pub connection_timeout: Duration,
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        Self {
            health_check_interval: Duration::from_secs(30),
            max_retries: 3,
            retry_delay: Duration::from_secs(1),
            connection_timeout: Duration::from_secs(10),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConnectionHealth {
    pub is_healthy: bool,
    pub last_check: Instant,
    pub consecutive_failures: u32,
    pub total_connections: u32,
    pub active_connections: u32,
    pub idle_connections: u32,
    pub last_error: Option<String>,
}

impl Default for ConnectionHealth {
    fn default() -> Self {
        Self {
            is_healthy: true,
            last_check: Instant::now(),
            consecutive_failures: 0,
            total_connections: 0,
            active_connections: 0,
            idle_connections: 0,
            last_error: None,
        }
    }
}

impl DatabaseConnection {
    /// Create a new database connection manager
    pub async fn new(pool: PgPool, config: ConnectionConfig) -> Result<Self, AppError> {
        let connection = Self {
            pool: Arc::new(pool),
            health_status: Arc::new(RwLock::new(ConnectionHealth::default())),
            config,
        };

        // Perform initial health check
        connection.check_health().await?;

        // Start background health monitoring
        connection.start_health_monitoring();

        Ok(connection)
    }

    /// Get the connection pool
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Get current health status
    pub async fn get_health(&self) -> ConnectionHealth {
        self.health_status.read().await.clone()
    }

    /// Check database health
    pub async fn check_health(&self) -> Result<(), AppError> {
        let start = Instant::now();
        
        match self.perform_health_check().await {
            Ok(stats) => {
                let mut health = self.health_status.write().await;
                health.is_healthy = true;
                health.last_check = start;
                health.consecutive_failures = 0;
                health.total_connections = stats.total_connections;
                health.active_connections = stats.active_connections;
                health.idle_connections = stats.idle_connections;
                health.last_error = None;

                debug!("Database health check passed: {:?}", stats);
                Ok(())
            }
            Err(e) => {
                let mut health = self.health_status.write().await;
                health.is_healthy = false;
                health.last_check = start;
                health.consecutive_failures += 1;
                health.last_error = Some(e.to_string());

                error!("Database health check failed: {}", e);
                Err(e)
            }
        }
    }

    /// Perform the actual health check
    async fn perform_health_check(&self) -> Result<ConnectionStats, AppError> {
        // Test basic connectivity
        let _result = sqlx::query("SELECT 1")
            .fetch_one(&*self.pool)
            .await
            .map_err(|e| AppError::Database(format!("Health check query failed: {}", e)))?;

        // Get connection statistics
        let stats_query = sqlx::query(
            r#"
            SELECT 
                count(*) as total_connections,
                count(*) filter (where state = 'active') as active_connections,
                count(*) filter (where state = 'idle') as idle_connections
            FROM pg_stat_activity 
            WHERE datname = current_database()
            "#
        )
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to get connection stats: {}", e)))?;

        Ok(ConnectionStats {
            total_connections: stats_query.get::<i64, _>("total_connections") as u32,
            active_connections: stats_query.get::<i64, _>("active_connections") as u32,
            idle_connections: stats_query.get::<i64, _>("idle_connections") as u32,
        })
    }

    /// Start background health monitoring
    fn start_health_monitoring(&self) {
        let connection = self.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(connection.config.health_check_interval);
            
            loop {
                interval.tick().await;
                
                if let Err(e) = connection.check_health().await {
                    warn!("Background health check failed: {}", e);
                    
                    // If we have too many consecutive failures, log an error
                    let health = connection.health_status.read().await;
                    if health.consecutive_failures >= connection.config.max_retries {
                        error!(
                            "Database connection has failed {} consecutive health checks",
                            health.consecutive_failures
                        );
                    }
                }
            }
        });
    }

    /// Execute a query with retry logic
    pub async fn execute_with_retry<F, R>(&self, operation: F) -> Result<R, AppError>
    where
        F: Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<R, sqlx::Error>> + Send>>,
    {
        let mut attempts = 0;
        
        loop {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    attempts += 1;
                    
                    if attempts >= self.config.max_retries {
                        return Err(AppError::Database(format!(
                            "Operation failed after {} attempts: {}",
                            attempts, e
                        )));
                    }
                    
                    warn!("Database operation failed (attempt {}): {}", attempts, e);
                    tokio::time::sleep(self.config.retry_delay).await;
                }
            }
        }
    }

    /// Get detailed connection information
    pub async fn get_connection_info(&self) -> Result<Vec<ConnectionInfo>, AppError> {
        let rows = sqlx::query(
            r#"
            SELECT 
                pid,
                usename,
                application_name,
                client_addr,
                state,
                query_start,
                state_change,
                query
            FROM pg_stat_activity 
            WHERE datname = current_database()
            ORDER BY query_start DESC
            "#
        )
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to get connection info: {}", e)))?;

        let mut connections = Vec::new();
        
        for row in rows {
            connections.push(ConnectionInfo {
                pid: row.get("pid"),
                username: row.get("usename"),
                application_name: row.get("application_name"),
                client_addr: row.get("client_addr"),
                state: row.get("state"),
                query_start: row.get("query_start"),
                state_change: row.get("state_change"),
                current_query: row.get("query"),
            });
        }

        Ok(connections)
    }

    /// Kill a specific connection
    pub async fn kill_connection(&self, pid: i32) -> Result<(), AppError> {
        sqlx::query("SELECT pg_terminate_backend($1)")
            .bind(pid)
            .execute(&*self.pool)
            .await
            .map_err(|e| AppError::Database(format!("Failed to kill connection {}: {}", pid, e)))?;

        info!("Terminated database connection with PID: {}", pid);
        Ok(())
    }

    /// Get database size information
    pub async fn get_database_size(&self) -> Result<DatabaseSize, AppError> {
        let row = sqlx::query(
            r#"
            SELECT 
                pg_database_size(current_database()) as database_size,
                (SELECT count(*) FROM information_schema.tables WHERE table_schema = 'public') as table_count,
                (SELECT count(*) FROM information_schema.columns WHERE table_schema = 'public') as column_count
            "#
        )
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to get database size: {}", e)))?;

        Ok(DatabaseSize {
            size_bytes: row.get::<i64, _>("database_size") as u64,
            table_count: row.get::<i64, _>("table_count") as u32,
            column_count: row.get::<i64, _>("column_count") as u32,
        })
    }
}

#[derive(Debug, Clone)]
struct ConnectionStats {
    pub total_connections: u32,
    pub active_connections: u32,
    pub idle_connections: u32,
}

#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    pub pid: i32,
    pub username: Option<String>,
    pub application_name: Option<String>,
    pub client_addr: Option<std::net::IpAddr>,
    pub state: Option<String>,
    pub query_start: Option<chrono::DateTime<chrono::Utc>>,
    pub state_change: Option<chrono::DateTime<chrono::Utc>>,
    pub current_query: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DatabaseSize {
    pub size_bytes: u64,
    pub table_count: u32,
    pub column_count: u32,
}

impl DatabaseSize {
    pub fn size_mb(&self) -> f64 {
        self.size_bytes as f64 / (1024.0 * 1024.0)
    }

    pub fn size_gb(&self) -> f64 {
        self.size_bytes as f64 / (1024.0 * 1024.0 * 1024.0)
    }
}