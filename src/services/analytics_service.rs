// Analytics Service
// Business logic for analytics and reporting

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::collections::HashMap;

use crate::error::ServiceError;
use crate::models::analytics::{AnalyticsReport, BlockchainStats};

pub struct AnalyticsService {
    db_pool: PgPool,
}

impl AnalyticsService {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }

    /// Get analytics for a merchant within a date range
    /// Supports filtering by blockchain and payment status
    pub async fn get_analytics(
        &self,
        merchant_id: i64,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
        blockchain: Option<String>,
        status: Option<String>,
    ) -> Result<AnalyticsReport, ServiceError> {
        // Build the base query with optional filters
        let mut query = String::from(
            r#"
            SELECT 
                COALESCE(SUM(CASE WHEN status = 'CONFIRMED' THEN amount_usd ELSE 0 END), 0) as total_volume_usd,
                COUNT(CASE WHEN status = 'CONFIRMED' THEN 1 END) as successful_payments,
                COUNT(CASE WHEN status IN ('FAILED', 'EXPIRED') THEN 1 END) as failed_payments,
                COALESCE(SUM(CASE WHEN status = 'CONFIRMED' THEN fee_amount_usd ELSE 0 END), 0) as total_fees_paid
            FROM payment_transactions
            WHERE merchant_id = $1
                AND created_at >= $2
                AND created_at <= $3
            "#,
        );

        // Add optional filters
        let mut param_count = 3;
        if blockchain.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND network = ${}", param_count));
        }
        if status.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND status = ${}", param_count));
        }

        // Execute the main query
        let mut query_builder = sqlx::query_as::<_, (Decimal, i64, i64, Decimal)>(&query)
            .bind(merchant_id)
            .bind(start_date)
            .bind(end_date);

        if let Some(ref bc) = blockchain {
            query_builder = query_builder.bind(bc);
        }
        if let Some(ref st) = status {
            query_builder = query_builder.bind(st);
        }

        let (total_volume_usd, successful_payments, failed_payments, total_fees_paid) =
            query_builder.fetch_one(&self.db_pool).await?;

        // Calculate average transaction value
        let average_transaction_value = if successful_payments > 0 {
            total_volume_usd / Decimal::from(successful_payments)
        } else {
            Decimal::ZERO
        };

        // Get blockchain-specific stats
        let by_blockchain = self
            .get_blockchain_stats(merchant_id, start_date, end_date, status.clone())
            .await?;

        Ok(AnalyticsReport {
            total_volume_usd,
            successful_payments,
            failed_payments,
            total_fees_paid,
            average_transaction_value,
            by_blockchain,
        })
    }

    /// Get statistics broken down by blockchain
    async fn get_blockchain_stats(
        &self,
        merchant_id: i64,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
        status: Option<String>,
    ) -> Result<HashMap<String, BlockchainStats>, ServiceError> {
        let mut query = String::from(
            r#"
            SELECT 
                network,
                COALESCE(SUM(CASE WHEN status = 'CONFIRMED' THEN amount_usd ELSE 0 END), 0) as volume_usd,
                COUNT(CASE WHEN status = 'CONFIRMED' THEN 1 END) as payment_count
            FROM payment_transactions
            WHERE merchant_id = $1
                AND created_at >= $2
                AND created_at <= $3
            "#,
        );

        if status.is_some() {
            query.push_str(" AND status = $4");
        }

        query.push_str(" GROUP BY network");

        let mut query_builder = sqlx::query_as::<_, (String, Decimal, i64)>(&query)
            .bind(merchant_id)
            .bind(start_date)
            .bind(end_date);

        if let Some(st) = status {
            query_builder = query_builder.bind(st);
        }

        let rows = query_builder.fetch_all(&self.db_pool).await?;

        let mut by_blockchain = HashMap::new();
        for (network, volume_usd, payment_count) in rows {
            let average_value = if payment_count > 0 {
                volume_usd / Decimal::from(payment_count)
            } else {
                Decimal::ZERO
            };

            by_blockchain.insert(
                network,
                BlockchainStats {
                    volume_usd,
                    payment_count,
                    average_value,
                },
            );
        }

        Ok(by_blockchain)
    }

    /// Export payment data as CSV
    /// Returns CSV string with all payment details for the specified date range
    pub async fn export_csv(
        &self,
        merchant_id: i64,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
        blockchain: Option<String>,
        status: Option<String>,
    ) -> Result<String, ServiceError> {
        // Build query to fetch payment details
        let mut query = String::from(
            r#"
            SELECT 
                payment_id,
                status,
                amount,
                amount_usd,
                crypto_type,
                network,
                transaction_hash,
                from_address,
                to_address,
                fee_percentage,
                fee_amount,
                fee_amount_usd,
                description,
                created_at,
                confirmed_at,
                expires_at
            FROM payment_transactions
            WHERE merchant_id = $1
                AND created_at >= $2
                AND created_at <= $3
            "#,
        );

        // Add optional filters
        let mut param_count = 3;
        if blockchain.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND network = ${}", param_count));
        }
        if status.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND status = ${}", param_count));
        }

        query.push_str(" ORDER BY created_at DESC");

        let mut query_builder = sqlx::query_as::<_, PaymentCsvRow>(&query)
            .bind(merchant_id)
            .bind(start_date)
            .bind(end_date);

        if let Some(ref bc) = blockchain {
            query_builder = query_builder.bind(bc);
        }
        if let Some(ref st) = status {
            query_builder = query_builder.bind(st);
        }

        let rows = query_builder.fetch_all(&self.db_pool).await?;

        // Build CSV string
        let mut csv = String::from(
            "Payment ID,Status,Amount,Amount USD,Crypto Type,Network,Transaction Hash,From Address,To Address,Fee Percentage,Fee Amount,Fee Amount USD,Description,Created At,Confirmed At,Expires At\n"
        );

        for row in rows {
            csv.push_str(&format!(
                "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}\n",
                escape_csv_field(&row.payment_id),
                escape_csv_field(&row.status),
                row.amount,
                row.amount_usd,
                escape_csv_field(&row.crypto_type),
                escape_csv_field(&row.network),
                escape_csv_field(&row.transaction_hash.unwrap_or_default()),
                escape_csv_field(&row.from_address.unwrap_or_default()),
                escape_csv_field(&row.to_address),
                row.fee_percentage,
                row.fee_amount,
                row.fee_amount_usd,
                escape_csv_field(&row.description.unwrap_or_default()),
                row.created_at.to_rfc3339(),
                row.confirmed_at.map(|dt| dt.to_rfc3339()).unwrap_or_default(),
                row.expires_at.to_rfc3339(),
            ));
        }

        Ok(csv)
    }
}

/// Helper struct for CSV export query results
#[derive(sqlx::FromRow)]
struct PaymentCsvRow {
    payment_id: String,
    status: String,
    amount: Decimal,
    amount_usd: Decimal,
    crypto_type: String,
    network: String,
    transaction_hash: Option<String>,
    from_address: Option<String>,
    to_address: String,
    fee_percentage: Decimal,
    fee_amount: Decimal,
    fee_amount_usd: Decimal,
    description: Option<String>,
    created_at: DateTime<Utc>,
    confirmed_at: Option<DateTime<Utc>>,
    expires_at: DateTime<Utc>,
}

/// Escape CSV field to handle commas, quotes, and newlines
fn escape_csv_field(field: &str) -> String {
    if field.contains(',') || field.contains('"') || field.contains('\n') {
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field.to_string()
    }
}
