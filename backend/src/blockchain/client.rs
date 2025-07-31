use crate::blockchain::types::*;
use ethers::prelude::*;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::{sleep, timeout};
use tracing::{debug, error, info, warn};

/// Main blockchain client that manages connections and provides high-level blockchain operations
#[derive(Clone)]
pub struct BlockchainClient {
    provider: Arc<Provider<Http>>,
    ws_provider: Option<Arc<Provider<Ws>>>,
    signer: Arc<LocalWallet>,
    network_config: NetworkConfig,
    retry_config: RetryConfig,
}

impl BlockchainClient {
    /// Create a new blockchain client
    pub async fn new(
        network_config: NetworkConfig,
        private_key: &str,
        retry_config: Option<RetryConfig>,
    ) -> BlockchainResult<Self> {
        // Create HTTP provider
        let provider = Provider::<Http>::try_from(&network_config.rpc_url)
            .map_err(|e| BlockchainError::Configuration(format!("Invalid RPC URL: {}", e)))?;

        // Create WebSocket provider if URL is provided
        let ws_provider = if let Some(ws_url) = &network_config.ws_url {
            match Provider::<Ws>::connect(ws_url).await {
                Ok(ws) => Some(Arc::new(ws)),
                Err(e) => {
                    warn!("Failed to connect to WebSocket provider: {}", e);
                    None
                }
            }
        } else {
            None
        };

        // Create wallet from private key
        let wallet: LocalWallet = private_key
            .parse()
            .map_err(|e| BlockchainError::Wallet(format!("Invalid private key: {}", e)))?;

        // Set chain ID
        let wallet = wallet.with_chain_id(network_config.chain_id);

        let client = Self {
            provider: Arc::new(provider),
            ws_provider,
            signer: Arc::new(wallet),
            network_config,
            retry_config: retry_config.unwrap_or_default(),
        };

        // Verify connection
        client.verify_connection().await?;

        info!(
            "Blockchain client initialized for network: {} (chain_id: {})",
            client.network_config.name, client.network_config.chain_id
        );

        Ok(client)
    }

    /// Verify blockchain connection and configuration
    async fn verify_connection(&self) -> BlockchainResult<()> {
        // Check network connection
        let chain_id = self.provider.get_chainid().await?;
        if chain_id.as_u64() != self.network_config.chain_id {
            return Err(BlockchainError::Configuration(format!(
                "Chain ID mismatch: expected {}, got {}",
                self.network_config.chain_id,
                chain_id.as_u64()
            )));
        }

        // Check wallet balance
        let balance = self.provider.get_balance(self.signer.address(), None).await?;
        if balance.is_zero() {
            warn!(
                "Wallet {} has zero balance on network {}",
                self.signer.address(),
                self.network_config.name
            );
        }

        // Verify contract exists
        let code = self
            .provider
            .get_code(self.network_config.contract_address, None)
            .await?;
        if code.is_empty() {
            return Err(BlockchainError::Configuration(format!(
                "No contract found at address: {:?}",
                self.network_config.contract_address
            )));
        }

        debug!("Blockchain connection verified successfully");
        Ok(())
    }

    /// Get the HTTP provider
    pub fn provider(&self) -> Arc<Provider<Http>> {
        self.provider.clone()
    }

    /// Get the WebSocket provider if available
    pub fn ws_provider(&self) -> Option<Arc<Provider<Ws>>> {
        self.ws_provider.clone()
    }

    /// Get the signer
    pub fn signer(&self) -> Arc<LocalWallet> {
        self.signer.clone()
    }

    /// Get network configuration
    pub fn network_config(&self) -> &NetworkConfig {
        &self.network_config
    }

    /// Get current block number
    pub async fn get_block_number(&self) -> BlockchainResult<u64> {
        let block_number = self.provider.get_block_number().await?;
        Ok(block_number.as_u64())
    }

    /// Get current gas price
    pub async fn get_gas_price(&self) -> BlockchainResult<U256> {
        let gas_price = self.provider.get_gas_price().await?;
        Ok(gas_price)
    }

    /// Get wallet balance
    pub async fn get_balance(&self, address: Option<Address>) -> BlockchainResult<U256> {
        let addr = address.unwrap_or(self.signer.address());
        let balance = self.provider.get_balance(addr, None).await?;
        Ok(balance)
    }

    /// Get wallet nonce
    pub async fn get_nonce(&self, address: Option<Address>) -> BlockchainResult<U256> {
        let addr = address.unwrap_or(self.signer.address());
        let nonce = self.provider.get_transaction_count(addr, None).await?;
        Ok(nonce)
    }

    /// Get wallet information
    pub async fn get_wallet_info(&self, address: Option<Address>) -> BlockchainResult<WalletInfo> {
        let addr = address.unwrap_or(self.signer.address());
        
        let (balance, nonce, code) = tokio::try_join!(
            self.provider.get_balance(addr, None),
            self.provider.get_transaction_count(addr, None),
            self.provider.get_code(addr, None)
        )?;

        Ok(WalletInfo {
            address: addr,
            balance,
            nonce,
            is_contract: !code.is_empty(),
        })
    }

    /// Get transaction receipt with retry logic
    pub async fn get_transaction_receipt(
        &self,
        tx_hash: H256,
    ) -> BlockchainResult<Option<TransactionReceipt>> {
        self.retry_operation(|| async {
            self.provider.get_transaction_receipt(tx_hash).await
        })
        .await
    }

    /// Wait for transaction confirmation
    pub async fn wait_for_confirmation(
        &self,
        tx_hash: H256,
        confirmations: Option<usize>,
        timeout_secs: Option<u64>,
    ) -> BlockchainResult<TransactionReceipt> {
        let confirmations = confirmations.unwrap_or(self.network_config.confirmations);
        let timeout_duration = Duration::from_secs(timeout_secs.unwrap_or(300)); // 5 minutes default

        debug!(
            "Waiting for transaction {} with {} confirmations",
            tx_hash, confirmations
        );

        let receipt = timeout(timeout_duration, async {
            loop {
                if let Some(receipt) = self.get_transaction_receipt(tx_hash).await? {
                    if receipt.status == Some(U64::from(1)) {
                        // Transaction succeeded
                        let current_block = self.get_block_number().await?;
                        let tx_block = receipt.block_number.unwrap().as_u64();
                        let current_confirmations = current_block.saturating_sub(tx_block);

                        if current_confirmations >= confirmations as u64 {
                            return Ok(receipt);
                        }
                    } else {
                        // Transaction failed
                        return Err(BlockchainError::Transaction(format!(
                            "Transaction {} failed",
                            tx_hash
                        )));
                    }
                }

                // Wait before checking again
                sleep(Duration::from_secs(self.network_config.block_time)).await;
            }
        })
        .await
        .map_err(|_| {
            BlockchainError::Transaction(format!("Transaction {} confirmation timeout", tx_hash))
        })??;

        info!(
            "Transaction {} confirmed with {} confirmations",
            tx_hash, confirmations
        );

        Ok(receipt)
    }

    /// Get transaction status
    pub async fn get_transaction_status(&self, tx_hash: H256) -> BlockchainResult<TransactionStatus> {
        let receipt = self.get_transaction_receipt(tx_hash).await?;

        if let Some(receipt) = receipt {
            let current_block = self.get_block_number().await?;
            let tx_block = receipt.block_number.unwrap().as_u64();
            let confirmations = current_block.saturating_sub(tx_block);

            let status = if receipt.status == Some(U64::from(1)) {
                if confirmations >= self.network_config.confirmations as u64 {
                    TransactionState::Confirmed
                } else {
                    TransactionState::Pending
                }
            } else {
                TransactionState::Failed
            };

            Ok(TransactionStatus {
                hash: tx_hash,
                status,
                confirmations,
                gas_used: receipt.gas_used,
                effective_gas_price: receipt.effective_gas_price,
                block_number: receipt.block_number.map(|n| n.as_u64()),
                timestamp: None, // Would need to fetch block details for timestamp
            })
        } else {
            // Transaction not found, might be pending or dropped
            Ok(TransactionStatus {
                hash: tx_hash,
                status: TransactionState::Pending,
                confirmations: 0,
                gas_used: None,
                effective_gas_price: None,
                block_number: None,
                timestamp: None,
            })
        }
    }

    /// Check network health
    pub async fn check_network_health(&self) -> BlockchainResult<NetworkHealth> {
        let start_time = std::time::Instant::now();

        let (latest_block, gas_price) = tokio::try_join!(
            self.get_block_number(),
            self.get_gas_price()
        )?;

        let is_connected = start_time.elapsed() < Duration::from_secs(10);

        Ok(NetworkHealth {
            is_connected,
            latest_block,
            gas_price,
            peer_count: None, // Not available via HTTP provider
            sync_status: if is_connected {
                SyncStatus::Synced
            } else {
                SyncStatus::Unknown
            },
            last_check: chrono::Utc::now().timestamp() as u64,
        })
    }

    /// Retry operation with exponential backoff
    pub async fn retry_operation<F, Fut, T>(&self, operation: F) -> BlockchainResult<T>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T, ProviderError>>,
    {
        let mut delay = self.retry_config.initial_delay;
        let mut last_error = None;

        for attempt in 1..=self.retry_config.max_attempts {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    last_error = Some(e);
                    
                    if attempt < self.retry_config.max_attempts {
                        debug!(
                            "Operation failed (attempt {}/{}), retrying in {}ms",
                            attempt, self.retry_config.max_attempts, delay
                        );
                        
                        sleep(Duration::from_millis(delay)).await;
                        delay = std::cmp::min(
                            (delay as f64 * self.retry_config.backoff_multiplier) as u64,
                            self.retry_config.max_delay,
                        );
                    }
                }
            }
        }

        Err(BlockchainError::Provider(last_error.unwrap()))
    }

    /// Estimate gas for a transaction
    pub async fn estimate_gas(&self, tx: &TransactionRequest) -> BlockchainResult<U256> {
        let gas_estimate = self.provider.estimate_gas(tx, None).await?;
        
        // Add 20% buffer to gas estimate
        let gas_with_buffer = gas_estimate * U256::from(120) / U256::from(100);
        
        Ok(gas_with_buffer)
    }

    /// Get optimal gas price with multiplier
    pub async fn get_optimal_gas_price(&self) -> BlockchainResult<U256> {
        let base_gas_price = self.get_gas_price().await?;
        let multiplied_price = (base_gas_price.as_u128() as f64 * self.network_config.gas_price_multiplier) as u128;
        let optimal_price = U256::from(multiplied_price);
        
        // Cap at max gas price
        Ok(std::cmp::min(optimal_price, self.network_config.max_gas_price))
    }

    /// Create a signed middleware for contract interactions
    pub fn create_signed_middleware(&self) -> Arc<SignerMiddleware<Provider<Http>, LocalWallet>> {
        Arc::new(SignerMiddleware::new(
            self.provider.clone(),
            self.signer.clone(),
        ))
    }
}