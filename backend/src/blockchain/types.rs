use ethers::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BlockchainError {
    #[error("Contract error: {0}")]
    Contract(#[from] ContractError<SignerMiddleware<Provider<Http>, LocalWallet>>),
    
    #[error("Provider error: {0}")]
    Provider(#[from] ProviderError),
    
    #[error("Wallet error: {0}")]
    Wallet(String),
    
    #[error("Transaction failed: {0}")]
    Transaction(String),
    
    #[error("Gas estimation failed: {0}")]
    GasEstimation(String),
    
    #[error("Network error: {0}")]
    Network(String),
    
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("Event processing error: {0}")]
    EventProcessing(String),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub type BlockchainResult<T> = Result<T, BlockchainError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub name: String,
    pub chain_id: u64,
    pub rpc_url: String,
    pub ws_url: Option<String>,
    pub contract_address: Address,
    pub vrf_coordinator: Address,
    pub link_token: Address,
    pub key_hash: H256,
    pub subscription_id: u64,
    pub confirmations: usize,
    pub gas_price_multiplier: f64,
    pub max_gas_price: U256,
    pub block_time: u64, // seconds
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionConfig {
    pub gas_limit: U256,
    pub gas_price: Option<U256>,
    pub max_fee_per_gas: Option<U256>,
    pub max_priority_fee_per_gas: Option<U256>,
    pub nonce: Option<U256>,
    pub confirmations: usize,
    pub timeout: u64, // seconds
}

impl Default for TransactionConfig {
    fn default() -> Self {
        Self {
            gas_limit: U256::from(500_000),
            gas_price: None,
            max_fee_per_gas: None,
            max_priority_fee_per_gas: None,
            nonce: None,
            confirmations: 3,
            timeout: 300, // 5 minutes
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaffleData {
    pub id: U256,
    pub creator: Address,
    pub title: String,
    pub description: String,
    pub total_boxes: U256,
    pub price_per_box: U256,
    pub start_time: U256,
    pub end_time: U256,
    pub status: RaffleStatus,
    pub winner: Address,
    pub total_participants: U256,
    pub total_revenue: U256,
    pub item_image_url: String,
    pub item_description: String,
    pub created_at: U256,
    pub max_participants_per_user: U256,
    pub minimum_participants: U256,
    pub requires_whitelist: bool,
    pub whitelist_merkle_root: [u8; 32],
    pub creator_fee_percentage: U256,
    pub platform_fee_percentage: U256,
    pub payment_token: Address,
    pub random_seed: U256,
    pub is_verified: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RaffleStatus {
    Active = 0,
    Completed = 1,
    Cancelled = 2,
}

impl From<u8> for RaffleStatus {
    fn from(value: u8) -> Self {
        match value {
            0 => RaffleStatus::Active,
            1 => RaffleStatus::Completed,
            2 => RaffleStatus::Cancelled,
            _ => RaffleStatus::Active, // Default fallback
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipationPurchasedEvent {
    pub raffle_id: U256,
    pub participant: Address,
    pub participation_id: U256,
    pub boxes_purchased: U256,
    pub total_cost: U256,
    pub block_number: u64,
    pub transaction_hash: H256,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaffleCompletedEvent {
    pub raffle_id: U256,
    pub winner: Address,
    pub total_revenue: U256,
    pub total_participants: U256,
    pub random_seed: U256,
    pub block_number: u64,
    pub transaction_hash: H256,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaffleCreatedEvent {
    pub raffle_id: U256,
    pub creator: Address,
    pub title: String,
    pub total_boxes: U256,
    pub price_per_box: U256,
    pub start_time: U256,
    pub end_time: U256,
    pub payment_token: Address,
    pub block_number: u64,
    pub transaction_hash: H256,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaffleCancelledEvent {
    pub raffle_id: U256,
    pub reason: String,
    pub block_number: u64,
    pub transaction_hash: H256,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefundIssuedEvent {
    pub raffle_id: U256,
    pub participant: Address,
    pub amount: U256,
    pub block_number: u64,
    pub transaction_hash: H256,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RandomnessRequestedEvent {
    pub raffle_id: U256,
    pub request_id: U256,
    pub block_number: u64,
    pub transaction_hash: H256,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RandomnessFulfilledEvent {
    pub raffle_id: U256,
    pub random_seed: U256,
    pub block_number: u64,
    pub transaction_hash: H256,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaffleVerifiedEvent {
    pub raffle_id: U256,
    pub verifier: Address,
    pub block_number: u64,
    pub transaction_hash: H256,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionStatus {
    pub hash: H256,
    pub status: TransactionState,
    pub confirmations: u64,
    pub gas_used: Option<U256>,
    pub effective_gas_price: Option<U256>,
    pub block_number: Option<u64>,
    pub timestamp: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionState {
    Pending,
    Confirmed,
    Failed,
    Dropped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasEstimate {
    pub gas_limit: U256,
    pub gas_price: U256,
    pub max_fee_per_gas: Option<U256>,
    pub max_priority_fee_per_gas: Option<U256>,
    pub estimated_cost: U256,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletInfo {
    pub address: Address,
    pub balance: U256,
    pub nonce: U256,
    pub is_contract: bool,
}

// Event filter types
#[derive(Debug, Clone)]
pub struct EventFilter {
    pub from_block: Option<BlockNumber>,
    pub to_block: Option<BlockNumber>,
    pub addresses: Vec<Address>,
    pub topics: Vec<Option<H256>>,
}

impl Default for EventFilter {
    fn default() -> Self {
        Self {
            from_block: Some(BlockNumber::Latest),
            to_block: None,
            addresses: Vec::new(),
            topics: Vec::new(),
        }
    }
}

// Retry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub initial_delay: u64, // milliseconds
    pub max_delay: u64,     // milliseconds
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: 1000,  // 1 second
            max_delay: 30000,     // 30 seconds
            backoff_multiplier: 2.0,
        }
    }
}

// Contract interaction types
#[derive(Debug, Clone)]
pub struct CreateRaffleParams {
    pub title: String,
    pub description: String,
    pub total_boxes: u64,
    pub price_per_box: U256,
    pub start_time: u64,
    pub end_time: u64,
    pub item_image_url: String,
    pub item_description: String,
    pub max_participants_per_user: u64,
    pub minimum_participants: u64,
    pub requires_whitelist: bool,
    pub whitelist_merkle_root: [u8; 32],
    pub creator_fee_percentage: u64,
    pub payment_token: Address,
}

#[derive(Debug, Clone)]
pub struct PurchaseParticipationParams {
    pub raffle_id: u64,
    pub boxes_to_purchase: u64,
    pub merkle_proof: Vec<[u8; 32]>,
    pub buyer_address: Address,
    pub payment_amount: U256,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaffleStats {
    pub total_participants: U256,
    pub total_revenue: U256,
    pub boxes_remaining: U256,
    pub is_completed: bool,
    pub winner: Address,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleConstants {
    pub default_admin_role: [u8; 32],
    pub raffle_manager_role: [u8; 32],
    pub operator_role: [u8; 32],
    pub pauser_role: [u8; 32],
}

// Network health monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkHealth {
    pub is_connected: bool,
    pub latest_block: u64,
    pub gas_price: U256,
    pub peer_count: Option<u64>,
    pub sync_status: SyncStatus,
    pub last_check: u64, // timestamp
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncStatus {
    Synced,
    Syncing,
    NotSyncing,
    Unknown,
}

// Configuration for different environments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainConfig {
    pub networks: HashMap<String, NetworkConfig>,
    pub default_network: String,
    pub retry_config: RetryConfig,
    pub monitoring: MonitoringConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub health_check_interval: u64, // seconds
    pub event_processing_batch_size: usize,
    pub max_blocks_per_query: u64,
    pub confirmation_blocks: u64,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            health_check_interval: 30,
            event_processing_batch_size: 100,
            max_blocks_per_query: 1000,
            confirmation_blocks: 12,
        }
    }
}