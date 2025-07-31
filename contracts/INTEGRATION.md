# Smart Contract Integration Guide

This guide provides comprehensive information for integrating the RaffleContract with the backend services and frontend applications.

## Contract Overview

The RaffleContract is the core smart contract that manages the raffle system on the blockchain. It provides transparent, verifiable, and fair raffle mechanics using Chainlink VRF for random number generation.

### Key Features
- Transparent raffle creation and management
- Secure box purchasing with exact payment validation
- Verifiable random winner selection using Chainlink VRF
- Role-based access control for platform operations
- Real-time event emission for off-chain monitoring
- Emergency pause functionality for security

## Contract Address and ABI

After deployment, the contract address and ABI can be found in:
- **Deployment Info**: `deployments/<network>.json`
- **Contract ABI**: `abi/RaffleContract.json`

### Network Addresses

```javascript
const CONTRACT_ADDRESSES = {
  mumbai: "0x...", // Deployed address on Mumbai testnet
  polygon: "0x...", // Deployed address on Polygon mainnet
  hardhat: "0x...", // Local development address
};
```

## Integration Architecture

### Backend Integration Flow

```
Backend Service → Smart Contract → Blockchain
     ↓                ↓              ↓
Database Updates ← Event Monitor ← Blockchain Events
```

1. **Raffle Creation**: Backend calls `createRaffle()` with item parameters
2. **Box Purchases**: Users interact directly with contract via frontend
3. **Event Monitoring**: Backend monitors blockchain events for state updates
4. **Database Sync**: Backend updates database based on blockchain events

## Core Contract Functions

### Administrative Functions

#### createRaffle(itemId, totalBoxes, boxPrice, totalWinners)
Creates a new raffle with specified parameters.

**Parameters:**
- `itemId` (uint256): Unique identifier for the item
- `totalBoxes` (uint256): Total number of boxes available
- `boxPrice` (uint256): Price per box in wei
- `totalWinners` (uint256): Number of winners to select

**Access:** Authorized callers only
**Events:** Emits `RaffleCreated`

```javascript
// Example usage
const tx = await raffleContract.createRaffle(
  123,                           // itemId
  100,                          // totalBoxes
  ethers.parseEther("0.01"),    // boxPrice (0.01 MATIC)
  1                             // totalWinners
);
```

#### cancelRaffle(raffleId, reason)
Cancels an active raffle.

**Parameters:**
- `raffleId` (uint256): ID of the raffle to cancel
- `reason` (string): Reason for cancellation

**Access:** Operator role only
**Events:** Emits `RaffleCancelled`

### User Functions

#### buyBox(raffleId)
Allows users to purchase a box in an active raffle.

**Parameters:**
- `raffleId` (uint256): ID of the raffle

**Payment:** Must send exact `boxPrice` in transaction value
**Events:** Emits `BoxPurchased`, potentially `RaffleFull` and `RandomnessRequested`

```javascript
// Example usage
const raffle = await raffleContract.getRaffle(raffleId);
const tx = await raffleContract.buyBox(raffleId, {
  value: raffle.boxPrice
});
```

### View Functions

#### getRaffle(raffleId)
Returns complete raffle information.

**Returns:** Raffle struct with all details

#### getWinners(raffleId)
Returns array of winner addresses for completed raffles.

#### getBoxOwners(raffleId)
Returns array of all box owner addresses.

#### getTotalRaffles()
Returns total number of raffles created.

## Event Monitoring

### Key Events to Monitor

#### RaffleCreated
```solidity
event RaffleCreated(
    uint256 indexed raffleId,
    uint256 indexed itemId,
    uint256 totalBoxes,
    uint256 boxPrice,
    uint256 totalWinners,
    address indexed creator
);
```

#### BoxPurchased
```solidity
event BoxPurchased(
    uint256 indexed raffleId,
    address indexed buyer,
    uint256 boxNumber,
    uint256 totalBoxesSold
);
```

#### WinnerSelected
```solidity
event WinnerSelected(
    uint256 indexed raffleId,
    address[] winners,
    uint256 randomWord
);
```

### Event Monitoring Implementation

```javascript
// Using ethers.js
const contract = new ethers.Contract(address, abi, provider);

// Listen for box purchases
contract.on("BoxPurchased", (raffleId, buyer, boxNumber, totalBoxesSold) => {
  console.log(`Box ${boxNumber} purchased in raffle ${raffleId} by ${buyer}`);
  // Update database
  updateRaffleProgress(raffleId, totalBoxesSold);
});

// Listen for winner selection
contract.on("WinnerSelected", (raffleId, winners, randomWord) => {
  console.log(`Winners selected for raffle ${raffleId}:`, winners);
  // Process winners and update database
  processRaffleCompletion(raffleId, winners);
});
```

## Backend Integration

### Rust Integration Example

```rust
use ethers::prelude::*;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct BlockchainClient {
    contract: RaffleContract<SignerMiddleware<Provider<Http>, LocalWallet>>,
    provider: Arc<Provider<Http>>,
}

impl BlockchainClient {
    pub async fn new(
        contract_address: Address,
        private_key: &str,
        rpc_url: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let provider = Provider::<Http>::try_from(rpc_url)?;
        let wallet: LocalWallet = private_key.parse()?;
        let client = SignerMiddleware::new(provider.clone(), wallet);
        
        let contract = RaffleContract::new(contract_address, Arc::new(client));
        
        Ok(Self {
            contract,
            provider: Arc::new(provider),
        })
    }

    pub async fn create_raffle(
        &self,
        item_id: u64,
        total_boxes: u64,
        box_price: U256,
        total_winners: u64,
    ) -> Result<TransactionReceipt, Box<dyn std::error::Error>> {
        let tx = self.contract
            .create_raffle(
                U256::from(item_id),
                U256::from(total_boxes),
                box_price,
                U256::from(total_winners),
            )
            .send()
            .await?;
            
        let receipt = tx.await?;
        Ok(receipt.unwrap())
    }
}
```

### Event Processing Service

```rust
use ethers::prelude::*;
use tokio_stream::StreamExt;

pub struct EventProcessor {
    contract: RaffleContract<Provider<Ws>>,
    db_pool: PgPool,
}

impl EventProcessor {
    pub async fn start_monitoring(&self) -> Result<(), Box<dyn std::error::Error>> {
        let events = self.contract.events().from_block(BlockNumber::Latest);
        let mut stream = events.stream().await?;

        while let Some(event) = stream.next().await {
            match event {
                Ok(RaffleContractEvents::BoxPurchasedFilter(box_purchased)) => {
                    self.handle_box_purchased(box_purchased).await?;
                }
                Ok(RaffleContractEvents::WinnerSelectedFilter(winner_selected)) => {
                    self.handle_winner_selected(winner_selected).await?;
                }
                Err(e) => {
                    log::error!("Event stream error: {}", e);
                }
            }
        }
        
        Ok(())
    }

    async fn handle_box_purchased(
        &self,
        event: BoxPurchasedFilter,
    ) -> Result<(), sqlx::Error> {
        // Update database with box purchase
        sqlx::query!(
            "UPDATE raffles SET boxes_sold = $1 WHERE id = $2",
            event.total_boxes_sold.as_u64() as i64,
            event.raffle_id.as_u64() as i64,
        )
        .execute(&self.db_pool)
        .await?;

        // Record box purchase
        sqlx::query!(
            "INSERT INTO box_purchases (raffle_id, buyer_address, box_number) VALUES ($1, $2, $3)",
            event.raffle_id.as_u64() as i64,
            format!("{:?}", event.buyer),
            event.box_number.as_u64() as i64,
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }
}
```

## Frontend Integration

### Web3 Connection Setup

```javascript
import { ethers } from 'ethers';
import RaffleContractABI from './abi/RaffleContract.json';

class RaffleContractService {
  constructor(contractAddress, provider) {
    this.contractAddress = contractAddress;
    this.provider = provider;
    this.contract = new ethers.Contract(
      contractAddress,
      RaffleContractABI,
      provider
    );
  }

  async connectWallet() {
    if (window.ethereum) {
      await window.ethereum.request({ method: 'eth_requestAccounts' });
      const signer = this.provider.getSigner();
      this.contractWithSigner = this.contract.connect(signer);
      return signer.getAddress();
    }
    throw new Error('No wallet found');
  }

  async buyBox(raffleId) {
    const raffle = await this.contract.getRaffle(raffleId);
    const tx = await this.contractWithSigner.buyBox(raffleId, {
      value: raffle.boxPrice
    });
    return tx.wait();
  }

  async getRaffleDetails(raffleId) {
    const raffle = await this.contract.getRaffle(raffleId);
    const boxOwners = await this.contract.getBoxOwners(raffleId);
    
    return {
      ...raffle,
      boxOwners,
      boxPrice: ethers.formatEther(raffle.boxPrice),
    };
  }

  subscribeToEvents(raffleId, callbacks) {
    const filter = this.contract.filters.BoxPurchased(raffleId);
    this.contract.on(filter, callbacks.onBoxPurchased);

    const winnerFilter = this.contract.filters.WinnerSelected(raffleId);
    this.contract.on(winnerFilter, callbacks.onWinnerSelected);
  }
}
```

### React Hook Example

```javascript
import { useState, useEffect } from 'react';
import { useWeb3 } from './useWeb3';

export function useRaffle(raffleId) {
  const { contract } = useWeb3();
  const [raffle, setRaffle] = useState(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    if (!contract || !raffleId) return;

    const loadRaffle = async () => {
      try {
        const raffleData = await contract.getRaffle(raffleId);
        const boxOwners = await contract.getBoxOwners(raffleId);
        
        setRaffle({
          ...raffleData,
          boxOwners,
          boxPrice: ethers.formatEther(raffleData.boxPrice),
        });
      } catch (error) {
        console.error('Failed to load raffle:', error);
      } finally {
        setLoading(false);
      }
    };

    loadRaffle();

    // Subscribe to events
    const handleBoxPurchased = (rId, buyer, boxNumber, totalSold) => {
      if (rId.toString() === raffleId.toString()) {
        setRaffle(prev => ({
          ...prev,
          boxesSold: totalSold,
          boxOwners: [...prev.boxOwners, buyer]
        }));
      }
    };

    contract.on('BoxPurchased', handleBoxPurchased);

    return () => {
      contract.off('BoxPurchased', handleBoxPurchased);
    };
  }, [contract, raffleId]);

  const buyBox = async () => {
    if (!contract || !raffle) return;
    
    try {
      const tx = await contract.buyBox(raffleId, {
        value: ethers.parseEther(raffle.boxPrice)
      });
      return tx.wait();
    } catch (error) {
      console.error('Failed to buy box:', error);
      throw error;
    }
  };

  return { raffle, loading, buyBox };
}
```

## Security Considerations

### Access Control
- Only authorized backend services can create raffles
- Users can only purchase boxes with exact payment
- Emergency pause functionality for critical issues

### Transaction Security
- All external calls protected with reentrancy guards
- Input validation on all public functions
- Proper error handling and custom error messages

### VRF Security
- Uses Chainlink VRF v2 for verifiable randomness
- Proper subscription management required
- Callback gas limits prevent DoS attacks

## Testing

### Unit Testing
```bash
cd contracts
npm test
```

### Integration Testing
```bash
# Deploy to local network
npm run node
npm run deploy:local

# Run interaction tests
npx hardhat run scripts/interact.js --network localhost
```

### Testnet Testing
```bash
# Deploy to Mumbai testnet
npm run deploy:mumbai

# Validate deployment
npx hardhat run scripts/validate-deployment.js --network mumbai

# Interactive testing
npx hardhat run scripts/interact.js --network mumbai
```

## Deployment Checklist

### Pre-Deployment
- [ ] Configure environment variables
- [ ] Set up Chainlink VRF subscription
- [ ] Fund deployment account with gas tokens
- [ ] Review contract parameters

### Post-Deployment
- [ ] Verify contract on block explorer
- [ ] Set up role permissions
- [ ] Add authorized callers
- [ ] Test basic functionality
- [ ] Configure event monitoring
- [ ] Update frontend contract addresses

### Production Checklist
- [ ] Security audit completed
- [ ] Load testing performed
- [ ] Monitoring and alerting configured
- [ ] Emergency procedures documented
- [ ] Backup and recovery tested

## Troubleshooting

### Common Issues

1. **Transaction Reverts**
   - Check user has sufficient balance
   - Verify exact payment amount
   - Ensure raffle is in correct state

2. **VRF Requests Failing**
   - Check subscription balance
   - Verify contract is added to subscription
   - Confirm network configuration

3. **Event Monitoring Issues**
   - Check WebSocket connection stability
   - Verify event filters are correct
   - Ensure proper error handling

### Support Resources
- Contract source code and tests
- Hardhat documentation
- Chainlink VRF documentation
- Ethers.js documentation

## Maintenance

### Regular Tasks
- Monitor VRF subscription balance
- Update authorized callers as needed
- Review and rotate access keys
- Monitor gas usage and optimize

### Upgrades
- The contract is not upgradeable by design for security
- New versions require full redeployment
- Migration procedures must be planned carefully

## License

MIT License - see LICENSE file for details.