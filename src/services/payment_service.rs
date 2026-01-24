// Payment Service
// Business logic for payment operations

use crate::error::ServiceError;
use crate::payment::models::{
    CreatePaymentRequest, PaymentFilters, PaymentList, PaymentResponse, PaymentStatus,
    PaymentTransaction, PartialPaymentInfo, PartialPaymentRecord, CryptoType,
};
use crate::payment::processor::PaymentProcessor;
use crate::payment::verifier::PaymentVerifier;
use crate::services::{webhook_service::WebhookService, price_service::PriceService};
use std::sync::Arc;
use chrono::Utc;
use rust_decimal::Decimal;
use sqlx::PgPool;

#[derive(Debug, thiserror::Error)]
pub enum PaymentServiceError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    #[error("Payment not found")]
    PaymentNotFound,
    #[error("Invalid filter parameters: {0}")]
    InvalidFilters(String),
    #[error("Service error: {0}")]
    ServiceError(#[from] ServiceError),
    #[error("Verification error: {0}")]
    VerificationError(String),
    #[error("Payment has expired")]
    PaymentExpired,
    #[error("Payment already confirmed")]
    PaymentAlreadyConfirmed,
}

pub struct PaymentService {
    db_pool: PgPool,
    processor: PaymentProcessor,
    verifier: PaymentVerifier,
}

impl PaymentService {
    pub fn new(db_pool: PgPool, payment_page_base_url: String, price_service: Arc<PriceService>, webhook_signing_key: String) -> Self {
        let webhook_service = WebhookService::new(db_pool.clone(), webhook_signing_key);
        
        Self {
            processor: PaymentProcessor::new(db_pool.clone(), payment_page_base_url, price_service),
            verifier: PaymentVerifier::new(db_pool.clone(), webhook_service),
            db_pool,
        }
    }

    /// Create a new payment request
    /// 
    /// # Arguments
    /// * `merchant_id` - The merchant creating the payment
    /// * `request` - Payment creation request details
    /// 
    /// # Returns
    /// * `PaymentResponse` with payment details
    /// 
    /// # Requirements
    /// * 2.1: Generate unique payment identifier
    /// * 2.2: Calculate crypto amount using real-time exchange rates
    /// * 2.6: Include platform fee in total amount
    pub async fn create_payment(
        &self,
        merchant_id: i64,
        request: CreatePaymentRequest,
    ) -> Result<PaymentResponse, PaymentServiceError> {
        Ok(self.processor.create_payment(merchant_id, request).await?)
    }

    /// Verify a payment with transaction hash
    /// 
    /// # Arguments
    /// * `payment_id` - Public payment ID (e.g., "pay_abc123")
    /// * `transaction_hash` - Blockchain transaction hash
    /// * `merchant_id` - Merchant ID for ownership verification
    /// 
    /// # Returns
    /// * `true` if payment is confirmed
    /// * `false` if payment is pending more confirmations
    /// 
    /// # Requirements
    /// * 3.1: Verify transaction hash exists on blockchain
    /// * 3.2: Confirm amount matches expected payment amount
    /// * 3.3: Confirm recipient address matches merchant's wallet
    pub async fn verify_payment(
        &self,
        payment_id: &str,
        transaction_hash: &str,
        merchant_id: i64,
    ) -> Result<bool, PaymentServiceError> {
        self.verifier
            .verify_payment(payment_id, transaction_hash, merchant_id)
            .await
            .map_err(|e| PaymentServiceError::VerificationError(e.to_string()))
    }

    /// Get a single payment by payment ID
    /// 
    /// # Arguments
    /// * `payment_id` - Public payment ID (e.g., "pay_abc123")
    /// * `merchant_id` - Merchant ID for ownership verification
    /// 
    /// # Returns
    /// * `PaymentResponse` with payment details
    pub async fn get_payment(
        &self,
        payment_id: &str,
        merchant_id: i64,
    ) -> Result<PaymentResponse, PaymentServiceError> {
        let payment = sqlx::query_as::<_, PaymentTransaction>(
            "SELECT * FROM payment_transactions WHERE payment_id = $1"
        )
        .bind(payment_id)
        .fetch_optional(&self.db_pool)
        .await?
        .ok_or(PaymentServiceError::PaymentNotFound)?;

        if payment.merchant_id != merchant_id {
            return Err(PaymentServiceError::PaymentNotFound);
        }

        self.convert_to_response(payment).await
    }

    /// List payments for a merchant with optional filters and pagination
    /// 
    /// # Arguments
    /// * `merchant_id` - The merchant ID to filter payments for
    /// * `filters` - Optional filters for status, blockchain, date range, and pagination
    /// 
    /// # Returns
    /// * `PaymentList` - Paginated list of payments with total count
    /// 
    /// # Requirements
    /// Validates: Requirements 11.3 - Support filtering analytics by date range, blockchain, and payment status
    pub async fn list_payments(
        &self,
        merchant_id: i64,
        filters: PaymentFilters,
    ) -> Result<PaymentList, PaymentServiceError> {
        // Validate and set pagination parameters
        let page = filters.page.unwrap_or(1).max(1);
        let page_size = filters.page_size.unwrap_or(20).min(100).max(1);
        let offset = ((page - 1) * page_size) as i64;

        // Build the base query
        let mut query = String::from(
            "SELECT * FROM payment_transactions WHERE merchant_id = $1"
        );
        let mut param_count = 1;

        // Add status filter
        if filters.status.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND status = ${}", param_count));
        }

        // Add blockchain filter
        if filters.blockchain.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND network = ${}", param_count));
        }

        // Add date range filters
        if filters.from_date.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND created_at >= ${}", param_count));
        }

        if filters.to_date.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND created_at <= ${}", param_count));
        }

        // Add ordering and pagination
        query.push_str(" ORDER BY created_at DESC");
        param_count += 1;
        query.push_str(&format!(" LIMIT ${}", param_count));
        param_count += 1;
        query.push_str(&format!(" OFFSET ${}", param_count));

        // Build the query with parameters
        let mut sql_query = sqlx::query_as::<_, PaymentTransaction>(&query)
            .bind(merchant_id);

        // Bind status filter
        if let Some(status) = filters.status {
            let status_str = match status {
                PaymentStatus::Pending => "PENDING",
                PaymentStatus::Confirming => "CONFIRMING",
                PaymentStatus::Confirmed => "CONFIRMED",
                PaymentStatus::Failed => "FAILED",
                PaymentStatus::Refunded => "REFUNDED",
            };
            sql_query = sql_query.bind(status_str);
        }

        // Bind blockchain filter
        if let Some(blockchain) = &filters.blockchain {
            sql_query = sql_query.bind(blockchain);
        }

        // Bind date filters
        if let Some(from_date) = filters.from_date {
            sql_query = sql_query.bind(from_date);
        }

        if let Some(to_date) = filters.to_date {
            sql_query = sql_query.bind(to_date);
        }

        // Bind pagination
        sql_query = sql_query.bind(page_size as i64).bind(offset);

        // Execute the query
        let payments = sql_query.fetch_all(&self.db_pool).await?;

        // Get total count for pagination
        let total = self.count_payments(merchant_id, &filters).await?;

        // Convert PaymentTransaction to PaymentResponse
        let mut payment_responses = Vec::new();
        for payment in payments {
            let payment_response = self.convert_to_response(payment).await?;
            payment_responses.push(payment_response);
        }

        // Calculate total pages
        let total_pages = ((total as f64) / (page_size as f64)).ceil() as u32;

        Ok(PaymentList {
            payments: payment_responses,
            total,
            page,
            page_size,
            total_pages,
        })
    }

    /// Count total payments matching the filters
    async fn count_payments(
        &self,
        merchant_id: i64,
        filters: &PaymentFilters,
    ) -> Result<i64, PaymentServiceError> {
        let mut query = String::from(
            "SELECT COUNT(*) FROM payment_transactions WHERE merchant_id = $1"
        );
        let mut param_count = 1;

        // Add status filter
        if filters.status.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND status = ${}", param_count));
        }

        // Add blockchain filter
        if filters.blockchain.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND network = ${}", param_count));
        }

        // Add date range filters
        if filters.from_date.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND created_at >= ${}", param_count));
        }

        if filters.to_date.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND created_at <= ${}", param_count));
        }

        // Build the query with parameters
        let mut sql_query = sqlx::query_scalar::<_, i64>(&query)
            .bind(merchant_id);

        // Bind status filter
        if let Some(status) = filters.status {
            let status_str = match status {
                PaymentStatus::Pending => "PENDING",
                PaymentStatus::Confirming => "CONFIRMING",
                PaymentStatus::Confirmed => "CONFIRMED",
                PaymentStatus::Failed => "FAILED",
                PaymentStatus::Refunded => "REFUNDED",
            };
            sql_query = sql_query.bind(status_str);
        }

        // Bind blockchain filter
        if let Some(blockchain) = &filters.blockchain {
            sql_query = sql_query.bind(blockchain);
        }

        // Bind date filters
        if let Some(from_date) = filters.from_date {
            sql_query = sql_query.bind(from_date);
        }

        if let Some(to_date) = filters.to_date {
            sql_query = sql_query.bind(to_date);
        }

        let count = sql_query.fetch_one(&self.db_pool).await?;
        Ok(count)
    }

    /// Convert PaymentTransaction to PaymentResponse
    async fn convert_to_response(
        &self,
        payment: PaymentTransaction,
    ) -> Result<PaymentResponse, PaymentServiceError> {
        // Parse crypto type from string
        let crypto_type = self.parse_crypto_type(&payment.crypto_type);

        // Parse status from string
        let status = self.parse_status(&payment.status);

        // Get partial payment info if enabled
        let partial_payments = if payment.partial_payments_enabled {
            let partial_records = self.get_partial_payments(payment.id).await?;
            Some(PartialPaymentInfo {
                enabled: true,
                total_paid: payment.total_paid,
                remaining_balance: payment.remaining_balance.unwrap_or(Decimal::ZERO),
                payments: partial_records,
            })
        } else {
            None
        };

        // Fetch payment link from database
        let payment_link = match sqlx::query!(
            "SELECT link_id FROM payment_links WHERE payment_id = $1",
            payment.id
        )
        .fetch_optional(&self.db_pool)
        .await? {
            Some(record) => format!("{}/pay/{}", 
                std::env::var("PAYMENT_PAGE_BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string()),
                record.link_id
            ),
            None => format!("{}/pay/{}", 
                std::env::var("PAYMENT_PAGE_BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string()),
                payment.payment_id
            ),
        };

        // Generate QR code data
        let qr_code_data = format!(
            "{}:{}?amount={}",
            crypto_type.network().to_lowercase(),
            payment.to_address,
            payment.amount
        );

        Ok(PaymentResponse {
            payment_id: payment.payment_id,
            status,
            amount: payment.amount,
            amount_usd: payment.amount_usd,
            crypto_type,
            network: payment.network,
            deposit_address: payment.to_address,
            payment_link,
            qr_code_data,
            fee_amount: payment.fee_amount,
            fee_amount_usd: payment.fee_amount_usd,
            expires_at: payment.expires_at,
            created_at: payment.created_at,
            confirmed_at: payment.confirmed_at,
            transaction_hash: payment.transaction_hash,
            partial_payments,
        })
    }

    /// Get partial payment records for a payment
    async fn get_partial_payments(
        &self,
        payment_id: i64,
    ) -> Result<Vec<PartialPaymentRecord>, PaymentServiceError> {
        let records = sqlx::query_as::<_, PartialPaymentRecord>(
            "SELECT * FROM partial_payments WHERE payment_id = $1 ORDER BY created_at ASC"
        )
        .bind(payment_id)
        .fetch_all(&self.db_pool)
        .await?;

        Ok(records)
    }

    /// Parse crypto type from string
    fn parse_crypto_type(&self, crypto_type_str: &str) -> CryptoType {
        match crypto_type_str {
            "USDT_BEP20" => CryptoType::UsdtBep20,
            "USDT_ARBITRUM" => CryptoType::UsdtArbitrum,
            "USDT_SPL" => CryptoType::UsdtSpl,
            "USDT_POLYGON" => CryptoType::UsdtPolygon,
            "USDT_ETH" => CryptoType::UsdtEth,
            "SOL" => CryptoType::Sol,
            "ETH" => CryptoType::Eth,
            "ARB" => CryptoType::Arb,
            "MATIC" => CryptoType::Matic,
            "BNB" => CryptoType::Bnb,
            _ => CryptoType::Sol, // Default fallback
        }
    }

    /// Parse payment status from string
    fn parse_status(&self, status_str: &str) -> PaymentStatus {
        match status_str {
            "PENDING" => PaymentStatus::Pending,
            "CONFIRMING" => PaymentStatus::Confirming,
            "CONFIRMED" => PaymentStatus::Confirmed,
            "FAILED" => PaymentStatus::Failed,
            "REFUNDED" => PaymentStatus::Refunded,
            _ => PaymentStatus::Pending, // Default fallback
        }
    }
}
