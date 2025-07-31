use sqlx::PgPool;
use uuid::Uuid;
use rust_decimal::Decimal;
use crate::models::*;
use crate::error::AppError;
use raffle_platform_shared::*;

pub mod box_purchase_tests;
pub mod seller_subscription_tests;
pub mod integration_tests;

/// Setup test database with clean state
pub async fn setup_test_db() -> PgPool {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost:5432/raffle_platform_test".to_string());
    
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to test database");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    pool
}

/// Clean up test data
pub async fn cleanup_test_data(pool: &PgPool) {
    let tables = vec![
        "box_purchases",
        "user_credits", 
        "raffles",
        "items",
        "sellers",
        "seller_subscriptions",
        "users",
    ];
    
    for table in tables {
        sqlx::query(&format!("TRUNCATE TABLE {} RESTART IDENTITY CASCADE", table))
            .execute(pool)
            .await
            .unwrap_or_else(|e| {
                eprintln!("Warning: Failed to truncate table {}: {}", table, e);
            });
    }
}

/// Create a test user
pub async fn create_test_user(pool: &PgPool, email: &str) -> User {
    let request = CreateUserRequest {
        username: format!("user_{}", Uuid::new_v4().to_string()[..8].to_string()),
        email: email.to_string(),
        phone_number: None,
    };

    User::create(
        pool,
        request,
        "password_hash".to_string(),
        format!("0x{}", hex::encode(&Uuid::new_v4().as_bytes()[..20])),
        "encrypted_private_key".to_string(),
        Some("encrypted_mnemonic".to_string()),
    )
    .await
    .expect("Failed to create test user")
}

/// Create a test seller
pub async fn create_test_seller(pool: &PgPool, user_id: Uuid) -> Seller {
    let request = CreateSellerRequest {
        company_name: Some("Test Company".to_string()),
        description: Some("Test Description".to_string()),
        subscription_id: None,
    };

    Seller::create(pool, user_id, request)
        .await
        .expect("Failed to create test seller")
}

/// Create a test item
pub async fn create_test_item(pool: &PgPool, seller_id: Option<Uuid>) -> Item {
    let request = CreateItemRequest {
        name: "Test Item".to_string(),
        description: Some("Test Description".to_string()),
        images: vec!["image1.jpg".to_string()],
        retail_price: Decimal::new(10000, 2), // $100.00
        cost_of_goods: Decimal::new(5000, 2), // $50.00
        stock_quantity: 1,
    };

    Item::create(pool, seller_id, request, None, None)
        .await
        .expect("Failed to create test item")
}

/// Create a test raffle
pub async fn create_test_raffle(pool: &PgPool, item_id: Uuid) -> Raffle {
    let request = CreateRaffleRequest {
        item_id,
        total_boxes: 100,
        box_price: Decimal::new(100, 2), // $1.00
        total_winners: 1,
        grid_rows: 10,
        grid_cols: 10,
    };

    Raffle::create(pool, request, None)
        .await
        .expect("Failed to create test raffle")
}

/// Create a test seller subscription
pub async fn create_test_subscription(pool: &PgPool, name: &str) -> SellerSubscription {
    SellerSubscription::create(
        pool,
        name.to_string(),
        Decimal::new(2999, 2), // $29.99
        Decimal::new(500, 2),  // 5%
        Decimal::new(300, 2),  // 3%
        Some(10),
        Some(serde_json::json!({"feature1": true})),
    )
    .await
    .expect("Failed to create test subscription")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_model_integration() {
        let pool = setup_test_db().await;
        cleanup_test_data(&pool).await;

        // Create test data chain
        let user = create_test_user(&pool, "test@example.com").await;
        let seller = create_test_seller(&pool, user.id).await;
        let item = create_test_item(&pool, Some(seller.id)).await;
        let raffle = create_test_raffle(&pool, item.id).await;

        // Verify relationships
        assert_eq!(seller.user_id, user.id);
        assert_eq!(item.seller_id, Some(seller.id));
        assert_eq!(raffle.item_id, item.id);

        // Test box purchase
        let box_purchase = BoxPurchase::create(
            &pool,
            raffle.id,
            user.id,
            1,
            Decimal::new(100, 2),
            None,
        )
        .await
        .expect("Failed to create box purchase");

        assert_eq!(box_purchase.raffle_id, raffle.id);
        assert_eq!(box_purchase.user_id, user.id);
        assert_eq!(box_purchase.box_number, 1);

        cleanup_test_data(&pool).await;
    }

    #[tokio::test]
    async fn test_user_crud_operations() {
        let pool = setup_test_db().await;
        cleanup_test_data(&pool).await;

        // Test create
        let user = create_test_user(&pool, "crud@example.com").await;
        assert!(!user.id.is_nil());
        assert_eq!(user.email, "crud@example.com");

        // Test find by id
        let found_user = User::find_by_id(&pool, user.id)
            .await
            .expect("Failed to find user")
            .expect("User not found");
        assert_eq!(found_user.id, user.id);

        // Test find by email
        let found_by_email = User::find_by_email(&pool, "crud@example.com")
            .await
            .expect("Failed to find user by email")
            .expect("User not found");
        assert_eq!(found_by_email.id, user.id);

        // Test update credit balance
        User::update_credit_balance(&pool, user.id, Decimal::new(5000, 2))
            .await
            .expect("Failed to update credit balance");

        let updated_user = User::find_by_id(&pool, user.id)
            .await
            .expect("Failed to find updated user")
            .expect("User not found");
        assert_eq!(updated_user.credit_balance, Decimal::new(5000, 2));

        // Test email verification
        User::verify_email(&pool, user.id)
            .await
            .expect("Failed to verify email");

        let verified_user = User::find_by_id(&pool, user.id)
            .await
            .expect("Failed to find verified user")
            .expect("User not found");
        assert!(verified_user.email_verified);

        cleanup_test_data(&pool).await;
    }

    #[tokio::test]
    async fn test_seller_subscription_functionality() {
        let pool = setup_test_db().await;
        cleanup_test_data(&pool).await;

        // Create subscription
        let subscription = create_test_subscription(&pool, "Test Plan").await;

        // Test fee calculations
        let item_value = Decimal::new(10000, 2); // $100.00
        let listing_fee = subscription.calculate_listing_fee(item_value);
        assert_eq!(listing_fee, Decimal::new(500, 2)); // 5% of $100 = $5.00

        let sale_amount = Decimal::new(5000, 2); // $50.00
        let transaction_fee = subscription.calculate_transaction_fee(sale_amount);
        assert_eq!(transaction_fee, Decimal::new(150, 2)); // 3% of $50 = $1.50

        // Test find by name
        let found_subscription = SellerSubscription::find_by_name(&pool, "Test Plan")
            .await
            .expect("Failed to find subscription")
            .expect("Subscription not found");
        assert_eq!(found_subscription.id, subscription.id);

        cleanup_test_data(&pool).await;
    }

    #[tokio::test]
    async fn test_box_purchase_operations() {
        let pool = setup_test_db().await;
        cleanup_test_data(&pool).await;

        // Setup test data
        let user = create_test_user(&pool, "box@example.com").await;
        let item = create_test_item(&pool, None).await;
        let raffle = create_test_raffle(&pool, item.id).await;

        // Test single box purchase
        let box_purchase = BoxPurchase::create(
            &pool,
            raffle.id,
            user.id,
            1,
            Decimal::new(100, 2),
            None,
        )
        .await
        .expect("Failed to create box purchase");

        // Test is_box_purchased
        let is_purchased = BoxPurchase::is_box_purchased(&pool, raffle.id, 1)
            .await
            .expect("Failed to check if box is purchased");
        assert!(is_purchased);

        let is_not_purchased = BoxPurchase::is_box_purchased(&pool, raffle.id, 2)
            .await
            .expect("Failed to check if box is purchased");
        assert!(!is_not_purchased);

        // Test get_purchased_box_numbers
        let purchased_numbers = BoxPurchase::get_purchased_box_numbers(&pool, raffle.id)
            .await
            .expect("Failed to get purchased box numbers");
        assert_eq!(purchased_numbers, vec![1]);

        // Test get_available_box_numbers
        let available_numbers = BoxPurchase::get_available_box_numbers(&pool, raffle.id, 5)
            .await
            .expect("Failed to get available box numbers");
        assert_eq!(available_numbers, vec![2, 3, 4, 5]);

        // Test bulk create
        let bulk_purchases = vec![
            (raffle.id, user.id, 2, Decimal::new(100, 2), None),
            (raffle.id, user.id, 3, Decimal::new(100, 2), None),
        ];

        let created_purchases = BoxPurchase::bulk_create(&pool, bulk_purchases)
            .await
            .expect("Failed to bulk create box purchases");
        assert_eq!(created_purchases.len(), 2);

        // Test statistics
        let stats = BoxPurchase::get_raffle_statistics(&pool, raffle.id)
            .await
            .expect("Failed to get raffle statistics");
        assert_eq!(stats.total_purchases, 3);
        assert_eq!(stats.unique_buyers, 1);
        assert_eq!(stats.total_revenue, Decimal::new(300, 2));

        cleanup_test_data(&pool).await;
    }

    #[tokio::test]
    async fn test_user_statistics() {
        let pool = setup_test_db().await;
        cleanup_test_data(&pool).await;

        // Create test users with different roles
        let mut user1 = create_test_user(&pool, "user1@example.com").await;
        let mut user2 = create_test_user(&pool, "user2@example.com").await;
        
        // Update one user to seller role
        User::update_role(&pool, user2.id, UserRole::Seller)
            .await
            .expect("Failed to update user role");

        // Verify email for one user
        User::verify_email(&pool, user1.id)
            .await
            .expect("Failed to verify email");

        // Get statistics
        let stats = User::get_statistics(&pool)
            .await
            .expect("Failed to get user statistics");

        assert_eq!(stats.total_users, 2);
        assert_eq!(stats.active_users, 2);
        assert_eq!(stats.verified_users, 1);
        assert_eq!(stats.sellers, 1);

        cleanup_test_data(&pool).await;
    }
}