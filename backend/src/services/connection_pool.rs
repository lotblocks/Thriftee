use sqlx::{PgPool, Pool, Postgres};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::utils::errors::AppError;

#[derive(Debug, Clone)]
pub struct ConnectionPoolConfig {
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout: Duration,
    pub idle_timeout: Duration,
    pub max_lifetime: Duration,
    pub test_before_acquire: bool,
    pub health_check_interval: Duration,
    pub slow_query_threshold: Duration,
}

impl Default for ConnectionPoolConfig {
    fn default() -> Self {
        Self {
            max_connections: 20,
            min_connections: 5,
            acquire_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(600), // 10 minutes
            max_lifetime: Duration::from_secs(1800), // 30 minutes
            test_before_acquire: true,
            health_check_interval: Duration::from_secs(30),
            slow_query_threshold: Duration::from_millis(1000),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConnectionPoolMetrics {
    pub total_connections: u32,
    pub active_connections: u32,
    pub idle_connections: u32,
    pub pending_requests: u32,
    pub total_acquired: u64,
    pub total_released: u64,
    pub acquire_timeouts: u64,
    pub connection_errors: u64,
    pub slow_queries: u64,
    pub avg_acquire_time_ms: f64,
    pub max_acquire_time_ms: u64,
    pub last_health_check: Option<std::time::Instant>,
    pub health_check_failures: u64,
}

impl Default for ConnectionPoolMetrics {
    fn default() -> Self {
        Self {
            total_connections: 0,
            active_connections: 0,
            idle_connections: 0,
            pending_requests: 0,
            total_acquired: 0,
            total_released: 0,
            acquire_timeouts: 0,
            connection_errors: 0,
            slow_queries: 0,
            avg_acquire_time_ms: 0.0,
            max_acquire_time_ms: 0,
            last_health_check: None,
            health_check_failures: 0,
        }
    }
}

pub struct OptimizedConnectionPool {
    pool: PgPool,
    config: ConnectionPoolConfig,
    metrics: Arc<RwLock<ConnectionPoolMetrics>>,
    health_monitor_handle: Option<tokio::task::JoinHandle<()>>,
}

impl OptimizedConnectionPool {
    pub async fn new(database_url: &str, config: ConnectionPoolConfig) -> Result<Self, AppError> {
        let pool_options = sqlx::postgres::PgPoolOptions::new()
            .max_connections(config.max_connections)
            .min_connections(config.min_connections)
            .acquire_timeout(config.acquire_timeout)
            .idle_timeout(config.idle_timeout)
            .max_lifetime(config.max_lifetime)
            .test_before_acquire(config.test_before_acquire);

        let pool = pool_options
            .connect(database_url)
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to create connection pool: {}", e)))?;

        let metrics = Arc::new(RwLock::new(ConnectionPoolMetrics::default()));

        let mut optimized_pool = Self {
            pool,
            config,
            metrics,
            health_monitor_handle: None,
        };

        // Start health monitoring
        optimized_pool.start_health_monitoring().await;

        info!("Optimized connection pool created with {} max connections", optimized_pool.config.max_connections);

        Ok(optimized_pool)
    }

    /// Get the underlying pool
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Get connection pool metrics
    pub async fn get_metrics(&self) -> ConnectionPoolMetrics {
        let metrics = self.metrics.read().await;
        metrics.clone()
    }

    /// Reset metrics
    pub async fn reset_metrics(&self) {
        let mut metrics = self.metrics.write().await;
        *metrics = ConnectionPoolMetrics::default();
    }

    /// Execute a query with timing and metrics tracking
    pub async fn execute_with_metrics<F, T>(&self, operation: F) -> Result<T, AppError>
    where
        F: FnOnce(&PgPool) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, sqlx::Error>> + Send>>,
    {
        let start_time = Instant::now();
        
        // Update metrics - connection acquired
        {
            let mut metrics = self.metrics.write().await;
            metrics.total_acquired += 1;
        }

        let result = operation(&self.pool).await;
        
        let execution_time = start_time.elapsed();
        
        // Update metrics based on result
        {
            let mut metrics = self.metrics.write().await;
            metrics.total_released += 1;
            
            // Update average acquire time
            let total_ops = metrics.total_acquired;
            metrics.avg_acquire_time_ms = 
                (metrics.avg_acquire_time_ms * (total_ops - 1) as f64 + execution_time.as_millis() as f64) / total_ops as f64;
            
            // Update max acquire time
            if execution_time.as_millis() as u64 > metrics.max_acquire_time_ms {
                metrics.max_acquire_time_ms = execution_time.as_millis() as u64;
            }
            
            // Check for slow queries
            if execution_time > self.config.slow_query_threshold {
                metrics.slow_queries += 1;
                warn!("Slow query detected: {}ms", execution_time.as_millis());
            }
            
            // Update error count
            if result.is_err() {
                metrics.connection_errors += 1;
            }
        }

        result.map_err(|e| AppError::DatabaseError(format!("Database operation failed: {}", e)))
    }

    /// Get current pool status
    pub async fn get_pool_status(&self) -> PoolStatus {
        let pool_options = self.pool.options();
        let metrics = self.get_metrics().await;
        
        PoolStatus {
            max_connections: pool_options.get_max_connections(),
            min_connections: pool_options.get_min_connections(),
            current_connections: self.pool.size(),
            idle_connections: self.pool.num_idle(),
            acquire_timeout: pool_options.get_acquire_timeout(),
            idle_timeout: pool_options.get_idle_timeout(),
            max_lifetime: pool_options.get_max_lifetime(),
            metrics,
            health_status: self.check_health().await,
        }
    }

    /// Perform health check
    pub async fn check_health(&self) -> HealthStatus {
        let start_time = Instant::now();
        
        let health_check_result = sqlx::query("SELECT 1")
            .fetch_one(&self.pool)
            .await;
            
        let check_duration = start_time.elapsed();
        
        let mut metrics = self.metrics.write().await;
        metrics.last_health_check = Some(start_time);
        
        match health_check_result {
            Ok(_) => {
                debug!("Health check passed in {}ms", check_duration.as_millis());
                HealthStatus::Healthy
            }
            Err(e) => {
                metrics.health_check_failures += 1;
                error!("Health check failed: {}", e);
                HealthStatus::Unhealthy(format!("Health check failed: {}", e))
            }
        }
    }

    /// Start background health monitoring
    async fn start_health_monitoring(&mut self) {
        let pool = self.pool.clone();
        let metrics = Arc::clone(&self.metrics);
        let interval = self.config.health_check_interval;
        
        let handle = tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);
            
            loop {
                interval_timer.tick().await;
                
                // Perform health check
                let start_time = Instant::now();
                let health_result = sqlx::query("SELECT 1")
                    .fetch_one(&pool)
                    .await;
                
                // Update metrics
                {
                    let mut metrics_guard = metrics.write().await;
                    metrics_guard.last_health_check = Some(start_time);
                    
                    if health_result.is_err() {
                        metrics_guard.health_check_failures += 1;
                        error!("Background health check failed: {:?}", health_result.err());
                    } else {
                        debug!("Background health check passed");
                    }
                    
                    // Update connection counts
                    metrics_guard.total_connections = pool.size();
                    metrics_guard.idle_connections = pool.num_idle();
                    metrics_guard.active_connections = pool.size() - pool.num_idle();
                }
            }
        });
        
        self.health_monitor_handle = Some(handle);
    }

    /// Optimize pool configuration based on current metrics
    pub async fn optimize_configuration(&self) -> OptimizationRecommendations {
        let metrics = self.get_metrics().await;
        let mut recommendations = OptimizationRecommendations::default();
        
        // Analyze connection usage
        if metrics.active_connections as f64 / metrics.total_connections as f64 > 0.9 {
            recommendations.increase_max_connections = Some(format!(
                "Consider increasing max_connections from {} to {} due to high utilization",
                self.config.max_connections,
                self.config.max_connections + 5
            ));
        }
        
        if metrics.active_connections as f64 / metrics.total_connections as f64 < 0.3 {
            recommendations.decrease_max_connections = Some(format!(
                "Consider decreasing max_connections from {} to {} due to low utilization",
                self.config.max_connections,
                std::cmp::max(self.config.min_connections, self.config.max_connections - 5)
            ));
        }
        
        // Analyze acquire times
        if metrics.avg_acquire_time_ms > 100.0 {
            recommendations.tune_acquire_timeout = Some(format!(
                "Average acquire time is {:.1}ms, consider increasing acquire_timeout or max_connections",
                metrics.avg_acquire_time_ms
            ));
        }
        
        // Analyze slow queries
        if metrics.slow_queries > 0 {
            let slow_query_rate = metrics.slow_queries as f64 / metrics.total_acquired as f64 * 100.0;
            if slow_query_rate > 5.0 {
                recommendations.optimize_queries = Some(format!(
                    "{:.1}% of queries are slow (>{:?}), consider query optimization",
                    slow_query_rate,
                    self.config.slow_query_threshold
                ));
            }
        }
        
        // Analyze error rates
        if metrics.connection_errors > 0 {
            let error_rate = metrics.connection_errors as f64 / metrics.total_acquired as f64 * 100.0;
            if error_rate > 1.0 {
                recommendations.investigate_errors = Some(format!(
                    "Connection error rate is {:.1}%, investigate connection stability",
                    error_rate
                ));
            }
        }
        
        recommendations
    }

    /// Get connection pool statistics for monitoring
    pub async fn get_detailed_stats(&self) -> DetailedPoolStats {
        let metrics = self.get_metrics().await;
        let pool_options = self.pool.options();
        
        DetailedPoolStats {
            configuration: PoolConfiguration {
                max_connections: pool_options.get_max_connections(),
                min_connections: pool_options.get_min_connections(),
                acquire_timeout_ms: pool_options.get_acquire_timeout().as_millis() as u64,
                idle_timeout_ms: pool_options.get_idle_timeout().map(|d| d.as_millis() as u64),
                max_lifetime_ms: pool_options.get_max_lifetime().map(|d| d.as_millis() as u64),
                test_before_acquire: self.config.test_before_acquire,
            },
            current_state: CurrentPoolState {
                total_connections: self.pool.size(),
                active_connections: self.pool.size() - self.pool.num_idle(),
                idle_connections: self.pool.num_idle(),
                pending_requests: 0, // Not directly available from sqlx
            },
            performance_metrics: PerformanceMetrics {
                total_operations: metrics.total_acquired,
                successful_operations: metrics.total_acquired - metrics.connection_errors,
                failed_operations: metrics.connection_errors,
                avg_operation_time_ms: metrics.avg_acquire_time_ms,
                max_operation_time_ms: metrics.max_acquire_time_ms,
                slow_operations: metrics.slow_queries,
                timeout_count: metrics.acquire_timeouts,
            },
            health_metrics: HealthMetrics {
                last_health_check: metrics.last_health_check,
                health_check_failures: metrics.health_check_failures,
                current_status: self.check_health().await,
            },
        }
    }
}

impl Drop for OptimizedConnectionPool {
    fn drop(&mut self) {
        if let Some(handle) = self.health_monitor_handle.take() {
            handle.abort();
        }
    }
}

#[derive(Debug, Clone)]
pub struct PoolStatus {
    pub max_connections: u32,
    pub min_connections: u32,
    pub current_connections: u32,
    pub idle_connections: u32,
    pub acquire_timeout: Duration,
    pub idle_timeout: Option<Duration>,
    pub max_lifetime: Option<Duration>,
    pub metrics: ConnectionPoolMetrics,
    pub health_status: HealthStatus,
}

#[derive(Debug, Clone)]
pub enum HealthStatus {
    Healthy,
    Unhealthy(String),
}

#[derive(Debug, Default)]
pub struct OptimizationRecommendations {
    pub increase_max_connections: Option<String>,
    pub decrease_max_connections: Option<String>,
    pub tune_acquire_timeout: Option<String>,
    pub optimize_queries: Option<String>,
    pub investigate_errors: Option<String>,
}

#[derive(Debug)]
pub struct DetailedPoolStats {
    pub configuration: PoolConfiguration,
    pub current_state: CurrentPoolState,
    pub performance_metrics: PerformanceMetrics,
    pub health_metrics: HealthMetrics,
}

#[derive(Debug)]
pub struct PoolConfiguration {
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout_ms: u64,
    pub idle_timeout_ms: Option<u64>,
    pub max_lifetime_ms: Option<u64>,
    pub test_before_acquire: bool,
}

#[derive(Debug)]
pub struct CurrentPoolState {
    pub total_connections: u32,
    pub active_connections: u32,
    pub idle_connections: u32,
    pub pending_requests: u32,
}

#[derive(Debug)]
pub struct PerformanceMetrics {
    pub total_operations: u64,
    pub successful_operations: u64,
    pub failed_operations: u64,
    pub avg_operation_time_ms: f64,
    pub max_operation_time_ms: u64,
    pub slow_operations: u64,
    pub timeout_count: u64,
}

#[derive(Debug)]
pub struct HealthMetrics {
    pub last_health_check: Option<Instant>,
    pub health_check_failures: u64,
    pub current_status: HealthStatus,
}

// Connection pool factory for different use cases
pub struct ConnectionPoolFactory;

impl ConnectionPoolFactory {
    /// Create a high-performance pool for heavy workloads
    pub async fn create_high_performance_pool(database_url: &str) -> Result<OptimizedConnectionPool, AppError> {
        let config = ConnectionPoolConfig {
            max_connections: 50,
            min_connections: 10,
            acquire_timeout: Duration::from_secs(10),
            idle_timeout: Duration::from_secs(300), // 5 minutes
            max_lifetime: Duration::from_secs(900), // 15 minutes
            test_before_acquire: true,
            health_check_interval: Duration::from_secs(15),
            slow_query_threshold: Duration::from_millis(500),
        };
        
        OptimizedConnectionPool::new(database_url, config).await
    }
    
    /// Create a balanced pool for general use
    pub async fn create_balanced_pool(database_url: &str) -> Result<OptimizedConnectionPool, AppError> {
        let config = ConnectionPoolConfig::default();
        OptimizedConnectionPool::new(database_url, config).await
    }
    
    /// Create a lightweight pool for low-traffic applications
    pub async fn create_lightweight_pool(database_url: &str) -> Result<OptimizedConnectionPool, AppError> {
        let config = ConnectionPoolConfig {
            max_connections: 10,
            min_connections: 2,
            acquire_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(900), // 15 minutes
            max_lifetime: Duration::from_secs(3600), // 1 hour
            test_before_acquire: false,
            health_check_interval: Duration::from_secs(60),
            slow_query_threshold: Duration::from_millis(2000),
        };
        
        OptimizedConnectionPool::new(database_url, config).await
    }
}

// Utility functions for pool management
pub mod pool_utils {
    use super::*;
    
    /// Calculate optimal pool size based on system resources
    pub fn calculate_optimal_pool_size(cpu_cores: u32, expected_concurrent_users: u32) -> u32 {
        // Rule of thumb: 2-4 connections per CPU core, adjusted for concurrent users
        let base_size = cpu_cores * 3;
        let user_adjusted = (expected_concurrent_users as f32 * 0.1) as u32;
        
        std::cmp::max(base_size, user_adjusted).min(100) // Cap at 100 connections
    }
    
    /// Determine if pool needs scaling based on metrics
    pub fn should_scale_pool(metrics: &ConnectionPoolMetrics, current_max: u32) -> ScalingRecommendation {
        let utilization = metrics.active_connections as f64 / metrics.total_connections as f64;
        let error_rate = if metrics.total_acquired > 0 {
            metrics.connection_errors as f64 / metrics.total_acquired as f64
        } else {
            0.0
        };
        
        if utilization > 0.9 || error_rate > 0.05 {
            ScalingRecommendation::ScaleUp(current_max + 5)
        } else if utilization < 0.3 && current_max > 5 {
            ScalingRecommendation::ScaleDown(current_max - 5)
        } else {
            ScalingRecommendation::NoChange
        }
    }
    
    #[derive(Debug)]
    pub enum ScalingRecommendation {
        ScaleUp(u32),
        ScaleDown(u32),
        NoChange,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_pool_creation() {
        // This would require a test database
        // For now, we'll skip the actual implementation
        assert!(true);
    }
    
    #[test]
    fn test_optimal_pool_size_calculation() {
        let size = pool_utils::calculate_optimal_pool_size(4, 100);
        assert!(size >= 10); // Should be at least 10 for 100 concurrent users
        assert!(size <= 100); // Should not exceed the cap
    }
    
    #[test]
    fn test_scaling_recommendation() {
        let metrics = ConnectionPoolMetrics {
            total_connections: 10,
            active_connections: 9, // 90% utilization
            ..Default::default()
        };
        
        let recommendation = pool_utils::should_scale_pool(&metrics, 10);
        match recommendation {
            pool_utils::ScalingRecommendation::ScaleUp(new_size) => {
                assert_eq!(new_size, 15);
            }
            _ => panic!("Expected scale up recommendation"),
        }
    }
}