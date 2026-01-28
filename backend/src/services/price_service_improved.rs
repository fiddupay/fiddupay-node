use crate::payment::models::CryptoType;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{info, warn};

#[derive(Clone)]
pub struct PriceCache {
    pub price: f64,
    pub timestamp: Instant,
}

#[derive(Clone)]
pub struct ApiFailureTracker {
    pub failure_count: u32,
    pub last_failure: Instant,
}

pub struct PriceService {
    cache: Arc<RwLock<HashMap<String, PriceCache>>>,
    failure_tracker: Arc<RwLock<HashMap<String, ApiFailureTracker>>>,
    cache_ttl: Duration,
    failure_threshold: u32,
    failure_reset_duration: Duration,
}

impl PriceService {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            failure_tracker: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl: Duration::from_secs(300), // 5 minutes cache
            failure_threshold: 3,
            failure_reset_duration: Duration::from_secs(900), // 15 minutes
        }
    }

    pub async fn get_price(&self, crypto_type: CryptoType) -> Result<f64, String> {
        let cache_key = format!("{:?}", crypto_type);
        
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(cached) = cache.get(&cache_key) {
                if cached.timestamp.elapsed() < self.cache_ttl {
                    return Ok(cached.price);
                }
            }
        }

        // Fetch new price
        let price = self.fetch_price(crypto_type).await?;
        
        // Update cache
        {
            let mut cache = self.cache.write().await;
            cache.insert(cache_key, PriceCache {
                price,
                timestamp: Instant::now(),
            });
        }

        Ok(price)
    }

    async fn is_api_failed(&self, api_name: &str) -> bool {
        let mut tracker = self.failure_tracker.write().await;
        if let Some(failure_info) = tracker.get_mut(api_name) {
            // Reset failure count if enough time has passed
            if failure_info.last_failure.elapsed() > self.failure_reset_duration {
                tracker.remove(api_name);
                return false;
            }
            failure_info.failure_count >= self.failure_threshold
        } else {
            false
        }
    }

    async fn record_api_failure(&self, api_name: &str) {
        let mut tracker = self.failure_tracker.write().await;
        let failure_info = tracker.entry(api_name.to_string()).or_insert(ApiFailureTracker {
            failure_count: 0,
            last_failure: Instant::now(),
        });
        
        failure_info.failure_count += 1;
        failure_info.last_failure = Instant::now();
        
        if failure_info.failure_count >= self.failure_threshold {
            warn!("[PRICE] API {} marked as failed after {} failures", api_name, failure_info.failure_count);
        }
    }

    async fn record_api_success(&self, api_name: &str) {
        let mut tracker = self.failure_tracker.write().await;
        tracker.remove(api_name);
    }

    async fn fetch_price(&self, crypto_type: CryptoType) -> Result<f64, String> {
        match crypto_type {
            CryptoType::Sol => self.fetch_sol_price().await,
            CryptoType::Eth => self.fetch_eth_price().await,
            CryptoType::Arb => self.fetch_arb_price().await,
            CryptoType::Matic => self.fetch_matic_price().await,
            CryptoType::Bnb => self.fetch_bnb_price().await,
            // USDT tokens use their blockchain's native currency price
            CryptoType::UsdtSpl => self.fetch_sol_price().await,
            CryptoType::UsdtBep20 => self.fetch_bnb_price().await,
            CryptoType::UsdtEth => self.fetch_eth_price().await,
            CryptoType::UsdtPolygon => self.fetch_matic_price().await,
            CryptoType::UsdtArbitrum => self.fetch_arb_price().await,
        }
    }

    async fn fetch_sol_price(&self) -> Result<f64, String> {
        // Primary: CoinGecko (only if not failed)
        if !self.is_api_failed("coingecko").await {
            if let Some(price) = self.fetch_from_coingecko("solana").await {
                self.record_api_success("coingecko").await;
                return Ok(price);
            } else {
                self.record_api_failure("coingecko").await;
            }
        }

        // Fallback APIs
        if let Some(price) = self.fetch_from_binance("SOLUSDT").await {
            return Ok(price);
        }
        if let Some(price) = self.fetch_from_cryptocompare("SOL").await {
            return Ok(price);
        }
        
        Err("Failed to fetch SOL price from all sources".to_string())
    }

    async fn fetch_eth_price(&self) -> Result<f64, String> {
        // Primary: CoinGecko (only if not failed)
        if !self.is_api_failed("coingecko").await {
            if let Some(price) = self.fetch_from_coingecko("ethereum").await {
                self.record_api_success("coingecko").await;
                return Ok(price);
            } else {
                self.record_api_failure("coingecko").await;
            }
        }

        // Fallback APIs
        if let Some(price) = self.fetch_from_binance("ETHUSDT").await {
            return Ok(price);
        }
        if let Some(price) = self.fetch_from_cryptocompare("ETH").await {
            return Ok(price);
        }
        
        Err("Failed to fetch ETH price from all sources".to_string())
    }

    async fn fetch_arb_price(&self) -> Result<f64, String> {
        // Primary: CoinGecko (only if not failed)
        if !self.is_api_failed("coingecko").await {
            if let Some(price) = self.fetch_from_coingecko("arbitrum").await {
                self.record_api_success("coingecko").await;
                return Ok(price);
            } else {
                self.record_api_failure("coingecko").await;
            }
        }

        // Fallback APIs
        if let Some(price) = self.fetch_from_binance("ARBUSDT").await {
            return Ok(price);
        }
        if let Some(price) = self.fetch_from_cryptocompare("ARB").await {
            return Ok(price);
        }
        
        Err("Failed to fetch ARB price from all sources".to_string())
    }

    async fn fetch_matic_price(&self) -> Result<f64, String> {
        // Primary: CoinGecko (only if not failed)
        if !self.is_api_failed("coingecko").await {
            if let Some(price) = self.fetch_from_coingecko("matic-network").await {
                self.record_api_success("coingecko").await;
                return Ok(price);
            } else {
                self.record_api_failure("coingecko").await;
            }
        }

        // Fallback APIs
        if let Some(price) = self.fetch_from_binance("MATICUSDT").await {
            return Ok(price);
        }
        if let Some(price) = self.fetch_from_cryptocompare("MATIC").await {
            return Ok(price);
        }
        
        Err("Failed to fetch MATIC price from all sources".to_string())
    }

    async fn fetch_bnb_price(&self) -> Result<f64, String> {
        // Primary: CoinGecko (only if not failed)
        if !self.is_api_failed("coingecko").await {
            if let Some(price) = self.fetch_from_coingecko("binancecoin").await {
                self.record_api_success("coingecko").await;
                return Ok(price);
            } else {
                self.record_api_failure("coingecko").await;
            }
        }

        // Fallback APIs
        if let Some(price) = self.fetch_from_binance("BNBUSDT").await {
            return Ok(price);
        }
        if let Some(price) = self.fetch_from_cryptocompare("BNB").await {
            return Ok(price);
        }
        
        Err("Failed to fetch BNB price from all sources".to_string())
    }

    async fn fetch_from_coingecko(&self, coin_id: &str) -> Option<f64> {
        let url = format!("https://api.coingecko.com/api/v3/simple/price?ids={}&vs_currencies=usd", coin_id);
        
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent("Mozilla/5.0 (compatible; FidduPay/1.0)")
            .build()
            .ok()?;
        
        match client.get(&url).send().await {
            Ok(resp) => {
                if !resp.status().is_success() {
                    warn!("[PRICE] CoinGecko returned status: {}", resp.status());
                    return None;
                }
                if let Ok(json) = resp.json::<Value>().await {
                    json[coin_id]["usd"].as_f64()
                } else {
                    None
                }
            }
            Err(e) => {
                warn!("[PRICE] CoinGecko error: {}", e);
                None
            }
        }
    }

    async fn fetch_from_binance(&self, symbol: &str) -> Option<f64> {
        let url = format!("https://api.binance.com/api/v3/ticker/price?symbol={}", symbol);
        
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent("Mozilla/5.0 (compatible; FidduPay/1.0)")
            .build()
            .ok()?;
        
        match client.get(&url).send().await {
            Ok(resp) => {
                if !resp.status().is_success() {
                    warn!("[PRICE] Binance returned status: {}", resp.status());
                    return None;
                }
                if let Ok(json) = resp.json::<Value>().await {
                    json["price"].as_str()?.parse().ok()
                } else {
                    None
                }
            }
            Err(e) => {
                warn!("[PRICE] Binance error: {}", e);
                None
            }
        }
    }

    async fn fetch_from_cryptocompare(&self, symbol: &str) -> Option<f64> {
        let url = format!("https://min-api.cryptocompare.com/data/price?fsym={}&tsyms=USD", symbol);
        
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent("Mozilla/5.0 (compatible; FidduPay/1.0)")
            .build()
            .ok()?;

        match client.get(&url).send().await {
            Ok(resp) => {
                if !resp.status().is_success() {
                    warn!("[PRICE] CryptoCompare returned status: {}", resp.status());
                    return None;
                }
                if let Ok(json) = resp.json::<Value>().await {
                    json["USD"].as_f64()
                } else {
                    None
                }
            }
            Err(e) => {
                warn!("[PRICE] CryptoCompare error: {}", e);
                None
            }
        }
    }
}
