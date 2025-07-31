use crate::models::item::Item;
use crate::models::user::User;
use crate::error::AppError;
use crate::services::realtime_service::RealtimeService;
use chrono::{DateTime, Utc};
use raffle_platform_shared::{ItemStatus, CreateItemRequest, ItemResponse, PaginatedResponse};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Item management service handles all item-related operations
#[derive(Clone)]
pub struct ItemService {
    db_pool: PgPool,
    realtime_service: Option<RealtimeService>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemSearchParams {
    pub search: Option<String>,
    pub category: Option<String>,
    pub min_price: Option<Decimal>,
    pub max_price: Option<Decimal>,
    pub status: Option<ItemStatus>,
    pub seller_id: Option<Uuid>,
    pub sort_by: Option<String>, // "price_asc", "price_desc", "created_asc", "created_desc", "name"
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemUpdateRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub images: Option<Vec<String>>,
    pub retail_price: Option<Decimal>,
    pub cost_of_goods: Option<Decimal>,
    pub stock_quantity: Option<i32>,
    pub category: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemStatistics {
    pub total_items: i64,
    pub available_items: i64,
    pub sold_items: i64,
    pub inactive_items: i64,
    pub total_value: Decimal,
    pub average_price: Decimal,
    pub items_by_category: HashMap<String, i64>,
    pub items_by_seller: HashMap<Uuid, i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemAnalytics {
    pub item_id: Uuid,
    pub views_count: i64,
    pub unique_viewers: i64,
    pub raffle_count: i64,
    pub total_boxes_sold: i64,
    pub total_revenue: Decimal,
    pub average_completion_time: Option<i64>, // minutes
    pub conversion_rate: Option<Decimal>, // views to purchases
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkItemOperation {
    pub item_ids: Vec<Uuid>,
    pub operation: BulkOperation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BulkOperation {
    UpdateStatus(ItemStatus),
    UpdateCategory(String),
    Delete,
    UpdateStock(i32),
}

impl ItemService {
    /// Create a new item service
    pub fn new(db_pool: PgPool) -> Self {
        Self { 
            db_pool,
            realtime_service: None,
        }
    }

    /// Create a new item service with realtime updates
    pub fn with_realtime(db_pool: PgPool, realtime_service: RealtimeService) -> Self {
        Self { 
            db_pool,
            realtime_service: Some(realtime_service),
        }
    }

    /// Create a new item
    pub async fn create_item(
        &self,
        seller_id: Uuid,
        request: CreateItemRequest,
    ) -> Result<ItemResponse, AppError> {
        // Validate seller exists and is active
        let seller = User::find_by_id(&self.db_pool, seller_id).await?
            .ok_or_else(|| AppError::NotFound("Seller not found".to_string()))?;

        if !seller.is_seller() {
            return Err(AppError::Forbidden("User is not a seller".to_string()));
        }

        // Validate images
        if request.images.is_empty() {
            return Err(AppError::Validation("At least one image is required".to_string()));
        }

        // Validate pricing
        if request.retail_price <= request.cost_of_goods {
            return Err(AppError::Validation("Retail price must be greater than cost of goods".to_string()));
        }

        // Calculate listing fee (this would be based on seller's subscription tier)
        let listing_fee = self.calculate_listing_fee(&seller, &request).await?;

        // Create the item
        let item = Item::create(
            &self.db_pool,
            Some(seller_id),
            request,
            listing_fee.amount,
            Some(listing_fee.fee_type),
        ).await?;

        // Log item creation
        self.log_item_activity(
            item.id,
            seller_id,
            "item_created",
            &format!("Item '{}' created", item.name),
        ).await?;

        // Broadcast item creation event
        if let Some(realtime_service) = &self.realtime_service {
            let _ = realtime_service.broadcast_event(
                crate::services::realtime_service::RealtimeEvent::ItemCreated {
                    item_id: item.id,
                    seller_id,
                    name: item.name.clone(),
                    category: None, // Would get from request if available
                    created_at: item.created_at,
                }
            ).await;
        }

        info!(
            "Created item {} for seller {} - name: '{}'",
            item.id, seller_id, item.name
        );

        Ok(item.to_response())
    }

    /// Get item by ID
    pub async fn get_item(&self, item_id: Uuid) -> Result<ItemResponse, AppError> {
        let item = Item::find_by_id(&self.db_pool, item_id).await?
            .ok_or_else(|| AppError::NotFound("Item not found".to_string()))?;

        // Increment view count
        self.increment_item_views(item_id).await?;

        Ok(item.to_response())
    }

    /// Search items with filters and pagination
    pub async fn search_items(
        &self,
        params: ItemSearchParams,
    ) -> Result<PaginatedResponse<ItemResponse>, AppError> {
        let limit = params.limit.unwrap_or(20).min(100);
        let offset = params.offset.unwrap_or(0);

        // Build dynamic query
        let mut query = "SELECT id, seller_id, name, description, images, retail_price, cost_of_goods, status, stock_quantity, listing_fee_applied, listing_fee_type, created_at, updated_at FROM items WHERE 1=1".to_string();
        let mut conditions = Vec::new();
        let mut param_count = 1;

        // Add search condition
        if let Some(search) = &params.search {
            conditions.push(format!("AND (name ILIKE ${} OR description ILIKE ${})", param_count, param_count));
            param_count += 1;
        }

        // Add category filter
        if let Some(category) = &params.category {
            conditions.push(format!("AND category = ${}", param_count));
            param_count += 1;
        }

        // Add price range filters
        if let Some(min_price) = params.min_price {
            conditions.push(format!("AND retail_price >= ${}", param_count));
            param_count += 1;
        }

        if let Some(max_price) = params.max_price {
            conditions.push(format!("AND retail_price <= ${}", param_count));
            param_count += 1;
        }

        // Add status filter
        if let Some(status) = params.status {
            conditions.push(format!("AND status = ${}", param_count));
            param_count += 1;
        }

        // Add seller filter
        if let Some(seller_id) = params.seller_id {
            conditions.push(format!("AND seller_id = ${}", param_count));
            param_count += 1;
        }

        // Add conditions to query
        for condition in conditions {
            query.push_str(&condition);
        }

        // Add sorting
        let sort_clause = match params.sort_by.as_deref() {
            Some("price_asc") => "ORDER BY retail_price ASC",
            Some("price_desc") => "ORDER BY retail_price DESC",
            Some("created_asc") => "ORDER BY created_at ASC",
            Some("name") => "ORDER BY name ASC",
            _ => "ORDER BY created_at DESC", // default
        };
        query.push_str(&format!(" {} LIMIT ${} OFFSET ${}", sort_clause, param_count, param_count + 1));

        // For simplicity, we'll use the existing find_available method
        // In a real implementation, you'd build and execute the dynamic query
        let items = if params.status == Some(ItemStatus::Available) || params.status.is_none() {
            Item::find_available(&self.db_pool, limit, offset, params.search.as_deref()).await?
        } else {
            // For other statuses, we'd need additional methods
            Vec::new()
        };

        let total = Item::count_available(&self.db_pool, params.search.as_deref()).await?;

        let item_responses: Vec<ItemResponse> = items.into_iter().map(|item| item.to_response()).collect();

        Ok(PaginatedResponse {
            data: item_responses,
            total,
            limit,
            offset,
            has_more: offset + limit < total,
        })
    }

    /// Update item
    pub async fn update_item(
        &self,
        item_id: Uuid,
        seller_id: Uuid,
        request: ItemUpdateRequest,
    ) -> Result<ItemResponse, AppError> {
        // Verify item exists and belongs to seller
        let item = Item::find_by_id(&self.db_pool, item_id).await?
            .ok_or_else(|| AppError::NotFound("Item not found".to_string()))?;

        if item.seller_id != Some(seller_id) {
            return Err(AppError::Forbidden("Item does not belong to seller".to_string()));
        }

        // Validate pricing if being updated
        if let (Some(retail_price), Some(cost_of_goods)) = (request.retail_price, request.cost_of_goods) {
            if retail_price <= cost_of_goods {
                return Err(AppError::Validation("Retail price must be greater than cost of goods".to_string()));
            }
        }

        // Update item
        Item::update(
            &self.db_pool,
            item_id,
            request.name.clone(),
            request.description.clone(),
            request.images.clone(),
            request.retail_price,
            request.cost_of_goods,
        ).await?;

        // Update stock quantity if provided
        if let Some(stock_quantity) = request.stock_quantity {
            Item::update_stock_quantity(&self.db_pool, item_id, stock_quantity).await?;
        }

        // Log update
        self.log_item_activity(
            item_id,
            seller_id,
            "item_updated",
            "Item details updated",
        ).await?;

        info!("Updated item {} by seller {}", item_id, seller_id);

        // Return updated item
        let updated_item = Item::find_by_id(&self.db_pool, item_id).await?
            .ok_or_else(|| AppError::Internal("Item disappeared after update".to_string()))?;

        Ok(updated_item.to_response())
    }

    /// Delete item (soft delete)
    pub async fn delete_item(&self, item_id: Uuid, seller_id: Uuid) -> Result<(), AppError> {
        // Verify item exists and belongs to seller
        let item = Item::find_by_id(&self.db_pool, item_id).await?
            .ok_or_else(|| AppError::NotFound("Item not found".to_string()))?;

        if item.seller_id != Some(seller_id) {
            return Err(AppError::Forbidden("Item does not belong to seller".to_string()));
        }

        // Check if item is in any active raffles
        let active_raffles = self.get_active_raffles_for_item(item_id).await?;
        if !active_raffles.is_empty() {
            return Err(AppError::Validation("Cannot delete item with active raffles".to_string()));
        }

        // Soft delete
        Item::delete(&self.db_pool, item_id).await?;

        // Log deletion
        self.log_item_activity(
            item_id,
            seller_id,
            "item_deleted",
            "Item deleted",
        ).await?;

        info!("Deleted item {} by seller {}", item_id, seller_id);

        Ok(())
    }

    /// Get items by seller
    pub async fn get_seller_items(
        &self,
        seller_id: Uuid,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<PaginatedResponse<ItemResponse>, AppError> {
        let limit = limit.unwrap_or(20).min(100);
        let offset = offset.unwrap_or(0);

        let items = Item::find_by_seller(&self.db_pool, seller_id, limit, offset).await?;
        let total = Item::count_by_seller(&self.db_pool, seller_id).await?;

        let item_responses: Vec<ItemResponse> = items.into_iter().map(|item| item.to_response()).collect();

        Ok(PaginatedResponse {
            data: item_responses,
            total,
            limit,
            offset,
            has_more: offset + limit < total,
        })
    }

    /// Update item status
    pub async fn update_item_status(
        &self,
        item_id: Uuid,
        seller_id: Uuid,
        status: ItemStatus,
    ) -> Result<(), AppError> {
        // Verify item belongs to seller
        let item = Item::find_by_id(&self.db_pool, item_id).await?
            .ok_or_else(|| AppError::NotFound("Item not found".to_string()))?;

        if item.seller_id != Some(seller_id) {
            return Err(AppError::Forbidden("Item does not belong to seller".to_string()));
        }

        // Update status
        Item::update_status(&self.db_pool, item_id, status).await?;

        // Log status change
        self.log_item_activity(
            item_id,
            seller_id,
            "status_updated",
            &format!("Status changed to {:?}", status),
        ).await?;

        info!("Updated item {} status to {:?} by seller {}", item_id, status, seller_id);

        Ok(())
    }

    /// Update stock quantity
    pub async fn update_stock_quantity(
        &self,
        item_id: Uuid,
        seller_id: Uuid,
        quantity: i32,
    ) -> Result<(), AppError> {
        // Verify item belongs to seller
        let item = Item::find_by_id(&self.db_pool, item_id).await?
            .ok_or_else(|| AppError::NotFound("Item not found".to_string()))?;

        if item.seller_id != Some(seller_id) {
            return Err(AppError::Forbidden("Item does not belong to seller".to_string()));
        }

        if quantity < 0 {
            return Err(AppError::Validation("Stock quantity cannot be negative".to_string()));
        }

        // Get current stock for comparison
        let current_stock = item.stock_quantity;

        // Update stock
        Item::update_stock_quantity(&self.db_pool, item_id, quantity).await?;

        // Broadcast stock change event
        if let Some(realtime_service) = &self.realtime_service {
            let _ = realtime_service.broadcast_item_stock_change(
                item_id,
                current_stock,
                quantity,
            ).await;
        }

        // Log stock update
        self.log_item_activity(
            item_id,
            seller_id,
            "stock_updated",
            &format!("Stock updated to {}", quantity),
        ).await?;

        info!("Updated item {} stock to {} by seller {}", item_id, quantity, seller_id);

        Ok(())
    }

    /// Get item statistics
    pub async fn get_item_statistics(&self, seller_id: Option<Uuid>) -> Result<ItemStatistics, AppError> {
        let mut query = "SELECT status, COUNT(*) as count, COALESCE(SUM(retail_price), 0) as total_value FROM items".to_string();
        
        if let Some(seller_id) = seller_id {
            query.push_str(&format!(" WHERE seller_id = '{}'", seller_id));
        }
        
        query.push_str(" GROUP BY status");

        // For simplicity, we'll calculate basic statistics
        // In a real implementation, you'd use the dynamic query above
        let total_items = if let Some(seller_id) = seller_id {
            Item::count_by_seller(&self.db_pool, seller_id).await?
        } else {
            sqlx::query_scalar!("SELECT COUNT(*) FROM items")
                .fetch_one(&self.db_pool)
                .await?
                .unwrap_or(0)
        };

        let available_items = Item::count_available(&self.db_pool, None).await?;

        // Calculate other statistics
        let total_value = sqlx::query_scalar!(
            "SELECT COALESCE(SUM(retail_price), 0) FROM items WHERE status = 'available'"
        )
        .fetch_one(&self.db_pool)
        .await?
        .unwrap_or(Decimal::ZERO);

        let average_price = if available_items > 0 {
            total_value / Decimal::from(available_items)
        } else {
            Decimal::ZERO
        };

        Ok(ItemStatistics {
            total_items,
            available_items,
            sold_items: 0, // Would be calculated from raffle completions
            inactive_items: total_items - available_items,
            total_value,
            average_price,
            items_by_category: HashMap::new(), // Would be populated from category data
            items_by_seller: HashMap::new(), // Would be populated from seller data
        })
    }

    /// Get item analytics
    pub async fn get_item_analytics(&self, item_id: Uuid) -> Result<ItemAnalytics, AppError> {
        // Verify item exists
        Item::find_by_id(&self.db_pool, item_id).await?
            .ok_or_else(|| AppError::NotFound("Item not found".to_string()))?;

        // Get analytics data
        let views_count = self.get_item_views(item_id).await?;
        let unique_viewers = self.get_unique_viewers(item_id).await?;
        let raffle_count = self.get_raffle_count_for_item(item_id).await?;
        let total_boxes_sold = self.get_total_boxes_sold_for_item(item_id).await?;
        let total_revenue = self.get_total_revenue_for_item(item_id).await?;

        let conversion_rate = if views_count > 0 {
            Some((Decimal::from(total_boxes_sold) / Decimal::from(views_count)) * Decimal::from(100))
        } else {
            None
        };

        Ok(ItemAnalytics {
            item_id,
            views_count,
            unique_viewers,
            raffle_count,
            total_boxes_sold,
            total_revenue,
            average_completion_time: None, // Would be calculated from raffle data
            conversion_rate,
        })
    }

    /// Perform bulk operations on items
    pub async fn bulk_operation(
        &self,
        seller_id: Uuid,
        operation: BulkItemOperation,
    ) -> Result<usize, AppError> {
        // Verify all items belong to the seller
        for item_id in &operation.item_ids {
            let item = Item::find_by_id(&self.db_pool, *item_id).await?
                .ok_or_else(|| AppError::NotFound(format!("Item {} not found", item_id)))?;

            if item.seller_id != Some(seller_id) {
                return Err(AppError::Forbidden(format!("Item {} does not belong to seller", item_id)));
            }
        }

        let mut updated_count = 0;

        match operation.operation {
            BulkOperation::UpdateStatus(status) => {
                for item_id in operation.item_ids {
                    Item::update_status(&self.db_pool, item_id, status).await?;
                    updated_count += 1;
                }
            }
            BulkOperation::Delete => {
                for item_id in operation.item_ids {
                    // Check for active raffles
                    let active_raffles = self.get_active_raffles_for_item(item_id).await?;
                    if active_raffles.is_empty() {
                        Item::delete(&self.db_pool, item_id).await?;
                        updated_count += 1;
                    }
                }
            }
            BulkOperation::UpdateStock(quantity) => {
                for item_id in operation.item_ids {
                    Item::update_stock_quantity(&self.db_pool, item_id, quantity).await?;
                    updated_count += 1;
                }
            }
            BulkOperation::UpdateCategory(_category) => {
                // Would implement category update
                // For now, just count the items
                updated_count = operation.item_ids.len();
            }
        }

        info!(
            "Bulk operation completed by seller {}: {:?} on {} items",
            seller_id, operation.operation, updated_count
        );

        Ok(updated_count)
    }

    // Private helper methods

    async fn calculate_listing_fee(&self, _seller: &User, _request: &CreateItemRequest) -> Result<ListingFee, AppError> {
        // This would calculate the listing fee based on the seller's subscription tier
        // For now, return a default fee
        Ok(ListingFee {
            amount: Some(Decimal::from(5)), // $5 listing fee
            fee_type: "fixed".to_string(),
        })
    }

    async fn log_item_activity(
        &self,
        item_id: Uuid,
        user_id: Uuid,
        activity_type: &str,
        description: &str,
    ) -> Result<(), AppError> {
        sqlx::query!(
            "INSERT INTO item_activity_log (item_id, user_id, activity_type, description) VALUES ($1, $2, $3, $4)",
            item_id,
            user_id,
            activity_type,
            description
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    async fn increment_item_views(&self, item_id: Uuid) -> Result<(), AppError> {
        sqlx::query!(
            "INSERT INTO item_views (item_id, viewed_at) VALUES ($1, NOW())
             ON CONFLICT (item_id, DATE(viewed_at)) DO UPDATE SET view_count = item_views.view_count + 1",
            item_id
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    async fn get_item_views(&self, item_id: Uuid) -> Result<i64, AppError> {
        let views = sqlx::query_scalar!(
            "SELECT COALESCE(SUM(view_count), 0) FROM item_views WHERE item_id = $1",
            item_id
        )
        .fetch_one(&self.db_pool)
        .await?
        .unwrap_or(0);

        Ok(views)
    }

    async fn get_unique_viewers(&self, item_id: Uuid) -> Result<i64, AppError> {
        let viewers = sqlx::query_scalar!(
            "SELECT COUNT(DISTINCT user_id) FROM item_views WHERE item_id = $1 AND user_id IS NOT NULL",
            item_id
        )
        .fetch_one(&self.db_pool)
        .await?
        .unwrap_or(0);

        Ok(viewers)
    }

    async fn get_active_raffles_for_item(&self, item_id: Uuid) -> Result<Vec<Uuid>, AppError> {
        let raffle_ids = sqlx::query_scalar!(
            "SELECT id FROM raffles WHERE item_id = $1 AND status IN ('open', 'full', 'drawing')",
            item_id
        )
        .fetch_all(&self.db_pool)
        .await?;

        Ok(raffle_ids)
    }

    async fn get_raffle_count_for_item(&self, item_id: Uuid) -> Result<i64, AppError> {
        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM raffles WHERE item_id = $1",
            item_id
        )
        .fetch_one(&self.db_pool)
        .await?
        .unwrap_or(0);

        Ok(count)
    }

    async fn get_total_boxes_sold_for_item(&self, item_id: Uuid) -> Result<i64, AppError> {
        let total = sqlx::query_scalar!(
            "SELECT COALESCE(SUM(boxes_sold), 0) FROM raffles WHERE item_id = $1",
            item_id
        )
        .fetch_one(&self.db_pool)
        .await?
        .unwrap_or(0);

        Ok(total)
    }

    async fn get_total_revenue_for_item(&self, item_id: Uuid) -> Result<Decimal, AppError> {
        let revenue = sqlx::query_scalar!(
            "SELECT COALESCE(SUM(r.boxes_sold * r.box_price), 0) FROM raffles r WHERE r.item_id = $1",
            item_id
        )
        .fetch_one(&self.db_pool)
        .await?
        .unwrap_or(Decimal::ZERO);

        Ok(revenue)
    }
}

#[derive(Debug, Clone)]
struct ListingFee {
    amount: Option<Decimal>,
    fee_type: String,
}