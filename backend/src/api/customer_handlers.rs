// Merchant Customer API Handlers
// Endpoints for managing and provisioning sub-account user wallets

use crate::api::state::AppState;
use crate::middleware::auth::MerchantContext;
use crate::models::merchant_customer::{
    CreateCustomerRequest, PayMerchantRequest, UpdateCustomerPermissionsRequest,
    UpdateCustomerStatusRequest,
};
use crate::services::merchant_customer_service::MerchantCustomerService;
use std::sync::Arc;
const _V1_PLACEHOLDER: &str = "v1";
use crate::error::ServiceError;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};
use serde_json::{json, Value};
use validator::Validate;

// Helper removed, now using state.merchant_service.verify_transaction_pin

pub async fn register_customer(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Json(req): Json<CreateCustomerRequest>,
) -> impl IntoResponse {
    // 1. Validate input
    if let Err(e) = req.validate() {
        return ServiceError::ValidationError(e.to_string()).into_response();
    }

    let service = MerchantCustomerService::new(
        state.db_pool.clone(),
        state.price_service.clone(),
        state.volume_tracking_service.clone(),
        state.notification_service.clone(),
        state.balance_service.clone(),
        Arc::new(state.config.clone()),
    );

    match service
        .register_customer(context.merchant_id, req, context.sandbox_mode)
        .await
    {
        Ok((customer, wallets)) => {
            // Log audit event
            let _ = state
                .audit_service
                .log_event(
                    context.merchant_id,
                    "customer_registration",
                    Some(&format!("Registered customer {}", customer.external_id)),
                    Some(json!({
                        "external_id": customer.external_id,
                        "email": customer.email,
                        "sandbox_mode": context.sandbox_mode
                    })),
                )
                .await;

            (
                StatusCode::CREATED,
                Json(json!({
                    "customer": customer,
                    "wallets": wallets,
                    "message": "Customer registered successfully with auto-provisioned wallets"
                })),
            )
                .into_response()
        }
        Err(e) => e.into_response(),
    }
}

pub async fn provision_customer_wallets(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Path(external_id): Path<String>,
    Json(req): Json<crate::models::merchant_customer::ProvisionWalletRequest>,
) -> impl IntoResponse {
    let service = MerchantCustomerService::new(
        state.db_pool.clone(),
        state.price_service.clone(),
        state.volume_tracking_service.clone(),
        state.notification_service.clone(),
        state.balance_service.clone(),
        Arc::new(state.config.clone()),
    );

    match service
        .provision_wallets(
            context.merchant_id,
            &external_id,
            req.networks.clone().unwrap_or_default(),
            context.sandbox_mode,
            false,
        )
        .await
    {
        Ok(wallets) => {
            // Log audit event
            let _ = state
                .audit_service
                .log_event(
                    context.merchant_id,
                    "customer_wallet_provision",
                    Some(&format!("Provisioned wallets for customer {}", external_id)),
                    Some(json!({
                        "external_id": external_id,
                        "networks": req.networks
                    })),
                )
                .await;

            let response_wallets: Vec<_> = wallets
                .iter()
                .map(|w| {
                    json!({
                        "crypto_type": w.crypto_type,
                        "network": w.network,
                        "address": w.address,
                        "created_at": w.created_at
                    })
                })
                .collect();

            (
                StatusCode::OK,
                Json(json!({
                    "external_id": external_id,
                    "wallets": response_wallets,
                    "message": "Customer wallets provisioned successfully across requested networks"
                })),
            )
                .into_response()
        }
        Err(e) => e.into_response(),
    }
}

#[derive(serde::Deserialize)]
pub struct BulkProvisionRequest {
    pub customer_ids: Option<Vec<String>>,
    pub all_customers: bool,
}

pub async fn bulk_provision_customer_wallets(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Json(req): Json<BulkProvisionRequest>,
) -> impl IntoResponse {
    let service = MerchantCustomerService::new(
        state.db_pool.clone(),
        state.price_service.clone(),
        state.volume_tracking_service.clone(),
        state.notification_service.clone(),
        state.balance_service.clone(),
        Arc::new(state.config.clone()),
    );

    match service
        .bulk_provision_wallets(
            context.merchant_id,
            req.customer_ids.clone(),
            req.all_customers,
            context.sandbox_mode,
        )
        .await
    {
        Ok(count) => {
            // Log audit event
            let _ = state
                .audit_service
                .log_event(
                    context.merchant_id,
                    "customer_bulk_wallet_provision",
                    Some(&format!("Bulk provisioned wallets for {} customers", count)),
                    Some(json!({
                        "customer_ids": req.customer_ids,
                        "all_customers": req.all_customers,
                        "count_success": count
                    })),
                )
                .await;

            (StatusCode::OK, Json(json!({
                "count": count,
                "message": format!("Successfully provisioned or regenerated wallets for {} customers", count)
            }))).into_response()
        }
        Err(e) => e.into_response(),
    }
}

pub async fn get_customer_balances(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Path(external_id): Path<String>,
) -> impl IntoResponse {
    let service = MerchantCustomerService::new(
        state.db_pool.clone(),
        state.price_service.clone(),
        state.volume_tracking_service.clone(),
        state.notification_service.clone(),
        state.balance_service.clone(),
        Arc::new(state.config.clone()),
    );

    match service
        .get_customer_balances(context.merchant_id, &external_id, context.sandbox_mode)
        .await
    {
        Ok(balances) => {
            use futures::future::join_all;
            use std::collections::HashMap;

            // 1. Gather all unique crypto types
            let mut unique_cryptos = std::collections::HashSet::new();
            for b in &balances {
                unique_cryptos.insert(b.crypto_type.clone());
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

            // 3. Process balances using pre-fetched prices
            let response_balances: Vec<_> = balances
                .into_iter()
                .map(|b| {
                    let price = price_map.get(&b.crypto_type).copied().unwrap_or(0.0);
                    let price_dec = rust_decimal::Decimal::from_f64_retain(price)
                        .unwrap_or(rust_decimal::Decimal::ZERO);

                    json!({
                        "id": b.id,
                        "customer_id": b.customer_id,
                        "merchant_id": b.merchant_id,
                        "crypto_type": b.crypto_type,
                        "available_balance": b.available_balance,
                        "available_balance_usd": (b.available_balance * price_dec).round_dp(2),
                        "locked_balance": b.locked_balance,
                        "locked_balance_usd": (b.locked_balance * price_dec).round_dp(2),
                        "total_balance": b.total_balance,
                        "total_balance_usd": (b.total_balance * price_dec).round_dp(2),
                        "last_updated_at": b.last_updated_at,
                        "sandbox_mode": b.sandbox_mode,
                    })
                })
                .collect();

            (
                StatusCode::OK,
                Json(json!({
                    "external_id": external_id,
                    "balances": response_balances
                })),
            )
                .into_response()
        }
        Err(e) => e.into_response(),
    }
}

pub async fn get_customer_wallets(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Path(external_id): Path<String>,
) -> impl IntoResponse {
    let service = MerchantCustomerService::new(
        state.db_pool.clone(),
        state.price_service.clone(),
        state.volume_tracking_service.clone(),
        state.notification_service.clone(),
        state.balance_service.clone(),
        Arc::new(state.config.clone()),
    );

    match service
        .get_customer_wallets(context.merchant_id, &external_id, context.sandbox_mode)
        .await
    {
        Ok(wallets) => (
            StatusCode::OK,
            Json(json!({
                "external_id": external_id,
                "wallets": wallets
            })),
        )
            .into_response(),
        Err(e) => e.into_response(),
    }
}

pub async fn get_deposit_address(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Path((external_id, crypto_type)): Path<(String, String)>,
) -> impl IntoResponse {
    let service = MerchantCustomerService::new(
        state.db_pool.clone(),
        state.price_service.clone(),
        state.volume_tracking_service.clone(),
        state.notification_service.clone(),
        state.balance_service.clone(),
        Arc::new(state.config.clone()),
    );

    match service
        .get_deposit_address(
            context.merchant_id,
            &external_id,
            &crypto_type,
            context.sandbox_mode,
        )
        .await
    {
        Ok(address) => (
            StatusCode::OK,
            Json(json!({
                "external_id": external_id,
                "crypto_type": crypto_type,
                "deposit_address": address
            })),
        )
            .into_response(),
        Err(e) => e.into_response(),
    }
}

#[derive(serde::Deserialize)]
pub struct TransactionQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn get_customer_transactions(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Path(external_id): Path<String>,
    axum::extract::Query(params): axum::extract::Query<TransactionQuery>,
) -> impl IntoResponse {
    let service = MerchantCustomerService::new(
        state.db_pool.clone(),
        state.price_service.clone(),
        state.volume_tracking_service.clone(),
        state.notification_service.clone(),
        state.balance_service.clone(),
        Arc::new(state.config.clone()),
    );
    let limit = params.limit.unwrap_or(50);
    let offset = params.offset.unwrap_or(0);

    match service
        .get_customer_transactions(
            context.merchant_id,
            &external_id,
            limit,
            offset,
            context.sandbox_mode,
        )
        .await
    {
        Ok((transactions, total)) => {
            use futures::future::join_all;
            use std::collections::HashMap;

            // 1. Gather all unique crypto types
            let mut unique_cryptos = std::collections::HashSet::new();
            for tx in &transactions {
                unique_cryptos.insert(tx.crypto_type.clone());
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

            // 3. Process transactions using pre-fetched prices
            let response_transactions: Vec<_> = transactions
                .into_iter()
                .map(|mut tx| {
                    let price = price_map.get(&tx.crypto_type).copied().unwrap_or(0.0);
                    let price_dec = rust_decimal::Decimal::from_f64_retain(price)
                        .unwrap_or(rust_decimal::Decimal::ZERO);
                    tx.amount_usd = (tx.amount * price_dec).round_dp(2);
                    tx
                })
                .collect();

            (
                StatusCode::OK,
                Json(json!({
                    "external_id": external_id,
                    "transactions": response_transactions,
                    "total": total,
                    "limit": limit,
                    "offset": offset
                })),
            )
                .into_response()
        }
        Err(e) => e.into_response(),
    }
}

pub async fn pay_merchant(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Path(external_id): Path<String>,
    Json(req): Json<PayMerchantRequest>,
) -> impl IntoResponse {
    let service = MerchantCustomerService::new(
        state.db_pool.clone(),
        state.price_service.clone(),
        state.volume_tracking_service.clone(),
        state.notification_service.clone(),
        state.balance_service.clone(),
        Arc::new(state.config.clone()),
    );

    tracing::debug!(
        external_id = %external_id,
        merchant_id = %context.merchant_id,
        crypto_type = %req.crypto_type,
        amount = %req.amount,
        sandbox_mode = %context.sandbox_mode,
        "Processing customer pay merchant request"
    );

    match service
        .pay_merchant(
            crate::services::merchant_customer_service::PayMerchantParams {
                merchant_id: context.merchant_id,
                external_id: &external_id,
                crypto_type_str: &req.crypto_type,
                amount_str: &req.amount,
                reference_id: req.reference_id.as_deref(),
                description: req.description.as_deref(),
                sandbox_mode: context.sandbox_mode,
            },
        )
        .await
    {
        Ok(transaction) => {
            tracing::info!(
                external_id = %external_id,
                transaction_id = %transaction.id,
                amount = %transaction.amount,
                "Customer payment to merchant successful"
            );

            // Log audit event
            let _ = state
                .audit_service
                .log_event(
                    context.merchant_id,
                    "customer_pay_merchant",
                    Some(&format!("Customer {} paid merchant", external_id)),
                    Some(json!({
                        "external_id": external_id,
                        "amount": req.amount,
                        "crypto_type": req.crypto_type,
                        "reference_id": req.reference_id
                    })),
                )
                .await;

            (
                StatusCode::OK,
                Json(json!({
                    "transaction": transaction,
                    "message": "Payment initiated. On-chain transaction will be processed."
                })),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!(
                external_id = %external_id,
                merchant_id = %context.merchant_id,
                error = ?e,
                "Customer pay merchant failed"
            );
            e.into_response()
        }
    }
}

pub async fn update_customer_status(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Path(external_id): Path<String>,
    Json(req): Json<UpdateCustomerStatusRequest>,
) -> impl IntoResponse {
    let service = MerchantCustomerService::new(
        state.db_pool.clone(),
        state.price_service.clone(),
        state.volume_tracking_service.clone(),
        state.notification_service.clone(),
        state.balance_service.clone(),
        Arc::new(state.config.clone()),
    );

    match service
        .update_customer_status(
            context.merchant_id,
            &external_id,
            &req.status,
            req.reason.as_deref(),
        )
        .await
    {
        Ok(customer) => {
            // Log audit event
            let _ = state
                .audit_service
                .log_event(
                    context.merchant_id,
                    "customer_status_update",
                    Some(&format!(
                        "Updated status for customer {} to {}",
                        external_id, req.status
                    )),
                    Some(json!({
                        "external_id": external_id,
                        "status": req.status
                    })),
                )
                .await;

            (
                StatusCode::OK,
                Json(json!({
                    "customer": customer,
                    "message": format!("Customer status updated to '{}'", req.status)
                })),
            )
                .into_response()
        }
        Err(e) => e.into_response(),
    }
}

pub async fn update_customer_permissions(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Path(external_id): Path<String>,
    Json(req): Json<UpdateCustomerPermissionsRequest>,
) -> impl IntoResponse {
    let service = MerchantCustomerService::new(
        state.db_pool.clone(),
        state.price_service.clone(),
        state.volume_tracking_service.clone(),
        state.notification_service.clone(),
        state.balance_service.clone(),
        Arc::new(state.config.clone()),
    );

    let withdrawal_limit = req
        .withdrawal_limit
        .as_deref()
        .and_then(|s| rust_decimal::Decimal::from_str_exact(s).ok());

    match service
        .update_customer_permissions(
            context.merchant_id,
            &external_id,
            req.can_withdraw,
            withdrawal_limit,
        )
        .await
    {
        Ok(customer) => {
            // Log audit event
            let _ = state
                .audit_service
                .log_event(
                    context.merchant_id,
                    "customer_permissions_update",
                    Some(&format!("Updated permissions for customer {}", external_id)),
                    Some(json!({
                        "external_id": external_id,
                        "can_withdraw": req.can_withdraw,
                        "withdrawal_limit": req.withdrawal_limit
                    })),
                )
                .await;

            (
                StatusCode::OK,
                Json(json!({
                    "customer": customer,
                    "message": "Customer permissions updated"
                })),
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": e.to_string()
            })),
        )
            .into_response(),
    }
}

pub async fn sweep_customer_wallet(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Path(external_id): Path<String>,
    Json(req): Json<crate::models::merchant_customer::SweepCustomerRequest>,
) -> impl IntoResponse {
    // 1. Verify Transaction PIN (Merchant)
    if let Err(e) = state
        .merchant_service
        .verify_transaction_pin(context.merchant_id, &req.pin)
        .await
    {
        return e.into_response();
    }

    let req_mode = req.sweep_mode.clone();
    let req_types = req.crypto_types.clone();

    let service = MerchantCustomerService::new(
        state.db_pool.clone(),
        state.price_service.clone(),
        state.volume_tracking_service.clone(),
        state.notification_service.clone(),
        state.balance_service.clone(),
        Arc::new(state.config.clone()),
    );

    match service
        .sweep_customer_wallet(
            context.merchant_id,
            &external_id,
            req,
            context.sandbox_mode,
            &state.config,
        )
        .await
    {
        Ok(swept_results) => {
            // Log audit event
            let sweeps_json: Vec<Value> = swept_results
                .iter()
                .map(|(ct, amt)| json!({"crypto_type": ct, "amount": amt}))
                .collect();
            let _ = state
                .audit_service
                .log_event(
                    context.merchant_id,
                    "customer_wallet_sweep",
                    Some(&format!(
                        "Swept funds from customer {} wallet using {} mode",
                        external_id, req_mode
                    )),
                    Some(json!({
                        "external_id": external_id,
                        "sweep_mode": req_mode,
                        "crypto_types": req_types,
                        "sweeps": sweeps_json
                    })),
                )
                .await;

            let response_sweeps: Vec<Value> = swept_results
                .into_iter()
                .map(|(ct, amt)| {
                    json!({
                        "crypto_type": ct,
                        "amount": amt
                    })
                })
                .collect();

            (
                StatusCode::OK,
                Json(json!({
                    "sweeps": response_sweeps,
                    "message": "Funds swept successfully to merchant external wallet"
                })),
            )
                .into_response()
        }
        Err(e) => e.into_response(),
    }
}

#[derive(serde::Deserialize)]
pub struct ListCustomersQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn list_customers(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    axum::extract::Query(params): axum::extract::Query<ListCustomersQuery>,
) -> impl IntoResponse {
    let service = MerchantCustomerService::new(
        state.db_pool.clone(),
        state.price_service.clone(),
        state.volume_tracking_service.clone(),
        state.notification_service.clone(),
        state.balance_service.clone(),
        Arc::new(state.config.clone()),
    );
    let limit = params.limit.unwrap_or(50);
    let offset = params.offset.unwrap_or(0);

    match service
        .list_customers(context.merchant_id, limit, offset)
        .await
    {
        Ok((customers, total)) => (
            StatusCode::OK,
            Json(json!({
                "customers": customers,
                "total": total,
                "limit": limit,
                "offset": offset,
                "has_more": (offset + customers.len() as i64) < total
            })),
        )
            .into_response(),
        Err(e) => e.into_response(),
    }
}

pub async fn get_customers_summary(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
) -> impl IntoResponse {
    let service = MerchantCustomerService::new(
        state.db_pool.clone(),
        state.price_service.clone(),
        state.volume_tracking_service.clone(),
        state.notification_service.clone(),
        state.balance_service.clone(),
        Arc::new(state.config.clone()),
    );

    match service
        .get_customers_summary(context.merchant_id, context.sandbox_mode)
        .await
    {
        Ok(summary) => (StatusCode::OK, Json(summary)).into_response(),
        Err(e) => e.into_response(),
    }
}

pub async fn deactivate_customer(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Path(external_id): Path<String>,
) -> impl IntoResponse {
    let service = MerchantCustomerService::new(
        state.db_pool.clone(),
        state.price_service.clone(),
        state.volume_tracking_service.clone(),
        state.notification_service.clone(),
        state.balance_service.clone(),
        Arc::new(state.config.clone()),
    );

    match service
        .deactivate_customer(context.merchant_id, &external_id)
        .await
    {
        Ok(_) => {
            // Log audit event
            let _ = state
                .audit_service
                .log_event(
                    context.merchant_id,
                    "customer_deactivation",
                    Some(&format!("Deactivated customer {}", external_id)),
                    Some(json!({
                        "external_id": external_id
                    })),
                )
                .await;

            (
                StatusCode::OK,
                Json(json!({
                    "message": "Customer deactivated successfully"
                })),
            )
                .into_response()
        }
        Err(e) => e.into_response(),
    }
}
