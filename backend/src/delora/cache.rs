// Delora Cache
// Redis + in-memory LRU caching layer with per-key mutex deduplication

use crate::delora::constants::{
    CACHE_PREFIX_CHAINS, CACHE_PREFIX_PRICES, CACHE_PREFIX_QUOTE, CACHE_PREFIX_TOKEN,
    CACHE_PREFIX_TOKENS, CACHE_PREFIX_TOOLS, MEMORY_CACHE_SIZE, METADATA_CACHE_TTL,
    PRICE_CACHE_TTL, QUOTE_SNAPSHOT_TTL,
};
use crate::delora::error::DeloraError;
use crate::delora::models::*;
use lru::LruCache;
use redis::AsyncCommands;
use serde::{de::DeserializeOwned, Serialize};
use std::collections::HashMap;
use std::future::Future;
use std::num::NonZeroUsize;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::debug;

pub struct DeloraCache {
    redis: redis::Client,
    memory: Arc<tokio::sync::RwLock<LruCache<String, MemoryCacheEntry>>>,
    locks: Arc<tokio::sync::RwLock<HashMap<String, Arc<Mutex<()>>>>>,
}

struct MemoryCacheEntry {
    data: Vec<u8>,
    expires_at: std::time::Instant,
}

impl DeloraCache {
    pub fn new(redis: redis::Client) -> Self {
        Self {
            redis,
            memory: Arc::new(tokio::sync::RwLock::new(LruCache::new(
                NonZeroUsize::new(MEMORY_CACHE_SIZE).expect("MEMORY_CACHE_SIZE > 0"),
            ))),
            locks: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }

    /// Get or create a per-key mutex for deduplication
    async fn get_lock(&self, key: &str) -> Arc<Mutex<()>> {
        let locks = self.locks.read().await;
        if let Some(lock) = locks.get(key) {
            return lock.clone();
        }
        drop(locks);
        let mut locks = self.locks.write().await;
        locks
            .entry(key.to_string())
            .or_insert_with(|| Arc::new(Mutex::new(())))
            .clone()
    }

    /// Clean up stale lock entries (called periodically or on key eviction)
    async fn _cleanup_locks(&self) {
        let mut locks = self.locks.write().await;
        if locks.len() > MEMORY_CACHE_SIZE * 2 {
            locks.clear();
        }
    }

    /// Get from memory → Redis → fetch and warm all layers.
    pub async fn get_or_fetch<T, F, Fut>(
        &self,
        key: &str,
        ttl_secs: u64,
        use_memory: bool,
        fetcher: F,
    ) -> Result<T, DeloraError>
    where
        T: Serialize + DeserializeOwned + Clone,
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T, DeloraError>>,
    {
        // 1. Memory cache
        if use_memory {
            if let Some(entry) = self.memory.read().await.peek(key) {
                if entry.expires_at > std::time::Instant::now() {
                    if let Ok(v) = serde_json::from_slice::<T>(&entry.data) {
                        return Ok(v);
                    }
                }
            }
        }

        // 2. Redis cache
        let mut conn = self.redis.get_multiplexed_async_connection().await?;
        let redis_val: Option<String> = conn.get(key).await.ok().flatten();
        if let Some(ref val) = redis_val {
            if let Ok(v) = serde_json::from_str::<T>(val) {
                if use_memory {
                    let serialized = serde_json::to_vec(&v).unwrap_or_default();
                    self.memory.write().await.put(
                        key.to_string(),
                        MemoryCacheEntry {
                            data: serialized,
                            expires_at: std::time::Instant::now()
                                + std::time::Duration::from_secs(std::cmp::min(ttl_secs, 300)),
                        },
                    );
                }
                return Ok(v);
            }
        }

        // 3. Dedup: acquire per-key lock, check Redis again, fetch, store
        let lock = self.get_lock(key).await;
        let _guard = lock.lock().await;

        // Double-check Redis inside the lock
        let redis_val2: Option<String> = conn.get(key).await.ok().flatten();
        if let Some(ref val) = redis_val2 {
            if let Ok(v) = serde_json::from_str::<T>(val) {
                return Ok(v);
            }
        }

        // Fetch fresh
        let result = fetcher().await?;
        let serialized = serde_json::to_vec(&result).unwrap_or_default();
        let _: Result<(), redis::RedisError> = conn.set_ex(key, &serialized, ttl_secs).await;

        if use_memory {
            self.memory.write().await.put(
                key.to_string(),
                MemoryCacheEntry {
                    data: serialized,
                    expires_at: std::time::Instant::now()
                        + std::time::Duration::from_secs(std::cmp::min(ttl_secs, 300)),
                },
            );
        }

        Ok(result)
    }

    // ── Structured Cache Methods ──────────────────────────────────────────

    pub async fn get_chains<F, Fut>(&self, fetcher: F) -> Result<Vec<ChainInfo>, DeloraError>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<Vec<ChainInfo>, DeloraError>>,
    {
        self.get_or_fetch(CACHE_PREFIX_CHAINS, METADATA_CACHE_TTL, true, fetcher)
            .await
    }

    pub async fn get_tokens<F, Fut>(&self, fetcher: F) -> Result<TokenListResponse, DeloraError>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<TokenListResponse, DeloraError>>,
    {
        self.get_or_fetch(CACHE_PREFIX_TOKENS, METADATA_CACHE_TTL, true, fetcher)
            .await
    }

    pub async fn get_tools<F, Fut>(&self, fetcher: F) -> Result<Vec<ToolInfo>, DeloraError>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<Vec<ToolInfo>, DeloraError>>,
    {
        self.get_or_fetch(CACHE_PREFIX_TOOLS, METADATA_CACHE_TTL, true, fetcher)
            .await
    }

    pub async fn get_prices<F, Fut>(&self, fetcher: F) -> Result<PriceResponse, DeloraError>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<PriceResponse, DeloraError>>,
    {
        self.get_or_fetch(CACHE_PREFIX_PRICES, PRICE_CACHE_TTL, false, fetcher)
            .await
    }

    pub async fn get_token_by_address<F, Fut>(
        &self,
        chain_id: u64,
        address: &str,
        fetcher: F,
    ) -> Result<TokenItem, DeloraError>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<TokenItem, DeloraError>>,
    {
        let key = format!(
            "{}:{}:{}",
            CACHE_PREFIX_TOKEN,
            chain_id,
            address.to_lowercase()
        );
        self.get_or_fetch(&key, METADATA_CACHE_TTL, false, fetcher)
            .await
    }

    /// Store a quote snapshot for later retrieval
    pub async fn store_quote_snapshot(
        &self,
        quote_id: &uuid::Uuid,
        quote: &QuoteResponse,
    ) -> Result<(), DeloraError> {
        let key = format!("{}:{}", CACHE_PREFIX_QUOTE, quote_id);
        let serialized = serde_json::to_string(quote)?;
        let mut conn = self.redis.get_multiplexed_async_connection().await?;
        let _: Result<(), redis::RedisError> =
            conn.set_ex(&key, serialized, QUOTE_SNAPSHOT_TTL).await;
        debug!("Stored quote snapshot: {}", quote_id);
        Ok(())
    }

    /// Retrieve a stored quote snapshot
    pub async fn get_quote_snapshot(
        &self,
        quote_id: &uuid::Uuid,
    ) -> Result<Option<QuoteResponse>, DeloraError> {
        let key = format!("{}:{}", CACHE_PREFIX_QUOTE, quote_id);
        let mut conn = self.redis.get_multiplexed_async_connection().await?;
        let val: Option<String> = conn.get(&key).await.ok().flatten();
        match val {
            Some(s) => Ok(Some(serde_json::from_str(&s)?)),
            None => Ok(None),
        }
    }

    /// Invalidate a cached key
    pub async fn invalidate(&self, key: &str) -> Result<(), DeloraError> {
        let mut conn = self.redis.get_multiplexed_async_connection().await?;
        let _: Result<(), redis::RedisError> = conn.del(key).await;
        self.memory.write().await.pop(key);
        self.locks.write().await.remove(key);
        debug!("Invalidated cache key: {}", key);
        Ok(())
    }
}
