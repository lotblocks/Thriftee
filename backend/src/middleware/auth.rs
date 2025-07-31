use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage, HttpResponse, FromRequest,
};
use futures_util::future::LocalBoxFuture;
use std::{
    future::{ready, Ready},
    rc::Rc,
};

use crate::error::AppError;
use crate::models::User;
use crate::utils::jwt::{Claims, JwtService};
use raffle_platform_shared::UserRole;

/// Authenticated user information extracted from JWT token
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user_id: uuid::Uuid,
    pub username: String,
    pub email: String,
    pub role: UserRole,
}

impl AuthenticatedUser {
    /// Create from JWT claims
    pub fn from_claims(claims: &Claims) -> Result<Self, AppError> {
        let user_id = uuid::Uuid::parse_str(&claims.sub)
            .map_err(|_| AppError::Internal("Invalid user ID in claims".to_string()))?;

        Ok(Self {
            user_id,
            username: claims.username.clone(),
            email: claims.email.clone(),
            role: claims.role,
        })
    }

    /// Check if user is admin
    pub fn is_admin(&self) -> bool {
        matches!(self.role, UserRole::Admin | UserRole::Operator)
    }

    /// Check if user is seller
    pub fn is_seller(&self) -> bool {
        matches!(self.role, UserRole::Seller | UserRole::Admin | UserRole::Operator)
    }

    /// Check if user has specific role or higher
    pub fn has_role(&self, required_role: UserRole) -> bool {
        has_required_role(&self.role, required_role)
    }
}

/// Implement FromRequest for AuthenticatedUser to use as handler parameter
impl actix_web::FromRequest for AuthenticatedUser {
    type Error = AppError;
    type Future = std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &actix_web::HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        let req = req.clone();
        Box::pin(async move {
            let claims = req
                .extensions()
                .get::<Claims>()
                .cloned()
                .ok_or_else(|| AppError::Internal("Claims not found in request".to_string()))?;

            AuthenticatedUser::from_claims(&claims)
        })
    }
}

pub struct AuthMiddleware {
    jwt_service: Rc<JwtService>,
    required_role: Option<UserRole>,
}

impl AuthMiddleware {
    pub fn new(jwt_service: JwtService) -> Self {
        Self {
            jwt_service: Rc::new(jwt_service),
            required_role: None,
        }
    }

    pub fn require_role(mut self, role: UserRole) -> Self {
        self.required_role = Some(role);
        self
    }
}

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddlewareService {
            service,
            jwt_service: self.jwt_service.clone(),
            required_role: self.required_role,
        }))
    }
}

pub struct AuthMiddlewareService<S> {
    service: S,
    jwt_service: Rc<JwtService>,
    required_role: Option<UserRole>,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
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
        let jwt_service = self.jwt_service.clone();
        let required_role = self.required_role;

        Box::pin(async move {
            // Extract Authorization header
            let auth_header = req
                .headers()
                .get("Authorization")
                .and_then(|h| h.to_str().ok())
                .and_then(|h| h.strip_prefix("Bearer "));

            let token = match auth_header {
                Some(token) => token,
                None => {
                    let response = HttpResponse::Unauthorized()
                        .json(serde_json::json!({
                            "error": "missing_token",
                            "message": "Authorization token is required"
                        }));
                    return Ok(req.into_response(response));
                }
            };

            // Validate token
            let claims = match jwt_service.validate_token(token) {
                Ok(claims) => claims,
                Err(e) => {
                    let response = HttpResponse::Unauthorized()
                        .json(serde_json::json!({
                            "error": "invalid_token",
                            "message": e.to_string()
                        }));
                    return Ok(req.into_response(response));
                }
            };

            // Check if it's an access token
            if claims.token_type != "access" {
                let response = HttpResponse::Unauthorized()
                    .json(serde_json::json!({
                        "error": "invalid_token_type",
                        "message": "Access token required"
                    }));
                return Ok(req.into_response(response));
            }

            // Check role requirements
            if let Some(required_role) = required_role {
                if !has_required_role(&claims.role, required_role) {
                    let response = HttpResponse::Forbidden()
                        .json(serde_json::json!({
                            "error": "insufficient_permissions",
                            "message": "Insufficient permissions for this operation"
                        }));
                    return Ok(req.into_response(response));
                }
            }

            // Add claims to request extensions
            req.extensions_mut().insert(claims);

            // Continue with the request
            let res = self.service.call(req).await?;
            Ok(res)
        })
    }
}

/// Check if user has required role or higher
fn has_required_role(user_role: &UserRole, required_role: UserRole) -> bool {
    match required_role {
        UserRole::User => true, // All roles can access user-level endpoints
        UserRole::Seller => matches!(user_role, UserRole::Seller | UserRole::Admin | UserRole::Operator),
        UserRole::Admin => matches!(user_role, UserRole::Admin | UserRole::Operator),
        UserRole::Operator => matches!(user_role, UserRole::Operator),
    }
}

/// Extract claims from request extensions
pub fn extract_claims(req: &ServiceRequest) -> Result<Claims, AppError> {
    req.extensions()
        .get::<Claims>()
        .cloned()
        .ok_or_else(|| AppError::Internal("Claims not found in request".to_string()))
}

/// Extract user ID from request
pub fn extract_user_id(req: &ServiceRequest) -> Result<uuid::Uuid, AppError> {
    let claims = extract_claims(req)?;
    uuid::Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Internal("Invalid user ID in claims".to_string()))
}

/// Extract user role from request
pub fn extract_user_role(req: &ServiceRequest) -> Result<UserRole, AppError> {
    let claims = extract_claims(req)?;
    Ok(claims.role)
}

/// Optional authentication middleware (doesn't fail if no token provided)
pub struct OptionalAuthMiddleware {
    jwt_service: Rc<JwtService>,
}

impl OptionalAuthMiddleware {
    pub fn new(jwt_service: JwtService) -> Self {
        Self {
            jwt_service: Rc::new(jwt_service),
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for OptionalAuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = OptionalAuthMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(OptionalAuthMiddlewareService {
            service,
            jwt_service: self.jwt_service.clone(),
        }))
    }
}

pub struct OptionalAuthMiddlewareService<S> {
    service: S,
    jwt_service: Rc<JwtService>,
}

impl<S, B> Service<ServiceRequest> for OptionalAuthMiddlewareService<S>
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
        let jwt_service = self.jwt_service.clone();

        Box::pin(async move {
            // Try to extract and validate token, but don't fail if missing
            if let Some(auth_header) = req
                .headers()
                .get("Authorization")
                .and_then(|h| h.to_str().ok())
                .and_then(|h| h.strip_prefix("Bearer "))
            {
                if let Ok(claims) = jwt_service.validate_token(auth_header) {
                    if claims.token_type == "access" {
                        req.extensions_mut().insert(claims);
                    }
                }
            }

            // Continue with the request regardless of authentication status
            let res = self.service.call(req).await?;
            Ok(res)
        })
    }
}

/// Enhanced authentication middleware with security features
pub struct SecureAuthMiddleware {
    jwt_service: Rc<JwtService>,
    pool: Arc<sqlx::PgPool>,
    required_role: Option<UserRole>,
    enable_security_checks: bool,
}

impl SecureAuthMiddleware {
    pub fn new(jwt_service: JwtService, pool: Arc<sqlx::PgPool>) -> Self {
        Self {
            jwt_service: Rc::new(jwt_service),
            pool,
            required_role: None,
            enable_security_checks: true,
        }
    }

    pub fn require_role(mut self, role: UserRole) -> Self {
        self.required_role = Some(role);
        self
    }

    pub fn disable_security_checks(mut self) -> Self {
        self.enable_security_checks = false;
        self
    }
}

impl<S, B> Transform<S, ServiceRequest> for SecureAuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = SecureAuthMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(SecureAuthMiddlewareService {
            service,
            jwt_service: self.jwt_service.clone(),
            pool: self.pool.clone(),
            required_role: self.required_role,
            enable_security_checks: self.enable_security_checks,
        }))
    }
}

pub struct SecureAuthMiddlewareService<S> {
    service: S,
    jwt_service: Rc<JwtService>,
    pool: Arc<sqlx::PgPool>,
    required_role: Option<UserRole>,
    enable_security_checks: bool,
}

impl<S, B> Service<ServiceRequest> for SecureAuthMiddlewareService<S>
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
        let jwt_service = self.jwt_service.clone();
        let pool = self.pool.clone();
        let required_role = self.required_role;
        let enable_security_checks = self.enable_security_checks;

        Box::pin(async move {
            // Extract Authorization header
            let auth_header = req
                .headers()
                .get("Authorization")
                .and_then(|h| h.to_str().ok())
                .and_then(|h| h.strip_prefix("Bearer "));

            let token = match auth_header {
                Some(token) => token,
                None => {
                    let response = HttpResponse::Unauthorized()
                        .json(serde_json::json!({
                            "error": "missing_token",
                            "message": "Authorization token is required"
                        }));
                    return Ok(req.into_response(response));
                }
            };

            // Extract client information for security checks
            let ip_address = req.connection_info().realip_remote_addr()
                .and_then(|ip| ip.parse().ok());
            let user_agent = req.headers()
                .get("User-Agent")
                .and_then(|h| h.to_str().ok());

            // Validate token with optional security checks
            let claims = if enable_security_checks {
                match jwt_service.validate_token_with_security_checks(
                    token,
                    &pool,
                    ip_address,
                    user_agent,
                ).await {
                    Ok(claims) => claims,
                    Err(e) => {
                        let response = HttpResponse::Unauthorized()
                            .json(serde_json::json!({
                                "error": "invalid_token",
                                "message": e.to_string()
                            }));
                        return Ok(req.into_response(response));
                    }
                }
            } else {
                match jwt_service.validate_token(token) {
                    Ok(claims) => claims,
                    Err(e) => {
                        let response = HttpResponse::Unauthorized()
                            .json(serde_json::json!({
                                "error": "invalid_token",
                                "message": e.to_string()
                            }));
                        return Ok(req.into_response(response));
                    }
                }
            };

            // Check if it's an access token
            if claims.token_type != "access" {
                let response = HttpResponse::Unauthorized()
                    .json(serde_json::json!({
                        "error": "invalid_token_type",
                        "message": "Access token required"
                    }));
                return Ok(req.into_response(response));
            }

            // Check role requirements
            if let Some(required_role) = required_role {
                if !has_required_role(&claims.role, required_role) {
                    let response = HttpResponse::Forbidden()
                        .json(serde_json::json!({
                            "error": "insufficient_permissions",
                            "message": "Insufficient permissions for this operation"
                        }));
                    return Ok(req.into_response(response));
                }
            }

            // Add claims to request extensions
            req.extensions_mut().insert(claims);

            // Continue with the request
            let res = self.service.call(req).await?;
            Ok(res)
        })
    }
}

/// Rate limiting middleware for authentication endpoints
pub struct RateLimitMiddleware {
    max_requests: u32,
    window_seconds: u64,
    // In production, you'd use Redis or another distributed cache
    request_counts: Arc<std::sync::Mutex<std::collections::HashMap<String, (u32, std::time::Instant)>>>,
}

impl RateLimitMiddleware {
    pub fn new(max_requests: u32, window_seconds: u64) -> Self {
        Self {
            max_requests,
            window_seconds,
            request_counts: Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
        }
    }

    fn is_rate_limited(&self, client_id: &str) -> bool {
        let mut counts = self.request_counts.lock().unwrap();
        let now = std::time::Instant::now();
        
        // Clean up old entries
        counts.retain(|_, (_, timestamp)| {
            now.duration_since(*timestamp).as_secs() < self.window_seconds
        });

        // Check current count
        match counts.get_mut(client_id) {
            Some((count, timestamp)) => {
                if now.duration_since(*timestamp).as_secs() < self.window_seconds {
                    *count += 1;
                    *count > self.max_requests
                } else {
                    *count = 1;
                    *timestamp = now;
                    false
                }
            }
            None => {
                counts.insert(client_id.to_string(), (1, now));
                false
            }
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for RateLimitMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = RateLimitMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RateLimitMiddlewareService {
            service,
            max_requests: self.max_requests,
            window_seconds: self.window_seconds,
            request_counts: self.request_counts.clone(),
        }))
    }
}

pub struct RateLimitMiddlewareService<S> {
    service: S,
    max_requests: u32,
    window_seconds: u64,
    request_counts: Arc<std::sync::Mutex<std::collections::HashMap<String, (u32, std::time::Instant)>>>,
}

impl<S, B> Service<ServiceRequest> for RateLimitMiddlewareService<S>
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
        let max_requests = self.max_requests;
        let window_seconds = self.window_seconds;
        let request_counts = self.request_counts.clone();

        Box::pin(async move {
            // Use IP address as client identifier
            let client_id = req.connection_info().realip_remote_addr()
                .unwrap_or("unknown")
                .to_string();

            // Check rate limit
            let is_limited = {
                let mut counts = request_counts.lock().unwrap();
                let now = std::time::Instant::now();
                
                // Clean up old entries
                counts.retain(|_, (_, timestamp)| {
                    now.duration_since(*timestamp).as_secs() < window_seconds
                });

                // Check current count
                match counts.get_mut(&client_id) {
                    Some((count, timestamp)) => {
                        if now.duration_since(*timestamp).as_secs() < window_seconds {
                            *count += 1;
                            *count > max_requests
                        } else {
                            *count = 1;
                            *timestamp = now;
                            false
                        }
                    }
                    None => {
                        counts.insert(client_id.clone(), (1, now));
                        false
                    }
                }
            };

            if is_limited {
                let response = HttpResponse::TooManyRequests()
                    .json(serde_json::json!({
                        "error": "rate_limit_exceeded",
                        "message": format!("Rate limit exceeded. Maximum {} requests per {} seconds", max_requests, window_seconds)
                    }));
                return Ok(req.into_response(response));
            }

            // Continue with the request
            let res = self.service.call(req).await?;
            Ok(res)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::jwt::JwtService;
    use actix_web::{test, web, App, HttpResponse};
    use std::env;
    use uuid::Uuid;

    async fn test_handler() -> Result<HttpResponse, Error> {
        Ok(HttpResponse::Ok().json(serde_json::json!({"message": "success"})))
    }

    fn setup_jwt_service() -> JwtService {
        env::set_var("JWT_SECRET", "test-secret-key-for-testing-only");
        JwtService::new().expect("Failed to create JWT service")
    }

    #[actix_web::test]
    async fn test_auth_middleware_no_token() {
        let jwt_service = setup_jwt_service();
        let app = test::init_service(
            App::new()
                .wrap(AuthMiddleware::new(jwt_service))
                .route("/test", web::get().to(test_handler)),
        )
        .await;

        let req = test::TestRequest::get().uri("/test").to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), 401);
    }

    #[actix_web::test]
    async fn test_auth_middleware_valid_token() {
        let jwt_service = setup_jwt_service();
        let user_id = Uuid::new_v4();
        let token = jwt_service
            .generate_access_token(
                user_id,
                "testuser".to_string(),
                "test@example.com".to_string(),
                UserRole::User,
            )
            .expect("Failed to generate token");

        let app = test::init_service(
            App::new()
                .wrap(AuthMiddleware::new(jwt_service))
                .route("/test", web::get().to(test_handler)),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/test")
            .insert_header(("Authorization", format!("Bearer {}", token)))
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), 200);
    }

    #[actix_web::test]
    async fn test_auth_middleware_role_check() {
        let jwt_service = setup_jwt_service();
        let user_id = Uuid::new_v4();
        let token = jwt_service
            .generate_access_token(
                user_id,
                "testuser".to_string(),
                "test@example.com".to_string(),
                UserRole::User,
            )
            .expect("Failed to generate token");

        let app = test::init_service(
            App::new()
                .wrap(AuthMiddleware::new(jwt_service).require_role(UserRole::Admin))
                .route("/admin", web::get().to(test_handler)),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/admin")
            .insert_header(("Authorization", format!("Bearer {}", token)))
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), 403); // Forbidden - user doesn't have admin role
    }

    #[test]
    fn test_role_hierarchy() {
        assert!(has_required_role(&UserRole::User, UserRole::User));
        assert!(has_required_role(&UserRole::Seller, UserRole::User));
        assert!(has_required_role(&UserRole::Admin, UserRole::User));
        assert!(has_required_role(&UserRole::Operator, UserRole::User));

        assert!(!has_required_role(&UserRole::User, UserRole::Seller));
        assert!(has_required_role(&UserRole::Seller, UserRole::Seller));
        assert!(has_required_role(&UserRole::Admin, UserRole::Seller));
        assert!(has_required_role(&UserRole::Operator, UserRole::Seller));

        assert!(!has_required_role(&UserRole::User, UserRole::Admin));
        assert!(!has_required_role(&UserRole::Seller, UserRole::Admin));
        assert!(has_required_role(&UserRole::Admin, UserRole::Admin));
        assert!(has_required_role(&UserRole::Operator, UserRole::Admin));

        assert!(!has_required_role(&UserRole::User, UserRole::Operator));
        assert!(!has_required_role(&UserRole::Seller, UserRole::Operator));
        assert!(!has_required_role(&UserRole::Admin, UserRole::Operator));
        assert!(has_required_role(&UserRole::Operator, UserRole::Operator));
    }
}