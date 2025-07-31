use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use raffle_platform_shared::{UserRole, JWT_ACCESS_TOKEN_EXPIRY, JWT_REFRESH_TOKEN_EXPIRY};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::env;
use std::sync::{Arc, RwLock};
use uuid::Uuid;

use crate::error::AppError;

#[cfg(test)]
mod tests;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,        // Subject (user ID)
    pub username: String,   // Username
    pub email: String,      // Email
    pub role: UserRole,     // User role
    pub exp: i64,          // Expiration time
    pub iat: i64,          // Issued at
    pub jti: String,       // JWT ID (for token revocation)
    pub token_type: String, // "access" or "refresh"
}

#[derive(Debug, Clone)]
pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    validation: Validation,
    revoked_tokens: Arc<RwLock<HashSet<String>>>, // In production, use Redis
}

impl JwtService {
    pub fn new() -> Result<Self, AppError> {
        let secret = env::var("JWT_SECRET")
            .map_err(|_| AppError::Internal("JWT_SECRET environment variable not set".to_string()))?;

        // Validate secret strength
        if secret.len() < 32 {
            return Err(AppError::Internal("JWT_SECRET must be at least 32 characters long".to_string()));
        }

        let encoding_key = EncodingKey::from_secret(secret.as_bytes());
        let decoding_key = DecodingKey::from_secret(secret.as_bytes());
        
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_required_spec_claims(&["exp", "sub", "iat", "jti"]);
        validation.validate_exp = true;
        validation.validate_nbf = false;
        validation.leeway = 30; // 30 seconds leeway for clock skew

        Ok(Self {
            encoding_key,
            decoding_key,
            validation,
            revoked_tokens: Arc::new(RwLock::new(HashSet::new())),
        })
    }

    /// Generate an access token
    pub fn generate_access_token(
        &self,
        user_id: Uuid,
        username: String,
        email: String,
        role: UserRole,
    ) -> Result<String, AppError> {
        let now = Utc::now();
        let exp = now + Duration::from_std(JWT_ACCESS_TOKEN_EXPIRY)
            .map_err(|_| AppError::Internal("Invalid token expiry duration".to_string()))?;

        let claims = Claims {
            sub: user_id.to_string(),
            username,
            email,
            role,
            exp: exp.timestamp(),
            iat: now.timestamp(),
            jti: Uuid::new_v4().to_string(),
            token_type: "access".to_string(),
        };

        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| AppError::Internal(format!("Failed to encode JWT: {}", e)))
    }

    /// Generate a refresh token
    pub fn generate_refresh_token(
        &self,
        user_id: Uuid,
        username: String,
        email: String,
        role: UserRole,
    ) -> Result<String, AppError> {
        let now = Utc::now();
        let exp = now + Duration::from_std(JWT_REFRESH_TOKEN_EXPIRY)
            .map_err(|_| AppError::Internal("Invalid token expiry duration".to_string()))?;

        let claims = Claims {
            sub: user_id.to_string(),
            username,
            email,
            role,
            exp: exp.timestamp(),
            iat: now.timestamp(),
            jti: Uuid::new_v4().to_string(),
            token_type: "refresh".to_string(),
        };

        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| AppError::Internal(format!("Failed to encode JWT: {}", e)))
    }

    /// Validate and decode a token
    pub fn validate_token(&self, token: &str) -> Result<Claims, AppError> {
        // First decode to get the JTI for revocation check
        let token_data = decode::<Claims>(token, &self.decoding_key, &self.validation)
            .map_err(|e| match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                    AppError::Authentication("Token has expired".to_string())
                }
                jsonwebtoken::errors::ErrorKind::InvalidToken => {
                    AppError::Authentication("Invalid token".to_string())
                }
                jsonwebtoken::errors::ErrorKind::InvalidSignature => {
                    AppError::Authentication("Invalid token signature".to_string())
                }
                _ => AppError::Authentication(format!("Token validation failed: {}", e)),
            })?;

        // Check if token is revoked
        if self.is_token_revoked(&token_data.claims.jti) {
            return Err(AppError::Authentication("Token has been revoked".to_string()));
        }

        Ok(token_data.claims)
    }

    /// Revoke a token by its JTI
    pub fn revoke_token(&self, jti: &str) -> Result<(), AppError> {
        let mut revoked_tokens = self.revoked_tokens.write()
            .map_err(|_| AppError::Internal("Failed to acquire write lock".to_string()))?;
        revoked_tokens.insert(jti.to_string());
        Ok(())
    }

    /// Check if a token is revoked
    pub fn is_token_revoked(&self, jti: &str) -> bool {
        self.revoked_tokens.read()
            .map(|tokens| tokens.contains(jti))
            .unwrap_or(false)
    }

    /// Revoke all tokens for a user (by extracting JTI from active sessions)
    pub async fn revoke_user_tokens(&self, user_id: Uuid, pool: &sqlx::PgPool) -> Result<(), AppError> {
        // Get all active sessions for the user
        let sessions = sqlx::query!(
            "SELECT refresh_token_hash FROM user_sessions WHERE user_id = $1 AND is_active = true",
            user_id
        )
        .fetch_all(pool)
        .await?;

        // Revoke all tokens (in a real implementation, you'd extract JTI from each token)
        let mut revoked_tokens = self.revoked_tokens.write()
            .map_err(|_| AppError::Internal("Failed to acquire write lock".to_string()))?;
        
        for session in sessions {
            // In practice, you'd decode the refresh token to get the JTI
            // For now, we'll use the hash as a placeholder
            revoked_tokens.insert(session.refresh_token_hash);
        }

        // Deactivate all sessions
        sqlx::query!(
            "UPDATE user_sessions SET is_active = false WHERE user_id = $1",
            user_id
        )
        .execute(pool)
        .await?;

        tracing::info!("Revoked all tokens for user: {}", user_id);
        Ok(())
    }

    /// Clean up expired revoked tokens (should be called periodically)
    pub fn cleanup_revoked_tokens(&self) -> Result<usize, AppError> {
        let mut revoked_tokens = self.revoked_tokens.write()
            .map_err(|_| AppError::Internal("Failed to acquire write lock".to_string()))?;
        
        let initial_count = revoked_tokens.len();
        
        // In a real implementation, you'd check which tokens have expired
        // and remove them from the revoked list. For now, we'll keep all.
        // This is because we don't store the expiration time with the JTI.
        
        Ok(initial_count - revoked_tokens.len())
    }

    /// Extract user ID from token without full validation (for expired tokens)
    pub fn extract_user_id_unsafe(&self, token: &str) -> Result<Uuid, AppError> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = false; // Don't validate expiration
        validation.validate_nbf = false;
        validation.validate_aud = false;

        let token_data = decode::<Claims>(token, &self.decoding_key, &validation)
            .map_err(|_| AppError::Authentication("Invalid token format".to_string()))?;

        Uuid::parse_str(&token_data.claims.sub)
            .map_err(|_| AppError::Authentication("Invalid user ID in token".to_string()))
    }

    /// Check if token is access token
    pub fn is_access_token(&self, token: &str) -> Result<bool, AppError> {
        let claims = self.validate_token(token)?;
        Ok(claims.token_type == "access")
    }

    /// Check if token is refresh token
    pub fn is_refresh_token(&self, token: &str) -> Result<bool, AppError> {
        let claims = self.validate_token(token)?;
        Ok(claims.token_type == "refresh")
    }

    /// Get token expiration time
    pub fn get_token_expiration(&self, token: &str) -> Result<chrono::DateTime<Utc>, AppError> {
        let claims = self.validate_token(token)?;
        chrono::DateTime::from_timestamp(claims.exp, 0)
            .ok_or_else(|| AppError::Internal("Invalid expiration timestamp".to_string()))
    }

    /// Get JWT ID from token
    pub fn get_jwt_id(&self, token: &str) -> Result<String, AppError> {
        let claims = self.validate_token(token)?;
        Ok(claims.jti)
    }

    /// Create token pair (access + refresh)
    pub fn create_token_pair(
        &self,
        user_id: Uuid,
        username: String,
        email: String,
        role: UserRole,
    ) -> Result<TokenPair, AppError> {
        let access_token = self.generate_access_token(user_id, username.clone(), email.clone(), role)?;
        let refresh_token = self.generate_refresh_token(user_id, username, email, role)?;

        Ok(TokenPair {
            access_token,
            refresh_token,
            expires_in: JWT_ACCESS_TOKEN_EXPIRY.as_secs() as i64,
        })
    }

    /// Validate token and return user info without checking revocation (for logout)
    pub fn validate_token_unsafe(&self, token: &str) -> Result<Claims, AppError> {
        decode::<Claims>(token, &self.decoding_key, &self.validation)
            .map(|data| data.claims)
            .map_err(|e| AppError::Authentication(format!("Token validation failed: {}", e)))
    }

    /// Get remaining time until token expires
    pub fn get_token_remaining_time(&self, token: &str) -> Result<Duration, AppError> {
        let claims = self.validate_token(token)?;
        let exp_time = chrono::DateTime::from_timestamp(claims.exp, 0)
            .ok_or_else(|| AppError::Internal("Invalid expiration timestamp".to_string()))?;
        let now = Utc::now();
        
        if exp_time > now {
            Ok(exp_time - now)
        } else {
            Err(AppError::Authentication("Token has expired".to_string()))
        }
    }

    /// Check if token will expire within the given duration
    pub fn will_expire_within(&self, token: &str, duration: Duration) -> Result<bool, AppError> {
        let remaining = self.get_token_remaining_time(token)?;
        Ok(remaining < duration)
    }

    /// Generate a secure token for password reset, email verification, etc.
    pub fn generate_secure_token(&self) -> String {
        crate::utils::crypto::generate_secure_token()
    }

    /// Validate token format without signature verification (for debugging)
    pub fn decode_token_unsafe(&self, token: &str) -> Result<Claims, AppError> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = false;
        validation.validate_nbf = false;
        validation.validate_aud = false;
        validation.insecure_disable_signature_validation();

        decode::<Claims>(token, &self.decoding_key, &validation)
            .map(|data| data.claims)
            .map_err(|e| AppError::Authentication(format!("Token decode failed: {}", e)))
    }

    /// Validate token with additional security checks
    pub async fn validate_token_with_security_checks(
        &self,
        token: &str,
        pool: &sqlx::PgPool,
        ip_address: Option<std::net::IpAddr>,
        user_agent: Option<&str>,
    ) -> Result<Claims, AppError> {
        // First, validate the token normally
        let claims = self.validate_token(token)?;

        // Additional security checks
        self.perform_security_checks(&claims, pool, ip_address, user_agent).await?;

        Ok(claims)
    }

    /// Perform additional security checks on token
    async fn perform_security_checks(
        &self,
        claims: &Claims,
        pool: &sqlx::PgPool,
        ip_address: Option<std::net::IpAddr>,
        user_agent: Option<&str>,
    ) -> Result<(), AppError> {
        let user_id = uuid::Uuid::parse_str(&claims.sub)
            .map_err(|_| AppError::Authentication("Invalid user ID in token".to_string()))?;

        // Check if user is still active
        let user_active = sqlx::query_scalar!(
            "SELECT is_active FROM users WHERE id = $1",
            user_id
        )
        .fetch_optional(pool)
        .await?
        .unwrap_or(false);

        if !user_active {
            return Err(AppError::Authentication("User account is deactivated".to_string()));
        }

        // For refresh tokens, check if the session is still valid
        if claims.token_type == "refresh" {
            let session_valid = sqlx::query_scalar!(
                "SELECT COUNT(*) FROM user_sessions WHERE user_id = $1 AND is_active = true AND expires_at > NOW()",
                user_id
            )
            .fetch_one(pool)
            .await?
            .unwrap_or(0) > 0;

            if !session_valid {
                return Err(AppError::Authentication("Session has expired or been revoked".to_string()));
            }
        }

        // Log suspicious activity (optional)
        if let (Some(ip), Some(ua)) = (ip_address, user_agent) {
            self.log_token_usage(user_id, ip, ua, &claims.token_type).await;
        }

        Ok(())
    }

    /// Log token usage for security monitoring
    async fn log_token_usage(
        &self,
        user_id: uuid::Uuid,
        ip_address: std::net::IpAddr,
        user_agent: &str,
        token_type: &str,
    ) {
        tracing::debug!(
            "Token usage: user_id={}, ip={}, user_agent={}, token_type={}",
            user_id,
            ip_address,
            user_agent,
            token_type
        );
    }

    /// Refresh access token using refresh token
    pub async fn refresh_access_token(
        &self,
        refresh_token: &str,
        pool: &sqlx::PgPool,
    ) -> Result<TokenPair, AppError> {
        // Validate refresh token
        let claims = self.validate_token(refresh_token)?;

        if claims.token_type != "refresh" {
            return Err(AppError::Authentication("Invalid token type for refresh".to_string()));
        }

        let user_id = uuid::Uuid::parse_str(&claims.sub)
            .map_err(|_| AppError::Authentication("Invalid user ID in token".to_string()))?;

        // Verify the refresh token exists in the database
        let session_exists = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM user_sessions WHERE user_id = $1 AND refresh_token_hash = $2 AND is_active = true AND expires_at > NOW()",
            user_id,
            crate::utils::crypto::hash_token(refresh_token)
        )
        .fetch_one(pool)
        .await?
        .unwrap_or(0) > 0;

        if !session_exists {
            return Err(AppError::Authentication("Invalid or expired refresh token".to_string()));
        }

        // Generate new token pair
        let new_token_pair = self.create_token_pair(
            user_id,
            claims.username,
            claims.email,
            claims.role,
        )?;

        // Update the session with new refresh token hash
        let new_refresh_hash = crate::utils::crypto::hash_token(&new_token_pair.refresh_token);
        sqlx::query!(
            "UPDATE user_sessions SET refresh_token_hash = $1, updated_at = NOW() WHERE user_id = $2 AND refresh_token_hash = $3",
            new_refresh_hash,
            user_id,
            crate::utils::crypto::hash_token(refresh_token)
        )
        .execute(pool)
        .await?;

        Ok(new_token_pair)
    }

    /// Validate token and check for suspicious patterns
    pub fn validate_token_with_anomaly_detection(&self, token: &str) -> Result<(Claims, Vec<SecurityAlert>), AppError> {
        let claims = self.validate_token(token)?;
        let mut alerts = Vec::new();

        // Check for unusual token patterns
        if let Ok(remaining_time) = self.get_token_remaining_time(token) {
            // Alert if token is being used very close to expiration
            if remaining_time < chrono::Duration::minutes(5) {
                alerts.push(SecurityAlert {
                    alert_type: SecurityAlertType::TokenNearExpiry,
                    message: "Token is close to expiration".to_string(),
                    severity: AlertSeverity::Low,
                });
            }
        }

        // Check for rapid token usage (would need additional state tracking)
        // This is a placeholder for more sophisticated anomaly detection

        Ok((claims, alerts))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAlert {
    pub alert_type: SecurityAlertType,
    pub message: String,
    pub severity: AlertSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityAlertType {
    TokenNearExpiry,
    SuspiciousActivity,
    UnusualLocation,
    RapidTokenUsage,
    InvalidTokenPattern,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn setup_jwt_service() -> JwtService {
        env::set_var("JWT_SECRET", "test-secret-key-for-testing-only");
        JwtService::new().expect("Failed to create JWT service")
    }

    #[test]
    fn test_token_generation_and_validation() {
        let jwt_service = setup_jwt_service();
        let user_id = Uuid::new_v4();
        let username = "testuser".to_string();
        let email = "test@example.com".to_string();
        let role = UserRole::User;

        // Generate access token
        let access_token = jwt_service
            .generate_access_token(user_id, username.clone(), email.clone(), role)
            .expect("Failed to generate access token");

        // Validate access token
        let claims = jwt_service
            .validate_token(&access_token)
            .expect("Failed to validate access token");

        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.username, username);
        assert_eq!(claims.email, email);
        assert_eq!(claims.role, role);
        assert_eq!(claims.token_type, "access");

        // Generate refresh token
        let refresh_token = jwt_service
            .generate_refresh_token(user_id, username.clone(), email.clone(), role)
            .expect("Failed to generate refresh token");

        // Validate refresh token
        let refresh_claims = jwt_service
            .validate_token(&refresh_token)
            .expect("Failed to validate refresh token");

        assert_eq!(refresh_claims.token_type, "refresh");
    }

    #[test]
    fn test_token_pair_creation() {
        let jwt_service = setup_jwt_service();
        let user_id = Uuid::new_v4();
        let username = "testuser".to_string();
        let email = "test@example.com".to_string();
        let role = UserRole::User;

        let token_pair = jwt_service
            .create_token_pair(user_id, username, email, role)
            .expect("Failed to create token pair");

        assert!(!token_pair.access_token.is_empty());
        assert!(!token_pair.refresh_token.is_empty());
        assert!(token_pair.expires_in > 0);

        // Validate both tokens
        assert!(jwt_service.is_access_token(&token_pair.access_token).unwrap());
        assert!(jwt_service.is_refresh_token(&token_pair.refresh_token).unwrap());
    }

    #[test]
    fn test_invalid_token() {
        let jwt_service = setup_jwt_service();
        let invalid_token = "invalid.token.here";

        let result = jwt_service.validate_token(invalid_token);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_user_id_unsafe() {
        let jwt_service = setup_jwt_service();
        let user_id = Uuid::new_v4();
        let username = "testuser".to_string();
        let email = "test@example.com".to_string();
        let role = UserRole::User;

        let token = jwt_service
            .generate_access_token(user_id, username, email, role)
            .expect("Failed to generate token");

        let extracted_id = jwt_service
            .extract_user_id_unsafe(&token)
            .expect("Failed to extract user ID");

        assert_eq!(extracted_id, user_id);
    }
}