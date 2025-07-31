use actix_web::{test, web, App};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{
    user::{User, UserRole},
    raffle::RaffleStatus,
    item::ItemCondition,
};
use crate::services::{
    auth_service::AuthService,
    raffle_service::RaffleService,
    credit_service::CreditService,
    blockchain_service::BlockchainService,
};
use crate::utils::test_helpers::{create_test_app, cleanup_test_data};

/// Test complete user registration and raffle participation workflow
#[actix_web::test]
async fn test_complete_user_raffle_workflow() {
    let pool = create_test_pool().await;
    let app = create_test_app(pool.clone()).await;
    
    // Step 1: User Registration
    let registration_data = json!({
        "email": "newuser@example.com",
        "password": "SecurePassword123!",
        "confirm_password": "SecurePassword123!"
    });

    let req = test::TestRequest::post()
        .uri("/api/auth/register")
        .set_json(&registration_data)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    let registration_body: serde_json::Value = test::read_body_json(resp).await;
    let access_token = registration_body["access_token"].as_str().unwrap();
    let user_id = registration_body["user"]["id"].as_str().unwrap();

    // Step 2: Add credits to user account
    add_credits_to_user(&pool, &Uuid::parse_str(user_id).unwrap(), 100.0).await;

    // Step 3: Create a seller and item for testing
    let seller = create_test_seller(&pool, "seller@example.com").await;
    let item = create_test_item(&pool, &seller.id, "iPhone 15 Pro").await;
    let raffle = create_test_raffle(&pool, &seller.id, &item.id, 10).await; // Small raffle for testing

    // Step 4: Browse available raffles
    let req = test::TestRequest::get()
        .uri("/api/raffles")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let raffles_body: serde_json::Value = test::read_body_json(resp).await;
    let raffles = raffles_body["raffles"].as_array().unwrap();
    assert!(raffles.len() > 0);

    // Step 5: Get specific raffle details
    let req = test::TestRequest::get()
        .uri(&format!("/api/raffles/{}", raffle.id))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let raffle_body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(raffle_body["raffle"]["id"], raffle.id.to_string());

    // Step 6: Purchase raffle boxes
    let purchase_data = json!({
        "box_numbers": [1, 2, 3],
        "payment_method": "credits"
    });

    let req = test::TestRequest::post()
        .uri(&format!("/api/raffles/{}/buy-box", raffle.id))
        .insert_header(("Authorization", format!("Bearer {}", access_token)))
        .set_json(&purchase_data)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let purchase_body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(purchase_body["participants"].as_array().unwrap().len(), 3);

    // Step 7: Check user's participation history
    let req = test::TestRequest::get()
        .uri("/api/users/participations")
        .insert_header(("Authorization", format!("Bearer {}", access_token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let participations_body: serde_json::Value = test::read_body_json(resp).await;
    let participations = participations_body["participations"].as_array().unwrap();
    assert!(participations.len() > 0);

    // Step 8: Check credit balance after purchase
    let req = test::TestRequest::get()
        .uri("/api/credits/balance")
        .insert_header(("Authorization", format!("Bearer {}", access_token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let balance_body: serde_json::Value = test::read_body_json(resp).await;
    let remaining_balance = balance_body["balance"].as_f64().unwrap();
    assert_eq!(remaining_balance, 70.0); // 100 - (3 * 10)

    cleanup_test_data(&pool, "newuser@example.com").await;
    cleanup_test_data(&pool, "seller@example.com").await;
}

/// Test seller workflow: registration, item creation, raffle creation
#[actix_web::test]
async fn test_seller_workflow() {
    let pool = create_test_pool().await;
    let app = create_test_app(pool.clone()).await;
    
    // Step 1: Seller Registration
    let registration_data = json!({
        "email": "newseller@example.com",
        "password": "SecurePassword123!",
        "confirm_password": "SecurePassword123!",
        "role": "seller"
    });

    let req = test::TestRequest::post()
        .uri("/api/auth/register")
        .set_json(&registration_data)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    let registration_body: serde_json::Value = test::read_body_json(resp).await;
    let access_token = registration_body["access_token"].as_str().unwrap();

    // Step 2: Create an item
    let item_data = json!({
        "title": "MacBook Pro M3",
        "description": "Latest MacBook Pro with M3 chip",
        "price": 2499.99,
        "category": "Electronics",
        "condition": "new",
        "image_urls": ["macbook1.jpg", "macbook2.jpg"]
    });

    let req = test::TestRequest::post()
        .uri("/api/items")
        .insert_header(("Authorization", format!("Bearer {}", access_token)))
        .set_json(&item_data)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    let item_body: serde_json::Value = test::read_body_json(resp).await;
    let item_id = item_body["item"]["id"].as_str().unwrap();

    // Step 3: Create a raffle for the item
    let raffle_data = json!({
        "item_id": item_id,
        "total_boxes": 250,
        "box_price": 12.0,
        "total_winners": 1,
        "end_date": "2024-12-31T23:59:59Z"
    });

    let req = test::TestRequest::post()
        .uri("/api/raffles")
        .insert_header(("Authorization", format!("Bearer {}", access_token)))
        .set_json(&raffle_data)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    let raffle_body: serde_json::Value = test::read_body_json(resp).await;
    let raffle_id = raffle_body["raffle"]["id"].as_str().unwrap();

    // Step 4: Check seller's raffles
    let req = test::TestRequest::get()
        .uri("/api/sellers/raffles")
        .insert_header(("Authorization", format!("Bearer {}", access_token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let seller_raffles_body: serde_json::Value = test::read_body_json(resp).await;
    let raffles = seller_raffles_body["raffles"].as_array().unwrap();
    assert!(raffles.len() > 0);

    // Step 5: Update raffle (if needed)
    let update_data = json!({
        "end_date": "2024-11-30T23:59:59Z"
    });

    let req = test::TestRequest::put()
        .uri(&format!("/api/raffles/{}", raffle_id))
        .insert_header(("Authorization", format!("Bearer {}", access_token)))
        .set_json(&update_data)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    cleanup_test_data(&pool, "newseller@example.com").await;
}

/// Test complete raffle lifecycle from creation to winner selection
#[actix_web::test]
async fn test_complete_raffle_lifecycle() {
    let pool = create_test_pool().await;
    let app = create_test_app(pool.clone()).await;
    
    // Setup: Create seller, item, and raffle
    let seller = create_test_seller(&pool, "lifecycle_seller@example.com").await;
    let item = create_test_item(&pool, &seller.id, "Test Lifecycle Item").await;
    let raffle = create_test_raffle(&pool, &seller.id, &item.id, 5).await; // Very small raffle

    // Create multiple buyers
    let mut buyers = Vec::new();
    let mut tokens = Vec::new();
    
    for i in 1..=5 {
        let email = format!("buyer{}@example.com", i);
        let buyer = create_test_user(&pool, &email, "password123").await;
        add_credits_to_user(&pool, &buyer.id, 50.0).await;
        let auth_tokens = get_auth_tokens(&pool, &buyer).await;
        buyers.push(buyer);
        tokens.push(auth_tokens);
    }

    // Phase 1: Buyers purchase boxes
    for (i, token) in tokens.iter().enumerate() {
        let purchase_data = json!({
            "box_numbers": [i + 1],
            "payment_method": "credits"
        });

        let req = test::TestRequest::post()
            .uri(&format!("/api/raffles/{}/buy-box", raffle.id))
            .insert_header(("Authorization", format!("Bearer {}", token.access_token)))
            .set_json(&purchase_data)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);
    }

    // Phase 2: Check that raffle is now full
    let req = test::TestRequest::get()
        .uri(&format!("/api/raffles/{}", raffle.id))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let raffle_body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(raffle_body["raffle"]["status"], "full");
    assert_eq!(raffle_body["raffle"]["boxes_sold"], 5);

    // Phase 3: Trigger winner selection (this would normally be done by blockchain)
    let raffle_service = RaffleService::new(pool.clone());
    let winners = raffle_service.select_winners(&raffle.id).await.unwrap();
    assert_eq!(winners.len(), 1);

    // Phase 4: Check that non-winners received credits
    for (i, buyer) in buyers.iter().enumerate() {
        let credit_service = CreditService::new(pool.clone());
        let balance = credit_service.get_user_balance(&buyer.id).await.unwrap();
        
        if winners.iter().any(|w| w.user_id == buyer.id) {
            // Winner should have original balance minus purchase
            assert_eq!(balance, 40.0); // 50 - 10
        } else {
            // Non-winner should have original balance (credits refunded)
            assert_eq!(balance, 50.0);
        }
    }

    // Cleanup
    cleanup_test_data(&pool, "lifecycle_seller@example.com").await;
    for i in 1..=5 {
        cleanup_test_data(&pool, &format!("buyer{}@example.com", i)).await;
    }
}

/// Test payment processing workflow with Stripe integration
#[actix_web::test]
async fn test_payment_workflow() {
    let pool = create_test_pool().await;
    let app = create_test_app(pool.clone()).await;
    
    // Create test user
    let user = create_test_user(&pool, "payment_user@example.com", "password123").await;
    let tokens = get_auth_tokens(&pool, &user).await;

    // Step 1: Create payment intent for credit purchase
    let payment_data = json!({
        "amount": 50.0,
        "currency": "usd",
        "payment_method": "card"
    });

    let req = test::TestRequest::post()
        .uri("/api/payments/create-intent")
        .insert_header(("Authorization", format!("Bearer {}", tokens.access_token)))
        .set_json(&payment_data)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let payment_body: serde_json::Value = test::read_body_json(resp).await;
    assert!(payment_body["client_secret"].is_string());
    let payment_intent_id = payment_body["payment_intent_id"].as_str().unwrap();

    // Step 2: Simulate successful payment webhook
    let webhook_data = json!({
        "type": "payment_intent.succeeded",
        "data": {
            "object": {
                "id": payment_intent_id,
                "amount": 5000, // $50.00 in cents
                "currency": "usd",
                "status": "succeeded",
                "metadata": {
                    "user_id": user.id.to_string(),
                    "credit_amount": "50.0"
                }
            }
        }
    });

    let req = test::TestRequest::post()
        .uri("/api/payments/webhook")
        .insert_header(("stripe-signature", "test_signature"))
        .set_json(&webhook_data)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    // Step 3: Verify credits were added to user account
    let req = test::TestRequest::get()
        .uri("/api/credits/balance")
        .insert_header(("Authorization", format!("Bearer {}", tokens.access_token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let balance_body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(balance_body["balance"].as_f64().unwrap(), 50.0);

    cleanup_test_data(&pool, "payment_user@example.com").await;
}

/// Test error handling and edge cases
#[actix_web::test]
async fn test_error_handling_workflow() {
    let pool = create_test_pool().await;
    let app = create_test_app(pool.clone()).await;
    
    // Test 1: Invalid authentication
    let req = test::TestRequest::get()
        .uri("/api/credits/balance")
        .insert_header(("Authorization", "Bearer invalid_token"))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);

    // Test 2: Accessing non-existent raffle
    let req = test::TestRequest::get()
        .uri(&format!("/api/raffles/{}", Uuid::new_v4()))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);

    // Test 3: Invalid input validation
    let user = create_test_user(&pool, "error_user@example.com", "password123").await;
    let tokens = get_auth_tokens(&pool, &user).await;

    let invalid_purchase_data = json!({
        "box_numbers": [], // Empty array should be invalid
        "payment_method": "credits"
    });

    let req = test::TestRequest::post()
        .uri(&format!("/api/raffles/{}/buy-box", Uuid::new_v4()))
        .insert_header(("Authorization", format!("Bearer {}", tokens.access_token)))
        .set_json(&invalid_purchase_data)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);

    cleanup_test_data(&pool, "error_user@example.com").await;
}

// Helper functions and test utilities
async fn create_test_pool() -> PgPool {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://test:test@localhost/raffle_platform_test".to_string());
    
    PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database")
}

// Additional helper functions would be implemented here...
// (Similar to the ones in the integration tests but adapted for E2E testing)

#[cfg(test)]
mod test_helpers {
    use super::*;
    // Test helper implementations...
}