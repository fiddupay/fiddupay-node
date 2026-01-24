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

pub struct PriceService {
    cache: Arc<RwLock<HashMap<String, PriceCache>>>,
    cache_ttl: Duration,
}

impl PriceService {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl: Duration::from_secs(180), // 3 minutes
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

    async fn fetch_price(&self, crypto_type: CryptoType) -> Result<f64, String> {
        match crypto_type {
            CryptoType::Sol => self.fetch_sol_price().await,
            CryptoType::Eth => self.fetch_eth_price().await,
            CryptoType::Arb => self.fetch_arb_price().await,
            CryptoType::Matic => self.fetch_matic_price().await,
            CryptoType::Bnb => self.fetch_bnb_price().await,
            _ => Ok(1.0), // USDT variants = $1.00
        }
    }

    async fn fetch_sol_price(&self) -> Result<f64, String> {
        // Try multiple sources
        if let Some(price) = self.fetch_from_coingecko("solana").await {
            return Ok(price);
        }
        if let Some(price) = self.fetch_from_binance("SOLUSDT").await {
            return Ok(price);
        }
        if let Some(price) = self.fetch_from_kraken("SOLUSD").await {
            return Ok(price);
        }
        Err("Failed to fetch SOL price from all sources".to_string())
    }

    async fn fetch_eth_price(&self) -> Result<f64, String> {
        if let Some(price) = self.fetch_from_coingecko("ethereum").await {
            return Ok(price);
        }
        if let Some(price) = self.fetch_from_binance("ETHUSDT").await {
            return Ok(price);
        }
        if let Some(price) = self.fetch_from_kraken("ETHUSD").await {
            return Ok(price);
        }
        Err("Failed to fetch ETH price".to_string())
    }

    async fn fetch_arb_price(&self) -> Result<f64, String> {
        if let Some(price) = self.fetch_from_coingecko("arbitrum").await {
            return Ok(price);
        }
        if let Some(price) = self.fetch_from_binance("ARBUSDT").await {
            return Ok(price);
        }
        Err("Failed to fetch ARB price".to_string())
    }

    async fn fetch_matic_price(&self) -> Result<f64, String> {
        if let Some(price) = self.fetch_from_coingecko("matic-network").await {
            return Ok(price);
        }
        if let Some(price) = self.fetch_from_binance("MATICUSDT").await {
            return Ok(price);
        }
        Err("Failed to fetch MATIC price".to_string())
    }

    async fn fetch_bnb_price(&self) -> Result<f64, String> {
        if let Some(price) = self.fetch_from_coingecko("binancecoin").await {
            return Ok(price);
        }
        if let Some(price) = self.fetch_from_binance("BNBUSDT").await {
            return Ok(price);
        }
        Err("Failed to fetch BNB price".to_string())
    }

    async fn fetch_from_coingecko(&self, coin_id: &str) -> Option<f64> {
        let url = format!("https://api.coingecko.com/api/v3/simple/price?ids={}&vs_currencies=usd", coin_id);
        
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent("Mozilla/5.0 (compatible; PayFlow/1.0)")
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
            .user_agent("Mozilla/5.0 (compatible; PayFlow/1.0)")
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

    async fn fetch_from_kraken(&self, pair: &str) -> Option<f64> {
        let url = format!("https://api.kraken.com/0/public/Ticker?pair={}", pair);
        
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent("Mozilla/5.0 (compatible; PayFlow/1.0)")
            .build()
            .ok()?;
        
        match client.get(&url).send().await {
            Ok(resp) => {
                if !resp.status().is_success() {
                    warn!("[PRICE] Kraken returned status: {}", resp.status());
                    return None;
                }
                if let Ok(json) = resp.json::<Value>().await {
                    json["result"][pair]["c"][0].as_str()?.parse().ok()
                } else {
                    None
                }
            }
            Err(e) => {
                warn!("[PRICE] Kraken error: {}", e);
                None
            }
        }
    }

    pub fn start_background_polling(&self) {
        let cache = self.cache.clone();
        let service = self.clone();
        
        tokio::spawn(async move {
            info!("[PRICE] Starting background price polling...");
            tokio::time::sleep(Duration::from_secs(2)).await;
            
            let mut interval = tokio::time::interval(Duration::from_secs(180));
            loop {
                interval.tick().await;
                
                // Update all currency prices
                for crypto_type in [CryptoType::Sol, CryptoType::Eth, CryptoType::Arb, CryptoType::Matic, CryptoType::Bnb] {
                    if let Ok(price) = service.get_price(crypto_type).await {
                        info!("[PRICE] Updated {:?}: ${:.2}", crypto_type, price);
                    }
                }
            }
        });
    }
}

impl Clone for PriceService {
    fn clone(&self) -> Self {
        Self {
            cache: self.cache.clone(),
            cache_ttl: self.cache_ttl,
        }
    }
}
