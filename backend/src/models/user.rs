use chrono::{DateTime, Utc};
use raffle_platform_shared::{UserRole, CreateUserRequest, UserResponse};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;
use crate::error::AppError;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: Option<String>,
    pub role: UserRole,
    pub credit_balance: Decimal,
    pub internal_wallet_address: String,
    pub internal_wallet_private_key_encrypted: String,
    pub internal_wallet_mnemonic_encrypted: Option<String>,
    pub phone_number: Option<String>,
    pub google_id: Option<String>,
    pub apple_id: Option<String>,
    pub is_active: bool,
    pub email_verified: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    /// Create a new user in the database
    pub async fn create(
        pool: &PgPool,
        request: CreateUserRequest,
        password_hash: String,
        wallet_address: String,
        encrypted_private_key: String,
        encrypted_mnemonic: Option<String>,
    ) -> Result<Self, AppError> {
        let user = sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (username, email, password_hash, internal_wallet_address, internal_wallet_private_key_encrypted, internal_wallet_mnemonic_encrypted, phone_number)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING 
                id, username, email, password_hash, 
                role as "role: UserRole", credit_balance, 
                internal_wallet_address, internal_wallet_private_key_encrypted, internal_wallet_mnemonic_encrypted,
                phone_number, google_id, apple_id, is_active, email_verified,
                created_at, updated_at
            "#,
            request.username,
            request.email,
            password_hash,
            wallet_address,
            encrypted_private_key,
            encrypted_mnemonic,
            request.phone_number
        )
        .fetch_one(pool)
        .await?;

        Ok(user)
    }

    /// Find user by email
    pub async fn find_by_email(pool: &PgPool, email: &str) -> Result<Option<Self>, AppError> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT 
                id, username, email, password_hash, 
                role as "role: UserRole", credit_balance, 
                internal_wallet_address, internal_wallet_private_key_encrypted,
                phone_number, google_id, apple_id, is_active, email_verified,
                created_at, updated_at
            FROM users 
            WHERE email = $1 AND is_active = true
            "#,
            email
        )
        .fetch_optional(pool)
        .await?;

        Ok(user)
    }

    /// Find user by ID
    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Self>, AppError> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT 
                id, username, email, password_hash, 
                role as "role: UserRole", credit_balance, 
                internal_wallet_address, internal_wallet_private_key_encrypted,
                phone_number, google_id, apple_id, is_active, email_verified,
                created_at, updated_at
            FROM users 
            WHERE id = $1 AND is_active = true
            "#,
            id
        )
        .fetch_optional(pool)
        .await?;

        Ok(user)
    }

    /// Find user by username
    pub async fn find_by_username(pool: &PgPool, username: &str) -> Result<Option<Self>, AppError> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT 
                id, username, email, password_hash, 
                role as "role: UserRole", credit_balance, 
                internal_wallet_address, internal_wallet_private_key_encrypted,
                phone_number, google_id, apple_id, is_active, email_verified,
                created_at, updated_at
            FROM users 
            WHERE username = $1 AND is_active = true
            "#,
            username
        )
        .fetch_optional(pool)
        .await?;

        Ok(user)
    }

    /// Update user's credit balance
    pub async fn update_credit_balance(
        pool: &PgPool,
        user_id: Uuid,
        new_balance: Decimal,
    ) -> Result<(), AppError> {
        sqlx::query!(
            "UPDATE users SET credit_balance = $1, updated_at = NOW() WHERE id = $2",
            new_balance,
            user_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Update user's email verification status
    pub async fn verify_email(pool: &PgPool, user_id: Uuid) -> Result<(), AppError> {
        sqlx::query!(
            "UPDATE users SET email_verified = true, updated_at = NOW() WHERE id = $1",
            user_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Update user's password
    pub async fn update_password(
        pool: &PgPool,
        user_id: Uuid,
        new_password_hash: String,
    ) -> Result<(), AppError> {
        sqlx::query!(
            "UPDATE users SET password_hash = $1, updated_at = NOW() WHERE id = $2",
            new_password_hash,
            user_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Deactivate user account
    pub async fn deactivate(pool: &PgPool, user_id: Uuid) -> Result<(), AppError> {
        sqlx::query!(
            "UPDATE users SET is_active = false, updated_at = NOW() WHERE id = $1",
            user_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Check if email exists
    pub async fn email_exists(pool: &PgPool, email: &str) -> Result<bool, AppError> {
        let count: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM users WHERE email = $1",
            email
        )
        .fetch_one(pool)
        .await?
        .unwrap_or(0);

        Ok(count > 0)
    }

    /// Check if username exists
    pub async fn username_exists(pool: &PgPool, username: &str) -> Result<bool, AppError> {
        let count: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM users WHERE username = $1",
            username
        )
        .fetch_one(pool)
        .await?
        .unwrap_or(0);

        Ok(count > 0)
    }

    /// Convert to response DTO (without sensitive data)
    pub fn to_response(&self) -> UserResponse {
        UserResponse {
            id: self.id,
            username: self.username.clone(),
            email: self.email.clone(),
            role: self.role,
            credit_balance: self.credit_balance,
            internal_wallet_address: self.internal_wallet_address.clone(),
            phone_number: self.phone_number.clone(),
            is_active: self.is_active,
            email_verified: self.email_verified,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }

    /// Get user's wallet address (for blockchain operations)
    pub fn wallet_address(&self) -> &str {
        &self.internal_wallet_address
    }

    /// Update user profile information
    pub async fn update_profile(
        pool: &PgPool,
        user_id: Uuid,
        username: Option<String>,
        phone_number: Option<String>,
    ) -> Result<Self, AppError> {
        let user = sqlx::query_as!(
            User,
            r#"
            UPDATE users 
            SET 
                username = COALESCE($2, username),
                phone_number = COALESCE($3, phone_number),
                updated_at = NOW()
            WHERE id = $1 AND is_active = true
            RETURNING 
                id, username, email, password_hash, 
                role as "role: UserRole", credit_balance, 
                internal_wallet_address, internal_wallet_private_key_encrypted, internal_wallet_mnemonic_encrypted,
                phone_number, google_id, apple_id, is_active, email_verified,
                created_at, updated_at
            "#,
            user_id,
            username,
            phone_number
        )
        .fetch_one(pool)
        .await?;

        Ok(user)
    }

    /// Find users by role
    pub async fn find_by_role(
        pool: &PgPool,
        role: UserRole,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Self>, AppError> {
        let users = sqlx::query_as!(
            User,
            r#"
            SELECT 
                id, username, email, password_hash, 
                role as "role: UserRole", credit_balance, 
                internal_wallet_address, internal_wallet_private_key_encrypted, internal_wallet_mnemonic_encrypted,
                phone_number, google_id, apple_id, is_active, email_verified,
                created_at, updated_at
            FROM users 
            WHERE role = $1 AND is_active = true
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            role as UserRole,
            limit,
            offset
        )
        .fetch_all(pool)
        .await?;

        Ok(users)
    }

    /// Search users by username or email
    pub async fn search(
        pool: &PgPool,
        query: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Self>, AppError> {
        let search_pattern = format!("%{}%", query.to_lowercase());
        
        let users = sqlx::query_as!(
            User,
            r#"
            SELECT 
                id, username, email, password_hash, 
                role as "role: UserRole", credit_balance, 
                internal_wallet_address, internal_wallet_private_key_encrypted, internal_wallet_mnemonic_encrypted,
                phone_number, google_id, apple_id, is_active, email_verified,
                created_at, updated_at
            FROM users 
            WHERE (LOWER(username) LIKE $1 OR LOWER(email) LIKE $1) 
            AND is_active = true
            ORDER BY username ASC
            LIMIT $2 OFFSET $3
            "#,
            search_pattern,
            limit,
            offset
        )
        .fetch_all(pool)
        .await?;

        Ok(users)
    }

    /// Get user statistics
    pub async fn get_statistics(pool: &PgPool) -> Result<UserStatistics, AppError> {
        let stats = sqlx::query!(
            r#"
            SELECT 
                COUNT(*) as total_users,
                COUNT(CASE WHEN is_active = true THEN 1 END) as active_users,
                COUNT(CASE WHEN email_verified = true THEN 1 END) as verified_users,
                COUNT(CASE WHEN role = 'seller' THEN 1 END) as sellers,
                COUNT(CASE WHEN role = 'admin' THEN 1 END) as admins,
                AVG(credit_balance) as average_credit_balance
            FROM users
            "#
        )
        .fetch_one(pool)
        .await?;

        Ok(UserStatistics {
            total_users: stats.total_users.unwrap_or(0) as u32,
            active_users: stats.active_users.unwrap_or(0) as u32,
            verified_users: stats.verified_users.unwrap_or(0) as u32,
            sellers: stats.sellers.unwrap_or(0) as u32,
            admins: stats.admins.unwrap_or(0) as u32,
            average_credit_balance: stats.average_credit_balance.unwrap_or(Decimal::ZERO),
        })
    }

    /// Update user's OAuth information
    pub async fn update_oauth_info(
        pool: &PgPool,
        user_id: Uuid,
        google_id: Option<String>,
        apple_id: Option<String>,
    ) -> Result<(), AppError> {
        sqlx::query!(
            "UPDATE users SET google_id = $2, apple_id = $3, updated_at = NOW() WHERE id = $1",
            user_id,
            google_id,
            apple_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Find user by OAuth ID
    pub async fn find_by_oauth_id(
        pool: &PgPool,
        provider: &str,
        oauth_id: &str,
    ) -> Result<Option<Self>, AppError> {
        let user = match provider {
            "google" => {
                sqlx::query_as!(
                    User,
                    r#"
                    SELECT 
                        id, username, email, password_hash, 
                        role as "role: UserRole", credit_balance, 
                        internal_wallet_address, internal_wallet_private_key_encrypted, internal_wallet_mnemonic_encrypted,
                        phone_number, google_id, apple_id, is_active, email_verified,
                        created_at, updated_at
                    FROM users 
                    WHERE google_id = $1 AND is_active = true
                    "#,
                    oauth_id
                )
                .fetch_optional(pool)
                .await?
            }
            "apple" => {
                sqlx::query_as!(
                    User,
                    r#"
                    SELECT 
                        id, username, email, password_hash, 
                        role as "role: UserRole", credit_balance, 
                        internal_wallet_address, internal_wallet_private_key_encrypted, internal_wallet_mnemonic_encrypted,
                        phone_number, google_id, apple_id, is_active, email_verified,
                        created_at, updated_at
                    FROM users 
                    WHERE apple_id = $1 AND is_active = true
                    "#,
                    oauth_id
                )
                .fetch_optional(pool)
                .await?
            }
            _ => return Err(AppError::Validation("Invalid OAuth provider".to_string())),
        };

        Ok(user)
    }

    /// Activate user account
    pub async fn activate(pool: &PgPool, user_id: Uuid) -> Result<(), AppError> {
        sqlx::query!(
            "UPDATE users SET is_active = true, updated_at = NOW() WHERE id = $1",
            user_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Update user role (admin only)
    pub async fn update_role(
        pool: &PgPool,
        user_id: Uuid,
        new_role: UserRole,
    ) -> Result<(), AppError> {
        sqlx::query!(
            "UPDATE users SET role = $2, updated_at = NOW() WHERE id = $1",
            user_id,
            new_role as UserRole
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct UserSession {
    pub id: Uuid,
    pub user_id: Uuid,
    pub refresh_token_hash: String,
    pub device_info: Option<serde_json::Value>,
    pub ip_address: Option<std::net::IpAddr>,
    pub user_agent: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl UserSession {
    /// Create a new session
    pub async fn create(
        pool: &PgPool,
        user_id: Uuid,
        refresh_token_hash: String,
        expires_at: DateTime<Utc>,
        device_info: Option<serde_json::Value>,
        ip_address: Option<std::net::IpAddr>,
        user_agent: Option<String>,
    ) -> Result<Self, AppError> {
        let session = sqlx::query_as!(
            UserSession,
            r#"
            INSERT INTO user_sessions (user_id, refresh_token_hash, expires_at, device_info, ip_address, user_agent)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, user_id, refresh_token_hash, device_info, 
                     ip_address as "ip_address: std::net::IpAddr", user_agent, 
                     expires_at, is_active, created_at, updated_at
            "#,
            user_id,
            refresh_token_hash,
            expires_at,
            device_info,
            ip_address,
            user_agent
        )
        .fetch_one(pool)
        .await?;

        Ok(session)
    }

    /// Find session by refresh token hash
    pub async fn find_by_refresh_token(
        pool: &PgPool,
        refresh_token_hash: &str,
    ) -> Result<Option<Self>, AppError> {
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
        .fetch_optional(pool)
        .await?;

        Ok(session)
    }

    /// Deactivate session
    pub async fn deactivate(pool: &PgPool, session_id: Uuid) -> Result<(), AppError> {
        sqlx::query!(
            "UPDATE user_sessions SET is_active = false, updated_at = NOW() WHERE id = $1",
            session_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Deactivate all sessions for a user
    pub async fn deactivate_all_for_user(pool: &PgPool, user_id: Uuid) -> Result<(), AppError> {
        sqlx::query!(
            "UPDATE user_sessions SET is_active = false, updated_at = NOW() WHERE user_id = $1",
            user_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Clean up expired sessions
    pub async fn cleanup_expired(pool: &PgPool) -> Result<u64, AppError> {
        let result = sqlx::query!(
            "DELETE FROM user_sessions WHERE expires_at < NOW() OR is_active = false"
        )
        .execute(pool)
        .await?;

        Ok(result.rows_affected())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStatistics {
    pub total_users: u32,
    pub active_users: u32,
    pub verified_users: u32,
    pub sellers: u32,
    pub admins: u32,
    pub average_credit_balance: Decimal,
}