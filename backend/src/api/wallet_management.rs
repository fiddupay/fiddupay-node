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
use sqlx::PgPool;

// ============================================================================
// Wallet Configuration Endpoints
// ============================================================================

pub async fn get_wallets(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
) -> impl IntoResponse {
    let wallet_service = WalletConfigService::new(state.db_pool.clone());
    
    // Determine sandbox mode from merchant record
    let sandbox_mode = match sqlx::query_scalar!("SELECT sandbox_mode FROM merchants WHERE id = $1", context.merchant_id)
        .fetch_one(&state.db_pool)
        .await {
            Ok(s) => s,
            Err(_) => false,
        };

    // Determine settlement mode to decide which configs to fetch
    let settlement_mode = match sqlx::query_scalar!("SELECT settlement_mode FROM merchants WHERE id = $1", context.merchant_id)
        .fetch_one(&state.db_pool)
        .await {
            Ok(s) => s,
            Err(_) => "managed".to_string(),
        };

    let result = if settlement_mode == "forwarding" {
        wallet_service.get_forwarding_configs(context.merchant_id, sandbox_mode).await
    } else {
        wallet_service.get_wallet_configs(context.merchant_id, sandbox_mode).await
    };

    match result {
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
        crypto_type: req.crypto_type.clone(),
        address: req.address.clone(),
        is_active: req.is_active,
    };
    
    // Determine sandbox mode from merchant record
    let sandbox_mode = match sqlx::query_scalar!("SELECT sandbox_mode FROM merchants WHERE id = $1", context.merchant_id)
        .fetch_one(&state.db_pool)
        .await {
            Ok(s) => s,
            Err(_) => false,
        };

    match wallet_service.configure_address_only(context.merchant_id, sandbox_mode, configure_request).await {
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
    
    // Determine sandbox mode from merchant record
    let sandbox_mode = match sqlx::query_scalar!("SELECT sandbox_mode FROM merchants WHERE id = $1", context.merchant_id)
        .fetch_one(&state.db_pool)
        .await {
            Ok(s) => s,
            Err(_) => false,
        };

    match wallet_service.generate_wallet(context.merchant_id, sandbox_mode, req).await {
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
    
    // Determine sandbox mode from merchant record
    let sandbox_mode = match sqlx::query_scalar!("SELECT sandbox_mode FROM merchants WHERE id = $1", context.merchant_id)
        .fetch_one(&state.db_pool)
        .await {
            Ok(s) => s,
            Err(_) => false,
        };

    match wallet_service.import_wallet(context.merchant_id, sandbox_mode, req).await {
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
    
    // Determine sandbox mode from merchant record
    let sandbox_mode = match sqlx::query_scalar!("SELECT sandbox_mode FROM merchants WHERE id = $1", context.merchant_id)
        .fetch_one(&state.db_pool)
        .await {
            Ok(s) => s,
            Err(_) => false,
        };

    match wallet_service.export_private_key(context.merchant_id, sandbox_mode, req).await {
        Ok(private_key) => (StatusCode::OK, Json(json!({
            "private_key": private_key,
            "warning": "⚠️ Keep this private key secure. Anyone with access can control your funds."
        }))).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, Json(json!({
            "error": e.to_string()
        }))).into_response(),
    }
}

pub async fn delete_wallet(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Path(crypto_type): Path<String>,
) -> impl IntoResponse {
    let wallet_service = WalletConfigService::new(state.db_pool.clone());

    // Lookup merchant info
    let merchant_info = sqlx::query!(
        "SELECT settlement_mode, sandbox_mode FROM merchants WHERE id = $1",
        context.merchant_id
    )
    .fetch_optional(&state.db_pool)
    .await
    .ok()
    .flatten();

    let settlement_mode = merchant_info.as_ref().map(|m| m.settlement_mode.clone()).unwrap_or_else(|| "managed".to_string());
    let sandbox_mode = merchant_info.map(|m| m.sandbox_mode).unwrap_or(false);

    let result = if settlement_mode == "forwarding" {
        wallet_service.delete_forwarding_config(context.merchant_id, sandbox_mode, crypto_type).await
    } else {
        wallet_service.delete_wallet_config(context.merchant_id, sandbox_mode, crypto_type).await
    };

    match result {
        Ok(_) => (StatusCode::OK, Json(json!({
            "message": "Wallet configuration removed successfully"
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
    
    // Determine sandbox mode from merchant record
    let sandbox_mode = match sqlx::query_scalar!("SELECT sandbox_mode FROM merchants WHERE id = $1", context.merchant_id)
        .fetch_one(&state.db_pool)
        .await {
            Ok(s) => s,
            Err(_) => false,
        };

    match wallet_service.validate_gas_for_withdrawal(
        context.merchant_id,
        sandbox_mode,
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
    
    // Determine sandbox mode from merchant record
    let sandbox_mode = match sqlx::query_scalar!("SELECT sandbox_mode FROM merchants WHERE id = $1", context.merchant_id)
        .fetch_one(&state.db_pool)
        .await {
            Ok(s) => s,
            Err(_) => false,
        };

    match wallet_service.can_withdraw(context.merchant_id, sandbox_mode, crypto_type, rust_decimal::Decimal::ZERO).await {
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
    pub crypto_type: String,
    pub address: String,
    pub is_active: Option<bool>,
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

#[derive(Debug, Deserialize)]
pub struct UnifiedWalletSetupRequest {
    pub crypto_type: String,
    pub mode: String, // "address", "generate", "import"
    pub address: Option<String>,
    pub private_key: Option<String>,
    pub is_active: Option<bool>,
    pub enable_all_evm: Option<bool>,
}

pub async fn setup_wallet(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Json(req): Json<UnifiedWalletSetupRequest>,
) -> impl IntoResponse {
    let wallet_service = WalletConfigService::new(state.db_pool.clone());
    
    match req.mode.as_str() {
        "address" => {
            if let Some(address) = req.address {
                // Check settlement mode to decide which table to write to
                let merchant_info = sqlx::query!(
                    "SELECT settlement_mode, sandbox_mode FROM merchants WHERE id = $1",
                    context.merchant_id
                )
                .fetch_optional(&state.db_pool)
                .await
                .ok()
                .flatten();
            
                let settlement_mode = merchant_info.as_ref().map(|m| m.settlement_mode.clone()).unwrap_or_else(|| "managed".to_string());
                let sandbox_mode = merchant_info.map(|m| m.sandbox_mode).unwrap_or(false);

                if settlement_mode == "forwarding" {
                    // Write to merchant_forwarding_wallets
                    let crypto_type = match CryptoType::from_string(&req.crypto_type) {
                        Ok(ct) => ct,
                        Err(e) => return (StatusCode::BAD_REQUEST, Json(json!({
                            "error": format!("{}", e)
                        }))).into_response(),
                    };
                    match wallet_service.set_forwarding_address(
                        context.merchant_id,
                        crypto_type,
                        address,
                        req.is_active.unwrap_or(true),
                        sandbox_mode,
                    ).await {
                        Ok(config) => (StatusCode::OK, Json(json!({
                            "wallet": config,
                            "mode": "address",
                            "message": "Forwarding address configured successfully."
                        }))).into_response(),
                        Err(e) => (StatusCode::BAD_REQUEST, Json(json!({"error": e.to_string()}))).into_response(),
                    }
                } else {
                    // Write to merchant_wallets (managed / imported)
                    let configure_request = ConfigureWalletRequest {
                        crypto_type: req.crypto_type,
                        address,
                        is_active: req.is_active,
                    };
                    match wallet_service.configure_address_only(context.merchant_id, sandbox_mode, configure_request).await {
                        Ok(config) => (StatusCode::OK, Json(json!({
                            "wallet": config,
                            "mode": "address",
                            "message": "Address-only wallet configured successfully."
                        }))).into_response(),
                        Err(e) => (StatusCode::BAD_REQUEST, Json(json!({"error": e.to_string()}))).into_response(),
                    }
                }
            } else {
                (StatusCode::BAD_REQUEST, Json(json!({"error": "Address is required for mode 'address'"}))).into_response()
            }
        },
        "generate" => {
            // Look up settlement mode to control behavior
            let merchant_info = sqlx::query!("SELECT settlement_mode, sandbox_mode FROM merchants WHERE id = $1", context.merchant_id)
                .fetch_optional(&state.db_pool)
                .await
                .ok()
                .flatten();
            
            let settlement_mode = merchant_info.as_ref().map(|m| m.settlement_mode.clone()).unwrap_or_else(|| "managed".to_string());
            let sandbox_mode = merchant_info.map(|m| m.sandbox_mode).unwrap_or(false);

            // Imported mode: merchants must use 'import' — they cannot generate wallets
            if settlement_mode == "imported" {
                return (StatusCode::BAD_REQUEST, Json(json!({
                    "error": "Wallet generation is not available in imported mode. Please use 'import' to provide your private key."
                }))).into_response();
            }

            let generate_request = GenerateWalletRequest {
                crypto_type: req.crypto_type,
                enable_all_evm: req.enable_all_evm,
            };

            let is_managed = settlement_mode == "managed";

            let result = if is_managed {
                wallet_service.generate_wallet_managed(context.merchant_id, sandbox_mode, generate_request).await
            } else {
                wallet_service.generate_wallet(context.merchant_id, sandbox_mode, generate_request).await
            };

            match result {
                Ok(response) => {
                    let msg = if is_managed {
                        "Wallet generated successfully. Keys are managed by the platform."
                    } else {
                        "Wallet generated successfully."
                    };
                    (StatusCode::CREATED, Json(json!({
                        "wallet": response,
                        "mode": "generate",
                        "managed": is_managed,
                        "message": msg
                    }))).into_response()
                }
                Err(e) => (StatusCode::BAD_REQUEST, Json(json!({"error": e.to_string()}))).into_response(),
            }
        },
        "import" => {
            // Look up settlement mode to control behavior
            let merchant_info = sqlx::query!("SELECT settlement_mode, sandbox_mode FROM merchants WHERE id = $1", context.merchant_id)
                .fetch_optional(&state.db_pool)
                .await
                .ok()
                .flatten();
            
            let settlement_mode = merchant_info.as_ref().map(|m| m.settlement_mode.clone()).unwrap_or_else(|| "managed".to_string());
            let sandbox_mode = merchant_info.map(|m| m.sandbox_mode).unwrap_or(false);

            if settlement_mode == "managed" {
                return (StatusCode::BAD_REQUEST, Json(json!({
                    "error": "Wallet import is not available in managed mode. Please use 'generate' to create a wallet."
                }))).into_response();
            }

            if let Some(private_key) = req.private_key {
                match wallet_service.import_wallet(context.merchant_id, sandbox_mode, ImportWalletRequest {
                    crypto_type: req.crypto_type,
                    private_key,
                    is_active: req.is_active,
                    enable_all_evm: req.enable_all_evm,
                }).await {
                    Ok(config) => (StatusCode::OK, Json(json!({
                        "wallet": config,
                        "mode": "import",
                        "message": "Wallet imported successfully."
                    }))).into_response(),
                    Err(e) => (StatusCode::BAD_REQUEST, Json(json!({"error": e.to_string()}))).into_response(),
                }
            } else {
                (StatusCode::BAD_REQUEST, Json(json!({"error": "Private key is required for mode 'import'"}))).into_response()
            }
        },
        _ => (StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid mode. Use 'address', 'generate', or 'import'."}))).into_response(),
    }
}

// ============================================================================
// Wallet Balance & Volume Endpoint
// ============================================================================

pub async fn get_wallet_balances(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
) -> impl IntoResponse {
    // Lookup merchant sandbox_mode and settlement_mode
    let merchant_info = sqlx::query!(
        "SELECT sandbox_mode, settlement_mode FROM merchants WHERE id = $1",
        context.merchant_id
    )
    .fetch_optional(&state.db_pool)
    .await
    .ok()
    .flatten();

    let sandbox_mode = merchant_info.as_ref().map(|m| m.sandbox_mode).unwrap_or(false);
    let settlement_mode = merchant_info.as_ref().map(|m| m.settlement_mode.clone()).unwrap_or_else(|| "managed".to_string());

    // Get wallet configs with their balances and transaction volume, isolated by sandbox mode
    // We only show balances for "managed" or "imported" mode, because "forwarding" mode wallets don't hold a balance on our platform.
    // However, if we are in forwarding mode, we still query merchant_forwarding_wallets to show the user their wallets, but with 0 balance.
    let is_forwarding = settlement_mode == "forwarding";
    
    let result = if is_forwarding {
        // Forwarding wallets don't have managed balances tracking, so return 0s
        sqlx::query_as::<_, WalletBalanceRow>(
            r#"
            SELECT
                crypto_type,
                network,
                address,
                is_active,
                0::numeric as available_balance,
                0::numeric as reserved_balance,
                0::numeric as total_balance,
                0::bigint as transaction_count,
                0::numeric as total_volume_crypto
            FROM merchant_forwarding_wallets
            WHERE merchant_id = $1 AND sandbox_mode = $2
            ORDER BY crypto_type
            "#
        )
        .bind(context.merchant_id)
        .bind(sandbox_mode)
        .fetch_all(&state.db_pool)
        .await
    } else {
        sqlx::query_as::<_, WalletBalanceRow>(
            r#"
            SELECT
                mw.crypto_type,
                mw.network,
                mw.address,
                mw.is_active,
                COALESCE(mb.available_balance, 0::numeric) as available_balance,
                COALESCE(mb.reserved_balance, 0::numeric) as reserved_balance,
                COALESCE(mb.total_balance, 0::numeric) as total_balance,
                COALESCE(tx_stats.tx_count, 0::bigint) as transaction_count,
                COALESCE(tx_stats.total_volume, 0::numeric) as total_volume_crypto
            FROM merchant_wallets mw
            LEFT JOIN merchant_balances mb
                ON mb.merchant_id = mw.merchant_id 
               AND mb.crypto_type = mw.crypto_type 
               AND mb.sandbox_mode = mw.sandbox_mode
            LEFT JOIN LATERAL (
                SELECT
                    COUNT(*)::bigint as tx_count,
                    COALESCE(SUM(amount_crypto), 0) as total_volume
                FROM payment_transactions
                WHERE merchant_id = mw.merchant_id
                  AND crypto_type = mw.crypto_type
                  AND status = 'CONFIRMED'
                  AND sandbox_mode = mw.sandbox_mode
            ) tx_stats ON true
            WHERE mw.merchant_id = $1 AND mw.sandbox_mode = $2
            ORDER BY mw.crypto_type
            "#
        )
        .bind(context.merchant_id)
        .bind(sandbox_mode)
        .fetch_all(&state.db_pool)
        .await
    };

    match result {
        Ok(wallets) => {
            let wallet_data: Vec<serde_json::Value> = wallets.iter().map(|w| json!({
                "crypto_type": w.crypto_type,
                "network": w.network,
                "address": w.address,
                "is_active": w.is_active,
                "available_balance": w.available_balance.to_string(),
                "reserved_balance": w.reserved_balance.to_string(),
                "total_balance": w.total_balance.to_string(),
                "transaction_count": w.transaction_count,
                "total_volume_crypto": w.total_volume_crypto.to_string()
            })).collect();

            (StatusCode::OK, Json(json!({
                "wallets": wallet_data
            }))).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to get wallet balances: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                "error": format!("Failed to get wallet balances: {}", e)
            }))).into_response()
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct WalletBalanceRow {
    pub crypto_type: String,
    pub network: String,
    pub address: String,
    pub is_active: bool,
    pub available_balance: Decimal,
    pub reserved_balance: Decimal,
    pub total_balance: Decimal,
    pub transaction_count: i64,
    pub total_volume_crypto: Decimal,
}
