use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Debug, Serialize, Deserialize)]
pub struct MerchantCurrency {
    pub crypto_type: String,
    pub is_enabled: bool,
    pub wallet_address: Option<String>,
}

pub struct CurrencyService {
    pool: PgPool,
}

impl CurrencyService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_supported_currencies(&self) -> Vec<(&'static str, &'static str, &'static str, &'static str)> {
        vec![
            // (crypto_type, currency_group, network_name, icon_url)
            ("USDT_ETH", "USDT", "Ethereum (ERC-20)", "https://raw.githubusercontent.com/spothq/cryptocurrency-icons/master/128/color/usdt.png"),
            ("USDT_BSC", "USDT", "Binance Smart Chain (BEP-20)", "https://raw.githubusercontent.com/spothq/cryptocurrency-icons/master/128/color/usdt.png"),
            ("USDT_POLYGON", "USDT", "Polygon (MATIC)", "https://raw.githubusercontent.com/spothq/cryptocurrency-icons/master/128/color/usdt.png"),
            ("USDT_ARBITRUM", "USDT", "Arbitrum One", "https://raw.githubusercontent.com/spothq/cryptocurrency-icons/master/128/color/usdt.png"),
            ("USDT_SOL", "USDT", "Solana (SPL)", "https://raw.githubusercontent.com/spothq/cryptocurrency-icons/master/128/color/usdt.png"),
            ("ETH", "ETH", "Ethereum", "https://raw.githubusercontent.com/spothq/cryptocurrency-icons/master/128/color/eth.png"),
            ("ARB", "ARB", "Arbitrum One", "https://cryptologos.cc/logos/arbitrum-arb-logo.png"),
            ("SOL", "SOL", "Solana", "https://raw.githubusercontent.com/spothq/cryptocurrency-icons/master/128/color/sol.png"),
            ("MATIC", "MATIC", "Polygon", "https://raw.githubusercontent.com/spothq/cryptocurrency-icons/master/128/color/matic.png"),
            ("BNB", "BNB", "Binance Smart Chain", "https://raw.githubusercontent.com/spothq/cryptocurrency-icons/master/128/color/bnb.png"),
        ]
    }

    pub fn get_currency_children(&self, currency_group: &str) -> Vec<&'static str> {
        match currency_group {
            "USDT" => vec!["USDT_ETH", "USDT_BSC", "USDT_POLYGON", "USDT_ARBITRUM", "USDT_SOL"],
            "ETH" => vec!["ETH"],
            "ARB" => vec!["ARB"],
            "SOL" => vec!["SOL"],
            "MATIC" => vec!["MATIC"],
            "BNB" => vec!["BNB"],
            _ => vec![],
        }
    }

    pub fn get_network_name(&self, crypto_type: &str) -> &'static str {
        match crypto_type {
            "USDT_ETH" | "ETH" => "Ethereum (ERC-20)",
            "USDT_BSC" | "BNB" => "Binance Smart Chain (BEP-20)",
            "USDT_POLYGON" | "MATIC" => "Polygon (MATIC)",
            "USDT_ARBITRUM" | "ARB" => "Arbitrum One",
            "USDT_SOL" | "SOL" => "Solana (SPL)",
            "BTC" => "Bitcoin",
            _ => "Unknown Network",
        }
    }

    pub fn get_required_confirmations(&self, crypto_type: &str) -> u32 {
        match crypto_type {
            "USDT_ETH" | "ETH" => 12,
            "USDT_BSC" | "BNB" => 15,
            "USDT_POLYGON" | "MATIC" => 30,
            "USDT_ARBITRUM" | "ARB" => 1,
            "USDT_SOL" | "SOL" => 32,
            "BTC" => 6,
            _ => 1,
        }
    }
}
