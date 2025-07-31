# JWT Authentication System Documentation

## Overview

The JWT Authentication System provides comprehensive security features for the Raffle Shopping Platform, including token generation, validation, refresh mechanisms, password hashing, and authentication middleware.

## Core Components

### 1. JWT Service (`backend/src/utils/jwt.rs`)

#### Features:
- **Token Generation**: Creates access and refresh tokens with configurable expiration
- **Token Validation**: Validates tokens with signature verification and expiration checks
- **Token Revocation**: Maintains revoked token list for security
- **Refresh Mechanism**: Secure token refresh with database validation
- **Security Checks**: Additional validation for user status and session validity
- **Anomaly Detection**: Monitors for suspicious token usage patterns

#### Key Methods:
- `create_token_pair()` - Generate access and refresh token pair
- `validate_token()` - Validate token with full security checks
- `refresh_access_token()` - Refresh access token using refresh token
- `revoke_token()` - Revoke specific token by JTI
- `validate_token_with_security_checks()` - Enhanced validation with database checks

#### Token Structure:
```rust
pub struct Claims {
    pub sub: String,        // Subject (user ID)
    pub username: String,   // Username
    pub email: String,      // Email
    pub role: UserRole,     // User role
    pub exp: i64,          // Expiration time
    pub iat: i64,          // Issued at
    pub jti: String,       // JWT ID (for token revocation)
    pub token_type: String, // "access" or "refresh"
}
```

### 2. Authentication Middleware (`backend/src/middleware/auth.rs`)

#### Middleware Types:

##### AuthMiddleware
- Standard authentication for protected routes
- Validates Bearer tokens in Authorization header
- Supports role-based access control
- Returns 401 for missing/invalid tokens

##### OptionalAuthMiddleware
- Optional authentication that doesn't fail if no token provided
- Useful for endpoints that work for both authenticated and anonymous users

##### SecureAuthMiddleware
- Enhanced security with additional database checks
- Validates user account status
- Logs security events
- IP address and user agent tracking

##### RateLimitMiddleware
- Prevents brute force attacks
- Configurable request limits per time window
- IP-based rate limiting

#### Usage Examples:
```rust
// Basic authentication
.wrap(AuthMiddleware::new(jwt_service))

// Role-based authentication
.wrap(AuthMiddleware::new(jwt_service).require_role(UserRole::Admin))

// Enhanced security
.wrap(SecureAuthMiddleware::new(jwt_service, pool))

// Rate limiting
.wrap(RateLimitMiddleware::new(10, 60)) // 10 requests per minute
```

### 3. Password Security (`backend/src/utils/crypto.rs`)

#### Enhanced Password Hashing:
- **Bcrypt with Cost Factor 12**: Secure password hashing with proper salt rounds
- **Password Strength Validation**: Comprehensive validation rules
- **Timing Attack Protection**: Constant-time comparison functions

#### Password Requirements:
- Minimum 8 characters, maximum 128 characters
- Must contain: lowercase, uppercase, digit, special character
- No common weak patterns (password, 123456, etc.)
- No sequential characters (abc, 123)
- No more than 2 consecutive identical characters

#### Additional Crypto Functions:
- `generate_secure_token()` - Cryptographically secure random tokens
- `hash_with_salt()` - Salted hashing for sensitive data
- `encrypt_data()` / `decrypt_data()` - AES-256-GCM encryption
- `derive_key_from_password()` - PBKDF2 key derivation

## Security Features

### 1. Token Security
- **Short-lived Access Tokens**: 15-minute expiration reduces exposure
- **Long-lived Refresh Tokens**: 7-day expiration with secure storage
- **Token Rotation**: New refresh token issued on each refresh
- **Revocation Support**: Immediate token invalidation capability
- **JTI Tracking**: Unique token identifiers for precise revocation

### 2. Session Management
- **Database-backed Sessions**: Refresh tokens stored securely in database
- **Session Validation**: Active session checks during token validation
- **Multi-device Support**: Users can have multiple active sessions
- **Session Revocation**: Individual or bulk session termination

### 3. Enhanced Security Checks
- **User Status Validation**: Checks if user account is active
- **IP Address Tracking**: Logs and monitors client IP addresses
- **User Agent Tracking**: Device fingerprinting for security
- **Anomaly Detection**: Monitors for suspicious usage patterns
- **Rate Limiting**: Prevents brute force and DoS attacks

### 4. Cryptographic Security
- **HMAC-SHA256 Signatures**: Secure token signing
- **Bcrypt Password Hashing**: Industry-standard password protection
- **AES-256-GCM Encryption**: Symmetric encryption for sensitive data
- **PBKDF2 Key Derivation**: Secure key generation from passwords
- **Cryptographically Secure Random**: High-entropy token generation

## Configuration

### Environment Variables
```bash
JWT_SECRET=your-secret-key-at-least-32-characters-long
```

### Token Expiration (in `shared/src/constants.rs`)
```rust
pub const JWT_ACCESS_TOKEN_EXPIRY: Duration = Duration::from_secs(15 * 60); // 15 minutes
pub const JWT_REFRESH_TOKEN_EXPIRY: Duration = Duration::from_secs(7 * 24 * 60 * 60); // 7 days
```

### Bcrypt Cost Factor
```rust
const BCRYPT_COST: u32 = 12; // Configurable in crypto.rs
```

## API Integration

### Authentication Flow
1. **Registration/Login**: User provides credentials
2. **Token Generation**: System creates access/refresh token pair
3. **Token Storage**: Refresh token hash stored in database
4. **API Requests**: Client includes access token in Authorization header
5. **Token Validation**: Middleware validates token and extracts user info
6. **Token Refresh**: Client uses refresh token to get new access token
7. **Logout**: Tokens are revoked and sessions terminated

### Handler Integration
```rust
// Extract authenticated user in handlers
pub async fn protected_handler(
    user: AuthenticatedUser, // Automatically extracted from JWT
) -> Result<HttpResponse, AppError> {
    // Access user.user_id, user.role, etc.
    Ok(HttpResponse::Ok().json(format!("Hello, {}", user.username)))
}

// Manual claims extraction
pub async fn manual_handler(
    claims: Claims, // Direct access to JWT claims
) -> Result<HttpResponse, AppError> {
    let user_id = Uuid::parse_str(&claims.sub)?;
    // Use claims data
    Ok(HttpResponse::Ok().json("Success"))
}
```

## Testing

### Comprehensive Test Coverage
- **Unit Tests**: All JWT functions tested individually
- **Integration Tests**: Full authentication flows
- **Security Tests**: Token revocation, expiration, validation
- **Middleware Tests**: Authentication middleware behavior
- **Crypto Tests**: Password hashing, encryption, key derivation
- **Concurrent Tests**: Multi-threaded token operations

### Test Categories
1. **Token Generation and Validation**
2. **Token Revocation and Cleanup**
3. **Refresh Token Flow**
4. **Security Checks and Anomaly Detection**
5. **Password Strength Validation**
6. **Encryption and Decryption**
7. **Middleware Authentication**
8. **Rate Limiting**

## Performance Considerations

### Optimizations
- **In-Memory Token Revocation**: Fast revocation checks (Redis recommended for production)
- **Connection Pooling**: Efficient database connections
- **Minimal Database Queries**: Optimized validation flow
- **Async Operations**: Non-blocking token operations

### Scalability
- **Stateless Tokens**: JWT tokens contain all necessary information
- **Database Session Storage**: Scalable refresh token management
- **Horizontal Scaling**: No server-side session state
- **Caching Support**: Ready for Redis integration

## Security Best Practices

### Implementation
- ✅ Strong password requirements with validation
- ✅ Secure password hashing with proper cost factor
- ✅ Short-lived access tokens
- ✅ Secure refresh token rotation
- ✅ Token revocation capability
- ✅ Rate limiting on authentication endpoints
- ✅ Input validation and sanitization
- ✅ Constant-time comparisons
- ✅ Cryptographically secure random generation

### Monitoring and Logging
- Security event logging
- Failed authentication tracking
- Suspicious activity detection
- Token usage monitoring
- Rate limit violation alerts

## Production Deployment

### Recommendations
1. **Use Redis for Token Revocation**: Replace in-memory storage
2. **Configure Proper Secrets**: Use strong, unique JWT secrets
3. **Enable Security Logging**: Monitor authentication events
4. **Set Up Rate Limiting**: Protect against brute force attacks
5. **Regular Token Cleanup**: Remove expired revoked tokens
6. **Monitor Performance**: Track authentication latency
7. **Security Audits**: Regular security reviews and penetration testing

### Environment Setup
```bash
# Production environment variables
JWT_SECRET=your-production-secret-key-minimum-32-characters
BCRYPT_COST=12
REDIS_URL=redis://localhost:6379
DATABASE_URL=postgresql://user:pass@host:5432/db
```

## Conclusion

The JWT Authentication System provides enterprise-grade security features with comprehensive token management, password security, and authentication middleware. The system is designed for scalability, security, and ease of use, with extensive testing and documentation to ensure reliable operation in production environments.