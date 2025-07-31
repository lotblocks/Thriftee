use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;
use crate::error::AppError;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct SellerSubscription {
    pub id: Uuid,
    pub name: String,
    pub monthly_fee: Decimal,
    pub listing_fee_percentage: Decimal,
    pub transaction_fee_percentage: Decimal,
    pub max_listings: Option<i32>,
    pub features: Option<serde_json::Value>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl SellerSubscription {
    /// Create a new seller subscription tier
    pub async fn create(
        pool: &PgPool,
        name: String,
        monthly_fee: Decimal,
        listing_fee_percentage: Decimal,
        transaction_fee_percentage: Decimal,
        max_listings: Option<i32>,
        features: Option<serde_json::Value>,
    ) -> Result<Self, AppError> {
        let subscription = sqlx::query_as!(
            SellerSubscription,
            r#"
            INSERT INTO seller_subscriptions (name, monthly_fee, listing_fee_percentage, transaction_fee_percentage, max_listings, features)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, name, monthly_fee, listing_fee_percentage, transaction_fee_percentage, max_listings, features, is_active, created_at, updated_at
            "#,
            name,
            monthly_fee,
            listing_fee_percentage,
            transaction_fee_percentage,
            max_listings,
            features
        )
        .fetch_one(pool)
        .await?;

        Ok(subscription)
    }

    /// Find subscription by ID
    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Self>, AppError> {
        let subscription = sqlx::query_as!(
            SellerSubscription,
            r#"
            SELECT id, name, monthly_fee, listing_fee_percentage, transaction_fee_percentage, max_listings, features, is_active, created_at, updated_at
            FROM seller_subscriptions 
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(pool)
        .await?;

        Ok(subscription)
    }

    /// Find all active subscriptions
    pub async fn find_active(pool: &PgPool) -> Result<Vec<Self>, AppError> {
        let subscriptions = sqlx::query_as!(
            SellerSubscription,
            r#"
            SELECT id, name, monthly_fee, listing_fee_percentage, transaction_fee_percentage, max_listings, features, is_active, created_at, updated_at
            FROM seller_subscriptions 
            WHERE is_active = true
            ORDER BY monthly_fee ASC
            "#
        )
        .fetch_all(pool)
        .await?;

        Ok(subscriptions)
    }

    /// Find subscription by name
    pub async fn find_by_name(pool: &PgPool, name: &str) -> Result<Option<Self>, AppError> {
        let subscription = sqlx::query_as!(
            SellerSubscription,
            r#"
            SELECT id, name, monthly_fee, listing_fee_percentage, transaction_fee_percentage, max_listings, features, is_active, created_at, updated_at
            FROM seller_subscriptions 
            WHERE name = $1 AND is_active = true
            "#,
            name
        )
        .fetch_optional(pool)
        .await?;

        Ok(subscription)
    }

    /// Update subscription details
    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        name: Option<String>,
        monthly_fee: Option<Decimal>,
        listing_fee_percentage: Option<Decimal>,
        transaction_fee_percentage: Option<Decimal>,
        max_listings: Option<Option<i32>>,
        features: Option<serde_json::Value>,
    ) -> Result<Self, AppError> {
        let subscription = sqlx::query_as!(
            SellerSubscription,
            r#"
            UPDATE seller_subscriptions 
            SET 
                name = COALESCE($2, name),
                monthly_fee = COALESCE($3, monthly_fee),
                listing_fee_percentage = COALESCE($4, listing_fee_percentage),
                transaction_fee_percentage = COALESCE($5, transaction_fee_percentage),
                max_listings = COALESCE($6, max_listings),
                features = COALESCE($7, features),
                updated_at = NOW()
            WHERE id = $1
            RETURNING id, name, monthly_fee, listing_fee_percentage, transaction_fee_percentage, max_listings, features, is_active, created_at, updated_at
            "#,
            id,
            name,
            monthly_fee,
            listing_fee_percentage,
            transaction_fee_percentage,
            max_listings,
            features
        )
        .fetch_one(pool)
        .await?;

        Ok(subscription)
    }

    /// Deactivate subscription
    pub async fn deactivate(pool: &PgPool, id: Uuid) -> Result<(), AppError> {
        sqlx::query!(
            "UPDATE seller_subscriptions SET is_active = false, updated_at = NOW() WHERE id = $1",
            id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Activate subscription
    pub async fn activate(pool: &PgPool, id: Uuid) -> Result<(), AppError> {
        sqlx::query!(
            "UPDATE seller_subscriptions SET is_active = true, updated_at = NOW() WHERE id = $1",
            id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Calculate listing fee for an item
    pub fn calculate_listing_fee(&self, item_value: Decimal) -> Decimal {
        item_value * self.listing_fee_percentage / Decimal::new(100, 0)
    }

    /// Calculate transaction fee for a sale
    pub fn calculate_transaction_fee(&self, sale_amount: Decimal) -> Decimal {
        sale_amount * self.transaction_fee_percentage / Decimal::new(100, 0)
    }

    /// Check if subscription allows more listings
    pub async fn can_create_listing(
        &self,
        pool: &PgPool,
        seller_id: Uuid,
    ) -> Result<bool, AppError> {
        if let Some(max_listings) = self.max_listings {
            let current_listings: i64 = sqlx::query_scalar!(
                "SELECT COUNT(*) FROM items WHERE seller_id = $1 AND status != 'sold'",
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

    /// Get subscription statistics
    pub async fn get_statistics(pool: &PgPool, id: Uuid) -> Result<SubscriptionStatistics, AppError> {
        let stats = sqlx::query!(
            r#"
            SELECT 
                COUNT(s.id) as subscriber_count,
                SUM(CASE WHEN s.subscription_expires_at > NOW() THEN 1 ELSE 0 END) as active_subscribers
            FROM sellers s
            WHERE s.current_subscription_id = $1
            "#,
            id
        )
        .fetch_one(pool)
        .await?;

        Ok(SubscriptionStatistics {
            subscriber_count: stats.subscriber_count.unwrap_or(0) as u32,
            active_subscribers: stats.active_subscribers.unwrap_or(0) as u32,
        })
    }

    /// Delete subscription (admin only)
    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<(), AppError> {
        // Check if any sellers are using this subscription
        let usage_count: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM sellers WHERE current_subscription_id = $1",
            id
        )
        .fetch_one(pool)
        .await?
        .unwrap_or(0);

        if usage_count > 0 {
            return Err(AppError::Validation(
                "Cannot delete subscription that is currently in use".to_string(),
            ));
        }

        sqlx::query!("DELETE FROM seller_subscriptions WHERE id = $1", id)
            .execute(pool)
            .await?;

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionStatistics {
    pub subscriber_count: u32,
    pub active_subscribers: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[sqlx::test]
    async fn test_seller_subscription_crud(pool: PgPool) {
        // Test create
        let subscription = SellerSubscription::create(
            &pool,
            "Basic Plan".to_string(),
            Decimal::new(2999, 2), // $29.99
            Decimal::new(500, 2),  // 5%
            Decimal::new(300, 2),  // 3%
            Some(10),
            Some(serde_json::json!({"feature1": true, "feature2": false})),
        )
        .await
        .expect("Failed to create subscription");

        assert_eq!(subscription.name, "Basic Plan");
        assert_eq!(subscription.monthly_fee, Decimal::new(2999, 2));

        // Test find by ID
        let found = SellerSubscription::find_by_id(&pool, subscription.id)
            .await
            .expect("Failed to find subscription")
            .expect("Subscription not found");

        assert_eq!(found.id, subscription.id);

        // Test find by name
        let found_by_name = SellerSubscription::find_by_name(&pool, "Basic Plan")
            .await
            .expect("Failed to find subscription by name")
            .expect("Subscription not found");

        assert_eq!(found_by_name.id, subscription.id);

        // Test update
        let updated = SellerSubscription::update(
            &pool,
            subscription.id,
            Some("Premium Plan".to_string()),
            Some(Decimal::new(4999, 2)),
            None,
            None,
            None,
            None,
        )
        .await
        .expect("Failed to update subscription");

        assert_eq!(updated.name, "Premium Plan");
        assert_eq!(updated.monthly_fee, Decimal::new(4999, 2));

        // Test fee calculations
        let item_value = Decimal::new(10000, 2); // $100.00
        let listing_fee = updated.calculate_listing_fee(item_value);
        let expected_listing_fee = Decimal::new(500, 2); // 5% of $100 = $5.00
        assert_eq!(listing_fee, expected_listing_fee);

        let sale_amount = Decimal::new(5000, 2); // $50.00
        let transaction_fee = updated.calculate_transaction_fee(sale_amount);
        let expected_transaction_fee = Decimal::new(150, 2); // 3% of $50 = $1.50
        assert_eq!(transaction_fee, expected_transaction_fee);
    }
}