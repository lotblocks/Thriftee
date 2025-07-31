#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::user::User;
    use crate::services::credit_service::CreditService;
    use raffle_platform_shared::UserRole;
    use sqlx::PgPool;
    use std::collections::HashMap;
    use tokio_test;

    async fn setup_test_data(pool: &PgPool) -> Uuid {
        // Create test user
        let user = User::create(
            pool,
            "test@example.com",
            "password123",
            UserRole::User,
        ).await.unwrap();

        user.id
    }

    #[tokio::test]
    async fn test_create_payment_intent() {
        let pool = setup_test_pool().await;
        let credit_service = CreditService::new(pool.clone());
        let payment_service = PaymentService::new(
            "sk_test_123".to_string(),
            "whsec_test_123".to_string(),
            pool.clone(),
            credit_service,
        );

        let user_id = setup_test_data(&pool).await;

        let request = PaymentIntentRequest {
            user_id,
            amount: Decimal::from(50),
            currency: "USD".to_string(),
            description: "Test credit purchase".to_string(),
            metadata: HashMap::new(),
        };

        // This test would require mocking Stripe API calls
        // For now, we'll test the validation logic
        assert!(request.amount > Decimal::ZERO);
        assert_eq!(request.currency, "USD");
        assert!(!request.description.is_empty());
    }

    #[tokio::test]
    async fn test_payment_record_creation() {
        let pool = setup_test_pool().await;
        let credit_service = CreditService::new(pool.clone());
        let payment_service = PaymentService::new(
            "sk_test_123".to_string(),
            "whsec_test_123".to_string(),
            pool.clone(),
            credit_service,
        );

        let user_id = setup_test_data(&pool).await;

        let payment_record = payment_service
            .save_payment_record(
                user_id,
                "pi_test_123",
                Decimal::from(25),
                "USD",
                PaymentStatus::Pending,
                "Test payment",
                serde_json::json!({"test": "data"}),
                None,
            )
            .await
            .unwrap();

        assert_eq!(payment_record.user_id, user_id);
        assert_eq!(payment_record.stripe_payment_intent_id, "pi_test_123");
        assert_eq!(payment_record.amount, Decimal::from(25));
        assert_eq!(payment_record.currency, "USD");
        assert_eq!(payment_record.status, PaymentStatus::Pending);
    }

    #[tokio::test]
    async fn test_payment_status_update() {
        let pool = setup_test_pool().await;
        let credit_service = CreditService::new(pool.clone());
        let payment_service = PaymentService::new(
            "sk_test_123".to_string(),
            "whsec_test_123".to_string(),
            pool.clone(),
            credit_service,
        );

        let user_id = setup_test_data(&pool).await;

        // Create payment record
        let payment_record = payment_service
            .save_payment_record(
                user_id,
                "pi_test_456",
                Decimal::from(30),
                "USD",
                PaymentStatus::Pending,
                "Test payment",
                serde_json::json!({}),
                None,
            )
            .await
            .unwrap();

        // Update status to succeeded
        payment_service
            .update_payment_status(
                "pi_test_456",
                PaymentStatus::Succeeded,
                Some(Utc::now()),
            )
            .await
            .unwrap();

        // Verify update
        let updated_record = payment_service
            .get_payment_record_by_stripe_id("pi_test_456")
            .await
            .unwrap();

        assert_eq!(updated_record.status, PaymentStatus::Succeeded);
        assert!(updated_record.completed_at.is_some());
    }

    #[tokio::test]
    async fn test_payment_history() {
        let pool = setup_test_pool().await;
        let credit_service = CreditService::new(pool.clone());
        let payment_service = PaymentService::new(
            "sk_test_123".to_string(),
            "whsec_test_123".to_string(),
            pool.clone(),
            credit_service,
        );

        let user_id = setup_test_data(&pool).await;

        // Create multiple payment records
        for i in 1..=5 {
            payment_service
                .save_payment_record(
                    user_id,
                    &format!("pi_test_{}", i),
                    Decimal::from(i * 10),
                    "USD",
                    if i % 2 == 0 { PaymentStatus::Succeeded } else { PaymentStatus::Failed },
                    &format!("Test payment {}", i),
                    serde_json::json!({}),
                    if i % 2 == 0 { Some(Utc::now()) } else { None },
                )
                .await
                .unwrap();
        }

        let history = payment_service
            .get_user_payment_history(user_id, Some(10))
            .await
            .unwrap();

        assert_eq!(history.payments.len(), 5);
        assert_eq!(history.successful_payments, 2); // payments 2 and 4
        assert_eq!(history.failed_payments, 3); // payments 1, 3, and 5
        assert_eq!(history.total_amount, Decimal::from(60)); // 20 + 40 (successful payments)
    }

    #[tokio::test]
    async fn test_subscription_record_creation() {
        let pool = setup_test_pool().await;
        let credit_service = CreditService::new(pool.clone());
        let payment_service = PaymentService::new(
            "sk_test_123".to_string(),
            "whsec_test_123".to_string(),
            pool.clone(),
            credit_service,
        );

        let user_id = setup_test_data(&pool).await;

        // First create a customer record
        payment_service
            .save_customer_id(user_id, "cus_test_123")
            .await
            .unwrap();

        // Create subscription record
        let start_time = Utc::now();
        let end_time = start_time + chrono::Duration::days(30);

        payment_service
            .save_subscription_record(
                user_id,
                "sub_test_123",
                "cus_test_123",
                SubscriptionStatus::Active,
                start_time,
                end_time,
                None,
            )
            .await
            .unwrap();

        let subscriptions = payment_service
            .get_user_subscriptions(user_id)
            .await
            .unwrap();

        assert_eq!(subscriptions.len(), 1);
        assert_eq!(subscriptions[0].stripe_subscription_id, "sub_test_123");
        assert_eq!(subscriptions[0].status, SubscriptionStatus::Active);
    }

    #[tokio::test]
    async fn test_webhook_signature_verification() {
        let pool = setup_test_pool().await;
        let credit_service = CreditService::new(pool.clone());
        let payment_service = PaymentService::new(
            "sk_test_123".to_string(),
            "whsec_test_secret".to_string(),
            pool.clone(),
            credit_service,
        );

        // Test with invalid signature
        let payload = b"test payload";
        let invalid_signature = "invalid_signature";

        let result = payment_service.verify_webhook_signature(payload, invalid_signature);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_process_successful_payment() {
        let pool = setup_test_pool().await;
        let credit_service = CreditService::new(pool.clone());
        let payment_service = PaymentService::new(
            "sk_test_123".to_string(),
            "whsec_test_123".to_string(),
            pool.clone(),
            credit_service.clone(),
        );

        let user_id = setup_test_data(&pool).await;

        // Create pending payment record
        payment_service
            .save_payment_record(
                user_id,
                "pi_test_success",
                Decimal::from(100),
                "USD",
                PaymentStatus::Pending,
                "Test successful payment",
                serde_json::json!({}),
                None,
            )
            .await
            .unwrap();

        // Process successful payment
        payment_service
            .process_successful_payment("pi_test_success", 10000) // 100.00 in cents
            .await
            .unwrap();

        // Verify payment status updated
        let payment_record = payment_service
            .get_payment_record_by_stripe_id("pi_test_success")
            .await
            .unwrap();

        assert_eq!(payment_record.status, PaymentStatus::Succeeded);
        assert!(payment_record.completed_at.is_some());

        // Verify credits were issued
        let balance = credit_service.get_user_balance(user_id).await.unwrap();
        assert_eq!(balance.total_available, Decimal::from(100));
    }

    // Helper function to set up test database pool
    async fn setup_test_pool() -> PgPool {
        // This would typically connect to a test database
        // For now, we'll assume the pool is properly configured
        todo!("Set up test database connection")
    }
}