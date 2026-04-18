// Balance Service - Tracks merchant balances across all networks

use crate::error::ServiceError;
use crate::payment::models::CryptoType;
use crate::services::price_service::PriceService;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct Balance {
    pub crypto_type: CryptoType,
    pub total_balance: Decimal,
    pub available_balance: Decimal,
    pub reserved_balance: Decimal,
    pub balance_usd: Decimal,
    pub available_usd: Decimal,
    pub reserved_usd: Decimal,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BalanceSummary {
    pub total_usd: Decimal,
    pub available_usd: Decimal,
    pub reserved_usd: Decimal,
    pub balances: Vec<Balance>,
}

pub struct BalanceService {
    db_pool: PgPool,
    price_service: Arc<PriceService>,
    redis_client: redis::Client,
}

impl BalanceService {
    pub fn new(
        db_pool: PgPool,
        price_service: Arc<PriceService>,
        redis_client: redis::Client,
    ) -> Self {
        Self {
            db_pool,
            price_service,
            redis_client,
        }
    }

    pub async fn get_balance(
        &self,
        merchant_id: i64,
        crypto_type: CryptoType,
        sandbox_mode: bool,
    ) -> Result<Balance, ServiceError> {
        let balance_record = sqlx::query(
            r#"
            SELECT 
                total_balance,
                available_balance,
                reserved_balance,
                last_updated
            FROM merchant_balances 
            WHERE merchant_id = $1 AND crypto_type = $2 AND sandbox_mode = $3
            "#,
        )
        .bind(merchant_id)
        .bind(crypto_type.to_string())
        .bind(sandbox_mode)
        .fetch_optional(&self.db_pool)
        .await?;

        let total = balance_record
            .as_ref()
            .and_then(|r| {
                r.try_get::<Option<Decimal>, _>("total_balance")
                    .ok()
                    .flatten()
            })
            .unwrap_or(Decimal::ZERO);
        let available = balance_record
            .as_ref()
            .and_then(|r| {
                r.try_get::<Option<Decimal>, _>("available_balance")
                    .ok()
                    .flatten()
            })
            .unwrap_or(Decimal::ZERO);
        let pending = balance_record
            .as_ref()
            .and_then(|r| {
                r.try_get::<Option<Decimal>, _>("reserved_balance")
                    .ok()
                    .flatten()
            })
            .unwrap_or(Decimal::ZERO);
        let last_updated = balance_record
            .as_ref()
            .and_then(|r| {
                r.try_get::<Option<DateTime<Utc>>, _>("last_updated")
                    .ok()
                    .flatten()
            })
            .unwrap_or_else(Utc::now);

        // Get current USD value
        let price: f64 = self
            .price_service
            .get_price(crypto_type)
            .await
            .unwrap_or(0.0);
        let price_decimal = Decimal::from_f64_retain(price).unwrap_or(Decimal::ZERO);

        let available_usd = available * price_decimal;
        let reserved_usd = pending * price_decimal;
        let balance_usd = available_usd;

        Ok(Balance {
            crypto_type,
            total_balance: total,
            available_balance: available,
            reserved_balance: pending,
            balance_usd,
            available_usd,
            reserved_usd,
            last_updated,
        })
    }

    pub async fn get_all_balances(
        &self,
        merchant_id: i64,
        sandbox_mode: bool,
    ) -> Result<BalanceSummary, ServiceError> {
        // 1. Fetch all balance records in a single batch query (O(1) database roundtrip)
        let rows = sqlx::query(
            r#"
            SELECT 
                crypto_type,
                total_balance,
                available_balance,
                reserved_balance,
                last_updated
            FROM merchant_balances 
            WHERE merchant_id = $1 AND sandbox_mode = $2
            "#,
        )
        .bind(merchant_id)
        .bind(sandbox_mode)
        .fetch_all(&self.db_pool)
        .await?;

        // 2. Map rows to crypto_type for easy access
        let mut balance_map: std::collections::HashMap<
            String,
            (Decimal, Decimal, Decimal, DateTime<Utc>),
        > = rows
            .into_iter()
            .map(|r| {
                let ct = r.get::<String, _>("crypto_type");
                let total = r
                    .get::<Option<Decimal>, _>("total_balance")
                    .unwrap_or(Decimal::ZERO);
                let available = r
                    .get::<Option<Decimal>, _>("available_balance")
                    .unwrap_or(Decimal::ZERO);
                let reserved = r
                    .get::<Option<Decimal>, _>("reserved_balance")
                    .unwrap_or(Decimal::ZERO);
                let updated = r.get::<DateTime<Utc>, _>("last_updated");
                (ct, (total, available, reserved, updated))
            })
            .collect();

        // 3. Define the list of supported types to return (consistent with original logic)
        let crypto_types = vec![
            CryptoType::Sol,
            CryptoType::WSol,
            CryptoType::UsdtSpl,
            CryptoType::Eth,
            CryptoType::UsdtEth,
            CryptoType::Bnb,
            CryptoType::UsdtBep20,
            CryptoType::Matic,
            CryptoType::UsdtPolygon,
            CryptoType::Arb,
            CryptoType::UsdtArbitrum,
        ];

        // 4. Fetch all prices in parallel once
        let mut tasks = Vec::new();
        for &ct in &crypto_types {
            let service = Arc::new(self.price_service.clone()); // Assuming price_service is Arc-wrapped enough
            tasks.push(async move {
                let price = service.get_price(ct).await.unwrap_or(0.0);
                (ct, price)
            });
        }
        let price_results = futures::future::join_all(tasks).await;
        let price_map: std::collections::HashMap<CryptoType, f64> =
            price_results.into_iter().collect();

        // 5. Build the summary
        let mut balances = Vec::new();
        let mut total_available_usd = Decimal::ZERO;
        let mut total_reserved_usd = Decimal::ZERO;

        for crypto_type in crypto_types {
            let ct_str = crypto_type.to_string();
            let (total_balance, available_balance, reserved_balance, last_updated) = balance_map
                .remove(&ct_str)
                .unwrap_or((Decimal::ZERO, Decimal::ZERO, Decimal::ZERO, Utc::now()));

            let price = price_map.get(&crypto_type).copied().unwrap_or(0.0);
            let price_dec = Decimal::from_f64_retain(price).unwrap_or(Decimal::ZERO);

            let available_usd = (available_balance * price_dec).round_dp(2);
            let reserved_usd = (reserved_balance * price_dec).round_dp(2);

            total_available_usd += available_usd;
            total_reserved_usd += reserved_usd;

            // Only include non-zero balances or if specifically relevant
            if total_balance > Decimal::ZERO || available_balance > Decimal::ZERO {
                balances.push(Balance {
                    crypto_type,
                    total_balance,
                    available_balance,
                    reserved_balance,
                    balance_usd: available_usd,
                    available_usd,
                    reserved_usd,
                    last_updated,
                });
            }
        }

        Ok(BalanceSummary {
            total_usd: total_available_usd + total_reserved_usd,
            available_usd: total_available_usd,
            reserved_usd: total_reserved_usd,
            balances,
        })
    }

    pub async fn update_balance(
        &self,
        merchant_id: i64,
        crypto_type: CryptoType,
        amount_change: Decimal,
        balance_type: &str,
        sandbox_mode: bool,
    ) -> Result<(), ServiceError> {
        // Ensure balance record exists
        self.initialize_balance(merchant_id, crypto_type, sandbox_mode)
            .await?;

        match balance_type {
            "available" => {
                sqlx::query(
                    r#"
                    UPDATE merchant_balances 
                    SET available_balance = available_balance + $1,
                        last_updated = $2
                    WHERE merchant_id = $3 AND crypto_type = $4 AND sandbox_mode = $5
                    "#,
                )
                .bind(amount_change)
                .bind(Utc::now())
                .bind(merchant_id)
                .bind(crypto_type.to_string())
                .bind(sandbox_mode)
                .execute(&self.db_pool)
                .await?;
            }
            "pending" => {
                sqlx::query(
                    r#"
                    UPDATE merchant_balances 
                    SET reserved_balance = reserved_balance + $1,
                        
                        last_updated = $2
                    WHERE merchant_id = $3 AND crypto_type = $4 AND sandbox_mode = $5
                    "#,
                )
                .bind(amount_change)
                .bind(Utc::now())
                .bind(merchant_id)
                .bind(crypto_type.to_string())
                .bind(sandbox_mode)
                .execute(&self.db_pool)
                .await?;
            }
            "total" => {
                sqlx::query(
                    r#"
                    UPDATE merchant_balances 
                    SET last_updated = $1
                    WHERE merchant_id = $2 AND crypto_type = $3 AND sandbox_mode = $4
                    "#,
                )
                .bind(Utc::now())
                .bind(merchant_id)
                .bind(crypto_type.to_string())
                .bind(sandbox_mode)
                .execute(&self.db_pool)
                .await?;
            }
            _ => {
                return Err(ServiceError::ValidationError(
                    "Invalid balance type".to_string(),
                ));
            }
        }

        Ok(())
    }

    pub async fn move_pending_to_available(
        &self,
        merchant_id: i64,
        crypto_type: CryptoType,
        amount: Decimal,
        sandbox_mode: bool,
    ) -> Result<(), ServiceError> {
        sqlx::query(
            r#"
            UPDATE merchant_balances 
            SET reserved_balance = reserved_balance - $1,
                available_balance = available_balance + $1,
                last_updated = $2
            WHERE merchant_id = $3 AND crypto_type = $4 AND sandbox_mode = $5
            "#,
        )
        .bind(amount)
        .bind(Utc::now())
        .bind(merchant_id)
        .bind(crypto_type.to_string())
        .bind(sandbox_mode)
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    async fn initialize_balance(
        &self,
        merchant_id: i64,
        crypto_type: CryptoType,
        sandbox_mode: bool,
    ) -> Result<(), ServiceError> {
        sqlx::query(
            r#"
            INSERT INTO merchant_balances (merchant_id, crypto_type, available_balance, reserved_balance, last_updated, sandbox_mode)
            VALUES ($1, $2, 0, 0, $3, $4)
            ON CONFLICT (merchant_id, crypto_type, sandbox_mode) DO NOTHING
            "#
        )
        .bind(merchant_id)
        .bind(crypto_type.to_string())
        .bind(Utc::now())
        .bind(sandbox_mode)
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    pub async fn refresh_balances_from_blockchain(
        &self,
        merchant_id: i64,
        sandbox_mode: bool,
    ) -> Result<(), ServiceError> {
        let crypto_types = vec![
            CryptoType::Sol,
            CryptoType::UsdtSpl,
            CryptoType::Eth,
            CryptoType::UsdtEth,
            CryptoType::Bnb,
            CryptoType::UsdtBep20,
            CryptoType::Matic,
            CryptoType::UsdtPolygon,
            CryptoType::Arb,
            CryptoType::UsdtArbitrum,
        ];

        for crypto_type in crypto_types {
            // Calculate balance from confirmed payments
            let balance_data = sqlx::query(
                r#"
                SELECT 
                    COALESCE(SUM(CASE WHEN status = 'CONFIRMED' THEN amount ELSE 0 END), 0) as confirmed_total,
                    COALESCE(SUM(CASE WHEN status = 'PENDING' THEN amount ELSE 0 END), 0) as pending_total
                FROM payment_transactions 
                WHERE merchant_id = $1 AND crypto_type = $2 AND sandbox_mode = $3
                "#
            )
            .bind(merchant_id)
            .bind(crypto_type.to_string())
            .bind(sandbox_mode)
            .fetch_one(&self.db_pool)
            .await?;

            // Calculate total refunds
            let refund_data = sqlx::query(
                r#"
                SELECT COALESCE(SUM(amount), 0) as total_refunded
                FROM refunds 
                WHERE merchant_id = $1 AND status = 'COMPLETED' AND sandbox_mode = $2
                AND payment_id IN (SELECT id FROM payment_transactions WHERE crypto_type = $3)
                "#,
            )
            .bind(merchant_id)
            .bind(sandbox_mode)
            .bind(crypto_type.to_string())
            .fetch_one(&self.db_pool)
            .await?;

            // Calculate withdrawals
            let withdrawal_data = sqlx::query(
                r#"
                SELECT COALESCE(SUM(amount), 0) as total_withdrawn
                FROM withdrawals 
                WHERE merchant_id = $1 AND crypto_type = $2 AND status = 'COMPLETED' AND sandbox_mode = $3
                "#
            )
            .bind(merchant_id)
            .bind(crypto_type.to_string())
            .bind(sandbox_mode)
            .fetch_one(&self.db_pool)
            .await?;

            let confirmed_balance: Decimal = balance_data
                .try_get::<Option<Decimal>, _>("confirmed_total")
                .ok()
                .flatten()
                .unwrap_or(Decimal::ZERO);
            let reserved_balance: Decimal = balance_data
                .try_get::<Option<Decimal>, _>("pending_total")
                .ok()
                .flatten()
                .unwrap_or(Decimal::ZERO);
            let withdrawn: Decimal = withdrawal_data
                .try_get::<Option<Decimal>, _>("total_withdrawn")
                .ok()
                .flatten()
                .unwrap_or(Decimal::ZERO);
            let refunded: Decimal = refund_data
                .try_get::<Option<Decimal>, _>("total_refunded")
                .ok()
                .flatten()
                .unwrap_or(Decimal::ZERO);

            let available_balance = confirmed_balance - withdrawn - refunded;
            let _total_balance = available_balance + reserved_balance;

            // Update balance record
            sqlx::query(
                r#"
                INSERT INTO merchant_balances (merchant_id, crypto_type, available_balance, reserved_balance, last_updated, sandbox_mode)
                VALUES ($1, $2, $3, $4, $5, $6)
                ON CONFLICT (merchant_id, crypto_type, sandbox_mode) 
                DO UPDATE SET 
                    available_balance = $3,
                    reserved_balance = $4,
                    last_updated = $5
                "#
            )
            .bind(merchant_id)
            .bind(crypto_type.to_string())
            .bind(available_balance)
            .bind(reserved_balance)
            .bind(Utc::now())
            .bind(sandbox_mode)
            .execute(&self.db_pool)
            .await?;
        }

        // 4. Push real-time update to dashboard
        let _ = self
            .broadcast_balance_update(merchant_id, sandbox_mode)
            .await;

        Ok(())
    }

    pub async fn broadcast_balance_update(
        &self,
        merchant_id: i64,
        sandbox_mode: bool,
    ) -> Result<(), ServiceError> {
        let summary = self.get_all_balances(merchant_id, sandbox_mode).await?;

        if let Ok(mut publish_conn) = self.redis_client.get_multiplexed_async_connection().await {
            let notification = serde_json::json!({
                "event": "merchant.balance_updated",
                "data": summary
            });
            let channel = format!("merchant_notifications:{}", merchant_id);
            let _: redis::RedisResult<()> = redis::cmd("PUBLISH")
                .arg(&channel)
                .arg(notification.to_string())
                .query_async(&mut publish_conn)
                .await;
        }

        Ok(())
    }
}
