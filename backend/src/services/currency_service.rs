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
            ("USDT_BEP20", "USDT", "Binance Smart Chain (BEP-20)", "https://raw.githubusercontent.com/spothq/cryptocurrency-icons/master/128/color/usdt.png"),
            ("USDT_POLYGON", "USDT", "Polygon (MATIC)", "https://raw.githubusercontent.com/spothq/cryptocurrency-icons/master/128/color/usdt.png"),
            ("USDT_ARBITRUM", "USDT", "Arbitrum One", "https://raw.githubusercontent.com/spothq/cryptocurrency-icons/master/128/color/usdt.png"),
            ("USDT_SPL", "USDT", "Solana (SPL)", "https://raw.githubusercontent.com/spothq/cryptocurrency-icons/master/128/color/usdt.png"),
            ("ETH", "ETH", "Ethereum", "https://raw.githubusercontent.com/spothq/cryptocurrency-icons/master/128/color/eth.png"),
            ("ARB", "ARB", "Arbitrum One", "https://raw.githubusercontent.com/trustwallet/assets/master/blockchains/arbitrum/info/logo.png"),
            ("SOL", "SOL", "Solana (SPL)", "https://raw.githubusercontent.com/spothq/cryptocurrency-icons/master/128/color/sol.png"),
            ("WSOL", "SOL", "Solana (SPL)", "https://raw.githubusercontent.com/spothq/cryptocurrency-icons/master/128/color/sol.png"),
            ("MATIC", "MATIC", "Polygon", "https://raw.githubusercontent.com/spothq/cryptocurrency-icons/master/128/color/matic.png"),
            ("BNB", "BNB", "Binance Smart Chain", "https://raw.githubusercontent.com/spothq/cryptocurrency-icons/master/128/color/bnb.png"),
        ]
    }

    pub fn get_currency_children(&self, currency_group: &str) -> Vec<&'static str> {
        match currency_group {
            "USDT" => vec!["USDT_ETH", "USDT_BEP20", "USDT_POLYGON", "USDT_ARBITRUM", "USDT_SPL"],
            "ETH" => vec!["ETH"],
            "ARB" => vec!["ARB"],
            "SOL" => vec!["SOL", "WSOL"],
            "MATIC" => vec!["MATIC"],
            "BNB" => vec!["BNB"],
            _ => vec![],
        }
    }

    pub fn get_network_name(&self, crypto_type: &str) -> &'static str {
        match crypto_type {
            "USDT_ETH" | "ETH" => "Ethereum (ERC-20)",
            "USDT_BEP20" | "USDT_BSC" | "BNB" => "Binance Smart Chain (BEP-20)",
            "USDT_POLYGON" | "MATIC" => "Polygon (MATIC)",
            "USDT_ARBITRUM" | "ARB" => "Arbitrum One",
            "USDT_SPL" | "SOL" | "WSOL" => "Solana (SPL)",
            "BTC" => "Bitcoin",
            _ => "Unknown Network",
        }
    }

    pub fn get_required_confirmations(&self, crypto_type: &str) -> u32 {
        match crypto_type {
            "USDT_ETH" | "ETH" => 12,
            "USDT_BEP20" | "USDT_BSC" | "BNB" => 15,
            "USDT_POLYGON" | "MATIC" => 30,
            "USDT_ARBITRUM" | "ARB" => 1,
            "USDT_SPL" | "SOL" | "WSOL" => 32,
            "BTC" => 6,
            _ => 1,
        }
    }

    pub async fn get_merchant_enabled_currencies(&self, merchant_id: i64) -> Vec<(&'static str, &'static str, &'static str, &'static str)> {
        let all_supported = self.get_supported_currencies().await;
        
        // Fetch active wallets for this merchant
        let rows_res = sqlx::query(
            "SELECT crypto_type FROM merchant_wallets WHERE merchant_id = $1 AND is_active = true"
        )
        .bind(merchant_id)
        .fetch_all(&self.pool)
        .await;

        use sqlx::Row;
        let active_wallets: Vec<String> = match rows_res {
            Ok(rows) => rows.into_iter().map(|r| r.get("crypto_type")).collect(),
            Err(_) => return vec![], // Return empty if error or no wallets
        };

        // Filter supported list
        all_supported.into_iter()
            .filter(|(crypto_type, _, _, _)| {
                active_wallets.contains(&crypto_type.to_string())
            })
            .collect()
    }
}
