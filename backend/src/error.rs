use actix_web::{HttpResponse, ResponseError};
use std::fmt;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Configuration error: {0}")]
    Config(#[from] config::ConfigError),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("JWT error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Authentication error: {0}")]
    Authentication(String),
    
    #[error("Authorization error: {0}")]
    Authorization(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Conflict: {0}")]
    Conflict(String),
    
    #[error("Internal server error: {0}")]
    Internal(String),
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        match self {
            AppError::Validation(msg) => HttpResponse::BadRequest().json(ErrorResponse {
                error: "validation_error".to_string(),
                message: msg.clone(),
            }),
            AppError::Authentication(msg) => HttpResponse::Unauthorized().json(ErrorResponse {
                error: "authentication_error".to_string(),
                message: msg.clone(),
            }),
            AppError::Authorization(msg) => HttpResponse::Forbidden().json(ErrorResponse {
                error: "authorization_error".to_string(),
                message: msg.clone(),
            }),
            AppError::NotFound(msg) => HttpResponse::NotFound().json(ErrorResponse {
                error: "not_found".to_string(),
                message: msg.clone(),
            }),
            AppError::Conflict(msg) => HttpResponse::Conflict().json(ErrorResponse {
                error: "conflict".to_string(),
                message: msg.clone(),
            }),
            _ => HttpResponse::InternalServerError().json(ErrorResponse {
                error: "internal_server_error".to_string(),
                message: "An internal server error occurred".to_string(),
            }),
        }
    }
}

#[derive(serde::Serialize)]
struct ErrorResponse {
    error: String,
    message: String,
}