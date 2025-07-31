use sqlx::{PgPool, Row};
use std::collections::HashMap;
use std::time::Duration;
use tracing::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

use crate::utils::errors::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryOptimizationSuggestion {
    pub query_hash: String,
    pub query_text: String,
    pub issue_type: OptimizationIssueType,
    pub severity: OptimizationSeverity,
    pub description: String,
    pub suggestion: String,
    pub estimated_improvement: f64, // Percentage improvement
    pub affected_tables: Vec<String>,
    pub recommended_indexes: Vec<IndexRecommendation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationIssueType {
    MissingIndex,
    UnusedIndex,
    SlowQuery,
    FullTableScan,
    InefficiientJoin,
    SuboptimalDataType,
    LargeResultSet,
    FrequentQuery,
    DeadlockProne,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexRecommendation {
    pub table_name: String,
    pub columns: Vec<String>,
    pub index_type: IndexType,
    pub estimated_size_mb: f64,
    pub estimated_performance_gain: f64,
    pub create_statement: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IndexType {
    BTree,
    Hash,
    Gin,
    Gist,
    Partial,
    Unique,
    Composite,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseHealthMetrics {
    pub timestamp: DateTime<Utc>,
    pub connection_pool_usage: f64,
    pub active_connections: u32,
    pub idle_connections: u32,
    pub slow_query_count: u32,
    pub deadlock_count: u32,
    pub cache_hit_ratio: f64,
    pub average_query_time_ms: f64,
    pub total_queries_per_second: f64,
    pub database_size_mb: u64,
    pub largest_tables: Vec<TableSizeInfo>,
    pub index_usage_stats: Vec<IndexUsageInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableSizeInfo {
    pub table_name: String,
    pub size_mb: u64,
    pub row_count: u64,
    pub index_size_mb: u64,
    pub bloat_ratio: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexUsageInfo {
    pub table_name: String,
    pub index_name: String,
    pub size_mb: u64,
    pub scans: u64,
    pub tuples_read: u64,
    pub tuples_fetched: u64,
    pub usage_ratio: f64,
}

#[derive(Debug, Clone)]
pub struct DatabaseOptimizationConfig {
    pub slow_query_threshold_ms: u64,
    pub analysis_window_hours: u32,
    pub min_query_count_for_analysis: u32,
    pub enable_automatic_index_suggestions: bool,
    pub enable_query_rewrite_suggestions: bool,
    pub max_suggestions_per_run: usize,
}

impl Default for DatabaseOptimizationConfig {
    fn default() -> Self {
        Self {
            slow_query_threshold_ms: 1000,
            analysis_window_hours: 24,
            min_query_count_for_analysis: 10,
            enable_automatic_index_suggestions: true,
            enable_query_rewrite_suggestions: true,
            max_suggestions_per_run: 50,
        }
    }
}

pub struct DatabaseOptimizationService {
    pool: PgPool,
    config: DatabaseOptimizationConfig,
}

impl DatabaseOptimizationService {
    pub fn new(pool: PgPool, config: DatabaseOptimizationConfig) -> Self {
        Self { pool, config }
    }

    /// Analyze database performance and generate optimization suggestions
    pub async fn analyze_performance(&self) -> Result<Vec<QueryOptimizationSuggestion>, AppError> {
        let mut suggestions = Vec::new();

        // Analyze slow queries
        suggestions.extend(self.analyze_slow_queries().await?);

        // Analyze missing indexes
        if self.config.enable_automatic_index_suggestions {
            suggestions.extend(self.suggest_missing_indexes().await?);
        }

        // Analyze unused indexes
        suggestions.extend(self.find_unused_indexes().await?);

        // Analyze table scans
        suggestions.extend(self.analyze_table_scans().await?);

        // Sort by severity and estimated improvement
        suggestions.sort_by(|a, b| {
            let severity_order = |s: &OptimizationSeverity| match s {
                OptimizationSeverity::Critical => 4,
                OptimizationSeverity::High => 3,
                OptimizationSeverity::Medium => 2,
                OptimizationSeverity::Low => 1,
            };
            
            severity_order(&b.severity).cmp(&severity_order(&a.severity))
                .then(b.estimated_improvement.partial_cmp(&a.estimated_improvement).unwrap_or(std::cmp::Ordering::Equal))
        });

        // Limit results
        suggestions.truncate(self.config.max_suggestions_per_run);

        Ok(suggestions)
    }

    /// Get current database health metrics
    pub async fn get_health_metrics(&self) -> Result<DatabaseHealthMetrics, AppError> {
        let connection_stats = self.get_connection_stats().await?;
        let query_stats = self.get_query_performance_stats().await?;
        let size_stats = self.get_database_size_stats().await?;
        let table_sizes = self.get_table_sizes().await?;
        let index_usage = self.get_index_usage_stats().await?;

        Ok(DatabaseHealthMetrics {
            timestamp: Utc::now(),
            connection_pool_usage: connection_stats.usage_percentage,
            active_connections: connection_stats.active,
            idle_connections: connection_stats.idle,
            slow_query_count: query_stats.slow_queries,
            deadlock_count: query_stats.deadlocks,
            cache_hit_ratio: query_stats.cache_hit_ratio,
            average_query_time_ms: query_stats.avg_query_time_ms,
            total_queries_per_second: query_stats.queries_per_second,
            database_size_mb: size_stats.total_size_mb,
            largest_tables: table_sizes,
            index_usage_stats: index_usage,
        })
    }

    /// Optimize database configuration
    pub async fn optimize_configuration(&self) -> Result<Vec<String>, AppError> {
        let mut recommendations = Vec::new();

        // Check shared_buffers
        let shared_buffers = self.get_config_value("shared_buffers").await?;
        if let Some(value) = shared_buffers {
            if self.parse_memory_setting(&value)? < 256 * 1024 * 1024 { // Less than 256MB
                recommendations.push(
                    "Consider increasing shared_buffers to 25% of available RAM for better performance".to_string()
                );
            }
        }

        // Check effective_cache_size
        let effective_cache_size = self.get_config_value("effective_cache_size").await?;
        if let Some(value) = effective_cache_size {
            if self.parse_memory_setting(&value)? < 1024 * 1024 * 1024 { // Less than 1GB
                recommendations.push(
                    "Consider increasing effective_cache_size to 75% of available RAM".to_string()
                );
            }
        }

        // Check work_mem
        let work_mem = self.get_config_value("work_mem").await?;
        if let Some(value) = work_mem {
            if self.parse_memory_setting(&value)? < 4 * 1024 * 1024 { // Less than 4MB
                recommendations.push(
                    "Consider increasing work_mem for better sort and hash performance".to_string()
                );
            }
        }

        // Check maintenance_work_mem
        let maintenance_work_mem = self.get_config_value("maintenance_work_mem").await?;
        if let Some(value) = maintenance_work_mem {
            if self.parse_memory_setting(&value)? < 64 * 1024 * 1024 { // Less than 64MB
                recommendations.push(
                    "Consider increasing maintenance_work_mem for faster VACUUM and CREATE INDEX".to_string()
                );
            }
        }

        // Check checkpoint settings
        let checkpoint_completion_target = self.get_config_value("checkpoint_completion_target").await?;
        if let Some(value) = checkpoint_completion_target {
            if value.parse::<f64>().unwrap_or(0.0) < 0.7 {
                recommendations.push(
                    "Consider setting checkpoint_completion_target to 0.9 for smoother checkpoints".to_string()
                );
            }
        }

        // Check WAL settings
        let wal_buffers = self.get_config_value("wal_buffers").await?;
        if let Some(value) = wal_buffers {
            if self.parse_memory_setting(&value)? < 16 * 1024 * 1024 { // Less than 16MB
                recommendations.push(
                    "Consider increasing wal_buffers to 16MB for better write performance".to_string()
                );
            }
        }

        Ok(recommendations)
    }

    /// Run database maintenance tasks
    pub async fn run_maintenance(&self) -> Result<MaintenanceReport, AppError> {
        let mut report = MaintenanceReport {
            timestamp: Utc::now(),
            tasks_completed: Vec::new(),
            errors: Vec::new(),
            total_duration_ms: 0,
        };

        let start_time = std::time::Instant::now();

        // Update table statistics
        match self.update_table_statistics().await {
            Ok(tables_updated) => {
                report.tasks_completed.push(format!("Updated statistics for {} tables", tables_updated));
            }
            Err(e) => {
                report.errors.push(format!("Failed to update table statistics: {}", e));
            }
        }

        // Reindex fragmented indexes
        match self.reindex_fragmented_indexes().await {
            Ok(indexes_reindexed) => {
                if indexes_reindexed > 0 {
                    report.tasks_completed.push(format!("Reindexed {} fragmented indexes", indexes_reindexed));
                }
            }
            Err(e) => {
                report.errors.push(format!("Failed to reindex fragmented indexes: {}", e));
            }
        }

        // Clean up old performance data
        match self.cleanup_old_performance_data().await {
            Ok(rows_deleted) => {
                if rows_deleted > 0 {
                    report.tasks_completed.push(format!("Cleaned up {} old performance records", rows_deleted));
                }
            }
            Err(e) => {
                report.errors.push(format!("Failed to cleanup old performance data: {}", e));
            }
        }

        report.total_duration_ms = start_time.elapsed().as_millis() as u64;
        Ok(report)
    }

    /// Create recommended indexes
    pub async fn create_recommended_index(&self, recommendation: &IndexRecommendation) -> Result<(), AppError> {
        info!("Creating recommended index: {}", recommendation.create_statement);

        // Execute the CREATE INDEX statement
        sqlx::query(&recommendation.create_statement)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::InternalServerError(format!("Failed to create index: {}", e)))?;

        info!("Successfully created index on table {}", recommendation.table_name);
        Ok(())
    }

    // Private helper methods

    async fn analyze_slow_queries(&self) -> Result<Vec<QueryOptimizationSuggestion>, AppError> {
        let mut suggestions = Vec::new();

        let slow_queries = sqlx::query!(
            r#"
            SELECT 
                query_hash,
                query_text,
                AVG(execution_time_ms) as avg_execution_time,
                COUNT(*) as execution_count,
                SUM(execution_time_ms) as total_execution_time
            FROM query_performance 
            WHERE timestamp >= NOW() - INTERVAL '24 hours'
                AND execution_time_ms > $1
            GROUP BY query_hash, query_text
            HAVING COUNT(*) >= $2
            ORDER BY AVG(execution_time_ms) DESC
            LIMIT 20
            "#,
            self.config.slow_query_threshold_ms as i64,
            self.config.min_query_count_for_analysis as i64
        )
        .fetch_all(&self.pool)
        .await?;

        for query in slow_queries {
            let avg_time = query.avg_execution_time.unwrap_or(0.0);
            let severity = if avg_time > 5000.0 {
                OptimizationSeverity::Critical
            } else if avg_time > 2000.0 {
                OptimizationSeverity::High
            } else if avg_time > 1000.0 {
                OptimizationSeverity::Medium
            } else {
                OptimizationSeverity::Low
            };

            suggestions.push(QueryOptimizationSuggestion {
                query_hash: query.query_hash,
                query_text: query.query_text.unwrap_or_default(),
                issue_type: OptimizationIssueType::SlowQuery,
                severity,
                description: format!(
                    "Query executes slowly with average time of {:.2}ms over {} executions",
                    avg_time, query.execution_count.unwrap_or(0)
                ),
                suggestion: "Consider adding appropriate indexes, optimizing WHERE clauses, or rewriting the query".to_string(),
                estimated_improvement: ((avg_time - 100.0) / avg_time * 100.0).max(0.0),
                affected_tables: self.extract_table_names(&query.query_text.unwrap_or_default()),
                recommended_indexes: Vec::new(), // Would be populated by index analysis
            });
        }

        Ok(suggestions)
    }

    async fn suggest_missing_indexes(&self) -> Result<Vec<QueryOptimizationSuggestion>, AppError> {
        let mut suggestions = Vec::new();

        // Analyze queries for potential missing indexes
        let queries_needing_indexes = sqlx::query!(
            r#"
            SELECT DISTINCT
                query_text,
                query_hash,
                AVG(execution_time_ms) as avg_execution_time,
                COUNT(*) as execution_count
            FROM query_performance 
            WHERE timestamp >= NOW() - INTERVAL '24 hours'
                AND query_text ILIKE '%WHERE%'
                AND execution_time_ms > 500
            GROUP BY query_text, query_hash
            HAVING COUNT(*) >= 5
            ORDER BY AVG(execution_time_ms) DESC
            LIMIT 10
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        for query in queries_needing_indexes {
            let query_text = query.query_text.unwrap_or_default();
            let tables = self.extract_table_names(&query_text);
            let where_columns = self.extract_where_columns(&query_text);

            if !where_columns.is_empty() {
                for table in &tables {
                    for column in &where_columns {
                        // Check if index already exists
                        if !self.index_exists(table, column).await? {
                            let recommendation = IndexRecommendation {
                                table_name: table.clone(),
                                columns: vec![column.clone()],
                                index_type: IndexType::BTree,
                                estimated_size_mb: 10.0, // Rough estimate
                                estimated_performance_gain: 50.0,
                                create_statement: format!(
                                    "CREATE INDEX CONCURRENTLY idx_{}_{} ON {} ({})",
                                    table, column, table, column
                                ),
                            };

                            suggestions.push(QueryOptimizationSuggestion {
                                query_hash: query.query_hash.clone(),
                                query_text: query_text.clone(),
                                issue_type: OptimizationIssueType::MissingIndex,
                                severity: OptimizationSeverity::Medium,
                                description: format!(
                                    "Query frequently filters on {}.{} without an index",
                                    table, column
                                ),
                                suggestion: format!(
                                    "Create an index on {}.{} to improve query performance",
                                    table, column
                                ),
                                estimated_improvement: 50.0,
                                affected_tables: vec![table.clone()],
                                recommended_indexes: vec![recommendation],
                            });
                        }
                    }
                }
            }
        }

        Ok(suggestions)
    }

    async fn find_unused_indexes(&self) -> Result<Vec<QueryOptimizationSuggestion>, AppError> {
        let mut suggestions = Vec::new();

        let unused_indexes = sqlx::query!(
            r#"
            SELECT 
                schemaname,
                tablename,
                indexname,
                pg_size_pretty(pg_relation_size(indexrelid)) as size,
                idx_scan,
                idx_tup_read,
                idx_tup_fetch
            FROM pg_stat_user_indexes 
            WHERE idx_scan < 10
                AND pg_relation_size(indexrelid) > 1024 * 1024  -- Larger than 1MB
            ORDER BY pg_relation_size(indexrelid) DESC
            LIMIT 10
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        for index in unused_indexes {
            suggestions.push(QueryOptimizationSuggestion {
                query_hash: "unused_index".to_string(),
                query_text: format!("Index: {}.{}", index.tablename, index.indexname),
                issue_type: OptimizationIssueType::UnusedIndex,
                severity: OptimizationSeverity::Low,
                description: format!(
                    "Index {} on table {} is rarely used ({} scans) but consumes space ({})",
                    index.indexname, index.tablename, index.idx_scan.unwrap_or(0), index.size.unwrap_or_default()
                ),
                suggestion: format!("Consider dropping unused index {}", index.indexname),
                estimated_improvement: 5.0, // Small improvement from reduced maintenance overhead
                affected_tables: vec![index.tablename],
                recommended_indexes: Vec::new(),
            });
        }

        Ok(suggestions)
    }

    async fn analyze_table_scans(&self) -> Result<Vec<QueryOptimizationSuggestion>, AppError> {
        let mut suggestions = Vec::new();

        let tables_with_scans = sqlx::query!(
            r#"
            SELECT 
                schemaname,
                tablename,
                seq_scan,
                seq_tup_read,
                idx_scan,
                idx_tup_fetch,
                n_tup_ins + n_tup_upd + n_tup_del as total_modifications
            FROM pg_stat_user_tables 
            WHERE seq_scan > idx_scan * 2  -- More sequential scans than index scans
                AND seq_tup_read > 10000     -- Reading significant number of rows
            ORDER BY seq_tup_read DESC
            LIMIT 10
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        for table in tables_with_scans {
            suggestions.push(QueryOptimizationSuggestion {
                query_hash: "table_scan".to_string(),
                query_text: format!("Table: {}", table.tablename),
                issue_type: OptimizationIssueType::FullTableScan,
                severity: OptimizationSeverity::Medium,
                description: format!(
                    "Table {} has {} sequential scans reading {} rows, vs {} index scans",
                    table.tablename,
                    table.seq_scan.unwrap_or(0),
                    table.seq_tup_read.unwrap_or(0),
                    table.idx_scan.unwrap_or(0)
                ),
                suggestion: "Analyze queries against this table and consider adding appropriate indexes".to_string(),
                estimated_improvement: 30.0,
                affected_tables: vec![table.tablename],
                recommended_indexes: Vec::new(),
            });
        }

        Ok(suggestions)
    }

    async fn get_connection_stats(&self) -> Result<ConnectionStats, AppError> {
        let stats = sqlx::query!(
            r#"
            SELECT 
                (SELECT setting::int FROM pg_settings WHERE name = 'max_connections') as max_connections,
                (SELECT count(*) FROM pg_stat_activity WHERE state = 'active') as active_connections,
                (SELECT count(*) FROM pg_stat_activity WHERE state = 'idle') as idle_connections,
                (SELECT count(*) FROM pg_stat_activity) as total_connections
            "#
        )
        .fetch_one(&self.pool)
        .await?;

        let max_conn = stats.max_connections.unwrap_or(100) as u32;
        let active = stats.active_connections.unwrap_or(0) as u32;
        let idle = stats.idle_connections.unwrap_or(0) as u32;
        let total = stats.total_connections.unwrap_or(0) as u32;

        Ok(ConnectionStats {
            max_connections: max_conn,
            active,
            idle,
            total,
            usage_percentage: (total as f64 / max_conn as f64) * 100.0,
        })
    }

    async fn get_query_performance_stats(&self) -> Result<QueryPerformanceStats, AppError> {
        let stats = sqlx::query!(
            r#"
            SELECT 
                COUNT(*) FILTER (WHERE execution_time_ms > $1) as slow_queries,
                AVG(execution_time_ms) as avg_query_time,
                COUNT(*) as total_queries,
                COALESCE(
                    (SELECT blks_hit::float / (blks_hit + blks_read) * 100 
                     FROM pg_stat_database 
                     WHERE datname = current_database()), 
                    0
                ) as cache_hit_ratio
            FROM query_performance 
            WHERE timestamp >= NOW() - INTERVAL '1 hour'
            "#,
            self.config.slow_query_threshold_ms as i64
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(QueryPerformanceStats {
            slow_queries: stats.slow_queries.unwrap_or(0) as u32,
            avg_query_time_ms: stats.avg_query_time.unwrap_or(0.0),
            total_queries: stats.total_queries.unwrap_or(0) as u64,
            queries_per_second: stats.total_queries.unwrap_or(0) as f64 / 3600.0, // Per hour to per second
            cache_hit_ratio: stats.cache_hit_ratio.unwrap_or(0.0),
            deadlocks: 0, // Would need additional query to get deadlock count
        })
    }

    async fn get_database_size_stats(&self) -> Result<DatabaseSizeStats, AppError> {
        let stats = sqlx::query!(
            "SELECT pg_database_size(current_database()) as database_size"
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(DatabaseSizeStats {
            total_size_mb: (stats.database_size.unwrap_or(0) / (1024 * 1024)) as u64,
        })
    }

    async fn get_table_sizes(&self) -> Result<Vec<TableSizeInfo>, AppError> {
        let tables = sqlx::query!(
            r#"
            SELECT 
                schemaname || '.' || tablename as table_name,
                pg_total_relation_size(schemaname||'.'||tablename) / (1024*1024) as size_mb,
                pg_relation_size(schemaname||'.'||tablename) / (1024*1024) as table_size_mb,
                (pg_total_relation_size(schemaname||'.'||tablename) - pg_relation_size(schemaname||'.'||tablename)) / (1024*1024) as index_size_mb,
                COALESCE(n_tup_ins + n_tup_upd + n_tup_del, 0) as row_count_estimate
            FROM pg_tables t
            LEFT JOIN pg_stat_user_tables s ON t.tablename = s.relname AND t.schemaname = s.schemaname
            WHERE schemaname NOT IN ('information_schema', 'pg_catalog')
            ORDER BY pg_total_relation_size(schemaname||'.'||tablename) DESC
            LIMIT 20
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        let mut result = Vec::new();
        for table in tables {
            result.push(TableSizeInfo {
                table_name: table.table_name.unwrap_or_default(),
                size_mb: table.size_mb.unwrap_or(0) as u64,
                row_count: table.row_count_estimate.unwrap_or(0) as u64,
                index_size_mb: table.index_size_mb.unwrap_or(0) as u64,
                bloat_ratio: 0.0, // Would need more complex query to calculate bloat
            });
        }

        Ok(result)
    }

    async fn get_index_usage_stats(&self) -> Result<Vec<IndexUsageInfo>, AppError> {
        let indexes = sqlx::query!(
            r#"
            SELECT 
                schemaname || '.' || tablename as table_name,
                indexname,
                pg_relation_size(indexrelid) / (1024*1024) as size_mb,
                idx_scan,
                idx_tup_read,
                idx_tup_fetch
            FROM pg_stat_user_indexes
            WHERE pg_relation_size(indexrelid) > 0
            ORDER BY pg_relation_size(indexrelid) DESC
            LIMIT 50
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        let mut result = Vec::new();
        for index in indexes {
            let scans = index.idx_scan.unwrap_or(0) as u64;
            let tuples_read = index.idx_tup_read.unwrap_or(0) as u64;
            
            result.push(IndexUsageInfo {
                table_name: index.table_name.unwrap_or_default(),
                index_name: index.indexname,
                size_mb: index.size_mb.unwrap_or(0) as u64,
                scans,
                tuples_read,
                tuples_fetched: index.idx_tup_fetch.unwrap_or(0) as u64,
                usage_ratio: if tuples_read > 0 { scans as f64 / tuples_read as f64 } else { 0.0 },
            });
        }

        Ok(result)
    }

    async fn get_config_value(&self, setting_name: &str) -> Result<Option<String>, AppError> {
        let result = sqlx::query!(
            "SELECT setting FROM pg_settings WHERE name = $1",
            setting_name
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(result.map(|r| r.setting))
    }

    fn parse_memory_setting(&self, setting: &str) -> Result<u64, AppError> {
        let setting = setting.trim().to_lowercase();
        
        if let Some(pos) = setting.find("gb") {
            let num: f64 = setting[..pos].parse()
                .map_err(|_| AppError::InternalServerError("Invalid memory setting".to_string()))?;
            Ok((num * 1024.0 * 1024.0 * 1024.0) as u64)
        } else if let Some(pos) = setting.find("mb") {
            let num: f64 = setting[..pos].parse()
                .map_err(|_| AppError::InternalServerError("Invalid memory setting".to_string()))?;
            Ok((num * 1024.0 * 1024.0) as u64)
        } else if let Some(pos) = setting.find("kb") {
            let num: f64 = setting[..pos].parse()
                .map_err(|_| AppError::InternalServerError("Invalid memory setting".to_string()))?;
            Ok((num * 1024.0) as u64)
        } else {
            // Assume bytes
            setting.parse()
                .map_err(|_| AppError::InternalServerError("Invalid memory setting".to_string()))
        }
    }

    async fn update_table_statistics(&self) -> Result<u32, AppError> {
        let tables = sqlx::query!(
            "SELECT schemaname, tablename FROM pg_tables WHERE schemaname NOT IN ('information_schema', 'pg_catalog')"
        )
        .fetch_all(&self.pool)
        .await?;

        let mut updated_count = 0;
        for table in tables {
            let table_name = format!("{}.{}", table.schemaname, table.tablename);
            match sqlx::query(&format!("ANALYZE {}", table_name))
                .execute(&self.pool)
                .await
            {
                Ok(_) => {
                    updated_count += 1;
                    debug!("Updated statistics for table: {}", table_name);
                }
                Err(e) => {
                    warn!("Failed to update statistics for table {}: {}", table_name, e);
                }
            }
        }

        Ok(updated_count)
    }

    async fn reindex_fragmented_indexes(&self) -> Result<u32, AppError> {
        // This is a simplified implementation
        // In production, you'd want more sophisticated fragmentation detection
        Ok(0)
    }

    async fn cleanup_old_performance_data(&self) -> Result<u64, AppError> {
        let result = sqlx::query!(
            "DELETE FROM query_performance WHERE timestamp < NOW() - INTERVAL '7 days'"
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    async fn index_exists(&self, table_name: &str, column_name: &str) -> Result<bool, AppError> {
        let result = sqlx::query!(
            r#"
            SELECT 1 FROM pg_indexes 
            WHERE tablename = $1 
                AND indexdef ILIKE '%' || $2 || '%'
            LIMIT 1
            "#,
            table_name,
            column_name
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(result.is_some())
    }

    fn extract_table_names(&self, query: &str) -> Vec<String> {
        // Simplified table name extraction
        // In production, you'd use a proper SQL parser
        let mut tables = Vec::new();
        let query_lower = query.to_lowercase();
        
        if let Some(from_pos) = query_lower.find(" from ") {
            let after_from = &query_lower[from_pos + 6..];
            if let Some(space_pos) = after_from.find(' ') {
                let table_name = after_from[..space_pos].trim();
                if !table_name.is_empty() {
                    tables.push(table_name.to_string());
                }
            }
        }
        
        tables
    }

    fn extract_where_columns(&self, query: &str) -> Vec<String> {
        // Simplified WHERE column extraction
        // In production, you'd use a proper SQL parser
        let mut columns = Vec::new();
        let query_lower = query.to_lowercase();
        
        if let Some(where_pos) = query_lower.find(" where ") {
            let where_clause = &query_lower[where_pos + 7..];
            // Look for patterns like "column_name ="
            for word in where_clause.split_whitespace() {
                if word.ends_with('=') || word.ends_with("!=") || word.ends_with('>') || word.ends_with('<') {
                    let column = word.trim_end_matches(|c: char| !c.is_alphanumeric() && c != '_');
                    if !column.is_empty() && column != "and" && column != "or" {
                        columns.push(column.to_string());
                    }
                }
            }
        }
        
        columns
    }
}

#[derive(Debug, Clone)]
struct ConnectionStats {
    max_connections: u32,
    active: u32,
    idle: u32,
    total: u32,
    usage_percentage: f64,
}

#[derive(Debug, Clone)]
struct QueryPerformanceStats {
    slow_queries: u32,
    avg_query_time_ms: f64,
    total_queries: u64,
    queries_per_second: f64,
    cache_hit_ratio: f64,
    deadlocks: u32,
}

#[derive(Debug, Clone)]
struct DatabaseSizeStats {
    total_size_mb: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceReport {
    pub timestamp: DateTime<Utc>,
    pub tasks_completed: Vec<String>,
    pub errors: Vec<String>,
    pub total_duration_ms: u64,
}

impl Clone for DatabaseOptimizationService {
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
            config: self.config.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_setting_parsing() {
        let service = DatabaseOptimizationService::new(
            // Mock pool would be needed
            todo!(),
            DatabaseOptimizationConfig::default()
        );

        assert_eq!(service.parse_memory_setting("256MB").unwrap(), 256 * 1024 * 1024);
        assert_eq!(service.parse_memory_setting("1GB").unwrap(), 1024 * 1024 * 1024);
        assert_eq!(service.parse_memory_setting("512KB").unwrap(), 512 * 1024);
    }

    #[test]
    fn test_table_name_extraction() {
        let service = DatabaseOptimizationService::new(
            // Mock pool would be needed
            todo!(),
            DatabaseOptimizationConfig::default()
        );

        let query = "SELECT * FROM users WHERE id = 1";
        let tables = service.extract_table_names(query);
        assert_eq!(tables, vec!["users"]);
    }
}