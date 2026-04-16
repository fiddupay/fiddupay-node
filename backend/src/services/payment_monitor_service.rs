// Payment Monitoring Service for Address-Only Mode
// Monitors deposit addresses for incoming payments and triggers auto-forwarding

use crate::error::ServiceError;
use crate::services::address_only_service::AddressOnlyService;
use rust_decimal::Decimal;
use sqlx::{PgPool, Row};

use alloy::{
    primitives::Address,
    providers::{Provider, ProviderBuilder},
};
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
        let mut interval = interval(Duration::from_secs(30));

        loop {
            interval.tick().await;

            if let Err(e) = self.check_pending_payments().await {
                tracing::error!("Payment monitoring error: {}", e);
            }
        }
    }

    /// Check all pending payments for incoming funds
    async fn check_pending_payments(&self) -> Result<(), ServiceError> {
        let pending_payments = sqlx::query(
            r#"
            SELECT payment_id, crypto_type, gateway_deposit_address, requested_amount
            FROM address_only_payments 
            WHERE status = 'PendingPayment' AND created_at > NOW() - INTERVAL '24 hours'
            "#,
        )
        .fetch_all(&self.db_pool)
        .await?;

        for payment in pending_payments {
            let payment_id: String = payment.get("payment_id");
            let crypto_type: String = payment.get("crypto_type");
            let gateway_deposit_address: String = payment.get("gateway_deposit_address");
            let requested_amount: Decimal = payment.get("requested_amount");
            if let Err(e) = self
                .check_payment_received(
                    &payment_id,
                    &crypto_type,
                    &gateway_deposit_address,
                    requested_amount,
                )
                .await
            {
                tracing::error!("Error checking payment {}: {}", payment_id, e);
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
            tracing::info!(
                "Payment received for {}: {} {}",
                payment_id,
                balance,
                crypto_type
            );

            let tx_hash = format!("received_tx_{}", uuid::Uuid::new_v4());

            self.address_service
                .process_received_payment(payment_id, balance, &tx_hash)
                .await?;
        } else if balance > Decimal::ZERO {
            tracing::info!(
                "Partial payment detected for {}: {}/{} {}",
                payment_id,
                balance,
                expected_amount,
                crypto_type
            );

            let tx_hash = format!("partial_tx_{}", uuid::Uuid::new_v4());

            self.address_service
                .process_partial_payment(payment_id, balance, &tx_hash)
                .await?;
        }

        Ok(())
    }

    /// Get balance for specific address (simplified implementation)
    async fn get_address_balance(
        &self,
        crypto_type: &str,
        address: &str,
    ) -> Result<Decimal, ServiceError> {
        match crypto_type {
            "ETH" => self.get_eth_balance(address).await,
            "BNB" => self.get_bnb_balance(address).await,
            "MATIC" => self.get_matic_balance(address).await,
            "ARB" => self.get_arb_balance(address).await,
            "SOL" => self.get_sol_balance(address).await,
            "BTC" => self.get_btc_balance(address).await,
            _ => Err(ServiceError::ValidationError(format!(
                "Unsupported crypto type: {}",
                crypto_type
            ))),
        }
    }

    async fn get_eth_balance(&self, address: &str) -> Result<Decimal, ServiceError> {
        let provider = ProviderBuilder::new().on_http(
            self.config
                .ethereum_rpc_url
                .parse()
                .map_err(|e| ServiceError::Internal(format!("Invalid RPC URL: {}", e)))?,
        );

        let addr: Address = address
            .parse()
            .map_err(|_| ServiceError::ValidationError("Invalid address".to_string()))?;

        let balance = provider
            .get_balance(addr)
            .await
            .map_err(|e| ServiceError::Internal(format!("ETH RPC error: {}", e)))?;

        let balance_wei: u128 = balance.try_into().unwrap_or(0);
        let mut dec = Decimal::from_i128_with_scale(balance_wei as i128, 18);
        dec.rescale(18);
        Ok(dec)
    }

    async fn get_bnb_balance(&self, address: &str) -> Result<Decimal, ServiceError> {
        let provider = ProviderBuilder::new().on_http(
            self.config
                .bsc_rpc_url
                .parse()
                .map_err(|e| ServiceError::Internal(format!("Invalid RPC URL: {}", e)))?,
        );

        let addr: Address = address
            .parse()
            .map_err(|_| ServiceError::ValidationError("Invalid address".to_string()))?;

        let balance = provider
            .get_balance(addr)
            .await
            .map_err(|e| ServiceError::Internal(format!("BNB RPC error: {}", e)))?;

        let balance_wei: u128 = balance.try_into().unwrap_or(0);
        let mut dec = Decimal::from_i128_with_scale(balance_wei as i128, 18);
        dec.rescale(18);
        Ok(dec)
    }

    async fn get_matic_balance(&self, address: &str) -> Result<Decimal, ServiceError> {
        let provider = ProviderBuilder::new().on_http(
            self.config
                .polygon_rpc_url
                .parse()
                .map_err(|e| ServiceError::Internal(format!("Invalid RPC URL: {}", e)))?,
        );

        let addr: Address = address
            .parse()
            .map_err(|_| ServiceError::ValidationError("Invalid address".to_string()))?;

        let balance = provider
            .get_balance(addr)
            .await
            .map_err(|e| ServiceError::Internal(format!("Polygon RPC error: {}", e)))?;

        let balance_wei: u128 = balance.try_into().unwrap_or(0);
        let mut dec = Decimal::from_i128_with_scale(balance_wei as i128, 18);
        dec.rescale(18);
        Ok(dec)
    }

    async fn get_arb_balance(&self, address: &str) -> Result<Decimal, ServiceError> {
        let provider = ProviderBuilder::new().on_http(
            self.config
                .arbitrum_rpc_url
                .parse()
                .map_err(|e| ServiceError::Internal(format!("Invalid RPC URL: {}", e)))?,
        );

        let addr: Address = address
            .parse()
            .map_err(|_| ServiceError::ValidationError("Invalid address".to_string()))?;

        let balance = provider
            .get_balance(addr)
            .await
            .map_err(|e| ServiceError::Internal(format!("Arbitrum RPC error: {}", e)))?;

        let balance_wei: u128 = balance.try_into().unwrap_or(0);
        let mut dec = Decimal::from_i128_with_scale(balance_wei as i128, 18);
        dec.rescale(18);
        Ok(dec)
    }

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

        if let Some(result) = response
            .get("result")
            .and_then(|v| v.get("value"))
            .and_then(|v| v.as_u64())
        {
            let mut dec = Decimal::from_i128_with_scale(result as i128, 9);
            dec.rescale(9);
            Ok(dec)
        } else {
            Ok(Decimal::ZERO)
        }
    }

    async fn get_btc_balance(&self, address: &str) -> Result<Decimal, ServiceError> {
        let is_sandbox = self.config.bitcoin_rpc_url.contains("testnet");
        let api_config =
            crate::utils::bitcoin_api::BitcoinApiConfig::from_config(&self.config, is_sandbox);

        let response = crate::utils::bitcoin_api::get_with_failover(
            &api_config,
            &format!("address/{}", address),
        )
        .await
        .map_err(|e| ServiceError::Internal(format!("BTC API error: {}", e)))?;

        let chain_stats = response.get("chain_stats");
        let funded_sum = chain_stats
            .and_then(|v| v.get("funded_txo_sum"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let spent_sum = chain_stats
            .and_then(|v| v.get("spent_txo_sum"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        let mempool_stats = response.get("mempool_stats");
        let mempool_funded = mempool_stats
            .and_then(|v| v.get("funded_txo_sum"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let mempool_spent = mempool_stats
            .and_then(|v| v.get("spent_txo_sum"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        let total_funded = funded_sum + mempool_funded;
        let total_spent = spent_sum + mempool_spent;

        if total_funded > total_spent {
            let satoshis = total_funded - total_spent;
            Ok(Decimal::new(satoshis as i64, 8))
        } else {
            Ok(Decimal::ZERO)
        }
    }
}
