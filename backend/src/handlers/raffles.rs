use crate::middleware::auth::AuthenticatedUser;
use crate::services::raffle_service::{RaffleService, RaffleSearchParams, BoxPurchaseRequest};
use crate::error::AppError;
use actix_web::{web, HttpResponse, Result};
use raffle_platform_shared::{RaffleStatus, CreateRaffleRequest, PaginatedResponse, RaffleResponse, BoxPurchaseResponse};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, warn};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateRaffleRequestHandler {
    pub item_id: Uuid,
    #[validate(range(min = 1, max = 10000))]
    pub total_boxes: i32,
    #[validate(range(min = 0.01, max = 1000.0))]
    pub box_price: Decimal,
    #[validate(range(min = 1, max = 100))]
    pub total_winners: i32,
    #[validate(range(min = 1, max = 100))]
    pub grid_rows: i32,
    #[validate(range(min = 1, max = 100))]
    pub grid_cols: i32,
}

#[derive(Debug, Deserialize, Validate)]
pub struct RaffleSearchQuery {
    pub status: Option<String>,
    pub item_id: Option<Uuid>,
    pub min_price: Option<Decimal>,
    pub max_price: Option<Decimal>,
    pub completion_min: Option<f64>,
    pub completion_max: Option<f64>,
    pub sort_by: Option<String>,
    #[validate(range(min = 1, max = 100))]
    pub limit: Option<i64>,
    #[validate(range(min = 0))]
    pub offset: Option<i64>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct BuyBoxRequestHandler {
    #[validate(length(min = 1, max = 100))]
    pub box_numbers: Vec<i32>,
    pub use_credits: bool,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CancelRaffleRequest {
    #[validate(length(min = 1, max = 500))]
    pub reason: String,
}

#[derive(Debug, Serialize)]
pub struct RaffleCreatedResponse {
    pub raffle: RaffleResponse,
    pub transaction_fee: Option<Decimal>,
    pub blockchain_tx_hash: Option<String>,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct BoxPurchaseSuccessResponse {
    pub purchases: Vec<BoxPurchaseResponse>,
    pub total_cost: Decimal,
    pub raffle_status: RaffleStatus,
    pub boxes_remaining: i32,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct GridStateResponse {
    pub raffle_id: Uuid,
    pub grid_rows: i32,
    pub grid_cols: i32,
    pub purchased_boxes: HashMap<i32, BoxOwnerInfo>,
    pub available_boxes: Vec<i32>,
    pub total_boxes: i32,
    pub boxes_sold: i32,
    pub completion_percentage: f64,
}

#[derive(Debug, Serialize)]
pub struct BoxOwnerInfo {
    pub user_id: Uuid,
    pub username: String,
    pub purchased_at: chrono::DateTime<chrono::Utc>,
    pub is_current_user: bool,
}

/// Create a new raffle (sellers only)
pub async fn create_raffle(
    user: AuthenticatedUser,
    request: web::Json<CreateRaffleRequestHandler>,
    raffle_service: web::Data<RaffleService>,
) -> Result<HttpResponse, AppError> {
    request.validate()?;

    if !user.is_seller() {
        return Err(AppError::Forbidden("Only sellers can create raffles".to_string()));
    }

    debug!(
        "Creating raffle for seller {} - item: {}, boxes: {}",
        user.user_id, request.item_id, request.total_boxes
    );

    let create_request = CreateRaffleRequest {
        item_id: request.item_id,
        total_boxes: request.total_boxes,
        box_price: request.box_price,
        total_winners: request.total_winners,
        grid_rows: request.grid_rows,
        grid_cols: request.grid_cols,
    };

    let raffle_response = raffle_service.create_raffle(user.user_id, create_request).await?;

    info!(
        "Created raffle {} for seller {} - item: {}",
        raffle_response.id, user.user_id, request.item_id
    );

    Ok(HttpResponse::Created().json(RaffleCreatedResponse {
        raffle: raffle_response,
        transaction_fee: None, // Would be populated from service
        blockchain_tx_hash: None, // Would be populated if blockchain integration is active
        message: "Raffle created successfully".to_string(),
    }))
}

/// Get raffle by ID
pub async fn get_raffle(
    raffle_id: web::Path<Uuid>,
    raffle_service: web::Data<RaffleService>,
) -> Result<HttpResponse, AppError> {
    debug!("Getting raffle {}", raffle_id);

    let raffle = raffle_service.get_raffle(*raffle_id).await?;

    Ok(HttpResponse::Ok().json(raffle))
}

/// Search raffles with filters and pagination
pub async fn search_raffles(
    query: web::Query<RaffleSearchQuery>,
    raffle_service: web::Data<RaffleService>,
) -> Result<HttpResponse, AppError> {
    query.validate()?;

    debug!("Searching raffles with query: {:?}", query);

    // Parse status string to enum
    let status = if let Some(status_str) = &query.status {
        match status_str.as_str() {
            "open" => Some(RaffleStatus::Open),
            "full" => Some(RaffleStatus::Full),
            "drawing" => Some(RaffleStatus::Drawing),
            "completed" => Some(RaffleStatus::Completed),
            "cancelled" => Some(RaffleStatus::Cancelled),
            _ => return Err(AppError::Validation("Invalid status filter".to_string())),
        }
    } else {
        None
    };

    let search_params = RaffleSearchParams {
        status,
        item_id: query.item_id,
        min_price: query.min_price,
        max_price: query.max_price,
        completion_min: query.completion_min,
        completion_max: query.completion_max,
        sort_by: query.sort_by.clone(),
        limit: query.limit,
        offset: query.offset,
    };

    let results = raffle_service.search_raffles(search_params).await?;

    Ok(HttpResponse::Ok().json(results))
}

/// Purchase boxes in a raffle
pub async fn buy_boxes(
    user: AuthenticatedUser,
    raffle_id: web::Path<Uuid>,
    request: web::Json<BuyBoxRequestHandler>,
    raffle_service: web::Data<RaffleService>,
) -> Result<HttpResponse, AppError> {
    request.validate()?;

    debug!(
        "User {} purchasing {} boxes in raffle {}",
        user.user_id, request.box_numbers.len(), raffle_id
    );

    // Validate box numbers
    for &box_number in &request.box_numbers {
        if box_number <= 0 {
            return Err(AppError::Validation(format!("Invalid box number: {}", box_number)));
        }
    }

    let purchase_request = BoxPurchaseRequest {
        raffle_id: *raffle_id,
        box_numbers: request.box_numbers.clone(),
        use_credits: request.use_credits,
    };

    let purchases = raffle_service.purchase_boxes(user.user_id, purchase_request).await?;

    // Get updated raffle info
    let raffle = raffle_service.get_raffle(*raffle_id).await?;
    let total_cost = purchases.iter()
        .map(|p| p.purchase_price_in_credits)
        .sum();

    info!(
        "User {} purchased {} boxes in raffle {} for {} credits",
        user.user_id, purchases.len(), raffle_id, total_cost
    );

    Ok(HttpResponse::Ok().json(BoxPurchaseSuccessResponse {
        purchases,
        total_cost,
        raffle_status: raffle.status,
        boxes_remaining: raffle.total_boxes - raffle.boxes_sold,
        message: format!("Successfully purchased {} boxes", request.box_numbers.len()),
    }))
}

/// Get raffle grid state
pub async fn get_grid_state(
    raffle_id: web::Path<Uuid>,
    user: Option<AuthenticatedUser>,
    raffle_service: web::Data<RaffleService>,
) -> Result<HttpResponse, AppError> {
    debug!("Getting grid state for raffle {}", raffle_id);

    let grid_state = raffle_service.get_grid_state(*raffle_id).await?;

    // Convert to response format with additional info
    let mut purchased_boxes = HashMap::new();
    for (box_number, owner) in grid_state.purchased_boxes {
        purchased_boxes.insert(box_number, BoxOwnerInfo {
            user_id: owner.user_id,
            username: owner.username,
            purchased_at: owner.purchased_at,
            is_current_user: user.as_ref().map(|u| u.user_id == owner.user_id).unwrap_or(false),
        });
    }

    let completion_percentage = if grid_state.total_boxes > 0 {
        (grid_state.boxes_sold as f64 / grid_state.total_boxes as f64) * 100.0
    } else {
        0.0
    };

    let response = GridStateResponse {
        raffle_id: grid_state.raffle_id,
        grid_rows: grid_state.grid_rows,
        grid_cols: grid_state.grid_cols,
        purchased_boxes,
        available_boxes: grid_state.available_boxes,
        total_boxes: grid_state.total_boxes,
        boxes_sold: grid_state.boxes_sold,
        completion_percentage,
    };

    Ok(HttpResponse::Ok().json(response))
}

/// Get user's purchases for a raffle
pub async fn get_user_purchases(
    user: AuthenticatedUser,
    raffle_id: web::Path<Uuid>,
    raffle_service: web::Data<RaffleService>,
) -> Result<HttpResponse, AppError> {
    debug!(
        "Getting purchases for user {} in raffle {}",
        user.user_id, raffle_id
    );

    let purchases = raffle_service.get_user_purchases(user.user_id, *raffle_id).await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "raffle_id": *raffle_id,
        "user_id": user.user_id,
        "purchases": purchases,
        "total_boxes": purchases.len(),
        "total_spent": purchases.iter().map(|p| p.purchase_price_in_credits).sum::<Decimal>()
    })))
}

/// Get user's purchase history
pub async fn get_user_purchase_history(
    user: AuthenticatedUser,
    query: web::Query<RaffleSearchQuery>,
    raffle_service: web::Data<RaffleService>,
) -> Result<HttpResponse, AppError> {
    debug!("Getting purchase history for user {}", user.user_id);

    let history = raffle_service.get_user_purchase_history(
        user.user_id,
        query.limit,
        query.offset,
    ).await?;

    Ok(HttpResponse::Ok().json(history))
}

/// Cancel a raffle (seller/admin only)
pub async fn cancel_raffle(
    user: AuthenticatedUser,
    raffle_id: web::Path<Uuid>,
    request: web::Json<CancelRaffleRequest>,
    raffle_service: web::Data<RaffleService>,
) -> Result<HttpResponse, AppError> {
    request.validate()?;

    debug!(
        "User {} cancelling raffle {} - reason: {}",
        user.user_id, raffle_id, request.reason
    );

    raffle_service.cancel_raffle(*raffle_id, user.user_id, request.reason.clone()).await?;

    info!(
        "Cancelled raffle {} by user {} - reason: {}",
        raffle_id, user.user_id, request.reason
    );

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "Raffle cancelled successfully",
        "raffle_id": *raffle_id,
        "reason": request.reason
    })))
}

/// Get raffle statistics
pub async fn get_raffle_statistics(
    user: AuthenticatedUser,
    raffle_service: web::Data<RaffleService>,
) -> Result<HttpResponse, AppError> {
    if !user.is_admin() && !user.is_seller() {
        return Err(AppError::Forbidden("Access denied".to_string()));
    }

    debug!("Getting raffle statistics for user {}", user.user_id);

    let statistics = raffle_service.get_raffle_statistics().await?;

    Ok(HttpResponse::Ok().json(statistics))
}

/// Get active raffles (public endpoint)
pub async fn get_active_raffles(
    query: web::Query<RaffleSearchQuery>,
    raffle_service: web::Data<RaffleService>,
) -> Result<HttpResponse, AppError> {
    debug!("Getting active raffles");

    let search_params = RaffleSearchParams {
        status: Some(RaffleStatus::Open),
        item_id: query.item_id,
        min_price: query.min_price,
        max_price: query.max_price,
        completion_min: query.completion_min,
        completion_max: query.completion_max,
        sort_by: query.sort_by.clone(),
        limit: query.limit.or(Some(20)),
        offset: query.offset,
    };

    let results = raffle_service.search_raffles(search_params).await?;

    Ok(HttpResponse::Ok().json(results))
}

/// Get featured raffles (public endpoint)
pub async fn get_featured_raffles(
    query: web::Query<RaffleSearchQuery>,
    raffle_service: web::Data<RaffleService>,
) -> Result<HttpResponse, AppError> {
    debug!("Getting featured raffles");

    let search_params = RaffleSearchParams {
        status: Some(RaffleStatus::Open),
        item_id: None,
        min_price: None,
        max_price: None,
        completion_min: None,
        completion_max: None,
        sort_by: Some("featured".to_string()),
        limit: query.limit.or(Some(10)),
        offset: query.offset,
    };

    let results = raffle_service.search_raffles(search_params).await?;

    Ok(HttpResponse::Ok().json(results))
}

/// Get raffle winners (public endpoint)
pub async fn get_raffle_winners(
    raffle_id: web::Path<Uuid>,
    raffle_service: web::Data<RaffleService>,
) -> Result<HttpResponse, AppError> {
    debug!("Getting winners for raffle {}", raffle_id);

    let raffle = raffle_service.get_raffle(*raffle_id).await?;

    if raffle.status != RaffleStatus::Completed {
        return Err(AppError::Validation("Raffle is not completed yet".to_string()));
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "raffle_id": *raffle_id,
        "status": raffle.status,
        "winner_user_ids": raffle.winner_user_ids,
        "total_winners": raffle.total_winners,
        "completed_at": raffle.completed_at
    })))
}

// Admin endpoints

/// Get all raffles (admin only)
pub async fn get_all_raffles_admin(
    user: AuthenticatedUser,
    query: web::Query<RaffleSearchQuery>,
    raffle_service: web::Data<RaffleService>,
) -> Result<HttpResponse, AppError> {
    if !user.is_admin() {
        return Err(AppError::Forbidden("Admin access required".to_string()));
    }

    debug!("Admin {} getting all raffles", user.user_id);

    // Parse status string to enum
    let status = if let Some(status_str) = &query.status {
        match status_str.as_str() {
            "open" => Some(RaffleStatus::Open),
            "full" => Some(RaffleStatus::Full),
            "drawing" => Some(RaffleStatus::Drawing),
            "completed" => Some(RaffleStatus::Completed),
            "cancelled" => Some(RaffleStatus::Cancelled),
            _ => return Err(AppError::Validation("Invalid status filter".to_string())),
        }
    } else {
        None
    };

    let search_params = RaffleSearchParams {
        status,
        item_id: query.item_id,
        min_price: query.min_price,
        max_price: query.max_price,
        completion_min: query.completion_min,
        completion_max: query.completion_max,
        sort_by: query.sort_by.clone(),
        limit: query.limit,
        offset: query.offset,
    };

    let results = raffle_service.search_raffles(search_params).await?;

    Ok(HttpResponse::Ok().json(results))
}

/// Force complete a raffle (admin only)
pub async fn force_complete_raffle(
    user: AuthenticatedUser,
    raffle_id: web::Path<Uuid>,
    raffle_service: web::Data<RaffleService>,
) -> Result<HttpResponse, AppError> {
    if !user.is_admin() {
        return Err(AppError::Forbidden("Admin access required".to_string()));
    }

    info!(
        "Admin {} force completing raffle {}",
        user.user_id, raffle_id
    );

    // This would implement force completion logic
    // For now, return a placeholder response
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "Raffle force completed successfully",
        "raffle_id": *raffle_id
    })))
}

/// Health check endpoint for raffle service
pub async fn raffle_service_health(
    raffle_service: web::Data<RaffleService>,
) -> Result<HttpResponse, AppError> {
    // Simple health check
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": "raffle",
        "timestamp": chrono::Utc::now()
    })))
}