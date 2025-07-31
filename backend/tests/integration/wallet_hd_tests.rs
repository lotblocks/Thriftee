use raffle_platform_backend::services::wallet_service::WalletService;
use raffle_platform_backend::models::User;
use raffle_platform_shared::CreateUserRequest;
use sqlx::PgPool;
use uuid::Uuid;

async fn setup_test_db() -> PgPool {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost:5432/raffle_platform_test".to_string());
    
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to test database");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    pool
}

#[tokio::test]
async fn test_hd_wallet_integration() {
    let pool = setup_test_db().await;
    let wallet_service = WalletService::new(pool.clone());
    let user_id = Uuid::new_v4();
    let password = "test_password_123";

    // Generate HD wallet
    let (address, encrypted_key, encrypted_mnemonic) = wallet_service
        .generate_wallet_for_user(user_id, password)
        .await
        .expect("Failed to generate HD wallet");

    // Create user with HD wallet
    let create_request = CreateUserRequest {
        username: "testuser".to_string(),
        email: "test@example.com".to_string(),
        phone_number: None,
    };

    let user = User::create(
        &pool,
        create_request,
        "hashed_password".to_string(),
        address.clone(),
        encrypted_key,
        Some(encrypted_mnemonic),
    )
    .await
    .expect("Failed to create user");

    // Verify user was created with wallet
    assert_eq!(user.internal_wallet_address, address);
    assert!(user.internal_wallet_mnemonic_encrypted.is_some());

    // Test wallet retrieval
    let wallet = wallet_service
        .get_user_wallet(user.id, password)
        .await
        .expect("Failed to retrieve wallet");

    assert_eq!(format!("{:?}", wallet.address()), address);

    // Test mnemonic retrieval
    let mnemonic = wallet_service
        .get_user_mnemonic(user.id, password)
        .await
        .expect("Failed to retrieve mnemonic");

    assert!(mnemonic.is_some());
    let mnemonic_phrase = mnemonic.unwrap();
    
    // Validate mnemonic format (should be 12 words)
    assert_eq!(mnemonic_phrase.split_whitespace().count(), 12);

    // Test wallet generation from retrieved mnemonic
    let (derived_address, _) = wallet_service
        .generate_wallet_from_mnemonic(user.id, password, &mnemonic_phrase, 0, 0)
        .await
        .expect("Failed to generate wallet from mnemonic");

    // Should generate the same address
    assert_eq!(derived_address, address);
}

#[tokio::test]
async fn test_bip44_derivation_paths() {
    let pool = setup_test_db().await;
    let wallet_service = WalletService::new(pool);
    let user_id = Uuid::new_v4();
    let password = "test_password_123";

    // Test mnemonic
    let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

    // Generate wallets with different account indices
    let (address1, _) = wallet_service
        .generate_wallet_from_mnemonic(user_id, password, mnemonic, 0, 0)
        .await
        .expect("Failed to generate wallet");

    let (address2, _) = wallet_service
        .generate_wallet_from_mnemonic(user_id, password, mnemonic, 1, 0)
        .await
        .expect("Failed to generate wallet");

    let (address3, _) = wallet_service
        .generate_wallet_from_mnemonic(user_id, password, mnemonic, 0, 1)
        .await
        .expect("Failed to generate wallet");

    // All addresses should be different
    assert_ne!(address1, address2);
    assert_ne!(address1, address3);
    assert_ne!(address2, address3);

    // But same parameters should generate same address
    let (address1_repeat, _) = wallet_service
        .generate_wallet_from_mnemonic(user_id, password, mnemonic, 0, 0)
        .await
        .expect("Failed to generate wallet");

    assert_eq!(address1, address1_repeat);
}

#[test]
fn test_mnemonic_validation() {
    // Valid mnemonic
    let valid_mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    assert!(WalletService::validate_mnemonic(valid_mnemonic).is_ok());

    // Invalid mnemonic (wrong word count)
    let invalid_mnemonic = "abandon abandon abandon";
    assert!(WalletService::validate_mnemonic(invalid_mnemonic).is_err());

    // Invalid mnemonic (invalid word)
    let invalid_word_mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon invalid";
    assert!(WalletService::validate_mnemonic(invalid_word_mnemonic).is_err());
}

#[test]
fn test_mnemonic_generation() {
    let mnemonic = WalletService::generate_mnemonic_phrase()
        .expect("Failed to generate mnemonic");

    // Should be 12 words
    assert_eq!(mnemonic.split_whitespace().count(), 12);

    // Should be valid
    assert!(WalletService::validate_mnemonic(&mnemonic).is_ok());

    // Generate another one - should be different
    let mnemonic2 = WalletService::generate_mnemonic_phrase()
        .expect("Failed to generate second mnemonic");

    assert_ne!(mnemonic, mnemonic2);
}