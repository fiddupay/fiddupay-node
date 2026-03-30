// Bitcoin API Client with Failover Support
// Provides a resilient HTTP client for Esplorer-compatible APIs
// (Blockstream / Mempool.space) with automatic failover to a backup provider.

use tracing::{info, warn};

/// Configuration for the Bitcoin API client. Holds the primary and backup URLs.
pub struct BitcoinApiConfig {
    pub primary_url: String,
    pub backup_url: String,
}

impl BitcoinApiConfig {
    /// Create a mainnet config from global app config.
    pub fn mainnet(config: &crate::config::Config) -> Self {
        Self {
            primary_url: config.bitcoin_rpc_url.trim_end_matches('/').to_string(),
            backup_url: config.bitcoin_rpc_url_backup.trim_end_matches('/').to_string(),
        }
    }

    /// Create a testnet config from global app config.
    pub fn testnet(config: &crate::config::Config) -> Self {
        Self {
            primary_url: config.bitcoin_testnet_rpc_url.trim_end_matches('/').to_string(),
            backup_url: config.bitcoin_testnet_rpc_url_backup.trim_end_matches('/').to_string(),
        }
    }

    /// Select testnet or mainnet config based on whether `is_sandbox` is true.
    pub fn from_config(config: &crate::config::Config, is_sandbox: bool) -> Self {
        if is_sandbox {
            Self::testnet(config)
        } else {
            Self::mainnet(config)
        }
    }
}

/// Performs a GET request to the primary Bitcoin API URL.
/// If the request fails (network error, timeout, or a 5xx response),
/// it automatically retries using the backup URL.
/// Returns the raw JSON response body on success.
pub async fn get_with_failover(
    config: &BitcoinApiConfig,
    path: &str,
) -> Result<serde_json::Value, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))?;

    let primary_url = format!("{}/{}", config.primary_url, path.trim_start_matches('/'));

    match fetch_json(&client, &primary_url).await {
        Ok(response) => {
            return Ok(response);
        }
        Err(e) => {
            warn!(
                primary_url = %primary_url,
                error = %e,
                "Bitcoin primary API failed, failing over to backup"
            );
        }
    }

    // Failover to backup
    let backup_url = format!("{}/{}", config.backup_url, path.trim_start_matches('/'));
    info!(backup_url = %backup_url, "Retrying Bitcoin API request using backup provider");

    fetch_json(&client, &backup_url)
        .await
        .map_err(|e| format!(
            "Bitcoin API failover also failed. Primary: {}, Backup: {}, Error: {}",
            primary_url, backup_url, e
        ))
}

/// Performs a POST request to broadcast a raw Bitcoin transaction.
/// Tries primary URL first, then failover to backup.
/// Returns the transaction ID (txid) from the API response body on success.
pub async fn post_tx_with_failover(
    config: &BitcoinApiConfig,
    tx_hex: &str,
) -> Result<String, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))?;

    let primary_url = format!("{}/tx", config.primary_url);

    match broadcast_tx(&client, &primary_url, tx_hex).await {
        Ok(txid) => {
            return Ok(txid);
        }
        Err(e) => {
            warn!(
                primary_url = %primary_url,
                error = %e,
                "Bitcoin broadcast via primary API failed, failing over to backup"
            );
        }
    }

    // Failover to backup
    let backup_url = format!("{}/tx", config.backup_url);
    info!(backup_url = %backup_url, "Retrying Bitcoin broadcast using backup provider");

    broadcast_tx(&client, &backup_url, tx_hex)
        .await
        .map_err(|e| format!(
            "Bitcoin broadcast failover also failed. Primary: {}, Backup: {}, Error: {}",
            primary_url, backup_url, e
        ))
}

/// Internal helper: perform a GET and parse JSON response.
async fn fetch_json(client: &reqwest::Client, url: &str) -> Result<serde_json::Value, String> {
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    let status = response.status();
    if status.is_server_error() {
        let body = response.text().await.unwrap_or_default();
        return Err(format!("Server error ({}): {}", status, body));
    }

    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(format!("HTTP error ({}): {}", status, body));
    }

    response
        .json::<serde_json::Value>()
        .await
        .map_err(|e| format!("Failed to parse JSON response: {}", e))
}

/// Internal helper: POST raw transaction hex and return response text (txid).
async fn broadcast_tx(client: &reqwest::Client, url: &str, tx_hex: &str) -> Result<String, String> {
    let response = client
        .post(url)
        .header("Content-Type", "text/plain")
        .body(tx_hex.to_string())
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    let status = response.status();
    let body = response.text().await.unwrap_or_default();

    if status.is_server_error() {
        return Err(format!("Server error ({}): {}", status, body));
    }

    if !status.is_success() {
        return Err(format!("Broadcast failed ({}): {}", status, body));
    }

    // Esplorer-compatible APIs return the raw txid as text
    Ok(body.trim().to_string())
}
