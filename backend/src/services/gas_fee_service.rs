// Gas Fee Service
// Fetches real-time gas fees using proper RPC methods (2026)

use crate::error::ServiceError;
use crate::payment::models::CryptoType;
use reqwest::Client;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

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

#[derive(Clone)]
pub struct GasFeeService {
    client: Client,
    config: crate::config::Config,
}

impl GasFeeService {
    pub fn new(config: crate::config::Config) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }

    /// Get real-time gas fees for all supported networks
    pub async fn get_all_gas_estimates(&self) -> Result<HashMap<String, GasFeeEstimate>, ServiceError> {
        let mut estimates = HashMap::new();

        // Fetch gas fees for each network using proper RPC methods
        estimates.insert("ethereum".to_string(), self.get_ethereum_gas_rpc().await?);
        estimates.insert("bsc".to_string(), self.get_bsc_gas_rpc().await?);
        estimates.insert("polygon".to_string(), self.get_polygon_gas_rpc().await?);
        estimates.insert("arbitrum".to_string(), self.get_arbitrum_gas_rpc().await?);
        estimates.insert("solana".to_string(), self.get_solana_gas_rpc().await?);

        Ok(estimates)
    }

    /// Get gas estimate for specific crypto type
    pub async fn get_gas_estimate(&self, crypto_type: CryptoType) -> Result<GasFeeEstimate, ServiceError> {
        match crypto_type {
            CryptoType::Eth | CryptoType::UsdtEth => self.get_ethereum_gas_rpc().await,
            CryptoType::Bnb | CryptoType::UsdtBep20 => self.get_bsc_gas_rpc().await,
            CryptoType::Matic | CryptoType::UsdtPolygon => self.get_polygon_gas_rpc().await,
            CryptoType::Arb | CryptoType::UsdtArbitrum => self.get_arbitrum_gas_rpc().await,
            CryptoType::Sol | CryptoType::UsdtSpl => self.get_solana_gas_rpc().await,
        }
    }

    /// Ethereum gas fees using eth_feeHistory RPC method (EIP-1559) - 2026 method
    async fn get_ethereum_gas_rpc(&self) -> Result<GasFeeEstimate, ServiceError> {
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

        let response: Value = self.client
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
            Err(ServiceError::Internal("Invalid ETH RPC response".to_string()))
        }
    }

    /// BSC gas fees using eth_gasPrice RPC method
    async fn get_bsc_gas_rpc(&self) -> Result<GasFeeEstimate, ServiceError> {
        let rpc_payload = json!({
            "jsonrpc": "2.0",
            "method": "eth_gasPrice",
            "params": [],
            "id": 1
        });

        let response: Value = self.client
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
            Err(ServiceError::Internal("Invalid BSC RPC response".to_string()))
        }
    }

    /// Polygon gas fees using eth_feeHistory RPC method
    async fn get_polygon_gas_rpc(&self) -> Result<GasFeeEstimate, ServiceError> {
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

        let response: Value = self.client
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
            Err(ServiceError::Internal("Invalid Polygon RPC response".to_string()))
        }
    }

    /// Arbitrum gas fees using eth_gasPrice RPC method
    async fn get_arbitrum_gas_rpc(&self) -> Result<GasFeeEstimate, ServiceError> {
        let rpc_payload = json!({
            "jsonrpc": "2.0",
            "method": "eth_gasPrice",
            "params": [],
            "id": 1
        });

        let response: Value = self.client
            .post(&self.config.arbitrum_rpc_url)
            .json(&rpc_payload)
            .send()
            .await
            .map_err(|e| ServiceError::Internal(format!("Arbitrum RPC error: {}", e)))?
            .json()
            .await
            .map_err(|e| ServiceError::Internal(format!("Arbitrum RPC parse error: {}", e)))?;

        if let Some(result) = response.get("result").and_then(|v| v.as_str()) {
            let gas_price_wei = u64::from_str_radix(&result[2..], 16)
                .map_err(|_| ServiceError::Internal("Invalid Arbitrum gas price hex".to_string()))?;

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
            Err(ServiceError::Internal("Invalid Arbitrum RPC response".to_string()))
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

        let response: Value = self.client
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
            Err(ServiceError::Internal("Invalid Solana RPC response".to_string()))
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
            CryptoType::Eth | CryptoType::Bnb | CryptoType::Matic | CryptoType::Arb | CryptoType::Sol => {
                Ok(native_balance >= withdrawal_amount + gas_estimate.estimated_withdrawal_cost)
            }
            // USDT variants: need separate gas deposit
            CryptoType::UsdtEth | CryptoType::UsdtBep20 | CryptoType::UsdtPolygon | CryptoType::UsdtArbitrum | CryptoType::UsdtSpl => {
                Ok(native_balance >= gas_estimate.estimated_withdrawal_cost)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use std::env;

    fn get_test_config() -> Config {
        // Set test RPC URLs - working 2026 endpoints
        env::set_var("DATABASE_URL", "postgresql://test:test@localhost:5432/test");
        env::set_var("REDIS_URL", "redis://localhost:6379");
        env::set_var("ENCRYPTION_KEY", "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef");
        env::set_var("JWT_SECRET", "test-secret");
        
        // Working 2026 RPC endpoints
        env::set_var("ETHEREUM_RPC_URL", "https://eth.llamarpc.com");
        env::set_var("BSC_RPC_URL", "https://bsc-dataseed.binance.org");
        env::set_var("POLYGON_RPC_URL", "https://polygon-rpc.com");
        env::set_var("ARBITRUM_RPC_URL", "https://arb1.arbitrum.io/rpc");
        env::set_var("SOLANA_RPC_URL", "https://api.mainnet-beta.solana.com");
        
        Config::from_env().expect("Failed to create test config")
    }

    #[tokio::test]
    async fn test_working_rpc_endpoints_2026() {
        let config = get_test_config();
        let service = GasFeeService::new(config);
        
        // Test all networks
        let networks = vec![
            (CryptoType::Eth, "Ethereum"),
            (CryptoType::Bnb, "BSC"),
            (CryptoType::Matic, "Polygon"),
            (CryptoType::Arb, "Arbitrum"),
            (CryptoType::Sol, "Solana"),
        ];
        
        for (crypto_type, name) in networks {
            let result = service.get_gas_estimate(crypto_type).await;
            assert!(result.is_ok(), "{} RPC failed: {:?}", name, result.err());
            
            let estimate = result.unwrap();
            assert!(estimate.standard_fee > Decimal::ZERO, "{} returned zero fee", name);
            println!(" {}: {} {}", name, estimate.standard_fee, estimate.native_currency);
        }
    }
}
