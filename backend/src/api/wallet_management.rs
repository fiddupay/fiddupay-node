// Wallet Management API Endpoints
// Handles 3-mode wallet configuration and management

use crate::api::state::AppState;
use crate::error::ServiceError;
use crate::middleware::auth::{require_kyc_tier, MerchantContext};
use crate::payment::models::CryptoType;
use crate::services::wallet_config_service::{
    ConfigureWalletRequest, GenerateWalletRequest, WalletConfigService,
};
use crate::services::withdrawal_processor::WithdrawalProcessor;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};
use rust_decimal::Decimal;
use serde::Deserialize;
use serde_json::json;
use sqlx::{PgPool, Row};

// Helper: get only settlement_mode (sandbox_mode should come from context headers)
async fn get_merchant_settlement_mode(pool: &PgPool, merchant_id: i64) -> String {
    let row = sqlx::query("SELECT settlement_mode FROM merchants WHERE id = $1")
        .bind(merchant_id)
        .fetch_optional(pool)
        .await;

    match row {
        Ok(Some(r)) => r.get("settlement_mode"),
        _ => "managed".to_string(),
    }
}

// ============================================================================
// Wallet Configuration Endpoints
// ============================================================================

pub async fn get_wallets(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
) -> impl IntoResponse {
    let wallet_service = WalletConfigService::new(state.db_pool.clone());

    let sandbox_mode = context.sandbox_mode;
    let settlement_mode = get_merchant_settlement_mode(&state.db_pool, context.merchant_id).await;

    let result = if settlement_mode == "forwarding" {
        wallet_service
            .get_forwarding_configs(context.merchant_id, sandbox_mode)
            .await
    } else {
        wallet_service
            .get_wallet_configs(context.merchant_id, sandbox_mode)
            .await
    };

    match result {
        Ok(configs) => (
            StatusCode::OK,
            Json(json!({
                "wallets": configs,
                "supported_networks": ["ethereum", "bsc", "polygon", "arbitrum", "solana"]
            })),
        )
            .into_response(),
        Err(e) => ServiceError::Internal(e.to_string()).into_response(),
    }
}

pub async fn delete_wallet(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Path(crypto_type): Path<String>,
) -> impl IntoResponse {
    let wallet_service = WalletConfigService::new(state.db_pool.clone());

    let sandbox_mode = context.sandbox_mode;
    let settlement_mode = get_merchant_settlement_mode(&state.db_pool, context.merchant_id).await;

    let result = if settlement_mode == "forwarding" {
        wallet_service
            .delete_forwarding_config(context.merchant_id, sandbox_mode, crypto_type.clone())
            .await
    } else {
        wallet_service
            .delete_wallet_config(context.merchant_id, sandbox_mode, crypto_type.clone())
            .await
    };

    match result {
        Ok(_) => {
            // Log wallet deletion and trace
            let _ = state
                .audit_service
                .log_event(
                    context.merchant_id,
                    "wallet_deletion",
                    Some(&format!("Removed wallet configuration for {}", crypto_type)),
                    Some(json!({"crypto_type": crypto_type})),
                )
                .await;
            tracing::info!(
                "EVENT: wallet_deletion | Merchant: {} | Crypto: {}",
                context.merchant_id,
                crypto_type
            );

            (
                StatusCode::OK,
                Json(json!({
                    "message": "Wallet configuration removed successfully"
                })),
            )
                .into_response()
        }
        Err(e) => ServiceError::BadRequest(e.to_string()).into_response(),
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

    let sandbox_mode = context.sandbox_mode;

    match wallet_service
        .validate_gas_for_withdrawal(
            context.merchant_id,
            sandbox_mode,
            params.crypto_type,
            params.amount,
        )
        .await
    {
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
        Err(e) => ServiceError::BadRequest(e.to_string()).into_response(),
    }
}

pub async fn get_gas_estimates(
    State(state): State<AppState>,
    Extension(_context): Extension<MerchantContext>,
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
        Err(e) => {
            ServiceError::Internal(format!("Failed to fetch gas estimates: {}", e)).into_response()
        }
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

    let sandbox_mode = context.sandbox_mode;

    match wallet_service
        .can_withdraw(
            context.merchant_id,
            sandbox_mode,
            crypto_type,
            rust_decimal::Decimal::ZERO,
        )
        .await
    {
        Ok(can_withdraw) => {
            let message = if can_withdraw {
                "Withdrawal available - wallet has private key access"
            } else {
                "Withdrawal not available - configure wallet (generate)"
            };

            (
                StatusCode::OK,
                Json(json!({
                    "crypto_type": crypto_type,
                    "can_withdraw": can_withdraw,
                    "message": message
                })),
            )
                .into_response()
        }
        Err(e) => ServiceError::Internal(e.to_string()).into_response(),
    }
}

// ============================================================================
// Withdrawal Processing
// ============================================================================

pub async fn process_withdrawal(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Path(withdrawal_id): Path<String>,
    Json(_req): Json<ProcessWithdrawalRequest>,
) -> impl IntoResponse {
    // 0. KYC Check: Must be Tier 1 to process Live withdrawals
    if !context.sandbox_mode {
        if let Err(e) = require_kyc_tier(&context, 1) {
            return e.into_response();
        }
    }
    let processor = WithdrawalProcessor::new(
        state.db_pool.clone(),
        state.config.clone(),
        state.notification_service.clone(),
        state.blockchain_sender.clone(),
        state.balance_service.clone(),
    );

    match processor.process_withdrawal(&withdrawal_id).await {
        Ok(result) => {
            // Log audit event
            let _ = state
                .audit_service
                .log_event(
                    context.merchant_id,
                    "withdrawal_processing",
                    Some(&format!("Processed withdrawal {}", withdrawal_id)),
                    Some(json!({
                        "withdrawal_id": withdrawal_id
                    })),
                )
                .await;

            (
                StatusCode::OK,
                Json(json!({
                    "withdrawal": result,
                    "message": "Withdrawal processed successfully"
                })),
            )
                .into_response()
        }
        Err(e) => ServiceError::BadRequest(e.to_string()).into_response(),
    }
}

// ============================================================================
// Request/Response Types
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct WalletBalancesQuery {
    pub exclude_stats: Option<bool>,
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
    pub mode: String, // "address", "generate"
    pub address: Option<String>,
    pub is_active: Option<bool>,
}

pub async fn setup_wallet(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Json(req): Json<UnifiedWalletSetupRequest>,
) -> impl IntoResponse {
    let wallet_service = WalletConfigService::new(state.db_pool.clone());

    // 1. Requirement: Must be Tier 1 (ID Verified) to setup Live wallets
    if !context.sandbox_mode {
        if let Err(e) = require_kyc_tier(&context, 1) {
            return e.into_response();
        }
    }

    match req.mode.as_str() {
        "address" => {
            if let Some(address) = req.address {
                let sandbox_mode = context.sandbox_mode;
                let settlement_mode =
                    get_merchant_settlement_mode(&state.db_pool, context.merchant_id).await;

                if settlement_mode == "forwarding" {
                    let crypto_type = match CryptoType::from_string(&req.crypto_type) {
                        Ok(ct) => ct,
                        Err(e) => {
                            return ServiceError::BadRequest(format!("{}", e)).into_response()
                        }
                    };
                    match wallet_service
                        .set_forwarding_address(
                            context.merchant_id,
                            crypto_type,
                            address.clone(),
                            req.is_active.unwrap_or(true),
                            sandbox_mode,
                        )
                        .await
                    {
                        Ok(config) => {
                            // Log forwarding setup and trace
                            let _ = state
                                .audit_service
                                .log_event(
                                    context.merchant_id,
                                    "wallet_setup_forwarding",
                                    Some(&format!(
                                        "Configured {} forwarding address",
                                        req.crypto_type
                                    )),
                                    Some(json!({
                                        "crypto_type": req.crypto_type,
                                        "address": address,
                                        "is_active": req.is_active.unwrap_or(true)
                                    })),
                                )
                                .await;
                            tracing::info!(
                                "EVENT: wallet_setup_forwarding | Merchant: {} | Crypto: {}",
                                context.merchant_id,
                                req.crypto_type
                            );

                            (
                                StatusCode::OK,
                                Json(json!({
                                    "wallet": config,
                                    "mode": "address",
                                    "message": "Forwarding address configured successfully."
                                })),
                            )
                                .into_response()
                        }
                        Err(e) => (
                            StatusCode::BAD_REQUEST,
                            Json(json!({"error": e.to_string()})),
                        )
                            .into_response(),
                    }
                } else {
                    let configure_request = ConfigureWalletRequest {
                        crypto_type: req.crypto_type.clone(),
                        address: address.clone(),
                        is_active: req.is_active,
                    };
                    match wallet_service
                        .configure_address_only(
                            context.merchant_id,
                            sandbox_mode,
                            configure_request,
                        )
                        .await
                    {
                        Ok(config) => {
                            // Log address-only setup and trace
                            let _ = state
                                .audit_service
                                .log_event(
                                    context.merchant_id,
                                    "wallet_setup_address_only",
                                    Some(&format!(
                                        "Configured {} address-only wallet",
                                        req.crypto_type
                                    )),
                                    Some(json!({
                                        "crypto_type": req.crypto_type,
                                        "address": address,
                                        "is_active": req.is_active
                                    })),
                                )
                                .await;
                            tracing::info!(
                                "EVENT: wallet_setup_address_only | Merchant: {} | Crypto: {}",
                                context.merchant_id,
                                req.crypto_type
                            );

                            (
                                StatusCode::OK,
                                Json(json!({
                                    "wallet": config,
                                    "mode": "address",
                                    "message": "Address-only wallet configured successfully."
                                })),
                            )
                                .into_response()
                        }
                        Err(e) => ServiceError::BadRequest(e.to_string()).into_response(),
                    }
                }
            } else {
                ServiceError::BadRequest("Address is required for mode 'address'".to_string())
                    .into_response()
            }
        }
        "generate" => {
            let sandbox_mode = context.sandbox_mode;
            let settlement_mode =
                get_merchant_settlement_mode(&state.db_pool, context.merchant_id).await;

            let generate_request = GenerateWalletRequest {
                crypto_type: req.crypto_type.clone(),
            };

            let is_managed = settlement_mode == "managed";

            let result = if is_managed {
                wallet_service
                    .generate_wallet_managed(context.merchant_id, sandbox_mode, generate_request)
                    .await
            } else {
                wallet_service
                    .generate_wallet(context.merchant_id, sandbox_mode, generate_request)
                    .await
            };

            match result {
                Ok(response) => {
                    let msg = if is_managed {
                        "Wallet generated successfully. Keys are managed by the platform."
                    } else {
                        "Wallet generated successfully."
                    };

                    // Log wallet generation and trace
                    let _ = state
                        .audit_service
                        .log_event(
                            context.merchant_id,
                            "wallet_generation",
                            Some(&format!(
                                "Generated new {} wallet ({})",
                                req.crypto_type,
                                if is_managed {
                                    "managed"
                                } else {
                                    "user-managed"
                                }
                            )),
                            Some(json!({
                                "crypto_type": req.crypto_type,
                                "managed": is_managed
                            })),
                        )
                        .await;
                    tracing::info!(
                        "EVENT: wallet_generation | Merchant: {} | Crypto: {} | Managed: {}",
                        context.merchant_id,
                        req.crypto_type,
                        is_managed
                    );

                    (
                        StatusCode::CREATED,
                        Json(json!({
                            "wallet": response,
                            "mode": "generate",
                            "managed": is_managed,
                            "message": msg
                        })),
                    )
                        .into_response()
                }
                Err(e) => ServiceError::BadRequest(e.to_string()).into_response(),
            }
        }
        _ => ServiceError::BadRequest("Invalid mode. Use 'address' or 'generate'.".to_string())
            .into_response(),
    }
}

// ============================================================================
// Wallet Balance & Volume Endpoint
// ============================================================================

pub async fn get_wallet_balances(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Query(params): Query<WalletBalancesQuery>,
) -> impl IntoResponse {
    let sandbox_mode = context.sandbox_mode;
    let settlement_mode = get_merchant_settlement_mode(&state.db_pool, context.merchant_id).await;
    let exclude_stats = params.exclude_stats.unwrap_or(false);

    // 1. Check Redis Cache First
    let cache_key = format!(
        "merchant_balances:{}:{}:{}",
        context.merchant_id, sandbox_mode, exclude_stats
    );
    if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
        let cached: redis::RedisResult<String> = redis::cmd("GET")
            .arg(&cache_key)
            .query_async(&mut conn)
            .await;
        if let Ok(json_str) = cached {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&json_str) {
                return (StatusCode::OK, Json(val)).into_response();
            }
        }
    }

    tracing::info!(
        "get_wallet_balances (CACHE MISS): merchant_id={}, sandbox_mode={}, settlement_mode={}",
        context.merchant_id,
        sandbox_mode,
        settlement_mode
    );

    let is_forwarding = settlement_mode == "forwarding";

    let result: Result<Vec<WalletBalanceRow>, sqlx::Error> = if is_forwarding {
        sqlx::query_as::<_, WalletBalanceRow>(
            r#"
            SELECT
                crypto_type,
                network,
                address,
                is_active,
                0::numeric as "available_balance",
                0::numeric as "reserved_balance",
                0::numeric as "total_balance",
                0::bigint as "transaction_count",
                0::numeric as "total_volume_crypto"
            FROM merchant_forwarding_wallets
            WHERE merchant_id = $1 AND sandbox_mode = $2 AND address != ''
            ORDER BY crypto_type
            "#,
        )
        .bind(context.merchant_id)
        .bind(sandbox_mode)
        .fetch_all(&state.db_pool)
        .await
    } else {
        let query = if exclude_stats {
            r#"
            SELECT
                mw.crypto_type,
                mw.network,
                mw.address,
                mw.is_active,
                COALESCE(mb.available_balance, 0::numeric) as "available_balance",
                COALESCE(mb.reserved_balance, 0::numeric) as "reserved_balance",
                (COALESCE(mb.available_balance, 0::numeric) + COALESCE(mb.reserved_balance, 0::numeric)) as "total_balance",
                0::bigint as "transaction_count",
                0::numeric as "total_volume_crypto"
            FROM merchant_wallets mw
            LEFT JOIN merchant_balances mb
                ON mw.merchant_id = mb.merchant_id
               AND mw.crypto_type = mb.crypto_type
               AND mw.sandbox_mode = mb.sandbox_mode
            WHERE mw.merchant_id = $1 AND mw.sandbox_mode = $2 AND mw.address != ''
            ORDER BY mw.crypto_type
            "#
        } else {
            r#"
            SELECT
                mw.crypto_type,
                mw.network,
                mw.address,
                mw.is_active,
                COALESCE(mb.available_balance, 0::numeric) as "available_balance",
                COALESCE(mb.reserved_balance, 0::numeric) as "reserved_balance",
                (COALESCE(mb.available_balance, 0::numeric) + COALESCE(mb.reserved_balance, 0::numeric)) as "total_balance",
                COALESCE(tx_stats.tx_count, 0::bigint) as "transaction_count",
                COALESCE(tx_stats.total_volume, 0::numeric) as "total_volume_crypto"
            FROM merchant_wallets mw
            LEFT JOIN merchant_balances mb
                ON mw.merchant_id = mb.merchant_id
               AND mw.crypto_type = mb.crypto_type
               AND mw.sandbox_mode = mb.sandbox_mode
            LEFT JOIN (
                SELECT
                    merchant_id,
                    crypto_type,
                    sandbox_mode,
                    COUNT(*)::bigint as tx_count,
                    COALESCE(SUM(amount), 0::numeric) as total_volume
                FROM payment_transactions
                WHERE merchant_id = $1 AND sandbox_mode = $2 AND status = 'CONFIRMED'
                GROUP BY merchant_id, crypto_type, sandbox_mode
            ) tx_stats ON mw.merchant_id = tx_stats.merchant_id 
                      AND mw.crypto_type = tx_stats.crypto_type 
                      AND mw.sandbox_mode = tx_stats.sandbox_mode
            WHERE mw.merchant_id = $1 AND mw.sandbox_mode = $2 AND mw.address != ''
            ORDER BY mw.crypto_type
            "#
        };

        sqlx::query_as::<_, WalletBalanceRow>(query)
            .bind(context.merchant_id)
            .bind(sandbox_mode)
            .fetch_all(&state.db_pool)
            .await
    };

    match result {
        Ok(wallets) => {
            use futures::future::join_all;
            use std::collections::HashMap;

            // 1. Gather all unique crypto types
            let mut unique_cryptos = std::collections::HashSet::new();
            for w in &wallets {
                unique_cryptos.insert(w.crypto_type.clone());
            }

            // 2. Fetch required prices in parallel once
            let mut price_map = HashMap::new();
            let price_tasks = unique_cryptos.into_iter().map(|ct_str| {
                let state = state.clone();
                async move {
                    if let Ok(ct_enum) = crate::payment::models::CryptoType::from_string(&ct_str) {
                        let price = state.price_service.get_price(ct_enum).await.unwrap_or(0.0);
                        Some((ct_str, price))
                    } else {
                        None
                    }
                }
            });

            let price_results = join_all(price_tasks).await;
            for res in price_results.into_iter().flatten() {
                price_map.insert(res.0, res.1);
            }

            // 3. Process wallets using pre-fetched prices
            let mut total_available_usd = Decimal::ZERO;
            let mut total_reserved_usd = Decimal::ZERO;
            let mut overall_total_usd = Decimal::ZERO;

            let response_balances: Vec<_> = wallets
                .into_iter()
                .map(|w| {
                    let price = price_map.get(&w.crypto_type).copied().unwrap_or(0.0);
                    let price_dec = Decimal::from_f64_retain(price).unwrap_or(Decimal::ZERO);

                    let total_usd = (w.total_balance * price_dec).round_dp(2);
                    let available_usd = (w.available_balance * price_dec).round_dp(2);
                    let reserved_usd = (w.reserved_balance * price_dec).round_dp(2);
                    let total_volume_usd = (w.total_volume_crypto * price_dec).round_dp(2);

                    total_available_usd += available_usd;
                    total_reserved_usd += reserved_usd;
                    overall_total_usd += total_usd;

                    json!({
                        "crypto_type": w.crypto_type,
                        "network": w.network,
                        "address": w.address,
                        "is_active": w.is_active,
                        "available_balance": w.available_balance.to_string(),
                        "available_usd": available_usd.to_string(),
                        "reserved_balance": w.reserved_balance.to_string(),
                        "reserved_usd": reserved_usd.to_string(),
                        "total_balance": w.total_balance.to_string(),
                        "total_usd": total_usd.to_string(),
                        "balance_usd": total_usd.to_string(), // Frontend legacy compatibility
                        "transaction_count": w.transaction_count,
                        "total_volume_crypto": w.total_volume_crypto.to_string(),
                        "total_volume_usd": total_volume_usd.to_string()
                    })
                })
                .collect();

            let final_response = json!({
                "total_usd": overall_total_usd.to_string(),
                "available_usd": total_available_usd.to_string(),
                "reserved_usd": total_reserved_usd.to_string(),
                "balances": response_balances
            });

            // 4. Save to Cache for next time (5 minute TTL)
            if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
                if let Ok(json_str) = serde_json::to_string(&final_response) {
                    let _: redis::RedisResult<()> = redis::cmd("SETEX")
                        .arg(&cache_key)
                        .arg(300) // 5 minutes
                        .arg(json_str)
                        .query_async(&mut conn)
                        .await;
                }
            }

            (StatusCode::OK, Json(final_response)).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to get wallet balances: {:?}", e);
            ServiceError::Internal(format!("Failed to get wallet balances: {}", e)).into_response()
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
