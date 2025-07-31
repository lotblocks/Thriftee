use super::*;
use crate::database::Database;
use std::env;
use tokio_test;

async fn setup_test_db() -> PgPool {
    let database_url = env::var("DATABASE_URL")
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
async fn test_hd_wallet_generation_with_bip44() {
    let pool = setup_test_db().await;
    let wallet_service = WalletService::new(pool);
    let user_id = Uuid::new_v4();
    let password = "test_password_123";

    let (address, encrypted_key, encrypted_mnemonic) = wallet_service
        .generate_wallet_for_user(user_id, password)
        .await
        .expect("Failed to generate HD wallet");

    // Verify address format
    assert!(address.starts_with("0x"));
    assert_eq!(address.len(), 42);
    
    // Verify encrypted data is not empty
    assert!(!encrypted_key.is_empty());
    assert!(!encrypted_mnemonic.is_empty());
    
    // Verify we can decrypt and use the wallet
    let decryption_key = derive_key_from_password(password, &format!("wallet_salt_{}", user_id).as_bytes()[..32]);
    let decrypted_mnemonic = decrypt_sensitive_data(&encrypted_mnemonic, &decryption_key)
        .expect("Failed to decrypt mnemonic");
    
    // Verify mnemonic is valid
    let mnemonic = Mnemonic::parse_in(Language::English, &decrypted_mnemonic)
        .expect("Invalid mnemonic phrase");
    
    // Verify mnemonic has correct word count (12 words)
    assert_eq!(decrypted_mnemonic.split_whitespace().count(), 12);
}

#[tokio::test]
async fn test_deterministic_wallet_generation() {
    let pool = setup_test_db().await;
    let wallet_service = WalletService::new(pool);
    let user_id = Uuid::new_v4();
    let password = "test_password_123";

    // Known test mnemonic
    let test_mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    
    // Generate wallet from same mnemonic multiple times
    let (address1, _) = wallet_service
        .generate_wallet_from_mnemonic(user_id, password, test_mnemonic, 0, 0)
        .await
        .expect("Failed to generate wallet from mnemonic");
    
    let (address2, _) = wallet_service
        .generate_wallet_from_mnemonic(user_id, password, test_mnemonic, 0, 0)
        .await
        .expect("Failed to generate wallet from mnemonic");
    
    // Should generate the same address
    assert_eq!(address1, address2);
    
    // Different account index should generate different address
    let (address3, _) = wallet_service
        .generate_wallet_from_mnemonic(user_id, password, test_mnemonic, 1, 0)
        .await
        .expect("Failed to generate wallet from mnemonic");
    
    assert_ne!(address1, address3);
}

#[tokio::test]
async fn test_wallet_balance_checking() {
    let pool = setup_test_db().await;
    let wallet_service = WalletService::new(pool);
    let user_id = Uuid::new_v4();
    let password = "test_password_123";

    // Generate wallet
    let (address, encrypted_key, _) = wallet_service
        .generate_wallet_for_user(user_id, password)
        .await
        .expect("Failed to generate wallet");

    // Create user record
    sqlx::query!(
        "INSERT INTO users (id, username, email, password_hash, internal_wallet_address, internal_wallet_private_key_encrypted) VALUES ($1, $2, $3, $4, $5, $6)",
        user_id,
        "testuser",
        "test@example.com",
        "hashed_password",
        address,
        encrypted_key
    )
    .execute(&pool)
    .await
    .expect("Failed to insert test user");

    // Test balance checking (will fail with test RPC, but tests the flow)
    let result = wallet_service
        .check_wallet_balance(user_id, "https://polygon-rpc.com")
        .await;
    
    // We expect this to fail in test environment, but it should be a connection error, not a logic error
    match result {
        Err(AppError::Internal(msg)) => {
            assert!(msg.contains("Failed to connect to provider") || msg.contains("Failed to get balance"));
        }
        Ok(balance) => {
            // If it succeeds (unlikely in test), balance should be valid
            assert!(balance >= U256::zero());
        }
        _ => panic!("Unexpected error type"),
    }
}

#[tokio::test]
async fn test_transaction_signing() {
    let pool = setup_test_db().await;
    let wallet_service = WalletService::new(pool);
    let user_id = Uuid::new_v4();
    let password = "test_password_123";

    // Generate wallet
    let (address, encrypted_key, _) = wallet_service
        .generate_wallet_for_user(user_id, password)
        .await
        .expect("Failed to generate wallet");

    // Create user record
    sqlx::query!(
        "INSERT INTO users (id, username, email, password_hash, internal_wallet_address, internal_wallet_private_key_encrypted) VALUES ($1, $2, $3, $4, $5, $6)",
        user_id,
        "testuser",
        "test@example.com",
        "hashed_password",
        address,
        encrypted_key
    )
    .execute(&pool)
    .await
    .expect("Failed to insert test user");

    // Create a test transaction
    let to_address = Address::from_str("0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6")
        .expect("Invalid address");
    
    let transaction = TransactionRequest::new()
        .to(to_address)
        .value(U256::from(1000000000000000000u64)) // 1 ETH in wei
        .gas(U256::from(21000))
        .gas_price(U256::from(20000000000u64)); // 20 gwei

    // Test transaction signing
    let signature = wallet_service
        .sign_transaction(user_id, password, transaction)
        .await
        .expect("Failed to sign transaction");

    // Verify signature is not empty
    assert_ne!(signature.r, U256::zero());
    assert_ne!(signature.s, U256::zero());
}

#[tokio::test]
async fn test_multiple_address_derivation() {
    let pool = setup_test_db().await;
    let wallet_service = WalletService::new(pool);
    let user_id = Uuid::new_v4();
    let password = "test_password_123";

    // Generate wallet
    let (address, encrypted_key, _) = wallet_service
        .generate_wallet_for_user(user_id, password)
        .await
        .expect("Failed to generate wallet");

    // Create user record
    sqlx::query!(
        "INSERT INTO users (id, username, email, password_hash, internal_wallet_address, internal_wallet_private_key_encrypted) VALUES ($1, $2, $3, $4, $5, $6)",
        user_id,
        "testuser",
        "test@example.com",
        "hashed_password",
        address,
        encrypted_key
    )
    .execute(&pool)
    .await
    .expect("Failed to insert test user");

    // Derive multiple addresses
    let addresses = wallet_service
        .derive_addresses(user_id, password, 10)
        .await
        .expect("Failed to derive addresses");

    assert_eq!(addresses.len(), 10);
    
    // All addresses should be valid and unique
    for (i, addr) in addresses.iter().enumerate() {
        assert!(addr.starts_with("0x"));
        assert_eq!(addr.len(), 42);
        
        // Check uniqueness
        for (j, other_addr) in addresses.iter().enumerate() {
            if i != j {
                assert_ne!(addr, other_addr, "Addresses at index {} and {} are the same", i, j);
            }
        }
    }
}

#[tokio::test]
async fn test_wallet_encryption_security() {
    let pool = setup_test_db().await;
    let wallet_service = WalletService::new(pool);
    let user_id = Uuid::new_v4();
    let password = "test_password_123";
    let wrong_password = "wrong_password";

    // Generate wallet
    let (address, encrypted_key, _) = wallet_service
        .generate_wallet_for_user(user_id, password)
        .await
        .expect("Failed to generate wallet");

    // Create user record
    sqlx::query!(
        "INSERT INTO users (id, username, email, password_hash, internal_wallet_address, internal_wallet_private_key_encrypted) VALUES ($1, $2, $3, $4, $5, $6)",
        user_id,
        "testuser",
        "test@example.com",
        "hashed_password",
        address,
        encrypted_key
    )
    .execute(&pool)
    .await
    .expect("Failed to insert test user");

    // Test correct password works
    let wallet = wallet_service
        .get_user_wallet(user_id, password)
        .await
        .expect("Failed to get wallet with correct password");
    
    assert_eq!(format!("{:?}", wallet.address()), address);

    // Test wrong password fails
    let result = wallet_service
        .get_user_wallet(user_id, wrong_password)
        .await;
    
    assert!(result.is_err(), "Should fail with wrong password");
}

#[tokio::test]
async fn test_private_key_export_import_cycle() {
    let pool = setup_test_db().await;
    let wallet_service = WalletService::new(pool);
    let user_id1 = Uuid::new_v4();
    let user_id2 = Uuid::new_v4();
    let password = "test_password_123";

    // Generate original wallet
    let (address1, encrypted_key1, _) = wallet_service
        .generate_wallet_for_user(user_id1, password)
        .await
        .expect("Failed to generate wallet");

    // Create user record
    sqlx::query!(
        "INSERT INTO users (id, username, email, password_hash, internal_wallet_address, internal_wallet_private_key_encrypted) VALUES ($1, $2, $3, $4, $5, $6)",
        user_id1,
        "testuser1",
        "test1@example.com",
        "hashed_password",
        address1,
        encrypted_key1
    )
    .execute(&pool)
    .await
    .expect("Failed to insert test user");

    // Export private key
    let exported_key = wallet_service
        .export_private_key(user_id1, password)
        .await
        .expect("Failed to export private key");

    // Import to new user
    let address2 = wallet_service
        .import_wallet(user_id2, password, &exported_key)
        .await
        .expect("Failed to import wallet");

    // Addresses should match
    assert_eq!(address1, address2);

    // Both wallets should be able to sign the same message identically
    let message = b"test message";
    
    let signature1 = wallet_service
        .sign_message(user_id1, password, message)
        .await
        .expect("Failed to sign with original wallet");

    // Create second user record for imported wallet
    sqlx::query!(
        "INSERT INTO users (id, username, email, password_hash, internal_wallet_address, internal_wallet_private_key_encrypted) VALUES ($1, $2, $3, $4, $5, $6)",
        user_id2,
        "testuser2",
        "test2@example.com",
        "hashed_password",
        address2,
        "dummy_encrypted_key" // Will be updated by import_wallet
    )
    .execute(&pool)
    .await
    .expect("Failed to insert second test user");

    let signature2 = wallet_service
        .sign_message(user_id2, password, message)
        .await
        .expect("Failed to sign with imported wallet");

    // Signatures should be identical
    assert_eq!(signature1.r, signature2.r);
    assert_eq!(signature1.s, signature2.s);
    assert_eq!(signature1.v, signature2.v);
}

#[test]
fn test_address_validation() {
    // Valid addresses
    assert!(WalletService::is_valid_address("0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6"));
    assert!(WalletService::is_valid_address("0x0000000000000000000000000000000000000000"));
    
    // Invalid addresses
    assert!(!WalletService::is_valid_address("invalid_address"));
    assert!(!WalletService::is_valid_address("0x123")); // Too short
    assert!(!WalletService::is_valid_address("742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6")); // Missing 0x
    assert!(!WalletService::is_valid_address("0xGGGd35Cc6634C0532925a3b8D4C9db96C4b4d8b6")); // Invalid hex
}

#[tokio::test]
async fn test_gas_estimation() {
    let pool = setup_test_db().await;
    let wallet_service = WalletService::new(pool);

    let gas_price = wallet_service
        .get_optimal_gas_price()
        .await
        .expect("Failed to get gas price");
    
    assert!(gas_price > U256::zero());
    assert!(gas_price <= U256::from(1000000000000u64)); // Should be reasonable (< 1000 gwei)

    let transaction = TransactionRequest::new()
        .to(Address::from_str("0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6").unwrap())
        .value(U256::from(1000000000000000000u64));

    let gas_estimate = wallet_service
        .estimate_gas(&transaction)
        .await
        .expect("Failed to estimate gas");
    
    assert!(gas_estimate > U256::zero());
    assert!(gas_estimate <= U256::from(10000000u64)); // Should be reasonable
}