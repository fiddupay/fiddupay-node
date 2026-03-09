use crate::payment::models::CryptoType;
use crate::payment::price_fetcher::PriceFetcher;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{info, warn};
use futures::FutureExt;
use futures::future::BoxFuture;
use futures::future::Shared;

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
    config: crate::config::Config,
    in_flight_requests: Arc<RwLock<HashMap<String, futures::future::Shared<futures::future::BoxFuture<'static, Result<f64, String>>>>>>,
}

impl PriceService {
    pub fn new(config: crate::config::Config) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            failure_tracker: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl: Duration::from_secs(config.price_cache_ttl_seconds as u64),
            failure_threshold: 3,
            failure_reset_duration: Duration::from_secs(900), // 15 minutes
            config,
            in_flight_requests: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn get_price(&self, crypto_type: CryptoType) -> Result<f64, String> {
        let cache_key = format!("{:?}", crypto_type);
        
        // 1. Check cache first
        if let Some(price) = self.get_cached_price(&cache_key).await {
            return Ok(price);
        }

        // 2. Check for in-flight request or start one
        let shared_future = {
            let mut in_flight = self.in_flight_requests.write().await;
            if let Some(shared) = in_flight.get(&cache_key) {
                shared.clone()
            } else {
                let service = self.clone();
                let key_clone = cache_key.clone();
                let future = async move {
                    let res = service.fetch_price(crypto_type).await;
                    
                    // Update cache on success
                    if let Ok(price) = res {
                        let mut cache = service.cache.write().await;
                        cache.insert(key_clone.clone(), PriceCache {
                            price,
                            timestamp: Instant::now(),
                        });
                    }
                    
                    // Clean up in-flight tracker
                    let mut in_flight = service.in_flight_requests.write().await;
                    in_flight.remove(&key_clone);
                    
                    res
                }.boxed().shared();
                
                in_flight.insert(cache_key, future.clone());
                future
            }
        };

        shared_future.await
    }

    async fn get_cached_price(&self, key: &str) -> Option<f64> {
        let cache = self.cache.read().await;
        cache.get(key).and_then(|cached| {
            if cached.timestamp.elapsed() < self.cache_ttl {
                Some(cached.price)
            } else {
                None
            }
        })
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

    pub fn start_background_polling(&self) {
        let service = self.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(300)); // 5 minutes
            loop {
                interval.tick().await;
                
                // Update all cryptocurrency prices
                let cryptos = vec![
                    CryptoType::Sol,
                    CryptoType::Eth,
                    CryptoType::Arb,
                    CryptoType::Matic,
                    CryptoType::Bnb,
                ];
                
                for crypto in cryptos {
                    if let Ok(price) = service.get_price(crypto).await {
                        info!("[PRICE] Updated {:?}: ${:.2}", crypto, price);
                    }
                }
            }
        });
    }
}

impl Clone for PriceService {
    fn clone(&self) -> Self {
        Self {
            cache: Arc::clone(&self.cache),
            failure_tracker: Arc::clone(&self.failure_tracker),
            cache_ttl: self.cache_ttl,
            failure_threshold: self.failure_threshold,
            failure_reset_duration: self.failure_reset_duration,
            config: self.config.clone(),
            in_flight_requests: Arc::clone(&self.in_flight_requests),
        }
    }
}
