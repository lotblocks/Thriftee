# GitHub Repository Setup

## Step 1: Create Repository on GitHub

1. Go to https://github.com/new
2. Repository name: `unit-shopping-platform` (or your preferred name)
3. Description: `Revolutionary e-commerce platform with raffle-based shopping and blockchain transparency`
4. Set to **Public** or **Private** (your choice)
5. **DO NOT** initialize with README, .gitignore, or license (we already have these)
6. Click "Create repository"

## Step 2: Link Local Repository to GitHub

After creating the repository on GitHub, run these commands:

```bash
# Add the remote repository (replace YOUR_USERNAME with your GitHub username)
git remote add origin https://github.com/YOUR_USERNAME/unit-shopping-platform.git

# Push the code to GitHub
git branch -M main
git push -u origin main
```

## Step 3: Set Up Branch Protection (Recommended)

1. Go to your repository on GitHub
2. Click "Settings" tab
3. Click "Branches" in the left sidebar
4. Click "Add rule"
5. Branch name pattern: `main`
6. Enable:
   - ✅ Require a pull request before merging
   - ✅ Require status checks to pass before merging
   - ✅ Require branches to be up to date before merging
   - ✅ Include administrators
7. Click "Create"

## Step 4: Set Up GitHub Actions (Optional)

Create `.github/workflows/ci.yml` for automated testing:

```yaml
name: CI

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

env:
  CARGO_TERM_COLOR: always

jobs:
  test:
    runs-on: ubuntu-latest
    
    services:
      postgres:
        image: postgres:15
        env:
          POSTGRES_PASSWORD: postgres
          POSTGRES_DB: raffle_platform_test
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
        ports:
          - 5432:5432
      
      redis:
        image: redis:7-alpine
        options: >-
          --health-cmd "redis-cli ping"
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
        ports:
          - 6379:6379

    steps:
    - uses: actions/checkout@v4
    
    - name: Install Rust
      uses: dtolnay/rust-toolchain@stable
      with:
        components: rustfmt, clippy
    
    - name: Cache cargo registry
      uses: actions/cache@v3
      with:
        path: ~/.cargo/registry
        key: ${{ runner.os }}-cargo-registry-${{ hashFiles('**/Cargo.lock') }}
    
    - name: Cache cargo index
      uses: actions/cache@v3
      with:
        path: ~/.cargo/git
        key: ${{ runner.os }}-cargo-index-${{ hashFiles('**/Cargo.lock') }}
    
    - name: Cache cargo build
      uses: actions/cache@v3
      with:
        path: target
        key: ${{ runner.os }}-cargo-build-target-${{ hashFiles('**/Cargo.lock') }}
    
    - name: Install sqlx-cli
      run: cargo install sqlx-cli --no-default-features --features rustls,postgres
    
    - name: Run migrations
      run: |
        cd backend
        sqlx migrate run
      env:
        DATABASE_URL: postgresql://postgres:postgres@localhost:5432/raffle_platform_test
    
    - name: Check formatting
      run: cargo fmt --all -- --check
    
    - name: Run clippy
      run: cargo clippy --all-targets --all-features -- -D warnings
    
    - name: Run tests
      run: cargo test --verbose
      env:
        DATABASE_URL: postgresql://postgres:postgres@localhost:5432/raffle_platform_test
        REDIS_URL: redis://localhost:6379
        JWT_SECRET: test-secret
```

## Step 5: Repository Settings

### Secrets (for CI/CD)
Go to Settings > Secrets and variables > Actions, and add:
- `DATABASE_URL` (for testing)
- `STRIPE_SECRET_KEY` (test key)
- Other environment variables as needed

### Topics
Add topics to help others discover your repository:
- `rust`
- `blockchain`
- `ecommerce`
- `raffle`
- `web3`
- `actix-web`
- `postgresql`
- `react`
- `typescript`

### About Section
Add a description and website URL if you have one.

## Step 6: Create Issues and Project Board (Optional)

### Create Initial Issues
1. Go to "Issues" tab
2. Click "New issue"
3. Create issues for upcoming tasks:
   - "Implement user authentication system"
   - "Develop smart contracts"
   - "Create interactive raffle grid"
   - "Set up payment integration"

### Set Up Project Board
1. Go to "Projects" tab
2. Click "New project"
3. Choose "Board" template
4. Name it "Unit Shopping Platform Development"
5. Add columns: "Backlog", "In Progress", "Review", "Done"

## Repository Structure

Your repository now includes:

```
├── .kiro/                   # Kiro IDE specifications
├── backend/                 # Rust backend
├── contracts/               # Smart contracts
├── shared/                  # Shared utilities
├── scripts/                 # Setup scripts
├── docs/                    # Documentation
├── README.md               # Project overview
├── DEVELOPMENT.md          # Development guide
├── Cargo.toml              # Workspace configuration
├── docker-compose.yml      # Development services
└── .gitignore              # Git ignore rules
```

## Next Steps

1. Create the GitHub repository
2. Link your local repository
3. Set up branch protection
4. Add GitHub Actions workflow
5. Start implementing the next tasks from your specification

Your project is now ready for collaborative development! 🚀