use crate::blockchain::types::*;
use crate::blockchain::client::BlockchainClient;
use ethers::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Transaction manager handles transaction lifecycle, retry logic, and monitoring
#[derive(Clone)]
pub struct TransactionManager {
    client: Arc<BlockchainClient>,
    pending_transactions: Arc<RwLock<HashMap<Uuid, PendingTransaction>>>,
    nonce_manager: Arc<RwLock<NonceManager>>,
}

#[derive(Debug, Clone)]
struct PendingTransaction {
    id: Uuid,
    hash: Option<H256>,
    request: TransactionRequest,
    config: TransactionConfig,
    status: TransactionState,
    attempts: u32,
    created_at: u64,
    last_attempt: Option<u64>,
}

#[derive(Debug, Clone)]
struct NonceManager {
    current_nonce: U256,
    pending_nonces: std::collections::BTreeSet<U256>,
    last_update: u64,
}

impl TransactionManager {
    /// Create a new transaction manager
    pub async fn new(client: Arc<BlockchainClient>) -> BlockchainResult<Self> {
        let current_nonce = client.get_nonce(None).await?;
        
        let nonce_manager = NonceManager {
            current_nonce,
            pending_nonces: std::collections::BTreeSet::new(),
            last_update: chrono::Utc::now().timestamp() as u64,
        };

        Ok(Self {
            client,
            pending_transactions: Arc::new(RwLock::new(HashMap::new())),
            nonce_manager: Arc::new(RwLock::new(nonce_manager)),
        })
    }

    /// Submit a transaction with automatic retry and monitoring
    pub async fn submit_transaction(
        &self,
        mut request: TransactionRequest,
        config: Option<TransactionConfig>,
    ) -> BlockchainResult<Uuid> {
        let config = config.unwrap_or_default();
        let tx_id = Uuid::new_v4();

        // Prepare transaction
        self.prepare_transaction(&mut request, &config).await?;

        // Create pending transaction
        let pending_tx = PendingTransaction {
            id: tx_id,
            hash: None,
            request: request.clone(),
            config: config.clone(),
            status: TransactionState::Pending,
            attempts: 0,
            created_at: chrono::Utc::now().timestamp() as u64,
            last_attempt: None,
        };

        // Store pending transaction
        {
            let mut pending = self.pending_transactions.write().await;
            pending.insert(tx_id, pending_tx);
        }

        // Submit transaction
        self.attempt_transaction_submission(tx_id).await?;

        info!("Transaction {} submitted successfully", tx_id);
        Ok(tx_id)
    }

    /// Prepare transaction with gas estimation and nonce management
    async fn prepare_transaction(
        &self,
        request: &mut TransactionRequest,
        config: &TransactionConfig,
    ) -> BlockchainResult<()> {
        // Set sender
        request.from = Some(self.client.signer().address());

        // Set nonce if not provided
        if request.nonce.is_none() {
            request.nonce = Some(self.get_next_nonce().await?);
        }

        // Estimate gas if not provided
        if request.gas.is_none() {
            let gas_estimate = self.client.estimate_gas(request).await?;
            request.gas = Some(std::cmp::min(gas_estimate, config.gas_limit));
        }

        // Set gas price if not provided
        if request.gas_price.is_none() && config.gas_price.is_some() {
            request.gas_price = config.gas_price;
        } else if request.gas_price.is_none() {
            request.gas_price = Some(self.client.get_optimal_gas_price().await?);
        }

        // Set EIP-1559 fields if provided
        if let Some(max_fee) = config.max_fee_per_gas {
            request.max_fee_per_gas = Some(max_fee);
        }
        if let Some(max_priority_fee) = config.max_priority_fee_per_gas {
            request.max_priority_fee_per_gas = Some(max_priority_fee);
        }

        Ok(())
    }

    /// Get next available nonce
    async fn get_next_nonce(&self) -> BlockchainResult<U256> {
        let mut nonce_manager = self.nonce_manager.write().await;
        
        // Update nonce from blockchain if it's been a while
        let now = chrono::Utc::now().timestamp() as u64;
        if now - nonce_manager.last_update > 60 {
            let blockchain_nonce = self.client.get_nonce(None).await?;
            if blockchain_nonce > nonce_manager.current_nonce {
                nonce_manager.current_nonce = blockchain_nonce;
                nonce_manager.pending_nonces.clear();
            }
            nonce_manager.last_update = now;
        }

        // Find next available nonce
        let mut next_nonce = nonce_manager.current_nonce;
        while nonce_manager.pending_nonces.contains(&next_nonce) {
            next_nonce += U256::one();
        }

        nonce_manager.pending_nonces.insert(next_nonce);
        Ok(next_nonce)
    }

    /// Release nonce when transaction is confirmed or failed
    async fn release_nonce(&self, nonce: U256) {
        let mut nonce_manager = self.nonce_manager.write().await;
        nonce_manager.pending_nonces.remove(&nonce);
    }

    /// Attempt to submit transaction
    async fn attempt_transaction_submission(&self, tx_id: Uuid) -> BlockchainResult<()> {
        let (request, config) = {
            let mut pending = self.pending_transactions.write().await;
            let tx = pending.get_mut(&tx_id).ok_or_else(|| {
                BlockchainError::Transaction("Transaction not found".to_string())
            })?;

            tx.attempts += 1;
            tx.last_attempt = Some(chrono::Utc::now().timestamp() as u64);

            (tx.request.clone(), tx.config.clone())
        };

        // Create signed middleware
        let middleware = self.client.create_signed_middleware();

        // Submit transaction
        match middleware.send_transaction(request, None).await {
            Ok(pending_tx) => {
                let tx_hash = *pending_tx.tx_hash();
                
                // Update pending transaction with hash
                {
                    let mut pending = self.pending_transactions.write().await;
                    if let Some(tx) = pending.get_mut(&tx_id) {
                        tx.hash = Some(tx_hash);
                        tx.status = TransactionState::Pending;
                    }
                }

                debug!("Transaction {} submitted with hash: {}", tx_id, tx_hash);

                // Start monitoring transaction
                let manager = self.clone();
                tokio::spawn(async move {
                    if let Err(e) = manager.monitor_transaction(tx_id, tx_hash, config).await {
                        error!("Failed to monitor transaction {}: {}", tx_id, e);
                    }
                });

                Ok(())
            }
            Err(e) => {
                error!("Failed to submit transaction {}: {}", tx_id, e);
                
                // Check if we should retry
                let should_retry = {
                    let pending = self.pending_transactions.read().await;
                    if let Some(tx) = pending.get(&tx_id) {
                        tx.attempts < 3 && self.is_retryable_error(&e)
                    } else {
                        false
                    }
                };

                if should_retry {
                    warn!("Retrying transaction {} (attempt {})", tx_id, {
                        let pending = self.pending_transactions.read().await;
                        pending.get(&tx_id).map(|tx| tx.attempts).unwrap_or(0)
                    });
                    
                    // Wait before retry
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    return self.attempt_transaction_submission(tx_id).await;
                } else {
                    // Mark as failed
                    {
                        let mut pending = self.pending_transactions.write().await;
                        if let Some(tx) = pending.get_mut(&tx_id) {
                            tx.status = TransactionState::Failed;
                            if let Some(nonce) = tx.request.nonce {
                                self.release_nonce(nonce).await;
                            }
                        }
                    }
                    
                    return Err(BlockchainError::Transaction(format!(
                        "Transaction submission failed after retries: {}",
                        e
                    )));
                }
            }
        }
    }

    /// Monitor transaction until confirmation or failure
    async fn monitor_transaction(
        &self,
        tx_id: Uuid,
        tx_hash: H256,
        config: TransactionConfig,
    ) -> BlockchainResult<()> {
        let timeout_duration = tokio::time::Duration::from_secs(config.timeout);
        let start_time = std::time::Instant::now();

        loop {
            // Check timeout
            if start_time.elapsed() > timeout_duration {
                warn!("Transaction {} monitoring timeout", tx_id);
                {
                    let mut pending = self.pending_transactions.write().await;
                    if let Some(tx) = pending.get_mut(&tx_id) {
                        tx.status = TransactionState::Dropped;
                        if let Some(nonce) = tx.request.nonce {
                            self.release_nonce(nonce).await;
                        }
                    }
                }
                break;
            }

            // Check transaction status
            match self.client.get_transaction_status(tx_hash).await {
                Ok(status) => {
                    match status.status {
                        TransactionState::Confirmed => {
                            info!("Transaction {} confirmed", tx_id);
                            {
                                let mut pending = self.pending_transactions.write().await;
                                if let Some(tx) = pending.get_mut(&tx_id) {
                                    tx.status = TransactionState::Confirmed;
                                    if let Some(nonce) = tx.request.nonce {
                                        self.release_nonce(nonce).await;
                                    }
                                }
                            }
                            break;
                        }
                        TransactionState::Failed => {
                            error!("Transaction {} failed", tx_id);
                            {
                                let mut pending = self.pending_transactions.write().await;
                                if let Some(tx) = pending.get_mut(&tx_id) {
                                    tx.status = TransactionState::Failed;
                                    if let Some(nonce) = tx.request.nonce {
                                        self.release_nonce(nonce).await;
                                    }
                                }
                            }
                            break;
                        }
                        TransactionState::Pending => {
                            // Continue monitoring
                            debug!("Transaction {} still pending", tx_id);
                        }
                        TransactionState::Dropped => {
                            warn!("Transaction {} was dropped", tx_id);
                            {
                                let mut pending = self.pending_transactions.write().await;
                                if let Some(tx) = pending.get_mut(&tx_id) {
                                    tx.status = TransactionState::Dropped;
                                    if let Some(nonce) = tx.request.nonce {
                                        self.release_nonce(nonce).await;
                                    }
                                }
                            }
                            break;
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to check transaction {} status: {}", tx_id, e);
                }
            }

            // Wait before next check
            tokio::time::sleep(tokio::time::Duration::from_secs(
                self.client.network_config().block_time,
            ))
            .await;
        }

        Ok(())
    }

    /// Check if error is retryable
    fn is_retryable_error(&self, error: &ContractError<SignerMiddleware<Provider<Http>, LocalWallet>>) -> bool {
        match error {
            ContractError::ProviderError { e } => {
                // Network errors are generally retryable
                matches!(e, ProviderError::HTTPError(_) | ProviderError::JsonRpcClientError(_))
            }
            ContractError::MiddlewareError { e: _ } => true, // Middleware errors might be retryable
            _ => false, // Contract revert errors are not retryable
        }
    }

    /// Get transaction status
    pub async fn get_transaction_status(&self, tx_id: Uuid) -> Option<TransactionStatus> {
        let pending = self.pending_transactions.read().await;
        if let Some(tx) = pending.get(&tx_id) {
            if let Some(hash) = tx.hash {
                // Try to get detailed status from blockchain
                if let Ok(status) = self.client.get_transaction_status(hash).await {
                    return Some(status);
                }
            }
            
            // Return basic status from pending transaction
            Some(TransactionStatus {
                hash: tx.hash.unwrap_or_default(),
                status: tx.status.clone(),
                confirmations: 0,
                gas_used: None,
                effective_gas_price: None,
                block_number: None,
                timestamp: Some(tx.created_at),
            })
        } else {
            None
        }
    }

    /// Get all pending transactions
    pub async fn get_pending_transactions(&self) -> Vec<(Uuid, TransactionStatus)> {
        let pending = self.pending_transactions.read().await;
        let mut result = Vec::new();

        for (id, tx) in pending.iter() {
            let status = TransactionStatus {
                hash: tx.hash.unwrap_or_default(),
                status: tx.status.clone(),
                confirmations: 0,
                gas_used: None,
                effective_gas_price: None,
                block_number: None,
                timestamp: Some(tx.created_at),
            };
            result.push(*id, status);
        }

        result
    }

    /// Clean up old completed transactions
    pub async fn cleanup_completed_transactions(&self, max_age_hours: u64) {
        let cutoff_time = chrono::Utc::now().timestamp() as u64 - (max_age_hours * 3600);
        let mut pending = self.pending_transactions.write().await;
        
        pending.retain(|_, tx| {
            matches!(tx.status, TransactionState::Pending) || tx.created_at > cutoff_time
        });
    }

    /// Cancel a pending transaction (if possible)
    pub async fn cancel_transaction(&self, tx_id: Uuid) -> BlockchainResult<()> {
        let mut pending = self.pending_transactions.write().await;
        if let Some(tx) = pending.get_mut(&tx_id) {
            if matches!(tx.status, TransactionState::Pending) {
                tx.status = TransactionState::Dropped;
                if let Some(nonce) = tx.request.nonce {
                    self.release_nonce(nonce).await;
                }
                info!("Transaction {} cancelled", tx_id);
                Ok(())
            } else {
                Err(BlockchainError::Transaction(
                    "Cannot cancel non-pending transaction".to_string(),
                ))
            }
        } else {
            Err(BlockchainError::Transaction(
                "Transaction not found".to_string(),
            ))
        }
    }

    /// Speed up a pending transaction by increasing gas price
    pub async fn speed_up_transaction(
        &self,
        tx_id: Uuid,
        new_gas_price: U256,
    ) -> BlockchainResult<Uuid> {
        let original_request = {
            let pending = self.pending_transactions.read().await;
            let tx = pending.get(&tx_id).ok_or_else(|| {
                BlockchainError::Transaction("Transaction not found".to_string())
            })?;

            if !matches!(tx.status, TransactionState::Pending) {
                return Err(BlockchainError::Transaction(
                    "Can only speed up pending transactions".to_string(),
                ));
            }

            tx.request.clone()
        };

        // Create new transaction with higher gas price and same nonce
        let mut new_request = original_request;
        new_request.gas_price = Some(new_gas_price);

        let config = TransactionConfig {
            gas_price: Some(new_gas_price),
            ..Default::default()
        };

        // Submit replacement transaction
        self.submit_transaction(new_request, Some(config)).await
    }
}