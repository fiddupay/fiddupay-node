use serde::{Deserialize, Serialize};
use sqlx::Type;
use std::fmt;
use std::str::FromStr;
use rust_decimal::Decimal;
use chrono::{DateTime, Utc};

/// Payment status enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "payment_status", rename_all = "UPPERCASE")]
pub enum PaymentStatus {
    Pending,
    Confirmed,
    Failed,
    Expired,
    Confirming,
    Refunded,
    SelectionRequired,
    Cancelled,
}

impl PaymentStatus {
    pub fn from_string(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "PENDING" => PaymentStatus::Pending,
            "CONFIRMED" => PaymentStatus::Confirmed,
            "FAILED" => PaymentStatus::Failed,
            "EXPIRED" => PaymentStatus::Expired,
            "CONFIRMING" => PaymentStatus::Confirming,
            "REFUNDED" => PaymentStatus::Refunded,
            "SELECTION_REQUIRED" => PaymentStatus::SelectionRequired,
            "CANCELLED" => PaymentStatus::Cancelled,
            _ => PaymentStatus::Pending, // Default fallback
        }
    }
}

/// Cryptocurrency type enumeration (5 supported payment methods)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
#[sqlx(type_name = "varchar", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CryptoType {
    #[serde(rename = "USDT_BEP20")]
    UsdtBep20,        // USDT on Binance Smart Chain (BEP20)
    #[serde(rename = "USDT_ARBITRUM")]
    UsdtArbitrum,     // USDT on Arbitrum One
    #[serde(rename = "USDT_SPL")]
    UsdtSpl,          // USDT on Solana (SPL token)
    #[serde(rename = "USDT_POLYGON")]
    UsdtPolygon,      // USDT on Polygon
    #[serde(rename = "USDT_ETH")]
    UsdtEth,          // USDT on Ethereum (ERC20)
    #[serde(rename = "SOL")]
    Sol,              // Solana native
    #[serde(rename = "ETH")]
    Eth,              // Ethereum native
    #[serde(rename = "ARB")]
    Arb,              // Arbitrum native
    #[serde(rename = "MATIC")]
    Matic,            // Polygon native
    #[serde(rename = "BNB")]
    Bnb,              // BSC native
}

impl std::fmt::Display for CryptoType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            CryptoType::Sol => write!(f, "SOL"),
            CryptoType::UsdtSpl => write!(f, "USDT_SPL"),
            CryptoType::Eth => write!(f, "ETH"),
            CryptoType::UsdtEth => write!(f, "USDT_ETH"),
            CryptoType::Bnb => write!(f, "BNB"),
            CryptoType::UsdtBep20 => write!(f, "USDT_BEP20"),
            CryptoType::Matic => write!(f, "MATIC"),
            CryptoType::UsdtPolygon => write!(f, "USDT_POLYGON"),
            CryptoType::Arb => write!(f, "ARB"),
            CryptoType::UsdtArbitrum => write!(f, "USDT_ARBITRUM"),
        }
    }
}

impl FromStr for CryptoType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "USDT_BEP20" => Ok(CryptoType::UsdtBep20),
            "USDT_ARBITRUM" => Ok(CryptoType::UsdtArbitrum),
            "USDT_SPL" => Ok(CryptoType::UsdtSpl),
            "USDT_POLYGON" => Ok(CryptoType::UsdtPolygon),
            "USDT_ETH" => Ok(CryptoType::UsdtEth),
            "SOL" => Ok(CryptoType::Sol),
            "ETH" => Ok(CryptoType::Eth),
            "ARB" => Ok(CryptoType::Arb),
            "MATIC" => Ok(CryptoType::Matic),
            "BNB" => Ok(CryptoType::Bnb),
            _ => Err(format!("Unknown crypto type: {}", s)),
        }
    }
}

impl CryptoType {
    /// Parse a crypto type string into a CryptoType enum.
    /// Returns an error for unrecognized strings instead of silently defaulting.
    pub fn from_string(s: &str) -> Result<Self, crate::error::ServiceError> {
        match s.to_uppercase().as_str() {
            "SOL" => Ok(CryptoType::Sol),
            "ETH" => Ok(CryptoType::Eth),
            "BNB" => Ok(CryptoType::Bnb),
            "MATIC" => Ok(CryptoType::Matic),
            "ARB" => Ok(CryptoType::Arb),
            "USDT_SOL" | "USDT_SPL" => Ok(CryptoType::UsdtSpl),
            "USDT_ETH" | "USDT_ERC20" => Ok(CryptoType::UsdtEth),
            "USDT_BNB" | "USDT_BEP20" | "USDT_BSC" => Ok(CryptoType::UsdtBep20),
            "USDT_MATIC" | "USDT_POLYGON" => Ok(CryptoType::UsdtPolygon),
            "USDT_ARB" | "USDT_ARBITRUM" => Ok(CryptoType::UsdtArbitrum),
            unknown => {
                tracing::warn!("Unknown crypto type string: '{}', rejecting", unknown);
                Err(crate::error::ServiceError::ValidationError(
                    format!("Unknown crypto type: '{}'", s)
                ))
            }
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            CryptoType::UsdtBep20 => "USDT",
            CryptoType::UsdtArbitrum => "USDT",
            CryptoType::UsdtSpl => "USDT",
            CryptoType::UsdtPolygon => "USDT",
            CryptoType::UsdtEth => "USDT",
            CryptoType::Sol => "SOL",
            CryptoType::Eth => "ETH",
            CryptoType::Arb => "ARB",
            CryptoType::Matic => "MATIC",
            CryptoType::Bnb => "BNB",
        }
    }

    pub fn network(&self) -> &'static str {
        match self {
            CryptoType::UsdtBep20 => "BEP20",
            CryptoType::UsdtArbitrum => "ARBITRUM",
            CryptoType::UsdtSpl => "SOLANA_SPL",
            CryptoType::UsdtPolygon => "POLYGON",
            CryptoType::UsdtEth => "ETHEREUM",
            CryptoType::Sol => "SOLANA",
            CryptoType::Eth => "ETHEREUM",
            CryptoType::Arb => "ARBITRUM",
            CryptoType::Matic => "POLYGON",
            CryptoType::Bnb => "BEP20",
        }
    }

    pub fn uri_scheme(&self) -> &'static str {
        match self {
            CryptoType::Sol | CryptoType::UsdtSpl => "solana",
            _ => "ethereum", // All EVM chains use ethereum: prefix for EIP-681
        }
    }

    pub fn required_confirmations(&self) -> u32 {
        // These should be configurable via environment variables
        // For now, using reasonable defaults that match the config
        match self {
            CryptoType::UsdtBep20 => 15,     // BSC: configurable via CONFIRMATION_BLOCKS_BSC
            CryptoType::UsdtArbitrum => 1,   // Arbitrum: configurable via CONFIRMATION_BLOCKS_ARBITRUM
            CryptoType::UsdtSpl => 32,       // Solana SPL: configurable via CONFIRMATION_BLOCKS_SOL
            CryptoType::UsdtPolygon => 30,   // Polygon: configurable via CONFIRMATION_BLOCKS_POLYGON
            CryptoType::UsdtEth => 12,       // Ethereum: configurable via CONFIRMATION_BLOCKS_ETH
            CryptoType::Sol => 32,           // Solana: configurable via CONFIRMATION_BLOCKS_SOL
            CryptoType::Eth => 12,           // Ethereum: configurable via CONFIRMATION_BLOCKS_ETH
            CryptoType::Arb => 1,            // Arbitrum: configurable via CONFIRMATION_BLOCKS_ARBITRUM
            CryptoType::Matic => 30,         // Polygon: configurable via CONFIRMATION_BLOCKS_POLYGON
            CryptoType::Bnb => 15,           // BSC: configurable via CONFIRMATION_BLOCKS_BSC
        }
    }

    pub fn required_confirmations_from_config(&self, config: &crate::config::Config) -> u32 {
        match self {
            CryptoType::UsdtBep20 | CryptoType::Bnb => config.confirmation_blocks_bsc,
            CryptoType::UsdtArbitrum | CryptoType::Arb => config.confirmation_blocks_arbitrum,
            CryptoType::UsdtSpl | CryptoType::Sol => config.confirmation_blocks_sol,
            CryptoType::UsdtPolygon | CryptoType::Matic => config.confirmation_blocks_polygon,
            CryptoType::UsdtEth | CryptoType::Eth => config.confirmation_blocks_eth,
        }
    }

    pub fn rpc_url_from_config<'a>(&self, config: &'a crate::config::Config) -> &'a str {
        match self {
            CryptoType::UsdtBep20 | CryptoType::Bnb => &config.bsc_rpc_url,
            CryptoType::UsdtArbitrum | CryptoType::Arb => &config.arbitrum_rpc_url,
            CryptoType::UsdtSpl | CryptoType::Sol => &config.solana_rpc_url,
            CryptoType::UsdtPolygon | CryptoType::Matic => &config.polygon_rpc_url,
            CryptoType::UsdtEth | CryptoType::Eth => &config.ethereum_rpc_url,
        }
    }

    pub fn is_native_currency(&self) -> bool {
        matches!(self, CryptoType::Sol | CryptoType::Eth | CryptoType::Bnb | CryptoType::Matic | CryptoType::Arb)
    }

    pub fn get_native_currency(&self) -> CryptoType {
        match self {
            CryptoType::UsdtSpl => CryptoType::Sol,
            CryptoType::UsdtEth => CryptoType::Eth,
            CryptoType::UsdtBep20 => CryptoType::Bnb,
            CryptoType::UsdtPolygon => CryptoType::Matic,
            CryptoType::UsdtArbitrum => CryptoType::Arb,
            _ => *self, // Already native
        }
    }
}

/// Payment creation request
#[derive(Debug, Deserialize)]
pub struct CreatePaymentRequest {
    #[serde(with = "rust_decimal::serde::str_option", default)]
    pub amount: Option<Decimal>,
    #[serde(with = "rust_decimal::serde::str_option", default)]
    pub amount_usd: Option<Decimal>,
    pub crypto_type: Option<CryptoType>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub webhook_url: Option<String>,
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
    #[serde(default)]
    pub expires_in: Option<i32>, // seconds
    #[serde(default)]
    pub expiration_minutes: Option<i32>,
    #[serde(default)]
    pub partial_payments_enabled: Option<bool>,
    
    // Invoice-specific fields
    #[serde(default)]
    pub is_invoice: bool,
    #[serde(default)]
    pub customer_name: Option<String>,
    #[serde(default)]
    pub customer_email: Option<String>,
    #[serde(default)]
    pub items: Option<Vec<InvoiceItem>>,
    #[serde(default)]
    pub tax: Option<Decimal>,
    #[serde(default)]
    pub due_date: Option<chrono::NaiveDate>,
    #[serde(default)]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceItem {
    pub description: String,
    pub quantity: i32,
    pub unit_price: Decimal,
    pub amount: Decimal,
}

impl CreatePaymentRequest {
    pub fn validate(&self) -> Result<(), String> {
        // Enforce input field separation based on currency type
        if let Some(ct) = self.crypto_type {
            if ct.as_str() == "USDT" {
                // Stablecoins MUST use amount_usd
                if self.amount_usd.is_none() {
                    return Err("USDT payments require 'amount_usd'".to_string());
                }
                if self.amount.is_some() {
                    return Err("USDT payments should use 'amount_usd', not 'amount'".to_string());
                }
                if self.amount_usd.unwrap() <= Decimal::ZERO {
                    return Err("Amount USD must be positive".to_string());
                }
            } else {
                // Volatile Cryptos MUST use amount
                if self.amount.is_none() {
                    return Err(format!("{} payments require 'amount'", ct));
                }
                if self.amount_usd.is_some() {
                    return Err(format!("{} payments should use 'amount', not 'amount_usd'", ct));
                }
                if self.amount.unwrap() <= Decimal::ZERO {
                    return Err("Amount must be positive".to_string());
                }
            }
        } else {
            // Multi-currency checkout MUST use amount_usd
            if self.amount_usd.is_none() {
                return Err("Multi-currency checkout requires 'amount_usd'".to_string());
            }
            if self.amount_usd.unwrap() <= Decimal::ZERO {
                return Err("Amount USD must be positive".to_string());
            }
        }
        Ok(())
    }
}

/// Payment response
#[derive(Debug, Serialize)]
pub struct PaymentResponse {
    pub payment_id: String,
    pub crypto_type: Option<String>, 
    #[serde(with = "rust_decimal::serde::str_option")]
    pub amount: Option<Decimal>,
    #[serde(with = "rust_decimal::serde::str")]
    pub amount_usd: Decimal,
    pub to_address: Option<String>,
    pub status: PaymentStatus,
    pub confirmations: i32,
    pub required_confirmations: i32,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub confirmed_at: Option<DateTime<Utc>>,
    pub description: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub network: Option<String>,
    pub deposit_address: Option<String>,
    pub payment_link: Option<String>,
    pub qr_code_data: Option<String>,
    #[serde(with = "rust_decimal::serde::str_option")]
    pub fee_amount: Option<Decimal>,
    #[serde(with = "rust_decimal::serde::str_option")]
    pub fee_amount_usd: Option<Decimal>,
    pub transaction_hash: Option<String>,
    pub from_address: Option<String>,
    pub partial_payments: Option<serde_json::Value>,
    pub last_verification_at: Option<DateTime<Utc>>,
}

impl From<crate::models::payment::Payment> for PaymentResponse {
    fn from(payment: crate::models::payment::Payment) -> Self {
        Self {
            payment_id: payment.payment_id,
            crypto_type: payment.crypto_type, 
            amount: payment.amount,
            amount_usd: payment.amount_usd,
            to_address: payment.to_address.clone(),
            status: PaymentStatus::from_string(&payment.status),
            confirmations: payment.confirmations.unwrap_or(0),
            required_confirmations: payment.required_confirmations.unwrap_or(1),
            expires_at: payment.expires_at,
            created_at: payment.created_at,
            confirmed_at: payment.confirmed_at,
            description: payment.description,
            metadata: payment.metadata,
            network: payment.network,
            deposit_address: payment.to_address,
            payment_link: None,
            qr_code_data: None,
            fee_amount: payment.fee_amount,
            fee_amount_usd: Some(payment.fee_amount_usd),
            transaction_hash: payment.transaction_hash,
            from_address: payment.from_address,
            partial_payments: None,
            last_verification_at: payment.last_verification_at,
        }
    }
}
#[derive(Debug, Deserialize)]
pub struct PaymentFilters {
    pub status: Option<String>,
    pub crypto_type: Option<String>,
    pub blockchain: Option<String>,
    pub from_date: Option<DateTime<Utc>>,
    pub to_date: Option<DateTime<Utc>>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub is_sandbox: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct PaymentList {
    pub data: Vec<PaymentResponse>,
    pub pagination: PaginationInfo,
}

#[derive(Debug, Serialize)]
pub struct PaginationInfo {
    pub page: i64,
    pub page_size: i64,
    pub total_pages: i64,
    pub total_count: i64,
}

pub type PaymentTransaction = crate::models::payment::Payment;
pub type PartialPaymentInfo = crate::models::payment::Payment;
pub type PartialPaymentRecord = crate::models::payment::Payment;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainTransaction {
    pub hash: String,
    pub from_address: String,
    pub to_address: String,
    pub amount: Decimal,
    pub confirmations: u32,
    pub block_number: Option<u64>,
    pub timestamp: Option<DateTime<Utc>>,
    pub success: bool,
}
