use crate::error::ServiceError;
use crate::payment::models::CryptoType;
use crate::services::blockchain_transaction_sender::BlockchainTransactionSender;
use crate::services::notification_service::NotificationService;
use crate::services::price_service::PriceService;
use crate::services::webhook_notification_service::WebhookNotificationService;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::{PgPool, Row};
use std::str::FromStr;
use std::sync::Arc;
use tracing::info;

use crate::services::balance_service::BalanceService;

pub struct BalanceMonitor {
    db_pool: PgPool,
    blockchain_sender: Arc<BlockchainTransactionSender>,
    price_service: Arc<PriceService>,
    notification_service: Arc<NotificationService>,
    webhook_service: Arc<WebhookNotificationService>,
    balance_service: Arc<BalanceService>,
}

pub struct BalanceCheckParams<'a> {
    pub merchant_id: i64,
    pub business_name: &'a str,
    pub wallet_id: i64,
    pub crypto_type: &'a CryptoType,
    pub network: &'a str,
    pub address: &'a str,
    pub threshold_usd: Decimal,
    pub is_live: bool,
}

impl BalanceMonitor {
    pub fn new(
        db_pool: PgPool,
        blockchain_sender: Arc<BlockchainTransactionSender>,
        price_service: Arc<PriceService>,
        notification_service: Arc<NotificationService>,
        webhook_service: Arc<WebhookNotificationService>,
        balance_service: Arc<BalanceService>,
    ) -> Self {
        Self {
            db_pool,
            blockchain_sender,
            price_service,
            notification_service,
            webhook_service,
            balance_service,
        }
    }

    /// On-demand check for a single merchant (called on login/dashboard)
    /// Only performs actual RPC calls if not checked in the last hour
    pub async fn check_merchant_on_demand(
        &self,
        merchant_id: i64,
        is_live: bool,
    ) -> Result<(), ServiceError> {
        // 1. Force a ledger recalculation from transactions & withdrawals & refunds
        // This ensures the dashboard always shows fresh data upon login/view.
        let _ = self
            .balance_service
            .refresh_balances_from_blockchain(merchant_id, !is_live)
            .await;

        let merchant = sqlx::query(
            "SELECT id, business_name, low_balance_threshold_usd, low_balance_alerts_enabled FROM merchants WHERE id = $1 AND is_active = true"
        )
        .bind(merchant_id)
        .fetch_optional(&self.db_pool)
        .await
        .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        if let Some(row) = merchant {
            let alerts_enabled: bool = row.get("low_balance_alerts_enabled");
            if !alerts_enabled {
                return Ok(());
            }

            let threshold: Decimal = row.get("low_balance_threshold_usd");
            if threshold.is_zero() {
                return Ok(());
            }

            let business_name: String = row.get("business_name");
            self.process_merchant_wallets(merchant_id, &business_name, threshold, is_live)
                .await?;
        }

        Ok(())
    }

    async fn process_merchant_wallets(
        &self,
        merchant_id: i64,
        business_name: &str,
        threshold_usd: Decimal,
        is_live: bool,
    ) -> Result<(), ServiceError> {
        let wallets = sqlx::query(
            r#"
            SELECT id, crypto_type, network, address, last_low_balance_alert_at 
            FROM merchant_wallets 
            WHERE merchant_id = $1 AND is_active = true
            "#,
        )
        .bind(merchant_id)
        .fetch_all(&self.db_pool)
        .await
        .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        for wallet_row in wallets {
            let wallet_id: i64 = wallet_row.get("id");
            let crypto_str: String = wallet_row.get("crypto_type");
            let network: String = wallet_row.get("network");
            let address: String = wallet_row.get("address");
            let last_alert: Option<DateTime<Utc>> = wallet_row.get("last_low_balance_alert_at");

            // Cooldown logic
            if let Some(last) = last_alert {
                let cooldown_hours = 12; // Standardized 12-hour cooldown for balance alerts
                if Utc::now().signed_duration_since(last).num_hours() < cooldown_hours {
                    continue;
                }
            }

            if let Ok(crypto_type) = CryptoType::from_str(&crypto_str) {
                if !self.is_native_gas_currency(&crypto_type) {
                    continue;
                }

                let params = BalanceCheckParams {
                    merchant_id,
                    business_name,
                    wallet_id,
                    crypto_type: &crypto_type,
                    network: &network,
                    address: &address,
                    threshold_usd,
                    is_live,
                };
                let _ = self.check_wallet_balance(params).await;
            }
        }
        Ok(())
    }

    async fn check_wallet_balance(
        &self,
        params: BalanceCheckParams<'_>,
    ) -> Result<(), ServiceError> {
        // [Balance check logic remains the same as previously implemented]
        // [Including U256 conversion, USD price fetching, and notification triggers]

        let balance_u256 = self
            .blockchain_sender
            .get_native_balance(*params.crypto_type, params.address, !params.is_live)
            .await?;

        let divisor = match params.crypto_type {
            CryptoType::Sol => 1_000_000_000.0,
            _ => 1_000_000_000_000_000_000.0,
        };

        let balance_u128: u128 = balance_u256.try_into().unwrap_or(0);
        let balance_decimal =
            Decimal::from_f64_retain(balance_u128 as f64 / divisor).unwrap_or(Decimal::ZERO);

        let price = self
            .price_service
            .get_price(*params.crypto_type)
            .await
            .map(|p| Decimal::from_f64_retain(p).unwrap_or(Decimal::ZERO))
            .unwrap_or(Decimal::ZERO);

        let balance_usd = balance_decimal * price;

        if balance_usd < params.threshold_usd {
            info!(
                "LOW BALANCE: {} ({}) has {} {} (~${:.2}) on {}.",
                params.business_name,
                params.merchant_id,
                balance_decimal,
                params.crypto_type,
                balance_usd,
                params.network
            );

            let title = format!("Low Balance Alert: {}", params.crypto_type);
            let message = format!(
                "Your {} wallet ({}) balance is low: {} {} (~${:.2} USD). Please top up to pay for fees.",
                params.network,
                params.address,
                crate::utils::format::format_crypto_amount(balance_decimal),
                params.crypto_type,
                balance_usd
            );

            let _ = self
                .notification_service
                .create_notification(
                    params.merchant_id,
                    &title,
                    &message,
                    "warning",
                    "balance.low_onchain",
                    !params.is_live,
                )
                .await;

            // Trigger external webhook alert
            let webhook_details = serde_json::json!({
                "alert_type": "LOW_BALANCE_USD_ONCHAIN",
                "crypto_type": params.crypto_type.to_string(),
                "current_balance": balance_decimal,
                "current_balance_usd": balance_usd,
                "threshold_usd": params.threshold_usd,
                "address": params.address,
                "network": params.network,
                "timestamp": Utc::now().to_rfc3339()
            });

            let _ = self
                .webhook_service
                .send_balance_alert_webhook(params.merchant_id, "balance.low", webhook_details)
                .await;

            // Update last_alert time
            sqlx::query("UPDATE merchant_wallets SET last_low_balance_alert_at = $1 WHERE id = $2")
                .bind(Utc::now())
                .bind(params.wallet_id)
                .execute(&self.db_pool)
                .await
                .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
        }

        Ok(())
    }

    fn is_native_gas_currency(&self, crypto: &CryptoType) -> bool {
        matches!(
            crypto,
            CryptoType::Eth
                | CryptoType::Bnb
                | CryptoType::Matic
                | CryptoType::Arb
                | CryptoType::Sol
        )
    }
}
