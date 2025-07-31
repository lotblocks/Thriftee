use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;
use crate::error::AppError;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Notification {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub message: String,
    pub notification_type: String,
    pub data: Option<serde_json::Value>,
    pub is_read: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationType {
    RaffleWin,
    RaffleLoss,
    CreditExpiring,
    CreditIssued,
    PaymentReceived,
    PaymentFailed,
    SellerVerification,
    ItemSold,
    RaffleCompleted,
    SystemAlert,
    SecurityAlert,
}

impl NotificationType {
    pub fn as_str(&self) -> &'static str {
        match self {
            NotificationType::RaffleWin => "raffle_win",
            NotificationType::RaffleLoss => "raffle_loss",
            NotificationType::CreditExpiring => "credit_expiring",
            NotificationType::CreditIssued => "credit_issued",
            NotificationType::PaymentReceived => "payment_received",
            NotificationType::PaymentFailed => "payment_failed",
            NotificationType::SellerVerification => "seller_verification",
            NotificationType::ItemSold => "item_sold",
            NotificationType::RaffleCompleted => "raffle_completed",
            NotificationType::SystemAlert => "system_alert",
            NotificationType::SecurityAlert => "security_alert",
        }
    }
}

impl Notification {
    /// Create a new notification
    pub async fn create(
        pool: &PgPool,
        user_id: Uuid,
        title: String,
        message: String,
        notification_type: NotificationType,
        data: Option<serde_json::Value>,
    ) -> Result<Self, AppError> {
        let notification = sqlx::query_as!(
            Notification,
            r#"
            INSERT INTO notifications (user_id, title, message, type, data)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, user_id, title, message, type as notification_type, data, is_read, created_at
            "#,
            user_id,
            title,
            message,
            notification_type.as_str(),
            data
        )
        .fetch_one(pool)
        .await?;

        Ok(notification)
    }

    /// Find notifications for a user
    pub async fn find_by_user(
        pool: &PgPool,
        user_id: Uuid,
        limit: i64,
        offset: i64,
        unread_only: bool,
    ) -> Result<Vec<Self>, AppError> {
        let notifications = if unread_only {
            sqlx::query_as!(
                Notification,
                r#"
                SELECT id, user_id, title, message, type as notification_type, data, is_read, created_at
                FROM notifications 
                WHERE user_id = $1 AND is_read = false
                ORDER BY created_at DESC
                LIMIT $2 OFFSET $3
                "#,
                user_id,
                limit,
                offset
            )
            .fetch_all(pool)
            .await?
        } else {
            sqlx::query_as!(
                Notification,
                r#"
                SELECT id, user_id, title, message, type as notification_type, data, is_read, created_at
                FROM notifications 
                WHERE user_id = $1
                ORDER BY created_at DESC
                LIMIT $2 OFFSET $3
                "#,
                user_id,
                limit,
                offset
            )
            .fetch_all(pool)
            .await?
        };

        Ok(notifications)
    }

    /// Mark notification as read
    pub async fn mark_as_read(pool: &PgPool, id: Uuid) -> Result<(), AppError> {
        sqlx::query!(
            "UPDATE notifications SET is_read = true WHERE id = $1",
            id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Mark all notifications as read for a user
    pub async fn mark_all_as_read(pool: &PgPool, user_id: Uuid) -> Result<u64, AppError> {
        let result = sqlx::query!(
            "UPDATE notifications SET is_read = true WHERE user_id = $1 AND is_read = false",
            user_id
        )
        .execute(pool)
        .await?;

        Ok(result.rows_affected())
    }

    /// Get unread count for a user
    pub async fn get_unread_count(pool: &PgPool, user_id: Uuid) -> Result<i64, AppError> {
        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM notifications WHERE user_id = $1 AND is_read = false",
            user_id
        )
        .fetch_one(pool)
        .await?;

        Ok(count.unwrap_or(0))
    }

    /// Delete old notifications (cleanup)
    pub async fn cleanup_old_notifications(
        pool: &PgPool,
        days_old: i32,
    ) -> Result<u64, AppError> {
        let result = sqlx::query!(
            "DELETE FROM notifications WHERE created_at < NOW() - INTERVAL '%d days' AND is_read = true",
            days_old
        )
        .execute(pool)
        .await?;

        Ok(result.rows_affected())
    }

    /// Create raffle win notification
    pub async fn create_raffle_win_notification(
        pool: &PgPool,
        user_id: Uuid,
        raffle_id: Uuid,
        item_name: String,
    ) -> Result<Self, AppError> {
        let title = "Congratulations! You won!".to_string();
        let message = format!("You won the raffle for {}! Check your winnings to claim your prize.", item_name);
        let data = serde_json::json!({
            "raffle_id": raffle_id,
            "item_name": item_name
        });

        Self::create(
            pool,
            user_id,
            title,
            message,
            NotificationType::RaffleWin,
            Some(data),
        ).await
    }

    /// Create credit expiring notification
    pub async fn create_credit_expiring_notification(
        pool: &PgPool,
        user_id: Uuid,
        credit_amount: rust_decimal::Decimal,
        expires_at: DateTime<Utc>,
    ) -> Result<Self, AppError> {
        let title = "Credits Expiring Soon".to_string();
        let message = format!(
            "You have ${} in credits expiring on {}. Use them before they expire!",
            credit_amount,
            expires_at.format("%Y-%m-%d")
        );
        let data = serde_json::json!({
            "credit_amount": credit_amount,
            "expires_at": expires_at
        });

        Self::create(
            pool,
            user_id,
            title,
            message,
            NotificationType::CreditExpiring,
            Some(data),
        ).await
    }

    /// Create credit issued notification
    pub async fn create_credit_issued_notification(
        pool: &PgPool,
        user_id: Uuid,
        credit_amount: rust_decimal::Decimal,
        source: &str,
    ) -> Result<Self, AppError> {
        let title = "Credits Added".to_string();
        let message = format!("${} in credits have been added to your account from {}.", credit_amount, source);
        let data = serde_json::json!({
            "credit_amount": credit_amount,
            "source": source
        });

        Self::create(
            pool,
            user_id,
            title,
            message,
            NotificationType::CreditIssued,
            Some(data),
        ).await
    }

    /// Create seller verification notification
    pub async fn create_seller_verification_notification(
        pool: &PgPool,
        user_id: Uuid,
        is_approved: bool,
    ) -> Result<Self, AppError> {
        let (title, message) = if is_approved {
            (
                "Seller Account Approved".to_string(),
                "Your seller account has been approved! You can now start listing items.".to_string(),
            )
        } else {
            (
                "Seller Account Review".to_string(),
                "Your seller account application needs additional information. Please check your email for details.".to_string(),
            )
        };

        let data = serde_json::json!({
            "is_approved": is_approved
        });

        Self::create(
            pool,
            user_id,
            title,
            message,
            NotificationType::SellerVerification,
            Some(data),
        ).await
    }
}