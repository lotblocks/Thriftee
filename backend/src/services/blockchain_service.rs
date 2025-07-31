use crate::blockchain::*;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

/// High-level blockchain service that manages all blockchain operations
#[derive(Clone)]
pub struct BlockchainService {
    client: Arc<BlockchainClient>,
    transaction_manager: Arc<TransactionManager>,
    gas_manager: Arc<GasManager>,
    wallet_manager: Arc<WalletManager>,
    contract_client: Arc<RaffleContractClient>,
    config: BlockchainConfig,
}

impl BlockchainService {
    /// Initialize the blockchain service
    pub async fn new(
        network_name: &str,
        private_key: &str,
        master_seed: &[u8],
        config: BlockchainConfig,
    ) -> Result<Self> {
        // Get network configuration
        let network_config = config
            .networks
            .get(network_name)
            .ok_or_else(|| anyhow::anyhow!("Network {} not found in configuration", network_name))?
            .clone();

        info!("Initializing blockchain service for network: {}", network_name);

        // Initialize blockchain client
        let client = Arc::new(
            BlockchainClient::new(network_config, private_key, Some(config.retry_config.clone()))
                .await
                .map_err(|e| anyhow::anyhow!("Failed to create blockchain client: {}", e))?,
        );

        // Initialize transaction manager
        let transaction_manager = Arc::new(
            TransactionManager::new(client.clone())
                .await
                .map_err(|e| anyhow::anyhow!("Failed to create transaction manager: {}", e))?,
        );

        // Initialize gas manager
        let gas_manager = Arc::new(
            GasManager::new(client.clone(), None)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to create gas manager: {}", e))?,
        );

        // Initialize wallet manager
        let wallet_manager = Arc::new(WalletManager::new());
        wallet_manager
            .initialize_with_seed(master_seed)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to initialize wallet manager: {}", e))?;

        // Initialize contract client
        let contract_client = Arc::new(
            RaffleContractClient::new(client.clone(), transaction_manager.clone(), gas_manager.clone())
                .await
                .map_err(|e| anyhow::anyhow!("Failed to create contract client: {}", e))?,
        );

        info!("Blockchain service initialized successfully");

        Ok(Self {
            client,
            transaction_manager,
            gas_manager,
            wallet_manager,
            contract_client,
            config,
        })
    }

    /// Get the blockchain client
    pub fn client(&self) -> Arc<BlockchainClient> {
        self.client.clone()
    }

    /// Get the transaction manager
    pub fn transaction_manager(&self) -> Arc<TransactionManager> {
        self.transaction_manager.clone()
    }

    /// Get the gas manager
    pub fn gas_manager(&self) -> Arc<GasManager> {
        self.gas_manager.clone()
    }

    /// Get the wallet manager
    pub fn wallet_manager(&self) -> Arc<WalletManager> {
        self.wallet_manager.clone()
    }

    /// Get the contract client
    pub fn contract_client(&self) -> Arc<RaffleContractClient> {
        self.contract_client.clone()
    }

    /// Create a new raffle
    pub async fn create_raffle(
        &self,
        item_id: u64,
        total_boxes: u64,
        box_price_wei: u128,
        total_winners: u64,
    ) -> Result<uuid::Uuid> {
        let params = CreateRaffleParams {
            item_id,
            total_boxes,
            box_price: ethers::types::U256::from(box_price_wei),
            total_winners,
        };

        self.contract_client
            .create_raffle(params)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create raffle: {}", e))
    }

    /// Buy a box in a raffle for a user
    pub async fn buy_box_for_user(
        &self,
        user_id: &str,
        raffle_id: u64,
        encryption_key: &[u8],
    ) -> Result<uuid::Uuid> {
        // Get user's wallet address
        let wallet_address = self
            .wallet_manager
            .get_wallet_address(user_id)
            .await
            .ok_or_else(|| anyhow::anyhow!("Wallet not found for user: {}", user_id))?;

        // Get raffle details to determine box price
        let raffle = self
            .contract_client
            .get_raffle(raffle_id)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get raffle details: {}", e))?;

        // Buy box
        self.contract_client
            .buy_box(raffle_id, wallet_address, raffle.box_price)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to buy box: {}", e))
    }

    /// Get raffle details
    pub async fn get_raffle(&self, raffle_id: u64) -> Result<RaffleData> {
        self.contract_client
            .get_raffle(raffle_id)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get raffle: {}", e))
    }

    /// Get multiple raffles
    pub async fn get_raffles(&self, raffle_ids: Vec<u64>) -> Result<Vec<RaffleData>> {
        self.contract_client
            .get_raffles_batch(raffle_ids)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get raffles: {}", e))
    }

    /// Get active raffles
    pub async fn get_active_raffles(&self, limit: Option<u64>) -> Result<Vec<RaffleSummary>> {
        self.contract_client
            .get_active_raffles(limit)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get active raffles: {}", e))
    }

    /// Create wallet for user
    pub async fn create_user_wallet(
        &self,
        user_id: &str,
        encryption_key: &[u8],
    ) -> Result<crate::blockchain::wallet::WalletInfo> {
        self.wallet_manager
            .create_wallet_for_user(user_id, encryption_key)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create wallet: {}", e))
    }

    /// Get user wallet info
    pub async fn get_user_wallet_info(
        &self,
        user_id: &str,
    ) -> Option<crate::blockchain::wallet::WalletInfo> {
        self.wallet_manager.get_wallet_info(user_id).await
    }

    /// Get wallet balance
    pub async fn get_wallet_balance(&self, address: ethers::types::Address) -> Result<ethers::types::U256> {
        self.client
            .get_balance(Some(address))
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get balance: {}", e))
    }

    /// Get transaction status
    pub async fn get_transaction_status(&self, tx_id: uuid::Uuid) -> Option<TransactionStatus> {
        self.transaction_manager.get_transaction_status(tx_id).await
    }

    /// Get gas price recommendations
    pub async fn get_gas_recommendations(&self) -> Result<crate::blockchain::gas::GasPriorityLevels> {
        self.gas_manager
            .get_gas_prices_by_priority()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get gas recommendations: {}", e))
    }

    /// Get network health
    pub async fn get_network_health(&self) -> Result<NetworkHealth> {
        self.client
            .check_network_health()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to check network health: {}", e))
    }

    /// Cancel a raffle (admin only)
    pub async fn cancel_raffle(&self, raffle_id: u64, reason: String) -> Result<uuid::Uuid> {
        self.contract_client
            .cancel_raffle(raffle_id, reason)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to cancel raffle: {}", e))
    }

    /// Pause the contract (admin only)
    pub async fn pause_contract(&self) -> Result<uuid::Uuid> {
        self.contract_client
            .pause_contract()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to pause contract: {}", e))
    }

    /// Unpause the contract (admin only)
    pub async fn unpause_contract(&self) -> Result<uuid::Uuid> {
        self.contract_client
            .unpause_contract()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to unpause contract: {}", e))
    }

    /// Check if contract is paused
    pub async fn is_contract_paused(&self) -> Result<bool> {
        self.contract_client
            .is_paused()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to check contract status: {}", e))
    }

    /// Get contract address
    pub fn get_contract_address(&self) -> ethers::types::Address {
        self.contract_client.contract_address()
    }

    /// Cleanup old transactions
    pub async fn cleanup_old_transactions(&self, max_age_hours: u64) {
        self.transaction_manager
            .cleanup_completed_transactions(max_age_hours)
            .await;
    }

    /// Get blockchain statistics
    pub async fn get_blockchain_statistics(&self) -> Result<BlockchainStatistics> {
        let network_health = self.get_network_health().await?;
        let gas_stats = self
            .gas_manager
            .get_gas_statistics()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get gas statistics: {}", e))?;
        let wallet_stats = self.wallet_manager.get_wallet_statistics().await;
        let pending_transactions = self.transaction_manager.get_pending_transactions().await;

        Ok(BlockchainStatistics {
            network_health,
            gas_statistics: gas_stats,
            wallet_statistics: wallet_stats,
            pending_transaction_count: pending_transactions.len(),
            contract_address: self.get_contract_address(),
        })
    }

    /// Start background tasks
    pub async fn start_background_tasks(&self) {
        // Start transaction cleanup task
        let tx_manager = self.transaction_manager.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(3600)); // 1 hour
            loop {
                interval.tick().await;
                tx_manager.cleanup_completed_transactions(24).await; // Clean up transactions older than 24 hours
            }
        });

        // Start wallet cleanup task
        let wallet_manager = self.wallet_manager.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(86400)); // 24 hours
            loop {
                interval.tick().await;
                let cleaned = wallet_manager.cleanup_unused_wallets(30).await; // Clean up wallets unused for 30 days
                if cleaned > 0 {
                    info!("Cleaned up {} unused wallets", cleaned);
                }
            }
        });

        info!("Background blockchain tasks started");
    }
}

#[derive(Debug, Clone)]
pub struct BlockchainStatistics {
    pub network_health: NetworkHealth,
    pub gas_statistics: crate::blockchain::gas::GasStatistics,
    pub wallet_statistics: crate::blockchain::wallet::WalletStatistics,
    pub pending_transaction_count: usize,
    pub contract_address: ethers::types::Address,
}

/// Load blockchain configuration from environment or config file
pub fn load_blockchain_config() -> Result<BlockchainConfig> {
    let mut networks = HashMap::new();

    // Mumbai testnet configuration
    if let Ok(mumbai_rpc) = std::env::var("MUMBAI_RPC_URL") {
        let mumbai_config = NetworkConfig {
            name: "mumbai".to_string(),
            chain_id: 80001,
            rpc_url: mumbai_rpc,
            ws_url: std::env::var("MUMBAI_WS_URL").ok(),
            contract_address: std::env::var("MUMBAI_CONTRACT_ADDRESS")
                .unwrap_or_default()
                .parse()
                .unwrap_or_default(),
            vrf_coordinator: "0x7a1BaC17Ccc5b313516C5E16fb24f7659aA5ebed".parse().unwrap(),
            link_token: "0x326C977E6efc84E512bB9C30f76E30c160eD06FB".parse().unwrap(),
            key_hash: "0x4b09e658ed251bcafeebbc69400383d49f344ace09b9576fe248bb02c003fe9f".parse().unwrap(),
            subscription_id: std::env::var("VRF_SUBSCRIPTION_ID")
                .unwrap_or("1".to_string())
                .parse()
                .unwrap_or(1),
            confirmations: 3,
            gas_price_multiplier: 1.1,
            max_gas_price: ethers::types::U256::from(100_000_000_000u64), // 100 gwei
            block_time: 2,
        };
        networks.insert("mumbai".to_string(), mumbai_config);
    }

    // Polygon mainnet configuration
    if let Ok(polygon_rpc) = std::env::var("POLYGON_RPC_URL") {
        let polygon_config = NetworkConfig {
            name: "polygon".to_string(),
            chain_id: 137,
            rpc_url: polygon_rpc,
            ws_url: std::env::var("POLYGON_WS_URL").ok(),
            contract_address: std::env::var("POLYGON_CONTRACT_ADDRESS")
                .unwrap_or_default()
                .parse()
                .unwrap_or_default(),
            vrf_coordinator: "0xAE975071Be8F8eE67addBC1A82488F1C24858067".parse().unwrap(),
            link_token: "0xb0897686c545045aFc77CF20eC7A532E3120E0F1".parse().unwrap(),
            key_hash: "0xcc294a196eeeb44da2888d17c0625cc88d70d9760a69d58d853ba6581a9ab0cd".parse().unwrap(),
            subscription_id: std::env::var("VRF_SUBSCRIPTION_ID")
                .unwrap_or("1".to_string())
                .parse()
                .unwrap_or(1),
            confirmations: 12,
            gas_price_multiplier: 1.2,
            max_gas_price: ethers::types::U256::from(500_000_000_000u64), // 500 gwei
            block_time: 2,
        };
        networks.insert("polygon".to_string(), polygon_config);
    }

    // Local development configuration
    if let Ok(local_rpc) = std::env::var("LOCAL_RPC_URL") {
        let local_config = NetworkConfig {
            name: "local".to_string(),
            chain_id: 31337,
            rpc_url: local_rpc,
            ws_url: std::env::var("LOCAL_WS_URL").ok(),
            contract_address: std::env::var("LOCAL_CONTRACT_ADDRESS")
                .unwrap_or_default()
                .parse()
                .unwrap_or_default(),
            vrf_coordinator: ethers::types::Address::zero(), // Mock for local
            link_token: ethers::types::Address::zero(),
            key_hash: ethers::types::H256::zero(),
            subscription_id: 1,
            confirmations: 1,
            gas_price_multiplier: 1.0,
            max_gas_price: ethers::types::U256::from(20_000_000_000u64), // 20 gwei
            block_time: 1,
        };
        networks.insert("local".to_string(), local_config);
    }

    let default_network = std::env::var("BLOCKCHAIN_NETWORK").unwrap_or("mumbai".to_string());

    Ok(BlockchainConfig {
        networks,
        default_network,
        retry_config: RetryConfig::default(),
        monitoring: MonitoringConfig::default(),
    })
}