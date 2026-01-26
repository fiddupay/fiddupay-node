// Payment Monitoring Service for Address-Only Mode
// Monitors deposit addresses for incoming payments and triggers auto-forwarding

use crate::error::ServiceError;
use crate::services::address_only_service::{AddressOnlyService, AddressOnlyStatus};
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::collections::HashMap;
use tokio::time::{interval, Duration};

pub struct PaymentMonitorService {
    db_pool: PgPool,
    address_service: AddressOnlyService,
    config: crate::config::Config,
}

impl PaymentMonitorService {
    pub fn new(
        db_pool: PgPool,
        address_service: AddressOnlyService,
        config: crate::config::Config,
    ) -> Self {
        Self {
            db_pool,
            address_service,
            config,
        }
    }

    /// Start monitoring all pending payments
    pub async fn start_monitoring(&self) -> Result<(), ServiceError> {
        let mut interval = interval(Duration::from_secs(30)); // Check every 30 seconds

        loop {
            interval.tick().await;
            
            if let Err(e) = self.check_pending_payments().await {
                tracing::error!("Payment monitoring error: {}", e);
            }
        }
    }

    /// Check all pending payments for incoming funds
    async fn check_pending_payments(&self) -> Result<(), ServiceError> {
        let pending_payments = sqlx::query!(
            r#"
            SELECT payment_id, crypto_type, gateway_deposit_address, requested_amount
            FROM address_only_payments 
            WHERE status = 'PendingPayment' AND created_at > NOW() - INTERVAL '24 hours'
            "#
        )
        .fetch_all(&self.db_pool)
        .await?;

        for payment in pending_payments {
            if let Err(e) = self.check_payment_received(&payment.payment_id, &payment.crypto_type, &payment.gateway_deposit_address, payment.requested_amount).await {
                tracing::error!("Error checking payment {}: {}", payment.payment_id, e);
            }
        }

        Ok(())
    }

    /// Check if payment has been received for specific address
    async fn check_payment_received(
        &self,
        payment_id: &str,
        crypto_type: &str,
        address: &str,
        expected_amount: Decimal,
    ) -> Result<(), ServiceError> {
        let balance = self.get_address_balance(crypto_type, address).await?;
        
        if balance >= expected_amount {
            tracing::info!("Payment received for {}: {} {}", payment_id, balance, crypto_type);
            
            // Simulate transaction hash (would be from blockchain query)
            let tx_hash = format!("received_tx_{}", uuid::Uuid::new_v4());
            
            // Process the received payment
            self.address_service
                .process_received_payment(payment_id, balance, &tx_hash)
                .await?;
        }

        Ok(())
    }

    /// Get balance for specific address (simplified implementation)
    async fn get_address_balance(&self, crypto_type: &str, address: &str) -> Result<Decimal, ServiceError> {
        match crypto_type {
            "ETH" => self.get_eth_balance(address).await,
            "BNB" => self.get_bnb_balance(address).await,
            "MATIC" => self.get_matic_balance(address).await,
            "ARB" => self.get_arb_balance(address).await,
            "SOL" => self.get_sol_balance(address).await,
            _ => Err(ServiceError::ValidationError("Unsupported crypto type".to_string())),
        }
    }

    /// Get Ethereum balance
    async fn get_eth_balance(&self, address: &str) -> Result<Decimal, ServiceError> {
        let rpc_payload = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "eth_getBalance",
            "params": [address, "latest"],
            "id": 1
        });

        let client = reqwest::Client::new();
        let response: serde_json::Value = client
            .post(&self.config.ethereum_rpc_url)
            .json(&rpc_payload)
            .send()
            .await
            .map_err(|e| ServiceError::Internal(format!("ETH RPC error: {}", e)))?
            .json()
            .await
            .map_err(|e| ServiceError::Internal(format!("ETH RPC parse error: {}", e)))?;

        if let Some(result) = response.get("result").and_then(|v| v.as_str()) {
            let balance_wei = u128::from_str_radix(&result[2..], 16)
                .map_err(|_| ServiceError::Internal("Invalid balance hex".to_string()))?;
            
            // Convert wei to ETH
            Ok(Decimal::new(balance_wei as i64, 18))
        } else {
            Ok(Decimal::ZERO)
        }
    }

    /// Get BNB balance
    async fn get_bnb_balance(&self, address: &str) -> Result<Decimal, ServiceError> {
        // Similar to ETH but using BSC RPC
        let rpc_payload = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "eth_getBalance",
            "params": [address, "latest"],
            "id": 1
        });

        let client = reqwest::Client::new();
        let response: serde_json::Value = client
            .post(&self.config.bsc_rpc_url)
            .json(&rpc_payload)
            .send()
            .await
            .map_err(|e| ServiceError::Internal(format!("BNB RPC error: {}", e)))?
            .json()
            .await
            .map_err(|e| ServiceError::Internal(format!("BNB RPC parse error: {}", e)))?;

        if let Some(result) = response.get("result").and_then(|v| v.as_str()) {
            let balance_wei = u128::from_str_radix(&result[2..], 16)
                .map_err(|_| ServiceError::Internal("Invalid balance hex".to_string()))?;
            
            Ok(Decimal::new(balance_wei as i64, 18))
        } else {
            Ok(Decimal::ZERO)
        }
    }

    /// Get MATIC balance
    async fn get_matic_balance(&self, address: &str) -> Result<Decimal, ServiceError> {
        // Similar implementation for Polygon
        Ok(Decimal::ZERO) // Simplified for now
    }

    /// Get ARB balance
    async fn get_arb_balance(&self, address: &str) -> Result<Decimal, ServiceError> {
        // Similar implementation for Arbitrum
        Ok(Decimal::ZERO) // Simplified for now
    }

    /// Get SOL balance
    async fn get_sol_balance(&self, address: &str) -> Result<Decimal, ServiceError> {
        let rpc_payload = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "getBalance",
            "params": [address],
            "id": 1
        });

        let client = reqwest::Client::new();
        let response: serde_json::Value = client
            .post(&self.config.solana_rpc_url)
            .json(&rpc_payload)
            .send()
            .await
            .map_err(|e| ServiceError::Internal(format!("SOL RPC error: {}", e)))?
            .json()
            .await
            .map_err(|e| ServiceError::Internal(format!("SOL RPC parse error: {}", e)))?;

        if let Some(result) = response.get("result").and_then(|v| v.get("value")).and_then(|v| v.as_u64()) {
            // Convert lamports to SOL
            Ok(Decimal::new(result as i64, 9))
        } else {
            Ok(Decimal::ZERO)
        }
    }
}
