// Delora Endpoint Rate Limiter
// Per-endpoint Redis-based sliding window rate limiting
// Complements the global IP rate limiter (100/min) with tighter bounds
// for the quote endpoint (which burns external Delora API quota).

use redis::AsyncCommands;
use std::time::Duration;
use tracing::warn;

pub struct DeloraRateLimiter {
    redis: redis::Client,
    quote_limit_per_min: u32,
    register_limit_per_min: u32,
    ban_threshold_per_5min: u32,
    ban_duration: Duration,
}

impl DeloraRateLimiter {
    pub fn new(redis: redis::Client) -> Self {
        Self {
            redis,
            quote_limit_per_min: 30, // quotes are expensive (external API call)
            register_limit_per_min: 10, // registrations are cheap, but POST
            ban_threshold_per_5min: 150, // if a single IP burns 150 quotes in 5 min, auto-ban
            ban_duration: Duration::from_secs(3600), // 1-hour ban
        }
    }

    /// Check if the IP is allowed to request a quote. Returns Ok(()) or an error.
    pub async fn check_quote(&self, ip: &str) -> Result<(), DeloraRateLimitError> {
        self.check_ip_ban(ip).await?;
        let key = format!("delora:rl:quote:{}", ip);
        self.check_sliding_window(&key, self.quote_limit_per_min, 60)
            .await?;
        self.check_abuse_window(ip).await?;
        Ok(())
    }

    /// Check if the IP is allowed to register a cross-chain tx.
    pub async fn check_register(&self, ip: &str) -> Result<(), DeloraRateLimitError> {
        self.check_ip_ban(ip).await?;
        let key = format!("delora:rl:register:{}", ip);
        self.check_sliding_window(&key, self.register_limit_per_min, 60)
            .await?;
        Ok(())
    }

    /// Ban an IP explicitly (caller guarantees this is warranted).
    pub async fn ban_ip(&self, ip: &str) -> Result<(), redis::RedisError> {
        let mut conn = self.redis.get_multiplexed_async_connection().await?;
        let ban_key = format!("delora:ban:{}", ip);
        let _: Result<(), redis::RedisError> = conn
            .set_ex(&ban_key, "1", self.ban_duration.as_secs())
            .await;
        warn!(ip = %ip, duration_secs = self.ban_duration.as_secs(), "Delora rate limiter: IP banned");
        Ok(())
    }

    // ── Internal ───────────────────────────────────────────────────────────

    async fn check_ip_ban(&self, ip: &str) -> Result<(), DeloraRateLimitError> {
        let mut conn = self
            .redis
            .get_multiplexed_async_connection()
            .await
            .map_err(|_| DeloraRateLimitError::RedisError)?;
        let ban_key = format!("delora:ban:{}", ip);
        let banned: bool = conn.exists(&ban_key).await.unwrap_or(false);
        if banned {
            return Err(DeloraRateLimitError::Banned);
        }
        Ok(())
    }

    async fn check_sliding_window(
        &self,
        key: &str,
        limit: u32,
        window_secs: u64,
    ) -> Result<(), DeloraRateLimitError> {
        let mut conn = self
            .redis
            .get_multiplexed_async_connection()
            .await
            .map_err(|_| DeloraRateLimitError::RedisError)?;

        let now = chrono::Utc::now().timestamp_millis() as f64;
        let window_start = now - ((window_secs as f64) * 1000.0);

        // Remove expired entries
        let _: Result<(), redis::RedisError> = conn.zrembyscore(key, "-inf", window_start).await;

        // Count current entries in the window
        let count: u32 = conn.zcount(key, window_start, now).await.unwrap_or(0);

        if count >= limit {
            return Err(DeloraRateLimitError::RateLimited {
                limit,
                window_secs,
                current: count,
            });
        }

        // Add current request with unique score to avoid collisions
        let _: Result<(), redis::RedisError> = conn.zadd(key, now, now).await;

        // Set expiry on the key itself (cleanup)
        let _: Result<(), redis::RedisError> = conn.expire(key, (window_secs * 2) as i64).await;

        Ok(())
    }

    /// Anti-abuse: track total quotes across a 5-minute window. If threshold
    /// is exceeded, auto-ban the IP.
    async fn check_abuse_window(&self, ip: &str) -> Result<(), DeloraRateLimitError> {
        let mut conn = self
            .redis
            .get_multiplexed_async_connection()
            .await
            .map_err(|_| DeloraRateLimitError::RedisError)?;

        let key = format!("delora:rl:abuse:{}", ip);
        let window_secs = 300; // 5 minutes
        let now = chrono::Utc::now().timestamp_millis() as f64;
        let window_start = now - ((window_secs as f64) * 1000.0);

        let _: Result<(), redis::RedisError> = conn.zrembyscore(&key, "-inf", window_start).await;

        let count: u32 = conn.zcount(&key, window_start, now).await.unwrap_or(0);
        let _: Result<(), redis::RedisError> = conn.zadd(&key, now, now).await;

        if count >= self.ban_threshold_per_5min {
            warn!(
                ip = %ip,
                count = count,
                threshold = self.ban_threshold_per_5min,
                "Delora rate limiter: abuse threshold exceeded, auto-banning IP"
            );
            drop(conn);
            self.ban_ip(ip).await.ok();
            return Err(DeloraRateLimitError::Banned);
        }

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DeloraRateLimitError {
    #[error("Rate limited: {current}/{limit} requests in the last {window_secs}s")]
    RateLimited {
        limit: u32,
        window_secs: u64,
        current: u32,
    },
    #[error("IP temporarily banned due to excessive requests")]
    Banned,
    #[error("Redis error in rate limiter")]
    RedisError,
}

impl axum::response::IntoResponse for DeloraRateLimitError {
    fn into_response(self) -> axum::response::Response {
        use axum::http::StatusCode;
        let (status, message) = match self {
            DeloraRateLimitError::RateLimited {
                limit, window_secs, ..
            } => (
                StatusCode::TOO_MANY_REQUESTS,
                format!(
                    "Too many requests. Limit: {}/{}s. Please wait.",
                    limit, window_secs
                ),
            ),
            DeloraRateLimitError::Banned => (
                StatusCode::FORBIDDEN,
                "Access temporarily suspended due to excessive requests.".into(),
            ),
            DeloraRateLimitError::RedisError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Rate limiter unavailable.".into(),
            ),
        };
        (status, message).into_response()
    }
}
