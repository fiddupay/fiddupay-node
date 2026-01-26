// Payment Processor
// Creates and manages payment requests

use chrono::{Duration, Utc};
use nanoid::nanoid;
use rust_decimal::Decimal;
use sqlx::PgPool;
use tracing::info;

use crate::error::ServiceError;
use crate::services::{merchant_service::MerchantService, price_service::PriceService};
use std::sync::Arc;
use super::models::{CreatePaymentRequest, PaymentResponse, PaymentStatus, CryptoType};

use super::fee_calculator::FeeCalculator;

pub struct PaymentProcessor {
    db_pool: PgPool,
    price_service: Arc<PriceService>,
    merchant_service: MerchantService,
    payment_page_base_url: String,
}

impl PaymentProcessor {
    pub fn new(db_pool: PgPool, payment_page_base_url: String, price_service: Arc<PriceService>, config: crate::config::Config) -> Self {
        Self {
            db_pool: db_pool.clone(),
            price_service,
            merchant_service: MerchantService::new(db_pool, config),
            payment_page_base_url,
        }
    }

    /// Create a new payment request for a merchant
    /// 
    /// Generates a unique payment ID, calculates crypto amount using real-time prices,
    /// calculates fees, and creates a payment record in the database.
    /// 
    /// # Arguments
    /// * `merchant_id` - ID of the merchant creating the payment
    /// * `request` - Payment creation request with amount, crypto type, etc.
    /// 
    /// # Returns
    /// * `PaymentResponse` with payment details including deposit address and payment link
    /// 
    /// # Requirements
    /// * 2.1: Generate unique payment identifier
    /// * 2.2: Calculate crypto amount using real-time exchange rates
    /// * 2.3: Generate payment address for selected blockchain
    /// * 2.6: Include platform fee in total amount
    /// * 6.1: Calculate fees and include in total
    pub async fn create_payment(
        &self,
        merchant_id: i64,
        request: CreatePaymentRequest,
    ) -> Result<PaymentResponse, ServiceError> {
        // Generate unique payment ID (e.g., "pay_abc123xyz")
        let payment_id = self.generate_payment_id();
        
        // Get merchant's wallet address for this crypto type
        let merchant_wallet = self.merchant_service
            .get_wallet_address(merchant_id, request.crypto_type)
            .await?;
        
        // Get merchant to retrieve fee percentage
        let merchant = sqlx::query!(
            "SELECT fee_percentage, sandbox_mode FROM merchants WHERE id = $1",
            merchant_id
        )
        .fetch_one(&self.db_pool)
        .await?;
        
        let fee_percentage = merchant.fee_percentage;
        let is_sandbox = merchant.sandbox_mode;
        
        // Validate fee percentage is within acceptable bounds (0.1% - 5%)
        FeeCalculator::validate_fee_percentage(fee_percentage)?;
        
        // Calculate fee amounts using FeeCalculator
        let fee_amount_usd = FeeCalculator::calculate_fee_usd(request.amount_usd.unwrap_or(Decimal::ZERO), fee_percentage);
        let total_amount_usd = FeeCalculator::calculate_total_with_fee(request.amount_usd.unwrap_or(Decimal::ZERO), fee_amount_usd);
        
        // Calculate crypto amount based on type
        let (crypto_amount, fee_amount_crypto) = if request.crypto_type.as_str() == "USDT" {
            // For stablecoins (USDT), amount is 1:1 with USD
            let crypto_price = Decimal::ONE;
            (
                total_amount_usd,
                FeeCalculator::calculate_fee_crypto(fee_amount_usd, crypto_price)
            )
        } else {
            // For non-stablecoins, get price and divide USD by price
            let crypto_price = self.price_service
                .get_price(request.crypto_type)
                .await
                .map_err(|e| ServiceError::Internal(format!("Failed to fetch price: {}", e)))?;
            
            let crypto_price_decimal = Decimal::from_f64_retain(crypto_price)
                .ok_or_else(|| ServiceError::Internal("Invalid price conversion".to_string()))?;
            
            (
                total_amount_usd / crypto_price_decimal,
                FeeCalculator::calculate_fee_crypto(fee_amount_usd, crypto_price_decimal)
            )
        };
        // Calculate expiration time
        let expiration_minutes = request.expiration_minutes.unwrap_or(15);
        let expires_at = Utc::now() + Duration::minutes(expiration_minutes as i64);
        
        // Get network and required confirmations
        // Get network based on sandbox mode (testnet for sandbox, mainnet for production)
        let network = if is_sandbox {
            match request.crypto_type.as_str() {
                "SOL" => "Solana Devnet",
                "ETH" => "Ethereum Sepolia",
                "BNB" => "BSC Testnet", 
                "MATIC" => "Polygon Mumbai",
                "ARB" => "Arbitrum Sepolia",
                "USDT" => match request.crypto_type.network() {
                    "Solana" => "Solana Devnet",
                    "Ethereum" => "Ethereum Sepolia",
                    "BSC" => "BSC Testnet",
                    "Polygon" => "Polygon Mumbai", 
                    "Arbitrum" => "Arbitrum Sepolia",
                    _ => "Unknown Testnet"
                },
                _ => "Unknown Testnet"
            }
        } else {
            request.crypto_type.network()
        };
        let required_confirmations = request.crypto_type.required_confirmations() as i32;
        
        // Determine if partial payments are enabled
        let partial_payments_enabled = false; // Simplified for now
        
        // Store payment in database
        let payment = sqlx::query_as!(
            crate::models::payment::Payment,
            r#"
            INSERT INTO payment_transactions (
                payment_id, merchant_id, crypto_type, amount, amount_usd, to_address,
                status, expires_at, fee_percentage, fee_amount, fee_amount_usd, network,
                required_confirmations
            )
            VALUES ($1, $2, $3, $4, $5, $6, 'PENDING', $7, $8, $9, $10, $11, $12)
            RETURNING id, payment_id, merchant_id, crypto_type, amount, amount_usd, to_address,
                     status, expires_at, created_at, confirmed_at, description, metadata,
                     confirmations, required_confirmations
            "#,
            payment_id,
            merchant_id,
            request.crypto_type.to_string(),
            crypto_amount,
            total_amount_usd,
            merchant_wallet,
            expires_at,
            Decimal::new(25, 1), // 2.5%
            fee_amount_crypto,
            fee_amount_usd,
            request.crypto_type.network(),
            1 // required_confirmations
        )
        .fetch_one(&self.db_pool)
        .await?;

        // Generate payment link and QR code
        let payment_link = format!("{}/pay/{}", 
            std::env::var("PAYMENT_PAGE_BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string()),
            payment_id
        );
        
        let qr_code_data = format!(
            "{}:{}?amount={}",
            network.to_lowercase(),
            merchant_wallet,
            crypto_amount
        );
        
        info!(
            "Created payment {} for merchant {} - Amount: {} {} (${} + ${} fee)",
            payment_id, merchant_id, crypto_amount, request.crypto_type.as_str(),
            request.amount_usd.unwrap_or(Decimal::ZERO), fee_amount_usd
        );
        
        Ok(PaymentResponse {
            payment_id,
            status: PaymentStatus::Pending,
            amount: crypto_amount,
            amount_usd: total_amount_usd,
            crypto_type: request.crypto_type,
            to_address: merchant_wallet.clone(),
            network: Some(network.to_string()),
            deposit_address: Some(merchant_wallet),
            payment_link: Some(payment_link),
            qr_code_data: Some(qr_code_data),
            fee_amount: Some(fee_amount_crypto),
            fee_amount_usd: Some(fee_amount_usd),
            expires_at,
            created_at: payment.created_at,
            confirmed_at: None,
            transaction_hash: None,
            confirmations: 0,
            required_confirmations: 1,
            description: None,
            metadata: None,
            partial_payments: None,
        })
    }

    /// Generate a unique payment ID
    fn generate_payment_id(&self) -> String {
use crate::utils::api_keys::ApiKeyGenerator;

        ApiKeyGenerator::generate_payment_id()
    }
}
