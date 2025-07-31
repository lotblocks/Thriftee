use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage, HttpResponse,
};
use futures_util::future::LocalBoxFuture;
use redis::{AsyncCommands, Client as RedisClient};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    future::{ready, Ready},
    rc::Rc,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::sync::RwLock;
use tracing::{error, warn};

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
            burst_limit: Some(20),
            block_duration: Some(Duration::from_secs(300)), // 5 minutes
        }
    }
}

#[derive(Debug, Clone)]
pub struct RateLimitRule {
    pub path_pattern: String,
    pub method: Option<String>,
    pub config: RateLimitConfig,
    pub user_specific: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct RateLimitEntry {
    count: u32,
    window_start: u64,
    blocked_until: Option<u64>,
    burst_count: u32,
    last_request: u64,
}

impl Default for RateLimitEntry {
    fn default() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        Self {
            count: 0,
            window_start: now,
            blocked_until: None,
            burst_count: 0,
            last_request: now,
        }
    }
}

pub struct RateLimiter {
    redis_client: Option<RedisClient>,
    memory_store: Arc<RwLock<HashMap<String, RateLimitEntry>>>,
    rules: Vec<RateLimitRule>,
    default_config: RateLimitConfig,
}

impl RateLimiter {
    pub fn new(redis_url: Option<String>) -> Self {
        let redis_client = redis_url.and_then(|url| {
            RedisClient::open(url).ok()
        });

        Self {
            redis_client,
            memory_store: Arc::new(RwLock::new(HashMap::new())),
            rules: Vec::new(),
            default_config: RateLimitConfig::default(),
        }
    }

    pub fn add_rule(mut self, rule: RateLimitRule) -> Self {
        self.rules.push(rule);
        self
    }

    pub fn with_default_config(mut self, config: RateLimitConfig) -> Self {
        self.default_config = config;
        self
    }

    fn find_matching_rule(&self, path: &str, method: &str) -> Option<&RateLimitRule> {
        self.rules.iter().find(|rule| {
            let path_matches = if rule.path_pattern.contains('*') {
                // Simple wildcard matching
                let pattern = rule.path_pattern.replace('*', ".*");
                regex::Regex::new(&pattern)
                    .map(|re| re.is_match(path))
                    .unwrap_or(false)
            } else {
                path == rule.path_pattern || path.starts_with(&rule.path_pattern)
            };

            let method_matches = rule.method.as_ref()
                .map(|m| m.eq_ignore_ascii_case(method))
                .unwrap_or(true);

            path_matches && method_matches
        })
    }

    async fn get_rate_limit_entry(&self, key: &str) -> Result<RateLimitEntry, Error> {
        if let Some(ref redis_client) = self.redis_client {
            match redis_client.get_async_connection().await {
                Ok(mut conn) => {
                    let data: Option<String> = conn.get(key).await.unwrap_or(None);
                    if let Some(data) = data {
                        if let Ok(entry) = serde_json::from_str::<RateLimitEntry>(&data) {
                            return Ok(entry);
                        }
                    }
                }
                Err(e) => {
                    error!("Redis connection error: {}", e);
                }
            }
        }

        // Fallback to memory store
        let store = self.memory_store.read().await;
        Ok(store.get(key).cloned().unwrap_or_default())
    }

    async fn set_rate_limit_entry(&self, key: &str, entry: &RateLimitEntry, ttl: Duration) -> Result<(), Error> {
        if let Some(ref redis_client) = self.redis_client {
            match redis_client.get_async_connection().await {
                Ok(mut conn) => {
                    let data = serde_json::to_string(entry).unwrap();
                    let _: () = conn.setex(key, ttl.as_secs() as usize, data).await.unwrap_or(());
                    return Ok(());
                }
                Err(e) => {
                    error!("Redis connection error: {}", e);
                }
            }
        }

        // Fallback to memory store
        let mut store = self.memory_store.write().await;
        store.insert(key.to_string(), entry.clone());
        Ok(())
    }

    async fn check_rate_limit(
        &self,
        key: &str,
        config: &RateLimitConfig,
    ) -> Result<(bool, RateLimitHeaders), Error> {
        let mut entry = self.get_rate_limit_entry(key).await?;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Check if currently blocked
        if let Some(blocked_until) = entry.blocked_until {
            if now < blocked_until {
                return Ok((false, RateLimitHeaders {
                    limit: config.requests_per_window,
                    remaining: 0,
                    reset: blocked_until,
                    retry_after: Some(blocked_until - now),
                }));
            } else {
                entry.blocked_until = None;
            }
        }

        // Check if we need to reset the window
        if now >= entry.window_start + config.window_duration.as_secs() {
            entry.count = 0;
            entry.window_start = now;
            entry.burst_count = 0;
        }

        // Check burst limit
        if let Some(burst_limit) = config.burst_limit {
            let time_since_last = now - entry.last_request;
            if time_since_last < 1 {
                entry.burst_count += 1;
                if entry.burst_count > burst_limit {
                    // Apply temporary block
                    if let Some(block_duration) = config.block_duration {
                        entry.blocked_until = Some(now + block_duration.as_secs());
                        warn!("Rate limit burst exceeded for key: {}", key);
                    }
                    
                    self.set_rate_limit_entry(key, &entry, config.window_duration).await?;
                    return Ok((false, RateLimitHeaders {
                        limit: config.requests_per_window,
                        remaining: 0,
                        reset: entry.window_start + config.window_duration.as_secs(),
                        retry_after: entry.blocked_until.map(|b| b - now),
                    }));
                }
            } else {
                entry.burst_count = 0;
            }
        }

        // Check main rate limit
        entry.count += 1;
        entry.last_request = now;

        let allowed = entry.count <= config.requests_per_window;
        let remaining = if allowed {
            config.requests_per_window - entry.count
        } else {
            0
        };

        if !allowed {
            warn!("Rate limit exceeded for key: {} (count: {})", key, entry.count);
            
            // Apply block if configured
            if let Some(block_duration) = config.block_duration {
                entry.blocked_until = Some(now + block_duration.as_secs());
            }
        }

        self.set_rate_limit_entry(key, &entry, config.window_duration).await?;

        Ok((allowed, RateLimitHeaders {
            limit: config.requests_per_window,
            remaining,
            reset: entry.window_start + config.window_duration.as_secs(),
            retry_after: if !allowed { entry.blocked_until.map(|b| b - now) } else { None },
        }))
    }

    fn get_client_key(&self, req: &ServiceRequest, rule: Option<&RateLimitRule>) -> String {
        let user_specific = rule.map(|r| r.user_specific).unwrap_or(false);
        
        if user_specific {
            if let Some(user_id) = req.extensions().get::<String>() {
                return format!("rate_limit:user:{}:{}", user_id, req.path());
            }
        }

        // Use IP address as fallback
        let ip = req
            .connection_info()
            .realip_remote_addr()
            .unwrap_or("unknown")
            .to_string();
        
        format!("rate_limit:ip:{}:{}", ip, req.path())
    }
}

#[derive(Debug)]
struct RateLimitHeaders {
    limit: u32,
    remaining: u32,
    reset: u64,
    retry_after: Option<u64>,
}

pub struct RateLimitMiddleware<S> {
    service: Rc<S>,
    limiter: Arc<RateLimiter>,
}

impl<S, B> Service<ServiceRequest> for RateLimitMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let limiter = self.limiter.clone();

        Box::pin(async move {
            let path = req.path();
            let method = req.method().as_str();
            
            let rule = limiter.find_matching_rule(path, method);
            let config = rule.map(|r| &r.config).unwrap_or(&limiter.default_config);
            let key = limiter.get_client_key(&req, rule);

            match limiter.check_rate_limit(&key, config).await {
                Ok((allowed, headers)) => {
                    if allowed {
                        let mut res = service.call(req).await?;
                        
                        // Add rate limit headers
                        let response_headers = res.headers_mut();
                        response_headers.insert(
                            actix_web::http::header::HeaderName::from_static("x-ratelimit-limit"),
                            actix_web::http::header::HeaderValue::from(headers.limit),
                        );
                        response_headers.insert(
                            actix_web::http::header::HeaderName::from_static("x-ratelimit-remaining"),
                            actix_web::http::header::HeaderValue::from(headers.remaining),
                        );
                        response_headers.insert(
                            actix_web::http::header::HeaderName::from_static("x-ratelimit-reset"),
                            actix_web::http::header::HeaderValue::from(headers.reset),
                        );

                        Ok(res)
                    } else {
                        let mut response = HttpResponse::TooManyRequests()
                            .insert_header(("X-RateLimit-Limit", headers.limit.to_string()))
                            .insert_header(("X-RateLimit-Remaining", "0"))
                            .insert_header(("X-RateLimit-Reset", headers.reset.to_string()));

                        if let Some(retry_after) = headers.retry_after {
                            response = response.insert_header(("Retry-After", retry_after.to_string()));
                        }

                        Ok(req.into_response(
                            response.json(serde_json::json!({
                                "error": "Rate limit exceeded",
                                "message": "Too many requests. Please try again later.",
                                "retry_after": headers.retry_after
                            }))
                        ))
                    }
                }
                Err(e) => {
                    error!("Rate limiting error: {}", e);
                    // Allow request on rate limiter error
                    service.call(req).await
                }
            }
        })
    }
}

pub struct RateLimitFactory {
    limiter: Arc<RateLimiter>,
}

impl RateLimitFactory {
    pub fn new(limiter: RateLimiter) -> Self {
        Self {
            limiter: Arc::new(limiter),
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for RateLimitFactory
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = RateLimitMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RateLimitMiddleware {
            service: Rc::new(service),
            limiter: self.limiter.clone(),
        }))
    }
}

// DDoS Protection middleware
pub struct DDoSProtection {
    redis_client: Option<RedisClient>,
    memory_store: Arc<RwLock<HashMap<String, DDoSEntry>>>,
    config: DDoSConfig,
}

#[derive(Debug, Clone)]
pub struct DDoSConfig {
    pub suspicious_threshold: u32,
    pub block_threshold: u32,
    pub time_window: Duration,
    pub block_duration: Duration,
    pub check_user_agent: bool,
    pub check_referrer: bool,
}

impl Default for DDoSConfig {
    fn default() -> Self {
        Self {
            suspicious_threshold: 50,
            block_threshold: 100,
            time_window: Duration::from_secs(60),
            block_duration: Duration::from_secs(3600), // 1 hour
            check_user_agent: true,
            check_referrer: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DDoSEntry {
    request_count: u32,
    suspicious_patterns: u32,
    window_start: u64,
    blocked_until: Option<u64>,
    user_agents: Vec<String>,
    paths: Vec<String>,
}

impl DDoSProtection {
    pub fn new(redis_url: Option<String>, config: DDoSConfig) -> Self {
        let redis_client = redis_url.and_then(|url| {
            RedisClient::open(url).ok()
        });

        Self {
            redis_client,
            memory_store: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    async fn analyze_request(&self, req: &ServiceRequest) -> bool {
        let ip = req.connection_info().realip_remote_addr().unwrap_or("unknown");
        let user_agent = req.headers()
            .get("user-agent")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("");
        let path = req.path();

        let key = format!("ddos:{}", ip);
        let mut entry = self.get_ddos_entry(&key).await.unwrap_or_default();
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Check if currently blocked
        if let Some(blocked_until) = entry.blocked_until {
            if now < blocked_until {
                return false;
            } else {
                entry.blocked_until = None;
            }
        }

        // Reset window if needed
        if now >= entry.window_start + self.config.time_window.as_secs() {
            entry = DDoSEntry {
                request_count: 0,
                suspicious_patterns: 0,
                window_start: now,
                blocked_until: None,
                user_agents: Vec::new(),
                paths: Vec::new(),
            };
        }

        entry.request_count += 1;

        // Check for suspicious patterns
        if self.config.check_user_agent {
            if user_agent.is_empty() || 
               user_agent.contains("bot") || 
               user_agent.contains("crawler") ||
               user_agent.len() < 10 {
                entry.suspicious_patterns += 1;
            }
            
            if !entry.user_agents.contains(&user_agent.to_string()) {
                entry.user_agents.push(user_agent.to_string());
                if entry.user_agents.len() > 10 {
                    entry.suspicious_patterns += 5;
                }
            }
        }

        if !entry.paths.contains(&path.to_string()) {
            entry.paths.push(path.to_string());
            if entry.paths.len() > 20 {
                entry.suspicious_patterns += 3;
            }
        }

        // Check thresholds
        let is_suspicious = entry.suspicious_patterns >= self.config.suspicious_threshold;
        let should_block = entry.request_count >= self.config.block_threshold || 
                          entry.suspicious_patterns >= self.config.block_threshold;

        if should_block {
            entry.blocked_until = Some(now + self.config.block_duration.as_secs());
            warn!("DDoS protection activated for IP: {} (requests: {}, suspicious: {})", 
                  ip, entry.request_count, entry.suspicious_patterns);
        } else if is_suspicious {
            warn!("Suspicious activity detected for IP: {} (patterns: {})", 
                  ip, entry.suspicious_patterns);
        }

        self.set_ddos_entry(&key, &entry, self.config.time_window).await.ok();

        !should_block
    }

    async fn get_ddos_entry(&self, key: &str) -> Result<DDoSEntry, Error> {
        if let Some(ref redis_client) = self.redis_client {
            if let Ok(mut conn) = redis_client.get_async_connection().await {
                if let Ok(Some(data)) = conn.get::<_, Option<String>>(key).await {
                    if let Ok(entry) = serde_json::from_str::<DDoSEntry>(&data) {
                        return Ok(entry);
                    }
                }
            }
        }

        let store = self.memory_store.read().await;
        Ok(store.get(key).cloned().unwrap_or_default())
    }

    async fn set_ddos_entry(&self, key: &str, entry: &DDoSEntry, ttl: Duration) -> Result<(), Error> {
        if let Some(ref redis_client) = self.redis_client {
            if let Ok(mut conn) = redis_client.get_async_connection().await {
                let data = serde_json::to_string(entry).unwrap();
                let _: () = conn.setex(key, ttl.as_secs() as usize, data).await.unwrap_or(());
                return Ok(());
            }
        }

        let mut store = self.memory_store.write().await;
        store.insert(key.to_string(), entry.clone());
        Ok(())
    }
}

impl Default for DDoSEntry {
    fn default() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        Self {
            request_count: 0,
            suspicious_patterns: 0,
            window_start: now,
            blocked_until: None,
            user_agents: Vec::new(),
            paths: Vec::new(),
        }
    }
}

pub struct DDoSMiddleware<S> {
    service: Rc<S>,
    protection: Arc<DDoSProtection>,
}

impl<S, B> Service<ServiceRequest> for DDoSMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let protection = self.protection.clone();

        Box::pin(async move {
            if protection.analyze_request(&req).await {
                service.call(req).await
            } else {
                Ok(req.into_response(
                    HttpResponse::TooManyRequests()
                        .insert_header(("Retry-After", "3600"))
                        .json(serde_json::json!({
                            "error": "Request blocked",
                            "message": "Your request has been blocked due to suspicious activity.",
                            "retry_after": 3600
                        }))
                ))
            }
        })
    }
}

pub struct DDoSFactory {
    protection: Arc<DDoSProtection>,
}

impl DDoSFactory {
    pub fn new(protection: DDoSProtection) -> Self {
        Self {
            protection: Arc::new(protection),
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for DDoSFactory
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = DDoSMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(DDoSMiddleware {
            service: Rc::new(service),
            protection: self.protection.clone(),
        }))
    }
}