use super::*;
use std::env;
use tokio_test;

fn setup_jwt_service() -> JwtService {
    env::set_var("JWT_SECRET", "test-secret-key-for-testing-only-must-be-at-least-32-chars");
    JwtService::new().expect("Failed to create JWT service")
}

async fn setup_test_db() -> sqlx::PgPool {
    let database_url = env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost:5432/raffle_platform_test".to_string());
    
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to test database");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    pool
}

#[tokio::test]
async fn test_enhanced_token_generation_and_validation() {
    let jwt_service = setup_jwt_service();
    let user_id = Uuid::new_v4();
    let username = "testuser".to_string();
    let email = "test@example.com".to_string();
    let role = UserRole::User;

    // Test token pair creation
    let token_pair = jwt_service
        .create_token_pair(user_id, username.clone(), email.clone(), role)
        .expect("Failed to create token pair");

    assert!(!token_pair.access_token.is_empty());
    assert!(!token_pair.refresh_token.is_empty());
    assert!(token_pair.expires_in > 0);

    // Validate access token
    let access_claims = jwt_service
        .validate_token(&token_pair.access_token)
        .expect("Failed to validate access token");

    assert_eq!(access_claims.sub, user_id.to_string());
    assert_eq!(access_claims.username, username);
    assert_eq!(access_claims.email, email);
    assert_eq!(access_claims.role, role);
    assert_eq!(access_claims.token_type, "access");

    // Validate refresh token
    let refresh_claims = jwt_service
        .validate_token(&token_pair.refresh_token)
        .expect("Failed to validate refresh token");

    assert_eq!(refresh_claims.token_type, "refresh");
    assert_eq!(refresh_claims.sub, user_id.to_string());
}

#[tokio::test]
async fn test_token_revocation() {
    let jwt_service = setup_jwt_service();
    let user_id = Uuid::new_v4();
    let username = "testuser".to_string();
    let email = "test@example.com".to_string();
    let role = UserRole::User;

    // Generate token
    let token = jwt_service
        .generate_access_token(user_id, username, email, role)
        .expect("Failed to generate token");

    // Validate token (should work)
    let claims = jwt_service
        .validate_token(&token)
        .expect("Failed to validate token");

    // Revoke token
    jwt_service
        .revoke_token(&claims.jti)
        .expect("Failed to revoke token");

    // Validate token again (should fail)
    let result = jwt_service.validate_token(&token);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("revoked"));
}

#[tokio::test]
async fn test_token_expiration_checks() {
    let jwt_service = setup_jwt_service();
    let user_id = Uuid::new_v4();
    let username = "testuser".to_string();
    let email = "test@example.com".to_string();
    let role = UserRole::User;

    let token = jwt_service
        .generate_access_token(user_id, username, email, role)
        .expect("Failed to generate token");

    // Test remaining time
    let remaining_time = jwt_service
        .get_token_remaining_time(&token)
        .expect("Failed to get remaining time");

    assert!(remaining_time > Duration::zero());
    assert!(remaining_time <= Duration::from_std(JWT_ACCESS_TOKEN_EXPIRY).unwrap());

    // Test expiration check
    let will_expire_soon = jwt_service
        .will_expire_within(&token, Duration::hours(1))
        .expect("Failed to check expiration");

    // Should not expire within 1 hour for a fresh token
    assert!(!will_expire_soon);

    // Test with longer duration
    let will_expire_eventually = jwt_service
        .will_expire_within(&token, Duration::days(1))
        .expect("Failed to check expiration");

    // Should expire within 1 day
    assert!(will_expire_eventually);
}

#[tokio::test]
async fn test_token_type_validation() {
    let jwt_service = setup_jwt_service();
    let user_id = Uuid::new_v4();
    let username = "testuser".to_string();
    let email = "test@example.com".to_string();
    let role = UserRole::User;

    let access_token = jwt_service
        .generate_access_token(user_id, username.clone(), email.clone(), role)
        .expect("Failed to generate access token");

    let refresh_token = jwt_service
        .generate_refresh_token(user_id, username, email, role)
        .expect("Failed to generate refresh token");

    // Test token type checks
    assert!(jwt_service.is_access_token(&access_token).unwrap());
    assert!(!jwt_service.is_refresh_token(&access_token).unwrap());

    assert!(jwt_service.is_refresh_token(&refresh_token).unwrap());
    assert!(!jwt_service.is_access_token(&refresh_token).unwrap());
}

#[tokio::test]
async fn test_unsafe_token_operations() {
    let jwt_service = setup_jwt_service();
    let user_id = Uuid::new_v4();
    let username = "testuser".to_string();
    let email = "test@example.com".to_string();
    let role = UserRole::User;

    let token = jwt_service
        .generate_access_token(user_id, username, email, role)
        .expect("Failed to generate token");

    // Test unsafe user ID extraction
    let extracted_id = jwt_service
        .extract_user_id_unsafe(&token)
        .expect("Failed to extract user ID");

    assert_eq!(extracted_id, user_id);

    // Test unsafe token validation
    let claims = jwt_service
        .validate_token_unsafe(&token)
        .expect("Failed to validate token unsafely");

    assert_eq!(claims.sub, user_id.to_string());

    // Test unsafe token decoding
    let decoded_claims = jwt_service
        .decode_token_unsafe(&token)
        .expect("Failed to decode token unsafely");

    assert_eq!(decoded_claims.sub, user_id.to_string());
}

#[tokio::test]
async fn test_refresh_token_flow() {
    let pool = setup_test_db().await;
    let jwt_service = setup_jwt_service();
    let user_id = Uuid::new_v4();
    let username = "testuser".to_string();
    let email = "test@example.com".to_string();
    let role = UserRole::User;

    // Create initial token pair
    let initial_tokens = jwt_service
        .create_token_pair(user_id, username.clone(), email.clone(), role)
        .expect("Failed to create initial token pair");

    // Create a user session in the database
    let refresh_hash = crate::utils::crypto::hash_token(&initial_tokens.refresh_token);
    sqlx::query!(
        "INSERT INTO user_sessions (user_id, refresh_token_hash, expires_at) VALUES ($1, $2, $3)",
        user_id,
        refresh_hash,
        Utc::now() + Duration::days(7)
    )
    .execute(&pool)
    .await
    .expect("Failed to create session");

    // Test refresh token flow
    let new_tokens = jwt_service
        .refresh_access_token(&initial_tokens.refresh_token, &pool)
        .await
        .expect("Failed to refresh access token");

    assert!(!new_tokens.access_token.is_empty());
    assert!(!new_tokens.refresh_token.is_empty());
    assert_ne!(new_tokens.access_token, initial_tokens.access_token);
    assert_ne!(new_tokens.refresh_token, initial_tokens.refresh_token);

    // Validate new tokens
    let new_access_claims = jwt_service
        .validate_token(&new_tokens.access_token)
        .expect("Failed to validate new access token");

    assert_eq!(new_access_claims.sub, user_id.to_string());
    assert_eq!(new_access_claims.token_type, "access");

    // Old refresh token should no longer work
    let old_refresh_result = jwt_service
        .refresh_access_token(&initial_tokens.refresh_token, &pool)
        .await;

    assert!(old_refresh_result.is_err());
}

#[tokio::test]
async fn test_security_checks() {
    let pool = setup_test_db().await;
    let jwt_service = setup_jwt_service();
    let user_id = Uuid::new_v4();
    let username = "testuser".to_string();
    let email = "test@example.com".to_string();
    let role = UserRole::User;

    // Create a user in the database
    sqlx::query!(
        "INSERT INTO users (id, username, email, password_hash, internal_wallet_address, internal_wallet_private_key_encrypted, is_active) VALUES ($1, $2, $3, $4, $5, $6, $7)",
        user_id,
        username,
        email,
        "password_hash",
        "0x1234567890123456789012345678901234567890",
        "encrypted_key",
        true
    )
    .execute(&pool)
    .await
    .expect("Failed to create user");

    let token = jwt_service
        .generate_access_token(user_id, username, email, role)
        .expect("Failed to generate token");

    // Test security checks with active user
    let claims = jwt_service
        .validate_token_with_security_checks(
            &token,
            &pool,
            Some("127.0.0.1".parse().unwrap()),
            Some("Mozilla/5.0 Test Browser"),
        )
        .await
        .expect("Failed to validate token with security checks");

    assert_eq!(claims.sub, user_id.to_string());

    // Deactivate user
    sqlx::query!(
        "UPDATE users SET is_active = false WHERE id = $1",
        user_id
    )
    .execute(&pool)
    .await
    .expect("Failed to deactivate user");

    // Security checks should now fail
    let result = jwt_service
        .validate_token_with_security_checks(
            &token,
            &pool,
            Some("127.0.0.1".parse().unwrap()),
            Some("Mozilla/5.0 Test Browser"),
        )
        .await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("deactivated"));
}

#[tokio::test]
async fn test_anomaly_detection() {
    let jwt_service = setup_jwt_service();
    let user_id = Uuid::new_v4();
    let username = "testuser".to_string();
    let email = "test@example.com".to_string();
    let role = UserRole::User;

    let token = jwt_service
        .generate_access_token(user_id, username, email, role)
        .expect("Failed to generate token");

    // Test anomaly detection
    let (claims, alerts) = jwt_service
        .validate_token_with_anomaly_detection(&token)
        .expect("Failed to validate token with anomaly detection");

    assert_eq!(claims.sub, user_id.to_string());
    
    // Fresh token should not have alerts
    assert!(alerts.is_empty());
}

#[tokio::test]
async fn test_jwt_id_operations() {
    let jwt_service = setup_jwt_service();
    let user_id = Uuid::new_v4();
    let username = "testuser".to_string();
    let email = "test@example.com".to_string();
    let role = UserRole::User;

    let token = jwt_service
        .generate_access_token(user_id, username, email, role)
        .expect("Failed to generate token");

    // Get JWT ID
    let jti = jwt_service
        .get_jwt_id(&token)
        .expect("Failed to get JWT ID");

    assert!(!jti.is_empty());

    // Validate that JTI is a valid UUID
    let _uuid = Uuid::parse_str(&jti).expect("JTI should be a valid UUID");

    // Get token expiration
    let expiration = jwt_service
        .get_token_expiration(&token)
        .expect("Failed to get token expiration");

    assert!(expiration > Utc::now());
}

#[tokio::test]
async fn test_token_cleanup() {
    let jwt_service = setup_jwt_service();

    // Add some revoked tokens
    jwt_service.revoke_token("test-jti-1").expect("Failed to revoke token");
    jwt_service.revoke_token("test-jti-2").expect("Failed to revoke token");

    // Test cleanup (in a real implementation, this would remove expired tokens)
    let cleaned_count = jwt_service
        .cleanup_revoked_tokens()
        .expect("Failed to cleanup revoked tokens");

    // Since we don't actually remove tokens in the simplified implementation,
    // the count should be 0
    assert_eq!(cleaned_count, 0);
}

#[test]
fn test_invalid_jwt_secret() {
    env::set_var("JWT_SECRET", "short"); // Too short
    let result = JwtService::new();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("at least 32 characters"));
}

#[test]
fn test_missing_jwt_secret() {
    env::remove_var("JWT_SECRET");
    let result = JwtService::new();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not set"));
}

#[tokio::test]
async fn test_concurrent_token_operations() {
    let jwt_service = Arc::new(setup_jwt_service());
    let user_id = Uuid::new_v4();
    let username = "testuser".to_string();
    let email = "test@example.com".to_string();
    let role = UserRole::User;

    // Test concurrent token generation
    let mut handles = Vec::new();
    
    for i in 0..10 {
        let jwt_service = jwt_service.clone();
        let username = format!("{}_{}", username, i);
        let email = format!("{}_{}", email, i);
        
        let handle = tokio::spawn(async move {
            jwt_service.create_token_pair(user_id, username, email, role)
        });
        
        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        let result = handle.await.expect("Task panicked");
        assert!(result.is_ok());
    }
}