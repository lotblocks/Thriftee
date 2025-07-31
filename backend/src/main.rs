use actix_web::{web, App, HttpServer, Result};
use tracing::{info, Level};
use tracing_subscriber;
use std::sync::Arc;

mod config;
mod database;
mod error;
mod blockchain;
mod handlers;
mod middleware;
mod models;
mod services;
mod utils;

use config::AppConfig;
use database::Database;
use error::AppError;
use middleware::auth::AuthMiddleware;
use utils::jwt::JwtService;

#[actix_web::main]
async fn main() -> Result<(), AppError> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    // Load configuration
    let config = AppConfig::from_env()?;
    info!("Starting Raffle Platform Backend on {}:{}", config.host, config.port);

    // Initialize database
    let database = Database::new(&config.database_url).await?;
    
    // Run migrations
    database.migrate().await?;

    // Initialize JWT service
    let jwt_service = Arc::new(JwtService::new()?);

    // Initialize services
    let auth_service = services::AuthService::new(database.pool().clone(), jwt_service.clone());
    let wallet_service = services::WalletService::new(database.pool().clone());
    let credit_service = services::CreditService::new(database.pool().clone());
    let item_service = services::ItemService::with_realtime(database.pool().clone(), realtime_service.clone());
    let blockchain_service = services::BlockchainService::new(
        config.blockchain_rpc_url.clone(),
        config.blockchain_ws_url.clone(),
        config.contract_address.clone(),
        config.deployer_private_key.clone(),
    ).await?;
    let notification_service = services::NotificationService::new();
    let realtime_service = services::RealtimeService::new(database.pool().clone());
    let raffle_service = services::RaffleService::new(
        database.pool().clone(),
        credit_service.clone(),
        blockchain_service.clone(),
        notification_service.clone(),
        realtime_service.clone(),
    );
    let payment_service = services::PaymentService::new(
        config.stripe_secret_key.clone(),
        config.stripe_webhook_secret.clone(),
        database.pool().clone(),
        credit_service.clone(),
    );

    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(database.clone()))
            .app_data(web::Data::new(jwt_service.clone()))
            .app_data(web::Data::new(auth_service.clone()))
            .app_data(web::Data::new(wallet_service.clone()))
            .app_data(web::Data::new(credit_service.clone()))
            .app_data(web::Data::new(item_service.clone()))
            .app_data(web::Data::new(raffle_service.clone()))
            .app_data(web::Data::new(payment_service.clone()))
            .app_data(web::Data::new(std::sync::Arc::new(realtime_service.clone())))
            .service(
                web::scope("/api/v1")
                    .service(handlers::health::health_check)
                    .service(
                        web::scope("/auth")
                            .service(handlers::auth::register)
                            .service(handlers::auth::login)
                            .service(handlers::auth::refresh_token)
                            .service(handlers::auth::forgot_password)
                            .service(handlers::auth::reset_password)
                            .service(handlers::auth::verify_email)
                            // Protected auth endpoints
                            .service(
                                web::scope("")
                                    .wrap(AuthMiddleware::new(jwt_service.clone()))
                                    .service(handlers::auth::logout)
                                    .service(handlers::auth::get_current_user)
                                    .service(handlers::auth::update_current_user)
                                    .service(handlers::auth::change_password)
                            )
                    )
                    .service(
                        web::scope("/wallet")
                            .wrap(AuthMiddleware::new(jwt_service.clone()))
                            .service(handlers::wallet::get_wallet_address)
                            .service(handlers::wallet::sign_message)
                            .service(handlers::wallet::verify_signature)
                            .service(handlers::wallet::export_private_key)
                            .service(handlers::wallet::import_private_key)
                            .service(handlers::wallet::rotate_wallet_encryption)
                            .service(handlers::wallet::get_wallet_balance)
                    )
                    .service(
                        web::scope("/credits")
                            .wrap(AuthMiddleware::new(jwt_service.clone()))
                            .service(handlers::credits::get_credit_balance)
                            .service(handlers::credits::get_credit_history)
                            .service(handlers::credits::get_expiring_credits)
                            .service(handlers::credits::redeem_credits)
                            .service(handlers::credits::check_sufficient_credits)
                            .service(handlers::credits::get_free_items)
                            .service(handlers::credits::redeem_free_item)
                            // Admin endpoints
                            .service(handlers::credits::issue_credits)
                            .service(handlers::credits::issue_bonus_credits)
                            .service(handlers::credits::get_credit_statistics)
                            .service(handlers::credits::get_users_with_expiring_credits)
                            .service(handlers::credits::cleanup_expired_credits)
                            .service(handlers::credits::get_user_credit_balance)
                            // Health check
                            .service(handlers::credits::credit_service_health)
                    )
                    .service(
                        web::scope("/payments")
                            .wrap(AuthMiddleware::new(jwt_service.clone()))
                            // User endpoints
                            .route("/create-intent", web::post().to(handlers::payments::create_payment_intent))
                            .route("/confirm-intent/{payment_intent_id}", web::post().to(handlers::payments::confirm_payment_intent))
                            .route("/intent-status/{payment_intent_id}", web::get().to(handlers::payments::get_payment_intent_status))
                            .route("/history", web::get().to(handlers::payments::get_payment_history))
                            .route("/statistics", web::get().to(handlers::payments::get_payment_statistics))
                            
                            // Subscription endpoints
                            .route("/subscriptions", web::post().to(handlers::payments::create_subscription))
                            .route("/subscriptions", web::get().to(handlers::payments::get_user_subscriptions))
                            .route("/subscriptions/{subscription_id}/cancel", web::post().to(handlers::payments::cancel_subscription))
                            
                            // Admin endpoints
                            .route("/admin/analytics", web::get().to(handlers::payments::get_payment_analytics))
                            .route("/admin/user/{user_id}", web::get().to(handlers::payments::get_user_payment_details))
                            .route("/admin/refund/{payment_id}", web::post().to(handlers::payments::process_refund))
                            
                            // Health check
                            .route("/health", web::get().to(handlers::payments::payment_service_health))
                    )
                    .service(
                        web::scope("/items")
                            // Public endpoints
                            .route("", web::get().to(handlers::items::search_items))
                            .route("/popular", web::get().to(handlers::items::get_popular_items))
                            .route("/categories", web::get().to(handlers::items::get_item_categories))
                            .route("/{item_id}", web::get().to(handlers::items::get_item))
                            
                            // Protected endpoints
                            .service(
                                web::scope("")
                                    .wrap(AuthMiddleware::new(jwt_service.clone()))
                                    // Seller endpoints
                                    .route("", web::post().to(handlers::items::create_item))
                                    .route("/my-items", web::get().to(handlers::items::get_seller_items))
                                    .route("/{item_id}", web::put().to(handlers::items::update_item))
                                    .route("/{item_id}", web::delete().to(handlers::items::delete_item))
                                    .route("/{item_id}/status", web::put().to(handlers::items::update_item_status))
                                    .route("/{item_id}/stock", web::put().to(handlers::items::update_item_stock))
                                    .route("/{item_id}/analytics", web::get().to(handlers::items::get_item_analytics))
                                    .route("/bulk-operation", web::post().to(handlers::items::bulk_operation))
                                    .route("/statistics", web::get().to(handlers::items::get_item_statistics))
                                    
                                    // Admin endpoints
                                    .route("/admin/all", web::get().to(handlers::items::get_all_items_admin))
                                    .route("/admin/statistics", web::get().to(handlers::items::get_platform_item_statistics))
                                    
                                    // Health check
                                    .route("/health", web::get().to(handlers::items::item_service_health))
                            )
                    )
                    .service(
                        web::scope("/raffles")
                            // Public endpoints
                            .route("", web::get().to(handlers::raffles::search_raffles))
                            .route("/active", web::get().to(handlers::raffles::get_active_raffles))
                            .route("/featured", web::get().to(handlers::raffles::get_featured_raffles))
                            .route("/{raffle_id}", web::get().to(handlers::raffles::get_raffle))
                            .route("/{raffle_id}/grid", web::get().to(handlers::raffles::get_grid_state))
                            .route("/{raffle_id}/winners", web::get().to(handlers::raffles::get_raffle_winners))
                            
                            // Protected endpoints
                            .service(
                                web::scope("")
                                    .wrap(AuthMiddleware::new(jwt_service.clone()))
                                    // User endpoints
                                    .route("/{raffle_id}/buy-boxes", web::post().to(handlers::raffles::buy_boxes))
                                    .route("/{raffle_id}/my-purchases", web::get().to(handlers::raffles::get_user_purchases))
                                    .route("/my-history", web::get().to(handlers::raffles::get_user_purchase_history))
                                    
                                    // Seller endpoints
                                    .route("", web::post().to(handlers::raffles::create_raffle))
                                    .route("/{raffle_id}/cancel", web::post().to(handlers::raffles::cancel_raffle))
                                    .route("/statistics", web::get().to(handlers::raffles::get_raffle_statistics))
                                    
                                    // Admin endpoints
                                    .route("/admin/all", web::get().to(handlers::raffles::get_all_raffles_admin))
                                    .route("/admin/{raffle_id}/force-complete", web::post().to(handlers::raffles::force_complete_raffle))
                                    
                                    // Health check
                                    .route("/health", web::get().to(handlers::raffles::raffle_service_health))
                            )
                    )
                    // WebSocket endpoints (no auth required for connection, auth happens after connection)
                    .route("/ws", web::get().to(handlers::websocket::websocket_handler))
                    .route("/ws/stats", web::get().to(handlers::websocket::websocket_stats))
            )
            .service(
                web::scope("/webhooks")
                    .service(handlers::webhooks::stripe_webhook)
                    .service(handlers::webhooks::blockchain_webhook)
                    .service(handlers::webhooks::notification_webhook)
            )
            // Additional webhook endpoint for payments (no auth required)
            .route("/api/payments/webhook", web::post().to(handlers::payments::stripe_webhook))
    })
    .bind(format!("{}:{}", config.host, config.port))?
    .run()
    .await
    .map_err(AppError::from)
}