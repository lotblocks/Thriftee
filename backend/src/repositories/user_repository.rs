use crate::error::AppError;
use crate::models::{User, UserSession};
use crate::repositories::{PaginatedResult, PaginationParams, Repository};
use raffle_platform_shared::{CreateUserRequest, UserResponse};
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct UserRepository {
    pool: Arc<PgPool>,
}

impl UserRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    /// Find user by email
    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>, AppError> {
        User::find_by_email(&self.pool, email).await
    }

    /// Find user by username
    pub async fn find_by_username(&self, username: &str) -> Result<Option<User>, AppError> {
        User::find_by_username(&self.pool, username).await
    }

    /// Check if email exists
    pub async fn email_exists(&self, email: &str) -> Result<bool, AppError> {
        User::email_exists(&self.pool, email).await
    }

    /// Check if username exists
    pub async fn username_exists(&self, username: &str) -> Result<bool, AppError> {
        User::username_exists(&self.pool, username).await
    }

    /// Update user's credit balance
    pub async fn update_credit_balance(&self, user_id: Uuid, new_balance: Decimal) -> Result<(), AppError> {
        User::update_credit_balance(&self.pool, user_id, new_balance).await
    }

    /// Verify user's email
    pub async fn verify_email(&self, user_id: Uuid) -> Result<(), AppError> {
        User::verify_email(&self.pool, user_id).await
    }

    /// Update user's password
    pub async fn update_password(&self, user_id: Uuid, new_password_hash: String) -> Result<(), AppError> {
        User::update_password(&self.pool, user_id, new_password_hash).await
    }

    /// Deactivate user account
    pub async fn deactivate(&self, user_id: Uuid) -> Result<(), AppError> {
        User::deactivate(&self.pool, user_id).await
    }

    /// Get all users with pagination
    pub async fn find_all(&self, pagination: PaginationParams) -> Result<PaginatedResult<User>, AppError> {
        let users = sqlx::query_as!(
            User,
            r#"
            SELECT 
                id, username, email, password_hash, 
                role as "role: raffle_platform_shared::UserRole", credit_balance, 
                internal_wallet_address, internal_wallet_private_key_encrypted,
                phone_number, google_id, apple_id, is_active, email_verified,
                created_at, updated_at
            FROM users 
            WHERE is_active = true
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
            pagination.limit,
            pagination.offset
        )
        .fetch_all(&*self.pool)
        .await?;

        let total = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM users WHERE is_active = true"
        )
        .fetch_one(&*self.pool)
        .await?
        .unwrap_or(0);

        Ok(PaginatedResult::new(users, total, pagination.limit, pagination.offset))
    }

    /// Search users by username or email
    pub async fn search(&self, query: &str, pagination: PaginationParams) -> Result<PaginatedResult<User>, AppError> {
        let search_pattern = format!("%{}%", query);
        
        let users = sqlx::query_as!(
            User,
            r#"
            SELECT 
                id, username, email, password_hash, 
                role as "role: raffle_platform_shared::UserRole", credit_balance, 
                internal_wallet_address, internal_wallet_private_key_encrypted,
                phone_number, google_id, apple_id, is_active, email_verified,
                created_at, updated_at
            FROM users 
            WHERE is_active = true 
            AND (username ILIKE $1 OR email ILIKE $1)
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            search_pattern,
            pagination.limit,
            pagination.offset
        )
        .fetch_all(&*self.pool)
        .await?;

        let total = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM users WHERE is_active = true AND (username ILIKE $1 OR email ILIKE $1)",
            search_pattern
        )
        .fetch_one(&*self.pool)
        .await?
        .unwrap_or(0);

        Ok(PaginatedResult::new(users, total, pagination.limit, pagination.offset))
    }

    /// Get user statistics
    pub async fn get_user_stats(&self, user_id: Uuid) -> Result<UserStats, AppError> {
        let stats = sqlx::query!(
            r#"
            SELECT 
                u.credit_balance,
                COALESCE(bp_count.total_boxes, 0) as total_boxes_purchased,
                COALESCE(bp_count.total_spent, 0) as total_credits_spent,
                COALESCE(win_count.total_wins, 0) as total_wins,
                COALESCE(raffle_count.total_raffles, 0) as total_raffles_participated
            FROM users u
            LEFT JOIN (
                SELECT 
                    user_id, 
                    COUNT(*) as total_boxes,
                    SUM(purchase_price_in_credits) as total_spent
                FROM box_purchases 
                WHERE user_id = $1
                GROUP BY user_id
            ) bp_count ON u.id = bp_count.user_id
            LEFT JOIN (
                SELECT 
                    user_id,
                    COUNT(*) as total_wins
                FROM box_purchases bp
                JOIN raffles r ON bp.raffle_id = r.id
                WHERE bp.user_id = $1 AND r.winner_user_ids @> ARRAY[bp.user_id]
                GROUP BY user_id
            ) win_count ON u.id = win_count.user_id
            LEFT JOIN (
                SELECT 
                    user_id,
                    COUNT(DISTINCT raffle_id) as total_raffles
                FROM box_purchases
                WHERE user_id = $1
                GROUP BY user_id
            ) raffle_count ON u.id = raffle_count.user_id
            WHERE u.id = $1
            "#,
            user_id
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(UserStats {
            credit_balance: stats.credit_balance,
            total_boxes_purchased: stats.total_boxes_purchased.unwrap_or(0),
            total_credits_spent: stats.total_spent.unwrap_or(Decimal::ZERO),
            total_wins: stats.total_wins.unwrap_or(0),
            total_raffles_participated: stats.total_raffles.unwrap_or(0),
        })
    }

    /// Create user session
    pub async fn create_session(
        &self,
        user_id: Uuid,
        refresh_token_hash: String,
        expires_at: chrono::DateTime<chrono::Utc>,
        device_info: Option<serde_json::Value>,
        ip_address: Option<std::net::IpAddr>,
        user_agent: Option<String>,
    ) -> Result<UserSession, AppError> {
        UserSession::create(
            &self.pool,
            user_id,
            refresh_token_hash,
            expires_at,
            device_info,
            ip_address,
            user_agent,
        ).await
    }

    /// Find session by refresh token
    pub async fn find_session_by_refresh_token(&self, refresh_token_hash: &str) -> Result<Option<UserSession>, AppError> {
        UserSession::find_by_refresh_token(&self.pool, refresh_token_hash).await
    }

    /// Deactivate session
    pub async fn deactivate_session(&self, session_id: Uuid) -> Result<(), AppError> {
        UserSession::deactivate(&self.pool, session_id).await
    }

    /// Deactivate all sessions for user
    pub async fn deactivate_all_sessions(&self, user_id: Uuid) -> Result<(), AppError> {
        UserSession::deactivate_all_for_user(&self.pool, user_id).await
    }

    /// Clean up expired sessions
    pub async fn cleanup_expired_sessions(&self) -> Result<u64, AppError> {
        UserSession::cleanup_expired(&self.pool).await
    }
}

impl Repository for UserRepository {
    type Entity = User;
    type Id = Uuid;
    type CreateRequest = (CreateUserRequest, String, String, String); // (request, password_hash, wallet_address, encrypted_key)
    type UpdateRequest = (); // Not implemented for this example

    async fn find_by_id(&self, id: Self::Id) -> Result<Option<Self::Entity>, AppError> {
        User::find_by_id(&self.pool, id).await
    }

    async fn create(&self, request: Self::CreateRequest) -> Result<Self::Entity, AppError> {
        let (user_request, password_hash, wallet_address, encrypted_key) = request;
        User::create(&self.pool, user_request, password_hash, wallet_address, encrypted_key).await
    }

    async fn update(&self, _id: Self::Id, _request: Self::UpdateRequest) -> Result<Self::Entity, AppError> {
        // Implementation would go here
        todo!("User update not implemented in this example")
    }

    async fn delete(&self, id: Self::Id) -> Result<bool, AppError> {
        self.deactivate(id).await?;
        Ok(true)
    }
}

#[derive(Debug, Clone)]
pub struct UserStats {
    pub credit_balance: Decimal,
    pub total_boxes_purchased: i64,
    pub total_credits_spent: Decimal,
    pub total_wins: i64,
    pub total_raffles_participated: i64,
}