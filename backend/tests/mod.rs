// Test modules
pub mod integration;
pub mod e2e;
pub mod load;
pub mod security;
pub mod blockchain;

// Common test utilities
pub mod utils {
    pub mod test_helpers;
    pub mod test_database;
    pub mod mock_services;
}

// Test configuration
use once_cell::sync::Lazy;
use sqlx::PgPool;
use std::env;

pub static TEST_CONFIG: Lazy<TestConfig> = Lazy::new(|| {
    TestConfig::from_env()
});

#[derive(Debug, Clone)]
pub struct TestConfig {
    pub database_url: String,
    pub blockchain_rpc_url: String,
    pub stripe_secret_key: String,
    pub jwt_secret: String,
    pub test_timeout_seconds: u64,
    pub enable_blockchain_tests: bool,
    pub enable_load_tests: bool,
    pub max_concurrent_tests: usize,
}

impl TestConfig {
    pub fn from_env() -> Self {
        Self {
            database_url: env::var("TEST_DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://test:test@localhost/raffle_platform_test".to_string()),
            blockchain_rpc_url: env::var("TEST_BLOCKCHAIN_RPC_URL")
                .unwrap_or_else(|_| "http://localhost:8545".to_string()),
            stripe_secret_key: env::var("TEST_STRIPE_SECRET_KEY")
                .unwrap_or_else(|_| "sk_test_123456789".to_string()),
            jwt_secret: env::var("TEST_JWT_SECRET")
                .unwrap_or_else(|_| "test_jwt_secret_key_for_testing_only".to_string()),
            test_timeout_seconds: env::var("TEST_TIMEOUT_SECONDS")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .unwrap_or(30),
            enable_blockchain_tests: env::var("ENABLE_BLOCKCHAIN_TESTS")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
            enable_load_tests: env::var("ENABLE_LOAD_TESTS")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
            max_concurrent_tests: env::var("MAX_CONCURRENT_TESTS")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .unwrap_or(10),
        }
    }
}

// Test database setup and teardown
pub async fn setup_test_database() -> PgPool {
    let pool = PgPool::connect(&TEST_CONFIG.database_url)
        .await
        .expect("Failed to connect to test database");
    
    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");
    
    pool
}

pub async fn cleanup_test_database(pool: &PgPool) {
    // Clean up test data
    let tables = vec![
        "blockchain_events",
        "raffle_participants", 
        "user_credits",
        "raffles",
        "items",
        "users",
    ];
    
    for table in tables {
        sqlx::query(&format!("TRUNCATE TABLE {} CASCADE", table))
            .execute(pool)
            .await
            .unwrap_or_else(|e| {
                eprintln!("Warning: Failed to truncate table {}: {}", table, e);
            });
    }
}

// Test result aggregation
#[derive(Debug, Default)]
pub struct TestResults {
    pub total_tests: usize,
    pub passed_tests: usize,
    pub failed_tests: usize,
    pub skipped_tests: usize,
    pub test_duration: std::time::Duration,
}

impl TestResults {
    pub fn success_rate(&self) -> f64 {
        if self.total_tests == 0 {
            0.0
        } else {
            (self.passed_tests as f64 / self.total_tests as f64) * 100.0
        }
    }
    
    pub fn print_summary(&self) {
        println!("\n=== Test Results Summary ===");
        println!("Total tests: {}", self.total_tests);
        println!("Passed: {}", self.passed_tests);
        println!("Failed: {}", self.failed_tests);
        println!("Skipped: {}", self.skipped_tests);
        println!("Success rate: {:.2}%", self.success_rate());
        println!("Duration: {:?}", self.test_duration);
        println!("============================\n");
    }
}

// Custom test macros
#[macro_export]
macro_rules! integration_test {
    ($name:ident, $test_fn:expr) => {
        #[actix_web::test]
        async fn $name() {
            let pool = crate::tests::setup_test_database().await;
            
            let result = std::panic::AssertUnwindSafe($test_fn(pool.clone()))
                .catch_unwind()
                .await;
            
            crate::tests::cleanup_test_database(&pool).await;
            
            match result {
                Ok(Ok(())) => {},
                Ok(Err(e)) => panic!("Test failed: {:?}", e),
                Err(e) => std::panic::resume_unwind(e),
            }
        }
    };
}

#[macro_export]
macro_rules! load_test {
    ($name:ident, $test_fn:expr) => {
        #[actix_web::test]
        async fn $name() {
            if !crate::tests::TEST_CONFIG.enable_load_tests {
                println!("Skipping load test {} (disabled)", stringify!($name));
                return;
            }
            
            let pool = crate::tests::setup_test_database().await;
            
            let result = std::panic::AssertUnwindSafe($test_fn(pool.clone()))
                .catch_unwind()
                .await;
            
            crate::tests::cleanup_test_database(&pool).await;
            
            match result {
                Ok(Ok(())) => {},
                Ok(Err(e)) => panic!("Load test failed: {:?}", e),
                Err(e) => std::panic::resume_unwind(e),
            }
        }
    };
}

#[macro_export]
macro_rules! blockchain_test {
    ($name:ident, $test_fn:expr) => {
        #[actix_web::test]
        async fn $name() {
            if !crate::tests::TEST_CONFIG.enable_blockchain_tests {
                println!("Skipping blockchain test {} (disabled)", stringify!($name));
                return;
            }
            
            let pool = crate::tests::setup_test_database().await;
            
            let result = std::panic::AssertUnwindSafe($test_fn(pool.clone()))
                .catch_unwind()
                .await;
            
            crate::tests::cleanup_test_database(&pool).await;
            
            match result {
                Ok(Ok(())) => {},
                Ok(Err(e)) => panic!("Blockchain test failed: {:?}", e),
                Err(e) => std::panic::resume_unwind(e),
            }
        }
    };
}

// Test utilities for common operations
pub mod test_utils {
    use super::*;
    use crate::models::user::{User, UserRole};
    use uuid::Uuid;
    
    pub async fn create_test_user_with_role(
        pool: &PgPool, 
        email: &str, 
        password: &str,
        role: UserRole
    ) -> User {
        let password_hash = bcrypt::hash(password, bcrypt::DEFAULT_COST).unwrap();
        
        sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (email, password_hash, role, is_verified)
            VALUES ($1, $2, $3, true)
            RETURNING id, email, password_hash, role as "role: UserRole", 
                     is_verified, created_at, updated_at
            "#,
            email,
            password_hash,
            role as UserRole
        )
        .fetch_one(pool)
        .await
        .expect("Failed to create test user")
    }
    
    pub async fn cleanup_user(pool: &PgPool, email: &str) {
        sqlx::query!("DELETE FROM users WHERE email = $1", email)
            .execute(pool)
            .await
            .expect("Failed to cleanup user");
    }
    
    pub async fn wait_for_blockchain_confirmation() {
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }
    
    pub fn generate_test_email(prefix: &str) -> String {
        format!("{}+{}@test.example.com", prefix, Uuid::new_v4())
    }
}