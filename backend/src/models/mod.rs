//! Database models for the Raffle Shopping Platform
//! 
//! This module contains all the database models and their associated CRUD operations.
//! Each model corresponds to a database table and provides type-safe interactions
//! with the database using sqlx.

pub mod audit;
pub mod box_purchase;
pub mod credit;
pub mod free_item;
pub mod item;
pub mod notification;
pub mod raffle;
pub mod seller;
pub mod seller_subscription;
pub mod system_settings;
pub mod transaction;
pub mod user;

#[cfg(test)]
pub mod tests;

// Re-export commonly used models
pub use audit::AuditLog;
pub use box_purchase::{BoxPurchase, BoxPurchaseStatistics};
pub use credit::UserCredit;
pub use free_item::FreeRedeemableItem;
pub use item::Item;
pub use notification::{Notification, NotificationType};
pub use raffle::Raffle;
pub use seller::Seller;
pub use seller_subscription::{SellerSubscription, SubscriptionStatistics};
pub use system_settings::{SystemSetting, SystemSettings};
pub use transaction::{Transaction, TransactionSummary};
pub use user::{User, UserSession};

/// Common database operations trait
pub trait DatabaseModel {
    type Id;
    
    /// Find a model by its ID
    async fn find_by_id(pool: &sqlx::PgPool, id: Self::Id) -> Result<Option<Self>, crate::error::AppError>
    where
        Self: Sized;
}

/// Pagination helper
#[derive(Debug, Clone)]
pub struct Pagination {
    pub limit: i64,
    pub offset: i64,
}

impl Pagination {
    pub fn new(limit: Option<i64>, offset: Option<i64>) -> Self {
        Self {
            limit: limit.unwrap_or(20).min(100).max(1),
            offset: offset.unwrap_or(0).max(0),
        }
    }

    pub fn page(page: i64, per_page: i64) -> Self {
        let per_page = per_page.min(100).max(1);
        let page = page.max(1);
        Self {
            limit: per_page,
            offset: (page - 1) * per_page,
        }
    }
}

/// Common filter options
#[derive(Debug, Clone)]
pub struct DateFilter {
    pub start_date: Option<chrono::DateTime<chrono::Utc>>,
    pub end_date: Option<chrono::DateTime<chrono::Utc>>,
}

impl DateFilter {
    pub fn new(
        start_date: Option<chrono::DateTime<chrono::Utc>>,
        end_date: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Self {
        Self {
            start_date,
            end_date,
        }
    }

    pub fn last_days(days: i64) -> Self {
        let end_date = chrono::Utc::now();
        let start_date = end_date - chrono::Duration::days(days);
        Self {
            start_date: Some(start_date),
            end_date: Some(end_date),
        }
    }

    pub fn this_month() -> Self {
        let now = chrono::Utc::now();
        let start_date = now
            .with_day(1)
            .unwrap()
            .with_hour(0)
            .unwrap()
            .with_minute(0)
            .unwrap()
            .with_second(0)
            .unwrap()
            .with_nanosecond(0)
            .unwrap();
        
        Self {
            start_date: Some(start_date),
            end_date: Some(now),
        }
    }
}