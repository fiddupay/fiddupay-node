# Delora Cross-Chain Integration — Engineering Plan

## Overview

Integrate Delora's cross-chain swap infrastructure into FidduPay so customers can pay with **any token on any EVM/SVM chain** and merchants receive their configured asset atomically — all while FidduPay earns a 0.5% integrator fee on every cross-chain transaction.

---

## Architecture Diagram

```
┌────────────────────────────────────────────────────────────────────────┐
│                          FRONTEND (React)                              │
│  ┌──────────────┐    ┌──────────────────┐    ┌──────────────────────┐ │
│  │ Payment Page │───▶│ CrossChainPicker │───▶│ DeloraQuoteDisplay   │ │
│  │ (existing)   │    │ (new component)  │    │ (new component)      │ │
│  └──────────────┘    └──────────────────┘    └──────────────────────┘ │
└──────────────────────────────────────┬─────────────────────────────────┘
                                       │ REST API
┌──────────────────────────────────────▼─────────────────────────────────┐
│                       BACKEND (Rust/Axum)                               │
│                                                                         │
│  ┌──────────────────────────────────────────────────────────────┐     │
│  │                    API Endpoints                               │     │
│  │  GET  /api/v1/payments/cross-chain-quote   (public)           │     │
│  │  POST /api/v1/payments/cross-chain-register (public)          │     │
│  │  GET  /api/v1/payments/cross-chain-status/:link_id (public)   │     │
│  └──────────────────────┬───────────────────────────────────────┘     │
│                         │                                               │
│  ┌──────────────────────▼───────────────────────────────────────┐     │
│  │                    DeloraService                                │     │
│  │  ┌──────────────┐  ┌──────────────┐  ┌────────────────────┐  │     │
│  │  │ DeloraClient  │  │ CacheService │  │ BridgeTaskManager  │  │     │
│  │  │ (HTTP pool)   │  │ (Redis)      │  │ (background poll)  │  │     │
│  │  └──────┬───────┘  └──────┬───────┘  └────────┬───────────┘  │     │
│  └─────────┼─────────────────┼───────────────────┼──────────────┘     │
│            │                 │                   │                      │
│  ┌─────────▼─────────────────▼───────────────────▼──────────────┐     │
│  │              DeloraClient (shared reqwest pool)                │     │
│  │  CircuitBreaker │ RateLimiter │ RetryWithBackoff │ Timeout    │     │
│  └──────────────────────────────┬───────────────────────────────┘     │
│                                 │                                       │
│  ┌──────────────────────────────▼───────────────────────────────┐     │
│  │              Database (PostgreSQL)                             │     │
│  │  cross_chain_payments │ cross_chain_registrations              │     │
│  │  extends payment_transactions with cross-chain metadata        │     │
│  └──────────────────────────────────────────────────────────────┘     │
└─────────────────────────────────────────────────────────────────────────┘
                                 │ HTTPS
┌────────────────────────────────▼─────────────────────────────────────────┐
│                        Delora API (api.delora.build)                      │
│  /v1/quotes  │ /v1/advanced/routes  │ /v1/advanced/stepTransaction       │
│  /v1/chains  │ /v1/tokens            │ /v1/tools  │ /v1/prices           │
└──────────────────────────────────────────────────────────────────────────┘
```

---

## Module Structure

All new files live under `backend/src/delora/`:

```
backend/src/delora/
├── mod.rs                          # Module re-exports
├── client.rs                       # DeloraClient — shared reqwest pool, retry, circuit breaker
├── models.rs                       # All request/response Rust types
├── cache.rs                        # DeloraCache — Redis caching layer
├── service.rs                      # DeloraService — business logic orchestrator
├── handlers.rs                     # Axum handler functions
├── routes.rs                       # Route definitions
├── bridge_monitor.rs               # Background task for bridge tx completion tracking
├── validation.rs                   # Calldata sanitization, address verification
├── error.rs                        # Delora-specific error types
└── constants.rs                    # Chain IDs, token addresses, adapter names
```

### Files Modified

| File | Change |
|------|--------|
| `backend/src/api/routes.rs` | Add cross-chain route entries |
| `backend/src/api/state.rs` | Add `DeloraService` to `AppState` |
| `backend/src/config.rs` | Add Delora config fields |
| `backend/src/main.rs` | Start bridge monitor background task |
| `backend/migrations/` | New migration for cross-chain tables |
| `backend/Cargo.toml` | No new deps needed (reqwest, redis, serde already available) |
| `frontend/src/` | New CrossChainPicker + DeloraQuoteDisplay components |

---

## 1. Data Models (`backend/src/delora/models.rs`)

### 1.1 Delora API Types

```rust
// --- Request Types ---

#[derive(Debug, Serialize)]
pub struct QuoteRequest {
    pub sender_address: String,
    pub receiver_address: String,
    pub origin_chain_id: u64,
    pub destination_chain_id: u64,
    pub amount: String,           // base units, positive integer string
    pub origin_currency: String,  // token address
    pub destination_currency: String,
    pub integrator: String,       // "fiddupay"
    pub fee: f64,                 // 0.005 = 0.5%
    pub slippage: Option<f64>,
    pub include_bridges: Option<String>,
    pub exclude_bridges: Option<String>,
    pub include_exchanges: Option<String>,
    pub exclude_exchanges: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AdvancedRoutesRequest {
    pub sender_address: String,
    pub receiver_address: String,
    pub origin_chain_id: u64,
    pub destination_chain_id: u64,
    pub amount: String,
    pub origin_currency: String,
    pub destination_currency: String,
    pub integrator: String,
    pub fee: f64,
    pub slippage: Option<f64>,
    pub include_bridges: Option<String>,
    pub exclude_bridges: Option<String>,
    pub include_exchanges: Option<String>,
    pub exclude_exchanges: Option<String>,
    pub max_routes: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct StepTransactionRequest {
    pub step: RouteStep,
    pub context: StepTransactionContext,
}

#[derive(Debug, Serialize)]
pub struct StepTransactionContext {
    pub sender_address: String,
    pub receiver_address: String,
}

// --- Response Types ---

#[derive(Debug, Deserialize)]
pub struct QuoteResponse {
    pub input_amount: String,
    pub output_amount: String,
    pub min_output_amount: Option<String>,
    pub adapter: String,
    pub calldata: Calldata,
    pub fees: FeeInfo,
    pub gas: Option<GasInfo>,
    pub warnings: Vec<DeloraWarning>,
    pub approval_address: Option<String>,
    pub estimated_time_sec: Option<u64>,
    pub bridge_scan: Option<serde_json::Value>,
    pub usd: Option<UsdPrices>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Calldata {
    pub to: String,
    pub value: String,
    pub data: String,
}

#[derive(Debug, Deserialize)]
pub struct FeeInfo {
    pub total: FeeItem,
    pub breakdown: Vec<FeeItem>,
    pub total_usd: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct FeeItem {
    pub amount: String,
    pub currency_symbol: String,
    pub currency_address: Option<String>,
    pub chain_id: u64,
    pub decimals: Option<u32>,
    #[serde(rename = "type")]
    pub fee_type: Option<String>,
    pub amount_usd: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GasInfo {
    pub gas_price: Option<String>,
    pub gas_limit: Option<String>,
    pub max_fee_per_gas: Option<String>,
    pub max_priority_fee_per_gas: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DeloraWarning {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct UsdPrices {
    pub origin_amount_usd: Option<String>,
    pub destination_amount_usd: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AdvancedRoutesResponse {
    pub routes: Vec<AdvancedRoute>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedRoute {
    pub id: String,
    pub input_amount: String,
    pub output_amount: Option<String>,
    pub min_output_amount: Option<String>,
    pub fees: Option<FeeInfo>,
    pub adapter: String,
    pub is_multistep: bool,
    pub steps: Vec<RouteStep>,
    pub estimated_time_sec: Option<u64>,
    pub warnings: Option<Vec<DeloraWarning>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteStep {
    pub id: String,
    pub route_id: String,
    #[serde(rename = "type")]
    pub step_type: String,  // "swap" or "bridge"
    pub tool: String,
    pub action: StepAction,
    pub estimate: StepEstimate,
    pub execution: Option<serde_json::Value>,
    pub transaction_request: Option<TransactionRequest>,
    pub integrator: Option<String>,
    pub fee: Option<f64>,
    pub warnings: Option<Vec<DeloraWarning>>,
    pub bridge_scan: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepAction {
    pub from_chain_id: u64,
    pub to_chain_id: u64,
    pub from_token: TokenInfo,
    pub to_token: TokenInfo,
    pub from_amount: String,
    pub slippage: Option<f64>,
    pub from_address: Option<String>,
    pub to_address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenInfo {
    pub address: String,
    pub symbol: String,
    pub decimals: u32,
    pub chain_id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepEstimate {
    pub from_amount: String,
    pub to_amount: String,
    pub to_amount_min: Option<String>,
    pub fees: Option<FeeInfo>,
    pub approval_address: Option<String>,
    pub estimated_time_sec: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionRequest {
    pub to: String,
    pub value: String,
    pub data: String,
    pub gas: Option<GasInfo>,
}

#[derive(Debug, Deserialize)]
pub struct ChainInfo {
    pub chain_id: u64,
    pub name: String,
    pub native_currency: Option<NativeCurrencyInfo>,
    pub rpc_urls: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct NativeCurrencyInfo {
    pub name: String,
    pub symbol: String,
    pub decimals: u32,
}

#[derive(Debug, Deserialize)]
pub struct TokenListResponse {
    pub tokens: HashMap<String, Vec<TokenItem>>,  // chain_id → tokens
}

#[derive(Debug, Deserialize)]
pub struct TokenItem {
    pub address: String,
    pub symbol: String,
    pub name: String,
    pub decimals: u32,
    pub chain_id: u64,
}

#[derive(Debug, Deserialize)]
pub struct ToolInfo {
    pub key: String,
    pub name: String,
    pub capabilities: Vec<String>,  // "swap", "bridge", multi
}

#[derive(Debug, Deserialize)]
pub struct PriceResponse {
    pub prices: HashMap<String, f64>,  // token_symbol → usd_price
    pub updated_at: String,
}
```

### 1.2 Internal Application Types

```rust
/// What our frontend receives (sanitized, merchant-address-redacted)
#[derive(Debug, Serialize)]
pub struct CrossChainQuoteResponse {
    pub quote_id: String,                  // UUID v4, used for registration correlation
    pub origin_chain_id: u64,
    pub destination_chain_id: u64,
    pub origin_currency: CurrencySummary,
    pub destination_currency: CurrencySummary,
    pub input_amount: String,               // customer pays this
    pub input_amount_display: String,       // human-readable
    pub output_amount: String,              // merchant receives this
    pub output_amount_display: String,      // human-readable
    pub min_output_amount: String,
    pub fees: QuoteFeeBreakdown,
    pub estimated_time_sec: Option<u64>,
    pub calldata: Calldata,                 // to, value, data for wallet to sign
    pub gas: Option<GasInfo>,
    pub approval_address: Option<String>,   // if ERC-20, spender to approve
    pub warnings: Vec<DeloraWarning>,
    pub expires_at: String,                 // ISO8601, quote valid until
    pub route: Option<AdvancedRouteSummary>, // if advanced route used
}

#[derive(Debug, Serialize)]
pub struct AdvancedRouteSummary {
    pub route_id: String,
    pub adapter: String,
    pub is_multistep: bool,
    pub steps_count: u32,
}

#[derive(Debug, Serialize)]
pub struct CurrencySummary {
    pub symbol: String,
    pub name: String,
    pub address: String,
    pub chain_id: u64,
    pub chain_name: String,
    pub decimals: u32,
}

#[derive(Debug, Serialize)]
pub struct QuoteFeeBreakdown {
    pub delora_fee: String,      // Delora's routing fee
    pub delora_fee_usd: String,
    pub integrator_fee: String,  // FidduPay's cut
    pub integrator_fee_usd: String,
    pub gas_fee_estimate: String,
    pub gas_fee_estimate_usd: String,
    pub total_fee: String,
    pub total_fee_usd: String,
}

/// Internal tracking state
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CrossChainPaymentStatus {
    QuoteRequested,    // quote fetched, shown to user
    TxSubmitted,       // user submitted on-chain tx, hash registered with us
    TxConfirmed,       // on-chain tx confirmed on origin chain
    BridgePending,     // bridge in progress (origin confirmed, dest not yet)
    BridgeComplete,    // funds arrived on destination, awaiting confirmations
    Completed,         // funds confirmed on destination, merchant credited
    Failed,            // tx failed on origin, or bridge reverted
    Expired,           // quote expired, user didn't act
}

#[derive(Debug, Serialize)]
pub struct CrossChainStatusResponse {
    pub payment_id: String,
    pub status: CrossChainPaymentStatus,
    pub origin_tx_hash: Option<String>,
    pub destination_tx_hash: Option<String>,
    pub bridge_scan_url: Option<String>,
    pub confirmations: Option<i32>,
    pub estimated_completion_sec: Option<u64>,
}
```

### 1.3 Database Migration

```sql
-- Migration: 20260711000000_create_cross_chain_payments.sql

CREATE TABLE cross_chain_payments (
    id BIGSERIAL PRIMARY KEY,
    quote_id UUID NOT NULL UNIQUE,
    payment_transaction_id BIGINT REFERENCES payment_transactions(id) ON DELETE SET NULL,
    merchant_id BIGINT NOT NULL REFERENCES merchants(id),
    invoice_id UUID,

    -- Quote snapshot (immutable record of what was shown to customer)
    origin_chain_id BIGINT NOT NULL,
    origin_currency_address VARCHAR(255) NOT NULL,
    origin_currency_symbol VARCHAR(50) NOT NULL,
    origin_currency_decimals INT NOT NULL,
    destination_chain_id BIGINT NOT NULL,
    destination_currency_address VARCHAR(255) NOT NULL,
    destination_currency_symbol VARCHAR(50) NOT NULL,
    destination_currency_decimals INT NOT NULL,
    input_amount VARCHAR(100) NOT NULL,       -- string, base units
    output_amount VARCHAR(100) NOT NULL,      -- string, base units
    min_output_amount VARCHAR(100) NOT NULL,

    -- Fee tracking
    delora_fee_amount VARCHAR(100),
    delora_fee_usd VARCHAR(100),
    integrator_fee_amount VARCHAR(100),
    integrator_fee_usd VARCHAR(100),
    integrator_fee_rate DECIMAL(5,4),         -- e.g., 0.0050

    -- Route info
    adapter VARCHAR(100) NOT NULL,
    route_id VARCHAR(255),
    route_snapshot JSONB,                      -- full route from Delora for debugging
    is_multistep BOOLEAN NOT NULL DEFAULT false,
    is_advanced BOOLEAN NOT NULL DEFAULT false,

    -- Execution tracking
    status VARCHAR(50) NOT NULL DEFAULT 'quote_requested',
    sender_address VARCHAR(255),
    merchant_destination_address VARCHAR(255) NOT NULL,
    calldata JSONB NOT NULL,
    calldata_to VARCHAR(255) NOT NULL,         -- contract address from calldata (indexed for validation)
    approval_address VARCHAR(255),             -- if ERC-20 approval needed

    -- On-chain tracking
    origin_tx_hash VARCHAR(255),
    destination_tx_hash VARCHAR(255),
    origin_block_number BIGINT,
    destination_block_number BIGINT,
    origin_confirmations INT DEFAULT 0,
    bridge_scan_metadata JSONB,                -- bridge tracking URL/ID from Delora

    -- Timing
    quote_expires_at TIMESTAMPTZ NOT NULL,
    tx_submitted_at TIMESTAMPTZ,
    origin_confirmed_at TIMESTAMPTZ,
    bridge_completed_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    failed_reason TEXT,

    -- Delora response metadata
    delora_warnings JSONB DEFAULT '[]',
    estimated_time_sec BIGINT,
    gas_info JSONB,

    -- Waste/prevention
    sandbox_mode BOOLEAN NOT NULL DEFAULT false,
    idempotency_key VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- Indexes for lookups
CREATE INDEX idx_ccp_quote_id ON cross_chain_payments(quote_id);
CREATE INDEX idx_ccp_merchant_id ON cross_chain_payments(merchant_id);
CREATE INDEX idx_ccp_payment_transaction_id ON cross_chain_payments(payment_transaction_id);
CREATE INDEX idx_ccp_status ON cross_chain_payments(status);
CREATE INDEX idx_ccp_origin_tx_hash ON cross_chain_payments(origin_tx_hash) WHERE origin_tx_hash IS NOT NULL;
CREATE INDEX idx_ccp_bridge_pending ON cross_chain_payments(status) 
    WHERE status IN ('tx_confirmed', 'bridge_pending');
CREATE INDEX idx_ccp_quote_expires_at ON cross_chain_payments(quote_expires_at) 
    WHERE status = 'quote_requested';

-- Prevent double-registration of same origin tx for different cross-chain payments
CREATE UNIQUE INDEX idx_ccp_origin_tx_unique 
    ON cross_chain_payments(origin_tx_hash) 
    WHERE origin_tx_hash IS NOT NULL AND deleted_at IS NULL;

-- Trigger for updated_at
CREATE TRIGGER update_cross_chain_payments_updated_at
    BEFORE UPDATE ON cross_chain_payments
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
```

---

## 2. DeloraClient (`backend/src/delora/client.rs`)

A shared, pooled HTTP client with resilience built in. **One instance per AppState**, not per-request.

```rust
pub struct DeloraClient {
    http: reqwest::Client,
    base_url: String,                    // "https://api.delora.build"
    api_key: Option<String>,
    rate_limiter: Arc<governor::RateLimiter<
        governor::state::InMemoryState,
        governor::clock::DefaultClock,
        governor::middleware::NoOpMiddleware,
    >>,
    circuit_breaker: Arc<RwLock<CircuitBreaker>>,
    metrics: DeloraClientMetrics,
}

pub struct DeloraClientMetrics {
    total_requests: AtomicU64,
    successful_requests: AtomicU64,
    failed_requests: AtomicU64,
    rate_limited_requests: AtomicU64,
    circuit_breaks: AtomicU64,
    average_latency_ms: AtomicU64,       // approximate, for health checks
}
```

### Construction

```rust
impl DeloraClient {
    pub fn new(api_key: Option<String>, config: &DeloraConfig) -> Self {
        let http = reqwest::Client::builder()
            .pool_max_idle_per_host(config.pool_max_idle_per_host)    // default: 10
            .pool_idle_timeout(Duration::from_secs(config.pool_idle_timeout_secs)) // default: 90
            .timeout(Duration::from_secs(config.request_timeout_secs)) // default: 30
            .connect_timeout(Duration::from_secs(config.connect_timeout_secs)) // default: 10
            .tcp_keepalive(Duration::from_secs(config.tcp_keepalive_secs)) // default: 60
            .http2_prior_knowledge()
            .user_agent(format!("FidduPay/{} (Delora Integration)", env!("CARGO_PKG_VERSION")))
            .build()
            .expect("Failed to create Delora HTTP client");
        
        let rate_limiter = Arc::new(governor::RateLimiter::direct(
            governor::Quota::per_minute(config.rate_limit_per_minute) // default: 180
        ));
        
        let circuit_breaker = Arc::new(RwLock::new(
            CircuitBreaker::new(config.circuit_breaker_threshold, config.circuit_breaker_timeout_secs)
        ));
        
        Self { http, base_url: config.base_url.clone(), api_key, rate_limiter, circuit_breaker, metrics: Default::default() }
    }
}
```

### Core Request Method

```rust
impl DeloraClient {
    /// Generic request with retry, circuit breaker, rate limiting
    async fn request<T: DeserializeOwned>(
        &self,
        method: Method,
        path: &str,
        query: Option<&[(String, String)]>,
        body: Option<&impl Serialize>,
    ) -> Result<T, DeloraError> {
        // 1. Check circuit breaker
        {
            let cb = self.circuit_breaker.read().await;
            if cb.is_open() {
                self.metrics.circuit_breaks.fetch_add(1, Ordering::Relaxed);
                return Err(DeloraError::CircuitBreakerOpen);
            }
        }
        
        // 2. Rate limit
        self.rate_limiter.until_ready().await;
        
        // 3. Build request
        let url = format!("{}{}", self.base_url, path);
        let mut req = self.http.request(method.clone(), &url);
        
        if let Some(key) = &self.api_key {
            req = req.header("x-api-key", key);
        }
        
        if let Some(query) = query {
            req = req.query(&query.iter().map(|(k,v)| (k.as_str(), v.as_str())).collect::<Vec<_>>());
        }
        
        if let Some(body) = body {
            req = req.json(body);
        }
        
        // 4. Execute with retry
        let start = Instant::now();
        let result = retry_with_delora_backoff(
            || async { self.execute_request(req.try_clone().unwrap()).await },
            config.max_retries, // default: 3
        ).await;
        let elapsed = start.elapsed();
        self.metrics.total_requests.fetch_add(1, Ordering::Relaxed);
        self.update_latency(elapsed);
        
        // 5. Handle result
        match result {
            Ok(response) => {
                self.metrics.successful_requests.fetch_add(1, Ordering::Relaxed);
                self.circuit_breaker.write().await.record_success();
                Ok(response)
            }
            Err(e) => {
                self.metrics.failed_requests.fetch_add(1, Ordering::Relaxed);
                self.circuit_breaker.write().await.record_failure();
                
                if matches!(e, DeloraError::RateLimited { .. }) {
                    self.metrics.rate_limited_requests.fetch_add(1, Ordering::Relaxed);
                }
                Err(e)
            }
        }
    }
    
    async fn execute_request<T: DeserializeOwned>(
        &self,
        req: reqwest::Request,
    ) -> Result<T, DeloraError> {
        let response = self.http.execute(req).await?;
        let status = response.status();
        
        // Rate limit headers
        let rate_limit_remaining = response.headers()
            .get("x-ratelimit-remaining")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok());
        
        if status == StatusCode::TOO_MANY_REQUESTS {
            // Parse Retry-After if present, else use default backoff
            return Err(DeloraError::RateLimited {
                retry_after_secs: extract_retry_after(&response).unwrap_or(60),
            });
        }
        
        if !status.is_success() {
            let error_body: DeloraApiError = response.json().await?;
            return Err(DeloraError::ApiError {
                status: status.as_u16(),
                code: error_body.code,
                message: error_body.message,
            });
        }
        
        Ok(response.json().await?)
    }
}
```

### Typed API Methods

```rust
impl DeloraClient {
    pub async fn get_quote(&self, params: &QuoteRequest) -> Result<QuoteResponse, DeloraError> { ... }
    
    pub async fn get_advanced_routes(&self, params: &AdvancedRoutesRequest) -> Result<AdvancedRoutesResponse, DeloraError> { ... }
    
    pub async fn populate_step_transaction(&self, step: &RouteStep, context: &StepTransactionContext) -> Result<RouteStep, DeloraError> { ... }
    
    pub async fn get_chains(&self) -> Result<Vec<ChainInfo>, DeloraError> { ... }
    
    pub async fn get_tokens(&self) -> Result<TokenListResponse, DeloraError> { ... }
    
    pub async fn get_tools(&self) -> Result<Vec<ToolInfo>, DeloraError> { ... }
    
    pub async fn get_prices(&self) -> Result<PriceResponse, DeloraError> { ... }
    
    pub async fn get_token(&self, chain_id: u64, address_or_symbol: &str) -> Result<TokenItem, DeloraError> { ... }
}
```

### Delora-Specific Retry

```rust
/// Retry with exponential backoff that respects Delora-specific error codes
async fn retry_with_delora_backoff<F, Fut, T>(
    operation: F,
    max_retries: u32,
) -> Result<T, DeloraError>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, DeloraError>>,
{
    let mut attempt = 0;
    loop {
        match operation().await {
            Ok(v) => return Ok(v),
            Err(e) => {
                attempt += 1;
                
                let is_retryable = matches!(&e,
                    DeloraError::RateLimited { .. } |
                    DeloraError::HttpError { .. } |     // network timeouts, DNS
                    DeloraError::ApiError { status: 500..=599, .. }  // server errors
                );
                
                if !is_retryable || attempt > max_retries {
                    return Err(e);
                }
                
                // Calculate backoff
                let base_delay = match &e {
                    DeloraError::RateLimited { retry_after_secs } => 
                        Duration::from_secs(*retry_after_secs),
                    _ => Duration::from_secs(2u64.pow(attempt - 1)),
                };
                
                // Add jitter (±25%)
                let jitter = (base_delay.as_millis() as f64 * 0.25) as u64;
                let jittered = base_delay + Duration::from_millis(jitter);
                
                tracing::warn!(
                    attempt = attempt,
                    error = %e,
                    retry_in_ms = jittered.as_millis(),
                    "Delora API call failed, retrying"
                );
                
                tokio::time::sleep(jittered).await;
            }
        }
    }
}
```

---

## 3. DeloraCache (`backend/src/delora/cache.rs`)

Multi-layer caching with Redis as the shared layer and an in-memory read-through cache for hot keys.

```rust
pub struct DeloraCache {
    redis: redis::Client,
    memory: Arc<RwLock<LruCache<String, CacheEntry>>>,
}

struct CacheEntry {
    data: Vec<u8>,          // serialized JSON or msgpack
    expires_at: Instant,
}

// Cache key patterns (all namespaced with "delora:"):
//   delora:chains                              → Vec<ChainInfo>         [TTL: 3600s]
//   delora:tokens:{chain_id}                   → Vec<TokenItem>         [TTL: 3600s]  
//   delora:tools                               → Vec<ToolInfo>          [TTL: 3600s]
//   delora:prices                              → PriceResponse          [TTL: 60s]
//   delora:quote:{sha256(sender+origin+dst+amount+currency)}           [TTL: 15s]
//   delora:token:{chain_id}:{address}           → TokenItem             [TTL: 3600s]
//   delora:quote:{quote_id}                     → QuoteResponse         [TTL: 30s]
```

### Design Decisions

- **Metadata (chains, tokens, tools):** 1-hour TTL. These change rarely. Cache miss triggers a single fetch that warms the cache.
- **Prices:** 60-second TTL. Delora's `/v1/prices` endpoint returns cached snapshots anyway.
- **Quotes:** 15-second TTL in Redis (quotes expire ~30s), 30-second TTL for stored quote snapshots by quote_id. No memory-level caching for quotes — they're short-lived and user-specific.
- **In-memory LRU:** Limited to 512 entries for hot metadata keys. Only stores chains/tokens/tools metadata since those are read frequently and change rarely.

### Interface

```rust
impl DeloraCache {
    // Generic get-or-fetch pattern
    async fn get_or_fetch<T: Serialize + DeserializeOwned>(
        &self,
        key: &str,
        ttl_secs: u64,
        fetcher: impl Future<Output = Result<T, DeloraError>>,
    ) -> Result<T, DeloraError>;
    
    // Shared-future deduplication for concurrent callers
    // Uses existing pattern from PriceService: Arc<RwLock<HashMap<String, SharedFuture>>>
    // Ensures 100 concurrent requests for "chains" only fire 1 HTTP call
    async fn get_or_fetch_deduped<T>(
        &self,
        key: &str,
        ttl_secs: u64,
        fetcher: impl FnOnce() -> Fut,
    ) -> Result<T, DeloraError>;
    
    // Invalidate a specific cache key
    async fn invalidate(&self, key: &str);
    
    // Store a quote snapshot by quote_id (used during quote→register correlation)
    async fn store_quote_snapshot(&self, quote_id: &Uuid, quote: &QuoteResponse) -> Result<()>;
    async fn get_quote_snapshot(&self, quote_id: &Uuid) -> Result<Option<QuoteResponse>>;
    
    // Warm-up: called at startup to preload chains/tokens/tools
    async fn warmup(&self, client: &DeloraClient) -> Result<()>;
}
```

---

## 4. DeloraService (`backend/src/delora/service.rs`)

Business logic orchestrator. This is called by the handler layer.

```rust
pub struct DeloraService {
    client: Arc<DeloraClient>,
    cache: Arc<DeloraCache>,
    db: PgPool,
    config: DeloraConfig,
}

impl DeloraService {
    /// Get a cross-chain quote for a customer payment
    /// This is the core function that wraps the Delora quote in FidduPay's context
    pub async fn get_cross_chain_quote(
        &self,
        link_id: &str,                    // payment link ID
        sender_address: &str,             // customer wallet
        origin_chain_id: u64,
        origin_currency_address: &str,    // what the customer has
    ) -> Result<CrossChainQuoteResponse, DeloraError> {
        // 1. Lookup payment link → get invoice/payment details
        let payment = self.lookup_payment_by_link(link_id).await?;
        
        // 2. Get merchant destination details
        let merchant_addr = self.get_merchant_destination(&payment).await?;
        let dest_chain = payment.destination_chain_id;  // from payment_transactions
        let dest_currency = payment.destination_currency_address;
        let invoice_amount = payment.amount;
        
        // 3. If same-chain + same-currency → skip Delora, return direct payment info
        if origin_chain_id == dest_chain && origin_currency_address == dest_currency {
            return self.build_direct_quote(payment, merchant_addr).await;
        }
        
        // 4. Check if this origin chain/currency combo is supported
        self.validate_supported_tokens(origin_chain_id, origin_currency_address).await?;
        
        // 5. Resolve token metadata for display
        let (origin_meta, dest_meta) = tokio::try_join!(
            self.resolve_token_metadata(origin_chain_id, origin_currency_address),
            self.resolve_token_metadata(dest_chain, &dest_currency),
        )?;
        
        // 6. Build quote request
        let quote_req = QuoteRequest {
            sender_address: sender_address.to_string(),
            receiver_address: merchant_addr.clone(),
            origin_chain_id,
            destination_chain_id: dest_chain,
            amount: invoice_amount.to_string(), // base units
            origin_currency: origin_currency_address.to_string(),
            destination_currency: dest_currency.clone(),
            integrator: self.config.integrator_id.clone(),
            fee: self.config.default_integrator_fee,
            slippage: Some(self.config.default_slippage),
            include_bridges: None,
            exclude_bridges: None,
            include_exchanges: None,
            exclude_exchanges: None,
        };
        
        // 7. Try simple quote first, fall back to advanced if needed
        let (quote, route_summary) = match self.client.get_quote(&quote_req).await {
            Ok(q) => (q, None),
            Err(_) => {
                // Fall back to advanced routes for cross-chain
                let advanced = self.get_advanced_quote(&quote_req).await?;
                (advanced.0, Some(advanced.1))
            }
        };
        
        // 8. Generate quote_id for later correlation
        let quote_id = Uuid::new_v4();
        
        // 9. Store quote snapshot in cache + DB
        self.cache.store_quote_snapshot(&quote_id, &quote).await?;
        self.persist_cross_chain_payment(&quote_id, &payment, &quote, &quote_req, route_summary.as_ref()).await?;
        
        // 10. Build sanitized response for frontend
        Ok(CrossChainQuoteResponse {
            quote_id: quote_id.to_string(),
            origin_chain_id,
            destination_chain_id: dest_chain,
            origin_currency: origin_meta,
            destination_currency: dest_meta,
            input_amount: quote.input_amount.clone(),
            input_amount_display: self.format_amount(&quote.input_amount, origin_meta.decimals),
            output_amount: quote.output_amount.clone(),
            output_amount_display: self.format_amount(&quote.output_amount, dest_meta.decimals),
            min_output_amount: quote.min_output_amount.clone().unwrap_or_default(),
            fees: self.build_fee_breakdown(&quote, &self.config),
            estimated_time_sec: quote.estimated_time_sec,
            calldata: quote.calldata.clone(),
            gas: quote.gas.clone(),
            approval_address: quote.approval_address.clone(),
            warnings: quote.warnings.clone(),
            expires_at: (Utc::now() + chrono::Duration::seconds(30)).to_rfc3339(),
            route: route_summary,
        })
    }
    
    /// Fall back to advanced routes for multi-step cross-chain swaps
    async fn get_advanced_quote(
        &self,
        simple_req: &QuoteRequest,
    ) -> Result<(QuoteResponse, AdvancedRouteSummary), DeloraError> {
        let adv_req = AdvancedRoutesRequest {
            sender_address: simple_req.sender_address.clone(),
            receiver_address: simple_req.receiver_address.clone(),
            origin_chain_id: simple_req.origin_chain_id,
            destination_chain_id: simple_req.destination_chain_id,
            amount: simple_req.amount.clone(),
            origin_currency: simple_req.origin_currency.clone(),
            destination_currency: simple_req.destination_currency.clone(),
            integrator: simple_req.integrator.clone(),
            fee: simple_req.fee,
            slippage: simple_req.slippage,
            include_bridges: simple_req.include_bridges.clone(),
            exclude_bridges: simple_req.exclude_bridges.clone(),
            include_exchanges: simple_req.include_exchanges.clone(),
            exclude_exchanges: simple_req.exclude_exchanges.clone(),
            max_routes: Some(3),
        };
        
        let routes = self.client.get_advanced_routes(&adv_req).await?;
        let best_route = routes.routes.into_iter().next()
            .ok_or(DeloraError::NoRoutesFound)?;
        
        // Populate first step for execution data
        let first_step = best_route.steps.first()
            .ok_or(DeloraError::NoStepsInRoute)?;
        
        let populated = self.client.populate_step_transaction(
            first_step,
            &StepTransactionContext {
                sender_address: simple_req.sender_address.clone(),
                receiver_address: simple_req.receiver_address.clone(),
            }
        ).await?;
        
        // Convert populated step into a standard QuoteResponse shape
        let quote = QuoteResponse {
            input_amount: best_route.input_amount,
            output_amount: best_route.output_amount.unwrap_or_default(),
            min_output_amount: best_route.min_output_amount,
            adapter: best_route.adapter,
            calldata: Calldata {
                to: populated.transaction_request.as_ref().map(|t| t.to.clone()).unwrap_or_default(),
                value: populated.transaction_request.as_ref().map(|t| t.value.clone()).unwrap_or_default(),
                data: populated.transaction_request.as_ref().map(|t| t.data.clone()).unwrap_or_default(),
            },
            fees: best_route.fees.unwrap_or_default(),
            gas: populated.transaction_request.and_then(|t| t.gas),
            warnings: best_route.warnings.unwrap_or_default(),
            approval_address: populated.estimate.approval_address,
            estimated_time_sec: best_route.estimated_time_sec,
            bridge_scan: populated.bridge_scan,
            usd: None,
        };
        
        let summary = AdvancedRouteSummary {
            route_id: best_route.id,
            adapter: best_route.adapter,
            is_multistep: best_route.is_multistep,
            steps_count: best_route.steps.len() as u32,
        };
        
        Ok((quote, summary))
    }
    
    /// Register a transaction hash after the customer signs and submits
    pub async fn register_cross_chain_tx(
        &self,
        quote_id: &Uuid,
        tx_hash: &str,
        sender_address: &str,
    ) -> Result<CrossChainStatusResponse, DeloraError> {
        // 1. Lookup cross_chain_payment by quote_id with FOR UPDATE
        let mut tx = self.db.begin().await?;
        
        let cc_payment = sqlx::query_as::<_, CrossChainPayment>(
            "SELECT * FROM cross_chain_payments WHERE quote_id = $1 AND deleted_at IS NULL FOR UPDATE"
        )
        .bind(quote_id)
        .fetch_optional(&mut *tx)
        .await?;
        
        let cc_payment = cc_payment.ok_or(DeloraError::QuoteNotFound)?;
        
        // 2. Idempotency check — already registered?
        if cc_payment.status == CrossChainPaymentStatus::TxSubmitted || 
           cc_payment.status == CrossChainPaymentStatus::TxConfirmed ||
           cc_payment.status == CrossChainPaymentStatus::Completed {
            // Return current status (idempotent re-read)
            tx.rollback().await?;
            return Ok(self.build_status_response(&cc_payment));
        }
        
        // 3. Check quote hasn't expired
        if Utc::now() > cc_payment.quote_expires_at {
            // Allow registration anyway? No — if expired, reject unless tx was submitted before expiry.
            // Check tx timestamp from blockchain (expensive). Compromise: allow with +60s grace period.
            let grace = Utc::now() - chrono::Duration::seconds(60);
            if cc_payment.quote_expires_at < grace {
                sqlx::query("UPDATE cross_chain_payments SET status = 'expired', failed_reason = 'Quote expired before registration' WHERE id = $1")
                    .bind(cc_payment.id).execute(&mut *tx).await?;
                tx.commit().await?;
                return Err(DeloraError::QuoteExpired);
            }
        }
        
        // 4. Verify sender_address matches (prevent front-end spoofing)
        if let Some(ref stored_sender) = cc_payment.sender_address {
            if stored_sender != sender_address {
                tx.rollback().await?;
                return Err(DeloraError::SenderMismatch);
            }
        }
        
        // 5. Check origin_tx_hash uniqueness across ALL cross-chain payments
        let existing = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM cross_chain_payments WHERE origin_tx_hash = $1 AND id != $2 AND deleted_at IS NULL)"
        )
        .bind(tx_hash)
        .bind(cc_payment.id)
        .fetch_one(&mut *tx)
        .await?;
        
        if existing {
            tx.rollback().await?;
            return Err(DeloraError::TransactionAlreadyRegistered);
        }
        
        // 6. Update record
        sqlx::query(
            "UPDATE cross_chain_payments 
             SET status = 'tx_submitted', origin_tx_hash = $1, sender_address = $2,
                 tx_submitted_at = NOW(), updated_at = NOW()
             WHERE id = $3 AND status = 'quote_requested'"
        )
        .bind(tx_hash)
        .bind(sender_address)
        .bind(cc_payment.id)
        .execute(&mut *tx)
        .await?;
        
        // 7. If this is linked to a payment_transaction, update that too
        if let Some(pt_id) = cc_payment.payment_transaction_id {
            sqlx::query(
                "UPDATE payment_transactions 
                 SET status = 'CONFIRMING', transaction_hash = $1, last_verification_at = NOW()
                 WHERE id = $2 AND status IN ('PENDING', 'SELECTION_REQUIRED')"
            )
            .bind(tx_hash)
            .bind(pt_id)
            .execute(&mut *tx)
            .await?;
        }
        
        tx.commit().await?;
        
        // 8. Fire side effects: Redis pub/sub notification
        let _ = self.publish_cross_chain_update(&cc_payment).await;
        
        Ok(self.build_status_response(&cc_payment))
    }
    
    /// Get real-time status of a cross-chain payment
    pub async fn get_cross_chain_status(&self, link_id: &str) -> Result<CrossChainStatusResponse, DeloraError>;
    
    /// Get supported chains for the frontend picker
    pub async fn get_supported_chains(&self) -> Result<Vec<ChainSummary>, DeloraError>;
    
    /// Get supported tokens for a specific chain (for frontend picker)
    pub async fn get_supported_tokens(&self, chain_id: u64) -> Result<Vec<TokenSummary>, DeloraError>;
    
    // --- Internal helpers ---
    async fn lookup_payment_by_link(&self, link_id: &str) -> Result<PaymentLookup, DeloraError>;
    async fn get_merchant_destination(&self, payment: &PaymentLookup) -> Result<String, DeloraError>;
    async fn resolve_token_metadata(&self, chain_id: u64, address: &str) -> Result<CurrencySummary, DeloraError>;
    async fn persist_cross_chain_payment(&self, ...) -> Result<(), DeloraError>;
    fn build_fee_breakdown(&self, quote: &QuoteResponse, config: &DeloraConfig) -> QuoteFeeBreakdown;
    fn format_amount(&self, amount: &str, decimals: u32) -> String;
    async fn validate_supported_tokens(&self, chain_id: u64, address: &str) -> Result<(), DeloraError>;
    async fn publish_cross_chain_update(&self, payment: &CrossChainPayment) -> Result<()>;
    fn build_status_response(&self, payment: &CrossChainPayment) -> CrossChainStatusResponse;
    async fn build_direct_quote(&self, payment: PaymentLookup, addr: String) -> Result<CrossChainQuoteResponse, DeloraError>;
}
```

---

## 5. Calldata Validation (`backend/src/delora/validation.rs`)

```rust
pub struct CalldataValidator;

impl CalldataValidator {
    /// Validate that the calldata `to` address is a known Delora router contract.
    /// Prevent frontend/API tampering where a malicious actor could redirect funds.
    pub fn validate_router_contract(calldata_to: &str, origin_chain_id: u64) -> Result<(), DeloraError> {
        let allowed_contracts = get_allowed_routers_for_chain(origin_chain_id);
        let normalized = calldata_to.to_lowercase();
        
        if !allowed_contracts.iter().any(|c| c.to_lowercase() == normalized) {
            return Err(DeloraError::InvalidRouterContract {
                got: calldata_to.to_string(),
                chain_id: origin_chain_id,
            });
        }
        Ok(())
    }
    
    /// Verify the destination address in calldata matches the merchant's address.
    /// This is critical: the calldata encodes where funds end up.
    pub fn verify_destination_in_calldata(
        calldata: &Calldata,
        merchant_address: &str,
    ) -> Result<(), DeloraError> {
        // Decode calldata (simplified — real impl would parse the ABI-encoded data)
        // For Delora, the merchant address is NOT in calldata directly — it's in the
        // receiverAddress parameter. The calldata.to is the router contract.
        // This validation happens at the quote request level (we pass receiverAddress=merchant_addr).
        // Here we verify the calldata format looks valid (non-empty, correct hex).
        
        if calldata.to.is_empty() || !calldata.to.starts_with("0x") {
            return Err(DeloraError::InvalidCalldata("Empty or malformed 'to' address".into()));
        }
        if calldata.data.is_empty() || !calldata.data.starts_with("0x") {
            return Err(DeloraError::InvalidCalldata("Empty or malformed 'data' field".into()));
        }
        Ok(())
    }
    
    /// Validate that we're not being asked to send funds to an unexpected chain
    pub fn validate_chain_support(chain_id: u64) -> Result<(), DeloraError> {
        // Check against Delora's /v1/chains response (cached)
        // Placeholder: supported chains include major EVM + Solana
        Ok(())
    }
}
```

---

## 6. API Routes & Handlers (`backend/src/delora/handlers.rs`)

### GET `/api/v1/payments/cross-chain-quote`

```rust
#[derive(Deserialize)]
pub struct CrossChainQuoteQuery {
    pub link_id: String,
    pub sender_address: String,
    pub origin_chain_id: u64,
    pub origin_currency: String,  // token address
    #[serde(default)]
    pub slippage: Option<f64>,
    #[serde(default)]
    pub use_advanced: bool,
}

pub async fn get_cross_chain_quote(
    State(state): State<AppState>,
    Query(params): Query<CrossChainQuoteQuery>,
    // Public endpoint — no auth middleware
) -> Result<Json<CrossChainQuoteResponse>, AppError> {
    // Input validation
    if params.sender_address.trim().is_empty() {
        return Err(AppError::BadRequest("sender_address required".into()));
    }
    
    let result = state.delora_service.get_cross_chain_quote(
        &params.link_id,
        &params.sender_address,
        params.origin_chain_id,
        &params.origin_currency,
    ).await;
    
    match result {
        Ok(quote) => Ok(Json(quote)),
        Err(e) => {
            tracing::warn!(
                link_id = %params.link_id,
                error = %e,
                "Failed to get cross-chain quote"
            );
            Err(map_delora_error_to_app_error(e))
        }
    }
}
```

### POST `/api/v1/payments/cross-chain-register`

```rust
#[derive(Deserialize)]
pub struct RegisterCrossChainRequest {
    pub quote_id: Uuid,
    pub tx_hash: String,
    pub sender_address: String,
}

#[derive(Serialize)]
pub struct RegisterCrossChainResponse {
    pub status: CrossChainPaymentStatus,
    pub payment_id: Option<String>,
    pub message: String,
}

pub async fn register_cross_chain_tx(
    State(state): State<AppState>,
    Json(payload): Json<RegisterCrossChainRequest>,
) -> Result<Json<RegisterCrossChainResponse>, AppError> {
    // Validate tx_hash format (0x + 64 hex chars for EVM, base58 for Solana)
    if !is_valid_tx_hash(&payload.tx_hash) {
        return Err(AppError::BadRequest("Invalid transaction hash format".into()));
    }
    
    let status = state.delora_service.register_cross_chain_tx(
        &payload.quote_id,
        &payload.tx_hash,
        &payload.sender_address,
    ).await;
    
    match status {
        Ok(s) => Ok(Json(RegisterCrossChainResponse {
            status: s.status,
            payment_id: Some(s.payment_id),
            message: "Transaction registered successfully".into(),
        })),
        Err(DeloraError::TransactionAlreadyRegistered) => {
            // Idempotent — return success if already registered
            Ok(Json(RegisterCrossChainResponse {
                status: CrossChainPaymentStatus::TxSubmitted,
                payment_id: None,
                message: "Transaction already registered".into(),
            }))
        }
        Err(e) => Err(map_delora_error_to_app_error(e)),
    }
}
```

### GET `/api/v1/payments/cross-chain-status/:link_id`

```rust
pub async fn get_cross_chain_status(
    State(state): State<AppState>,
    Path(link_id): Path<String>,
) -> Result<Json<CrossChainStatusResponse>, AppError> {
    state.delora_service.get_cross_chain_status(&link_id)
        .await
        .map(Json)
        .map_err(map_delora_error_to_app_error)
}
```

### GET `/api/v1/payments/cross-chain/chains`

```rust
pub async fn get_supported_chains(
    State(state): State<AppState>,
) -> Result<Json<Vec<ChainSummary>>, AppError> {
    state.delora_service.get_supported_chains()
        .await
        .map(Json)
        .map_err(map_delora_error_to_app_error)
}
```

### GET `/api/v1/payments/cross-chain/tokens/:chain_id`

```rust
pub async fn get_supported_tokens(
    State(state): State<AppState>,
    Path(chain_id): Path<u64>,
) -> Result<Json<Vec<TokenSummary>>, AppError> {
    state.delora_service.get_supported_tokens(chain_id)
        .await
        .map(Json)
        .map_err(map_delora_error_to_app_error)
}
```

---

## 7. Bridge Monitor (`backend/src/delora/bridge_monitor.rs`)

A background task that periodically polls for pending bridge completions.

```rust
pub struct BridgeMonitor {
    db: PgPool,
    service: Arc<DeloraService>,
    client: Arc<DeloraClient>,
    poll_interval: Duration,          // default: 30 seconds
    max_concurrent_checks: usize,     // default: 10
}

impl BridgeMonitor {
    pub fn start(self) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(self.poll_interval);
            loop {
                interval.tick().await;
                if let Err(e) = self.poll_pending_bridges().await {
                    tracing::error!(error = %e, "Bridge monitor polling failed");
                }
                // Memory safety: interval ensures bounded loop rate,
                // connection reaper below prevents long-lived stale connections
            }
        })
    }
    
    async fn poll_pending_bridges(&self) -> Result<()> {
        // Find all cross-chain payments in tx_confirmed or bridge_pending status
        // that haven't been updated in the last 5 minutes
        let pending = sqlx::query_as::<_, CrossChainPayment>(
            "SELECT * FROM cross_chain_payments 
             WHERE status IN ('tx_confirmed', 'bridge_pending')
             AND updated_at < NOW() - INTERVAL '5 minutes'
             AND deleted_at IS NULL
             ORDER BY updated_at ASC
             LIMIT $1"
        )
        .bind(self.max_concurrent_checks as i64)
        .fetch_all(&self.db)
        .await?;
        
        // Process concurrently with a semaphore (bounded concurrency, no memory bloat)
        let semaphore = Arc::new(tokio::sync::Semaphore::new(self.max_concurrent_checks));
        let mut handles = Vec::with_capacity(pending.len());
        
        for payment in pending {
            let permit = semaphore.clone().acquire_owned().await?;
            let db = self.db.clone();
            let client = self.client.clone();
            
            handles.push(tokio::spawn(async move {
                let _permit = permit; // holds permit until done
                Self::check_bridge_status(payment, &db, &client).await
            }));
        }
        
        // Await all, log failures individually (don't fail the whole batch)
        for handle in handles {
            if let Err(e) = handle.await {
                tracing::error!(error = %e, "Bridge check task panicked");
            }
        }
        
        Ok(())
    }
    
    async fn check_bridge_status(
        payment: CrossChainPayment,
        db: &PgPool,
        client: &DeloraClient,
    ) -> Result<()> {
        // For EVM chains: use blockchain monitor to check destination chain for incoming tx
        // For bridge_scan metadata from Delora: use the bridge tracking URL/API
        
        if let Some(ref bridge_meta) = payment.bridge_scan_metadata {
            // Some bridges provide tracking APIs (e.g., Across, Relay)
            // Check if funds have arrived on destination
            // ...
        }
        
        // Simplest check: query the merchant's destination address on the destination chain
        // for new transactions since last check. Use existing blockchain monitor infrastructure.
        
        // If confirmed → update status to bridge_complete or completed
        // If stuck for > configurable threshold → alert
        
        Ok(())
    }
}
```

---

## 8. Config Additions (`backend/src/config.rs`)

```rust
#[derive(Clone, Debug)]
pub struct DeloraConfig {
    pub base_url: String,                        // "https://api.delora.build"
    pub api_key: Option<String>,                 // DELORA_API_KEY env var
    pub integrator_id: String,                   // "fiddupay" — DELORA_INTEGRATOR_ID
    pub default_integrator_fee: f64,             // 0.005 = 0.5% — DELORA_DEFAULT_FEE
    pub default_slippage: f64,                   // 0.005 = 0.5% — DELORA_DEFAULT_SLIPPAGE
    pub enabled: bool,                           // DELORA_ENABLED=true
    pub sandbox_mode: bool,                      // DELORA_SANDBOX=false
    pub request_timeout_secs: u64,               // 30
    pub connect_timeout_secs: u64,               // 10
    pub pool_max_idle_per_host: usize,           // 10
    pub pool_idle_timeout_secs: u64,             // 90
    pub tcp_keepalive_secs: u64,                 // 60
    pub rate_limit_per_minute: u32,              // 180 (leaves margin below 200)
    pub max_retries: u32,                        // 3
    pub circuit_breaker_threshold: u32,          // 5
    pub circuit_breaker_timeout_secs: u64,       // 60
    pub quote_cache_ttl_secs: u64,               // 15
    pub metadata_cache_ttl_secs: u64,            // 3600
    pub bridge_poll_interval_secs: u64,          // 30
    pub bridge_max_concurrent_checks: usize,     // 10
    pub quote_expiry_grace_seconds: i64,         // 60
    pub max_quote_amount_usd: f64,               // 10000.00 — anti-abuse
    pub min_quote_amount_usd: f64,               // 1.00
}

impl Default for DeloraConfig {
    fn default() -> Self {
        Self {
            base_url: "https://api.delora.build".into(),
            api_key: None,
            integrator_id: "fiddupay".into(),
            default_integrator_fee: 0.005,
            default_slippage: 0.005,
            enabled: false,
            sandbox_mode: false,
            request_timeout_secs: 30,
            connect_timeout_secs: 10,
            pool_max_idle_per_host: 10,
            pool_idle_timeout_secs: 90,
            tcp_keepalive_secs: 60,
            rate_limit_per_minute: 180,
            max_retries: 3,
            circuit_breaker_threshold: 5,
            circuit_breaker_timeout_secs: 60,
            quote_cache_ttl_secs: 15,
            metadata_cache_ttl_secs: 3600,
            bridge_poll_interval_secs: 30,
            bridge_max_concurrent_checks: 10,
            quote_expiry_grace_seconds: 60,
            max_quote_amount_usd: 10000.0,
            min_quote_amount_usd: 1.0,
        }
    }
}
```

### Env var bindings

| Env Var | Config Field | Default |
|---------|-------------|---------|
| `DELORA_ENABLED` | `enabled` | `false` |
| `DELORA_API_KEY` | `api_key` | `None` |
| `DELORA_INTEGRATOR_ID` | `integrator_id` | `"fiddupay"` |
| `DELORA_DEFAULT_FEE` | `default_integrator_fee` | `0.005` |
| `DELORA_DEFAULT_SLIPPAGE` | `default_slippage` | `0.005` |
| `DELORA_SANDBOX` | `sandbox_mode` | `false` |
| `DELORA_TIMEOUT_SECS` | `request_timeout_secs` | `30` |
| `DELORA_RATE_LIMIT_PER_MIN` | `rate_limit_per_minute` | `180` |
| `DELORA_MAX_RETRIES` | `max_retries` | `3` |
| `DELORA_MAX_QUOTE_USD` | `max_quote_amount_usd` | `10000` |

---

## 9. Double-Credit Prevention for Cross-Chain

The existing three-layer defense must be extended for cross-chain payments because the same transaction hash can exist on **different chains**.

### Layer 1 (DB): Composite Unique Constraint

```sql
-- Replaces the simple unique index on origin_tx_hash
CREATE UNIQUE INDEX idx_ccp_origin_tx_chain_unique 
    ON cross_chain_payments(origin_tx_hash, origin_chain_id) 
    WHERE origin_tx_hash IS NOT NULL AND deleted_at IS NULL;
```

### Layer 2 (App): Hash + Chain Check

In `register_cross_chain_tx`, the uniqueness check queries both hash AND chain:
```sql
SELECT EXISTS(SELECT 1 FROM cross_chain_payments 
    WHERE origin_tx_hash = $1 AND origin_chain_id = $2 
    AND id != $3 AND deleted_at IS NULL)
```

### Layer 3 (Transaction): FOR UPDATE + Status Check

Already covered: `SELECT ... FOR UPDATE` then checking status != COMPLETED before any mutation.

### Payment Transaction Linking

When `register_cross_chain_tx` links to a `payment_transactions` row, it sets status to CONFIRMING. The existing `confirm_payment()` flow already has its own three-layer defense (`FOR UPDATE` + status check + ON CONFLICT balance credit). Cross-chain registration does NOT credit the balance — that's done by `confirm_payment()` later, triggered by the bridge monitor detecting destination-chain arrival. This separation means:

1. Registration = "we know about the tx, tracking it" (sets CONFIRMING on payment_transactions)
2. Bridge completion = "funds arrived on destination" (bridge monitor detects)
3. Confirmation = "confirmations met on destination" → calls existing `confirm_payment()` which does the balance credit

The existing `confirm_payment()` remains the single path to balance credit. Cross-chain doesn't bypass it.

---

## 10. Error Handling (`backend/src/delora/error.rs`)

```rust
#[derive(Debug, thiserror::Error)]
pub enum DeloraError {
    #[error("Circuit breaker is open")]
    CircuitBreakerOpen,
    
    #[error("Rate limited, retry after {retry_after_secs}s")]
    RateLimited { retry_after_secs: u64 },
    
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),
    
    #[error("API error {status}: {code} - {message}")]
    ApiError { status: u16, code: String, message: String },
    
    #[error("No routes found for the requested swap")]
    NoRoutesFound,
    
    #[error("Route has no steps")]
    NoStepsInRoute,
    
    #[error("Quote not found: {0}")]
    QuoteNotFound(#[from] Uuid),
    
    #[error("Quote has expired")]
    QuoteExpired,
    
    #[error("Sender address mismatch")]
    SenderMismatch,
    
    #[error("Transaction already registered")]
    TransactionAlreadyRegistered,
    
    #[error("Invalid router contract: got {got} for chain {chain_id}")]
    InvalidRouterContract { got: String, chain_id: u64 },
    
    #[error("Invalid calldata: {0}")]
    InvalidCalldata(String),
    
    #[error("Amount exceeds maximum: ${amount} > ${max}")]
    AmountExceedsLimit { amount: f64, max: f64 },
    
    #[error("Amount below minimum: ${amount} < ${min}")]
    AmountBelowLimit { amount: f64, min: f64 },
    
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

// Mapper for converting to AppError
fn map_delora_error_to_app_error(e: DeloraError) -> AppError {
    match e {
        DeloraError::CircuitBreakerOpen => 
            AppError::ServiceUnavailable("Cross-chain swap temporarily unavailable".into()),
        DeloraError::RateLimited { .. } =>
            AppError::TooManyRequests("Too many swap requests, please try again".into()),
        DeloraError::QuoteNotFound(_) =>
            AppError::NotFound("Quote not found or expired".into()),
        DeloraError::QuoteExpired =>
            AppError::Gone("Quote has expired, please request a new one".into()),
        DeloraError::TransactionAlreadyRegistered =>
            // Not an error from the user's perspective — idempotent success
            AppError::Conflict("Transaction already registered".into()),
        DeloraError::NoRoutesFound =>
            AppError::BadRequest("No swap route available for this token pair".into()),
        _ => {
            tracing::error!(error = %e, "Delora internal error");
            AppError::InternalServerError("Swap service error, please try again".into())
        }
    }
}
```

---

## 11. Memory Leak Prevention

| Concern | Mitigation |
|---------|-----------|
| **Unbounded SharedFuture map** | Bounded `LruCache` (512 entries, LRU eviction) instead of unbounded `HashMap` for dedup futures. Stale futures are evicted. |
| **Circuit breaker state accumulation** | `CircuitBreaker` state is per-`DeloraClient` instance (one total), not per-endpoint. No accumulation. |
| **tokio::spawn leak** | Bridge monitor uses `tokio::sync::Semaphore` with `max_concurrent_checks` to bound concurrent tasks. Failed tasks are awaited and dropped. |
| **Redis connection leak** | Single multiplexed connection from `redis::Client::get_multiplexed_async_connection()`, reused. No per-request connections. |
| **reqwest connection pool** | Configured with `pool_max_idle_per_host(10)` and `pool_idle_timeout(90s)` — connections are reaped after idle timeout. |
| **Quote snapshot cache** | Redis TTL (30s) ensures auto-expiry. No manual cleanup needed. |
| **CrossChainPayment DB rows** | Status transitions are monotonic → terminal. No polling on completed/failed/expired rows. Indexed by status for efficient bridge monitor queries. |
| **Rate limiter state** | `governor::InMemoryState` is fixed-size (sliding window counters). No unbounded growth. |

---

## 12. Frontend Integration (`frontend/src/`)

### New Components

```
frontend/src/
├── components/
│   └── payment/
│       ├── CrossChainPicker.tsx       # Token/chain selector dropdown
│       ├── DeloraQuoteDisplay.tsx     # Quote details, fee breakdown, warnings
│       └── CrossChainStatusPoller.tsx # Real-time status polling
├── hooks/
│   └── useCrossChainQuote.ts          # React Query hook for quote fetching
├── services/
│   └── deloraService.ts              # API client for backend endpoints
└── types/
    └── delora.ts                      # TypeScript type definitions
```

### Flow

```
1. Payment page loads → check if payment is multi-currency or has cross-chain enabled
2. If yes → show CrossChainPicker with supported chains/tokens (fetched from /chains, /tokens endpoints)
3. Customer selects their origin chain/token → triggers GET /cross-chain-quote
4. DeloraQuoteDisplay renders:
   - "You pay: 1,000 USDC on Base"
   - "Merchant receives: ~995 USDT on Polygon"  
   - Fee breakdown (Delora: $X, FidduPay: $Y, Gas: ~$Z)
   - Estimated time: ~2 minutes
   - Warnings (if any)
5. Customer clicks "Pay with Wallet" → wallet signature flow
   - If ERC-20: check allowance → approve if needed
   - Send transaction using calldata from quote
6. Customer gets tx hash → POST /cross-chain-register
7. CrossChainStatusPoller polls GET /cross-chain-status/:link_id every 5s
   - Shows: "Transaction submitted → Confirmed on Base → Bridging → Arrived on Polygon → Confirmed"
   - When completed: redirect to success page
```

### TypeScript Types (`frontend/src/types/delora.ts`)

```typescript
interface CrossChainQuoteResponse {
  quoteId: string;
  originChainId: number;
  destinationChainId: number;
  originCurrency: CurrencySummary;
  destinationCurrency: CurrencySummary;
  inputAmount: string;
  inputAmountDisplay: string;
  outputAmount: string;
  outputAmountDisplay: string;
  minOutputAmount: string;
  fees: QuoteFeeBreakdown;
  estimatedTimeSec?: number;
  calldata: { to: string; value: string; data: string };
  gas?: { gasPrice?: string; maxFeePerGas?: string; maxPriorityFeePerGas?: string; gasLimit?: string };
  approvalAddress?: string;
  warnings: Array<{ code: string; message: string }>;
  expiresAt: string;
  route?: AdvancedRouteSummary;
}

type CrossChainPaymentStatus = 
  | 'quote_requested' | 'tx_submitted' | 'tx_confirmed' 
  | 'bridge_pending' | 'bridge_complete' | 'completed' | 'failed' | 'expired';
```

---

## 13. Integration with Existing PaymentVerifier

The bridge monitor detects destination-chain arrival by querying the merchant's deposit address on the destination chain. When confirmations are met, it calls `PaymentVerifier::confirm_payment()` — the **exact same function** used for native payments. This means:

- Same `FOR UPDATE` locking
- Same idempotency check (`status != 'CONFIRMED'`)
- Same atomic balance credit (`ON CONFLICT DO UPDATE`)
- Same Redis Pub/Sub broadcast
- Same webhook dispatch

No modification to `verifier.rs` is needed. The bridge monitor just needs to call the existing public method.

---

## 14. Implementation Order

| Phase | Files | Description |
|-------|-------|-------------|
| **1. Models** | `models.rs`, `error.rs`, `constants.rs` | All Rust types, error enum, chain/token constants |
| **2. Config** | `config.rs` | Add `DeloraConfig` struct + env var parsing |
| **3. Client** | `client.rs` | `DeloraClient` with pooling, retry, circuit breaker |
| **4. Cache** | `cache.rs` | `DeloraCache` with Redis + LRU |
| **5. Service** | `service.rs` | `DeloraService` with quote + register logic |
| **6. Validation** | `validation.rs` | Calldata/address validation |
| **7. DB Migration** | `migrations/20260711...sql` | `cross_chain_payments` table |
| **8. API Layer** | `handlers.rs`, `routes.rs`, `mod.rs` | Route definitions + handler functions |
| **9. AppState wiring** | `api/state.rs`, `api/routes.rs`, `main.rs` | Add `DeloraService` to state, mount routes, start monitor |
| **10. Bridge Monitor** | `bridge_monitor.rs` | Background task for bridge completion |
| **11. Frontend** | `CrossChainPicker.tsx`, `DeloraQuoteDisplay.tsx`, etc. | UI components |
| **12. Tests** | `backend/src/delora/tests/` | Integration tests with mock Delora server |

---

## 15. Verification

### Unit Tests
- `DeloraClient` retry behavior (mock HTTP server returning 429, 500, success)
- `CalldataValidator` with valid/invalid router addresses
- `DeloraCache` get-or-fetch with concurrent callers (SharedFuture dedup)
- `DeloraService::get_cross_chain_quote` with same-chain skip
- `DeloraService::register_cross_chain_tx` idempotency (repeat registration)

### Integration Tests
- End-to-end: mock Delora API → quote request → register → status polling
- Double-credit: two concurrent registrations with same tx_hash
- Bridge monitor: seed DB with tx_confirmed row, mock destination chain response, verify status transitions

### Manual Testing Checklist
- [ ] Payment page loads cross-chain picker when payment is multi-currency
- [ ] Quote displays correct input/output amounts and fees
- [ ] ERC-20 approval flow works (approval_address is set)
- [ ] Wallet signing produces valid transaction
- [ ] Registration succeeds, status polling shows progress
- [ ] Bridge completion triggers merchant balance credit
- [ ] Webhook fires on completion
- [ ] Redis pub/sub delivers real-time status update to dashboard
- [ ] Circuit breaker opens after 5 consecutive failures, closes after timeout
- [ ] Backend restart doesn't lose tracked bridge payments (DB-backed)
- [ ] Quote expiry rejects stale registrations (beyond grace period)
