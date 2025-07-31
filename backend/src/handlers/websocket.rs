use crate::services::realtime_service::{RealtimeService, WebSocketMessage, ClientMessage, SubscribeMessage, AuthMessage, EventSubscription};
use actix::prelude::*;
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// WebSocket connection manager for real-time event broadcasting
pub type WebSocketManager = RealtimeService;

// WebSocketManager is now an alias for RealtimeService

/// WebSocket actor for handling individual connections
pub struct WebSocketActor {
    id: Uuid,
    manager: Arc<RealtimeService>,
    hb: Instant,
}

impl WebSocketActor {
    pub fn new(manager: Arc<RealtimeService>) -> Self {
        Self {
            id: Uuid::new_v4(),
            manager,
            hb: Instant::now(),
        }
    }

    fn hb(&self, ctx: &mut <Self as Actor>::Context) {
        ctx.run_interval(Duration::from_secs(30), |act, ctx| {
            if Instant::now().duration_since(act.hb) > Duration::from_secs(60) {
                info!("WebSocket heartbeat failed, disconnecting: {}", act.id);
                ctx.stop();
                return;
            }

            ctx.ping(b"");
        });
    }

    fn handle_client_message(&self, msg: &str, ctx: &mut ws::WebsocketContext<Self>) {
        match serde_json::from_str::<ClientMessage>(msg) {
            Ok(client_msg) => {
                match client_msg.message_type.as_str() {
                    "subscribe" => {
                        if let Ok(subscribe_msg) = serde_json::from_value::<SubscribeMessage>(client_msg.data) {
                            let manager = self.manager.clone();
                            let connection_id = self.id;
                            
                            ctx.spawn(async move {
                                if let Err(e) = manager.update_subscriptions(&connection_id, subscribe_msg.subscriptions).await {
                                    error!("Failed to update subscriptions: {}", e);
                                }
                            }.into_actor(self));
                        }
                    }
                    "auth" => {
                        if let Ok(auth_msg) = serde_json::from_value::<AuthMessage>(client_msg.data) {
                            let manager = self.manager.clone();
                            let connection_id = self.id;
                            
                            ctx.spawn(async move {
                                if let Err(e) = manager.authenticate_connection(&connection_id, &auth_msg.token).await {
                                    error!("Failed to authenticate connection: {}", e);
                                }
                            }.into_actor(self));
                        }
                    }
                    "ping" => {
                        let manager = self.manager.clone();
                        let connection_id = self.id;
                        
                        ctx.spawn(async move {
                            manager.update_ping(&connection_id).await;
                        }.into_actor(self));
                        
                        // Send pong response
                        let pong_msg = WebSocketMessage {
                            message_type: "pong".to_string(),
                            data: serde_json::json!({"timestamp": chrono::Utc::now().timestamp()}),
                            timestamp: chrono::Utc::now(),
                        };
                        
                        if let Ok(json) = serde_json::to_string(&pong_msg) {
                            ctx.text(json);
                        }
                    }
                    _ => {
                        warn!("Unknown message type: {}", client_msg.message_type);
                    }
                }
            }
            Err(e) => {
                warn!("Failed to parse client message: {}", e);
            }
        }
    }
}

impl Actor for WebSocketActor {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);
        
        let manager = self.manager.clone();
        let connection_id = self.id;
        let addr = ctx.address().recipient();
        
        ctx.spawn(async move {
            manager.add_connection(connection_id, addr, None, None).await;
        }.into_actor(self));
        
        info!("WebSocket connection started: {}", self.id);
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        let manager = self.manager.clone();
        let connection_id = self.id;
        
        tokio::spawn(async move {
            manager.remove_connection(&connection_id).await;
        });
        
        info!("WebSocket connection stopped: {}", self.id);
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WebSocketActor {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.hb = Instant::now();
            }
            Ok(ws::Message::Text(text)) => {
                self.hb = Instant::now();
                self.handle_client_message(&text, ctx);
            }
            Ok(ws::Message::Binary(_)) => {
                warn!("Binary messages not supported");
            }
            Ok(ws::Message::Close(reason)) => {
                info!("WebSocket connection closed: {:?}", reason);
                ctx.stop();
            }
            _ => ctx.stop(),
        }
    }
}

impl Handler<WebSocketMessage> for WebSocketActor {
    type Result = ();

    fn handle(&mut self, msg: WebSocketMessage, ctx: &mut Self::Context) {
        if let Ok(json) = serde_json::to_string(&msg) {
            ctx.text(json);
        }
    }
}

/// WebSocket endpoint handler
pub async fn websocket_handler(
    req: HttpRequest,
    stream: web::Payload,
    manager: web::Data<Arc<RealtimeService>>,
) -> Result<HttpResponse, Error> {
    let actor = WebSocketActor::new(manager.get_ref().clone());
    ws::start(actor, &req, stream)
}

/// Get WebSocket connection statistics
pub async fn websocket_stats(
    manager: web::Data<Arc<RealtimeService>>,
) -> Result<HttpResponse, Error> {
    let stats = manager.get_connection_stats().await;
    Ok(HttpResponse::Ok().json(stats))
}