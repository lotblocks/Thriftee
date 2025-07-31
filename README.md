# Unit Shopping Platform

A revolutionary e-commerce platform that combines traditional online shopping with gamification through a raffle-based system. The platform creates an engaging, transparent, and fair shopping experience where users purchase "boxes" or "units" of items using credits, with winners selected through verifiable blockchain-based randomness.

## 🚀 Features

- **Interactive Visual Grid**: Progressive image reveal as boxes are purchased
- **Blockchain Transparency**: Verifiable winner selection using Chainlink VRF
- **No-Loss Guarantee**: Credit system ensures users never lose money
- **Multi-Stakeholder Platform**: Supports buyers, sellers, and platform operators
- **Real-time Updates**: WebSocket-based live raffle progress
- **Secure Payments**: Stripe integration for credit purchases
- **Mobile Responsive**: Optimized for all devices

## 🏗️ Architecture

### Backend
- **Language**: Rust
- **Framework**: Actix-web
- **Database**: PostgreSQL
- **Cache**: Redis
- **Blockchain**: Polygon (Ethereum-compatible)

### Frontend
- **Framework**: React with TypeScript
- **Styling**: Tailwind CSS
- **State Management**: React Query
- **UI Components**: Shadcn/UI

### Smart Contracts
- **Language**: Solidity
- **Randomness**: Chainlink VRF
- **Network**: Polygon (Mumbai testnet for development)

## 🛠️ Development Setup

### Prerequisites

- Rust (latest stable)
- Node.js (18+)
- Docker and Docker Compose
- Git

### Quick Start

1. **Clone the repository**
   ```bash
   git clone <your-repo-url>
   cd unit-shopping-platform
   ```

2. **Start the database services**
   ```bash
   docker-compose up -d postgres redis
   ```

3. **Set up environment variables**
   ```bash
   cp .env.example .env
   # Edit .env with your configuration
   ```

4. **Run database migrations**
   ```bash
   cd backend
   cargo install sqlx-cli
   sqlx migrate run
   ```

5. **Start the backend**
   ```bash
   cargo run
   ```

6. **Install frontend dependencies** (when frontend is added)
   ```bash
   cd frontend
   npm install
   npm run dev
   ```

### Environment Variables

Copy `.env.example` to `.env` and configure:

- `DATABASE_URL`: PostgreSQL connection string
- `REDIS_URL`: Redis connection string
- `JWT_SECRET`: Secret key for JWT tokens
- `STRIPE_SECRET_KEY`: Stripe API secret key
- `BLOCKCHAIN_RPC_URL`: Polygon RPC endpoint
- `CONTRACT_ADDRESS`: Deployed smart contract address

## 📁 Project Structure

```
├── backend/                 # Rust backend API
│   ├── src/
│   │   ├── handlers/       # HTTP request handlers
│   │   ├── models/         # Database models
│   │   ├── services/       # Business logic
│   │   ├── middleware/     # Custom middleware
│   │   └── utils/          # Utility functions
│   └── migrations/         # Database migrations
├── contracts/              # Smart contracts
├── shared/                 # Shared types and utilities
├── frontend/               # React frontend (to be added)
└── docs/                   # Documentation
```

## 🧪 Testing

### Backend Tests
```bash
cd backend
cargo test
```

### Integration Tests
```bash
# Start test database
docker-compose -f docker-compose.test.yml up -d
cargo test --test integration
```

## 🚀 Deployment

### Development
```bash
docker-compose up
```

### Production
- Use environment-specific configuration
- Deploy smart contracts to Polygon mainnet
- Set up proper monitoring and logging
- Configure SSL certificates
- Set up CI/CD pipeline

## 📊 Business Model

### Revenue Streams
1. **Seller Subscription Fees**: Monthly recurring revenue
2. **Listing Fees**: Fees for item listings
3. **Transaction Fees**: Percentage of successful sales
4. **Credit Purchase Margins**: Small margin on credit purchases

### Key Metrics
- Monthly Active Users (MAU)
- Raffle completion rate
- Average Revenue Per User (ARPU)
- Credit redemption rate

## 🔒 Security

- JWT-based authentication with refresh tokens
- Encrypted private key storage
- Input validation and sanitization
- Rate limiting and DDoS protection
- Regular security audits

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🆘 Support

For support, email support@unitshoppingplatform.com or join our Discord community.

## 🗺️ Roadmap

- [x] Core infrastructure setup
- [ ] User authentication system
- [ ] Smart contract development
- [ ] Interactive raffle grid
- [ ] Payment integration
- [ ] Mobile app development
- [ ] Advanced analytics
- [ ] Multi-language support