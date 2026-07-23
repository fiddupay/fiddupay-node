// Merchant Customer Models
// Handles sub-account user data, wallet mapping, and customer transactions

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;

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
    pub status: String, // active, flagged, suspended, blocked
    pub status_reason: Option<String>,
    pub can_withdraw: bool,
    pub withdrawal_limit: Option<Decimal>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
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
    pub amount_usd: Decimal,
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

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateCustomerRequest {
    #[validate(length(min = 1, max = 100))]
    pub external_id: String,
    #[validate(email)]
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
    pub sweep_mode: String,
    pub crypto_types: Option<Vec<String>>,
    pub amount: Option<String>,
    pub pin: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PayMerchantRequest {
    pub crypto_type: String,
    pub amount: String,
    pub reference_id: Option<String>, // merchant's order/product ID
    pub description: Option<String>,  // e.g. "Purchase: Premium Plan"
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateCustomerStatusRequest {
    pub status: String, // active, flagged, suspended, blocked
    pub reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateCustomerPermissionsRequest {
    pub can_withdraw: Option<bool>,
    pub withdrawal_limit: Option<String>, // decimal as string
}

// ============================================================================
// Batch On-Chain Consolidation Structs
// ============================================================================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UnsweptAssetItem {
    pub crypto_type: String,
    pub currency: String,
    pub network: String,
    pub total_crypto_amount: String,
    pub total_usd_amount: String,
    pub wallet_count: i64,
    pub target_master_address: Option<String>,
    pub has_sufficient_gas: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UnsweptAssetsSummaryResponse {
    pub assets: Vec<UnsweptAssetItem>,
    pub total_unswept_usd: String,
    pub total_wallets_count: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BatchSweepRequest {
    pub sweep_scope: String, // "NETWORK_CURRENCY" or "ALL"
    pub crypto_type: Option<String>,
    pub pin: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BatchSweepResponse {
    pub status: String,
    pub swept_wallets_count: i64,
    pub total_crypto_swept: String,
    pub total_usd_swept: String,
    pub swept_cryptos: Vec<String>,
    pub message: String,
}
