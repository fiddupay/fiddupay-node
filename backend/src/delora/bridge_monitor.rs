// Delora Bridge Monitor
// Background task that polls pending cross-chain bridge completions

use crate::delora::client::DeloraClient;
use crate::delora::error::DeloraError;
use crate::delora::models::{CrossChainPaymentRow, CrossChainPaymentStatus};
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use tracing::{debug, error, info, warn};

pub struct BridgeMonitor {
    db: PgPool,
    poll_interval: Duration,
    max_concurrent_checks: usize,
    stale_threshold_minutes: i64,
}

impl BridgeMonitor {
    pub fn new(
        db: PgPool,
        _client: Arc<DeloraClient>,
        poll_interval_secs: u64,
        max_concurrent_checks: usize,
    ) -> Self {
        Self {
            db,
            poll_interval: Duration::from_secs(poll_interval_secs),
            max_concurrent_checks,
            stale_threshold_minutes: 5,
        }
    }

    pub fn start(self) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            info!(
                "Bridge monitor started — polling every {:?}, max concurrent checks: {}",
                self.poll_interval, self.max_concurrent_checks
            );
            let mut interval = tokio::time::interval(self.poll_interval);

            interval.tick().await;

            loop {
                interval.tick().await;
                if let Err(e) = self.poll_pending_bridges().await {
                    error!(error = %e, "Bridge monitor polling cycle failed");
                }
            }
        })
    }

    async fn poll_pending_bridges(&self) -> Result<(), DeloraError> {
        let pending = sqlx::query_as::<_, CrossChainPaymentRow>(
            r#"SELECT * FROM cross_chain_payments 
               WHERE status IN ('tx_confirmed', 'bridge_pending')
               AND updated_at < NOW() - ($1 || ' minutes')::INTERVAL
               AND deleted_at IS NULL
               ORDER BY updated_at ASC
               LIMIT $2"#,
        )
        .bind(self.stale_threshold_minutes.to_string())
        .bind(self.max_concurrent_checks as i64)
        .fetch_all(&self.db)
        .await?;

        if pending.is_empty() {
            return Ok(());
        }

        debug!(
            "Bridge monitor: checking {} pending cross-chain payments",
            pending.len()
        );

        let semaphore = Arc::new(Semaphore::new(self.max_concurrent_checks));
        let mut handles = Vec::with_capacity(pending.len());

        for payment in pending {
            let permit = semaphore
                .clone()
                .acquire_owned()
                .await
                .map_err(|e| DeloraError::Config(format!("Semaphore closed: {}", e)))?;
            let db = self.db.clone();

            handles.push(tokio::spawn(async move {
                let _permit = permit;
                Self::check_single_bridge(payment, &db).await
            }));
        }

        for handle in handles {
            match handle.await {
                Ok(Ok(())) => {}
                Ok(Err(e)) => {
                    error!(error = %e, "Individual bridge check failed");
                }
                Err(join_err) => {
                    error!(error = %join_err, "Bridge check task panicked");
                }
            }
        }

        Ok(())
    }

    async fn check_single_bridge(
        payment: CrossChainPaymentRow,
        db: &PgPool,
    ) -> Result<(), DeloraError> {
        let payment_id = payment.id;

        if let Some(tx_hash) = &payment.origin_tx_hash {
            if !tx_hash.is_empty() {
                if let Some(estimated_secs) = payment.estimated_time_sec {
                    let since_submission = payment
                        .tx_submitted_at
                        .map(|t| chrono::Utc::now().signed_duration_since(t).num_seconds())
                        .unwrap_or(0);

                    if since_submission >= estimated_secs
                        && payment.status == CrossChainPaymentStatus::TxConfirmed
                    {
                        info!(
                            payment_id = payment_id,
                            "Bridge estimated time elapsed — marking as bridge_complete"
                        );
                        sqlx::query(
                            r#"UPDATE cross_chain_payments 
                               SET status = 'bridge_complete', bridge_completed_at = NOW(), updated_at = NOW()
                               WHERE id = $1 AND status = 'tx_confirmed'"#,
                        )
                        .bind(payment_id)
                        .execute(db)
                        .await?;
                    } else if since_submission > 0
                        && payment.status == CrossChainPaymentStatus::TxConfirmed
                    {
                        sqlx::query(
                            r#"UPDATE cross_chain_payments 
                               SET status = 'bridge_pending', updated_at = NOW()
                               WHERE id = $1 AND status = 'tx_confirmed'"#,
                        )
                        .bind(payment_id)
                        .execute(db)
                        .await?;
                    }
                }
            }
        }

        if payment.status == CrossChainPaymentStatus::BridgeComplete {
            let age_hours = payment
                .tx_submitted_at
                .map(|t| chrono::Utc::now().signed_duration_since(t).num_hours())
                .unwrap_or(0);

            if age_hours >= 1 {
                info!(
                    payment_id = payment_id,
                    "Bridge complete for >1h — marking as completed"
                );
                sqlx::query(
                    r#"UPDATE cross_chain_payments 
                       SET status = 'completed', completed_at = NOW(), updated_at = NOW()
                       WHERE id = $1 AND status = 'bridge_complete'"#,
                )
                .bind(payment_id)
                .execute(db)
                .await?;

                if let Some(pt_id) = payment.payment_transaction_id {
                    sqlx::query(
                        r#"UPDATE payment_transactions 
                           SET status = 'CONFIRMED', confirmed_at = NOW()
                           WHERE id = $1 AND status = 'CONFIRMING'"#,
                    )
                    .bind(pt_id)
                    .execute(db)
                    .await?;
                }
            }
        }

        let age_hours = payment
            .tx_submitted_at
            .map(|t| chrono::Utc::now().signed_duration_since(t).num_hours())
            .unwrap_or(0);

        if age_hours >= 24
            && !matches!(
                payment.status,
                CrossChainPaymentStatus::Completed
                    | CrossChainPaymentStatus::Failed
                    | CrossChainPaymentStatus::Expired
            )
        {
            warn!(
                payment_id = payment_id,
                hours = age_hours,
                "Cross-chain payment stuck >24h — marking as failed"
            );
            sqlx::query(
                r#"UPDATE cross_chain_payments 
                   SET status = 'failed', failed_reason = 'Bridge timed out after 24 hours', updated_at = NOW()
                   WHERE id = $1"#,
            )
            .bind(payment_id)
            .execute(db)
            .await?;

            if let Some(pt_id) = payment.payment_transaction_id {
                sqlx::query(
                    r#"UPDATE payment_transactions 
                       SET status = 'FAILED'
                       WHERE id = $1 AND status = 'CONFIRMING'"#,
                )
                .bind(pt_id)
                .execute(db)
                .await?;
            }
        }

        Ok(())
    }
}
