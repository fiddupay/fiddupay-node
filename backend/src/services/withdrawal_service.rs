use crate::error::ServiceError;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Debug, Deserialize)]
pub struct WithdrawalRequest {
    pub crypto_type: String,
    pub amount: Decimal,
    pub destination_address: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Withdrawal {
    pub id: i32,
    pub withdrawal_id: String,
    pub merchant_id: i64,
    pub crypto_type: String,
    pub amount: Decimal,
    pub destination_address: String,
    pub status: String,
    pub fee: Decimal,
    pub net_amount: Decimal,
    pub transaction_hash: Option<String>,
    pub rejection_reason: Option<String>,
    pub requires_approval: bool,
    pub approved_by: Option<i32>,
    pub approved_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

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
    ) -> Result<Withdrawal, ServiceError> {
        let withdrawal_id = format!("wd_{}", uuid::Uuid::new_v4().to_string().replace("-", ""));
        
        let withdrawal = sqlx::query_as!(
            Withdrawal,
            r#"
            INSERT INTO withdrawals (
                withdrawal_id, merchant_id, crypto_type, amount, destination_address,
                status, fee, net_amount, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, 'PENDING', $6, $7, NOW(), NOW())
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
        )
        .fetch_one(&self.db_pool)
        .await?;

        Ok(withdrawal)
    }

    pub async fn get_withdrawal(
        &self,
        merchant_id: i64,
        withdrawal_id: &str,
    ) -> Result<Withdrawal, ServiceError> {
        let withdrawal = sqlx::query_as!(
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
        .await?;

        withdrawal.ok_or(ServiceError::PaymentNotFound)
    }

    pub async fn list_withdrawals(
        &self,
        merchant_id: i64,
    ) -> Result<Vec<Withdrawal>, ServiceError> {
        let withdrawals = sqlx::query_as!(
            Withdrawal,
            r#"
            SELECT id, withdrawal_id, merchant_id, crypto_type, 
                   amount, destination_address, status, fee, net_amount, transaction_hash,
                   rejection_reason, requires_approval, approved_by, approved_at, 
                   completed_at, created_at, updated_at
            FROM withdrawals 
            WHERE merchant_id = $1
            ORDER BY created_at DESC
            "#,
            merchant_id
        )
        .fetch_all(&self.db_pool)
        .await?;

        Ok(withdrawals)
    }

    pub async fn cancel_withdrawal(
        &self,
        merchant_id: i64,
        withdrawal_id: &str,
    ) -> Result<Withdrawal, ServiceError> {
        let withdrawal = sqlx::query_as!(
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
        .await?;

        withdrawal.ok_or(ServiceError::PaymentNotFound)
    }
}
