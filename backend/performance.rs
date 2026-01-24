// Performance optimizations for ChainPay
// This module contains performance-critical optimizations

use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// High-performance cache for frequently accessed data
pub struct PerformanceCache {
    // Cache for merchant data to avoid repeated DB queries
    merchant_cache: Arc<RwLock<HashMap<i64, CachedMerchant>>>,
    // Cache for payment status to reduce DB load
    payment_cache: Arc<RwLock<HashMap<String, CachedPayment>>>,
    // Cache for price data to avoid repeated API calls
    price_cache: Arc<RwLock<HashMap<String, CachedPrice>>>,
}

#[derive(Clone)]
struct CachedMerchant {
    merchant_id: i64,
    email: String,
    business_name: String,
    is_active: bool,
    cached_at: DateTime<Utc>,
}

#[derive(Clone)]
struct CachedPayment {
    payment_id: String,
    status: String,
    amount_usd: String,
    cached_at: DateTime<Utc>,
}

#[derive(Clone)]
struct CachedPrice {
    crypto_type: String,
    price_usd: String,
    cached_at: DateTime<Utc>,
}

impl PerformanceCache {
    pub fn new() -> Self {
        Self {
            merchant_cache: Arc::new(RwLock::new(HashMap::new())),
            payment_cache: Arc::new(RwLock::new(HashMap::new())),
            price_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get cached merchant data if available and not expired
    pub async fn get_merchant(&self, merchant_id: i64) -> Option<CachedMerchant> {
        let cache = self.merchant_cache.read().await;
        if let Some(cached) = cache.get(&merchant_id) {
            // Cache for 5 minutes
            if Utc::now().signed_duration_since(cached.cached_at).num_minutes() < 5 {
                return Some(cached.clone());
            }
        }
        None
    }

    /// Cache merchant data
    pub async fn cache_merchant(&self, merchant: CachedMerchant) {
        let mut cache = self.merchant_cache.write().await;
        cache.insert(merchant.merchant_id, merchant);
    }

    /// Get cached payment data if available and not expired
    pub async fn get_payment(&self, payment_id: &str) -> Option<CachedPayment> {
        let cache = self.payment_cache.read().await;
        if let Some(cached) = cache.get(payment_id) {
            // Cache for 1 minute for payments (they change frequently)
            if Utc::now().signed_duration_since(cached.cached_at).num_seconds() < 60 {
                return Some(cached.clone());
            }
        }
        None
    }

    /// Cache payment data
    pub async fn cache_payment(&self, payment: CachedPayment) {
        let mut cache = self.payment_cache.write().await;
        cache.insert(payment.payment_id.clone(), payment);
    }

    /// Get cached price data if available and not expired
    pub async fn get_price(&self, crypto_type: &str) -> Option<CachedPrice> {
        let cache = self.price_cache.read().await;
        if let Some(cached) = cache.get(crypto_type) {
            // Cache for 30 seconds for prices
            if Utc::now().signed_duration_since(cached.cached_at).num_seconds() < 30 {
                return Some(cached.clone());
            }
        }
        None
    }

    /// Cache price data
    pub async fn cache_price(&self, price: CachedPrice) {
        let mut cache = self.price_cache.write().await;
        cache.insert(price.crypto_type.clone(), price);
    }

    /// Clean expired entries from all caches
    pub async fn cleanup_expired(&self) {
        let now = Utc::now();
        
        // Clean merchant cache (5 minute expiry)
        {
            let mut cache = self.merchant_cache.write().await;
            cache.retain(|_, cached| {
                now.signed_duration_since(cached.cached_at).num_minutes() < 5
            });
        }

        // Clean payment cache (1 minute expiry)
        {
            let mut cache = self.payment_cache.write().await;
            cache.retain(|_, cached| {
                now.signed_duration_since(cached.cached_at).num_seconds() < 60
            });
        }

        // Clean price cache (30 second expiry)
        {
            let mut cache = self.price_cache.write().await;
            cache.retain(|_, cached| {
                now.signed_duration_since(cached.cached_at).num_seconds() < 30
            });
        }
    }
}

/// String interning for commonly used strings to reduce allocations
pub struct StringInterner {
    strings: Arc<RwLock<HashMap<String, Arc<str>>>>,
}

impl StringInterner {
    pub fn new() -> Self {
        Self {
            strings: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get an interned string, creating it if it doesn't exist
    pub async fn intern(&self, s: &str) -> Arc<str> {
        // First try to get existing
        {
            let cache = self.strings.read().await;
            if let Some(interned) = cache.get(s) {
                return interned.clone();
            }
        }

        // Create new interned string
        let mut cache = self.strings.write().await;
        // Double-check in case another thread added it
        if let Some(interned) = cache.get(s) {
            return interned.clone();
        }

        let interned: Arc<str> = s.into();
        cache.insert(s.to_string(), interned.clone());
        interned
    }
}

/// Batch operations for database efficiency
pub struct BatchOperations;

impl BatchOperations {
    /// Batch insert payments for better performance
    pub async fn batch_insert_payments(
        pool: &sqlx::PgPool,
        payments: &[(&str, &str, &str, &str)], // (payment_id, merchant_id, amount_usd, crypto_type)
    ) -> Result<(), sqlx::Error> {
        if payments.is_empty() {
            return Ok(());
        }

        let mut query_builder = sqlx::QueryBuilder::new(
            "INSERT INTO payments (payment_id, merchant_id, amount_usd, crypto_type, status, created_at) "
        );

        query_builder.push_values(payments, |mut b, payment| {
            b.push_bind(payment.0)
             .push_bind(payment.1)
             .push_bind(payment.2)
             .push_bind(payment.3)
             .push_bind("PENDING")
             .push_bind(Utc::now());
        });

        query_builder.build().execute(pool).await?;
        Ok(())
    }

    /// Batch update payment statuses
    pub async fn batch_update_payment_status(
        pool: &sqlx::PgPool,
        updates: &[(&str, &str)], // (payment_id, status)
    ) -> Result<(), sqlx::Error> {
        if updates.is_empty() {
            return Ok(());
        }

        // Use a more efficient approach with unnest for batch updates
        let payment_ids: Vec<&str> = updates.iter().map(|(id, _)| *id).collect();
        let statuses: Vec<&str> = updates.iter().map(|(_, status)| *status).collect();

        sqlx::query!(
            r#"
            UPDATE payments 
            SET status = data.status, updated_at = NOW()
            FROM (
                SELECT unnest($1::text[]) as payment_id, 
                       unnest($2::text[]) as status
            ) as data
            WHERE payments.payment_id = data.payment_id
            "#,
            &payment_ids,
            &statuses
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}

/// Connection pool optimization
pub struct OptimizedPool;

impl OptimizedPool {
    /// Create an optimized database connection pool
    pub async fn create_optimized_pool(database_url: &str) -> Result<sqlx::PgPool, sqlx::Error> {
        sqlx::postgres::PgPoolOptions::new()
            .max_connections(20)
            .min_connections(5)
            .acquire_timeout(std::time::Duration::from_secs(10))
            .idle_timeout(std::time::Duration::from_secs(300))
            .max_lifetime(std::time::Duration::from_secs(1800))
            // Enable prepared statement caching
            .after_connect(|conn, _meta| {
                Box::pin(async move {
                    // Set connection-level optimizations
                    sqlx::query("SET statement_timeout = '30s'")
                        .execute(conn)
                        .await?;
                    sqlx::query("SET lock_timeout = '10s'")
                        .execute(conn)
                        .await?;
                    Ok(())
                })
            })
            .connect(database_url)
            .await
    }
}
