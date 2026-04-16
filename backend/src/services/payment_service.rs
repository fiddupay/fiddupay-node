// Payment Service
// Business logic for payment operations

use crate::error::ServiceError;
use crate::payment::models::{
    CreatePaymentRequest, CryptoType, PaymentFilters, PaymentList, PaymentResponse, PaymentStatus,
    PaymentTransaction,
};
use crate::payment::processor::PaymentProcessor;
use crate::payment::verifier::PaymentVerifier;
use crate::services::invoice_service::InvoiceService;
use crate::services::notification_service::NotificationService;
use crate::services::price_service::PriceService;
use crate::services::webhook_service::WebhookService;
// Removed unused imports: use chrono::Utc; use rust_decimal::Decimal;
use sqlx::PgPool;
use std::sync::Arc;

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

impl axum::response::IntoResponse for PaymentServiceError {
    fn into_response(self) -> axum::response::Response {
        let service_err = match self {
            PaymentServiceError::DatabaseError(e) => ServiceError::Database(e),
            PaymentServiceError::PaymentNotFound => ServiceError::PaymentNotFound,
            PaymentServiceError::ServiceError(e) => e,
            PaymentServiceError::InvalidFilters(msg) => ServiceError::ValidationError(msg),
            PaymentServiceError::VerificationError(msg) => ServiceError::ValidationError(msg),
            PaymentServiceError::PaymentExpired => {
                ServiceError::ValidationError("Payment has expired".to_string())
            }
            PaymentServiceError::PaymentAlreadyConfirmed => {
                ServiceError::ValidationError("Payment already confirmed".to_string())
            }
        };
        service_err.into_response()
    }
}

pub struct PaymentService {
    db_pool: PgPool,
    processor: PaymentProcessor,
    verifier: PaymentVerifier,
    config: crate::config::Config,
}

pub struct PaymentServiceConfig {
    pub db_pool: PgPool,
    pub payment_page_base_url: String,
    pub price_service: Arc<PriceService>,
    pub invoice_service: Arc<InvoiceService>,
    pub audit_service: Arc<crate::services::audit_service::AuditService>,
    pub webhook_signing_key: String,
    pub config: crate::config::Config,
    pub redis_client: redis::Client,
    pub volume_tracking: Arc<crate::services::volume_tracking_service::VolumeTrackingService>,
    pub notification_service: Arc<NotificationService>,
}

impl PaymentService {
    pub fn new(deps: PaymentServiceConfig) -> Self {
        let webhook_service =
            WebhookService::new(deps.db_pool.clone(), deps.webhook_signing_key.clone());

        Self {
            processor: PaymentProcessor::new(crate::payment::processor::PaymentProcessorConfig {
                db_pool: deps.db_pool.clone(),
                payment_page_base_url: deps.payment_page_base_url,
                price_service: deps.price_service.clone(),
                invoice_service: deps.invoice_service.clone(),
                audit_service: deps.audit_service,
                config: deps.config.clone(),
                volume_tracking: deps.volume_tracking,
                notification_service: deps.notification_service.clone(),
            }),
            verifier: PaymentVerifier::new(
                deps.db_pool.clone(),
                webhook_service,
                deps.price_service,
                deps.config.clone(),
                deps.redis_client,
                deps.notification_service.clone(),
            ),
            db_pool: deps.db_pool,
            config: deps.config,
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

    /// Cancel a pending payment
    pub async fn cancel_payment(
        &self,
        merchant_id: i64,
        payment_id: &str,
    ) -> Result<(), PaymentServiceError> {
        Ok(self
            .processor
            .cancel_payment(merchant_id, payment_id)
            .await?)
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

    /// Verify payment by scanning address
    ///
    /// # Arguments
    /// * `payment_id` - Public payment ID
    /// * `merchant_id` - Merchant ID
    ///
    /// # Returns
    /// * `true` if payment confirmed
    pub async fn verify_payment_by_address(
        &self,
        payment_id: &str,
        merchant_id: i64,
    ) -> Result<bool, PaymentServiceError> {
        self.verifier
            .verify_payment_by_address(payment_id, merchant_id)
            .await
            .map_err(|e| PaymentServiceError::VerificationError(e.to_string()))
    }

    /// Verify a customer static deposit
    pub async fn verify_customer_deposit(
        &self,
        customer_id: i64,
        tx_hash: &str,
        merchant_id: i64,
        crypto_type: &str,
        is_sandbox: bool,
    ) -> Result<bool, PaymentServiceError> {
        self.verifier
            .verify_customer_deposit(customer_id, tx_hash, merchant_id, crypto_type, is_sandbox)
            .await
            .map_err(|e| PaymentServiceError::VerificationError(e.to_string()))
    }

    /// Verify a merchant static deposit
    pub async fn verify_merchant_deposit(
        &self,
        merchant_id: i64,
        tx_hash: &str,
        crypto_type: &str,
        is_sandbox: bool,
    ) -> Result<bool, PaymentServiceError> {
        self.verifier
            .verify_merchant_deposit(merchant_id, tx_hash, crypto_type, is_sandbox)
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
            "SELECT * FROM payment_transactions WHERE payment_id = $1",
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
        let page = filters.page.unwrap_or(1).max(1);
        let page_size = filters.page_size.unwrap_or(20).clamp(1, 100);
        let offset = (page - 1) * page_size;

        // Build the base query
        let mut query = String::from("SELECT * FROM payment_transactions WHERE merchant_id = $1");
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

        // Add sandbox filter
        if filters.is_sandbox.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND sandbox_mode = ${}", param_count));
        }

        // Add ordering and pagination
        query.push_str(" ORDER BY created_at DESC");
        param_count += 1;
        query.push_str(&format!(" LIMIT ${}", param_count));
        param_count += 1;
        query.push_str(&format!(" OFFSET ${}", param_count));

        // Build the query with parameters
        let mut sql_query = sqlx::query_as::<_, PaymentTransaction>(&query).bind(merchant_id);

        // Bind status filter
        if let Some(status) = &filters.status {
            sql_query = sql_query.bind(status);
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

        // Bind sandbox filter
        if let Some(is_sandbox) = filters.is_sandbox {
            sql_query = sql_query.bind(is_sandbox);
        }

        sql_query = sql_query.bind(page_size).bind(offset);

        // Execute queries in parallel
        let (payments_res, total_res) = tokio::join!(
            sql_query.fetch_all(&self.db_pool),
            self.count_payments(merchant_id, &filters)
        );

        let payments = payments_res?;
        let total = total_res?;

        // Convert PaymentTransaction to PaymentResponse
        let mut payment_responses = Vec::new();
        for payment in payments {
            let payment_response = self.convert_to_response(payment).await?;
            payment_responses.push(payment_response);
        }

        // Calculate total pages
        let total_pages = ((total as f64) / (page_size as f64)).ceil() as i64;

        Ok(PaymentList {
            data: payment_responses,
            pagination: crate::payment::models::PaginationInfo {
                page,
                page_size,
                total_pages,
                total_count: total,
            },
        })
    }

    /// Count total payments matching the filters
    async fn count_payments(
        &self,
        merchant_id: i64,
        filters: &PaymentFilters,
    ) -> Result<i64, PaymentServiceError> {
        let mut query =
            String::from("SELECT COUNT(*) FROM payment_transactions WHERE merchant_id = $1");
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

        // Add sandbox filter
        if filters.is_sandbox.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND sandbox_mode = ${}", param_count));
        }

        // Build the query with parameters
        let mut sql_query = sqlx::query_scalar::<_, i64>(&query).bind(merchant_id);

        // Bind status filter
        if let Some(status) = &filters.status {
            sql_query = sql_query.bind(status);
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

        if let Some(is_sandbox) = filters.is_sandbox {
            sql_query = sql_query.bind(is_sandbox);
        }

        let count = sql_query.fetch_one(&self.db_pool).await?;
        Ok(count)
    }

    /// Convert PaymentTransaction to PaymentResponse
    async fn convert_to_response(
        &self,
        payment: PaymentTransaction,
    ) -> Result<PaymentResponse, PaymentServiceError> {
        // Parse crypto type from string if exists
        let crypto_type = payment
            .crypto_type
            .as_deref()
            .map(|s| self.parse_crypto_type(s));

        // Parse status from string
        let status = self.parse_status(&payment.status);

        // Fetch payment link from database
        let payment_link = format!(
            "{}/{}",
            self.config.payment_page_base_url, payment.payment_id
        );

        // Generate QR code data only if we have the necessary info
        let qr_code_data = if let (Some(ct), Some(addr), Some(amt)) =
            (&crypto_type, &payment.to_address, &payment.amount)
        {
            Some(format!(
                "{}:{}?amount={}",
                ct.network().to_lowercase(),
                addr,
                amt
            ))
        } else {
            None
        };

        Ok(PaymentResponse {
            payment_id: payment.payment_id,
            status,
            amount: payment.amount,
            amount_usd: payment.amount_usd,
            crypto_type: payment.crypto_type.clone(),
            network: payment.network,
            deposit_address: payment.to_address.clone(),
            payment_link: Some(payment_link),
            qr_code_data,
            fee_amount: payment.fee_amount,
            fee_amount_usd: Some(payment.fee_amount_usd),
            expires_at: payment.expires_at,
            created_at: payment.created_at,
            confirmed_at: payment.confirmed_at,
            transaction_hash: payment.transaction_hash,
            from_address: payment.from_address,
            partial_payments: None,
            to_address: payment.to_address,
            confirmations: payment.confirmations.unwrap_or(0),
            required_confirmations: payment.required_confirmations.unwrap_or(1),
            description: payment.description,
            metadata: payment.metadata,
            last_verification_at: payment.last_verification_at,
        })
    }

    /// Parse crypto type from string
    fn parse_crypto_type(&self, crypto_type_str: &str) -> CryptoType {
        CryptoType::from_string(crypto_type_str).unwrap_or(CryptoType::Sol)
    }

    /// Parse payment status from string
    fn parse_status(&self, status_str: &str) -> PaymentStatus {
        PaymentStatus::from_string(status_str)
    }
}
