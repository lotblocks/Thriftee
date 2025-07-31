use crate::error::AppError;
use crate::models::raffle::Raffle;
use crate::models::item::Item;
use crate::models::user::User;
use actix::prelude::*;
use actix_web_actors::ws;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::broadcast;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Real-time service for managing WebSocket connections and broadcasting events
#[derive(Clone)]
pub struct RealtimeService {
    db_pool: PgPool,
    connections: Arc<tokio::sync::RwLock<HashMap<Uuid, ConnectionInfo>>>,
    event_sender: broadcast::Sender<RealtimeEvent>,
    _event_receiver: broadcast::Receiver<RealtimeEvent>,
}

#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub addr: Recipient<WebSocketMessage>,
    pub subscriptions: Vec<EventSubscription>,
    pub connected_at: Instant,
    pub last_ping: Instant,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventSubscription {
    pub event_type: String,
    pub raffle_id: Option<Uuid>,
    pub item_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub room: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RealtimeEvent {
    // Raffle events
    RaffleCreated {
        raffle_id: Uuid,
        item_id: Uuid,
        seller_id: Uuid,
        total_boxes: i32,
        box_price: rust_decimal::Decimal,
        created_at: DateTime<Utc>,
    },
    BoxPurchased {
        raffle_id: Uuid,
        user_id: Uuid,
        box_number: i32,
        boxes_remaining: i32,
        completion_percentage: f64,
        purchased_at: DateTime<Utc>,
    },
    RaffleFull {
        raffle_id: Uuid,
        item_id: Uuid,
        total_boxes: i32,
        completed_at: DateTime<Utc>,
    },
    WinnerSelected {
        raffle_id: Uuid,
        item_id: Uuid,
        winner_user_ids: Vec<Uuid>,
        completed_at: DateTime<Utc>,
    },
    RaffleCancelled {
        raffle_id: Uuid,
        item_id: Uuid,
        reason: String,
        cancelled_at: DateTime<Utc>,
    },
    
    // Item events
    ItemCreated {
        item_id: Uuid,
        seller_id: Uuid,
        name: String,
        category: Option<String>,
        created_at: DateTime<Utc>,
    },
    ItemUpdated {
        item_id: Uuid,
        seller_id: Uuid,
        changes: Vec<String>,
        updated_at: DateTime<Utc>,
    },
    ItemStockChanged {
        item_id: Uuid,
        old_quantity: i32,
        new_quantity: i32,
        updated_at: DateTime<Utc>,
    },
    
    // User events
    UserJoined {
        user_id: Uuid,
        username: String,
        joined_at: DateTime<Utc>,
    },
    UserLeft {
        user_id: Uuid,
        left_at: DateTime<Utc>,
    },
    
    // Credit events
    CreditsIssued {
        user_id: Uuid,
        amount: rust_decimal::Decimal,
        source: String,
        issued_at: DateTime<Utc>,
    },
    CreditsRedeemed {
        user_id: Uuid,
        amount: rust_decimal::Decimal,
        item_id: Option<Uuid>,
        redeemed_at: DateTime<Utc>,
    },
    
    // System events
    SystemMaintenance {
        message: String,
        scheduled_at: DateTime<Utc>,
    },
    SystemAlert {
        level: String,
        message: String,
        created_at: DateTime<Utc>,
    },
}

#[derive(Debug, Clone, Message)]
#[rtype(result = "()")]
pub struct WebSocketMessage {
    pub message_type: String,
    pub data: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientMessage {
    pub message_type: String,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscribeMessage {
    pub subscriptions: Vec<EventSubscription>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthMessage {
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoinRoomMessage {
    pub room: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaveRoomMessage {
    pub room: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConnectionStats {
    pub total_connections: usize,
    pub authenticated_connections: usize,
    pub active_rooms: HashMap<String, usize>,
    pub events_sent_last_hour: u64,
}

impl RealtimeService {
    /// Create a new realtime service
    pub fn new(db_pool: PgPool) -> Self {
        let (event_sender, event_receiver) = broadcast::channel(1000);
        
        let service = Self {
            db_pool,
            connections: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            event_sender,
            _event_receiver: event_receiver,
        };

        // Start background tasks
        let service_clone = service.clone();
        tokio::spawn(async move {
            service_clone.start_event_broadcasting().await;
        });

        let service_clone = service.clone();
        tokio::spawn(async move {
            service_clone.cleanup_stale_connections().await;
        });

        service
    }

    /// Add a new WebSocket connection
    pub async fn add_connection(
        &self,
        connection_id: Uuid,
        addr: Recipient<WebSocketMessage>,
        user_agent: Option<String>,
        ip_address: Option<String>,
    ) {
        let connection = ConnectionInfo {
            id: connection_id,
            user_id: None,
            addr,
            subscriptions: Vec::new(),
            connected_at: Instant::now(),
            last_ping: Instant::now(),
            user_agent,
            ip_address,
        };

        let mut connections = self.connections.write().await;
        connections.insert(connection_id, connection);
        
        info!("WebSocket connection added: {}", connection_id);
    }

    /// Remove a WebSocket connection
    pub async fn remove_connection(&self, connection_id: &Uuid) {
        let mut connections = self.connections.write().await;
        if let Some(connection) = connections.remove(connection_id) {
            // Broadcast user left event if authenticated
            if let Some(user_id) = connection.user_id {
                let _ = self.broadcast_event(RealtimeEvent::UserLeft {
                    user_id,
                    left_at: Utc::now(),
                }).await;
            }
            info!("WebSocket connection removed: {}", connection_id);
        }
    }

    /// Authenticate a WebSocket connection
    pub async fn authenticate_connection(
        &self,
        connection_id: &Uuid,
        token: &str,
    ) -> Result<(), AppError> {
        // Validate JWT token and get user ID
        let user_id = self.validate_jwt_token(token).await?;
        
        let mut connections = self.connections.write().await;
        if let Some(connection) = connections.get_mut(connection_id) {
            connection.user_id = Some(user_id);
            
            // Get user details for broadcast
            if let Ok(Some(user)) = User::find_by_id(&self.db_pool, user_id).await {
                // Broadcast user joined event
                let _ = self.broadcast_event(RealtimeEvent::UserJoined {
                    user_id,
                    username: user.email, // Using email as username for now
                    joined_at: Utc::now(),
                }).await;
            }
            
            info!("WebSocket connection authenticated: {} for user {}", connection_id, user_id);
            Ok(())
        } else {
            Err(AppError::NotFound("Connection not found".to_string()))
        }
    }

    /// Update subscriptions for a connection
    pub async fn update_subscriptions(
        &self,
        connection_id: &Uuid,
        subscriptions: Vec<EventSubscription>,
    ) -> Result<(), AppError> {
        let mut connections = self.connections.write().await;
        if let Some(connection) = connections.get_mut(connection_id) {
            connection.subscriptions = subscriptions;
            debug!("Updated subscriptions for connection: {}", connection_id);
            Ok(())
        } else {
            Err(AppError::NotFound("Connection not found".to_string()))
        }
    }

    /// Update ping timestamp for a connection
    pub async fn update_ping(&self, connection_id: &Uuid) {
        let mut connections = self.connections.write().await;
        if let Some(connection) = connections.get_mut(connection_id) {
            connection.last_ping = Instant::now();
        }
    }

    /// Broadcast an event to all relevant connections
    pub async fn broadcast_event(&self, event: RealtimeEvent) -> Result<(), AppError> {
        let _ = self.event_sender.send(event);
        Ok(())
    }

    /// Broadcast raffle box purchase event
    pub async fn broadcast_box_purchase(
        &self,
        raffle_id: Uuid,
        user_id: Uuid,
        box_number: i32,
        boxes_remaining: i32,
        completion_percentage: f64,
    ) -> Result<(), AppError> {
        self.broadcast_event(RealtimeEvent::BoxPurchased {
            raffle_id,
            user_id,
            box_number,
            boxes_remaining,
            completion_percentage,
            purchased_at: Utc::now(),
        }).await
    }

    /// Broadcast raffle completion event
    pub async fn broadcast_raffle_full(
        &self,
        raffle_id: Uuid,
        item_id: Uuid,
        total_boxes: i32,
    ) -> Result<(), AppError> {
        self.broadcast_event(RealtimeEvent::RaffleFull {
            raffle_id,
            item_id,
            total_boxes,
            completed_at: Utc::now(),
        }).await
    }

    /// Broadcast winner selection event
    pub async fn broadcast_winner_selected(
        &self,
        raffle_id: Uuid,
        item_id: Uuid,
        winner_user_ids: Vec<Uuid>,
    ) -> Result<(), AppError> {
        self.broadcast_event(RealtimeEvent::WinnerSelected {
            raffle_id,
            item_id,
            winner_user_ids,
            completed_at: Utc::now(),
        }).await
    }

    /// Broadcast item stock change event
    pub async fn broadcast_item_stock_change(
        &self,
        item_id: Uuid,
        old_quantity: i32,
        new_quantity: i32,
    ) -> Result<(), AppError> {
        self.broadcast_event(RealtimeEvent::ItemStockChanged {
            item_id,
            old_quantity,
            new_quantity,
            updated_at: Utc::now(),
        }).await
    }

    /// Get connection statistics
    pub async fn get_connection_stats(&self) -> ConnectionStats {
        let connections = self.connections.read().await;
        let total_connections = connections.len();
        let authenticated_connections = connections.values()
            .filter(|c| c.user_id.is_some())
            .count();

        // Count active rooms
        let mut active_rooms = HashMap::new();
        for connection in connections.values() {
            for subscription in &connection.subscriptions {
                if let Some(room) = &subscription.room {
                    *active_rooms.entry(room.clone()).or_insert(0) += 1;
                }
            }
        }

        ConnectionStats {
            total_connections,
            authenticated_connections,
            active_rooms,
            events_sent_last_hour: 0, // Would track this in production
        }
    }

    /// Send message to specific user
    pub async fn send_to_user(&self, user_id: Uuid, message: WebSocketMessage) -> Result<(), AppError> {
        let connections = self.connections.read().await;
        let mut sent = false;

        for connection in connections.values() {
            if connection.user_id == Some(user_id) {
                if connection.addr.try_send(message.clone()).is_ok() {
                    sent = true;
                }
            }
        }

        if sent {
            Ok(())
        } else {
            Err(AppError::NotFound("User not connected".to_string()))
        }
    }

    /// Send message to specific room
    pub async fn send_to_room(&self, room: &str, message: WebSocketMessage) -> Result<usize, AppError> {
        let connections = self.connections.read().await;
        let mut sent_count = 0;

        for connection in connections.values() {
            let in_room = connection.subscriptions.iter()
                .any(|s| s.room.as_ref() == Some(&room.to_string()));

            if in_room {
                if connection.addr.try_send(message.clone()).is_ok() {
                    sent_count += 1;
                }
            }
        }

        Ok(sent_count)
    }

    // Private helper methods

    async fn start_event_broadcasting(&self) {
        let mut event_receiver = self.event_sender.subscribe();
        
        info!("Started realtime event broadcasting");

        while let Ok(event) = event_receiver.recv().await {
            self.process_and_broadcast_event(event).await;
        }

        warn!("Realtime event broadcasting stopped");
    }

    async fn process_and_broadcast_event(&self, event: RealtimeEvent) {
        let connections = self.connections.read().await;
        let mut failed_connections = Vec::new();

        for (connection_id, connection) in connections.iter() {
            if self.should_send_event_to_connection(connection, &event) {
                let message = self.create_websocket_message(&event);
                
                if connection.addr.try_send(message).is_err() {
                    failed_connections.push(*connection_id);
                }
            }
        }

        // Remove failed connections
        drop(connections);
        if !failed_connections.is_empty() {
            let mut connections = self.connections.write().await;
            for connection_id in failed_connections {
                connections.remove(&connection_id);
                debug!("Removed failed WebSocket connection: {}", connection_id);
            }
        }
    }

    fn should_send_event_to_connection(&self, connection: &ConnectionInfo, event: &RealtimeEvent) -> bool {
        if connection.subscriptions.is_empty() {
            return false;
        }

        for subscription in &connection.subscriptions {
            match event {
                RealtimeEvent::RaffleCreated { raffle_id, .. } => {
                    if subscription.event_type == "raffle_created" || subscription.event_type == "all" {
                        if subscription.raffle_id.is_none() || subscription.raffle_id == Some(*raffle_id) {
                            return true;
                        }
                    }
                }
                RealtimeEvent::BoxPurchased { raffle_id, user_id, .. } => {
                    if subscription.event_type == "box_purchased" || subscription.event_type == "all" {
                        if subscription.raffle_id.is_none() || subscription.raffle_id == Some(*raffle_id) {
                            return true;
                        }
                        if subscription.user_id == Some(*user_id) {
                            return true;
                        }
                    }
                }
                RealtimeEvent::WinnerSelected { raffle_id, winner_user_ids, .. } => {
                    if subscription.event_type == "winner_selected" || subscription.event_type == "all" {
                        if subscription.raffle_id.is_none() || subscription.raffle_id == Some(*raffle_id) {
                            return true;
                        }
                        if let Some(user_id) = connection.user_id {
                            if winner_user_ids.contains(&user_id) {
                                return true;
                            }
                        }
                    }
                }
                RealtimeEvent::ItemStockChanged { item_id, .. } => {
                    if subscription.event_type == "item_updated" || subscription.event_type == "all" {
                        if subscription.item_id.is_none() || subscription.item_id == Some(*item_id) {
                            return true;
                        }
                    }
                }
                RealtimeEvent::CreditsIssued { user_id, .. } |
                RealtimeEvent::CreditsRedeemed { user_id, .. } => {
                    if subscription.event_type == "credits" || subscription.event_type == "all" {
                        if connection.user_id == Some(*user_id) {
                            return true;
                        }
                    }
                }
                _ => {
                    if subscription.event_type == "all" {
                        return true;
                    }
                }
            }
        }

        false
    }

    fn create_websocket_message(&self, event: &RealtimeEvent) -> WebSocketMessage {
        let (message_type, data) = match event {
            RealtimeEvent::RaffleCreated { .. } => ("raffle_created", serde_json::to_value(event).unwrap_or_default()),
            RealtimeEvent::BoxPurchased { .. } => ("box_purchased", serde_json::to_value(event).unwrap_or_default()),
            RealtimeEvent::RaffleFull { .. } => ("raffle_full", serde_json::to_value(event).unwrap_or_default()),
            RealtimeEvent::WinnerSelected { .. } => ("winner_selected", serde_json::to_value(event).unwrap_or_default()),
            RealtimeEvent::RaffleCancelled { .. } => ("raffle_cancelled", serde_json::to_value(event).unwrap_or_default()),
            RealtimeEvent::ItemCreated { .. } => ("item_created", serde_json::to_value(event).unwrap_or_default()),
            RealtimeEvent::ItemUpdated { .. } => ("item_updated", serde_json::to_value(event).unwrap_or_default()),
            RealtimeEvent::ItemStockChanged { .. } => ("item_stock_changed", serde_json::to_value(event).unwrap_or_default()),
            RealtimeEvent::UserJoined { .. } => ("user_joined", serde_json::to_value(event).unwrap_or_default()),
            RealtimeEvent::UserLeft { .. } => ("user_left", serde_json::to_value(event).unwrap_or_default()),
            RealtimeEvent::CreditsIssued { .. } => ("credits_issued", serde_json::to_value(event).unwrap_or_default()),
            RealtimeEvent::CreditsRedeemed { .. } => ("credits_redeemed", serde_json::to_value(event).unwrap_or_default()),
            RealtimeEvent::SystemMaintenance { .. } => ("system_maintenance", serde_json::to_value(event).unwrap_or_default()),
            RealtimeEvent::SystemAlert { .. } => ("system_alert", serde_json::to_value(event).unwrap_or_default()),
        };

        WebSocketMessage {
            message_type: message_type.to_string(),
            data,
            timestamp: Utc::now(),
        }
    }

    async fn cleanup_stale_connections(&self) {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        
        loop {
            interval.tick().await;
            
            let mut connections = self.connections.write().await;
            let mut stale_connections = Vec::new();
            
            for (connection_id, connection) in connections.iter() {
                if connection.last_ping.elapsed() > Duration::from_secs(300) {
                    stale_connections.push(*connection_id);
                }
            }
            
            for connection_id in stale_connections {
                if let Some(connection) = connections.remove(&connection_id) {
                    if let Some(user_id) = connection.user_id {
                        // Broadcast user left event
                        let _ = self.event_sender.send(RealtimeEvent::UserLeft {
                            user_id,
                            left_at: Utc::now(),
                        });
                    }
                    debug!("Removed stale WebSocket connection: {}", connection_id);
                }
            }
        }
    }

    async fn validate_jwt_token(&self, token: &str) -> Result<Uuid, AppError> {
        // This would integrate with your JWT service
        // For now, return a placeholder
        use crate::utils::jwt::JwtService;
        let jwt_service = JwtService::new()?;
        let claims = jwt_service.validate_token(token)?;
        Ok(claims.user_id)
    }
}