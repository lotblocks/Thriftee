use actix_web::{get, HttpResponse, Result};
use serde_json::json;

#[get("/health")]
pub async fn health_check() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(json!({
        "status": "healthy",
        "service": "raffle-platform-backend",
        "version": env!("CARGO_PKG_VERSION")
    })))
}