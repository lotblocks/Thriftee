use chrono::{DateTime, Utc};
use raffle_platform_shared::{TransactionType, Status};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;
use crate::error::AppError;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Transaction {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub seller_id: Option<Uuid>,
    pub amount: Decimal,
    pub transaction_type: TransactionType,
    pub status: String,
    pub payment_gateway_ref: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Transaction {
    /// Create a new transaction
    pub async fn create(
        pool: &PgPool,
        user_id: Option<Uuid>,
        seller_id: Option<Uuid>,
        amount: Decimal,
        transaction_type: TransactionType,
        payment_gateway_ref: Option<String>,
        metadata: Option<serde_json::Value>,
    ) -> Result<Self, AppError> {
        let transaction = sqlx::query_as!(
            Transaction,
            r#"
            INSERT INTO transactions (user_id, seller_id, amount, type, payment_gateway_ref, metadata)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING 
                id, user_id, seller_id, amount, 
                type as "transaction_type: TransactionType", 
                status, payment_gateway_ref, metadata, created_at, updated_at
            "#,
            user_id,
            seller_id,
            amount,
            transaction_type as TransactionType,
            payment_gateway_ref,
            metadata
        )
        .fetch_one(pool)
        .await?;

        Ok(transaction)
    }

    /// Find transaction by ID
    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Self>, AppError> {
        let transaction = sqlx::query_as!(
            Transaction,
            r#"
            SELECT 
                id, user_id, seller_id, amount, 
                type as "transaction_type: TransactionType", 
                status, payment_gateway_ref, metadata, created_at, updated_at
            FROM transactions 
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(pool)
        .await?;

        Ok(transaction)
    }

    /// Find transactions by user
    pub async fn find_by_user(
        pool: &PgPool,
        user_id: Uuid,
        limit: i64,
        offset: i64,
        transaction_type: Option<TransactionType>,
    ) -> Result<Vec<Self>, AppError> {
        let transactions = match transaction_type {
            Some(tx_type) => {
                sqlx::query_as!(
                    Transaction,
                    r#"
                    SELECT 
                        id, user_id, seller_id, amount, 
                        type as "transaction_type: TransactionType", 
                        status, payment_gateway_ref, metadata, created_at, updated_at
                    FROM transactions 
                    WHERE user_id = $1 AND type = $2
                    ORDER BY created_at DESC
                    LIMIT $3 OFFSET $4
                    "#,
                    user_id,
                    tx_type as TransactionType,
                    limit,
                    offset
                )
                .fetch_all(pool)
                .await?
            }
            None => {
                sqlx::query_as!(
                    Transaction,
                    r#"
                    SELECT 
                        id, user_id, seller_id, amount, 
                        type as "transaction_type: TransactionType", 
                        status, payment_gateway_ref, metadata, created_at, updated_at
                    FROM transactions 
                    WHERE user_id = $1
                    ORDER BY created_at DESC
                    LIMIT $2 OFFSET $3
                    "#,
                    user_id,
                    limit,
                    offset
                )
                .fetch_all(pool)
                .await?
            }
        };

        Ok(transactions)
    }

    /// Find transactions by seller
    pub async fn find_by_seller(
        pool: &PgPool,
        seller_id: Uuid,
        limit: i64,
        offset: i64,
        transaction_type: Option<TransactionType>,
    ) -> Result<Vec<Self>, AppError> {
        let transactions = match transaction_type {
            Some(tx_type) => {
                sqlx::query_as!(
                    Transaction,
                    r#"
                    SELECT 
                        id, user_id, seller_id, amount, 
                        type as "transaction_type: TransactionType", 
                        status, payment_gateway_ref, metadata, created_at, updated_at
                    FROM transactions 
                    WHERE seller_id = $1 AND type = $2
                    ORDER BY created_at DESC
                    LIMIT $3 OFFSET $4
                    "#,
                    seller_id,
                    tx_type as TransactionType,
                    limit,
                    offset
                )
                .fetch_all(pool)
                .await?
            }
            None => {
                sqlx::query_as!(
                    Transaction,
                    r#"
                    SELECT 
                        id, user_id, seller_id, amount, 
                        type as "transaction_type: TransactionType", 
                        status, payment_gateway_ref, metadata, created_at, updated_at
                    FROM transactions 
                    WHERE seller_id = $1
                    ORDER BY created_at DESC
                    LIMIT $2 OFFSET $3
                    "#,
                    seller_id,
                    limit,
                    offset
                )
                .fetch_all(pool)
                .await?
            }
        };

        Ok(transactions)
    }

    /// Update transaction status
    pub async fn update_status(
        pool: &PgPool,
        id: Uuid,
        status: Status,
        payment_gateway_ref: Option<String>,
    ) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            UPDATE transactions 
            SET status = $1, payment_gateway_ref = COALESCE($2, payment_gateway_ref), updated_at = NOW() 
            WHERE id = $3
            "#,
            status.to_string(),
            payment_gateway_ref,
            id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Find pending transactions
    pub async fn find_pending(
        pool: &PgPool,
        limit: i64,
        older_than_minutes: i32,
    ) -> Result<Vec<Self>, AppError> {
        let transactions = sqlx::query_as!(
            Transaction,
            r#"
            SELECT 
                id, user_id, seller_id, amount, 
                type as "transaction_type: TransactionType", 
                status, payment_gateway_ref, metadata, created_at, updated_at
            FROM transactions 
            WHERE status = 'pending' 
            AND created_at < NOW() - INTERVAL '%d minutes'
            ORDER BY created_at ASC
            LIMIT $1
            "#,
            limit,
            older_than_minutes
        )
        .fetch_all(pool)
        .await?;

        Ok(transactions)
    }

    /// Calculate total revenue for a date range
    pub async fn calculate_revenue(
        pool: &PgPool,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
        transaction_types: Option<Vec<TransactionType>>,
    ) -> Result<Decimal, AppError> {
        let revenue = match transaction_types {
            Some(types) => {
                let type_strings: Vec<String> = types.iter().map(|t| t.to_string()).collect();
                sqlx::query_scalar!(
                    r#"
                    SELECT COALESCE(SUM(amount), 0) 
                    FROM transactions 
                    WHERE status = 'completed' 
                    AND created_at >= $1 AND created_at <= $2
                    AND type = ANY($3)
                    "#,
                    start_date,
                    end_date,
                    &type_strings
                )
                .fetch_one(pool)
                .await?
            }
            None => {
                sqlx::query_scalar!(
                    r#"
                    SELECT COALESCE(SUM(amount), 0) 
                    FROM transactions 
                    WHERE status = 'completed' 
                    AND created_at >= $1 AND created_at <= $2
                    "#,
                    start_date,
                    end_date
                )
                .fetch_one(pool)
                .await?
            }
        };

        Ok(revenue.unwrap_or(Decimal::ZERO))
    }

    /// Create credit deposit transaction
    pub async fn create_credit_deposit(
        pool: &PgPool,
        user_id: Uuid,
        amount: Decimal,
        payment_gateway_ref: String,
        metadata: Option<serde_json::Value>,
    ) -> Result<Self, AppError> {
        Self::create(
            pool,
            Some(user_id),
            None,
            amount,
            TransactionType::CreditDeposit,
            Some(payment_gateway_ref),
            metadata,
        ).await
    }

    /// Create box purchase transaction
    pub async fn create_box_purchase(
        pool: &PgPool,
        user_id: Uuid,
        amount: Decimal,
        raffle_id: Uuid,
        box_number: i32,
    ) -> Result<Self, AppError> {
        let metadata = serde_json::json!({
            "raffle_id": raffle_id,
            "box_number": box_number
        });

        Self::create(
            pool,
            Some(user_id),
            None,
            -amount, // Negative amount for deduction
            TransactionType::BoxPurchaseCreditDeduction,
            None,
            Some(metadata),
        ).await
    }

    /// Create seller payout transaction
    pub async fn create_seller_payout(
        pool: &PgPool,
        seller_id: Uuid,
        amount: Decimal,
        payment_gateway_ref: Option<String>,
        metadata: Option<serde_json::Value>,
    ) -> Result<Self, AppError> {
        Self::create(
            pool,
            None,
            Some(seller_id),
            amount,
            TransactionType::Payout,
            payment_gateway_ref,
            metadata,
        ).await
    }

    /// Create seller fee transaction
    pub async fn create_seller_fee(
        pool: &PgPool,
        seller_id: Uuid,
        amount: Decimal,
        fee_type: TransactionType,
        metadata: Option<serde_json::Value>,
    ) -> Result<Self, AppError> {
        // Validate fee type
        match fee_type {
            TransactionType::SellerSubscriptionFee 
            | TransactionType::SellerListingFee 
            | TransactionType::SellerTransactionFee => {},
            _ => return Err(AppError::Validation("Invalid seller fee type".to_string())),
        }

        Self::create(
            pool,
            None,
            Some(seller_id),
            -amount, // Negative amount for fee deduction
            fee_type,
            None,
            metadata,
        ).await
    }

    /// Get transaction summary for user
    pub async fn get_user_summary(
        pool: &PgPool,
        user_id: Uuid,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<TransactionSummary, AppError> {
        let summary = sqlx::query!(
            r#"
            SELECT 
                COALESCE(SUM(CASE WHEN amount > 0 THEN amount ELSE 0 END), 0) as total_credits_added,
                COALESCE(SUM(CASE WHEN amount < 0 THEN ABS(amount) ELSE 0 END), 0) as total_credits_spent,
                COUNT(*) as total_transactions,
                COUNT(CASE WHEN type = 'box_purchase_credit_deduction' THEN 1 END) as box_purchases
            FROM transactions 
            WHERE user_id = $1 
            AND created_at >= $2 AND created_at <= $3
            AND status = 'completed'
            "#,
            user_id,
            start_date,
            end_date
        )
        .fetch_one(pool)
        .await?;

        Ok(TransactionSummary {
            total_credits_added: summary.total_credits_added.unwrap_or(Decimal::ZERO),
            total_credits_spent: summary.total_credits_spent.unwrap_or(Decimal::ZERO),
            total_transactions: summary.total_transactions.unwrap_or(0),
            box_purchases: summary.box_purchases.unwrap_or(0),
        })
    }

    /// Check if transaction is completed
    pub fn is_completed(&self) -> bool {
        self.status == "completed"
    }

    /// Check if transaction is pending
    pub fn is_pending(&self) -> bool {
        self.status == "pending"
    }

    /// Check if transaction failed
    pub fn is_failed(&self) -> bool {
        self.status == "failed"
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionSummary {
    pub total_credits_added: Decimal,
    pub total_credits_spent: Decimal,
    pub total_transactions: i64,
    pub box_purchases: i64,
}

impl TransactionType {
    pub fn to_string(&self) -> String {
        match self {
            TransactionType::CreditDeposit => "credit_deposit".to_string(),
            TransactionType::CreditWithdrawal => "credit_withdrawal".to_string(),
            TransactionType::BoxPurchaseCreditDeduction => "box_purchase_credit_deduction".to_string(),
            TransactionType::ItemPurchaseCreditDeduction => "item_purchase_credit_deduction".to_string(),
            TransactionType::RaffleWinCreditAddition => "raffle_win_credit_addition".to_string(),
            TransactionType::Payout => "payout".to_string(),
            TransactionType::FreeItemRedemption => "free_item_redemption".to_string(),
            TransactionType::SellerSubscriptionFee => "seller_subscription_fee".to_string(),
            TransactionType::SellerListingFee => "seller_listing_fee".to_string(),
            TransactionType::SellerTransactionFee => "seller_transaction_fee".to_string(),
        }
    }
}