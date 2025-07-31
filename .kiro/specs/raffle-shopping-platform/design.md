# Design Document

## Overview

The Unit Shopping Platform is a modern web application that revolutionizes e-commerce through a raffle-based shopping experience. The platform combines traditional online shopping with gamification elements, where users purchase "boxes" or "units" of items using credits, and winners are selected through a verifiable blockchain-based random selection process. The system ensures transparency, fairness, and guarantees no financial loss for participants through a comprehensive credit recovery system.

### Core Innovation

The platform's key innovation is the interactive visual grid system where item images are revealed progressively as boxes are purchased, creating an engaging visual experience that builds anticipation and community engagement around each raffle.

## Architecture

### High-Level Architecture

The platform follows a modern full-stack architecture with clear separation of concerns:

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Frontend      │    │    Backend      │    │   Blockchain    │
│   (React SPA)   │◄──►│   (Rust API)    │◄──►│ Smart Contract  │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         │                       ▼                       ▼
         │              ┌─────────────────┐    ┌─────────────────┐
         │              │   PostgreSQL    │    │  Chainlink VRF  │
         │              │    Database     │    │   (Randomness)  │
         └──────────────┤                 │    └─────────────────┘
                        └─────────────────┘
                                 │
                        ┌─────────────────┐
                        │ Payment Gateway │
                        │    (Stripe)     │
                        └─────────────────┘
```

### Technology Stack

**Frontend:**
- React.js with TypeScript for type safety
- Tailwind CSS for responsive styling
- Shadcn/UI for accessible component library
- React Query for efficient data fetching and caching
- WebSocket client for real-time updates

**Backend:**
- Rust with Actix-web framework for high performance
- sqlx for type-safe database interactions
- JWT with refresh tokens for authentication
- WebSocket support for real-time communication
- ethers-rs for blockchain interactions

**Database:**
- PostgreSQL for relational data storage
- Redis for session management and caching

**Blockchain:**
- Polygon network for cost-effective transactions
- Solidity smart contracts for raffle logic
- Chainlink VRF for verifiable randomness

**External Services:**
- Stripe for payment processing
- AWS S3 for image storage
- CloudFront CDN for global content delivery

## Components and Interfaces

### Frontend Components

#### 1. Authentication System
- **LoginForm**: Supports email/password, OAuth (Google, Apple), and SMS OTP
- **RegistrationForm**: Multi-step registration with email verification
- **ProfileManager**: User profile editing and account settings

#### 2. Interactive Raffle Grid
- **RaffleGrid**: Main component displaying the interactive image grid
- **GridCell**: Individual cell component representing a purchasable box
- **ImageReveal**: Handles progressive image revelation as boxes are purchased
- **BoxSelector**: Allows users to select and purchase multiple boxes

#### 3. Real-time Updates
- **WebSocketProvider**: Context provider for WebSocket connections
- **RealTimeUpdater**: Component that listens for and applies real-time updates
- **NotificationSystem**: Displays real-time notifications and alerts

#### 4. User Dashboard
- **CreditManager**: Displays credit balance, purchase history, and expiration alerts
- **ParticipationHistory**: Shows past and active raffle participation
- **WinningsTracker**: Displays won items and redemption options

#### 5. Seller Dashboard
- **ItemListing**: Form for creating new raffle items
- **InventoryManager**: Tracks stock levels and item status
- **AnalyticsDashboard**: Revenue tracking and performance metrics
- **SubscriptionManager**: Handles seller subscription tiers and payments

### Backend Services

#### 1. User Service
```rust
pub struct UserService {
    db: Arc<PgPool>,
    jwt_service: Arc<JwtService>,
}

impl UserService {
    pub async fn register_user(&self, user_data: CreateUserRequest) -> Result<User>;
    pub async fn authenticate(&self, credentials: LoginRequest) -> Result<AuthResponse>;
    pub async fn generate_internal_wallet(&self, user_id: Uuid) -> Result<WalletAddress>;
    pub async fn update_credit_balance(&self, user_id: Uuid, amount: Decimal) -> Result<()>;
}
```

#### 2. Raffle Service
```rust
pub struct RaffleService {
    db: Arc<PgPool>,
    blockchain_client: Arc<BlockchainClient>,
    websocket_broadcaster: Arc<WebSocketBroadcaster>,
}

impl RaffleService {
    pub async fn create_raffle(&self, item_id: Uuid, params: RaffleParams) -> Result<Raffle>;
    pub async fn purchase_box(&self, raffle_id: Uuid, user_id: Uuid, box_number: u32) -> Result<()>;
    pub async fn process_winner_selection(&self, raffle_id: Uuid) -> Result<Vec<Uuid>>;
    pub async fn handle_blockchain_event(&self, event: BlockchainEvent) -> Result<()>;
}
```

#### 3. Credit Service
```rust
pub struct CreditService {
    db: Arc<PgPool>,
    notification_service: Arc<NotificationService>,
}

impl CreditService {
    pub async fn issue_credits(&self, user_id: Uuid, amount: Decimal, credit_type: CreditType) -> Result<()>;
    pub async fn redeem_credits(&self, user_id: Uuid, item_id: Uuid, amount: Decimal) -> Result<()>;
    pub async fn process_expiring_credits(&self) -> Result<()>;
    pub async fn offer_free_item_redemption(&self, user_id: Uuid) -> Result<Vec<Item>>;
}
```

#### 4. Payment Service
```rust
pub struct PaymentService {
    stripe_client: Arc<StripeClient>,
    db: Arc<PgPool>,
}

impl PaymentService {
    pub async fn create_payment_intent(&self, amount: Decimal, user_id: Uuid) -> Result<PaymentIntent>;
    pub async fn process_webhook(&self, webhook_data: StripeWebhook) -> Result<()>;
    pub async fn process_seller_payout(&self, seller_id: Uuid, amount: Decimal) -> Result<()>;
}
```

### Blockchain Smart Contract Interface

#### Raffle Contract
```solidity
contract RaffleContract {
    struct Raffle {
        uint256 itemId;
        uint256 totalBoxes;
        uint256 boxPrice;
        uint256 boxesSold;
        uint256 totalWinners;
        address[] winnerAddresses;
        RaffleStatus status;
        uint256 requestId;
    }
    
    mapping(uint256 => Raffle) public raffles;
    mapping(uint256 => address[]) public boxOwners;
    
    function createRaffle(uint256 _itemId, uint256 _totalBoxes, uint256 _boxPrice, uint256 _totalWinners) external;
    function buyBox(uint256 _raffleId) external;
    function fulfillRandomWords(uint256 _requestId, uint256[] memory _randomWords) internal override;
    function getWinners(uint256 _raffleId) external view returns (address[] memory);
}
```

## Data Models

### Core Entities

#### User Model
```sql
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR(50) UNIQUE NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255),
    role user_role NOT NULL DEFAULT 'user',
    credit_balance DECIMAL(10,2) DEFAULT 0.00,
    internal_wallet_address VARCHAR(42) UNIQUE NOT NULL,
    internal_wallet_private_key_encrypted TEXT NOT NULL,
    phone_number VARCHAR(20),
    google_id VARCHAR(255),
    apple_id VARCHAR(255),
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);
```

#### Item Model
```sql
CREATE TABLE items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    seller_id UUID REFERENCES users(id),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    images TEXT[] NOT NULL,
    retail_price DECIMAL(10,2) NOT NULL,
    cost_of_goods DECIMAL(10,2) NOT NULL,
    status item_status DEFAULT 'available',
    stock_quantity INTEGER DEFAULT 1,
    listing_fee_applied DECIMAL(10,2),
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);
```

#### Raffle Model
```sql
CREATE TABLE raffles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    item_id UUID REFERENCES items(id) NOT NULL,
    total_boxes INTEGER NOT NULL,
    box_price DECIMAL(10,2) NOT NULL,
    boxes_sold INTEGER DEFAULT 0,
    total_winners INTEGER NOT NULL,
    status raffle_status DEFAULT 'open',
    winner_user_ids UUID[],
    blockchain_tx_hash VARCHAR(66),
    grid_rows INTEGER NOT NULL,
    grid_cols INTEGER NOT NULL,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);
```

#### Credit Model
```sql
CREATE TABLE user_credits (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) NOT NULL,
    amount DECIMAL(10,2) NOT NULL,
    source credit_source NOT NULL,
    credit_type credit_type DEFAULT 'general',
    redeemable_on_item_id UUID REFERENCES items(id),
    expires_at TIMESTAMP,
    is_transferable BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT NOW()
);
```

### Relationships and Constraints

- Users can have multiple credit entries with different types and expiration dates
- Each raffle is linked to exactly one item
- Box purchases create a many-to-many relationship between users and raffles
- Sellers have subscription tiers that determine their fee structure
- Transactions track all financial activities across the platform

## Error Handling

### Frontend Error Handling

#### Network Errors
- Implement retry logic with exponential backoff
- Display user-friendly error messages
- Maintain offline state indicators
- Cache critical data for offline access

#### Validation Errors
- Real-time form validation with clear error messages
- Server-side validation error display
- Input sanitization and type checking

#### WebSocket Connection Errors
- Automatic reconnection with connection status indicators
- Graceful degradation when real-time features are unavailable
- Fallback to polling for critical updates

### Backend Error Handling

#### Database Errors
```rust
#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(#[from] sqlx::Error),
    #[error("Transaction failed: {0}")]
    TransactionFailed(String),
    #[error("Constraint violation: {0}")]
    ConstraintViolation(String),
}
```

#### Blockchain Errors
```rust
#[derive(Debug, thiserror::Error)]
pub enum BlockchainError {
    #[error("Transaction failed: {0}")]
    TransactionFailed(String),
    #[error("Insufficient gas: {0}")]
    InsufficientGas(String),
    #[error("Contract call failed: {0}")]
    ContractCallFailed(String),
}
```

#### Payment Processing Errors
- Stripe webhook signature verification
- Payment failure handling with user notification
- Refund processing for failed transactions
- Fraud detection and prevention

### Smart Contract Error Handling

#### Raffle State Validation
```solidity
modifier validRaffleState(uint256 _raffleId, RaffleStatus _expectedStatus) {
    require(raffles[_raffleId].status == _expectedStatus, "Invalid raffle state");
    _;
}

modifier sufficientBoxes(uint256 _raffleId) {
    require(raffles[_raffleId].boxesSold < raffles[_raffleId].totalBoxes, "Raffle is full");
    _;
}
```

#### Access Control
```solidity
modifier onlyAuthorized() {
    require(authorizedCallers[msg.sender], "Unauthorized caller");
    _;
}
```

## Testing Strategy

### Frontend Testing

#### Unit Testing
- Component testing with React Testing Library
- Hook testing for custom React hooks
- Utility function testing with Jest
- Mock WebSocket connections for real-time features

#### Integration Testing
- API integration testing with MSW (Mock Service Worker)
- End-to-end user flows with Cypress
- Cross-browser compatibility testing
- Mobile responsiveness testing

#### Visual Testing
- Screenshot testing for UI consistency
- Interactive grid functionality testing
- Animation and transition testing

### Backend Testing

#### Unit Testing
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_create_raffle() {
        let service = setup_test_service().await;
        let raffle = service.create_raffle(item_id, raffle_params).await.unwrap();
        assert_eq!(raffle.status, RaffleStatus::Open);
    }
    
    #[tokio::test]
    async fn test_purchase_box() {
        let service = setup_test_service().await;
        let result = service.purchase_box(raffle_id, user_id, box_number).await;
        assert!(result.is_ok());
    }
}
```

#### Integration Testing
- Database integration testing with test containers
- Blockchain integration testing with local test network
- Payment gateway testing with Stripe test mode
- WebSocket communication testing

#### Load Testing
- Concurrent user simulation for raffle participation
- Database performance testing under load
- WebSocket connection scaling tests
- Payment processing throughput testing

### Smart Contract Testing

#### Unit Testing
```javascript
describe("RaffleContract", function() {
    it("Should create a raffle correctly", async function() {
        await raffleContract.createRaffle(itemId, totalBoxes, boxPrice, totalWinners);
        const raffle = await raffleContract.raffles(0);
        expect(raffle.totalBoxes).to.equal(totalBoxes);
    });
    
    it("Should select winners fairly", async function() {
        // Fill all boxes
        for (let i = 0; i < totalBoxes; i++) {
            await raffleContract.connect(users[i]).buyBox(raffleId);
        }
        // Verify winner selection
        const winners = await raffleContract.getWinners(raffleId);
        expect(winners.length).to.equal(totalWinners);
    });
});
```

#### Security Testing
- Reentrancy attack testing
- Access control verification
- Gas optimization testing
- Randomness manipulation resistance testing

### Performance Testing

#### Frontend Performance
- Bundle size optimization
- Image loading optimization
- Real-time update performance
- Mobile device performance testing

#### Backend Performance
- API response time benchmarking
- Database query optimization
- Concurrent request handling
- Memory usage profiling

#### Blockchain Performance
- Gas cost optimization
- Transaction throughput testing
- Network congestion handling
- Smart contract execution efficiency

This comprehensive design document provides the foundation for implementing the Unit Shopping Platform with all the features and requirements specified. The architecture ensures scalability, security, and maintainability while delivering an engaging user experience through the innovative interactive raffle system.