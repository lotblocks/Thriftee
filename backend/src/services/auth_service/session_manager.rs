use crate::error::AppError;
use crate::models::UserSession;
use crate::utils::crypto::{hash_token, generate_secure_token};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct SessionManager {
    pool: Arc<PgPool>,
    config: SessionConfig,
}

#[derive(Debug, Clone)]
pub struct SessionConfig {
    pub max_sessions_per_user: u32,
    pub session_timeout: Duration,
    pub cleanup_interval: Duration,
    pub enable_device_tracking: bool,
    pub enable_location_tracking: bool,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            max_sessions_per_user: 5,
            session_timeout: Duration::days(7),
            cleanup_interval: Duration::hours(1),
            enable_device_tracking: true,
            enable_location_tracking: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub id: Uuid,
    pub user_id: Uuid,
    pub device_info: Option<DeviceInfo>,
    pub ip_address: Option<IpAddr>,
    pub created_at: DateTime<Utc>,
    pub last_used: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub is_current: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub device_type: String,
    pub browser: Option<String>,
    pub os: Option<String>,
    pub user_agent: String,
}

impl SessionManager {
    pub fn new(pool: Arc<PgPool>, config: SessionConfig) -> Self {
        let manager = Self { pool, config };
        
        // Start background cleanup task
        manager.start_cleanup_task();
        
        manager
    }

    /// Create a new session
    pub async fn create_session(
        &self,
        user_id: Uuid,
        refresh_token_hash: String,
        ip_address: Option<IpAddr>,
        user_agent: Option<String>,
    ) -> Result<UserSession, AppError> {
        // Check if user has too many active sessions
        self.enforce_session_limit(user_id).await?;

        let expires_at = Utc::now() + self.config.session_timeout;
        let device_info = if self.config.enable_device_tracking {
            user_agent.as_ref().map(|ua| self.parse_device_info(ua))
        } else {
            None
        };

        let session = sqlx::query_as!(
            UserSession,
            r#"
            INSERT INTO user_sessions (user_id, refresh_token_hash, device_info, ip_address, user_agent, expires_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, user_id, refresh_token_hash, device_info, 
                     ip_address as "ip_address: std::net::IpAddr", user_agent, 
                     expires_at, is_active, created_at, updated_at
            "#,
            user_id,
            refresh_token_hash,
            device_info.map(|d| serde_json::to_value(d).unwrap()),
            ip_address,
            user_agent,
            expires_at
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(session)
    }

    /// Get active sessions for a user
    pub async fn get_user_sessions(&self, user_id: Uuid) -> Result<Vec<SessionInfo>, AppError> {
        let sessions = sqlx::query!(
            r#"
            SELECT id, user_id, device_info, ip_address as "ip_address: std::net::IpAddr", created_at, updated_at as last_used, expires_at, is_active
            FROM user_sessions 
            WHERE user_id = $1 AND is_active = true AND expires_at > NOW()
            ORDER BY updated_at DESC
            "#,
            user_id
        )
        .fetch_all(&*self.pool)
        .await?;

        let mut session_infos = Vec::new();
        for (index, session) in sessions.iter().enumerate() {
            let device_info = session.device_info.as_ref()
                .and_then(|v| serde_json::from_value(v.clone()).ok());
            
            let ip_address = session.ip_address;

            session_infos.push(SessionInfo {
                id: session.id,
                user_id: session.user_id,
                device_info,
                ip_address,
                created_at: session.created_at,
                last_used: session.last_used,
                expires_at: session.expires_at,
                is_current: index == 0, // Most recent session is current
            });
        }

        Ok(session_infos)
    }

    /// Revoke a specific session
    pub async fn revoke_session(&self, session_id: Uuid, user_id: Uuid) -> Result<(), AppError> {
        let result = sqlx::query!(
            "UPDATE user_sessions SET is_active = false WHERE id = $1 AND user_id = $2",
            session_id,
            user_id
        )
        .execute(&*self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Session not found".to_string()));
        }

        Ok(())
    }

    /// Revoke all sessions except the current one
    pub async fn revoke_other_sessions(
        &self,
        user_id: Uuid,
        current_session_id: Uuid,
    ) -> Result<u64, AppError> {
        let result = sqlx::query!(
            "UPDATE user_sessions SET is_active = false WHERE user_id = $1 AND id != $2 AND is_active = true",
            user_id,
            current_session_id
        )
        .execute(&*self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    /// Revoke all sessions for a user
    pub async fn revoke_all_sessions(&self, user_id: Uuid) -> Result<u64, AppError> {
        let result = sqlx::query!(
            "UPDATE user_sessions SET is_active = false WHERE user_id = $1 AND is_active = true",
            user_id
        )
        .execute(&*self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    /// Update session activity
    pub async fn update_session_activity(
        &self,
        refresh_token_hash: &str,
        ip_address: Option<IpAddr>,
    ) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            UPDATE user_sessions 
            SET updated_at = NOW(), ip_address = COALESCE($2, ip_address)
            WHERE refresh_token_hash = $1 AND is_active = true
            "#,
            refresh_token_hash,
            ip_address
        )
        .execute(&*self.pool)
        .await?;

        Ok(())
    }

    /// Check if session exists and is valid
    pub async fn validate_session(&self, refresh_token_hash: &str) -> Result<bool, AppError> {
        let count: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM user_sessions WHERE refresh_token_hash = $1 AND is_active = true AND expires_at > NOW()",
            refresh_token_hash
        )
        .fetch_one(&*self.pool)
        .await?
        .unwrap_or(0);

        Ok(count > 0)
    }

    /// Get session by refresh token hash
    pub async fn get_session_by_token(&self, refresh_token_hash: &str) -> Result<Option<UserSession>, AppError> {
        let session = sqlx::query_as!(
            UserSession,
            r#"
            SELECT id, user_id, refresh_token_hash, device_info, 
                   ip_address as "ip_address: std::net::IpAddr", user_agent, 
                   expires_at, is_active, created_at, updated_at
            FROM user_sessions 
            WHERE refresh_token_hash = $1 AND is_active = true AND expires_at > NOW()
            "#,
            refresh_token_hash
        )
        .fetch_optional(&*self.pool)
        .await?;

        Ok(session)
    }

    /// Enforce session limit per user
    async fn enforce_session_limit(&self, user_id: Uuid) -> Result<(), AppError> {
        let active_sessions: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM user_sessions WHERE user_id = $1 AND is_active = true AND expires_at > NOW()",
            user_id
        )
        .fetch_one(&*self.pool)
        .await?
        .unwrap_or(0);

        if active_sessions >= self.config.max_sessions_per_user as i64 {
            // Revoke the oldest session
            sqlx::query!(
                r#"
                UPDATE user_sessions 
                SET is_active = false 
                WHERE id = (
                    SELECT id FROM user_sessions 
                    WHERE user_id = $1 AND is_active = true 
                    ORDER BY created_at ASC 
                    LIMIT 1
                )
                "#,
                user_id
            )
            .execute(&*self.pool)
            .await?;
        }

        Ok(())
    }

    /// Parse device information from user agent
    fn parse_device_info(&self, user_agent: &str) -> DeviceInfo {
        // This is a simplified parser. In production, you'd use a proper user agent parser
        let device_type = if user_agent.contains("Mobile") {
            "mobile".to_string()
        } else if user_agent.contains("Tablet") {
            "tablet".to_string()
        } else {
            "desktop".to_string()
        };

        let browser = if user_agent.contains("Chrome") {
            Some("Chrome".to_string())
        } else if user_agent.contains("Firefox") {
            Some("Firefox".to_string())
        } else if user_agent.contains("Safari") {
            Some("Safari".to_string())
        } else {
            None
        };

        let os = if user_agent.contains("Windows") {
            Some("Windows".to_string())
        } else if user_agent.contains("Mac") {
            Some("macOS".to_string())
        } else if user_agent.contains("Linux") {
            Some("Linux".to_string())
        } else if user_agent.contains("Android") {
            Some("Android".to_string())
        } else if user_agent.contains("iOS") {
            Some("iOS".to_string())
        } else {
            None
        };

        DeviceInfo {
            device_type,
            browser,
            os,
            user_agent: user_agent.to_string(),
        }
    }

    /// Start background cleanup task
    fn start_cleanup_task(&self) {
        let pool = self.pool.clone();
        let interval = self.config.cleanup_interval;

        tokio::spawn(async move {
            let mut cleanup_interval = tokio::time::interval(interval.to_std().unwrap());
            
            loop {
                cleanup_interval.tick().await;
                
                if let Err(e) = Self::cleanup_expired_sessions(&pool).await {
                    tracing::error!("Failed to cleanup expired sessions: {}", e);
                }
            }
        });
    }

    /// Clean up expired sessions
    async fn cleanup_expired_sessions(pool: &PgPool) -> Result<u64, AppError> {
        let result = sqlx::query!(
            "DELETE FROM user_sessions WHERE expires_at < NOW() OR (is_active = false AND updated_at < NOW() - INTERVAL '30 days')"
        )
        .execute(pool)
        .await?;

        if result.rows_affected() > 0 {
            tracing::info!("Cleaned up {} expired sessions", result.rows_affected());
        }

        Ok(result.rows_affected())
    }

    /// Get session statistics
    pub async fn get_session_statistics(&self) -> Result<SessionStatistics, AppError> {
        let stats = sqlx::query!(
            r#"
            SELECT 
                COUNT(*) as total_sessions,
                COUNT(CASE WHEN is_active = true AND expires_at > NOW() THEN 1 END) as active_sessions,
                COUNT(DISTINCT user_id) as unique_users,
                AVG(EXTRACT(EPOCH FROM (expires_at - created_at))) as avg_session_duration
            FROM user_sessions
            WHERE created_at > NOW() - INTERVAL '30 days'
            "#
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(SessionStatistics {
            total_sessions: stats.total_sessions.unwrap_or(0) as u32,
            active_sessions: stats.active_sessions.unwrap_or(0) as u32,
            unique_users: stats.unique_users.unwrap_or(0) as u32,
            average_session_duration: stats.avg_session_duration.unwrap_or(0.0) as u64,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStatistics {
    pub total_sessions: u32,
    pub active_sessions: u32,
    pub unique_users: u32,
    pub average_session_duration: u64, // in seconds
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup_test_db() -> PgPool {
        let database_url = std::env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost:5432/raffle_platform_test".to_string());
        
        sqlx::postgres::PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to connect to test database")
    }

    #[tokio::test]
    async fn test_session_creation() {
        let pool = Arc::new(setup_test_db().await);
        let config = SessionConfig::default();
        let session_manager = SessionManager::new(pool, config);

        let user_id = Uuid::new_v4();
        let refresh_token_hash = "test_hash".to_string();
        let ip_address = Some("127.0.0.1".parse().unwrap());
        let user_agent = Some("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36".to_string());

        let session = session_manager
            .create_session(user_id, refresh_token_hash, ip_address, user_agent)
            .await
            .expect("Failed to create session");

        assert_eq!(session.user_id, user_id);
        assert!(session.is_active);
    }

    #[tokio::test]
    async fn test_device_info_parsing() {
        let pool = Arc::new(setup_test_db().await);
        let config = SessionConfig::default();
        let session_manager = SessionManager::new(pool, config);

        let user_agent = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36";
        let device_info = session_manager.parse_device_info(user_agent);

        assert_eq!(device_info.device_type, "desktop");
        assert_eq!(device_info.browser, Some("Chrome".to_string()));
        assert_eq!(device_info.os, Some("Windows".to_string()));
    }
}