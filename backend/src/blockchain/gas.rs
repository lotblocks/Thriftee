use crate::blockchain::types::*;
use crate::blockchain::client::BlockchainClient;
use ethers::prelude::*;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Gas manager handles gas price optimization and estimation
#[derive(Clone)]
pub struct GasManager {
    client: Arc<BlockchainClient>,
    gas_history: Arc<RwLock<GasHistory>>,
    config: GasConfig,
}

#[derive(Debug, Clone)]
struct GasHistory {
    prices: VecDeque<GasPricePoint>,
    last_update: u64,
}

#[derive(Debug, Clone)]
struct GasPricePoint {
    timestamp: u64,
    gas_price: U256,
    base_fee: Option<U256>,
    priority_fee: Option<U256>,
    block_number: u64,
}

#[derive(Debug, Clone)]
pub struct GasConfig {
    pub history_size: usize,
    pub update_interval: u64, // seconds
    pub price_multiplier: f64,
    pub max_gas_price: U256,
    pub min_gas_price: U256,
    pub priority_fee_multiplier: f64,
    pub max_priority_fee: U256,
}

impl Default for GasConfig {
    fn default() -> Self {
        Self {
            history_size: 100,
            update_interval: 30,
            price_multiplier: 1.1, // 10% above current price
            max_gas_price: U256::from(100_000_000_000u64), // 100 gwei
            min_gas_price: U256::from(1_000_000_000u64),   // 1 gwei
            priority_fee_multiplier: 1.2,
            max_priority_fee: U256::from(5_000_000_000u64), // 5 gwei
        }
    }
}

impl GasManager {
    /// Create a new gas manager
    pub async fn new(client: Arc<BlockchainClient>, config: Option<GasConfig>) -> BlockchainResult<Self> {
        let config = config.unwrap_or_default();
        
        let gas_history = GasHistory {
            prices: VecDeque::with_capacity(config.history_size),
            last_update: 0,
        };

        let manager = Self {
            client,
            gas_history: Arc::new(RwLock::new(gas_history)),
            config,
        };

        // Initialize with current gas price
        manager.update_gas_history().await?;

        Ok(manager)
    }

    /// Get current optimal gas price
    pub async fn get_optimal_gas_price(&self) -> BlockchainResult<U256> {
        self.update_gas_history_if_needed().await?;

        let history = self.gas_history.read().await;
        if let Some(latest) = history.prices.back() {
            let optimal_price = self.calculate_optimal_price(latest.gas_price);
            Ok(optimal_price)
        } else {
            // Fallback to current network price
            let current_price = self.client.get_gas_price().await?;
            Ok(self.calculate_optimal_price(current_price))
        }
    }

    /// Get EIP-1559 gas parameters
    pub async fn get_eip1559_gas_params(&self) -> BlockchainResult<(U256, U256)> {
        self.update_gas_history_if_needed().await?;

        let history = self.gas_history.read().await;
        if let Some(latest) = history.prices.back() {
            let base_fee = latest.base_fee.unwrap_or(latest.gas_price);
            let priority_fee = latest.priority_fee.unwrap_or_else(|| {
                // Estimate priority fee as 10% of base fee
                base_fee / U256::from(10)
            });

            let optimal_priority_fee = self.calculate_optimal_priority_fee(priority_fee);
            let max_fee_per_gas = base_fee * U256::from(2) + optimal_priority_fee;

            Ok((max_fee_per_gas, optimal_priority_fee))
        } else {
            // Fallback calculation
            let current_price = self.client.get_gas_price().await?;
            let priority_fee = current_price / U256::from(10);
            let max_fee = current_price * U256::from(2);
            
            Ok((max_fee, priority_fee))
        }
    }

    /// Estimate gas for a transaction with buffer
    pub async fn estimate_gas_with_buffer(
        &self,
        tx: &TransactionRequest,
        buffer_percent: Option<u8>,
    ) -> BlockchainResult<GasEstimate> {
        let buffer_percent = buffer_percent.unwrap_or(20); // 20% default buffer
        
        // Get base gas estimate
        let base_estimate = self.client.estimate_gas(tx).await?;
        
        // Add buffer
        let gas_limit = base_estimate * U256::from(100 + buffer_percent) / U256::from(100);
        
        // Get gas price information
        let gas_price = self.get_optimal_gas_price().await?;
        let (max_fee_per_gas, max_priority_fee_per_gas) = self.get_eip1559_gas_params().await?;
        
        // Calculate estimated cost
        let estimated_cost = gas_limit * gas_price;
        
        Ok(GasEstimate {
            gas_limit,
            gas_price,
            max_fee_per_gas: Some(max_fee_per_gas),
            max_priority_fee_per_gas: Some(max_priority_fee_per_gas),
            estimated_cost,
        })
    }

    /// Get gas price for different priority levels
    pub async fn get_gas_prices_by_priority(&self) -> BlockchainResult<GasPriorityLevels> {
        self.update_gas_history_if_needed().await?;
        
        let base_price = self.client.get_gas_price().await?;
        
        Ok(GasPriorityLevels {
            slow: self.calculate_price_with_multiplier(base_price, 0.9),    // 10% below
            standard: self.calculate_price_with_multiplier(base_price, 1.0), // Current price
            fast: self.calculate_price_with_multiplier(base_price, 1.2),     // 20% above
            instant: self.calculate_price_with_multiplier(base_price, 1.5),  // 50% above
        })
    }

    /// Update gas history if needed
    async fn update_gas_history_if_needed(&self) -> BlockchainResult<()> {
        let now = chrono::Utc::now().timestamp() as u64;
        let should_update = {
            let history = self.gas_history.read().await;
            now - history.last_update > self.config.update_interval
        };

        if should_update {
            self.update_gas_history().await?;
        }

        Ok(())
    }

    /// Update gas price history
    async fn update_gas_history(&self) -> BlockchainResult<()> {
        let now = chrono::Utc::now().timestamp() as u64;
        let block_number = self.client.get_block_number().await?;
        
        // Get current gas price
        let gas_price = self.client.get_gas_price().await?;
        
        // Try to get EIP-1559 information
        let (base_fee, priority_fee) = self.get_current_eip1559_info().await.unwrap_or((None, None));
        
        let price_point = GasPricePoint {
            timestamp: now,
            gas_price,
            base_fee,
            priority_fee,
            block_number,
        };

        let mut history = self.gas_history.write().await;
        
        // Add new price point
        history.prices.push_back(price_point);
        
        // Remove old entries if we exceed capacity
        while history.prices.len() > self.config.history_size {
            history.prices.pop_front();
        }
        
        history.last_update = now;
        
        debug!(
            "Updated gas history: current price = {} gwei, history size = {}",
            gas_price.as_u64() / 1_000_000_000,
            history.prices.len()
        );

        Ok(())
    }

    /// Get current EIP-1559 information
    async fn get_current_eip1559_info(&self) -> BlockchainResult<(Option<U256>, Option<U256>)> {
        // Try to get the latest block with full transaction details
        match self.client.provider().get_block(BlockNumber::Latest).await {
            Ok(Some(block)) => {
                let base_fee = block.base_fee_per_gas;
                // Priority fee would need to be calculated from recent transactions
                // For now, we'll estimate it
                let priority_fee = base_fee.map(|bf| bf / U256::from(10));
                Ok((base_fee, priority_fee))
            }
            _ => Ok((None, None)),
        }
    }

    /// Calculate optimal gas price with multiplier and bounds
    fn calculate_optimal_price(&self, base_price: U256) -> U256 {
        let multiplied = (base_price.as_u128() as f64 * self.config.price_multiplier) as u128;
        let optimal = U256::from(multiplied);
        
        // Apply bounds
        if optimal > self.config.max_gas_price {
            self.config.max_gas_price
        } else if optimal < self.config.min_gas_price {
            self.config.min_gas_price
        } else {
            optimal
        }
    }

    /// Calculate optimal priority fee
    fn calculate_optimal_priority_fee(&self, base_priority_fee: U256) -> U256 {
        let multiplied = (base_priority_fee.as_u128() as f64 * self.config.priority_fee_multiplier) as u128;
        let optimal = U256::from(multiplied);
        
        std::cmp::min(optimal, self.config.max_priority_fee)
    }

    /// Calculate price with specific multiplier
    fn calculate_price_with_multiplier(&self, base_price: U256, multiplier: f64) -> U256 {
        let multiplied = (base_price.as_u128() as f64 * multiplier) as u128;
        let result = U256::from(multiplied);
        
        // Apply bounds
        if result > self.config.max_gas_price {
            self.config.max_gas_price
        } else if result < self.config.min_gas_price {
            self.config.min_gas_price
        } else {
            result
        }
    }

    /// Get gas price statistics
    pub async fn get_gas_statistics(&self) -> BlockchainResult<GasStatistics> {
        self.update_gas_history_if_needed().await?;
        
        let history = self.gas_history.read().await;
        
        if history.prices.is_empty() {
            return Ok(GasStatistics::default());
        }

        let prices: Vec<U256> = history.prices.iter().map(|p| p.gas_price).collect();
        
        let min_price = prices.iter().min().copied().unwrap_or_default();
        let max_price = prices.iter().max().copied().unwrap_or_default();
        let avg_price = if !prices.is_empty() {
            let sum: U256 = prices.iter().fold(U256::zero(), |acc, &price| acc + price);
            sum / U256::from(prices.len())
        } else {
            U256::zero()
        };

        // Calculate median
        let mut sorted_prices = prices.clone();
        sorted_prices.sort();
        let median_price = if sorted_prices.is_empty() {
            U256::zero()
        } else if sorted_prices.len() % 2 == 0 {
            let mid = sorted_prices.len() / 2;
            (sorted_prices[mid - 1] + sorted_prices[mid]) / U256::from(2)
        } else {
            sorted_prices[sorted_prices.len() / 2]
        };

        Ok(GasStatistics {
            current_price: history.prices.back().map(|p| p.gas_price).unwrap_or_default(),
            min_price,
            max_price,
            avg_price,
            median_price,
            sample_count: prices.len(),
            last_update: history.last_update,
        })
    }

    /// Predict gas price trend
    pub async fn predict_gas_trend(&self) -> BlockchainResult<GasTrend> {
        self.update_gas_history_if_needed().await?;
        
        let history = self.gas_history.read().await;
        
        if history.prices.len() < 10 {
            return Ok(GasTrend::Stable); // Not enough data
        }

        // Compare recent prices with older prices
        let recent_count = std::cmp::min(5, history.prices.len() / 2);
        let recent_prices: Vec<U256> = history.prices
            .iter()
            .rev()
            .take(recent_count)
            .map(|p| p.gas_price)
            .collect();

        let older_prices: Vec<U256> = history.prices
            .iter()
            .rev()
            .skip(recent_count)
            .take(recent_count)
            .map(|p| p.gas_price)
            .collect();

        if recent_prices.is_empty() || older_prices.is_empty() {
            return Ok(GasTrend::Stable);
        }

        let recent_avg = recent_prices.iter().fold(U256::zero(), |acc, &price| acc + price) 
            / U256::from(recent_prices.len());
        let older_avg = older_prices.iter().fold(U256::zero(), |acc, &price| acc + price) 
            / U256::from(older_prices.len());

        // Calculate percentage change
        if older_avg.is_zero() {
            return Ok(GasTrend::Stable);
        }

        let change_ratio = if recent_avg > older_avg {
            (recent_avg - older_avg).as_u128() as f64 / older_avg.as_u128() as f64
        } else {
            -((older_avg - recent_avg).as_u128() as f64 / older_avg.as_u128() as f64)
        };

        if change_ratio > 0.1 {
            Ok(GasTrend::Rising)
        } else if change_ratio < -0.1 {
            Ok(GasTrend::Falling)
        } else {
            Ok(GasTrend::Stable)
        }
    }

    /// Get recommended gas price for transaction urgency
    pub async fn get_recommended_gas_price(&self, urgency: TransactionUrgency) -> BlockchainResult<U256> {
        let base_price = self.get_optimal_gas_price().await?;
        
        let multiplier = match urgency {
            TransactionUrgency::Low => 0.9,
            TransactionUrgency::Standard => 1.0,
            TransactionUrgency::High => 1.3,
            TransactionUrgency::Urgent => 1.6,
        };

        Ok(self.calculate_price_with_multiplier(base_price, multiplier))
    }
}

#[derive(Debug, Clone)]
pub struct GasPriorityLevels {
    pub slow: U256,
    pub standard: U256,
    pub fast: U256,
    pub instant: U256,
}

#[derive(Debug, Clone)]
pub struct GasStatistics {
    pub current_price: U256,
    pub min_price: U256,
    pub max_price: U256,
    pub avg_price: U256,
    pub median_price: U256,
    pub sample_count: usize,
    pub last_update: u64,
}

impl Default for GasStatistics {
    fn default() -> Self {
        Self {
            current_price: U256::zero(),
            min_price: U256::zero(),
            max_price: U256::zero(),
            avg_price: U256::zero(),
            median_price: U256::zero(),
            sample_count: 0,
            last_update: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GasTrend {
    Rising,
    Falling,
    Stable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransactionUrgency {
    Low,      // Can wait for lower gas prices
    Standard, // Normal priority
    High,     // Needs faster confirmation
    Urgent,   // Needs immediate confirmation
}