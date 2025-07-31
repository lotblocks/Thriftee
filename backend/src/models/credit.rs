use chrono::{DateTime, Utc};
use raffle_platform_shared::{CreditSource, CreditType, CreditResponse};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;
use crate::error::AppError;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct UserCredit {
    pub id: Uuid,
    pub user_id: Uuid,
    pub amount: Decimal,
    pub source: CreditSource,
    pub credit_type: CreditType,
    pub redeemable_on_item_id: Option<Uuid>,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_transferable: bool,
    pub is_used: bool,
    pub used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl UserCredit {
    /// Create new credits for a user
    pub async fn create(
        pool: &PgPool,
        user_id: Uuid,
        amount: Decimal,
        source: CreditSource,
        credit_type: CreditType,
        redeemable_on_item_id: Option<Uuid>,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<Self, AppError> {
        let credit = sqlx::query_as!(
            UserCredit,
            r#"
            INSERT INTO user_credits (user_id, amount, source, credit_type, redeemable_on_item_id, expires_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING 
                id, user_id, amount, 
                source as "source: CreditSource", 
                credit_type as "credit_type: CreditType",
                redeemable_on_item_id, expires_at, is_transferable, is_used, used_at, created_at
            "#,
            user_id,
            amount,
            source as CreditSource,
            credit_type as CreditType,
            redeemable_on_item_id,
            expires_at
        )
        .fetch_one(pool)
        .await?;

        Ok(credit)
    }

    /// Find credits by user
    pub async fn find_by_user(
        pool: &PgPool,
        user_id: Uuid,
        include_used: bool,
    ) -> Result<Vec<Self>, AppError> {
        let credits = if include_used {
            sqlx::query_as!(
                UserCredit,
                r#"
                SELECT 
                    id, user_id, amount, 
                    source as "source: CreditSource", 
                    credit_type as "credit_type: CreditType",
                    redeemable_on_item_id, expires_at, is_transferable, is_used, used_at, created_at
                FROM user_credits 
                WHERE user_id = $1
                ORDER BY created_at DESC
                "#,
                user_id
            )
            .fetch_all(pool)
            .await?
        } else {
            sqlx::query_as!(
                UserCredit,
                r#"
                SELECT 
                    id, user_id, amount, 
                    source as "source: CreditSource", 
                    credit_type as "credit_type: CreditType",
                    redeemable_on_item_id, expires_at, is_transferable, is_used, used_at, created_at
                FROM user_credits 
                WHERE user_id = $1 AND is_used = false
                ORDER BY created_at DESC
                "#,
                user_id
            )
            .fetch_all(pool)
            .await?
        };

        Ok(credits)
    }

    /// Find available credits for a user (unused and not expired)
    pub async fn find_available_by_user(
        pool: &PgPool,
        user_id: Uuid,
        credit_type: Option<CreditType>,
        item_id: Option<Uuid>,
    ) -> Result<Vec<Self>, AppError> {
        let credits = match (credit_type, item_id) {
            (Some(ct), Some(item)) => {
                sqlx::query_as!(
                    UserCredit,
                    r#"
                    SELECT 
                        id, user_id, amount, 
                        source as "source: CreditSource", 
                        credit_type as "credit_type: CreditType",
                        redeemable_on_item_id, expires_at, is_transferable, is_used, used_at, created_at
                    FROM user_credits 
                    WHERE user_id = $1 AND is_used = false 
                    AND (expires_at IS NULL OR expires_at > NOW())
                    AND credit_type = $2
                    AND (redeemable_on_item_id IS NULL OR redeemable_on_item_id = $3)
                    ORDER BY expires_at ASC NULLS LAST, created_at ASC
                    "#,
                    user_id,
                    ct as CreditType,
                    item
                )
                .fetch_all(pool)
                .await?
            }
            (Some(ct), None) => {
                sqlx::query_as!(
                    UserCredit,
                    r#"
                    SELECT 
                        id, user_id, amount, 
                        source as "source: CreditSource", 
                        credit_type as "credit_type: CreditType",
                        redeemable_on_item_id, expires_at, is_transferable, is_used, used_at, created_at
                    FROM user_credits 
                    WHERE user_id = $1 AND is_used = false 
                    AND (expires_at IS NULL OR expires_at > NOW())
                    AND credit_type = $2
                    ORDER BY expires_at ASC NULLS LAST, created_at ASC
                    "#,
                    user_id,
                    ct as CreditType
                )
                .fetch_all(pool)
                .await?
            }
            (None, Some(item)) => {
                sqlx::query_as!(
                    UserCredit,
                    r#"
                    SELECT 
                        id, user_id, amount, 
                        source as "source: CreditSource", 
                        credit_type as "credit_type: CreditType",
                        redeemable_on_item_id, expires_at, is_transferable, is_used, used_at, created_at
                    FROM user_credits 
                    WHERE user_id = $1 AND is_used = false 
                    AND (expires_at IS NULL OR expires_at > NOW())
                    AND (redeemable_on_item_id IS NULL OR redeemable_on_item_id = $2)
                    ORDER BY expires_at ASC NULLS LAST, created_at ASC
                    "#,
                    user_id,
                    item
                )
                .fetch_all(pool)
                .await?
            }
            (None, None) => {
                sqlx::query_as!(
                    UserCredit,
                    r#"
                    SELECT 
                        id, user_id, amount, 
                        source as "source: CreditSource", 
                        credit_type as "credit_type: CreditType",
                        redeemable_on_item_id, expires_at, is_transferable, is_used, used_at, created_at
                    FROM user_credits 
                    WHERE user_id = $1 AND is_used = false 
                    AND (expires_at IS NULL OR expires_at > NOW())
                    ORDER BY expires_at ASC NULLS LAST, created_at ASC
                    "#,
                    user_id
                )
                .fetch_all(pool)
                .await?
            }
        };

        Ok(credits)
    }

    /// Find expiring credits (within specified days)
    pub async fn find_expiring(
        pool: &PgPool,
        user_id: Uuid,
        days: i64,
    ) -> Result<Vec<Self>, AppError> {
        let credits = sqlx::query_as!(
            UserCredit,
            r#"
            SELECT 
                id, user_id, amount, 
                source as "source: CreditSource", 
                credit_type as "credit_type: CreditType",
                redeemable_on_item_id, expires_at, is_transferable, is_used, used_at, created_at
            FROM user_credits 
            WHERE user_id = $1 AND is_used = false 
            AND expires_at IS NOT NULL 
            AND expires_at <= NOW() + INTERVAL '%d days'
            AND expires_at > NOW()
            ORDER BY expires_at ASC
            "#,
            user_id,
            days
        )
        .fetch_all(pool)
        .await?;

        Ok(credits)
    }

    /// Mark credits as used
    pub async fn mark_as_used(
        pool: &PgPool,
        credit_ids: &[Uuid],
        used_amount: Decimal,
    ) -> Result<Vec<Self>, AppError> {
        let mut remaining_amount = used_amount;
        let mut used_credits = Vec::new();

        for credit_id in credit_ids {
            if remaining_amount <= Decimal::ZERO {
                break;
            }

            let credit = sqlx::query_as!(
                UserCredit,
                r#"
                SELECT 
                    id, user_id, amount, 
                    source as "source: CreditSource", 
                    credit_type as "credit_type: CreditType",
                    redeemable_on_item_id, expires_at, is_transferable, is_used, used_at, created_at
                FROM user_credits 
                WHERE id = $1 AND is_used = false
                "#,
                credit_id
            )
            .fetch_optional(pool)
            .await?;

            if let Some(mut credit) = credit {
                if credit.amount <= remaining_amount {
                    // Use entire credit
                    sqlx::query!(
                        "UPDATE user_credits SET is_used = true, used_at = NOW() WHERE id = $1",
                        credit.id
                    )
                    .execute(pool)
                    .await?;

                    remaining_amount -= credit.amount;
                    credit.is_used = true;
                    credit.used_at = Some(Utc::now());
                    used_credits.push(credit);
                } else {
                    // Partial use - split the credit
                    let used_portion = remaining_amount;
                    let remaining_portion = credit.amount - remaining_amount;

                    // Mark original as used
                    sqlx::query!(
                        "UPDATE user_credits SET amount = $1, is_used = true, used_at = NOW() WHERE id = $2",
                        used_portion,
                        credit.id
                    )
                    .execute(pool)
                    .await?;

                    // Create new credit for remaining amount
                    sqlx::query!(
                        r#"
                        INSERT INTO user_credits (user_id, amount, source, credit_type, redeemable_on_item_id, expires_at, is_transferable)
                        VALUES ($1, $2, $3, $4, $5, $6, $7)
                        "#,
                        credit.user_id,
                        remaining_portion,
                        credit.source as CreditSource,
                        credit.credit_type as CreditType,
                        credit.redeemable_on_item_id,
                        credit.expires_at,
                        credit.is_transferable
                    )
                    .execute(pool)
                    .await?;

                    credit.amount = used_portion;
                    credit.is_used = true;
                    credit.used_at = Some(Utc::now());
                    used_credits.push(credit);
                    remaining_amount = Decimal::ZERO;
                }
            }
        }

        if remaining_amount > Decimal::ZERO {
            return Err(AppError::Validation("Insufficient credits available".to_string()));
        }

        Ok(used_credits)
    }

    /// Calculate total available credits for a user
    pub async fn calculate_total_available(
        pool: &PgPool,
        user_id: Uuid,
        credit_type: Option<CreditType>,
        item_id: Option<Uuid>,
    ) -> Result<Decimal, AppError> {
        let total = match (credit_type, item_id) {
            (Some(ct), Some(item)) => {
                sqlx::query_scalar!(
                    r#"
                    SELECT COALESCE(SUM(amount), 0) 
                    FROM user_credits 
                    WHERE user_id = $1 AND is_used = false 
                    AND (expires_at IS NULL OR expires_at > NOW())
                    AND credit_type = $2
                    AND (redeemable_on_item_id IS NULL OR redeemable_on_item_id = $3)
                    "#,
                    user_id,
                    ct as CreditType,
                    item
                )
                .fetch_one(pool)
                .await?
            }
            (Some(ct), None) => {
                sqlx::query_scalar!(
                    r#"
                    SELECT COALESCE(SUM(amount), 0) 
                    FROM user_credits 
                    WHERE user_id = $1 AND is_used = false 
                    AND (expires_at IS NULL OR expires_at > NOW())
                    AND credit_type = $2
                    "#,
                    user_id,
                    ct as CreditType
                )
                .fetch_one(pool)
                .await?
            }
            (None, Some(item)) => {
                sqlx::query_scalar!(
                    r#"
                    SELECT COALESCE(SUM(amount), 0) 
                    FROM user_credits 
                    WHERE user_id = $1 AND is_used = false 
                    AND (expires_at IS NULL OR expires_at > NOW())
                    AND (redeemable_on_item_id IS NULL OR redeemable_on_item_id = $2)
                    "#,
                    user_id,
                    item
                )
                .fetch_one(pool)
                .await?
            }
            (None, None) => {
                sqlx::query_scalar!(
                    r#"
                    SELECT COALESCE(SUM(amount), 0) 
                    FROM user_credits 
                    WHERE user_id = $1 AND is_used = false 
                    AND (expires_at IS NULL OR expires_at > NOW())
                    "#,
                    user_id
                )
                .fetch_one(pool)
                .await?
            }
        };

        Ok(total.unwrap_or(Decimal::ZERO))
    }

    /// Clean up expired credits
    pub async fn cleanup_expired(pool: &PgPool) -> Result<u64, AppError> {
        let result = sqlx::query!(
            "DELETE FROM user_credits WHERE expires_at < NOW() AND is_used = false"
        )
        .execute(pool)
        .await?;

        Ok(result.rows_affected())
    }

    /// Find credit by ID
    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Self>, AppError> {
        let credit = sqlx::query_as!(
            UserCredit,
            r#"
            SELECT 
                id, user_id, amount, 
                source as "source: CreditSource", 
                credit_type as "credit_type: CreditType",
                redeemable_on_item_id, expires_at, is_transferable, is_used, used_at, created_at
            FROM user_credits 
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(pool)
        .await?;

        Ok(credit)
    }

    /// Convert to response DTO
    pub fn to_response(&self) -> CreditResponse {
        CreditResponse {
            id: self.id,
            user_id: self.user_id,
            amount: self.amount,
            source: self.source,
            credit_type: self.credit_type,
            redeemable_on_item_id: self.redeemable_on_item_id,
            expires_at: self.expires_at,
            is_transferable: self.is_transferable,
            is_used: self.is_used,
            used_at: self.used_at,
            created_at: self.created_at,
        }
    }

    /// Check if credit is expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            expires_at < Utc::now()
        } else {
            false
        }
    }

    /// Check if credit is available for use
    pub fn is_available(&self) -> bool {
        !self.is_used && !self.is_expired()
    }

    /// Check if credit can be used for a specific item
    pub fn can_be_used_for_item(&self, item_id: Option<Uuid>) -> bool {
        if !self.is_available() {
            return false;
        }

        match (self.credit_type, self.redeemable_on_item_id, item_id) {
            (CreditType::General, _, _) => true,
            (CreditType::ItemSpecific, Some(redeemable_item), Some(target_item)) => {
                redeemable_item == target_item
            }
            (CreditType::ItemSpecific, None, _) => true, // General item-specific credit
            _ => false,
        }
    }
}