// Payment Models
// Data structures for payment requests and responses

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::payment::models::{CryptoType, PaymentStatus};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePaymentRequest {
    pub amount_usd: Decimal,
    pub crypto_type: CryptoType,
    pub description: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub expiration_minutes: Option<u32>,
    pub partial_payments_enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentResponse {
    pub payment_id: String,
    pub status: PaymentStatus,
    pub amount: Decimal,
    pub amount_usd: Decimal,
    pub crypto_type: CryptoType,
    pub network: String,
    pub deposit_address: String,
    pub payment_link: String,
    pub qr_code_data: String,
    pub fee_amount: Decimal,
    pub fee_amount_usd: Decimal,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub confirmed_at: Option<DateTime<Utc>>,
    pub transaction_hash: Option<String>,
    pub partial_payments: Option<PartialPaymentInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartialPaymentInfo {
    pub enabled: bool,
    pub total_paid: Decimal,
    pub remaining_balance: Decimal,
    pub payments: Vec<PartialPaymentRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartialPaymentRecord {
    pub transaction_hash: String,
    pub amount: Decimal,
    pub confirmed_at: DateTime<Utc>,
}
