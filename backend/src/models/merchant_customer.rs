// Merchant Customer Models
// Handles sub-account user data, wallet mapping, and customer transactions

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MerchantCustomer {
    pub id: i64,
    pub merchant_id: i64,
    pub external_id: String,
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub is_active: bool,
    pub status: String,               // active, flagged, suspended, blocked
    pub status_reason: Option<String>,
    pub can_withdraw: bool,
    pub withdrawal_limit: Option<Decimal>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub sandbox_mode: bool,
    pub transaction_pin_hash: Option<String>,
    pub pin_setup_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MerchantCustomerWallet {
    pub id: i64,
    pub customer_id: i64,
    pub merchant_id: i64,
    pub crypto_type: String,
    pub network: String,
    pub address: String,
    #[serde(skip_serializing)]
    pub encrypted_private_key: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub sandbox_mode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MerchantCustomerBalance {
    pub id: i64,
    pub customer_id: i64,
    pub merchant_id: i64,
    pub crypto_type: String,
    pub available_balance: Decimal,
    pub locked_balance: Decimal,
    pub total_balance: Decimal,
    pub last_updated_at: DateTime<Utc>,
    pub sandbox_mode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CustomerTransaction {
    pub id: i64,
    pub customer_id: i64,
    pub merchant_id: i64,
    #[sqlx(rename = "type")]
    #[serde(rename = "type")]
    pub tx_type: String, // WITHDRAWAL, MERCHANT_PAYMENT, SWEEP
    pub crypto_type: String,
    pub amount: Decimal,
    pub fee: Decimal,
    pub status: String,
    pub destination_address: Option<String>,
    pub transaction_hash: Option<String>,
    pub reference_id: Option<String>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub sandbox_mode: bool,
}

// ============================================================================
// Request structs
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateCustomerRequest {
    pub external_id: String,
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProvisionWalletRequest {
    pub networks: Option<Vec<String>>, // "evm", "solana"
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomerWithdrawalRequest {
    pub crypto_type: String,
    pub amount: String,
    pub destination_address: String,
    pub pin: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SweepCustomerRequest {
    pub crypto_type: String,
    pub amount: Option<String>,
    pub pin: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PayMerchantRequest {
    pub crypto_type: String,
    pub amount: String,
    pub reference_id: Option<String>,  // merchant's order/product ID
    pub description: Option<String>,   // e.g. "Purchase: Premium Plan"
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateCustomerStatusRequest {
    pub status: String,         // active, flagged, suspended, blocked
    pub reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateCustomerPermissionsRequest {
    pub can_withdraw: Option<bool>,
    pub withdrawal_limit: Option<String>, // decimal as string
}
