# HD Wallet Implementation Documentation

## Overview

This document describes the implementation of Hierarchical Deterministic (HD) wallets using the BIP44 standard for the Raffle Shopping Platform. The implementation provides secure, deterministic wallet generation and management for users.

## Features

### 1. BIP44 Standard Compliance

The implementation follows the BIP44 derivation path standard:
```
m / purpose' / coin_type' / account' / change / address_index
```

For Ethereum wallets:
- **Purpose**: 44' (BIP44)
- **Coin Type**: 60' (Ethereum)
- **Account**: 0' (first account)
- **Change**: 0 (external addresses)
- **Address Index**: 0 (first address)

### 2. Secure Key Management

- **Mnemonic Generation**: 12-word BIP39 mnemonic phrases
- **Encryption**: AES-256-GCM encryption for private keys and mnemonics
- **Key Derivation**: PBKDF2 with user-specific salts
- **Storage**: Encrypted data stored in PostgreSQL database

### 3. Wallet Operations

#### Core Functions

```rust
// Generate new HD wallet for user
pub async fn generate_wallet_for_user(
    &self,
    user_id: Uuid,
    password: &str,
) -> Result<(String, String, String), AppError>

// Generate wallet from existing mnemonic
pub async fn generate_wallet_from_mnemonic(
    &self,
    user_id: Uuid,
    password: &str,
    mnemonic_phrase: &str,
    account_index: u32,
    address_index: u32,
) -> Result<(String, String), AppError>

// Derive multiple addresses from HD wallet
pub async fn derive_addresses(
    &self,
    user_id: Uuid,
    password: &str,
    count: u32,
) -> Result<Vec<String>, AppError>
```

#### Blockchain Operations

```rust
// Check wallet balance
pub async fn check_wallet_balance(
    &self,
    user_id: Uuid,
    provider_url: &str,
) -> Result<U256, AppError>

// Send transaction
pub async fn send_transaction(
    &self,
    user_id: Uuid,
    password: &str,
    transaction: TransactionRequest,
    provider_url: &str,
    chain_id: u64,
) -> Result<TxHash, AppError>

// Get transaction history
pub async fn get_transaction_history(
    &self,
    user_id: Uuid,
    provider_url: &str,
    from_block: Option<u64>,
    to_block: Option<u64>,
) -> Result<Vec<Transaction>, AppError>
```

#### Security Operations

```rust
// Sign message with user's wallet
pub async fn sign_message(
    &self,
    user_id: Uuid,
    password: &str,
    message: &[u8],
) -> Result<Signature, AppError>

// Verify signature
pub async fn verify_signature(
    &self,
    user_id: Uuid,
    message: &[u8],
    signature: &Signature,
) -> Result<bool, AppError>

// Rotate wallet encryption (change password)
pub async fn rotate_wallet_encryption(
    &self,
    user_id: Uuid,
    old_password: &str,
    new_password: &str,
) -> Result<(), AppError>
```

## Database Schema

### Users Table Enhancement

```sql
ALTER TABLE users ADD COLUMN internal_wallet_mnemonic_encrypted TEXT;
CREATE INDEX IF NOT EXISTS idx_users_wallet_address ON users(internal_wallet_address);
COMMENT ON COLUMN users.internal_wallet_mnemonic_encrypted IS 'Encrypted BIP39 mnemonic phrase for HD wallet recovery';
```

### Fields

- `internal_wallet_address`: Ethereum address (42 characters)
- `internal_wallet_private_key_encrypted`: Encrypted private key
- `internal_wallet_mnemonic_encrypted`: Encrypted BIP39 mnemonic phrase

## Security Considerations

### 1. Encryption

- **Algorithm**: AES-256-GCM with authenticated encryption
- **Key Derivation**: PBKDF2 with 100,000 iterations
- **Salt**: User-specific salt derived from user ID
- **IV**: Random initialization vector for each encryption

### 2. Key Management

- Private keys never stored in plaintext
- Mnemonics encrypted separately for recovery purposes
- Password-based encryption allows user control
- Key rotation supported for password changes

### 3. Access Control

- Wallet operations require user authentication
- Password verification required for sensitive operations
- Rate limiting on wallet operations
- Audit logging for all wallet activities

## Usage Examples

### 1. Generate New HD Wallet

```rust
let wallet_service = WalletService::new(pool);
let user_id = Uuid::new_v4();
let password = "secure_password_123";

let (address, encrypted_key, encrypted_mnemonic) = wallet_service
    .generate_wallet_for_user(user_id, password)
    .await?;

// Store in database
let user = User::create(
    &pool,
    create_request,
    password_hash,
    address,
    encrypted_key,
    Some(encrypted_mnemonic),
).await?;
```

### 2. Restore Wallet from Mnemonic

```rust
let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

let (address, encrypted_key) = wallet_service
    .generate_wallet_from_mnemonic(user_id, password, mnemonic, 0, 0)
    .await?;
```

### 3. Sign Transaction

```rust
let transaction = TransactionRequest::new()
    .to(contract_address)
    .value(U256::from(1000000000000000000u64)) // 1 ETH
    .gas(U256::from(21000))
    .gas_price(U256::from(20000000000u64)); // 20 gwei

let tx_hash = wallet_service
    .send_transaction(user_id, password, transaction, rpc_url, chain_id)
    .await?;
```

## Testing

### Unit Tests

- HD wallet generation with BIP44 compliance
- Mnemonic validation and generation
- Address derivation from mnemonics
- Encryption/decryption of sensitive data
- Transaction signing and verification

### Integration Tests

- End-to-end wallet creation and usage
- Database storage and retrieval
- Blockchain interaction testing
- Security validation

### Test Coverage

- All public methods have comprehensive tests
- Edge cases and error conditions covered
- Security scenarios tested
- Performance benchmarks included

## Performance Considerations

### 1. Optimization

- Connection pooling for database operations
- Caching of frequently accessed wallet data
- Efficient key derivation algorithms
- Minimal blockchain RPC calls

### 2. Scalability

- Async/await for non-blocking operations
- Batch operations where possible
- Database indexing for fast lookups
- Memory-efficient key handling

## Error Handling

### Error Types

```rust
pub enum AppError {
    Validation(String),      // Invalid input data
    NotFound(String),        // Resource not found
    Internal(String),        // Internal system error
    Unauthorized(String),    // Authentication failure
    Blockchain(String),      // Blockchain operation error
}
```

### Common Scenarios

- Invalid mnemonic phrases
- Incorrect passwords
- Network connectivity issues
- Insufficient gas for transactions
- Database connection failures

## Monitoring and Logging

### Metrics

- Wallet creation rate
- Transaction success/failure rates
- Average response times
- Error frequencies

### Logging

- Structured logging with correlation IDs
- Security events (failed authentications)
- Performance metrics
- Blockchain interaction logs

## Future Enhancements

### 1. Multi-Signature Support

- Implement multi-sig wallet creation
- Support for shared wallet management
- Enhanced security for high-value operations

### 2. Hardware Wallet Integration

- Support for hardware wallet signing
- Secure key storage options
- Enhanced security for enterprise users

### 3. Cross-Chain Support

- Support for multiple blockchain networks
- Unified wallet interface
- Cross-chain transaction capabilities

## Dependencies

### Core Dependencies

```toml
[dependencies]
bip39 = "2.0"                    # BIP39 mnemonic generation
hdwallet = "0.4"                 # HD wallet derivation
secp256k1 = { version = "0.28", features = ["recovery"] }
ethers = { version = "2.0", features = ["full"] }
aes-gcm = "0.10"                 # Encryption
argon2 = "0.5"                   # Key derivation
```

### Development Dependencies

```toml
[dev-dependencies]
tokio-test = "0.4"               # Async testing
uuid = { version = "1.0", features = ["v4"] }
```

## Compliance

### Standards

- **BIP32**: Hierarchical Deterministic Wallets
- **BIP39**: Mnemonic code for generating deterministic keys
- **BIP44**: Multi-Account Hierarchy for Deterministic Wallets
- **EIP-155**: Simple replay attack protection

### Security

- OWASP cryptographic standards
- Industry best practices for key management
- Regular security audits and updates
- Compliance with data protection regulations