use actix_web::{HttpRequest, HttpResponse, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use tracing::warn;

#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Invalid input: {field} - {message}")]
    InvalidInput { field: String, message: String },
    #[error("Malicious content detected in field: {field}")]
    MaliciousContent { field: String },
    #[error("Input too long: {field} exceeds maximum length of {max_length}")]
    TooLong { field: String, max_length: usize },
    #[error("Required field missing: {field}")]
    Required { field: String },
    #[error("Invalid format: {field} - {expected_format}")]
    InvalidFormat { field: String, expected_format: String },
}

#[derive(Debug, Clone)]
pub struct ValidationRule {
    pub required: bool,
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub pattern: Option<Regex>,
    pub allowed_chars: Option<String>,
    pub sanitize: bool,
    pub check_xss: bool,
    pub check_sql_injection: bool,
    pub custom_validator: Option<fn(&str) -> Result<(), ValidationError>>,
}

impl Default for ValidationRule {
    fn default() -> Self {
        Self {
            required: false,
            min_length: None,
            max_length: Some(1000), // Default max length
            pattern: None,
            allowed_chars: None,
            sanitize: true,
            check_xss: true,
            check_sql_injection: true,
            custom_validator: None,
        }
    }
}

pub struct InputValidator {
    xss_patterns: Vec<Regex>,
    sql_injection_patterns: Vec<Regex>,
    malicious_patterns: Vec<Regex>,
}

impl InputValidator {
    pub fn new() -> Self {
        let xss_patterns = vec![
            Regex::new(r"(?i)<script[^>]*>.*?</script>").unwrap(),
            Regex::new(r"(?i)<iframe[^>]*>.*?</iframe>").unwrap(),
            Regex::new(r"(?i)<object[^>]*>.*?</object>").unwrap(),
            Regex::new(r"(?i)<embed[^>]*>.*?</embed>").unwrap(),
            Regex::new(r"(?i)<link[^>]*>").unwrap(),
            Regex::new(r"(?i)<meta[^>]*>").unwrap(),
            Regex::new(r"(?i)javascript:").unwrap(),
            Regex::new(r"(?i)vbscript:").unwrap(),
            Regex::new(r"(?i)data:text/html").unwrap(),
            Regex::new(r"(?i)on\w+\s*=").unwrap(), // Event handlers
            Regex::new(r"(?i)expression\s*\(").unwrap(),
            Regex::new(r"(?i)url\s*\(\s*javascript:").unwrap(),
        ];

        let sql_injection_patterns = vec![
            Regex::new(r"(?i)(\bor\b|\band\b)\s+\d+\s*=\s*\d+").unwrap(),
            Regex::new(r"(?i)\bunion\b.*\bselect\b").unwrap(),
            Regex::new(r"(?i)\bselect\b.*\bfrom\b").unwrap(),
            Regex::new(r"(?i)\binsert\b.*\binto\b").unwrap(),
            Regex::new(r"(?i)\bupdate\b.*\bset\b").unwrap(),
            Regex::new(r"(?i)\bdelete\b.*\bfrom\b").unwrap(),
            Regex::new(r"(?i)\bdrop\b.*\btable\b").unwrap(),
            Regex::new(r"(?i)\bcreate\b.*\btable\b").unwrap(),
            Regex::new(r"(?i)\balter\b.*\btable\b").unwrap(),
            Regex::new(r"(?i)--").unwrap(), // SQL comments
            Regex::new(r"/\*.*\*/").unwrap(), // SQL block comments
            Regex::new(r"(?i)\bexec\b|\bexecute\b").unwrap(),
            Regex::new(r"(?i)\bsp_\w+").unwrap(), // Stored procedures
            Regex::new(r"(?i)\bxp_\w+").unwrap(), // Extended procedures
        ];

        let malicious_patterns = vec![
            Regex::new(r"(?i)\.\.[\\/]").unwrap(), // Path traversal
            Regex::new(r"(?i)%2e%2e[\\/]").unwrap(), // URL encoded path traversal
            Regex::new(r"(?i)\\x[0-9a-f]{2}").unwrap(), // Hex encoding
            Regex::new(r"(?i)%[0-9a-f]{2}").unwrap(), // URL encoding suspicious
            Regex::new(r"(?i)<\?php").unwrap(), // PHP code
            Regex::new(r"(?i)<%.*%>").unwrap(), // ASP code
            Regex::new(r"(?i)\$\{.*\}").unwrap(), // Template injection
            Regex::new(r"(?i){{.*}}").unwrap(), // Template injection
            Regex::new(r"(?i)eval\s*\(").unwrap(), // Code evaluation
            Regex::new(r"(?i)system\s*\(").unwrap(), // System calls
            Regex::new(r"(?i)exec\s*\(").unwrap(), // Code execution
            Regex::new(r"(?i)passthru\s*\(").unwrap(), // Command execution
            Regex::new(r"(?i)shell_exec\s*\(").unwrap(), // Shell execution
        ];

        Self {
            xss_patterns,
            sql_injection_patterns,
            malicious_patterns,
        }
    }

    pub fn validate_field(&self, field_name: &str, value: &str, rule: &ValidationRule) -> Result<String, ValidationError> {
        // Check if required
        if rule.required && value.trim().is_empty() {
            return Err(ValidationError::Required {
                field: field_name.to_string(),
            });
        }

        // Skip validation for empty optional fields
        if !rule.required && value.trim().is_empty() {
            return Ok(String::new());
        }

        let mut sanitized_value = value.to_string();

        // Length validation
        if let Some(min_len) = rule.min_length {
            if value.len() < min_len {
                return Err(ValidationError::InvalidInput {
                    field: field_name.to_string(),
                    message: format!("Minimum length is {}", min_len),
                });
            }
        }

        if let Some(max_len) = rule.max_length {
            if value.len() > max_len {
                return Err(ValidationError::TooLong {
                    field: field_name.to_string(),
                    max_length: max_len,
                });
            }
        }

        // Pattern validation
        if let Some(ref pattern) = rule.pattern {
            if !pattern.is_match(value) {
                return Err(ValidationError::InvalidFormat {
                    field: field_name.to_string(),
                    expected_format: "Pattern does not match".to_string(),
                });
            }
        }

        // Character whitelist validation
        if let Some(ref allowed_chars) = rule.allowed_chars {
            for ch in value.chars() {
                if !allowed_chars.contains(ch) {
                    return Err(ValidationError::InvalidInput {
                        field: field_name.to_string(),
                        message: format!("Character '{}' is not allowed", ch),
                    });
                }
            }
        }

        // Security checks
        if rule.check_xss && self.contains_xss(value) {
            warn!("XSS attempt detected in field: {}", field_name);
            return Err(ValidationError::MaliciousContent {
                field: field_name.to_string(),
            });
        }

        if rule.check_sql_injection && self.contains_sql_injection(value) {
            warn!("SQL injection attempt detected in field: {}", field_name);
            return Err(ValidationError::MaliciousContent {
                field: field_name.to_string(),
            });
        }

        if self.contains_malicious_content(value) {
            warn!("Malicious content detected in field: {}", field_name);
            return Err(ValidationError::MaliciousContent {
                field: field_name.to_string(),
            });
        }

        // Sanitization
        if rule.sanitize {
            sanitized_value = self.sanitize_input(&sanitized_value);
        }

        // Custom validation
        if let Some(validator) = rule.custom_validator {
            validator(&sanitized_value)?;
        }

        Ok(sanitized_value)
    }

    fn contains_xss(&self, input: &str) -> bool {
        self.xss_patterns.iter().any(|pattern| pattern.is_match(input))
    }

    fn contains_sql_injection(&self, input: &str) -> bool {
        self.sql_injection_patterns.iter().any(|pattern| pattern.is_match(input))
    }

    fn contains_malicious_content(&self, input: &str) -> bool {
        self.malicious_patterns.iter().any(|pattern| pattern.is_match(input))
    }

    fn sanitize_input(&self, input: &str) -> String {
        let mut sanitized = input.to_string();
        
        // Remove null bytes
        sanitized = sanitized.replace('\0', "");
        
        // Normalize whitespace
        sanitized = sanitized.trim().to_string();
        
        // Remove control characters except newlines and tabs
        sanitized = sanitized.chars()
            .filter(|&c| c == '\n' || c == '\t' || !c.is_control())
            .collect();
        
        // HTML entity encoding for basic characters
        sanitized = sanitized
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#x27;");
        
        sanitized
    }

    pub fn validate_json<T>(&self, json_str: &str, max_size: usize) -> Result<T, ValidationError>
    where
        T: for<'de> Deserialize<'de>,
    {
        // Check JSON size
        if json_str.len() > max_size {
            return Err(ValidationError::TooLong {
                field: "json_payload".to_string(),
                max_length: max_size,
            });
        }

        // Check for malicious content in JSON
        if self.contains_malicious_content(json_str) {
            return Err(ValidationError::MaliciousContent {
                field: "json_payload".to_string(),
            });
        }

        // Parse JSON
        serde_json::from_str(json_str).map_err(|e| ValidationError::InvalidFormat {
            field: "json_payload".to_string(),
            expected_format: format!("Valid JSON: {}", e),
        })
    }
}

// Predefined validation rules for common fields
pub struct CommonValidationRules;

impl CommonValidationRules {
    pub fn email() -> ValidationRule {
        ValidationRule {
            required: true,
            max_length: Some(254),
            pattern: Some(Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap()),
            ..Default::default()
        }
    }

    pub fn password() -> ValidationRule {
        ValidationRule {
            required: true,
            min_length: Some(8),
            max_length: Some(128),
            pattern: Some(Regex::new(r"^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)(?=.*[@$!%*?&])[A-Za-z\d@$!%*?&]").unwrap()),
            check_xss: false, // Passwords can contain special characters
            ..Default::default()
        }
    }

    pub fn username() -> ValidationRule {
        ValidationRule {
            required: true,
            min_length: Some(3),
            max_length: Some(30),
            pattern: Some(Regex::new(r"^[a-zA-Z0-9_-]+$").unwrap()),
            ..Default::default()
        }
    }

    pub fn name() -> ValidationRule {
        ValidationRule {
            required: false,
            max_length: Some(100),
            allowed_chars: Some("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ '-".to_string()),
            ..Default::default()
        }
    }

    pub fn description() -> ValidationRule {
        ValidationRule {
            required: false,
            max_length: Some(2000),
            ..Default::default()
        }
    }

    pub fn url() -> ValidationRule {
        ValidationRule {
            required: false,
            max_length: Some(2048),
            pattern: Some(Regex::new(r"^https?://[^\s/$.?#].[^\s]*$").unwrap()),
            ..Default::default()
        }
    }

    pub fn phone() -> ValidationRule {
        ValidationRule {
            required: false,
            pattern: Some(Regex::new(r"^\+?[1-9]\d{1,14}$").unwrap()),
            ..Default::default()
        }
    }

    pub fn numeric_id() -> ValidationRule {
        ValidationRule {
            required: true,
            pattern: Some(Regex::new(r"^\d+$").unwrap()),
            max_length: Some(20),
            ..Default::default()
        }
    }

    pub fn uuid() -> ValidationRule {
        ValidationRule {
            required: true,
            pattern: Some(Regex::new(r"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$").unwrap()),
            ..Default::default()
        }
    }

    pub fn amount() -> ValidationRule {
        ValidationRule {
            required: true,
            pattern: Some(Regex::new(r"^\d+(\.\d{1,2})?$").unwrap()),
            max_length: Some(20),
            ..Default::default()
        }
    }
}

// Validation schema for different endpoints
#[derive(Debug)]
pub struct ValidationSchema {
    pub fields: HashMap<String, ValidationRule>,
}

impl ValidationSchema {
    pub fn new() -> Self {
        Self {
            fields: HashMap::new(),
        }
    }

    pub fn add_field(mut self, name: &str, rule: ValidationRule) -> Self {
        self.fields.insert(name.to_string(), rule);
        self
    }

    pub fn validate(&self, validator: &InputValidator, data: &HashMap<String, String>) -> Result<HashMap<String, String>, Vec<ValidationError>> {
        let mut errors = Vec::new();
        let mut validated_data = HashMap::new();

        for (field_name, rule) in &self.fields {
            let value = data.get(field_name).unwrap_or(&String::new());
            
            match validator.validate_field(field_name, value, rule) {
                Ok(sanitized_value) => {
                    validated_data.insert(field_name.clone(), sanitized_value);
                }
                Err(error) => {
                    errors.push(error);
                }
            }
        }

        if errors.is_empty() {
            Ok(validated_data)
        } else {
            Err(errors)
        }
    }
}

// Predefined schemas for common endpoints
pub struct ValidationSchemas;

impl ValidationSchemas {
    pub fn user_registration() -> ValidationSchema {
        ValidationSchema::new()
            .add_field("email", CommonValidationRules::email())
            .add_field("password", CommonValidationRules::password())
            .add_field("first_name", CommonValidationRules::name())
            .add_field("last_name", CommonValidationRules::name())
    }

    pub fn user_login() -> ValidationSchema {
        ValidationSchema::new()
            .add_field("email", CommonValidationRules::email())
            .add_field("password", ValidationRule {
                required: true,
                max_length: Some(128),
                check_xss: false,
                ..Default::default()
            })
    }

    pub fn item_creation() -> ValidationSchema {
        ValidationSchema::new()
            .add_field("title", ValidationRule {
                required: true,
                min_length: Some(3),
                max_length: Some(200),
                ..Default::default()
            })
            .add_field("description", CommonValidationRules::description())
            .add_field("price", CommonValidationRules::amount())
            .add_field("category", ValidationRule {
                required: true,
                max_length: Some(50),
                pattern: Some(Regex::new(r"^[a-zA-Z0-9_-]+$").unwrap()),
                ..Default::default()
            })
    }

    pub fn raffle_creation() -> ValidationSchema {
        ValidationSchema::new()
            .add_field("item_id", CommonValidationRules::uuid())
            .add_field("total_boxes", ValidationRule {
                required: true,
                pattern: Some(Regex::new(r"^[1-9]\d{0,3}$").unwrap()), // 1-9999
                ..Default::default()
            })
            .add_field("box_price", CommonValidationRules::amount())
            .add_field("total_winners", ValidationRule {
                required: true,
                pattern: Some(Regex::new(r"^[1-9]\d{0,2}$").unwrap()), // 1-999
                ..Default::default()
            })
    }

    pub fn box_purchase() -> ValidationSchema {
        ValidationSchema::new()
            .add_field("raffle_id", CommonValidationRules::uuid())
            .add_field("box_numbers", ValidationRule {
                required: true,
                max_length: Some(1000),
                pattern: Some(Regex::new(r"^\[\s*\d+(\s*,\s*\d+)*\s*\]$").unwrap()),
                ..Default::default()
            })
    }

    pub fn credit_purchase() -> ValidationSchema {
        ValidationSchema::new()
            .add_field("amount", CommonValidationRules::amount())
            .add_field("payment_method_id", ValidationRule {
                required: true,
                pattern: Some(Regex::new(r"^pm_[a-zA-Z0-9]+$").unwrap()), // Stripe payment method ID
                ..Default::default()
            })
    }
}

// Helper function to create validation error response
pub fn validation_error_response(errors: Vec<ValidationError>) -> HttpResponse {
    let error_messages: Vec<serde_json::Value> = errors
        .into_iter()
        .map(|e| {
            serde_json::json!({
                "field": match &e {
                    ValidationError::InvalidInput { field, .. } => field,
                    ValidationError::MaliciousContent { field } => field,
                    ValidationError::TooLong { field, .. } => field,
                    ValidationError::Required { field } => field,
                    ValidationError::InvalidFormat { field, .. } => field,
                },
                "message": e.to_string()
            })
        })
        .collect();

    HttpResponse::BadRequest().json(serde_json::json!({
        "error": "Validation failed",
        "details": error_messages
    }))
}

// Middleware for request size limiting
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use futures_util::future::LocalBoxFuture;
use std::{
    future::{ready, Ready},
    rc::Rc,
};

pub struct RequestSizeLimit {
    max_size: usize,
}

impl RequestSizeLimit {
    pub fn new(max_size: usize) -> Self {
        Self { max_size }
    }
}

pub struct RequestSizeLimitMiddleware<S> {
    service: Rc<S>,
    max_size: usize,
}

impl<S, B> Service<ServiceRequest> for RequestSizeLimitMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let max_size = self.max_size;

        Box::pin(async move {
            if let Some(content_length) = req.headers().get("content-length") {
                if let Ok(length_str) = content_length.to_str() {
                    if let Ok(length) = length_str.parse::<usize>() {
                        if length > max_size {
                            return Ok(req.into_response(
                                HttpResponse::PayloadTooLarge()
                                    .json(serde_json::json!({
                                        "error": "Request too large",
                                        "max_size": max_size
                                    }))
                            ));
                        }
                    }
                }
            }

            service.call(req).await
        })
    }
}

impl<S, B> Transform<S, ServiceRequest> for RequestSizeLimit
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = RequestSizeLimitMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RequestSizeLimitMiddleware {
            service: Rc::new(service),
            max_size: self.max_size,
        }))
    }
}