// Balance Service - Tracks merchant balances across all networks

use crate::error::ServiceError;
use crate::payment::models::CryptoType;
use crate::services::price_service::PriceService;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct Balance {
    pub crypto_type: CryptoType,
    pub total_balance: Decimal,
    pub available_balance: Decimal,
    pub reserved_balance: Decimal,
    pub balance_usd: Decimal,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BalanceSummary {
    pub total_usd: Decimal,
    pub balances: Vec<Balance>,
}

pub struct BalanceService {
    db_pool: PgPool,
    price_service: Arc<PriceService>,
}

impl BalanceService {
    pub fn new(db_pool: PgPool, price_service: Arc<PriceService>) -> Self {
        Self {
            db_pool,
            price_service,
        }
    }

    pub async fn get_balance(
        &self,
        merchant_id: i64,
        crypto_type: CryptoType,
    ) -> Result<Balance, ServiceError> {
        let balance_record = sqlx::query!(
            r#"
            SELECT 
                total_balance,
                available_balance,
                reserved_balance,
                last_updated
            FROM merchant_balances 
            WHERE merchant_id = $1 AND crypto_type = $2
            "#,
            merchant_id,
            crypto_type as CryptoType
        )
        .fetch_optional(&self.db_pool)
        .await?;

        let total = balance_record.as_ref().and_then(|r| r.total_balance).unwrap_or(Decimal::ZERO);
        let available = balance_record.as_ref().map_or(Decimal::ZERO, |r| r.available_balance);
        let pending = balance_record.as_ref().map_or(Decimal::ZERO, |r| r.reserved_balance);
        let last_updated = Utc::now();

        // Get current USD value
        let price: f64 = self.price_service.get_price(crypto_type).await.unwrap_or(0.0);
        let balance_usd = available * Decimal::from_f64_retain(price).unwrap_or(Decimal::ZERO);

        Ok(Balance {
            crypto_type,
            total_balance: total,
            available_balance: available,
            reserved_balance: pending,
            balance_usd,
            last_updated,
        })
    }

    pub async fn get_all_balances(&self, merchant_id: i64) -> Result<BalanceSummary, ServiceError> {
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

        let mut balances = Vec::new();
        let mut total_usd = Decimal::ZERO;

        for crypto_type in crypto_types {
            let balance = self.get_balance(merchant_id, crypto_type).await?;
            total_usd += balance.balance_usd;
            
            // Only include non-zero balances
            if balance.total_balance > Decimal::ZERO {
                balances.push(balance);
            }
        }

        Ok(BalanceSummary {
            total_usd,
            balances,
        })
    }

    pub async fn update_balance(
        &self,
        merchant_id: i64,
        crypto_type: CryptoType,
        amount_change: Decimal,
        balance_type: &str, // "available", "pending", "total"
    ) -> Result<(), ServiceError> {
        // Ensure balance record exists
        self.initialize_balance(merchant_id, crypto_type).await?;

        match balance_type {
            "available" => {
                sqlx::query!(
                    r#"
                    UPDATE merchant_balances 
                    SET available_balance = available_balance + $1,
                        last_updated = $2
                    WHERE merchant_id = $3 AND crypto_type = $4
                    "#,
                    amount_change,
                    Utc::now(),
                    merchant_id,
                    crypto_type as CryptoType
                )
                .execute(&self.db_pool)
                .await?;
            }
            "pending" => {
                sqlx::query!(
                    r#"
                    UPDATE merchant_balances 
                    SET reserved_balance = reserved_balance + $1,
                        
                        last_updated = $2
                    WHERE merchant_id = $3 AND crypto_type = $4
                    "#,
                    amount_change,
                    Utc::now(),
                    merchant_id,
                    crypto_type as CryptoType
                )
                .execute(&self.db_pool)
                .await?;
            }
            "total" => {
                sqlx::query!(
                    r#"
                    UPDATE merchant_balances 
                    SET last_updated = $1
                    WHERE merchant_id = $2 AND crypto_type = $3
                    "#,
                    Utc::now(),
                    merchant_id,
                    crypto_type as CryptoType
                )
                .execute(&self.db_pool)
                .await?;
            }
            _ => {
                return Err(ServiceError::ValidationError(
                    "Invalid balance type".to_string()
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
    ) -> Result<(), ServiceError> {
        sqlx::query!(
            r#"
            UPDATE merchant_balances 
            SET reserved_balance = reserved_balance - $1,
                available_balance = available_balance + $1,
                last_updated = $2
            WHERE merchant_id = $3 AND crypto_type = $4
            "#,
            amount,
            Utc::now(),
            merchant_id,
            crypto_type as CryptoType
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    async fn initialize_balance(
        &self,
        merchant_id: i64,
        crypto_type: CryptoType,
    ) -> Result<(), ServiceError> {
        sqlx::query!(
            r#"
            INSERT INTO merchant_balances (merchant_id, crypto_type, available_balance, reserved_balance, last_updated)
            VALUES ($1, $2, 0, 0, $3)
            ON CONFLICT (merchant_id, crypto_type) DO NOTHING
            "#,
            merchant_id,
            crypto_type as CryptoType,
            Utc::now()
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    pub async fn refresh_balances_from_blockchain(
        &self,
        merchant_id: i64,
    ) -> Result<(), ServiceError> {
        // This would integrate with blockchain monitoring to get real balances
        // For now, we'll recalculate from payment transactions
        
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
            let balance_data = sqlx::query!(
                r#"
                SELECT 
                    COALESCE(SUM(CASE WHEN status = 'CONFIRMED' THEN amount ELSE 0 END), 0) as confirmed_total,
                    COALESCE(SUM(CASE WHEN status = 'PENDING' THEN amount ELSE 0 END), 0) as pending_total
                FROM payment_transactions 
                WHERE merchant_id = $1 AND crypto_type = $2
                "#,
                merchant_id,
                crypto_type as CryptoType
            )
            .fetch_one(&self.db_pool)
            .await?;

            // Calculate withdrawals
            let withdrawal_data = sqlx::query!(
                r#"
                SELECT COALESCE(SUM(amount), 0) as total_withdrawn
                FROM withdrawals 
                WHERE merchant_id = $1 AND crypto_type = $2 AND status = 'COMPLETED'
                "#,
                merchant_id,
                crypto_type as CryptoType
            )
            .fetch_one(&self.db_pool)
            .await?;

            let confirmed_balance = balance_data.confirmed_total.unwrap_or(Decimal::ZERO);
            let reserved_balance = balance_data.pending_total.unwrap_or(Decimal::ZERO);
            let withdrawn = withdrawal_data.total_withdrawn.unwrap_or(Decimal::ZERO);

            let available_balance = confirmed_balance - withdrawn;
            let total_balance = available_balance + reserved_balance;

            // Update balance record
            sqlx::query!(
                r#"
                INSERT INTO merchant_balances (merchant_id, crypto_type, available_balance, reserved_balance, last_updated)
                VALUES ($1, $2, $3, $4, $5)
                ON CONFLICT (merchant_id, crypto_type) 
                DO UPDATE SET 
                    available_balance = $3,
                    reserved_balance = $4,
                    last_updated = $5
                "#,
                merchant_id,
                crypto_type as CryptoType,
                available_balance,
                reserved_balance,
                Utc::now()
            )
            .execute(&self.db_pool)
            .await?;
        }

        Ok(())
    }
}
