use regex::Regex;
use std::collections::HashSet;
use validator::{ValidationError, ValidationErrors};

use crate::error::AppError;
use raffle_platform_shared::*;

/// Validate email format
pub fn validate_email(email: &str) -> Result<(), ValidationError> {
    let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    
    if email.len() > 254 {
        return Err(ValidationError::new("email_too_long"));
    }
    
    if !email_regex.is_match(email) {
        return Err(ValidationError::new("invalid_email_format"));
    }
    
    Ok(())
}

/// Validate username format
pub fn validate_username(username: &str) -> Result<(), ValidationError> {
    let username_regex = Regex::new(USERNAME_PATTERN).unwrap();
    
    if username.len() < 3 {
        return Err(ValidationError::new("username_too_short"));
    }
    
    if username.len() > 50 {
        return Err(ValidationError::new("username_too_long"));
    }
    
    if !username_regex.is_match(username) {
        return Err(ValidationError::new("invalid_username_format"));
    }
    
    // Check for reserved usernames
    let reserved_usernames = [
        "admin", "administrator", "root", "system", "api", "www", "mail", "ftp",
        "support", "help", "info", "contact", "sales", "marketing", "noreply",
        "postmaster", "webmaster", "hostmaster", "abuse", "security", "privacy",
        "legal", "billing", "accounts", "finance", "hr", "jobs", "careers",
        "news", "press", "media", "blog", "forum", "community", "social",
        "mobile", "app", "apps", "dev", "developer", "test", "testing",
        "stage", "staging", "prod", "production", "beta", "alpha", "demo",
        "null", "undefined", "anonymous", "guest", "user", "users", "member",
        "members", "public", "private", "internal", "external", "official",
    ];
    
    if reserved_usernames.contains(&username.to_lowercase().as_str()) {
        return Err(ValidationError::new("reserved_username"));
    }
    
    Ok(())
}

/// Validate phone number format
pub fn validate_phone_number(phone: &str) -> Result<(), ValidationError> {
    let phone_regex = Regex::new(PHONE_PATTERN).unwrap();
    
    if !phone_regex.is_match(phone) {
        return Err(ValidationError::new("invalid_phone_format"));
    }
    
    Ok(())
}

/// Validate blockchain address format
pub fn validate_blockchain_address(address: &str) -> Result<(), ValidationError> {
    let address_regex = Regex::new(BLOCKCHAIN_ADDRESS_PATTERN).unwrap();
    
    if !address_regex.is_match(address) {
        return Err(ValidationError::new("invalid_blockchain_address"));
    }
    
    Ok(())
}

/// Validate blockchain transaction hash
pub fn validate_tx_hash(tx_hash: &str) -> Result<(), ValidationError> {
    let tx_hash_regex = Regex::new(BLOCKCHAIN_TX_HASH_PATTERN).unwrap();
    
    if !tx_hash_regex.is_match(tx_hash) {
        return Err(ValidationError::new("invalid_transaction_hash"));
    }
    
    Ok(())
}

/// Validate item name
pub fn validate_item_name(name: &str) -> Result<(), ValidationError> {
    if name.trim().is_empty() {
        return Err(ValidationError::new("item_name_empty"));
    }
    
    if name.len() > 255 {
        return Err(ValidationError::new("item_name_too_long"));
    }
    
    // Check for inappropriate content (basic check)
    let inappropriate_words = [
        "spam", "scam", "fake", "counterfeit", "illegal", "stolen",
        "drugs", "weapon", "explosive", "adult", "porn", "sex",
    ];
    
    let name_lower = name.to_lowercase();
    if inappropriate_words.iter().any(|&word| name_lower.contains(word)) {
        return Err(ValidationError::new("inappropriate_content"));
    }
    
    Ok(())
}

/// Validate item description
pub fn validate_item_description(description: &str) -> Result<(), ValidationError> {
    if description.len() > 5000 {
        return Err(ValidationError::new("description_too_long"));
    }
    
    Ok(())
}

/// Validate image URLs
pub fn validate_image_urls(urls: &[String]) -> Result<(), ValidationError> {
    if urls.is_empty() {
        return Err(ValidationError::new("no_images_provided"));
    }
    
    if urls.len() > MAX_IMAGES_PER_ITEM {
        return Err(ValidationError::new("too_many_images"));
    }
    
    let url_regex = Regex::new(r"^https?://[^\s/$.?#].[^\s]*\.(jpg|jpeg|png|webp)(\?[^\s]*)?$").unwrap();
    
    for url in urls {
        if !url_regex.is_match(url) {
            return Err(ValidationError::new("invalid_image_url"));
        }
        
        if url.len() > 2048 {
            return Err(ValidationError::new("image_url_too_long"));
        }
    }
    
    // Check for duplicate URLs
    let unique_urls: HashSet<_> = urls.iter().collect();
    if unique_urls.len() != urls.len() {
        return Err(ValidationError::new("duplicate_image_urls"));
    }
    
    Ok(())
}

/// Validate raffle parameters
pub fn validate_raffle_params(
    total_boxes: i32,
    box_price: rust_decimal::Decimal,
    total_winners: i32,
    grid_rows: i32,
    grid_cols: i32,
) -> Result<(), ValidationError> {
    // Validate total boxes
    if total_boxes < MIN_RAFFLE_BOXES || total_boxes > MAX_RAFFLE_BOXES {
        return Err(ValidationError::new("invalid_total_boxes"));
    }
    
    // Validate box price
    if box_price < MIN_BOX_PRICE || box_price > MAX_BOX_PRICE {
        return Err(ValidationError::new("invalid_box_price"));
    }
    
    // Validate total winners
    if total_winners < 1 || total_winners > total_boxes {
        return Err(ValidationError::new("invalid_total_winners"));
    }
    
    // Validate grid dimensions
    if grid_rows < MIN_GRID_SIZE || grid_rows > MAX_GRID_SIZE {
        return Err(ValidationError::new("invalid_grid_rows"));
    }
    
    if grid_cols < MIN_GRID_SIZE || grid_cols > MAX_GRID_SIZE {
        return Err(ValidationError::new("invalid_grid_cols"));
    }
    
    // Validate that grid can accommodate all boxes
    if grid_rows * grid_cols < total_boxes {
        return Err(ValidationError::new("grid_too_small"));
    }
    
    Ok(())
}

/// Validate credit amount
pub fn validate_credit_amount(amount: rust_decimal::Decimal) -> Result<(), ValidationError> {
    if amount < MIN_CREDIT_AMOUNT {
        return Err(ValidationError::new("credit_amount_too_small"));
    }
    
    if amount > MAX_CREDIT_AMOUNT {
        return Err(ValidationError::new("credit_amount_too_large"));
    }
    
    Ok(())
}

/// Validate pagination parameters
pub fn validate_pagination(limit: Option<i64>, offset: Option<i64>) -> Result<(i64, i64), ValidationError> {
    let limit = limit.unwrap_or(DEFAULT_PAGE_SIZE);
    let offset = offset.unwrap_or(0);
    
    if limit < 1 || limit > MAX_PAGE_SIZE {
        return Err(ValidationError::new("invalid_limit"));
    }
    
    if offset < 0 {
        return Err(ValidationError::new("invalid_offset"));
    }
    
    Ok((limit, offset))
}

/// Convert validation errors to AppError
pub fn validation_errors_to_app_error(errors: ValidationErrors) -> AppError {
    let mut error_messages = Vec::new();
    
    for (field, field_errors) in errors.field_errors() {
        for error in field_errors {
            let message = match error.code.as_ref() {
                "email" => "Invalid email format",
                "length" => "Invalid length",
                "range" => "Value out of range",
                "required" => "Field is required",
                "email_too_long" => "Email address is too long",
                "invalid_email_format" => "Invalid email format",
                "username_too_short" => "Username must be at least 3 characters",
                "username_too_long" => "Username must be less than 50 characters",
                "invalid_username_format" => "Username can only contain letters, numbers, and underscores",
                "reserved_username" => "This username is reserved",
                "invalid_phone_format" => "Invalid phone number format",
                "invalid_blockchain_address" => "Invalid blockchain address format",
                "invalid_transaction_hash" => "Invalid transaction hash format",
                "item_name_empty" => "Item name cannot be empty",
                "item_name_too_long" => "Item name is too long",
                "inappropriate_content" => "Content contains inappropriate words",
                "description_too_long" => "Description is too long",
                "no_images_provided" => "At least one image is required",
                "too_many_images" => "Too many images provided",
                "invalid_image_url" => "Invalid image URL format",
                "image_url_too_long" => "Image URL is too long",
                "duplicate_image_urls" => "Duplicate image URLs are not allowed",
                "invalid_total_boxes" => "Invalid number of total boxes",
                "invalid_box_price" => "Invalid box price",
                "invalid_total_winners" => "Invalid number of winners",
                "invalid_grid_rows" => "Invalid grid rows",
                "invalid_grid_cols" => "Invalid grid columns",
                "grid_too_small" => "Grid is too small for the number of boxes",
                "credit_amount_too_small" => "Credit amount is too small",
                "credit_amount_too_large" => "Credit amount is too large",
                "invalid_limit" => "Invalid pagination limit",
                "invalid_offset" => "Invalid pagination offset",
                _ => "Validation error",
            };
            
            error_messages.push(format!("{}: {}", field, message));
        }
    }
    
    AppError::Validation(error_messages.join(", "))
}

/// Sanitize user input to prevent XSS and other attacks
pub fn sanitize_input(input: &str) -> String {
    // Remove potentially dangerous characters and sequences
    input
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
        .replace('&', "&amp;")
        .replace('\0', "")
        .trim()
        .to_string()
}

/// Validate and sanitize search query
pub fn validate_search_query(query: &str) -> Result<String, ValidationError> {
    let sanitized = sanitize_input(query);
    
    if sanitized.is_empty() {
        return Err(ValidationError::new("empty_search_query"));
    }
    
    if sanitized.len() > 100 {
        return Err(ValidationError::new("search_query_too_long"));
    }
    
    // Remove SQL injection patterns
    let dangerous_patterns = [
        "select", "insert", "update", "delete", "drop", "create", "alter",
        "union", "script", "javascript", "vbscript", "onload", "onerror",
    ];
    
    let query_lower = sanitized.to_lowercase();
    if dangerous_patterns.iter().any(|&pattern| query_lower.contains(pattern)) {
        return Err(ValidationError::new("potentially_dangerous_query"));
    }
    
    Ok(sanitized)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    #[test]
    fn test_email_validation() {
        assert!(validate_email("test@example.com").is_ok());
        assert!(validate_email("user.name+tag@domain.co.uk").is_ok());
        
        assert!(validate_email("invalid-email").is_err());
        assert!(validate_email("@domain.com").is_err());
        assert!(validate_email("user@").is_err());
    }

    #[test]
    fn test_username_validation() {
        assert!(validate_username("validuser123").is_ok());
        assert!(validate_username("user_name").is_ok());
        
        assert!(validate_username("ab").is_err()); // Too short
        assert!(validate_username("admin").is_err()); // Reserved
        assert!(validate_username("user-name").is_err()); // Invalid character
    }

    #[test]
    fn test_phone_validation() {
        assert!(validate_phone_number("+1234567890").is_ok());
        assert!(validate_phone_number("1234567890").is_ok());
        
        assert!(validate_phone_number("123").is_err()); // Too short
        assert!(validate_phone_number("abc123").is_err()); // Invalid characters
    }

    #[test]
    fn test_item_name_validation() {
        assert!(validate_item_name("Valid Item Name").is_ok());
        
        assert!(validate_item_name("").is_err()); // Empty
        assert!(validate_item_name("   ").is_err()); // Only whitespace
        assert!(validate_item_name("Fake Item").is_err()); // Inappropriate content
    }

    #[test]
    fn test_raffle_params_validation() {
        assert!(validate_raffle_params(100, Decimal::from_str("1.00").unwrap(), 1, 10, 10).is_ok());
        
        assert!(validate_raffle_params(0, Decimal::from_str("1.00").unwrap(), 1, 10, 10).is_err()); // Invalid boxes
        assert!(validate_raffle_params(100, Decimal::from_str("0.001").unwrap(), 1, 10, 10).is_err()); // Invalid price
        assert!(validate_raffle_params(100, Decimal::from_str("1.00").unwrap(), 101, 10, 10).is_err()); // Too many winners
        assert!(validate_raffle_params(100, Decimal::from_str("1.00").unwrap(), 1, 5, 5).is_err()); // Grid too small
    }

    #[test]
    fn test_sanitize_input() {
        assert_eq!(sanitize_input("<script>alert('xss')</script>"), "&lt;script&gt;alert(&#x27;xss&#x27;)&lt;/script&gt;");
        assert_eq!(sanitize_input("  normal text  "), "normal text");
        assert_eq!(sanitize_input("text with \"quotes\" & ampersand"), "text with &quot;quotes&quot; &amp; ampersand");
    }

    #[test]
    fn test_search_query_validation() {
        assert!(validate_search_query("normal search").is_ok());
        
        assert!(validate_search_query("").is_err()); // Empty
        assert!(validate_search_query("SELECT * FROM users").is_err()); // SQL injection
        assert!(validate_search_query("<script>alert('xss')</script>").is_ok()); // XSS (sanitized)
    }
}