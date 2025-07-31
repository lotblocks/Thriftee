use rust_decimal::Decimal;
use std::time::Duration;

// JWT Configuration
pub const JWT_ACCESS_TOKEN_EXPIRY: Duration = Duration::from_secs(15 * 60); // 15 minutes
pub const JWT_REFRESH_TOKEN_EXPIRY: Duration = Duration::from_secs(7 * 24 * 60 * 60); // 7 days

// Pagination defaults
pub const DEFAULT_PAGE_SIZE: i64 = 20;
pub const MAX_PAGE_SIZE: i64 = 100;

// Credit system
pub const DEFAULT_CREDIT_EXPIRY_DAYS: i64 = 365; // 1 year
pub const MIN_CREDIT_AMOUNT: Decimal = Decimal::from_parts(1, 0, 0, false, 2); // 0.01
pub const MAX_CREDIT_AMOUNT: Decimal = Decimal::from_parts(1000000, 0, 0, false, 2); // 10,000.00

// Raffle constraints
pub const MIN_RAFFLE_BOXES: i32 = 1;
pub const MAX_RAFFLE_BOXES: i32 = 10000;
pub const MIN_BOX_PRICE: Decimal = Decimal::from_parts(1, 0, 0, false, 2); // 0.01
pub const MAX_BOX_PRICE: Decimal = Decimal::from_parts(100000, 0, 0, false, 2); // 1,000.00
pub const MIN_GRID_SIZE: i32 = 1;
pub const MAX_GRID_SIZE: i32 = 100;

// Rate limiting
pub const LOGIN_RATE_LIMIT_PER_MINUTE: u32 = 5;
pub const API_RATE_LIMIT_PER_MINUTE: u32 = 100;
pub const WEBHOOK_RATE_LIMIT_PER_MINUTE: u32 = 1000;

// File upload limits
pub const MAX_IMAGE_SIZE_MB: usize = 10;
pub const MAX_IMAGES_PER_ITEM: usize = 10;
pub const ALLOWED_IMAGE_TYPES: &[&str] = &["image/jpeg", "image/png", "image/webp"];

// Blockchain configuration
pub const BLOCKCHAIN_CONFIRMATION_BLOCKS: u64 = 3;
pub const GAS_PRICE_BUFFER_PERCENTAGE: u64 = 10; // 10% buffer
pub const MAX_GAS_PRICE_GWEI: u64 = 100;

// Session management
pub const MAX_SESSIONS_PER_USER: i32 = 5;
pub const SESSION_CLEANUP_INTERVAL_HOURS: u64 = 24;

// Audit logging
pub const AUDIT_LOG_RETENTION_DAYS: i64 = 2555; // 7 years for compliance
pub const MAX_AUDIT_LOG_METADATA_SIZE: usize = 10240; // 10KB

// Notification settings
pub const EMAIL_RATE_LIMIT_PER_HOUR: u32 = 10;
pub const SMS_RATE_LIMIT_PER_HOUR: u32 = 5;

// Success messages
pub const SUCCESS_USER_CREATED: &str = "User registered successfully";
pub const SUCCESS_LOGIN: &str = "Login successful";
pub const SUCCESS_LOGOUT: &str = "Logout successful";
pub const SUCCESS_PASSWORD_RESET: &str = "Password reset email sent";
pub const SUCCESS_PASSWORD_CHANGED: &str = "Password changed successfully";
pub const SUCCESS_EMAIL_VERIFIED: &str = "Email verified successfully";
pub const SUCCESS_PROFILE_UPDATED: &str = "Profile updated successfully";

// Error messages
pub const ERROR_INVALID_CREDENTIALS: &str = "Invalid email or password";
pub const ERROR_EMAIL_ALREADY_EXISTS: &str = "Email address is already registered";
pub const ERROR_USERNAME_ALREADY_EXISTS: &str = "Username is already taken";
pub const ERROR_USER_NOT_FOUND: &str = "User not found";
pub const ERROR_INVALID_TOKEN: &str = "Invalid or expired token";
pub const ERROR_EMAIL_NOT_VERIFIED: &str = "Email address not verified";
pub const ERROR_ACCOUNT_DISABLED: &str = "Account has been disabled";
pub const ERROR_INSUFFICIENT_PERMISSIONS: &str = "Insufficient permissions";
pub const ERROR_RATE_LIMIT_EXCEEDED: &str = "Rate limit exceeded. Please try again later";
pub const PUSH_NOTIFICATION_RATE_LIMIT_PER_HOUR: u32 = 50;

// Cache TTL (Time To Live)
pub const USER_CACHE_TTL_SECONDS: u64 = 300; // 5 minutes
pub const RAFFLE_CACHE_TTL_SECONDS: u64 = 60; // 1 minute
pub const ITEM_CACHE_TTL_SECONDS: u64 = 600; // 10 minutes

// Database connection pool
pub const DB_MAX_CONNECTIONS: u32 = 20;
pub const DB_MIN_CONNECTIONS: u32 = 5;
pub const DB_CONNECTION_TIMEOUT_SECONDS: u64 = 30;

// WebSocket configuration
pub const WS_HEARTBEAT_INTERVAL_SECONDS: u64 = 30;
pub const WS_CLIENT_TIMEOUT_SECONDS: u64 = 60;
pub const MAX_WS_CONNECTIONS_PER_USER: usize = 3;

// Error messages
pub const ERROR_INVALID_CREDENTIALS: &str = "Invalid email or password";
pub const ERROR_USER_NOT_FOUND: &str = "User not found";
pub const ERROR_EMAIL_ALREADY_EXISTS: &str = "Email already exists";
pub const ERROR_USERNAME_ALREADY_EXISTS: &str = "Username already exists";
pub const ERROR_INSUFFICIENT_CREDITS: &str = "Insufficient credits";
pub const ERROR_RAFFLE_NOT_FOUND: &str = "Raffle not found";
pub const ERROR_RAFFLE_COMPLETED: &str = "Raffle already completed";
pub const ERROR_BOX_ALREADY_PURCHASED: &str = "Box already purchased";
pub const ERROR_INVALID_BOX_NUMBER: &str = "Invalid box number";
pub const ERROR_UNAUTHORIZED: &str = "Unauthorized access";
pub const ERROR_FORBIDDEN: &str = "Access forbidden";
pub const ERROR_RATE_LIMIT_EXCEEDED: &str = "Rate limit exceeded";

// Success messages
pub const SUCCESS_USER_CREATED: &str = "User created successfully";
pub const SUCCESS_LOGIN: &str = "Login successful";
pub const SUCCESS_LOGOUT: &str = "Logout successful";
pub const SUCCESS_PASSWORD_RESET: &str = "Password reset email sent";
pub const SUCCESS_RAFFLE_CREATED: &str = "Raffle created successfully";
pub const SUCCESS_BOX_PURCHASED: &str = "Box purchased successfully";
pub const SUCCESS_CREDITS_REDEEMED: &str = "Credits redeemed successfully";

// Validation patterns
pub const USERNAME_PATTERN: &str = r"^[a-zA-Z0-9_]{3,50}$";
pub const PHONE_PATTERN: &str = r"^\+?[1-9]\d{1,14}$";
pub const BLOCKCHAIN_ADDRESS_PATTERN: &str = r"^0x[a-fA-F0-9]{40}$";
pub const BLOCKCHAIN_TX_HASH_PATTERN: &str = r"^0x[a-fA-F0-9]{64}$";

// Feature flags (for gradual rollout)
pub const FEATURE_SOCIAL_LOGIN: bool = true;
pub const FEATURE_SMS_NOTIFICATIONS: bool = true;
pub const FEATURE_PUSH_NOTIFICATIONS: bool = true;
pub const FEATURE_ANALYTICS_TRACKING: bool = true;
pub const FEATURE_ADVANCED_METRICS: bool = false; // Disabled by default