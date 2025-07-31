use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use chrono::Utc;
use futures_util::future::LocalBoxFuture;
use std::{
    future::{ready, Ready},
    rc::Rc,
    time::Instant,
};
use tracing::{debug, error, warn};

use crate::services::performance_service::{PerformanceMetrics, PerformanceService};

pub struct PerformanceMiddleware {
    service: PerformanceService,
}

impl PerformanceMiddleware {
    pub fn new(service: PerformanceService) -> Self {
        Self { service }
    }
}

impl<S, B> Transform<S, ServiceRequest> for PerformanceMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = PerformanceMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(PerformanceMiddlewareService {
            service: Rc::new(service),
            performance_service: self.service.clone(),
        }))
    }
}

pub struct PerformanceMiddlewareService<S> {
    service: Rc<S>,
    performance_service: PerformanceService,
}

impl<S, B> Service<ServiceRequest> for PerformanceMiddlewareService<S>
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
        let service = Rc::clone(&self.service);
        let performance_service = self.performance_service.clone();
        
        Box::pin(async move {
            let start_time = Instant::now();
            let method = req.method().to_string();
            let path = req.path().to_string();
            let user_agent = req
                .headers()
                .get("user-agent")
                .and_then(|h| h.to_str().ok())
                .map(|s| s.to_string());
            let ip_address = req
                .connection_info()
                .peer_addr()
                .map(|addr| addr.to_string());
            
            // Extract user ID from request extensions if available
            let user_id = req.extensions().get::<uuid::Uuid>().copied();
            
            // Get request size
            let request_size = req
                .headers()
                .get("content-length")
                .and_then(|h| h.to_str().ok())
                .and_then(|s| s.parse::<u64>().ok());

            // Call the service
            let response = service.call(req).await?;
            
            let response_time = start_time.elapsed();
            let status_code = response.status().as_u16();
            
            // Get response size
            let response_size = response
                .headers()
                .get("content-length")
                .and_then(|h| h.to_str().ok())
                .and_then(|s| s.parse::<u64>().ok());

            // Create performance metrics
            let metrics = PerformanceMetrics {
                timestamp: Utc::now(),
                endpoint: path,
                method,
                response_time_ms: response_time.as_millis() as u64,
                status_code,
                user_id,
                ip_address,
                user_agent,
                request_size,
                response_size,
                database_query_time_ms: None, // Would be set by database middleware
                cache_hit: None, // Would be set by cache middleware
                error_details: if status_code >= 400 {
                    Some(format!("HTTP {}", status_code))
                } else {
                    None
                },
            };

            // Record metrics asynchronously
            let perf_service = performance_service.clone();
            tokio::spawn(async move {
                if let Err(e) = perf_service.record_request_metrics(metrics).await {
                    error!("Failed to record performance metrics: {}", e);
                }
            });

            // Log slow requests
            if response_time.as_millis() > 1000 {
                warn!(
                    "Slow request: {} {} took {}ms",
                    metrics.method,
                    metrics.endpoint,
                    response_time.as_millis()
                );
            }

            // Log errors
            if status_code >= 500 {
                error!(
                    "Server error: {} {} returned {}",
                    metrics.method, metrics.endpoint, status_code
                );
            }

            Ok(response)
        })
    }
}

// Database query timing middleware
pub struct DatabaseTimingMiddleware;

impl DatabaseTimingMiddleware {
    pub fn new() -> Self {
        Self
    }
}

impl<S, B> Transform<S, ServiceRequest> for DatabaseTimingMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = DatabaseTimingMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(DatabaseTimingMiddlewareService {
            service: Rc::new(service),
        }))
    }
}

pub struct DatabaseTimingMiddlewareService<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for DatabaseTimingMiddlewareService<S>
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
        let service = Rc::clone(&self.service);
        
        Box::pin(async move {
            // Set up database timing context
            req.extensions_mut().insert(DatabaseTimingContext::new());
            
            let response = service.call(req).await?;
            
            // Extract timing information if available
            if let Some(timing_context) = response.request().extensions().get::<DatabaseTimingContext>() {
                let total_db_time = timing_context.get_total_time();
                if total_db_time.as_millis() > 500 {
                    warn!("Slow database operations: {}ms total", total_db_time.as_millis());
                }
                
                debug!("Database timing: {}ms across {} queries", 
                    total_db_time.as_millis(), 
                    timing_context.get_query_count()
                );
            }
            
            Ok(response)
        })
    }
}

// Context for tracking database timing
#[derive(Debug)]
pub struct DatabaseTimingContext {
    start_time: Instant,
    query_times: Vec<std::time::Duration>,
}

impl DatabaseTimingContext {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            query_times: Vec::new(),
        }
    }
    
    pub fn record_query_time(&mut self, duration: std::time::Duration) {
        self.query_times.push(duration);
    }
    
    pub fn get_total_time(&self) -> std::time::Duration {
        self.query_times.iter().sum()
    }
    
    pub fn get_query_count(&self) -> usize {
        self.query_times.len()
    }
    
    pub fn get_average_query_time(&self) -> Option<std::time::Duration> {
        if self.query_times.is_empty() {
            None
        } else {
            Some(self.get_total_time() / self.query_times.len() as u32)
        }
    }
}

// Cache hit tracking middleware
pub struct CacheTrackingMiddleware;

impl CacheTrackingMiddleware {
    pub fn new() -> Self {
        Self
    }
}

impl<S, B> Transform<S, ServiceRequest> for CacheTrackingMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = CacheTrackingMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(CacheTrackingMiddlewareService {
            service: Rc::new(service),
        }))
    }
}

pub struct CacheTrackingMiddlewareService<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for CacheTrackingMiddlewareService<S>
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
        let service = Rc::clone(&self.service);
        
        Box::pin(async move {
            // Set up cache tracking context
            req.extensions_mut().insert(CacheTrackingContext::new());
            
            let response = service.call(req).await?;
            
            // Extract cache information if available
            if let Some(cache_context) = response.request().extensions().get::<CacheTrackingContext>() {
                let hit_rate = cache_context.get_hit_rate();
                if hit_rate < 0.8 && cache_context.get_total_operations() > 0 {
                    warn!("Low cache hit rate: {:.1}%", hit_rate * 100.0);
                }
                
                debug!("Cache stats: {:.1}% hit rate across {} operations", 
                    hit_rate * 100.0, 
                    cache_context.get_total_operations()
                );
            }
            
            Ok(response)
        })
    }
}

// Context for tracking cache operations
#[derive(Debug)]
pub struct CacheTrackingContext {
    hits: u32,
    misses: u32,
}

impl CacheTrackingContext {
    pub fn new() -> Self {
        Self { hits: 0, misses: 0 }
    }
    
    pub fn record_hit(&mut self) {
        self.hits += 1;
    }
    
    pub fn record_miss(&mut self) {
        self.misses += 1;
    }
    
    pub fn get_hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }
    
    pub fn get_total_operations(&self) -> u32 {
        self.hits + self.misses
    }
}

// Request size limiting middleware
pub struct RequestSizeLimitMiddleware {
    max_size: usize,
}

impl RequestSizeLimitMiddleware {
    pub fn new(max_size: usize) -> Self {
        Self { max_size }
    }
}

impl<S, B> Transform<S, ServiceRequest> for RequestSizeLimitMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = RequestSizeLimitMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RequestSizeLimitMiddlewareService {
            service: Rc::new(service),
            max_size: self.max_size,
        }))
    }
}

pub struct RequestSizeLimitMiddlewareService<S> {
    service: Rc<S>,
    max_size: usize,
}

impl<S, B> Service<ServiceRequest> for RequestSizeLimitMiddlewareService<S>
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
        let service = Rc::clone(&self.service);
        let max_size = self.max_size;
        
        Box::pin(async move {
            // Check request size
            if let Some(content_length) = req.headers().get("content-length") {
                if let Ok(size_str) = content_length.to_str() {
                    if let Ok(size) = size_str.parse::<usize>() {
                        if size > max_size {
                            warn!("Request size {} exceeds limit {}", size, max_size);
                            return Err(actix_web::error::ErrorPayloadTooLarge(
                                format!("Request size {} exceeds maximum allowed size {}", size, max_size)
                            ));
                        }
                    }
                }
            }
            
            service.call(req).await
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_database_timing_context() {
        let mut context = DatabaseTimingContext::new();
        
        context.record_query_time(std::time::Duration::from_millis(100));
        context.record_query_time(std::time::Duration::from_millis(200));
        
        assert_eq!(context.get_query_count(), 2);
        assert_eq!(context.get_total_time(), std::time::Duration::from_millis(300));
        assert_eq!(context.get_average_query_time(), Some(std::time::Duration::from_millis(150)));
    }
    
    #[test]
    fn test_cache_tracking_context() {
        let mut context = CacheTrackingContext::new();
        
        context.record_hit();
        context.record_hit();
        context.record_miss();
        
        assert_eq!(context.get_total_operations(), 3);
        assert!((context.get_hit_rate() - 0.6666666666666666).abs() < f64::EPSILON);
    }
}