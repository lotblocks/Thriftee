use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;
use crate::error::AppError;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct SystemSetting {
    pub id: Uuid,
    pub key: String,
    pub value: serde_json::Value,
    pub description: Option<String>,
    pub is_public: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl SystemSetting {
    /// Create or update a system setting
    pub async fn upsert(
        pool: &PgPool,
        key: String,
        value: serde_json::Value,
        description: Option<String>,
        is_public: bool,
    ) -> Result<Self, AppError> {
        let setting = sqlx::query_as!(
            SystemSetting,
            r#"
            INSERT INTO system_settings (key, value, description, is_public)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (key) 
            DO UPDATE SET 
                value = EXCLUDED.value,
                description = EXCLUDED.description,
                is_public = EXCLUDED.is_public,
                updated_at = NOW()
            RETURNING id, key, value, description, is_public, created_at, updated_at
            "#,
            key,
            value,
            description,
            is_public
        )
        .fetch_one(pool)
        .await?;

        Ok(setting)
    }

    /// Get a setting by key
    pub async fn get_by_key(pool: &PgPool, key: &str) -> Result<Option<Self>, AppError> {
        let setting = sqlx::query_as!(
            SystemSetting,
            "SELECT id, key, value, description, is_public, created_at, updated_at FROM system_settings WHERE key = $1",
            key
        )
        .fetch_optional(pool)
        .await?;

        Ok(setting)
    }

    /// Get all public settings
    pub async fn get_public_settings(pool: &PgPool) -> Result<Vec<Self>, AppError> {
        let settings = sqlx::query_as!(
            SystemSetting,
            "SELECT id, key, value, description, is_public, created_at, updated_at FROM system_settings WHERE is_public = true ORDER BY key"
        )
        .fetch_all(pool)
        .await?;

        Ok(settings)
    }

    /// Get all settings (admin only)
    pub async fn get_all_settings(pool: &PgPool) -> Result<Vec<Self>, AppError> {
        let settings = sqlx::query_as!(
            SystemSetting,
            "SELECT id, key, value, description, is_public, created_at, updated_at FROM system_settings ORDER BY key"
        )
        .fetch_all(pool)
        .await?;

        Ok(settings)
    }

    /// Delete a setting
    pub async fn delete_by_key(pool: &PgPool, key: &str) -> Result<bool, AppError> {
        let result = sqlx::query!(
            "DELETE FROM system_settings WHERE key = $1",
            key
        )
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Get setting value as string
    pub fn get_string_value(&self) -> Option<String> {
        self.value.as_str().map(|s| s.to_string())
    }

    /// Get setting value as integer
    pub fn get_int_value(&self) -> Option<i64> {
        self.value.as_i64()
    }

    /// Get setting value as float
    pub fn get_float_value(&self) -> Option<f64> {
        self.value.as_f64()
    }

    /// Get setting value as boolean
    pub fn get_bool_value(&self) -> Option<bool> {
        self.value.as_bool()
    }

    /// Get setting value as decimal
    pub fn get_decimal_value(&self) -> Option<rust_decimal::Decimal> {
        if let Some(s) = self.value.as_str() {
            s.parse().ok()
        } else if let Some(f) = self.value.as_f64() {
            rust_decimal::Decimal::try_from(f).ok()
        } else {
            None
        }
    }
}

/// Helper struct for common system settings
pub struct SystemSettings;

impl SystemSettings {
    /// Get platform name
    pub async fn get_platform_name(pool: &PgPool) -> Result<String, AppError> {
        let setting = SystemSetting::get_by_key(pool, "platform_name").await?;
        Ok(setting
            .and_then(|s| s.get_string_value())
            .unwrap_or_else(|| "Raffle Shopping Platform".to_string()))
    }

    /// Get maximum boxes per raffle
    pub async fn get_max_boxes_per_raffle(pool: &PgPool) -> Result<i32, AppError> {
        let setting = SystemSetting::get_by_key(pool, "max_boxes_per_raffle").await?;
        Ok(setting
            .and_then(|s| s.get_int_value())
            .unwrap_or(10000) as i32)
    }

    /// Get minimum box price
    pub async fn get_min_box_price(pool: &PgPool) -> Result<rust_decimal::Decimal, AppError> {
        let setting = SystemSetting::get_by_key(pool, "min_box_price").await?;
        Ok(setting
            .and_then(|s| s.get_decimal_value())
            .unwrap_or_else(|| rust_decimal::Decimal::new(100, 2))) // $1.00
    }

    /// Get credit expiration days
    pub async fn get_credit_expiration_days(pool: &PgPool) -> Result<i32, AppError> {
        let setting = SystemSetting::get_by_key(pool, "credit_expiration_days").await?;
        Ok(setting
            .and_then(|s| s.get_int_value())
            .unwrap_or(365) as i32)
    }

    /// Get free item credit threshold
    pub async fn get_free_item_credit_threshold(pool: &PgPool) -> Result<rust_decimal::Decimal, AppError> {
        let setting = SystemSetting::get_by_key(pool, "free_item_credit_threshold").await?;
        Ok(setting
            .and_then(|s| s.get_decimal_value())
            .unwrap_or_else(|| rust_decimal::Decimal::new(1000, 2))) // $10.00
    }

    /// Get platform fee percentage
    pub async fn get_platform_fee_percentage(pool: &PgPool) -> Result<rust_decimal::Decimal, AppError> {
        let setting = SystemSetting::get_by_key(pool, "platform_fee_percentage").await?;
        Ok(setting
            .and_then(|s| s.get_decimal_value())
            .unwrap_or_else(|| rust_decimal::Decimal::new(500, 2))) // 5.00%
    }

    /// Set platform name
    pub async fn set_platform_name(pool: &PgPool, name: String) -> Result<SystemSetting, AppError> {
        SystemSetting::upsert(
            pool,
            "platform_name".to_string(),
            serde_json::Value::String(name),
            Some("The name of the platform".to_string()),
            true,
        ).await
    }

    /// Set maximum boxes per raffle
    pub async fn set_max_boxes_per_raffle(pool: &PgPool, max_boxes: i32) -> Result<SystemSetting, AppError> {
        SystemSetting::upsert(
            pool,
            "max_boxes_per_raffle".to_string(),
            serde_json::Value::Number(serde_json::Number::from(max_boxes)),
            Some("Maximum number of boxes allowed per raffle".to_string()),
            false,
        ).await
    }

    /// Set minimum box price
    pub async fn set_min_box_price(pool: &PgPool, min_price: rust_decimal::Decimal) -> Result<SystemSetting, AppError> {
        SystemSetting::upsert(
            pool,
            "min_box_price".to_string(),
            serde_json::Value::String(min_price.to_string()),
            Some("Minimum price per box in credits".to_string()),
            false,
        ).await
    }

    /// Set credit expiration days
    pub async fn set_credit_expiration_days(pool: &PgPool, days: i32) -> Result<SystemSetting, AppError> {
        SystemSetting::upsert(
            pool,
            "credit_expiration_days".to_string(),
            serde_json::Value::Number(serde_json::Number::from(days)),
            Some("Default number of days before credits expire".to_string()),
            false,
        ).await
    }

    /// Set platform fee percentage
    pub async fn set_platform_fee_percentage(pool: &PgPool, percentage: rust_decimal::Decimal) -> Result<SystemSetting, AppError> {
        SystemSetting::upsert(
            pool,
            "platform_fee_percentage".to_string(),
            serde_json::Value::String(percentage.to_string()),
            Some("Platform fee percentage on completed raffles".to_string()),
            false,
        ).await
    }
}