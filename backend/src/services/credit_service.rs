use crate::models::credit::UserCredit;
use crate::models::item::Item;
use crate::models::user::User;
use crate::error::AppError;
use chrono::{DateTime, Duration, Utc};
use raffle_platform_shared::{CreditSource, CreditType, CreditResponse};
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::collections::HashMap;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

#[cfg(test)]
mod tests;

/// Credit management service handles all credit-related operations
#[derive(Clone)]
pub struct CreditService {
    db_pool: PgPool,
}

#[derive(Debug, Clone)]
pub struct CreditBalance {
    pub total_general: Decimal,
    pub total_item_specific: Decimal,
    pub total_available: Decimal,
    pub expiring_soon: Decimal,
    pub expired: Decimal,
}

#[derive(Debug, Clone)]
pub struct CreditRedemptionRequest {
    pub user_id: Uuid,
    pub amount: Decimal,
    pub item_id: Option<Uuid>,
    pub credit_type: Option<CreditType>,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct CreditRedemptionResult {
    pub used_credits: Vec<UserCredit>,
    pub total_amount_used: Decimal,
    pub remaining_balance: Decimal,
}

#[derive(Debug, Clone)]
pub struct CreditIssuanceRequest {
    pub user_id: Uuid,
    pub amount: Decimal,
    pub source: CreditSource,
    pub credit_type: CreditType,
    pub redeemable_on_item_id: Option<Uuid>,
    pub expires_at: Option<DateTime<Utc>>,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct ExpirationNotification {
    pub user_id: Uuid,
    pub credits: Vec<UserCredit>,
    pub total_amount: Decimal,
    pub days_until_expiry: i64,
}

#[derive(Debug, Clone)]
pub struct CreditStatistics {
    pub total_credits_issued: Decimal,
    pub total_credits_used: Decimal,
    pub total_credits_expired: Decimal,
    pub active_users_with_credits: i64,
    pub credits_by_source: HashMap<CreditSource, Decimal>,
    pub credits_by_type: HashMap<CreditType, Decimal>,
}

impl CreditService {
    /// Create a new credit service
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }

    /// Issue credits to a user
    pub async fn issue_credits(
        &self,
        request: CreditIssuanceRequest,
    ) -> Result<UserCredit, AppError> {
        // Validate user exists
        let user = User::find_by_id(&self.db_pool, request.user_id).await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        // Validate item if specified
        if let Some(item_id) = request.redeemable_on_item_id {
            Item::find_by_id(&self.db_pool, item_id).await?
                .ok_or_else(|| AppError::NotFound("Item not found".to_string()))?;
        }

        // Validate amount is positive
        if request.amount <= Decimal::ZERO {
            return Err(AppError::Validation("Credit amount must be positive".to_string()));
        }

        // Create the credit
        let credit = UserCredit::create(
            &self.db_pool,
            request.user_id,
            request.amount,
            request.source,
            request.credit_type,
            request.redeemable_on_item_id,
            request.expires_at,
        ).await?;

        // Log the credit issuance
        self.log_credit_transaction(
            request.user_id,
            request.amount,
            "credit_issued",
            &request.description,
            Some(credit.id),
        ).await?;

        info!(
            "Issued {} credits to user {} (source: {:?}, type: {:?})",
            request.amount, request.user_id, request.source, request.credit_type
        );

        Ok(credit)
    }

    /// Issue credits for raffle loss (when user doesn't win)
    pub async fn issue_raffle_loss_credits(
        &self,
        user_id: Uuid,
        raffle_id: Uuid,
        amount: Decimal,
        item_id: Option<Uuid>,
    ) -> Result<UserCredit, AppError> {
        let expires_at = Some(Utc::now() + Duration::days(365)); // 1 year expiry

        let request = CreditIssuanceRequest {
            user_id,
            amount,
            source: CreditSource::RaffleLoss,
            credit_type: if item_id.is_some() { CreditType::ItemSpecific } else { CreditType::General },
            redeemable_on_item_id: item_id,
            expires_at,
            description: format!("Credits for raffle loss (raffle_id: {})", raffle_id),
        };

        self.issue_credits(request).await
    }

    /// Issue bonus credits (promotional or admin-issued)
    pub async fn issue_bonus_credits(
        &self,
        user_id: Uuid,
        amount: Decimal,
        expires_at: Option<DateTime<Utc>>,
        description: String,
    ) -> Result<UserCredit, AppError> {
        let request = CreditIssuanceRequest {
            user_id,
            amount,
            source: CreditSource::Bonus,
            credit_type: CreditType::General,
            redeemable_on_item_id: None,
            expires_at,
            description,
        };

        self.issue_credits(request).await
    }

    /// Issue refund credits
    pub async fn issue_refund_credits(
        &self,
        user_id: Uuid,
        amount: Decimal,
        item_id: Option<Uuid>,
        description: String,
    ) -> Result<UserCredit, AppError> {
        let expires_at = Some(Utc::now() + Duration::days(90)); // 90 days for refunds

        let request = CreditIssuanceRequest {
            user_id,
            amount,
            source: CreditSource::Refund,
            credit_type: if item_id.is_some() { CreditType::ItemSpecific } else { CreditType::General },
            redeemable_on_item_id: item_id,
            expires_at,
            description,
        };

        self.issue_credits(request).await
    }

    /// Redeem credits for a purchase
    pub async fn redeem_credits(
        &self,
        request: CreditRedemptionRequest,
    ) -> Result<CreditRedemptionResult, AppError> {
        // Validate user exists
        User::find_by_id(&self.db_pool, request.user_id).await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        // Validate amount is positive
        if request.amount <= Decimal::ZERO {
            return Err(AppError::Validation("Redemption amount must be positive".to_string()));
        }

        // Find available credits
        let available_credits = UserCredit::find_available_by_user(
            &self.db_pool,
            request.user_id,
            request.credit_type,
            request.item_id,
        ).await?;

        // Check if sufficient credits are available
        let total_available: Decimal = available_credits.iter().map(|c| c.amount).sum();
        if total_available < request.amount {
            return Err(AppError::Validation(format!(
                "Insufficient credits. Available: {}, Required: {}",
                total_available, request.amount
            )));
        }

        // Select credits to use (FIFO - first expiring first)
        let mut credits_to_use = Vec::new();
        let mut remaining_amount = request.amount;

        for credit in available_credits {
            if remaining_amount <= Decimal::ZERO {
                break;
            }

            if credit.can_be_used_for_item(request.item_id) {
                credits_to_use.push(credit.id);
                remaining_amount -= credit.amount.min(remaining_amount);
            }
        }

        // Use the credits
        let used_credits = UserCredit::mark_as_used(
            &self.db_pool,
            &credits_to_use,
            request.amount,
        ).await?;

        let total_amount_used: Decimal = used_credits.iter().map(|c| c.amount).sum();

        // Calculate remaining balance
        let remaining_balance = UserCredit::calculate_total_available(
            &self.db_pool,
            request.user_id,
            request.credit_type,
            request.item_id,
        ).await?;

        // Log the redemption
        self.log_credit_transaction(
            request.user_id,
            total_amount_used,
            "credit_redeemed",
            &request.description,
            None,
        ).await?;

        info!(
            "Redeemed {} credits for user {} (item_id: {:?})",
            total_amount_used, request.user_id, request.item_id
        );

        Ok(CreditRedemptionResult {
            used_credits,
            total_amount_used,
            remaining_balance,
        })
    }

    /// Get user's credit balance
    pub async fn get_user_balance(&self, user_id: Uuid) -> Result<CreditBalance, AppError> {
        // Get all available credits
        let available_credits = UserCredit::find_available_by_user(
            &self.db_pool,
            user_id,
            None,
            None,
        ).await?;

        let mut total_general = Decimal::ZERO;
        let mut total_item_specific = Decimal::ZERO;
        let mut expiring_soon = Decimal::ZERO;

        let now = Utc::now();
        let expiry_threshold = now + Duration::days(30); // 30 days

        for credit in &available_credits {
            match credit.credit_type {
                CreditType::General => total_general += credit.amount,
                CreditType::ItemSpecific => total_item_specific += credit.amount,
            }

            // Check if expiring soon
            if let Some(expires_at) = credit.expires_at {
                if expires_at <= expiry_threshold {
                    expiring_soon += credit.amount;
                }
            }
        }

        let total_available = total_general + total_item_specific;

        // Get expired credits
        let expired = self.get_expired_credits_amount(user_id).await?;

        Ok(CreditBalance {
            total_general,
            total_item_specific,
            total_available,
            expiring_soon,
            expired,
        })
    }

    /// Get user's credit history
    pub async fn get_user_credit_history(
        &self,
        user_id: Uuid,
        include_used: bool,
        limit: Option<i64>,
    ) -> Result<Vec<CreditResponse>, AppError> {
        let mut credits = UserCredit::find_by_user(&self.db_pool, user_id, include_used).await?;

        if let Some(limit) = limit {
            credits.truncate(limit as usize);
        }

        Ok(credits.into_iter().map(|c| c.to_response()).collect())
    }

    /// Get expiring credits for a user
    pub async fn get_expiring_credits(
        &self,
        user_id: Uuid,
        days: i64,
    ) -> Result<Vec<CreditResponse>, AppError> {
        let credits = UserCredit::find_expiring(&self.db_pool, user_id, days).await?;
        Ok(credits.into_iter().map(|c| c.to_response()).collect())
    }

    /// Get users with expiring credits (for notifications)
    pub async fn get_users_with_expiring_credits(
        &self,
        days: i64,
    ) -> Result<Vec<ExpirationNotification>, AppError> {
        let expiring_credits = sqlx::query_as!(
            UserCredit,
            r#"
            SELECT 
                id, user_id, amount, 
                source as "source: CreditSource", 
                credit_type as "credit_type: CreditType",
                redeemable_on_item_id, expires_at, is_transferable, is_used, used_at, created_at
            FROM user_credits 
            WHERE is_used = false 
            AND expires_at IS NOT NULL 
            AND expires_at <= NOW() + INTERVAL '%d days'
            AND expires_at > NOW()
            ORDER BY user_id, expires_at ASC
            "#,
            days
        )
        .fetch_all(&self.db_pool)
        .await?;

        // Group by user
        let mut user_credits: HashMap<Uuid, Vec<UserCredit>> = HashMap::new();
        for credit in expiring_credits {
            user_credits.entry(credit.user_id).or_default().push(credit);
        }

        let mut notifications = Vec::new();
        for (user_id, credits) in user_credits {
            let total_amount: Decimal = credits.iter().map(|c| c.amount).sum();
            
            // Find the earliest expiry date
            let earliest_expiry = credits.iter()
                .filter_map(|c| c.expires_at)
                .min()
                .unwrap_or_else(|| Utc::now() + Duration::days(days));

            let days_until_expiry = (earliest_expiry - Utc::now()).num_days();

            notifications.push(ExpirationNotification {
                user_id,
                credits,
                total_amount,
                days_until_expiry,
            });
        }

        Ok(notifications)
    }

    /// Clean up expired credits
    pub async fn cleanup_expired_credits(&self) -> Result<u64, AppError> {
        let deleted_count = UserCredit::cleanup_expired(&self.db_pool).await?;
        
        if deleted_count > 0 {
            info!("Cleaned up {} expired credits", deleted_count);
        }

        Ok(deleted_count)
    }

    /// Get credit statistics
    pub async fn get_credit_statistics(&self) -> Result<CreditStatistics, AppError> {
        // Total credits issued
        let total_issued = sqlx::query_scalar!(
            "SELECT COALESCE(SUM(amount), 0) FROM user_credits"
        )
        .fetch_one(&self.db_pool)
        .await?
        .unwrap_or(Decimal::ZERO);

        // Total credits used
        let total_used = sqlx::query_scalar!(
            "SELECT COALESCE(SUM(amount), 0) FROM user_credits WHERE is_used = true"
        )
        .fetch_one(&self.db_pool)
        .await?
        .unwrap_or(Decimal::ZERO);

        // Total credits expired
        let total_expired = sqlx::query_scalar!(
            "SELECT COALESCE(SUM(amount), 0) FROM user_credits WHERE expires_at < NOW() AND is_used = false"
        )
        .fetch_one(&self.db_pool)
        .await?
        .unwrap_or(Decimal::ZERO);

        // Active users with credits
        let active_users = sqlx::query_scalar!(
            r#"
            SELECT COUNT(DISTINCT user_id) 
            FROM user_credits 
            WHERE is_used = false 
            AND (expires_at IS NULL OR expires_at > NOW())
            "#
        )
        .fetch_one(&self.db_pool)
        .await?
        .unwrap_or(0);

        // Credits by source
        let credits_by_source_rows = sqlx::query!(
            r#"
            SELECT source as "source: CreditSource", COALESCE(SUM(amount), 0) as total
            FROM user_credits 
            GROUP BY source
            "#
        )
        .fetch_all(&self.db_pool)
        .await?;

        let mut credits_by_source = HashMap::new();
        for row in credits_by_source_rows {
            credits_by_source.insert(row.source, row.total.unwrap_or(Decimal::ZERO));
        }

        // Credits by type
        let credits_by_type_rows = sqlx::query!(
            r#"
            SELECT credit_type as "credit_type: CreditType", COALESCE(SUM(amount), 0) as total
            FROM user_credits 
            GROUP BY credit_type
            "#
        )
        .fetch_all(&self.db_pool)
        .await?;

        let mut credits_by_type = HashMap::new();
        for row in credits_by_type_rows {
            credits_by_type.insert(row.credit_type, row.total.unwrap_or(Decimal::ZERO));
        }

        Ok(CreditStatistics {
            total_credits_issued: total_issued,
            total_credits_used: total_used,
            total_credits_expired: total_expired,
            active_users_with_credits: active_users,
            credits_by_source,
            credits_by_type,
        })
    }

    /// Check if user has sufficient credits for a purchase
    pub async fn check_sufficient_credits(
        &self,
        user_id: Uuid,
        required_amount: Decimal,
        item_id: Option<Uuid>,
        credit_type: Option<CreditType>,
    ) -> Result<bool, AppError> {
        let available_amount = UserCredit::calculate_total_available(
            &self.db_pool,
            user_id,
            credit_type,
            item_id,
        ).await?;

        Ok(available_amount >= required_amount)
    }

    /// Get free items available for credit redemption
    pub async fn get_free_items_for_credits(&self, user_id: Uuid) -> Result<Vec<Item>, AppError> {
        // Get user's expiring credits
        let expiring_credits = self.get_expiring_credits(user_id, 7).await?; // 7 days
        
        if expiring_credits.is_empty() {
            return Ok(Vec::new());
        }

        // Find items that can be redeemed with expiring credits
        let free_items = sqlx::query_as!(
            Item,
            r#"
            SELECT 
                id, seller_id, name, description, images, retail_price, cost_of_goods,
                status as "status: ItemStatus", stock_quantity, listing_fee_applied, listing_fee_type,
                created_at, updated_at
            FROM items 
            WHERE status = 'available' 
            AND retail_price <= $1
            ORDER BY retail_price ASC
            LIMIT 20
            "#,
            expiring_credits.iter().map(|c| c.amount).sum::<Decimal>()
        )
        .fetch_all(&self.db_pool)
        .await?;

        Ok(free_items)
    }

    /// Redeem free item with expiring credits
    pub async fn redeem_free_item(
        &self,
        user_id: Uuid,
        item_id: Uuid,
    ) -> Result<CreditRedemptionResult, AppError> {
        // Get item details
        let item = Item::find_by_id(&self.db_pool, item_id).await?
            .ok_or_else(|| AppError::NotFound("Item not found".to_string()))?;

        // Check if user has expiring credits that can cover the item
        let expiring_credits = self.get_expiring_credits(user_id, 7).await?;
        let total_expiring: Decimal = expiring_credits.iter().map(|c| c.amount).sum();

        if total_expiring < item.retail_price {
            return Err(AppError::Validation(
                "Insufficient expiring credits for this item".to_string()
            ));
        }

        // Redeem credits for the free item
        let request = CreditRedemptionRequest {
            user_id,
            amount: item.retail_price,
            item_id: Some(item_id),
            credit_type: None, // Allow any type
            description: format!("Free item redemption: {}", item.name),
        };

        self.redeem_credits(request).await
    }

    /// Get expired credits amount for a user
    async fn get_expired_credits_amount(&self, user_id: Uuid) -> Result<Decimal, AppError> {
        let expired_amount = sqlx::query_scalar!(
            r#"
            SELECT COALESCE(SUM(amount), 0) 
            FROM user_credits 
            WHERE user_id = $1 
            AND expires_at < NOW() 
            AND is_used = false
            "#,
            user_id
        )
        .fetch_one(&self.db_pool)
        .await?
        .unwrap_or(Decimal::ZERO);

        Ok(expired_amount)
    }

    /// Log credit transaction for audit purposes
    async fn log_credit_transaction(
        &self,
        user_id: Uuid,
        amount: Decimal,
        transaction_type: &str,
        description: &str,
        credit_id: Option<Uuid>,
    ) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            INSERT INTO credit_transactions (user_id, amount, transaction_type, description, credit_id)
            VALUES ($1, $2, $3, $4, $5)
            "#,
            user_id,
            amount,
            transaction_type,
            description,
            credit_id
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    /// Start background tasks for credit management
    pub async fn start_background_tasks(&self) {
        let service = self.clone();
        
        // Daily cleanup of expired credits
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(86400)); // 24 hours
            
            loop {
                interval.tick().await;
                
                match service.cleanup_expired_credits().await {
                    Ok(count) => {
                        if count > 0 {
                            info!("Daily cleanup: removed {} expired credits", count);
                        }
                    }
                    Err(e) => {
                        error!("Failed to cleanup expired credits: {}", e);
                    }
                }
            }
        });

        info!("Credit service background tasks started");
    }

    /// Send expiration notifications (to be called by notification service)
    pub async fn get_expiration_notifications(&self) -> Result<Vec<ExpirationNotification>, AppError> {
        // Get notifications for credits expiring in 7 days
        let seven_day_notifications = self.get_users_with_expiring_credits(7).await?;
        
        // Get notifications for credits expiring in 1 day
        let one_day_notifications = self.get_users_with_expiring_credits(1).await?;

        // Combine and deduplicate
        let mut all_notifications = seven_day_notifications;
        all_notifications.extend(one_day_notifications);

        // Remove duplicates (keep the one with shorter expiry)
        let mut unique_notifications: HashMap<Uuid, ExpirationNotification> = HashMap::new();
        for notification in all_notifications {
            match unique_notifications.get(&notification.user_id) {
                Some(existing) if existing.days_until_expiry <= notification.days_until_expiry => {
                    // Keep existing (shorter expiry)
                }
                _ => {
                    unique_notifications.insert(notification.user_id, notification);
                }
            }
        }

        Ok(unique_notifications.into_values().collect())
    }
}