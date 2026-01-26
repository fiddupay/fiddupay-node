// Wallet Management API Endpoints
// Handles 3-mode wallet configuration and management

use crate::api::state::AppState;
use crate::middleware::auth::MerchantContext;
use crate::services::wallet_config_service::{
    WalletConfigService, ConfigureWalletRequest, GenerateWalletRequest, 
    ImportWalletRequest, ExportKeyRequest, GasValidationResult
};
use crate::services::withdrawal_processor::WithdrawalProcessor;
use crate::payment::models::CryptoType;
use axum::{
    extract::{State, Path, Query},
    http::StatusCode,
    response::IntoResponse,
    Json, Extension,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use rust_decimal::Decimal;

// ============================================================================
// Wallet Configuration Endpoints
// ============================================================================

pub async fn get_wallet_configs(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
) -> impl IntoResponse {
    let wallet_service = WalletConfigService::new(state.db_pool.clone());
    
    match wallet_service.get_wallet_configs(context.merchant_id).await {
        Ok(configs) => (StatusCode::OK, Json(json!({
            "wallets": configs,
            "supported_networks": ["ethereum", "bsc", "polygon", "arbitrum", "solana"]
        }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
            "error": e.to_string()
        }))).into_response(),
    }
}

pub async fn configure_address_only_wallet(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Json(req): Json<ConfigureAddressRequest>,
) -> impl IntoResponse {
    let wallet_service = WalletConfigService::new(state.db_pool.clone());
    
    let configure_request = ConfigureWalletRequest {
        crypto_type: req.network.clone(),
        address: req.address.clone(),
    };
    
    match wallet_service.configure_address_only(context.merchant_id, configure_request).await {
        Ok(config) => (StatusCode::OK, Json(json!({
            "wallet": config,
            "message": "Address-only wallet configured successfully. No withdrawal capability."
        }))).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, Json(json!({
            "error": e.to_string()
        }))).into_response(),
    }
}

pub async fn generate_wallet(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Json(req): Json<GenerateWalletRequest>,
) -> impl IntoResponse {
    let wallet_service = WalletConfigService::new(state.db_pool.clone());
    
    match wallet_service.generate_wallet(context.merchant_id, req).await {
        Ok(response) => (StatusCode::CREATED, Json(json!({
            "wallet": response,
            "message": "Wallet generated successfully. Save the private key securely - it won't be shown again."
        }))).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, Json(json!({
            "error": e.to_string()
        }))).into_response(),
    }
}

pub async fn import_wallet(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Json(req): Json<ImportWalletRequest>,
) -> impl IntoResponse {
    let wallet_service = WalletConfigService::new(state.db_pool.clone());
    
    match wallet_service.import_wallet(context.merchant_id, req).await {
        Ok(config) => (StatusCode::OK, Json(json!({
            "wallet": config,
            "message": "Private key imported successfully. Withdrawal capability enabled."
        }))).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, Json(json!({
            "error": e.to_string()
        }))).into_response(),
    }
}

pub async fn export_private_key(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Json(req): Json<ExportKeyRequest>,
) -> impl IntoResponse {
    let wallet_service = WalletConfigService::new(state.db_pool.clone());
    
    match wallet_service.export_private_key(context.merchant_id, req).await {
        Ok(private_key) => (StatusCode::OK, Json(json!({
            "private_key": private_key,
            "warning": "⚠️ Keep this private key secure. Anyone with access can control your funds."
        }))).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, Json(json!({
            "error": e.to_string()
        }))).into_response(),
    }
}

// ============================================================================
// Gas Fee Validation Endpoints
// ============================================================================

pub async fn check_gas_requirements(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Query(params): Query<GasCheckQuery>,
) -> impl IntoResponse {
    let wallet_service = WalletConfigService::new(state.db_pool.clone());
    
    match wallet_service.validate_gas_for_withdrawal(
        context.merchant_id,
        params.crypto_type,
        params.amount,
    ).await {
        Ok(result) => {
            let response = if result.valid {
                json!({
                    "status": "sufficient",
                    "message": result.message,
                    "can_withdraw": true
                })
            } else {
                json!({
                    "status": "insufficient",
                    "message": result.message,
                    "can_withdraw": false
                })
            };
            
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => (StatusCode::BAD_REQUEST, Json(json!({
            "error": e.to_string()
        }))).into_response(),
    }
}

pub async fn get_gas_estimates(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
) -> impl IntoResponse {
    let gas_service = crate::services::gas_fee_service::GasFeeService::new(state.config.clone());
    
    match gas_service.get_all_gas_estimates().await {
        Ok(estimates) => {
            let response = json!({
                "networks": estimates,
                "notes": [
                    "Native currencies (ETH, BNB, MATIC, ARB, SOL) have gas auto-deducted from withdrawal amount",
                    "USDT withdrawals require separate gas deposit in the network's native currency",
                    "Gas estimates are fetched in real-time from blockchain networks",
                    "Actual costs may vary based on network congestion"
                ]
            });
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
            "error": format!("Failed to fetch gas estimates: {}", e)
        }))).into_response(),
    }
}

// ============================================================================
// Withdrawal Capability Check
// ============================================================================

pub async fn check_withdrawal_capability(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Path(crypto_type): Path<CryptoType>,
) -> impl IntoResponse {
    let wallet_service = WalletConfigService::new(state.db_pool.clone());
    
    match wallet_service.can_withdraw(context.merchant_id, crypto_type, rust_decimal::Decimal::ZERO).await {
        Ok(can_withdraw) => {
            let message = if can_withdraw {
                "Withdrawal available - wallet has private key access"
            } else {
                "Withdrawal not available - configure wallet with private key access (generate or import)"
            };
            
            (StatusCode::OK, Json(json!({
                "crypto_type": crypto_type,
                "can_withdraw": can_withdraw,
                "message": message
            }))).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
            "error": e.to_string()
        }))).into_response(),
    }
}

// ============================================================================
// Withdrawal Processing
// ============================================================================

pub async fn process_withdrawal(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Path(withdrawal_id): Path<String>,
    Json(req): Json<ProcessWithdrawalRequest>,
) -> impl IntoResponse {
    let processor = WithdrawalProcessor::new(state.db_pool.clone());
    
    match processor.process_withdrawal(&withdrawal_id).await {
        Ok(result) => (StatusCode::OK, Json(json!({
            "withdrawal": result,
            "message": "Withdrawal processed successfully"
        }))).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, Json(json!({
            "error": e.to_string()
        }))).into_response(),
    }
}

// ============================================================================
// Request/Response Types
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct ConfigureAddressRequest {
    pub network: String,
    pub address: String,
}

#[derive(Debug, Deserialize)]
pub struct GasCheckQuery {
    pub crypto_type: CryptoType,
    pub amount: Decimal,
}

#[derive(Debug, Deserialize)]
pub struct ProcessWithdrawalRequest {
    pub encryption_password: String,
}
