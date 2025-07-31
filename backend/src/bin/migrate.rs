use anyhow::{Context, Result};
use sqlx::postgres::PgPoolOptions;
use std::env;
use std::fs;
use std::path::Path;
use tracing::{info, warn, error};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load environment variables
    dotenv::dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .context("DATABASE_URL environment variable is required")?;

    // Create database connection pool
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .context("Failed to connect to database")?;

    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    let command = args.get(1).map(|s| s.as_str()).unwrap_or("up");

    match command {
        "up" => {
            info!("Running database migrations...");
            run_migrations(&pool).await?;
            info!("Database migrations completed successfully");
        }
        "down" => {
            let steps = args.get(2)
                .and_then(|s| s.parse::<u32>().ok())
                .unwrap_or(1);
            info!("Rolling back {} migration(s)...", steps);
            rollback_migrations(&pool, steps).await?;
            info!("Database rollback completed successfully");
        }
        "status" => {
            show_migration_status(&pool).await?;
        }
        "reset" => {
            warn!("Resetting database - this will drop all data!");
            reset_database(&pool).await?;
            info!("Database reset completed");
        }
        _ => {
            eprintln!("Usage: migrate [up|down [steps]|status|reset]");
            eprintln!("  up      - Run all pending migrations (default)");
            eprintln!("  down    - Rollback migrations (default: 1 step)");
            eprintln!("  status  - Show migration status");
            eprintln!("  reset   - Drop all tables and reset database");
            std::process::exit(1);
        }
    }

    Ok(())
}

async fn run_migrations(pool: &sqlx::PgPool) -> Result<()> {
    // Create migrations table if it doesn't exist
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS schema_migrations (
            version VARCHAR(255) PRIMARY KEY,
            applied_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
        )
        "#,
    )
    .execute(pool)
    .await
    .context("Failed to create migrations table")?;

    // Get list of migration files
    let migrations_dir = Path::new("migrations");
    if !migrations_dir.exists() {
        return Err(anyhow::anyhow!("Migrations directory not found"));
    }

    let mut migration_files = Vec::new();
    for entry in fs::read_dir(migrations_dir)? {
        let entry = entry?;
        let path = entry.path();
        if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
            if filename.ends_with(".sql") && !filename.starts_with("rollback_") {
                migration_files.push(filename.to_string());
            }
        }
    }
    migration_files.sort();

    // Check which migrations have already been applied
    let applied_migrations: Vec<String> = sqlx::query_scalar(
        "SELECT version FROM schema_migrations ORDER BY version"
    )
    .fetch_all(pool)
    .await
    .context("Failed to fetch applied migrations")?;

    // Run pending migrations
    for migration_file in migration_files {
        let version = migration_file.replace(".sql", "");
        
        if applied_migrations.contains(&version) {
            info!("Migration {} already applied, skipping", version);
            continue;
        }

        info!("Applying migration: {}", version);
        
        let migration_path = migrations_dir.join(&migration_file);
        let migration_sql = fs::read_to_string(&migration_path)
            .with_context(|| format!("Failed to read migration file: {}", migration_file))?;

        // Start transaction
        let mut tx = pool.begin().await?;

        // Execute migration
        sqlx::query(&migration_sql)
            .execute(&mut *tx)
            .await
            .with_context(|| format!("Failed to execute migration: {}", version))?;

        // Record migration as applied
        sqlx::query(
            "INSERT INTO schema_migrations (version) VALUES ($1)"
        )
        .bind(&version)
        .execute(&mut *tx)
        .await
        .with_context(|| format!("Failed to record migration: {}", version))?;

        // Commit transaction
        tx.commit().await?;
        
        info!("Successfully applied migration: {}", version);
    }

    Ok(())
}

async fn rollback_migrations(pool: &sqlx::PgPool, steps: u32) -> Result<()> {
    // Get applied migrations in reverse order
    let applied_migrations: Vec<String> = sqlx::query_scalar(
        "SELECT version FROM schema_migrations ORDER BY version DESC LIMIT $1"
    )
    .bind(steps as i64)
    .fetch_all(pool)
    .await
    .context("Failed to fetch applied migrations")?;

    if applied_migrations.is_empty() {
        info!("No migrations to rollback");
        return Ok(());
    }

    let migrations_dir = Path::new("migrations");

    for version in applied_migrations {
        info!("Rolling back migration: {}", version);
        
        let rollback_file = format!("rollback_{}.sql", version);
        let rollback_path = migrations_dir.join(&rollback_file);
        
        if !rollback_path.exists() {
            warn!("Rollback file not found for migration: {}", version);
            continue;
        }

        let rollback_sql = fs::read_to_string(&rollback_path)
            .with_context(|| format!("Failed to read rollback file: {}", rollback_file))?;

        // Start transaction
        let mut tx = pool.begin().await?;

        // Execute rollback
        sqlx::query(&rollback_sql)
            .execute(&mut *tx)
            .await
            .with_context(|| format!("Failed to execute rollback: {}", version))?;

        // Remove migration record
        sqlx::query(
            "DELETE FROM schema_migrations WHERE version = $1"
        )
        .bind(&version)
        .execute(&mut *tx)
        .await
        .with_context(|| format!("Failed to remove migration record: {}", version))?;

        // Commit transaction
        tx.commit().await?;
        
        info!("Successfully rolled back migration: {}", version);
    }

    Ok(())
}

async fn show_migration_status(pool: &sqlx::PgPool) -> Result<()> {
    // Check if migrations table exists
    let table_exists: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS (
            SELECT FROM information_schema.tables 
            WHERE table_schema = 'public' 
            AND table_name = 'schema_migrations'
        )
        "#
    )
    .fetch_one(pool)
    .await
    .context("Failed to check migrations table")?;

    if !table_exists {
        info!("No migrations have been run yet");
        return Ok(());
    }

    // Get applied migrations
    let applied_migrations: Vec<(String, chrono::DateTime<chrono::Utc>)> = sqlx::query_as(
        "SELECT version, applied_at FROM schema_migrations ORDER BY version"
    )
    .fetch_all(pool)
    .await
    .context("Failed to fetch migration status")?;

    if applied_migrations.is_empty() {
        info!("No migrations have been applied");
    } else {
        info!("Applied migrations:");
        for (version, applied_at) in applied_migrations {
            info!("  {} (applied at: {})", version, applied_at.format("%Y-%m-%d %H:%M:%S UTC"));
        }
    }

    // Get pending migrations
    let migrations_dir = Path::new("migrations");
    if migrations_dir.exists() {
        let mut all_migrations = Vec::new();
        for entry in fs::read_dir(migrations_dir)? {
            let entry = entry?;
            let path = entry.path();
            if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                if filename.ends_with(".sql") && !filename.starts_with("rollback_") {
                    let version = filename.replace(".sql", "");
                    all_migrations.push(version);
                }
            }
        }
        all_migrations.sort();

        let applied_versions: Vec<String> = applied_migrations.into_iter().map(|(v, _)| v).collect();
        let pending_migrations: Vec<String> = all_migrations
            .into_iter()
            .filter(|m| !applied_versions.contains(m))
            .collect();

        if pending_migrations.is_empty() {
            info!("No pending migrations");
        } else {
            info!("Pending migrations:");
            for migration in pending_migrations {
                info!("  {}", migration);
            }
        }
    }

    Ok(())
}

async fn reset_database(pool: &sqlx::PgPool) -> Result<()> {
    // Get all applied migrations in reverse order
    let applied_migrations: Vec<String> = sqlx::query_scalar(
        "SELECT version FROM schema_migrations ORDER BY version DESC"
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    // Rollback all migrations
    if !applied_migrations.is_empty() {
        rollback_migrations(pool, applied_migrations.len() as u32).await?;
    }

    // Drop the migrations table
    sqlx::query("DROP TABLE IF EXISTS schema_migrations")
        .execute(pool)
        .await
        .context("Failed to drop migrations table")?;

    info!("Database has been reset");
    Ok(())
}