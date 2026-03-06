// Merchant Customer Models
// Handles sub-account user data and wallet mapping

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
    pub encrypted_private_key: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
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
}

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
    pub networks: Vec<String>, // "evm", "solana"
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomerWithdrawalRequest {
    pub crypto_type: String,
    pub amount: String, // String to preserve precision, converted to Decimal in service
    pub destination_address: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SweepCustomerRequest {
    pub crypto_type: String,
    pub amount: Option<String>, // If None, sweeps entire available balance
}
