use chrono::{DateTime, Utc};
use raffle_platform_shared::{RaffleStatus, CreateRaffleRequest, RaffleResponse, BoxPurchaseResponse};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;
use crate::error::AppError;
use crate::models::item::Item;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Raffle {
    pub id: Uuid,
    pub item_id: Uuid,
    pub total_boxes: i32,
    pub box_price: Decimal,
    pub boxes_sold: i32,
    pub total_winners: i32,
    pub status: RaffleStatus,
    pub winner_user_ids: Vec<Uuid>,
    pub blockchain_tx_hash: Option<String>,
    pub grid_rows: i32,
    pub grid_cols: i32,
    pub transaction_fee_applied: Option<Decimal>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Raffle {
    /// Create a new raffle
    pub async fn create(
        pool: &PgPool,
        request: CreateRaffleRequest,
        transaction_fee: Option<Decimal>,
    ) -> Result<Self, AppError> {
        // Validate that grid can accommodate all boxes
        if request.grid_rows * request.grid_cols < request.total_boxes {
            return Err(AppError::Validation(
                "Grid size is too small for the number of boxes".to_string(),
            ));
        }

        // Validate that total winners doesn't exceed total boxes
        if request.total_winners > request.total_boxes {
            return Err(AppError::Validation(
                "Total winners cannot exceed total boxes".to_string(),
            ));
        }

        let raffle = sqlx::query_as!(
            Raffle,
            r#"
            INSERT INTO raffles (item_id, total_boxes, box_price, total_winners, grid_rows, grid_cols, transaction_fee_applied)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING 
                id, item_id, total_boxes, box_price, boxes_sold, total_winners,
                status as "status: RaffleStatus", winner_user_ids, blockchain_tx_hash,
                grid_rows, grid_cols, transaction_fee_applied, started_at, completed_at,
                created_at, updated_at
            "#,
            request.item_id,
            request.total_boxes,
            request.box_price,
            request.total_winners,
            request.grid_rows,
            request.grid_cols,
            transaction_fee
        )
        .fetch_one(pool)
        .await?;

        Ok(raffle)
    }

    /// Find raffle by ID
    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Self>, AppError> {
        let raffle = sqlx::query_as!(
            Raffle,
            r#"
            SELECT 
                id, item_id, total_boxes, box_price, boxes_sold, total_winners,
                status as "status: RaffleStatus", winner_user_ids, blockchain_tx_hash,
                grid_rows, grid_cols, transaction_fee_applied, started_at, completed_at,
                created_at, updated_at
            FROM raffles 
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(pool)
        .await?;

        Ok(raffle)
    }

    /// Find raffles by item
    pub async fn find_by_item(pool: &PgPool, item_id: Uuid) -> Result<Vec<Self>, AppError> {
        let raffles = sqlx::query_as!(
            Raffle,
            r#"
            SELECT 
                id, item_id, total_boxes, box_price, boxes_sold, total_winners,
                status as "status: RaffleStatus", winner_user_ids, blockchain_tx_hash,
                grid_rows, grid_cols, transaction_fee_applied, started_at, completed_at,
                created_at, updated_at
            FROM raffles 
            WHERE item_id = $1
            ORDER BY created_at DESC
            "#,
            item_id
        )
        .fetch_all(pool)
        .await?;

        Ok(raffles)
    }

    /// Find active raffles
    pub async fn find_active(
        pool: &PgPool,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Self>, AppError> {
        let raffles = sqlx::query_as!(
            Raffle,
            r#"
            SELECT 
                id, item_id, total_boxes, box_price, boxes_sold, total_winners,
                status as "status: RaffleStatus", winner_user_ids, blockchain_tx_hash,
                grid_rows, grid_cols, transaction_fee_applied, started_at, completed_at,
                created_at, updated_at
            FROM raffles 
            WHERE status IN ('open', 'full', 'drawing')
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
            limit,
            offset
        )
        .fetch_all(pool)
        .await?;

        Ok(raffles)
    }

    /// Update raffle status
    pub async fn update_status(
        pool: &PgPool,
        id: Uuid,
        status: RaffleStatus,
    ) -> Result<(), AppError> {
        let now = Utc::now();
        
        match status {
            RaffleStatus::Full => {
                sqlx::query!(
                    "UPDATE raffles SET status = $1, started_at = $2, updated_at = NOW() WHERE id = $3",
                    status as RaffleStatus,
                    now,
                    id
                )
                .execute(pool)
                .await?;
            }
            RaffleStatus::Completed => {
                sqlx::query!(
                    "UPDATE raffles SET status = $1, completed_at = $2, updated_at = NOW() WHERE id = $3",
                    status as RaffleStatus,
                    now,
                    id
                )
                .execute(pool)
                .await?;
            }
            _ => {
                sqlx::query!(
                    "UPDATE raffles SET status = $1, updated_at = NOW() WHERE id = $2",
                    status as RaffleStatus,
                    id
                )
                .execute(pool)
                .await?;
            }
        }

        Ok(())
    }

    /// Update boxes sold count
    pub async fn update_boxes_sold(
        pool: &PgPool,
        id: Uuid,
        boxes_sold: i32,
    ) -> Result<(), AppError> {
        sqlx::query!(
            "UPDATE raffles SET boxes_sold = $1, updated_at = NOW() WHERE id = $2",
            boxes_sold,
            id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Set winners
    pub async fn set_winners(
        pool: &PgPool,
        id: Uuid,
        winner_user_ids: Vec<Uuid>,
        blockchain_tx_hash: Option<String>,
    ) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            UPDATE raffles 
            SET winner_user_ids = $1, blockchain_tx_hash = $2, status = 'completed', 
                completed_at = NOW(), updated_at = NOW() 
            WHERE id = $3
            "#,
            &winner_user_ids,
            blockchain_tx_hash,
            id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Get raffle with item details
    pub async fn find_with_item(pool: &PgPool, id: Uuid) -> Result<Option<(Self, Item)>, AppError> {
        let result = sqlx::query!(
            r#"
            SELECT 
                r.id, r.item_id, r.total_boxes, r.box_price, r.boxes_sold, r.total_winners,
                r.status as "raffle_status: RaffleStatus", r.winner_user_ids, r.blockchain_tx_hash,
                r.grid_rows, r.grid_cols, r.transaction_fee_applied, r.started_at, r.completed_at,
                r.created_at as raffle_created_at, r.updated_at as raffle_updated_at,
                i.id as item_id, i.seller_id, i.name, i.description, i.images, 
                i.retail_price, i.cost_of_goods, i.status as "item_status: crate::models::item::ItemStatus", 
                i.stock_quantity, i.listing_fee_applied, i.listing_fee_type,
                i.created_at as item_created_at, i.updated_at as item_updated_at
            FROM raffles r
            JOIN items i ON r.item_id = i.id
            WHERE r.id = $1
            "#,
            id
        )
        .fetch_optional(pool)
        .await?;

        if let Some(row) = result {
            let raffle = Raffle {
                id: row.id,
                item_id: row.item_id,
                total_boxes: row.total_boxes,
                box_price: row.box_price,
                boxes_sold: row.boxes_sold,
                total_winners: row.total_winners,
                status: row.raffle_status,
                winner_user_ids: row.winner_user_ids,
                blockchain_tx_hash: row.blockchain_tx_hash,
                grid_rows: row.grid_rows,
                grid_cols: row.grid_cols,
                transaction_fee_applied: row.transaction_fee_applied,
                started_at: row.started_at,
                completed_at: row.completed_at,
                created_at: row.raffle_created_at,
                updated_at: row.raffle_updated_at,
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

            Ok(Some((raffle, item)))
        } else {
            Ok(None)
        }
    }

    /// Count active raffles
    pub async fn count_active(pool: &PgPool) -> Result<i64, AppError> {
        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM raffles WHERE status IN ('open', 'full', 'drawing')"
        )
        .fetch_one(pool)
        .await?;

        Ok(count.unwrap_or(0))
    }

    /// Convert to response DTO
    pub fn to_response(&self, item: Option<crate::models::item::Item>) -> RaffleResponse {
        RaffleResponse {
            id: self.id,
            item_id: self.item_id,
            item: item.map(|i| i.to_response()),
            total_boxes: self.total_boxes,
            box_price: self.box_price,
            boxes_sold: self.boxes_sold,
            total_winners: self.total_winners,
            status: self.status,
            winner_user_ids: self.winner_user_ids.clone(),
            grid_rows: self.grid_rows,
            grid_cols: self.grid_cols,
            started_at: self.started_at,
            completed_at: self.completed_at,
            created_at: self.created_at,
        }
    }

    /// Check if raffle is full
    pub fn is_full(&self) -> bool {
        self.boxes_sold >= self.total_boxes
    }

    /// Check if raffle is completed
    pub fn is_completed(&self) -> bool {
        self.status == RaffleStatus::Completed
    }

    /// Calculate total revenue
    pub fn total_revenue(&self) -> Decimal {
        Decimal::from(self.boxes_sold) * self.box_price
    }

    /// Calculate completion percentage
    pub fn completion_percentage(&self) -> f64 {
        if self.total_boxes > 0 {
            (self.boxes_sold as f64 / self.total_boxes as f64) * 100.0
        } else {
            0.0
        }
    }
}

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
        let purchase = sqlx::query_as!(
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

        Ok(purchase)
    }

    /// Find purchases by raffle
    pub async fn find_by_raffle(pool: &PgPool, raffle_id: Uuid) -> Result<Vec<Self>, AppError> {
        let purchases = sqlx::query_as!(
            BoxPurchase,
            "SELECT id, raffle_id, user_id, box_number, purchase_price_in_credits, transaction_id, blockchain_tx_hash, created_at FROM box_purchases WHERE raffle_id = $1 ORDER BY box_number",
            raffle_id
        )
        .fetch_all(pool)
        .await?;

        Ok(purchases)
    }

    /// Find purchases by user
    pub async fn find_by_user(
        pool: &PgPool,
        user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Self>, AppError> {
        let purchases = sqlx::query_as!(
            BoxPurchase,
            "SELECT id, raffle_id, user_id, box_number, purchase_price_in_credits, transaction_id, blockchain_tx_hash, created_at FROM box_purchases WHERE user_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
            user_id,
            limit,
            offset
        )
        .fetch_all(pool)
        .await?;

        Ok(purchases)
    }

    /// Update blockchain transaction hash
    pub async fn update_blockchain_tx_hash(
        pool: &PgPool,
        id: Uuid,
        tx_hash: String,
    ) -> Result<(), AppError> {
        sqlx::query!(
            "UPDATE box_purchases SET blockchain_tx_hash = $1 WHERE id = $2",
            tx_hash,
            id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Check if box is already purchased
    pub async fn is_box_purchased(
        pool: &PgPool,
        raffle_id: Uuid,
        box_number: i32,
    ) -> Result<bool, AppError> {
        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM box_purchases WHERE raffle_id = $1 AND box_number = $2",
            raffle_id,
            box_number
        )
        .fetch_one(pool)
        .await?;

        Ok(count.unwrap_or(0) > 0)
    }

    /// Get user's purchases for a specific raffle
    pub async fn find_user_purchases_for_raffle(
        pool: &PgPool,
        user_id: Uuid,
        raffle_id: Uuid,
    ) -> Result<Vec<Self>, AppError> {
        let purchases = sqlx::query_as!(
            BoxPurchase,
            "SELECT id, raffle_id, user_id, box_number, purchase_price_in_credits, transaction_id, blockchain_tx_hash, created_at FROM box_purchases WHERE user_id = $1 AND raffle_id = $2 ORDER BY box_number",
            user_id,
            raffle_id
        )
        .fetch_all(pool)
        .await?;

        Ok(purchases)
    }

    /// Convert to response DTO
    pub fn to_response(&self) -> BoxPurchaseResponse {
        BoxPurchaseResponse {
            id: self.id,
            raffle_id: self.raffle_id,
            user_id: self.user_id,
            box_number: self.box_number,
            purchase_price_in_credits: self.purchase_price_in_credits,
            blockchain_tx_hash: self.blockchain_tx_hash.clone(),
            created_at: self.created_at,
        }
    }
}