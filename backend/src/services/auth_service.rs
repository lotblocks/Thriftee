use crate::error::AppError;
use crate::models::{User, UserSession, AuditLog};
use crate::repositories::UserRepository;
use crate::utils::{
    crypto::{hash_password, verify_password, hash_token, generate_secure_token, validate_password_strength},
    jwt::{JwtService, TokenPair},
    validation::{validate_email, validate_username},
};
use chrono::{Duration, Utc};
use raffle_platform_shared::{
    CreateUserRequest, LoginRequest, AuthResponse, UserRole,
    ERROR_INVALID_CREDENTIALS, ERROR_EMAIL_ALREADY_EXISTS, ERROR_USERNAME_ALREADY_EXISTS,
};
use sqlx::PgPool;
use std::net::IpAddr;
use std::sync::Arc;
use uuid::Uuid;

pub mod session_manager;
pub use session_manager::{SessionManager, SessionConfig, SessionInfo, DeviceInfo};

#[derive(Clone)]
pub struct AuthService {
    user_repository: UserRepository,
    jwt_service: Arc<JwtService>,
    session_manager: SessionManager,
    pool: Arc<PgPool>,
}

impl AuthService {
    pub fn new(pool: Arc<PgPool>, jwt_service: Arc<JwtService>) -> Self {
        let user_repository = UserRepository::new(pool.clone());
        let session_manager = SessionManager::new(pool.clone(), SessionConfig::default());
        
        Self {
            user_repository,
            jwt_service,
            session_manager,
            pool,
        }
    }

    pub fn with_session_config(pool: Arc<PgPool>, jwt_service: Arc<JwtService>, session_config: SessionConfig) -> Self {
        let user_repository = UserRepository::new(pool.clone());
        let session_manager = SessionManager::new(pool.clone(), session_config);
        
        Self {
            user_repository,
            jwt_service,
            session_manager,
            pool,
        }
    }

    /// Register a new user
    pub async fn register(
        &self,
        request: CreateUserRequest,
        ip_address: Option<IpAddr>,
        user_agent: Option<String>,
    ) -> Result<AuthResponse, AppError> {
        // Validate input
        validate_email(&request.email)?;
        validate_username(&request.username)?;
        validate_password_strength(&request.password)?;

        // Check if email already exists
        if self.user_repository.email_exists(&request.email).await? {
            return Err(AppError::Conflict(ERROR_EMAIL_ALREADY_EXISTS.to_string()));
        }

        // Check if username already exists
        if self.user_repository.username_exists(&request.username).await? {
            return Err(AppError::Conflict(ERROR_USERNAME_ALREADY_EXISTS.to_string()));
        }

        // Hash password
        let password_hash = hash_password(&request.password)?;

        // Generate wallet for user
        let wallet_service = crate::services::WalletService::new(self.pool.clone());
        let temp_user_id = Uuid::new_v4();
        let (wallet_address, encrypted_private_key, encrypted_mnemonic) = wallet_service
            .generate_wallet_for_user(temp_user_id, &request.password)
            .await?;

        // Create user
        let user = User::create(
            &self.pool,
            request.clone(),
            password_hash,
            wallet_address,
            encrypted_private_key,
            Some(encrypted_mnemonic),
        ).await?;

        // Generate tokens
        let token_pair = self.jwt_service.create_token_pair(
            user.id,
            user.username.clone(),
            user.email.clone(),
            user.role,
        )?;

        // Create session
        let refresh_token_hash = hash_token(&token_pair.refresh_token);
        let expires_at = Utc::now() + Duration::days(7);
        
        let device_info = self.create_device_info(&user_agent);
        
        self.user_repository.create_session(
            user.id,
            refresh_token_hash,
            expires_at,
            device_info,
            ip_address,
            user_agent.clone(),
        ).await?;

        // Log registration
        AuditLog::create(
            &self.pool,
            Some(user.id),
            raffle_platform_shared::AuditAction::Create,
            Some("user".to_string()),
            Some(user.id),
            None,
            Some(serde_json::json!({
                "username": user.username,
                "email": user.email,
                "registration_method": "email"
            })),
            ip_address,
            user_agent,
        ).await?;

        Ok(AuthResponse {
            access_token: token_pair.access_token,
            refresh_token: token_pair.refresh_token,
            user: user.to_response(),
            expires_in: token_pair.expires_in,
        })
    }

    /// Login user with email and password
    pub async fn login(
        &self,
        request: LoginRequest,
        ip_address: Option<IpAddr>,
        user_agent: Option<String>,
    ) -> Result<AuthResponse, AppError> {
        // Validate input
        validate_email(&request.email)?;

        // Find user by email
        let user = self.user_repository.find_by_email(&request.email)
            .await?
            .ok_or_else(|| AppError::Authentication(ERROR_INVALID_CREDENTIALS.to_string()))?;

        // Check if user is active
        if !user.is_active {
            return Err(AppError::Authentication("Account is deactivated".to_string()));
        }

        // Verify password
        let password_hash = user.password_hash
            .as_ref()
            .ok_or_else(|| AppError::Authentication("Account uses social login".to_string()))?;

        if !verify_password(&request.password, password_hash)? {
            // Log failed login attempt
            AuditLog::create(
                &self.pool,
                Some(user.id),
                raffle_platform_shared::AuditAction::SecurityEvent,
                Some("auth".to_string()),
                Some(user.id),
                None,
                Some(serde_json::json!({
                    "event": "failed_login",
                    "email": request.email,
                    "reason": "invalid_password"
                })),
                ip_address,
                user_agent.clone(),
            ).await?;

            return Err(AppError::Authentication(ERROR_INVALID_CREDENTIALS.to_string()));
        }

        // Generate tokens
        let token_pair = self.jwt_service.create_token_pair(
            user.id,
            user.username.clone(),
            user.email.clone(),
            user.role,
        )?;

        // Create session
        let refresh_token_hash = hash_token(&token_pair.refresh_token);
        let expires_at = Utc::now() + Duration::days(7);
        
        let device_info = self.create_device_info(&user_agent);
        
        self.user_repository.create_session(
            user.id,
            refresh_token_hash,
            expires_at,
            device_info,
            ip_address,
            user_agent.clone(),
        ).await?;

        // Log successful login
        AuditLog::log_user_login(&self.pool, user.id, ip_address, user_agent).await?;

        Ok(AuthResponse {
            access_token: token_pair.access_token,
            refresh_token: token_pair.refresh_token,
            user: user.to_response(),
            expires_in: token_pair.expires_in,
        })
    }

    /// Refresh access token using refresh token
    pub async fn refresh_token(
        &self,
        refresh_token: &str,
        ip_address: Option<IpAddr>,
        user_agent: Option<String>,
    ) -> Result<AuthResponse, AppError> {
        // Validate refresh token
        let claims = self.jwt_service.validate_token(refresh_token)?;
        
        if claims.token_type != "refresh" {
            return Err(AppError::Authentication("Invalid token type".to_string()));
        }

        // Check if session exists and is active
        let refresh_token_hash = hash_token(refresh_token);
        let session = self.user_repository.find_session_by_refresh_token(&refresh_token_hash)
            .await?
            .ok_or_else(|| AppError::Authentication("Invalid refresh token".to_string()))?;

        // Get user
        let user_id = Uuid::parse_str(&claims.sub)
            .map_err(|_| AppError::Authentication("Invalid user ID in token".to_string()))?;
        
        let user = self.user_repository.find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::Authentication("User not found".to_string()))?;

        // Check if user is still active
        if !user.is_active {
            return Err(AppError::Authentication("Account is deactivated".to_string()));
        }

        // Revoke old refresh token
        self.jwt_service.revoke_token(&claims.jti)?;

        // Generate new tokens
        let token_pair = self.jwt_service.create_token_pair(
            user.id,
            user.username.clone(),
            user.email.clone(),
            user.role,
        )?;

        // Deactivate old session and create new one
        self.user_repository.deactivate_session(session.id).await?;
        
        let new_refresh_token_hash = hash_token(&token_pair.refresh_token);
        let expires_at = Utc::now() + Duration::days(7);
        
        let device_info = self.create_device_info(&user_agent);
        
        self.user_repository.create_session(
            user.id,
            new_refresh_token_hash,
            expires_at,
            device_info,
            ip_address,
            user_agent,
        ).await?;

        Ok(AuthResponse {
            access_token: token_pair.access_token,
            refresh_token: token_pair.refresh_token,
            user: user.to_response(),
            expires_in: token_pair.expires_in,
        })
    }

    /// Logout user (revoke all sessions)
    pub async fn logout(
        &self,
        user_id: Uuid,
        access_token: Option<&str>,
        ip_address: Option<IpAddr>,
        user_agent: Option<String>,
    ) -> Result<(), AppError> {
        // Revoke access token if provided
        if let Some(token) = access_token {
            if let Ok(claims) = self.jwt_service.validate_token_unsafe(token) {
                self.jwt_service.revoke_token(&claims.jti)?;
            }
        }

        // Deactivate all sessions for the user
        self.user_repository.deactivate_all_sessions(user_id).await?;

        // Log logout
        AuditLog::log_user_logout(&self.pool, user_id, ip_address, user_agent).await?;

        Ok(())
    }

    /// Change user password
    pub async fn change_password(
        &self,
        user_id: Uuid,
        current_password: &str,
        new_password: &str,
        ip_address: Option<IpAddr>,
        user_agent: Option<String>,
    ) -> Result<(), AppError> {
        // Validate new password
        validate_password_strength(new_password)?;

        // Get user
        let user = self.user_repository.find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        // Verify current password
        let password_hash = user.password_hash
            .as_ref()
            .ok_or_else(|| AppError::Authentication("Account uses social login".to_string()))?;

        if !verify_password(current_password, password_hash)? {
            return Err(AppError::Authentication("Current password is incorrect".to_string()));
        }

        // Hash new password
        let new_password_hash = hash_password(new_password)?;

        // Update password
        self.user_repository.update_password(user_id, new_password_hash).await?;

        // Deactivate all sessions for security (user will need to log in again)
        self.user_repository.deactivate_all_sessions(user_id).await?;

        // Log password change
        AuditLog::log_password_change(&self.pool, user_id, ip_address, user_agent).await?;

        Ok(())
    }

    /// Initiate password reset
    pub async fn initiate_password_reset(
        &self,
        email: &str,
        ip_address: Option<IpAddr>,
        user_agent: Option<String>,
    ) -> Result<(), AppError> {
        // Always return success to prevent email enumeration
        // But only send email if user exists
        
        if let Some(user) = self.user_repository.find_by_email(email).await? {
            // Generate reset token
            let reset_token = generate_secure_token();
            let token_hash = hash_token(&reset_token);
            let expires_at = Utc::now() + Duration::hours(1); // 1 hour expiry

            // Store reset token in database
            sqlx::query!(
                r#"
                INSERT INTO password_reset_tokens (user_id, token, expires_at)
                VALUES ($1, $2, $3)
                ON CONFLICT (user_id) 
                DO UPDATE SET token = EXCLUDED.token, expires_at = EXCLUDED.expires_at, is_used = false
                "#,
                user.id,
                token_hash,
                expires_at
            )
            .execute(&*self.pool)
            .await?;

            // Log password reset request
            AuditLog::create(
                &self.pool,
                Some(user.id),
                raffle_platform_shared::AuditAction::SecurityEvent,
                Some("auth".to_string()),
                Some(user.id),
                None,
                Some(serde_json::json!({
                    "event": "password_reset_requested",
                    "email": email
                })),
                ip_address,
                user_agent,
            ).await?;

            // TODO: Send email with reset token
            // For now, we'll just log it (in production, use proper email service)
            tracing::info!("Password reset token for {}: {}", email, reset_token);
        }

        Ok(())
    }

    /// Complete password reset
    pub async fn complete_password_reset(
        &self,
        token: &str,
        new_password: &str,
        ip_address: Option<IpAddr>,
        user_agent: Option<String>,
    ) -> Result<(), AppError> {
        // Validate new password
        validate_password_strength(new_password)?;

        // Find and validate reset token
        let token_hash = hash_token(token);
        let reset_record = sqlx::query!(
            r#"
            SELECT user_id, expires_at, is_used 
            FROM password_reset_tokens 
            WHERE token = $1 AND expires_at > NOW() AND is_used = false
            "#,
            token_hash
        )
        .fetch_optional(&*self.pool)
        .await?
        .ok_or_else(|| AppError::Authentication("Invalid or expired reset token".to_string()))?;

        // Hash new password
        let password_hash = hash_password(new_password)?;

        // Update password and mark token as used
        let mut tx = self.pool.begin().await?;

        sqlx::query!(
            "UPDATE users SET password_hash = $1, updated_at = NOW() WHERE id = $2",
            password_hash,
            reset_record.user_id
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!(
            "UPDATE password_reset_tokens SET is_used = true WHERE token = $1",
            token_hash
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        // Deactivate all sessions for security
        self.user_repository.deactivate_all_sessions(reset_record.user_id).await?;

        // Log password reset completion
        AuditLog::create(
            &self.pool,
            Some(reset_record.user_id),
            raffle_platform_shared::AuditAction::SecurityEvent,
            Some("auth".to_string()),
            Some(reset_record.user_id),
            None,
            Some(serde_json::json!({
                "event": "password_reset_completed"
            })),
            ip_address,
            user_agent,
        ).await?;

        Ok(())
    }

    /// Verify email address
    pub async fn verify_email(
        &self,
        user_id: Uuid,
        token: &str,
    ) -> Result<(), AppError> {
        // Find and validate verification token
        let token_hash = hash_token(token);
        let verification_record = sqlx::query!(
            r#"
            SELECT user_id, expires_at, is_used 
            FROM email_verification_tokens 
            WHERE token = $1 AND user_id = $2 AND expires_at > NOW() AND is_used = false
            "#,
            token_hash,
            user_id
        )
        .fetch_optional(&*self.pool)
        .await?
        .ok_or_else(|| AppError::Authentication("Invalid or expired verification token".to_string()))?;

        // Update user email verification status and mark token as used
        let mut tx = self.pool.begin().await?;

        self.user_repository.verify_email(user_id).await?;

        sqlx::query!(
            "UPDATE email_verification_tokens SET is_used = true WHERE token = $1",
            token_hash
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(())
    }

    /// Get current user info
    pub async fn get_current_user(&self, user_id: Uuid) -> Result<User, AppError> {
        self.user_repository.find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))
    }

    /// Check if user has required role
    pub fn check_role(&self, user_role: UserRole, required_role: UserRole) -> Result<(), AppError> {
        if !self.has_required_role(user_role, required_role) {
            return Err(AppError::Authorization("Insufficient permissions".to_string()));
        }
        Ok(())
    }

    /// Helper method to check role hierarchy
    fn has_required_role(&self, user_role: UserRole, required_role: UserRole) -> bool {
        match required_role {
            UserRole::User => true, // All roles can access user-level endpoints
            UserRole::Seller => matches!(user_role, UserRole::Seller | UserRole::Admin | UserRole::Operator),
            UserRole::Admin => matches!(user_role, UserRole::Admin | UserRole::Operator),
            UserRole::Operator => matches!(user_role, UserRole::Operator),
        }
    }

    /// Create device info from user agent
    fn create_device_info(&self, user_agent: &Option<String>) -> Option<serde_json::Value> {
        user_agent.as_ref().map(|ua| {
            serde_json::json!({
                "user_agent": ua,
                "parsed_at": Utc::now()
            })
        })
    }

    /// Clean up expired sessions and tokens
    pub async fn cleanup_expired_data(&self) -> Result<(), AppError> {
        // Clean up expired sessions
        let sessions_cleaned = self.user_repository.cleanup_expired_sessions().await?;
        
        // Clean up revoked tokens
        let tokens_cleaned = self.jwt_service.cleanup_revoked_tokens()?;
        
        tracing::info!(
            "Cleanup completed: {} expired sessions, {} revoked tokens",
            sessions_cleaned,
            tokens_cleaned
        );

        Ok(())
    }
}    
/// Update user profile
    pub async fn update_user_profile(
        &self,
        user_id: Uuid,
        update_request: crate::handlers::auth::UpdateProfileRequest,
    ) -> Result<User, AppError> {
        User::update_profile(
            &self.pool,
            user_id,
            update_request.username,
            update_request.phone_number,
        )
        .await
    }

    /// Get current session info from refresh token
    pub async fn get_current_session_info(
        &self,
        refresh_token: &str,
    ) -> Result<crate::handlers::auth::SessionInfo, AppError> {
        let token_hash = hash_token(refresh_token);
        
        let session = self.session_manager
            .get_session_by_token(&token_hash)
            .await?
            .ok_or_else(|| AppError::NotFound("Session not found".to_string()))?;

        Ok(crate::handlers::auth::SessionInfo {
            session_id: session.id,
            device_info: session.user_agent,
            ip_address: session.ip_address,
            expires_at: session.expires_at,
        })
    }

    /// Logout current session
    pub async fn logout_current_session(&self, jti: &str) -> Result<(), AppError> {
        // In a real implementation, you'd map JTI to session ID
        // For now, we'll revoke the token by JTI
        self.jwt_service.revoke_token(jti)?;
        Ok(())
    }

    /// Logout all sessions for a user
    pub async fn logout_all_sessions(&self, user_id: Uuid) -> Result<u64, AppError> {
        // Revoke all JWT tokens for the user
        self.jwt_service.revoke_user_tokens(user_id, &self.pool).await?;
        
        // Revoke all sessions
        self.session_manager.revoke_all_sessions(user_id).await
    }

    /// Verify email with token (public method)
    pub async fn verify_email(&self, token: &str) -> Result<(), AppError> {
        // Find and validate verification token
        let token_hash = hash_token(token);
        let verification_record = sqlx::query!(
            r#"
            SELECT user_id, expires_at, is_used 
            FROM email_verification_tokens 
            WHERE token = $1 AND expires_at > NOW() AND is_used = false
            "#,
            token_hash
        )
        .fetch_optional(&*self.pool)
        .await?
        .ok_or_else(|| AppError::Authentication("Invalid or expired verification token".to_string()))?;

        // Update user as verified and mark token as used
        let mut tx = self.pool.begin().await?;

        sqlx::query!(
            "UPDATE users SET email_verified = true, updated_at = NOW() WHERE id = $1",
            verification_record.user_id
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!(
            "UPDATE email_verification_tokens SET is_used = true WHERE token = $1",
            token_hash
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        // Log email verification
        AuditLog::create(
            &self.pool,
            Some(verification_record.user_id),
            raffle_platform_shared::AuditAction::UserAction,
            Some("auth".to_string()),
            Some(verification_record.user_id),
            None,
            Some(serde_json::json!({
                "event": "email_verified"
            })),
            None,
            None,
        ).await?;

        Ok(())
    }

    /// Resend email verification
    pub async fn resend_email_verification(&self, email: &str) -> Result<(), AppError> {
        // Find user by email
        let user = User::find_by_email(&self.pool, email)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        // Check if already verified
        if user.email_verified {
            return Err(AppError::Validation("Email is already verified".to_string()));
        }

        // Generate new verification token
        let verification_token = generate_secure_token();
        let token_hash = hash_token(&verification_token);
        let expires_at = Utc::now() + Duration::hours(24); // 24 hour expiry

        // Store verification token in database
        sqlx::query!(
            r#"
            INSERT INTO email_verification_tokens (user_id, token, expires_at)
            VALUES ($1, $2, $3)
            ON CONFLICT (user_id) 
            DO UPDATE SET token = EXCLUDED.token, expires_at = EXCLUDED.expires_at, is_used = false
            "#,
            user.id,
            token_hash,
            expires_at
        )
        .execute(&*self.pool)
        .await?;

        // TODO: Send email with verification token
        // For now, we'll just log it (in production, use proper email service)
        tracing::info!("Email verification token for {}: {}", email, verification_token);

        Ok(())
    }

    /// Get user sessions
    pub async fn get_user_sessions(&self, user_id: Uuid) -> Result<Vec<SessionInfo>, AppError> {
        self.session_manager.get_user_sessions(user_id).await
    }

    /// Revoke a specific session
    pub async fn revoke_session(&self, session_id: Uuid, user_id: Uuid) -> Result<(), AppError> {
        self.session_manager.revoke_session(session_id, user_id).await
    }