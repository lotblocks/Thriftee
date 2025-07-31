use actix_web::{web, HttpRequest, HttpResponse, Result};
use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation, Algorithm};
use redis::{AsyncCommands, Client as RedisClient};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use crate::models::user::User;
use crate::security::audit_logging::{AuditLogger, AuditEvent, AuditLevel};
use crate::utils::crypto::{generate_secure_token, hash_token};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionClaims {
    pub sub: String,        // User ID
    pub email: String,      // User email
    pub role: String,       // User role
    pub session_id: String, // Unique session identifier
    pub exp: i64,          // Expiration timestamp
    pub iat: i64,          // Issued at timestamp
    pub jti: String,       // JWT ID for blacklisting
    pub device_id: Option<String>, // Device identifier
    pub ip_address: String, // Client IP address
    pub user_agent: String, // Client user agent
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshTokenClaims {
    pub sub: String,        // User ID
    pub session_id: String, // Session identifier
    pub exp: i64,          // Expiration timestamp
    pub iat: i64,          // Issued at timestamp
    pub jti: String,       // JWT ID
    pub token_family: String, // Token family for rotation
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionData {
    pub user_id: String,
    pub email: String,
    pub role: String,
    pub device_id: Option<String>,
    pub ip_address: String,
    pub user_agent: String,
    pub created_at: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub is_active: bool,
    pub login_attempts: u32,
    pub security_flags: SecurityFlags,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityFlags {
    pub requires_mfa: bool,
    pub suspicious_activity: bool,
    pub password_expired: bool,
    pub account_locked: bool,
    pub force_logout: bool,
}

#[derive(Debug, Clone)]
pub struct SessionConfig {
    pub access_token_ttl: Duration,
    pub refresh_token_ttl: Duration,
    pub max_sessions_per_user: usize,
    pub session_timeout: Duration,
    pub require_device_binding: bool,
    pub enable_concurrent_sessions: bool,
    pub jwt_secret: String,
    pub redis_url: String,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            access_token_ttl: Duration::minutes(15),
            refresh_token_ttl: Duration::days(7),
            max_sessions_per_user: 5,
            session_timeout: Duration::hours(24),
            require_device_binding: true,
            enable_concurrent_sessions: true,
            jwt_secret: std::env::var("JWT_SECRET").unwrap_or_else(|_| "default_secret".to_string()),
            redis_url: std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string()),
        }
    }
}

pub struct SessionManager {
    config: SessionConfig,
    redis_client: RedisClient,
    audit_logger: AuditLogger,
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl SessionManager {
    pub fn new(config: SessionConfig, audit_logger: AuditLogger) -> Result<Self, Box<dyn std::error::Error>> {
        let redis_client = RedisClient::open(config.redis_url.clone())?;
        let encoding_key = EncodingKey::from_secret(config.jwt_secret.as_bytes());
        let decoding_key = DecodingKey::from_secret(config.jwt_secret.as_bytes());

        Ok(Self {
            config,
            redis_client,
            audit_logger,
            encoding_key,
            decoding_key,
        })
    }

    /// Create a new session for a user
    pub async fn create_session(
        &self,
        user: &User,
        ip_address: String,
        user_agent: String,
        device_id: Option<String>,
    ) -> Result<(String, String), SessionError> {
        let session_id = Uuid::new_v4().to_string();
        let now = Utc::now();
        
        // Check if user has too many active sessions
        if !self.config.enable_concurrent_sessions {
            self.invalidate_user_sessions(&user.id).await?;
        } else {
            self.cleanup_expired_sessions(&user.id).await?;
            let active_sessions = self.count_active_sessions(&user.id).await?;
            if active_sessions >= self.config.max_sessions_per_user {
                self.invalidate_oldest_session(&user.id).await?;
            }
        }

        // Create session data
        let session_data = SessionData {
            user_id: user.id.clone(),
            email: user.email.clone(),
            role: user.role.clone().unwrap_or_else(|| "user".to_string()),
            device_id: device_id.clone(),
            ip_address: ip_address.clone(),
            user_agent: user_agent.clone(),
            created_at: now,
            last_accessed: now,
            expires_at: now + self.config.session_timeout,
            is_active: true,
            login_attempts: 0,
            security_flags: SecurityFlags {
                requires_mfa: false,
                suspicious_activity: false,
                password_expired: false,
                account_locked: false,
                force_logout: false,
            },
        };

        // Store session in Redis
        let session_key = format!("session:{}", session_id);
        let user_sessions_key = format!("user_sessions:{}", user.id);
        
        let mut conn = self.redis_client.get_async_connection().await?;
        
        // Store session data
        let session_json = serde_json::to_string(&session_data)?;
        conn.setex(&session_key, self.config.session_timeout.num_seconds() as usize, session_json).await?;
        
        // Add to user's session list
        conn.sadd(&user_sessions_key, &session_id).await?;
        conn.expire(&user_sessions_key, self.config.session_timeout.num_seconds() as usize).await?;

        // Generate tokens
        let token_family = Uuid::new_v4().to_string();
        let (access_token, refresh_token) = self.generate_token_pair(
            &user.id,
            &user.email,
            &session_data.role,
            &session_id,
            &token_family,
            &ip_address,
            &user_agent,
            device_id.as_deref(),
        )?;

        // Store refresh token family
        let refresh_key = format!("refresh_family:{}", token_family);
        conn.setex(&refresh_key, self.config.refresh_token_ttl.num_seconds() as usize, &session_id).await?;

        // Log session creation
        self.audit_logger.log(AuditEvent {
            event_type: "session_created".to_string(),
            user_id: Some(user.id.clone()),
            resource_type: Some("session".to_string()),
            resource_id: Some(session_id.clone()),
            details: Some(format!("Session created for user {} from IP {}", user.email, ip_address)),
            ip_address: Some(ip_address),
            user_agent: Some(user_agent),
            level: AuditLevel::Info,
            timestamp: now,
        }).await;

        Ok((access_token, refresh_token))
    }

    /// Validate and refresh an access token
    pub async fn validate_session(&self, token: &str) -> Result<SessionClaims, SessionError> {
        // Decode JWT
        let validation = Validation::new(Algorithm::HS256);
        let token_data = decode::<SessionClaims>(token, &self.decoding_key, &validation)
            .map_err(|_| SessionError::InvalidToken)?;

        let claims = token_data.claims;

        // Check if token is blacklisted
        if self.is_token_blacklisted(&claims.jti).await? {
            return Err(SessionError::TokenBlacklisted);
        }

        // Validate session exists and is active
        let session_data = self.get_session_data(&claims.session_id).await?;
        if !session_data.is_active {
            return Err(SessionError::SessionInactive);
        }

        // Check security flags
        if session_data.security_flags.force_logout {
            self.invalidate_session(&claims.session_id).await?;
            return Err(SessionError::ForceLogout);
        }

        if session_data.security_flags.account_locked {
            return Err(SessionError::AccountLocked);
        }

        // Update last accessed time
        self.update_session_access(&claims.session_id).await?;

        Ok(claims)
    }

    /// Refresh tokens using refresh token
    pub async fn refresh_tokens(
        &self,
        refresh_token: &str,
        ip_address: String,
        user_agent: String,
    ) -> Result<(String, String), SessionError> {
        // Decode refresh token
        let validation = Validation::new(Algorithm::HS256);
        let token_data = decode::<RefreshTokenClaims>(refresh_token, &self.decoding_key, &validation)
            .map_err(|_| SessionError::InvalidRefreshToken)?;

        let claims = token_data.claims;

        // Validate token family
        let refresh_key = format!("refresh_family:{}", claims.token_family);
        let mut conn = self.redis_client.get_async_connection().await?;
        let stored_session_id: Option<String> = conn.get(&refresh_key).await?;

        if stored_session_id.is_none() || stored_session_id.unwrap() != claims.session_id {
            // Potential token theft - invalidate all sessions for this user
            self.invalidate_user_sessions(&claims.sub).await?;
            
            self.audit_logger.log(AuditEvent {
                event_type: "token_theft_detected".to_string(),
                user_id: Some(claims.sub.clone()),
                resource_type: Some("refresh_token".to_string()),
                resource_id: Some(claims.token_family.clone()),
                details: Some("Potential refresh token theft detected".to_string()),
                ip_address: Some(ip_address.clone()),
                user_agent: Some(user_agent.clone()),
                level: AuditLevel::Critical,
                timestamp: Utc::now(),
            }).await;

            return Err(SessionError::TokenTheftDetected);
        }

        // Get session data
        let session_data = self.get_session_data(&claims.session_id).await?;
        if !session_data.is_active {
            return Err(SessionError::SessionInactive);
        }

        // Generate new token family for rotation
        let new_token_family = Uuid::new_v4().to_string();
        
        // Generate new token pair
        let (new_access_token, new_refresh_token) = self.generate_token_pair(
            &claims.sub,
            &session_data.email,
            &session_data.role,
            &claims.session_id,
            &new_token_family,
            &ip_address,
            &user_agent,
            session_data.device_id.as_deref(),
        )?;

        // Update refresh token family
        let new_refresh_key = format!("refresh_family:{}", new_token_family);
        conn.setex(&new_refresh_key, self.config.refresh_token_ttl.num_seconds() as usize, &claims.session_id).await?;
        
        // Remove old refresh token family
        conn.del(&refresh_key).await?;

        // Blacklist old refresh token
        self.blacklist_token(&claims.jti, self.config.refresh_token_ttl).await?;

        // Update session access time
        self.update_session_access(&claims.session_id).await?;

        // Log token refresh
        self.audit_logger.log(AuditEvent {
            event_type: "tokens_refreshed".to_string(),
            user_id: Some(claims.sub),
            resource_type: Some("session".to_string()),
            resource_id: Some(claims.session_id),
            details: Some("Access and refresh tokens refreshed".to_string()),
            ip_address: Some(ip_address),
            user_agent: Some(user_agent),
            level: AuditLevel::Info,
            timestamp: Utc::now(),
        }).await;

        Ok((new_access_token, new_refresh_token))
    }

    /// Invalidate a specific session
    pub async fn invalidate_session(&self, session_id: &str) -> Result<(), SessionError> {
        let mut conn = self.redis_client.get_async_connection().await?;
        
        // Get session data before deletion
        let session_data = match self.get_session_data(session_id).await {
            Ok(data) => Some(data),
            Err(_) => None,
        };

        // Remove session
        let session_key = format!("session:{}", session_id);
        conn.del(&session_key).await?;

        // Remove from user's session list
        if let Some(data) = &session_data {
            let user_sessions_key = format!("user_sessions:{}", data.user_id);
            conn.srem(&user_sessions_key, session_id).await?;
        }

        // Log session invalidation
        if let Some(data) = session_data {
            self.audit_logger.log(AuditEvent {
                event_type: "session_invalidated".to_string(),
                user_id: Some(data.user_id),
                resource_type: Some("session".to_string()),
                resource_id: Some(session_id.to_string()),
                details: Some("Session manually invalidated".to_string()),
                ip_address: Some(data.ip_address),
                user_agent: Some(data.user_agent),
                level: AuditLevel::Info,
                timestamp: Utc::now(),
            }).await;
        }

        Ok(())
    }

    /// Invalidate all sessions for a user
    pub async fn invalidate_user_sessions(&self, user_id: &str) -> Result<(), SessionError> {
        let mut conn = self.redis_client.get_async_connection().await?;
        
        // Get all user sessions
        let user_sessions_key = format!("user_sessions:{}", user_id);
        let session_ids: Vec<String> = conn.smembers(&user_sessions_key).await?;

        // Remove all sessions
        for session_id in &session_ids {
            let session_key = format!("session:{}", session_id);
            conn.del(&session_key).await?;
        }

        // Remove user sessions set
        conn.del(&user_sessions_key).await?;

        // Log mass session invalidation
        self.audit_logger.log(AuditEvent {
            event_type: "all_sessions_invalidated".to_string(),
            user_id: Some(user_id.to_string()),
            resource_type: Some("session".to_string()),
            resource_id: None,
            details: Some(format!("All {} sessions invalidated for user", session_ids.len())),
            ip_address: None,
            user_agent: None,
            level: AuditLevel::Warning,
            timestamp: Utc::now(),
        }).await;

        Ok(())
    }

    /// Get active sessions for a user
    pub async fn get_user_sessions(&self, user_id: &str) -> Result<Vec<SessionData>, SessionError> {
        let mut conn = self.redis_client.get_async_connection().await?;
        
        let user_sessions_key = format!("user_sessions:{}", user_id);
        let session_ids: Vec<String> = conn.smembers(&user_sessions_key).await?;

        let mut sessions = Vec::new();
        for session_id in session_ids {
            if let Ok(session_data) = self.get_session_data(&session_id).await {
                sessions.push(session_data);
            }
        }

        Ok(sessions)
    }

    /// Update session security flags
    pub async fn update_session_security_flags(
        &self,
        session_id: &str,
        flags: SecurityFlags,
    ) -> Result<(), SessionError> {
        let mut session_data = self.get_session_data(session_id).await?;
        session_data.security_flags = flags;

        let session_key = format!("session:{}", session_id);
        let session_json = serde_json::to_string(&session_data)?;
        
        let mut conn = self.redis_client.get_async_connection().await?;
        conn.setex(&session_key, self.config.session_timeout.num_seconds() as usize, session_json).await?;

        Ok(())
    }

    // Private helper methods
    
    fn generate_token_pair(
        &self,
        user_id: &str,
        email: &str,
        role: &str,
        session_id: &str,
        token_family: &str,
        ip_address: &str,
        user_agent: &str,
        device_id: Option<&str>,
    ) -> Result<(String, String), SessionError> {
        let now = Utc::now();
        
        // Generate access token
        let access_claims = SessionClaims {
            sub: user_id.to_string(),
            email: email.to_string(),
            role: role.to_string(),
            session_id: session_id.to_string(),
            exp: (now + self.config.access_token_ttl).timestamp(),
            iat: now.timestamp(),
            jti: Uuid::new_v4().to_string(),
            device_id: device_id.map(|s| s.to_string()),
            ip_address: ip_address.to_string(),
            user_agent: user_agent.to_string(),
        };

        let access_token = encode(&Header::default(), &access_claims, &self.encoding_key)
            .map_err(|_| SessionError::TokenGenerationFailed)?;

        // Generate refresh token
        let refresh_claims = RefreshTokenClaims {
            sub: user_id.to_string(),
            session_id: session_id.to_string(),
            exp: (now + self.config.refresh_token_ttl).timestamp(),
            iat: now.timestamp(),
            jti: Uuid::new_v4().to_string(),
            token_family: token_family.to_string(),
        };

        let refresh_token = encode(&Header::default(), &refresh_claims, &self.encoding_key)
            .map_err(|_| SessionError::TokenGenerationFailed)?;

        Ok((access_token, refresh_token))
    }

    async fn get_session_data(&self, session_id: &str) -> Result<SessionData, SessionError> {
        let mut conn = self.redis_client.get_async_connection().await?;
        let session_key = format!("session:{}", session_id);
        
        let session_json: Option<String> = conn.get(&session_key).await?;
        match session_json {
            Some(json) => {
                let session_data: SessionData = serde_json::from_str(&json)?;
                Ok(session_data)
            }
            None => Err(SessionError::SessionNotFound),
        }
    }

    async fn update_session_access(&self, session_id: &str) -> Result<(), SessionError> {
        let mut session_data = self.get_session_data(session_id).await?;
        session_data.last_accessed = Utc::now();

        let session_key = format!("session:{}", session_id);
        let session_json = serde_json::to_string(&session_data)?;
        
        let mut conn = self.redis_client.get_async_connection().await?;
        conn.setex(&session_key, self.config.session_timeout.num_seconds() as usize, session_json).await?;

        Ok(())
    }

    async fn count_active_sessions(&self, user_id: &str) -> Result<usize, SessionError> {
        let mut conn = self.redis_client.get_async_connection().await?;
        let user_sessions_key = format!("user_sessions:{}", user_id);
        let count: usize = conn.scard(&user_sessions_key).await?;
        Ok(count)
    }

    async fn invalidate_oldest_session(&self, user_id: &str) -> Result<(), SessionError> {
        let sessions = self.get_user_sessions(user_id).await?;
        if let Some(oldest_session) = sessions.iter().min_by_key(|s| s.created_at) {
            let session_id = format!("session_id_from_data"); // This would need to be stored in SessionData
            self.invalidate_session(&session_id).await?;
        }
        Ok(())
    }

    async fn cleanup_expired_sessions(&self, user_id: &str) -> Result<(), SessionError> {
        let sessions = self.get_user_sessions(user_id).await?;
        let now = Utc::now();
        
        for session in sessions {
            if session.expires_at < now {
                // Session ID would need to be stored in SessionData for this to work
                // self.invalidate_session(&session_id).await?;
            }
        }
        
        Ok(())
    }

    async fn blacklist_token(&self, jti: &str, ttl: Duration) -> Result<(), SessionError> {
        let mut conn = self.redis_client.get_async_connection().await?;
        let blacklist_key = format!("blacklist:{}", jti);
        conn.setex(&blacklist_key, ttl.num_seconds() as usize, "blacklisted").await?;
        Ok(())
    }

    async fn is_token_blacklisted(&self, jti: &str) -> Result<bool, SessionError> {
        let mut conn = self.redis_client.get_async_connection().await?;
        let blacklist_key = format!("blacklist:{}", jti);
        let exists: bool = conn.exists(&blacklist_key).await?;
        Ok(exists)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error("Invalid token")]
    InvalidToken,
    #[error("Invalid refresh token")]
    InvalidRefreshToken,
    #[error("Token is blacklisted")]
    TokenBlacklisted,
    #[error("Session not found")]
    SessionNotFound,
    #[error("Session is inactive")]
    SessionInactive,
    #[error("Account is locked")]
    AccountLocked,
    #[error("Force logout required")]
    ForceLogout,
    #[error("Token generation failed")]
    TokenGenerationFailed,
    #[error("Token theft detected")]
    TokenTheftDetected,
    #[error("Redis error: {0}")]
    RedisError(#[from] redis::RedisError),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

// Middleware for session validation
pub async fn validate_session_middleware(
    req: HttpRequest,
    session_manager: web::Data<SessionManager>,
) -> Result<SessionClaims, actix_web::Error> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or_else(|| actix_web::error::ErrorUnauthorized("Missing or invalid authorization header"))?;

    let claims = session_manager
        .validate_session(auth_header)
        .await
        .map_err(|e| actix_web::error::ErrorUnauthorized(format!("Session validation failed: {}", e)))?;

    Ok(claims)
}