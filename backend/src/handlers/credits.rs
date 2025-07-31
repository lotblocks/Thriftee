use crate::middleware::auth::AuthenticatedUser;
use crate::services::credit_service::{
    CreditService, CreditIssuanceRequest, CreditRedemptionRequest, CreditBalance,
};
use crate::error::AppError;
use actix_web::{web, HttpResponse, Result};
use chrono::{DateTime, Utc};
use raffle_platform_shared::{CreditSource, CreditType, CreditResponse};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};
use uuid::Uuid;
use validator::Validate;
use crate::utils::validation::validation_errors_to_app_error;

#[derive(Debug, Deserialize, Validate)]
pub struct GetCreditBalanceQuery {
    #[validate(range(min = 1, max = 1000))]
    pub limit: Option<i64>,
    pub include_used: Option<bool>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct GetExpiringCreditsQuery {
    #[validate(range(min = 1, max = 365))]
    pub days: Option<i64>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct RedeemCreditsRequest {
    #[validate(range(min = 0.01))]
    pub amount: Decimal,
    pub item_id: Option<Uuid>,
    pub credit_type: Option<CreditType>,
    #[validate(length(min = 1, max = 500))]
    pub description: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct IssueCreditsRequest {
    pub user_id: Uuid,
    #[validate(range(min = 0.01))]
    pub amount: Decimal,
    pub source: CreditSource,
    pub credit_type: CreditType,
    pub redeemable_on_item_id: Option<Uuid>,
    pub expires_at: Option<DateTime<Utc>>,
    #[validate(length(min = 1, max = 500))]
    pub description: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct IssueBonusCreditsRequest {
    #[validate(range(min = 0.01))]
    pub amount: Decimal,
    pub expires_at: Option<DateTime<Utc>>,
    #[validate(length(min = 1, max = 500))]
    pub description: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct RedeemFreeItemRequest {
    pub item_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct CreditBalanceResponse {
    pub balance: CreditBalance,
    pub recent_credits: Vec<CreditResponse>,
}

#[derive(Debug, Serialize)]
pub struct CreditRedemptionResponse {
    pub success: bool,
    pub total_amount_used: Decimal,
    pub remaining_balance: Decimal,
    pub used_credits_count: usize,
}

#[derive(Debug, Serialize)]
pub struct ExpiringCreditsResponse {
    pub expiring_credits: Vec<CreditResponse>,
    pub total_amount: Decimal,
    pub days_until_expiry: i64,
}

#[derive(Debug, Serialize)]
pub struct FreeItemsResponse {
    pub available_items: Vec<serde_json::Value>, // Will be Item objects
    pub total_expiring_credits: Decimal,
}

/// Get user's credit balance and recent history
#[actix_web::get("/balance")]
pub async fn get_credit_balance(
    user: AuthenticatedUser,
    query: web::Query<GetCreditBalanceQuery>,
    credit_service: web::Data<CreditService>,
) -> Result<HttpResponse, AppError> {
    query.validate().map_err(validation_errors_to_app_error)?;

    debug!("Getting credit balance for user: {}", user.user_id);

    // Get balance
    let balance = credit_service.get_user_balance(user.user_id).await?;

    // Get recent credit history
    let recent_credits = credit_service
        .get_user_credit_history(
            user.user_id,
            query.include_used.unwrap_or(false),
            query.limit.or(Some(10)),
        )
        .await?;

    let response = CreditBalanceResponse {
        balance,
        recent_credits,
    };

    Ok(HttpResponse::Ok().json(response))
}

/// Get user's credit history
#[actix_web::get("/history")]
pub async fn get_credit_history(
    user: AuthenticatedUser,
    query: web::Query<GetCreditBalanceQuery>,
    credit_service: web::Data<CreditService>,
) -> Result<HttpResponse, AppError> {
    query.validate().map_err(validation_errors_to_app_error)?;

    debug!("Getting credit history for user: {}", user.user_id);

    let credits = credit_service
        .get_user_credit_history(
            user.user_id,
            query.include_used.unwrap_or(true),
            query.limit.or(Some(50)),
        )
        .await?;

    Ok(HttpResponse::Ok().json(credits))
}

/// Get expiring credits for user
#[actix_web::get("/expiring")]
pub async fn get_expiring_credits(
    user: AuthenticatedUser,
    query: web::Query<GetExpiringCreditsQuery>,
    credit_service: web::Data<CreditService>,
) -> Result<HttpResponse, AppError> {
    query.validate().map_err(validation_errors_to_app_error)?;

    let days = query.days.unwrap_or(30);
    debug!("Getting expiring credits for user: {} (within {} days)", user.user_id, days);

    let expiring_credits = credit_service
        .get_expiring_credits(user.user_id, days)
        .await?;

    let total_amount: Decimal = expiring_credits.iter().map(|c| c.amount).sum();

    let response = ExpiringCreditsResponse {
        expiring_credits,
        total_amount,
        days_until_expiry: days,
    };

    Ok(HttpResponse::Ok().json(response))
}

/// Redeem credits for a purchase
#[actix_web::post("/redeem")]
pub async fn redeem_credits(
    user: AuthenticatedUser,
    request: web::Json<RedeemCreditsRequest>,
    credit_service: web::Data<CreditService>,
) -> Result<HttpResponse, AppError> {
    request.validate().map_err(validation_errors_to_app_error)?;

    info!(
        "User {} redeeming {} credits for item {:?}",
        user.user_id, request.amount, request.item_id
    );

    let redemption_request = CreditRedemptionRequest {
        user_id: user.user_id,
        amount: request.amount,
        item_id: request.item_id,
        credit_type: request.credit_type,
        description: request.description.clone(),
    };

    let result = credit_service.redeem_credits(redemption_request).await?;

    let response = CreditRedemptionResponse {
        success: true,
        total_amount_used: result.total_amount_used,
        remaining_balance: result.remaining_balance,
        used_credits_count: result.used_credits.len(),
    };

    Ok(HttpResponse::Ok().json(response))
}

/// Check if user has sufficient credits
#[actix_web::get("/check-sufficient")]
pub async fn check_sufficient_credits(
    user: AuthenticatedUser,
    query: web::Query<RedeemCreditsRequest>,
    credit_service: web::Data<CreditService>,
) -> Result<HttpResponse, AppError> {
    query.validate().map_err(validation_errors_to_app_error)?;

    debug!(
        "Checking if user {} has sufficient credits: {} for item {:?}",
        user.user_id, query.amount, query.item_id
    );

    let has_sufficient = credit_service
        .check_sufficient_credits(
            user.user_id,
            query.amount,
            query.item_id,
            query.credit_type,
        )
        .await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "has_sufficient_credits": has_sufficient,
        "required_amount": query.amount,
        "item_id": query.item_id
    })))
}

/// Get free items available for credit redemption
#[actix_web::get("/free-items")]
pub async fn get_free_items(
    user: AuthenticatedUser,
    credit_service: web::Data<CreditService>,
) -> Result<HttpResponse, AppError> {
    debug!("Getting free items for user: {}", user.user_id);

    let free_items = credit_service.get_free_items_for_credits(user.user_id).await?;
    
    // Get total expiring credits
    let expiring_credits = credit_service.get_expiring_credits(user.user_id, 7).await?;
    let total_expiring_credits: Decimal = expiring_credits.iter().map(|c| c.amount).sum();

    let response = FreeItemsResponse {
        available_items: free_items.into_iter().map(|item| {
            serde_json::json!({
                "id": item.id,
                "title": item.title,
                "description": item.description,
                "price": item.price,
                "category": item.category,
                "image_urls": item.image_urls,
                "status": item.status
            })
        }).collect(),
        total_expiring_credits,
    };

    Ok(HttpResponse::Ok().json(response))
}

/// Redeem free item with expiring credits
#[actix_web::post("/redeem-free-item")]
pub async fn redeem_free_item(
    user: AuthenticatedUser,
    request: web::Json<RedeemFreeItemRequest>,
    credit_service: web::Data<CreditService>,
) -> Result<HttpResponse, AppError> {
    info!(
        "User {} redeeming free item: {}",
        user.user_id, request.item_id
    );

    let result = credit_service
        .redeem_free_item(user.user_id, request.item_id)
        .await?;

    let response = CreditRedemptionResponse {
        success: true,
        total_amount_used: result.total_amount_used,
        remaining_balance: result.remaining_balance,
        used_credits_count: result.used_credits.len(),
    };

    Ok(HttpResponse::Ok().json(response))
}

// Admin endpoints

/// Issue credits to a user (admin only)
#[actix_web::post("/admin/issue")]
pub async fn issue_credits(
    user: AuthenticatedUser,
    request: web::Json<IssueCreditsRequest>,
    credit_service: web::Data<CreditService>,
) -> Result<HttpResponse, AppError> {
    // Check if user is admin
    if !user.is_admin() {
        return Err(AppError::Forbidden("Admin access required".to_string()));
    }

    request.validate().map_err(validation_errors_to_app_error)?;

    info!(
        "Admin {} issuing {} credits to user {} (source: {:?})",
        user.user_id, request.amount, request.user_id, request.source
    );

    let issuance_request = CreditIssuanceRequest {
        user_id: request.user_id,
        amount: request.amount,
        source: request.source,
        credit_type: request.credit_type,
        redeemable_on_item_id: request.redeemable_on_item_id,
        expires_at: request.expires_at,
        description: request.description.clone(),
    };

    let credit = credit_service.issue_credits(issuance_request).await?;

    Ok(HttpResponse::Created().json(credit.to_response()))
}

/// Issue bonus credits to a user (admin only)
#[actix_web::post("/admin/issue-bonus/{user_id}")]
pub async fn issue_bonus_credits(
    user: AuthenticatedUser,
    target_user_id: web::Path<Uuid>,
    request: web::Json<IssueBonusCreditsRequest>,
    credit_service: web::Data<CreditService>,
) -> Result<HttpResponse, AppError> {
    // Check if user is admin
    if !user.is_admin() {
        return Err(AppError::Forbidden("Admin access required".to_string()));
    }

    request.validate().map_err(validation_errors_to_app_error)?;

    info!(
        "Admin {} issuing {} bonus credits to user {}",
        user.user_id, request.amount, target_user_id
    );

    let credit = credit_service
        .issue_bonus_credits(
            *target_user_id,
            request.amount,
            request.expires_at,
            request.description.clone(),
        )
        .await?;

    Ok(HttpResponse::Created().json(credit.to_response()))
}

/// Get credit statistics (admin only)
#[actix_web::get("/admin/statistics")]
pub async fn get_credit_statistics(
    user: AuthenticatedUser,
    credit_service: web::Data<CreditService>,
) -> Result<HttpResponse, AppError> {
    // Check if user is admin
    if !user.is_admin() {
        return Err(AppError::Forbidden("Admin access required".to_string()));
    }

    debug!("Admin {} requesting credit statistics", user.user_id);

    let statistics = credit_service.get_credit_statistics().await?;

    Ok(HttpResponse::Ok().json(statistics))
}

/// Get users with expiring credits (admin only)
#[actix_web::get("/admin/expiring-users")]
pub async fn get_users_with_expiring_credits(
    user: AuthenticatedUser,
    query: web::Query<GetExpiringCreditsQuery>,
    credit_service: web::Data<CreditService>,
) -> Result<HttpResponse, AppError> {
    // Check if user is admin
    if !user.is_admin() {
        return Err(AppError::Forbidden("Admin access required".to_string()));
    }

    query.validate().map_err(validation_errors_to_app_error)?;

    let days = query.days.unwrap_or(7);
    debug!("Admin {} getting users with expiring credits (within {} days)", user.user_id, days);

    let notifications = credit_service
        .get_users_with_expiring_credits(days)
        .await?;

    Ok(HttpResponse::Ok().json(notifications))
}

/// Cleanup expired credits (admin only)
#[actix_web::post("/admin/cleanup-expired")]
pub async fn cleanup_expired_credits(
    user: AuthenticatedUser,
    credit_service: web::Data<CreditService>,
) -> Result<HttpResponse, AppError> {
    // Check if user is admin
    if !user.is_admin() {
        return Err(AppError::Forbidden("Admin access required".to_string()));
    }

    info!("Admin {} triggering expired credits cleanup", user.user_id);

    let deleted_count = credit_service.cleanup_expired_credits().await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "deleted_count": deleted_count,
        "message": format!("Cleaned up {} expired credits", deleted_count)
    })))
}

/// Get specific user's credit balance (admin only)
#[actix_web::get("/admin/user-balance/{user_id}")]
pub async fn get_user_credit_balance(
    user: AuthenticatedUser,
    target_user_id: web::Path<Uuid>,
    query: web::Query<GetCreditBalanceQuery>,
    credit_service: web::Data<CreditService>,
) -> Result<HttpResponse, AppError> {
    // Check if user is admin
    if !user.is_admin() {
        return Err(AppError::Forbidden("Admin access required".to_string()));
    }

    query.validate().map_err(validation_errors_to_app_error)?;

    debug!("Admin {} getting credit balance for user: {}", user.user_id, target_user_id);

    // Get balance
    let balance = credit_service.get_user_balance(*target_user_id).await?;

    // Get recent credit history
    let recent_credits = credit_service
        .get_user_credit_history(
            *target_user_id,
            query.include_used.unwrap_or(false),
            query.limit.or(Some(10)),
        )
        .await?;

    let response = CreditBalanceResponse {
        balance,
        recent_credits,
    };

    Ok(HttpResponse::Ok().json(response))
}

/// Health check endpoint for credit service
#[actix_web::get("/health")]
pub async fn credit_service_health(
    credit_service: web::Data<CreditService>,
) -> Result<HttpResponse, AppError> {
    // Simple health check - try to get statistics
    match credit_service.get_credit_statistics().await {
        Ok(stats) => {
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "status": "healthy",
                "total_credits_issued": stats.total_credits_issued,
                "active_users_with_credits": stats.active_users_with_credits,
                "timestamp": Utc::now()
            })))
        }
        Err(e) => {
            warn!("Credit service health check failed: {}", e);
            Ok(HttpResponse::ServiceUnavailable().json(serde_json::json!({
                "status": "unhealthy",
                "error": "Database connection failed",
                "timestamp": Utc::now()
            })))
        }
    }
}

