use crate::blockchain::types::*;
use crate::blockchain::client::BlockchainClient;
use crate::blockchain::contract::RaffleContract;
use ethers::prelude::*;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tokio_stream::StreamExt;
use tracing::{debug, error, info, warn};

/// Event processor handles real-time blockchain event monitoring and processing
#[derive(Clone)]
pub struct EventProcessor {
    client: Arc<BlockchainClient>,
    db_pool: PgPool,
    contract: RaffleContract<Provider<Ws>>,
    event_sender: broadcast::Sender<ProcessedEvent>,
    last_processed_block: Arc<RwLock<u64>>,
    config: EventProcessorConfig,
}

#[derive(Debug, Clone)]
pub struct EventProcessorConfig {
    pub batch_size: usize,
    pub confirmation_blocks: u64,
    pub retry_attempts: u32,
    pub retry_delay_ms: u64,
    pub max_blocks_per_query: u64,
    pub health_check_interval: u64,
}

impl Default for EventProcessorConfig {
    fn default() -> Self {
        Self {
            batch_size: 100,
            confirmation_blocks: 12,
            retry_attempts: 3,
            retry_delay_ms: 1000,
            max_blocks_per_query: 1000,
            health_check_interval: 30,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ProcessedEvent {
    RaffleCreated(RaffleCreatedEvent),
    BoxPurchased(BoxPurchaseEvent),
    WinnerSelected(WinnerSelectedEvent),
    RaffleCancelled(RaffleCancelledEvent),
    RaffleFull(RaffleFullEvent),
    RandomnessRequested(RandomnessRequestedEvent),
}

#[derive(Debug, Clone)]
pub struct RaffleCancelledEvent {
    pub raffle_id: U256,
    pub reason: String,
    pub block_number: u64,
    pub transaction_hash: H256,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct RaffleFullEvent {
    pub raffle_id: U256,
    pub total_boxes: U256,
    pub block_number: u64,
    pub transaction_hash: H256,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct RandomnessRequestedEvent {
    pub raffle_id: U256,
    pub request_id: U256,
    pub block_number: u64,
    pub transaction_hash: H256,
    pub timestamp: u64,
}

impl EventProcessor {
    /// Create a new event processor
    pub async fn new(
        client: Arc<BlockchainClient>,
        db_pool: PgPool,
        config: Option<EventProcessorConfig>,
    ) -> BlockchainResult<Self> {
        let config = config.unwrap_or_default();
        
        // Get WebSocket provider for real-time events
        let ws_provider = client
            .ws_provider()
            .ok_or_else(|| BlockchainError::Configuration("WebSocket provider not available".to_string()))?;

        // Create contract instance with WebSocket provider
        let contract = RaffleContract::new(client.network_config().contract_address, ws_provider);

        // Create event broadcast channel
        let (event_sender, _) = broadcast::channel(1000);

        // Get last processed block from database
        let last_processed_block = Arc::new(RwLock::new(
            Self::get_last_processed_block(&db_pool).await.unwrap_or(0)
        ));

        Ok(Self {
            client,
            db_pool,
            contract,
            event_sender,
            last_processed_block,
            config,
        })
    }

    /// Start event monitoring
    pub async fn start_monitoring(&self) -> BlockchainResult<()> {
        info!("Starting blockchain event monitoring");

        // Start real-time event monitoring
        let processor = self.clone();
        tokio::spawn(async move {
            if let Err(e) = processor.monitor_real_time_events().await {
                error!("Real-time event monitoring failed: {}", e);
            }
        });

        // Start historical event processing
        let processor = self.clone();
        tokio::spawn(async move {
            if let Err(e) = processor.process_historical_events().await {
                error!("Historical event processing failed: {}", e);
            }
        });

        // Start health check task
        let processor = self.clone();
        tokio::spawn(async move {
            processor.health_check_loop().await;
        });

        info!("Event monitoring started successfully");
        Ok(())
    }

    /// Monitor real-time events using WebSocket
    async fn monitor_real_time_events(&self) -> BlockchainResult<()> {
        info!("Starting real-time event monitoring");

        // Create event stream for all contract events
        let events = self.contract.events().from_block(BlockNumber::Latest);
        let mut stream = events.stream().await
            .map_err(|e| BlockchainError::EventProcessing(format!("Failed to create event stream: {}", e)))?;

        while let Some(event) = stream.next().await {
            match event {
                Ok(log) => {
                    if let Err(e) = self.process_event_log(log).await {
                        error!("Failed to process event: {}", e);
                    }
                }
                Err(e) => {
                    warn!("Event stream error: {}", e);
                    // Try to reconnect after a delay
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    break;
                }
            }
        }

        warn!("Real-time event monitoring stopped, attempting to restart");
        // Recursive call to restart monitoring
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
        self.monitor_real_time_events().await
    }

    /// Process historical events to catch up
    async fn process_historical_events(&self) -> BlockchainResult<()> {
        info!("Processing historical events");

        let current_block = self.client.get_block_number().await?;
        let last_processed = *self.last_processed_block.read().await;
        
        if current_block <= last_processed {
            debug!("No new blocks to process");
            return Ok(());
        }

        let mut from_block = last_processed + 1;
        
        while from_block <= current_block {
            let to_block = std::cmp::min(
                from_block + self.config.max_blocks_per_query - 1,
                current_block
            );

            debug!("Processing blocks {} to {}", from_block, to_block);

            // Get events for this block range
            let filter = Filter::new()
                .address(self.client.network_config().contract_address)
                .from_block(from_block)
                .to_block(to_block);

            match self.client.provider().get_logs(&filter).await {
                Ok(logs) => {
                    for log in logs {
                        if let Err(e) = self.process_raw_log(log).await {
                            error!("Failed to process historical log: {}", e);
                        }
                    }
                    
                    // Update last processed block
                    {
                        let mut last_block = self.last_processed_block.write().await;
                        *last_block = to_block;
                    }
                    
                    // Save to database
                    if let Err(e) = self.save_last_processed_block(to_block).await {
                        error!("Failed to save last processed block: {}", e);
                    }
                }
                Err(e) => {
                    error!("Failed to get logs for blocks {} to {}: {}", from_block, to_block, e);
                    // Wait before retrying
                    tokio::time::sleep(tokio::time::Duration::from_millis(self.config.retry_delay_ms)).await;
                    continue;
                }
            }

            from_block = to_block + 1;
            
            // Small delay to avoid overwhelming the RPC
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }

        info!("Historical event processing completed");
        Ok(())
    }

    /// Process a contract event log
    async fn process_event_log(&self, log: RaffleContractEvents) -> BlockchainResult<()> {
        let processed_event = match log {
            RaffleContractEvents::RaffleCreatedFilter(event) => {
                let processed = self.handle_raffle_created(event).await?;
                ProcessedEvent::RaffleCreated(processed)
            }
            RaffleContractEvents::BoxPurchasedFilter(event) => {
                let processed = self.handle_box_purchased(event).await?;
                ProcessedEvent::BoxPurchased(processed)
            }
            RaffleContractEvents::WinnerSelectedFilter(event) => {
                let processed = self.handle_winner_selected(event).await?;
                ProcessedEvent::WinnerSelected(processed)
            }
            RaffleContractEvents::RaffleCancelledFilter(event) => {
                let processed = self.handle_raffle_cancelled(event).await?;
                ProcessedEvent::RaffleCancelled(processed)
            }
            RaffleContractEvents::RaffleFullFilter(event) => {
                let processed = self.handle_raffle_full(event).await?;
                ProcessedEvent::RaffleFull(processed)
            }
            RaffleContractEvents::RandomnessRequestedFilter(event) => {
                let processed = self.handle_randomness_requested(event).await?;
                ProcessedEvent::RandomnessRequested(processed)
            }
        };

        // Broadcast the processed event
        if let Err(e) = self.event_sender.send(processed_event) {
            warn!("Failed to broadcast event: {}", e);
        }

        Ok(())
    }

    /// Process a raw log (for historical events)
    async fn process_raw_log(&self, log: Log) -> BlockchainResult<()> {
        // Parse the log using the contract interface
        if let Ok(parsed) = self.contract.decode_event::<RaffleContractEvents>("RaffleCreated", log.topics.clone(), log.data.clone()) {
            self.process_event_log(parsed).await?;
        } else if let Ok(parsed) = self.contract.decode_event::<RaffleContractEvents>("BoxPurchased", log.topics.clone(), log.data.clone()) {
            self.process_event_log(parsed).await?;
        } else if let Ok(parsed) = self.contract.decode_event::<RaffleContractEvents>("WinnerSelected", log.topics.clone(), log.data.clone()) {
            self.process_event_log(parsed).await?;
        } else if let Ok(parsed) = self.contract.decode_event::<RaffleContractEvents>("RaffleCancelled", log.topics.clone(), log.data.clone()) {
            self.process_event_log(parsed).await?;
        } else if let Ok(parsed) = self.contract.decode_event::<RaffleContractEvents>("RaffleFull", log.topics.clone(), log.data.clone()) {
            self.process_event_log(parsed).await?;
        } else if let Ok(parsed) = self.contract.decode_event::<RaffleContractEvents>("RandomnessRequested", log.topics.clone(), log.data.clone()) {
            self.process_event_log(parsed).await?;
        } else {
            debug!("Unknown event log: {:?}", log);
        }

        Ok(())
    }

    /// Handle RaffleCreated event
    async fn handle_raffle_created(&self, event: RaffleCreatedFilter) -> BlockchainResult<RaffleCreatedEvent> {
        let block_info = self.get_block_info(event.meta.block_number).await?;
        
        let processed_event = RaffleCreatedEvent {
            raffle_id: event.raffle_id,
            item_id: event.item_id,
            total_boxes: event.total_boxes,
            box_price: event.box_price,
            total_winners: event.total_winners,
            creator: event.creator,
            block_number: block_info.number,
            transaction_hash: event.meta.transaction_hash,
            timestamp: block_info.timestamp,
        };

        // Update database
        if let Err(e) = self.save_raffle_created_event(&processed_event).await {
            error!("Failed to save RaffleCreated event: {}", e);
        }

        info!(
            "Processed RaffleCreated event: raffle_id={}, item_id={}, total_boxes={}",
            processed_event.raffle_id, processed_event.item_id, processed_event.total_boxes
        );

        Ok(processed_event)
    }

    /// Handle BoxPurchased event
    async fn handle_box_purchased(&self, event: BoxPurchasedFilter) -> BlockchainResult<BoxPurchaseEvent> {
        let block_info = self.get_block_info(event.meta.block_number).await?;
        
        let processed_event = BoxPurchaseEvent {
            raffle_id: event.raffle_id,
            buyer: event.buyer,
            box_number: event.box_number,
            total_boxes_sold: event.total_boxes_sold,
            block_number: block_info.number,
            transaction_hash: event.meta.transaction_hash,
            timestamp: block_info.timestamp,
        };

        // Update database
        if let Err(e) = self.save_box_purchased_event(&processed_event).await {
            error!("Failed to save BoxPurchased event: {}", e);
        }

        info!(
            "Processed BoxPurchased event: raffle_id={}, buyer={:?}, box_number={}",
            processed_event.raffle_id, processed_event.buyer, processed_event.box_number
        );

        Ok(processed_event)
    }

    /// Handle WinnerSelected event
    async fn handle_winner_selected(&self, event: WinnerSelectedFilter) -> BlockchainResult<WinnerSelectedEvent> {
        let block_info = self.get_block_info(event.meta.block_number).await?;
        
        let processed_event = WinnerSelectedEvent {
            raffle_id: event.raffle_id,
            winners: event.winners,
            random_word: event.random_word,
            block_number: block_info.number,
            transaction_hash: event.meta.transaction_hash,
            timestamp: block_info.timestamp,
        };

        // Update database
        if let Err(e) = self.save_winner_selected_event(&processed_event).await {
            error!("Failed to save WinnerSelected event: {}", e);
        }

        info!(
            "Processed WinnerSelected event: raffle_id={}, winners={:?}",
            processed_event.raffle_id, processed_event.winners
        );

        Ok(processed_event)
    }

    /// Handle RaffleCancelled event
    async fn handle_raffle_cancelled(&self, event: RaffleCancelledFilter) -> BlockchainResult<RaffleCancelledEvent> {
        let block_info = self.get_block_info(event.meta.block_number).await?;
        
        let processed_event = RaffleCancelledEvent {
            raffle_id: event.raffle_id,
            reason: event.reason,
            block_number: block_info.number,
            transaction_hash: event.meta.transaction_hash,
            timestamp: block_info.timestamp,
        };

        // Update database
        if let Err(e) = self.save_raffle_cancelled_event(&processed_event).await {
            error!("Failed to save RaffleCancelled event: {}", e);
        }

        info!(
            "Processed RaffleCancelled event: raffle_id={}, reason={}",
            processed_event.raffle_id, processed_event.reason
        );

        Ok(processed_event)
    }

    /// Handle RaffleFull event
    async fn handle_raffle_full(&self, event: RaffleFullFilter) -> BlockchainResult<RaffleFullEvent> {
        let block_info = self.get_block_info(event.meta.block_number).await?;
        
        let processed_event = RaffleFullEvent {
            raffle_id: event.raffle_id,
            total_boxes: event.total_boxes,
            block_number: block_info.number,
            transaction_hash: event.meta.transaction_hash,
            timestamp: block_info.timestamp,
        };

        // Update database
        if let Err(e) = self.save_raffle_full_event(&processed_event).await {
            error!("Failed to save RaffleFull event: {}", e);
        }

        info!(
            "Processed RaffleFull event: raffle_id={}, total_boxes={}",
            processed_event.raffle_id, processed_event.total_boxes
        );

        Ok(processed_event)
    }

    /// Handle RandomnessRequested event
    async fn handle_randomness_requested(&self, event: RandomnessRequestedFilter) -> BlockchainResult<RandomnessRequestedEvent> {
        let block_info = self.get_block_info(event.meta.block_number).await?;
        
        let processed_event = RandomnessRequestedEvent {
            raffle_id: event.raffle_id,
            request_id: event.request_id,
            block_number: block_info.number,
            transaction_hash: event.meta.transaction_hash,
            timestamp: block_info.timestamp,
        };

        // Update database
        if let Err(e) = self.save_randomness_requested_event(&processed_event).await {
            error!("Failed to save RandomnessRequested event: {}", e);
        }

        info!(
            "Processed RandomnessRequested event: raffle_id={}, request_id={}",
            processed_event.raffle_id, processed_event.request_id
        );

        Ok(processed_event)
    }

    /// Get block information
    async fn get_block_info(&self, block_number: Option<U64>) -> BlockchainResult<BlockInfo> {
        let block_num = block_number.unwrap_or_else(|| U64::zero());
        
        match self.client.provider().get_block(block_num).await? {
            Some(block) => Ok(BlockInfo {
                number: block.number.unwrap_or_default().as_u64(),
                timestamp: block.timestamp.as_u64(),
                hash: block.hash.unwrap_or_default(),
            }),
            None => Err(BlockchainError::EventProcessing(
                format!("Block {} not found", block_num)
            )),
        }
    }

    /// Subscribe to processed events
    pub fn subscribe_to_events(&self) -> broadcast::Receiver<ProcessedEvent> {
        self.event_sender.subscribe()
    }

    /// Health check loop
    async fn health_check_loop(&self) {
        let mut interval = tokio::time::interval(
            tokio::time::Duration::from_secs(self.config.health_check_interval)
        );

        loop {
            interval.tick().await;
            
            match self.client.check_network_health().await {
                Ok(health) => {
                    if !health.is_connected {
                        warn!("Blockchain network health check failed");
                    } else {
                        debug!("Blockchain network health check passed");
                    }
                }
                Err(e) => {
                    error!("Network health check error: {}", e);
                }
            }
        }
    }

    // Database operations
    async fn get_last_processed_block(db_pool: &PgPool) -> Result<u64, sqlx::Error> {
        let row = sqlx::query!(
            "SELECT last_processed_block FROM event_processor_state WHERE id = 1"
        )
        .fetch_optional(db_pool)
        .await?;

        Ok(row.map(|r| r.last_processed_block as u64).unwrap_or(0))
    }

    async fn save_last_processed_block(&self, block_number: u64) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO event_processor_state (id, last_processed_block) VALUES (1, $1)
             ON CONFLICT (id) DO UPDATE SET last_processed_block = $1",
            block_number as i64
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    async fn save_raffle_created_event(&self, event: &RaffleCreatedEvent) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO blockchain_events (event_type, raffle_id, block_number, transaction_hash, timestamp, data)
             VALUES ('raffle_created', $1, $2, $3, $4, $5)
             ON CONFLICT (transaction_hash, event_type, raffle_id) DO NOTHING",
            event.raffle_id.as_u64() as i64,
            event.block_number as i64,
            format!("{:?}", event.transaction_hash),
            chrono::DateTime::from_timestamp(event.timestamp as i64, 0),
            serde_json::to_value(event).unwrap()
        )
        .execute(&self.db_pool)
        .await?;

        // Update raffle status in database
        sqlx::query!(
            "UPDATE raffles SET status = 'open', created_at = $1 WHERE blockchain_raffle_id = $2",
            chrono::DateTime::from_timestamp(event.timestamp as i64, 0),
            event.raffle_id.as_u64() as i64
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    async fn save_box_purchased_event(&self, event: &BoxPurchaseEvent) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO blockchain_events (event_type, raffle_id, block_number, transaction_hash, timestamp, data)
             VALUES ('box_purchased', $1, $2, $3, $4, $5)
             ON CONFLICT (transaction_hash, event_type, raffle_id) DO NOTHING",
            event.raffle_id.as_u64() as i64,
            event.block_number as i64,
            format!("{:?}", event.transaction_hash),
            chrono::DateTime::from_timestamp(event.timestamp as i64, 0),
            serde_json::to_value(event).unwrap()
        )
        .execute(&self.db_pool)
        .await?;

        // Update raffle boxes sold
        sqlx::query!(
            "UPDATE raffles SET boxes_sold = $1 WHERE blockchain_raffle_id = $2",
            event.total_boxes_sold.as_u64() as i32,
            event.raffle_id.as_u64() as i64
        )
        .execute(&self.db_pool)
        .await?;

        // Record box purchase
        sqlx::query!(
            "INSERT INTO box_purchases (raffle_id, buyer_address, box_number, transaction_hash, purchased_at)
             VALUES ((SELECT id FROM raffles WHERE blockchain_raffle_id = $1), $2, $3, $4, $5)
             ON CONFLICT (raffle_id, box_number) DO NOTHING",
            event.raffle_id.as_u64() as i64,
            format!("{:?}", event.buyer),
            event.box_number.as_u64() as i32,
            format!("{:?}", event.transaction_hash),
            chrono::DateTime::from_timestamp(event.timestamp as i64, 0)
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    async fn save_winner_selected_event(&self, event: &WinnerSelectedEvent) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO blockchain_events (event_type, raffle_id, block_number, transaction_hash, timestamp, data)
             VALUES ('winner_selected', $1, $2, $3, $4, $5)
             ON CONFLICT (transaction_hash, event_type, raffle_id) DO NOTHING",
            event.raffle_id.as_u64() as i64,
            event.block_number as i64,
            format!("{:?}", event.transaction_hash),
            chrono::DateTime::from_timestamp(event.timestamp as i64, 0),
            serde_json::to_value(event).unwrap()
        )
        .execute(&self.db_pool)
        .await?;

        // Update raffle status to completed
        sqlx::query!(
            "UPDATE raffles SET status = 'completed', completed_at = $1 WHERE blockchain_raffle_id = $2",
            chrono::DateTime::from_timestamp(event.timestamp as i64, 0),
            event.raffle_id.as_u64() as i64
        )
        .execute(&self.db_pool)
        .await?;

        // Record winners
        for (index, winner) in event.winners.iter().enumerate() {
            sqlx::query!(
                "INSERT INTO raffle_winners (raffle_id, winner_address, winner_index, selected_at)
                 VALUES ((SELECT id FROM raffles WHERE blockchain_raffle_id = $1), $2, $3, $4)
                 ON CONFLICT (raffle_id, winner_index) DO NOTHING",
                event.raffle_id.as_u64() as i64,
                format!("{:?}", winner),
                index as i32,
                chrono::DateTime::from_timestamp(event.timestamp as i64, 0)
            )
            .execute(&self.db_pool)
            .await?;
        }

        Ok(())
    }

    async fn save_raffle_cancelled_event(&self, event: &RaffleCancelledEvent) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO blockchain_events (event_type, raffle_id, block_number, transaction_hash, timestamp, data)
             VALUES ('raffle_cancelled', $1, $2, $3, $4, $5)
             ON CONFLICT (transaction_hash, event_type, raffle_id) DO NOTHING",
            event.raffle_id.as_u64() as i64,
            event.block_number as i64,
            format!("{:?}", event.transaction_hash),
            chrono::DateTime::from_timestamp(event.timestamp as i64, 0),
            serde_json::to_value(event).unwrap()
        )
        .execute(&self.db_pool)
        .await?;

        // Update raffle status to cancelled
        sqlx::query!(
            "UPDATE raffles SET status = 'cancelled' WHERE blockchain_raffle_id = $1",
            event.raffle_id.as_u64() as i64
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    async fn save_raffle_full_event(&self, event: &RaffleFullEvent) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO blockchain_events (event_type, raffle_id, block_number, transaction_hash, timestamp, data)
             VALUES ('raffle_full', $1, $2, $3, $4, $5)
             ON CONFLICT (transaction_hash, event_type, raffle_id) DO NOTHING",
            event.raffle_id.as_u64() as i64,
            event.block_number as i64,
            format!("{:?}", event.transaction_hash),
            chrono::DateTime::from_timestamp(event.timestamp as i64, 0),
            serde_json::to_value(event).unwrap()
        )
        .execute(&self.db_pool)
        .await?;

        // Update raffle status to full
        sqlx::query!(
            "UPDATE raffles SET status = 'full' WHERE blockchain_raffle_id = $1",
            event.raffle_id.as_u64() as i64
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    async fn save_randomness_requested_event(&self, event: &RandomnessRequestedEvent) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO blockchain_events (event_type, raffle_id, block_number, transaction_hash, timestamp, data)
             VALUES ('randomness_requested', $1, $2, $3, $4, $5)
             ON CONFLICT (transaction_hash, event_type, raffle_id) DO NOTHING",
            event.raffle_id.as_u64() as i64,
            event.block_number as i64,
            format!("{:?}", event.transaction_hash),
            chrono::DateTime::from_timestamp(event.timestamp as i64, 0),
            serde_json::to_value(event).unwrap()
        )
        .execute(&self.db_pool)
        .await?;

        // Update raffle status to random_requested
        sqlx::query!(
            "UPDATE raffles SET status = 'random_requested' WHERE blockchain_raffle_id = $1",
            event.raffle_id.as_u64() as i64
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
struct BlockInfo {
    number: u64,
    timestamp: u64,
    hash: H256,
}

// Event filter types for the contract
use ethers::contract::EthEvent;

#[derive(Debug, Clone, EthEvent)]
#[ethevent(name = "RaffleCreated")]
pub struct RaffleCreatedFilter {
    #[ethevent(indexed)]
    pub raffle_id: U256,
    #[ethevent(indexed)]
    pub item_id: U256,
    pub total_boxes: U256,
    pub box_price: U256,
    pub total_winners: U256,
    #[ethevent(indexed)]
    pub creator: Address,
}

#[derive(Debug, Clone, EthEvent)]
#[ethevent(name = "BoxPurchased")]
pub struct BoxPurchasedFilter {
    #[ethevent(indexed)]
    pub raffle_id: U256,
    #[ethevent(indexed)]
    pub buyer: Address,
    pub box_number: U256,
    pub total_boxes_sold: U256,
}

#[derive(Debug, Clone, EthEvent)]
#[ethevent(name = "WinnerSelected")]
pub struct WinnerSelectedFilter {
    #[ethevent(indexed)]
    pub raffle_id: U256,
    pub winners: Vec<Address>,
    pub random_word: U256,
}

#[derive(Debug, Clone, EthEvent)]
#[ethevent(name = "RaffleCancelled")]
pub struct RaffleCancelledFilter {
    #[ethevent(indexed)]
    pub raffle_id: U256,
    pub reason: String,
}

#[derive(Debug, Clone, EthEvent)]
#[ethevent(name = "RaffleFull")]
pub struct RaffleFullFilter {
    #[ethevent(indexed)]
    pub raffle_id: U256,
    pub total_boxes: U256,
}

#[derive(Debug, Clone, EthEvent)]
#[ethevent(name = "RandomnessRequested")]
pub struct RandomnessRequestedFilter {
    #[ethevent(indexed)]
    pub raffle_id: U256,
    #[ethevent(indexed)]
    pub request_id: U256,
}