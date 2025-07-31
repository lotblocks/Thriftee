use actix_web::{get, post, put, web, HttpRequest, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;
use ethers::types::{Address, U256, TransactionRequest};
use std::str::FromStr;

use crate::error::AppError;
use crate::middleware::auth::{extract_user_id, Claims};
use crate::services::WalletService;

#[derive(Debug, Serialize, Deserialize)]
pub struct SignMessageRequest {
    pub message: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SignTransactionRequest {
    pub to: String,
    pub value: Option<String>, // Wei amount as string
    pub data: Option<String>,  // Hex encoded data
    pub gas_limit: Option<String>,
    pub gas_price: Option<String>,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SendTransactionRequest {
    pub to: String,
    pub value: Option<String>, // Wei amount as string
    pub data: Option<String>,  // Hex encoded data
    pub gas_limit: Option<String>,
    pub gas_price: Option<String>,
    pub password: String,
    pub provider_url: String,
    pub chain_id: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImportWalletRequest {
    pub private_key: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportWalletRequest {
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RotateEncryptionRequest {
    pub old_password: String,
    pub new_password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GenerateFromMnemonicRequest {
    pub mnemonic_phrase: String,
    pub password: String,
    pub account_index: Option<u32>,
    pub address_index: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeriveAddressesRequest {
    pub password: String,
    pub count: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CheckBalanceRequest {
    pub provider_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetMnemonicRequest {
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WalletInfoResponse {
    pub address: String,
    pub balance: Option<String>, // Wei amount as string
    pub transaction_count: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SignatureResponse {
    pub signature: String,
    pub r: String,
    pub s: String,
    pub v: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionResponse {
    pub transaction_hash: String,
    pub status: String,
}

/// Get wallet address for authenticated user
#[get("/address")]
pub async fn get_wallet_address(
    claims: Claims,
    wallet_service: web::Data<WalletService>,
) -> Result<HttpResponse, AppError> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Authentication("Invalid user ID in token".to_string()))?;

    let address = wallet_service.get_user_wallet_address(user_id).await?;

    Ok(HttpResponse::Ok().json(json!({
        "address": address
    })))
}

/// Get comprehensive wallet information
#[get("/info")]
pub async fn get_wallet_info(
    claims: Claims,
    wallet_service: web::Data<WalletService>,
    query: web::Query<CheckBalanceRequest>,
) -> Result<HttpResponse, AppError> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Authentication("Invalid user ID in token".to_string()))?;

    let address = wallet_service.get_user_wallet_address(user_id).await?;
    
    // Try to get balance and transaction count
    let balance = wallet_service
        .check_wallet_balance(user_id, &query.provider_url)
        .await
        .ok()
        .map(|b| b.to_string());
    
    let transaction_count = wallet_service
        .get_transaction_count(user_id, &query.provider_url)
        .await
        .ok()
        .map(|n| n.as_u64());

    let wallet_info = WalletInfoResponse {
        address,
        balance,
        transaction_count,
    };

    Ok(HttpResponse::Ok().json(wallet_info))
}

/// Sign a message with user's wallet
#[post("/sign-message")]
pub async fn sign_message(
    claims: Claims,
    wallet_service: web::Data<WalletService>,
    req: web::Json<SignMessageRequest>,
) -> Result<HttpResponse, AppError> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Authentication("Invalid user ID in token".to_string()))?;

    let signature = wallet_service
        .sign_message(user_id, &req.password, req.message.as_bytes())
        .await?;

    let signature_response = SignatureResponse {
        signature: format!("0x{}", hex::encode(signature.to_vec())),
        r: format!("0x{:064x}", signature.r),
        s: format!("0x{:064x}", signature.s),
        v: signature.v,
    };

    Ok(HttpResponse::Ok().json(signature_response))
}

/// Sign a transaction with user's wallet
#[post("/sign-transaction")]
pub async fn sign_transaction(
    claims: Claims,
    wallet_service: web::Data<WalletService>,
    req: web::Json<SignTransactionRequest>,
) -> Result<HttpResponse, AppError> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Authentication("Invalid user ID in token".to_string()))?;

    // Parse transaction parameters
    let to_address = Address::from_str(&req.to)
        .map_err(|_| AppError::Validation("Invalid 'to' address".to_string()))?;

    let mut transaction = TransactionRequest::new().to(to_address);

    if let Some(value_str) = &req.value {
        let value = U256::from_dec_str(value_str)
            .map_err(|_| AppError::Validation("Invalid value format".to_string()))?;
        transaction = transaction.value(value);
    }

    if let Some(data_str) = &req.data {
        let data = hex::decode(data_str.strip_prefix("0x").unwrap_or(data_str))
            .map_err(|_| AppError::Validation("Invalid data format".to_string()))?;
        transaction = transaction.data(data);
    }

    if let Some(gas_limit_str) = &req.gas_limit {
        let gas_limit = U256::from_dec_str(gas_limit_str)
            .map_err(|_| AppError::Validation("Invalid gas limit format".to_string()))?;
        transaction = transaction.gas(gas_limit);
    }

    if let Some(gas_price_str) = &req.gas_price {
        let gas_price = U256::from_dec_str(gas_price_str)
            .map_err(|_| AppError::Validation("Invalid gas price format".to_string()))?;
        transaction = transaction.gas_price(gas_price);
    }

    let signature = wallet_service
        .sign_transaction(user_id, &req.password, transaction)
        .await?;

    let signature_response = SignatureResponse {
        signature: format!("0x{}", hex::encode(signature.to_vec())),
        r: format!("0x{:064x}", signature.r),
        s: format!("0x{:064x}", signature.s),
        v: signature.v,
    };

    Ok(HttpResponse::Ok().json(signature_response))
}

/// Send a transaction using user's wallet
#[post("/send-transaction")]
pub async fn send_transaction(
    claims: Claims,
    wallet_service: web::Data<WalletService>,
    req: web::Json<SendTransactionRequest>,
) -> Result<HttpResponse, AppError> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Authentication("Invalid user ID in token".to_string()))?;

    // Parse transaction parameters
    let to_address = Address::from_str(&req.to)
        .map_err(|_| AppError::Validation("Invalid 'to' address".to_string()))?;

    let mut transaction = TransactionRequest::new().to(to_address);

    if let Some(value_str) = &req.value {
        let value = U256::from_dec_str(value_str)
            .map_err(|_| AppError::Validation("Invalid value format".to_string()))?;
        transaction = transaction.value(value);
    }

    if let Some(data_str) = &req.data {
        let data = hex::decode(data_str.strip_prefix("0x").unwrap_or(data_str))
            .map_err(|_| AppError::Validation("Invalid data format".to_string()))?;
        transaction = transaction.data(data);
    }

    if let Some(gas_limit_str) = &req.gas_limit {
        let gas_limit = U256::from_dec_str(gas_limit_str)
            .map_err(|_| AppError::Validation("Invalid gas limit format".to_string()))?;
        transaction = transaction.gas(gas_limit);
    }

    if let Some(gas_price_str) = &req.gas_price {
        let gas_price = U256::from_dec_str(gas_price_str)
            .map_err(|_| AppError::Validation("Invalid gas price format".to_string()))?;
        transaction = transaction.gas_price(gas_price);
    }

    let tx_hash = wallet_service
        .send_transaction(user_id, &req.password, transaction, &req.provider_url, req.chain_id)
        .await?;

    let transaction_response = TransactionResponse {
        transaction_hash: format!("0x{:064x}", tx_hash),
        status: "pending".to_string(),
    };

    Ok(HttpResponse::Ok().json(transaction_response))
}
            "signature": format!("{:?}", signature),
            "message": message
        }
    })))
}

#[post("/verify-signature")]
pub async fn verify_signature(
    pool: web::Data<PgPool>,
    request: HttpRequest,
    verify_request: web::Json<serde_json::Value>,
) -> Result<HttpResponse, AppError> {
    let user_id = extract_user_id(&request.into())?;
    let wallet_service = WalletService::new(pool.get_ref().clone());

    let message = verify_request
        .get("message")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Validation("Message is required".to_string()))?;

    let signature_str = verify_request
        .get("signature")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Validation("Signature is required".to_string()))?;

    // Parse signature (simplified - in production you'd want better parsing)
    let signature = ethers::types::Signature::from_str(signature_str)
        .map_err(|e| AppError::Validation(format!("Invalid signature format: {}", e)))?;

    let is_valid = wallet_service
        .verify_signature(user_id, message.as_bytes(), &signature)
        .await?;

    Ok(HttpResponse::Ok().json(json!({
        "data": {
            "is_valid": is_valid,
            "message": message,
            "signature": signature_str
        }
    })))
}

#[post("/export-private-key")]
pub async fn export_private_key(
    pool: web::Data<PgPool>,
    request: HttpRequest,
    export_request: web::Json<serde_json::Value>,
) -> Result<HttpResponse, AppError> {
    let user_id = extract_user_id(&request.into())?;
    let wallet_service = WalletService::new(pool.get_ref().clone());

    let password = export_request
        .get("password")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Validation("Password is required".to_string()))?;

    let private_key = wallet_service
        .export_private_key(user_id, password)
        .await?;

    Ok(HttpResponse::Ok().json(json!({
        "data": {
            "private_key": private_key,
            "warning": "Keep this private key secure and never share it with anyone!"
        }
    })))
}

#[post("/import-private-key")]
pub async fn import_private_key(
    pool: web::Data<PgPool>,
    request: HttpRequest,
    import_request: web::Json<serde_json::Value>,
) -> Result<HttpResponse, AppError> {
    let user_id = extract_user_id(&request.into())?;
    let wallet_service = WalletService::new(pool.get_ref().clone());

    let password = import_request
        .get("password")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Validation("Password is required".to_string()))?;

    let private_key = import_request
        .get("private_key")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Validation("Private key is required".to_string()))?;

    let new_address = wallet_service
        .import_wallet(user_id, password, private_key)
        .await?;

    Ok(HttpResponse::Ok().json(json!({
        "message": "Wallet imported successfully",
        "data": {
            "address": new_address
        }
    })))
}

#[post("/rotate-encryption")]
pub async fn rotate_wallet_encryption(
    pool: web::Data<PgPool>,
    request: HttpRequest,
    rotate_request: web::Json<serde_json::Value>,
) -> Result<HttpResponse, AppError> {
    let user_id = extract_user_id(&request.into())?;
    let wallet_service = WalletService::new(pool.get_ref().clone());

    let old_password = rotate_request
        .get("old_password")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Validation("Old password is required".to_string()))?;

    let new_password = rotate_request
        .get("new_password")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Validation("New password is required".to_string()))?;

    wallet_service
        .rotate_wallet_encryption(user_id, old_password, new_password)
        .await?;

    Ok(HttpResponse::Ok().json(json!({
        "message": "Wallet encryption rotated successfully"
    })))
}

#[get("/balance")]
pub async fn get_wallet_balance(
    pool: web::Data<PgPool>,
    request: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let user_id = extract_user_id(&request.into())?;
    let wallet_service = WalletService::new(pool.get_ref().clone());

    let address = wallet_service.get_user_wallet_address(user_id).await?;

    // In a real implementation, you'd query the blockchain for the actual balance
    // For now, we'll return a placeholder
    Ok(HttpResponse::Ok().json(json!({
        "data": {
            "address": address,
            "balance": "0.0",
            "currency": "MATIC",
            "note": "Balance querying not implemented yet - requires blockchain provider integration"
        }
    })))
}

use ethers::types::Signature;
use std::str::FromStr;/
// Export wallet private key (for backup)
#[post("/export")]
pub async fn export_private_key(
    claims: Claims,
    wallet_service: web::Data<WalletService>,
    req: web::Json<ExportWalletRequest>,
) -> Result<HttpResponse, AppError> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Authentication("Invalid user ID in token".to_string()))?;

    let private_key = wallet_service
        .export_private_key(user_id, &req.password)
        .await?;

    Ok(HttpResponse::Ok().json(json!({
        "private_key": format!("0x{}", private_key),
        "warning": "Keep this private key secure and never share it with anyone"
    })))
}

/// Import wallet from private key
#[post("/import")]
pub async fn import_wallet(
    claims: Claims,
    wallet_service: web::Data<WalletService>,
    req: web::Json<ImportWalletRequest>,
) -> Result<HttpResponse, AppError> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Authentication("Invalid user ID in token".to_string()))?;

    // Remove 0x prefix if present
    let private_key = req.private_key.strip_prefix("0x").unwrap_or(&req.private_key);

    let address = wallet_service
        .import_wallet(user_id, &req.password, private_key)
        .await?;

    Ok(HttpResponse::Ok().json(json!({
        "address": address,
        "message": "Wallet imported successfully"
    })))
}

/// Rotate wallet encryption (change password)
#[put("/rotate-encryption")]
pub async fn rotate_encryption(
    claims: Claims,
    wallet_service: web::Data<WalletService>,
    req: web::Json<RotateEncryptionRequest>,
) -> Result<HttpResponse, AppError> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Authentication("Invalid user ID in token".to_string()))?;

    wallet_service
        .rotate_wallet_encryption(user_id, &req.old_password, &req.new_password)
        .await?;

    Ok(HttpResponse::Ok().json(json!({
        "message": "Wallet encryption updated successfully"
    })))
}

/// Generate wallet from mnemonic phrase
#[post("/generate-from-mnemonic")]
pub async fn generate_from_mnemonic(
    claims: Claims,
    wallet_service: web::Data<WalletService>,
    req: web::Json<GenerateFromMnemonicRequest>,
) -> Result<HttpResponse, AppError> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Authentication("Invalid user ID in token".to_string()))?;

    let account_index = req.account_index.unwrap_or(0);
    let address_index = req.address_index.unwrap_or(0);

    let (address, _encrypted_key) = wallet_service
        .generate_wallet_from_mnemonic(
            user_id,
            &req.password,
            &req.mnemonic_phrase,
            account_index,
            address_index,
        )
        .await?;

    Ok(HttpResponse::Ok().json(json!({
        "address": address,
        "account_index": account_index,
        "address_index": address_index,
        "message": "Wallet generated from mnemonic successfully"
    })))
}

/// Derive multiple addresses from HD wallet
#[post("/derive-addresses")]
pub async fn derive_addresses(
    claims: Claims,
    wallet_service: web::Data<WalletService>,
    req: web::Json<DeriveAddressesRequest>,
) -> Result<HttpResponse, AppError> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Authentication("Invalid user ID in token".to_string()))?;

    if req.count == 0 || req.count > 100 {
        return Err(AppError::Validation("Count must be between 1 and 100".to_string()));
    }

    let addresses = wallet_service
        .derive_addresses(user_id, &req.password, req.count)
        .await?;

    Ok(HttpResponse::Ok().json(json!({
        "addresses": addresses,
        "count": addresses.len()
    })))
}

/// Get wallet mnemonic phrase (for backup)
#[post("/mnemonic")]
pub async fn get_mnemonic(
    claims: Claims,
    wallet_service: web::Data<WalletService>,
    req: web::Json<GetMnemonicRequest>,
) -> Result<HttpResponse, AppError> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Authentication("Invalid user ID in token".to_string()))?;

    let mnemonic = wallet_service
        .get_user_mnemonic(user_id, &req.password)
        .await?;

    match mnemonic {
        Some(phrase) => Ok(HttpResponse::Ok().json(json!({
            "mnemonic": phrase,
            "warning": "Keep this mnemonic phrase secure and never share it with anyone"
        }))),
        None => Ok(HttpResponse::NotFound().json(json!({
            "error": "No mnemonic found for this wallet"
        })))
    }
}

/// Check wallet balance
#[get("/balance")]
pub async fn check_balance(
    claims: Claims,
    wallet_service: web::Data<WalletService>,
    query: web::Query<CheckBalanceRequest>,
) -> Result<HttpResponse, AppError> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Authentication("Invalid user ID in token".to_string()))?;

    let balance = wallet_service
        .check_wallet_balance(user_id, &query.provider_url)
        .await?;

    Ok(HttpResponse::Ok().json(json!({
        "balance": balance.to_string(),
        "balance_eth": ethers::utils::format_ether(balance),
        "provider_url": query.provider_url
    })))
}

/// Get transaction count (nonce)
#[get("/nonce")]
pub async fn get_nonce(
    claims: Claims,
    wallet_service: web::Data<WalletService>,
    query: web::Query<CheckBalanceRequest>,
) -> Result<HttpResponse, AppError> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Authentication("Invalid user ID in token".to_string()))?;

    let nonce = wallet_service
        .get_transaction_count(user_id, &query.provider_url)
        .await?;

    Ok(HttpResponse::Ok().json(json!({
        "nonce": nonce.as_u64(),
        "provider_url": query.provider_url
    })))
}

/// Validate mnemonic phrase
#[post("/validate-mnemonic")]
pub async fn validate_mnemonic(
    req: web::Json<serde_json::Value>,
) -> Result<HttpResponse, AppError> {
    let mnemonic_phrase = req
        .get("mnemonic")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Validation("Mnemonic phrase is required".to_string()))?;

    match WalletService::validate_mnemonic(mnemonic_phrase) {
        Ok(_) => Ok(HttpResponse::Ok().json(json!({
            "valid": true,
            "message": "Mnemonic phrase is valid"
        }))),
        Err(e) => Ok(HttpResponse::BadRequest().json(json!({
            "valid": false,
            "error": e.to_string()
        })))
    }
}

/// Generate new mnemonic phrase
#[post("/generate-mnemonic")]
pub async fn generate_mnemonic() -> Result<HttpResponse, AppError> {
    let mnemonic = WalletService::generate_mnemonic_phrase()?;

    Ok(HttpResponse::Ok().json(json!({
        "mnemonic": mnemonic,
        "word_count": mnemonic.split_whitespace().count(),
        "warning": "Keep this mnemonic phrase secure and never share it with anyone"
    })))
}

/// Validate wallet address format
#[post("/validate-address")]
pub async fn validate_address(
    req: web::Json<serde_json::Value>,
) -> Result<HttpResponse, AppError> {
    let address = req
        .get("address")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Validation("Address is required".to_string()))?;

    let is_valid = WalletService::is_valid_address(address);

    Ok(HttpResponse::Ok().json(json!({
        "valid": is_valid,
        "address": address
    })))
}

/// Get optimal gas price
#[get("/gas-price")]
pub async fn get_gas_price(
    wallet_service: web::Data<WalletService>,
) -> Result<HttpResponse, AppError> {
    let gas_price = wallet_service.get_optimal_gas_price().await?;

    Ok(HttpResponse::Ok().json(json!({
        "gas_price": gas_price.to_string(),
        "gas_price_gwei": ethers::utils::format_units(gas_price, "gwei").unwrap_or_default()
    })))
}

/// Estimate gas for transaction
#[post("/estimate-gas")]
pub async fn estimate_gas(
    wallet_service: web::Data<WalletService>,
    req: web::Json<SignTransactionRequest>,
) -> Result<HttpResponse, AppError> {
    // Parse transaction parameters (similar to sign_transaction)
    let to_address = Address::from_str(&req.to)
        .map_err(|_| AppError::Validation("Invalid 'to' address".to_string()))?;

    let mut transaction = TransactionRequest::new().to(to_address);

    if let Some(value_str) = &req.value {
        let value = U256::from_dec_str(value_str)
            .map_err(|_| AppError::Validation("Invalid value format".to_string()))?;
        transaction = transaction.value(value);
    }

    if let Some(data_str) = &req.data {
        let data = hex::decode(data_str.strip_prefix("0x").unwrap_or(data_str))
            .map_err(|_| AppError::Validation("Invalid data format".to_string()))?;
        transaction = transaction.data(data);
    }

    let gas_estimate = wallet_service.estimate_gas(&transaction).await?;

    Ok(HttpResponse::Ok().json(json!({
        "gas_estimate": gas_estimate.to_string(),
        "gas_estimate_number": gas_estimate.as_u64()
    })))
}