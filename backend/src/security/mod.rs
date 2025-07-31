pub mod audit_logging;
pub mod input_validation;
pub mod rate_limiting;
pub mod encryption;
pub mod session_management;
pub mod monitoring;

use actix_web::web;
use sqlx::PgPool;
use std::sync::Arc;

use audit_logging::AuditLogger;
use input_validation::InputValidator;
use rate_limiting::{RateLimitMiddleware, MemoryRateLimitStore, RedisRateLimitStore, RateLimitStore};

pub struct SecurityConfig {
    pub audit_logger: Arc<AuditLogger>,
    pub input_validator: Arc<InputValidator>,
    pub rate_limit_store: Arc<dyn RateLimitStore>,
    pub enable_audit_logging: bool,
    pub enable_rate_limiting: bool,
    pub enable_input_validation: bool,
    pub max_request_size: usize,
    pub session_timeout: std::time::Duration,
}

impl SecurityConfig {
    pub fn new(pool: PgPool, redis_url: Option<String>) -> Self {
        let audit_logger = Arc::new(AuditLogger::new(pool));
        let input_validator = Arc::new(InputValidator::new());
        
        let rate_limit_store: Arc<dyn RateLimitStore> = if let Some(redis_url) = redis_url {
            match RedisRateLimitStore::new(&redis_url) {
                Ok(store) => Arc::new(store),
                Err(e) => {
                    tracing::warn!("Failed to connect to Redis for rate limiting: {}. Falling back to memory store.", e);
                    Arc::new(MemoryRateLimitStore::new())
                }
            }
        } else {
            Arc::new(MemoryRateLimitStore::new())
        };

        Self {
            audit_logger,
            input_validator,
            rate_limit_store,
            enable_audit_logging: true,
            enable_rate_limiting: true,
            enable_input_validation: true,
            max_request_size: 10 * 1024 * 1024, // 10MB
            session_timeout: std::time::Duration::from_secs(3600), // 1 hour
        }
    }

    pub fn configure_services(&self, cfg: &mut web::ServiceConfig) {
        // Add security-related services to the application
        cfg.app_data(web::Data::new(self.audit_logger.clone()))
           .app_data(web::Data::new(self.input_validator.clone()));
    }

    pub fn create_rate_limit_middleware(&self) -> RateLimitMiddleware<()> {
        RateLimitMiddleware::new(self.rate_limit_store.clone())
            .with_default_api_rules()
    }
}

// Security headers middleware
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures_util::future::LocalBoxFuture;
use std::{
    future::{ready, Ready},
    rc::Rc,
};

pub struct SecurityHeaders;

impl<S, B> Transform<S, ServiceRequest> for SecurityHeaders
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = SecurityHeadersMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(SecurityHeadersMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct SecurityHeadersMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for SecurityHeadersMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();

        Box::pin(async move {
            let mut res = service.call(req).await?;

            // Add security headers
            let headers = res.headers_mut();
            
            // Prevent XSS attacks
            headers.insert(
                actix_web::http::header::HeaderName::from_static("x-xss-protection"),
                actix_web::http::HeaderValue::from_static("1; mode=block"),
            );
            
            // Prevent MIME type sniffing
            headers.insert(
                actix_web::http::header::HeaderName::from_static("x-content-type-options"),
                actix_web::http::HeaderValue::from_static("nosniff"),
            );
            
            // Prevent clickjacking
            headers.insert(
                actix_web::http::header::HeaderName::from_static("x-frame-options"),
                actix_web::http::HeaderValue::from_static("DENY"),
            );
            
            // Content Security Policy
            headers.insert(
                actix_web::http::header::HeaderName::from_static("content-security-policy"),
                actix_web::http::HeaderValue::from_static(
                    "default-src 'self'; script-src 'self' 'unsafe-inline' https://js.stripe.com; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:; connect-src 'self' https://api.stripe.com; frame-src https://js.stripe.com;"
                ),
            );
            
            // Strict Transport Security (HTTPS only)
            headers.insert(
                actix_web::http::header::HeaderName::from_static("strict-transport-security"),
                actix_web::http::HeaderValue::from_static("max-age=31536000; includeSubDomains"),
            );
            
            // Referrer Policy
            headers.insert(
                actix_web::http::header::HeaderName::from_static("referrer-policy"),
                actix_web::http::HeaderValue::from_static("strict-origin-when-cross-origin"),
            );
            
            // Permissions Policy
            headers.insert(
                actix_web::http::header::HeaderName::from_static("permissions-policy"),
                actix_web::http::HeaderValue::from_static(
                    "geolocation=(), microphone=(), camera=(), payment=(self)"
                ),
            );

            Ok(res)
        })
    }
}