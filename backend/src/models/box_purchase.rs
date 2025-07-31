use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;
use crate::error::AppError;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct BoxPurchase {
    pub id: Uuid,
    pub raffle_id: Uuid,
    pub user_id: Uuid,
    pub box_number: i32,
    pub purchase_price_in_credits: Decimal,
    pub transaction_id: Option<Uuid>,
    pub blockchain_tx_hash: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl BoxPurchase {
    /// Create a new box purchase
    pub async fn create(
        pool: &PgPool,
        raffle_id: Uuid,
        user_id: Uuid,
        box_number: i32,
        purchase_price_in_credits: Decimal,
        transaction_id: Option<Uuid>,
    ) -> Result<Self, AppError> {
        let box_purchase = sqlx::query_as!(
            BoxPurchase,
            r#"
            INSERT INTO box_purchases (raffle_id, user_id, box_number, purchase_price_in_credits, transaction_id)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, raffle_id, user_id, box_number, purchase_price_in_credits, transaction_id, blockchain_tx_hash, created_at
            "#,
            raffle_id,
            user_id,
            box_number,
            purchase_price_in_credits,
            transaction_id
        )
        .fetch_one(pool)
        .await?;

        Ok(box_purchase)
    }

    /// Find box purchase by ID
    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Self>, AppError> {
        let box_purchase = sqlx::query_as!(
            BoxPurchase,
            r#"
            SELECT id, raffle_id, user_id, box_number, purchase_price_in_credits, transaction_id, blockchain_tx_hash, created_at
            FROM box_purchases 
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(pool)
        .await?;

        Ok(box_purchase)
    }

    /// Find box purchases by raffle ID
    pub async fn find_by_raffle_id(
        pool: &PgPool,
        raffle_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Self>, AppError> {
        let box_purchases = sqlx::query_as!(
            BoxPurchase,
            r#"
            SELECT id, raffle_id, user_id, box_number, purchase_price_in_credits, transaction_id, blockchain_tx_hash, created_at
            FROM box_purchases 
            WHERE raffle_id = $1
            ORDER BY box_number ASC
            LIMIT $2 OFFSET $3
            "#,
            raffle_id,
            limit,
            offset
        )
        .fetch_all(pool)
        .await?;

        Ok(box_purchases)
    }

    /// Find box purchases by user ID
    pub async fn find_by_user_id(
        pool: &PgPool,
        user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Self>, AppError> {
        let box_purchases = sqlx::query_as!(
            BoxPurchase,
            r#"
            SELECT id, raffle_id, user_id, box_number, purchase_price_in_credits, transaction_id, blockchain_tx_hash, created_at
            FROM box_purchases 
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

        Ok(box_purchases)
    }

    /// Find box purchases by user and raffle
    pub async fn find_by_user_and_raffle(
        pool: &PgPool,
        user_id: Uuid,
        raffle_id: Uuid,
    ) -> Result<Vec<Self>, AppError> {
        let box_purchases = sqlx::query_as!(
            BoxPurchase,
            r#"
            SELECT id, raffle_id, user_id, box_number, purchase_price_in_credits, transaction_id, blockchain_tx_hash, created_at
            FROM box_purchases 
            WHERE user_id = $1 AND raffle_id = $2
            ORDER BY box_number ASC
            "#,
            user_id,
            raffle_id
        )
        .fetch_all(pool)
        .await?;

        Ok(box_purchases)
    }

    /// Check if a box number is already purchased
    pub async fn is_box_purchased(
        pool: &PgPool,
        raffle_id: Uuid,
        box_number: i32,
    ) -> Result<bool, AppError> {
        let count: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM box_purchases WHERE raffle_id = $1 AND box_number = $2",
            raffle_id,
            box_number
        )
        .fetch_one(pool)
        .await?
        .unwrap_or(0);

        Ok(count > 0)
    }

    /// Get purchased box numbers for a raffle
    pub async fn get_purchased_box_numbers(
        pool: &PgPool,
        raffle_id: Uuid,
    ) -> Result<Vec<i32>, AppError> {
        let box_numbers: Vec<i32> = sqlx::query_scalar!(
            "SELECT box_number FROM box_purchases WHERE raffle_id = $1 ORDER BY box_number ASC",
            raffle_id
        )
        .fetch_all(pool)
        .await?;

        Ok(box_numbers)
    }

    /// Get available box numbers for a raffle
    pub async fn get_available_box_numbers(
        pool: &PgPool,
        raffle_id: Uuid,
        total_boxes: i32,
    ) -> Result<Vec<i32>, AppError> {
        let purchased_numbers = Self::get_purchased_box_numbers(pool, raffle_id).await?;
        let all_numbers: Vec<i32> = (1..=total_boxes).collect();
        let available_numbers: Vec<i32> = all_numbers
            .into_iter()
            .filter(|n| !purchased_numbers.contains(n))
            .collect();

        Ok(available_numbers)
    }

    /// Update blockchain transaction hash
    pub async fn update_blockchain_tx_hash(
        pool: &PgPool,
        id: Uuid,
        blockchain_tx_hash: &str,
    ) -> Result<(), AppError> {
        sqlx::query!(
            "UPDATE box_purchases SET blockchain_tx_hash = $1 WHERE id = $2",
            blockchain_tx_hash,
            id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Get total spent by user on a raffle
    pub async fn get_user_total_spent(
        pool: &PgPool,
        user_id: Uuid,
        raffle_id: Uuid,
    ) -> Result<Decimal, AppError> {
        let total: Option<Decimal> = sqlx::query_scalar!(
            "SELECT SUM(purchase_price_in_credits) FROM box_purchases WHERE user_id = $1 AND raffle_id = $2",
            user_id,
            raffle_id
        )
        .fetch_one(pool)
        .await?;

        Ok(total.unwrap_or(Decimal::ZERO))
    }

    /// Get box purchase statistics for a raffle
    pub async fn get_raffle_statistics(
        pool: &PgPool,
        raffle_id: Uuid,
    ) -> Result<BoxPurchaseStatistics, AppError> {
        let stats = sqlx::query!(
            r#"
            SELECT 
                COUNT(*) as total_purchases,
                COUNT(DISTINCT user_id) as unique_buyers,
                SUM(purchase_price_in_credits) as total_revenue,
                AVG(purchase_price_in_credits) as average_price
            FROM box_purchases 
            WHERE raffle_id = $1
            "#,
            raffle_id
        )
        .fetch_one(pool)
        .await?;

        Ok(BoxPurchaseStatistics {
            total_purchases: stats.total_purchases.unwrap_or(0) as u32,
            unique_buyers: stats.unique_buyers.unwrap_or(0) as u32,
            total_revenue: stats.total_revenue.unwrap_or(Decimal::ZERO),
            average_price: stats.average_price.unwrap_or(Decimal::ZERO),
        })
    }

    /// Delete box purchase (for testing or admin purposes)
    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<(), AppError> {
        sqlx::query!("DELETE FROM box_purchases WHERE id = $1", id)
            .execute(pool)
            .await?;

        Ok(())
    }

    /// Bulk create box purchases (for batch operations)
    pub async fn bulk_create(
        pool: &PgPool,
        purchases: Vec<(Uuid, Uuid, i32, Decimal, Option<Uuid>)>, // (raffle_id, user_id, box_number, price, transaction_id)
    ) -> Result<Vec<Self>, AppError> {
        let mut created_purchases = Vec::new();

        // Use a transaction for atomicity
        let mut tx = pool.begin().await?;

        for (raffle_id, user_id, box_number, price, transaction_id) in purchases {
            let box_purchase = sqlx::query_as!(
                BoxPurchase,
                r#"
                INSERT INTO box_purchases (raffle_id, user_id, box_number, purchase_price_in_credits, transaction_id)
                VALUES ($1, $2, $3, $4, $5)
                RETURNING id, raffle_id, user_id, box_number, purchase_price_in_credits, transaction_id, blockchain_tx_hash, created_at
                "#,
                raffle_id,
                user_id,
                box_number,
                price,
                transaction_id
            )
            .fetch_one(&mut *tx)
            .await?;

            created_purchases.push(box_purchase);
        }

        tx.commit().await?;
        Ok(created_purchases)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoxPurchaseStatistics {
    pub total_purchases: u32,
    pub unique_buyers: u32,
    pub total_revenue: Decimal,
    pub average_price: Decimal,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{user::User, item::Item, raffle::Raffle};
    use raffle_platform_shared::{CreateUserRequest, CreateItemRequest, CreateRaffleRequest, UserRole, ItemStatus, RaffleStatus};

    async fn setup_test_data(pool: &PgPool) -> (User, Item, Raffle) {
        // Create test user
        let user_request = CreateUserRequest {
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            phone_number: None,
        };

        let user = User::create(
            pool,
            user_request,
            "password_hash".to_string(),
            "0x1234567890123456789012345678901234567890".to_string(),
            "encrypted_key".to_string(),
            Some("encrypted_mnemonic".to_string()),
        )
        .await
        .expect("Failed to create test user");

        // Create test item
        let item_request = CreateItemRequest {
            name: "Test Item".to_string(),
            description: Some("Test Description".to_string()),
            images: vec!["image1.jpg".to_string()],
            retail_price: Decimal::new(10000, 2), // $100.00
            cost_of_goods: Decimal::new(5000, 2), // $50.00
            stock_quantity: 1,
        };

        let item = Item::create(pool, None, item_request, None, None)
            .await
            .expect("Failed to create test item");

        // Create test raffle
        let raffle_request = CreateRaffleRequest {
            item_id: item.id,
            total_boxes: 100,
            box_price: Decimal::new(100, 2), // $1.00
            total_winners: 1,
            grid_rows: 10,
            grid_cols: 10,
        };

        let raffle = Raffle::create(pool, raffle_request, None)
            .await
            .expect("Failed to create test raffle");

        (user, item, raffle)
    }

    #[sqlx::test]
    async fn test_box_purchase_crud(pool: PgPool) {
        let (user, _item, raffle) = setup_test_data(&pool).await;

        // Test create
        let box_purchase = BoxPurchase::create(
            &pool,
            raffle.id,
            user.id,
            1,
            Decimal::new(100, 2),
            None,
        )
        .await
        .expect("Failed to create box purchase");

        assert_eq!(box_purchase.raffle_id, raffle.id);
        assert_eq!(box_purchase.user_id, user.id);
        assert_eq!(box_purchase.box_number, 1);

        // Test find by ID
        let found = BoxPurchase::find_by_id(&pool, box_purchase.id)
            .await
            .expect("Failed to find box purchase")
            .expect("Box purchase not found");

        assert_eq!(found.id, box_purchase.id);

        // Test is_box_purchased
        let is_purchased = BoxPurchase::is_box_purchased(&pool, raffle.id, 1)
            .await
            .expect("Failed to check if box is purchased");

        assert!(is_purchased);

        // Test get_purchased_box_numbers
        let purchased_numbers = BoxPurchase::get_purchased_box_numbers(&pool, raffle.id)
            .await
            .expect("Failed to get purchased box numbers");

        assert_eq!(purchased_numbers, vec![1]);
    }
}