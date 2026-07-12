// Delora HTTP Client
// Shared reqwest pool with circuit breaker, rate limiting, retry, and metrics

use crate::delora::error::DeloraError;
use crate::delora::models::*;
use crate::utils::circuit_breaker::CircuitBreaker;
use governor::{
    clock::DefaultClock,
    middleware::NoOpMiddleware,
    state::{InMemoryState, NotKeyed},
    Quota, RateLimiter,
};
use reqwest::{Method, StatusCode};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::num::NonZeroU32;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::debug;

/// Configuration for Delora HTTP client
#[derive(Debug, Clone)]
pub struct DeloraClientConfig {
    pub base_url: String,
    pub api_key: Option<String>,
    pub pool_max_idle_per_host: usize,
    pub pool_idle_timeout_secs: u64,
    pub request_timeout_secs: u64,
    pub connect_timeout_secs: u64,
    pub tcp_keepalive_secs: u64,
    pub rate_limit_per_minute: u32,
    pub max_retries: u32,
    pub circuit_breaker_threshold: u32,
    pub circuit_breaker_timeout_secs: u64,
}

impl Default for DeloraClientConfig {
    fn default() -> Self {
        Self {
            base_url: "https://api.delora.build".into(),
            api_key: None,
            pool_max_idle_per_host: 10,
            pool_idle_timeout_secs: 90,
            request_timeout_secs: 30,
            connect_timeout_secs: 10,
            tcp_keepalive_secs: 60,
            rate_limit_per_minute: 180,
            max_retries: 3,
            circuit_breaker_threshold: 5,
            circuit_breaker_timeout_secs: 60,
        }
    }
}

#[derive(Debug, Default)]
struct ClientMetrics {
    total_requests: AtomicU64,
    successful_requests: AtomicU64,
    failed_requests: AtomicU64,
    rate_limited_requests: AtomicU64,
    circuit_breaks: AtomicU64,
    average_latency_ms: AtomicU64,
}

/// Shared, pooled HTTP client for all Delora API calls.
/// One instance per AppState.
pub struct DeloraClient {
    http: reqwest::Client,
    base_url: String,
    api_key: Option<String>,
    rate_limiter: Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock, NoOpMiddleware>>,
    circuit_breaker: Arc<CircuitBreaker>,
    max_retries: u32,
    metrics: ClientMetrics,
}

impl DeloraClient {
    pub fn new(config: DeloraClientConfig) -> Self {
        let http = reqwest::Client::builder()
            .pool_max_idle_per_host(config.pool_max_idle_per_host)
            .pool_idle_timeout(Duration::from_secs(config.pool_idle_timeout_secs))
            .timeout(Duration::from_secs(config.request_timeout_secs))
            .connect_timeout(Duration::from_secs(config.connect_timeout_secs))
            .tcp_keepalive(Duration::from_secs(config.tcp_keepalive_secs))
            .http2_prior_knowledge()
            .user_agent(format!(
                "FidduPay/{} (Delora Integration)",
                option_env!("CARGO_PKG_VERSION").unwrap_or("2.4.6")
            ))
            .build()
            .expect("Failed to create Delora HTTP client");

        let quota = Quota::per_minute(
            NonZeroU32::new(config.rate_limit_per_minute.max(1))
                .expect("rate_limit_per_minute must be >= 1"),
        );
        let rate_limiter = Arc::new(RateLimiter::direct(quota));

        let circuit_breaker = Arc::new(CircuitBreaker::new(
            config.circuit_breaker_threshold,
            config.circuit_breaker_timeout_secs,
        ));

        Self {
            http,
            base_url: config.base_url,
            api_key: config.api_key,
            rate_limiter,
            circuit_breaker,
            max_retries: config.max_retries,
            metrics: ClientMetrics::default(),
        }
    }

    // ── Public API Methods ───────────────────────────────────────────────

    pub async fn get_quote(&self, params: &QuoteRequest) -> Result<QuoteResponse, DeloraError> {
        let query: Vec<(String, String)> = vec![
            ("senderAddress".into(), params.sender_address.clone()),
            ("receiverAddress".into(), params.receiver_address.clone()),
            ("originChainId".into(), params.origin_chain_id.to_string()),
            (
                "destinationChainId".into(),
                params.destination_chain_id.to_string(),
            ),
            ("amount".into(), params.amount.clone()),
            ("originCurrency".into(), params.origin_currency.clone()),
            (
                "destinationCurrency".into(),
                params.destination_currency.clone(),
            ),
            ("integrator".into(), params.integrator.clone()),
            ("fee".into(), params.fee.to_string()),
        ];

        self.get("/v1/quotes", &query).await
    }

    pub async fn get_advanced_routes(
        &self,
        params: &AdvancedRoutesRequest,
    ) -> Result<AdvancedRoutesResponse, DeloraError> {
        self.post("/v1/advanced/routes", params).await
    }

    pub async fn populate_step_transaction(
        &self,
        step: &RouteStep,
        context: &StepTransactionContext,
    ) -> Result<RouteStep, DeloraError> {
        let body = serde_json::json!({
            "step": step,
            "context": context,
        });
        self.post("/v1/advanced/stepTransaction", &body).await
    }

    pub async fn get_chains(&self) -> Result<Vec<ChainInfo>, DeloraError> {
        self.get("/v1/chains", &[]).await
    }

    pub async fn get_tokens(&self) -> Result<TokenListResponse, DeloraError> {
        self.get("/v1/tokens", &[]).await
    }

    pub async fn get_tools(&self) -> Result<Vec<ToolInfo>, DeloraError> {
        self.get("/v1/tools", &[]).await
    }

    pub async fn get_prices(&self) -> Result<PriceResponse, DeloraError> {
        self.get("/v1/prices", &[]).await
    }

    pub async fn get_token(
        &self,
        chain_id: u64,
        address_or_symbol: &str,
    ) -> Result<TokenItem, DeloraError> {
        let query = vec![
            ("chainId".into(), chain_id.to_string()),
            ("addressOrSymbol".into(), address_or_symbol.to_string()),
        ];
        self.get("/v1/token", &query).await
    }

    /// Expose metrics for health monitoring
    pub fn metrics_summary(&self) -> DeloraMetricsSummary {
        DeloraMetricsSummary {
            total: self.metrics.total_requests.load(Ordering::Relaxed),
            success: self.metrics.successful_requests.load(Ordering::Relaxed),
            failed: self.metrics.failed_requests.load(Ordering::Relaxed),
            rate_limited: self.metrics.rate_limited_requests.load(Ordering::Relaxed),
            circuit_breaks: self.metrics.circuit_breaks.load(Ordering::Relaxed),
            avg_latency_ms: self.metrics.average_latency_ms.load(Ordering::Relaxed),
        }
    }

    // ── Internal HTTP Methods ───────────────────────────────────────────

    async fn get<T: DeserializeOwned>(
        &self,
        path: &str,
        query: &[(String, String)],
    ) -> Result<T, DeloraError> {
        let url = format!("{}{}", self.base_url, path);
        let query_params: Vec<(&str, &str)> = query
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        self.execute_with_retry(&url, None, Some(&query_params))
            .await
    }

    async fn post<T: DeserializeOwned, B: Serialize + Send + Sync>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, DeloraError> {
        let url = format!("{}{}", self.base_url, path);
        let body_json = serde_json::to_value(body).map_err(DeloraError::Serialization)?;
        let body_string = serde_json::to_string(&body_json).map_err(DeloraError::Serialization)?;
        self.execute_with_retry::<T>(&url, Some(&body_string), None)
            .await
    }

    /// Core execution method with circuit breaker, rate limiting, and retry
    async fn execute_with_retry<T: DeserializeOwned>(
        &self,
        url: &str,
        body_json: Option<&str>,
        query_params: Option<&[(&str, &str)]>,
    ) -> Result<T, DeloraError> {
        // 1. Circuit breaker check + half-open transition
        {
            if !self.circuit_breaker.try_half_open().await {
                self.metrics.circuit_breaks.fetch_add(1, Ordering::Relaxed);
                return Err(DeloraError::CircuitBreakerOpen);
            }
        }

        // 2. Rate limit
        self.rate_limiter.until_ready().await;
        self.metrics.total_requests.fetch_add(1, Ordering::Relaxed);

        // 3. Execute with retry
        let mut last_error: Option<DeloraError> = None;
        let url_owned = url.to_string();
        let body_owned = body_json.map(|s| s.to_string());
        let query_owned = query_params.map(|q| q.to_vec());

        for attempt in 0..=self.max_retries {
            if attempt > 0 {
                let delay = self.backoff_delay(attempt, &last_error);
                debug!(
                    "Delora retry attempt {}/{}, delay={:?}",
                    attempt, self.max_retries, delay
                );
                tokio::time::sleep(delay).await;
            }

            let start = Instant::now();
            let mut req = if let Some(ref body_str) = body_owned {
                self.http
                    .request(Method::POST, &url_owned)
                    .body(body_str.clone())
                    .header("Content-Type", "application/json")
            } else {
                let mut r = self.http.request(Method::GET, &url_owned);
                if let Some(ref q) = query_owned {
                    r = r.query(&q.iter().map(|(k, v)| (*k, *v)).collect::<Vec<_>>());
                }
                r
            };

            if let Some(ref key) = self.api_key {
                req = req.header("x-api-key", key.as_str());
            }

            match req.send().await {
                Ok(response) => {
                    let elapsed = start.elapsed();

                    let status = response.status();

                    if status == StatusCode::TOO_MANY_REQUESTS {
                        self.metrics
                            .rate_limited_requests
                            .fetch_add(1, Ordering::Relaxed);
                        let retry_after = response
                            .headers()
                            .get("x-ratelimit-reset")
                            .and_then(|v| v.to_str().ok())
                            .and_then(parse_iso_to_seconds_from_now);
                        last_error = Some(DeloraError::RateLimited {
                            retry_after_secs: retry_after.unwrap_or(60),
                        });
                        continue;
                    }

                    if !status.is_success() {
                        let error_body: Result<DeloraApiError, _> = response.json().await;
                        let api_err = match error_body {
                            Ok(e) => DeloraError::ApiError {
                                status: status.as_u16(),
                                code: e.code,
                                message: e.message,
                            },
                            Err(_) => DeloraError::ApiError {
                                status: status.as_u16(),
                                code: "UNKNOWN".into(),
                                message: format!("HTTP {}", status.as_u16()),
                            },
                        };

                        let is_retryable = status.is_server_error();
                        if is_retryable && attempt < self.max_retries {
                            last_error = Some(api_err);
                            continue;
                        }
                        self.metrics.failed_requests.fetch_add(1, Ordering::Relaxed);
                        self.circuit_breaker.record_failure().await;
                        return Err(api_err);
                    }

                    // Success
                    self.metrics
                        .successful_requests
                        .fetch_add(1, Ordering::Relaxed);
                    self.update_latency(elapsed);
                    self.circuit_breaker.record_success().await;

                    return Ok(response.json().await?);
                }
                Err(e) => {
                    let _elapsed = start.elapsed();

                    let is_retryable = e.is_timeout() || e.is_connect() || e.is_request();
                    let err = DeloraError::Http(e);

                    if is_retryable && attempt < self.max_retries {
                        last_error = Some(err);
                        continue;
                    }
                    self.metrics.failed_requests.fetch_add(1, Ordering::Relaxed);
                    self.circuit_breaker.record_failure().await;
                    return Err(err);
                }
            }
        }

        // All retries exhausted
        self.metrics.failed_requests.fetch_add(1, Ordering::Relaxed);
        self.circuit_breaker.record_failure().await;

        Err(last_error.unwrap_or(DeloraError::ApiError {
            status: 500,
            code: "MAX_RETRIES".into(),
            message: "All retry attempts exhausted".into(),
        }))
    }

    fn backoff_delay(&self, attempt: u32, last_error: &Option<DeloraError>) -> Duration {
        if let Some(DeloraError::RateLimited { retry_after_secs }) = last_error {
            return Duration::from_secs(*retry_after_secs);
        }
        let base = 2u64.pow(attempt.saturating_sub(1));
        Duration::from_secs(base.min(30))
    }

    fn update_latency(&self, elapsed: Duration) {
        let ms = elapsed.as_millis() as u64;
        let current = self.metrics.average_latency_ms.load(Ordering::Relaxed);
        // Exponential moving average: new = (current * 0.9) + (ms * 0.1)
        let new = ((current as f64 * 0.9) + (ms as f64 * 0.1)) as u64;
        self.metrics
            .average_latency_ms
            .store(new, Ordering::Relaxed);
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DeloraMetricsSummary {
    pub total: u64,
    pub success: u64,
    pub failed: u64,
    pub rate_limited: u64,
    pub circuit_breaks: u64,
    pub avg_latency_ms: u64,
}

fn parse_iso_to_seconds_from_now(s: &str) -> Option<u64> {
    chrono::DateTime::parse_from_rfc3339(s).ok().and_then(|dt| {
        let now = chrono::Utc::now();
        let dur = dt.signed_duration_since(now);
        if dur.num_seconds() > 0 {
            Some(dur.num_seconds() as u64)
        } else {
            None
        }
    })
}
