# HD Wallet System Documentation

## Overview

The HD (Hierarchical Deterministic) Wallet System provides comprehensive blockchain wallet functionality for the Raffle Shopping Platform, implementing BIP44 standard for deterministic wallet generation, secure private key management, and transaction signing capabilities.

## Core Components

### 1. Wallet Service (`backend/src/services/wallet_service.rs`)

#### Features:
- **HD Wallet Generation**: BIP44 standard implementation for deterministic wallet creation
- **Secure Key Management**: AES-256-GCM encryption for private keys and mnemonic phrases
- **Transaction Signing**: Support for message and transaction signing
- **Multi-Address Derivation**: Generate multiple addresses from single seed
- **Blockchain Integration**: Balance checking, transaction sending, and history retrieval
- **Import/Export**: Wallet backup and restoration capabilities

#### Key Methods:
- `generate_wallet_for_user()` - Generate new HD wallet with BIP44 derivation
- `get_user_wallet()` - Decrypt and retrieve user's wallet
- `sign_transaction()` - Sign blockchain transactions
- `sign_message()` - Sign arbitrary messages
- `check_wallet_balance()` - Query blockchain for wallet balance
- `send_transaction()` - Send signed transactions to blockchain

### 2. Wallet Handlers (`backend/src/handlers/wallet.rs`)

#### Comprehensive API Endpoints:

##### Wallet Information
- `GET /api/wallet/address` - Get wallet address
- `GET /api/wallet/info` - Get comprehensive wallet information
- `GET /api/wallet/balance` - Check wallet balance
- `GET /api/wallet/nonce` - Get transaction count (nonce)

##### Transaction Operations
- `POST /api/wallet/sign-message` - Sign arbitrary messages
- `POST /api/wallet/sign-transaction` - Sign transactions
- `POST /api/wallet/send-transaction` - Send transactions to blockchain
- `POST /api/wallet/estimate-gas` - Estimate gas for transactions
- `GET /api/wallet/gas-price` - Get optimal gas price

##### Wallet Management
- `POST /api/wallet/export` - Export private key for backup
- `POST /api/wallet/import` - Import wallet from private key
- `PUT /api/wallet/rotate-encryption` - Change wallet password
- `POST /api/wallet/generate-from-mnemonic` - Generate wallet from mnemonic
- `POST /api/wallet/derive-addresses` - Derive multiple addresses

##### Mnemonic Operations
- `POST /api/wallet/mnemonic` - Get wallet mnemonic phrase
- `POST /api/wallet/generate-mnemonic` - Generate new mnemonic
- `POST /api/wallet/validate-mnemonic` - Validate mnemonic phrase

##### Utility Functions
- `POST /api/wallet/validate-address` - Validate address format

### 3. Cryptographic Security (`backend/src/utils/crypto.rs`)

#### Enhanced Crypto Functions:
- `encrypt_sensitive_data()` - AES-256-GCM encryption for wallet data
- `decrypt_sensitive_data()` - Secure decryption of wallet data
- `derive_key_from_password()` - PBKDF2 key derivation for encryption
- `generate_wallet_private_key()` - Secure private key generation

## BIP44 Implementation

### Derivation Path Structure
```
m / purpose' / coin_type' / account' / change / address_index
```

### Ethereum Implementation
```
m / 44' / 60' / 0' / 0 / 0  (First Ethereum address)
m / 44' / 60' / 0' / 0 / 1  (Second Ethereum address)
m / 44' / 60' / 1' / 0 / 0  (First address of second account)
```

### Key Components:
- **Purpose**: 44 (BIP44)
- **Coin Type**: 60 (Ethereum)
- **Account**: User-configurable account index
- **Change**: 0 (external addresses)
- **Address Index**: Sequential address generation

## Security Features

### 1. Private Key Protection
- **AES-256-GCM Encryption**: Military-grade encryption for private keys
- **Password-Derived Keys**: PBKDF2 with 10,000 iterations
- **User-Specific Salts**: Unique salt per user for key derivation
- **Secure Storage**: Encrypted keys stored in database

### 2. Mnemonic Security
- **BIP39 Standard**: 12-word mnemonic phrases
- **Encrypted Storage**: Mnemonic phrases encrypted with user password
- **Backup Support**: Secure export for user backup
- **Validation**: Comprehensive mnemonic validation

### 3. Transaction Security
- **Local Signing**: Private keys never leave the server
- **Signature Verification**: Verify signatures against wallet addresses
- **Gas Optimization**: Automatic gas price optimization
- **Nonce Management**: Proper transaction ordering

## API Integration

### Authentication
All wallet endpoints require JWT authentication:
```
Authorization: Bearer <access_token>
```

### Request/Response Examples

#### Get Wallet Address
```bash
GET /api/wallet/address
Authorization: Bearer <token>

Response:
{
  "address": "0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6"
}
```

#### Sign Message
```bash
POST /api/wallet/sign-message
Authorization: Bearer <token>
Content-Type: application/json

{
  "message": "Hello, blockchain!",
  "password": "user_password"
}

Response:
{
  "signature": "0x...",
  "r": "0x...",
  "s": "0x...",
  "v": 27
}
```

#### Send Transaction
```bash
POST /api/wallet/send-transaction
Authorization: Bearer <token>
Content-Type: application/json

{
  "to": "0x742d35Cc6634C0532925a3b8D4C9db96C4b4d8b6",
  "value": "1000000000000000000",
  "password": "user_password",
  "provider_url": "https://polygon-rpc.com",
  "chain_id": 137
}

Response:
{
  "transaction_hash": "0x...",
  "status": "pending"
}
```

#### Export Private Key
```bash
POST /api/wallet/export
Authorization: Bearer <token>
Content-Type: application/json

{
  "password": "user_password"
}

Response:
{
  "private_key": "0x...",
  "warning": "Keep this private key secure and never share it with anyone"
}
```

## Database Integration

### User Model Enhancement
```sql
CREATE TABLE users (
    id UUID PRIMARY KEY,
    username VARCHAR NOT NULL,
    email VARCHAR NOT NULL,
    password_hash VARCHAR NOT NULL,
    internal_wallet_address VARCHAR NOT NULL,
    internal_wallet_private_key_encrypted TEXT NOT NULL,
    internal_wallet_mnemonic_encrypted TEXT,
    -- other fields...
);
```

### Wallet Data Storage:
- **Address**: Plain text wallet address (0x...)
- **Encrypted Private Key**: AES-256-GCM encrypted private key
- **Encrypted Mnemonic**: AES-256-GCM encrypted mnemonic phrase
- **User-Specific Salt**: Derived from user ID for encryption

## Blockchain Integration

### Supported Networks
- **Ethereum Mainnet** (Chain ID: 1)
- **Polygon** (Chain ID: 137)
- **Polygon Mumbai Testnet** (Chain ID: 80001)
- **Other EVM-compatible networks**

### Provider Integration
- **HTTP RPC**: Standard JSON-RPC over HTTP
- **WebSocket**: Real-time blockchain event monitoring
- **Gas Optimization**: Dynamic gas price calculation
- **Transaction Confirmation**: Wait for block confirmations

### Transaction Types
- **ETH/MATIC Transfers**: Native token transfers
- **Smart Contract Interactions**: Contract method calls
- **Token Transfers**: ERC-20 token operations
- **Multi-signature**: Support for multi-sig wallets

## Error Handling

### Common Error Types
- **Invalid Password**: Decryption failures
- **Invalid Address**: Malformed blockchain addresses
- **Insufficient Balance**: Not enough funds for transactions
- **Network Errors**: Blockchain connectivity issues
- **Gas Estimation Failures**: Transaction gas calculation errors

### Error Response Format
```json
{
  "error": "error_code",
  "message": "Human readable error message",
  "details": "Additional error context"
}
```

## Testing

### Comprehensive Test Coverage
- **HD Wallet Generation**: BIP44 compliance testing
- **Encryption/Decryption**: Cryptographic security testing
- **Transaction Signing**: Signature verification testing
- **Address Derivation**: Multi-address generation testing
- **Import/Export**: Wallet backup/restore testing
- **Blockchain Integration**: Network interaction testing

### Test Categories
1. **Unit Tests**: Individual function testing
2. **Integration Tests**: Database and service integration
3. **Security Tests**: Cryptographic security validation
4. **Blockchain Tests**: Network interaction testing
5. **Performance Tests**: Large-scale operation testing

## Performance Considerations

### Optimizations
- **Connection Pooling**: Efficient database connections
- **Caching**: Wallet address and balance caching
- **Async Operations**: Non-blocking blockchain operations
- **Batch Processing**: Multiple address derivation

### Scalability
- **Stateless Operations**: No server-side wallet state
- **Database Scaling**: Encrypted wallet data storage
- **Network Failover**: Multiple RPC provider support
- **Load Balancing**: Horizontal service scaling

## Security Best Practices

### Implementation
- ✅ BIP44 standard HD wallet generation
- ✅ AES-256-GCM encryption for sensitive data
- ✅ PBKDF2 key derivation with high iteration count
- ✅ User-specific encryption salts
- ✅ Secure mnemonic phrase generation and storage
- ✅ Local transaction signing (keys never transmitted)
- ✅ Comprehensive input validation
- ✅ Secure random number generation

### Operational Security
- **Private Key Isolation**: Keys never leave secure environment
- **Audit Logging**: All wallet operations logged
- **Rate Limiting**: Protection against brute force attacks
- **Access Control**: JWT-based authentication required
- **Backup Security**: Encrypted backup procedures

## Production Deployment

### Environment Configuration
```bash
# Blockchain RPC endpoints
ETHEREUM_RPC_URL=https://mainnet.infura.io/v3/your-key
POLYGON_RPC_URL=https://polygon-rpc.com
MUMBAI_RPC_URL=https://rpc-mumbai.maticvigil.com

# Security settings
WALLET_ENCRYPTION_ROUNDS=10000
BACKUP_ENCRYPTION_KEY=your-backup-key
```

### Monitoring and Alerting
- **Transaction Monitoring**: Failed transaction alerts
- **Balance Monitoring**: Low balance notifications
- **Security Monitoring**: Suspicious activity detection
- **Performance Monitoring**: Response time tracking

### Backup and Recovery
- **Database Backups**: Regular encrypted database backups
- **Key Recovery**: Secure key recovery procedures
- **Disaster Recovery**: Multi-region deployment support
- **User Backup**: Mnemonic phrase backup guidance

## Integration with Authentication System

### Automatic Wallet Generation
- Wallets are automatically generated during user registration
- Private keys encrypted with user's password
- Mnemonic phrases stored for backup purposes
- Wallet addresses immediately available after registration

### Password Integration
- Wallet encryption tied to user authentication password
- Password changes require wallet re-encryption
- Secure password validation before wallet operations
- Multi-factor authentication support for sensitive operations

## Future Enhancements

### Planned Features
1. **Multi-Chain Support**: Additional blockchain networks
2. **Hardware Wallet Integration**: Ledger/Trezor support
3. **Multi-Signature Wallets**: Enhanced security for high-value operations
4. **Social Recovery**: Distributed key recovery mechanisms
5. **Gas Optimization**: Advanced gas price prediction
6. **Transaction Batching**: Multiple operations in single transaction

### Scalability Improvements
1. **Redis Caching**: Distributed caching for wallet data
2. **Microservice Architecture**: Separate wallet service
3. **Event-Driven Updates**: Real-time balance updates
4. **Load Balancing**: Geographic distribution of services

## Conclusion

The HD Wallet System provides enterprise-grade blockchain wallet functionality with comprehensive security, BIP44 compliance, and seamless integration with the authentication system. The implementation supports all major wallet operations while maintaining the highest security standards for private key management and transaction signing.

The system is designed for scalability, security, and ease of use, providing users with full control over their blockchain assets while ensuring that private keys remain secure and never leave the platform's secure environment.