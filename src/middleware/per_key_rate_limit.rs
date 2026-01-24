// Per-API-Key Rate Limiting Middleware
// Implements individual rate limits for each API key

use crate::middleware::auth::MerchantContext;
use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use governor::{
    clock::DefaultClock,
    state::{InMemoryState, NotKeyed},
    Quota, RateLimiter,
};
use serde_json::json;
use std::collections::HashMap;
use std::num::NonZeroU32;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Per-key rate limiter
pub type KeyRateLimiter = Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>;

/// Rate limiter storage
pub struct PerKeyRateLimiter {
    limiters: RwLock<HashMap<String, KeyRateLimiter>>,
}

impl PerKeyRateLimiter {
    pub fn new() -> Self {
        Self {
            limiters: RwLock::new(HashMap::new()),
        }
    }

    /// Get or create rate limiter for API key
    pub async fn get_limiter(&self, api_key: &str) -> KeyRateLimiter {
        let mut limiters = self.limiters.write().await;
        limiters
            .entry(api_key.to_string())
            .or_insert_with(|| {
                let quota = Quota::per_minute(NonZeroU32::new(100).unwrap());
                Arc::new(RateLimiter::direct(quota))
            })
            .clone()
    }

    /// Check rate limit for API key
    pub async fn check(&self, api_key: &str) -> Result<(), ()> {
        let limiter = self.get_limiter(api_key).await;
        limiter.check().map_err(|_| ())
    }

    /// Cleanup old limiters (call periodically)
    pub async fn cleanup(&self) {
        let mut limiters = self.limiters.write().await;
        limiters.retain(|_, limiter| Arc::strong_count(limiter) > 1);
    }
}

/// Per-API-key rate limiting middleware
pub async fn per_key_rate_limit_middleware(
    limiter: Arc<PerKeyRateLimiter>,
    request: Request,
    next: Next,
) -> Result<Response, impl IntoResponse> {
    // Get merchant context (must run after auth middleware)
    if let Some(context) = request.extensions().get::<MerchantContext>() {
        // Check rate limit for this API key
        match limiter.check(&context.api_key).await {
            Ok(_) => Ok(next.run(request).await),
            Err(_) => Err((
                StatusCode::TOO_MANY_REQUESTS,
                axum::Json(json!({
                    "error": "Rate limit exceeded",
                    "message": "Too many requests for this API key. Limit: 100 requests per minute"
                }))
            )),
        }
    } else {
        // No merchant context, allow request (public endpoints)
        Ok(next.run(request).await)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_per_key_rate_limiting() {
        let limiter = Arc::new(PerKeyRateLimiter::new());
        
        // First request should succeed
        assert!(limiter.check("test_key_1").await.is_ok());
        
        // Different key should have separate limit
        assert!(limiter.check("test_key_2").await.is_ok());
        
        // Same key should share limit
        for _ in 0..99 {
            assert!(limiter.check("test_key_1").await.is_ok());
        }
        
        // 101st request should fail
        assert!(limiter.check("test_key_1").await.is_err());
        
        // Different key should still work
        assert!(limiter.check("test_key_2").await.is_ok());
    }

    #[tokio::test]
    async fn test_cleanup() {
        let limiter = Arc::new(PerKeyRateLimiter::new());
        
        // Create limiter for key
        let _ = limiter.get_limiter("test_key").await;
        
        // Should have 1 limiter
        assert_eq!(limiter.limiters.read().await.len(), 1);
        
        // Cleanup should remove unused limiters
        limiter.cleanup().await;
        assert_eq!(limiter.limiters.read().await.len(), 0);
    }
}
