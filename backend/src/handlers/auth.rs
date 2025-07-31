use crate::error::AppError;
use crate::middleware::auth::Claims;
use crate::models::User;
use crate::services::auth_service::AuthService;
use crate::services::wallet_service::WalletService;
use actix_web::{web, HttpRequest, HttpResponse, Result, post, get, put};
use raffle_platform_shared::{
    AuthResponse, CreateUserRequest, LoginRequest, UserRole,
    ERROR_INVALID_CREDENTIALS, ERROR_EMAIL_ALREADY_EXISTS, ERROR_USERNAME_ALREADY_EXISTS,
};
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::sync::Arc;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use regex;



#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResetPasswordRequest {
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfirmPasswordResetRequest {
    pub token: String,
    pub new_password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateProfileRequest {
    pub username: Option<String>,
    pub phone_number: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserProfileResponse {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub role: UserRole,
    pub credit_balance: Decimal,
    pub internal_wallet_address: String,
    pub phone_number: Option<String>,
    pub is_active: bool,
    pub email_verified: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegistrationResponse {
    pub user: UserProfileResponse,
    pub tokens: TokenResponse,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
    pub token_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub user: UserProfileResponse,
    pub tokens: TokenResponse,
    pub session_info: SessionInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionInfo {
    pub session_id: Uuid,
    pub device_info: Option<String>,
    pub ip_address: Option<String>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogoutRequest {
    pub logout_all_devices: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmailVerificationRequest {
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResendVerificationRequest {
    pub email: String,
}

/// Register a new user with comprehensive validation and wallet generation
#[post(\"/register\")]
pub async fn register(
    auth_service: web::Data<AuthService>,
    wallet_service: web::Data<WalletService>,
    req: web::Json<CreateUserRequest>,
    http_req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let ip_address = http_req
        .connection_info()
        .realip_remote_addr()
        .and_then(|ip| ip.parse::<IpAddr>().ok());

    let user_agent = http_req
        .headers()
        .get(\"User-Agent\")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());

    // Validate request data
    validate_registration_request(&req)?;

    // Register user with enhanced response
    let auth_response = auth_service
        .register(req.into_inner(), ip_address, user_agent)
        .await?;

    // Convert to enhanced registration response
    let user_profile = UserProfileResponse {
        id: auth_response.user.id,
        username: auth_response.user.username,
        email: auth_response.user.email,
        role: auth_response.user.role,
        credit_balance: auth_response.user.credit_balance,
        internal_wallet_address: auth_response.user.internal_wallet_address,
        phone_number: auth_response.user.phone_number,
        is_active: auth_response.user.is_active,
        email_verified: auth_response.user.email_verified,
        created_at: auth_response.user.created_at,
        updated_at: auth_response.user.updated_at,
    };

    let token_response = TokenResponse {
        access_token: auth_response.access_token,
        refresh_token: auth_response.refresh_token,
        expires_in: auth_response.expires_in,
        token_type: \"Bearer\".to_string(),
    };

    let registration_response = RegistrationResponse {
        user: user_profile,
        tokens: token_response,
        message: \"Registration successful. Please check your email to verify your account.\".to_string(),
    };

    Ok(HttpResponse::Created().json(registration_response))
}

/// Login user with enhanced session management
#[post(\"/login\")]
pub async fn login(
    auth_service: web::Data<AuthService>,
    req: web::Json<LoginRequest>,
    http_req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let ip_address = http_req
        .connection_info()
        .realip_remote_addr()
        .and_then(|ip| ip.parse::<IpAddr>().ok());

    let user_agent = http_req
        .headers()
        .get(\"User-Agent\")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());

    // Validate login request
    validate_login_request(&req)?;

    // Perform login with enhanced response
    let auth_response = auth_service
        .login(req.into_inner(), ip_address, user_agent)
        .await?;

    // Get session information
    let session_info = auth_service
        .get_current_session_info(&auth_response.refresh_token)
        .await
        .unwrap_or_else(|_| SessionInfo {
            session_id: Uuid::new_v4(),
            device_info: user_agent.clone(),
            ip_address: ip_address.map(|ip| ip.to_string()),
            expires_at: Utc::now() + chrono::Duration::days(7),
        });

    // Convert to enhanced login response
    let user_profile = UserProfileResponse {
        id: auth_response.user.id,
        username: auth_response.user.username,
        email: auth_response.user.email,
        role: auth_response.user.role,
        credit_balance: auth_response.user.credit_balance,
        internal_wallet_address: auth_response.user.internal_wallet_address,
        phone_number: auth_response.user.phone_number,
        is_active: auth_response.user.is_active,
        email_verified: auth_response.user.email_verified,
        created_at: auth_response.user.created_at,
        updated_at: auth_response.user.updated_at,
    };

    let token_response = TokenResponse {
        access_token: auth_response.access_token,
        refresh_token: auth_response.refresh_token,
        expires_in: auth_response.expires_in,
        token_type: \"Bearer\".to_string(),
    };

    let login_response = LoginResponse {
        user: user_profile,
        tokens: token_response,
        session_info,
    };

    Ok(HttpResponse::Ok().json(login_response))
}

/// Refresh access token
#[post(\"/refresh\")]
pub async fn refresh_token(
    auth_service: web::Data<AuthService>,
    req: web::Json<RefreshTokenRequest>,
) -> Result<HttpResponse, AppError> {
    let auth_response = auth_service
        .refresh_token(&req.refresh_token)
        .await?;

    Ok(HttpResponse::Ok().json(auth_response))
}

/// Initiate password reset
#[post(\"/forgot-password\")]
pub async fn forgot_password(
    auth_service: web::Data<AuthService>,
    req: web::Json<ResetPasswordRequest>,
    http_req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let ip_address = http_req
        .connection_info()
        .realip_remote_addr()
        .and_then(|ip| ip.parse::<IpAddr>().ok());

    let user_agent = http_req
        .headers()
        .get(\"User-Agent\")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());

    auth_service
        .initiate_password_reset(&req.email, ip_address, user_agent)
        .await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        \"message\": \"Password reset email sent\"
    })))
}

/// Complete password reset
#[post(\"/reset-password\")]
pub async fn reset_password(
    auth_service: web::Data<AuthService>,
    req: web::Json<ConfirmPasswordResetRequest>,
    http_req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let ip_address = http_req
        .connection_info()
        .realip_remote_addr()
        .and_then(|ip| ip.parse::<IpAddr>().ok());

    let user_agent = http_req
        .headers()
        .get(\"User-Agent\")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());

    auth_service
        .complete_password_reset(&req.token, &req.new_password, ip_address, user_agent)
        .await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        \"message\": \"Password reset successfully\"
    })))
}

/// Verify email address
#[post(\"/verify-email\")]
pub async fn verify_email(
    auth_service: web::Data<AuthService>,
    req: web::Json<EmailVerificationRequest>,
) -> Result<HttpResponse, AppError> {
    auth_service.verify_email(&req.token).await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        \"message\": \"Email verified successfully\"
    })))
}

/// Logout user
#[post(\"/logout\")]
pub async fn logout(
    claims: Claims,
    auth_service: web::Data<AuthService>,
    req: web::Json<LogoutRequest>,
) -> Result<HttpResponse, AppError> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Authentication(\"Invalid user ID in token\".to_string()))?;

    let logout_all = req.logout_all_devices.unwrap_or(false);

    if logout_all {
        auth_service.logout_all_sessions(user_id).await?;
    } else {
        // Get current session from JWT ID and revoke it
        auth_service.logout_current_session(&claims.jti).await?;
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        \"message\": if logout_all { \"Logged out from all devices\" } else { \"Logged out successfully\" }
    })))
}

/// Get current user profile with enhanced information
#[get(\"/current-user\")]
pub async fn get_current_user(
    claims: Claims,
    auth_service: web::Data<AuthService>,
) -> Result<HttpResponse, AppError> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Authentication(\"Invalid user ID in token\".to_string()))?;

    let user = auth_service.get_current_user(user_id).await?;
    
    let user_profile = UserProfileResponse {
        id: user.id,
        username: user.username,
        email: user.email,
        role: user.role,
        credit_balance: user.credit_balance,
        internal_wallet_address: user.internal_wallet_address,
        phone_number: user.phone_number,
        is_active: user.is_active,
        email_verified: user.email_verified,
        created_at: user.created_at,
        updated_at: user.updated_at,
    };

    Ok(HttpResponse::Ok().json(user_profile))
}

/// Update current user profile
#[put(\"/current-user\")]
pub async fn update_current_user(
    claims: Claims,
    auth_service: web::Data<AuthService>,
    req: web::Json<UpdateProfileRequest>,
) -> Result<HttpResponse, AppError> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Authentication(\"Invalid user ID in token\".to_string()))?;

    // Validate update request
    validate_profile_update_request(&req)?;

    let updated_user = auth_service
        .update_user_profile(user_id, req.into_inner())
        .await?;

    let user_profile = UserProfileResponse {
        id: updated_user.id,
        username: updated_user.username,
        email: updated_user.email,
        role: updated_user.role,
        credit_balance: updated_user.credit_balance,
        internal_wallet_address: updated_user.internal_wallet_address,
        phone_number: updated_user.phone_number,
        is_active: updated_user.is_active,
        email_verified: updated_user.email_verified,
        created_at: updated_user.created_at,
        updated_at: updated_user.updated_at,
    };

    Ok(HttpResponse::Ok().json(user_profile))
}

/// Change user password
#[post(\"/change-password\")]
pub async fn change_password(
    claims: Claims,
    auth_service: web::Data<AuthService>,
    req: web::Json<ChangePasswordRequest>,
    http_req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Authentication(\"Invalid user ID in token\".to_string()))?;

    let ip_address = http_req
        .connection_info()
        .realip_remote_addr()
        .and_then(|ip| ip.parse::<IpAddr>().ok());

    let user_agent = http_req
        .headers()
        .get(\"User-Agent\")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());

    auth_service
        .change_password(user_id, &req.current_password, &req.new_password, ip_address, user_agent)
        .await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        \"message\": \"Password changed successfully\"
    })))
}

// Validation functions

/// Validate registration request
fn validate_registration_request(req: &CreateUserRequest) -> Result<(), AppError> {
    // Validate username
    if req.username.trim().is_empty() {
        return Err(AppError::Validation(\"Username cannot be empty\".to_string()));
    }
    
    if req.username.len() < 3 || req.username.len() > 30 {
        return Err(AppError::Validation(\"Username must be between 3 and 30 characters\".to_string()));
    }
    
    if !req.username.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
        return Err(AppError::Validation(\"Username can only contain letters, numbers, underscores, and hyphens\".to_string()));
    }

    // Validate email
    if req.email.trim().is_empty() {
        return Err(AppError::Validation(\"Email cannot be empty\".to_string()));
    }
    
    if !is_valid_email(&req.email) {
        return Err(AppError::Validation(\"Invalid email format\".to_string()));
    }

    // Validate phone number if provided
    if let Some(phone) = &req.phone_number {
        if !phone.trim().is_empty() && !is_valid_phone_number(phone) {
            return Err(AppError::Validation(\"Invalid phone number format\".to_string()));
        }
    }

    Ok(())
}

/// Validate login request
fn validate_login_request(req: &LoginRequest) -> Result<(), AppError> {
    if req.email.trim().is_empty() {
        return Err(AppError::Validation(\"Email cannot be empty\".to_string()));
    }
    
    if req.password.trim().is_empty() {
        return Err(AppError::Validation(\"Password cannot be empty\".to_string()));
    }

    Ok(())
}

/// Validate profile update request
fn validate_profile_update_request(req: &UpdateProfileRequest) -> Result<(), AppError> {
    // Validate username if provided
    if let Some(username) = &req.username {
        if username.trim().is_empty() {
            return Err(AppError::Validation(\"Username cannot be empty\".to_string()));
        }
        
        if username.len() < 3 || username.len() > 30 {
            return Err(AppError::Validation(\"Username must be between 3 and 30 characters\".to_string()));
        }
        
        if !username.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
            return Err(AppError::Validation(\"Username can only contain letters, numbers, underscores, and hyphens\".to_string()));
        }
    }

    // Validate phone number if provided
    if let Some(phone) = &req.phone_number {
        if !phone.trim().is_empty() && !is_valid_phone_number(phone) {
            return Err(AppError::Validation(\"Invalid phone number format\".to_string()));
        }
    }

    Ok(())
}

/// Validate email format
fn is_valid_email(email: &str) -> bool {
    let email_regex = regex::Regex::new(r\"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}$\").unwrap();
    email_regex.is_match(email)
}

/// Validate phone number format
fn is_valid_phone_number(phone: &str) -> bool {
    let phone_regex = regex::Regex::new(r\"^\\+?[1-9]\\d{1,14}$\").unwrap();
    phone_regex.is_match(phone)
}