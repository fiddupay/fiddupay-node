// Gas Fee Service
// Fetches real-time gas fees using proper RPC methods (2026)

use crate::error::ServiceError;
use crate::payment::models::CryptoType;
use reqwest::Client;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GasFeeEstimate {
    pub network: String,
    pub native_currency: String,
    pub standard_fee: Decimal,
    pub fast_fee: Decimal,
    pub estimated_withdrawal_cost: Decimal,
    pub base_fee: Option<Decimal>,
    pub priority_fee: Option<Decimal>,
}

#[derive(Debug, Deserialize)]
struct InfuraGasFeeLevel {
    #[serde(rename = "suggestedMaxPriorityFeePerGas")]
    suggested_max_priority_fee_per_gas: String,
    #[serde(rename = "suggestedMaxFeePerGas")]
    suggested_max_fee_per_gas: String,
}

#[derive(Debug, Deserialize)]
struct InfuraGasApiResponse {
    low: InfuraGasFeeLevel,
    medium: InfuraGasFeeLevel,
    high: InfuraGasFeeLevel,
    #[serde(rename = "estimatedBaseFee")]
    estimated_base_fee: String,
}

#[derive(Clone)]
pub struct GasFeeService {
    client: Client,
    config: crate::config::Config,
}

impl GasFeeService {
    pub fn new(config: crate::config::Config) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(5))
                .build()
                .unwrap_or_default(),
            config,
        }
    }

    /// Helper for Infura Gas API
    async fn get_infura_gas_api(
        &self,
        chain_id: u64,
        network: &str,
        native_currency: &str,
    ) -> Result<GasFeeEstimate, ServiceError> {
        let keys = &self.config.infura_api_keys;
        if keys.is_empty() {
            return Err(ServiceError::Internal(
                "No Infura API keys configured".to_string(),
            ));
        }

        let mut last_err = None;
        for key in keys {
            let url = format!(
                "https://gas.api.infura.io/v3/{}/networks/{}/suggestedGasFees",
                key, chain_id
            );
            match self.client.get(&url).send().await {
                Ok(resp) if resp.status().is_success() => {
                    match resp.json::<InfuraGasApiResponse>().await {
                        Ok(data) => {
                            let parse_gwei_to_primary = |gwei_str: &str| -> Decimal {
                                let gwei_dec = Decimal::from_str(gwei_str).unwrap_or(Decimal::ZERO);
                                // Gwei to Native (e.g. ETH) means divide by 1_000_000_000 (10^9)
                                // Then multiply by 21,000 for standard gas limit.
                                let mut result = gwei_dec * Decimal::new(21000, 0);
                                result.set_scale(result.scale() + 9).unwrap_or(());
                                result
                            };

                            let base_fee_eth = parse_gwei_to_primary(&data.estimated_base_fee);
                            let standard_fee =
                                parse_gwei_to_primary(&data.medium.suggested_max_fee_per_gas);
                            let fast_fee =
                                parse_gwei_to_primary(&data.high.suggested_max_fee_per_gas);
                            let priority_fee = parse_gwei_to_primary(
                                &data.medium.suggested_max_priority_fee_per_gas,
                            );

                            return Ok(GasFeeEstimate {
                                network: network.to_string(),
                                native_currency: native_currency.to_string(),
                                standard_fee,
                                fast_fee,
                                estimated_withdrawal_cost: standard_fee,
                                base_fee: Some(base_fee_eth),
                                priority_fee: Some(priority_fee),
                            });
                        }
                        Err(e) => {
                            last_err = Some(e.to_string());
                        }
                    }
                }
                Ok(resp) => {
                    last_err = Some(format!(
                        "Infura Gas API HTTP {}, chain {}",
                        resp.status(),
                        chain_id
                    ));
                }
                Err(e) => {
                    last_err = Some(e.to_string());
                }
            }
        }

        Err(ServiceError::Internal(format!(
            "Infura Gas API failed: {:?}",
            last_err
        )))
    }

    /// Get real-time gas fees for all supported networks (Parallel Execution)
    pub async fn get_all_gas_estimates(
        &self,
    ) -> Result<HashMap<String, GasFeeEstimate>, ServiceError> {
        let (eth, bsc, poly, arb, sol) = tokio::try_join!(
            self.get_ethereum_gas_rpc(),
            self.get_bsc_gas_rpc(),
            self.get_polygon_gas_rpc(),
            self.get_arbitrum_gas_rpc(),
            self.get_solana_gas_rpc()
        )?;

        let mut estimates = HashMap::new();
        estimates.insert("ethereum".to_string(), eth);
        estimates.insert("bsc".to_string(), bsc);
        estimates.insert("polygon".to_string(), poly);
        estimates.insert("arbitrum".to_string(), arb);
        estimates.insert("solana".to_string(), sol);

        Ok(estimates)
    }

    /// Get gas estimate for specific crypto type
    pub async fn get_gas_estimate(
        &self,
        crypto_type: CryptoType,
    ) -> Result<GasFeeEstimate, ServiceError> {
        match crypto_type {
            CryptoType::Eth | CryptoType::UsdtEth => self.get_ethereum_gas_rpc().await,
            CryptoType::Bnb | CryptoType::UsdtBep20 | CryptoType::BusdBep20 => {
                self.get_bsc_gas_rpc().await
            }
            CryptoType::Matic | CryptoType::UsdtPolygon => self.get_polygon_gas_rpc().await,
            CryptoType::Arb | CryptoType::UsdtArbitrum => self.get_arbitrum_gas_rpc().await,
            CryptoType::Sol | CryptoType::UsdtSpl | CryptoType::WSol => {
                self.get_solana_gas_rpc().await
            }
            CryptoType::Btc => {
                // Return a flat estimate for BTC for now
                Ok(GasFeeEstimate {
                    network: "bitcoin".to_string(),
                    native_currency: "BTC".to_string(),
                    standard_fee: Decimal::new(1000, 8), // 0.00001 BTC
                    fast_fee: Decimal::new(2000, 8),
                    estimated_withdrawal_cost: Decimal::new(1000, 8),
                    base_fee: None,
                    priority_fee: None,
                })
            }
        }
    }

    /// Ethereum gas fees using eth_feeHistory RPC method (EIP-1559) - 2026 method
    async fn get_ethereum_gas_rpc(&self) -> Result<GasFeeEstimate, ServiceError> {
        if let Ok(estimate) = self.get_infura_gas_api(1, "ethereum", "ETH").await {
            return Ok(estimate);
        }

        let rpc_payload = json!({
            "jsonrpc": "2.0",
            "method": "eth_feeHistory",
            "params": [
                "0x4", // 4 blocks
                "latest",
                [10.0, 25.0, 50.0] // 10th, 25th, 50th percentiles
            ],
            "id": 1
        });

        let response: Value = self
            .client
            .post(&self.config.ethereum_rpc_url)
            .json(&rpc_payload)
            .send()
            .await
            .map_err(|e| ServiceError::Internal(format!("ETH RPC error: {}", e)))?
            .json()
            .await
            .map_err(|e| ServiceError::Internal(format!("ETH RPC parse error: {}", e)))?;

        if let Some(result) = response.get("result") {
            let base_fee_per_gas = result["baseFeePerGas"]
                .as_array()
                .and_then(|arr| arr.last())
                .and_then(|v| v.as_str())
                .ok_or_else(|| ServiceError::Internal("Invalid base fee format".to_string()))?;

            let reward = result["reward"]
                .as_array()
                .and_then(|arr| arr.last())
                .and_then(|v| v.as_array())
                .and_then(|arr| arr.get(1)) // 25th percentile
                .and_then(|v| v.as_str())
                .ok_or_else(|| ServiceError::Internal("Invalid reward format".to_string()))?;

            // Convert hex to decimal (wei to ETH)
            let base_fee_wei = u64::from_str_radix(&base_fee_per_gas[2..], 16)
                .map_err(|_| ServiceError::Internal("Invalid base fee hex".to_string()))?;
            let priority_fee_wei = u64::from_str_radix(&reward[2..], 16)
                .map_err(|_| ServiceError::Internal("Invalid priority fee hex".to_string()))?;

            let gas_limit = 21000u64; // Standard ETH transfer
            let base_fee_eth = Decimal::new(base_fee_wei as i64 * gas_limit as i64, 18);
            let priority_fee_eth = Decimal::new(priority_fee_wei as i64 * gas_limit as i64, 18);
            let total_fee = base_fee_eth + priority_fee_eth;

            Ok(GasFeeEstimate {
                network: "ethereum".to_string(),
                native_currency: "ETH".to_string(),
                standard_fee: total_fee,
                fast_fee: total_fee * Decimal::new(15, 1), // 1.5x for fast
                estimated_withdrawal_cost: total_fee,
                base_fee: Some(base_fee_eth),
                priority_fee: Some(priority_fee_eth),
            })
        } else {
            Err(ServiceError::Internal(
                "Invalid ETH RPC response".to_string(),
            ))
        }
    }

    /// BSC gas fees using eth_gasPrice RPC method
    async fn get_bsc_gas_rpc(&self) -> Result<GasFeeEstimate, ServiceError> {
        if let Ok(estimate) = self.get_infura_gas_api(56, "bsc", "BNB").await {
            return Ok(estimate);
        }

        let rpc_payload = json!({
            "jsonrpc": "2.0",
            "method": "eth_gasPrice",
            "params": [],
            "id": 1
        });

        let response: Value = self
            .client
            .post(&self.config.bsc_rpc_url)
            .json(&rpc_payload)
            .send()
            .await
            .map_err(|e| ServiceError::Internal(format!("BSC RPC error: {}", e)))?
            .json()
            .await
            .map_err(|e| ServiceError::Internal(format!("BSC RPC parse error: {}", e)))?;

        if let Some(result) = response.get("result").and_then(|v| v.as_str()) {
            let gas_price_wei = u64::from_str_radix(&result[2..], 16)
                .map_err(|_| ServiceError::Internal("Invalid BSC gas price hex".to_string()))?;

            let gas_limit = 21000u64;
            let gas_fee_bnb = Decimal::new(gas_price_wei as i64 * gas_limit as i64, 18);

            Ok(GasFeeEstimate {
                network: "bsc".to_string(),
                native_currency: "BNB".to_string(),
                standard_fee: gas_fee_bnb,
                fast_fee: gas_fee_bnb * Decimal::new(12, 1), // 1.2x for fast
                estimated_withdrawal_cost: gas_fee_bnb,
                base_fee: None,
                priority_fee: None,
            })
        } else {
            Err(ServiceError::Internal(
                "Invalid BSC RPC response".to_string(),
            ))
        }
    }

    /// Polygon gas fees using eth_feeHistory RPC method
    async fn get_polygon_gas_rpc(&self) -> Result<GasFeeEstimate, ServiceError> {
        if let Ok(estimate) = self.get_infura_gas_api(137, "polygon", "MATIC").await {
            return Ok(estimate);
        }

        let rpc_payload = json!({
            "jsonrpc": "2.0",
            "method": "eth_feeHistory",
            "params": [
                "0x4", // 4 blocks
                "latest",
                [10.0, 25.0, 50.0] // percentiles
            ],
            "id": 1
        });

        let response: Value = self
            .client
            .post(&self.config.polygon_rpc_url)
            .json(&rpc_payload)
            .send()
            .await
            .map_err(|e| ServiceError::Internal(format!("Polygon RPC error: {}", e)))?
            .json()
            .await
            .map_err(|e| ServiceError::Internal(format!("Polygon RPC parse error: {}", e)))?;

        if let Some(result) = response.get("result") {
            let base_fee_per_gas = result["baseFeePerGas"]
                .as_array()
                .and_then(|arr| arr.last())
                .and_then(|v| v.as_str())
                .unwrap_or("0x0");

            let reward = result["reward"]
                .as_array()
                .and_then(|arr| arr.last())
                .and_then(|v| v.as_array())
                .and_then(|arr| arr.get(1))
                .and_then(|v| v.as_str())
                .unwrap_or("0x0");

            let base_fee_wei = u64::from_str_radix(&base_fee_per_gas[2..], 16).unwrap_or(0);
            let priority_fee_wei = u64::from_str_radix(&reward[2..], 16).unwrap_or(0);

            let gas_limit = 21000u64;
            let base_fee_matic = Decimal::new(base_fee_wei as i64 * gas_limit as i64, 18);
            let priority_fee_matic = Decimal::new(priority_fee_wei as i64 * gas_limit as i64, 18);
            let total_fee = base_fee_matic + priority_fee_matic;

            Ok(GasFeeEstimate {
                network: "polygon".to_string(),
                native_currency: "MATIC".to_string(),
                standard_fee: total_fee,
                fast_fee: total_fee * Decimal::new(15, 1),
                estimated_withdrawal_cost: total_fee,
                base_fee: Some(base_fee_matic),
                priority_fee: Some(priority_fee_matic),
            })
        } else {
            Err(ServiceError::Internal(
                "Invalid Polygon RPC response".to_string(),
            ))
        }
    }

    /// Arbitrum gas fees using eth_gasPrice RPC method
    async fn get_arbitrum_gas_rpc(&self) -> Result<GasFeeEstimate, ServiceError> {
        if let Ok(estimate) = self.get_infura_gas_api(42161, "arbitrum", "ARB").await {
            return Ok(estimate);
        }

        let rpc_payload = json!({
            "jsonrpc": "2.0",
            "method": "eth_gasPrice",
            "params": [],
            "id": 1
        });

        let response: Value = self
            .client
            .post(&self.config.arbitrum_rpc_url)
            .json(&rpc_payload)
            .send()
            .await
            .map_err(|e| ServiceError::Internal(format!("Arbitrum RPC error: {}", e)))?
            .json()
            .await
            .map_err(|e| ServiceError::Internal(format!("Arbitrum RPC parse error: {}", e)))?;

        if let Some(result) = response.get("result").and_then(|v| v.as_str()) {
            let gas_price_wei = u64::from_str_radix(&result[2..], 16).map_err(|_| {
                ServiceError::Internal("Invalid Arbitrum gas price hex".to_string())
            })?;

            let gas_limit = 21000u64;
            let gas_fee_arb = Decimal::new(gas_price_wei as i64 * gas_limit as i64, 18);

            Ok(GasFeeEstimate {
                network: "arbitrum".to_string(),
                native_currency: "ARB".to_string(),
                standard_fee: gas_fee_arb,
                fast_fee: gas_fee_arb * Decimal::new(11, 1), // 1.1x for fast
                estimated_withdrawal_cost: gas_fee_arb,
                base_fee: None,
                priority_fee: None,
            })
        } else {
            Err(ServiceError::Internal(
                "Invalid Arbitrum RPC response".to_string(),
            ))
        }
    }

    /// Solana gas fees using getRecentPrioritizationFees RPC method - 2026 method
    async fn get_solana_gas_rpc(&self) -> Result<GasFeeEstimate, ServiceError> {
        let rpc_payload = json!({
            "jsonrpc": "2.0",
            "method": "getRecentPrioritizationFees",
            "params": [
                [] // Empty array for global fees, or specify account addresses for targeted fees
            ],
            "id": 1
        });

        let response: Value = self
            .client
            .post(&self.config.solana_rpc_url)
            .json(&rpc_payload)
            .send()
            .await
            .map_err(|e| ServiceError::Internal(format!("Solana RPC error: {}", e)))?
            .json()
            .await
            .map_err(|e| ServiceError::Internal(format!("Solana RPC parse error: {}", e)))?;

        if let Some(result) = response.get("result").and_then(|v| v.as_array()) {
            // Calculate median prioritization fee from recent blocks
            let mut fees: Vec<u64> = result
                .iter()
                .filter_map(|item| item.get("prioritizationFee").and_then(|v| v.as_u64()))
                .collect();

            fees.sort();
            let median_priority_fee = if fees.is_empty() {
                0
            } else {
                fees[fees.len() / 2]
            };

            // Base transaction fee is 5000 lamports per signature
            let base_fee_lamports = 5000u64;
            let total_fee_lamports = base_fee_lamports + median_priority_fee;

            // Convert lamports to SOL (1 SOL = 1,000,000,000 lamports)
            let total_fee_sol = Decimal::new(total_fee_lamports as i64, 9);
            let priority_fee_sol = Decimal::new(median_priority_fee as i64, 9);

            Ok(GasFeeEstimate {
                network: "solana".to_string(),
                native_currency: "SOL".to_string(),
                standard_fee: total_fee_sol,
                fast_fee: total_fee_sol + (priority_fee_sol * Decimal::new(2, 0)), // 2x priority for fast
                estimated_withdrawal_cost: total_fee_sol,
                base_fee: Some(Decimal::new(base_fee_lamports as i64, 9)),
                priority_fee: Some(priority_fee_sol),
            })
        } else {
            Err(ServiceError::Internal(
                "Invalid Solana RPC response".to_string(),
            ))
        }
    }

    /// Check if merchant has sufficient gas for withdrawal
    pub async fn validate_gas_sufficiency(
        &self,
        crypto_type: CryptoType,
        native_balance: Decimal,
        withdrawal_amount: Decimal,
    ) -> Result<bool, ServiceError> {
        let gas_estimate = self.get_gas_estimate(crypto_type).await?;

        match crypto_type {
            // Native currencies: deduct gas from withdrawal amount
            CryptoType::Eth
            | CryptoType::Bnb
            | CryptoType::Matic
            | CryptoType::Arb
            | CryptoType::Sol
            | CryptoType::Btc => {
                Ok(native_balance >= withdrawal_amount + gas_estimate.estimated_withdrawal_cost)
            }
            // USDT variants: need separate gas deposit
            CryptoType::UsdtEth
            | CryptoType::UsdtBep20
            | CryptoType::UsdtPolygon
            | CryptoType::UsdtArbitrum
            | CryptoType::UsdtSpl
            | CryptoType::WSol
            | CryptoType::BusdBep20 => Ok(native_balance >= gas_estimate.estimated_withdrawal_cost),
        }
    }
}
