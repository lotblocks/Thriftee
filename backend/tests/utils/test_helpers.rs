use actix_web::{test, web, App, HttpServer};
use sqlx::PgPool;
use uuid::Uuid;
use std::sync::Arc;

use crate::handlers;
use crate::middleware;
use crate::models::{
    user::{User, UserRole},
    item::{Item, ItemCondition},
    raffle::{Raffle, RaffleStatus},
};
use crate::services::auth_service::AuthService;

/// Create a test application instance with all routes configured
pub async fn create_test_app(
    pool: PgPool,
) -> impl actix_web::dev::Service<
    actix_web::dev::ServiceRequest,
    Response = actix_web::dev::ServiceResponse,
    Error = actix_web::Error,
> {
    test::init_service(
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(create_test_config()))
            .service(
                web::scope("/api")
                    .service(
                        web::scope("/auth")
                            .route("/register", web::post().to(handlers::auth::register))
                            .route("/login", web::post().to(handlers::auth::login))
                            .route("/refresh", web::post().to(handlers::auth::refresh_token))
                            .route("/logout", web::post().to(handlers::auth::logout))
                            .route("/me", web::get().to(handlers::auth::get_current_user))
                    )
                    .service(
                        web::scope("/raffles")
                            .route("", web::get().to(handlers::raffles::list_raffles))
                            .route("", web::post().to(handlers::raffles::create_raffle))
                            .route("/{id}", web::get().to(handlers::raffles::get_raffle))
                            .route("/{id}", web::put().to(handlers::raffles::update_raffle))
                            .route("/{id}/buy-box", web::post().to(handlers::raffles::buy_box))
                    )
                    .service(
                        web::scope("/items")
                            .route("", web::get().to(handlers::items::list_items))
                            .route("", web::post().to(handlers::items::create_item))
                            .route("/{id}", web::get().to(handlers::items::get_item))
                            .route("/{id}", web::put().to(handlers::items::update_item))
                            .route("/{id}", web::delete().to(handlers::items::delete_item))
                    )
                    .service(
                        web::scope("/credits")
                            .route("/balance", web::get().to(handlers::credits::get_balance))
                            .route("/history", web::get().to(handlers::credits::get_history))
                            .route("/redeem", web::post().to(handlers::credits::redeem_credits))
                    )
                    .service(
                        web::scope("/payments")
                            .route("/create-intent", web::post().to(handlers::payments::create_payment_intent))
                            .route("/webhook", web::post().to(handlers::payments::handle_webhook))
                            .route("/history", web::get().to(handlers::payments::get_payment_history))
                    )
                    .service(
                        web::scope("/users")
                            .route("/participations", web::get().to(handlers::auth::get_user_participations))
                            .route("/profile", web::get().to(handlers::auth::get_user_profile))
                            .route("/profile", web::put().to(handlers::auth::update_user_profile))
                    )
                    .service(
                        web::scope("/sellers")
                            .route("/raffles", web::get().to(handlers::raffles::get_seller_raffles))
                            .route("/analytics", web::get().to(handlers::raffles::get_seller_analytics))
                    )
                    .service(
                        web::scope("/admin")
                            .route("/users", web::get().to(handlers::auth::admin_list_users))
                            .route("/raffles", web::get().to(handlers::raffles::admin_list_raffles))
                            .route("/analytics", web::get().to(handlers::raffles::admin_get_analytics))
                    )
                    .wrap(middleware::auth::AuthMiddleware::new())
                    .wrap(middleware::cors::cors_middleware())
                    .wrap(middleware::rate_limit::RateLimitMiddleware::new())
    ).await
}

/// Create a test user with default role
pub async fn create_test_user(pool: &PgPool, email: &str, password: &str) -> User {
    create_test_user_with_role(pool, email, password, UserRole::User).await
}

/// Create a test user with specific role
pub async fn create_test_user_with_role(
    pool: &PgPool, 
    email: &str, 
    password: &str,
    role: UserRole
) -> User {
    let password_hash = bcrypt::hash(password, bcrypt::DEFAULT_COST).unwrap();
    
    sqlx::query_as!(
        User,
        r#"
        INSERT INTO users (email, password_hash, role, is_verified, wallet_address)
        VALUES ($1, $2, $3, true, $4)
        RETURNING id, email, password_hash, role as "role: UserRole", 
                 is_verified, wallet_address, created_at, updated_at
        "#,
        email,
        password_hash,
        role as UserRole,
        Some(format!("0x{:040x}", rand::random::<u64>()))
    )
    .fetch_one(pool)
    .await
    .expect("Failed to create test user")
}

/// Create a test seller
pub async fn create_test_seller(pool: &PgPool, email: &str) -> User {
    create_test_user_with_role(pool, email, "password123", UserRole::Seller).await
}

/// Create a test admin
pub async fn create_test_admin(pool: &PgPool, email: &str) -> User {
    create_test_user_with_role(pool, email, "password123", UserRole::Admin).await
}

/// Create a test item
pub async fn create_test_item(pool: &PgPool, seller_id: &Uuid, title: &str) -> Item {
    sqlx::query_as!(
        Item,
        r#"
        INSERT INTO items (seller_id, title, description, price, category, condition, image_urls)
        VALUES ($1, $2, 'Test description for item', 99.99, 'Electronics', $3, ARRAY['test1.jpg', 'test2.jpg'])
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

/// Create a test raffle
pub async fn create_test_raffle(
    pool: &PgPool, 
    seller_id: &Uuid, 
    item_id: &Uuid,
    total_boxes: i32
) -> Raffle {
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

/// Add credits to a user account
pub async fn add_credits_to_user(pool: &PgPool, user_id: &Uuid, amount: f64) {
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

/// Get authentication tokens for a user
pub async fn get_auth_tokens(pool: &PgPool, user: &User) -> AuthTokens {
    let auth_service = AuthService::new(pool.clone());
    let tokens = auth_service.generate_tokens(user).await.unwrap();
    AuthTokens {
        access_token: tokens.access_token,
        refresh_token: tokens.refresh_token,
    }
}

/// Clean up test data for a user
pub async fn cleanup_test_data(pool: &PgPool, email: &str) {
    // Delete in order to respect foreign key constraints
    sqlx::query!(
        r#"
        DELETE FROM blockchain_events 
        WHERE raffle_id IN (
            SELECT r.id FROM raffles r 
            JOIN items i ON r.item_id = i.id 
            JOIN users u ON i.seller_id = u.id 
            WHERE u.email = $1
        )
        "#,
        email
    )
    .execute(pool)
    .await
    .unwrap_or_else(|e| {
        eprintln!("Warning: Failed to cleanup blockchain_events: {}", e);
    });

    sqlx::query!(
        r#"
        DELETE FROM raffle_participants 
        WHERE raffle_id IN (
            SELECT r.id FROM raffles r 
            JOIN items i ON r.item_id = i.id 
            JOIN users u ON i.seller_id = u.id 
            WHERE u.email = $1
        ) OR user_id IN (SELECT id FROM users WHERE email = $1)
        "#,
        email
    )
    .execute(pool)
    .await
    .unwrap_or_else(|e| {
        eprintln!("Warning: Failed to cleanup raffle_participants: {}", e);
    });

    sqlx::query!(
        r#"
        DELETE FROM user_credits 
        WHERE user_id IN (SELECT id FROM users WHERE email = $1)
        "#,
        email
    )
    .execute(pool)
    .await
    .unwrap_or_else(|e| {
        eprintln!("Warning: Failed to cleanup user_credits: {}", e);
    });

    sqlx::query!(
        r#"
        DELETE FROM raffles 
        WHERE item_id IN (
            SELECT i.id FROM items i 
            JOIN users u ON i.seller_id = u.id 
            WHERE u.email = $1
        )
        "#,
        email
    )
    .execute(pool)
    .await
    .unwrap_or_else(|e| {
        eprintln!("Warning: Failed to cleanup raffles: {}", e);
    });

    sqlx::query!(
        r#"
        DELETE FROM items 
        WHERE seller_id IN (SELECT id FROM users WHERE email = $1)
        "#,
        email
    )
    .execute(pool)
    .await
    .unwrap_or_else(|e| {
        eprintln!("Warning: Failed to cleanup items: {}", e);
    });

    sqlx::query!("DELETE FROM users WHERE email = $1", email)
        .execute(pool)
        .await
        .unwrap_or_else(|e| {
            eprintln!("Warning: Failed to cleanup user: {}", e);
        });
}

/// Create test configuration
pub fn create_test_config() -> crate::config::AppConfig {
    crate::config::AppConfig {
        database_url: std::env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://test:test@localhost/raffle_platform_test".to_string()),
        jwt_secret: "test_jwt_secret_for_testing_only".to_string(),
        stripe_secret_key: "sk_test_123456789".to_string(),
        stripe_webhook_secret: "whsec_test_123456789".to_string(),
        blockchain_rpc_url: "http://localhost:8545".to_string(),
        chainlink_vrf_coordinator: "0x1234567890123456789012345678901234567890".to_string(),
        chainlink_key_hash: "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890".to_string(),
        chainlink_fee: 100000000000000000u64, // 0.1 LINK
        server_host: "127.0.0.1".to_string(),
        server_port: 8080,
        cors_allowed_origins: vec!["http://localhost:3000".to_string()],
        rate_limit_requests_per_minute: 100,
        log_level: "debug".to_string(),
    }
}

/// Wait for async operations to complete
pub async fn wait_for_async_operations() {
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
}

/// Generate a unique test email
pub fn generate_test_email(prefix: &str) -> String {
    format!("{}+{}@test.example.com", prefix, Uuid::new_v4())
}

/// Create multiple test users for load testing
pub async fn create_multiple_test_users(
    pool: &PgPool, 
    count: usize, 
    prefix: &str
) -> Vec<(User, AuthTokens)> {
    let mut users = Vec::new();
    
    for i in 0..count {
        let email = format!("{}{}@test.example.com", prefix, i);
        let user = create_test_user(pool, &email, "password123").await;
        let tokens = get_auth_tokens(pool, &user).await;
        users.push((user, tokens));
    }
    
    users
}

/// Clean up multiple test users
pub async fn cleanup_multiple_test_users(pool: &PgPool, prefix: &str, count: usize) {
    for i in 0..count {
        let email = format!("{}{}@test.example.com", prefix, i);
        cleanup_test_data(pool, &email).await;
    }
}

/// Verify database consistency after operations
pub async fn verify_database_consistency(pool: &PgPool) -> Result<(), String> {
    // Check that all raffle participants have valid user and raffle references
    let orphaned_participants = sqlx::query!(
        r#"
        SELECT COUNT(*) as count FROM raffle_participants rp
        LEFT JOIN users u ON rp.user_id = u.id
        LEFT JOIN raffles r ON rp.raffle_id = r.id
        WHERE u.id IS NULL OR r.id IS NULL
        "#
    )
    .fetch_one(pool)
    .await
    .map_err(|e| format!("Failed to check orphaned participants: {}", e))?;

    if orphaned_participants.count.unwrap_or(0) > 0 {
        return Err("Found orphaned raffle participants".to_string());
    }

    // Check that raffle boxes_sold matches actual participants
    let inconsistent_raffles = sqlx::query!(
        r#"
        SELECT r.id, r.boxes_sold, COUNT(rp.id) as actual_participants
        FROM raffles r
        LEFT JOIN raffle_participants rp ON r.id = rp.raffle_id
        GROUP BY r.id, r.boxes_sold
        HAVING r.boxes_sold != COUNT(rp.id)
        "#
    )
    .fetch_all(pool)
    .await
    .map_err(|e| format!("Failed to check raffle consistency: {}", e))?;

    if !inconsistent_raffles.is_empty() {
        return Err(format!("Found {} raffles with inconsistent box counts", inconsistent_raffles.len()));
    }

    // Check that user credit balances are non-negative
    let negative_balances = sqlx::query!(
        r#"
        SELECT COUNT(*) as count FROM (
            SELECT user_id, SUM(amount) as balance
            FROM user_credits
            WHERE expires_at > NOW()
            GROUP BY user_id
            HAVING SUM(amount) < 0
        ) as negative_users
        "#
    )
    .fetch_one(pool)
    .await
    .map_err(|e| format!("Failed to check credit balances: {}", e))?;

    if negative_balances.count.unwrap_or(0) > 0 {
        return Err("Found users with negative credit balances".to_string());
    }

    Ok(())
}

/// Authentication tokens for testing
#[derive(Debug, Clone)]
pub struct AuthTokens {
    pub access_token: String,
    pub refresh_token: String,
}

/// Test performance metrics
#[derive(Debug, Default)]
pub struct PerformanceMetrics {
    pub total_requests: usize,
    pub successful_requests: usize,
    pub failed_requests: usize,
    pub average_response_time: std::time::Duration,
    pub min_response_time: std::time::Duration,
    pub max_response_time: std::time::Duration,
    pub requests_per_second: f64,
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        Self {
            min_response_time: std::time::Duration::from_secs(999),
            ..Default::default()
        }
    }

    pub fn add_request(&mut self, duration: std::time::Duration, success: bool) {
        self.total_requests += 1;
        if success {
            self.successful_requests += 1;
        } else {
            self.failed_requests += 1;
        }
        
        self.min_response_time = self.min_response_time.min(duration);
        self.max_response_time = self.max_response_time.max(duration);
    }

    pub fn calculate_averages(&mut self, total_duration: std::time::Duration) {
        if self.successful_requests > 0 {
            self.requests_per_second = self.total_requests as f64 / total_duration.as_secs_f64();
        }
    }

    pub fn print_summary(&self) {
        println!("Performance Metrics:");
        println!("  Total requests: {}", self.total_requests);
        println!("  Successful: {}", self.successful_requests);
        println!("  Failed: {}", self.failed_requests);
        println!("  Success rate: {:.2}%", 
                (self.successful_requests as f64 / self.total_requests as f64) * 100.0);
        println!("  Requests/sec: {:.2}", self.requests_per_second);
        println!("  Avg response time: {:?}", self.average_response_time);
        println!("  Min response time: {:?}", self.min_response_time);
        println!("  Max response time: {:?}", self.max_response_time);
    }
}