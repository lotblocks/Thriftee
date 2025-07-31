use bcrypt::{hash, verify, DEFAULT_COST};
use rand::Rng;
use sha2::{Digest, Sha256};
use std::fmt::Write;

use crate::error::AppError;

/// Bcrypt cost factor for password hashing
/// Higher values are more secure but slower
/// 12 is a good balance between security and performance
const BCRYPT_COST: u32 = 12;

/// Hash a password using bcrypt with proper salt rounds
pub fn hash_password(password: &str) -> Result<String, AppError> {
    // Validate password strength before hashing
    validate_password_strength(password)?;
    
    hash(password, BCRYPT_COST)
        .map_err(|e| AppError::Internal(format!("Failed to hash password: {}", e)))
}

/// Verify a password against its hash
pub fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    verify(password, hash)
        .map_err(|e| AppError::Internal(format!("Failed to verify password: {}", e)))
}

/// Validate password strength according to security requirements
pub fn validate_password_strength(password: &str) -> Result<(), AppError> {
    if password.len() < 8 {
        return Err(AppError::Validation("Password must be at least 8 characters long".to_string()));
    }

    if password.len() > 128 {
        return Err(AppError::Validation("Password must be no more than 128 characters long".to_string()));
    }

    let has_lowercase = password.chars().any(|c| c.is_lowercase());
    let has_uppercase = password.chars().any(|c| c.is_uppercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    let has_special = password.chars().any(|c| "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c));

    let mut missing_requirements = Vec::new();

    if !has_lowercase {
        missing_requirements.push("lowercase letter");
    }
    if !has_uppercase {
        missing_requirements.push("uppercase letter");
    }
    if !has_digit {
        missing_requirements.push("digit");
    }
    if !has_special {
        missing_requirements.push("special character");
    }

    if !missing_requirements.is_empty() {
        return Err(AppError::Validation(format!(
            "Password must contain at least one: {}",
            missing_requirements.join(", ")
        )));
    }

    // Check for common weak patterns
    let password_lower = password.to_lowercase();
    let weak_patterns = [
        "password", "123456", "qwerty", "abc123", "admin", "user",
        "login", "welcome", "letmein", "monkey", "dragon", "master",
    ];

    for pattern in &weak_patterns {
        if password_lower.contains(pattern) {
            return Err(AppError::Validation(
                "Password contains common weak patterns and is not secure".to_string()
            ));
        }
    }

    // Check for sequential characters
    if has_sequential_chars(&password_lower) {
        return Err(AppError::Validation(
            "Password should not contain sequential characters (e.g., 123, abc)".to_string()
        ));
    }

    // Check for repeated characters
    if has_repeated_chars(password) {
        return Err(AppError::Validation(
            "Password should not contain more than 2 consecutive identical characters".to_string()
        ));
    }

    Ok(())
}

/// Check if password contains sequential characters
fn has_sequential_chars(password: &str) -> bool {
    let chars: Vec<char> = password.chars().collect();
    
    for window in chars.windows(3) {
        if let [a, b, c] = window {
            let a_code = *a as u32;
            let b_code = *b as u32;
            let c_code = *c as u32;
            
            // Check for ascending sequence
            if b_code == a_code + 1 && c_code == b_code + 1 {
                return true;
            }
            
            // Check for descending sequence
            if b_code == a_code - 1 && c_code == b_code - 1 {
                return true;
            }
        }
    }
    
    false
}

/// Check if password has too many repeated characters
fn has_repeated_chars(password: &str) -> bool {
    let chars: Vec<char> = password.chars().collect();
    
    for window in chars.windows(3) {
        if window[0] == window[1] && window[1] == window[2] {
            return true;
        }
    }
    
    false
}

/// Generate a secure random token for password reset, email verification, etc.
pub fn generate_secure_token() -> String {
    let mut rng = rand::thread_rng();
    let bytes: [u8; 32] = rng.gen();
    hex::encode(bytes)
}

/// Hash a token for secure storage (e.g., refresh tokens, reset tokens)
pub fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    let result = hasher.finalize();
    
    let mut hash_string = String::new();
    for byte in result {
        write!(&mut hash_string, "{:02x}", byte).unwrap();
    }
    hash_string
}

/// Generate a cryptographically secure random string of specified length
pub fn generate_random_string(length: usize) -> String {
    use rand::distributions::Alphanumeric;
    use rand::{thread_rng, Rng};

    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

/// Generate a secure session token with high entropy
pub fn generate_session_token() -> String {
    let mut rng = rand::thread_rng();
    let bytes: [u8; 48] = rng.gen(); // 384 bits of entropy
    base64::encode_config(bytes, base64::URL_SAFE_NO_PAD)
}

/// Generate a secure API key
pub fn generate_api_key() -> String {
    let mut rng = rand::thread_rng();
    let bytes: [u8; 32] = rng.gen(); // 256 bits of entropy
    format!("rp_{}", base64::encode_config(bytes, base64::URL_SAFE_NO_PAD))
}

/// Constant-time string comparison to prevent timing attacks
pub fn secure_compare(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let mut result = 0u8;
    for (byte_a, byte_b) in a.bytes().zip(b.bytes()) {
        result |= byte_a ^ byte_b;
    }

    result == 0
}

/// Hash data with salt for secure storage
pub fn hash_with_salt(data: &str, salt: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    hasher.update(salt.as_bytes());
    let result = hasher.finalize();
    
    let mut hash_string = String::new();
    for byte in result {
        write!(&mut hash_string, "{:02x}", byte).unwrap();
    }
    hash_string
}

/// Generate a cryptographically secure salt
pub fn generate_salt() -> String {
    let mut rng = rand::thread_rng();
    let bytes: [u8; 16] = rng.gen();
    hex::encode(bytes)
}

/// Generate a wallet private key (32 bytes)
pub fn generate_wallet_private_key() -> [u8; 32] {
    let mut rng = rand::thread_rng();
    rng.gen()
}

/// Encrypt sensitive data using AES-256-GCM
pub fn encrypt_sensitive_data(data: &str, key: &[u8; 32]) -> Result<String, AppError> {
    use aes_gcm::{
        aead::{Aead, KeyInit, OsRng},
        Aes256Gcm, Nonce
    };
    
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| AppError::Internal(format!("Failed to create cipher: {}", e)))?;
    
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher.encrypt(&nonce, data.as_bytes())
        .map_err(|e| AppError::Internal(format!("Encryption failed: {}", e)))?;
    
    // Combine nonce and ciphertext
    let mut result = nonce.to_vec();
    result.extend_from_slice(&ciphertext);
    
    Ok(base64::encode(result))
}

/// Decrypt sensitive data using AES-256-GCM
pub fn decrypt_sensitive_data(encrypted_data: &str, key: &[u8; 32]) -> Result<String, AppError> {
    use aes_gcm::{
        aead::{Aead, KeyInit},
        Aes256Gcm, Nonce
    };
    
    let encrypted_bytes = base64::decode(encrypted_data)
        .map_err(|e| AppError::Internal(format!("Failed to decode base64: {}", e)))?;
    
    if encrypted_bytes.len() < 12 {
        return Err(AppError::Internal("Encrypted data too short".to_string()));
    }
    
    let (nonce_bytes, ciphertext) = encrypted_bytes.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);
    
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| AppError::Internal(format!("Failed to create cipher: {}", e)))?;
    
    let plaintext = cipher.decrypt(nonce, ciphertext)
        .map_err(|e| AppError::Internal(format!("Decryption failed: {}", e)))?;
    
    String::from_utf8(plaintext)
        .map_err(|e| AppError::Internal(format!("Failed to convert to string: {}", e)))
}

/// Derive key from password using PBKDF2 with custom iterations
pub fn derive_key_from_password_with_iterations(password: &str, salt: &[u8], iterations: u32) -> Result<Vec<u8>, AppError> {
    use pbkdf2::pbkdf2_hmac;
    use sha2::Sha256;
    
    let mut key = vec![0u8; 32]; // 256-bit key
    pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, iterations, &mut key);
    Ok(key)
}

/// Encrypt data using AES-256-GCM
pub fn encrypt_data(data: &[u8], key: &[u8]) -> Result<Vec<u8>, AppError> {
    use aes_gcm::{Aes256Gcm, Key, Nonce, aead::{Aead, NewAead}};
    
    if key.len() != 32 {
        return Err(AppError::Internal("Key must be 32 bytes for AES-256".to_string()));
    }
    
    let key = Key::from_slice(key);
    let cipher = Aes256Gcm::new(key);
    
    // Generate random nonce
    let mut rng = rand::thread_rng();
    let nonce_bytes: [u8; 12] = rng.gen();
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    let ciphertext = cipher.encrypt(nonce, data)
        .map_err(|e| AppError::Internal(format!("Encryption failed: {}", e)))?;
    
    // Prepend nonce to ciphertext
    let mut result = nonce_bytes.to_vec();
    result.extend_from_slice(&ciphertext);
    
    Ok(result)
}

/// Decrypt data using AES-256-GCM
pub fn decrypt_data(encrypted_data: &[u8], key: &[u8]) -> Result<Vec<u8>, AppError> {
    use aes_gcm::{Aes256Gcm, Key, Nonce, aead::{Aead, NewAead}};
    
    if key.len() != 32 {
        return Err(AppError::Internal("Key must be 32 bytes for AES-256".to_string()));
    }
    
    if encrypted_data.len() < 12 {
        return Err(AppError::Internal("Encrypted data too short".to_string()));
    }
    
    let key = Key::from_slice(key);
    let cipher = Aes256Gcm::new(key);
    
    // Extract nonce and ciphertext
    let (nonce_bytes, ciphertext) = encrypted_data.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);
    
    let plaintext = cipher.decrypt(nonce, ciphertext)
        .map_err(|e| AppError::Internal(format!("Decryption failed: {}", e)))?;
    
    Ok(plaintext)
}

/// Generate a secure API key
pub fn generate_api_key() -> String {
    format!("rsp_{}", generate_secure_token())
}

/// Validate password strength
pub fn validate_password_strength(password: &str) -> Result<(), AppError> {
    if password.len() < 8 {
        return Err(AppError::Validation("Password must be at least 8 characters long".to_string()));
    }

    if password.len() > 128 {
        return Err(AppError::Validation("Password must be less than 128 characters long".to_string()));
    }

    let has_lowercase = password.chars().any(|c| c.is_lowercase());
    let has_uppercase = password.chars().any(|c| c.is_uppercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    let has_special = password.chars().any(|c| "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c));

    let strength_score = [has_lowercase, has_uppercase, has_digit, has_special]
        .iter()
        .filter(|&&x| x)
        .count();

    if strength_score < 3 {
        return Err(AppError::Validation(
            "Password must contain at least 3 of the following: lowercase letter, uppercase letter, digit, special character".to_string()
        ));
    }

    // Check for common weak passwords
    let weak_passwords = [
        "password", "123456", "password123", "admin", "qwerty", "letmein",
        "welcome", "monkey", "dragon", "master", "shadow", "123456789",
        "football", "baseball", "superman", "michael", "jordan", "harley",
    ];

    let password_lower = password.to_lowercase();
    if weak_passwords.iter().any(|&weak| password_lower.contains(weak)) {
        return Err(AppError::Validation("Password contains common weak patterns".to_string()));
    }

    Ok(())
}

/// Generate a wallet private key (32 bytes)
pub fn generate_wallet_private_key() -> [u8; 32] {
    let mut rng = rand::thread_rng();
    rng.gen()
}

/// Encrypt sensitive data using AES-256-GCM
pub fn encrypt_sensitive_data(data: &str, key: &[u8; 32]) -> Result<String, AppError> {
    use aes_gcm::{
        aead::{Aead, KeyInit, OsRng},
        Aes256Gcm, Nonce,
    };

    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| AppError::Internal(format!("Failed to create cipher: {}", e)))?;

    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher
        .encrypt(&nonce, data.as_bytes())
        .map_err(|e| AppError::Internal(format!("Encryption failed: {}", e)))?;

    // Combine nonce and ciphertext
    let mut result = nonce.to_vec();
    result.extend_from_slice(&ciphertext);
    
    Ok(base64::encode(result))
}

/// Decrypt sensitive data using AES-256-GCM
pub fn decrypt_sensitive_data(encrypted_data: &str, key: &[u8; 32]) -> Result<String, AppError> {
    use aes_gcm::{
        aead::{Aead, KeyInit},
        Aes256Gcm, Nonce,
    };

    let data = base64::decode(encrypted_data)
        .map_err(|e| AppError::Internal(format!("Failed to decode base64: {}", e)))?;

    if data.len() < 12 {
        return Err(AppError::Internal("Invalid encrypted data length".to_string()));
    }

    let (nonce_bytes, ciphertext) = data.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);

    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| AppError::Internal(format!("Failed to create cipher: {}", e)))?;

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| AppError::Internal(format!("Decryption failed: {}", e)))?;

    String::from_utf8(plaintext)
        .map_err(|e| AppError::Internal(format!("Failed to convert to string: {}", e)))
}

/// Generate encryption key from password and salt (simplified version for wallet encryption)
pub fn derive_key_from_password(password: &str, salt: &[u8]) -> [u8; 32] {
    use pbkdf2::pbkdf2_hmac;
    use sha2::Sha256;
    
    let mut key = [0u8; 32];
    pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, 10000, &mut key);
    key
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hashing() {
        let password = "TestPassword123!";
        let hash = hash_password(password).expect("Failed to hash password");
        
        assert!(verify_password(password, &hash).expect("Failed to verify password"));
        assert!(!verify_password("wrong_password", &hash).expect("Failed to verify password"));
    }

    #[test]
    fn test_password_strength_validation() {
        // Valid passwords
        assert!(validate_password_strength("StrongPass123!").is_ok());
        assert!(validate_password_strength("MySecure@Pass1").is_ok());
        
        // Invalid passwords
        assert!(validate_password_strength("weak").is_err()); // Too short
        assert!(validate_password_strength("password123").is_err()); // Common weak pattern
        assert!(validate_password_strength("alllowercase").is_err()); // Not enough variety
        assert!(validate_password_strength("ALLUPPERCASE").is_err()); // Not enough variety
        assert!(validate_password_strength("NoNumbers!").is_err()); // No digits
        assert!(validate_password_strength("NoSpecial123").is_err()); // No special chars
        assert!(validate_password_strength("Sequential123abc").is_err()); // Sequential chars
        assert!(validate_password_strength("Repeated111!").is_err()); // Repeated chars
    }

    #[test]
    fn test_secure_token_generation() {
        let token1 = generate_secure_token();
        let token2 = generate_secure_token();
        
        assert_ne!(token1, token2);
        assert_eq!(token1.len(), 64); // 32 bytes = 64 hex chars
        
        let session_token = generate_session_token();
        assert!(!session_token.is_empty());
        
        let api_key = generate_api_key();
        assert!(api_key.starts_with("rp_"));
    }

    #[test]
    fn test_secure_compare() {
        assert!(secure_compare("hello", "hello"));
        assert!(!secure_compare("hello", "world"));
        assert!(!secure_compare("hello", "hello2"));
        assert!(!secure_compare("hello2", "hello"));
    }

    #[test]
    fn test_salt_and_hash() {
        let data = "test_data";
        let salt1 = generate_salt();
        let salt2 = generate_salt();
        
        assert_ne!(salt1, salt2);
        
        let hash1 = hash_with_salt(data, &salt1);
        let hash2 = hash_with_salt(data, &salt2);
        
        assert_ne!(hash1, hash2); // Different salts should produce different hashes
        
        let hash1_again = hash_with_salt(data, &salt1);
        assert_eq!(hash1, hash1_again); // Same salt should produce same hash
    }

    #[test]
    fn test_key_derivation() {
        let password = "test_password";
        let salt = b"test_salt_123456";
        let iterations = 10000;
        
        let key1 = derive_key_from_password_with_iterations(password, salt, iterations)
            .expect("Failed to derive key");
        let key2 = derive_key_from_password_with_iterations(password, salt, iterations)
            .expect("Failed to derive key");
        
        assert_eq!(key1, key2); // Same inputs should produce same key
        assert_eq!(key1.len(), 32); // Should be 256 bits
        
        let key3 = derive_key_from_password_with_iterations("different_password", salt, iterations)
            .expect("Failed to derive key");
        assert_ne!(key1, key3); // Different passwords should produce different keys
    }

    #[test]
    fn test_encryption_decryption() {
        let data = b"Hello, World! This is a test message.";
        let password = "encryption_password";
        let salt = b"test_salt_123456";
        
        let key = derive_key_from_password_with_iterations(password, salt, 10000)
            .expect("Failed to derive key");
        
        let encrypted = encrypt_data(data, &key)
            .expect("Failed to encrypt data");
        
        assert_ne!(encrypted, data); // Encrypted data should be different
        assert!(encrypted.len() > data.len()); // Should include nonce and auth tag
        
        let decrypted = decrypt_data(&encrypted, &key)
            .expect("Failed to decrypt data");
        
        assert_eq!(decrypted, data); // Decrypted should match original
    }

    #[test]
    fn test_encryption_with_wrong_key() {
        let data = b"test data";
        let key1 = derive_key_from_password_with_iterations("password1", b"salt", 1000)
            .expect("Failed to derive key1");
        let key2 = derive_key_from_password_with_iterations("password2", b"salt", 1000)
            .expect("Failed to derive key2");
        
        let encrypted = encrypt_data(data, &key1)
            .expect("Failed to encrypt");
        
        let result = decrypt_data(&encrypted, &key2);
        assert!(result.is_err()); // Should fail with wrong key
    }

    #[test]
    fn test_wallet_private_key_generation() {
        let key1 = generate_wallet_private_key();
        let key2 = generate_wallet_private_key();
        
        assert_ne!(key1, key2); // Should generate different keys
        assert_eq!(key1.len(), 32); // Should be 32 bytes
        assert_eq!(key2.len(), 32); // Should be 32 bytes
    }

    #[test]
    fn test_sensitive_data_encryption() {
        let data = "sensitive wallet data";
        let key = generate_wallet_private_key();
        
        let encrypted = encrypt_sensitive_data(data, &key).expect("Failed to encrypt");
        let decrypted = decrypt_sensitive_data(&encrypted, &key).expect("Failed to decrypt");
        
        assert_eq!(data, decrypted);
        assert_ne!(data, encrypted); // Encrypted should be different
    }

    #[test]
    fn test_sensitive_data_encryption_with_wrong_key() {
        let data = "sensitive wallet data";
        let key1 = generate_wallet_private_key();
        let key2 = generate_wallet_private_key();
        
        let encrypted = encrypt_sensitive_data(data, &key1).expect("Failed to encrypt");
        let result = decrypt_sensitive_data(&encrypted, &key2);
        
        assert!(result.is_err()); // Should fail with wrong key
    }

    #[test]
    fn test_token_generation() {
        let token1 = generate_secure_token();
        let token2 = generate_secure_token();
        
        assert_ne!(token1, token2);
        assert_eq!(token1.len(), 64); // 32 bytes = 64 hex chars
    }

    #[test]
    fn test_token_hashing() {
        let token = "test_token_123";
        let hash1 = hash_token(token);
        let hash2 = hash_token(token);
        
        assert_eq!(hash1, hash2); // Same input should produce same hash
        assert_eq!(hash1.len(), 64); // SHA-256 = 64 hex chars
    }

    #[test]
    fn test_password_strength_validation() {
        // Valid passwords
        assert!(validate_password_strength("StrongPass123!").is_ok());
        assert!(validate_password_strength("MySecure@Pass1").is_ok());
        
        // Invalid passwords
        assert!(validate_password_strength("weak").is_err()); // Too short
        assert!(validate_password_strength("password123").is_err()); // Common weak pattern
        assert!(validate_password_strength("alllowercase").is_err()); // Not enough variety
        assert!(validate_password_strength("ALLUPPERCASE").is_err()); // Not enough variety
    }

    #[test]
    fn test_random_string_generation() {
        let str1 = generate_random_string(16);
        let str2 = generate_random_string(16);
        
        assert_ne!(str1, str2);
        assert_eq!(str1.len(), 16);
        assert_eq!(str2.len(), 16);
    }

    #[test]
    fn test_api_key_generation() {
        let key1 = generate_api_key();
        let key2 = generate_api_key();
        
        assert_ne!(key1, key2);
        assert!(key1.starts_with("rsp_"));
        assert!(key2.starts_with("rsp_"));
    }

    #[test]
    fn test_encryption_decryption() {
        let data = "sensitive information";
        let key = generate_wallet_private_key();
        
        let encrypted = encrypt_sensitive_data(data, &key).expect("Failed to encrypt");
        let decrypted = decrypt_sensitive_data(&encrypted, &key).expect("Failed to decrypt");
        
        assert_eq!(data, decrypted);
        assert_ne!(data, encrypted);
    }
}