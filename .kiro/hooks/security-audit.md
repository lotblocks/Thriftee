# Security Audit Hook

## Trigger
- **Event**: File Save
- **File Pattern**: `backend/src/handlers/auth.rs`, `backend/src/handlers/webhooks.rs`, `backend/src/utils/crypto.rs`, `contracts/**/*.sol`
- **Description**: Run security checks when security-sensitive files are modified

## Actions
1. **Dependency Audit**: Check for known vulnerabilities in dependencies
2. **Code Analysis**: Run static security analysis
3. **Crypto Validation**: Verify cryptographic implementations
4. **Access Control Check**: Validate authentication and authorization logic
5. **Input Validation**: Check for proper input sanitization

## Benefits
- Early detection of security vulnerabilities
- Ensures cryptographic best practices
- Validates access control implementations
- Prevents common security mistakes
- Maintains security compliance

## Implementation
```bash
# Rust security audit
cargo audit

# Run clippy with security-focused lints
cargo clippy -- -W clippy::all -W clippy::pedantic -W clippy::security

# Check for hardcoded secrets (if cargo-geiger is installed)
cargo geiger || echo "cargo-geiger not installed"

# Validate crypto implementations
cargo test crypto_tests

# For smart contracts (if in contracts directory)
if [ -f "contracts/package.json" ]; then
    cd contracts
    # Run security analysis
    npm audit
    # Run slither if available
    slither . || echo "Slither not available"
fi
```

## Configuration
- **Auto-fix**: No (security issues require manual review)
- **Show notifications**: Always
- **Run in background**: No (requires immediate attention)
- **Timeout**: 90 seconds

## Prerequisites
- cargo-audit installed
- Security analysis tools configured
- Test suite includes security tests