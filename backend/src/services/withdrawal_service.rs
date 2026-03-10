use crate::error::ServiceError;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::models::withdrawal::Withdrawal;
use crate::services::price_service::PriceService;
use crate::payment::models::CryptoType;
use std::sync::Arc;
use rust_decimal::prelude::FromPrimitive;

#[derive(Debug, Deserialize)]
pub struct WithdrawalRequest {
    pub crypto_type: String,
    pub amount: Decimal,
    pub destination_address: String,
}

pub struct WithdrawalService {
    db_pool: PgPool,
    price_service: Arc<PriceService>,
}

impl WithdrawalService {
    pub fn new(db_pool: PgPool, price_service: Arc<PriceService>) -> Self {
        Self { db_pool, price_service }
    }

    pub async fn create_withdrawal(
        &self,
        merchant_id: i64,
        request: WithdrawalRequest,
        sandbox_mode: bool,
    ) -> Result<Withdrawal, ServiceError> {
        let withdrawal_id = format!("wd_{}", uuid::Uuid::new_v4().to_string().replace("-", ""));
        
        // Calculate USD amount (Requirement 20)
        let crypto_type_enum = CryptoType::from_string(&request.crypto_type)?;
        let price = self.price_service.get_price(crypto_type_enum).await.unwrap_or(0.0);
        let amount_usd = request.amount * Decimal::from_f64(price).unwrap_or(Decimal::ZERO);
        let amount_usd = amount_usd.round_dp(2);

        let withdrawal_res: Result<Withdrawal, sqlx::Error> = sqlx::query_as::<_, Withdrawal>(
            r#"
            INSERT INTO withdrawals (
                withdrawal_id, merchant_id, crypto_type, amount, amount_usd, destination_address,
                status, fee, net_amount, created_at, updated_at, sandbox_mode
            )
            VALUES ($1, $2, $3, $4, $5, $6, 'PENDING', $7, $8, NOW(), NOW(), $9)
            RETURNING id, withdrawal_id, merchant_id, crypto_type, 
                     amount, amount_usd, destination_address, status, fee, net_amount, transaction_hash,
                     rejection_reason, requires_approval, approved_by, approved_at, 
                     completed_at, created_at, updated_at
            "#
        )
        .bind(&withdrawal_id)
        .bind(merchant_id)
        .bind(&request.crypto_type)
        .bind(request.amount)
        .bind(amount_usd)
        .bind(&request.destination_address)
        .bind(Decimal::ZERO) // fee
        .bind(request.amount) // net_amount
        .bind(sandbox_mode)
        .fetch_one(&self.db_pool)
        .await;

        let withdrawal = withdrawal_res?;

        tracing::info!(
            "Withdrawal created: id={}, merchant_id={}, crypto_type={}, amount={}, target={}, sandbox={}",
            withdrawal.withdrawal_id,
            withdrawal.merchant_id,
            withdrawal.crypto_type,
            withdrawal.amount,
            withdrawal.destination_address,
            sandbox_mode
        );

        Ok(withdrawal)
    }

    pub async fn get_withdrawal(
        &self,
        merchant_id: i64,
        withdrawal_id: &str,
    ) -> Result<Withdrawal, ServiceError> {
        let withdrawal_res: Result<Option<Withdrawal>, sqlx::Error> = sqlx::query_as::<_, Withdrawal>(
            r#"
            SELECT id, withdrawal_id, merchant_id, crypto_type, 
                   amount, amount_usd, destination_address, status, fee, net_amount, transaction_hash,
                   rejection_reason, requires_approval, approved_by, approved_at, 
                   completed_at, created_at, updated_at
            FROM withdrawals 
            WHERE withdrawal_id = $1 AND merchant_id = $2
            "#
        )
        .bind(withdrawal_id)
        .bind(merchant_id)
        .fetch_optional(&self.db_pool)
        .await;

        let withdrawal = withdrawal_res?;

        withdrawal.ok_or(ServiceError::PaymentNotFound)
    }

    pub async fn list_withdrawals(
        &self,
        merchant_id: i64,
        sandbox_mode: bool,
    ) -> Result<Vec<Withdrawal>, ServiceError> {
        let withdrawals_res: Result<Vec<Withdrawal>, sqlx::Error> = sqlx::query_as::<_, Withdrawal>(
            r#"
            SELECT id, withdrawal_id, merchant_id, crypto_type, 
                   amount, amount_usd, destination_address, status, fee, net_amount, transaction_hash,
                   rejection_reason, requires_approval, approved_by, approved_at, 
                   completed_at, created_at, updated_at
            FROM withdrawals 
            WHERE merchant_id = $1 AND sandbox_mode = $2
            ORDER BY created_at DESC
            "#
        )
        .bind(merchant_id)
        .bind(sandbox_mode)
        .fetch_all(&self.db_pool)
        .await;

        let withdrawals = withdrawals_res?;

        Ok(withdrawals)
    }

    pub async fn cancel_withdrawal(
        &self,
        merchant_id: i64,
        withdrawal_id: &str,
    ) -> Result<Withdrawal, ServiceError> {
        let withdrawal_res: Result<Option<Withdrawal>, sqlx::Error> = sqlx::query_as::<_, Withdrawal>(
            r#"
            UPDATE withdrawals 
            SET status = 'CANCELLED', updated_at = NOW()
            WHERE withdrawal_id = $1 AND merchant_id = $2 AND status = 'PENDING'
            RETURNING id, withdrawal_id, merchant_id, crypto_type, 
                     amount, amount_usd, destination_address, status, fee, net_amount, transaction_hash,
                     rejection_reason, requires_approval, approved_by, approved_at, 
                     completed_at, created_at, updated_at
            "#
        )
        .bind(withdrawal_id)
        .bind(merchant_id)
        .fetch_optional(&self.db_pool)
        .await;

        let withdrawal = withdrawal_res?;

        withdrawal.ok_or(ServiceError::PaymentNotFound)
    }
}
