use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{warn, info};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

pub struct CircuitBreaker {
    state: Arc<RwLock<CircuitState>>,
    failure_count: Arc<RwLock<u32>>,
    last_failure_time: Arc<RwLock<Option<Instant>>>,
    failure_threshold: u32,
    timeout: Duration,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u32, timeout_secs: u64) -> Self {
        Self {
            state: Arc::new(RwLock::new(CircuitState::Closed)),
            failure_count: Arc::new(RwLock::new(0)),
            last_failure_time: Arc::new(RwLock::new(None)),
            failure_threshold,
            timeout: Duration::from_secs(timeout_secs),
        }
    }

    pub async fn call<F, Fut, T, E>(&self, operation: F) -> Result<T, E>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, E>>,
        E: From<String>,
    {
        let state = *self.state.read().await;

        match state {
            CircuitState::Open => {
                let last_failure = self.last_failure_time.read().await;
                if let Some(time) = *last_failure {
                    if time.elapsed() >= self.timeout {
                        info!("Circuit breaker transitioning to half-open");
                        *self.state.write().await = CircuitState::HalfOpen;
                    } else {
                        return Err(E::from("Circuit breaker is open".to_string()));
                    }
                }
            }
            _ => {}
        }

        match operation().await {
            Ok(result) => {
                self.on_success().await;
                Ok(result)
            }
            Err(e) => {
                self.on_failure().await;
                Err(e)
            }
        }
    }

    async fn on_success(&self) {
        let state = *self.state.read().await;
        if state == CircuitState::HalfOpen {
            info!("Circuit breaker closing after successful call");
            *self.state.write().await = CircuitState::Closed;
        }
        *self.failure_count.write().await = 0;
    }

    async fn on_failure(&self) {
        let mut count = self.failure_count.write().await;
        *count += 1;

        if *count >= self.failure_threshold {
            warn!("Circuit breaker opening after {} failures", count);
            *self.state.write().await = CircuitState::Open;
            *self.last_failure_time.write().await = Some(Instant::now());
        }
    }

    pub async fn is_open(&self) -> bool {
        *self.state.read().await == CircuitState::Open
    }
}
