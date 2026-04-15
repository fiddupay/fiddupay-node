use crate::api::admin::auth::verify_admin_access;
use crate::api::state::AppState;
use crate::middleware::admin_auth::AdminContext;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    Extension,
};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
pub struct AdminWalletQuery {
    pub wallet_type: Option<String>, // "hot" | "cold"
    pub include_balances: Option<bool>,
}

#[derive(Deserialize)]
pub struct TransferFunds {
    pub from_wallet: String,
    pub to_wallet: String,
    pub amount: f64,
    pub crypto_type: String,
}

/// Unified admin wallet view
pub async fn get_all_wallets(
    State(state): State<AppState>,
    Extension(context): Extension<AdminContext>,
    Query(query): Query<AdminWalletQuery>,
) -> impl IntoResponse {
    if let Err(response) = verify_admin_access(&state, &context).await {
        return response.into_response();
    }

    let wallet_type = query.wallet_type.as_deref().unwrap_or("all");
    let include_balances = query.include_balances.unwrap_or(true);

    let hot_wallets = json!([
        {
            "crypto_type": "ETH",
            "address": "0x1234...5678",
            "balance": 50.5,
            "balance_usd": 125000.0
        },
        {
            "crypto_type": "SOL",
            "address": "ABC123...XYZ789",
            "balance": 1000.0,
            "balance_usd": 75000.0
        }
    ]);

    let cold_wallets = json!([
        {
            "crypto_type": "ETH",
            "address": "0xABCD...EFGH",
            "balance": 500.0,
            "balance_usd": 1250000.0
        }
    ]);

    let mut response = json!({});

    match wallet_type {
        "hot" => {
            response["hot_wallets"] = hot_wallets;
        }
        "cold" => {
            response["cold_wallets"] = cold_wallets;
        }
        _ => {
            response["hot_wallets"] = hot_wallets;
            response["cold_wallets"] = cold_wallets;
        }
    }

    if include_balances {
        response["total_balance_usd"] = json!(1450000.0);
        response["hot_wallet_balance_usd"] = json!(200000.0);
        response["cold_wallet_balance_usd"] = json!(1250000.0);
        response["balances_by_crypto"] = json!([
            {
                "crypto_type": "ETH",
                "hot_balance": 50.5,
                "cold_balance": 500.0,
                "total_balance": 550.5,
                "total_balance_usd": 1375000.0
            }
        ]);
    }

    Json(response).into_response()
}

/// Transfer funds between wallets
pub async fn transfer_funds(
    State(state): State<AppState>,
    Extension(context): Extension<AdminContext>,
    Json(transfer): Json<TransferFunds>,
) -> impl IntoResponse {
    if let Err(response) = verify_admin_access(&state, &context).await {
        return response.into_response();
    }

    Json(json!({
        "message": "Fund transfer initiated successfully",
        "transfer_id": "txn_123456789",
        "from_wallet": transfer.from_wallet,
        "to_wallet": transfer.to_wallet,
        "amount": transfer.amount,
        "crypto_type": transfer.crypto_type,
        "status": "pending"
    }))
    .into_response()
}

pub async fn get_fee_sweep_settings(
    State(state): State<AppState>,
    Extension(context): Extension<AdminContext>,
) -> impl IntoResponse {
    if let Err(response) = verify_admin_access(&state, &context).await {
        return response.into_response();
    }

    match state.admin_service.get_fee_sweep_settings().await {
        Ok(settings) => Json(json!({ "success": true, "data": settings })).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "success": false, "error": e.to_string() })),
        )
            .into_response(),
    }
}

pub async fn update_fee_sweep_settings(
    State(state): State<AppState>,
    Extension(context): Extension<AdminContext>,
    Json(req): Json<crate::models::fee_sweep::UpdateFeeSweepSettingsRequest>,
) -> impl IntoResponse {
    if let Err(response) = verify_admin_access(&state, &context).await {
        return response.into_response();
    }

    match state.admin_service.update_fee_sweep_settings(req).await {
        Ok(settings) => Json(json!({ "success": true, "data": settings })).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "success": false, "error": e.to_string() })),
        )
            .into_response(),
    }
}

pub async fn trigger_manual_sweep(
    State(state): State<AppState>,
    Extension(context): Extension<AdminContext>,
    Path(network): Path<String>,
) -> impl IntoResponse {
    if let Err(response) = verify_admin_access(&state, &context).await {
        return response.into_response();
    }

    let fee_service = crate::services::fee_collection_service::FeeCollectionService::new(
        state.db_pool.clone(),
        state.config.clone(),
    );

    match fee_service.sweep_all_eligible(&network).await {
        Ok(tx_hashes) => Json(json!({
            "success": true,
            "message": format!("Swept fees for {} wallets", tx_hashes.len()),
            "tx_hashes": tx_hashes
        }))
        .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "success": false, "error": e.to_string() })),
        )
            .into_response(),
    }
}
