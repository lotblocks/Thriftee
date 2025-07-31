use redis::{AsyncCommands, Client as RedisClient, RedisError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;
use std::time::Duration;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::utils::errors::AppError;

#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub default_ttl: Duration,
    pub max_key_size: usize,
    pub max_value_size: usize,
    pub enable_compression: bool,
    pub compression_threshold: usize,
    pub key_prefix: String,
    pub enable_metrics: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            default_ttl: Duration::from_secs(3600), // 1 hour
            max_key_size: 250,
            max_value_size: 1024 * 1024, // 1MB
            enable_compression: true,
            compression_threshold: 1024, // 1KB
            key_prefix: "raffle_platform".to_string(),
            enable_metrics: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheMetrics {
    pub hits: u64,
    pub misses: u64,
    pub sets: u64,
    pub deletes: u64,
    pub errors: u64,
    pub total_operations: u64,
}

impl CacheMetrics {
    pub fn hit_rate(&self) -> f64 {
        if self.hits + self.misses == 0 {
            0.0
        } else {
            (self.hits as f64) / ((self.hits + self.misses) as f64) * 100.0
        }
    }

    pub fn error_rate(&self) -> f64 {
        if self.total_operations == 0 {
            0.0
        } else {
            (self.errors as f64) / (self.total_operations as f64) * 100.0
        }
    }
}

pub struct CacheService {
    client: RedisClient,
    config: CacheConfig,
    metrics: std::sync::Arc<std::sync::RwLock<CacheMetrics>>,
}

impl CacheService {
    pub fn new(client: RedisClient, config: CacheConfig) -> Self {
        Self {
            client,
            config,
            metrics: std::sync::Arc::new(std::sync::RwLock::new(CacheMetrics {
                hits: 0,
                misses: 0,
                sets: 0,
                deletes: 0,
                errors: 0,
                total_operations: 0,
            })),
        }
    }

    /// Get a value from cache
    pub async fn get<T>(&self, key: &str) -> Result<Option<T>, AppError>
    where
        T: for<'de> Deserialize<'de> + Debug,
    {
        let full_key = self.build_key(key);
        self.increment_total_operations();

        match self.get_raw(&full_key).await {
            Ok(Some(data)) => {
                match self.deserialize::<T>(&data) {
                    Ok(value) => {
                        self.increment_hits();
                        debug!("Cache hit for key: {}", key);
                        Ok(Some(value))
                    }
                    Err(e) => {
                        self.increment_errors();
                        error!("Failed to deserialize cached value for key {}: {}", key, e);
                        // Remove corrupted data
                        let _ = self.delete(key).await;
                        Ok(None)
                    }
                }
            }
            Ok(None) => {
                self.increment_misses();
                debug!("Cache miss for key: {}", key);
                Ok(None)
            }
            Err(e) => {
                self.increment_errors();
                error!("Cache get error for key {}: {}", key, e);
                Err(e)
            }
        }
    }

    /// Set a value in cache
    pub async fn set<T>(&self, key: &str, value: &T, ttl: Option<Duration>) -> Result<(), AppError>
    where
        T: Serialize + Debug,
    {
        let full_key = self.build_key(key);
        self.increment_total_operations();

        let serialized = self.serialize(value)?;
        let ttl = ttl.unwrap_or(self.config.default_ttl);

        match self.set_raw(&full_key, &serialized, ttl).await {
            Ok(_) => {
                self.increment_sets();
                debug!("Cache set for key: {} with TTL: {:?}", key, ttl);
                Ok(())
            }
            Err(e) => {
                self.increment_errors();
                error!("Cache set error for key {}: {}", key, e);
                Err(e)
            }
        }
    }

    /// Delete a value from cache
    pub async fn delete(&self, key: &str) -> Result<bool, AppError> {
        let full_key = self.build_key(key);
        self.increment_total_operations();

        match self.delete_raw(&full_key).await {
            Ok(deleted) => {
                if deleted {
                    self.increment_deletes();
                    debug!("Cache delete for key: {}", key);
                }
                Ok(deleted)
            }
            Err(e) => {
                self.increment_errors();
                error!("Cache delete error for key {}: {}", key, e);
                Err(e)
            }
        }
    }

    /// Check if a key exists in cache
    pub async fn exists(&self, key: &str) -> Result<bool, AppError> {
        let full_key = self.build_key(key);
        self.increment_total_operations();

        let mut conn = self.client.get_async_connection().await
            .map_err(|e| AppError::InternalServerError(format!("Redis connection failed: {}", e)))?;

        let exists: bool = conn.exists(&full_key).await
            .map_err(|e| AppError::InternalServerError(format!("Redis exists failed: {}", e)))?;

        Ok(exists)
    }

    /// Get multiple values from cache
    pub async fn get_many<T>(&self, keys: &[&str]) -> Result<HashMap<String, T>, AppError>
    where
        T: for<'de> Deserialize<'de> + Debug,
    {
        let full_keys: Vec<String> = keys.iter().map(|k| self.build_key(k)).collect();
        let mut conn = self.client.get_async_connection().await
            .map_err(|e| AppError::InternalServerError(format!("Redis connection failed: {}", e)))?;

        let values: Vec<Option<String>> = conn.get(&full_keys).await
            .map_err(|e| AppError::InternalServerError(format!("Redis mget failed: {}", e)))?;

        let mut result = HashMap::new();
        for (i, value) in values.into_iter().enumerate() {
            if let Some(data) = value {
                match self.deserialize::<T>(&data) {
                    Ok(deserialized) => {
                        result.insert(keys[i].to_string(), deserialized);
                        self.increment_hits();
                    }
                    Err(e) => {
                        error!("Failed to deserialize cached value for key {}: {}", keys[i], e);
                        self.increment_errors();
                    }
                }
            } else {
                self.increment_misses();
            }
        }

        Ok(result)
    }

    /// Set multiple values in cache
    pub async fn set_many<T>(&self, values: HashMap<&str, &T>, ttl: Option<Duration>) -> Result<(), AppError>
    where
        T: Serialize + Debug,
    {
        let mut conn = self.client.get_async_connection().await
            .map_err(|e| AppError::InternalServerError(format!("Redis connection failed: {}", e)))?;

        let ttl_secs = ttl.unwrap_or(self.config.default_ttl).as_secs();

        for (key, value) in values {
            let full_key = self.build_key(key);
            let serialized = self.serialize(value)?;
            
            let _: () = conn.set_ex(&full_key, &serialized, ttl_secs).await
                .map_err(|e| AppError::InternalServerError(format!("Redis setex failed: {}", e)))?;
            
            self.increment_sets();
        }

        Ok(())
    }

    /// Increment a counter in cache
    pub async fn increment(&self, key: &str, delta: i64) -> Result<i64, AppError> {
        let full_key = self.build_key(key);
        let mut conn = self.client.get_async_connection().await
            .map_err(|e| AppError::InternalServerError(format!("Redis connection failed: {}", e)))?;

        let result: i64 = conn.incr(&full_key, delta).await
            .map_err(|e| AppError::InternalServerError(format!("Redis incr failed: {}", e)))?;

        Ok(result)
    }

    /// Set expiration for a key
    pub async fn expire(&self, key: &str, ttl: Duration) -> Result<bool, AppError> {
        let full_key = self.build_key(key);
        let mut conn = self.client.get_async_connection().await
            .map_err(|e| AppError::InternalServerError(format!("Redis connection failed: {}", e)))?;

        let result: bool = conn.expire(&full_key, ttl.as_secs() as usize).await
            .map_err(|e| AppError::InternalServerError(format!("Redis expire failed: {}", e)))?;

        Ok(result)
    }

    /// Get TTL for a key
    pub async fn ttl(&self, key: &str) -> Result<Option<Duration>, AppError> {
        let full_key = self.build_key(key);
        let mut conn = self.client.get_async_connection().await
            .map_err(|e| AppError::InternalServerError(format!("Redis connection failed: {}", e)))?;

        let ttl_secs: i64 = conn.ttl(&full_key).await
            .map_err(|e| AppError::InternalServerError(format!("Redis ttl failed: {}", e)))?;

        match ttl_secs {
            -2 => Ok(None), // Key doesn't exist
            -1 => Ok(None), // Key exists but has no expiration
            secs if secs > 0 => Ok(Some(Duration::from_secs(secs as u64))),
            _ => Ok(None),
        }
    }

    /// Clear all keys with the configured prefix
    pub async fn clear_all(&self) -> Result<u64, AppError> {
        let pattern = format!("{}:*", self.config.key_prefix);
        let mut conn = self.client.get_async_connection().await
            .map_err(|e| AppError::InternalServerError(format!("Redis connection failed: {}", e)))?;

        let keys: Vec<String> = conn.keys(&pattern).await
            .map_err(|e| AppError::InternalServerError(format!("Redis keys failed: {}", e)))?;

        if keys.is_empty() {
            return Ok(0);
        }

        let deleted: u64 = conn.del(&keys).await
            .map_err(|e| AppError::InternalServerError(format!("Redis del failed: {}", e)))?;

        Ok(deleted)
    }

    /// Get cache metrics
    pub fn get_metrics(&self) -> CacheMetrics {
        self.metrics.read().unwrap().clone()
    }

    /// Reset cache metrics
    pub fn reset_metrics(&self) {
        let mut metrics = self.metrics.write().unwrap();
        *metrics = CacheMetrics {
            hits: 0,
            misses: 0,
            sets: 0,
            deletes: 0,
            errors: 0,
            total_operations: 0,
        };
    }

    /// Get cache info from Redis
    pub async fn get_cache_info(&self) -> Result<HashMap<String, String>, AppError> {
        let mut conn = self.client.get_async_connection().await
            .map_err(|e| AppError::InternalServerError(format!("Redis connection failed: {}", e)))?;

        let info: String = redis::cmd("INFO").query_async(&mut conn).await
            .map_err(|e| AppError::InternalServerError(format!("Redis INFO failed: {}", e)))?;

        let mut result = HashMap::new();
        for line in info.lines() {
            if let Some((key, value)) = line.split_once(':') {
                result.insert(key.to_string(), value.to_string());
            }
        }

        Ok(result)
    }

    // Private helper methods

    fn build_key(&self, key: &str) -> String {
        if key.len() > self.config.max_key_size {
            warn!("Key length {} exceeds maximum {}", key.len(), self.config.max_key_size);
        }
        format!("{}:{}", self.config.key_prefix, key)
    }

    async fn get_raw(&self, key: &str) -> Result<Option<String>, AppError> {
        let mut conn = self.client.get_async_connection().await
            .map_err(|e| AppError::InternalServerError(format!("Redis connection failed: {}", e)))?;

        let value: Option<String> = conn.get(key).await
            .map_err(|e| AppError::InternalServerError(format!("Redis get failed: {}", e)))?;

        Ok(value)
    }

    async fn set_raw(&self, key: &str, value: &str, ttl: Duration) -> Result<(), AppError> {
        let mut conn = self.client.get_async_connection().await
            .map_err(|e| AppError::InternalServerError(format!("Redis connection failed: {}", e)))?;

        let _: () = conn.set_ex(key, value, ttl.as_secs()).await
            .map_err(|e| AppError::InternalServerError(format!("Redis setex failed: {}", e)))?;

        Ok(())
    }

    async fn delete_raw(&self, key: &str) -> Result<bool, AppError> {
        let mut conn = self.client.get_async_connection().await
            .map_err(|e| AppError::InternalServerError(format!("Redis connection failed: {}", e)))?;

        let deleted: u64 = conn.del(key).await
            .map_err(|e| AppError::InternalServerError(format!("Redis del failed: {}", e)))?;

        Ok(deleted > 0)
    }

    fn serialize<T>(&self, value: &T) -> Result<String, AppError>
    where
        T: Serialize,
    {
        let serialized = serde_json::to_string(value)
            .map_err(|e| AppError::InternalServerError(format!("Serialization failed: {}", e)))?;

        if serialized.len() > self.config.max_value_size {
            return Err(AppError::BadRequest(format!(
                "Value size {} exceeds maximum {}",
                serialized.len(),
                self.config.max_value_size
            )));
        }

        // Apply compression if enabled and value is large enough
        if self.config.enable_compression && serialized.len() > self.config.compression_threshold {
            // In a real implementation, you'd use a compression library like flate2
            // For now, we'll just return the serialized string
            Ok(serialized)
        } else {
            Ok(serialized)
        }
    }

    fn deserialize<T>(&self, data: &str) -> Result<T, AppError>
    where
        T: for<'de> Deserialize<'de>,
    {
        // In a real implementation, you'd check for compression headers and decompress
        serde_json::from_str(data)
            .map_err(|e| AppError::InternalServerError(format!("Deserialization failed: {}", e)))
    }

    fn increment_hits(&self) {
        if let Ok(mut metrics) = self.metrics.write() {
            metrics.hits += 1;
        }
    }

    fn increment_misses(&self) {
        if let Ok(mut metrics) = self.metrics.write() {
            metrics.misses += 1;
        }
    }

    fn increment_sets(&self) {
        if let Ok(mut metrics) = self.metrics.write() {
            metrics.sets += 1;
        }
    }

    fn increment_deletes(&self) {
        if let Ok(mut metrics) = self.metrics.write() {
            metrics.deletes += 1;
        }
    }

    fn increment_errors(&self) {
        if let Ok(mut metrics) = self.metrics.write() {
            metrics.errors += 1;
        }
    }

    fn increment_total_operations(&self) {
        if let Ok(mut metrics) = self.metrics.write() {
            metrics.total_operations += 1;
        }
    }
}

impl Clone for CacheService {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            config: self.config.clone(),
            metrics: Arc::clone(&self.metrics),
        }
    }
}

// Cache-aside pattern implementation
pub struct CacheAsideService<T> {
    cache: CacheService,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> CacheAsideService<T>
where
    T: Serialize + for<'de> Deserialize<'de> + Debug + Clone,
{
    pub fn new(cache: CacheService) -> Self {
        Self {
            cache,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Get value with cache-aside pattern
    pub async fn get_or_fetch<F, Fut>(
        &self,
        key: &str,
        fetch_fn: F,
        ttl: Option<Duration>,
    ) -> Result<T, AppError>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, AppError>>,
    {
        // Try to get from cache first
        if let Some(cached_value) = self.cache.get::<T>(key).await? {
            return Ok(cached_value);
        }

        // Cache miss - fetch from source
        let value = fetch_fn().await?;

        // Store in cache for next time
        if let Err(e) = self.cache.set(key, &value, ttl).await {
            warn!("Failed to cache value for key {}: {}", key, e);
        }

        Ok(value)
    }

    /// Update value with write-through pattern
    pub async fn update<F, Fut>(
        &self,
        key: &str,
        update_fn: F,
        ttl: Option<Duration>,
    ) -> Result<T, AppError>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, AppError>>,
    {
        // Update the source
        let value = update_fn().await?;

        // Update cache
        if let Err(e) = self.cache.set(key, &value, ttl).await {
            warn!("Failed to update cache for key {}: {}", key, e);
        }

        Ok(value)
    }

    /// Delete value from both cache and source
    pub async fn delete<F, Fut>(&self, key: &str, delete_fn: F) -> Result<(), AppError>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<(), AppError>>,
    {
        // Delete from source first
        delete_fn().await?;

        // Remove from cache
        if let Err(e) = self.cache.delete(key).await {
            warn!("Failed to delete from cache for key {}: {}", key, e);
        }

        Ok(())
    }
}

// Distributed cache lock implementation
pub struct DistributedLock {
    cache: CacheService,
    key: String,
    value: String,
    ttl: Duration,
}

impl DistributedLock {
    pub async fn acquire(
        cache: CacheService,
        resource: &str,
        ttl: Duration,
    ) -> Result<Option<Self>, AppError> {
        let key = format!("lock:{}", resource);
        let value = Uuid::new_v4().to_string();

        let mut conn = cache.client.get_async_connection().await
            .map_err(|e| AppError::InternalServerError(format!("Redis connection failed: {}", e)))?;

        // Try to acquire lock using SET NX EX
        let result: Option<String> = redis::cmd("SET")
            .arg(&key)
            .arg(&value)
            .arg("NX")
            .arg("EX")
            .arg(ttl.as_secs())
            .query_async(&mut conn)
            .await
            .map_err(|e| AppError::InternalServerError(format!("Lock acquisition failed: {}", e)))?;

        if result.is_some() {
            Ok(Some(Self {
                cache,
                key,
                value,
                ttl,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn release(self) -> Result<bool, AppError> {
        let mut conn = self.cache.client.get_async_connection().await
            .map_err(|e| AppError::InternalServerError(format!("Redis connection failed: {}", e)))?;

        // Use Lua script to ensure we only delete our own lock
        let script = r#"
            if redis.call("GET", KEYS[1]) == ARGV[1] then
                return redis.call("DEL", KEYS[1])
            else
                return 0
            end
        "#;

        let result: i32 = redis::Script::new(script)
            .key(&self.key)
            .arg(&self.value)
            .invoke_async(&mut conn)
            .await
            .map_err(|e| AppError::InternalServerError(format!("Lock release failed: {}", e)))?;

        Ok(result == 1)
    }

    pub async fn extend(&mut self, additional_ttl: Duration) -> Result<bool, AppError> {
        let mut conn = self.cache.client.get_async_connection().await
            .map_err(|e| AppError::InternalServerError(format!("Redis connection failed: {}", e)))?;

        let new_ttl = self.ttl + additional_ttl;

        // Use Lua script to extend TTL only if we own the lock
        let script = r#"
            if redis.call("GET", KEYS[1]) == ARGV[1] then
                return redis.call("EXPIRE", KEYS[1], ARGV[2])
            else
                return 0
            end
        "#;

        let result: i32 = redis::Script::new(script)
            .key(&self.key)
            .arg(&self.value)
            .arg(new_ttl.as_secs())
            .invoke_async(&mut conn)
            .await
            .map_err(|e| AppError::InternalServerError(format!("Lock extension failed: {}", e)))?;

        if result == 1 {
            self.ttl = new_ttl;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

// Auto-release lock on drop
impl Drop for DistributedLock {
    fn drop(&mut self) {
        let cache = self.cache.clone();
        let key = self.key.clone();
        let value = self.value.clone();

        tokio::spawn(async move {
            if let Ok(mut conn) = cache.client.get_async_connection().await {
                let script = r#"
                    if redis.call("GET", KEYS[1]) == ARGV[1] then
                        return redis.call("DEL", KEYS[1])
                    else
                        return 0
                    end
                "#;

                let _: Result<i32, _> = redis::Script::new(script)
                    .key(&key)
                    .arg(&value)
                    .invoke_async(&mut conn)
                    .await;
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_cache_basic_operations() {
        // This would require a Redis instance for testing
        // In a real implementation, you'd use a test container or mock
    }

    #[tokio::test]
    async fn test_distributed_lock() {
        // Test distributed lock functionality
    }

    #[tokio::test]
    async fn test_cache_aside_pattern() {
        // Test cache-aside pattern implementation
    }
}        
self.increment_total_operations();
        self.increment_sets();

        let serialized = self.serialize(value)?;
        let ttl = ttl.unwrap_or(self.config.default_ttl);

        match self.set_raw(&full_key, &serialized, Some(ttl)).await {
            Ok(_) => {
                debug!("Cache set for key: {} (TTL: {:?})", key, ttl);
                Ok(())
            }
            Err(e) => {
                self.increment_errors();
                error!("Cache set error for key {}: {}", key, e);
                Err(e)
            }
        }
    }

    /// Set multiple values in cache
    pub async fn set_many<T>(&self, entries: Vec<(&str, &T, Option<Duration>)>) -> Result<(), AppError>
    where
        T: Serialize + Debug,
    {
        for (key, value, ttl) in entries {
            self.set(key, value, ttl).await?;
        }
        Ok(())
    }

    /// Get multiple values from cache
    pub async fn get_many<T>(&self, keys: &[&str]) -> Result<Vec<(String, Option<T>)>, AppError>
    where
        T: for<'de> Deserialize<'de> + Debug,
    {
        let mut results = Vec::new();
        for &key in keys {
            let value = self.get::<T>(key).await?;
            results.push((key.to_string(), value));
        }
        Ok(results)
    }

    /// Delete a value from cache
    pub async fn delete(&self, key: &str) -> Result<bool, AppError> {
        let full_key = self.build_key(key);
        self.increment_total_operations();
        self.increment_deletes();

        match self.delete_raw(&full_key).await {
            Ok(deleted) => {
                debug!("Cache delete for key: {} (existed: {})", key, deleted);
                Ok(deleted)
            }
            Err(e) => {
                self.increment_errors();
                error!("Cache delete error for key {}: {}", key, e);
                Err(e)
            }
        }
    }

    /// Delete multiple keys from cache
    pub async fn delete_many(&self, keys: &[&str]) -> Result<u64, AppError> {
        let mut deleted_count = 0;
        for &key in keys {
            if self.delete(key).await? {
                deleted_count += 1;
            }
        }
        Ok(deleted_count)
    }

    /// Delete keys by pattern
    pub async fn delete_by_pattern(&self, pattern: &str) -> Result<u64, AppError> {
        let full_pattern = self.build_key(pattern);
        let mut conn = self.client.get_async_connection().await
            .map_err(|e| AppError::InternalServerError(format!("Redis connection failed: {}", e)))?;

        // Get keys matching pattern
        let keys: Vec<String> = redis::cmd("KEYS")
            .arg(&full_pattern)
            .query_async(&mut conn)
            .await
            .map_err(|e| AppError::InternalServerError(format!("Redis KEYS failed: {}", e)))?;

        if keys.is_empty() {
            return Ok(0);
        }

        // Delete keys
        let deleted: u64 = redis::cmd("DEL")
            .arg(&keys)
            .query_async(&mut conn)
            .await
            .map_err(|e| AppError::InternalServerError(format!("Redis DEL failed: {}", e)))?;

        debug!("Deleted {} keys matching pattern: {}", deleted, pattern);
        Ok(deleted)
    }

    /// Check if a key exists in cache
    pub async fn exists(&self, key: &str) -> Result<bool, AppError> {
        let full_key = self.build_key(key);
        let mut conn = self.client.get_async_connection().await
            .map_err(|e| AppError::InternalServerError(format!("Redis connection failed: {}", e)))?;

        let exists: bool = redis::cmd("EXISTS")
            .arg(&full_key)
            .query_async(&mut conn)
            .await
            .map_err(|e| AppError::InternalServerError(format!("Redis EXISTS failed: {}", e)))?;

        Ok(exists)
    }

    /// Set expiration time for a key
    pub async fn expire(&self, key: &str, ttl: Duration) -> Result<bool, AppError> {
        let full_key = self.build_key(key);
        let mut conn = self.client.get_async_connection().await
            .map_err(|e| AppError::InternalServerError(format!("Redis connection failed: {}", e)))?;

        let result: bool = redis::cmd("EXPIRE")
            .arg(&full_key)
            .arg(ttl.as_secs())
            .query_async(&mut conn)
            .await
            .map_err(|e| AppError::InternalServerError(format!("Redis EXPIRE failed: {}", e)))?;

        Ok(result)
    }

    /// Get time to live for a key
    pub async fn ttl(&self, key: &str) -> Result<Option<Duration>, AppError> {
        let full_key = self.build_key(key);
        let mut conn = self.client.get_async_connection().await
            .map_err(|e| AppError::InternalServerError(format!("Redis connection failed: {}", e)))?;

        let ttl_seconds: i64 = redis::cmd("TTL")
            .arg(&full_key)
            .query_async(&mut conn)
            .await
            .map_err(|e| AppError::InternalServerError(format!("Redis TTL failed: {}", e)))?;

        match ttl_seconds {
            -2 => Ok(None), // Key doesn't exist
            -1 => Ok(Some(Duration::MAX)), // Key exists but has no expiration
            seconds if seconds > 0 => Ok(Some(Duration::from_secs(seconds as u64))),
            _ => Ok(None),
        }
    }

    /// Increment a numeric value in cache
    pub async fn increment(&self, key: &str, delta: i64) -> Result<i64, AppError> {
        let full_key = self.build_key(key);
        let mut conn = self.client.get_async_connection().await
            .map_err(|e| AppError::InternalServerError(format!("Redis connection failed: {}", e)))?;

        let result: i64 = if delta == 1 {
            redis::cmd("INCR").arg(&full_key).query_async(&mut conn).await
        } else {
            redis::cmd("INCRBY").arg(&full_key).arg(delta).query_async(&mut conn).await
        }
        .map_err(|e| AppError::InternalServerError(format!("Redis increment failed: {}", e)))?;

        Ok(result)
    }

    /// Decrement a numeric value in cache
    pub async fn decrement(&self, key: &str, delta: i64) -> Result<i64, AppError> {
        self.increment(key, -delta).await
    }

    /// Add item to a set
    pub async fn set_add(&self, key: &str, member: &str) -> Result<bool, AppError> {
        let full_key = self.build_key(key);
        let mut conn = self.client.get_async_connection().await
            .map_err(|e| AppError::InternalServerError(format!("Redis connection failed: {}", e)))?;

        let added: i64 = redis::cmd("SADD")
            .arg(&full_key)
            .arg(member)
            .query_async(&mut conn)
            .await
            .map_err(|e| AppError::InternalServerError(format!("Redis SADD failed: {}", e)))?;

        Ok(added > 0)
    }

    /// Remove item from a set
    pub async fn set_remove(&self, key: &str, member: &str) -> Result<bool, AppError> {
        let full_key = self.build_key(key);
        let mut conn = self.client.get_async_connection().await
            .map_err(|e| AppError::InternalServerError(format!("Redis connection failed: {}", e)))?;

        let removed: i64 = redis::cmd("SREM")
            .arg(&full_key)
            .arg(member)
            .query_async(&mut conn)
            .await
            .map_err(|e| AppError::InternalServerError(format!("Redis SREM failed: {}", e)))?;

        Ok(removed > 0)
    }

    /// Check if item is in set
    pub async fn set_contains(&self, key: &str, member: &str) -> Result<bool, AppError> {
        let full_key = self.build_key(key);
        let mut conn = self.client.get_async_connection().await
            .map_err(|e| AppError::InternalServerError(format!("Redis connection failed: {}", e)))?;

        let is_member: bool = redis::cmd("SISMEMBER")
            .arg(&full_key)
            .arg(member)
            .query_async(&mut conn)
            .await
            .map_err(|e| AppError::InternalServerError(format!("Redis SISMEMBER failed: {}", e)))?;

        Ok(is_member)
    }

    /// Get all members of a set
    pub async fn set_members(&self, key: &str) -> Result<Vec<String>, AppError> {
        let full_key = self.build_key(key);
        let mut conn = self.client.get_async_connection().await
            .map_err(|e| AppError::InternalServerError(format!("Redis connection failed: {}", e)))?;

        let members: Vec<String> = redis::cmd("SMEMBERS")
            .arg(&full_key)
            .query_async(&mut conn)
            .await
            .map_err(|e| AppError::InternalServerError(format!("Redis SMEMBERS failed: {}", e)))?;

        Ok(members)
    }

    /// Push item to list (left side)
    pub async fn list_push_left(&self, key: &str, value: &str) -> Result<i64, AppError> {
        let full_key = self.build_key(key);
        let mut conn = self.client.get_async_connection().await
            .map_err(|e| AppError::InternalServerError(format!("Redis connection failed: {}", e)))?;

        let length: i64 = redis::cmd("LPUSH")
            .arg(&full_key)
            .arg(value)
            .query_async(&mut conn)
            .await
            .map_err(|e| AppError::InternalServerError(format!("Redis LPUSH failed: {}", e)))?;

        Ok(length)
    }

    /// Push item to list (right side)
    pub async fn list_push_right(&self, key: &str, value: &str) -> Result<i64, AppError> {
        let full_key = self.build_key(key);
        let mut conn = self.client.get_async_connection().await
            .map_err(|e| AppError::InternalServerError(format!("Redis connection failed: {}", e)))?;

        let length: i64 = redis::cmd("RPUSH")
            .arg(&full_key)
            .arg(value)
            .query_async(&mut conn)
            .await
            .map_err(|e| AppError::InternalServerError(format!("Redis RPUSH failed: {}", e)))?;

        Ok(length)
    }

    /// Pop item from list (left side)
    pub async fn list_pop_left(&self, key: &str) -> Result<Option<String>, AppError> {
        let full_key = self.build_key(key);
        let mut conn = self.client.get_async_connection().await
            .map_err(|e| AppError::InternalServerError(format!("Redis connection failed: {}", e)))?;

        let value: Option<String> = redis::cmd("LPOP")
            .arg(&full_key)
            .query_async(&mut conn)
            .await
            .map_err(|e| AppError::InternalServerError(format!("Redis LPOP failed: {}", e)))?;

        Ok(value)
    }

    /// Pop item from list (right side)
    pub async fn list_pop_right(&self, key: &str) -> Result<Option<String>, AppError> {
        let full_key = self.build_key(key);
        let mut conn = self.client.get_async_connection().await
            .map_err(|e| AppError::InternalServerError(format!("Redis connection failed: {}", e)))?;

        let value: Option<String> = redis::cmd("RPOP")
            .arg(&full_key)
            .query_async(&mut conn)
            .await
            .map_err(|e| AppError::InternalServerError(format!("Redis RPOP failed: {}", e)))?;

        Ok(value)
    }

    /// Get list length
    pub async fn list_length(&self, key: &str) -> Result<i64, AppError> {
        let full_key = self.build_key(key);
        let mut conn = self.client.get_async_connection().await
            .map_err(|e| AppError::InternalServerError(format!("Redis connection failed: {}", e)))?;

        let length: i64 = redis::cmd("LLEN")
            .arg(&full_key)
            .query_async(&mut conn)
            .await
            .map_err(|e| AppError::InternalServerError(format!("Redis LLEN failed: {}", e)))?;

        Ok(length)
    }

    /// Get cache metrics
    pub async fn get_metrics(&self) -> CacheMetrics {
        let metrics = self.metrics.read().unwrap();
        metrics.clone()
    }

    /// Reset cache metrics
    pub async fn reset_metrics(&self) {
        let mut metrics = self.metrics.write().unwrap();
        *metrics = CacheMetrics {
            hits: 0,
            misses: 0,
            sets: 0,
            deletes: 0,
            errors: 0,
            total_operations: 0,
        };
    }

    /// Clear all cache data (use with caution)
    pub async fn clear_all(&self) -> Result<(), AppError> {
        let pattern = format!("{}:*", self.config.key_prefix);
        self.delete_by_pattern(&pattern).await?;
        Ok(())
    }

    /// Get cache info from Redis
    pub async fn get_cache_info(&self) -> Result<HashMap<String, String>, AppError> {
        let mut conn = self.client.get_async_connection().await
            .map_err(|e| AppError::InternalServerError(format!("Redis connection failed: {}", e)))?;

        let info: String = redis::cmd("INFO")
            .arg("memory")
            .query_async(&mut conn)
            .await
            .map_err(|e| AppError::InternalServerError(format!("Redis INFO failed: {}", e)))?;

        let mut info_map = HashMap::new();
        for line in info.lines() {
            if let Some((key, value)) = line.split_once(':') {
                info_map.insert(key.to_string(), value.to_string());
            }
        }

        Ok(info_map)
    }

    // Private helper methods

    fn build_key(&self, key: &str) -> String {
        if key.len() > self.config.max_key_size {
            warn!("Key length exceeds maximum: {} > {}", key.len(), self.config.max_key_size);
        }
        format!("{}:{}", self.config.key_prefix, key)
    }

    async fn get_raw(&self, key: &str) -> Result<Option<Vec<u8>>, AppError> {
        let mut conn = self.client.get_async_connection().await
            .map_err(|e| AppError::InternalServerError(format!("Redis connection failed: {}", e)))?;

        let data: Option<Vec<u8>> = conn.get(key).await
            .map_err(|e| AppError::InternalServerError(format!("Redis GET failed: {}", e)))?;

        Ok(data)
    }

    async fn set_raw(&self, key: &str, data: &[u8], ttl: Option<Duration>) -> Result<(), AppError> {
        if data.len() > self.config.max_value_size {
            return Err(AppError::BadRequest(format!(
                "Value size exceeds maximum: {} > {}",
                data.len(),
                self.config.max_value_size
            )));
        }

        let mut conn = self.client.get_async_connection().await
            .map_err(|e| AppError::InternalServerError(format!("Redis connection failed: {}", e)))?;

        if let Some(ttl) = ttl {
            conn.set_ex(key, data, ttl.as_secs()).await
        } else {
            conn.set(key, data).await
        }
        .map_err(|e| AppError::InternalServerError(format!("Redis SET failed: {}", e)))?;

        Ok(())
    }

    async fn delete_raw(&self, key: &str) -> Result<bool, AppError> {
        let mut conn = self.client.get_async_connection().await
            .map_err(|e| AppError::InternalServerError(format!("Redis connection failed: {}", e)))?;

        let deleted: i64 = conn.del(key).await
            .map_err(|e| AppError::InternalServerError(format!("Redis DEL failed: {}", e)))?;

        Ok(deleted > 0)
    }

    fn serialize<T>(&self, value: &T) -> Result<Vec<u8>, AppError>
    where
        T: Serialize,
    {
        let serialized = serde_json::to_vec(value)
            .map_err(|e| AppError::InternalServerError(format!("Serialization failed: {}", e)))?;

        if self.config.enable_compression && serialized.len() > self.config.compression_threshold {
            // In a real implementation, you'd use a compression library like flate2
            // For now, we'll just return the serialized data
            Ok(serialized)
        } else {
            Ok(serialized)
        }
    }

    fn deserialize<T>(&self, data: &[u8]) -> Result<T, AppError>
    where
        T: for<'de> Deserialize<'de>,
    {
        // In a real implementation, you'd check if data is compressed and decompress if needed
        serde_json::from_slice(data)
            .map_err(|e| AppError::InternalServerError(format!("Deserialization failed: {}", e)))
    }

    fn increment_hits(&self) {
        if let Ok(mut metrics) = self.metrics.write() {
            metrics.hits += 1;
        }
    }

    fn increment_misses(&self) {
        if let Ok(mut metrics) = self.metrics.write() {
            metrics.misses += 1;
        }
    }

    fn increment_sets(&self) {
        if let Ok(mut metrics) = self.metrics.write() {
            metrics.sets += 1;
        }
    }

    fn increment_deletes(&self) {
        if let Ok(mut metrics) = self.metrics.write() {
            metrics.deletes += 1;
        }
    }

    fn increment_errors(&self) {
        if let Ok(mut metrics) = self.metrics.write() {
            metrics.errors += 1;
        }
    }

    fn increment_total_operations(&self) {
        if let Ok(mut metrics) = self.metrics.write() {
            metrics.total_operations += 1;
        }
    }
}

impl Clone for CacheService {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            config: self.config.clone(),
            metrics: Arc::clone(&self.metrics),
        }
    }
}

// Cache-aside pattern implementation
pub struct CacheAsideService<T> {
    cache: CacheService,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> CacheAsideService<T>
where
    T: Serialize + for<'de> Deserialize<'de> + Debug + Clone,
{
    pub fn new(cache: CacheService) -> Self {
        Self {
            cache,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Get value with cache-aside pattern
    pub async fn get_or_load<F, Fut>(
        &self,
        key: &str,
        loader: F,
        ttl: Option<Duration>,
    ) -> Result<T, AppError>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, AppError>>,
    {
        // Try to get from cache first
        if let Some(cached_value) = self.cache.get::<T>(key).await? {
            return Ok(cached_value);
        }

        // Load from source
        let value = loader().await?;

        // Store in cache for next time
        self.cache.set(key, &value, ttl).await?;

        Ok(value)
    }

    /// Update value with write-through pattern
    pub async fn update<F, Fut>(
        &self,
        key: &str,
        updater: F,
        ttl: Option<Duration>,
    ) -> Result<T, AppError>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, AppError>>,
    {
        // Update the source
        let value = updater().await?;

        // Update cache
        self.cache.set(key, &value, ttl).await?;

        Ok(value)
    }

    /// Delete value from both cache and source
    pub async fn delete<F, Fut>(&self, key: &str, deleter: F) -> Result<(), AppError>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<(), AppError>>,
    {
        // Delete from source
        deleter().await?;

        // Delete from cache
        self.cache.delete(key).await?;

        Ok(())
    }
}

// Distributed cache lock implementation
pub struct DistributedLock {
    cache: CacheService,
    key: String,
    value: String,
    ttl: Duration,
}

impl DistributedLock {
    pub fn new(cache: CacheService, resource: &str, ttl: Duration) -> Self {
        let key = format!("lock:{}", resource);
        let value = Uuid::new_v4().to_string();
        
        Self {
            cache,
            key,
            value,
            ttl,
        }
    }

    /// Acquire the lock
    pub async fn acquire(&self) -> Result<bool, AppError> {
        let mut conn = self.cache.client.get_async_connection().await
            .map_err(|e| AppError::InternalServerError(format!("Redis connection failed: {}", e)))?;

        let full_key = self.cache.build_key(&self.key);
        
        // Use SET with NX (only if not exists) and EX (expiration)
        let result: Option<String> = redis::cmd("SET")
            .arg(&full_key)
            .arg(&self.value)
            .arg("NX")
            .arg("EX")
            .arg(self.ttl.as_secs())
            .query_async(&mut conn)
            .await
            .map_err(|e| AppError::InternalServerError(format!("Redis SET NX failed: {}", e)))?;

        Ok(result.is_some())
    }

    /// Release the lock
    pub async fn release(&self) -> Result<bool, AppError> {
        let mut conn = self.cache.client.get_async_connection().await
            .map_err(|e| AppError::InternalServerError(format!("Redis connection failed: {}", e)))?;

        let full_key = self.cache.build_key(&self.key);
        
        // Lua script to ensure we only delete our own lock
        let script = r#"
            if redis.call("GET", KEYS[1]) == ARGV[1] then
                return redis.call("DEL", KEYS[1])
            else
                return 0
            end
        "#;

        let result: i64 = redis::Script::new(script)
            .key(&full_key)
            .arg(&self.value)
            .invoke_async(&mut conn)
            .await
            .map_err(|e| AppError::InternalServerError(format!("Redis script failed: {}", e)))?;

        Ok(result > 0)
    }

    /// Extend the lock TTL
    pub async fn extend(&self, additional_ttl: Duration) -> Result<bool, AppError> {
        let mut conn = self.cache.client.get_async_connection().await
            .map_err(|e| AppError::InternalServerError(format!("Redis connection failed: {}", e)))?;

        let full_key = self.cache.build_key(&self.key);
        
        // Lua script to extend TTL only if we own the lock
        let script = r#"
            if redis.call("GET", KEYS[1]) == ARGV[1] then
                return redis.call("EXPIRE", KEYS[1], ARGV[2])
            else
                return 0
            end
        "#;

        let result: i64 = redis::Script::new(script)
            .key(&full_key)
            .arg(&self.value)
            .arg(additional_ttl.as_secs())
            .invoke_async(&mut conn)
            .await
            .map_err(|e| AppError::InternalServerError(format!("Redis script failed: {}", e)))?;

        Ok(result > 0)
    }
}

// Cache warming utilities
pub struct CacheWarmer {
    cache: CacheService,
}

impl CacheWarmer {
    pub fn new(cache: CacheService) -> Self {
        Self { cache }
    }

    /// Warm cache with frequently accessed data
    pub async fn warm_frequently_accessed_data(&self) -> Result<(), AppError> {
        info!("Starting cache warming for frequently accessed data");

        // This would typically load data from your database
        // and populate the cache with commonly accessed items
        
        // Example: Warm user sessions, popular raffles, etc.
        // In a real implementation, you'd query your database for this data
        
        info!("Cache warming completed");
        Ok(())
    }

    /// Warm cache for specific user
    pub async fn warm_user_data(&self, user_id: Uuid) -> Result<(), AppError> {
        debug!("Warming cache for user: {}", user_id);
        
        // Load and cache user-specific data
        // This would typically include user profile, preferences, recent activity, etc.
        
        Ok(())
    }

    /// Warm cache for specific raffle
    pub async fn warm_raffle_data(&self, raffle_id: Uuid) -> Result<(), AppError> {
        debug!("Warming cache for raffle: {}", raffle_id);
        
        // Load and cache raffle-specific data
        // This would include raffle details, participants, current state, etc.
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use redis::Client;

    async fn create_test_cache() -> CacheService {
        let client = Client::open("redis://127.0.0.1:6379").unwrap();
        let config = CacheConfig {
            key_prefix: "test".to_string(),
            ..Default::default()
        };
        CacheService::new(client, config)
    }

    #[tokio::test]
    async fn test_cache_set_get() {
        let cache = create_test_cache().await;
        
        let key = "test_key";
        let value = "test_value";
        
        cache.set(key, &value, None).await.unwrap();
        let retrieved: Option<String> = cache.get(key).await.unwrap();
        
        assert_eq!(retrieved, Some(value.to_string()));
    }

    #[tokio::test]
    async fn test_cache_expiration() {
        let cache = create_test_cache().await;
        
        let key = "expiring_key";
        let value = "expiring_value";
        let ttl = Duration::from_secs(1);
        
        cache.set(key, &value, Some(ttl)).await.unwrap();
        
        // Should exist immediately
        let retrieved: Option<String> = cache.get(key).await.unwrap();
        assert_eq!(retrieved, Some(value.to_string()));
        
        // Wait for expiration
        tokio::time::sleep(Duration::from_secs(2)).await;
        
        // Should be expired
        let retrieved: Option<String> = cache.get(key).await.unwrap();
        assert_eq!(retrieved, None);
    }

    #[tokio::test]
    async fn test_cache_delete() {
        let cache = create_test_cache().await;
        
        let key = "delete_key";
        let value = "delete_value";
        
        cache.set(key, &value, None).await.unwrap();
        let deleted = cache.delete(key).await.unwrap();
        assert!(deleted);
        
        let retrieved: Option<String> = cache.get(key).await.unwrap();
        assert_eq!(retrieved, None);
    }

    #[tokio::test]
    async fn test_cache_metrics() {
        let cache = create_test_cache().await;
        
        cache.reset_metrics().await;
        
        // Perform some operations
        let _: Option<String> = cache.get("nonexistent").await.unwrap();
        cache.set("test", &"value", None).await.unwrap();
        let _: Option<String> = cache.get("test").await.unwrap();
        
        let metrics = cache.get_metrics().await;
        assert_eq!(metrics.misses, 1);
        assert_eq!(metrics.sets, 1);
        assert_eq!(metrics.hits, 1);
        assert!(metrics.hit_rate() > 0.0);
    }

    #[tokio::test]
    async fn test_distributed_lock() {
        let cache = create_test_cache().await;
        let lock = DistributedLock::new(cache, "test_resource", Duration::from_secs(10));
        
        // Acquire lock
        let acquired = lock.acquire().await.unwrap();
        assert!(acquired);
        
        // Try to acquire again (should fail)
        let acquired_again = lock.acquire().await.unwrap();
        assert!(!acquired_again);
        
        // Release lock
        let released = lock.release().await.unwrap();
        assert!(released);
        
        // Should be able to acquire again
        let acquired_after_release = lock.acquire().await.unwrap();
        assert!(acquired_after_release);
    }
}