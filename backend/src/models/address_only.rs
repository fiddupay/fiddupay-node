// Address Only Payment Models
// Database models for address-only mode payments

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AddressOnlyPayment {
    pub id: i64,
    pub payment_id: String,
    pub merchant_id: i64,
    pub crypto_type: String,
    pub gateway_deposit_address: String,
    pub merchant_destination_address: String,
    pub requested_amount: Decimal,
    pub processing_fee: Decimal,
    pub forwarding_amount: Decimal,
    pub status: String,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AddressOnlyForwardingTx {
    pub id: i64,
    pub payment_id: String,
    pub destination_address: String,
    pub amount: Decimal,
    pub gas_fee: Decimal,
    pub tx_hash: String,
    pub status: String,
    pub created_at: Option<DateTime<Utc>>,
    pub confirmed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DepositKeypair {
    pub id: i64,
    pub payment_id: String,
    pub address: String,
    pub encrypted_private_key: String,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WebhookLog {
    pub id: i64,
    pub payment_id: String,
    pub webhook_url: String,
    pub status_code: i32,
    pub attempted_at: Option<DateTime<Utc>>,
}
