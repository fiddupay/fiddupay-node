use sqlx::PgPool;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::error::ServiceError;
use crate::services::price_service::PriceService;
use crate::payment::models::CryptoType;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct MerchantBalance {
    pub crypto_type: String,
    pub available_balance: Decimal,
    pub reserved_balance: Decimal,
    pub total_balance: Decimal,
    pub available_usd: Option<Decimal>,
    pub total_usd: Option<Decimal>,
}

#[derive(Debug, Serialize)]
pub struct BalanceResponse {
    pub total_usd: Decimal,
    pub available_usd: Decimal,
    pub reserved_usd: Decimal,
    pub balances: Vec<MerchantBalance>,
}

#[derive(Debug, Serialize)]
pub struct BalanceHistoryEntry {
    pub id: i32,
    pub crypto_type: String,
    pub amount: Decimal,
    pub balance_type: String,
    pub change_type: String,
    pub reason: String,
    pub reference_id: Option<String>,
    pub balance_before: Decimal,
    pub balance_after: Decimal,
    pub created_at: DateTime<Utc>,
}

pub struct BalanceService {
    pool: PgPool,
    price_service: Arc<PriceService>,
}

impl BalanceService {
    pub fn new(pool: PgPool, price_service: Arc<PriceService>) -> Self {
        Self { pool, price_service }
    }

    pub async fn credit_available(&self, merchant_id: i64, crypto_type: &str, amount: Decimal, reason: &str, reference_id: Option<&str>) -> Result<(), ServiceError> {
        let mut tx = self.pool.begin().await?;

        // Get or create balance
        let balance = sqlx::query!(
            "INSERT INTO merchant_balances (merchant_id, crypto_type, available_balance)
             VALUES ($1, $2, $3)
             ON CONFLICT (merchant_id, crypto_type)
             DO UPDATE SET available_balance = merchant_balances.available_balance + $3, last_updated = NOW()
             RETURNING available_balance - $3 as balance_before, available_balance as balance_after",
            merchant_id, crypto_type, amount
        )
        .fetch_one(&mut *tx)
        .await?;

        // Record history
        sqlx::query!(
            "INSERT INTO balance_history (merchant_id, crypto_type, amount, balance_type, change_type, reason, reference_id, balance_before, balance_after)
             VALUES ($1, $2, $3, 'AVAILABLE', 'CREDIT', $4, $5, $6, $7)",
            merchant_id, crypto_type, amount, reason, reference_id,
            balance.balance_before,
            balance.balance_after
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    pub async fn debit_available(&self, merchant_id: i64, crypto_type: &str, amount: Decimal, reason: &str, reference_id: Option<&str>) -> Result<(), ServiceError> {
        let mut tx = self.pool.begin().await?;

        // Check sufficient balance
        let balance = sqlx::query!(
            "SELECT available_balance FROM merchant_balances WHERE merchant_id = $1 AND crypto_type = $2",
            merchant_id, crypto_type
        )
        .fetch_optional(&mut *tx)
        .await?;

        let current = balance.map(|b| b.available_balance).unwrap_or(Decimal::ZERO);
        if current < amount {
            return Err(ServiceError::ValidationError("Insufficient balance".to_string()));
        }

        // Debit balance
        let result = sqlx::query!(
            "UPDATE merchant_balances SET available_balance = available_balance - $3, last_updated = NOW()
             WHERE merchant_id = $1 AND crypto_type = $2
             RETURNING available_balance + $3 as balance_before, available_balance as balance_after",
            merchant_id, crypto_type, amount
        )
        .fetch_one(&mut *tx)
        .await?;

        // Record history
        sqlx::query!(
            "INSERT INTO balance_history (merchant_id, crypto_type, amount, balance_type, change_type, reason, reference_id, balance_before, balance_after)
             VALUES ($1, $2, $3, 'AVAILABLE', 'DEBIT', $4, $5, $6, $7)",
            merchant_id, crypto_type, amount, reason, reference_id,
            result.balance_before,
            result.balance_after
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    pub async fn reserve(&self, merchant_id: i64, crypto_type: &str, amount: Decimal, reason: &str, reference_id: Option<&str>) -> Result<(), ServiceError> {
        let mut tx = self.pool.begin().await?;

        // Move from available to reserved
        let balance = sqlx::query!(
            "SELECT available_balance FROM merchant_balances WHERE merchant_id = $1 AND crypto_type = $2",
            merchant_id, crypto_type
        )
        .fetch_optional(&mut *tx)
        .await?;

        let current = balance.map(|b| b.available_balance).unwrap_or(Decimal::ZERO);
        if current < amount {
            return Err(ServiceError::ValidationError("Insufficient available balance".to_string()));
        }

        sqlx::query!(
            "UPDATE merchant_balances 
             SET available_balance = available_balance - $3, reserved_balance = reserved_balance + $3, last_updated = NOW()
             WHERE merchant_id = $1 AND crypto_type = $2",
            merchant_id, crypto_type, amount
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!(
            "INSERT INTO balance_history (merchant_id, crypto_type, amount, balance_type, change_type, reason, reference_id, balance_before, balance_after)
             VALUES ($1, $2, $3, 'RESERVED', 'CREDIT', $4, $5, 0, $3)",
            merchant_id, crypto_type, amount, reason, reference_id
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    pub async fn release_reserve(&self, merchant_id: i64, crypto_type: &str, amount: Decimal, reason: &str, reference_id: Option<&str>) -> Result<(), ServiceError> {
        let mut tx = self.pool.begin().await?;

        sqlx::query!(
            "UPDATE merchant_balances 
             SET reserved_balance = reserved_balance - $3, available_balance = available_balance + $3, last_updated = NOW()
             WHERE merchant_id = $1 AND crypto_type = $2",
            merchant_id, crypto_type, amount
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query!(
            "INSERT INTO balance_history (merchant_id, crypto_type, amount, balance_type, change_type, reason, reference_id, balance_before, balance_after)
             VALUES ($1, $2, $3, 'RESERVED', 'DEBIT', $4, $5, $3, 0)",
            merchant_id, crypto_type, amount, reason, reference_id
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    pub async fn get_balance(&self, merchant_id: i64) -> Result<BalanceResponse, ServiceError> {
        let balances = sqlx::query!(
            "SELECT crypto_type, available_balance, reserved_balance, total_balance
             FROM merchant_balances WHERE merchant_id = $1",
            merchant_id
        )
        .fetch_all(&self.pool)
        .await?;

        let mut total_usd = Decimal::ZERO;
        let mut available_usd = Decimal::ZERO;
        let mut reserved_usd = Decimal::ZERO;

        let mut balance_list = Vec::new();
        for b in balances {
            let balance_available_usd = self.calculate_usd_value(&b.crypto_type, &b.available_balance).await.unwrap_or(Decimal::ZERO);
            let balance_total_usd = self.calculate_usd_value(&b.crypto_type, &b.total_balance.unwrap_or(Decimal::ZERO)).await.unwrap_or(Decimal::ZERO);
            let balance_reserved_usd = self.calculate_usd_value(&b.crypto_type, &b.reserved_balance).await.unwrap_or(Decimal::ZERO);
            
            // Accumulate totals
            available_usd += balance_available_usd;
            total_usd += balance_total_usd;
            reserved_usd += balance_reserved_usd;
            
            balance_list.push(MerchantBalance {
                crypto_type: b.crypto_type,
                available_balance: b.available_balance,
                reserved_balance: b.reserved_balance,
                total_balance: b.total_balance.unwrap_or(Decimal::ZERO),
                available_usd: Some(balance_available_usd),
                total_usd: Some(balance_total_usd),
            });
        }

        Ok(BalanceResponse {
            total_usd,
            available_usd,
            reserved_usd,
            balances: balance_list,
        })
    }

    pub async fn get_history(&self, merchant_id: i64, limit: i64) -> Result<Vec<BalanceHistoryEntry>, ServiceError> {
        let records = sqlx::query_as!(
            BalanceHistoryEntry,
            "SELECT id, crypto_type, amount, balance_type, change_type, reason, reference_id, balance_before, balance_after, created_at
             FROM balance_history WHERE merchant_id = $1 ORDER BY created_at DESC LIMIT $2",
            merchant_id, limit
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(records)
    }

    pub async fn credit_from_payment(&self, payment_id: &str) -> Result<(), ServiceError> {
        // Get payment details
        let payment = sqlx::query!(
            "SELECT merchant_id, amount, crypto_type, fee_amount FROM payment_transactions 
             WHERE payment_id = $1 AND status = 'CONFIRMED'",
            payment_id
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| ServiceError::NotFound("Payment not found or not confirmed".to_string()))?;

        // Credit the net amount (amount - fee)
        let net_amount = payment.amount - payment.fee_amount;
        
        self.credit_available(
            payment.merchant_id,
            &payment.crypto_type,
            net_amount,
            "PAYMENT_CONFIRMED",
            Some(payment_id)
        ).await?;

        Ok(())
    }

    async fn calculate_usd_value(&self, crypto_type: &str, amount: &Decimal) -> Option<Decimal> {
        if amount.is_zero() {
            return Some(Decimal::ZERO);
        }

        match crypto_type {
            // USDT variants have 1:1 USD parity
            "USDT" | "USDT_SOL" | "USDT_BSC" | "USDT_POLYGON" | "USDT_ARBITRUM" | "USDT_ETH" => {
                Some(*amount)
            }
            // For other cryptocurrencies, use price service
            "SOL" => self.get_price_and_calculate(CryptoType::Sol, amount).await,
            "ETH" => self.get_price_and_calculate(CryptoType::Eth, amount).await,
            "ARB" => self.get_price_and_calculate(CryptoType::Arb, amount).await,
            "MATIC" => self.get_price_and_calculate(CryptoType::Matic, amount).await,
            "BNB" => self.get_price_and_calculate(CryptoType::Bnb, amount).await,
            _ => None,
        }
    }

    async fn get_price_and_calculate(&self, crypto_type: CryptoType, amount: &Decimal) -> Option<Decimal> {
        match self.price_service.get_price(crypto_type).await {
            Ok(price) => {
                let price_decimal = Decimal::from_f64_retain(price)?;
                Some(*amount * price_decimal)
            }
            Err(_) => None,
        }
    }
}
