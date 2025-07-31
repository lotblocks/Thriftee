use actix_web::{post, web, HttpRequest, HttpResponse, Result};
use serde_json::Value;
use tracing::{error, info, warn};

use crate::error::AppError;
use crate::services::payment_service::PaymentService;
use crate::services::blockchain_service::BlockchainService;
use crate::utils::webhook_verification;

/// Stripe webhook handler
/// Handles payment events from Stripe including successful payments,
/// failed payments, and subscription changes
#[post("/webhooks/stripe")]
pub async fn stripe_webhook(
    req: HttpRequest,
    body: web::Bytes,
    payment_service: web::Data<PaymentService>,
) -> Result<HttpResponse, AppError> {
    // Verify webhook signature
    let signature = req
        .headers()
        .get("stripe-signature")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| AppError::Validation("Missing Stripe signature".to_string()))?;

    webhook_verification::verify_stripe_signature(&body, signature)?;

    // Parse webhook payload
    let payload: Value = serde_json::from_slice(&body)
        .map_err(|e| AppError::Validation(format!("Invalid JSON: {}", e)))?;

    let event_type = payload["type"]
        .as_str()
        .ok_or_else(|| AppError::Validation("Missing event type".to_string()))?;

    info!("Received Stripe webhook: {}", event_type);

    match event_type {
        "payment_intent.succeeded" => {
            handle_payment_success(&payment_service, &payload).await?;
        }
        "payment_intent.payment_failed" => {
            handle_payment_failure(&payment_service, &payload).await?;
        }
        "invoice.payment_succeeded" => {
            handle_subscription_payment_success(&payment_service, &payload).await?;
        }
        "invoice.payment_failed" => {
            handle_subscription_payment_failure(&payment_service, &payload).await?;
        }
        "customer.subscription.deleted" => {
            handle_subscription_cancelled(&payment_service, &payload).await?;
        }
        _ => {
            warn!("Unhandled Stripe webhook event: {}", event_type);
        }
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "success",
        "message": "Webhook processed successfully"
    })))
}

/// Blockchain webhook handler
/// Handles events from the smart contract including box purchases,
/// raffle completions, and winner selections
#[post("/webhooks/blockchain")]
pub async fn blockchain_webhook(
    req: HttpRequest,
    body: web::Bytes,
    blockchain_service: web::Data<BlockchainService>,
) -> Result<HttpResponse, AppError> {
    // Verify webhook signature (if using a service like Alchemy)
    if let Some(signature) = req.headers().get("x-alchemy-signature") {
        let signature_str = signature.to_str()
            .map_err(|_| AppError::Validation("Invalid signature header".to_string()))?;
        webhook_verification::verify_blockchain_signature(&body, signature_str)?;
    }

    // Parse webhook payload
    let payload: Value = serde_json::from_slice(&body)
        .map_err(|e| AppError::Validation(format!("Invalid JSON: {}", e)))?;

    let event_type = payload["type"]
        .as_str()
        .unwrap_or("unknown");

    info!("Received blockchain webhook: {}", event_type);

    // Process blockchain events
    match event_type {
        "ADDRESS_ACTIVITY" => {
            handle_address_activity(&blockchain_service, &payload).await?;
        }
        "MINED_TRANSACTION" => {
            handle_mined_transaction(&blockchain_service, &payload).await?;
        }
        _ => {
            // Handle direct contract events
            if let Some(logs) = payload["logs"].as_array() {
                for log in logs {
                    handle_contract_event(&blockchain_service, log).await?;
                }
            }
        }
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "success",
        "message": "Blockchain webhook processed successfully"
    })))
}

/// Email/SMS notification webhook handler
/// Handles delivery status updates from notification services
#[post("/webhooks/notifications")]
pub async fn notification_webhook(
    req: HttpRequest,
    body: web::Bytes,
) -> Result<HttpResponse, AppError> {
    // Parse webhook payload
    let payload: Value = serde_json::from_slice(&body)
        .map_err(|e| AppError::Validation(format!("Invalid JSON: {}", e)))?;

    let event_type = payload["event"]
        .as_str()
        .ok_or_else(|| AppError::Validation("Missing event type".to_string()))?;

    info!("Received notification webhook: {}", event_type);

    match event_type {
        "delivered" => {
            handle_notification_delivered(&payload).await?;
        }
        "failed" => {
            handle_notification_failed(&payload).await?;
        }
        "bounced" => {
            handle_notification_bounced(&payload).await?;
        }
        _ => {
            warn!("Unhandled notification webhook event: {}", event_type);
        }
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "success",
        "message": "Notification webhook processed successfully"
    })))
}

// Helper functions for handling specific webhook events

async fn handle_payment_success(
    payment_service: &PaymentService,
    payload: &Value,
) -> Result<(), AppError> {
    let payment_intent_id = payload["data"]["object"]["id"]
        .as_str()
        .ok_or_else(|| AppError::Validation("Missing payment intent ID".to_string()))?;

    let amount = payload["data"]["object"]["amount"]
        .as_u64()
        .ok_or_else(|| AppError::Validation("Missing amount".to_string()))?;

    info!("Processing successful payment: {} for amount: {}", payment_intent_id, amount);

    payment_service.process_successful_payment(payment_intent_id, amount).await?;
    Ok(())
}

async fn handle_payment_failure(
    payment_service: &PaymentService,
    payload: &Value,
) -> Result<(), AppError> {
    let payment_intent_id = payload["data"]["object"]["id"]
        .as_str()
        .ok_or_else(|| AppError::Validation("Missing payment intent ID".to_string()))?;

    let failure_reason = payload["data"]["object"]["last_payment_error"]["message"]
        .as_str()
        .unwrap_or("Unknown error");

    error!("Payment failed: {} - {}", payment_intent_id, failure_reason);

    payment_service.process_failed_payment(payment_intent_id, failure_reason).await?;
    Ok(())
}

async fn handle_subscription_payment_success(
    payment_service: &PaymentService,
    payload: &Value,
) -> Result<(), AppError> {
    let subscription_id = payload["data"]["object"]["subscription"]
        .as_str()
        .ok_or_else(|| AppError::Validation("Missing subscription ID".to_string()))?;

    info!("Processing successful subscription payment: {}", subscription_id);

    payment_service.process_subscription_payment_success(subscription_id).await?;
    Ok(())
}

async fn handle_subscription_payment_failure(
    payment_service: &PaymentService,
    payload: &Value,
) -> Result<(), AppError> {
    let subscription_id = payload["data"]["object"]["subscription"]
        .as_str()
        .ok_or_else(|| AppError::Validation("Missing subscription ID".to_string()))?;

    error!("Subscription payment failed: {}", subscription_id);

    payment_service.process_subscription_payment_failure(subscription_id).await?;
    Ok(())
}

async fn handle_subscription_cancelled(
    payment_service: &PaymentService,
    payload: &Value,
) -> Result<(), AppError> {
    let subscription_id = payload["data"]["object"]["id"]
        .as_str()
        .ok_or_else(|| AppError::Validation("Missing subscription ID".to_string()))?;

    info!("Processing subscription cancellation: {}", subscription_id);

    payment_service.process_subscription_cancellation(subscription_id).await?;
    Ok(())
}

async fn handle_address_activity(
    blockchain_service: &BlockchainService,
    payload: &Value,
) -> Result<(), AppError> {
    let activity = &payload["activity"];
    if let Some(activities) = activity.as_array() {
        for activity in activities {
            blockchain_service.process_address_activity(activity).await?;
        }
    }
    Ok(())
}

async fn handle_mined_transaction(
    blockchain_service: &BlockchainService,
    payload: &Value,
) -> Result<(), AppError> {
    let tx_hash = payload["data"]["transaction"]["hash"]
        .as_str()
        .ok_or_else(|| AppError::Validation("Missing transaction hash".to_string()))?;

    info!("Processing mined transaction: {}", tx_hash);

    blockchain_service.process_mined_transaction(tx_hash).await?;
    Ok(())
}

async fn handle_contract_event(
    blockchain_service: &BlockchainService,
    log: &Value,
) -> Result<(), AppError> {
    let topics = log["topics"]
        .as_array()
        .ok_or_else(|| AppError::Validation("Missing topics".to_string()))?;

    if let Some(event_signature) = topics.get(0).and_then(|t| t.as_str()) {
        match event_signature {
            // BoxPurchased event signature
            "0x..." => {
                blockchain_service.handle_box_purchased_event(log).await?;
            }
            // WinnerSelected event signature
            "0x..." => {
                blockchain_service.handle_winner_selected_event(log).await?;
            }
            // RaffleFull event signature
            "0x..." => {
                blockchain_service.handle_raffle_full_event(log).await?;
            }
            _ => {
                warn!("Unhandled contract event: {}", event_signature);
            }
        }
    }

    Ok(())
}

async fn handle_notification_delivered(payload: &Value) -> Result<(), AppError> {
    let message_id = payload["message_id"]
        .as_str()
        .ok_or_else(|| AppError::Validation("Missing message ID".to_string()))?;

    info!("Notification delivered: {}", message_id);

    // Update notification status in database
    // TODO: Implement notification status tracking

    Ok(())
}

async fn handle_notification_failed(payload: &Value) -> Result<(), AppError> {
    let message_id = payload["message_id"]
        .as_str()
        .ok_or_else(|| AppError::Validation("Missing message ID".to_string()))?;

    let reason = payload["reason"]
        .as_str()
        .unwrap_or("Unknown reason");

    error!("Notification failed: {} - {}", message_id, reason);

    // Update notification status and potentially retry
    // TODO: Implement notification retry logic

    Ok(())
}

async fn handle_notification_bounced(payload: &Value) -> Result<(), AppError> {
    let message_id = payload["message_id"]
        .as_str()
        .ok_or_else(|| AppError::Validation("Missing message ID".to_string()))?;

    warn!("Notification bounced: {}", message_id);

    // Mark email/phone as invalid
    // TODO: Implement bounce handling

    Ok(())
}