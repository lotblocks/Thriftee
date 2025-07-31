use actix_web::{test, web, App};
use serde_json::json;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::Semaphore;
use uuid::Uuid;
use futures::future::join_all;

use crate::models::{
    user::{User, UserRole},
    raffle::RaffleStatus,
    item::ItemCondition,
};
use crate::services::raffle_service::RaffleService;
use crate::utils::test_helpers::{create_test_app, create_test_user, cleanup_test_data};

/// Test concurrent box purchases to ensure data consistency
#[actix_web::test]
async fn test_concurrent_box_purchases() {
    let pool = create_test_pool().await;
    let app = Arc::new(create_test_app(pool.clone()).await);
    
    // Setup: Create seller, item, and raffle
    let seller = create_test_seller(&pool, "load_seller@example.com").await;
    let item = create_test_item(&pool, &seller.id, "Load Test Item").await;
    let raffle = create_test_raffle(&pool, &seller.id, &item.id, 100).await;

    // Create multiple concurrent users
    let num_users = 50;
    let mut users = Vec::new();
    let mut tokens = Vec::new();

    for i in 0..num_users {
        let email = format!("loaduser{}@example.com", i);
        let user = create_test_user(&pool, &email, "password123").await;
        add_credits_to_user(&pool, &user.id, 100.0).await;
        let auth_tokens = get_auth_tokens(&pool, &user).await;
        users.push(user);
        tokens.push(auth_tokens);
    }

    // Semaphore to control concurrency level
    let semaphore = Arc::new(Semaphore::new(20)); // Max 20 concurrent requests
    let mut handles = Vec::new();

    // Each user tries to buy 2 boxes concurrently
    for (i, token) in tokens.iter().enumerate() {
        let app_clone = app.clone();
        let semaphore_clone = semaphore.clone();
        let raffle_id = raffle.id;
        let access_token = token.access_token.clone();
        let box_numbers = vec![i * 2 + 1, i * 2 + 2]; // Each user gets unique boxes

        let handle = tokio::spawn(async move {
            let _permit = semaphore_clone.acquire().await.unwrap();
            
            let purchase_data = json!({
                "box_numbers": box_numbers,
                "payment_method": "credits"
            });

            let req = test::TestRequest::post()
                .uri(&format!("/api/raffles/{}/buy-box", raffle_id))
                .insert_header(("Authorization", format!("Bearer {}", access_token)))
                .set_json(&purchase_data)
                .to_request();

            let resp = test::call_service(&*app_clone, req).await;
            (resp.status(), i)
        });

        handles.push(handle);
    }

    // Wait for all requests to complete
    let results = join_all(handles).await;
    
    // Analyze results
    let mut successful_purchases = 0;
    let mut failed_purchases = 0;

    for result in results {
        let (status, user_index) = result.unwrap();
        if status == 200 {
            successful_purchases += 1;
        } else {
            failed_purchases += 1;
            println!("User {} failed with status: {}", user_index, status);
        }
    }

    println!("Successful purchases: {}", successful_purchases);
    println!("Failed purchases: {}", failed_purchases);

    // Verify data consistency
    let req = test::TestRequest::get()
        .uri(&format!("/api/raffles/{}", raffle.id))
        .to_request();

    let resp = test::call_service(&*app, req).await;
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    let boxes_sold = body["raffle"]["boxes_sold"].as_i64().unwrap();
    
    // Each successful purchase should have bought 2 boxes
    assert_eq!(boxes_sold, successful_purchases * 2);

    // Cleanup
    cleanup_test_data(&pool, "load_seller@example.com").await;
    for i in 0..num_users {
        cleanup_test_data(&pool, &format!("loaduser{}@example.com", i)).await;
    }
}

/// Test race condition handling when multiple users try to buy the same box
#[actix_web::test]
async fn test_race_condition_same_box() {
    let pool = create_test_pool().await;
    let app = Arc::new(create_test_app(pool.clone()).await);
    
    // Setup
    let seller = create_test_seller(&pool, "race_seller@example.com").await;
    let item = create_test_item(&pool, &seller.id, "Race Test Item").await;
    let raffle = create_test_raffle(&pool, &seller.id, &item.id, 10).await;

    // Create multiple users who will all try to buy the same box
    let num_users = 10;
    let mut tokens = Vec::new();

    for i in 0..num_users {
        let email = format!("raceuser{}@example.com", i);
        let user = create_test_user(&pool, &email, "password123").await;
        add_credits_to_user(&pool, &user.id, 50.0).await;
        let auth_tokens = get_auth_tokens(&pool, &user).await;
        tokens.push(auth_tokens);
    }

    let mut handles = Vec::new();

    // All users try to buy box #1 simultaneously
    for (i, token) in tokens.iter().enumerate() {
        let app_clone = app.clone();
        let raffle_id = raffle.id;
        let access_token = token.access_token.clone();

        let handle = tokio::spawn(async move {
            let purchase_data = json!({
                "box_numbers": [1], // Everyone tries to buy the same box
                "payment_method": "credits"
            });

            let req = test::TestRequest::post()
                .uri(&format!("/api/raffles/{}/buy-box", raffle_id))
                .insert_header(("Authorization", format!("Bearer {}", access_token)))
                .set_json(&purchase_data)
                .to_request();

            let resp = test::call_service(&*app_clone, req).await;
            (resp.status(), i)
        });

        handles.push(handle);
    }

    let results = join_all(handles).await;
    
    // Only one user should succeed, others should get conflict errors
    let mut successful = 0;
    let mut conflicts = 0;

    for result in results {
        let (status, _) = result.unwrap();
        match status {
            200 => successful += 1,
            400 | 409 => conflicts += 1, // Box already sold or conflict
            _ => panic!("Unexpected status code: {}", status),
        }
    }

    assert_eq!(successful, 1, "Exactly one user should succeed");
    assert_eq!(conflicts, num_users - 1, "All other users should get conflicts");

    // Verify only one box was sold
    let req = test::TestRequest::get()
        .uri(&format!("/api/raffles/{}", raffle.id))
        .to_request();

    let resp = test::call_service(&*app, req).await;
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["raffle"]["boxes_sold"].as_i64().unwrap(), 1);

    // Cleanup
    cleanup_test_data(&pool, "race_seller@example.com").await;
    for i in 0..num_users {
        cleanup_test_data(&pool, &format!("raceuser{}@example.com", i)).await;
    }
}

/// Test system performance under high load
#[actix_web::test]
async fn test_high_load_performance() {
    let pool = create_test_pool().await;
    let app = Arc::new(create_test_app(pool.clone()).await);
    
    // Setup multiple raffles
    let seller = create_test_seller(&pool, "perf_seller@example.com").await;
    let mut raffles = Vec::new();
    
    for i in 0..5 {
        let item = create_test_item(&pool, &seller.id, &format!("Perf Item {}", i)).await;
        let raffle = create_test_raffle(&pool, &seller.id, &item.id, 50).await;
        raffles.push(raffle);
    }

    // Create many users
    let num_users = 100;
    let mut tokens = Vec::new();

    for i in 0..num_users {
        let email = format!("perfuser{}@example.com", i);
        let user = create_test_user(&pool, &email, "password123").await;
        add_credits_to_user(&pool, &user.id, 200.0).await;
        let auth_tokens = get_auth_tokens(&pool, &user).await;
        tokens.push(auth_tokens);
    }

    let start_time = std::time::Instant::now();
    let semaphore = Arc::new(Semaphore::new(50)); // Higher concurrency
    let mut handles = Vec::new();

    // Each user makes multiple requests across different raffles
    for (i, token) in tokens.iter().enumerate() {
        let app_clone = app.clone();
        let semaphore_clone = semaphore.clone();
        let access_token = token.access_token.clone();
        let raffle_id = raffles[i % raffles.len()].id;
        let box_number = (i % 50) + 1; // Distribute across available boxes

        let handle = tokio::spawn(async move {
            let _permit = semaphore_clone.acquire().await.unwrap();
            
            let purchase_data = json!({
                "box_numbers": [box_number],
                "payment_method": "credits"
            });

            let req = test::TestRequest::post()
                .uri(&format!("/api/raffles/{}/buy-box", raffle_id))
                .insert_header(("Authorization", format!("Bearer {}", access_token)))
                .set_json(&purchase_data)
                .to_request();

            let request_start = std::time::Instant::now();
            let resp = test::call_service(&*app_clone, req).await;
            let request_duration = request_start.elapsed();

            (resp.status(), request_duration, i)
        });

        handles.push(handle);
    }

    let results = join_all(handles).await;
    let total_duration = start_time.elapsed();

    // Analyze performance metrics
    let mut successful_requests = 0;
    let mut total_response_time = std::time::Duration::new(0, 0);
    let mut max_response_time = std::time::Duration::new(0, 0);
    let mut min_response_time = std::time::Duration::from_secs(999);

    for result in results {
        let (status, duration, _) = result.unwrap();
        if status == 200 {
            successful_requests += 1;
            total_response_time += duration;
            max_response_time = max_response_time.max(duration);
            min_response_time = min_response_time.min(duration);
        }
    }

    let avg_response_time = total_response_time / successful_requests;
    let requests_per_second = num_users as f64 / total_duration.as_secs_f64();

    println!("Performance Metrics:");
    println!("Total requests: {}", num_users);
    println!("Successful requests: {}", successful_requests);
    println!("Total duration: {:?}", total_duration);
    println!("Requests per second: {:.2}", requests_per_second);
    println!("Average response time: {:?}", avg_response_time);
    println!("Min response time: {:?}", min_response_time);
    println!("Max response time: {:?}", max_response_time);

    // Performance assertions
    assert!(avg_response_time.as_millis() < 500, "Average response time should be under 500ms");
    assert!(max_response_time.as_millis() < 2000, "Max response time should be under 2s");
    assert!(requests_per_second > 10.0, "Should handle at least 10 requests per second");

    // Cleanup
    cleanup_test_data(&pool, "perf_seller@example.com").await;
    for i in 0..num_users {
        cleanup_test_data(&pool, &format!("perfuser{}@example.com", i)).await;
    }
}

/// Test database connection pool under load
#[actix_web::test]
async fn test_database_connection_pool_load() {
    let pool = create_test_pool().await;
    let app = Arc::new(create_test_app(pool.clone()).await);
    
    // Create a user for authentication
    let user = create_test_user(&pool, "dbload_user@example.com", "password123").await;
    let tokens = get_auth_tokens(&pool, &user).await;

    let num_requests = 200;
    let semaphore = Arc::new(Semaphore::new(100)); // High concurrency to stress the pool
    let mut handles = Vec::new();

    // Make many concurrent requests that hit the database
    for i in 0..num_requests {
        let app_clone = app.clone();
        let semaphore_clone = semaphore.clone();
        let access_token = tokens.access_token.clone();

        let handle = tokio::spawn(async move {
            let _permit = semaphore_clone.acquire().await.unwrap();
            
            // Mix of different endpoints to test various database operations
            let endpoint = match i % 4 {
                0 => "/api/raffles",
                1 => "/api/credits/balance",
                2 => "/api/users/participations",
                _ => "/api/auth/me",
            };

            let req = if endpoint == "/api/raffles" {
                test::TestRequest::get().uri(endpoint).to_request()
            } else {
                test::TestRequest::get()
                    .uri(endpoint)
                    .insert_header(("Authorization", format!("Bearer {}", access_token)))
                    .to_request()
            };

            let start = std::time::Instant::now();
            let resp = test::call_service(&*app_clone, req).await;
            let duration = start.elapsed();

            (resp.status(), duration)
        });

        handles.push(handle);
    }

    let results = join_all(handles).await;
    
    let mut successful = 0;
    let mut total_time = std::time::Duration::new(0, 0);

    for result in results {
        let (status, duration) = result.unwrap();
        if status.is_success() {
            successful += 1;
            total_time += duration;
        }
    }

    let avg_time = total_time / successful;
    let success_rate = (successful as f64 / num_requests as f64) * 100.0;

    println!("Database Load Test Results:");
    println!("Total requests: {}", num_requests);
    println!("Successful requests: {}", successful);
    println!("Success rate: {:.2}%", success_rate);
    println!("Average response time: {:?}", avg_time);

    // Assertions
    assert!(success_rate > 95.0, "Success rate should be above 95%");
    assert!(avg_time.as_millis() < 1000, "Average response time should be under 1s");

    cleanup_test_data(&pool, "dbload_user@example.com").await;
}

// Helper functions
async fn create_test_pool() -> PgPool {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://test:test@localhost/raffle_platform_test".to_string());
    
    PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database")
}

// Additional helper functions would be implemented here...
// (Similar to previous test files)

#[cfg(test)]
mod load_test_helpers {
    use super::*;
    // Load test specific helper implementations...
}