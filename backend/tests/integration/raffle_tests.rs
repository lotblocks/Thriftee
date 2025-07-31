use actix_web::{test, web, App};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::handlers::raffles;
use crate::models::{
    raffle::{Raffle, RaffleStatus},
    item::{Item, ItemCondition},
    user::{User, UserRole},
};
use crate::services::{raffle_service::RaffleService, item_service::ItemService};
use crate::utils::test_helpers::{create_test_app, create_test_user, create_test_item, cleanup_test_data};

#[actix_web::test]
async fn test_create_raffle_success() {
    let pool = create_test_pool().await;
    let app = create_test_app(pool.clone()).await;
    
    // Create test seller and item
    let seller = create_test_user(&pool, "seller@example.com", "password123").await;
    let item = create_test_item(&pool, &seller.id, "Test Item").await;
    let tokens = get_auth_tokens(&pool, &seller).await;

    let raffle_data = json!({
        "item_id": item.id,
        "total_boxes": 100,
        "box_price": 10.0,
        "total_winners": 1,
        "end_date": "2024-12-31T23:59:59Z"
    });

    let req = test::TestRequest::post()
        .uri("/api/raffles")
        .insert_header(("Authorization", format!("Bearer {}", tokens.access_token)))
        .set_json(&raffle_data)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["raffle"]["id"].is_string());
    assert_eq!(body["raffle"]["total_boxes"], 100);
    assert_eq!(body["raffle"]["box_price"], 10.0);
    assert_eq!(body["raffle"]["status"], "active");

    cleanup_test_data(&pool, "seller@example.com").await;
}

#[actix_web::test]
async fn test_create_raffle_unauthorized() {
    let pool = create_test_pool().await;
    let app = create_test_app(pool.clone()).await;
    
    let raffle_data = json!({
        "item_id": Uuid::new_v4(),
        "total_boxes": 100,
        "box_price": 10.0,
        "total_winners": 1,
        "end_date": "2024-12-31T23:59:59Z"
    });

    let req = test::TestRequest::post()
        .uri("/api/raffles")
        .set_json(&raffle_data)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);
}

#[actix_web::test]
async fn test_get_raffle_success() {
    let pool = create_test_pool().await;
    let app = create_test_app(pool.clone()).await;
    
    // Create test data
    let seller = create_test_user(&pool, "seller@example.com", "password123").await;
    let item = create_test_item(&pool, &seller.id, "Test Item").await;
    let raffle = create_test_raffle(&pool, &seller.id, &item.id).await;

    let req = test::TestRequest::get()
        .uri(&format!("/api/raffles/{}", raffle.id))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["raffle"]["id"], raffle.id.to_string());
    assert_eq!(body["raffle"]["total_boxes"], raffle.total_boxes);

    cleanup_test_data(&pool, "seller@example.com").await;
}

#[actix_web::test]
async fn test_get_raffle_not_found() {
    let pool = create_test_pool().await;
    let app = create_test_app(pool.clone()).await;
    
    let non_existent_id = Uuid::new_v4();
    let req = test::TestRequest::get()
        .uri(&format!("/api/raffles/{}", non_existent_id))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["error"], "Raffle not found");
}

#[actix_web::test]
async fn test_buy_box_success() {
    let pool = create_test_pool().await;
    let app = create_test_app(pool.clone()).await;
    
    // Create test data
    let seller = create_test_user(&pool, "seller@example.com", "password123").await;
    let buyer = create_test_user(&pool, "buyer@example.com", "password123").await;
    let item = create_test_item(&pool, &seller.id, "Test Item").await;
    let raffle = create_test_raffle(&pool, &seller.id, &item.id).await;
    
    // Add credits to buyer
    add_credits_to_user(&pool, &buyer.id, 100.0).await;
    
    let tokens = get_auth_tokens(&pool, &buyer).await;

    let purchase_data = json!({
        "box_numbers": [1, 2, 3],
        "payment_method": "credits"
    });

    let req = test::TestRequest::post()
        .uri(&format!("/api/raffles/{}/buy-box", raffle.id))
        .insert_header(("Authorization", format!("Bearer {}", tokens.access_token)))
        .set_json(&purchase_data)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["participants"].as_array().unwrap().len(), 3);
    assert_eq!(body["total_cost"], 30.0);

    cleanup_test_data(&pool, "seller@example.com").await;
    cleanup_test_data(&pool, "buyer@example.com").await;
}

#[actix_web::test]
async fn test_buy_box_insufficient_credits() {
    let pool = create_test_pool().await;
    let app = create_test_app(pool.clone()).await;
    
    // Create test data
    let seller = create_test_user(&pool, "seller@example.com", "password123").await;
    let buyer = create_test_user(&pool, "buyer@example.com", "password123").await;
    let item = create_test_item(&pool, &seller.id, "Test Item").await;
    let raffle = create_test_raffle(&pool, &seller.id, &item.id).await;
    
    // Don't add enough credits to buyer
    add_credits_to_user(&pool, &buyer.id, 5.0).await;
    
    let tokens = get_auth_tokens(&pool, &buyer).await;

    let purchase_data = json!({
        "box_numbers": [1, 2, 3],
        "payment_method": "credits"
    });

    let req = test::TestRequest::post()
        .uri(&format!("/api/raffles/{}/buy-box", raffle.id))
        .insert_header(("Authorization", format!("Bearer {}", tokens.access_token)))
        .set_json(&purchase_data)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["error"].as_str().unwrap().contains("Insufficient credits"));

    cleanup_test_data(&pool, "seller@example.com").await;
    cleanup_test_data(&pool, "buyer@example.com").await;
}

#[actix_web::test]
async fn test_buy_box_already_sold() {
    let pool = create_test_pool().await;
    let app = create_test_app(pool.clone()).await;
    
    // Create test data
    let seller = create_test_user(&pool, "seller@example.com", "password123").await;
    let buyer1 = create_test_user(&pool, "buyer1@example.com", "password123").await;
    let buyer2 = create_test_user(&pool, "buyer2@example.com", "password123").await;
    let item = create_test_item(&pool, &seller.id, "Test Item").await;
    let raffle = create_test_raffle(&pool, &seller.id, &item.id).await;
    
    // Add credits to both buyers
    add_credits_to_user(&pool, &buyer1.id, 100.0).await;
    add_credits_to_user(&pool, &buyer2.id, 100.0).await;
    
    // First buyer purchases box 1
    let raffle_service = RaffleService::new(pool.clone());
    raffle_service.buy_boxes(&raffle.id, &buyer1.id, &[1]).await.unwrap();
    
    let tokens = get_auth_tokens(&pool, &buyer2).await;

    let purchase_data = json!({
        "box_numbers": [1], // Try to buy the same box
        "payment_method": "credits"
    });

    let req = test::TestRequest::post()
        .uri(&format!("/api/raffles/{}/buy-box", raffle.id))
        .insert_header(("Authorization", format!("Bearer {}", tokens.access_token)))
        .set_json(&purchase_data)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["error"].as_str().unwrap().contains("already sold"));

    cleanup_test_data(&pool, "seller@example.com").await;
    cleanup_test_data(&pool, "buyer1@example.com").await;
    cleanup_test_data(&pool, "buyer2@example.com").await;
}

#[actix_web::test]
async fn test_list_raffles_with_filters() {
    let pool = create_test_pool().await;
    let app = create_test_app(pool.clone()).await;
    
    // Create test data
    let seller = create_test_user(&pool, "seller@example.com", "password123").await;
    let item1 = create_test_item(&pool, &seller.id, "Electronics Item").await;
    let item2 = create_test_item(&pool, &seller.id, "Fashion Item").await;
    let raffle1 = create_test_raffle(&pool, &seller.id, &item1.id).await;
    let raffle2 = create_test_raffle(&pool, &seller.id, &item2.id).await;

    // Test without filters
    let req = test::TestRequest::get()
        .uri("/api/raffles")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["raffles"].as_array().unwrap().len() >= 2);

    // Test with status filter
    let req = test::TestRequest::get()
        .uri("/api/raffles?status=active")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    let raffles = body["raffles"].as_array().unwrap();
    for raffle in raffles {
        assert_eq!(raffle["status"], "active");
    }

    cleanup_test_data(&pool, "seller@example.com").await;
}

#[actix_web::test]
async fn test_raffle_completion_workflow() {
    let pool = create_test_pool().await;
    let app = create_test_app(pool.clone()).await;
    
    // Create test data with small raffle for easy completion
    let seller = create_test_user(&pool, "seller@example.com", "password123").await;
    let buyer = create_test_user(&pool, "buyer@example.com", "password123").await;
    let item = create_test_item(&pool, &seller.id, "Test Item").await;
    
    // Create raffle with only 2 boxes for easy testing
    let raffle = create_small_test_raffle(&pool, &seller.id, &item.id, 2).await;
    
    add_credits_to_user(&pool, &buyer.id, 100.0).await;
    let tokens = get_auth_tokens(&pool, &buyer).await;

    // Buy all boxes to complete the raffle
    let purchase_data = json!({
        "box_numbers": [1, 2],
        "payment_method": "credits"
    });

    let req = test::TestRequest::post()
        .uri(&format!("/api/raffles/{}/buy-box", raffle.id))
        .insert_header(("Authorization", format!("Bearer {}", tokens.access_token)))
        .set_json(&purchase_data)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    // Check that raffle status changed to full
    let req = test::TestRequest::get()
        .uri(&format!("/api/raffles/{}", raffle.id))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["raffle"]["status"], "full");

    cleanup_test_data(&pool, "seller@example.com").await;
    cleanup_test_data(&pool, "buyer@example.com").await;
}

// Helper functions
async fn create_test_pool() -> PgPool {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://test:test@localhost/raffle_platform_test".to_string());
    
    PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database")
}

async fn create_test_raffle(pool: &PgPool, seller_id: &Uuid, item_id: &Uuid) -> Raffle {
    sqlx::query_as!(
        Raffle,
        r#"
        INSERT INTO raffles (seller_id, item_id, total_boxes, box_price, total_winners, status, end_date)
        VALUES ($1, $2, 100, 10.0, 1, $3, NOW() + INTERVAL '30 days')
        RETURNING id, seller_id, item_id, total_boxes, box_price, total_winners, 
                 boxes_sold, status as "status: RaffleStatus", created_at, updated_at, end_date
        "#,
        seller_id,
        item_id,
        RaffleStatus::Active as RaffleStatus
    )
    .fetch_one(pool)
    .await
    .expect("Failed to create test raffle")
}

async fn create_small_test_raffle(pool: &PgPool, seller_id: &Uuid, item_id: &Uuid, total_boxes: i32) -> Raffle {
    sqlx::query_as!(
        Raffle,
        r#"
        INSERT INTO raffles (seller_id, item_id, total_boxes, box_price, total_winners, status, end_date)
        VALUES ($1, $2, $3, 10.0, 1, $4, NOW() + INTERVAL '30 days')
        RETURNING id, seller_id, item_id, total_boxes, box_price, total_winners, 
                 boxes_sold, status as "status: RaffleStatus", created_at, updated_at, end_date
        "#,
        seller_id,
        item_id,
        total_boxes,
        RaffleStatus::Active as RaffleStatus
    )
    .fetch_one(pool)
    .await
    .expect("Failed to create test raffle")
}

async fn add_credits_to_user(pool: &PgPool, user_id: &Uuid, amount: f64) {
    sqlx::query!(
        r#"
        INSERT INTO user_credits (user_id, amount, credit_type, expires_at)
        VALUES ($1, $2, 'general', NOW() + INTERVAL '30 days')
        "#,
        user_id,
        amount
    )
    .execute(pool)
    .await
    .expect("Failed to add credits to user");
}

async fn get_auth_tokens(pool: &PgPool, user: &User) -> AuthTokens {
    let auth_service = crate::services::auth_service::AuthService::new(pool.clone());
    auth_service.generate_tokens(user).await.unwrap()
}

struct AuthTokens {
    access_token: String,
    refresh_token: String,
}

#[cfg(test)]
mod test_helpers {
    use super::*;
    use actix_web::{web, App};
    use crate::handlers;
    use crate::middleware;

    pub async fn create_test_item(pool: &PgPool, seller_id: &Uuid, title: &str) -> Item {
        sqlx::query_as!(
            Item,
            r#"
            INSERT INTO items (seller_id, title, description, price, category, condition, image_urls)
            VALUES ($1, $2, 'Test description', 99.99, 'Electronics', $3, ARRAY['test.jpg'])
            RETURNING id, seller_id, title, description, price, category, 
                     condition as "condition: ItemCondition", image_urls, 
                     created_at, updated_at
            "#,
            seller_id,
            title,
            ItemCondition::New as ItemCondition
        )
        .fetch_one(pool)
        .await
        .expect("Failed to create test item")
    }
}