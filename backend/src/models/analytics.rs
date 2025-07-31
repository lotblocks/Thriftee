use chrono::{DateTime, Utc, NaiveDate};
use raffle_platform_shared::{RaffleMetricsResponse, PlatformStatsResponse};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;
use crate::error::AppError;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct UserActivityLog {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub session_id: Option<Uuid>,
    pub activity_type: String,
    pub page_url: Option<String>,
    pub referrer: Option<String>,
    pub ip_address: Option<std::net::IpAddr>,
    pub user_agent: Option<String>,
    pub device_type: Option<String>,
    pub browser: Option<String>,
    pub os: Option<String>,
    pub country: Option<String>,
    pub city: Option<String>,
    pub duration_seconds: Option<i32>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

impl UserActivityLog {
    /// Create a new activity log entry
    pub async fn create(
        pool: &PgPool,
        user_id: Option<Uuid>,
        session_id: Option<Uuid>,
        activity_type: String,
        page_url: Option<String>,
        referrer: Option<String>,
        ip_address: Option<std::net::IpAddr>,
        user_agent: Option<String>,
        device_type: Option<String>,
        browser: Option<String>,
        os: Option<String>,
        country: Option<String>,
        city: Option<String>,
        duration_seconds: Option<i32>,
        metadata: Option<serde_json::Value>,
    ) -> Result<Self, AppError> {
        let log = sqlx::query_as!(
            UserActivityLog,
            r#\"
            INSERT INTO user_activity_logs (
                user_id, session_id, activity_type, page_url, referrer, ip_address,
                user_agent, device_type, browser, os, country, city, duration_seconds, metadata
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            RETURNING 
                id, user_id, session_id, activity_type, page_url, referrer,
                ip_address as \"ip_address: std::net::IpAddr\", user_agent, device_type,
                browser, os, country, city, duration_seconds, metadata, created_at
            \"#,
            user_id,
            session_id,
            activity_type,
            page_url,
            referrer,
            ip_address,
            user_agent,
            device_type,
            browser,
            os,
            country,
            city,
            duration_seconds,
            metadata
        )
        .fetch_one(pool)
        .await?;

        Ok(log)
    }

    /// Find activity logs by user
    pub async fn find_by_user(
        pool: &PgPool,
        user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Self>, AppError> {
        let logs = sqlx::query_as!(
            UserActivityLog,
            r#\"
            SELECT 
                id, user_id, session_id, activity_type, page_url, referrer,
                ip_address as \"ip_address: std::net::IpAddr\", user_agent, device_type,
                browser, os, country, city, duration_seconds, metadata, created_at
            FROM user_activity_logs 
            WHERE user_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            \"#,
            user_id,
            limit,
            offset
        )
        .fetch_all(pool)
        .await?;

        Ok(logs)
    }

    /// Find activity logs by type in time period
    pub async fn find_by_type_in_period(
        pool: &PgPool,
        activity_type: &str,
        hours: i64,
    ) -> Result<Vec<Self>, AppError> {
        let logs = sqlx::query_as!(
            UserActivityLog,
            r#\"
            SELECT 
                id, user_id, session_id, activity_type, page_url, referrer,
                ip_address as \"ip_address: std::net::IpAddr\", user_agent, device_type,
                browser, os, country, city, duration_seconds, metadata, created_at
            FROM user_activity_logs 
            WHERE activity_type = $1 AND created_at >= NOW() - INTERVAL '%d hours'
            ORDER BY created_at DESC
            \"#,
            activity_type,
            hours
        )
        .fetch_all(pool)
        .await?;

        Ok(logs)
    }

    /// Get unique users count in time period
    pub async fn count_unique_users_in_period(
        pool: &PgPool,
        hours: i64,
    ) -> Result<i64, AppError> {
        let count = sqlx::query_scalar!(
            r#\"
            SELECT COUNT(DISTINCT user_id) 
            FROM user_activity_logs 
            WHERE user_id IS NOT NULL AND created_at >= NOW() - INTERVAL '%d hours'
            \"#,
            hours
        )
        .fetch_one(pool)
        .await?;

        Ok(count.unwrap_or(0))
    }

    /// Clean up old activity logs
    pub async fn cleanup_old_logs(pool: &PgPool, retention_days: i64) -> Result<u64, AppError> {
        let result = sqlx::query!(
            \"DELETE FROM user_activity_logs WHERE created_at < NOW() - INTERVAL '%d days'\",
            retention_days
        )
        .execute(pool)
        .await?;

        Ok(result.rows_affected())
    }
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct RaffleMetrics {
    pub id: Uuid,
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
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl RaffleMetrics {
    /// Create or update raffle metrics
    pub async fn upsert(
        pool: &PgPool,
        raffle_id: Uuid,
        views_count: Option<i32>,
        unique_viewers: Option<i32>,
        conversion_rate: Option<Decimal>,
        average_boxes_per_user: Option<Decimal>,
        time_to_completion_minutes: Option<i32>,
        peak_concurrent_users: Option<i32>,
        total_revenue: Option<Decimal>,
        platform_fee: Option<Decimal>,
        seller_payout: Option<Decimal>,
    ) -> Result<Self, AppError> {
        let metrics = sqlx::query_as!(
            RaffleMetrics,
            r#\"
            INSERT INTO raffle_metrics (
                raffle_id, views_count, unique_viewers, conversion_rate, average_boxes_per_user,
                time_to_completion_minutes, peak_concurrent_users, total_revenue, platform_fee, seller_payout
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (raffle_id) DO UPDATE SET
                views_count = COALESCE($2, raffle_metrics.views_count),
                unique_viewers = COALESCE($3, raffle_metrics.unique_viewers),
                conversion_rate = COALESCE($4, raffle_metrics.conversion_rate),
                average_boxes_per_user = COALESCE($5, raffle_metrics.average_boxes_per_user),
                time_to_completion_minutes = COALESCE($6, raffle_metrics.time_to_completion_minutes),
                peak_concurrent_users = COALESCE($7, raffle_metrics.peak_concurrent_users),
                total_revenue = COALESCE($8, raffle_metrics.total_revenue),
                platform_fee = COALESCE($9, raffle_metrics.platform_fee),
                seller_payout = COALESCE($10, raffle_metrics.seller_payout),
                updated_at = NOW()
            RETURNING 
                id, raffle_id, views_count, unique_viewers, conversion_rate, average_boxes_per_user,
                time_to_completion_minutes, peak_concurrent_users, total_revenue, platform_fee, 
                seller_payout, created_at, updated_at
            \"#,
            raffle_id,
            views_count.unwrap_or(0),
            unique_viewers.unwrap_or(0),
            conversion_rate,
            average_boxes_per_user,
            time_to_completion_minutes,
            peak_concurrent_users.unwrap_or(0),
            total_revenue,
            platform_fee,
            seller_payout
        )
        .fetch_one(pool)
        .await?;

        Ok(metrics)
    }

    /// Find metrics by raffle
    pub async fn find_by_raffle(pool: &PgPool, raffle_id: Uuid) -> Result<Option<Self>, AppError> {
        let metrics = sqlx::query_as!(
            RaffleMetrics,
            r#\"
            SELECT 
                id, raffle_id, views_count, unique_viewers, conversion_rate, average_boxes_per_user,
                time_to_completion_minutes, peak_concurrent_users, total_revenue, platform_fee, 
                seller_payout, created_at, updated_at
            FROM raffle_metrics 
            WHERE raffle_id = $1
            \"#,
            raffle_id
        )
        .fetch_optional(pool)
        .await?;

        Ok(metrics)
    }

    /// Increment view count
    pub async fn increment_views(pool: &PgPool, raffle_id: Uuid) -> Result<(), AppError> {
        sqlx::query!(
            r#\"
            INSERT INTO raffle_metrics (raffle_id, views_count, unique_viewers)
            VALUES ($1, 1, 0)
            ON CONFLICT (raffle_id) DO UPDATE SET
                views_count = raffle_metrics.views_count + 1,
                updated_at = NOW()
            \"#,
            raffle_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Update peak concurrent users
    pub async fn update_peak_concurrent_users(
        pool: &PgPool,
        raffle_id: Uuid,
        concurrent_users: i32,
    ) -> Result<(), AppError> {
        sqlx::query!(
            r#\"
            INSERT INTO raffle_metrics (raffle_id, views_count, unique_viewers, peak_concurrent_users)
            VALUES ($1, 0, 0, $2)
            ON CONFLICT (raffle_id) DO UPDATE SET
                peak_concurrent_users = GREATEST(raffle_metrics.peak_concurrent_users, $2),
                updated_at = NOW()
            \"#,
            raffle_id,
            concurrent_users
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Convert to response DTO
    pub fn to_response(&self) -> RaffleMetricsResponse {
        RaffleMetricsResponse {
            raffle_id: self.raffle_id,
            views_count: self.views_count,
            unique_viewers: self.unique_viewers,
            conversion_rate: self.conversion_rate,
            average_boxes_per_user: self.average_boxes_per_user,
            time_to_completion_minutes: self.time_to_completion_minutes,
            peak_concurrent_users: self.peak_concurrent_users,
            total_revenue: self.total_revenue,
            platform_fee: self.platform_fee,
            seller_payout: self.seller_payout,
        }
    }
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct DailyPlatformStats {
    pub id: Uuid,
    pub date: NaiveDate,
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
    pub created_at: DateTime<Utc>,
}

impl DailyPlatformStats {
    /// Create or update daily stats
    pub async fn upsert_for_date(
        pool: &PgPool,
        date: NaiveDate,
    ) -> Result<Self, AppError> {
        // Calculate stats for the given date
        let stats = Self::calculate_stats_for_date(pool, date).await?;

        let daily_stats = sqlx::query_as!(
            DailyPlatformStats,
            r#\"
            INSERT INTO daily_platform_stats (
                date, total_users, new_users, active_users, total_sellers, new_sellers,
                active_sellers, total_raffles, completed_raffles, total_revenue,
                total_credits_issued, total_credits_redeemed, average_raffle_completion_time
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            ON CONFLICT (date) DO UPDATE SET
                total_users = $2,
                new_users = $3,
                active_users = $4,
                total_sellers = $5,
                new_sellers = $6,
                active_sellers = $7,
                total_raffles = $8,
                completed_raffles = $9,
                total_revenue = $10,
                total_credits_issued = $11,
                total_credits_redeemed = $12,
                average_raffle_completion_time = $13
            RETURNING 
                id, date, total_users, new_users, active_users, total_sellers, new_sellers,
                active_sellers, total_raffles, completed_raffles, total_revenue,
                total_credits_issued, total_credits_redeemed, average_raffle_completion_time, created_at
            \"#,
            date,
            stats.total_users,
            stats.new_users,
            stats.active_users,
            stats.total_sellers,
            stats.new_sellers,
            stats.active_sellers,
            stats.total_raffles,
            stats.completed_raffles,
            stats.total_revenue,
            stats.total_credits_issued,
            stats.total_credits_redeemed,
            stats.average_raffle_completion_time
        )
        .fetch_one(pool)
        .await?;

        Ok(daily_stats)
    }

    /// Calculate stats for a specific date
    async fn calculate_stats_for_date(
        pool: &PgPool,
        date: NaiveDate,
    ) -> Result<PlatformStatsResponse, AppError> {
        let start_of_day = date.and_hms_opt(0, 0, 0).unwrap().and_utc();
        let end_of_day = date.and_hms_opt(23, 59, 59).unwrap().and_utc();

        // Total users up to this date
        let total_users: i32 = sqlx::query_scalar!(
            \"SELECT COUNT(*)::int FROM users WHERE created_at <= $1\",
            end_of_day
        )
        .fetch_one(pool)
        .await?
        .unwrap_or(0);

        // New users on this date
        let new_users: i32 = sqlx::query_scalar!(
            \"SELECT COUNT(*)::int FROM users WHERE created_at >= $1 AND created_at <= $2\",
            start_of_day,
            end_of_day
        )
        .fetch_one(pool)
        .await?
        .unwrap_or(0);

        // Active users on this date (users who had activity)
        let active_users: i32 = sqlx::query_scalar!(
            r#\"
            SELECT COUNT(DISTINCT user_id)::int 
            FROM user_activity_logs 
            WHERE user_id IS NOT NULL AND created_at >= $1 AND created_at <= $2
            \"#,
            start_of_day,
            end_of_day
        )
        .fetch_one(pool)
        .await?
        .unwrap_or(0);

        // Total sellers up to this date
        let total_sellers: i32 = sqlx::query_scalar!(
            \"SELECT COUNT(*)::int FROM sellers WHERE created_at <= $1\",
            end_of_day
        )
        .fetch_one(pool)
        .await?
        .unwrap_or(0);

        // New sellers on this date
        let new_sellers: i32 = sqlx::query_scalar!(
            \"SELECT COUNT(*)::int FROM sellers WHERE created_at >= $1 AND created_at <= $2\",
            start_of_day,
            end_of_day
        )
        .fetch_one(pool)
        .await?
        .unwrap_or(0);

        // Active sellers on this date (sellers with activity)
        let active_sellers: i32 = sqlx::query_scalar!(
            r#\"
            SELECT COUNT(DISTINCT s.id)::int 
            FROM sellers s
            JOIN items i ON s.id = i.seller_id
            JOIN raffles r ON i.id = r.item_id
            WHERE r.created_at >= $1 AND r.created_at <= $2
            \"#,
            start_of_day,
            end_of_day
        )
        .fetch_one(pool)
        .await?
        .unwrap_or(0);

        // Total raffles up to this date
        let total_raffles: i32 = sqlx::query_scalar!(
            \"SELECT COUNT(*)::int FROM raffles WHERE created_at <= $1\",
            end_of_day
        )
        .fetch_one(pool)
        .await?
        .unwrap_or(0);

        // Completed raffles on this date
        let completed_raffles: i32 = sqlx::query_scalar!(
            r#\"
            SELECT COUNT(*)::int FROM raffles 
            WHERE status = 'completed' AND completed_at >= $1 AND completed_at <= $2
            \"#,
            start_of_day,
            end_of_day
        )
        .fetch_one(pool)
        .await?
        .unwrap_or(0);

        // Total revenue on this date
        let total_revenue: Decimal = sqlx::query_scalar!(
            r#\"
            SELECT COALESCE(SUM(r.box_price * r.boxes_sold), 0) 
            FROM raffles r 
            WHERE r.completed_at >= $1 AND r.completed_at <= $2
            \"#,
            start_of_day,
            end_of_day
        )
        .fetch_one(pool)
        .await?
        .unwrap_or(Decimal::ZERO);

        // Total credits issued on this date
        let total_credits_issued: Decimal = sqlx::query_scalar!(
            r#\"
            SELECT COALESCE(SUM(amount), 0) 
            FROM user_credits 
            WHERE created_at >= $1 AND created_at <= $2
            \"#,
            start_of_day,
            end_of_day
        )
        .fetch_one(pool)
        .await?
        .unwrap_or(Decimal::ZERO);

        // Total credits redeemed on this date
        let total_credits_redeemed: Decimal = sqlx::query_scalar!(
            r#\"
            SELECT COALESCE(SUM(amount), 0) 
            FROM user_credits 
            WHERE is_used = true AND used_at >= $1 AND used_at <= $2
            \"#,
            start_of_day,
            end_of_day
        )
        .fetch_one(pool)
        .await?
        .unwrap_or(Decimal::ZERO);

        // Average raffle completion time on this date
        let average_raffle_completion_time: Option<i32> = sqlx::query_scalar!(
            r#\"
            SELECT AVG(EXTRACT(EPOCH FROM (completed_at - created_at)) / 60)::int
            FROM raffles 
            WHERE status = 'completed' AND completed_at >= $1 AND completed_at <= $2
            \"#,
            start_of_day,
            end_of_day
        )
        .fetch_one(pool)
        .await?;

        Ok(PlatformStatsResponse {
            date,
            total_users,
            new_users,
            active_users,
            total_sellers,
            new_sellers,
            active_sellers,
            total_raffles,
            completed_raffles,
            total_revenue,
            total_credits_issued,
            total_credits_redeemed,
            average_raffle_completion_time,
        })
    }

    /// Find stats by date range
    pub async fn find_by_date_range(
        pool: &PgPool,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<Self>, AppError> {
        let stats = sqlx::query_as!(
            DailyPlatformStats,
            r#\"
            SELECT 
                id, date, total_users, new_users, active_users, total_sellers, new_sellers,
                active_sellers, total_raffles, completed_raffles, total_revenue,
                total_credits_issued, total_credits_redeemed, average_raffle_completion_time, created_at
            FROM daily_platform_stats 
            WHERE date >= $1 AND date <= $2
            ORDER BY date ASC
            \"#,
            start_date,
            end_date
        )
        .fetch_all(pool)
        .await?;

        Ok(stats)
    }

    /// Convert to response DTO
    pub fn to_response(&self) -> PlatformStatsResponse {
        PlatformStatsResponse {
            date: self.date,
            total_users: self.total_users,
            new_users: self.new_users,
            active_users: self.active_users,
            total_sellers: self.total_sellers,
            new_sellers: self.new_sellers,
            active_sellers: self.active_sellers,
            total_raffles: self.total_raffles,
            completed_raffles: self.completed_raffles,
            total_revenue: self.total_revenue,
            total_credits_issued: self.total_credits_issued,
            total_credits_redeemed: self.total_credits_redeemed,
            average_raffle_completion_time: self.average_raffle_completion_time,
        }
    }
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct SellerMetrics {
    pub id: Uuid,
    pub seller_id: Uuid,
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    pub total_items_listed: i32,
    pub total_raffles_completed: i32,
    pub total_revenue: Decimal,
    pub total_fees_paid: Decimal,
    pub average_completion_time: Option<i32>,
    pub conversion_rate: Option<Decimal>,
    pub customer_satisfaction_score: Option<Decimal>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl SellerMetrics {
    /// Create or update seller metrics for a period
    pub async fn upsert_for_period(
        pool: &PgPool,
        seller_id: Uuid,
        period_start: NaiveDate,
        period_end: NaiveDate,
    ) -> Result<Self, AppError> {
        // Calculate metrics for the period
        let start_datetime = period_start.and_hms_opt(0, 0, 0).unwrap().and_utc();
        let end_datetime = period_end.and_hms_opt(23, 59, 59).unwrap().and_utc();

        // Total items listed in period
        let total_items_listed: i32 = sqlx::query_scalar!(
            \"SELECT COUNT(*)::int FROM items WHERE seller_id = $1 AND created_at >= $2 AND created_at <= $3\",
            seller_id,
            start_datetime,
            end_datetime
        )
        .fetch_one(pool)
        .await?
        .unwrap_or(0);

        // Total raffles completed in period
        let total_raffles_completed: i32 = sqlx::query_scalar!(
            r#\"
            SELECT COUNT(*)::int 
            FROM raffles r
            JOIN items i ON r.item_id = i.id
            WHERE i.seller_id = $1 AND r.status = 'completed' 
            AND r.completed_at >= $2 AND r.completed_at <= $3
            \"#,
            seller_id,
            start_datetime,
            end_datetime
        )
        .fetch_one(pool)
        .await?
        .unwrap_or(0);

        // Total revenue in period
        let total_revenue: Decimal = sqlx::query_scalar!(
            r#\"
            SELECT COALESCE(SUM(r.box_price * r.boxes_sold), 0)
            FROM raffles r
            JOIN items i ON r.item_id = i.id
            WHERE i.seller_id = $1 AND r.status = 'completed' 
            AND r.completed_at >= $2 AND r.completed_at <= $3
            \"#,
            seller_id,
            start_datetime,
            end_datetime
        )
        .fetch_one(pool)
        .await?
        .unwrap_or(Decimal::ZERO);

        // Total fees paid in period
        let total_fees_paid: Decimal = sqlx::query_scalar!(
            r#\"
            SELECT COALESCE(SUM(amount), 0)
            FROM transactions
            WHERE seller_id = $1 
            AND type IN ('seller_subscription_fee', 'seller_listing_fee', 'seller_transaction_fee')
            AND created_at >= $2 AND created_at <= $3
            \"#,
            seller_id,
            start_datetime,
            end_datetime
        )
        .fetch_one(pool)
        .await?
        .unwrap_or(Decimal::ZERO);

        // Average completion time in period
        let average_completion_time: Option<i32> = sqlx::query_scalar!(
            r#\"
            SELECT AVG(EXTRACT(EPOCH FROM (r.completed_at - r.created_at)) / 60)::int
            FROM raffles r
            JOIN items i ON r.item_id = i.id
            WHERE i.seller_id = $1 AND r.status = 'completed' 
            AND r.completed_at >= $2 AND r.completed_at <= $3
            \"#,
            seller_id,
            start_datetime,
            end_datetime
        )
        .fetch_one(pool)
        .await?;

        // Conversion rate (completed raffles / total raffles)
        let total_raffles: i32 = sqlx::query_scalar!(
            r#\"
            SELECT COUNT(*)::int 
            FROM raffles r
            JOIN items i ON r.item_id = i.id
            WHERE i.seller_id = $1 AND r.created_at >= $2 AND r.created_at <= $3
            \"#,
            seller_id,
            start_datetime,
            end_datetime
        )
        .fetch_one(pool)
        .await?
        .unwrap_or(0);

        let conversion_rate = if total_raffles > 0 {
            Some(Decimal::from(total_raffles_completed) / Decimal::from(total_raffles) * Decimal::from(100))
        } else {
            None
        };

        let metrics = sqlx::query_as!(
            SellerMetrics,
            r#\"
            INSERT INTO seller_metrics (
                seller_id, period_start, period_end, total_items_listed, total_raffles_completed,
                total_revenue, total_fees_paid, average_completion_time, conversion_rate
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (seller_id, period_start, period_end) DO UPDATE SET
                total_items_listed = $4,
                total_raffles_completed = $5,
                total_revenue = $6,
                total_fees_paid = $7,
                average_completion_time = $8,
                conversion_rate = $9,
                updated_at = NOW()
            RETURNING 
                id, seller_id, period_start, period_end, total_items_listed, total_raffles_completed,
                total_revenue, total_fees_paid, average_completion_time, conversion_rate,
                customer_satisfaction_score, created_at, updated_at
            \"#,
            seller_id,
            period_start,
            period_end,
            total_items_listed,
            total_raffles_completed,
            total_revenue,
            total_fees_paid,
            average_completion_time,
            conversion_rate
        )
        .fetch_one(pool)
        .await?;

        Ok(metrics)
    }

    /// Find metrics by seller and period
    pub async fn find_by_seller_and_period(
        pool: &PgPool,
        seller_id: Uuid,
        period_start: NaiveDate,
        period_end: NaiveDate,
    ) -> Result<Vec<Self>, AppError> {
        let metrics = sqlx::query_as!(
            SellerMetrics,
            r#\"
            SELECT 
                id, seller_id, period_start, period_end, total_items_listed, total_raffles_completed,
                total_revenue, total_fees_paid, average_completion_time, conversion_rate,
                customer_satisfaction_score, created_at, updated_at
            FROM seller_metrics 
            WHERE seller_id = $1 AND period_start >= $2 AND period_end <= $3
            ORDER BY period_start ASC
            \"#,
            seller_id,
            period_start,
            period_end
        )
        .fetch_all(pool)
        .await?;

        Ok(metrics)
    }
}
"