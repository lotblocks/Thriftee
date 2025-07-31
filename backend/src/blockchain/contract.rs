use crate::blockchain::types::*;
use crate::blockchain::client::BlockchainClient;
use crate::blockchain::transaction::TransactionManager;
use crate::blockchain::gas::GasManager;
use ethers::prelude::*;
use std::sync::Arc;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

// Generated contract bindings would go here
// For now, we'll define the contract interface manually
abigen!(
    RaffleContract,
    r#"[
        function createRaffle(string title, string description, uint256 totalBoxes, uint256 pricePerBox, uint256 startTime, uint256 endTime, string itemImageUrl, string itemDescription, uint256 maxParticipantsPerUser, uint256 minimumParticipants, bool requiresWhitelist, bytes32 whitelistMerkleRoot, uint256 creatorFeePercentage, address paymentToken) external returns (uint256)
        function purchaseParticipation(uint256 raffleId, uint256 boxesToPurchase, bytes32[] merkleProof) external payable
        function requestRandomWinner(uint256 raffleId) external
        function raffles(uint256 raffleId) external view returns (tuple(uint256 id, address creator, string title, string description, uint256 totalBoxes, uint256 pricePerBox, uint256 startTime, uint256 endTime, uint8 status, address winner, uint256 totalParticipants, uint256 totalRevenue, string itemImageUrl, string itemDescription, uint256 createdAt, uint256 maxParticipantsPerUser, uint256 minimumParticipants, bool requiresWhitelist, bytes32 whitelistMerkleRoot, uint256 creatorFeePercentage, uint256 platformFeePercentage, address paymentToken, uint256 randomSeed, bool isVerified))
        function getRaffleParticipants(uint256 raffleId) external view returns (address[])
        function getParticipantBoxes(uint256 raffleId, address participant) external view returns (uint256)
        function isWhitelisted(uint256 raffleId, address user, bytes32[] proof) external view returns (bool)
        function canCompleteRaffle(uint256 raffleId) external view returns (bool)
        function getActiveRaffles() external view returns (uint256[])
        function getRaffleStats(uint256 raffleId) external view returns (uint256 totalParticipants, uint256 totalRevenue, uint256 boxesRemaining, bool isCompleted, address winner)
        function claimRefund(uint256 raffleId) external
        function verifyRaffle(uint256 raffleId) external
        function updateWhitelist(uint256 raffleId, bytes32 newMerkleRoot) external
        function pause() external
        function unpause() external
        function paused() external view returns (bool)
        function hasRole(bytes32 role, address account) external view returns (bool)
        function grantRole(bytes32 role, address account) external
        function revokeRole(bytes32 role, address account) external
        function DEFAULT_ADMIN_ROLE() external view returns (bytes32)
        function RAFFLE_MANAGER_ROLE() external view returns (bytes32)
        function OPERATOR_ROLE() external view returns (bytes32)
        function PAUSER_ROLE() external view returns (bytes32)
        event RaffleCreated(uint256 indexed raffleId, address indexed creator, string title, uint256 totalBoxes, uint256 pricePerBox, uint256 startTime, uint256 endTime, address paymentToken)
        event ParticipationPurchased(uint256 indexed raffleId, address indexed participant, uint256 indexed participationId, uint256 boxesPurchased, uint256 totalCost)
        event RaffleCompleted(uint256 indexed raffleId, address indexed winner, uint256 totalRevenue, uint256 totalParticipants, uint256 randomSeed)
        event RaffleCancelled(uint256 indexed raffleId, string reason)
        event RefundIssued(uint256 indexed raffleId, address indexed participant, uint256 amount)
        event RandomnessRequested(uint256 indexed raffleId, uint256 indexed requestId)
        event RandomnessFulfilled(uint256 indexed raffleId, uint256 randomSeed)
        event RaffleVerified(uint256 indexed raffleId, address indexed verifier)
        event WhitelistUpdated(uint256 indexed raffleId, bytes32 merkleRoot)
        event FeesUpdated(uint256 platformFee, uint256 creatorFee)
        event EmergencyWithdraw(address indexed token, uint256 amount, address indexed recipient)
    ]"#
);

/// High-level client for interacting with the RaffleContract
#[derive(Clone)]
pub struct RaffleContractClient {
    contract: RaffleContract<SignerMiddleware<Provider<Http>, LocalWallet>>,
    client: Arc<BlockchainClient>,
    transaction_manager: Arc<TransactionManager>,
    gas_manager: Arc<GasManager>,
}

impl RaffleContractClient {
    /// Create a new raffle contract client
    pub async fn new(
        client: Arc<BlockchainClient>,
        transaction_manager: Arc<TransactionManager>,
        gas_manager: Arc<GasManager>,
    ) -> BlockchainResult<Self> {
        let middleware = client.create_signed_middleware();
        let contract = RaffleContract::new(client.network_config().contract_address, middleware);

        Ok(Self {
            contract,
            client,
            transaction_manager,
            gas_manager,
        })
    }

    /// Create a new raffle
    pub async fn create_raffle(
        &self,
        params: CreateRaffleParams,
    ) -> BlockchainResult<Uuid> {
        info!(
            "Creating raffle '{}' with {} boxes at {} wei each",
            params.title, params.total_boxes, params.price_per_box
        );

        // Prepare transaction
        let call = self.contract.create_raffle(
            params.title,
            params.description,
            U256::from(params.total_boxes),
            params.price_per_box,
            U256::from(params.start_time),
            U256::from(params.end_time),
            params.item_image_url,
            params.item_description,
            U256::from(params.max_participants_per_user),
            U256::from(params.minimum_participants),
            params.requires_whitelist,
            params.whitelist_merkle_root,
            U256::from(params.creator_fee_percentage),
            params.payment_token,
        );

        let tx_request = call.tx;
        
        // Get gas estimate
        let gas_estimate = self.gas_manager.estimate_gas_with_buffer(&tx_request, Some(20)).await?;
        
        // Configure transaction
        let tx_config = TransactionConfig {
            gas_limit: gas_estimate.gas_limit,
            gas_price: Some(gas_estimate.gas_price),
            max_fee_per_gas: gas_estimate.max_fee_per_gas,
            max_priority_fee_per_gas: gas_estimate.max_priority_fee_per_gas,
            confirmations: self.client.network_config().confirmations,
            ..Default::default()
        };

        // Submit transaction
        let tx_id = self.transaction_manager.submit_transaction(tx_request, Some(tx_config)).await?;

        debug!("Raffle creation transaction submitted with ID: {}", tx_id);
        Ok(tx_id)
    }

    /// Purchase participation in a raffle
    pub async fn purchase_participation(
        &self,
        raffle_id: u64,
        boxes_to_purchase: u64,
        merkle_proof: Vec<[u8; 32]>,
        buyer_address: Address,
        payment_amount: U256,
    ) -> BlockchainResult<Uuid> {
        info!(
            "Purchasing {} boxes in raffle {} for address {:?} with payment {}",
            boxes_to_purchase, raffle_id, buyer_address, payment_amount
        );

        // Prepare transaction
        let call = self.contract.purchase_participation(
            U256::from(raffle_id),
            U256::from(boxes_to_purchase),
            merkle_proof,
        );
        let mut tx_request = call.tx;
        tx_request.value = Some(payment_amount);
        tx_request.from = Some(buyer_address);

        // Get gas estimate
        let gas_estimate = self.gas_manager.estimate_gas_with_buffer(&tx_request, Some(15)).await?;
        
        // Configure transaction
        let tx_config = TransactionConfig {
            gas_limit: gas_estimate.gas_limit,
            gas_price: Some(gas_estimate.gas_price),
            max_fee_per_gas: gas_estimate.max_fee_per_gas,
            max_priority_fee_per_gas: gas_estimate.max_priority_fee_per_gas,
            confirmations: self.client.network_config().confirmations,
            ..Default::default()
        };

        // Submit transaction
        let tx_id = self.transaction_manager.submit_transaction(tx_request, Some(tx_config)).await?;

        debug!("Box purchase transaction submitted with ID: {}", tx_id);
        Ok(tx_id)
    }

    /// Get raffle details
    pub async fn get_raffle(&self, raffle_id: u64) -> BlockchainResult<RaffleData> {
        debug!("Getting raffle details for raffle {}", raffle_id);

        let raffle_tuple = self.contract.raffles(U256::from(raffle_id)).call().await
            .map_err(|e| BlockchainError::Contract(e))?;

        // Convert tuple to RaffleData struct
        let raffle_data = RaffleData {
            id: raffle_tuple.0,
            creator: raffle_tuple.1,
            title: raffle_tuple.2,
            description: raffle_tuple.3,
            total_boxes: raffle_tuple.4,
            price_per_box: raffle_tuple.5,
            start_time: raffle_tuple.6,
            end_time: raffle_tuple.7,
            status: RaffleStatus::from(raffle_tuple.8),
            winner: raffle_tuple.9,
            total_participants: raffle_tuple.10,
            total_revenue: raffle_tuple.11,
            item_image_url: raffle_tuple.12,
            item_description: raffle_tuple.13,
            created_at: raffle_tuple.14,
            max_participants_per_user: raffle_tuple.15,
            minimum_participants: raffle_tuple.16,
            requires_whitelist: raffle_tuple.17,
            whitelist_merkle_root: raffle_tuple.18,
            creator_fee_percentage: raffle_tuple.19,
            platform_fee_percentage: raffle_tuple.20,
            payment_token: raffle_tuple.21,
            random_seed: raffle_tuple.22,
            is_verified: raffle_tuple.23,
        };

        Ok(raffle_data)
    }

    /// Get raffle participants
    pub async fn get_raffle_participants(&self, raffle_id: u64) -> BlockchainResult<Vec<Address>> {
        debug!("Getting participants for raffle {}", raffle_id);

        let participants = self.contract.get_raffle_participants(U256::from(raffle_id)).call().await
            .map_err(|e| BlockchainError::Contract(e))?;

        Ok(participants)
    }

    /// Get participant box count
    pub async fn get_participant_boxes(&self, raffle_id: u64, participant: Address) -> BlockchainResult<u64> {
        debug!("Getting box count for participant {:?} in raffle {}", participant, raffle_id);

        let box_count = self.contract.get_participant_boxes(U256::from(raffle_id), participant).call().await
            .map_err(|e| BlockchainError::Contract(e))?;

        Ok(box_count.as_u64())
    }

    /// Check if user is whitelisted
    pub async fn is_whitelisted(
        &self,
        raffle_id: u64,
        user: Address,
        proof: Vec<[u8; 32]>,
    ) -> BlockchainResult<bool> {
        let is_whitelisted = self.contract.is_whitelisted(U256::from(raffle_id), user, proof).call().await
            .map_err(|e| BlockchainError::Contract(e))?;

        Ok(is_whitelisted)
    }

    /// Check if raffle can be completed
    pub async fn can_complete_raffle(&self, raffle_id: u64) -> BlockchainResult<bool> {
        let can_complete = self.contract.can_complete_raffle(U256::from(raffle_id)).call().await
            .map_err(|e| BlockchainError::Contract(e))?;

        Ok(can_complete)
    }

    /// Get active raffles
    pub async fn get_active_raffles(&self) -> BlockchainResult<Vec<u64>> {
        let active_raffle_ids = self.contract.get_active_raffles().call().await
            .map_err(|e| BlockchainError::Contract(e))?;

        Ok(active_raffle_ids.into_iter().map(|id| id.as_u64()).collect())
    }

    /// Get raffle statistics
    pub async fn get_raffle_stats(&self, raffle_id: u64) -> BlockchainResult<RaffleStats> {
        let stats_tuple = self.contract.get_raffle_stats(U256::from(raffle_id)).call().await
            .map_err(|e| BlockchainError::Contract(e))?;

        Ok(RaffleStats {
            total_participants: stats_tuple.0,
            total_revenue: stats_tuple.1,
            boxes_remaining: stats_tuple.2,
            is_completed: stats_tuple.3,
            winner: stats_tuple.4,
        })
    }

    /// Request random winner for a raffle
    pub async fn request_random_winner(&self, raffle_id: u64) -> BlockchainResult<Uuid> {
        info!("Requesting random winner for raffle {}", raffle_id);

        let call = self.contract.request_random_winner(U256::from(raffle_id));
        let tx_request = call.tx;

        let gas_estimate = self.gas_manager.estimate_gas_with_buffer(&tx_request, Some(25)).await?;
        
        let tx_config = TransactionConfig {
            gas_limit: gas_estimate.gas_limit,
            gas_price: Some(gas_estimate.gas_price),
            max_fee_per_gas: gas_estimate.max_fee_per_gas,
            max_priority_fee_per_gas: gas_estimate.max_priority_fee_per_gas,
            confirmations: self.client.network_config().confirmations,
            ..Default::default()
        };

        let tx_id = self.transaction_manager.submit_transaction(tx_request, Some(tx_config)).await?;
        debug!("Random winner request transaction submitted with ID: {}", tx_id);
        Ok(tx_id)
    }

    /// Claim refund for cancelled raffle
    pub async fn claim_refund(&self, raffle_id: u64, user_address: Address) -> BlockchainResult<Uuid> {
        info!("Claiming refund for raffle {} for user {:?}", raffle_id, user_address);

        let call = self.contract.claim_refund(U256::from(raffle_id));
        let mut tx_request = call.tx;
        tx_request.from = Some(user_address);

        let gas_estimate = self.gas_manager.estimate_gas_with_buffer(&tx_request, Some(20)).await?;
        
        let tx_config = TransactionConfig {
            gas_limit: gas_estimate.gas_limit,
            gas_price: Some(gas_estimate.gas_price),
            max_fee_per_gas: gas_estimate.max_fee_per_gas,
            max_priority_fee_per_gas: gas_estimate.max_priority_fee_per_gas,
            confirmations: self.client.network_config().confirmations,
            ..Default::default()
        };

        let tx_id = self.transaction_manager.submit_transaction(tx_request, Some(tx_config)).await?;
        Ok(tx_id)
    }

    /// Verify a raffle (operator only)
    pub async fn verify_raffle(&self, raffle_id: u64) -> BlockchainResult<Uuid> {
        info!("Verifying raffle {}", raffle_id);

        let call = self.contract.verify_raffle(U256::from(raffle_id));
        let tx_request = call.tx;

        let gas_estimate = self.gas_manager.estimate_gas_with_buffer(&tx_request, Some(20)).await?;
        
        let tx_config = TransactionConfig {
            gas_limit: gas_estimate.gas_limit,
            gas_price: Some(gas_estimate.gas_price),
            max_fee_per_gas: gas_estimate.max_fee_per_gas,
            max_priority_fee_per_gas: gas_estimate.max_priority_fee_per_gas,
            confirmations: self.client.network_config().confirmations,
            ..Default::default()
        };

        let tx_id = self.transaction_manager.submit_transaction(tx_request, Some(tx_config)).await?;
        Ok(tx_id)
    }

    /// Update whitelist for a raffle
    pub async fn update_whitelist(
        &self,
        raffle_id: u64,
        new_merkle_root: [u8; 32],
    ) -> BlockchainResult<Uuid> {
        info!("Updating whitelist for raffle {}", raffle_id);

        let call = self.contract.update_whitelist(U256::from(raffle_id), new_merkle_root);
        let tx_request = call.tx;

        let gas_estimate = self.gas_manager.estimate_gas_with_buffer(&tx_request, Some(20)).await?;
        
        let tx_config = TransactionConfig {
            gas_limit: gas_estimate.gas_limit,
            gas_price: Some(gas_estimate.gas_price),
            max_fee_per_gas: gas_estimate.max_fee_per_gas,
            max_priority_fee_per_gas: gas_estimate.max_priority_fee_per_gas,
            confirmations: self.client.network_config().confirmations,
            ..Default::default()
        };

        let tx_id = self.transaction_manager.submit_transaction(tx_request, Some(tx_config)).await?;
        Ok(tx_id)
    }

    /// Check if address has role
    pub async fn has_role(&self, role: [u8; 32], account: Address) -> BlockchainResult<bool> {
        let has_role = self.contract.has_role(role, account).call().await
            .map_err(|e| BlockchainError::Contract(e))?;

        Ok(has_role)
    }

    /// Grant role to account (admin only)
    pub async fn grant_role(&self, role: [u8; 32], account: Address) -> BlockchainResult<Uuid> {
        info!("Granting role to account: {:?}", account);

        let call = self.contract.grant_role(role, account);
        let tx_request = call.tx;

        let gas_estimate = self.gas_manager.estimate_gas_with_buffer(&tx_request, Some(20)).await?;
        
        let tx_config = TransactionConfig {
            gas_limit: gas_estimate.gas_limit,
            gas_price: Some(gas_estimate.gas_price),
            max_fee_per_gas: gas_estimate.max_fee_per_gas,
            max_priority_fee_per_gas: gas_estimate.max_priority_fee_per_gas,
            confirmations: self.client.network_config().confirmations,
            ..Default::default()
        };

        let tx_id = self.transaction_manager.submit_transaction(tx_request, Some(tx_config)).await?;
        Ok(tx_id)
    }

    /// Revoke role from account (admin only)
    pub async fn revoke_role(&self, role: [u8; 32], account: Address) -> BlockchainResult<Uuid> {
        info!("Revoking role from account: {:?}", account);

        let call = self.contract.revoke_role(role, account);
        let tx_request = call.tx;

        let gas_estimate = self.gas_manager.estimate_gas_with_buffer(&tx_request, Some(20)).await?;
        
        let tx_config = TransactionConfig {
            gas_limit: gas_estimate.gas_limit,
            gas_price: Some(gas_estimate.gas_price),
            max_fee_per_gas: gas_estimate.max_fee_per_gas,
            max_priority_fee_per_gas: gas_estimate.max_priority_fee_per_gas,
            confirmations: self.client.network_config().confirmations,
            ..Default::default()
        };

        let tx_id = self.transaction_manager.submit_transaction(tx_request, Some(tx_config)).await?;
        Ok(tx_id)
    }

    /// Get role constants
    pub async fn get_roles(&self) -> BlockchainResult<RoleConstants> {
        let (default_admin, raffle_manager, operator, pauser) = tokio::try_join!(
            self.contract.default_admin_role().call(),
            self.contract.raffle_manager_role().call(),
            self.contract.operator_role().call(),
            self.contract.pauser_role().call()
        ).map_err(|e| BlockchainError::Contract(e))?;

        Ok(RoleConstants {
            default_admin_role: default_admin,
            raffle_manager_role: raffle_manager,
            operator_role: operator,
            pauser_role: pauser,
        })
    }

    /// Pause the contract (admin only)
    pub async fn pause_contract(&self) -> BlockchainResult<Uuid> {
        info!("Pausing contract");

        let call = self.contract.pause();
        let tx_request = call.tx;

        let gas_estimate = self.gas_manager.estimate_gas_with_buffer(&tx_request, Some(20)).await?;
        
        let tx_config = TransactionConfig {
            gas_limit: gas_estimate.gas_limit,
            gas_price: Some(gas_estimate.gas_price),
            max_fee_per_gas: gas_estimate.max_fee_per_gas,
            max_priority_fee_per_gas: gas_estimate.max_priority_fee_per_gas,
            confirmations: self.client.network_config().confirmations,
            ..Default::default()
        };

        let tx_id = self.transaction_manager.submit_transaction(tx_request, Some(tx_config)).await?;
        Ok(tx_id)
    }

    /// Unpause the contract (admin only)
    pub async fn unpause_contract(&self) -> BlockchainResult<Uuid> {
        info!("Unpausing contract");

        let call = self.contract.unpause();
        let tx_request = call.tx;

        let gas_estimate = self.gas_manager.estimate_gas_with_buffer(&tx_request, Some(20)).await?;
        
        let tx_config = TransactionConfig {
            gas_limit: gas_estimate.gas_limit,
            gas_price: Some(gas_estimate.gas_price),
            max_fee_per_gas: gas_estimate.max_fee_per_gas,
            max_priority_fee_per_gas: gas_estimate.max_priority_fee_per_gas,
            confirmations: self.client.network_config().confirmations,
            ..Default::default()
        };

        let tx_id = self.transaction_manager.submit_transaction(tx_request, Some(tx_config)).await?;
        Ok(tx_id)
    }

    /// Check if contract is paused
    pub async fn is_paused(&self) -> BlockchainResult<bool> {
        let is_paused = self.contract.paused().call().await
            .map_err(|e| BlockchainError::Contract(e))?;

        Ok(is_paused)
    }

    /// Get contract address
    pub fn contract_address(&self) -> Address {
        self.contract.address()
    }

    /// Get contract instance for direct access
    pub fn contract(&self) -> &RaffleContract<SignerMiddleware<Provider<Http>, LocalWallet>> {
        &self.contract
    }

    /// Estimate gas for creating a raffle
    pub async fn estimate_create_raffle_gas(
        &self,
        params: &CreateRaffleParams,
    ) -> BlockchainResult<GasEstimate> {
        let call = self.contract.create_raffle(
            U256::from(params.item_id),
            U256::from(params.total_boxes),
            params.box_price,
            U256::from(params.total_winners),
        );

        self.gas_manager.estimate_gas_with_buffer(&call.tx, Some(20)).await
    }

    /// Estimate gas for buying a box
    pub async fn estimate_buy_box_gas(
        &self,
        raffle_id: u64,
        payment_amount: U256,
    ) -> BlockchainResult<GasEstimate> {
        let call = self.contract.buy_box(U256::from(raffle_id));
        let mut tx_request = call.tx;
        tx_request.value = Some(payment_amount);

        self.gas_manager.estimate_gas_with_buffer(&tx_request, Some(15)).await
    }

    /// Get multiple raffles in batch
    pub async fn get_raffles_batch(&self, raffle_ids: Vec<u64>) -> BlockchainResult<Vec<RaffleData>> {
        let mut raffles = Vec::new();
        
        // For now, we'll fetch them sequentially
        // In production, you might want to use multicall for better efficiency
        for raffle_id in raffle_ids {
            match self.get_raffle(raffle_id).await {
                Ok(raffle) => raffles.push(raffle),
                Err(e) => {
                    warn!("Failed to fetch raffle {}: {}", raffle_id, e);
                    // Continue with other raffles
                }
            }
        }

        Ok(raffles)
    }

    /// Get raffle summary (lightweight version)
    pub async fn get_raffle_summary(&self, raffle_id: u64) -> BlockchainResult<RaffleSummary> {
        let raffle = self.get_raffle(raffle_id).await?;
        
        Ok(RaffleSummary {
            id: raffle.id,
            item_id: raffle.item_id,
            total_boxes: raffle.total_boxes,
            boxes_sold: raffle.boxes_sold,
            box_price: raffle.box_price,
            status: raffle.status,
            created_at: raffle.created_at,
        })
    }

    /// Check if raffle exists
    pub async fn raffle_exists(&self, raffle_id: u64) -> BlockchainResult<bool> {
        let total_raffles = self.get_total_raffles().await?;
        Ok(raffle_id < total_raffles)
    }

    /// Get active raffles (status = Open)
    pub async fn get_active_raffles(&self, limit: Option<u64>) -> BlockchainResult<Vec<RaffleSummary>> {
        let total_raffles = self.get_total_raffles().await?;
        let limit = limit.unwrap_or(100).min(total_raffles);
        
        let mut active_raffles = Vec::new();
        
        // Start from the most recent raffles
        for i in (0..total_raffles).rev().take(limit as usize) {
            if let Ok(summary) = self.get_raffle_summary(i).await {
                if matches!(summary.status, RaffleStatus::Open) {
                    active_raffles.push(summary);
                }
            }
        }

        Ok(active_raffles)
    }
}

#[derive(Debug, Clone)]
pub struct RaffleSummary {
    pub id: U256,
    pub item_id: U256,
    pub total_boxes: U256,
    pub boxes_sold: U256,
    pub box_price: U256,
    pub status: RaffleStatus,
    pub created_at: U256,
}