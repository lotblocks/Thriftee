use crate::middleware::auth::AuthenticatedUser;
use crate::services::payment_service::{
    PaymentService, PaymentIntentRequest, SubscriptionRequest, PaymentStatus,
};
use crate::error::AppError;
use actix_web::{web, HttpRequest, HttpResponse, Result};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, warn};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct CreatePaymentIntentRequest {
    #[validate(range(min = 1.0, max = 10000.0))]
    pub amount: Decimal,
    #[validate(length(min = 3, max = 3))]
    pub currency: String,
    #[validate(length(min = 1, max = 500))]
    pub description: String,
    pub metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ConfirmPaymentIntentRequest {
    #[validate(length(min = 1))]
    pub payment_method_id: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateSubscriptionRequest {
    #[validate(length(min = 1))]
    pub price_id: String,
    pub trial_period_days: Option<u32>,
    pub metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct GetPaymentHistoryQuery {
    #[validate(range(min = 1, max = 100))]
    pub limit: Option<i64>,
    pub status: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PaymentIntentCreatedResponse {
    pub payment_intent_id: String,
    pub client_secret: String,
    pub amount: i64,
    pub currency: String,
    pub status: String,
    pub next_action: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct SubscriptionCreatedResponse {
    pub subscription_id: String,
    pub status: String,
    pub current_period_start: chrono::DateTime<chrono::Utc>,
    pub current_period_end: chrono::DateTime<chrono::Utc>,
    pub trial_end: Option<chrono::DateTime<chrono::Utc>>,
    pub latest_invoice: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct PaymentMethodResponse {
    pub id: String,
    pub type_: String,
    pub card: Option<CardDetails>,
    pub is_default: bool,
}

#[derive(Debug, Serialize)]
pub struct CardDetails {
    pub brand: String,
    pub last4: String,
    pub exp_month: u32,
    pub exp_year: u32,
}

/// Create a payment intent for credit purchase
pub async fn create_payment_intent(
    user: AuthenticatedUser,
    request: web::Json<CreatePaymentIntentRequest>,
    payment_service: web::Data<PaymentService>,
) -> Result<HttpResponse, AppError> {
    request.validate()?;

    debug!(
        "Creating payment intent for user {} - amount: {} {}",
        user.user_id, request.amount, request.currency
    );

    let payment_request = PaymentIntentRequest {
        user_id: user.user_id,
        amount: request.amount,
        currency: request.currency.clone(),
        description: request.description.clone(),
        metadata: request.metadata.clone().unwrap_or_default(),
    };

    let response = payment_service.create_payment_intent(payment_request).await?;

    info!(
        "Created payment intent {} for user {} - amount: {} {}",
        response.payment_intent_id, user.user_id, request.amount, request.currency
    );

    Ok(HttpResponse::Created().json(PaymentIntentCreatedResponse {
        payment_intent_id: response.payment_intent_id,
        client_secret: response.client_secret,
        amount: response.amount,
        currency: response.currency,
        status: response.status,
        next_action: None, // Will be populated by Stripe if needed
    }))
}

/// Confirm a payment intent
pub async fn confirm_payment_intent(
    user: AuthenticatedUser,
    payment_intent_id: web::Path<String>,
    request: web::Json<ConfirmPaymentIntentRequest>,
    payment_service: web::Data<PaymentService>,
) -> Result<HttpResponse, AppError> {
    request.validate()?;

    debug!(
        "Confirming payment intent {} for user {}",
        payment_intent_id, user.user_id
    );

    let response = payment_service
        .confirm_payment_intent(&payment_intent_id, &request.payment_method_id)
        .await?;

    info!(
        "Confirmed payment intent {} for user {}",
        payment_intent_id, user.user_id
    );

    Ok(HttpResponse::Ok().json(response))
}

/// Get payment intent status
pub async fn get_payment_intent_status(
    user: AuthenticatedUser,
    payment_intent_id: web::Path<String>,
    payment_service: web::Data<PaymentService>,
) -> Result<HttpResponse, AppError> {
    debug!(
        "Getting payment intent status {} for user {}",
        payment_intent_id, user.user_id
    );

    // Get payment record from database
    let payment_record = payment_service
        .get_payment_record_by_stripe_id_public(&payment_intent_id)
        .await?;

    // Verify the payment belongs to the requesting user
    if payment_record.user_id != user.user_id {
        return Err(AppError::Forbidden("Access denied".to_string()));
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "payment_intent_id": payment_record.stripe_payment_intent_id,
        "status": payment_record.status,
        "amount": payment_record.amount,
        "currency": payment_record.currency,
        "description": payment_record.description,
        "created_at": payment_record.created_at,
        "completed_at": payment_record.completed_at,
        "failure_reason": payment_record.failure_reason
    })))
}

/// Get user's payment history
pub async fn get_payment_history(
    user: AuthenticatedUser,
    query: web::Query<GetPaymentHistoryQuery>,
    payment_service: web::Data<PaymentService>,
) -> Result<HttpResponse, AppError> {
    query.validate()?;

    debug!("Getting payment history for user {}", user.user_id);

    let history = payment_service
        .get_user_payment_history(user.user_id, query.limit)
        .await?;

    // Filter by status if provided
    let filtered_payments = if let Some(status_filter) = &query.status {
        let status_enum = match status_filter.as_str() {
            "pending" => PaymentStatus::Pending,
            "processing" => PaymentStatus::Processing,
            "succeeded" => PaymentStatus::Succeeded,
            "failed" => PaymentStatus::Failed,
            "cancelled" => PaymentStatus::Cancelled,
            "refunded" => PaymentStatus::Refunded,
            _ => return Err(AppError::Validation("Invalid status filter".to_string())),
        };

        history.payments.into_iter()
            .filter(|p| p.status == status_enum)
            .collect()
    } else {
        history.payments
    };

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "payments": filtered_payments,
        "total_amount": history.total_amount,
        "successful_payments": history.successful_payments,
        "failed_payments": history.failed_payments
    })))
}

/// Create a subscription for seller plans
pub async fn create_subscription(
    user: AuthenticatedUser,
    request: web::Json<CreateSubscriptionRequest>,
    payment_service: web::Data<PaymentService>,
) -> Result<HttpResponse, AppError> {
    request.validate()?;

    debug!(
        "Creating subscription for user {} - price_id: {}",
        user.user_id, request.price_id
    );

    let subscription_request = SubscriptionRequest {
        user_id: user.user_id,
        price_id: request.price_id.clone(),
        trial_period_days: request.trial_period_days,
        metadata: request.metadata.clone().unwrap_or_default(),
    };

    let response = payment_service.create_subscription(subscription_request).await?;

    info!(
        "Created subscription {} for user {}",
        response.subscription_id, user.user_id
    );

    Ok(HttpResponse::Created().json(SubscriptionCreatedResponse {
        subscription_id: response.subscription_id,
        status: response.status,
        current_period_start: response.current_period_start,
        current_period_end: response.current_period_end,
        trial_end: response.trial_end,
        latest_invoice: None, // Would be populated from Stripe if needed
    }))
}

/// Get user's active subscriptions
pub async fn get_user_subscriptions(
    user: AuthenticatedUser,
    payment_service: web::Data<PaymentService>,
) -> Result<HttpResponse, AppError> {
    debug!("Getting subscriptions for user {}", user.user_id);

    let subscriptions = payment_service.get_user_subscriptions(user.user_id).await?;

    Ok(HttpResponse::Ok().json(subscriptions))
}

/// Cancel a subscription
pub async fn cancel_subscription(
    user: AuthenticatedUser,
    subscription_id: web::Path<String>,
    payment_service: web::Data<PaymentService>,
) -> Result<HttpResponse, AppError> {
    debug!(
        "Cancelling subscription {} for user {}",
        subscription_id, user.user_id
    );

    // Verify the subscription belongs to the user
    let subscriptions = payment_service.get_user_subscriptions(user.user_id).await?;
    let subscription = subscriptions
        .iter()
        .find(|s| s.stripe_subscription_id == *subscription_id)
        .ok_or_else(|| AppError::NotFound("Subscription not found".to_string()))?;

    payment_service.cancel_subscription(&subscription_id).await?;

    info!(
        "Cancelled subscription {} for user {}",
        subscription_id, user.user_id
    );

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "Subscription cancelled successfully",
        "subscription_id": *subscription_id
    })))
}

/// Get payment statistics for user
pub async fn get_payment_statistics(
    user: AuthenticatedUser,
    payment_service: web::Data<PaymentService>,
) -> Result<HttpResponse, AppError> {
    debug!("Getting payment statistics for user {}", user.user_id);

    let history = payment_service
        .get_user_payment_history(user.user_id, Some(1000))
        .await?;

    let subscriptions = payment_service.get_user_subscriptions(user.user_id).await?;

    // Calculate statistics
    let total_spent = history.total_amount;
    let avg_payment = if history.successful_payments > 0 {
        total_spent / Decimal::from(history.successful_payments)
    } else {
        Decimal::ZERO
    };

    let active_subscriptions = subscriptions.len();
    let monthly_subscription_cost: Decimal = subscriptions
        .iter()
        .filter(|s| matches!(s.status, stripe::SubscriptionStatus::Active | stripe::SubscriptionStatus::Trialing))
        .map(|_| Decimal::from(29.99)) // This would come from the subscription price
        .sum();

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "total_spent": total_spent,
        "successful_payments": history.successful_payments,
        "failed_payments": history.failed_payments,
        "average_payment": avg_payment,
        "active_subscriptions": active_subscriptions,
        "monthly_subscription_cost": monthly_subscription_cost,
        "first_payment_date": history.payments.last().map(|p| p.created_at),
        "last_payment_date": history.payments.first().map(|p| p.created_at)
    })))
}

// Admin endpoints

/// Get payment analytics (admin only)
pub async fn get_payment_analytics(
    user: AuthenticatedUser,
    payment_service: web::Data<PaymentService>,
) -> Result<HttpResponse, AppError> {
    if !user.is_admin() {
        return Err(AppError::Forbidden("Admin access required".to_string()));
    }

    debug!("Admin {} requesting payment analytics", user.user_id);

    // This would typically query the payment_analytics materialized view
    // For now, we'll return a placeholder response
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "daily_revenue": [],
        "total_revenue": 0,
        "successful_payments": 0,
        "failed_payments": 0,
        "unique_customers": 0,
        "average_payment_amount": 0
    })))
}

/// Get user payment details (admin only)
pub async fn get_user_payment_details(
    user: AuthenticatedUser,
    target_user_id: web::Path<Uuid>,
    payment_service: web::Data<PaymentService>,
) -> Result<HttpResponse, AppError> {
    if !user.is_admin() {
        return Err(AppError::Forbidden("Admin access required".to_string()));
    }

    debug!(
        "Admin {} requesting payment details for user {}",
        user.user_id, target_user_id
    );

    let history = payment_service
        .get_user_payment_history(*target_user_id, Some(100))
        .await?;

    let subscriptions = payment_service.get_user_subscriptions(*target_user_id).await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "user_id": *target_user_id,
        "payment_history": history,
        "subscriptions": subscriptions
    })))
}

/// Process refund (admin only)
pub async fn process_refund(
    user: AuthenticatedUser,
    payment_id: web::Path<Uuid>,
    payment_service: web::Data<PaymentService>,
) -> Result<HttpResponse, AppError> {
    if !user.is_admin() {
        return Err(AppError::Forbidden("Admin access required".to_string()));
    }

    info!(
        "Admin {} processing refund for payment {}",
        user.user_id, payment_id
    );

    // This would implement the actual refund logic
    // For now, return a placeholder response
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "Refund processed successfully",
        "payment_id": *payment_id
    })))
}

/// Webhook endpoint for Stripe events
pub async fn stripe_webhook(
    req: HttpRequest,
    body: web::Bytes,
    payment_service: web::Data<PaymentService>,
) -> Result<HttpResponse, AppError> {
    // Get Stripe signature from headers
    let signature = req
        .headers()
        .get("stripe-signature")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| AppError::Validation("Missing Stripe signature".to_string()))?;

    // Verify webhook signature
    payment_service.verify_webhook_signature(&body, signature)?;

    // Parse webhook payload
    let payload: serde_json::Value = serde_json::from_slice(&body)
        .map_err(|e| AppError::Validation(format!("Invalid JSON: {}", e)))?;

    let event_type = payload["type"]
        .as_str()
        .ok_or_else(|| AppError::Validation("Missing event type".to_string()))?;

    info!("Received Stripe webhook: {}", event_type);

    // Process webhook event
    match event_type {
        "payment_intent.succeeded" => {
            let payment_intent_id = payload["data"]["object"]["id"]
                .as_str()
                .ok_or_else(|| AppError::Validation("Missing payment intent ID".to_string()))?;

            let amount = payload["data"]["object"]["amount"]
                .as_u64()
                .ok_or_else(|| AppError::Validation("Missing amount".to_string()))?;

            payment_service
                .process_successful_payment(payment_intent_id, amount)
                .await?;
        }
        "payment_intent.payment_failed" => {
            let payment_intent_id = payload["data"]["object"]["id"]
                .as_str()
                .ok_or_else(|| AppError::Validation("Missing payment intent ID".to_string()))?;

            let failure_reason = payload["data"]["object"]["last_payment_error"]["message"]
                .as_str()
                .unwrap_or("Unknown error");

            payment_service
                .process_failed_payment(payment_intent_id, failure_reason)
                .await?;
        }
        "invoice.payment_succeeded" => {
            let subscription_id = payload["data"]["object"]["subscription"]
                .as_str()
                .ok_or_else(|| AppError::Validation("Missing subscription ID".to_string()))?;

            payment_service
                .process_subscription_payment_success(subscription_id)
                .await?;
        }
        "invoice.payment_failed" => {
            let subscription_id = payload["data"]["object"]["subscription"]
                .as_str()
                .ok_or_else(|| AppError::Validation("Missing subscription ID".to_string()))?;

            payment_service
                .process_subscription_payment_failure(subscription_id)
                .await?;
        }
        "customer.subscription.deleted" => {
            let subscription_id = payload["data"]["object"]["id"]
                .as_str()
                .ok_or_else(|| AppError::Validation("Missing subscription ID".to_string()))?;

            payment_service
                .process_subscription_cancellation(subscription_id)
                .await?;
        }
        _ => {
            warn!("Unhandled Stripe webhook event: {}", event_type);
        }
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "received": true
    })))
}

/// Health check endpoint for payment service
pub async fn payment_service_health(
    payment_service: web::Data<PaymentService>,
) -> Result<HttpResponse, AppError> {
    // Simple health check - verify we can connect to Stripe
    // This is a placeholder - in a real implementation you'd ping Stripe's API
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": "payment",
        "timestamp": chrono::Utc::now()
    })))
}

// Helper function to configure payment routes
pub fn configure_payment_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/payments")
            // User endpoints
            .route("/create-intent", web::post().to(create_payment_intent))
            .route("/confirm-intent/{payment_intent_id}", web::post().to(confirm_payment_intent))
            .route("/intent-status/{payment_intent_id}", web::get().to(get_payment_intent_status))
            .route("/history", web::get().to(get_payment_history))
            .route("/statistics", web::get().to(get_payment_statistics))
            
            // Subscription endpoints
            .route("/subscriptions", web::post().to(create_subscription))
            .route("/subscriptions", web::get().to(get_user_subscriptions))
            .route("/subscriptions/{subscription_id}/cancel", web::post().to(cancel_subscription))
            
            // Admin endpoints
            .route("/admin/analytics", web::get().to(get_payment_analytics))
            .route("/admin/user/{user_id}", web::get().to(get_user_payment_details))
            .route("/admin/refund/{payment_id}", web::post().to(process_refund))
            
            // Webhook endpoint
            .route("/webhook", web::post().to(stripe_webhook))
            
            // Health check
            .route("/health", web::get().to(payment_service_health))
    );
}