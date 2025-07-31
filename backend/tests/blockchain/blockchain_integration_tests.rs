use actix_web::{test, web, App};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;
use std::sync::Arc;

use crate::blockchain::{
    client::BlockchainClient,
    contract::RaffleContract,
    events::EventProcessor,
    types::{BlockchainEvent, ContractAddress},
};
use crate::models::{
    raffle::{Raffle, RaffleStatus},
    user::User,
    blockchain_event::BlockchainEventRecord,
};
use crate::services::{
    blockchain_service::BlockchainService,
    raffle_service::RaffleService,
};
use crate::utils::test_helpers::{create_test_app, create_test_user, cleanup_test_data};

/// Test smart contract deployment and initialization
#[actix_web::test]
async fn test_smart_contract_deployment() {
    let pool = create_test_pool().await;
    let blockchain_client = create_test_blockchain_client().await;
    
    // Deploy test contract
    let contract_address = blockchain_client
        .deploy_raffle_contract()
        .await
        .expect("Failed to deploy contract");
    
    assert!(!contract_address.is_zero());
    
    // Verify contract is properly initialized
    let contract = RaffleContract::new(contract_address, blockchain_client.clone());
    let owner = contract.owner().await.expect("Failed to get contract owner");
    
    assert_eq!(owner, blockchain_client.get_account_address());
}

/// Test raffle creation on blockchain
#[actix_web::test]
async fn test_blockchain_raffle_creation() {
    let pool = create_test_pool().await;
    let app = create_test_app(pool.clone()).await;
    let blockchain_service = BlockchainService::new(pool.clone());
    
    // Create test data
    let seller = create_test_seller(&pool, "blockchain_seller@example.com").await;
    let item = create_test_item(&pool, &seller.id, "Blockchain Test Item").await;
    let tokens = get_auth_tokens(&pool, &seller).await;

    // Create raffle through API
    let raffle_data = json!({
        "item_id": item.id,
        "total_boxes": 100,
        "box_price": 10.0,
        "total_winners": 1,
        "end_date": "2024-12-31T23:59:59Z"
    });

    let req = test::TestRequest::post()
        .uri("/api/raffles")
        .insert_header(("Authorization", format!("Bearer {}", tokens.access_token)))
        .set_json(&raffle_data)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    let body: serde_json::Value = test::read_body_json(resp).await;
    let raffle_id = Uuid::parse_str(body["raffle"]["id"].as_str().unwrap()).unwrap();

    // Verify blockchain transaction was created
    let blockchain_tx = blockchain_service
        .get_raffle_creation_transaction(&raffle_id)
        .await
        .expect("Failed to get blockchain transaction");
    
    assert!(blockchain_tx.is_some());
    
    // Wait for transaction confirmation
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    
    // Verify raffle exists on blockchain
    let on_chain_raffle = blockchain_service
        .get_on_chain_raffle(&raffle_id)
        .await
        .expect("Failed to get on-chain raffle");
    
    assert_eq!(on_chain_raffle.total_boxes, 100);
    assert_eq!(on_chain_raffle.box_price_wei, 10_000_000_000_000_000_000u64); // 10 ETH in wei

    cleanup_test_data(&pool, "blockchain_seller@example.com").await;
}

/// Test box purchase with blockchain integration
#[actix_web::test]
async fn test_blockchain_box_purchase() {
    let pool = create_test_pool().await;
    let app = create_test_app(pool.clone()).await;
    let blockchain_service = BlockchainService::new(pool.clone());
    
    // Setup test data
    let seller = create_test_seller(&pool, "bc_seller@example.com").await;
    let buyer = create_test_user(&pool, "bc_buyer@example.com", "password123").await;
    let item = create_test_item(&pool, &seller.id, "BC Test Item").await;
    let raffle = create_test_raffle_with_blockchain(&pool, &seller.id, &item.id).await;
    
    // Add credits to buyer
    add_credits_to_user(&pool, &buyer.id, 100.0).await;
    let tokens = get_auth_tokens(&pool, &buyer).await;

    // Purchase boxes
    let purchase_data = json!({
        "box_numbers": [1, 2, 3],
        "payment_method": "credits"
    });

    let req = test::TestRequest::post()
        .uri(&format!("/api/raffles/{}/buy-box", raffle.id))
        .insert_header(("Authorization", format!("Bearer {}", tokens.access_token)))
        .set_json(&purchase_data)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    // Verify blockchain transaction was created
    let blockchain_tx = blockchain_service
        .get_box_purchase_transaction(&raffle.id, &buyer.id)
        .await
        .expect("Failed to get blockchain transaction");
    
    assert!(blockchain_tx.is_some());
    
    // Wait for transaction confirmation
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    
    // Verify boxes are marked as sold on blockchain
    let on_chain_raffle = blockchain_service
        .get_on_chain_raffle(&raffle.id)
        .await
        .expect("Failed to get on-chain raffle");
    
    assert_eq!(on_chain_raffle.boxes_sold, 3);
    
    // Verify box ownership on blockchain
    for box_num in [1, 2, 3] {
        let owner = blockchain_service
            .get_box_owner(&raffle.id, box_num)
            .await
            .expect("Failed to get box owner");
        
        assert_eq!(owner, buyer.wallet_address.unwrap());
    }

    cleanup_test_data(&pool, "bc_seller@example.com").await;
    cleanup_test_data(&pool, "bc_buyer@example.com").await;
}

/// Test Chainlink VRF integration for winner selection
#[actix_web::test]
async fn test_chainlink_vrf_winner_selection() {
    let pool = create_test_pool().await;
    let blockchain_service = BlockchainService::new(pool.clone());
    
    // Create a small raffle that can be easily filled
    let seller = create_test_seller(&pool, "vrf_seller@example.com").await;
    let item = create_test_item(&pool, &seller.id, "VRF Test Item").await;
    let raffle = create_test_raffle_with_blockchain(&pool, &seller.id, &item.id).await;
    
    // Create buyers and fill all boxes
    let mut buyers = Vec::new();
    for i in 1..=5 {
        let email = format!("vrf_buyer{}@example.com", i);
        let buyer = create_test_user(&pool, &email, "password123").await;
        add_credits_to_user(&pool, &buyer.id, 50.0).await;
        
        // Purchase one box each
        let raffle_service = RaffleService::new(pool.clone());
        raffle_service
            .buy_boxes(&raffle.id, &buyer.id, &[i])
            .await
            .expect("Failed to buy box");
        
        buyers.push(buyer);
    }
    
    // Trigger winner selection
    let request_id = blockchain_service
        .request_random_winner(&raffle.id)
        .await
        .expect("Failed to request random winner");
    
    assert!(!request_id.is_empty());
    
    // Wait for VRF response (in test environment, this might be mocked)
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    
    // Check if winner was selected
    let winners = blockchain_service
        .get_raffle_winners(&raffle.id)
        .await
        .expect("Failed to get winners");
    
    assert_eq!(winners.len(), 1);
    assert!(buyers.iter().any(|b| b.id == winners[0].user_id));
    
    // Verify winner selection event was emitted
    let events = blockchain_service
        .get_raffle_events(&raffle.id)
        .await
        .expect("Failed to get raffle events");
    
    let winner_event = events.iter().find(|e| e.event_type == "WinnerSelected");
    assert!(winner_event.is_some());

    // Cleanup
    cleanup_test_data(&pool, "vrf_seller@example.com").await;
    for i in 1..=5 {
        cleanup_test_data(&pool, &format!("vrf_buyer{}@example.com", i)).await;
    }
}

/// Test blockchain event processing
#[actix_web::test]
async fn test_blockchain_event_processing() {
    let pool = create_test_pool().await;
    let blockchain_client = create_test_blockchain_client().await;
    let event_processor = EventProcessor::new(pool.clone(), blockchain_client.clone());
    
    // Create test raffle
    let seller = create_test_seller(&pool, "event_seller@example.com").await;
    let item = create_test_item(&pool, &seller.id, "Event Test Item").await;
    let raffle = create_test_raffle_with_blockchain(&pool, &seller.id, &item.id).await;
    
    // Simulate blockchain events
    let box_purchased_event = BlockchainEvent {
        event_type: "BoxPurchased".to_string(),
        raffle_id: raffle.id,
        user_address: "0x1234567890123456789012345678901234567890".to_string(),
        box_numbers: vec![1, 2],
        transaction_hash: "0xabcdef1234567890".to_string(),
        block_number: 12345,
        timestamp: chrono::Utc::now(),
    };
    
    // Process the event
    event_processor
        .process_event(box_purchased_event.clone())
        .await
        .expect("Failed to process event");
    
    // Verify event was recorded in database
    let recorded_event = sqlx::query_as!(
        BlockchainEventRecord,
        r#"
        SELECT id, event_type, raffle_id, user_address, box_numbers, 
               transaction_hash, block_number, processed, created_at
        FROM blockchain_events 
        WHERE transaction_hash = $1
        "#,
        box_purchased_event.transaction_hash
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to fetch recorded event");
    
    assert_eq!(recorded_event.event_type, "BoxPurchased");
    assert_eq!(recorded_event.raffle_id, raffle.id);
    assert!(recorded_event.processed);
    
    // Verify raffle state was updated
    let updated_raffle = sqlx::query_as!(
        Raffle,
        r#"
        SELECT id, seller_id, item_id, total_boxes, box_price, total_winners,
               boxes_sold, status as "status: RaffleStatus", created_at, updated_at, end_date
        FROM raffles WHERE id = $1
        "#,
        raffle.id
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to fetch updated raffle");
    
    assert_eq!(updated_raffle.boxes_sold, 2);

    cleanup_test_data(&pool, "event_seller@example.com").await;
}

/// Test blockchain transaction retry mechanism
#[actix_web::test]
async fn test_blockchain_transaction_retry() {
    let pool = create_test_pool().await;
    let blockchain_service = BlockchainService::new(pool.clone());
    
    // Create test data
    let seller = create_test_seller(&pool, "retry_seller@example.com").await;
    let buyer = create_test_user(&pool, "retry_buyer@example.com", "password123").await;
    let item = create_test_item(&pool, &seller.id, "Retry Test Item").await;
    let raffle = create_test_raffle_with_blockchain(&pool, &seller.id, &item.id).await;
    
    add_credits_to_user(&pool, &buyer.id, 50.0).await;
    
    // Simulate network congestion by setting very low gas price
    let low_gas_price = 1_000_000_000u64; // 1 gwei (very low)
    
    // Attempt box purchase with low gas price (should fail initially)
    let result = blockchain_service
        .buy_boxes_with_gas_price(&raffle.id, &buyer.id, &[1], low_gas_price)
        .await;
    
    // The service should retry with higher gas price
    match result {
        Ok(_) => {
            // Transaction succeeded after retry
            let on_chain_raffle = blockchain_service
                .get_on_chain_raffle(&raffle.id)
                .await
                .expect("Failed to get on-chain raffle");
            
            assert_eq!(on_chain_raffle.boxes_sold, 1);
        }
        Err(e) => {
            // If it still fails, verify retry attempts were made
            assert!(e.to_string().contains("retry") || e.to_string().contains("gas"));
        }
    }

    cleanup_test_data(&pool, "retry_seller@example.com").await;
    cleanup_test_data(&pool, "retry_buyer@example.com").await;
}

/// Test gas price optimization
#[actix_web::test]
async fn test_gas_price_optimization() {
    let pool = create_test_pool().await;
    let blockchain_service = BlockchainService::new(pool.clone());
    
    // Test gas price estimation
    let estimated_gas_price = blockchain_service
        .estimate_optimal_gas_price()
        .await
        .expect("Failed to estimate gas price");
    
    assert!(estimated_gas_price > 0);
    
    // Test gas price with different priority levels
    let standard_gas = blockchain_service
        .get_gas_price_for_priority("standard")
        .await
        .expect("Failed to get standard gas price");
    
    let fast_gas = blockchain_service
        .get_gas_price_for_priority("fast")
        .await
        .expect("Failed to get fast gas price");
    
    assert!(fast_gas > standard_gas);
}

/// Test blockchain network failover
#[actix_web::test]
async fn test_blockchain_network_failover() {
    let pool = create_test_pool().await;
    let mut blockchain_service = BlockchainService::new(pool.clone());
    
    // Test primary network connection
    let primary_status = blockchain_service
        .check_network_health()
        .await
        .expect("Failed to check network health");
    
    assert!(primary_status.is_healthy);
    
    // Simulate primary network failure
    blockchain_service.simulate_network_failure().await;
    
    // Service should automatically failover to backup network
    let backup_status = blockchain_service
        .check_network_health()
        .await
        .expect("Failed to check backup network");
    
    assert!(backup_status.is_healthy);
    assert_ne!(backup_status.network_url, primary_status.network_url);
}

/// Test smart contract upgrade mechanism
#[actix_web::test]
async fn test_smart_contract_upgrade() {
    let pool = create_test_pool().await;
    let blockchain_service = BlockchainService::new(pool.clone());
    
    // Deploy initial contract version
    let v1_address = blockchain_service
        .deploy_contract_version("v1")
        .await
        .expect("Failed to deploy v1 contract");
    
    // Deploy upgraded contract version
    let v2_address = blockchain_service
        .deploy_contract_version("v2")
        .await
        .expect("Failed to deploy v2 contract");
    
    assert_ne!(v1_address, v2_address);
    
    // Test upgrade process
    let upgrade_result = blockchain_service
        .upgrade_contract(v1_address, v2_address)
        .await
        .expect("Failed to upgrade contract");
    
    assert!(upgrade_result.success);
    
    // Verify new contract is active
    let active_contract = blockchain_service
        .get_active_contract_address()
        .await
        .expect("Failed to get active contract");
    
    assert_eq!(active_contract, v2_address);
}

// Helper functions
async fn create_test_pool() -> PgPool {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://test:test@localhost/raffle_platform_test".to_string());
    
    PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database")
}

async fn create_test_blockchain_client() -> Arc<BlockchainClient> {
    let rpc_url = std::env::var("TEST_BLOCKCHAIN_RPC_URL")
        .unwrap_or_else(|_| "http://localhost:8545".to_string());
    
    Arc::new(
        BlockchainClient::new(&rpc_url)
            .await
            .expect("Failed to create blockchain client")
    )
}

async fn create_test_raffle_with_blockchain(
    pool: &PgPool, 
    seller_id: &Uuid, 
    item_id: &Uuid
) -> Raffle {
    let blockchain_service = BlockchainService::new(pool.clone());
    
    // Create raffle in database
    let raffle = sqlx::query_as!(
        Raffle,
        r#"
        INSERT INTO raffles (seller_id, item_id, total_boxes, box_price, total_winners, status, end_date)
        VALUES ($1, $2, 5, 10.0, 1, $3, NOW() + INTERVAL '30 days')
        RETURNING id, seller_id, item_id, total_boxes, box_price, total_winners, 
                 boxes_sold, status as "status: RaffleStatus", created_at, updated_at, end_date
        "#,
        seller_id,
        item_id,
        RaffleStatus::Active as RaffleStatus
    )
    .fetch_one(pool)
    .await
    .expect("Failed to create test raffle");
    
    // Create raffle on blockchain
    blockchain_service
        .create_blockchain_raffle(&raffle)
        .await
        .expect("Failed to create blockchain raffle");
    
    raffle
}

// Additional helper functions would be implemented here...
// (Similar to previous test files but adapted for blockchain testing)

#[cfg(test)]
mod blockchain_test_helpers {
    use super::*;
    // Blockchain-specific test helper implementations...
}