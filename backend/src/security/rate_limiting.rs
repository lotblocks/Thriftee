use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpResponse, Result,
};
use futures_util::future::LocalBoxFuture;
use redis::{AsyncCommands, Client as RedisClient};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    future::{ready, Ready},
    rc::Rc,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tracing::{debug, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub requests_per_window: u32,
    pub window_duration: Duration,
    pub burst_limit: Option<u32>,
    pub block_duration: Option<Duration>,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_window: 100,
            window_duration: Duration::from_secs(60),
            burst_limit: Some(10),
            block_duration: Some(Duration::from_secs(300)), // 5 minutes
        }
    }
}

#[derive(Debug, Clone)]
pub struct RateLimitRule {
    pub path_pattern: String,
    pub method: Option<String>,
    pub config: RateLimitConfig,
    pub per_user: bool,
    pub per_ip: bool,
}

impl RateLimitRule {
    pub fn new(path_pattern: &str) -> Self {
        Self {
            path_pattern: path_pattern.to_string(),
            method: None,
            config: RateLimitConfig::default(),
            per_user: false,
            per_ip: true,
        }
    }

    pub fn with_method(mut self, method: &str) -> Self {
        self.method = Some(method.to_uppercase());
        self
    }

    pub fn with_config(mut self, config: RateLimitConfig) -> Self {
        self.config = config;
        self
    }

    pub fn per_user(mut self) -> Self {
        self.per_user = true;
        self
    }

    pub fn per_ip(mut self) -> Self {
        self.per_ip = true;
        self
    }

    pub fn matches(&self, path: &str, method: &str) -> bool {
        let path_matches = if self.path_pattern.contains('*') {
            // Simple wildcard matching
            let pattern = self.path_pattern.replace('*', ".*");
            regex::Regex::new(&pattern)
                .map(|re| re.is_match(path))
                .unwrap_or(false)
        } else {
            path == self.path_pattern || path.starts_with(&self.path_pattern)
        };

        let method_matches = self.method.as_ref()
            .map(|m| m == method)
            .unwrap_or(true);

        path_matches && method_matches
    }
}

#[derive(Debug)]
struct RateLimitEntry {
    count: u32,
    window_start: Instant,
    blocked_until: Option<Instant>,
}

impl RateLimitEntry {
    fn new() -> Self {
        Self {
            count: 1,
            window_start: Instant::now(),
            blocked_until: None,
        }
    }

    fn is_blocked(&self) -> bool {
        self.blocked_until
            .map(|blocked_until| Instant::now() < blocked_until)
            .unwrap_or(false)
    }

    fn should_reset_window(&self, window_duration: Duration) -> bool {
        Instant::now().duration_since(self.window_start) >= window_duration
    }

    fn increment(&mut self, config: &RateLimitConfig) -> bool {
        if self.is_blocked() {
            return false;
        }

        if self.should_reset_window(config.window_duration) {
            self.count = 1;
            self.window_start = Instant::now();
            self.blocked_until = None;
            return true;
        }

        self.count += 1;

        // Check if we've exceeded the rate limit
        if self.count > config.requests_per_window {
            if let Some(block_duration) = config.block_duration {
                self.blocked_until = Some(Instant::now() + block_duration);
            }
            return false;
        }

        // Check burst limit
        if let Some(burst_limit) = config.burst_limit {
            if self.count > burst_limit {
                return false;
            }
        }

        true
    }
}

pub trait RateLimitStore: Send + Sync {
    fn check_and_increment(
        &self,
        key: &str,
        config: &RateLimitConfig,
    ) -> LocalBoxFuture<'_, Result<bool, Error>>;
    
    fn get_remaining(
        &self,
        key: &str,
        config: &RateLimitConfig,
    ) -> LocalBoxFuture<'_, Result<u32, Error>>;
}

// In-memory rate limit store (for development/testing)
pub struct MemoryRateLimitStore {
    entries: Arc<Mutex<HashMap<String, RateLimitEntry>>>,
}

impl MemoryRateLimitStore {
    pub fn new() -> Self {
        Self {
            entries: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl RateLimitStore for MemoryRateLimitStore {
    fn check_and_increment(
        &self,
        key: &str,
        config: &RateLimitConfig,
    ) -> LocalBoxFuture<'_, Result<bool, Error>> {
        let key = key.to_string();
        let config = config.clone();
        let entries = self.entries.clone();

        Box::pin(async move {
            let mut entries = entries.lock().unwrap();
            
            let entry = entries.entry(key).or_insert_with(RateLimitEntry::new);
            let allowed = entry.increment(&config);
            
            Ok(allowed)
        })
    }

    fn get_remaining(
        &self,
        key: &str,
        config: &RateLimitConfig,
    ) -> LocalBoxFuture<'_, Result<u32, Error>> {
        let key = key.to_string();
        let config = config.clone();
        let entries = self.entries.clone();

        Box::pin(async move {
            let entries = entries.lock().unwrap();
            
            if let Some(entry) = entries.get(&key) {
                if entry.should_reset_window(config.window_duration) {
                    Ok(config.requests_per_window)
                } else {
                    Ok(config.requests_per_window.saturating_sub(entry.count))
                }
            } else {
                Ok(config.requests_per_window)
            }
        })
    }
}

// Redis-based rate limit store (for production)
pub struct RedisRateLimitStore {
    client: RedisClient,
}

impl RedisRateLimitStore {
    pub fn new(redis_url: &str) -> Result<Self, redis::RedisError> {
        let client = RedisClient::open(redis_url)?;
        Ok(Self { client })
    }
}

impl RateLimitStore for RedisRateLimitStore {
    fn check_and_increment(
        &self,
        key: &str,
        config: &RateLimitConfig,
    ) -> LocalBoxFuture<'_, Result<bool, Error>> {
        let key = format!("rate_limit:{}", key);
        let window_seconds = config.window_duration.as_secs();
        let limit = config.requests_per_window;
        let client = self.client.clone();

        Box::pin(async move {
            let mut conn = client
                .get_async_connection()
                .await
                .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

            // Use Redis sliding window log algorithm
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();

            let window_start = now - window_seconds;

            // Remove expired entries
            let _: () = conn
                .zrembyscore(&key, 0, window_start as f64)
                .await
                .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

            // Count current entries
            let current_count: u32 = conn
                .zcard(&key)
                .await
                .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

            if current_count >= limit {
                return Ok(false);
            }

            // Add current request
            let _: () = conn
                .zadd(&key, now, now)
                .await
                .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

            // Set expiration
            let _: () = conn
                .expire(&key, window_seconds as usize)
                .await
                .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

            Ok(true)
        })
    }

    fn get_remaining(
        &self,
        key: &str,
        config: &RateLimitConfig,
    ) -> LocalBoxFuture<'_, Result<u32, Error>> {
        let key = format!("rate_limit:{}", key);
        let window_seconds = config.window_duration.as_secs();
        let limit = config.requests_per_window;
        let client = self.client.clone();

        Box::pin(async move {
            let mut conn = client
                .get_async_connection()
                .await
                .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();

            let window_start = now - window_seconds;

            // Remove expired entries and count current
            let _: () = conn
                .zrembyscore(&key, 0, window_start as f64)
                .await
                .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

            let current_count: u32 = conn
                .zcard(&key)
                .await
                .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

            Ok(limit.saturating_sub(current_count))
        })
    }
}

pub struct RateLimitMiddleware<S> {
    store: Arc<dyn RateLimitStore>,
    rules: Vec<RateLimitRule>,
}

impl<S> RateLimitMiddleware<S> {
    pub fn new(store: Arc<dyn RateLimitStore>) -> Self {
        Self {
            store,
            rules: Vec::new(),
        }
    }

    pub fn with_rules(mut self, rules: Vec<RateLimitRule>) -> Self {
        self.rules = rules;
        self
    }

    pub fn add_rule(mut self, rule: RateLimitRule) -> Self {
        self.rules.push(rule);
        self
    }

    // Predefined rule sets
    pub fn with_default_api_rules(self) -> Self {
        self.add_rule(
            RateLimitRule::new("/api/auth/login")
                .with_method("POST")
                .with_config(RateLimitConfig {
                    requests_per_window: 5,
                    window_duration: Duration::from_secs(300), // 5 minutes
                    burst_limit: Some(3),
                    block_duration: Some(Duration::from_secs(900)), // 15 minutes
                })
                .per_ip(),
        )
        .add_rule(
            RateLimitRule::new("/api/auth/register")
                .with_method("POST")
                .with_config(RateLimitConfig {
                    requests_per_window: 3,
                    window_duration: Duration::from_secs(3600), // 1 hour
                    burst_limit: Some(1),
                    block_duration: Some(Duration::from_secs(3600)), // 1 hour
                })
                .per_ip(),
        )
        .add_rule(
            RateLimitRule::new("/api/raffles/*/buy-box")
                .with_method("POST")
                .with_config(RateLimitConfig {
                    requests_per_window: 10,
                    window_duration: Duration::from_secs(60),
                    burst_limit: Some(5),
                    block_duration: Some(Duration::from_secs(300)),
                })
                .per_user(),
        )
        .add_rule(
            RateLimitRule::new("/api/payments/*")
                .with_config(RateLimitConfig {
                    requests_per_window: 20,
                    window_duration: Duration::from_secs(300),
                    burst_limit: Some(5),
                    block_duration: Some(Duration::from_secs(600)),
                })
                .per_user(),
        )
        .add_rule(
            RateLimitRule::new("/api/*")
                .with_config(RateLimitConfig {
                    requests_per_window: 1000,
                    window_duration: Duration::from_secs(3600),
                    burst_limit: Some(100),
                    block_duration: None,
                })
                .per_ip(),
        )
    }

    fn get_rate_limit_key(&self, req: &ServiceRequest, rule: &RateLimitRule) -> Option<String> {
        let mut key_parts = Vec::new();

        if rule.per_ip {
            if let Some(ip) = req.connection_info().realip_remote_addr() {
                key_parts.push(format!("ip:{}", ip));
            } else {
                return None;
            }
        }

        if rule.per_user {
            // Extract user ID from request (you'll need to implement this based on your auth system)
            if let Some(user_id) = self.extract_user_id(req) {
                key_parts.push(format!("user:{}", user_id));
            } else if rule.per_ip {
                // Fall back to IP-based limiting if user not authenticated
            } else {
                return None;
            }
        }

        key_parts.push(format!("path:{}", rule.path_pattern));
        
        if let Some(method) = &rule.method {
            key_parts.push(format!("method:{}", method));
        }

        Some(key_parts.join(":"))
    }

    fn extract_user_id(&self, _req: &ServiceRequest) -> Option<String> {
        // TODO: Implement user ID extraction from JWT token or session
        // This would typically involve:
        // 1. Extracting the Authorization header
        // 2. Validating the JWT token
        // 3. Extracting the user ID from the token claims
        None
    }

    fn find_matching_rule(&self, path: &str, method: &str) -> Option<&RateLimitRule> {
        self.rules
            .iter()
            .find(|rule| rule.matches(path, method))
    }
}

impl<S, B> Transform<S, ServiceRequest> for RateLimitMiddleware<S>
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
            store: self.store.clone(),
            rules: self.rules.clone(),
        }))
    }
}

pub struct RateLimitMiddlewareService<S> {
    service: S,
    store: Arc<dyn RateLimitStore>,
    rules: Vec<RateLimitRule>,
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
        let path = req.path().to_string();
        let method = req.method().to_string();
        
        // Find matching rule
        let rule = self.rules
            .iter()
            .find(|rule| rule.matches(&path, &method))
            .cloned();

        if let Some(rule) = rule {
            let store = self.store.clone();
            let service = self.service.call(req);

            Box::pin(async move {
                // Generate rate limit key
                let key = format!("{}:{}:{}", 
                    req.connection_info().realip_remote_addr().unwrap_or("unknown"),
                    path,
                    method
                );

                // Check rate limit
                match store.check_and_increment(&key, &rule.config).await {
                    Ok(allowed) => {
                        if allowed {
                            debug!("Rate limit check passed for key: {}", key);
                            service.await
                        } else {
                            warn!("Rate limit exceeded for key: {}", key);
                            
                            let remaining = store.get_remaining(&key, &rule.config).await
                                .unwrap_or(0);
                            
                            let response = HttpResponse::TooManyRequests()
                                .insert_header(("X-RateLimit-Limit", rule.config.requests_per_window.to_string()))
                                .insert_header(("X-RateLimit-Remaining", remaining.to_string()))
                                .insert_header(("X-RateLimit-Reset", rule.config.window_duration.as_secs().to_string()))
                                .json(serde_json::json!({
                                    "error": "Rate limit exceeded",
                                    "message": "Too many requests. Please try again later.",
                                    "retry_after": rule.config.window_duration.as_secs()
                                }));
                            
                            Ok(req.into_response(response))
                        }
                    }
                    Err(e) => {
                        warn!("Rate limit check failed: {}", e);
                        // Continue with request if rate limiting fails
                        service.await
                    }
                }
            })
        } else {
            // No matching rule, proceed with request
            self.service.call(req)
        }
    }
}