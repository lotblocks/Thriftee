use chrono::{DateTime, Utc};
use raffle_platform_shared::{CreateSellerRequest, SellerResponse};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;
use crate::error::AppError;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Seller {
    pub id: Uuid,
    pub user_id: Uuid,
    pub company_name: Option<String>,
    pub description: Option<String>,
    pub payout_details: Option<serde_json::Value>,
    pub current_subscription_id: Option<Uuid>,
    pub subscription_expires_at: Option<DateTime<Utc>>,
    pub is_verified: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Seller {
    /// Create a new seller profile
    pub async fn create(
        pool: &PgPool,
        user_id: Uuid,
        request: CreateSellerRequest,
    ) -> Result<Self, AppError> {
        let seller = sqlx::query_as!(
            Seller,
            r#"
            INSERT INTO sellers (user_id, company_name, description, current_subscription_id)
            VALUES ($1, $2, $3, $4)
            RETURNING 
                id, user_id, company_name, description, payout_details,
                current_subscription_id, subscription_expires_at, is_verified,
                created_at, updated_at
            "#,
            user_id,
            request.company_name,
            request.description,
            request.subscription_id
        )
        .fetch_one(pool)
        .await?;

        Ok(seller)
    }

    /// Find seller by user ID
    pub async fn find_by_user_id(pool: &PgPool, user_id: Uuid) -> Result<Option<Self>, AppError> {
        let seller = sqlx::query_as!(
            Seller,
            r#"
            SELECT 
                id, user_id, company_name, description, payout_details,
                current_subscription_id, subscription_expires_at, is_verified,
                created_at, updated_at
            FROM sellers 
            WHERE user_id = $1
            "#,
            user_id
        )
        .fetch_optional(pool)
        .await?;

        Ok(seller)
    }

    /// Find seller by ID
    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Self>, AppError> {
        let seller = sqlx::query_as!(
            Seller,
            r#"
            SELECT 
                id, user_id, company_name, description, payout_details,
                current_subscription_id, subscription_expires_at, is_verified,
                created_at, updated_at
            FROM sellers 
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(pool)
        .await?;

        Ok(seller)
    }

    /// Find verified sellers
    pub async fn find_verified(
        pool: &PgPool,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Self>, AppError> {
        let sellers = sqlx::query_as!(
            Seller,
            r#"
            SELECT 
                id, user_id, company_name, description, payout_details,
                current_subscription_id, subscription_expires_at, is_verified,
                created_at, updated_at
            FROM sellers 
            WHERE is_verified = true
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
            limit,
            offset
        )
        .fetch_all(pool)
        .await?;

        Ok(sellers)
    }

    /// Update seller verification status
    pub async fn update_verification_status(
        pool: &PgPool,
        id: Uuid,
        is_verified: bool,
    ) -> Result<(), AppError> {
        sqlx::query!(
            "UPDATE sellers SET is_verified = $1, updated_at = NOW() WHERE id = $2",
            is_verified,
            id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Update seller subscription
    pub async fn update_subscription(
        pool: &PgPool,
        id: Uuid,
        subscription_id: Uuid,
        expires_at: DateTime<Utc>,
    ) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            UPDATE sellers 
            SET current_subscription_id = $1, subscription_expires_at = $2, updated_at = NOW() 
            WHERE id = $3
            "#,
            subscription_id,
            expires_at,
            id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Update payout details
    pub async fn update_payout_details(
        pool: &PgPool,
        id: Uuid,
        payout_details: serde_json::Value,
    ) -> Result<(), AppError> {
        sqlx::query!(
            "UPDATE sellers SET payout_details = $1, updated_at = NOW() WHERE id = $2",
            payout_details,
            id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Update seller profile
    pub async fn update_profile(
        pool: &PgPool,
        id: Uuid,
        company_name: Option<String>,
        description: Option<String>,
    ) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            UPDATE sellers 
            SET company_name = $1, description = $2, updated_at = NOW() 
            WHERE id = $3
            "#,
            company_name,
            description,
            id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Check if subscription is active
    pub fn has_active_subscription(&self) -> bool {
        if let Some(expires_at) = self.subscription_expires_at {
            expires_at > Utc::now()
        } else {
            false
        }
    }

    /// Convert to response DTO
    pub fn to_response(&self) -> SellerResponse {
        SellerResponse {
            id: self.id,
            user_id: self.user_id,
            company_name: self.company_name.clone(),
            description: self.description.clone(),
            current_subscription_id: self.current_subscription_id,
            subscription_expires_at: self.subscription_expires_at,
            is_verified: self.is_verified,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct SellerSubscription {
    pub id: Uuid,
    pub name: String,
    pub monthly_fee: Decimal,
    pub listing_fee_percentage: Decimal,
    pub transaction_fee_percentage: Decimal,
    pub max_listings: Option<i32>,
    pub features: serde_json::Value,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl SellerSubscription {
    /// Find all active subscription tiers
    pub async fn find_active(pool: &PgPool) -> Result<Vec<Self>, AppError> {
        let subscriptions = sqlx::query_as!(
            SellerSubscription,
            r#"
            SELECT 
                id, name, monthly_fee, listing_fee_percentage, transaction_fee_percentage,
                max_listings, features, is_active, created_at, updated_at
            FROM seller_subscriptions 
            WHERE is_active = true
            ORDER BY monthly_fee ASC
            "#
        )
        .fetch_all(pool)
        .await?;

        Ok(subscriptions)
    }

    /// Find subscription by ID
    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Self>, AppError> {
        let subscription = sqlx::query_as!(
            SellerSubscription,
            r#"
            SELECT 
                id, name, monthly_fee, listing_fee_percentage, transaction_fee_percentage,
                max_listings, features, is_active, created_at, updated_at
            FROM seller_subscriptions 
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(pool)
        .await?;

        Ok(subscription)
    }

    /// Calculate listing fee for an item
    pub fn calculate_listing_fee(&self, item_value: Decimal) -> Decimal {
        (item_value * self.listing_fee_percentage) / Decimal::from(100)
    }

    /// Calculate transaction fee for a sale
    pub fn calculate_transaction_fee(&self, sale_amount: Decimal) -> Decimal {
        (sale_amount * self.transaction_fee_percentage) / Decimal::from(100)
    }

    /// Check if seller can list more items
    pub async fn can_list_more_items(
        &self,
        pool: &PgPool,
        seller_id: Uuid,
    ) -> Result<bool, AppError> {
        if let Some(max_listings) = self.max_listings {
            let current_listings = sqlx::query_scalar!(
                "SELECT COUNT(*) FROM items WHERE seller_id = $1 AND status = 'available'",
                seller_id
            )
            .fetch_one(pool)
            .await?
            .unwrap_or(0);

            Ok(current_listings < max_listings as i64)
        } else {
            Ok(true) // Unlimited listings
        }
    }
}