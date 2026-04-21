// Merchant Models
// Data structures for merchant accounts and wallets

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, sqlx::Type)]
#[sqlx(type_name = "user_role", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum UserRole {
    #[serde(rename = "SUPER_ADMIN")]
    SuperAdmin,
    #[serde(rename = "ADMIN")]
    Admin,
    #[serde(rename = "MODERATOR")]
    Moderator,
    #[serde(rename = "MERCHANT")]
    Merchant,
    #[serde(rename = "USER")]
    User,
}

impl UserRole {
    pub fn as_str(&self) -> &str {
        match self {
            UserRole::SuperAdmin => "SUPER_ADMIN",
            UserRole::Admin => "ADMIN",
            UserRole::Moderator => "MODERATOR",
            UserRole::Merchant => "MERCHANT",
            UserRole::User => "USER",
        }
    }

    pub fn can_manage_users(&self) -> bool {
        matches!(
            self,
            UserRole::SuperAdmin | UserRole::Admin | UserRole::Merchant
        )
    }

    pub fn can_approve_withdrawals(&self) -> bool {
        matches!(self, UserRole::SuperAdmin | UserRole::Admin)
    }

    pub fn can_view_analytics(&self) -> bool {
        !matches!(self, UserRole::User)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Merchant {
    pub id: i64,
    pub email: String,
    pub business_name: String,
    #[serde(skip_serializing)]
    pub live_api_key_hash: Option<String>,
    #[serde(skip_serializing)]
    pub test_api_key_hash: Option<String>,
    pub live_publishable_key: Option<String>,
    pub test_publishable_key: Option<String>,
    #[serde(skip_serializing)]
    pub password_hash: Option<String>,
    pub fee_percentage: Decimal,
    pub customer_pays_fee: bool, // true = customer pays, false = merchant pays
    pub is_active: bool,
    pub sandbox_mode: bool,
    pub settlement_mode: String,
    pub kyc_verified: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub api_key_expires_at: Option<DateTime<Utc>>,
    pub daily_limit_usd: Option<Decimal>,
    pub role: UserRole,
    pub redirect_url: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub gender: Option<String>,
    pub phone_number: Option<String>,
    pub country: Option<String>,
    pub applicant_role: Option<String>,
    pub business_country: Option<String>,
    pub business_license_number: Option<String>,
    pub business_certificate_url: Option<String>,
    pub terms_accepted: bool,
    pub wallets_locked: bool,
    pub customer_wallets_locked: bool,
    #[serde(skip_serializing)]
    pub transaction_pin_hash: Option<String>,
    pub pin_setup_at: Option<DateTime<Utc>>,
    pub low_balance_threshold_usd: Decimal,
    pub low_balance_alerts_enabled: bool,
    // Compliance & Intelligence Fields
    #[serde(skip_serializing)]
    pub nin_bvn_hash: Option<String>,
    pub social_handles: serde_json::Value, // JSONB
    pub kyc_tier: i32,
    pub compliance_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MerchantWallet {
    pub id: i64,
    pub merchant_id: i64,
    pub crypto_type: String,
    pub network: String,
    pub address: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub wallet_mode: Option<String>,
    #[serde(skip_serializing)]
    pub encrypted_private_key: Option<String>,
    pub sandbox_mode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerchantRegistrationRequest {
    pub email: String,
    pub business_name: String,
    pub password: String,
    // Step 1 KYC
    pub first_name: String,
    pub last_name: String,
    pub gender: String,
    pub phone_number: String,
    pub country: String,
    pub applicant_role: String,
    pub terms_accepted: bool,
    // Step 2 Business
    pub business_country: String,
    pub business_license_number: Option<String>,
    pub business_certificate_url: Option<String>,
    // Compliance Fields
    pub nin_bvn: Option<String>,
    pub twitter_handle: Option<String>,
    pub instagram_handle: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerchantRegistrationResponse {
    pub merchant_id: i64,
    pub api_key: String,
}
