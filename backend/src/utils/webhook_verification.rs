use crate::error::AppError;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::env;

type HmacSha256 = Hmac<Sha256>;

/// Verify Stripe webhook signature
/// This ensures the webhook actually came from Stripe and hasn't been tampered with
pub fn verify_stripe_signature(payload: &[u8], signature: &str) -> Result<(), AppError> {
    let webhook_secret = env::var("STRIPE_WEBHOOK_SECRET")
        .map_err(|_| AppError::Internal("Missing STRIPE_WEBHOOK_SECRET".to_string()))?;

    // Parse the signature header
    let signatures: Vec<&str> = signature.split(',').collect();
    let mut timestamp = None;
    let mut v1_signature = None;

    for sig in signatures {
        if let Some(ts) = sig.strip_prefix("t=") {
            timestamp = Some(ts);
        } else if let Some(v1) = sig.strip_prefix("v1=") {
            v1_signature = Some(v1);
        }
    }

    let timestamp = timestamp
        .ok_or_else(|| AppError::Validation("Missing timestamp in signature".to_string()))?;
    let v1_signature = v1_signature
        .ok_or_else(|| AppError::Validation("Missing v1 signature".to_string()))?;

    // Create the signed payload
    let signed_payload = format!("{}.{}", timestamp, std::str::from_utf8(payload)
        .map_err(|_| AppError::Validation("Invalid UTF-8 in payload".to_string()))?);

    // Compute the expected signature
    let mut mac = HmacSha256::new_from_slice(webhook_secret.as_bytes())
        .map_err(|_| AppError::Internal("Invalid webhook secret".to_string()))?;
    mac.update(signed_payload.as_bytes());
    let expected_signature = hex::encode(mac.finalize().into_bytes());

    // Compare signatures
    if expected_signature != v1_signature {
        return Err(AppError::Authentication("Invalid webhook signature".to_string()));
    }

    // Check timestamp to prevent replay attacks (within 5 minutes)
    let current_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|_| AppError::Internal("System time error".to_string()))?
        .as_secs();

    let webhook_time: u64 = timestamp.parse()
        .map_err(|_| AppError::Validation("Invalid timestamp format".to_string()))?;

    if current_time.saturating_sub(webhook_time) > 300 {
        return Err(AppError::Authentication("Webhook timestamp too old".to_string()));
    }

    Ok(())
}

/// Verify blockchain webhook signature (for services like Alchemy)
pub fn verify_blockchain_signature(payload: &[u8], signature: &str) -> Result<(), AppError> {
    let webhook_secret = env::var("BLOCKCHAIN_WEBHOOK_SECRET")
        .map_err(|_| AppError::Internal("Missing BLOCKCHAIN_WEBHOOK_SECRET".to_string()))?;

    // Remove 'sha256=' prefix if present
    let signature = signature.strip_prefix("sha256=").unwrap_or(signature);

    // Compute the expected signature
    let mut mac = HmacSha256::new_from_slice(webhook_secret.as_bytes())
        .map_err(|_| AppError::Internal("Invalid webhook secret".to_string()))?;
    mac.update(payload);
    let expected_signature = hex::encode(mac.finalize().into_bytes());

    // Compare signatures
    if expected_signature != signature {
        return Err(AppError::Authentication("Invalid blockchain webhook signature".to_string()));
    }

    Ok(())
}

/// Verify notification service webhook signature
pub fn verify_notification_signature(payload: &[u8], signature: &str) -> Result<(), AppError> {
    let webhook_secret = env::var("NOTIFICATION_WEBHOOK_SECRET")
        .map_err(|_| AppError::Internal("Missing NOTIFICATION_WEBHOOK_SECRET".to_string()))?;

    // Compute the expected signature
    let mut mac = HmacSha256::new_from_slice(webhook_secret.as_bytes())
        .map_err(|_| AppError::Internal("Invalid webhook secret".to_string()))?;
    mac.update(payload);
    let expected_signature = hex::encode(mac.finalize().into_bytes());

    // Compare signatures
    if expected_signature != signature {
        return Err(AppError::Authentication("Invalid notification webhook signature".to_string()));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stripe_signature_verification() {
        // Test with a known good signature
        let payload = b"test payload";
        let secret = "test_secret";
        let timestamp = "1234567890";
        
        // This would be a real test with actual Stripe signature format
        // For now, we'll just test that the function doesn't panic
        std::env::set_var("STRIPE_WEBHOOK_SECRET", secret);
        
        let signature = format!("t={},v1=invalid_signature", timestamp);
        let result = verify_stripe_signature(payload, &signature);
        
        // Should fail with invalid signature
        assert!(result.is_err());
    }
}