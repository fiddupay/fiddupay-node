// Fee Structure API - Demonstrates who pays what fees
use crate::error::ServiceError;
use crate::services::gas_fee_service::GasFeeService;
use crate::payment::models::CryptoType;
use axum::{extract::Query, response::Json, Extension};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct FeeEstimateQuery {
    pub crypto_type: CryptoType,
    pub payment_amount: Decimal,
}

#[derive(Debug, Serialize)]
pub struct FeeBreakdown {
    pub payment_amount: Decimal,
    pub network_fee: NetworkFeeDetails,
    pub processing_fee: ProcessingFeeDetails,
    pub total_user_pays: Decimal,
    pub merchant_receives: Decimal,
}

#[derive(Debug, Serialize)]
pub struct NetworkFeeDetails {
    pub paid_by: String, // Always "user"
    pub currency: String,
    pub base_fee: Option<Decimal>,
    pub priority_fee: Option<Decimal>,
    pub total: Decimal,
    pub reason: String,
}

#[derive(Debug, Serialize)]
pub struct ProcessingFeeDetails {
    pub paid_by: String, // "merchant" or "user"
    pub rate: Decimal,
    pub amount: Decimal,
    pub reason: String,
}

/// Get comprehensive fee breakdown showing who pays what
pub async fn get_fee_breakdown(
    Query(params): Query<FeeEstimateQuery>,
    Extension(gas_service): Extension<GasFeeService>,
) -> Result<Json<FeeBreakdown>, ServiceError> {
    
    // Get current network gas fees
    let gas_estimate = gas_service.get_gas_estimate(params.crypto_type).await?;
    
    // Processing fee configuration (merchant pays by default)
    let processing_fee_rate = Decimal::new(5, 3); // 0.5%
    let processing_fee_amount = params.payment_amount * processing_fee_rate;
    
    // Network fees are ALWAYS paid by user (blockchain requirement)
    let network_fee = NetworkFeeDetails {
        paid_by: "user".to_string(),
        currency: gas_estimate.native_currency.clone(),
        base_fee: gas_estimate.base_fee,
        priority_fee: gas_estimate.priority_fee,
        total: gas_estimate.estimated_withdrawal_cost,
        reason: "Blockchain protocol requirement - cannot be passed to merchant".to_string(),
    };
    
    // Processing fees are paid by merchant (FidduPay revenue)
    let processing_fee = ProcessingFeeDetails {
        paid_by: "merchant".to_string(),
        rate: processing_fee_rate,
        amount: processing_fee_amount,
        reason: "Gateway processing fee - deducted from merchant settlement".to_string(),
    };
    
    // Calculate totals
    let total_user_pays = params.payment_amount + gas_estimate.estimated_withdrawal_cost;
    let merchant_receives = params.payment_amount - processing_fee_amount;
    
    Ok(Json(FeeBreakdown {
        payment_amount: params.payment_amount,
        network_fee,
        processing_fee,
        total_user_pays,
        merchant_receives,
    }))
}
