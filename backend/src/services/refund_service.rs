// Refund Service
// Business logic for refund operations

use chrono::Utc;
use nanoid::nanoid;
use rust_decimal::Decimal;
use sqlx::PgPool;
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

    /// Create a refund for a payment
    /// 
    /// Creates a refund record for a completed payment. Supports both full and partial refunds.
    /// Validates that the refund amount does not exceed the original payment amount.
    /// 
    /// # Arguments
    /// * `merchant_id` - ID of the merchant requesting the refund
    /// * `payment_id` - Public payment ID (e.g., "pay_abc123")
    /// * `amount` - Optional refund amount (None = full refund)
    /// * `reason` - Reason for the refund
    /// 
    /// # Returns
    /// * `RefundResponse` containing refund details
    /// 
    /// # Requirements
    /// * 9.1: Create refund record for completed payment
    /// * 9.2: Support full or partial refund amounts
    /// * 9.3: Validate refund amount does not exceed original payment
    pub async fn create_refund(
        &self,
        merchant_id: i64,
        payment_id: String,
        amount: Option<Decimal>,
        reason: String,
    ) -> Result<RefundResponse, ServiceError> {
        // Fetch the payment to validate it exists and belongs to the merchant
        let payment = sqlx::query!(
            r#"
            SELECT id, merchant_id, amount, amount_usd, crypto_type, status
            FROM payment_transactions
            WHERE payment_id = $1
            "#,
            &payment_id
        )
        .fetch_optional(&self.db_pool)
        .await?
        .ok_or(ServiceError::PaymentNotFound)?;

        // Verify the payment belongs to this merchant
        if payment.merchant_id != merchant_id {
            return Err(ServiceError::PaymentNotFound);
        }

        // Verify the payment is confirmed (can only refund confirmed payments)
        if payment.status != "CONFIRMED" {
            return Err(ServiceError::Internal(
                "Can only refund confirmed payments".to_string()
            ));
        }

        // Calculate total already refunded for this payment
        let total_refunded = sqlx::query!(
            r#"
            SELECT COALESCE(SUM(amount), 0) as total_refunded
            FROM refunds
            WHERE payment_id = $1 AND status IN ('pending', 'completed')
            "#,
            payment.id
        )
        .fetch_one(&self.db_pool)
        .await?
        .total_refunded
        .unwrap_or(Decimal::ZERO);

        // Determine refund amount (full or partial)
        let refund_amount = amount.unwrap_or(payment.amount);

        // Validate refund amount doesn't exceed remaining payment amount
        let remaining_amount = payment.amount - total_refunded;
        if refund_amount > remaining_amount {
            return Err(ServiceError::Internal(format!(
                "Refund amount {} exceeds remaining payment amount {}",
                refund_amount, remaining_amount
            )));
        }

        // Validate refund amount is positive
        if refund_amount <= Decimal::ZERO {
            return Err(ServiceError::Internal(
                "Refund amount must be positive".to_string()
            ));
        }

        // Calculate USD amount for the refund (proportional to original payment)
        let refund_amount_usd = if refund_amount == payment.amount {
            // Full refund - use exact USD amount
            payment.amount_usd
        } else {
            // Partial refund - calculate proportional USD amount
            (payment.amount_usd / payment.amount) * refund_amount
        };

        // Generate unique refund ID
        let refund_id = format!("ref_{}", nanoid!(16));

        // Insert refund record
        let refund = sqlx::query!(
            r#"
            INSERT INTO refunds (
                refund_id, merchant_id, payment_id, amount, amount_usd,
                reason, status, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, refund_id, merchant_id, payment_id, amount, amount_usd,
                      reason, status, transaction_hash, created_at, completed_at
            "#,
            &refund_id,
            merchant_id,
            payment.id,
            refund_amount,
            refund_amount_usd,
            &reason,
            "pending",
            Utc::now()
        )
        .fetch_one(&self.db_pool)
        .await?;

        info!(
            "Created refund {} for payment {} - amount: {} (${:.2})",
            refund_id, payment_id, refund_amount, refund_amount_usd
        );

        Ok(RefundResponse {
            refund_id: refund.refund_id,
            payment_id,
            amount: refund.amount,
            amount_usd: refund.amount_usd,
            status: refund.status,
            reason: refund.reason,
            transaction_hash: refund.transaction_hash,
            created_at: refund.created_at,
            completed_at: refund.completed_at,
        })
    }

    /// Complete a refund with transaction hash
    /// 
    /// Updates a refund record with the blockchain transaction hash and marks it as completed.
    /// Triggers a webhook notification to inform the merchant.
    /// 
    /// # Arguments
    /// * `refund_id` - Public refund ID (e.g., "ref_abc123")
    /// * `transaction_hash` - Blockchain transaction hash for the refund
    /// 
    /// # Returns
    /// * `Ok(())` if the refund was successfully completed
    /// 
    /// # Requirements
    /// * 9.5: Trigger webhook notification on refund
    /// * 9.6: Store refund transaction hash
    pub async fn complete_refund(
        &self,
        refund_id: String,
        transaction_hash: String,
    ) -> Result<(), ServiceError> {
        // Fetch the refund to validate it exists
        let refund = sqlx::query!(
            r#"
            SELECT id, merchant_id, payment_id, status
            FROM refunds
            WHERE refund_id = $1
            "#,
            &refund_id
        )
        .fetch_optional(&self.db_pool)
        .await?
        .ok_or_else(|| ServiceError::Internal("Refund not found".to_string()))?;

        // Verify the refund is still pending
        if refund.status != "pending" {
            return Err(ServiceError::Internal(format!(
                "Refund is already in {} status",
                refund.status
            )));
        }

        // Update refund with transaction hash and mark as completed
        sqlx::query!(
            r#"
            UPDATE refunds
            SET transaction_hash = $1, status = $2, completed_at = $3
            WHERE refund_id = $4
            "#,
            &transaction_hash,
            "completed",
            Utc::now(),
            &refund_id
        )
        .execute(&self.db_pool)
        .await?;

        info!(
            "Completed refund {} with transaction hash: {}",
            refund_id, transaction_hash
        );

        // Fetch payment details for webhook
        let payment = sqlx::query!(
            r#"
            SELECT payment_id, amount, crypto_type
            FROM payment_transactions
            WHERE id = $1
            "#,
            refund.payment_id
        )
        .fetch_one(&self.db_pool)
        .await?;

        // Trigger webhook notification
        let webhook_payload = WebhookPayload {
            event_type: "refund.completed".to_string(),
            payment_id: payment.payment_id,
            merchant_id: refund.merchant_id,
            status: PaymentStatus::Refunded,
            amount: payment.amount,
            crypto_type: payment.crypto_type,
            transaction_hash: Some(transaction_hash),
            timestamp: Utc::now().timestamp(),
        };

        // Queue webhook for delivery (don't fail if webhook fails)
        if let Err(e) = self
            .webhook_service
            .queue_webhook(refund.merchant_id, refund.payment_id, webhook_payload)
            .await
        {
            error!("Failed to queue webhook for refund {}: {}", refund_id, e);
        }

        Ok(())
    }

    /// Get refund details
    /// 
    /// Retrieves the details of a specific refund.
    /// 
    /// # Arguments
    /// * `refund_id` - Public refund ID (e.g., "ref_abc123")
    /// 
    /// # Returns
    /// * `RefundResponse` containing refund details
    pub async fn get_refund(&self, refund_id: String) -> Result<RefundResponse, ServiceError> {
        let refund = sqlx::query!(
            r#"
            SELECT r.refund_id, r.merchant_id, r.payment_id, r.amount, r.amount_usd,
                   r.reason, r.status, r.transaction_hash, r.created_at, r.completed_at,
                   p.payment_id as public_payment_id
            FROM refunds r
            JOIN payment_transactions p ON r.payment_id = p.id
            WHERE r.refund_id = $1
            "#,
            &refund_id
        )
        .fetch_optional(&self.db_pool)
        .await?
        .ok_or_else(|| ServiceError::Internal("Refund not found".to_string()))?;

        Ok(RefundResponse {
            refund_id: refund.refund_id,
            payment_id: refund.public_payment_id,
            amount: refund.amount,
            amount_usd: refund.amount_usd,
            status: refund.status,
            reason: refund.reason,
            transaction_hash: refund.transaction_hash,
            created_at: refund.created_at,
            completed_at: refund.completed_at,
        })
    }

    /// Calculate merchant balance with refunds
    /// 
    /// Calculates the total balance for a merchant by summing confirmed payments
    /// and subtracting completed refunds.
    /// 
    /// # Arguments
    /// * `merchant_id` - ID of the merchant
    /// 
    /// # Returns
    /// * Total balance in USD after accounting for refunds
    /// 
    /// # Requirements
    /// * 9.7: Subtract refunded amounts from total when calculating merchant balances
    pub async fn calculate_merchant_balance(
        &self,
        merchant_id: i64,
    ) -> Result<Decimal, ServiceError> {
        // Sum of all confirmed payments
        let total_payments = sqlx::query!(
            r#"
            SELECT COALESCE(SUM(amount_usd), 0) as total
            FROM payment_transactions
            WHERE merchant_id = $1 AND status = 'CONFIRMED'
            "#,
            merchant_id
        )
        .fetch_one(&self.db_pool)
        .await?
        .total
        .unwrap_or(Decimal::ZERO);

        // Sum of all completed refunds
        let total_refunds = sqlx::query!(
            r#"
            SELECT COALESCE(SUM(amount_usd), 0) as total
            FROM refunds
            WHERE merchant_id = $1 AND status = 'completed'
            "#,
            merchant_id
        )
        .fetch_one(&self.db_pool)
        .await?
        .total
        .unwrap_or(Decimal::ZERO);

        // Calculate net balance
        let balance = total_payments - total_refunds;

        info!(
            "Calculated balance for merchant {}: ${:.2} (payments: ${:.2}, refunds: ${:.2})",
            merchant_id, balance, total_payments, total_refunds
        );

        Ok(balance)
    }
}
