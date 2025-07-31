# Development Guide

## Prerequisites

### Required Software

1. **Rust** (latest stable)
   - Windows: Download from https://rustup.rs/
   - macOS/Linux: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`

2. **Docker Desktop**
   - Download from https://www.docker.com/products/docker-desktop/

3. **Git**
   - Download from https://git-scm.com/

### Optional but Recommended

- **Visual Studio Code** with Rust extensions
- **PostgreSQL client** (psql or pgAdmin)
- **Redis CLI** for debugging

## Setup Instructions

### Windows

1. Run the setup script:
   ```powershell
   .\scripts\setup.ps1
   ```

### macOS/Linux

1. Make the script executable and run:
   ```bash
   chmod +x scripts/setup.sh
   ./scripts/setup.sh
   ```

### Manual Setup

If the scripts don't work, follow these steps:

1. **Install dependencies**:
   ```bash
   cargo install sqlx-cli --no-default-features --features rustls,postgres
   ```

2. **Start services**:
   ```bash
   docker-compose up -d postgres redis
   ```

3. **Run migrations**:
   ```bash
   cd backend
   sqlx migrate run
   cd ..
   ```

4. **Build project**:
   ```bash
   cargo build
   ```

## Development Workflow

### Starting the Development Environment

1. **Start database services**:
   ```bash
   docker-compose up -d
   ```

2. **Run the backend**:
   ```bash
   cargo run
   ```

3. **Access the API**:
   - Health check: http://localhost:8080/api/v1/health

### Database Management

#### Creating Migrations
```bash
cd backend
sqlx migrate add <migration_name>
```

#### Running Migrations
```bash
sqlx migrate run
```

#### Reverting Migrations
```bash
sqlx migrate revert
```

### Testing

#### Unit Tests
```bash
cargo test
```

#### Integration Tests
```bash
# Start test database
docker-compose -f docker-compose.test.yml up -d
cargo test --test integration
```

#### Test Coverage
```bash
cargo install cargo-tarpaulin
cargo tarpaulin --out Html
```

### Code Quality

#### Formatting
```bash
cargo fmt
```

#### Linting
```bash
cargo clippy
```

#### Security Audit
```bash
cargo install cargo-audit
cargo audit
```

## Project Structure

```
├── backend/                 # Rust backend API
│   ├── src/
│   │   ├── config.rs       # Configuration management
│   │   ├── database.rs     # Database connection
│   │   ├── error.rs        # Error handling
│   │   ├── handlers/       # HTTP request handlers
│   │   ├── middleware/     # Custom middleware
│   │   ├── models/         # Database models
│   │   ├── services/       # Business logic
│   │   └── utils/          # Utility functions
│   ├── migrations/         # Database migrations
│   └── tests/              # Integration tests
├── contracts/              # Smart contracts
├── shared/                 # Shared types and utilities
├── frontend/               # React frontend (to be added)
├── scripts/                # Setup and utility scripts
└── docs/                   # Documentation
```

## Environment Configuration

### Development Environment

Copy `.env.example` to `.env` and configure:

```env
# Server
HOST=127.0.0.1
PORT=8080

# Database
DATABASE_URL=postgresql://postgres:postgres@localhost:5432/raffle_platform

# Redis
REDIS_URL=redis://localhost:6379

# JWT
JWT_SECRET=dev-jwt-secret-change-in-production

# Stripe (test keys)
STRIPE_SECRET_KEY=sk_test_...
STRIPE_WEBHOOK_SECRET=whsec_...

# Blockchain (Mumbai testnet)
BLOCKCHAIN_RPC_URL=https://polygon-mumbai.g.alchemy.com/v2/...
BLOCKCHAIN_WS_URL=wss://polygon-mumbai.g.alchemy.com/v2/...
```

### Production Environment

- Use strong, unique secrets
- Configure proper database credentials
- Set up SSL certificates
- Use production blockchain networks
- Enable monitoring and logging

## Debugging

### Database Issues

1. **Check if PostgreSQL is running**:
   ```bash
   docker-compose ps postgres
   ```

2. **Connect to database**:
   ```bash
   docker-compose exec postgres psql -U postgres -d raffle_platform
   ```

3. **View logs**:
   ```bash
   docker-compose logs postgres
   ```

### Backend Issues

1. **Check logs**:
   ```bash
   RUST_LOG=debug cargo run
   ```

2. **Test API endpoints**:
   ```bash
   curl http://localhost:8080/api/v1/health
   ```

### Common Issues

#### "cargo: command not found"
- Install Rust from https://rustup.rs/
- Restart your terminal

#### "docker: command not found"
- Install Docker Desktop
- Make sure Docker is running

#### Database connection errors
- Check if PostgreSQL container is running
- Verify DATABASE_URL in .env file
- Check if port 5432 is available

## Contributing

1. **Create a feature branch**:
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes**

3. **Run tests**:
   ```bash
   cargo test
   cargo clippy
   cargo fmt
   ```

4. **Commit your changes**:
   ```bash
   git add .
   git commit -m "feat: add your feature description"
   ```

5. **Push and create PR**:
   ```bash
   git push origin feature/your-feature-name
   ```

## Performance Monitoring

### Metrics to Track

- API response times
- Database query performance
- Memory usage
- CPU utilization
- Error rates

### Tools

- **Logging**: tracing crate with structured logging
- **Metrics**: Prometheus integration (to be added)
- **Monitoring**: Grafana dashboards (to be added)
- **APM**: Application Performance Monitoring (to be added)

## Security Considerations

### Development

- Never commit secrets to version control
- Use test API keys for development
- Regularly update dependencies
- Run security audits

### Production

- Use environment variables for secrets
- Enable HTTPS/TLS
- Implement rate limiting
- Regular security audits
- Monitor for vulnerabilities