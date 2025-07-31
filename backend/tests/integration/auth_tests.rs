use actix_web::{test, web, App};
use serde_json::json;
use sqlx::PgPool;
use std::sync::Arc;

use crate::handlers::auth::{login, register, refresh_token, logout};
use crate::middleware::auth::AuthMiddleware;
use crate::models::user::{User, UserRole};
use crate::services::auth_service::AuthService;
use crate::utils::test_helpers::{create_test_app, create_test_user, cleanup_test_data};

#[actix_web::test]
async fn test_user_registration_success() {
    let pool = create_test_pool().await;
    let app = create_test_app(pool.clone()).await;
    
    let registration_data = json!({
        "email": "test@example.com",
        "password": "SecurePassword123!",
        "confirm_password": "SecurePassword123!"
    });

    let req = test::TestRequest::post()
        .uri("/api/auth/register")
        .set_json(&registration_data)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["user"]["id"].is_string());
    assert_eq!(body["user"]["email"], "test@example.com");
    assert!(body["access_token"].is_string());
    assert!(body["refresh_token"].is_string());

    cleanup_test_data(&pool, "test@example.com").await;
}

#[actix_web::test]
async fn test_user_registration_duplicate_email() {
    let pool = create_test_pool().await;
    let app = create_test_app(pool.clone()).await;
    
    // Create a user first
    create_test_user(&pool, "existing@example.com", "password123").await;

    let registration_data = json!({
        "email": "existing@example.com",
        "password": "SecurePassword123!",
        "confirm_password": "SecurePassword123!"
    });

    let req = test::TestRequest::post()
        .uri("/api/auth/register")
        .set_json(&registration_data)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 409);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["error"], "Email already exists");

    cleanup_test_data(&pool, "existing@example.com").await;
}

#[actix_web::test]
async fn test_user_registration_invalid_password() {
    let pool = create_test_pool().await;
    let app = create_test_app(pool.clone()).await;
    
    let registration_data = json!({
        "email": "test@example.com",
        "password": "weak",
        "confirm_password": "weak"
    });

    let req = test::TestRequest::post()
        .uri("/api/auth/register")
        .set_json(&registration_data)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["error"].as_str().unwrap().contains("Password"));
}

#[actix_web::test]
async fn test_user_login_success() {
    let pool = create_test_pool().await;
    let app = create_test_app(pool.clone()).await;
    
    // Create a test user
    let user = create_test_user(&pool, "login@example.com", "SecurePassword123!").await;

    let login_data = json!({
        "email": "login@example.com",
        "password": "SecurePassword123!"
    });

    let req = test::TestRequest::post()
        .uri("/api/auth/login")
        .set_json(&login_data)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["user"]["id"], user.id.to_string());
    assert_eq!(body["user"]["email"], "login@example.com");
    assert!(body["access_token"].is_string());
    assert!(body["refresh_token"].is_string());

    cleanup_test_data(&pool, "login@example.com").await;
}

#[actix_web::test]
async fn test_user_login_invalid_credentials() {
    let pool = create_test_pool().await;
    let app = create_test_app(pool.clone()).await;
    
    let login_data = json!({
        "email": "nonexistent@example.com",
        "password": "wrongpassword"
    });

    let req = test::TestRequest::post()
        .uri("/api/auth/login")
        .set_json(&login_data)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["error"], "Invalid credentials");
}

#[actix_web::test]
async fn test_token_refresh_success() {
    let pool = create_test_pool().await;
    let app = create_test_app(pool.clone()).await;
    
    // Create user and get tokens
    let user = create_test_user(&pool, "refresh@example.com", "SecurePassword123!").await;
    let auth_service = AuthService::new(pool.clone());
    let tokens = auth_service.generate_tokens(&user).await.unwrap();

    let refresh_data = json!({
        "refresh_token": tokens.refresh_token
    });

    let req = test::TestRequest::post()
        .uri("/api/auth/refresh")
        .set_json(&refresh_data)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["access_token"].is_string());
    assert!(body["refresh_token"].is_string());

    cleanup_test_data(&pool, "refresh@example.com").await;
}

#[actix_web::test]
async fn test_token_refresh_invalid_token() {
    let pool = create_test_pool().await;
    let app = create_test_app(pool.clone()).await;
    
    let refresh_data = json!({
        "refresh_token": "invalid_token"
    });

    let req = test::TestRequest::post()
        .uri("/api/auth/refresh")
        .set_json(&refresh_data)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["error"], "Invalid refresh token");
}

#[actix_web::test]
async fn test_logout_success() {
    let pool = create_test_pool().await;
    let app = create_test_app(pool.clone()).await;
    
    // Create user and get tokens
    let user = create_test_user(&pool, "logout@example.com", "SecurePassword123!").await;
    let auth_service = AuthService::new(pool.clone());
    let tokens = auth_service.generate_tokens(&user).await.unwrap();

    let req = test::TestRequest::post()
        .uri("/api/auth/logout")
        .insert_header(("Authorization", format!("Bearer {}", tokens.access_token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["message"], "Logged out successfully");

    cleanup_test_data(&pool, "logout@example.com").await;
}

#[actix_web::test]
async fn test_protected_route_without_token() {
    let pool = create_test_pool().await;
    let app = create_test_app(pool.clone()).await;
    
    let req = test::TestRequest::get()
        .uri("/api/auth/me")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["error"], "Missing authorization token");
}

#[actix_web::test]
async fn test_protected_route_with_valid_token() {
    let pool = create_test_pool().await;
    let app = create_test_app(pool.clone()).await;
    
    // Create user and get tokens
    let user = create_test_user(&pool, "protected@example.com", "SecurePassword123!").await;
    let auth_service = AuthService::new(pool.clone());
    let tokens = auth_service.generate_tokens(&user).await.unwrap();

    let req = test::TestRequest::get()
        .uri("/api/auth/me")
        .insert_header(("Authorization", format!("Bearer {}", tokens.access_token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["user"]["id"], user.id.to_string());
    assert_eq!(body["user"]["email"], "protected@example.com");

    cleanup_test_data(&pool, "protected@example.com").await;
}

#[actix_web::test]
async fn test_rate_limiting() {
    let pool = create_test_pool().await;
    let app = create_test_app(pool.clone()).await;
    
    let login_data = json!({
        "email": "nonexistent@example.com",
        "password": "wrongpassword"
    });

    // Make multiple requests to trigger rate limiting
    for i in 0..10 {
        let req = test::TestRequest::post()
            .uri("/api/auth/login")
            .set_json(&login_data)
            .to_request();

        let resp = test::call_service(&app, req).await;
        
        if i < 5 {
            assert_eq!(resp.status(), 401); // Invalid credentials
        } else {
            assert_eq!(resp.status(), 429); // Rate limited
        }
    }
}

// Helper functions
async fn create_test_pool() -> PgPool {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://test:test@localhost/raffle_platform_test".to_string());
    
    PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database")
}

#[cfg(test)]
mod test_helpers {
    use super::*;
    use actix_web::{web, App, HttpServer};
    use crate::handlers;
    use crate::middleware;

    pub async fn create_test_app(pool: PgPool) -> impl actix_web::dev::Service<
        actix_web::dev::ServiceRequest,
        Response = actix_web::dev::ServiceResponse,
        Error = actix_web::Error,
    > {
        test::init_service(
            App::new()
                .app_data(web::Data::new(pool))
                .service(
                    web::scope("/api/auth")
                        .route("/register", web::post().to(handlers::auth::register))
                        .route("/login", web::post().to(handlers::auth::login))
                        .route("/refresh", web::post().to(handlers::auth::refresh_token))
                        .route("/logout", web::post().to(handlers::auth::logout))
                        .route("/me", web::get().to(handlers::auth::get_current_user))
                        .wrap(middleware::auth::AuthMiddleware::new())
                )
        ).await
    }

    pub async fn create_test_user(pool: &PgPool, email: &str, password: &str) -> User {
        let password_hash = bcrypt::hash(password, bcrypt::DEFAULT_COST).unwrap();
        
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
            UserRole::User as UserRole
        )
        .fetch_one(pool)
        .await
        .expect("Failed to create test user")
    }

    pub async fn cleanup_test_data(pool: &PgPool, email: &str) {
        sqlx::query!("DELETE FROM users WHERE email = $1", email)
            .execute(pool)
            .await
            .expect("Failed to cleanup test data");
    }
}