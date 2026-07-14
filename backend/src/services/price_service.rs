use crate::payment::models::CryptoType;
use futures::FutureExt;
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

type SharedPriceFuture =
    futures::future::Shared<futures::future::BoxFuture<'static, Result<f64, String>>>;
type PriceInFlightMap = Arc<RwLock<HashMap<String, SharedPriceFuture>>>;

pub struct PriceService {
    cache: Arc<RwLock<HashMap<String, PriceCache>>>,
    // The "Singleton State" - always current, always fast
    prices: Arc<RwLock<HashMap<CryptoType, f64>>>,
    failure_tracker: Arc<RwLock<HashMap<String, ApiFailureTracker>>>,
    cache_ttl: Duration,
    failure_threshold: u32,
    failure_reset_duration: Duration,
    in_flight_requests: PriceInFlightMap,
    // Network-enabled flags — only warm up prices for active networks
    solana_enabled: bool,
    bitcoin_enabled: bool,
    ethereum_enabled: bool,
    bsc_enabled: bool,
    polygon_enabled: bool,
    arbitrum_enabled: bool,
}

impl PriceService {
    pub fn new(config: crate::config::Config) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            prices: Arc::new(RwLock::new(HashMap::new())), // In-memory fast storage
            failure_tracker: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl: Duration::from_secs(config.price_cache_ttl_seconds),
            failure_threshold: 3,
            failure_reset_duration: Duration::from_secs(900), // 15 minutes
            in_flight_requests: Arc::new(RwLock::new(HashMap::new())),
            solana_enabled: config.solana_enabled,
            bitcoin_enabled: config.bitcoin_enabled,
            ethereum_enabled: config.ethereum_enabled,
            bsc_enabled: config.bsc_enabled,
            polygon_enabled: config.polygon_enabled,
            arbitrum_enabled: config.arbitrum_enabled,
        }
    }

    /// High-performance non-blocking price retrieval.
    /// Resolves in <1ms if the singleton has been warmed up.
    pub async fn get_price(&self, crypto_type: CryptoType) -> Result<f64, String> {
        // 1. Check the singleton map first (The true High-Performance path)
        {
            let prices = self.prices.read().await;
            if let Some(&price) = prices.get(&crypto_type) {
                return Ok(price);
            }
        }

        // 2. Fallback to cache if singleton is empty (e.g. during immediate startup)
        let cache_key = format!("{:?}", crypto_type);
        if let Some(price) = self.get_cached_price(&cache_key).await {
            return Ok(price);
        }

        // 3. Fallback to fetch only if absolutely necessary
        // In the new model, this is only hit if background polling hasn't finished yet
        let shared_future = {
            let mut in_flight = self.in_flight_requests.write().await;
            if let Some(shared) = in_flight.get(&cache_key) {
                shared.clone()
            } else {
                let service = self.clone();
                let key_clone = cache_key.clone();
                let future = async move {
                    let res = service.fetch_price(crypto_type).await;

                    if let Ok(price) = res {
                        // Update singleton and cache concurrently
                        let mut p_map = service.prices.write().await;
                        p_map.insert(crypto_type, price);

                        let mut cache = service.cache.write().await;
                        cache.insert(
                            key_clone.clone(),
                            PriceCache {
                                price,
                                timestamp: Instant::now(),
                            },
                        );
                    }

                    let mut in_flight = service.in_flight_requests.write().await;
                    in_flight.remove(&key_clone);

                    res
                }
                .boxed()
                .shared();

                in_flight.insert(cache_key, future.clone());
                future
            }
        };

        shared_future.await
    }

    /// Background task that keeps the singleton warmed up.
    /// This is the ONLY place where external blocking API calls happen.
    pub fn start_background_polling(&self) {
        let service = self.clone();
        tokio::spawn(async move {
            info!("[PRICE] High-Performance Background Sync initialized");

            // Immediate warmup on start
            let _ = service.warmup_prices().await;

            let mut interval = tokio::time::interval(Duration::from_secs(300)); // Sync every 5 minutes
            loop {
                interval.tick().await;
                let _ = service.warmup_prices().await;
            }
        });
    }

    async fn warmup_prices(&self) -> Result<(), String> {
        // Build a list of only the cryptos whose network is enabled in config.
        // Stablecoins (price = 1.0) are always skipped — they don't need a fetch.
        let mut cryptos: Vec<CryptoType> = Vec::new();

        if self.solana_enabled {
            cryptos.push(CryptoType::Sol);
        }
        if self.bitcoin_enabled {
            cryptos.push(CryptoType::Btc);
        }
        if self.ethereum_enabled {
            cryptos.push(CryptoType::Eth);
        }
        if self.bsc_enabled {
            cryptos.push(CryptoType::Bnb);
        }
        if self.polygon_enabled {
            cryptos.push(CryptoType::Matic);
        }
        if self.arbitrum_enabled {
            cryptos.push(CryptoType::Arb);
        }

        if cryptos.is_empty() {
            info!("[PRICE] All networks disabled — skipping price warmup");
            return Ok(());
        }

        info!(
            "[PRICE] Warming up prices for {} enabled network(s): {:?}",
            cryptos.len(),
            cryptos
        );

        for crypto in cryptos {
            match self.fetch_price(crypto).await {
                Ok(price) => {
                    let mut prices = self.prices.write().await;
                    prices.insert(crypto, price);
                }
                Err(e) => {
                    warn!("[PRICE] Warmup failed for {:?}: {}", crypto, e);
                }
            }
            // Stagger requests to avoid hammering the price APIs
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
        Ok(())
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
        let failure_info = tracker
            .entry(api_name.to_string())
            .or_insert(ApiFailureTracker {
                failure_count: 0,
                last_failure: Instant::now(),
            });

        failure_info.failure_count += 1;
        failure_info.last_failure = Instant::now();

        if failure_info.failure_count >= self.failure_threshold {
            warn!(
                "[PRICE] API {} marked as failed after {} failures",
                api_name, failure_info.failure_count
            );
        }
    }

    async fn record_api_success(&self, api_name: &str) {
        let mut tracker = self.failure_tracker.write().await;
        tracker.remove(api_name);
    }

    async fn fetch_price(&self, crypto_type: CryptoType) -> Result<f64, String> {
        match crypto_type {
            CryptoType::Sol | CryptoType::WSol => self.fetch_sol_price().await,
            CryptoType::Eth => self.fetch_eth_price().await,
            CryptoType::Arb => self.fetch_arb_price().await,
            CryptoType::Matic => self.fetch_matic_price().await,
            CryptoType::Bnb => self.fetch_bnb_price().await,
            CryptoType::Btc => self.fetch_btc_price().await,
            CryptoType::UsdtSpl
            | CryptoType::UsdtBep20
            | CryptoType::UsdtEth
            | CryptoType::UsdtPolygon
            | CryptoType::UsdtArbitrum
            | CryptoType::BusdBep20 => Ok(1.0),
        }
    }

    async fn fetch_sol_price(&self) -> Result<f64, String> {
        if !self.is_api_failed("coingecko").await {
            if let Some(price) = self.fetch_from_coingecko("solana").await {
                self.record_api_success("coingecko").await;
                return Ok(price);
            } else {
                self.record_api_failure("coingecko").await;
            }
        }
        if let Some(price) = self.fetch_from_cryptocompare("SOL").await {
            return Ok(price);
        }
        Err("Failed to fetch SOL price".to_string())
    }

    async fn fetch_eth_price(&self) -> Result<f64, String> {
        if !self.is_api_failed("coingecko").await {
            if let Some(price) = self.fetch_from_coingecko("ethereum").await {
                self.record_api_success("coingecko").await;
                return Ok(price);
            } else {
                self.record_api_failure("coingecko").await;
            }
        }
        if let Some(price) = self.fetch_from_cryptocompare("ETH").await {
            return Ok(price);
        }
        Err("Failed to fetch ETH price".to_string())
    }

    async fn fetch_arb_price(&self) -> Result<f64, String> {
        if !self.is_api_failed("coingecko").await {
            if let Some(price) = self.fetch_from_coingecko("arbitrum").await {
                self.record_api_success("coingecko").await;
                return Ok(price);
            } else {
                self.record_api_failure("coingecko").await;
            }
        }
        if let Some(price) = self.fetch_from_cryptocompare("ARB").await {
            return Ok(price);
        }
        Err("Failed to fetch ARB price".to_string())
    }

    async fn fetch_matic_price(&self) -> Result<f64, String> {
        if !self.is_api_failed("coingecko").await {
            if let Some(price) = self.fetch_from_coingecko("matic-network").await {
                self.record_api_success("coingecko").await;
                return Ok(price);
            } else {
                self.record_api_failure("coingecko").await;
            }
        }
        if let Some(price) = self.fetch_from_cryptocompare("MATIC").await {
            return Ok(price);
        }
        Err("Failed to fetch MATIC price".to_string())
    }

    async fn fetch_bnb_price(&self) -> Result<f64, String> {
        if !self.is_api_failed("coingecko").await {
            if let Some(price) = self.fetch_from_coingecko("binancecoin").await {
                self.record_api_success("coingecko").await;
                return Ok(price);
            } else {
                self.record_api_failure("coingecko").await;
            }
        }
        if let Some(price) = self.fetch_from_cryptocompare("BNB").await {
            return Ok(price);
        }
        Err("Failed to fetch BNB price".to_string())
    }

    async fn fetch_btc_price(&self) -> Result<f64, String> {
        if !self.is_api_failed("coingecko").await {
            if let Some(price) = self.fetch_from_coingecko("bitcoin").await {
                self.record_api_success("coingecko").await;
                return Ok(price);
            } else {
                self.record_api_failure("coingecko").await;
            }
        }
        if let Some(price) = self.fetch_from_cryptocompare("BTC").await {
            return Ok(price);
        }
        Err("Failed to fetch BTC price".to_string())
    }

    async fn fetch_from_coingecko(&self, coin_id: &str) -> Option<f64> {
        let url = format!(
            "https://api.coingecko.com/api/v3/simple/price?ids={}&vs_currencies=usd",
            coin_id
        );
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent("Mozilla/5.0 (compatible; FidduPay/1.0)")
            .build()
            .ok()?;

        match client.get(&url).send().await {
            Ok(resp) => {
                if !resp.status().is_success() {
                    return None;
                }
                if let Ok(json) = resp.json::<Value>().await {
                    json[coin_id]["usd"].as_f64()
                } else {
                    None
                }
            }
            Err(_) => None,
        }
    }

    async fn fetch_from_cryptocompare(&self, symbol: &str) -> Option<f64> {
        let url = format!(
            "https://min-api.cryptocompare.com/data/price?fsym={}&tsyms=USD",
            symbol
        );
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent("Mozilla/5.0 (compatible; FidduPay/1.0)")
            .build()
            .ok()?;

        match client.get(&url).send().await {
            Ok(resp) => {
                if !resp.status().is_success() {
                    return None;
                }
                if let Ok(json) = resp.json::<Value>().await {
                    json["USD"].as_f64()
                } else {
                    None
                }
            }
            Err(_) => None,
        }
    }
}

impl Clone for PriceService {
    fn clone(&self) -> Self {
        Self {
            cache: Arc::clone(&self.cache),
            prices: Arc::clone(&self.prices),
            failure_tracker: Arc::clone(&self.failure_tracker),
            cache_ttl: self.cache_ttl,
            failure_threshold: self.failure_threshold,
            failure_reset_duration: self.failure_reset_duration,
            in_flight_requests: Arc::clone(&self.in_flight_requests),
            solana_enabled: self.solana_enabled,
            bitcoin_enabled: self.bitcoin_enabled,
            ethereum_enabled: self.ethereum_enabled,
            bsc_enabled: self.bsc_enabled,
            polygon_enabled: self.polygon_enabled,
            arbitrum_enabled: self.arbitrum_enabled,
        }
    }
}
