use crate::services::credit_service::{CreditService, ExpirationNotification};
use crate::error::AppError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Notification service handles sending notifications to users
#[derive(Clone)]
pub struct NotificationService {
    credit_service: CreditService,
    notification_queue: std::sync::Arc<RwLock<Vec<PendingNotification>>>,
    sent_notifications: std::sync::Arc<RwLock<HashMap<String, DateTime<Utc>>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingNotification {
    pub id: Uuid,
    pub user_id: Uuid,
    pub notification_type: NotificationType,
    pub title: String,
    pub message: String,
    pub data: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub scheduled_for: Option<DateTime<Utc>>,
    pub attempts: u32,
    pub max_attempts: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum NotificationType {
    CreditExpiring,
    CreditExpired,
    RaffleWon,
    RaffleLost,
    BoxPurchased,
    PaymentReceived,
    SystemAlert,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPreferences {
    pub user_id: Uuid,
    pub email_enabled: bool,
    pub push_enabled: bool,
    pub sms_enabled: bool,
    pub credit_expiry_notifications: bool,
    pub raffle_notifications: bool,
    pub payment_notifications: bool,
    pub marketing_notifications: bool,
}

impl Default for NotificationPreferences {
    fn default() -> Self {
        Self {
            user_id: Uuid::new_v4(),
            email_enabled: true,
            push_enabled: true,
            sms_enabled: false,
            credit_expiry_notifications: true,
            raffle_notifications: true,
            payment_notifications: true,
            marketing_notifications: false,
        }
    }
}

impl NotificationService {
    /// Create a new notification service
    pub fn new(credit_service: CreditService) -> Self {
        Self {
            credit_service,
            notification_queue: std::sync::Arc::new(RwLock::new(Vec::new())),
            sent_notifications: std::sync::Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start background notification processing
    pub async fn start_background_tasks(&self) {
        let service = self.clone();
        
        // Credit expiration notification task
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(3600)); // Every hour
            
            loop {
                interval.tick().await;
                
                if let Err(e) = service.process_credit_expiration_notifications().await {
                    error!("Failed to process credit expiration notifications: {}", e);
                }
            }
        });

        // Notification queue processor
        let service = self.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60)); // Every minute
            
            loop {
                interval.tick().await;
                
                if let Err(e) = service.process_notification_queue().await {
                    error!("Failed to process notification queue: {}", e);
                }
            }
        });

        info!("Notification service background tasks started");
    }

    /// Process credit expiration notifications
    async fn process_credit_expiration_notifications(&self) -> Result<(), AppError> {
        debug!("Processing credit expiration notifications");

        let notifications = self.credit_service.get_expiration_notifications().await?;
        
        for notification in notifications {
            self.send_credit_expiration_notification(notification).await?;
        }

        Ok(())
    }

    /// Send credit expiration notification
    async fn send_credit_expiration_notification(
        &self,
        notification: ExpirationNotification,
    ) -> Result<(), AppError> {
        let notification_key = format!("credit_expiry_{}_{}", notification.user_id, notification.days_until_expiry);
        
        // Check if we've already sent this notification recently
        {
            let sent_notifications = self.sent_notifications.read().await;
            if let Some(last_sent) = sent_notifications.get(&notification_key) {
                let hours_since_last = (Utc::now() - *last_sent).num_hours();
                if hours_since_last < 24 {
                    debug!("Skipping duplicate notification: {}", notification_key);
                    return Ok(());
                }
            }
        }

        let (title, message) = if notification.days_until_expiry <= 1 {
            (
                "Credits Expiring Soon!".to_string(),
                format!(
                    "You have {} credits expiring within 24 hours. Redeem them now to avoid losing them!",
                    notification.total_amount
                ),
            )
        } else {
            (
                "Credits Expiring Soon".to_string(),
                format!(
                    "You have {} credits expiring in {} days. Don't forget to use them!",
                    notification.total_amount,
                    notification.days_until_expiry
                ),
            )
        };

        let pending_notification = PendingNotification {
            id: Uuid::new_v4(),
            user_id: notification.user_id,
            notification_type: NotificationType::CreditExpiring,
            title,
            message,
            data: serde_json::to_value(&notification)?,
            created_at: Utc::now(),
            scheduled_for: None,
            attempts: 0,
            max_attempts: 3,
        };

        self.queue_notification(pending_notification).await;

        // Mark as sent
        {
            let mut sent_notifications = self.sent_notifications.write().await;
            sent_notifications.insert(notification_key, Utc::now());
        }

        Ok(())
    }

    /// Queue a notification for processing
    pub async fn queue_notification(&self, notification: PendingNotification) {
        let mut queue = self.notification_queue.write().await;
        queue.push(notification);
        debug!("Queued notification for user: {}", notification.user_id);
    }

    /// Process the notification queue
    async fn process_notification_queue(&self) -> Result<(), AppError> {
        let mut queue = self.notification_queue.write().await;
        let mut processed_indices = Vec::new();

        for (index, notification) in queue.iter_mut().enumerate() {
            // Check if notification is scheduled for later
            if let Some(scheduled_for) = notification.scheduled_for {
                if scheduled_for > Utc::now() {
                    continue;
                }
            }

            // Check if we've exceeded max attempts
            if notification.attempts >= notification.max_attempts {
                warn!(
                    "Notification {} exceeded max attempts ({}), removing from queue",
                    notification.id, notification.max_attempts
                );
                processed_indices.push(index);
                continue;
            }

            // Attempt to send the notification
            notification.attempts += 1;
            
            match self.send_notification(notification).await {
                Ok(()) => {
                    info!("Successfully sent notification: {}", notification.id);
                    processed_indices.push(index);
                }
                Err(e) => {
                    error!(
                        "Failed to send notification {} (attempt {}): {}",
                        notification.id, notification.attempts, e
                    );
                    
                    // Schedule retry with exponential backoff
                    let delay_minutes = 2_u64.pow(notification.attempts - 1).min(60); // Max 1 hour
                    notification.scheduled_for = Some(Utc::now() + chrono::Duration::minutes(delay_minutes as i64));
                }
            }
        }

        // Remove processed notifications (in reverse order to maintain indices)
        for &index in processed_indices.iter().rev() {
            queue.remove(index);
        }

        Ok(())
    }

    /// Send a notification (implement actual sending logic here)
    async fn send_notification(&self, notification: &PendingNotification) -> Result<(), AppError> {
        // Get user preferences
        let preferences = self.get_user_notification_preferences(notification.user_id).await?;
        
        // Check if user wants this type of notification
        let should_send = match notification.notification_type {
            NotificationType::CreditExpiring | NotificationType::CreditExpired => {
                preferences.credit_expiry_notifications
            }
            NotificationType::RaffleWon | NotificationType::RaffleLost | NotificationType::BoxPurchased => {
                preferences.raffle_notifications
            }
            NotificationType::PaymentReceived => {
                preferences.payment_notifications
            }
            NotificationType::SystemAlert => true, // Always send system alerts
        };

        if !should_send {
            debug!("User {} has disabled notifications of type {:?}", notification.user_id, notification.notification_type);
            return Ok(());
        }

        // Send via different channels based on preferences
        let mut sent_via_any_channel = false;

        if preferences.email_enabled {
            match self.send_email_notification(notification).await {
                Ok(()) => {
                    debug!("Email notification sent to user: {}", notification.user_id);
                    sent_via_any_channel = true;
                }
                Err(e) => {
                    warn!("Failed to send email notification: {}", e);
                }
            }
        }

        if preferences.push_enabled {
            match self.send_push_notification(notification).await {
                Ok(()) => {
                    debug!("Push notification sent to user: {}", notification.user_id);
                    sent_via_any_channel = true;
                }
                Err(e) => {
                    warn!("Failed to send push notification: {}", e);
                }
            }
        }

        if preferences.sms_enabled {
            match self.send_sms_notification(notification).await {
                Ok(()) => {
                    debug!("SMS notification sent to user: {}", notification.user_id);
                    sent_via_any_channel = true;
                }
                Err(e) => {
                    warn!("Failed to send SMS notification: {}", e);
                }
            }
        }

        if sent_via_any_channel {
            // Log the notification
            self.log_notification_sent(notification).await?;
            Ok(())
        } else {
            Err(AppError::Internal("No notification channels available".to_string()))
        }
    }

    /// Send email notification (placeholder implementation)
    async fn send_email_notification(&self, notification: &PendingNotification) -> Result<(), AppError> {
        // TODO: Implement actual email sending logic
        // This could use services like SendGrid, AWS SES, etc.
        
        debug!(
            "Would send email to user {}: {} - {}",
            notification.user_id, notification.title, notification.message
        );
        
        // Simulate email sending delay
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        Ok(())
    }

    /// Send push notification (placeholder implementation)
    async fn send_push_notification(&self, notification: &PendingNotification) -> Result<(), AppError> {
        // TODO: Implement actual push notification logic
        // This could use services like Firebase Cloud Messaging, Apple Push Notification Service, etc.
        
        debug!(
            "Would send push notification to user {}: {} - {}",
            notification.user_id, notification.title, notification.message
        );
        
        // Simulate push notification delay
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        
        Ok(())
    }

    /// Send SMS notification (placeholder implementation)
    async fn send_sms_notification(&self, notification: &PendingNotification) -> Result<(), AppError> {
        // TODO: Implement actual SMS sending logic
        // This could use services like Twilio, AWS SNS, etc.
        
        debug!(
            "Would send SMS to user {}: {} - {}",
            notification.user_id, notification.title, notification.message
        );
        
        // Simulate SMS sending delay
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        
        Ok(())
    }

    /// Get user notification preferences (placeholder implementation)
    async fn get_user_notification_preferences(&self, user_id: Uuid) -> Result<NotificationPreferences, AppError> {
        // TODO: Implement actual database lookup for user preferences
        // For now, return default preferences
        
        Ok(NotificationPreferences {
            user_id,
            ..Default::default()
        })
    }

    /// Log notification sent (placeholder implementation)
    async fn log_notification_sent(&self, notification: &PendingNotification) -> Result<(), AppError> {
        // TODO: Implement actual logging to database
        
        info!(
            "Notification {} sent to user {} via configured channels",
            notification.id, notification.user_id
        );
        
        Ok(())
    }

    /// Send raffle result notification
    pub async fn send_raffle_result_notification(
        &self,
        user_id: Uuid,
        raffle_id: Uuid,
        won: bool,
        item_title: String,
        credits_received: Option<rust_decimal::Decimal>,
    ) -> Result<(), AppError> {
        let (notification_type, title, message) = if won {
            (
                NotificationType::RaffleWon,
                "Congratulations! You Won!".to_string(),
                format!("You won the raffle for '{}'! Check your account for details.", item_title),
            )
        } else {
            let credits_msg = if let Some(credits) = credits_received {
                format!(" You received {} credits that you can use for future purchases.", credits)
            } else {
                String::new()
            };
            
            (
                NotificationType::RaffleLost,
                "Raffle Results".to_string(),
                format!("The raffle for '{}' has ended.{}", item_title, credits_msg),
            )
        };

        let notification = PendingNotification {
            id: Uuid::new_v4(),
            user_id,
            notification_type,
            title,
            message,
            data: serde_json::json!({
                "raffle_id": raffle_id,
                "item_title": item_title,
                "won": won,
                "credits_received": credits_received
            }),
            created_at: Utc::now(),
            scheduled_for: None,
            attempts: 0,
            max_attempts: 3,
        };

        self.queue_notification(notification).await;
        Ok(())
    }

    /// Send box purchase confirmation notification
    pub async fn send_box_purchase_notification(
        &self,
        user_id: Uuid,
        raffle_id: Uuid,
        item_title: String,
        box_number: u32,
        total_boxes: u32,
    ) -> Result<(), AppError> {
        let notification = PendingNotification {
            id: Uuid::new_v4(),
            user_id,
            notification_type: NotificationType::BoxPurchased,
            title: "Box Purchase Confirmed".to_string(),
            message: format!(
                "You purchased box #{} for '{}'. {} of {} boxes sold.",
                box_number, item_title, box_number, total_boxes
            ),
            data: serde_json::json!({
                "raffle_id": raffle_id,
                "item_title": item_title,
                "box_number": box_number,
                "total_boxes": total_boxes
            }),
            created_at: Utc::now(),
            scheduled_for: None,
            attempts: 0,
            max_attempts: 3,
        };

        self.queue_notification(notification).await;
        Ok(())
    }

    /// Get notification queue status
    pub async fn get_queue_status(&self) -> NotificationQueueStatus {
        let queue = self.notification_queue.read().await;
        let sent_notifications = self.sent_notifications.read().await;

        NotificationQueueStatus {
            pending_notifications: queue.len(),
            total_sent_today: sent_notifications.values()
                .filter(|&&sent_at| (Utc::now() - sent_at).num_hours() < 24)
                .count(),
            queue_oldest_notification: queue.iter()
                .map(|n| n.created_at)
                .min(),
        }
    }

    /// Clear old sent notification records
    pub async fn cleanup_old_records(&self) {
        let mut sent_notifications = self.sent_notifications.write().await;
        let cutoff = Utc::now() - chrono::Duration::days(7);
        
        sent_notifications.retain(|_, &mut sent_at| sent_at > cutoff);
        
        debug!("Cleaned up old notification records");
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct NotificationQueueStatus {
    pub pending_notifications: usize,
    pub total_sent_today: usize,
    pub queue_oldest_notification: Option<DateTime<Utc>>,
}