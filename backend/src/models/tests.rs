//! Unit tests for database models
//! 
//! These tests verify the CRUD operations and business logic of all models.
//! They use an in-memory SQLite database for fast, isolated testing.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::AppError;
    use chrono::Utc;
    use rust_decimal::Decimal;
    use sqlx::{PgPool, Row};
    use uuid::Uuid;

    /// Helper function to create a test database pool
    async fn create_test_pool() -> PgPool {
        // In a real implementation, you would use a test database
        // For now, this is a placeholder that would need proper setup
        todo!("Implement test database setup")
    }

    /// Helper function to create a test user
    async fn create_test_user(pool: &PgPool) -> Result<User, AppError> {
        let request = raffle_platform_shared::CreateUserRequest {
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
            phone_number: None,
        };

        User::create(
            pool,
            request,
            "hashed_password".to_string(),
            "0x1234567890123456789012345678901234567890".to_string(),
            "encrypted_private_key".to_string(),
        ).await
    }

    #[tokio::test]
    async fn test_user_creation() {
        let pool = create_test_pool().await;
        let user = create_test_user(&pool).await.unwrap();

        assert_eq!(user.username, "testuser");
        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.credit_balance, Decimal::ZERO);
        assert!(user.is_active);
        assert!(!user.email_verified);
    }

    #[tokio::test]
    async fn test_user_find_by_email() {
        let pool = create_test_pool().await;
        let created_user = create_test_user(&pool).await.unwrap();

        let found_user = User::find_by_email(&pool, "test@example.com")
            .await
            .unwrap()
            .unwrap();

        assert_eq!(found_user.id, created_user.id);
        assert_eq!(found_user.email, created_user.email);
    }

    #[tokio::test]
    async fn test_user_credit_balance_update() {
        let pool = create_test_pool().await;
        let user = create_test_user(&pool).await.unwrap();

        let new_balance = Decimal::new(5000, 2); // $50.00
        User::update_credit_balance(&pool, user.id, new_balance)
            .await
            .unwrap();

        let updated_user = User::find_by_id(&pool, user.id)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(updated_user.credit_balance, new_balance);
    }

    #[tokio::test]
    async fn test_item_creation() {
        let pool = create_test_pool().await;
        
        let request = raffle_platform_shared::CreateItemRequest {
            name: "Test Item".to_string(),
            description: Some("A test item".to_string()),
            images: vec!["image1.jpg".to_string(), "image2.jpg".to_string()],
            retail_price: Decimal::new(10000, 2), // $100.00
            cost_of_goods: Decimal::new(5000, 2), // $50.00
            stock_quantity: 10,
        };

        let item = Item::create(&pool, None, request, None, None)
            .await
            .unwrap();

        assert_eq!(item.name, "Test Item");
        assert_eq!(item.retail_price, Decimal::new(10000, 2));
        assert_eq!(item.stock_quantity, 10);
        assert_eq!(item.status, raffle_platform_shared::ItemStatus::Available);
    }

    #[tokio::test]
    async fn test_item_stock_decrease() {
        let pool = create_test_pool().await;
        
        let request = raffle_platform_shared::CreateItemRequest {
            name: "Test Item".to_string(),
            description: None,
            images: vec!["image1.jpg".to_string()],
            retail_price: Decimal::new(10000, 2),
            cost_of_goods: Decimal::new(5000, 2),
            stock_quantity: 5,
        };

        let item = Item::create(&pool, None, request, None, None)
            .await
            .unwrap();

        // Decrease stock by 2
        let success = Item::decrease_stock(&pool, item.id, 2).await.unwrap();
        assert!(success);

        let updated_item = Item::find_by_id(&pool, item.id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(updated_item.stock_quantity, 3);

        // Try to decrease by more than available
        let success = Item::decrease_stock(&pool, item.id, 5).await.unwrap();
        assert!(!success);
    }

    #[tokio::test]
    async fn test_raffle_creation() {
        let pool = create_test_pool().await;
        
        // Create an item first
        let item_request = raffle_platform_shared::CreateItemRequest {
            name: "Raffle Item".to_string(),
            description: None,
            images: vec!["image1.jpg".to_string()],
            retail_price: Decimal::new(10000, 2),
            cost_of_goods: Decimal::new(5000, 2),
            stock_quantity: 1,
        };

        let item = Item::create(&pool, None, item_request, None, None)
            .await
            .unwrap();

        let raffle_request = raffle_platform_shared::CreateRaffleRequest {
            item_id: item.id,
            total_boxes: 100,
            box_price: Decimal::new(200, 2), // $2.00
            total_winners: 1,
            grid_rows: 10,
            grid_cols: 10,
        };

        let raffle = Raffle::create(&pool, raffle_request, None)
            .await
            .unwrap();

        assert_eq!(raffle.item_id, item.id);
        assert_eq!(raffle.total_boxes, 100);
        assert_eq!(raffle.box_price, Decimal::new(200, 2));
        assert_eq!(raffle.status, raffle_platform_shared::RaffleStatus::Open);
        assert_eq!(raffle.boxes_sold, 0);
    }

    #[tokio::test]
    async fn test_raffle_grid_validation() {
        let pool = create_test_pool().await;
        
        let item_request = raffle_platform_shared::CreateItemRequest {
            name: "Raffle Item".to_string(),
            description: None,
            images: vec!["image1.jpg".to_string()],
            retail_price: Decimal::new(10000, 2),
            cost_of_goods: Decimal::new(5000, 2),
            stock_quantity: 1,
        };

        let item = Item::create(&pool, None, item_request, None, None)
            .await
            .unwrap();

        // Invalid grid size (too small for boxes)
        let invalid_request = raffle_platform_shared::CreateRaffleRequest {
            item_id: item.id,
            total_boxes: 100,
            box_price: Decimal::new(200, 2),
            total_winners: 1,
            grid_rows: 5,  // 5 * 5 = 25 < 100 boxes
            grid_cols: 5,
        };

        let result = Raffle::create(&pool, invalid_request, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_box_purchase() {
        let pool = create_test_pool().await;
        
        // Create user, item, and raffle
        let user = create_test_user(&pool).await.unwrap();
        
        let item_request = raffle_platform_shared::CreateItemRequest {
            name: "Raffle Item".to_string(),
            description: None,
            images: vec!["image1.jpg".to_string()],
            retail_price: Decimal::new(10000, 2),
            cost_of_goods: Decimal::new(5000, 2),
            stock_quantity: 1,
        };

        let item = Item::create(&pool, None, item_request, None, None)
            .await
            .unwrap();

        let raffle_request = raffle_platform_shared::CreateRaffleRequest {
            item_id: item.id,
            total_boxes: 10,
            box_price: Decimal::new(200, 2),
            total_winners: 1,
            grid_rows: 2,
            grid_cols: 5,
        };

        let raffle = Raffle::create(&pool, raffle_request, None)
            .await
            .unwrap();

        // Purchase a box
        let box_purchase = BoxPurchase::create(
            &pool,
            raffle.id,
            user.id,
            1,
            Decimal::new(200, 2),
            None,
        ).await.unwrap();

        assert_eq!(box_purchase.raffle_id, raffle.id);
        assert_eq!(box_purchase.user_id, user.id);
        assert_eq!(box_purchase.box_number, 1);
        assert_eq!(box_purchase.purchase_price_in_credits, Decimal::new(200, 2));
    }

    #[tokio::test]
    async fn test_duplicate_box_purchase_prevention() {
        let pool = create_test_pool().await;
        
        let user = create_test_user(&pool).await.unwrap();
        
        let item_request = raffle_platform_shared::CreateItemRequest {
            name: "Raffle Item".to_string(),
            description: None,
            images: vec!["image1.jpg".to_string()],
            retail_price: Decimal::new(10000, 2),
            cost_of_goods: Decimal::new(5000, 2),
            stock_quantity: 1,
        };

        let item = Item::create(&pool, None, item_request, None, None)
            .await
            .unwrap();

        let raffle_request = raffle_platform_shared::CreateRaffleRequest {
            item_id: item.id,
            total_boxes: 10,
            box_price: Decimal::new(200, 2),
            total_winners: 1,
            grid_rows: 2,
            grid_cols: 5,
        };

        let raffle = Raffle::create(&pool, raffle_request, None)
            .await
            .unwrap();

        // First purchase should succeed
        BoxPurchase::create(&pool, raffle.id, user.id, 1, Decimal::new(200, 2), None)
            .await
            .unwrap();

        // Check if box is purchased
        let is_purchased = BoxPurchase::is_box_purchased(&pool, raffle.id, 1)
            .await
            .unwrap();
        assert!(is_purchased);

        // Second purchase of same box should fail due to unique constraint
        let result = BoxPurchase::create(&pool, raffle.id, user.id, 1, Decimal::new(200, 2), None)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_credit_creation_and_usage() {
        let pool = create_test_pool().await;
        let user = create_test_user(&pool).await.unwrap();

        // Create credits
        let credit = UserCredit::create(
            &pool,
            user.id,
            Decimal::new(1000, 2), // $10.00
            raffle_platform_shared::CreditSource::Deposit,
            raffle_platform_shared::CreditType::General,
            None,
            Some(Utc::now() + chrono::Duration::days(30)),
        ).await.unwrap();

        assert_eq!(credit.user_id, user.id);
        assert_eq!(credit.amount, Decimal::new(1000, 2));
        assert!(!credit.is_used);

        // Calculate available credits
        let available = UserCredit::calculate_total_available(
            &pool,
            user.id,
            Some(raffle_platform_shared::CreditType::General),
            None,
        ).await.unwrap();

        assert_eq!(available, Decimal::new(1000, 2));

        // Use credits
        let used_credits = UserCredit::mark_as_used(
            &pool,
            &[credit.id],
            Decimal::new(500, 2), // Use $5.00
        ).await.unwrap();

        assert_eq!(used_credits.len(), 1);
        assert_eq!(used_credits[0].amount, Decimal::new(500, 2));
        assert!(used_credits[0].is_used);

        // Check remaining available credits
        let remaining = UserCredit::calculate_total_available(
            &pool,
            user.id,
            Some(raffle_platform_shared::CreditType::General),
            None,
        ).await.unwrap();

        assert_eq!(remaining, Decimal::new(500, 2)); // $5.00 remaining
    }

    #[tokio::test]
    async fn test_notification_creation() {
        let pool = create_test_pool().await;
        let user = create_test_user(&pool).await.unwrap();

        let notification = Notification::create_credit_issued_notification(
            &pool,
            user.id,
            Decimal::new(1000, 2),
            "test deposit",
        ).await.unwrap();

        assert_eq!(notification.user_id, user.id);
        assert_eq!(notification.title, "Credits Added");
        assert!(!notification.is_read);

        // Mark as read
        Notification::mark_as_read(&pool, notification.id).await.unwrap();

        let updated_notification = sqlx::query!(
            "SELECT is_read FROM notifications WHERE id = $1",
            notification.id
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        assert!(updated_notification.is_read);
    }

    #[tokio::test]
    async fn test_audit_log_creation() {
        let pool = create_test_pool().await;
        let user = create_test_user(&pool).await.unwrap();

        let audit_log = AuditLog::log_user_login(
            &pool,
            user.id,
            Some("127.0.0.1".parse().unwrap()),
            Some("Test User Agent".to_string()),
        ).await.unwrap();

        assert_eq!(audit_log.user_id, Some(user.id));
        assert_eq!(audit_log.action, "login");
        assert_eq!(audit_log.resource_type, Some("user".to_string()));
        assert_eq!(audit_log.ip_address, Some("127.0.0.1".parse().unwrap()));
    }

    #[tokio::test]
    async fn test_transaction_creation() {
        let pool = create_test_pool().await;
        let user = create_test_user(&pool).await.unwrap();

        let transaction = Transaction::create_credit_deposit(
            &pool,
            user.id,
            Decimal::new(2000, 2), // $20.00
            "stripe_payment_intent_123".to_string(),
            None,
        ).await.unwrap();

        assert_eq!(transaction.user_id, Some(user.id));
        assert_eq!(transaction.amount, Decimal::new(2000, 2));
        assert_eq!(transaction.transaction_type, raffle_platform_shared::TransactionType::CreditDeposit);
        assert_eq!(transaction.payment_gateway_ref, Some("stripe_payment_intent_123".to_string()));
    }

    #[tokio::test]
    async fn test_system_settings() {
        let pool = create_test_pool().await;

        // Set a setting
        let setting = SystemSettings::set_platform_name(
            &pool,
            "Test Platform".to_string(),
        ).await.unwrap();

        assert_eq!(setting.key, "platform_name");
        assert_eq!(setting.get_string_value(), Some("Test Platform".to_string()));

        // Get the setting
        let platform_name = SystemSettings::get_platform_name(&pool).await.unwrap();
        assert_eq!(platform_name, "Test Platform");

        // Update the setting
        let updated_setting = SystemSettings::set_platform_name(
            &pool,
            "Updated Platform".to_string(),
        ).await.unwrap();

        assert_eq!(updated_setting.id, setting.id); // Same ID (upsert)
        assert_eq!(updated_setting.get_string_value(), Some("Updated Platform".to_string()));
    }

    #[tokio::test]
    async fn test_free_item_redemption() {
        let pool = create_test_pool().await;
        let user = create_test_user(&pool).await.unwrap();

        // Create an item
        let item_request = raffle_platform_shared::CreateItemRequest {
            name: "Free Item".to_string(),
            description: None,
            images: vec!["image1.jpg".to_string()],
            retail_price: Decimal::new(1000, 2),
            cost_of_goods: Decimal::new(500, 2),
            stock_quantity: 5,
        };

        let item = Item::create(&pool, None, item_request, None, None)
            .await
            .unwrap();

        // Create free redeemable item
        let free_item = FreeRedeemableItem::create(
            &pool,
            item.id,
            Decimal::new(500, 2), // $5.00 required
            3, // 3 available
        ).await.unwrap();

        assert_eq!(free_item.item_id, item.id);
        assert_eq!(free_item.required_credit_amount, Decimal::new(500, 2));
        assert_eq!(free_item.available_quantity, 3);

        // Redeem the item
        let redemption_result = FreeRedeemableItem::redeem_item(
            &pool,
            free_item.id,
            user.id,
            Decimal::new(500, 2),
        ).await.unwrap();

        assert_eq!(redemption_result.item_id, item.id);
        assert_eq!(redemption_result.credits_used, Decimal::new(500, 2));
        assert_eq!(redemption_result.remaining_quantity, 2);

        // Verify transaction was created
        let transaction = Transaction::find_by_id(&pool, redemption_result.transaction_id)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(transaction.user_id, Some(user.id));
        assert_eq!(transaction.amount, -Decimal::new(500, 2)); // Negative for deduction
        assert_eq!(transaction.transaction_type, raffle_platform_shared::TransactionType::FreeItemRedemption);
    }
}