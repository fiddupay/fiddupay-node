// Analytics Service
// Business logic for analytics and reporting

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::collections::HashMap;

use crate::error::ServiceError;
use crate::models::analytics::{AnalyticsReport, BlockchainStats};
use crate::services::price_service::PriceService;
use crate::payment::models::CryptoType;
use std::sync::Arc;

pub struct AnalyticsService {
    db_pool: PgPool,
    price_service: Arc<PriceService>,
}

impl AnalyticsService {
    pub fn new(db_pool: PgPool, price_service: Arc<PriceService>) -> Self {
        Self { db_pool, price_service }
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
        sandbox_mode: Option<bool>,
    ) -> Result<AnalyticsReport, ServiceError> {
        // Build the base query with optional filters
        let mut query = String::from(
            r#"
            SELECT 
                COALESCE(SUM(CASE WHEN status = 'CONFIRMED' THEN amount_usd ELSE 0 END), 0) as total_volume_usd,
                COUNT(CASE WHEN status = 'CONFIRMED' THEN 1 END) as successful_payments,
                COUNT(CASE WHEN status IN ('FAILED', 'EXPIRED') THEN 1 END) as failed_payments,
                COUNT(CASE WHEN status NOT IN ('CONFIRMED', 'FAILED', 'EXPIRED', 'CANCELLED', 'REFUNDED') THEN 1 END) as pending_payments,
                COUNT(*) as total_payments,
                COALESCE(SUM(CASE WHEN status = 'CONFIRMED' THEN fee_amount_usd ELSE 0 END), 0) as total_fees_paid
            FROM payment_transactions
            WHERE merchant_id = $1
                AND created_at >= $2
                AND created_at <= $3
            "#,
        );

        // Add optional filters
        let mut param_count = 3;
        if sandbox_mode.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND sandbox_mode = ${}", param_count));
        }
        if blockchain.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND network = ${}", param_count));
        }
        if status.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND status = ${}", param_count));
        }

        // Execute the main query
        let mut query_builder = sqlx::query_as::<_, (Decimal, i64, i64, i64, i64, Decimal)>(&query)
            .bind(merchant_id)
            .bind(start_date)
            .bind(end_date);

        if let Some(sb) = sandbox_mode {
            query_builder = query_builder.bind(sb);
        }
        if let Some(ref bc) = blockchain {
            query_builder = query_builder.bind(bc);
        }
        if let Some(ref st) = status {
            query_builder = query_builder.bind(st);
        }

        let (total_volume_usd, successful_payments, failed_payments, pending_payments, total_payments, total_fees_paid) =
            query_builder.fetch_one(&self.db_pool).await?;

        // Calculate average transaction value
        let average_transaction_value = if successful_payments > 0 {
            total_volume_usd / Decimal::from(successful_payments)
        } else {
            Decimal::ZERO
        };

        // Get blockchain-specific stats
        let by_blockchain = self
            .get_blockchain_stats(merchant_id, start_date, end_date, status.clone(), sandbox_mode)
            .await?;

        // Get payment trends (daily)
        let payment_trends = self
            .get_payment_trends(merchant_id, start_date, end_date, sandbox_mode)
            .await?;

        Ok(AnalyticsReport {
            total_volume_usd,
            successful_payments,
            failed_payments,
            pending_payments,
            total_payments,
            total_fees_paid,
            average_transaction_value,
            by_blockchain,
            payment_trends,
        })
    }

    /// Get historical balance points for a merchant
    pub async fn get_balance_history(
        &self,
        merchant_id: i64,
        _limit: i64,
        sandbox_mode: bool,
    ) -> Result<crate::models::analytics::BalanceHistory, ServiceError> {
        // Fetch all confirmed payments and completed withdrawals
        let query = r#"
            (SELECT 
                'payment' as txn_type,
                amount as crypto_amount,
                crypto_type,
                created_at
            FROM payment_transactions
            WHERE merchant_id = $1 AND sandbox_mode = $2 AND status = 'CONFIRMED')
            
            UNION ALL
            
            (SELECT 
                'withdrawal' as txn_type,
                amount as crypto_amount,
                crypto_type,
                created_at
            FROM withdrawals
            WHERE merchant_id = $1 AND sandbox_mode = $2 AND status = 'COMPLETED')
            
            ORDER BY created_at ASC
        "#;

        let rows = sqlx::query(query)
            .bind(merchant_id)
            .bind(sandbox_mode)
            .fetch_all(&self.db_pool)
            .await?;

        // Current prices for USD calculation (simplified: using current price for all historical points)
        // In a real system, we'd use historical prices.
        let prices = self.get_current_prices().await?;

        let mut running_balances: HashMap<String, Decimal> = HashMap::new();
        let mut daily_points: HashMap<String, crate::models::analytics::BalanceTrendPoint> = HashMap::new();
        let mut dates: Vec<String> = Vec::new();

        use sqlx::Row;
        for row in rows {
            let txn_type: String = row.get("txn_type");
            let amount: Decimal = row.get("crypto_amount");
            let crypto_type: String = row.get("crypto_type");
            let created_at: DateTime<Utc> = row.get("created_at");
            let date_str = created_at.format("%Y-%m-%d").to_string();

            let balance = running_balances.entry(crypto_type).or_insert(Decimal::ZERO);
            if txn_type == "payment" {
                *balance += amount;
            } else {
                *balance -= amount;
            }

            // Calculate total USD for this point
            let mut total_usd = Decimal::ZERO;
            for (ct, amt) in &running_balances {
                if let Some(price) = prices.get(ct) {
                    total_usd += amt * price;
                }
            }

            if !daily_points.contains_key(&date_str) {
                dates.push(date_str.clone());
            }

            daily_points.insert(date_str, crate::models::analytics::BalanceTrendPoint {
                date: created_at.format("%Y-%m-%d").to_string(),
                total_usd,
                balances: running_balances.clone(),
            });
        }

        let mut points: Vec<crate::models::analytics::BalanceTrendPoint> = dates.into_iter()
            .map(|d| daily_points.remove(&d).unwrap())
            .collect();

        // Ensure we have at least one point if history exists
        if points.is_empty() {
             return Ok(crate::models::analytics::BalanceHistory { points: vec![] });
        }

        Ok(crate::models::analytics::BalanceHistory { points })
    }

    /// Helper to get current prices for all supported assets using PriceService
    async fn get_current_prices(&self) -> Result<HashMap<String, Decimal>, ServiceError> {
        let crypto_types = vec![
            CryptoType::Sol,
            CryptoType::WSol,
            CryptoType::UsdtSpl,
            CryptoType::Eth,
            CryptoType::UsdtEth,
            CryptoType::Bnb,
            CryptoType::UsdtBep20,
            CryptoType::Matic,
            CryptoType::UsdtPolygon,
            CryptoType::Arb,
            CryptoType::UsdtArbitrum,
        ];

        let mut prices = HashMap::new();
        for ct in crypto_types {
            if let Ok(price) = self.price_service.get_price(ct).await {
                if let Some(price_decimal) = Decimal::from_f64_retain(price) {
                    // Store under the Display format (e.g., "SOL", "USDT_SPL")
                    prices.insert(ct.to_string(), price_decimal);
                }
            }
        }
        Ok(prices)
    }

    /// Get daily payment trends
    async fn get_payment_trends(
        &self,
        merchant_id: i64,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
        sandbox_mode: Option<bool>,
    ) -> Result<Vec<crate::models::analytics::TimeSeriesPoint>, ServiceError> {
        let mut query = String::from(
            r#"
            SELECT 
                DATE(created_at) as date,
                COALESCE(SUM(CASE WHEN status = 'CONFIRMED' THEN amount_usd ELSE 0 END), 0) as volume_usd,
                COUNT(CASE WHEN status = 'CONFIRMED' THEN 1 END) as count
            FROM payment_transactions
            WHERE merchant_id = $1
                AND created_at >= $2
                AND created_at <= $3
            "#,
        );

        let mut param_count = 3;
        if sandbox_mode.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND sandbox_mode = ${}", param_count));
        }

        query.push_str(" GROUP BY DATE(created_at) ORDER BY date ASC");

        let mut query_builder = sqlx::query(&query)
            .bind(merchant_id)
            .bind(start_date)
            .bind(end_date);

        if let Some(sb) = sandbox_mode {
            query_builder = query_builder.bind(sb);
        }

        let rows = query_builder.fetch_all(&self.db_pool).await?;

        use sqlx::Row;
        let points = rows
            .into_iter()
            .map(|row| crate::models::analytics::TimeSeriesPoint {
                date: row.get::<chrono::NaiveDate, _>("date").to_string(),
                volume_usd: row.get::<Decimal, _>("volume_usd"),
                count: row.get::<i64, _>("count"),
            })
            .collect();

        Ok(points)
    }

    /// Get statistics broken down by blockchain
    async fn get_blockchain_stats(
        &self,
        merchant_id: i64,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
        status: Option<String>,
        sandbox_mode: Option<bool>,
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

        let mut param_count = 3;
        if sandbox_mode.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND sandbox_mode = ${}", param_count));
        }
        if status.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND status = ${}", param_count));
        }

        query.push_str(" GROUP BY network");

        let mut query_builder = sqlx::query_as::<_, (String, Decimal, i64)>(&query)
            .bind(merchant_id)
            .bind(start_date)
            .bind(end_date);

        if let Some(sb) = sandbox_mode {
            query_builder = query_builder.bind(sb);
        }
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
        sandbox_mode: Option<bool>,
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
        if sandbox_mode.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND sandbox_mode = ${}", param_count));
        }
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

        if let Some(sb) = sandbox_mode {
            query_builder = query_builder.bind(sb);
        }
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
                row.amount.unwrap_or_default(),
                row.amount_usd,
                escape_csv_field(&row.crypto_type.as_deref().unwrap_or_default()),
                escape_csv_field(&row.network.as_deref().unwrap_or_default()),
                escape_csv_field(&row.transaction_hash.as_deref().unwrap_or_default()),
                escape_csv_field(&row.from_address.as_deref().unwrap_or_default()),
                escape_csv_field(&row.to_address.as_deref().unwrap_or_default()),
                row.fee_percentage,
                row.fee_amount.unwrap_or_default(),
                row.fee_amount_usd,
                escape_csv_field(&row.description.as_deref().unwrap_or_default()),
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
    amount: Option<Decimal>,
    amount_usd: Decimal,
    crypto_type: Option<String>,
    network: Option<String>,
    transaction_hash: Option<String>,
    from_address: Option<String>,
    to_address: Option<String>,
    fee_percentage: Decimal,
    fee_amount: Option<Decimal>,
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
