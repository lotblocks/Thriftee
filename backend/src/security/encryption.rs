use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use argon2::{
    password_hash::{rand_core::OsRng as ArgonOsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use tracing::{error, info};

#[derive(Error, Debug)]
pub enum EncryptionError {
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),
    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),
    #[error("Key derivation failed: {0}")]
    KeyDerivationFailed(String),
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Password hashing failed: {0}")]
    PasswordHashingFailed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedData {
    pub ciphertext: String,
    pub nonce: String,
    pub algorithm: String,
}

pub struct EncryptionService {
    master_key: [u8; 32],
    argon2: Argon2<'static>,
}

impl EncryptionService {
    pub fn new(master_key: &[u8; 32]) -> Self {
        Self {
            master_key: *master_key,
            argon2: Argon2::default(),
        }
    }

    pub fn from_env() -> Result<Self, EncryptionError> {
        let key_str = std::env::var("ENCRYPTION_MASTER_KEY")
            .map_err(|_| EncryptionError::InvalidInput("ENCRYPTION_MASTER_KEY not found".to_string()))?;
        
        let key_bytes = general_purpose::STANDARD
            .decode(&key_str)
            .map_err(|e| EncryptionError::InvalidInput(format!("Invalid master key format: {}", e)))?;
        
        if key_bytes.len() != 32 {
            return Err(EncryptionError::InvalidInput("Master key must be 32 bytes".to_string()));
        }
        
        let mut key = [0u8; 32];
        key.copy_from_slice(&key_bytes);
        
        Ok(Self::new(&key))
    }

    pub fn generate_master_key() -> [u8; 32] {
        Aes256Gcm::generate_key(OsRng).into()
    }

    pub fn encrypt_data(&self, plaintext: &[u8]) -> Result<EncryptedData, EncryptionError> {
        let key = Key::<Aes256Gcm>::from_slice(&self.master_key);
        let cipher = Aes256Gcm::new(key);
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        
        let ciphertext = cipher
            .encrypt(&nonce, plaintext)
            .map_err(|e| EncryptionError::EncryptionFailed(e.to_string()))?;
        
        Ok(EncryptedData {
            ciphertext: general_purpose::STANDARD.encode(&ciphertext),
            nonce: general_purpose::STANDARD.encode(&nonce),
            algorithm: "AES-256-GCM".to_string(),
        })
    }

    pub fn decrypt_data(&self, encrypted_data: &EncryptedData) -> Result<Vec<u8>, EncryptionError> {
        if encrypted_data.algorithm != "AES-256-GCM" {
            return Err(EncryptionError::DecryptionFailed(
                format!("Unsupported algorithm: {}", encrypted_data.algorithm)
            ));
        }

        let key = Key::<Aes256Gcm>::from_slice(&self.master_key);
        let cipher = Aes256Gcm::new(key);
        
        let ciphertext = general_purpose::STANDARD
            .decode(&encrypted_data.ciphertext)
            .map_err(|e| EncryptionError::DecryptionFailed(format!("Invalid ciphertext: {}", e)))?;
        
        let nonce_bytes = general_purpose::STANDARD
            .decode(&encrypted_data.nonce)
            .map_err(|e| EncryptionError::DecryptionFailed(format!("Invalid nonce: {}", e)))?;
        
        if nonce_bytes.len() != 12 {
            return Err(EncryptionError::DecryptionFailed("Invalid nonce length".to_string()));
        }
        
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        let plaintext = cipher
            .decrypt(nonce, ciphertext.as_ref())
            .map_err(|e| EncryptionError::DecryptionFailed(e.to_string()))?;
        
        Ok(plaintext)
    }

    pub fn encrypt_string(&self, plaintext: &str) -> Result<EncryptedData, EncryptionError> {
        self.encrypt_data(plaintext.as_bytes())
    }

    pub fn decrypt_string(&self, encrypted_data: &EncryptedData) -> Result<String, EncryptionError> {
        let plaintext_bytes = self.decrypt_data(encrypted_data)?;
        String::from_utf8(plaintext_bytes)
            .map_err(|e| EncryptionError::DecryptionFailed(format!("Invalid UTF-8: {}", e)))
    }

    pub fn encrypt_json<T>(&self, data: &T) -> Result<EncryptedData, EncryptionError>
    where
        T: Serialize,
    {
        let json_str = serde_json::to_string(data)
            .map_err(|e| EncryptionError::EncryptionFailed(format!("JSON serialization failed: {}", e)))?;
        
        self.encrypt_string(&json_str)
    }

    pub fn decrypt_json<T>(&self, encrypted_data: &EncryptedData) -> Result<T, EncryptionError>
    where
        T: for<'de> Deserialize<'de>,
    {
        let json_str = self.decrypt_string(encrypted_data)?;
        serde_json::from_str(&json_str)
            .map_err(|e| EncryptionError::DecryptionFailed(format!("JSON deserialization failed: {}", e)))
    }

    // Password hashing and verification
    pub fn hash_password(&self, password: &str) -> Result<String, EncryptionError> {
        let salt = SaltString::generate(&mut ArgonOsRng);
        let password_hash = self.argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| EncryptionError::PasswordHashingFailed(e.to_string()))?;
        
        Ok(password_hash.to_string())
    }

    pub fn verify_password(&self, password: &str, hash: &str) -> Result<bool, EncryptionError> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| EncryptionError::PasswordHashingFailed(e.to_string()))?;
        
        match self.argon2.verify_password(password.as_bytes(), &parsed_hash) {
            Ok(()) => Ok(true),
            Err(argon2::password_hash::Error::Password) => Ok(false),
            Err(e) => Err(EncryptionError::PasswordHashingFailed(e.to_string())),
        }
    }

    // Key derivation for different purposes
    pub fn derive_key(&self, purpose: &str, context: &str) -> Result<[u8; 32], EncryptionError> {
        use hkdf::Hkdf;
        use sha2::Sha256;

        let hk = Hkdf::<Sha256>::new(None, &self.master_key);
        let info = format!("{}:{}", purpose, context);
        let mut okm = [0u8; 32];
        
        hk.expand(info.as_bytes(), &mut okm)
            .map_err(|e| EncryptionError::KeyDerivationFailed(e.to_string()))?;
        
        Ok(okm)
    }

    // Encrypt sensitive database fields
    pub fn encrypt_pii(&self, data: &str, user_id: &str) -> Result<EncryptedData, EncryptionError> {
        let derived_key = self.derive_key("pii", user_id)?;
        let temp_service = EncryptionService::new(&derived_key);
        temp_service.encrypt_string(data)
    }

    pub fn decrypt_pii(&self, encrypted_data: &EncryptedData, user_id: &str) -> Result<String, EncryptionError> {
        let derived_key = self.derive_key("pii", user_id)?;
        let temp_service = EncryptionService::new(&derived_key);
        temp_service.decrypt_string(encrypted_data)
    }

    // Encrypt financial data
    pub fn encrypt_financial_data(&self, data: &str, user_id: &str) -> Result<EncryptedData, EncryptionError> {
        let derived_key = self.derive_key("financial", user_id)?;
        let temp_service = EncryptionService::new(&derived_key);
        temp_service.encrypt_string(data)
    }

    pub fn decrypt_financial_data(&self, encrypted_data: &EncryptedData, user_id: &str) -> Result<String, EncryptionError> {
        let derived_key = self.derive_key("financial", user_id)?;
        let temp_service = EncryptionService::new(&derived_key);
        temp_service.decrypt_string(encrypted_data)
    }
}

// Secure data storage for sensitive configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecureConfig {
    encrypted_values: HashMap<String, EncryptedData>,
}

impl SecureConfig {
    pub fn new() -> Self {
        Self {
            encrypted_values: HashMap::new(),
        }
    }

    pub fn set_value(&mut self, key: &str, value: &str, encryption_service: &EncryptionService) -> Result<(), EncryptionError> {
        let encrypted = encryption_service.encrypt_string(value)?;
        self.encrypted_values.insert(key.to_string(), encrypted);
        Ok(())
    }

    pub fn get_value(&self, key: &str, encryption_service: &EncryptionService) -> Result<Option<String>, EncryptionError> {
        if let Some(encrypted_data) = self.encrypted_values.get(key) {
            let decrypted = encryption_service.decrypt_string(encrypted_data)?;
            Ok(Some(decrypted))
        } else {
            Ok(None)
        }
    }

    pub fn remove_value(&mut self, key: &str) -> bool {
        self.encrypted_values.remove(key).is_some()
    }

    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.encrypted_values.keys()
    }
}

// Utility functions for common encryption tasks
pub fn encrypt_database_field(
    value: &str,
    field_type: &str,
    user_id: &str,
    encryption_service: &EncryptionService,
) -> Result<String, EncryptionError> {
    let encrypted_data = match field_type {
        "pii" => encryption_service.encrypt_pii(value, user_id)?,
        "financial" => encryption_service.encrypt_financial_data(value, user_id)?,
        _ => encryption_service.encrypt_string(value)?,
    };
    
    serde_json::to_string(&encrypted_data)
        .map_err(|e| EncryptionError::EncryptionFailed(format!("Serialization failed: {}", e)))
}

pub fn decrypt_database_field(
    encrypted_json: &str,
    field_type: &str,
    user_id: &str,
    encryption_service: &EncryptionService,
) -> Result<String, EncryptionError> {
    let encrypted_data: EncryptedData = serde_json::from_str(encrypted_json)
        .map_err(|e| EncryptionError::DecryptionFailed(format!("Deserialization failed: {}", e)))?;
    
    match field_type {
        "pii" => encryption_service.decrypt_pii(&encrypted_data, user_id),
        "financial" => encryption_service.decrypt_financial_data(&encrypted_data, user_id),
        _ => encryption_service.decrypt_string(&encrypted_data),
    }
}

// Key rotation utilities
pub struct KeyRotationManager {
    current_service: EncryptionService,
    previous_services: Vec<EncryptionService>,
}

impl KeyRotationManager {
    pub fn new(current_key: [u8; 32]) -> Self {
        Self {
            current_service: EncryptionService::new(&current_key),
            previous_services: Vec::new(),
        }
    }

    pub fn rotate_key(&mut self, new_key: [u8; 32]) {
        let old_service = std::mem::replace(
            &mut self.current_service,
            EncryptionService::new(&new_key),
        );
        self.previous_services.push(old_service);
        
        // Keep only the last 3 keys for backward compatibility
        if self.previous_services.len() > 3 {
            self.previous_services.remove(0);
        }
        
        info!("Encryption key rotated successfully");
    }

    pub fn encrypt(&self, data: &str) -> Result<EncryptedData, EncryptionError> {
        self.current_service.encrypt_string(data)
    }

    pub fn decrypt(&self, encrypted_data: &EncryptedData) -> Result<String, EncryptionError> {
        // Try current key first
        if let Ok(result) = self.current_service.decrypt_string(encrypted_data) {
            return Ok(result);
        }

        // Try previous keys
        for service in &self.previous_services {
            if let Ok(result) = service.decrypt_string(encrypted_data) {
                return Ok(result);
            }
        }

        Err(EncryptionError::DecryptionFailed(
            "Unable to decrypt with any available key".to_string()
        ))
    }

    pub fn re_encrypt_with_current_key(&self, encrypted_data: &EncryptedData) -> Result<EncryptedData, EncryptionError> {
        let plaintext = self.decrypt(encrypted_data)?;
        self.encrypt(&plaintext)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_decryption() {
        let key = EncryptionService::generate_master_key();
        let service = EncryptionService::new(&key);
        
        let plaintext = "Hello, World!";
        let encrypted = service.encrypt_string(plaintext).unwrap();
        let decrypted = service.decrypt_string(&encrypted).unwrap();
        
        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_password_hashing() {
        let key = EncryptionService::generate_master_key();
        let service = EncryptionService::new(&key);
        
        let password = "secure_password_123";
        let hash = service.hash_password(password).unwrap();
        
        assert!(service.verify_password(password, &hash).unwrap());
        assert!(!service.verify_password("wrong_password", &hash).unwrap());
    }

    #[test]
    fn test_key_derivation() {
        let key = EncryptionService::generate_master_key();
        let service = EncryptionService::new(&key);
        
        let key1 = service.derive_key("purpose1", "context1").unwrap();
        let key2 = service.derive_key("purpose1", "context2").unwrap();
        let key3 = service.derive_key("purpose2", "context1").unwrap();
        
        assert_ne!(key1, key2);
        assert_ne!(key1, key3);
        assert_ne!(key2, key3);
    }

    #[test]
    fn test_key_rotation() {
        let key1 = EncryptionService::generate_master_key();
        let key2 = EncryptionService::generate_master_key();
        
        let mut manager = KeyRotationManager::new(key1);
        
        let plaintext = "Test data";
        let encrypted = manager.encrypt(plaintext).unwrap();
        
        // Should decrypt with current key
        assert_eq!(manager.decrypt(&encrypted).unwrap(), plaintext);
        
        // Rotate key
        manager.rotate_key(key2);
        
        // Should still decrypt old data with previous key
        assert_eq!(manager.decrypt(&encrypted).unwrap(), plaintext);
        
        // New encryption should use new key
        let new_encrypted = manager.encrypt(plaintext).unwrap();
        assert_eq!(manager.decrypt(&new_encrypted).unwrap(), plaintext);
    }
}