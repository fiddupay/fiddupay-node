use crate::middleware::validation::validate_positive_amount;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct P2pProfile {
    pub id: i64,
    pub email: String,
    pub nickname: String,
    // Note: Don't serialize password_hash to API responses
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub kyc_level: i32,
    pub is_vendor: bool,
    pub is_active: bool,
    pub sandbox_mode: bool,
    pub total_trades: i32,
    pub completion_rate: Decimal,
    pub thumbs_up_count: i32,
    pub thumbs_down_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub gender: Option<String>,
    pub phone_number: Option<String>,
    pub country: Option<String>,
    pub terms_accepted: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct P2pWallet {
    pub id: i64,
    pub user_id: i64,
    pub crypto_type: String,
    pub network: String,
    pub address: String,
    #[serde(skip_serializing)]
    pub encrypted_private_key: String,
    pub is_active: bool,
    pub sandbox_mode: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct P2pBalance {
    pub id: i64,
    pub user_id: i64,
    pub crypto_type: String,
    pub available_balance: Decimal,
    pub locked_balance: Decimal,
    pub total_balance: Decimal,
    pub sandbox_mode: bool,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct P2pPaymentMethod {
    pub id: i64,
    pub user_id: i64,
    pub method_name: String,
    pub currency: String,
    pub account_name: String,
    pub account_number: String,
    pub bank_name: Option<String>,
    pub is_active: bool,
    pub sandbox_mode: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct P2pAd {
    pub id: i64,
    pub user_id: i64,
    pub ad_type: String, // 'BUY' or 'SELL'
    pub crypto_type: String,
    pub fiat_currency: String,
    pub price: Decimal,
    pub total_amount: Decimal,
    pub min_limit: Decimal,
    pub max_limit: Decimal,
    pub payment_time_limit: i32,
    pub status: String,
    pub terms_and_conditions: Option<String>,
    pub auto_reply: Option<String>,
    pub sandbox_mode: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct P2pTrade {
    pub id: i64,
    pub trade_id: String,
    pub ad_id: i64,
    pub maker_id: i64,
    pub taker_id: i64,
    pub crypto_amount: Decimal,
    pub fiat_amount: Decimal,
    pub price: Decimal,
    pub status: String,
    pub payment_method_id: i64,
    pub expires_at: DateTime<Utc>,
    pub paid_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub disputed_at: Option<DateTime<Utc>>,
    pub cancel_reason: Option<String>,
    pub sandbox_mode: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct P2pChatMessage {
    pub id: i64,
    pub trade_id: i64,
    pub sender_id: i64,
    pub message: String,
    pub attachment_url: Option<String>,
    pub is_system_message: bool,
    pub is_warning_broadcast: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct P2pRating {
    pub id: i64,
    pub trade_id: i64,
    pub reviewer_id: i64,
    pub target_id: i64,
    pub rating: String, // 'THUMBS_UP' or 'THUMBS_DOWN'
    pub comment: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct P2pPlatformFee {
    pub id: i64,
    pub trade_id: String,
    pub crypto_type: String,
    pub amount: Decimal,
    pub sandbox_mode: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct P2pSupportTicket {
    pub id: i64,
    pub user_id: i64,
    pub subject: String,
    pub category: String, // 'SCAM_REPORT', 'BUG', 'PAYMENT_ISSUE', 'OTHER'
    pub description: String,
    pub status: String,
    pub reported_user_id: Option<i64>,
    pub trade_id: Option<String>,
    pub attachment_url: Option<String>,
    pub admin_notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ==========================================
// Request Payloads
// ==========================================

#[derive(Debug, Deserialize, Clone, Validate)]
pub struct CreateProfileRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 3, max = 50))]
    pub nickname: String,
    #[validate(length(min = 8))]
    pub password: String,
    #[validate(length(min = 1, max = 100))]
    pub first_name: String,
    #[validate(length(min = 1, max = 100))]
    pub last_name: String,
    pub gender: String,
    pub phone_number: String,
    pub country: String,
    pub terms_accepted: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CreatePaymentMethodRequest {
    pub method_name: String,
    pub currency: String,
    pub account_name: String,
    pub account_number: String,
    pub bank_name: Option<String>,
}

#[derive(Debug, Deserialize, Clone, Validate)]
pub struct CreateAdRequest {
    pub ad_type: String,
    pub crypto_type: String,
    pub fiat_currency: String,
    #[validate(custom(function = "validate_positive_amount"))]
    pub price: Decimal,
    #[validate(custom(function = "validate_positive_amount"))]
    pub total_amount: Decimal,
    #[validate(custom(function = "validate_positive_amount"))]
    pub min_limit: Decimal,
    #[validate(custom(function = "validate_positive_amount"))]
    pub max_limit: Decimal,
    pub payment_time_limit: Option<i32>,
    pub terms_and_conditions: Option<String>,
    pub auto_reply: Option<String>,
    pub payment_method_ids: Vec<i64>,
}

#[derive(Debug, Deserialize, Clone, Validate)]
pub struct CreateTradeRequest {
    pub ad_id: i64,
    pub crypto_amount: Option<Decimal>,
    pub fiat_amount: Option<Decimal>,
    pub payment_method_id: i64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SendMessageRequest {
    pub message: String,
    pub attachment_url: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CreateDisputeRequest {
    pub reason: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Clone, Validate)]
pub struct CreateRatingRequest {
    pub rating: String,
    pub comment: Option<String>,
}

#[derive(Debug, Deserialize, Clone, Validate)]
pub struct CreateSupportTicketRequest {
    pub subject: String,
    pub category: String,
    pub description: String,
    pub reported_user_id: Option<i64>,
    pub trade_id: Option<String>,
    pub attachment_url: Option<String>,
}
