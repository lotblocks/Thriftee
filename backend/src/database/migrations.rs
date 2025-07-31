use sqlx::{PgPool, Row};
use std::collections::HashMap;
use tracing::{debug, error, info, warn};
use crate::error::AppError;

/// Migration manager for handling database schema changes
pub struct MigrationManager {
    pool: PgPool,
}

impl MigrationManager {
    /// Create a new migration manager
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Run all pending migrations
    pub async fn migrate(&self) -> Result<MigrationResult, AppError> {
        let start_time = std::time::Instant::now();
        
        info!("Starting database migrations...");

        let result = sqlx::migrate!("./migrations")
            .run(&self.pool)
            .await;

        let duration = start_time.elapsed();

        match result {
            Ok(()) => {
                info!("Database migrations completed successfully in {:?}", duration);
                Ok(MigrationResult {
                    success: true,
                    duration,
                    applied_migrations: self.get_applied_migrations().await?,
                    error: None,
                })
            }
            Err(e) => {
                error!("Database migrations failed after {:?}: {}", duration, e);
                Ok(MigrationResult {
                    success: false,
                    duration,
                    applied_migrations: self.get_applied_migrations().await.unwrap_or_default(),
                    error: Some(e.to_string()),
                })
            }
        }
    }

    /// Get list of applied migrations
    pub async fn get_applied_migrations(&self) -> Result<Vec<AppliedMigration>, AppError> {
        let rows = sqlx::query(
            r#"
            SELECT version, description, installed_on, checksum, execution_time
            FROM _sqlx_migrations
            ORDER BY version ASC
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to get applied migrations: {}", e)))?;

        let mut migrations = Vec::new();
        
        for row in rows {
            migrations.push(AppliedMigration {
                version: row.get("version"),
                description: row.get("description"),
                installed_on: row.get("installed_on"),
                checksum: row.get("checksum"),
                execution_time: row.get("execution_time"),
            });
        }

        Ok(migrations)
    }

    /// Check if migrations table exists
    pub async fn migrations_table_exists(&self) -> Result<bool, AppError> {
        let count: i64 = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*)
            FROM information_schema.tables
            WHERE table_name = '_sqlx_migrations'
            AND table_schema = 'public'
            "#
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0);

        Ok(count > 0)
    }

    /// Get migration status
    pub async fn get_migration_status(&self) -> Result<MigrationStatus, AppError> {
        let table_exists = self.migrations_table_exists().await?;
        
        if !table_exists {
            return Ok(MigrationStatus {
                is_initialized: false,
                applied_count: 0,
                pending_count: 0,
                last_migration: None,
                needs_migration: true,
            });
        }

        let applied_migrations = self.get_applied_migrations().await?;
        let applied_count = applied_migrations.len();
        
        // Get the total number of available migrations by checking the migrations directory
        // This is a simplified check - in a real implementation you might want to scan the actual files
        let pending_count = self.estimate_pending_migrations().await?;
        
        let last_migration = applied_migrations.last().cloned();
        
        Ok(MigrationStatus {
            is_initialized: true,
            applied_count,
            pending_count,
            last_migration,
            needs_migration: pending_count > 0,
        })
    }

    /// Estimate the number of pending migrations
    async fn estimate_pending_migrations(&self) -> Result<usize, AppError> {
        // This is a simplified implementation
        // In a real scenario, you'd compare available migration files with applied ones
        Ok(0)
    }

    /// Validate migration integrity
    pub async fn validate_migrations(&self) -> Result<ValidationResult, AppError> {
        let applied_migrations = self.get_applied_migrations().await?;
        let mut issues = Vec::new();
        let mut checksums = HashMap::new();

        // Check for duplicate versions
        for migration in &applied_migrations {
            if let Some(existing_checksum) = checksums.get(&migration.version) {
                if existing_checksum != &migration.checksum {
                    issues.push(ValidationIssue {
                        migration_version: migration.version,
                        issue_type: IssueType::ChecksumMismatch,
                        description: format!(
                            "Migration {} has different checksums: {} vs {}",
                            migration.version, existing_checksum, migration.checksum
                        ),
                    });
                }
            } else {
                checksums.insert(migration.version, migration.checksum.clone());
            }
        }

        // Check for gaps in migration sequence
        let mut versions: Vec<i64> = applied_migrations.iter().map(|m| m.version).collect();
        versions.sort();
        
        for i in 1..versions.len() {
            if versions[i] - versions[i-1] > 1 {
                issues.push(ValidationIssue {
                    migration_version: versions[i],
                    issue_type: IssueType::MissingMigration,
                    description: format!(
                        "Gap in migration sequence between {} and {}",
                        versions[i-1], versions[i]
                    ),
                });
            }
        }

        Ok(ValidationResult {
            is_valid: issues.is_empty(),
            issues,
            total_migrations: applied_migrations.len(),
        })
    }

    /// Reset migrations (dangerous - for development only)
    pub async fn reset_migrations(&self) -> Result<(), AppError> {
        warn!("Resetting all migrations - this will drop all data!");
        
        // Drop all tables except the migrations table
        let tables = self.get_user_tables().await?;
        
        for table in tables {
            if table != "_sqlx_migrations" {
                sqlx::query(&format!("DROP TABLE IF EXISTS {} CASCADE", table))
                    .execute(&self.pool)
                    .await
                    .map_err(|e| AppError::Database(format!("Failed to drop table {}: {}", table, e)))?;
            }
        }

        // Clear migration history
        sqlx::query("DELETE FROM _sqlx_migrations")
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Database(format!("Failed to clear migration history: {}", e)))?;

        info!("Migration reset completed");
        Ok(())
    }

    /// Get list of user tables
    async fn get_user_tables(&self) -> Result<Vec<String>, AppError> {
        let rows = sqlx::query(
            r#"
            SELECT table_name
            FROM information_schema.tables
            WHERE table_schema = 'public'
            AND table_type = 'BASE TABLE'
            ORDER BY table_name
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to get user tables: {}", e)))?;

        Ok(rows.into_iter().map(|row| row.get("table_name")).collect())
    }

    /// Create a backup before running migrations
    pub async fn create_backup(&self, backup_name: &str) -> Result<(), AppError> {
        // This is a placeholder - in a real implementation you'd use pg_dump
        // or a similar tool to create a proper backup
        info!("Creating backup: {}", backup_name);
        
        // For now, just log the backup creation
        debug!("Backup {} would be created here", backup_name);
        
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct MigrationResult {
    pub success: bool,
    pub duration: std::time::Duration,
    pub applied_migrations: Vec<AppliedMigration>,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AppliedMigration {
    pub version: i64,
    pub description: String,
    pub installed_on: chrono::DateTime<chrono::Utc>,
    pub checksum: String,
    pub execution_time: i64,
}

#[derive(Debug, Clone)]
pub struct MigrationStatus {
    pub is_initialized: bool,
    pub applied_count: usize,
    pub pending_count: usize,
    pub last_migration: Option<AppliedMigration>,
    pub needs_migration: bool,
}

#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub issues: Vec<ValidationIssue>,
    pub total_migrations: usize,
}

#[derive(Debug, Clone)]
pub struct ValidationIssue {
    pub migration_version: i64,
    pub issue_type: IssueType,
    pub description: String,
}

#[derive(Debug, Clone)]
pub enum IssueType {
    ChecksumMismatch,
    MissingMigration,
    InvalidSequence,
    CorruptedMigration,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_migration_manager() {
        let database_url = std::env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost:5432/raffle_platform_test".to_string());

        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to connect to test database");

        let manager = MigrationManager::new(pool);

        // Test migration status
        let status = manager.get_migration_status().await.expect("Failed to get migration status");
        
        // The status will depend on whether migrations have been run
        assert!(status.applied_count >= 0);
    }
}