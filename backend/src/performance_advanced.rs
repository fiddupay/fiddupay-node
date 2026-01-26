// Advanced Performance Optimizations
// Additional performance improvements for critical paths

use std::sync::Arc;
use tokio::sync::OnceCell;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Connection pool with optimized settings for high performance
pub struct HighPerformancePool {
    pool: sqlx::PgPool,
}

impl HighPerformancePool {
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(50) // Increased for high load
            .min_connections(10) // Keep connections warm
            .acquire_timeout(std::time::Duration::from_secs(5))
            .idle_timeout(std::time::Duration::from_secs(600)) // 10 minutes
            .max_lifetime(std::time::Duration::from_secs(3600)) // 1 hour
            .test_before_acquire(false) // Skip health checks for performance
            .after_connect(|conn, _meta| {
                Box::pin(async move {
                    // Optimize connection settings
                    sqlx::query("SET statement_timeout = '30s'").execute(&mut *conn).await?;
                    sqlx::query("SET lock_timeout = '10s'").execute(&mut *conn).await?;
                    sqlx::query("SET idle_in_transaction_session_timeout = '60s'").execute(&mut *conn).await?;
                    sqlx::query("SET tcp_keepalives_idle = '300'").execute(&mut *conn).await?;
                    sqlx::query("SET tcp_keepalives_interval = '30'").execute(&mut *conn).await?;
                    sqlx::query("SET tcp_keepalives_count = '3'").execute(&mut *conn).await?;
                    Ok(())
                })
            })
            .connect(database_url)
            .await?;

        Ok(Self { pool })
    }

    pub fn pool(&self) -> &sqlx::PgPool {
        &self.pool
    }
}

/// Optimized query patterns for common operations
pub struct OptimizedQueries;

impl OptimizedQueries {
    /// Get merchant with caching-friendly query
    pub async fn get_merchant_optimized(
        pool: &sqlx::PgPool,
        merchant_id: i64,
    ) -> Result<Option<crate::models::merchant::Merchant>, sqlx::Error> {
        sqlx::query_as!(
            crate::models::merchant::Merchant,
            r#"
            SELECT id, email, business_name, api_key_hash, fee_percentage, 
                   is_active, sandbox_mode, created_at, updated_at
            FROM merchants 
            WHERE id = $1 AND is_active = true
            "#,
            merchant_id
        )
        .fetch_optional(pool)
        .await
    }

    /// Get payment_transactions with optimized pagination
    pub async fn get_payment_transactions_paginated(
        pool: &sqlx::PgPool,
        merchant_id: i64,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<crate::models::payment::Payment>, sqlx::Error> {
        sqlx::query_as!(
            crate::models::payment::Payment,
            r#"
            SELECT id, payment_id, merchant_id, amount, amount_usd, crypto_type, 
                   status, to_address, expires_at, created_at,
                   confirmed_at, description, metadata, confirmations, required_confirmations
            FROM payment_transactions 
            WHERE merchant_id = $1 
            ORDER BY created_at DESC 
            LIMIT $2 OFFSET $3
            "#,
            merchant_id,
            limit,
            offset
        )
        .fetch_all(pool)
        .await
    }

    /// Bulk update payment statuses efficiently
    pub async fn bulk_update_payment_status(
        pool: &sqlx::PgPool,
        updates: &[(String, String)], // (payment_id, status)
    ) -> Result<u64, sqlx::Error> {
        if updates.is_empty() {
            return Ok(0);
        }

        let payment_ids: Vec<String> = updates.iter().map(|(id, _)| id.clone()).collect();
        let statuses: Vec<String> = updates.iter().map(|(_, status)| status.clone()).collect();

        let result = sqlx::query!(
            r#"
            UPDATE payment_transactions 
            SET status = data.status
            FROM (
                SELECT unnest($1::text[]) as payment_id, 
                       unnest($2::text[]) as status
            ) as data
            WHERE payment_transactions.payment_id = data.payment_id
            "#,
            &payment_ids,
            &statuses
        )
        .execute(pool)
        .await?;

        Ok(result.rows_affected())
    }
}

/// Memory-efficient response caching
#[derive(Clone, Serialize, Deserialize)]
pub struct CachedResponse {
    pub data: Vec<u8>,
    pub content_type: String,
    pub cached_at: chrono::DateTime<chrono::Utc>,
}

pub struct ResponseCache {
    cache: Arc<tokio::sync::RwLock<HashMap<String, CachedResponse>>>,
}

impl ResponseCache {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }

    pub async fn get(&self, key: &str) -> Option<CachedResponse> {
        let cache = self.cache.read().await;
        if let Some(cached) = cache.get(key) {
            // Check if still valid (5 minutes)
            if chrono::Utc::now().signed_duration_since(cached.cached_at).num_minutes() < 5 {
                return Some(cached.clone());
            }
        }
        None
    }

    pub async fn set(&self, key: String, response: CachedResponse) {
        let mut cache = self.cache.write().await;
        cache.insert(key, response);
    }

    pub async fn cleanup_expired(&self) {
        let mut cache = self.cache.write().await;
        let now = chrono::Utc::now();
        cache.retain(|_, cached| {
            now.signed_duration_since(cached.cached_at).num_minutes() < 5
        });
    }
}

/// Optimized HTTP client with connection pooling
pub struct OptimizedHttpClient {
    client: reqwest::Client,
}

impl OptimizedHttpClient {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .pool_max_idle_per_host(10)
            .pool_idle_timeout(std::time::Duration::from_secs(90))
            .timeout(std::time::Duration::from_secs(30))
            .connect_timeout(std::time::Duration::from_secs(10))
            .tcp_keepalive(std::time::Duration::from_secs(60))
            .http2_prior_knowledge()
            .build()
            .expect("Failed to create HTTP client");

        Self { client }
    }

    pub fn client(&self) -> &reqwest::Client {
        &self.client
    }
}

/// Global optimized instances
static HTTP_CLIENT: OnceCell<OptimizedHttpClient> = OnceCell::const_new();
static RESPONSE_CACHE: OnceCell<ResponseCache> = OnceCell::const_new();

pub async fn get_http_client() -> &'static OptimizedHttpClient {
    HTTP_CLIENT.get_or_init(|| async {
        OptimizedHttpClient::new()
    }).await
}

pub async fn get_response_cache() -> &'static ResponseCache {
    RESPONSE_CACHE.get_or_init(|| async {
        ResponseCache::new()
    }).await
}

/// Optimized serialization helpers
pub mod serialization {
    use serde::{Serialize, Deserialize};
    use std::sync::Arc;

    /// Pre-allocated buffer for JSON serialization
    thread_local! {
        static JSON_BUFFER: std::cell::RefCell<Vec<u8>> = std::cell::RefCell::new(Vec::with_capacity(4096));
    }

    /// Fast JSON serialization with buffer reuse
    pub fn serialize_json<T: Serialize>(value: &T) -> Result<Vec<u8>, serde_json::Error> {
        JSON_BUFFER.with(|buffer| {
            let mut buf = buffer.borrow_mut();
            buf.clear();
            serde_json::to_writer(&mut *buf, value)?;
            Ok(buf.clone())
        })
    }

    /// Optimized string interning for common values
    pub struct StringInterner {
        common_strings: Arc<std::sync::RwLock<std::collections::HashMap<&'static str, Arc<str>>>>,
    }

    impl StringInterner {
        pub fn new() -> Self {
            let mut common = std::collections::HashMap::new();
            
            // Pre-intern common values
            common.insert("PENDING", "PENDING".into());
            common.insert("CONFIRMED", "CONFIRMED".into());
            common.insert("FAILED", "FAILED".into());
            common.insert("EXPIRED", "EXPIRED".into());
            common.insert("SOL", "SOL".into());
            common.insert("USDT_ETH", "USDT_ETH".into());
            common.insert("USDT_BSC", "USDT_BSC".into());
            common.insert("USDT_POLYGON", "USDT_POLYGON".into());
            common.insert("USDT_ARBITRUM", "USDT_ARBITRUM".into());

            Self {
                common_strings: Arc::new(std::sync::RwLock::new(common)),
            }
        }

        pub fn get(&self, s: &str) -> Option<Arc<str>> {
            let cache = self.common_strings.read().ok()?;
            cache.get(s).cloned()
        }
    }
}

/// Performance monitoring utilities
pub struct PerformanceMonitor;

impl PerformanceMonitor {
    /// Log slow queries for optimization
    pub async fn log_slow_query(query: &str, duration_ms: u64) {
        if duration_ms > 100 {
            tracing::warn!(
                query = query,
                duration_ms = duration_ms,
                "Slow query detected"
            );
        }
    }

    /// Monitor cache hit rates
    pub async fn log_cache_stats(cache_name: &str, hits: u64, misses: u64) {
        let total = hits + misses;
        if total > 0 {
            let hit_rate = (hits as f64 / total as f64) * 100.0;
            tracing::info!(
                cache_name = cache_name,
                hit_rate = hit_rate,
                hits = hits,
                misses = misses,
                "Cache performance"
            );
        }
    }
}
