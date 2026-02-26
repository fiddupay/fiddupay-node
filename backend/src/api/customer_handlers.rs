// Merchant Customer API Handlers
// Endpoints for managing and provisioning sub-account user wallets

use crate::api::state::AppState;
use crate::middleware::auth::MerchantContext;
use crate::services::merchant_customer_service::MerchantCustomerService;
use crate::models::merchant_customer::{CreateCustomerRequest, ProvisionWalletRequest};
use axum::{
    extract::{State, Path},
    http::StatusCode,
    response::IntoResponse,
    Json, Extension,
};
use serde_json::json;

pub async fn register_customer(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Json(req): Json<CreateCustomerRequest>,
) -> impl IntoResponse {
    let service = MerchantCustomerService::new(state.db_pool.clone());
    
    match service.register_customer(context.merchant_id, req).await {
        Ok(customer) => (StatusCode::CREATED, Json(json!({
            "customer": customer,
            "message": "Customer registered successfully"
        }))).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, Json(json!({
            "error": e.to_string()
        }))).into_response(),
    }
}

pub async fn provision_customer_wallets(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Json(req): Json<ProvisionWalletRequest>,
) -> impl IntoResponse {
    let service = MerchantCustomerService::new(state.db_pool.clone());
    
    match service.provision_wallets(context.merchant_id, &req.external_id, req.networks).await {
        Ok(wallets) => {
            // Filter out sensitive data (private keys are stored encrypted, but we return addresses)
            let response_wallets: Vec<_> = wallets.iter().map(|w| json!({
                "crypto_type": w.crypto_type,
                "network": w.network,
                "address": w.address,
                "created_at": w.created_at
            })).collect();

            (StatusCode::OK, Json(json!({
                "external_id": req.external_id,
                "wallets": response_wallets,
                "message": "Customer wallets provisioned successfully across requested networks"
            }))).into_response()
        },
        Err(e) => (StatusCode::BAD_REQUEST, Json(json!({
            "error": e.to_string()
        }))).into_response(),
    }
}

pub async fn get_customer_balances(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Path(external_id): Path<String>,
) -> impl IntoResponse {
    let service = MerchantCustomerService::new(state.db_pool.clone());
    
    match service.get_customer_balances(context.merchant_id, &external_id).await {
        Ok(balances) => (StatusCode::OK, Json(json!({
            "external_id": external_id,
            "balances": balances
        }))).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, Json(json!({
            "error": e.to_string()
        }))).into_response(),
    }
}

pub async fn sweep_customer_wallet(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Path(external_id): Path<String>,
    Json(req): Json<crate::models::merchant_customer::SweepCustomerRequest>,
) -> impl IntoResponse {
    let service = MerchantCustomerService::new(state.db_pool.clone());
    
    match service.sweep_customer_wallet(
        context.merchant_id, 
        &external_id, 
        &req.crypto_type, 
        req.amount,
        context.sandbox_mode
    ).await {
        Ok(swept_amount) => (StatusCode::OK, Json(json!({
            "swept_amount": swept_amount,
            "message": "Funds swept successfully to merchant master balance"
        }))).into_response(),
        Err(e) => {
            let status = match e {
                crate::error::ServiceError::InsufficientFunds(_) => StatusCode::PAYMENT_REQUIRED,
                _ => StatusCode::BAD_REQUEST,
            };
            (status, Json(json!({
                "error": e.to_string()
            }))).into_response()
        },
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
    let service = MerchantCustomerService::new(state.db_pool.clone());
    let limit = params.limit.unwrap_or(50);
    let offset = params.offset.unwrap_or(0);
    
    match service.list_customers(context.merchant_id, limit, offset).await {
        Ok((customers, total)) => (StatusCode::OK, Json(json!({
            "customers": customers,
            "total": total,
            "limit": limit,
            "offset": offset,
            "has_more": offset + customers.len() as i64 < total
        }))).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, Json(json!({
            "error": e.to_string()
        }))).into_response(),
    }
}

pub async fn withdraw_from_customer(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Path(external_id): Path<String>,
    Json(req): Json<crate::models::merchant_customer::CustomerWithdrawalRequest>,
) -> impl IntoResponse {
    let service = MerchantCustomerService::new(state.db_pool.clone());
    
    match service.withdraw_from_customer(
        context.merchant_id,
        &external_id,
        &req.crypto_type,
        &req.amount,
        &req.destination_address,
        context.sandbox_mode
    ).await {
        Ok(withdrawal) => (StatusCode::OK, Json(json!({
            "withdrawal": withdrawal,
            "message": "Withdrawal requested successfully"
        }))).into_response(),
        Err(e) => {
            let status = match e {
                crate::error::ServiceError::InsufficientFunds(_) => StatusCode::PAYMENT_REQUIRED,
                _ => StatusCode::BAD_REQUEST,
            };
            (status, Json(json!({
                "error": e.to_string()
            }))).into_response()
        },
    }
}
