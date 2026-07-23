use crate::error::ServiceError;
use crate::payment::models::CryptoType;
use crate::services::notification_service::NotificationService;
use crate::services::price_service::PriceService;
use crate::services::webhook_notification_service::WebhookNotificationService;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use std::str::FromStr;
use std::sync::Arc;
use tracing::{error, info};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceAlert {
    pub merchant_id: i64,
    pub alert_type: String,
    pub crypto_type: String,
    pub current_balance: Decimal,
    pub threshold: Decimal,
    pub created_at: DateTime<Utc>,
}

pub struct BalanceMonitoringService {
    db_pool: PgPool,
    notification_service: Arc<NotificationService>,
    price_service: Arc<PriceService>,
    webhook_notif: Arc<WebhookNotificationService>,
}

impl BalanceMonitoringService {
    pub fn new(
        db_pool: PgPool,
        notification_service: Arc<NotificationService>,
        price_service: Arc<PriceService>,
        webhook_notif: Arc<WebhookNotificationService>,
    ) -> Self {
        Self {
            db_pool,
            notification_service,
            price_service,
            webhook_notif,
        }
    }

    pub async fn check_low_balances(&self) -> Result<Vec<BalanceAlert>, ServiceError> {
        info!("Running USD-based low balance check for all merchants...");

        // Single JOIN query to fetch merchants and their balances in 1 query (Eliminated N+1 DB Queries!)
        let rows = sqlx::query(
            r#"
            SELECT 
                m.id as merchant_id, 
                m.business_name, 
                m.low_balance_threshold_usd, 
                m.last_low_balance_total_alert_at,
                mb.crypto_type,
                mb.available_balance as balance
            FROM merchants m
            LEFT JOIN merchant_balances mb ON mb.merchant_id = m.id
            WHERE m.low_balance_threshold_usd > 0 
              AND m.low_balance_alerts_enabled = true 
              AND m.is_active = true
            "#,
        )
        .fetch_all(&self.db_pool)
        .await?;

        use std::collections::HashMap;

        struct MerchantBalanceGroup {
            low_balance_threshold_usd: Option<Decimal>,
            last_alert: Option<DateTime<Utc>>,
            balances: Vec<(String, Decimal)>,
        }

        let mut grouped: HashMap<i64, MerchantBalanceGroup> = HashMap::new();

        for row in rows {
            let m_id: i64 = row.get("merchant_id");
            let threshold: Option<Decimal> = row.get("low_balance_threshold_usd");
            let last_alert: Option<DateTime<Utc>> = row.get("last_low_balance_total_alert_at");
            let crypto_opt: Option<String> = row.get("crypto_type");
            let balance_opt: Option<Decimal> = row.get("balance");

            let entry = grouped.entry(m_id).or_insert_with(|| MerchantBalanceGroup {
                low_balance_threshold_usd: threshold,
                last_alert,
                balances: Vec::new(),
            });

            if let (Some(c), Some(b)) = (crypto_opt, balance_opt) {
                entry.balances.push((c, b));
            }
        }

        let mut alerts = Vec::new();

        for (merchant_id, group) in grouped {
            // Cooldown logic (12 hours)
            if let Some(last) = group.last_alert {
                if Utc::now().signed_duration_since(last).num_hours() < 12 {
                    continue;
                }
            }

            let mut total_usd_value = Decimal::ZERO;

            for (crypto_type, balance) in group.balances {
                if let Ok(crypto) = CryptoType::from_str(&crypto_type) {
                    if let Ok(price) = self.price_service.get_price(crypto).await {
                        let crypto_price =
                            Decimal::from_str(&price.to_string()).unwrap_or(Decimal::ZERO);
                        total_usd_value += balance * crypto_price;
                    }
                }
            }

            let threshold = group.low_balance_threshold_usd.unwrap_or_default();
            if total_usd_value < threshold {
                let alert = BalanceAlert {
                    merchant_id,
                    alert_type: "LOW_BALANCE_USD".to_string(),
                    crypto_type: "USD_TOTAL".to_string(),
                    current_balance: total_usd_value,
                    threshold,
                    created_at: Utc::now(),
                };

                if let Err(e) = self.send_balance_alert(alert.clone()).await {
                    error!(
                        "Failed to send balance alert for merchant {}: {}",
                        merchant_id, e
                    );
                }
                alerts.push(alert);
            }
        }

        Ok(alerts)
    }

    pub async fn check_large_withdrawals(
        &self,
        _hours: i32,
    ) -> Result<Vec<BalanceAlert>, ServiceError> {
        // Simplified - return empty for now
        Ok(vec![])
    }

    pub async fn send_balance_alert(&self, alert: BalanceAlert) -> Result<(), ServiceError> {
        // Simplified - just log to audit_logs
        let details = serde_json::json!({
            "alert_type": alert.alert_type,
            "crypto_type": alert.crypto_type,
            "current_balance": alert.current_balance,
            "threshold": alert.threshold
        });

        sqlx::query(
            "INSERT INTO audit_logs (merchant_id, action_type, details) VALUES ($1, $2, $3)",
        )
        .bind(alert.merchant_id)
        .bind("BALANCE_ALERT")
        .bind(&details)
        .execute(&self.db_pool)
        .await?;

        // Update merchant last alert time
        sqlx::query("UPDATE merchants SET last_low_balance_total_alert_at = NOW() WHERE id = $1")
            .bind(alert.merchant_id)
            .execute(&self.db_pool)
            .await?;

        // Create in-app notification
        let _ = self
            .notification_service
            .create_notification(
                alert.merchant_id,
                &format!("Low Balance Alert: {}", alert.crypto_type),
                &format!(
                    "Your {} balance is {} (Threshold: {})",
                    alert.crypto_type,
                    crate::utils::format::format_crypto_amount(alert.current_balance),
                    crate::utils::format::format_crypto_amount(alert.threshold)
                ),
                "warning",
                "balance.low",
                false, // Default to production mode for system alerts, or implement mode tracking
            )
            .await;

        // Create webhook alert
        let details = serde_json::json!({
            "alert_type": alert.alert_type,
            "crypto_type": alert.crypto_type,
            "current_balance": alert.current_balance,
            "threshold": alert.threshold,
            "timestamp": alert.created_at.to_rfc3339()
        });

        let _ = self
            .webhook_notif
            .send_balance_alert_webhook(alert.merchant_id, "balance.low", details)
            .await;

        Ok(())
    }
}
