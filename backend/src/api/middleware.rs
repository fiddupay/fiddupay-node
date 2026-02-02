use crate::config::Config;
use axum::{
    body::Body,
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use governor::{Quota, RateLimiter, state::{InMemoryState, NotKeyed}, clock::DefaultClock};
use std::sync::Arc;
use std::num::NonZeroU32;
use std::collections::HashMap;
use tokio::sync::RwLock;

pub type KeyLimiter = Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>;

pub struct IpRateLimiter {
    limiters: RwLock<HashMap<String, KeyLimiter>>,
    quota: Quota,
}

impl IpRateLimiter {
    pub fn new(requests_per_minute: u32) -> Self {
        let quota = Quota::per_minute(NonZeroU32::new(requests_per_minute).unwrap_or(NonZeroU32::new(100).unwrap()));
        Self {
            limiters: RwLock::new(HashMap::new()),
            quota,
        }
    }

    pub async fn check(&self, ip: &str) -> Result<(), ()> {
        let mut limiters = self.limiters.write().await;
        let limiter = limiters
            .entry(ip.to_string())
            .or_insert_with(|| Arc::new(RateLimiter::direct(self.quota)))
            .clone();
        
        limiter.check().map_err(|_| ())
    }
}

pub type RateLimiterInstance = Arc<IpRateLimiter>;

pub fn create_rate_limit_layer(config: &Config) -> RateLimiterInstance {
    let requests_per_minute = if config.rate_limit_requests_per_minute > 0 {
        config.rate_limit_requests_per_minute
    } else {
        100
    };
    
    Arc::new(IpRateLimiter::new(requests_per_minute))
}

pub async fn rate_limit_middleware(
    axum::extract::State(limiter): axum::extract::State<RateLimiterInstance>,
    request: Request,
    next: Next,
) -> Response {
    let ip = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.split(',').next())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    match limiter.check(&ip).await {
        Ok(_) => next.run(request).await,
        Err(_) => {
            (
                StatusCode::TOO_MANY_REQUESTS,
                "Too many requests",
            )
                .into_response()
        }
    }
}
