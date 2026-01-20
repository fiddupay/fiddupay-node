use redis::AsyncCommands;
use rust_decimal::Decimal;
use std::str::FromStr;
use crate::payment::price_fetcher::PriceFetcher;
use crate::payment::models::CryptoType;
use crate::error::ServiceError;
use crate::utils::circuit_breaker::CircuitBreaker;
use std::sync::Arc;

pub struct CachedPriceFetcher {
    redis: redis::Client,
    fetcher: PriceFetcher,
    circuit_breaker: Arc<CircuitBreaker>,
}

impl CachedPriceFetcher {
    pub fn new(redis_url: &str) -> Result<Self, ServiceError> {
        let redis = redis::Client::open(redis_url)
            .map_err(|e| ServiceError::InternalError(format!("Redis connection failed: {}", e)))?;
        
        Ok(Self {
            redis,
            fetcher: PriceFetcher::new(),
            circuit_breaker: Arc::new(CircuitBreaker::new(3, 60)),
        })
    }

    pub async fn get_price(&self, crypto_type: &CryptoType) -> Result<Decimal, ServiceError> {
        let cache_key = format!("price:{:?}", crypto_type);
        
        // Try cache first
        if let Ok(mut conn) = self.redis.get_multiplexed_async_connection().await {
            if let Ok(Some(cached)) = conn.get::<_, Option<String>>(&cache_key).await {
                if let Ok(price) = Decimal::from_str(&cached) {
                    return Ok(price);
                }
            }
        }

        // Fetch from API with circuit breaker
        let price = self.circuit_breaker.call(|| async {
            self.fetcher.fetch_price(crypto_type).await
                .map_err(|e| ServiceError::InternalError(e))
        }).await?;

        // Cache for 30 seconds
        if let Ok(mut conn) = self.redis.get_multiplexed_async_connection().await {
            let _: Result<(), _> = conn.set_ex(&cache_key, price.to_string(), 30).await;
        }

        Ok(price)
    }

    pub async fn get_price_with_fallback(&self, crypto_type: &CryptoType) -> Result<Decimal, ServiceError> {
        match self.get_price(crypto_type).await {
            Ok(price) => Ok(price),
            Err(_) => {
                // Try to get stale cache
                if let Ok(mut conn) = self.redis.get_multiplexed_async_connection().await {
                    let cache_key = format!("price:{:?}:backup", crypto_type);
                    if let Ok(Some(cached)) = conn.get::<_, Option<String>>(&cache_key).await {
                        if let Ok(price) = Decimal::from_str(&cached) {
                            return Ok(price);
                        }
                    }
                }
                Err(ServiceError::InternalError("Price unavailable".to_string()))
            }
        }
    }

    pub async fn get_price_in_currency(&self, crypto_type: &CryptoType, currency: &str) -> Result<Decimal, ServiceError> {
        let usd_price = self.get_price(crypto_type).await?;
        
        if currency == "USD" {
            return Ok(usd_price);
        }

        // Get exchange rate
        let rate = self.get_exchange_rate(currency).await?;
        Ok(usd_price * rate)
    }

    async fn get_exchange_rate(&self, currency: &str) -> Result<Decimal, ServiceError> {
        let cache_key = format!("exchange_rate:{}", currency);
        
        // Try cache
        if let Ok(mut conn) = self.redis.get_multiplexed_async_connection().await {
            if let Ok(Some(cached)) = conn.get::<_, Option<String>>(&cache_key).await {
                if let Ok(rate) = Decimal::from_str(&cached) {
                    return Ok(rate);
                }
            }
        }

        // Fetch from API (simplified - would use real forex API)
        let rate = match currency {
            "EUR" => Decimal::from_str("0.92").unwrap(),
            "GBP" => Decimal::from_str("0.79").unwrap(),
            _ => return Err(ServiceError::ValidationError(format!("Unsupported currency: {}", currency))),
        };

        // Cache for 5 minutes
        if let Ok(mut conn) = self.redis.get_multiplexed_async_connection().await {
            let _: Result<(), _> = conn.set_ex(&cache_key, rate.to_string(), 300).await;
        }

        Ok(rate)
    }
}
