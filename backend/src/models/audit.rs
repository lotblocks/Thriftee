use chrono::{DateTime, Utc};
use raffle_platform_shared::AuditAction;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use std::net::IpAddr;
use uuid::Uuid;
use crate::error::AppError;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct AuditLog {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub action: String,
    pub resource_type: Option<String>,
    pub resource_id: Option<Uuid>,
    pub old_values: Option<serde_json::Value>,
    pub new_values: Option<serde_json::Value>,
    pub ip_address: Option<IpAddr>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl AuditLog {
    /// Create a new audit log entry
    pub async fn create(
        pool: &PgPool,
        user_id: Option<Uuid>,
        action: AuditAction,
        resource_type: Option<String>,
        resource_id: Option<Uuid>,
        old_values: Option<serde_json::Value>,
        new_values: Option<serde_json::Value>,
        ip_address: Option<IpAddr>,
        user_agent: Option<String>,
    ) -> Result<Self, AppError> {
        let audit_log = sqlx::query_as!(
            AuditLog,
            r#"
            INSERT INTO audit_logs (user_id, action, resource_type, resource_id, old_values, new_values, ip_address, user_agent)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, user_id, action, resource_type, resource_id, old_values, new_values, 
                     ip_address as "ip_address: IpAddr", user_agent, created_at
            "#,
            user_id,
            action.as_str(),
            resource_type,
            resource_id,
            old_values,
            new_values,
            ip_address,
            user_agent
        )
        .fetch_one(pool)
        .await?;

        Ok(audit_log)
    }

    /// Find audit logs by user
    pub async fn find_by_user(
        pool: &PgPool,
        user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Self>, AppError> {
        let logs = sqlx::query_as!(
            AuditLog,
            r#"
            SELECT id, user_id, action, resource_type, resource_id, old_values, new_values, 
                   ip_address as "ip_address: IpAddr", user_agent, created_at
            FROM audit_logs 
            WHERE user_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            user_id,
            limit,
            offset
        )
        .fetch_all(pool)
        .await?;

        Ok(logs)
    }

    /// Find audit logs by action
    pub async fn find_by_action(
        pool: &PgPool,
        action: AuditAction,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Self>, AppError> {
        let logs = sqlx::query_as!(
            AuditLog,
            r#"
            SELECT id, user_id, action, resource_type, resource_id, old_values, new_values, 
                   ip_address as "ip_address: IpAddr", user_agent, created_at
            FROM audit_logs 
            WHERE action = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            action.as_str(),
            limit,
            offset
        )
        .fetch_all(pool)
        .await?;

        Ok(logs)
    }

    /// Find audit logs by resource
    pub async fn find_by_resource(
        pool: &PgPool,
        resource_type: String,
        resource_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Self>, AppError> {
        let logs = sqlx::query_as!(
            AuditLog,
            r#"
            SELECT id, user_id, action, resource_type, resource_id, old_values, new_values, 
                   ip_address as "ip_address: IpAddr", user_agent, created_at
            FROM audit_logs 
            WHERE resource_type = $1 AND resource_id = $2
            ORDER BY created_at DESC
            LIMIT $3 OFFSET $4
            "#,
            resource_type,
            resource_id,
            limit,
            offset
        )
        .fetch_all(pool)
        .await?;

        Ok(logs)
    }

    /// Find recent security events
    pub async fn find_security_events(
        pool: &PgPool,
        hours: i32,
        limit: i64,
    ) -> Result<Vec<Self>, AppError> {
        let logs = sqlx::query_as!(
            AuditLog,
            r#"
            SELECT id, user_id, action, resource_type, resource_id, old_values, new_values, 
                   ip_address as "ip_address: IpAddr", user_agent, created_at
            FROM audit_logs 
            WHERE action IN ('login', 'logout', 'password_change', 'security_event')
            AND created_at > NOW() - INTERVAL '%d hours'
            ORDER BY created_at DESC
            LIMIT $1
            "#,
            limit,
            hours
        )
        .fetch_all(pool)
        .await?;

        Ok(logs)
    }

    /// Clean up old audit logs
    pub async fn cleanup_old_logs(
        pool: &PgPool,
        retention_days: i32,
    ) -> Result<u64, AppError> {
        let result = sqlx::query!(
            "DELETE FROM audit_logs WHERE created_at < NOW() - INTERVAL '%d days'",
            retention_days
        )
        .execute(pool)
        .await?;

        Ok(result.rows_affected())
    }

    /// Log user login
    pub async fn log_user_login(
        pool: &PgPool,
        user_id: Uuid,
        ip_address: Option<IpAddr>,
        user_agent: Option<String>,
    ) -> Result<Self, AppError> {
        Self::create(
            pool,
            Some(user_id),
            AuditAction::Login,
            Some("user".to_string()),
            Some(user_id),
            None,
            None,
            ip_address,
            user_agent,
        ).await
    }

    /// Log user logout
    pub async fn log_user_logout(
        pool: &PgPool,
        user_id: Uuid,
        ip_address: Option<IpAddr>,
        user_agent: Option<String>,
    ) -> Result<Self, AppError> {
        Self::create(
            pool,
            Some(user_id),
            AuditAction::Logout,
            Some("user".to_string()),
            Some(user_id),
            None,
            None,
            ip_address,
            user_agent,
        ).await
    }

    /// Log password change
    pub async fn log_password_change(
        pool: &PgPool,
        user_id: Uuid,
        ip_address: Option<IpAddr>,
        user_agent: Option<String>,
    ) -> Result<Self, AppError> {
        Self::create(
            pool,
            Some(user_id),
            AuditAction::PasswordChange,
            Some("user".to_string()),
            Some(user_id),
            None,
            None,
            ip_address,
            user_agent,
        ).await
    }

    /// Log raffle creation
    pub async fn log_raffle_creation(
        pool: &PgPool,
        user_id: Option<Uuid>,
        raffle_id: Uuid,
        raffle_data: serde_json::Value,
        ip_address: Option<IpAddr>,
        user_agent: Option<String>,
    ) -> Result<Self, AppError> {
        Self::create(
            pool,
            user_id,
            AuditAction::RaffleCreate,
            Some("raffle".to_string()),
            Some(raffle_id),
            None,
            Some(raffle_data),
            ip_address,
            user_agent,
        ).await
    }

    /// Log box purchase
    pub async fn log_box_purchase(
        pool: &PgPool,
        user_id: Uuid,
        raffle_id: Uuid,
        box_number: i32,
        amount: rust_decimal::Decimal,
        ip_address: Option<IpAddr>,
        user_agent: Option<String>,
    ) -> Result<Self, AppError> {
        let purchase_data = serde_json::json!({
            "raffle_id": raffle_id,
            "box_number": box_number,
            "amount": amount
        });

        Self::create(
            pool,
            Some(user_id),
            AuditAction::BoxPurchase,
            Some("box_purchase".to_string()),
            Some(raffle_id),
            None,
            Some(purchase_data),
            ip_address,
            user_agent,
        ).await
    }

    /// Log payment transaction
    pub async fn log_payment(
        pool: &PgPool,
        user_id: Option<Uuid>,
        transaction_id: Uuid,
        amount: rust_decimal::Decimal,
        payment_method: String,
        ip_address: Option<IpAddr>,
        user_agent: Option<String>,
    ) -> Result<Self, AppError> {
        let payment_data = serde_json::json!({
            "transaction_id": transaction_id,
            "amount": amount,
            "payment_method": payment_method
        });

        Self::create(
            pool,
            user_id,
            AuditAction::Payment,
            Some("transaction".to_string()),
            Some(transaction_id),
            None,
            Some(payment_data),
            ip_address,
            user_agent,
        ).await
    }

    /// Log security event
    pub async fn log_security_event(
        pool: &PgPool,
        user_id: Option<Uuid>,
        event_type: String,
        event_data: serde_json::Value,
        ip_address: Option<IpAddr>,
        user_agent: Option<String>,
    ) -> Result<Self, AppError> {
        let security_data = serde_json::json!({
            "event_type": event_type,
            "event_data": event_data
        });

        Self::create(
            pool,
            user_id,
            AuditAction::SecurityEvent,
            Some("security".to_string()),
            None,
            None,
            Some(security_data),
            ip_address,
            user_agent,
        ).await
    }
}

impl AuditAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            AuditAction::Create => "create",
            AuditAction::Update => "update",
            AuditAction::Delete => "delete",
            AuditAction::Login => "login",
            AuditAction::Logout => "logout",
            AuditAction::PasswordChange => "password_change",
            AuditAction::EmailChange => "email_change",
            AuditAction::RoleChange => "role_change",
            AuditAction::Payment => "payment",
            AuditAction::Refund => "refund",
            AuditAction::Withdrawal => "withdrawal",
            AuditAction::CreditIssue => "credit_issue",
            AuditAction::RaffleCreate => "raffle_create",
            AuditAction::RaffleComplete => "raffle_complete",
            AuditAction::BoxPurchase => "box_purchase",
            AuditAction::AdminAction => "admin_action",
            AuditAction::SecurityEvent => "security_event",
        }
    }
}