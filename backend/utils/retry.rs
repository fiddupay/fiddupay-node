use std::time::Duration;
use tokio::time::sleep;
use tracing::{warn, error};

pub async fn retry_with_backoff<F, Fut, T, E>(
    operation: F,
    max_retries: u32,
    operation_name: &str,
) -> Result<T, E>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    let mut attempt = 0;
    
    loop {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                attempt += 1;
                if attempt >= max_retries {
                    error!("{} failed after {} attempts: {}", operation_name, max_retries, e);
                    return Err(e);
                }
                
                let delay = Duration::from_secs(2u64.pow(attempt - 1));
                warn!("{} failed (attempt {}/{}), retrying in {:?}: {}", 
                    operation_name, attempt, max_retries, delay, e);
                sleep(delay).await;
            }
        }
    }
}

pub async fn query_blockchain_with_retry<F, Fut, T>(
    query: F,
    blockchain: &str,
) -> Result<T, String>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, String>>,
{
    retry_with_backoff(query, 3, &format!("{} blockchain query", blockchain)).await
}
