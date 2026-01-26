// API endpoints for Address-Only Mode (Phase 1)
use crate::error::ServiceError;
use crate::middleware::auth::MerchantContext;
use crate::payment::models::CryptoType;
use crate::services::address_only_service::{AddressOnlyService, AddressOnlyPayment};
use axum::{extract::Query, response::Json, Extension};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CreateAddressOnlyPaymentRequest {
    pub crypto_type: CryptoType,
    pub merchant_address: String,
    pub amount: Decimal,
}

#[derive(Debug, Serialize)]
pub struct AddressOnlyPaymentResponse {
    pub payment_id: String,
    pub gateway_deposit_address: String,
    pub requested_amount: Decimal,
    pub customer_amount: Decimal, // Amount customer needs to pay
    pub processing_fee: Decimal,
    pub customer_pays_fee: bool, // Who pays the fee
    pub customer_instructions: String,
    pub supported_currencies: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct PaymentStatusQuery {
    pub payment_id: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateFeeSettingRequest {
    pub customer_pays_fee: bool,
}

/// Create address-only payment request (native currencies only)
pub async fn create_address_only_payment(
    Extension(context): Extension<MerchantContext>,
    Extension(address_service): Extension<AddressOnlyService>,
    Json(request): Json<CreateAddressOnlyPaymentRequest>,
) -> Result<Json<AddressOnlyPaymentResponse>, ServiceError> {
    
    let payment = address_service
        .create_payment_request(
            context.merchant_id,
            request.crypto_type,
            request.merchant_address,
            request.amount,
        )
        .await?;

    let response = AddressOnlyPaymentResponse {
        payment_id: payment.payment_id,
        gateway_deposit_address: payment.gateway_deposit_address,
        requested_amount: payment.requested_amount,
        customer_amount: payment.customer_amount,
        processing_fee: payment.processing_fee,
        customer_pays_fee: payment.customer_amount > payment.requested_amount,
        customer_instructions: format!(
            "Send exactly {} {} to the deposit address. {}",
            payment.customer_amount,
            request.crypto_type.to_string(),
            if payment.customer_amount > payment.requested_amount {
                "This includes the processing fee."
            } else {
                "Processing fee will be deducted from merchant's amount."
            }
        ),
        supported_currencies: vec![
            "ETH".to_string(),
            "BNB".to_string(), 
            "MATIC".to_string(),
            "ARB".to_string(),
            "SOL".to_string(),
        ],
    };

    Ok(Json(response))
}

/// Get payment status
pub async fn get_address_only_payment_status(
    Extension(context): Extension<MerchantContext>,
    Extension(address_service): Extension<AddressOnlyService>,
    Query(query): Query<PaymentStatusQuery>,
) -> Result<Json<AddressOnlyPayment>, ServiceError> {
    
    // TODO: Add merchant ownership validation
    let payment = address_service.get_payment_by_id(&query.payment_id).await?;
    
    Ok(Json(payment))
}

/// List supported native currencies for address-only mode
pub async fn get_supported_native_currencies() -> Json<Vec<String>> {
    Json(vec![
        "ETH".to_string(),
        "BNB".to_string(),
        "MATIC".to_string(), 
        "ARB".to_string(),
        "SOL".to_string(),
    ])
}

/// Get address-only mode health status
pub async fn get_address_only_health(
    Extension(manager): Extension<std::sync::Arc<crate::services::address_only_manager::AddressOnlyManager>>,
) -> Result<Json<crate::services::address_only_manager::AddressOnlyHealthStatus>, ServiceError> {
    let health = manager.health_check().await?;
    Ok(Json(health))
}

/// Get address-only mode statistics
pub async fn get_address_only_stats(
    Extension(context): Extension<MerchantContext>,
    Extension(address_service): Extension<AddressOnlyService>,
) -> Result<Json<AddressOnlyStats>, ServiceError> {
    let stats = address_service.get_merchant_stats(context.merchant_id).await?;
    Ok(Json(stats))
}

/// Update merchant fee payment setting
pub async fn update_fee_setting(
    Extension(context): Extension<MerchantContext>,
    Extension(address_service): Extension<AddressOnlyService>,
    Json(request): Json<UpdateFeeSettingRequest>,
) -> Result<Json<serde_json::Value>, ServiceError> {
    // Update merchant fee setting in database
    address_service.update_merchant_fee_setting(context.merchant_id, request.customer_pays_fee).await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": format!(
            "Fee payment setting updated: {}",
            if request.customer_pays_fee { "Customer pays fee" } else { "Merchant pays fee" }
        ),
        "customer_pays_fee": request.customer_pays_fee
    })))
}

/// Get merchant fee setting
pub async fn get_fee_setting(
    Extension(context): Extension<MerchantContext>,
    Extension(address_service): Extension<AddressOnlyService>,
) -> Result<Json<serde_json::Value>, ServiceError> {
    let customer_pays_fee = address_service.get_merchant_fee_setting(context.merchant_id).await?;

    Ok(Json(serde_json::json!({
        "customer_pays_fee": customer_pays_fee,
        "description": if customer_pays_fee { 
            "Customer pays processing fee" 
        } else { 
            "Merchant pays processing fee" 
        }
    })))
}

#[derive(Debug, Serialize)]
pub struct AddressOnlyStats {
    pub total_payments: i64,
    pub completed_payments: i64,
    pub pending_payments: i64,
    pub total_volume: Decimal,
    pub total_fees_collected: Decimal,
}
