use crate::models::raffle::{Raffle, BoxPurchase};
use crate::models::item::Item;
use crate::models::user::User;
use crate::services::credit_service::{CreditService, CreditRedemptionRequest, CreditIssuanceRequest};
use crate::services::blockchain_service::BlockchainService;
use crate::services::notification_service::NotificationService;
use crate::services::realtime_service::RealtimeService;
use crate::error::AppError;
use chrono::{DateTime, Utc};
use raffle_platform_shared::{RaffleStatus, CreateRaffleRequest, RaffleResponse, BoxPurchaseResponse, PaginatedResponse, CreditSource, CreditType};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Raffle management service handles all raffle-related operations
#[derive(Clone)]
pub struct RaffleService {
    db_pool: PgPool,
    credit_service: CreditService,
    blockchain_service: BlockchainService,
    notification_service: NotificationService,
    realtime_service: RealtimeService,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaffleSearchParams {
    pub status: Option<RaffleStatus>,
    pub item_id: Option<Uuid>,
    pub min_price: Option<Decimal>,
    pub max_price: Option<Decimal>,
    pub completion_min: Option<f64>, // 0-100 percentage
    pub completion_max: Option<f64>,
    pub sort_by: Option<String>, // "created_asc", "created_desc", "price_asc", "price_desc", "completion"
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoxPurchaseRequest {
    pub raffle_id: Uuid,
    pub box_numbers: Vec<i32>,
    pub use_credits: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaffleStatistics {
    pub total_raffles: i64,
    pub active_raffles: i64,
    pub completed_raffles: i64,
    pub cancelled_raffles: i64,
    pub total_revenue: Decimal,
    pub total_boxes_sold: i64,
    pub average_completion_time: Option<i64>, // minutes
    pub completion_rate: f64, // percentage
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaffleAnalytics {
    pub raffle_id: Uuid,
    pub views_count: i64,
    pub unique_viewers: i64,
    pub conversion_rate: Option<Decimal>, // views to purchases
    pub average_boxes_per_user: Option<Decimal>,
    pub time_to_completion: Option<i64>, // minutes
    pub peak_concurrent_users: i64,
    pub revenue_per_hour: Option<Decimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridState {
    pub raffle_id: Uuid,
    pub grid_rows: i32,
    pub grid_cols: i32,
    pub purchased_boxes: HashMap<i32, BoxOwner>,
    pub available_boxes: Vec<i32>,
    pub total_boxes: i32,
    pub boxes_sold: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoxOwner {
    pub user_id: Uuid,
    pub username: String,
    pub purchased_at: DateTime<Utc>,
}

impl RaffleService {
    /// Create a new raffle service
    pub fn new(
        db_pool: PgPool,
        credit_service: CreditService,
        blockchain_service: BlockchainService,
        notification_service: NotificationService,
        realtime_service: RealtimeService,
    ) -> Self {
        Self {
            db_pool,
            credit_service,
            blockchain_service,
            notification_service,
            realtime_service,
        }
    }

    /// Create a new raffle
    pub async fn create_raffle(
        &self,
        seller_id: Uuid,
        request: CreateRaffleRequest,
    ) -> Result<RaffleResponse, AppError> {
        // Validate seller exists and is active
        let seller = User::find_by_id(&self.db_pool, seller_id).await?
            .ok_or_else(|| AppError::NotFound("Seller not found".to_string()))?;

        if !seller.is_seller() {
            return Err(AppError::Forbidden("User is not a seller".to_string()));
        }

        // Validate item exists and belongs to seller
        let item = Item::find_by_id(&self.db_pool, request.item_id).await?
            .ok_or_else(|| AppError::NotFound("Item not found".to_string()))?;

        if item.seller_id != Some(seller_id) {
            return Err(AppError::Forbidden("Item does not belong to seller".to_string()));
        }

        if !item.is_available_for_raffle() {
            return Err(AppError::Validation("Item is not available for raffle".to_string()));
        }

        // Validate raffle parameters
        self.validate_raffle_parameters(&request)?;

        // Check if there's already an active raffle for this item
        let existing_raffles = Raffle::find_by_item(&self.db_pool, request.item_id).await?;
        let has_active_raffle = existing_raffles.iter().any(|r| {
            matches!(r.status, RaffleStatus::Open | RaffleStatus::Full | RaffleStatus::Drawing)
        });

        if has_active_raffle {
            return Err(AppError::Validation("Item already has an active raffle".to_string()));
        }

        // Calculate transaction fee
        let transaction_fee = self.calculate_transaction_fee(&seller, &request).await?;

        // Create the raffle
        let raffle = Raffle::create(&self.db_pool, request, Some(transaction_fee)).await?;

        // Create blockchain raffle if needed
        let blockchain_raffle_id = self.create_blockchain_raffle(&raffle).await?;

        // Update raffle with blockchain ID if created
        if let Some(blockchain_id) = blockchain_raffle_id {
            self.update_raffle_blockchain_id(raffle.id, blockchain_id).await?;
        }

        // Log raffle creation
        self.log_raffle_activity(
            raffle.id,
            seller_id,
            "raffle_created",
            &format!("Raffle created for item '{}'", item.name),
        ).await?;

        // Broadcast raffle creation event
        let _ = self.realtime_service.broadcast_event(
            crate::services::realtime_service::RealtimeEvent::RaffleCreated {
                raffle_id: raffle.id,
                item_id: request.item_id,
                seller_id,
                total_boxes: raffle.total_boxes,
                box_price: raffle.box_price,
                created_at: raffle.created_at,
            }
        ).await;

        info!(
            "Created raffle {} for item {} by seller {}",
            raffle.id, request.item_id, seller_id
        );

        Ok(raffle.to_response(Some(item)))
    }

    /// Get raffle by ID
    pub async fn get_raffle(&self, raffle_id: Uuid) -> Result<RaffleResponse, AppError> {
        let (raffle, item) = Raffle::find_with_item(&self.db_pool, raffle_id).await?
            .ok_or_else(|| AppError::NotFound("Raffle not found".to_string()))?;

        // Increment view count
        self.increment_raffle_views(raffle_id).await?;

        Ok(raffle.to_response(Some(item)))
    }

    /// Search raffles with filters
    pub async fn search_raffles(
        &self,
        params: RaffleSearchParams,
    ) -> Result<PaginatedResponse<RaffleResponse>, AppError> {
        let limit = params.limit.unwrap_or(20).min(100);
        let offset = params.offset.unwrap_or(0);

        // For simplicity, we'll use the existing find_active method
        // In a real implementation, you'd build dynamic queries based on params
        let raffles = if params.status.is_none() || params.status == Some(RaffleStatus::Open) {
            Raffle::find_active(&self.db_pool, limit, offset).await?
        } else {
            // Would implement other status filters
            Vec::new()
        };

        let total = Raffle::count_active(&self.db_pool).await?;

        let mut raffle_responses = Vec::new();
        for raffle in raffles {
            let item = Item::find_by_id(&self.db_pool, raffle.item_id).await?;
            raffle_responses.push(raffle.to_response(item));
        }

        Ok(PaginatedResponse {
            data: raffle_responses,
            total,
            limit,
            offset,
            has_more: offset + limit < total,
        })
    }

    /// Purchase boxes in a raffle
    pub async fn purchase_boxes(
        &self,
        user_id: Uuid,
        request: BoxPurchaseRequest,
    ) -> Result<Vec<BoxPurchaseResponse>, AppError> {
        // Validate user exists
        let user = User::find_by_id(&self.db_pool, user_id).await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        // Get raffle details
        let raffle = Raffle::find_by_id(&self.db_pool, request.raffle_id).await?
            .ok_or_else(|| AppError::NotFound("Raffle not found".to_string()))?;

        if raffle.status != RaffleStatus::Open {
            return Err(AppError::Validation("Raffle is not open for purchases".to_string()));
        }

        // Validate box numbers
        for &box_number in &request.box_numbers {
            if box_number < 1 || box_number > raffle.total_boxes {
                return Err(AppError::Validation(format!("Invalid box number: {}", box_number)));
            }

            // Check if box is already purchased
            if BoxPurchase::is_box_purchased(&self.db_pool, request.raffle_id, box_number).await? {
                return Err(AppError::Validation(format!("Box {} is already purchased", box_number)));
            }
        }

        // Check if raffle would be overfilled
        if raffle.boxes_sold + request.box_numbers.len() as i32 > raffle.total_boxes {
            return Err(AppError::Validation("Not enough boxes available".to_string()));
        }

        let total_cost = raffle.box_price * Decimal::from(request.box_numbers.len());

        // Process payment
        let mut purchases = Vec::new();
        let mut transaction_id = None;

        if request.use_credits {
            // Use credits for payment
            let redemption_request = CreditRedemptionRequest {
                user_id,
                amount: total_cost,
                item_id: Some(raffle.item_id),
                credit_type: None,
                description: format!("Box purchase for raffle {}", request.raffle_id),
            };

            let redemption_result = self.credit_service.redeem_credits(redemption_request).await?;
            transaction_id = Some(Uuid::new_v4()); // Generate transaction ID
        } else {
            // Would implement other payment methods (direct payment, etc.)
            return Err(AppError::Validation("Only credit payments are currently supported".to_string()));
        }

        // Create box purchases
        for box_number in request.box_numbers {
            let purchase = BoxPurchase::create(
                &self.db_pool,
                request.raffle_id,
                user_id,
                box_number,
                raffle.box_price,
                transaction_id,
            ).await?;

            purchases.push(purchase);
        }

        // Update raffle boxes sold count
        let new_boxes_sold = raffle.boxes_sold + purchases.len() as i32;
        Raffle::update_boxes_sold(&self.db_pool, request.raffle_id, new_boxes_sold).await?;

        // Broadcast box purchase events
        for purchase in &purchases {
            let _ = self.realtime_service.broadcast_box_purchase(
                request.raffle_id,
                user_id,
                purchase.box_number,
                raffle.total_boxes - new_boxes_sold,
                (new_boxes_sold as f64 / raffle.total_boxes as f64) * 100.0,
            ).await;
        }

        // Check if raffle is now full
        if new_boxes_sold >= raffle.total_boxes {
            Raffle::update_status(&self.db_pool, request.raffle_id, RaffleStatus::Full).await?;
            
            // Broadcast raffle full event
            let _ = self.realtime_service.broadcast_raffle_full(
                request.raffle_id,
                raffle.item_id,
                raffle.total_boxes,
            ).await;
            
            // Trigger winner selection process
            self.initiate_winner_selection(request.raffle_id).await?;
        }

        // Send notifications
        for purchase in &purchases {
            self.notification_service.send_box_purchase_notification(
                user_id,
                request.raffle_id,
                "Item Name".to_string(), // Would get from item
                purchase.box_number as u32,
                raffle.total_boxes as u32,
            ).await?;
        }

        // Log purchases
        self.log_raffle_activity(
            request.raffle_id,
            user_id,
            "boxes_purchased",
            &format!("Purchased {} boxes", purchases.len()),
        ).await?;

        info!(
            "User {} purchased {} boxes in raffle {}",
            user_id, purchases.len(), request.raffle_id
        );

        Ok(purchases.into_iter().map(|p| p.to_response()).collect())
    }

    /// Get raffle grid state
    pub async fn get_grid_state(&self, raffle_id: Uuid) -> Result<GridState, AppError> {
        let raffle = Raffle::find_by_id(&self.db_pool, raffle_id).await?
            .ok_or_else(|| AppError::NotFound("Raffle not found".to_string()))?;

        let purchases = BoxPurchase::find_by_raffle(&self.db_pool, raffle_id).await?;

        let mut purchased_boxes = HashMap::new();
        let mut available_boxes = Vec::new();

        // Get user details for purchased boxes
        for purchase in purchases {
            let user = User::find_by_id(&self.db_pool, purchase.user_id).await?
                .ok_or_else(|| AppError::Internal("Purchase user not found".to_string()))?;

            purchased_boxes.insert(purchase.box_number, BoxOwner {
                user_id: purchase.user_id,
                username: user.email, // Using email as username for now
                purchased_at: purchase.created_at,
            });
        }

        // Calculate available boxes
        for box_number in 1..=raffle.total_boxes {
            if !purchased_boxes.contains_key(&box_number) {
                available_boxes.push(box_number);
            }
        }

        Ok(GridState {
            raffle_id,
            grid_rows: raffle.grid_rows,
            grid_cols: raffle.grid_cols,
            purchased_boxes,
            available_boxes,
            total_boxes: raffle.total_boxes,
            boxes_sold: raffle.boxes_sold,
        })
    }

    /// Get user's box purchases for a raffle
    pub async fn get_user_purchases(
        &self,
        user_id: Uuid,
        raffle_id: Uuid,
    ) -> Result<Vec<BoxPurchaseResponse>, AppError> {
        let purchases = BoxPurchase::find_user_purchases_for_raffle(
            &self.db_pool,
            user_id,
            raffle_id,
        ).await?;

        Ok(purchases.into_iter().map(|p| p.to_response()).collect())
    }

    /// Get user's purchase history
    pub async fn get_user_purchase_history(
        &self,
        user_id: Uuid,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<PaginatedResponse<BoxPurchaseResponse>, AppError> {
        let limit = limit.unwrap_or(20).min(100);
        let offset = offset.unwrap_or(0);

        let purchases = BoxPurchase::find_by_user(&self.db_pool, user_id, limit, offset).await?;

        // Get total count (simplified)
        let total = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM box_purchases WHERE user_id = $1",
            user_id
        )
        .fetch_one(&self.db_pool)
        .await?
        .unwrap_or(0);

        let purchase_responses: Vec<BoxPurchaseResponse> = purchases
            .into_iter()
            .map(|p| p.to_response())
            .collect();

        Ok(PaginatedResponse {
            data: purchase_responses,
            total,
            limit,
            offset,
            has_more: offset + limit < total,
        })
    }

    /// Cancel a raffle (admin/seller only)
    pub async fn cancel_raffle(
        &self,
        raffle_id: Uuid,
        user_id: Uuid,
        reason: String,
    ) -> Result<(), AppError> {
        let raffle = Raffle::find_by_id(&self.db_pool, raffle_id).await?
            .ok_or_else(|| AppError::NotFound("Raffle not found".to_string()))?;

        // Check permissions (seller or admin)
        let user = User::find_by_id(&self.db_pool, user_id).await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        let item = Item::find_by_id(&self.db_pool, raffle.item_id).await?
            .ok_or_else(|| AppError::NotFound("Item not found".to_string()))?;

        if !user.is_admin() && item.seller_id != Some(user_id) {
            return Err(AppError::Forbidden("Not authorized to cancel this raffle".to_string()));
        }

        if raffle.status == RaffleStatus::Completed {
            return Err(AppError::Validation("Cannot cancel completed raffle".to_string()));
        }

        // Refund all purchases
        let purchases = BoxPurchase::find_by_raffle(&self.db_pool, raffle_id).await?;
        for purchase in purchases {
            // Issue refund credits
            let refund_request = CreditIssuanceRequest {
                user_id: purchase.user_id,
                amount: purchase.purchase_price_in_credits,
                source: CreditSource::Refund,
                credit_type: CreditType::General,
                redeemable_on_item_id: None,
                expires_at: None,
                description: format!("Refund for cancelled raffle {}", raffle_id),
            };

            self.credit_service.issue_credits(refund_request).await?;
        }

        // Update raffle status
        Raffle::update_status(&self.db_pool, raffle_id, RaffleStatus::Cancelled).await?;

        // Broadcast raffle cancellation event
        let _ = self.realtime_service.broadcast_event(
            crate::services::realtime_service::RealtimeEvent::RaffleCancelled {
                raffle_id,
                item_id: raffle.item_id,
                reason: reason.clone(),
                cancelled_at: chrono::Utc::now(),
            }
        ).await;

        // Log cancellation
        self.log_raffle_activity(
            raffle_id,
            user_id,
            "raffle_cancelled",
            &format!("Raffle cancelled: {}", reason),
        ).await?;

        info!("Cancelled raffle {} by user {}: {}", raffle_id, user_id, reason);

        Ok(())
    }

    /// Get raffle statistics
    pub async fn get_raffle_statistics(&self) -> Result<RaffleStatistics, AppError> {
        let total_raffles = sqlx::query_scalar!("SELECT COUNT(*) FROM raffles")
            .fetch_one(&self.db_pool)
            .await?
            .unwrap_or(0);

        let active_raffles = Raffle::count_active(&self.db_pool).await?;

        let completed_raffles = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM raffles WHERE status = 'completed'"
        )
        .fetch_one(&self.db_pool)
        .await?
        .unwrap_or(0);

        let cancelled_raffles = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM raffles WHERE status = 'cancelled'"
        )
        .fetch_one(&self.db_pool)
        .await?
        .unwrap_or(0);

        let total_revenue = sqlx::query_scalar!(
            "SELECT COALESCE(SUM(boxes_sold * box_price), 0) FROM raffles WHERE status = 'completed'"
        )
        .fetch_one(&self.db_pool)
        .await?
        .unwrap_or(Decimal::ZERO);

        let total_boxes_sold = sqlx::query_scalar!(
            "SELECT COALESCE(SUM(boxes_sold), 0) FROM raffles"
        )
        .fetch_one(&self.db_pool)
        .await?
        .unwrap_or(0);

        let completion_rate = if total_raffles > 0 {
            (completed_raffles as f64 / total_raffles as f64) * 100.0
        } else {
            0.0
        };

        Ok(RaffleStatistics {
            total_raffles,
            active_raffles,
            completed_raffles,
            cancelled_raffles,
            total_revenue,
            total_boxes_sold,
            average_completion_time: None, // Would calculate from raffle data
            completion_rate,
        })
    }

    // Private helper methods

    fn validate_raffle_parameters(&self, request: &CreateRaffleRequest) -> Result<(), AppError> {
        if request.total_boxes <= 0 {
            return Err(AppError::Validation("Total boxes must be positive".to_string()));
        }

        if request.box_price <= Decimal::ZERO {
            return Err(AppError::Validation("Box price must be positive".to_string()));
        }

        if request.total_winners <= 0 {
            return Err(AppError::Validation("Total winners must be positive".to_string()));
        }

        if request.total_winners > request.total_boxes {
            return Err(AppError::Validation("Total winners cannot exceed total boxes".to_string()));
        }

        if request.grid_rows <= 0 || request.grid_cols <= 0 {
            return Err(AppError::Validation("Grid dimensions must be positive".to_string()));
        }

        if request.grid_rows * request.grid_cols < request.total_boxes {
            return Err(AppError::Validation("Grid is too small for the number of boxes".to_string()));
        }

        Ok(())
    }

    async fn calculate_transaction_fee(&self, _seller: &User, _request: &CreateRaffleRequest) -> Result<Decimal, AppError> {
        // Calculate transaction fee based on seller's subscription tier
        // For now, return a fixed percentage
        Ok(Decimal::from(5)) // 5% transaction fee
    }

    async fn create_blockchain_raffle(&self, raffle: &Raffle) -> Result<Option<u64>, AppError> {
        // Create raffle on blockchain
        match self.blockchain_service.create_raffle(
            raffle.item_id.as_u128() as u64,
            raffle.total_boxes as u64,
            (raffle.box_price * Decimal::from(1_000_000_000_000_000_000u64)).to_u128().unwrap_or(0), // Convert to wei
            raffle.total_winners as u64,
        ).await {
            Ok(_tx_id) => {
                // Would get the actual blockchain raffle ID from the transaction
                Ok(Some(0)) // Placeholder
            }
            Err(e) => {
                warn!("Failed to create blockchain raffle: {}", e);
                Ok(None)
            }
        }
    }

    async fn update_raffle_blockchain_id(&self, raffle_id: Uuid, blockchain_id: u64) -> Result<(), AppError> {
        sqlx::query!(
            "UPDATE raffles SET blockchain_raffle_id = $1 WHERE id = $2",
            blockchain_id as i64,
            raffle_id
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    async fn initiate_winner_selection(&self, raffle_id: Uuid) -> Result<(), AppError> {
        // Update status to drawing
        Raffle::update_status(&self.db_pool, raffle_id, RaffleStatus::Drawing).await?;

        // This would trigger the blockchain winner selection process
        // For now, we'll just log it
        info!("Initiated winner selection for raffle {}", raffle_id);

        Ok(())
    }

    async fn log_raffle_activity(
        &self,
        raffle_id: Uuid,
        user_id: Uuid,
        activity_type: &str,
        description: &str,
    ) -> Result<(), AppError> {
        sqlx::query!(
            "INSERT INTO raffle_activity_log (raffle_id, user_id, activity_type, description) VALUES ($1, $2, $3, $4)",
            raffle_id,
            user_id,
            activity_type,
            description
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    async fn increment_raffle_views(&self, raffle_id: Uuid) -> Result<(), AppError> {
        sqlx::query!(
            "INSERT INTO raffle_views (raffle_id, viewed_at) VALUES ($1, NOW())
             ON CONFLICT (raffle_id, DATE(viewed_at)) DO UPDATE SET view_count = raffle_views.view_count + 1",
            raffle_id
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }
}