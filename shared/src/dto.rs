use crate::types::*;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

// User DTOs
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateUserRequest {
    #[validate(length(min = 3, max = 50))]
    pub username: String,
    
    #[validate(email)]
    pub email: String,
    
    #[validate(length(min = 8))]
    pub password: String,
    
    #[validate(phone)]
    pub phone_number: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub role: UserRole,
    pub credit_balance: Decimal,
    pub internal_wallet_address: String,
    pub phone_number: Option<String>,
    pub is_active: bool,
    pub email_verified: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email)]
    pub email: String,
    
    #[validate(length(min = 1))]
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub user: UserResponse,
    pub expires_in: i64,
}

// Item DTOs
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateItemRequest {
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    
    #[validate(length(max = 5000))]
    pub description: Option<String>,
    
    #[validate(length(min = 1))]
    pub images: Vec<String>,
    
    #[validate(range(min = 0.01))]
    pub retail_price: Decimal,
    
    #[validate(range(min = 0.0))]
    pub cost_of_goods: Decimal,
    
    #[validate(range(min = 1))]
    pub stock_quantity: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ItemResponse {
    pub id: Uuid,
    pub seller_id: Option<Uuid>,
    pub name: String,
    pub description: Option<String>,
    pub images: Vec<String>,
    pub retail_price: Decimal,
    pub cost_of_goods: Decimal,
    pub status: ItemStatus,
    pub stock_quantity: i32,
    pub listing_fee_applied: Option<Decimal>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Raffle DTOs
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateRaffleRequest {
    pub item_id: Uuid,
    
    #[validate(range(min = 1, max = 10000))]
    pub total_boxes: i32,
    
    #[validate(range(min = 0.01))]
    pub box_price: Decimal,
    
    #[validate(range(min = 1))]
    pub total_winners: i32,
    
    #[validate(range(min = 1, max = 100))]
    pub grid_rows: i32,
    
    #[validate(range(min = 1, max = 100))]
    pub grid_cols: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RaffleResponse {
    pub id: Uuid,
    pub item_id: Uuid,
    pub item: Option<ItemResponse>,
    pub total_boxes: i32,
    pub box_price: Decimal,
    pub boxes_sold: i32,
    pub total_winners: i32,
    pub status: RaffleStatus,
    pub winner_user_ids: Vec<Uuid>,
    pub grid_rows: i32,
    pub grid_cols: i32,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct PurchaseBoxRequest {
    pub raffle_id: Uuid,
    
    #[validate(range(min = 1))]
    pub box_number: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BoxPurchaseResponse {
    pub id: Uuid,
    pub raffle_id: Uuid,
    pub user_id: Uuid,
    pub box_number: i32,
    pub purchase_price_in_credits: Decimal,
    pub blockchain_tx_hash: Option<String>,
    pub created_at: DateTime<Utc>,
}

// Credit DTOs
#[derive(Debug, Serialize, Deserialize)]
pub struct CreditResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub amount: Decimal,
    pub source: CreditSource,
    pub credit_type: CreditType,
    pub redeemable_on_item_id: Option<Uuid>,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_transferable: bool,
    pub is_used: bool,
    pub used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct RedeemCreditRequest {
    pub credit_ids: Vec<Uuid>,
    pub item_id: Option<Uuid>,
    
    #[validate(range(min = 0.01))]
    pub amount: Decimal,
}

// Payment DTOs
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreatePaymentIntentRequest {
    #[validate(range(min = 1.0))]
    pub amount: Decimal,
    
    pub currency: String,
    pub payment_method_types: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaymentIntentResponse {
    pub client_secret: String,
    pub amount: Decimal,
    pub currency: String,
    pub status: String,
}

// Seller DTOs
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateSellerRequest {
    #[validate(length(min = 1, max = 255))]
    pub company_name: Option<String>,
    
    #[validate(length(max = 2000))]
    pub description: Option<String>,
    
    pub subscription_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SellerResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub company_name: Option<String>,
    pub description: Option<String>,
    pub current_subscription_id: Option<Uuid>,
    pub subscription_expires_at: Option<DateTime<Utc>>,
    pub is_verified: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Analytics DTOs
#[derive(Debug, Serialize, Deserialize)]
pub struct RaffleMetricsResponse {
    pub raffle_id: Uuid,
    pub views_count: i32,
    pub unique_viewers: i32,
    pub conversion_rate: Option<Decimal>,
    pub average_boxes_per_user: Option<Decimal>,
    pub time_to_completion_minutes: Option<i32>,
    pub peak_concurrent_users: i32,
    pub total_revenue: Option<Decimal>,
    pub platform_fee: Option<Decimal>,
    pub seller_payout: Option<Decimal>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlatformStatsResponse {
    pub date: chrono::NaiveDate,
    pub total_users: i32,
    pub new_users: i32,
    pub active_users: i32,
    pub total_sellers: i32,
    pub new_sellers: i32,
    pub active_sellers: i32,
    pub total_raffles: i32,
    pub completed_raffles: i32,
    pub total_revenue: Decimal,
    pub total_credits_issued: Decimal,
    pub total_credits_redeemed: Decimal,
    pub average_raffle_completion_time: Option<i32>,
}

// Common pagination and filtering
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct PaginationParams {
    #[validate(range(min = 1, max = 100))]
    pub limit: Option<i64>,
    
    #[validate(range(min = 0))]
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
    pub has_more: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FilterParams {
    pub status: Option<String>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
    pub search: Option<String>,
}