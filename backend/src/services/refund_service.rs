// Refund Service
// Business logic for refund operations

use chrono::Utc;
use nanoid::nanoid;
use rust_decimal::Decimal;
use sqlx::{PgPool, Row};
use std::sync::Arc;
use tracing::{error, info};

use crate::error::ServiceError;
use crate::models::refund::RefundResponse;
use crate::models::webhook::WebhookPayload;
use crate::payment::models::PaymentStatus;
use crate::services::webhook_service::WebhookService;

pub struct RefundService {
    db_pool: PgPool,
    webhook_service: Arc<WebhookService>,
}

impl RefundService {
    pub fn new(db_pool: PgPool, webhook_service: Arc<WebhookService>) -> Self {
        Self {
            db_pool,
            webhook_service,
        }
    }

    pub async fn create_refund(
        &self,
        merchant_id: i64,
        payment_id: String,
        amount: Option<Decimal>,
        reason: String,
    ) -> Result<RefundResponse, ServiceError> {
        // Fetch the payment to validate it exists and belongs to the merchant
        let payment = sqlx::query(
            r#"
            SELECT id, merchant_id, amount, amount_usd, crypto_type, status, from_address
            FROM payment_transactions
            WHERE payment_id = $1
            "#,
        )
        .bind(&payment_id)
        .fetch_optional(&self.db_pool)
        .await?
        .ok_or(ServiceError::PaymentNotFound)?;

        let p_id: i64 = payment.get("id");
        let p_merchant_id: i64 = payment.get("merchant_id");
        let p_amount: Option<Decimal> = payment.get("amount");
        let p_amount_usd: Decimal = payment.get("amount_usd");
        let p_crypto_type: Option<String> = payment.get("crypto_type");
        let p_status: String = payment.get("status");
        let p_from_address: Option<String> = payment.get("from_address");

        // Verify the payment belongs to this merchant
        if p_merchant_id != merchant_id {
            return Err(ServiceError::PaymentNotFound);
        }

        // Verify the payment is confirmed
        if p_status != "CONFIRMED" {
            return Err(ServiceError::Internal(
                "Can only refund confirmed payments".to_string(),
            ));
        }

        // Calculate total already refunded
        let total_refunded_row = sqlx::query(
            r#"
            SELECT COALESCE(SUM(amount), 0) as total_refunded
            FROM refunds
            WHERE payment_id = $1 AND status IN ('pending', 'completed')
            "#,
        )
        .bind(p_id)
        .fetch_one(&self.db_pool)
        .await?;
        let total_refunded: Decimal = total_refunded_row
            .try_get::<Option<Decimal>, _>("total_refunded")
            .ok()
            .flatten()
            .unwrap_or(Decimal::ZERO);

        // Determine refund amount
        let refund_amount = amount
            .or(p_amount)
            .ok_or_else(|| ServiceError::Internal("Payment amount is missing".to_string()))?;

        // Validate refund amount
        let payment_amount = p_amount.unwrap_or(Decimal::ZERO);
        let remaining_amount = payment_amount - total_refunded;
        if refund_amount > remaining_amount {
            return Err(ServiceError::Internal(format!(
                "Refund amount {} exceeds remaining payment amount {}",
                refund_amount, remaining_amount
            )));
        }

        if refund_amount <= Decimal::ZERO {
            return Err(ServiceError::Internal(
                "Refund amount must be positive".to_string(),
            ));
        }

        // Calculate USD amount for the refund
        let refund_amount_usd = if Some(refund_amount) == p_amount {
            p_amount_usd
        } else {
            let payment_amt = p_amount.unwrap_or(Decimal::ONE);
            (p_amount_usd / payment_amt) * refund_amount
        };

        // Generate unique refund ID
        let refund_id = format!("ref_{}", nanoid!(16));

        // Insert refund record
        let refund = sqlx::query(
            r#"
            INSERT INTO refunds (
                refund_id, merchant_id, payment_id, amount, amount_usd,
                reason, status, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, refund_id, merchant_id, payment_id, amount, amount_usd,
                      reason, status, transaction_hash, created_at, completed_at
            "#,
        )
        .bind(&refund_id)
        .bind(merchant_id)
        .bind(p_id)
        .bind(refund_amount)
        .bind(refund_amount_usd)
        .bind(&reason)
        .bind("pending")
        .bind(Utc::now())
        .fetch_one(&self.db_pool)
        .await?;

        info!(
            "Created refund {} for payment {} - amount: {} (${:.2})",
            refund_id, payment_id, refund_amount, refund_amount_usd
        );

        Ok(RefundResponse {
            refund_id: refund.get("refund_id"),
            payment_id,
            amount: refund.get("amount"),
            amount_usd: refund.get("amount_usd"),
            status: refund.get("status"),
            reason: refund.get("reason"),
            transaction_hash: refund.get("transaction_hash"),
            crypto_type: p_crypto_type.unwrap_or_else(|| "UNKNOWN".to_string()),
            target_address: p_from_address,
            created_at: refund.get("created_at"),
            completed_at: refund.get("completed_at"),
        })
    }

    pub async fn complete_refund(
        &self,
        refund_id: String,
        transaction_hash: String,
    ) -> Result<(), ServiceError> {
        // Fetch the refund to validate it exists
        let refund = sqlx::query(
            r#"
            SELECT id, merchant_id, payment_id, status
            FROM refunds
            WHERE refund_id = $1
            "#,
        )
        .bind(&refund_id)
        .fetch_optional(&self.db_pool)
        .await?
        .ok_or_else(|| ServiceError::Internal("Refund not found".to_string()))?;

        let r_status: String = refund.get("status");
        let r_merchant_id: i64 = refund.get("merchant_id");
        let r_payment_id: i64 = refund.get("payment_id");

        if r_status != "pending" {
            return Err(ServiceError::Internal(format!(
                "Refund is already in {} status",
                r_status
            )));
        }

        // Update refund
        sqlx::query(
            r#"
            UPDATE refunds
            SET transaction_hash = $1, status = $2, completed_at = $3
            WHERE refund_id = $4
            "#,
        )
        .bind(&transaction_hash)
        .bind("completed")
        .bind(Utc::now())
        .bind(&refund_id)
        .execute(&self.db_pool)
        .await?;

        info!(
            "Completed refund {} with transaction hash: {}",
            refund_id, transaction_hash
        );

        // Fetch payment details for webhook
        let payment = sqlx::query(
            r#"
            SELECT payment_id, amount, crypto_type
            FROM payment_transactions
            WHERE id = $1
            "#,
        )
        .bind(r_payment_id)
        .fetch_one(&self.db_pool)
        .await?;

        let pay_payment_id: String = payment.get("payment_id");
        let pay_amount: Decimal = payment
            .try_get::<Option<Decimal>, _>("amount")
            .ok()
            .flatten()
            .unwrap_or_default();
        let pay_crypto_type: String = payment
            .try_get::<Option<String>, _>("crypto_type")
            .ok()
            .flatten()
            .unwrap_or_else(|| "UNKNOWN".to_string());

        // Trigger webhook notification
        let webhook_payload = WebhookPayload {
            event_type: "refund.completed".to_string(),
            payment_id: pay_payment_id,
            merchant_id: r_merchant_id,
            status: PaymentStatus::Refunded,
            amount: pay_amount,
            crypto_type: pay_crypto_type,
            transaction_hash: Some(transaction_hash),
            customer_external_id: None,
            timestamp: Utc::now().timestamp(),
        };

        if let Err(e) = self
            .webhook_service
            .queue_webhook(r_merchant_id, Some(r_payment_id), webhook_payload)
            .await
        {
            error!("Failed to queue webhook for refund {}: {}", refund_id, e);
        }

        Ok(())
    }

    pub async fn get_refund(&self, refund_id: String) -> Result<RefundResponse, ServiceError> {
        let refund = sqlx::query(
            r#"
            SELECT r.refund_id, r.merchant_id, r.payment_id, r.amount, r.amount_usd,
                   r.reason, r.status, r.transaction_hash, r.created_at, r.completed_at,
                   p.payment_id as public_payment_id, p.crypto_type, p.from_address
            FROM refunds r
            JOIN payment_transactions p ON r.payment_id = p.id
            WHERE r.refund_id = $1
            "#,
        )
        .bind(&refund_id)
        .fetch_optional(&self.db_pool)
        .await?
        .ok_or_else(|| ServiceError::Internal("Refund not found".to_string()))?;

        Ok(RefundResponse {
            refund_id: refund.get("refund_id"),
            payment_id: refund.get("public_payment_id"),
            amount: refund.get("amount"),
            amount_usd: refund.get("amount_usd"),
            status: refund.get("status"),
            reason: refund.get("reason"),
            transaction_hash: refund.get("transaction_hash"),
            created_at: refund.get("created_at"),
            completed_at: refund.get("completed_at"),
            crypto_type: refund
                .try_get::<Option<String>, _>("crypto_type")
                .ok()
                .flatten()
                .unwrap_or_else(|| "UNKNOWN".to_string()),
            target_address: refund.get("from_address"),
        })
    }

    pub async fn calculate_merchant_balance(
        &self,
        merchant_id: i64,
    ) -> Result<Decimal, ServiceError> {
        let total_payments_row = sqlx::query(
            r#"
            SELECT COALESCE(SUM(amount_usd), 0) as total
            FROM payment_transactions
            WHERE merchant_id = $1 AND status = 'CONFIRMED'
            "#,
        )
        .bind(merchant_id)
        .fetch_one(&self.db_pool)
        .await?;
        let total_payments: Decimal = total_payments_row
            .try_get::<Option<Decimal>, _>("total")
            .ok()
            .flatten()
            .unwrap_or(Decimal::ZERO);

        let total_refunds_row = sqlx::query(
            r#"
            SELECT COALESCE(SUM(amount_usd), 0) as total
            FROM refunds
            WHERE merchant_id = $1 AND status = 'completed'
            "#,
        )
        .bind(merchant_id)
        .fetch_one(&self.db_pool)
        .await?;
        let total_refunds: Decimal = total_refunds_row
            .try_get::<Option<Decimal>, _>("total")
            .ok()
            .flatten()
            .unwrap_or(Decimal::ZERO);

        let balance = total_payments - total_refunds;

        info!(
            "Calculated balance for merchant {}: ${:.2} (payments: ${:.2}, refunds: ${:.2})",
            merchant_id, balance, total_payments, total_refunds
        );

        Ok(balance)
    }

    pub async fn list_refunds(
        &self,
        merchant_id: i64,
        limit: i64,
        offset: i64,
        is_sandbox: bool,
    ) -> Result<(Vec<RefundResponse>, i64), ServiceError> {
        // 1. Get total count
        let total_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM refunds WHERE merchant_id = $1 AND sandbox_mode = $2",
        )
        .bind(merchant_id)
        .bind(is_sandbox)
        .fetch_one(&self.db_pool)
        .await?;

        // 2. Fetch refund records with payment details joined
        let rows = sqlx::query(
            r#"
            SELECT r.refund_id, r.merchant_id, r.payment_id, r.amount, r.amount_usd,
                   r.reason, r.status, r.transaction_hash, r.created_at, r.completed_at,
                   p.payment_id as public_payment_id, p.crypto_type, p.from_address
            FROM refunds r
            JOIN payment_transactions p ON r.payment_id = p.id
            WHERE r.merchant_id = $1 AND r.sandbox_mode = $2
            ORDER BY r.created_at DESC
            LIMIT $3 OFFSET $4
            "#,
        )
        .bind(merchant_id)
        .bind(is_sandbox)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.db_pool)
        .await?;

        let refunds = rows
            .into_iter()
            .map(|row| RefundResponse {
                refund_id: row.get("refund_id"),
                payment_id: row.get("public_payment_id"),
                amount: row.get("amount"),
                amount_usd: row.get("amount_usd"),
                status: row.get("status"),
                reason: row.get("reason"),
                transaction_hash: row.get("transaction_hash"),
                created_at: row.get("created_at"),
                completed_at: row.get("completed_at"),
                crypto_type: row
                    .try_get::<Option<String>, _>("crypto_type")
                    .ok()
                    .flatten()
                    .unwrap_or_else(|| "UNKNOWN".to_string()),
                target_address: row.get("from_address"),
            })
            .collect();

        Ok((refunds, total_count))
    }
}
