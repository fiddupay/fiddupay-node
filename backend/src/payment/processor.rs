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
    config: crate::config::Config,
}

impl PaymentProcessor {
    pub fn new(db_pool: PgPool, _payment_page_base_url: String, price_service: Arc<PriceService>, config: crate::config::Config) -> Self {
        Self {
            db_pool: db_pool.clone(),
            price_service,
            merchant_service: MerchantService::new(db_pool, config.clone()),
            config,
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
        // Validate that exactly one of amount or amount_usd is provided
        request.validate()
            .map_err(|e| ServiceError::ValidationError(e))?;

        // Generate unique payment ID (e.g., "pay_abc123xyz")
        let payment_id = self.generate_payment_id();
        
        // Get merchant to retrieve fee percentage and preference
        let merchant = sqlx::query!(
            "SELECT fee_percentage, customer_pays_fee, sandbox_mode FROM merchants WHERE id = $1",
            merchant_id
        )
        .fetch_one(&self.db_pool)
        .await?;
        
        let fee_percentage = merchant.fee_percentage;
        let customer_pays_fee = merchant.customer_pays_fee;
        let is_sandbox = merchant.sandbox_mode;
        
        // Validate fee percentage is within acceptable bounds (0.1% - 5%)
        FeeCalculator::validate_fee_percentage(fee_percentage)?;

        let (crypto_amount, amount_usd, fee_amount_crypto, fee_amount_usd, merchant_wallet, status, network) = 
        if let Some(crypto_type) = request.crypto_type {
            // Case A: Specific crypto type provided - normal flow
            
            // Get merchant's wallet address for this crypto type
            let wallet = self.merchant_service
                .get_wallet_address(merchant_id, crypto_type)
                .await?;

            // Calculate amounts based on which input was provided
            let (crypto_amount, amount_usd, fee_amount_crypto, fee_amount_usd) = if let Some(usd_amount) = request.amount_usd {
                let fee_amount_usd = FeeCalculator::calculate_fee_usd(usd_amount, fee_percentage);
                let total_amount_usd = if customer_pays_fee {
                    FeeCalculator::calculate_total_with_fee(usd_amount, fee_amount_usd)
                } else {
                    usd_amount
                };
                
                let (crypto_total, fee_crypto) = if crypto_type.as_str() == "USDT" {
                    (total_amount_usd, fee_amount_usd)
                } else {
                    let crypto_price = self.price_service
                        .get_price(crypto_type)
                        .await
                        .map_err(|e| ServiceError::Internal(format!("Failed to fetch price: {}", e)))?;
                    
                    let crypto_price_decimal = Decimal::from_f64_retain(crypto_price)
                        .ok_or_else(|| ServiceError::Internal("Invalid price conversion".to_string()))?;
                    
                    (
                        total_amount_usd / crypto_price_decimal,
                        fee_amount_usd / crypto_price_decimal
                    )
                };
                
                (crypto_total, total_amount_usd, fee_crypto, fee_amount_usd)
            } else if let Some(crypto_amt) = request.amount {
                let base_amount_usd = if crypto_type.as_str() == "USDT" {
                    crypto_amt
                } else {
                    let crypto_price = self.price_service
                        .get_price(crypto_type)
                        .await
                        .map_err(|e| ServiceError::Internal(format!("Failed to fetch price: {}", e)))?;
                    
                    let crypto_price_decimal = Decimal::from_f64_retain(crypto_price)
                        .ok_or_else(|| ServiceError::Internal("Invalid price conversion".to_string()))?;
                    
                    crypto_amt * crypto_price_decimal
                };
                
                let fee_amount_usd = FeeCalculator::calculate_fee_usd(base_amount_usd, fee_percentage);
                
                let (final_crypto, final_usd, fee_crypto) = if customer_pays_fee {
                    let fee_crypto = if crypto_type.as_str() == "USDT" {
                        fee_amount_usd
                    } else {
                        let crypto_price = self.price_service
                            .get_price(crypto_type)
                            .await
                            .map_err(|e| ServiceError::Internal(format!("Failed to fetch price: {}", e)))?;
                        
                        let crypto_price_decimal = Decimal::from_f64_retain(crypto_price)
                            .ok_or_else(|| ServiceError::Internal("Invalid price conversion".to_string()))?;
                        
                        fee_amount_usd / crypto_price_decimal
                    };
                    (crypto_amt + fee_crypto, base_amount_usd + fee_amount_usd, fee_crypto)
                } else {
                    let fee_crypto = if crypto_type.as_str() == "USDT" {
                        fee_amount_usd
                    } else {
                        let crypto_price = self.price_service
                            .get_price(crypto_type)
                            .await
                            .map_err(|e| ServiceError::Internal(format!("Failed to fetch price: {}", e)))?;
                        
                        let crypto_price_decimal = Decimal::from_f64_retain(crypto_price)
                            .ok_or_else(|| ServiceError::Internal("Invalid price conversion".to_string()))?;
                        
                        fee_amount_usd / crypto_price_decimal
                    };
                    (crypto_amt, base_amount_usd, fee_crypto)
                };
                
                (final_crypto, final_usd, fee_crypto, fee_amount_usd)
            } else {
                return Err(ServiceError::ValidationError("Either amount or amount_usd must be provided".to_string()));
            };

            let network = if is_sandbox {
                match crypto_type.as_str() {
                    "SOL" => "Solana Devnet",
                    "ETH" => "Ethereum Sepolia",
                    "BNB" => "BSC Testnet", 
                    "MATIC" => "Polygon Mumbai",
                    "ARB" => "Arbitrum Sepolia",
                    "USDT" => match crypto_type.network() {
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
                crypto_type.network()
            };

            (Some(crypto_amount), amount_usd, Some(fee_amount_crypto), Some(fee_amount_usd), Some(wallet), PaymentStatus::Pending, Some(network.to_string()))
        } else {
            // Case B: No crypto type provided - multi-currency selection mode
            let amount_usd = request.amount_usd.ok_or_else(|| ServiceError::ValidationError("amount_usd is required for multi-currency selection".to_string()))?;
            let fee_amount_usd = FeeCalculator::calculate_fee_usd(amount_usd, fee_percentage);
            
            // If customer pays fee, add it to the total USD.
            let total_amount_usd = if customer_pays_fee {
                FeeCalculator::calculate_total_with_fee(amount_usd, fee_amount_usd)
            } else {
                amount_usd
            };

            (None, total_amount_usd, None, Some(fee_amount_usd), None, PaymentStatus::SelectionRequired, None)
        };

        // Calculate expiration time
        let expiration_minutes = request.expiration_minutes.unwrap_or(15);
        let expires_at = Utc::now() + Duration::minutes(expiration_minutes as i64);
        
        // Store payment in database
        let payment = sqlx::query_as!(
            crate::models::payment::Payment,
            r#"
            INSERT INTO payment_transactions (
                payment_id, merchant_id, crypto_type, amount, amount_usd, to_address,
                status, expires_at, fee_percentage, fee_amount, fee_amount_usd, network,
                required_confirmations, webhook_url, description
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            RETURNING id, payment_id, merchant_id, crypto_type, amount, amount_usd, to_address,
                     status, expires_at, created_at, confirmed_at, description, metadata,
                     confirmations, required_confirmations, transaction_hash, from_address, webhook_url,
                     fee_percentage, fee_amount, fee_amount_usd
            "#,
            payment_id,
            merchant_id,
            request.crypto_type.map(|ct| ct.to_string()),
            crypto_amount,
            amount_usd,
            merchant_wallet,
            match status {
                PaymentStatus::Pending => "PENDING",
                PaymentStatus::SelectionRequired => "SELECTION_REQUIRED",
                _ => "PENDING"
            },
            expires_at,
            fee_percentage,
            fee_amount_crypto,
            fee_amount_usd,
            network,
            1, // required_confirmations
            request.webhook_url,
            request.description
        )
        .fetch_one(&self.db_pool)
        .await?;

        // Generate payment link and QR code
        let payment_link = format!("{}/pay/{}", 
            self.config.payment_page_base_url,
            payment_id
        );
        
        let qr_code_data = if let (Some(net), Some(wallet), Some(amt)) = (&network, &merchant_wallet, crypto_amount) {
            Some(format!(
                "{}:{}?amount={}",
                net.to_lowercase(),
                wallet,
                amt
            ))
        } else {
            None
        };
        
        info!(
            "Created payment {} for merchant {} - Status: {:?} - Amount USD: ${}",
            payment_id, merchant_id, status, amount_usd
        );
        
        Ok(PaymentResponse {
            payment_id,
            status,
            amount: crypto_amount,
            amount_usd,
            crypto_type: request.crypto_type.map(|ct| ct.as_str().to_string()),
            to_address: merchant_wallet.clone(),
            network: network,
            deposit_address: merchant_wallet,
            payment_link: Some(payment_link),
            qr_code_data,
            fee_amount: fee_amount_crypto,
            fee_amount_usd,
            expires_at,
            created_at: payment.created_at,
            confirmed_at: None,
            transaction_hash: None,
            from_address: None,
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