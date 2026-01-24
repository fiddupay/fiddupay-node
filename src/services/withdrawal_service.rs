use sqlx::PgPool;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use nanoid::nanoid;
use tracing::{info, warn};
use crate::error::ServiceError;
use crate::services::balance_service::BalanceService;
use std::sync::Arc;

const MIN_WITHDRAWAL: f64 = 10.0;
const AUTO_APPROVE_THRESHOLD: f64 = 1000.0;
const WITHDRAWAL_FEE_PERCENT: f64 = 0.5; // 0.5%

#[derive(Debug, Serialize, Deserialize)]
pub struct WithdrawalRequest {
    pub crypto_type: String,
    pub amount: Decimal,
    pub destination_address: String,
}

#[derive(Debug, Serialize)]
pub struct Withdrawal {
    pub withdrawal_id: String,
    pub merchant_id: i64,
    pub crypto_type: String,
    pub amount: Decimal,
    pub destination_address: String,
    pub status: String,
    pub fee: Decimal,
    pub net_amount: Decimal,
    pub transaction_hash: Option<String>,
    pub requires_approval: bool,
    pub created_at: DateTime<Utc>,
    pub estimated_completion: Option<DateTime<Utc>>,
}

pub struct WithdrawalService {
    pool: PgPool,
    balance_service: Arc<BalanceService>,
}

impl WithdrawalService {
    pub fn new(pool: PgPool, balance_service: Arc<BalanceService>) -> Self {
        Self { pool, balance_service }
    }

    pub async fn create_withdrawal(&self, merchant_id: i64, req: WithdrawalRequest) -> Result<Withdrawal, ServiceError> {
        // Validate minimum
        let amount_f64 = req.amount.to_string().parse::<f64>().unwrap_or(0.0);
        if amount_f64 < MIN_WITHDRAWAL {
            return Err(ServiceError::ValidationError(format!("Minimum withdrawal is ${}", MIN_WITHDRAWAL)));
        }

        // Calculate fee
        let fee = req.amount * Decimal::from_f64_retain(WITHDRAWAL_FEE_PERCENT / 100.0).unwrap();
        let net_amount = req.amount - fee;

        // Check balance - handle crypto type variants
        let balance = match self.balance_service.get_balance(merchant_id).await {
            Ok(balance) => balance,
            Err(e) => {
                return Err(e);
            }
        };
        
        let crypto_balance = balance.balances.iter()
            .find(|b| {
                // Handle crypto type variants (USDT_ETH -> USDT, etc.)
                b.crypto_type == req.crypto_type || 
                (req.crypto_type.starts_with("USDT_") && b.crypto_type == "USDT") ||
                (req.crypto_type == "SOL" && b.crypto_type == "SOL")
            })
            .ok_or_else(|| ServiceError::ValidationError("No balance for this currency".to_string()))?;

        if crypto_balance.available_balance < req.amount {
            println!("DEBUG: Balance check failed - available: {}, requested: {}", crypto_balance.available_balance, req.amount);
            return Err(ServiceError::ValidationError("Insufficient balance".to_string()));
        }
        
        println!("DEBUG: Balance check passed - available: {}, requested: {}", crypto_balance.available_balance, req.amount);

        // Determine if needs approval
        let requires_approval = amount_f64 >= AUTO_APPROVE_THRESHOLD;
        let status = if requires_approval { "PENDING" } else { "APPROVED" };

        let withdrawal_id = format!("wd_{}", nanoid!(12));

        // Create withdrawal
        let record = sqlx::query!(
            r#"INSERT INTO withdrawals 
               (withdrawal_id, merchant_id, crypto_type, amount, destination_address, status, fee, net_amount, requires_approval)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
               RETURNING created_at"#,
            withdrawal_id, merchant_id, req.crypto_type, req.amount, req.destination_address,
            status, fee, net_amount, requires_approval
        )
        .fetch_one(&self.pool)
        .await?;

        // Reserve balance - use the crypto type from the balance record
        self.balance_service.reserve(
            merchant_id,
            &crypto_balance.crypto_type, // Use the actual stored crypto type
            req.amount,
            "WITHDRAWAL_RESERVED",
            Some(&withdrawal_id)
        ).await?;

        // Auto-process if approved
        if !requires_approval {
            // Process withdrawal to blockchain
            match self.process_blockchain_withdrawal(&withdrawal_id, &req.crypto_type, &req.amount, &req.destination_address).await {
                Ok(_) => {
                    // Update status to completed
                    sqlx::query!(
                        "UPDATE withdrawals SET status = 'completed', completed_at = NOW() WHERE withdrawal_id = $1",
                        withdrawal_id
                    ).execute(&self.pool).await?;
                }
                Err(e) => {
                    warn!("Failed to process withdrawal {}: {}", withdrawal_id, e);
                    // Update status to failed
                    sqlx::query!(
                        "UPDATE withdrawals SET status = 'failed' WHERE withdrawal_id = $1",
                        withdrawal_id
                    ).execute(&self.pool).await?;
                }
            }
        }

        Ok(Withdrawal {
            withdrawal_id,
            merchant_id,
            crypto_type: req.crypto_type,
            amount: req.amount,
            destination_address: req.destination_address,
            status: status.to_string(),
            fee,
            net_amount,
            transaction_hash: None,
            requires_approval,
            created_at: record.created_at,
            estimated_completion: None,
        })
    }

    pub async fn get_withdrawal(&self, merchant_id: i64, withdrawal_id: &str) -> Result<Withdrawal, ServiceError> {
        let record = sqlx::query!(
            r#"SELECT withdrawal_id, merchant_id, crypto_type, amount, destination_address, status, 
                      fee, net_amount, transaction_hash, requires_approval, created_at
               FROM withdrawals WHERE withdrawal_id = $1 AND merchant_id = $2"#,
            withdrawal_id, merchant_id
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| ServiceError::NotFound("Withdrawal not found".to_string()))?;

        Ok(Withdrawal {
            withdrawal_id: record.withdrawal_id,
            merchant_id: record.merchant_id,
            crypto_type: record.crypto_type,
            amount: record.amount,
            destination_address: record.destination_address,
            status: record.status,
            fee: record.fee,
            net_amount: record.net_amount,
            transaction_hash: record.transaction_hash,
            requires_approval: record.requires_approval,
            created_at: record.created_at,
            estimated_completion: None,
        })
    }

    pub async fn list_withdrawals(&self, merchant_id: i64, limit: i64) -> Result<Vec<Withdrawal>, ServiceError> {
        let records = sqlx::query!(
            r#"SELECT withdrawal_id, merchant_id, crypto_type, amount, destination_address, status, 
                      fee, net_amount, transaction_hash, requires_approval, created_at
               FROM withdrawals WHERE merchant_id = $1 ORDER BY created_at DESC LIMIT $2"#,
            merchant_id, limit
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(records.into_iter().map(|r| Withdrawal {
            withdrawal_id: r.withdrawal_id,
            merchant_id: r.merchant_id,
            crypto_type: r.crypto_type,
            amount: r.amount,
            destination_address: r.destination_address,
            status: r.status,
            fee: r.fee,
            net_amount: r.net_amount,
            transaction_hash: r.transaction_hash,
            requires_approval: r.requires_approval,
            created_at: r.created_at,
            estimated_completion: None,
        }).collect())
    }

    pub async fn cancel_withdrawal(&self, merchant_id: i64, withdrawal_id: &str) -> Result<(), ServiceError> {
        let withdrawal = self.get_withdrawal(merchant_id, withdrawal_id).await?;

        if withdrawal.status != "PENDING" {
            return Err(ServiceError::ValidationError("Can only cancel pending withdrawals".to_string()));
        }

        // Update status
        sqlx::query!(
            "UPDATE withdrawals SET status = 'CANCELLED', updated_at = NOW() WHERE withdrawal_id = $1",
            withdrawal_id
        )
        .execute(&self.pool)
        .await?;

        // Release reserved balance
        self.balance_service.release_reserve(
            merchant_id,
            &withdrawal.crypto_type,
            withdrawal.amount,
            "WITHDRAWAL_CANCELLED",
            Some(withdrawal_id)
        ).await?;

        Ok(())
    }

    pub async fn complete_withdrawal(&self, withdrawal_id: &str, transaction_hash: &str) -> Result<(), ServiceError> {
        let withdrawal = sqlx::query!(
            "SELECT merchant_id, crypto_type, amount FROM withdrawals WHERE withdrawal_id = $1",
            withdrawal_id
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| ServiceError::NotFound("Withdrawal not found".to_string()))?;

        // Update status
        sqlx::query!(
            "UPDATE withdrawals SET status = 'COMPLETED', transaction_hash = $2, completed_at = NOW(), updated_at = NOW() 
             WHERE withdrawal_id = $1",
            withdrawal_id, transaction_hash
        )
        .execute(&self.pool)
        .await?;

        // Debit reserved balance
        self.balance_service.debit_available(
            withdrawal.merchant_id,
            &withdrawal.crypto_type,
            withdrawal.amount,
            "WITHDRAWAL_COMPLETED",
            Some(withdrawal_id)
        ).await?;

        Ok(())
    }

    async fn process_blockchain_withdrawal(
        &self,
        withdrawal_id: &str,
        crypto_type: &str,
        amount: &Decimal,
        destination_address: &str,
    ) -> Result<String, ServiceError> {
        // This is a placeholder for actual blockchain transaction processing
        // In production, this would:
        // 1. Connect to the appropriate blockchain RPC
        // 2. Create and sign the transaction
        // 3. Broadcast the transaction
        // 4. Return the transaction hash
        
        info!("Processing withdrawal {} for {} {} to {}", 
            withdrawal_id, amount, crypto_type, destination_address);
        
        // For now, simulate successful processing
        // In production, replace with actual blockchain integration
        let mock_tx_hash = format!("tx_{}", nanoid::nanoid!(32));
        
        Ok(mock_tx_hash)
    }
}
