use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;
use crate::error::AppError;
use crate::models::item::Item;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct FreeRedeemableItem {
    pub id: Uuid,
    pub item_id: Uuid,
    pub required_credit_amount: Decimal,
    pub available_quantity: i32,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl FreeRedeemableItem {
    /// Create a new free redeemable item
    pub async fn create(
        pool: &PgPool,
        item_id: Uuid,
        required_credit_amount: Decimal,
        available_quantity: i32,
    ) -> Result<Self, AppError> {
        let free_item = sqlx::query_as!(
            FreeRedeemableItem,
            r#"
            INSERT INTO free_redeemable_items (item_id, required_credit_amount, available_quantity)
            VALUES ($1, $2, $3)
            RETURNING id, item_id, required_credit_amount, available_quantity, is_active, created_at, updated_at
            "#,
            item_id,
            required_credit_amount,
            available_quantity
        )
        .fetch_one(pool)
        .await?;

        Ok(free_item)
    }

    /// Find all active free redeemable items
    pub async fn find_active(
        pool: &PgPool,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<(Self, Item)>, AppError> {
        let results = sqlx::query!(
            r#"
            SELECT 
                fri.id, fri.item_id, fri.required_credit_amount, fri.available_quantity, 
                fri.is_active, fri.created_at as fri_created_at, fri.updated_at as fri_updated_at,
                i.id as item_id, i.seller_id, i.name, i.description, i.images, 
                i.retail_price, i.cost_of_goods, i.status as "item_status: crate::models::item::ItemStatus", 
                i.stock_quantity, i.listing_fee_applied, i.listing_fee_type,
                i.created_at as item_created_at, i.updated_at as item_updated_at
            FROM free_redeemable_items fri
            JOIN items i ON fri.item_id = i.id
            WHERE fri.is_active = true AND fri.available_quantity > 0
            AND i.status = 'available'
            ORDER BY fri.required_credit_amount ASC
            LIMIT $1 OFFSET $2
            "#,
            limit,
            offset
        )
        .fetch_all(pool)
        .await?;

        let mut items = Vec::new();
        for row in results {
            let free_item = FreeRedeemableItem {
                id: row.id,
                item_id: row.item_id,
                required_credit_amount: row.required_credit_amount,
                available_quantity: row.available_quantity,
                is_active: row.is_active,
                created_at: row.fri_created_at,
                updated_at: row.fri_updated_at,
            };

            let item = Item {
                id: row.item_id.unwrap(),
                seller_id: row.seller_id,
                name: row.name.unwrap(),
                description: row.description,
                images: row.images.unwrap(),
                retail_price: row.retail_price.unwrap(),
                cost_of_goods: row.cost_of_goods.unwrap(),
                status: row.item_status.unwrap(),
                stock_quantity: row.stock_quantity.unwrap(),
                listing_fee_applied: row.listing_fee_applied,
                listing_fee_type: row.listing_fee_type,
                created_at: row.item_created_at.unwrap(),
                updated_at: row.item_updated_at.unwrap(),
            };

            items.push((free_item, item));
        }

        Ok(items)
    }

    /// Find free items available for a specific credit amount
    pub async fn find_available_for_credits(
        pool: &PgPool,
        credit_amount: Decimal,
        limit: i64,
    ) -> Result<Vec<(Self, Item)>, AppError> {
        let results = sqlx::query!(
            r#"
            SELECT 
                fri.id, fri.item_id, fri.required_credit_amount, fri.available_quantity, 
                fri.is_active, fri.created_at as fri_created_at, fri.updated_at as fri_updated_at,
                i.id as item_id, i.seller_id, i.name, i.description, i.images, 
                i.retail_price, i.cost_of_goods, i.status as "item_status: crate::models::item::ItemStatus", 
                i.stock_quantity, i.listing_fee_applied, i.listing_fee_type,
                i.created_at as item_created_at, i.updated_at as item_updated_at
            FROM free_redeemable_items fri
            JOIN items i ON fri.item_id = i.id
            WHERE fri.is_active = true AND fri.available_quantity > 0
            AND i.status = 'available'
            AND fri.required_credit_amount <= $1
            ORDER BY fri.required_credit_amount DESC
            LIMIT $2
            "#,
            credit_amount,
            limit
        )
        .fetch_all(pool)
        .await?;

        let mut items = Vec::new();
        for row in results {
            let free_item = FreeRedeemableItem {
                id: row.id,
                item_id: row.item_id,
                required_credit_amount: row.required_credit_amount,
                available_quantity: row.available_quantity,
                is_active: row.is_active,
                created_at: row.fri_created_at,
                updated_at: row.fri_updated_at,
            };

            let item = Item {
                id: row.item_id.unwrap(),
                seller_id: row.seller_id,
                name: row.name.unwrap(),
                description: row.description,
                images: row.images.unwrap(),
                retail_price: row.retail_price.unwrap(),
                cost_of_goods: row.cost_of_goods.unwrap(),
                status: row.item_status.unwrap(),
                stock_quantity: row.stock_quantity.unwrap(),
                listing_fee_applied: row.listing_fee_applied,
                listing_fee_type: row.listing_fee_type,
                created_at: row.item_created_at.unwrap(),
                updated_at: row.item_updated_at.unwrap(),
            };

            items.push((free_item, item));
        }

        Ok(items)
    }

    /// Find by item ID
    pub async fn find_by_item_id(pool: &PgPool, item_id: Uuid) -> Result<Option<Self>, AppError> {
        let free_item = sqlx::query_as!(
            FreeRedeemableItem,
            r#"
            SELECT id, item_id, required_credit_amount, available_quantity, is_active, created_at, updated_at
            FROM free_redeemable_items 
            WHERE item_id = $1
            "#,
            item_id
        )
        .fetch_optional(pool)
        .await?;

        Ok(free_item)
    }

    /// Update available quantity
    pub async fn update_quantity(
        pool: &PgPool,
        id: Uuid,
        new_quantity: i32,
    ) -> Result<(), AppError> {
        sqlx::query!(
            "UPDATE free_redeemable_items SET available_quantity = $1, updated_at = NOW() WHERE id = $2",
            new_quantity,
            id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Decrease available quantity (for redemptions)
    pub async fn decrease_quantity(
        pool: &PgPool,
        id: Uuid,
        amount: i32,
    ) -> Result<bool, AppError> {
        let result = sqlx::query!(
            r#"
            UPDATE free_redeemable_items 
            SET available_quantity = available_quantity - $1, updated_at = NOW() 
            WHERE id = $2 AND available_quantity >= $1
            "#,
            amount,
            id
        )
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Update active status
    pub async fn update_active_status(
        pool: &PgPool,
        id: Uuid,
        is_active: bool,
    ) -> Result<(), AppError> {
        sqlx::query!(
            "UPDATE free_redeemable_items SET is_active = $1, updated_at = NOW() WHERE id = $2",
            is_active,
            id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Update required credit amount
    pub async fn update_required_credits(
        pool: &PgPool,
        id: Uuid,
        required_credit_amount: Decimal,
    ) -> Result<(), AppError> {
        sqlx::query!(
            "UPDATE free_redeemable_items SET required_credit_amount = $1, updated_at = NOW() WHERE id = $2",
            required_credit_amount,
            id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Check if item can be redeemed with given credits
    pub fn can_redeem_with_credits(&self, available_credits: Decimal) -> bool {
        self.is_active 
            && self.available_quantity > 0 
            && available_credits >= self.required_credit_amount
    }

    /// Get total count of active free items
    pub async fn count_active(pool: &PgPool) -> Result<i64, AppError> {
        let count = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) 
            FROM free_redeemable_items fri
            JOIN items i ON fri.item_id = i.id
            WHERE fri.is_active = true AND fri.available_quantity > 0
            AND i.status = 'available'
            "#
        )
        .fetch_one(pool)
        .await?;

        Ok(count.unwrap_or(0))
    }

    /// Redeem free item (atomic operation)
    pub async fn redeem_item(
        pool: &PgPool,
        free_item_id: Uuid,
        user_id: Uuid,
        credit_amount_used: Decimal,
    ) -> Result<RedemptionResult, AppError> {
        let mut tx = pool.begin().await?;

        // Check if item is still available
        let free_item = sqlx::query_as!(
            FreeRedeemableItem,
            "SELECT id, item_id, required_credit_amount, available_quantity, is_active, created_at, updated_at FROM free_redeemable_items WHERE id = $1 FOR UPDATE",
            free_item_id
        )
        .fetch_optional(&mut *tx)
        .await?;

        let free_item = match free_item {
            Some(item) => item,
            None => {
                tx.rollback().await?;
                return Err(AppError::NotFound("Free item not found".to_string()));
            }
        };

        if !free_item.is_active || free_item.available_quantity <= 0 {
            tx.rollback().await?;
            return Err(AppError::Validation("Item is no longer available for redemption".to_string()));
        }

        if credit_amount_used < free_item.required_credit_amount {
            tx.rollback().await?;
            return Err(AppError::Validation("Insufficient credits for redemption".to_string()));
        }

        // Decrease quantity
        sqlx::query!(
            "UPDATE free_redeemable_items SET available_quantity = available_quantity - 1, updated_at = NOW() WHERE id = $1",
            free_item_id
        )
        .execute(&mut *tx)
        .await?;

        // Create transaction record
        let transaction_id = sqlx::query_scalar!(
            r#"
            INSERT INTO transactions (user_id, amount, type, metadata, status)
            VALUES ($1, $2, 'free_item_redemption', $3, 'completed')
            RETURNING id
            "#,
            user_id,
            -credit_amount_used,
            serde_json::json!({
                "free_item_id": free_item_id,
                "item_id": free_item.item_id,
                "credits_used": credit_amount_used
            })
        )
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(RedemptionResult {
            transaction_id,
            item_id: free_item.item_id,
            credits_used: credit_amount_used,
            remaining_quantity: free_item.available_quantity - 1,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RedemptionResult {
    pub transaction_id: Uuid,
    pub item_id: Uuid,
    pub credits_used: Decimal,
    pub remaining_quantity: i32,
}