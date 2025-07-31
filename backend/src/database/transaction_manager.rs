use sqlx::{PgPool, Postgres, Transaction};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};
use uuid::Uuid;
use crate::error::AppError;

/// Transaction manager for handling database transactions with monitoring
pub struct TransactionManager {
    pool: Arc<PgPool>,
    active_transactions: Arc<Mutex<std::collections::HashMap<Uuid, TransactionInfo>>>,
    config: TransactionConfig,
}

#[derive(Debug, Clone)]
pub struct TransactionConfig {
    pub default_timeout: Duration,
    pub max_concurrent_transactions: usize,
    pub enable_monitoring: bool,
    pub log_slow_transactions: bool,
    pub slow_transaction_threshold: Duration,
}

impl Default for TransactionConfig {
    fn default() -> Self {
        Self {
            default_timeout: Duration::from_secs(30),
            max_concurrent_transactions: 100,
            enable_monitoring: true,
            log_slow_transactions: true,
            slow_transaction_threshold: Duration::from_secs(5),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TransactionInfo {
    pub id: Uuid,
    pub started_at: Instant,
    pub timeout: Duration,
    pub description: Option<String>,
}

impl TransactionManager {
    /// Create a new transaction manager
    pub fn new(pool: Arc<PgPool>, config: TransactionConfig) -> Self {
        let manager = Self {
            pool,
            active_transactions: Arc::new(Mutex::new(std::collections::HashMap::new())),
            config,
        };

        if manager.config.enable_monitoring {
            manager.start_monitoring();
        }

        manager
    }

    /// Begin a new managed transaction
    pub async fn begin_transaction(
        &self,
        description: Option<String>,
        timeout: Option<Duration>,
    ) -> Result<ManagedTransaction, AppError> {
        // Check if we've reached the maximum number of concurrent transactions
        {
            let active = self.active_transactions.lock().await;
            if active.len() >= self.config.max_concurrent_transactions {
                return Err(AppError::Database(
                    "Maximum number of concurrent transactions reached".to_string(),
                ));
            }
        }

        let transaction_id = Uuid::new_v4();
        let timeout = timeout.unwrap_or(self.config.default_timeout);
        
        let tx = self.pool
            .begin()
            .await
            .map_err(|e| AppError::Database(format!("Failed to begin transaction: {}", e)))?;

        let info = TransactionInfo {
            id: transaction_id,
            started_at: Instant::now(),
            timeout,
            description: description.clone(),
        };

        // Register the transaction
        {
            let mut active = self.active_transactions.lock().await;
            active.insert(transaction_id, info);
        }

        debug!(
            "Started transaction {} with timeout {:?}: {:?}",
            transaction_id, timeout, description
        );

        Ok(ManagedTransaction {
            id: transaction_id,
            transaction: Some(tx),
            manager: self.clone(),
            started_at: Instant::now(),
            description,
        })
    }

    /// Execute a function within a managed transaction
    pub async fn with_transaction<F, R>(
        &self,
        description: Option<String>,
        timeout: Option<Duration>,
        f: F,
    ) -> Result<R, AppError>
    where
        F: for<'c> FnOnce(&mut Transaction<'c, Postgres>) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<R, AppError>> + Send + 'c>>,
    {
        let mut managed_tx = self.begin_transaction(description, timeout).await?;
        
        let result = {
            let tx = managed_tx.transaction.as_mut().unwrap();
            f(tx).await
        };

        match result {
            Ok(value) => {
                managed_tx.commit().await?;
                Ok(value)
            }
            Err(e) => {
                managed_tx.rollback().await?;
                Err(e)
            }
        }
    }

    /// Get statistics about active transactions
    pub async fn get_transaction_stats(&self) -> TransactionStats {
        let active = self.active_transactions.lock().await;
        let now = Instant::now();
        
        let mut stats = TransactionStats {
            active_count: active.len(),
            longest_running: Duration::ZERO,
            average_duration: Duration::ZERO,
            timed_out_count: 0,
        };

        if !active.is_empty() {
            let mut total_duration = Duration::ZERO;
            
            for info in active.values() {
                let duration = now.duration_since(info.started_at);
                total_duration += duration;
                
                if duration > stats.longest_running {
                    stats.longest_running = duration;
                }
                
                if duration > info.timeout {
                    stats.timed_out_count += 1;
                }
            }
            
            stats.average_duration = total_duration / active.len() as u32;
        }

        stats
    }

    /// Get information about all active transactions
    pub async fn get_active_transactions(&self) -> Vec<TransactionInfo> {
        let active = self.active_transactions.lock().await;
        active.values().cloned().collect()
    }

    /// Force rollback of a specific transaction (admin function)
    pub async fn force_rollback(&self, transaction_id: Uuid) -> Result<(), AppError> {
        let mut active = self.active_transactions.lock().await;
        
        if active.remove(&transaction_id).is_some() {
            warn!("Force rolled back transaction: {}", transaction_id);
            Ok(())
        } else {
            Err(AppError::NotFound(format!(
                "Transaction {} not found",
                transaction_id
            )))
        }
    }

    /// Remove a transaction from tracking
    async fn unregister_transaction(&self, transaction_id: Uuid) {
        let mut active = self.active_transactions.lock().await;
        active.remove(&transaction_id);
    }

    /// Start background monitoring for transactions
    fn start_monitoring(&self) {
        let manager = self.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(10));
            
            loop {
                interval.tick().await;
                manager.check_transaction_timeouts().await;
            }
        });
    }

    /// Check for timed out transactions
    async fn check_transaction_timeouts(&self) {
        let now = Instant::now();
        let mut timed_out = Vec::new();
        
        {
            let active = self.active_transactions.lock().await;
            
            for (id, info) in active.iter() {
                let duration = now.duration_since(info.started_at);
                
                if duration > info.timeout {
                    timed_out.push(*id);
                    error!(
                        "Transaction {} has timed out after {:?}: {:?}",
                        id, duration, info.description
                    );
                }
                
                if self.config.log_slow_transactions && duration > self.config.slow_transaction_threshold {
                    warn!(
                        "Slow transaction {} running for {:?}: {:?}",
                        id, duration, info.description
                    );
                }
            }
        }

        // Clean up timed out transactions
        for id in timed_out {
            self.unregister_transaction(id).await;
        }
    }
}

impl Clone for TransactionManager {
    fn clone(&self) -> Self {
        Self {
            pool: Arc::clone(&self.pool),
            active_transactions: Arc::clone(&self.active_transactions),
            config: self.config.clone(),
        }
    }
}

/// A managed database transaction with automatic cleanup
pub struct ManagedTransaction {
    id: Uuid,
    transaction: Option<Transaction<'static, Postgres>>,
    manager: TransactionManager,
    started_at: Instant,
    description: Option<String>,
}

impl ManagedTransaction {
    /// Get the transaction ID
    pub fn id(&self) -> Uuid {
        self.id
    }

    /// Get the transaction duration
    pub fn duration(&self) -> Duration {
        self.started_at.elapsed()
    }

    /// Commit the transaction
    pub async fn commit(mut self) -> Result<(), AppError> {
        if let Some(tx) = self.transaction.take() {
            let duration = self.duration();
            
            tx.commit()
                .await
                .map_err(|e| AppError::Database(format!("Failed to commit transaction: {}", e)))?;

            self.manager.unregister_transaction(self.id).await;

            if self.manager.config.log_slow_transactions && duration > self.manager.config.slow_transaction_threshold {
                warn!(
                    "Slow transaction {} committed after {:?}: {:?}",
                    self.id, duration, self.description
                );
            } else {
                debug!(
                    "Committed transaction {} after {:?}: {:?}",
                    self.id, duration, self.description
                );
            }
        }

        Ok(())
    }

    /// Rollback the transaction
    pub async fn rollback(mut self) -> Result<(), AppError> {
        if let Some(tx) = self.transaction.take() {
            let duration = self.duration();
            
            tx.rollback()
                .await
                .map_err(|e| AppError::Database(format!("Failed to rollback transaction: {}", e)))?;

            self.manager.unregister_transaction(self.id).await;

            debug!(
                "Rolled back transaction {} after {:?}: {:?}",
                self.id, duration, self.description
            );
        }

        Ok(())
    }

    /// Get a mutable reference to the underlying transaction
    pub fn as_mut(&mut self) -> Option<&mut Transaction<'static, Postgres>> {
        self.transaction.as_mut()
    }
}

impl Drop for ManagedTransaction {
    fn drop(&mut self) {
        if self.transaction.is_some() {
            let manager = self.manager.clone();
            let id = self.id;
            let duration = self.duration();
            let description = self.description.clone();
            
            // Spawn a task to clean up the transaction
            tokio::spawn(async move {
                manager.unregister_transaction(id).await;
                warn!(
                    "Transaction {} was dropped without explicit commit/rollback after {:?}: {:?}",
                    id, duration, description
                );
            });
        }
    }
}

#[derive(Debug, Clone)]
pub struct TransactionStats {
    pub active_count: usize,
    pub longest_running: Duration,
    pub average_duration: Duration,
    pub timed_out_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_transaction_manager() {
        let database_url = std::env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost:5432/raffle_platform_test".to_string());

        let pool = Arc::new(
            sqlx::postgres::PgPoolOptions::new()
                .max_connections(5)
                .connect(&database_url)
                .await
                .expect("Failed to connect to test database")
        );

        let config = TransactionConfig {
            enable_monitoring: false, // Disable for testing
            ..Default::default()
        };

        let manager = TransactionManager::new(pool, config);

        // Test successful transaction
        let result = manager.with_transaction(
            Some("Test transaction".to_string()),
            None,
            |tx| {
                Box::pin(async move {
                    sqlx::query!("SELECT 1")
                        .execute(&mut **tx)
                        .await
                        .map_err(|e| AppError::Database(e.to_string()))?;
                    
                    Ok(42)
                })
            }
        ).await;

        assert_eq!(result.unwrap(), 42);

        // Test transaction stats
        let stats = manager.get_transaction_stats().await;
        assert_eq!(stats.active_count, 0);
    }
}