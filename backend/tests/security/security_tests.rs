use actix_web::{test, web, App};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::user::{User, UserRole};
use crate::utils::test_helpers::{create_test_app, create_test_user, cleanup_test_data};

/// Test SQL injection attempts
#[actix_web::test]
async fn test_sql_injection_protection() {
    let pool = create_test_pool().await;
    let app = create_test_app(pool.clone()).await;
    
    // Test SQL injection in login endpoint
    let malicious_login = json!({
        "email": "admin@example.com'; DROP TABLE users; --",
        "password": "password"
    });

    let req = test::TestRequest::post()
        .uri("/api/auth/login")
        .set_json(&malicious_login)
        .to_request();

    let resp = test::call_service(&app, req).await;
    // Should return 401 (invalid credentials) not 500 (server error)
    assert_eq!(resp.status(), 401);

    // Verify users table still exists by creating a user
    let user = create_test_user(&pool, "test@example.com", "password123").await;
    assert!(!user.id.to_string().is_empty());

    cleanup_test_data(&pool, "test@example.com").await;
}

/// Test XSS protection in input fields
#[actix_web::test]
async fn test_xss_protection() {
    let pool = create_test_pool().await;
    let app = create_test_app(pool.clone()).await;
    
    let user = create_test_user(&pool, "xss_user@example.com", "password123").await;
    let tokens = get_auth_tokens(&pool, &user).await;

    // Test XSS in item creation
    let malicious_item = json!({
        "title": "<script>alert('XSS')</script>",
        "description": "<img src=x onerror=alert('XSS')>",
        "price": 99.99,
        "category": "Electronics",
        "condition": "new",
        "image_urls": ["test.jpg"]
    });

    let req = test::TestRequest::post()
        .uri("/api/items")
        .insert_header(("Authorization", format!("Bearer {}", tokens.access_token)))
        .set_json(&malicious_item)
        .to_request();

    let resp = test::call_service(&app, req).await;
    
    if resp.status() == 201 {
        let body: serde_json::Value = test::read_body_json(resp).await;
        let title = body["item"]["title"].as_str().unwrap();
        let description = body["item"]["description"].as_str().unwrap();
        
        // Verify that script tags are escaped or removed
        assert!(!title.contains("<script>"));
        assert!(!description.contains("<img src=x onerror="));
    }

    cleanup_test_data(&pool, "xss_user@example.com").await;
}

/// Test authentication bypass attempts
#[actix_web::test]
async fn test_authentication_bypass_attempts() {
    let pool = create_test_pool().await;
    let app = create_test_app(pool.clone()).await;
    
    // Test 1: No token
    let req = test::TestRequest::get()
        .uri("/api/auth/me")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);

    // Test 2: Invalid token format
    let req = test::TestRequest::get()
        .uri("/api/auth/me")
        .insert_header(("Authorization", "Bearer invalid_token"))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);

    // Test 3: Malformed Authorization header
    let req = test::TestRequest::get()
        .uri("/api/auth/me")
        .insert_header(("Authorization", "NotBearer token"))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);

    // Test 4: Empty Authorization header
    let req = test::TestRequest::get()
        .uri("/api/auth/me")
        .insert_header(("Authorization", ""))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);
}

/// Test authorization and role-based access control
#[actix_web::test]
async fn test_authorization_controls() {
    let pool = create_test_pool().await;
    let app = create_test_app(pool.clone()).await;
    
    // Create users with different roles
    let regular_user = create_test_user(&pool, "user@example.com", "password123").await;
    let seller_user = create_test_seller(&pool, "seller@example.com").await;
    let admin_user = create_test_admin(&pool, "admin@example.com").await;

    let user_tokens = get_auth_tokens(&pool, &regular_user).await;
    let seller_tokens = get_auth_tokens(&pool, &seller_user).await;
    let admin_tokens = get_auth_tokens(&pool, &admin_user).await;

    // Test 1: Regular user trying to access seller-only endpoint
    let item_data = json!({
        "title": "Test Item",
        "description": "Test description",
        "price": 99.99,
        "category": "Electronics",
        "condition": "new",
        "image_urls": ["test.jpg"]
    });

    let req = test::TestRequest::post()
        .uri("/api/items")
        .insert_header(("Authorization", format!("Bearer {}", user_tokens.access_token)))
        .set_json(&item_data)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 403); // Forbidden

    // Test 2: Seller can access seller endpoints
    let req = test::TestRequest::post()
        .uri("/api/items")
        .insert_header(("Authorization", format!("Bearer {}", seller_tokens.access_token)))
        .set_json(&item_data)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201); // Created

    // Test 3: Regular user trying to access admin endpoint
    let req = test::TestRequest::get()
        .uri("/api/admin/users")
        .insert_header(("Authorization", format!("Bearer {}", user_tokens.access_token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 403); // Forbidden

    // Test 4: Admin can access admin endpoints
    let req = test::TestRequest::get()
        .uri("/api/admin/users")
        .insert_header(("Authorization", format!("Bearer {}", admin_tokens.access_token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200); // OK

    cleanup_test_data(&pool, "user@example.com").await;
    cleanup_test_data(&pool, "seller@example.com").await;
    cleanup_test_data(&pool, "admin@example.com").await;
}

/// Test input validation and sanitization
#[actix_web::test]
async fn test_input_validation() {
    let pool = create_test_pool().await;
    let app = create_test_app(pool.clone()).await;
    
    // Test 1: Invalid email format in registration
    let invalid_registration = json!({
        "email": "not_an_email",
        "password": "ValidPassword123!",
        "confirm_password": "ValidPassword123!"
    });

    let req = test::TestRequest::post()
        .uri("/api/auth/register")
        .set_json(&invalid_registration)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);

    // Test 2: Password too short
    let weak_password = json!({
        "email": "test@example.com",
        "password": "123",
        "confirm_password": "123"
    });

    let req = test::TestRequest::post()
        .uri("/api/auth/register")
        .set_json(&weak_password)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);

    // Test 3: Negative price in item creation
    let user = create_test_seller(&pool, "validation_seller@example.com").await;
    let tokens = get_auth_tokens(&pool, &user).await;

    let invalid_item = json!({
        "title": "Test Item",
        "description": "Test description",
        "price": -99.99, // Negative price
        "category": "Electronics",
        "condition": "new",
        "image_urls": ["test.jpg"]
    });

    let req = test::TestRequest::post()
        .uri("/api/items")
        .insert_header(("Authorization", format!("Bearer {}", tokens.access_token)))
        .set_json(&invalid_item)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);

    // Test 4: Invalid UUID format
    let req = test::TestRequest::get()
        .uri("/api/raffles/not-a-uuid")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);

    cleanup_test_data(&pool, "validation_seller@example.com").await;
}

/// Test rate limiting protection
#[actix_web::test]
async fn test_rate_limiting() {
    let pool = create_test_pool().await;
    let app = create_test_app(pool.clone()).await;
    
    let login_data = json!({
        "email": "nonexistent@example.com",
        "password": "wrongpassword"
    });

    let mut rate_limited = false;
    
    // Make multiple rapid requests
    for i in 0..20 {
        let req = test::TestRequest::post()
            .uri("/api/auth/login")
            .set_json(&login_data)
            .to_request();

        let resp = test::call_service(&app, req).await;
        
        if resp.status() == 429 {
            rate_limited = true;
            break;
        }
        
        // Small delay to avoid overwhelming the test
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }

    assert!(rate_limited, "Rate limiting should be triggered");
}

/// Test CSRF protection
#[actix_web::test]
async fn test_csrf_protection() {
    let pool = create_test_pool().await;
    let app = create_test_app(pool.clone()).await;
    
    let user = create_test_user(&pool, "csrf_user@example.com", "password123").await;
    let tokens = get_auth_tokens(&pool, &user).await;

    // Test state-changing operation without proper CSRF token
    let purchase_data = json!({
        "box_numbers": [1],
        "payment_method": "credits"
    });

    let req = test::TestRequest::post()
        .uri(&format!("/api/raffles/{}/buy-box", Uuid::new_v4()))
        .insert_header(("Authorization", format!("Bearer {}", tokens.access_token)))
        .insert_header(("Origin", "https://malicious-site.com"))
        .set_json(&purchase_data)
        .to_request();

    let resp = test::call_service(&app, req).await;
    // Should reject requests from unauthorized origins
    assert!(resp.status().is_client_error());

    cleanup_test_data(&pool, "csrf_user@example.com").await;
}

/// Test session security
#[actix_web::test]
async fn test_session_security() {
    let pool = create_test_pool().await;
    let app = create_test_app(pool.clone()).await;
    
    let user = create_test_user(&pool, "session_user@example.com", "password123").await;
    let tokens = get_auth_tokens(&pool, &user).await;

    // Test 1: Valid token works
    let req = test::TestRequest::get()
        .uri("/api/auth/me")
        .insert_header(("Authorization", format!("Bearer {}", tokens.access_token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    // Test 2: Logout invalidates token
    let req = test::TestRequest::post()
        .uri("/api/auth/logout")
        .insert_header(("Authorization", format!("Bearer {}", tokens.access_token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    // Test 3: Token should be invalid after logout
    let req = test::TestRequest::get()
        .uri("/api/auth/me")
        .insert_header(("Authorization", format!("Bearer {}", tokens.access_token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);

    cleanup_test_data(&pool, "session_user@example.com").await;
}

/// Test data exposure prevention
#[actix_web::test]
async fn test_data_exposure_prevention() {
    let pool = create_test_pool().await;
    let app = create_test_app(pool.clone()).await;
    
    let user = create_test_user(&pool, "exposure_user@example.com", "password123").await;
    let tokens = get_auth_tokens(&pool, &user).await;

    // Test that sensitive data is not exposed in API responses
    let req = test::TestRequest::get()
        .uri("/api/auth/me")
        .insert_header(("Authorization", format!("Bearer {}", tokens.access_token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    
    // Verify sensitive fields are not exposed
    assert!(body["user"]["password_hash"].is_null());
    assert!(body["user"]["password"].is_null());
    
    // Verify expected fields are present
    assert!(body["user"]["id"].is_string());
    assert!(body["user"]["email"].is_string());

    cleanup_test_data(&pool, "exposure_user@example.com").await;
}

/// Test file upload security
#[actix_web::test]
async fn test_file_upload_security() {
    let pool = create_test_pool().await;
    let app = create_test_app(pool.clone()).await;
    
    let user = create_test_seller(&pool, "upload_seller@example.com").await;
    let tokens = get_auth_tokens(&pool, &user).await;

    // Test 1: Malicious file extension
    let malicious_item = json!({
        "title": "Test Item",
        "description": "Test description",
        "price": 99.99,
        "category": "Electronics",
        "condition": "new",
        "image_urls": ["malicious.php", "script.js", "virus.exe"]
    });

    let req = test::TestRequest::post()
        .uri("/api/items")
        .insert_header(("Authorization", format!("Bearer {}", tokens.access_token)))
        .set_json(&malicious_item)
        .to_request();

    let resp = test::call_service(&app, req).await;
    
    if resp.status() == 201 {
        let body: serde_json::Value = test::read_body_json(resp).await;
        let image_urls = body["item"]["image_urls"].as_array().unwrap();
        
        // Verify that only safe image extensions are allowed
        for url in image_urls {
            let url_str = url.as_str().unwrap();
            assert!(
                url_str.ends_with(".jpg") || 
                url_str.ends_with(".jpeg") || 
                url_str.ends_with(".png") || 
                url_str.ends_with(".gif") ||
                url_str.ends_with(".webp"),
                "Only safe image extensions should be allowed"
            );
        }
    } else {
        // Should reject malicious file extensions
        assert_eq!(resp.status(), 400);
    }

    cleanup_test_data(&pool, "upload_seller@example.com").await;
}

// Helper functions
async fn create_test_pool() -> PgPool {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://test:test@localhost/raffle_platform_test".to_string());
    
    PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database")
}

async fn create_test_seller(pool: &PgPool, email: &str) -> User {
    let password_hash = bcrypt::hash("password123", bcrypt::DEFAULT_COST).unwrap();
    
    sqlx::query_as!(
        User,
        r#"
        INSERT INTO users (email, password_hash, role, is_verified)
        VALUES ($1, $2, $3, true)
        RETURNING id, email, password_hash, role as "role: UserRole", 
                 is_verified, created_at, updated_at
        "#,
        email,
        password_hash,
        UserRole::Seller as UserRole
    )
    .fetch_one(pool)
    .await
    .expect("Failed to create test seller")
}

async fn create_test_admin(pool: &PgPool, email: &str) -> User {
    let password_hash = bcrypt::hash("password123", bcrypt::DEFAULT_COST).unwrap();
    
    sqlx::query_as!(
        User,
        r#"
        INSERT INTO users (email, password_hash, role, is_verified)
        VALUES ($1, $2, $3, true)
        RETURNING id, email, password_hash, role as "role: UserRole", 
                 is_verified, created_at, updated_at
        "#,
        email,
        password_hash,
        UserRole::Admin as UserRole
    )
    .fetch_one(pool)
    .await
    .expect("Failed to create test admin")
}

struct AuthTokens {
    access_token: String,
    refresh_token: String,
}

async fn get_auth_tokens(pool: &PgPool, user: &User) -> AuthTokens {
    let auth_service = crate::services::auth_service::AuthService::new(pool.clone());
    let tokens = auth_service.generate_tokens(user).await.unwrap();
    AuthTokens {
        access_token: tokens.access_token,
        refresh_token: tokens.refresh_token,
    }
}