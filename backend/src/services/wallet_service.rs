use ethers::{
    core::k256::ecdsa::SigningKey,
    prelude::*,
    signers::{LocalWallet, Signer},
    types::{Address, Signature, TransactionRequest, U256},
};
use sqlx::PgPool;
use std::str::FromStr;
use uuid::Uuid;
use bip39::{Mnemonic, Language, Seed};
use hdwallet::{ExtendedPrivKey, ExtendedPubKey, KeyIndex, DefaultKeyChain};
use secp256k1::Secp256k1;
use sha2::{Digest, Sha256};

use crate::error::AppError;
use crate::models::User;
use crate::utils::crypto::{encrypt_sensitive_data, decrypt_sensitive_data, derive_key_from_password};

#[cfg(test)]
mod tests;

#[derive(Clone)]
pub struct WalletService {
    pool: PgPool,
}

impl WalletService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Generate a new HD wallet for a user using BIP44 standard
    pub async fn generate_wallet_for_user(
        &self,
        user_id: Uuid,
        password: &str,
    ) -> Result<(String, String, String), AppError> {
        // Generate a new mnemonic phrase (12 words)
        let mnemonic = Mnemonic::generate_in(Language::English, 12)
            .map_err(|e| AppError::Internal(format!("Failed to generate mnemonic: {}", e)))?;

        // Generate seed from mnemonic
        let seed = Seed::new(&mnemonic, "");

        // Create HD wallet using BIP44 derivation path for Ethereum
        // m/44'/60'/0'/0/0 (first account, first address)
        let secp = Secp256k1::new();
        let master_key = ExtendedPrivKey::new_master(&secp, seed.as_bytes())
            .map_err(|e| AppError::Internal(format!("Failed to create master key: {}", e)))?;

        // Derive account key using BIP44 path: m/44'/60'/0'/0/0
        let account_key = master_key
            .derive_priv(&secp, &[
                KeyIndex::Hardened(44),  // Purpose: BIP44
                KeyIndex::Hardened(60),  // Coin type: Ethereum
                KeyIndex::Hardened(0),   // Account: 0
                KeyIndex::Normal(0),     // Change: 0 (external)
                KeyIndex::Normal(0),     // Address index: 0
            ])
            .map_err(|e| AppError::Internal(format!("Failed to derive account key: {}", e)))?;

        // Create LocalWallet from derived private key
        let private_key_bytes = account_key.private_key.secret_bytes();
        let signing_key = SigningKey::from_slice(&private_key_bytes)
            .map_err(|e| AppError::Internal(format!("Failed to create signing key: {}", e)))?;

        let wallet = LocalWallet::from(signing_key);
        let address = format!("{:?}", wallet.address());

        // Derive encryption key from user password
        let salt = format!("wallet_salt_{}", user_id).as_bytes().to_vec();
        let encryption_key = derive_key_from_password(password, &salt[..32]);

        // Encrypt the private key
        let encrypted_private_key = encrypt_sensitive_data(
            &hex::encode(private_key_bytes),
            &encryption_key,
        )?;

        // Encrypt the mnemonic phrase for backup
        let encrypted_mnemonic = encrypt_sensitive_data(
            &mnemonic.phrase(),
            &encryption_key,
        )?;

        Ok((address, encrypted_private_key, encrypted_mnemonic))
    }

    /// Get wallet for user (decrypt private key)
    pub async fn get_user_wallet(
        &self,
        user_id: Uuid,
        password: &str,
    ) -> Result<LocalWallet, AppError> {
        let user = User::find_by_id(&self.pool, user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        // Derive decryption key
        let salt = format!("wallet_salt_{}", user_id).as_bytes().to_vec();
        let decryption_key = derive_key_from_password(password, &salt[..32]);

        // Decrypt private key
        let private_key_hex = decrypt_sensitive_data(
            &user.internal_wallet_private_key_encrypted,
            &decryption_key,
        )?;

        let private_key_bytes = hex::decode(private_key_hex)
            .map_err(|e| AppError::Internal(format!("Failed to decode private key: {}", e)))?;

        // Create wallet from private key
        let signing_key = SigningKey::from_slice(&private_key_bytes)
            .map_err(|e| AppError::Internal(format!("Invalid private key: {}", e)))?;

        let wallet = LocalWallet::from(signing_key);

        // Verify the address matches
        let expected_address = Address::from_str(&user.internal_wallet_address)
            .map_err(|e| AppError::Internal(format!("Invalid wallet address: {}", e)))?;

        if wallet.address() != expected_address {
            return Err(AppError::Internal("Wallet address mismatch".to_string()));
        }

        Ok(wallet)
    }

    /// Get wallet address for user (without decrypting private key)
    pub async fn get_user_wallet_address(&self, user_id: Uuid) -> Result<String, AppError> {
        let user = User::find_by_id(&self.pool, user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        Ok(user.internal_wallet_address)
    }

    /// Sign a transaction with user's wallet
    pub async fn sign_transaction(
        &self,
        user_id: Uuid,
        password: &str,
        transaction: TransactionRequest,
    ) -> Result<Signature, AppError> {
        let wallet = self.get_user_wallet(user_id, password).await?;
        
        // Convert TransactionRequest to TypedTransaction for signing
        let typed_tx = transaction.into();
        
        wallet
            .sign_transaction(&typed_tx)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to sign transaction: {}", e)))
    }

    /// Sign arbitrary data with user's wallet
    pub async fn sign_message(
        &self,
        user_id: Uuid,
        password: &str,
        message: &[u8],
    ) -> Result<Signature, AppError> {
        let wallet = self.get_user_wallet(user_id, password).await?;
        
        wallet
            .sign_message(message)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to sign message: {}", e)))
    }

    /// Verify a signature against user's wallet
    pub async fn verify_signature(
        &self,
        user_id: Uuid,
        message: &[u8],
        signature: &Signature,
    ) -> Result<bool, AppError> {
        let user = User::find_by_id(&self.pool, user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        let wallet_address = Address::from_str(&user.internal_wallet_address)
            .map_err(|e| AppError::Internal(format!("Invalid wallet address: {}", e)))?;

        // Recover address from signature
        let recovered_address = signature
            .recover(message)
            .map_err(|e| AppError::Internal(format!("Failed to recover address: {}", e)))?;

        Ok(recovered_address == wallet_address)
    }

    /// Create a transaction request for contract interaction
    pub fn create_contract_transaction(
        &self,
        contract_address: Address,
        data: Vec<u8>,
        value: Option<U256>,
        gas_limit: Option<U256>,
        gas_price: Option<U256>,
    ) -> TransactionRequest {
        let mut tx = TransactionRequest::new()
            .to(contract_address)
            .data(data);

        if let Some(value) = value {
            tx = tx.value(value);
        }

        if let Some(gas_limit) = gas_limit {
            tx = tx.gas(gas_limit);
        }

        if let Some(gas_price) = gas_price {
            tx = tx.gas_price(gas_price);
        }

        tx
    }

    /// Get optimal gas price (simplified implementation)
    pub async fn get_optimal_gas_price(&self) -> Result<U256, AppError> {
        // In a real implementation, you'd query the network for current gas prices
        // For now, return a reasonable default (20 gwei)
        Ok(U256::from(20_000_000_000u64))
    }

    /// Estimate gas for a transaction
    pub async fn estimate_gas(
        &self,
        transaction: &TransactionRequest,
    ) -> Result<U256, AppError> {
        // In a real implementation, you'd use a provider to estimate gas
        // For now, return a reasonable default
        Ok(U256::from(100_000u64))
    }

    /// Check if wallet address is valid
    pub fn is_valid_address(address: &str) -> bool {
        Address::from_str(address).is_ok()
    }

    /// Generate a deterministic wallet address from user ID (for testing)
    pub fn generate_deterministic_address(user_id: Uuid) -> String {
        use sha2::{Digest, Sha256};
        
        let mut hasher = Sha256::new();
        hasher.update(user_id.as_bytes());
        hasher.update(b"wallet_seed");
        let hash = hasher.finalize();
        
        // Take first 20 bytes for address
        let address_bytes = &hash[..20];
        format!("0x{}", hex::encode(address_bytes))
    }

    /// Rotate wallet encryption (change password)
    pub async fn rotate_wallet_encryption(
        &self,
        user_id: Uuid,
        old_password: &str,
        new_password: &str,
    ) -> Result<(), AppError> {
        // Get current wallet
        let wallet = self.get_user_wallet(user_id, old_password).await?;
        let private_key_bytes = wallet.signer().to_bytes();

        // Derive new encryption key
        let salt = format!("wallet_salt_{}", user_id).as_bytes().to_vec();
        let new_encryption_key = derive_key_from_password(new_password, &salt[..32]);

        // Encrypt with new key
        let new_encrypted_private_key = encrypt_sensitive_data(
            &hex::encode(private_key_bytes),
            &new_encryption_key,
        )?;

        // Update in database
        sqlx::query!(
            "UPDATE users SET internal_wallet_private_key_encrypted = $1, updated_at = NOW() WHERE id = $2",
            new_encrypted_private_key,
            user_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Export wallet private key (for backup purposes)
    pub async fn export_private_key(
        &self,
        user_id: Uuid,
        password: &str,
    ) -> Result<String, AppError> {
        let wallet = self.get_user_wallet(user_id, password).await?;
        let private_key_bytes = wallet.signer().to_bytes();
        Ok(hex::encode(private_key_bytes))
    }

    /// Import wallet from private key
    pub async fn import_wallet(
        &self,
        user_id: Uuid,
        password: &str,
        private_key_hex: &str,
    ) -> Result<String, AppError> {
        // Validate private key
        let private_key_bytes = hex::decode(private_key_hex)
            .map_err(|e| AppError::Validation(format!("Invalid private key format: {}", e)))?;

        let signing_key = SigningKey::from_slice(&private_key_bytes)
            .map_err(|e| AppError::Validation(format!("Invalid private key: {}", e)))?;

        let wallet = LocalWallet::from(signing_key);
        let address = format!("{:?}", wallet.address());

        // Encrypt private key
        let salt = format!("wallet_salt_{}", user_id).as_bytes().to_vec();
        let encryption_key = derive_key_from_password(password, &salt[..32]);
        let encrypted_private_key = encrypt_sensitive_data(private_key_hex, &encryption_key)?;

        // Update in database
        sqlx::query!(
            "UPDATE users SET internal_wallet_address = $1, internal_wallet_private_key_encrypted = $2, updated_at = NOW() WHERE id = $3",
            address,
            encrypted_private_key,
            user_id
        )
        .execute(&self.pool)
        .await?;

        Ok(address)
    }

    /// Generate HD wallet from mnemonic phrase
    pub async fn generate_wallet_from_mnemonic(
        &self,
        user_id: Uuid,
        password: &str,
        mnemonic_phrase: &str,
        account_index: u32,
        address_index: u32,
    ) -> Result<(String, String), AppError> {
        // Parse mnemonic
        let mnemonic = Mnemonic::parse_in(Language::English, mnemonic_phrase)
            .map_err(|e| AppError::Validation(format!("Invalid mnemonic phrase: {}", e)))?;

        // Generate seed from mnemonic
        let seed = Seed::new(&mnemonic, "");

        // Create HD wallet using BIP44 derivation path
        let secp = Secp256k1::new();
        let master_key = ExtendedPrivKey::new_master(&secp, seed.as_bytes())
            .map_err(|e| AppError::Internal(format!("Failed to create master key: {}", e)))?;

        // Derive account key using BIP44 path: m/44'/60'/account'/0/address_index
        let account_key = master_key
            .derive_priv(&secp, &[
                KeyIndex::Hardened(44),                    // Purpose: BIP44
                KeyIndex::Hardened(60),                    // Coin type: Ethereum
                KeyIndex::Hardened(account_index),         // Account
                KeyIndex::Normal(0),                       // Change: 0 (external)
                KeyIndex::Normal(address_index),           // Address index
            ])
            .map_err(|e| AppError::Internal(format!("Failed to derive account key: {}", e)))?;

        // Create LocalWallet from derived private key
        let private_key_bytes = account_key.private_key.secret_bytes();
        let signing_key = SigningKey::from_slice(&private_key_bytes)
            .map_err(|e| AppError::Internal(format!("Failed to create signing key: {}", e)))?;

        let wallet = LocalWallet::from(signing_key);
        let address = format!("{:?}", wallet.address());

        // Derive encryption key from user password
        let salt = format!("wallet_salt_{}", user_id).as_bytes().to_vec();
        let encryption_key = derive_key_from_password(password, &salt[..32]);

        // Encrypt the private key
        let encrypted_private_key = encrypt_sensitive_data(
            &hex::encode(private_key_bytes),
            &encryption_key,
        )?;

        Ok((address, encrypted_private_key))
    }

    /// Derive multiple addresses from HD wallet
    pub async fn derive_addresses(
        &self,
        user_id: Uuid,
        password: &str,
        count: u32,
    ) -> Result<Vec<String>, AppError> {
        let user = User::find_by_id(&self.pool, user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        // Get encrypted mnemonic (assuming it's stored in a separate field)
        // For now, we'll derive from the existing private key
        let wallet = self.get_user_wallet(user_id, password).await?;
        let private_key_bytes = wallet.signer().to_bytes();

        // For demonstration, we'll generate deterministic addresses
        // In a real implementation, you'd store the mnemonic and derive properly
        let mut addresses = Vec::new();
        for i in 0..count {
            let mut hasher = sha2::Sha256::new();
            hasher.update(&private_key_bytes);
            hasher.update(&i.to_be_bytes());
            let derived_key = hasher.finalize();

            let signing_key = SigningKey::from_slice(&derived_key)
                .map_err(|e| AppError::Internal(format!("Failed to derive key {}: {}", i, e)))?;

            let derived_wallet = LocalWallet::from(signing_key);
            addresses.push(format!("{:?}", derived_wallet.address()));
        }

        Ok(addresses)
    }

    /// Check wallet balance on blockchain
    pub async fn check_wallet_balance(
        &self,
        user_id: Uuid,
        provider_url: &str,
    ) -> Result<U256, AppError> {
        let user = User::find_by_id(&self.pool, user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        let wallet_address = Address::from_str(&user.internal_wallet_address)
            .map_err(|e| AppError::Internal(format!("Invalid wallet address: {}", e)))?;

        // Create provider connection
        let provider = Provider::<Http>::try_from(provider_url)
            .map_err(|e| AppError::Internal(format!("Failed to connect to provider: {}", e)))?;

        // Get balance
        let balance = provider
            .get_balance(wallet_address, None)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to get balance: {}", e)))?;

        Ok(balance)
    }

    /// Get transaction count (nonce) for wallet
    pub async fn get_transaction_count(
        &self,
        user_id: Uuid,
        provider_url: &str,
    ) -> Result<U256, AppError> {
        let user = User::find_by_id(&self.pool, user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        let wallet_address = Address::from_str(&user.internal_wallet_address)
            .map_err(|e| AppError::Internal(format!("Invalid wallet address: {}", e)))?;

        // Create provider connection
        let provider = Provider::<Http>::try_from(provider_url)
            .map_err(|e| AppError::Internal(format!("Failed to connect to provider: {}", e)))?;

        // Get transaction count
        let nonce = provider
            .get_transaction_count(wallet_address, None)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to get transaction count: {}", e)))?;

        Ok(nonce)
    }

    /// Send transaction using user's wallet
    pub async fn send_transaction(
        &self,
        user_id: Uuid,
        password: &str,
        transaction: TransactionRequest,
        provider_url: &str,
        chain_id: u64,
    ) -> Result<TxHash, AppError> {
        let wallet = self.get_user_wallet(user_id, password).await?;
        
        // Create provider connection
        let provider = Provider::<Http>::try_from(provider_url)
            .map_err(|e| AppError::Internal(format!("Failed to connect to provider: {}", e)))?;

        // Connect wallet to provider
        let wallet_with_provider = wallet.with_chain_id(chain_id).connect(provider);

        // Send transaction
        let pending_tx = wallet_with_provider
            .send_transaction(transaction, None)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to send transaction: {}", e)))?;

        Ok(pending_tx.tx_hash())
    }

    /// Get wallet transaction history
    pub async fn get_transaction_history(
        &self,
        user_id: Uuid,
        provider_url: &str,
        from_block: Option<u64>,
        to_block: Option<u64>,
    ) -> Result<Vec<Transaction>, AppError> {
        let user = User::find_by_id(&self.pool, user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        let wallet_address = Address::from_str(&user.internal_wallet_address)
            .map_err(|e| AppError::Internal(format!("Invalid wallet address: {}", e)))?;

        // Create provider connection
        let provider = Provider::<Http>::try_from(provider_url)
            .map_err(|e| AppError::Internal(format!("Failed to connect to provider: {}", e)))?;

        // Get transaction history (simplified - in practice you'd use event filters)
        let latest_block = provider
            .get_block_number()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to get latest block: {}", e)))?;

        let from = from_block.unwrap_or(0);
        let to = to_block.unwrap_or(latest_block.as_u64());

        let mut transactions = Vec::new();

        // This is a simplified implementation - in practice you'd use more efficient methods
        for block_num in from..=to.min(from + 100) { // Limit to 100 blocks for performance
            if let Ok(Some(block)) = provider.get_block_with_txs(block_num).await {
                for tx in block.transactions {
                    if tx.from == wallet_address || tx.to == Some(wallet_address) {
                        transactions.push(tx);
                    }
                }
            }
        }

        Ok(transactions)
    }

    /// Update user's encrypted mnemonic in database
    pub async fn update_user_mnemonic(
        &self,
        user_id: Uuid,
        encrypted_mnemonic: &str,
    ) -> Result<(), AppError> {
        sqlx::query!(
            "UPDATE users SET internal_wallet_mnemonic_encrypted = $1, updated_at = NOW() WHERE id = $2",
            encrypted_mnemonic,
            user_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get user's encrypted mnemonic from database
    pub async fn get_user_mnemonic(
        &self,
        user_id: Uuid,
        password: &str,
    ) -> Result<Option<String>, AppError> {
        let user = User::find_by_id(&self.pool, user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        if let Some(encrypted_mnemonic) = user.internal_wallet_mnemonic_encrypted {
            // Derive decryption key
            let salt = format!("wallet_salt_{}", user_id).as_bytes().to_vec();
            let decryption_key = derive_key_from_password(password, &salt[..32]);

            // Decrypt mnemonic
            let mnemonic = decrypt_sensitive_data(&encrypted_mnemonic, &decryption_key)?;
            Ok(Some(mnemonic))
        } else {
            Ok(None)
        }
    }

    /// Validate mnemonic phrase
    pub fn validate_mnemonic(mnemonic_phrase: &str) -> Result<(), AppError> {
        Mnemonic::parse_in(Language::English, mnemonic_phrase)
            .map_err(|e| AppError::Validation(format!("Invalid mnemonic phrase: {}", e)))?;
        Ok(())
    }

    /// Generate a new mnemonic phrase
    pub fn generate_mnemonic_phrase() -> Result<String, AppError> {
        let mnemonic = Mnemonic::generate_in(Language::English, 12)
            .map_err(|e| AppError::Internal(format!("Failed to generate mnemonic: {}", e)))?;
        Ok(mnemonic.phrase().to_string())
    }
}

