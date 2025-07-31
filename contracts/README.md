# Raffle Platform Smart Contracts

This directory contains the smart contracts for the Unit Shopping Platform raffle system, built with Solidity and deployed using Hardhat.

## Overview

The smart contract system enables a transparent, fair, and secure raffle platform where users can purchase "boxes" of items using cryptocurrency, with winners selected through verifiable randomness provided by Chainlink VRF.

## Contracts

### RaffleContract.sol

The main contract that handles:
- Raffle creation and management
- Box purchases and participant tracking
- Chainlink VRF integration for random winner selection
- Role-based access control
- Payment processing (ETH and ERC20 tokens)
- Whitelist functionality
- Fee distribution
- Emergency controls

### MockVRFCoordinator.sol

A mock contract for testing VRF functionality in local development environments.

## Features

### Core Functionality
- **Raffle Creation**: Sellers can create raffles with customizable parameters
- **Box Purchases**: Users can buy multiple boxes with participation limits
- **Random Winner Selection**: Uses Chainlink VRF for verifiable randomness
- **Multi-Payment Support**: Accepts ETH and ERC20 tokens
- **Whitelist Support**: Optional whitelist functionality using Merkle proofs

### Security Features
- **Role-Based Access Control**: Multiple roles for different permissions
- **Reentrancy Protection**: Guards against reentrancy attacks
- **Input Validation**: Comprehensive input validation and sanitization
- **Emergency Controls**: Pause functionality and emergency withdrawal
- **Access Controls**: Proper permission checks for all functions

### Advanced Features
- **Fee Management**: Configurable platform and creator fees
- **Refund System**: Automatic refunds for cancelled raffles
- **Event Emission**: Comprehensive event logging for off-chain monitoring
- **Gas Optimization**: Optimized for minimal gas usage
- **Upgrade Safety**: Built with future upgrades in mind

## Network Support

### Testnets
- **Polygon Mumbai**: Primary testnet for development
- **Ethereum Goerli**: Alternative testnet option

### Mainnets
- **Polygon**: Primary production network (low fees, fast transactions)
- **Ethereum**: Alternative production network

## Setup

### Prerequisites
- Node.js (v16 or higher)
- npm or yarn
- Git

### Installation

```bash
# Install dependencies
npm install

# Copy environment template
cp .env.example .env

# Edit .env with your configuration
```

### Environment Variables

Create a `.env` file with the following variables:

```env
# Private key for deployment (without 0x prefix)
PRIVATE_KEY=your_private_key_here

# RPC URLs
MUMBAI_RPC_URL=https://rpc-mumbai.maticvigil.com
POLYGON_RPC_URL=https://polygon-rpc.com
ETHEREUM_RPC_URL=https://mainnet.infura.io/v3/your_infura_project_id

# API Keys for contract verification
POLYGONSCAN_API_KEY=your_polygonscan_api_key
ETHERSCAN_API_KEY=your_etherscan_api_key

# Chainlink VRF Subscription IDs
VRF_SUBSCRIPTION_ID_MUMBAI=your_mumbai_subscription_id
VRF_SUBSCRIPTION_ID_POLYGON=your_polygon_subscription_id
VRF_SUBSCRIPTION_ID_ETHEREUM=your_ethereum_subscription_id

# Optional: Gas reporting
COINMARKETCAP_API_KEY=your_coinmarketcap_api_key
REPORT_GAS=true
```

## Usage

### Compilation

```bash
# Compile contracts
npm run compile

# Check contract sizes
npm run size
```

### Testing

```bash
# Run all tests
npm test

# Run tests with gas reporting
npm run gas-report

# Run coverage analysis
npm run coverage
```

### Deployment

```bash
# Deploy to Mumbai testnet
npm run deploy:mumbai

# Deploy to Polygon mainnet
npm run deploy:polygon

# Deploy to local hardhat network
npm run deploy
```

### Post-Deployment

```bash
# Validate deployment
npm run validate:mumbai <contract_address>

# Setup roles
npm run setup-roles:mumbai <contract_address> testnet

# Verify contract on block explorer
npm run verify:mumbai <contract_address>
```

### Interaction

```bash
# Interact with deployed contract
RAFFLE_CONTRACT_ADDRESS=<contract_address> npm run interact:mumbai
```

## Contract Architecture

### Role System

The contract uses OpenZeppelin's AccessControl for role management:

- **DEFAULT_ADMIN_ROLE**: Full administrative access
- **RAFFLE_MANAGER_ROLE**: Can manage raffles and settings
- **OPERATOR_ROLE**: Can perform operational tasks
- **PAUSER_ROLE**: Can pause/unpause the contract

### State Management

```solidity
enum RaffleStatus {
    Active,      // Raffle is accepting purchases
    Completed,   // Winner selected, raffle finished
    Cancelled    // Raffle cancelled, refunds available
}
```

### Events

The contract emits comprehensive events for off-chain monitoring:

- `RaffleCreated`: New raffle created
- `ParticipationPurchased`: Box purchased
- `RaffleCompleted`: Winner selected
- `RandomnessRequested`: VRF randomness requested
- `RandomnessFulfilled`: VRF randomness received

## Security Considerations

### Access Control
- All administrative functions are protected by role-based access control
- Multi-signature wallet recommended for production admin roles
- Emergency pause functionality for critical situations

### Randomness
- Uses Chainlink VRF for verifiable randomness
- No possibility of manipulation by contract owner or participants
- Proper handling of VRF callback failures

### Financial Security
- Reentrancy guards on all payable functions
- Proper handling of ETH and ERC20 transfers
- Refund mechanisms for cancelled raffles
- Fee distribution with proper accounting

### Input Validation
- Comprehensive validation of all user inputs
- Protection against overflow/underflow attacks
- Proper handling of edge cases

## Gas Optimization

The contract is optimized for gas efficiency:

- Packed structs to minimize storage slots
- Efficient loops and data structures
- Batch operations where possible
- Optimized event emission

## Testing

### Test Coverage

The test suite covers:
- All contract functions
- Edge cases and error conditions
- Role-based access control
- VRF integration (mocked)
- Gas usage optimization
- Security vulnerabilities

### Running Tests

```bash
# Run all tests
npm test

# Run specific test file
npx hardhat test test/RaffleContract.test.js

# Run tests with coverage
npm run coverage
```

## Deployment Guide

### Pre-Deployment Checklist

1. ✅ Environment variables configured
2. ✅ Chainlink VRF subscription created and funded
3. ✅ Deployment wallet funded with native tokens
4. ✅ Network configuration verified
5. ✅ Contract compiled successfully
6. ✅ Tests passing

### Deployment Steps

1. **Deploy Contract**
   ```bash
   npm run deploy:mumbai
   ```

2. **Validate Deployment**
   ```bash
   npm run validate:mumbai <contract_address>
   ```

3. **Setup Roles**
   ```bash
   npm run setup-roles:mumbai <contract_address> testnet
   ```

4. **Verify Contract**
   ```bash
   npm run verify:mumbai <contract_address>
   ```

5. **Test Interaction**
   ```bash
   RAFFLE_CONTRACT_ADDRESS=<contract_address> npm run interact:mumbai
   ```

### Post-Deployment

- Add contract address to backend configuration
- Update frontend contract addresses
- Configure monitoring and alerting
- Document deployment details

## Monitoring

### Events to Monitor

- `RaffleCreated`: Track new raffles
- `ParticipationPurchased`: Monitor box purchases
- `RaffleCompleted`: Track completed raffles
- `RandomnessRequested`: Monitor VRF requests
- `EmergencyWithdraw`: Alert on emergency actions

### Health Checks

- Contract balance (for refunds)
- LINK token balance (for VRF)
- Role assignments
- Pause status

## Troubleshooting

### Common Issues

1. **VRF Request Fails**
   - Check LINK balance
   - Verify subscription ID
   - Confirm gas lane configuration

2. **Transaction Reverts**
   - Check gas limits
   - Verify role permissions
   - Validate input parameters

3. **Deployment Fails**
   - Verify network configuration
   - Check wallet balance
   - Confirm RPC endpoint

### Support

For technical support:
1. Check the troubleshooting section
2. Review contract events and logs
3. Consult Hardhat and Chainlink documentation
4. Contact the development team

## License

MIT License - see LICENSE file for details.