# Smart Contract Compilation Hook

## Trigger
- **Event**: File Save
- **File Pattern**: `contracts/**/*.sol`
- **Description**: Automatically compile and test smart contracts when Solidity files are modified

## Actions
1. **Compile Contracts**: Run Hardhat compilation
2. **Generate Types**: Create TypeScript bindings
3. **Run Contract Tests**: Execute smart contract unit tests
4. **Gas Analysis**: Analyze gas usage for contract functions
5. **Security Check**: Run basic security analysis

## Benefits
- Catches compilation errors immediately
- Ensures contract tests pass after changes
- Provides immediate feedback on gas usage
- Maintains type safety between contracts and backend
- Early detection of security issues

## Implementation
```bash
# Navigate to contracts directory
cd contracts

# Compile contracts
npx hardhat compile

# Generate TypeScript bindings
npx hardhat typechain

# Run contract tests
npx hardhat test

# Analyze gas usage
npx hardhat test --gas-reporter

# Run security analysis (if slither is installed)
slither . || echo "Slither not installed, skipping security analysis"
```

## Configuration
- **Auto-fix**: No
- **Show notifications**: On errors and gas changes
- **Run in background**: Yes
- **Timeout**: 120 seconds

## Prerequisites
- Node.js and npm installed
- Hardhat configured
- Contract dependencies installed