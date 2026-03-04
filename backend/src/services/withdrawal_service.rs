use crate::error::ServiceError;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::models::withdrawal::Withdrawal;

pub struct WithdrawalService {
    db_pool: PgPool,
}

impl WithdrawalService {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }

    pub async fn create_withdrawal(
        &self,
        merchant_id: i64,
        request: WithdrawalRequest,
        sandbox_mode: bool,
    ) -> Result<Withdrawal, ServiceError> {
        let withdrawal_id = format!("wd_{}", uuid::Uuid::new_v4().to_string().replace("-", ""));
        
        let withdrawal_res: Result<Withdrawal, sqlx::Error> = sqlx::query_as!(
            Withdrawal,
            r#"
            INSERT INTO withdrawals (
                withdrawal_id, merchant_id, crypto_type, amount, destination_address,
                status, fee, net_amount, created_at, updated_at, sandbox_mode
            )
            VALUES ($1, $2, $3, $4, $5, 'PENDING', $6, $7, NOW(), NOW(), $8)
            RETURNING id, withdrawal_id, merchant_id, crypto_type, 
                     amount, destination_address, status, fee, net_amount, transaction_hash,
                     rejection_reason, requires_approval, approved_by, approved_at, 
                     completed_at, created_at, updated_at
            "#,
            withdrawal_id,
            merchant_id,
            request.crypto_type,
            request.amount,
            request.destination_address,
            Decimal::ZERO, // fee
            request.amount, // net_amount
            sandbox_mode
        )
        .fetch_one(&self.db_pool)
        .await;

        let withdrawal = withdrawal_res?;

        Ok(withdrawal)
    }

    pub async fn get_withdrawal(
        &self,
        merchant_id: i64,
        withdrawal_id: &str,
    ) -> Result<Withdrawal, ServiceError> {
        let withdrawal_res: Result<Option<Withdrawal>, sqlx::Error> = sqlx::query_as!(
            Withdrawal,
            r#"
            SELECT id, withdrawal_id, merchant_id, crypto_type, 
                   amount, destination_address, status, fee, net_amount, transaction_hash,
                   rejection_reason, requires_approval, approved_by, approved_at, 
                   completed_at, created_at, updated_at
            FROM withdrawals 
            WHERE withdrawal_id = $1 AND merchant_id = $2
            "#,
            withdrawal_id,
            merchant_id
        )
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
        let withdrawals_res: Result<Vec<Withdrawal>, sqlx::Error> = sqlx::query_as!(
            Withdrawal,
            r#"
            SELECT id, withdrawal_id, merchant_id, crypto_type, 
                   amount, destination_address, status, fee, net_amount, transaction_hash,
                   rejection_reason, requires_approval, approved_by, approved_at, 
                   completed_at, created_at, updated_at
            FROM withdrawals 
            WHERE merchant_id = $1 AND sandbox_mode = $2
            ORDER BY created_at DESC
            "#,
            merchant_id,
            sandbox_mode
        )
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
        let withdrawal_res: Result<Option<Withdrawal>, sqlx::Error> = sqlx::query_as!(
            Withdrawal,
            r#"
            UPDATE withdrawals 
            SET status = 'CANCELLED', updated_at = NOW()
            WHERE withdrawal_id = $1 AND merchant_id = $2 AND status = 'PENDING'
            RETURNING id, withdrawal_id, merchant_id, crypto_type, 
                     amount, destination_address, status, fee, net_amount, transaction_hash,
                     rejection_reason, requires_approval, approved_by, approved_at, 
                     completed_at, created_at, updated_at
            "#,
            withdrawal_id,
            merchant_id
        )
        .fetch_optional(&self.db_pool)
        .await;

        let withdrawal = withdrawal_res?;

        withdrawal.ok_or(ServiceError::PaymentNotFound)
    }
}
