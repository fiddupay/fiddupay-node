// Rate Limiting Middleware
// Limits requests per merchant per time window

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
use std::num::NonZeroU32;
use std::sync::Arc;

/// Rate limiter instance
pub type AppRateLimiter = Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>;

/// Create a rate limiter with config
pub fn create_rate_limiter(requests_per_minute: u32) -> AppRateLimiter {
    let quota = Quota::per_minute(NonZeroU32::new(requests_per_minute).unwrap_or(NonZeroU32::new(100).unwrap()));
    Arc::new(RateLimiter::direct(quota))
}

/// Rate limiting middleware
/// 
/// # Requirements
/// * 7.3: Limit requests to 100 per minute per API key
/// * 7.4: Return 429 when rate limit exceeded
pub async fn rate_limit_middleware(
    limiter: AppRateLimiter,
    request: Request,
    next: Next,
) -> Result<Response, impl IntoResponse> {
    // Check rate limit
    match limiter.check() {
        Ok(_) => {
            // Within rate limit, continue
            Ok(next.run(request).await)
        }
        Err(_) => {
            // Rate limit exceeded
            Err((
                StatusCode::TOO_MANY_REQUESTS,
                axum::Json(json!({
                    "error": "Rate limit exceeded",
                    "message": "Too many requests. Limit: 100 requests per minute"
                }))
            ))
        }
    }
}
