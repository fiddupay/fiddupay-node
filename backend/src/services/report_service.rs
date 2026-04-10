use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::{PgPool, FromRow};
use serde::Serialize;
use std::io::Cursor;
use csv::Writer;
use crate::error::ServiceError;

#[derive(Debug, FromRow, Serialize)]
pub struct PaymentReportRow {
    pub payment_id: String,
    pub status: String,
    pub amount: Option<Decimal>,
    pub amount_usd: Decimal,
    pub crypto_type: Option<String>,
    pub network: Option<String>,
    pub transaction_hash: Option<String>,
    pub from_address: Option<String>,
    pub to_address: Option<String>,
    pub fee_amount: Option<Decimal>,
    pub fee_amount_usd: Decimal,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub confirmed_at: Option<DateTime<Utc>>,
}

pub struct ReportService {
    db_pool: PgPool,
}

impl ReportService {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }

    pub async fn get_payment_data(
        &self,
        merchant_id: i64,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
        blockchain: Option<String>,
        status: Option<String>,
        sandbox_mode: Option<bool>,
    ) -> Result<Vec<PaymentReportRow>, ServiceError> {
        let mut query = String::from(
            r#"
            SELECT 
                payment_id, status, amount, amount_usd, crypto_type, network,
                transaction_hash, from_address, to_address, fee_amount, fee_amount_usd,
                description, created_at, confirmed_at
            FROM payment_transactions
            WHERE merchant_id = $1
                AND created_at >= $2
                AND created_at <= $3
            "#,
        );

        let mut param_count = 3;
        if let Some(_) = sandbox_mode {
            param_count += 1;
            query.push_str(&format!(" AND sandbox_mode = ${}", param_count));
        }
        if let Some(_) = blockchain {
            param_count += 1;
            query.push_str(&format!(" AND network = ${}", param_count));
        }
        if let Some(_) = status {
            param_count += 1;
            query.push_str(&format!(" AND status = ${}", param_count));
        }

        query.push_str(" ORDER BY created_at DESC");

        let mut query_builder = sqlx::query_as::<_, PaymentReportRow>(&query)
            .bind(merchant_id)
            .bind(start_date)
            .bind(end_date);

        if let Some(sb) = sandbox_mode {
            query_builder = query_builder.bind(sb);
        }
        if let Some(bc) = blockchain {
            query_builder = query_builder.bind(bc);
        }
        if let Some(st) = status {
            query_builder = query_builder.bind(st);
        }

        let rows = query_builder.fetch_all(&self.db_pool).await?;
        Ok(rows)
    }

    pub async fn generate_csv(
        &self,
        data: Vec<PaymentReportRow>,
    ) -> Result<Vec<u8>, ServiceError> {
        let mut wtr = Writer::from_writer(vec![]);
        
        for row in data {
            wtr.serialize(row).map_err(|e| ServiceError::InternalError(format!("CSV serialization failed: {}", e)))?;
        }
        
        let inner = wtr.into_inner().map_err(|e| ServiceError::InternalError(format!("CSV generation failed: {}", e)))?;
        Ok(inner)
    }

    pub async fn generate_pdf(
        &self,
        merchant_name: &str,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
        data: Vec<PaymentReportRow>,
    ) -> Result<Vec<u8>, ServiceError> {
        // Placeholder for PDF generation utilizing genpdf
        // For now, we return a simple notice or the data as a string in bytes
        // until we can verify font availability or use printpdf builtins.
        
        // TODO: Full implementation with genpdf once font strategy is finalized.
        let mut content = format!("FidduPay Transaction Report\n");
        content.push_str(&format!("Merchant: {}\n", merchant_name));
        content.push_str(&format!("Period: {} to {}\n\n", start_date, end_date));
        
        for row in data {
            content.push_str(&format!("{} | {} | {} USD | {}\n", row.created_at, row.payment_id, row.amount_usd, row.status));
        }
        
        Ok(content.into_bytes())
    }
}
