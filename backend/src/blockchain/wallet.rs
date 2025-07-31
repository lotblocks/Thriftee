use crate::blockchain::types::*;
use crate::utils::crypto::{encrypt_data, decrypt_data};
use ethers::prelude::*;
use ethers::signers::coins_bip39::{English, Mnemonic as EthersMnemonic};
use ethers::utils::keccak256;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use bip39::{Mnemonic as BipMnemonic, Language, Seed as BipSeed};
use hdwallet::{ExtendedPrivKey, KeyIndex};
use secp256k1::Secp256k1;

/// Wallet manager handles HD wallet generation, key management, and transaction signing
#[derive(Clone)]
pub struct WalletManager {
    master_key: Arc<RwLock<Option<Vec<u8>>>>,
    wallets: Arc<RwLock<HashMap<String, EncryptedWallet>>>,
    derivation_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct EncryptedWallet {
    user_id: String,
    encrypted_private_key: Vec<u8>,
    public_key: Vec<u8>,
    address: Address,
    derivation_index: u32,
    created_at: u64,
    last_used: u64,
}

#[derive(Debug, Clone)]
pub struct WalletInfo {
    pub user_id: String,
    pub address: Address,
    pub public_key: Vec<u8>,
    pub derivation_index: u32,
    pub created_at: u64,
    pub last_used: u64,
}

#[derive(Debug, Clone)]
pub struct WalletBalance {
    pub address: Address,
    pub balance: U256,
    pub nonce: U256,
    pub last_updated: u64,
}

impl WalletManager {
    /// Create a new wallet manager
    pub fn new() -> Self {
        Self {
            master_key: Arc::new(RwLock::new(None)),
            wallets: Arc::new(RwLock::new(HashMap::new())),
            derivation_path: "m/44'/60'/0'/0".to_string(), // Ethereum derivation path
        }
    }

    /// Initialize with master seed (should be called once at startup)
    pub async fn initialize_with_seed(&self, master_seed: &[u8]) -> BlockchainResult<()> {
        let mut master_key = self.master_key.write().await;
        *master_key = Some(master_seed.to_vec());
        
        info!("Wallet manager initialized with master seed");
        Ok(())
    }

    /// Generate master seed from mnemonic
    pub fn generate_master_seed_from_mnemonic(mnemonic: &str) -> BlockchainResult<Vec<u8>> {
        let mnemonic = EthersMnemonic::<English>::new_from_phrase(mnemonic)
            .map_err(|e| BlockchainError::Wallet(format!("Invalid mnemonic: {}", e)))?;
        
        Ok(mnemonic.to_seed(""))
    }

    /// Generate a new mnemonic phrase
    pub fn generate_mnemonic() -> BlockchainResult<String> {
        let mnemonic = EthersMnemonic::<English>::new(&mut rand::thread_rng())
            .map_err(|e| BlockchainError::Wallet(format!("Failed to generate mnemonic: {}", e)))?;
        
        Ok(mnemonic.to_phrase())
    }

    /// Create a new HD wallet for a user using BIP44 standard
    pub async fn create_wallet_for_user(
        &self,
        user_id: &str,
        encryption_key: &[u8],
    ) -> BlockchainResult<WalletInfo> {
        let master_key = self.master_key.read().await;
        let master_seed = master_key
            .as_ref()
            .ok_or_else(|| BlockchainError::Wallet("Master key not initialized".to_string()))?;

        // Find next available derivation index
        let derivation_index = self.get_next_derivation_index().await;

        // Derive wallet from master seed using BIP44
        let wallet = self.derive_wallet_from_seed_bip44(master_seed, derivation_index)?;
        
        // Encrypt private key
        let private_key_bytes = wallet.signer().to_bytes();
        let encrypted_private_key = encrypt_data(&private_key_bytes, encryption_key)
            .map_err(|e| BlockchainError::Wallet(format!("Failed to encrypt private key: {}", e)))?;

        // Get public key
        let public_key = wallet.signer().verifying_key().to_encoded_point(false);
        let public_key_bytes = public_key.as_bytes().to_vec();

        let now = chrono::Utc::now().timestamp() as u64;

        let encrypted_wallet = EncryptedWallet {
            user_id: user_id.to_string(),
            encrypted_private_key,
            public_key: public_key_bytes.clone(),
            address: wallet.address(),
            derivation_index,
            created_at: now,
            last_used: now,
        };

        // Store encrypted wallet
        {
            let mut wallets = self.wallets.write().await;
            wallets.insert(user_id.to_string(), encrypted_wallet);
        }

        info!(
            "Created HD wallet for user {} at address {:?} (BIP44 index: {})",
            user_id, wallet.address(), derivation_index
        );

        Ok(WalletInfo {
            user_id: user_id.to_string(),
            address: wallet.address(),
            public_key: public_key_bytes,
            derivation_index,
            created_at: now,
            last_used: now,
        })
    }

    /// Get wallet information for a user
    pub async fn get_wallet_info(&self, user_id: &str) -> Option<WalletInfo> {
        let wallets = self.wallets.read().await;
        wallets.get(user_id).map(|w| WalletInfo {
            user_id: w.user_id.clone(),
            address: w.address,
            public_key: w.public_key.clone(),
            derivation_index: w.derivation_index,
            created_at: w.created_at,
            last_used: w.last_used,
        })
    }

    /// Get wallet address for a user
    pub async fn get_wallet_address(&self, user_id: &str) -> Option<Address> {
        let wallets = self.wallets.read().await;
        wallets.get(user_id).map(|w| w.address)
    }

    /// Load wallet for signing (decrypts private key temporarily)
    pub async fn load_wallet_for_signing(
        &self,
        user_id: &str,
        encryption_key: &[u8],
    ) -> BlockchainResult<LocalWallet> {
        let encrypted_wallet = {
            let wallets = self.wallets.read().await;
            wallets
                .get(user_id)
                .cloned()
                .ok_or_else(|| BlockchainError::Wallet("Wallet not found".to_string()))?
        };

        // Decrypt private key
        let private_key_bytes = decrypt_data(&encrypted_wallet.encrypted_private_key, encryption_key)
            .map_err(|e| BlockchainError::Wallet(format!("Failed to decrypt private key: {}", e)))?;

        // Create wallet from private key
        let wallet = LocalWallet::from_bytes(&private_key_bytes)
            .map_err(|e| BlockchainError::Wallet(format!("Invalid private key: {}", e)))?;

        // Update last used timestamp
        {
            let mut wallets = self.wallets.write().await;
            if let Some(w) = wallets.get_mut(user_id) {
                w.last_used = chrono::Utc::now().timestamp() as u64;
            }
        }

        debug!("Loaded wallet for user {} for signing", user_id);
        Ok(wallet)
    }

    /// Sign a transaction for a user
    pub async fn sign_transaction(
        &self,
        user_id: &str,
        transaction: &TransactionRequest,
        encryption_key: &[u8],
        chain_id: u64,
    ) -> BlockchainResult<Signature> {
        let wallet = self.load_wallet_for_signing(user_id, encryption_key).await?;
        let wallet = wallet.with_chain_id(chain_id);

        // Convert TransactionRequest to TypedTransaction for signing
        let typed_tx = self.transaction_request_to_typed_transaction(transaction, chain_id)?;
        
        let signature = wallet
            .sign_transaction(&typed_tx)
            .await
            .map_err(|e| BlockchainError::Wallet(format!("Failed to sign transaction: {}", e)))?;

        debug!("Signed transaction for user {}", user_id);
        Ok(signature)
    }

    /// Sign a message for a user
    pub async fn sign_message(
        &self,
        user_id: &str,
        message: &[u8],
        encryption_key: &[u8],
    ) -> BlockchainResult<Signature> {
        let wallet = self.load_wallet_for_signing(user_id, encryption_key).await?;
        
        let signature = wallet
            .sign_message(message)
            .await
            .map_err(|e| BlockchainError::Wallet(format!("Failed to sign message: {}", e)))?;

        debug!("Signed message for user {}", user_id);
        Ok(signature)
    }

    /// Verify a signature
    pub fn verify_signature(
        &self,
        message: &[u8],
        signature: &Signature,
        expected_address: Address,
    ) -> BlockchainResult<bool> {
        let message_hash = keccak256(message);
        let recovered_address = signature
            .recover(message_hash)
            .map_err(|e| BlockchainError::Wallet(format!("Failed to recover address: {}", e)))?;

        Ok(recovered_address == expected_address)
    }

    /// Get all wallet addresses (for monitoring purposes)
    pub async fn get_all_wallet_addresses(&self) -> Vec<Address> {
        let wallets = self.wallets.read().await;
        wallets.values().map(|w| w.address).collect()
    }

    /// Get wallet statistics
    pub async fn get_wallet_statistics(&self) -> WalletStatistics {
        let wallets = self.wallets.read().await;
        let total_wallets = wallets.len();
        
        let now = chrono::Utc::now().timestamp() as u64;
        let active_wallets = wallets
            .values()
            .filter(|w| now - w.last_used < 86400) // Active in last 24 hours
            .count();

        let oldest_wallet = wallets
            .values()
            .map(|w| w.created_at)
            .min()
            .unwrap_or(now);

        WalletStatistics {
            total_wallets,
            active_wallets,
            oldest_wallet_age: now - oldest_wallet,
        }
    }

    /// Backup wallet data (encrypted)
    pub async fn backup_wallets(&self) -> BlockchainResult<Vec<u8>> {
        let wallets = self.wallets.read().await;
        let backup_data = serde_json::to_vec(&*wallets)
            .map_err(|e| BlockchainError::Serialization(e))?;
        
        info!("Created backup of {} wallets", wallets.len());
        Ok(backup_data)
    }

    /// Restore wallet data from backup
    pub async fn restore_wallets(&self, backup_data: &[u8]) -> BlockchainResult<()> {
        let restored_wallets: HashMap<String, EncryptedWallet> = serde_json::from_slice(backup_data)
            .map_err(|e| BlockchainError::Serialization(e))?;

        {
            let mut wallets = self.wallets.write().await;
            *wallets = restored_wallets;
        }

        let wallet_count = {
            let wallets = self.wallets.read().await;
            wallets.len()
        };

        info!("Restored {} wallets from backup", wallet_count);
        Ok(())
    }

    /// Derive wallet from seed using BIP44 derivation path
    fn derive_wallet_from_seed(&self, seed: &[u8], index: u32) -> BlockchainResult<LocalWallet> {
        // For simplicity, we'll use a hash-based derivation
        // In production, you might want to use proper BIP32/BIP44 derivation
        let mut hasher = sha2::Sha256::new();
        hasher.update(seed);
        hasher.update(&index.to_be_bytes());
        let derived_key = hasher.finalize();

        LocalWallet::from_bytes(&derived_key)
            .map_err(|e| BlockchainError::Wallet(format!("Failed to derive wallet: {}", e)))
    }

    /// Derive wallet from seed using proper BIP44 derivation path
    fn derive_wallet_from_seed_bip44(&self, seed: &[u8], account_index: u32) -> BlockchainResult<LocalWallet> {
        use bip39::{Mnemonic, Language, Seed as BipSeed};
        use hdwallet::{ExtendedPrivKey, KeyIndex};
        use secp256k1::Secp256k1;

        // Create secp256k1 context
        let secp = Secp256k1::new();

        // Create master key from seed
        let master_key = ExtendedPrivKey::new_master(&secp, seed)
            .map_err(|e| BlockchainError::Wallet(format!("Failed to create master key: {}", e)))?;

        // Derive using BIP44 path: m/44'/60'/account'/0/0
        let derived_key = master_key
            .derive_priv(&secp, &[
                KeyIndex::Hardened(44),           // Purpose: BIP44
                KeyIndex::Hardened(60),           // Coin type: Ethereum
                KeyIndex::Hardened(account_index), // Account
                KeyIndex::Normal(0),              // Change: 0 (external)
                KeyIndex::Normal(0),              // Address index: 0
            ])
            .map_err(|e| BlockchainError::Wallet(format!("Failed to derive key: {}", e)))?;

        // Create LocalWallet from derived private key
        let private_key_bytes = derived_key.private_key.secret_bytes();
        LocalWallet::from_bytes(&private_key_bytes)
            .map_err(|e| BlockchainError::Wallet(format!("Failed to create wallet: {}", e)))
    }

    /// Get next available derivation index
    async fn get_next_derivation_index(&self) -> u32 {
        let wallets = self.wallets.read().await;
        wallets
            .values()
            .map(|w| w.derivation_index)
            .max()
            .map(|max| max + 1)
            .unwrap_or(0)
    }

    /// Convert TransactionRequest to TypedTransaction
    fn transaction_request_to_typed_transaction(
        &self,
        request: &TransactionRequest,
        chain_id: u64,
    ) -> BlockchainResult<TypedTransaction> {
        let tx = if request.max_fee_per_gas.is_some() || request.max_priority_fee_per_gas.is_some() {
            // EIP-1559 transaction
            TypedTransaction::Eip1559(Eip1559TransactionRequest {
                to: request.to,
                from: request.from.unwrap_or_default(),
                data: request.data.clone().unwrap_or_default(),
                value: request.value,
                gas: request.gas,
                nonce: request.nonce,
                max_fee_per_gas: request.max_fee_per_gas,
                max_priority_fee_per_gas: request.max_priority_fee_per_gas,
                access_list: AccessList::default(),
                chain_id: Some(chain_id.into()),
            })
        } else {
            // Legacy transaction
            TypedTransaction::Legacy(TransactionRequest {
                to: request.to,
                from: request.from,
                data: request.data.clone(),
                value: request.value,
                gas: request.gas,
                gas_price: request.gas_price,
                nonce: request.nonce,
                chain_id: Some(chain_id.into()),
            })
        };

        Ok(tx)
    }

    /// Clean up old unused wallets (for maintenance)
    pub async fn cleanup_unused_wallets(&self, max_age_days: u64) -> usize {
        let cutoff_time = chrono::Utc::now().timestamp() as u64 - (max_age_days * 86400);
        let mut wallets = self.wallets.write().await;
        
        let initial_count = wallets.len();
        wallets.retain(|_, wallet| wallet.last_used > cutoff_time);
        let final_count = wallets.len();
        
        let removed_count = initial_count - final_count;
        if removed_count > 0 {
            info!("Cleaned up {} unused wallets", removed_count);
        }
        
        removed_count
    }
}

#[derive(Debug, Clone)]
pub struct WalletStatistics {
    pub total_wallets: usize,
    pub active_wallets: usize,
    pub oldest_wallet_age: u64, // seconds
}

// Additional imports needed
use sha2::{Digest, Sha256};