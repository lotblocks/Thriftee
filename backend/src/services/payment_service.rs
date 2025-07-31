use crate::error::AppError;
use crate::models::user::User;
use crate::services::credit_service::{CreditService, CreditIssuanceRequest};
use chrono::{DateTime, Utc};
use raffle_platform_shared::{CreditSource, CreditType};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use stripe::{
    Client, CreateCustomer, CreatePaymentIntent, CreatePrice, CreateProduct, CreateSubscription,
    Currency, Customer, PaymentIntent, PaymentIntentConfirmParams, Price, Product, Subscription,
    SubscriptionStatus, UpdateSubscription,
};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

#[cfg(test)]
mod tests;

/// Payment service handles all payment processing through Stripe
#[derive(Clone)]
pub struct PaymentService {
    stripe_client: Client,
    db_pool: PgPool,
    credit_service: CreditService,
    webhook_secret: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentIntentRequest {
    pub user_id: Uuid,
    pub amount: Decimal, // Amount in dollars
    pub currency: String,
    pub description: String,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentIntentResponse {
    pub payment_intent_id: String,
    pub client_secret: String,
    pub amount: i64, // Amount in cents
    pub currency: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionRequest {
    pub user_id: Uuid,
    pub price_id: String,
    pub trial_period_days: Option<u32>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionResponse {
    pub subscription_id: String,
    pub status: String,
    pub current_period_start: DateTime<Utc>,
    pub current_period_end: DateTime<Utc>,
    pub trial_end: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub stripe_payment_intent_id: String,
    pub amount: Decimal,
    pub currency: String,
    pub status: PaymentStatus,
    pub description: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub failure_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "payment_status", rename_all = "lowercase")]
pub enum PaymentStatus {
    Pending,
    Processing,
    Succeeded,
    Failed,
    Cancelled,
    Refunded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub stripe_subscription_id: String,
    pub stripe_customer_id: String,
    pub status: SubscriptionStatus,
    pub current_period_start: DateTime<Utc>,
    pub current_period_end: DateTime<Utc>,
    pub trial_end: Option<DateTime<Utc>>,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentHistory {
    pub payments: Vec<PaymentRecord>,
    pub total_amount: Decimal,
    pub successful_payments: usize,
    pub failed_payments: usize,
}

impl PaymentService {
    /// Create a new payment service
    pub fn new(
        stripe_secret_key: String,
        webhook_secret: String,
        db_pool: PgPool,
        credit_service: CreditService,
    ) -> Self {
        let stripe_client = Client::new(stripe_secret_key);

        Self {
            stripe_client,
            db_pool,
            credit_service,
            webhook_secret,
        }
    }

    /// Create a payment intent for credit purchase
    pub async fn create_payment_intent(
        &self,
        request: PaymentIntentRequest,
    ) -> Result<PaymentIntentResponse, AppError> {
        // Validate user exists
        let user = User::find_by_id(&self.db_pool, request.user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        // Convert amount to cents
        let amount_cents = (request.amount * Decimal::from(100)).to_i64().unwrap_or(0);

        if amount_cents <= 0 {
            return Err(AppError::Validation("Invalid amount".to_string()));
        }

        // Get or create Stripe customer
        let customer_id = self.get_or_create_customer(&user).await?;

        // Create payment intent
        let mut create_params = CreatePaymentIntent::new(amount_cents, Currency::USD);
        create_params.customer = Some(customer_id);
        create_params.description = Some(&request.description);
        create_params.automatic_payment_methods = Some(
            stripe::CreatePaymentIntentAutomaticPaymentMethods {
                enabled: true,
                allow_redirects: Some(stripe::CreatePaymentIntentAutomaticPaymentMethodsAllowRedirects::Never),
            }
        );

        // Add metadata
        let mut metadata = request.metadata.clone();
        metadata.insert("user_id".to_string(), request.user_id.to_string());
        metadata.insert("purpose".to_string(), "credit_purchase".to_string());
        create_params.metadata = Some(metadata.clone());

        let payment_intent = PaymentIntent::create(&self.stripe_client, create_params)
            .await
            .map_err(|e| AppError::External(format!("Stripe error: {}", e)))?;

        // Save payment record to database
        let payment_record = self
            .save_payment_record(
                request.user_id,
                &payment_intent.id,
                request.amount,
                &request.currency,
                PaymentStatus::Pending,
                &request.description,
                serde_json::to_value(metadata)?,
                None,
            )
            .await?;

        info!(
            "Created payment intent {} for user {} (amount: {})",
            payment_intent.id, request.user_id, request.amount
        );

        Ok(PaymentIntentResponse {
            payment_intent_id: payment_intent.id,
            client_secret: payment_intent.client_secret.unwrap_or_default(),
            amount: amount_cents,
            currency: request.currency,
            status: payment_intent.status.to_string(),
        })
    }

    /// Confirm a payment intent
    pub async fn confirm_payment_intent(
        &self,
        payment_intent_id: &str,
        payment_method_id: &str,
    ) -> Result<PaymentIntentResponse, AppError> {
        let mut confirm_params = PaymentIntentConfirmParams::new();
        confirm_params.payment_method = Some(payment_method_id.to_string());

        let payment_intent = PaymentIntent::confirm(
            &self.stripe_client,
            payment_intent_id,
            confirm_params,
        )
        .await
        .map_err(|e| AppError::External(format!("Stripe error: {}", e)))?;

        // Update payment record status
        self.update_payment_status(
            &payment_intent.id,
            PaymentStatus::Processing,
            None,
        )
        .await?;

        Ok(PaymentIntentResponse {
            payment_intent_id: payment_intent.id,
            client_secret: payment_intent.client_secret.unwrap_or_default(),
            amount: payment_intent.amount,
            currency: payment_intent.currency.to_string(),
            status: payment_intent.status.to_string(),
        })
    }

    /// Process successful payment (called from webhook)
    pub async fn process_successful_payment(
        &self,
        payment_intent_id: &str,
        amount_cents: u64,
    ) -> Result<(), AppError> {
        // Update payment record
        self.update_payment_status(
            payment_intent_id,
            PaymentStatus::Succeeded,
            Some(Utc::now()),
        )
        .await?;

        // Get payment record to find user
        let payment_record = self.get_payment_record_by_stripe_id(payment_intent_id).await?;

        // Convert cents back to dollars
        let amount_dollars = Decimal::from(amount_cents) / Decimal::from(100);

        // Issue credits to user
        let credit_request = CreditIssuanceRequest {
            user_id: payment_record.user_id,
            amount: amount_dollars,
            source: CreditSource::Deposit,
            credit_type: CreditType::General,
            redeemable_on_item_id: None,
            expires_at: None, // Purchased credits don't expire
            description: format!("Credit purchase via payment {}", payment_intent_id),
        };

        self.credit_service.issue_credits(credit_request).await?;

        info!(
            "Processed successful payment {} - issued {} credits to user {}",
            payment_intent_id, amount_dollars, payment_record.user_id
        );

        Ok(())
    }

    /// Process failed payment (called from webhook)
    pub async fn process_failed_payment(
        &self,
        payment_intent_id: &str,
        failure_reason: &str,
    ) -> Result<(), AppError> {
        self.update_payment_status_with_reason(
            payment_intent_id,
            PaymentStatus::Failed,
            None,
            Some(failure_reason.to_string()),
        )
        .await?;

        warn!(
            "Processed failed payment {} - reason: {}",
            payment_intent_id, failure_reason
        );

        Ok(())
    }

    /// Create a subscription for a seller
    pub async fn create_subscription(
        &self,
        request: SubscriptionRequest,
    ) -> Result<SubscriptionResponse, AppError> {
        // Validate user exists
        let user = User::find_by_id(&self.db_pool, request.user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        // Get or create Stripe customer
        let customer_id = self.get_or_create_customer(&user).await?;

        // Create subscription
        let mut create_params = CreateSubscription::new(customer_id);
        create_params.items = Some(vec![stripe::CreateSubscriptionItems {
            price: Some(request.price_id.clone()),
            quantity: Some(1),
            ..Default::default()
        }]);

        if let Some(trial_days) = request.trial_period_days {
            create_params.trial_period_days = Some(trial_days);
        }

        // Add metadata
        let mut metadata = request.metadata.clone();
        metadata.insert("user_id".to_string(), request.user_id.to_string());
        create_params.metadata = Some(metadata);

        let subscription = Subscription::create(&self.stripe_client, create_params)
            .await
            .map_err(|e| AppError::External(format!("Stripe error: {}", e)))?;

        // Save subscription record
        self.save_subscription_record(
            request.user_id,
            &subscription.id,
            &customer_id,
            subscription.status,
            DateTime::from_timestamp(subscription.current_period_start, 0).unwrap_or_else(Utc::now),
            DateTime::from_timestamp(subscription.current_period_end, 0).unwrap_or_else(Utc::now),
            subscription.trial_end.map(|ts| DateTime::from_timestamp(ts, 0).unwrap_or_else(Utc::now)),
        )
        .await?;

        info!(
            "Created subscription {} for user {}",
            subscription.id, request.user_id
        );

        Ok(SubscriptionResponse {
            subscription_id: subscription.id,
            status: subscription.status.to_string(),
            current_period_start: DateTime::from_timestamp(subscription.current_period_start, 0)
                .unwrap_or_else(Utc::now),
            current_period_end: DateTime::from_timestamp(subscription.current_period_end, 0)
                .unwrap_or_else(Utc::now),
            trial_end: subscription.trial_end
                .map(|ts| DateTime::from_timestamp(ts, 0).unwrap_or_else(Utc::now)),
        })
    }

    /// Cancel a subscription
    pub async fn cancel_subscription(&self, subscription_id: &str) -> Result<(), AppError> {
        // Cancel in Stripe
        let mut update_params = UpdateSubscription::new();
        update_params.cancel_at_period_end = Some(true);

        Subscription::update(&self.stripe_client, subscription_id, update_params)
            .await
            .map_err(|e| AppError::External(format!("Stripe error: {}", e)))?;

        // Update local record
        self.update_subscription_status(subscription_id, SubscriptionStatus::Active, Some(Utc::now()))
            .await?;

        info!("Cancelled subscription: {}", subscription_id);
        Ok(())
    }

    /// Process subscription payment success (called from webhook)
    pub async fn process_subscription_payment_success(
        &self,
        subscription_id: &str,
    ) -> Result<(), AppError> {
        // Update subscription status to active
        self.update_subscription_status(subscription_id, SubscriptionStatus::Active, None)
            .await?;

        info!("Processed successful subscription payment: {}", subscription_id);
        Ok(())
    }

    /// Process subscription payment failure (called from webhook)
    pub async fn process_subscription_payment_failure(
        &self,
        subscription_id: &str,
    ) -> Result<(), AppError> {
        // Update subscription status to past due
        self.update_subscription_status(subscription_id, SubscriptionStatus::PastDue, None)
            .await?;

        warn!("Processed failed subscription payment: {}", subscription_id);
        Ok(())
    }

    /// Process subscription cancellation (called from webhook)
    pub async fn process_subscription_cancellation(
        &self,
        subscription_id: &str,
    ) -> Result<(), AppError> {
        // Update subscription status to cancelled
        self.update_subscription_status(subscription_id, SubscriptionStatus::Canceled, Some(Utc::now()))
            .await?;

        info!("Processed subscription cancellation: {}", subscription_id);
        Ok(())
    }

    /// Get user's payment history
    pub async fn get_user_payment_history(
        &self,
        user_id: Uuid,
        limit: Option<i64>,
    ) -> Result<PaymentHistory, AppError> {
        let payments = sqlx::query_as!(
            PaymentRecord,
            r#"
            SELECT 
                id, user_id, stripe_payment_intent_id, amount, currency,
                status as "status: PaymentStatus",
                description, metadata, created_at, updated_at, completed_at, failure_reason
            FROM payments 
            WHERE user_id = $1 
            ORDER BY created_at DESC 
            LIMIT $2
            "#,
            user_id,
            limit.unwrap_or(50)
        )
        .fetch_all(&self.db_pool)
        .await?;

        let total_amount: Decimal = payments
            .iter()
            .filter(|p| p.status == PaymentStatus::Succeeded)
            .map(|p| p.amount)
            .sum();

        let successful_payments = payments
            .iter()
            .filter(|p| p.status == PaymentStatus::Succeeded)
            .count();

        let failed_payments = payments
            .iter()
            .filter(|p| p.status == PaymentStatus::Failed)
            .count();

        Ok(PaymentHistory {
            payments,
            total_amount,
            successful_payments,
            failed_payments,
        })
    }

    /// Get user's active subscriptions
    pub async fn get_user_subscriptions(&self, user_id: Uuid) -> Result<Vec<SubscriptionRecord>, AppError> {
        let subscriptions = sqlx::query_as!(
            SubscriptionRecord,
            r#"
            SELECT 
                id, user_id, stripe_subscription_id, stripe_customer_id,
                status as "status: SubscriptionStatus",
                current_period_start, current_period_end, trial_end, cancelled_at, created_at, updated_at
            FROM subscriptions 
            WHERE user_id = $1 
            AND status IN ('active', 'trialing', 'past_due')
            ORDER BY created_at DESC
            "#,
            user_id
        )
        .fetch_all(&self.db_pool)
        .await?;

        Ok(subscriptions)
    }

    /// Verify webhook signature
    pub fn verify_webhook_signature(&self, payload: &[u8], signature: &str) -> Result<(), AppError> {
        stripe::Webhook::construct_event(payload, signature, &self.webhook_secret)
            .map_err(|e| AppError::Validation(format!("Invalid webhook signature: {}", e)))?;
        Ok(())
    }

    // Private helper methods

    async fn get_or_create_customer(&self, user: &User) -> Result<String, AppError> {
        // Check if customer already exists in database
        if let Some(customer_id) = self.get_customer_id_for_user(user.id).await? {
            return Ok(customer_id);
        }

        // Create new customer in Stripe
        let mut create_params = CreateCustomer::new();
        create_params.email = Some(&user.email);
        create_params.metadata = Some({
            let mut metadata = HashMap::new();
            metadata.insert("user_id".to_string(), user.id.to_string());
            metadata
        });

        let customer = Customer::create(&self.stripe_client, create_params)
            .await
            .map_err(|e| AppError::External(format!("Stripe error: {}", e)))?;

        // Save customer ID to database
        self.save_customer_id(user.id, &customer.id).await?;

        Ok(customer.id)
    }

    async fn get_customer_id_for_user(&self, user_id: Uuid) -> Result<Option<String>, AppError> {
        let result = sqlx::query_scalar!(
            "SELECT stripe_customer_id FROM user_stripe_customers WHERE user_id = $1",
            user_id
        )
        .fetch_optional(&self.db_pool)
        .await?;

        Ok(result)
    }

    async fn save_customer_id(&self, user_id: Uuid, customer_id: &str) -> Result<(), AppError> {
        sqlx::query!(
            "INSERT INTO user_stripe_customers (user_id, stripe_customer_id) VALUES ($1, $2)
             ON CONFLICT (user_id) DO UPDATE SET stripe_customer_id = $2",
            user_id,
            customer_id
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    async fn save_payment_record(
        &self,
        user_id: Uuid,
        stripe_payment_intent_id: &str,
        amount: Decimal,
        currency: &str,
        status: PaymentStatus,
        description: &str,
        metadata: serde_json::Value,
        completed_at: Option<DateTime<Utc>>,
    ) -> Result<PaymentRecord, AppError> {
        let record = sqlx::query_as!(
            PaymentRecord,
            r#"
            INSERT INTO payments (user_id, stripe_payment_intent_id, amount, currency, status, description, metadata, completed_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING 
                id, user_id, stripe_payment_intent_id, amount, currency,
                status as "status: PaymentStatus",
                description, metadata, created_at, updated_at, completed_at, failure_reason
            "#,
            user_id,
            stripe_payment_intent_id,
            amount,
            currency,
            status as PaymentStatus,
            description,
            metadata,
            completed_at
        )
        .fetch_one(&self.db_pool)
        .await?;

        Ok(record)
    }

    async fn update_payment_status(
        &self,
        stripe_payment_intent_id: &str,
        status: PaymentStatus,
        completed_at: Option<DateTime<Utc>>,
    ) -> Result<(), AppError> {
        sqlx::query!(
            "UPDATE payments SET status = $1, completed_at = $2, updated_at = NOW() WHERE stripe_payment_intent_id = $3",
            status as PaymentStatus,
            completed_at,
            stripe_payment_intent_id
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    async fn update_payment_status_with_reason(
        &self,
        stripe_payment_intent_id: &str,
        status: PaymentStatus,
        completed_at: Option<DateTime<Utc>>,
        failure_reason: Option<String>,
    ) -> Result<(), AppError> {
        sqlx::query!(
            "UPDATE payments SET status = $1, completed_at = $2, failure_reason = $3, updated_at = NOW() WHERE stripe_payment_intent_id = $4",
            status as PaymentStatus,
            completed_at,
            failure_reason,
            stripe_payment_intent_id
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    async fn get_payment_record_by_stripe_id(
        &self,
        stripe_payment_intent_id: &str,
    ) -> Result<PaymentRecord, AppError> {
        let record = sqlx::query_as!(
            PaymentRecord,
            r#"
            SELECT 
                id, user_id, stripe_payment_intent_id, amount, currency,
                status as "status: PaymentStatus",
                description, metadata, created_at, updated_at, completed_at, failure_reason
            FROM payments 
            WHERE stripe_payment_intent_id = $1
            "#,
            stripe_payment_intent_id
        )
        .fetch_one(&self.db_pool)
        .await?;

        Ok(record)
    }

    async fn save_subscription_record(
        &self,
        user_id: Uuid,
        stripe_subscription_id: &str,
        stripe_customer_id: &str,
        status: SubscriptionStatus,
        current_period_start: DateTime<Utc>,
        current_period_end: DateTime<Utc>,
        trial_end: Option<DateTime<Utc>>,
    ) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            INSERT INTO subscriptions (user_id, stripe_subscription_id, stripe_customer_id, status, current_period_start, current_period_end, trial_end)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
            user_id,
            stripe_subscription_id,
            stripe_customer_id,
            status as SubscriptionStatus,
            current_period_start,
            current_period_end,
            trial_end
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    async fn update_subscription_status(
        &self,
        stripe_subscription_id: &str,
        status: SubscriptionStatus,
        cancelled_at: Option<DateTime<Utc>>,
    ) -> Result<(), AppError> {
        sqlx::query!(
            "UPDATE subscriptions SET status = $1, cancelled_at = $2, updated_at = NOW() WHERE stripe_subscription_id = $3",
            status as SubscriptionStatus,
            cancelled_at,
            stripe_subscription_id
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    /// Get payment record by Stripe payment intent ID (public method for handlers)
    pub async fn get_payment_record_by_stripe_id_public(
        &self,
        stripe_payment_intent_id: &str,
    ) -> Result<PaymentRecord, AppError> {
        self.get_payment_record_by_stripe_id(stripe_payment_intent_id).await
    }
}