// Price Fetcher for Payment Gateway
// Fetches real-time cryptocurrency prices from Bybit

use reqwest::Client;
use rust_decimal::Decimal;
use serde::Deserialize;
use std::str::FromStr;
use tracing::{info, warn};

// Get Bybit API URL from environment or use default
fn get_bybit_api_url() -> String {
    std::env::var("BYBIT_PRICE_API_URL")
        .unwrap_or_else(|_| "https://api.bybit.com".to_string())
}

#[derive(Debug, Deserialize)]
struct BybitResponse<T> {
    #[serde(rename = "retCode")]
    ret_code: i32,
    #[serde(rename = "retMsg")]
    ret_msg: String,
    result: T,
}

#[derive(Debug, Deserialize)]
struct TickerResult {
    list: Vec<TickerData>,
}

#[derive(Debug, Deserialize)]
struct TickerData {
    symbol: String,
    #[serde(rename = "lastPrice")]
    last_price: String,
}

pub struct PriceFetcher {
    client: Client,
}

impl PriceFetcher {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    /// Get SOL/USDT price from Bybit (spot or perpetual - they're usually the same)
    pub async fn get_sol_price(&self) -> Result<Decimal, Box<dyn std::error::Error + Send + Sync>> {
        // Try perpetual futures first (usually more liquid)
        match self.get_price("SOLUSDT", "linear").await {
            Ok(price) => return Ok(price),
            Err(_) => {
                // Fallback to spot price
                info!("Falling back to spot price for SOL/USDT");
                self.get_price("SOLUSDT", "spot").await
            }
        }
    }

    /// Get price for any symbol from Bybit
    pub async fn get_price(
        &self,
        symbol: &str,
        category: &str, // "spot" or "linear" (perpetual)
    ) -> Result<Decimal, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!(
            "{}/v5/market/tickers?category={}&symbol={}",
            get_bybit_api_url(), category, symbol
        );

        let response = self.client.get(&url).send().await?;
        let bybit_response: BybitResponse<TickerResult> = response.json().await?;

        if bybit_response.ret_code != 0 {
            return Err(format!("Bybit API error: {}", bybit_response.ret_msg).into());
        }

        let ticker = bybit_response.result.list.first()
            .ok_or("No ticker data found")?;

        let price = Decimal::from_str(&ticker.last_price)?;

        info!(" {}/{} price: ${}", symbol, category, price);
        Ok(price)
    }

    /// Calculate crypto amount from USD amount
    pub async fn calculate_crypto_amount(
        &self,
        usd_amount: Decimal,
        crypto: &str,
    ) -> Result<Decimal, Box<dyn std::error::Error + Send + Sync>> {
        match crypto {
            "USDT" => {
                // 1 USDT = 1 USD (stablecoin)
                Ok(usd_amount)
            }
            "SOL" => {
                let sol_price = self.get_sol_price().await?;
                if sol_price <= Decimal::ZERO {
                    return Err("Invalid SOL price".into());
                }
                // Amount in SOL = USD amount / SOL price
                Ok(usd_amount / sol_price)
            }
            _ => Err(format!("Unsupported crypto: {}", crypto).into()),
        }
    }
}

impl Default for PriceFetcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires network access
    async fn test_get_sol_price() {
        let fetcher = PriceFetcher::new();
        match fetcher.get_sol_price().await {
            Ok(price) => {
                println!("SOL/USDT price: ${}", price);
                assert!(price > Decimal::ZERO);
            }
            Err(e) => {
                println!("Error fetching price: {}", e);
            }
        }
    }

    #[tokio::test]
    #[ignore] // Requires network access
    async fn test_calculate_crypto_amount() {
        let fetcher = PriceFetcher::new();
        let usd_amount = Decimal::from(100); // $100

        // Test USDT (should be 1:1)
        match fetcher.calculate_crypto_amount(usd_amount, "USDT").await {
            Ok(amount) => {
                println!("$100 = {} USDT", amount);
                assert_eq!(amount, usd_amount);
            }
            Err(e) => println!("Error: {}", e),
        }

        // Test SOL (should divide by price)
        match fetcher.calculate_crypto_amount(usd_amount, "SOL").await {
            Ok(amount) => {
                println!("$100 = {} SOL", amount);
                assert!(amount > Decimal::ZERO);
            }
            Err(e) => println!("Error: {}", e),
        }
    }
}
