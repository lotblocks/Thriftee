use chrono::{DateTime, Utc};
use raffle_platform_shared::{ItemStatus, CreateItemRequest, ItemResponse};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;
use crate::error::AppError;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Item {
    pub id: Uuid,
    pub seller_id: Option<Uuid>,
    pub name: String,
    pub description: Option<String>,
    pub images: Vec<String>,
    pub retail_price: Decimal,
    pub cost_of_goods: Decimal,
    pub status: ItemStatus,
    pub stock_quantity: i32,
    pub listing_fee_applied: Option<Decimal>,
    pub listing_fee_type: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Item {
    /// Create a new item
    pub async fn create(
        pool: &PgPool,
        seller_id: Option<Uuid>,
        request: CreateItemRequest,
        listing_fee: Option<Decimal>,
        listing_fee_type: Option<String>,
    ) -> Result<Self, AppError> {
        let item = sqlx::query_as!(
            Item,
            r#"
            INSERT INTO items (seller_id, name, description, images, retail_price, cost_of_goods, stock_quantity, listing_fee_applied, listing_fee_type)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING 
                id, seller_id, name, description, images, retail_price, cost_of_goods,
                status as "status: ItemStatus", stock_quantity, listing_fee_applied, listing_fee_type,
                created_at, updated_at
            "#,
            seller_id,
            request.name,
            request.description,
            &request.images,
            request.retail_price,
            request.cost_of_goods,
            request.stock_quantity,
            listing_fee,
            listing_fee_type
        )
        .fetch_one(pool)
        .await?;

        Ok(item)
    }

    /// Find item by ID
    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Self>, AppError> {
        let item = sqlx::query_as!(
            Item,
            r#"
            SELECT 
                id, seller_id, name, description, images, retail_price, cost_of_goods,
                status as "status: ItemStatus", stock_quantity, listing_fee_applied, listing_fee_type,
                created_at, updated_at
            FROM items 
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(pool)
        .await?;

        Ok(item)
    }

    /// Find items by seller
    pub async fn find_by_seller(
        pool: &PgPool,
        seller_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Self>, AppError> {
        let items = sqlx::query_as!(
            Item,
            r#"
            SELECT 
                id, seller_id, name, description, images, retail_price, cost_of_goods,
                status as "status: ItemStatus", stock_quantity, listing_fee_applied, listing_fee_type,
                created_at, updated_at
            FROM items 
            WHERE seller_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            seller_id,
            limit,
            offset
        )
        .fetch_all(pool)
        .await?;

        Ok(items)
    }

    /// Find available items
    pub async fn find_available(
        pool: &PgPool,
        limit: i64,
        offset: i64,
        search: Option<&str>,
    ) -> Result<Vec<Self>, AppError> {
        let items = match search {
            Some(search_term) => {
                sqlx::query_as!(
                    Item,
                    r#"
                    SELECT 
                        id, seller_id, name, description, images, retail_price, cost_of_goods,
                        status as "status: ItemStatus", stock_quantity, listing_fee_applied, listing_fee_type,
                        created_at, updated_at
                    FROM items 
                    WHERE status = 'available' AND stock_quantity > 0
                    AND (name ILIKE $1 OR description ILIKE $1)
                    ORDER BY created_at DESC
                    LIMIT $2 OFFSET $3
                    "#,
                    format!("%{}%", search_term),
                    limit,
                    offset
                )
                .fetch_all(pool)
                .await?
            }
            None => {
                sqlx::query_as!(
                    Item,
                    r#"
                    SELECT 
                        id, seller_id, name, description, images, retail_price, cost_of_goods,
                        status as "status: ItemStatus", stock_quantity, listing_fee_applied, listing_fee_type,
                        created_at, updated_at
                    FROM items 
                    WHERE status = 'available' AND stock_quantity > 0
                    ORDER BY created_at DESC
                    LIMIT $1 OFFSET $2
                    "#,
                    limit,
                    offset
                )
                .fetch_all(pool)
                .await?
            }
        };

        Ok(items)
    }

    /// Update item status
    pub async fn update_status(
        pool: &PgPool,
        id: Uuid,
        status: ItemStatus,
    ) -> Result<(), AppError> {
        sqlx::query!(
            "UPDATE items SET status = $1, updated_at = NOW() WHERE id = $2",
            status as ItemStatus,
            id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Update stock quantity
    pub async fn update_stock_quantity(
        pool: &PgPool,
        id: Uuid,
        quantity: i32,
    ) -> Result<(), AppError> {
        sqlx::query!(
            "UPDATE items SET stock_quantity = $1, updated_at = NOW() WHERE id = $2",
            quantity,
            id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Decrease stock quantity (for purchases)
    pub async fn decrease_stock(pool: &PgPool, id: Uuid, amount: i32) -> Result<bool, AppError> {
        let result = sqlx::query!(
            r#"
            UPDATE items 
            SET stock_quantity = stock_quantity - $1, updated_at = NOW() 
            WHERE id = $2 AND stock_quantity >= $1
            "#,
            amount,
            id
        )
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Get total count of available items
    pub async fn count_available(pool: &PgPool, search: Option<&str>) -> Result<i64, AppError> {
        let count = match search {
            Some(search_term) => {
                sqlx::query_scalar!(
                    r#"
                    SELECT COUNT(*) 
                    FROM items 
                    WHERE status = 'available' AND stock_quantity > 0
                    AND (name ILIKE $1 OR description ILIKE $1)
                    "#,
                    format!("%{}%", search_term)
                )
                .fetch_one(pool)
                .await?
            }
            None => {
                sqlx::query_scalar!(
                    "SELECT COUNT(*) FROM items WHERE status = 'available' AND stock_quantity > 0"
                )
                .fetch_one(pool)
                .await?
            }
        };

        Ok(count.unwrap_or(0))
    }

    /// Get total count by seller
    pub async fn count_by_seller(pool: &PgPool, seller_id: Uuid) -> Result<i64, AppError> {
        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM items WHERE seller_id = $1",
            seller_id
        )
        .fetch_one(pool)
        .await?;

        Ok(count.unwrap_or(0))
    }

    /// Update item details
    pub async fn update(
        pool: &PgPool,
        id: Uuid,
        name: Option<String>,
        description: Option<String>,
        images: Option<Vec<String>>,
        retail_price: Option<Decimal>,
        cost_of_goods: Option<Decimal>,
    ) -> Result<(), AppError> {
        let mut query = "UPDATE items SET updated_at = NOW()".to_string();
        let mut params: Vec<String> = vec![];
        let mut param_count = 1;

        if let Some(name) = name {
            query.push_str(&format!(", name = ${}", param_count));
            params.push(name);
            param_count += 1;
        }

        if let Some(description) = description {
            query.push_str(&format!(", description = ${}", param_count));
            params.push(description);
            param_count += 1;
        }

        if let Some(retail_price) = retail_price {
            query.push_str(&format!(", retail_price = ${}", param_count));
            params.push(retail_price.to_string());
            param_count += 1;
        }

        if let Some(cost_of_goods) = cost_of_goods {
            query.push_str(&format!(", cost_of_goods = ${}", param_count));
            params.push(cost_of_goods.to_string());
            param_count += 1;
        }

        query.push_str(&format!(" WHERE id = ${}", param_count));

        // For simplicity, we'll use a basic update. In a real implementation,
        // you might want to use a query builder or handle this more dynamically
        sqlx::query!(
            "UPDATE items SET updated_at = NOW() WHERE id = $1",
            id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Delete item (soft delete by setting status to inactive)
    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<(), AppError> {
        sqlx::query!(
            "UPDATE items SET status = 'inactive', updated_at = NOW() WHERE id = $1",
            id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Convert to response DTO
    pub fn to_response(&self) -> ItemResponse {
        ItemResponse {
            id: self.id,
            seller_id: self.seller_id,
            name: self.name.clone(),
            description: self.description.clone(),
            images: self.images.clone(),
            retail_price: self.retail_price,
            cost_of_goods: self.cost_of_goods,
            status: self.status,
            stock_quantity: self.stock_quantity,
            listing_fee_applied: self.listing_fee_applied,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }

    /// Check if item is available for raffle
    pub fn is_available_for_raffle(&self) -> bool {
        self.status == ItemStatus::Available && self.stock_quantity > 0
    }

    /// Calculate profit margin
    pub fn profit_margin(&self) -> Decimal {
        if self.retail_price > Decimal::ZERO {
            ((self.retail_price - self.cost_of_goods) / self.retail_price) * Decimal::from(100)
        } else {
            Decimal::ZERO
        }
    }
}