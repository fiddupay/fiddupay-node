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
    // The "Singleton State" - always current, always fast
    prices: Arc<RwLock<HashMap<CryptoType, f64>>>,
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
            prices: Arc::new(RwLock::new(HashMap::new())), // In-memory fast storage
            failure_tracker: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl: Duration::from_secs(config.price_cache_ttl_seconds as u64),
            failure_threshold: 3,
            failure_reset_duration: Duration::from_secs(900), // 15 minutes
            config,
            in_flight_requests: Arc::new(RwLock::new(HashMap::new())),
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
                        cache.insert(key_clone.clone(), PriceCache {
                            price,
                            timestamp: Instant::now(),
                        });
                    }
                    
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
        let cryptos = vec![
            CryptoType::Sol,
            CryptoType::Eth,
            CryptoType::Arb,
            CryptoType::Matic,
            CryptoType::Bnb,
            CryptoType::Btc,
        ];
        
        for crypto in cryptos {
            // Fetch fresh price
            match self.fetch_price(crypto).await {
                Ok(price) => {
                    let mut prices = self.prices.write().await;
                    prices.insert(crypto, price);
                }
                Err(e) => {
                    warn!("[PRICE] Warmup failed for {:?}: {}", crypto, e);
                }
            }
            // Stagger requests
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
        Ok(())
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
            config: self.config.clone(),
            in_flight_requests: Arc::clone(&self.in_flight_requests),
        }
    }
}
