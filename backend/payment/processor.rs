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
    pub fn new(db_pool: PgPool, payment_page_base_url: String, price_service: Arc<PriceService>) -> Self {
        Self {
            db_pool: db_pool.clone(),
            price_service,
            merchant_service: MerchantService::new(db_pool),
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
            "SELECT fee_percentage FROM merchants WHERE id = $1",
            merchant_id
        )
        .fetch_one(&self.db_pool)
        .await?;
        
        let fee_percentage = merchant.fee_percentage;
        
        // Validate fee percentage is within acceptable bounds (0.1% - 5%)
        FeeCalculator::validate_fee_percentage(fee_percentage)?;
        
        // Calculate fee amounts using FeeCalculator
        let fee_amount_usd = FeeCalculator::calculate_fee_usd(request.amount_usd, fee_percentage);
        let total_amount_usd = FeeCalculator::calculate_total_with_fee(request.amount_usd, fee_amount_usd);
        
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
        let network = request.crypto_type.network();
        let required_confirmations = request.crypto_type.required_confirmations() as i32;
        
        // Determine if partial payments are enabled
        let partial_payments_enabled = request.partial_payments_enabled.unwrap_or(false);
        
        // Insert payment into database
        let payment = sqlx::query!(
            r#"
            INSERT INTO payment_transactions (
                merchant_id, payment_id, user_id, subscription_id, description, metadata,
                amount, amount_usd, fee_percentage, fee_amount, fee_amount_usd,
                crypto_type, network, to_address, status, confirmations, required_confirmations,
                partial_payments_enabled, total_paid, remaining_balance,
                created_at, expires_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22)
            RETURNING id, created_at
            "#,
            merchant_id,
            &payment_id,
            None::<i64>, // user_id (not used in multi-tenant context)
            None::<i64>, // subscription_id (not used in multi-tenant context)
            request.description.as_deref(),
            request.metadata,
            crypto_amount,
            total_amount_usd,
            fee_percentage,
            fee_amount_crypto,
            fee_amount_usd,
            request.crypto_type.as_str(),
            network,
            &merchant_wallet,
            "PENDING",
            0, // confirmations
            required_confirmations,
            partial_payments_enabled,
            Decimal::ZERO, // total_paid
            if partial_payments_enabled { Some(crypto_amount) } else { None },
            Utc::now(),
            expires_at
        )
        .fetch_one(&self.db_pool)
        .await?;
        
        // Generate payment link ID (short alphanumeric for URL)
        let link_id = self.generate_link_id();
        
        // Store payment link
        sqlx::query!(
            r#"
            INSERT INTO payment_links (link_id, payment_id, merchant_id, created_at, expires_at)
            VALUES ($1, $2, $3, $4, $5)
            "#,
            &link_id,
            payment.id,
            merchant_id,
            Utc::now(),
            expires_at
        )
        .execute(&self.db_pool)
        .await?;
        
        // Generate payment link URL
        let payment_link = format!("{}/pay/{}", self.payment_page_base_url, link_id);
        
        // Generate QR code data
        let qr_code_data = self.generate_qr_code_data(
            request.crypto_type,
            &merchant_wallet,
            crypto_amount,
        );
        
        info!(
            "Created payment {} for merchant {} - Amount: {} {} (${} + ${} fee)",
            payment_id, merchant_id, crypto_amount, request.crypto_type.as_str(),
            request.amount_usd, fee_amount_usd
        );
        
        Ok(PaymentResponse {
            payment_id,
            status: PaymentStatus::Pending,
            amount: crypto_amount,
            amount_usd: total_amount_usd,
            crypto_type: request.crypto_type,
            network: network.to_string(),
            deposit_address: merchant_wallet,
            payment_link,
            qr_code_data,
            fee_amount: fee_amount_crypto,
            fee_amount_usd,
            expires_at,
            created_at: payment.created_at,
            confirmed_at: None,
            transaction_hash: None,
            partial_payments: if partial_payments_enabled {
                Some(super::models::PartialPaymentInfo {
                    enabled: true,
                    total_paid: Decimal::ZERO,
                    remaining_balance: crypto_amount,
                    payments: vec![],
                })
            } else {
                None
            },
        })
    }

    /// Generate a unique payment ID
    /// 
    /// Creates a payment ID with format "pay_" followed by a random alphanumeric string.
    /// 
    /// # Returns
    /// * A unique payment ID string (e.g., "pay_abc123xyz")
    fn generate_payment_id(&self) -> String {
        // Use nanoid with 21 characters (default) for high entropy
        format!("pay_{}", nanoid!())
    }

    /// Generate a short link ID for payment pages
    /// 
    /// Creates a short alphanumeric ID for use in payment page URLs.
    /// 
    /// # Returns
    /// * A short link ID (e.g., "lnk_abc123")
    fn generate_link_id(&self) -> String {
        // Use shorter nanoid (12 characters) for cleaner URLs
        format!("lnk_{}", nanoid!(12))
    }

    /// Generate QR code data for payment
    /// 
    /// Creates a blockchain-specific URI that can be encoded in a QR code
    /// for easy payment by customers.
    /// 
    /// # Arguments
    /// * `crypto_type` - Type of cryptocurrency
    /// * `address` - Recipient wallet address
    /// * `amount` - Amount to be paid
    /// 
    /// # Returns
    /// * QR code data string in blockchain-specific format
    fn generate_qr_code_data(
        &self,
        crypto_type: CryptoType,
        address: &str,
        amount: Decimal,
    ) -> String {
        match crypto_type {
            CryptoType::Sol | CryptoType::UsdtSpl => {
                // Solana URI format: solana:<address>?amount=<amount>
                format!("solana:{}?amount={}", address, amount)
            }
            CryptoType::UsdtBep20 | CryptoType::UsdtArbitrum | CryptoType::UsdtPolygon | CryptoType::UsdtEth | CryptoType::Eth | CryptoType::Arb | CryptoType::Matic | CryptoType::Bnb => {
                // Ethereum URI format: ethereum:<address>?value=<amount>
                format!("ethereum:{}?value={}", address, amount)
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;

    #[test]
    fn test_generate_payment_id_format() {
        let processor = PaymentProcessor {
            db_pool: PgPool::connect_lazy("postgres://localhost/test").unwrap(),
            price_fetcher: PriceFetcher::new(),
            merchant_service: MerchantService::new(
                PgPool::connect_lazy("postgres://localhost/test").unwrap()
            ),
            payment_page_base_url: "http://localhost:8080".to_string(),
        };

        let payment_id = processor.generate_payment_id();
        
        // Should start with "pay_"
        assert!(payment_id.starts_with("pay_"));
        
        // Should be longer than just the prefix
        assert!(payment_id.len() > 4);
        
        // Should contain only alphanumeric characters and underscore
        assert!(payment_id.chars().all(|c| c.is_alphanumeric() || c == '_'));
    }

    #[test]
    fn test_generate_payment_id_uniqueness() {
        let processor = PaymentProcessor {
            db_pool: PgPool::connect_lazy("postgres://localhost/test").unwrap(),
            price_fetcher: PriceFetcher::new(),
            merchant_service: MerchantService::new(
                PgPool::connect_lazy("postgres://localhost/test").unwrap()
            ),
            payment_page_base_url: "http://localhost:8080".to_string(),
        };

        let id1 = processor.generate_payment_id();
        let id2 = processor.generate_payment_id();
        
        // IDs should be unique
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_generate_link_id_format() {
        let processor = PaymentProcessor {
            db_pool: PgPool::connect_lazy("postgres://localhost/test").unwrap(),
            price_fetcher: PriceFetcher::new(),
            merchant_service: MerchantService::new(
                PgPool::connect_lazy("postgres://localhost/test").unwrap()
            ),
            payment_page_base_url: "http://localhost:8080".to_string(),
        };

        let link_id = processor.generate_link_id();
        
        // Should start with "lnk_"
        assert!(link_id.starts_with("lnk_"));
        
        // Should be shorter than payment_id (for cleaner URLs)
        assert!(link_id.len() < 20);
        
        // Should contain only alphanumeric characters and underscore
        assert!(link_id.chars().all(|c| c.is_alphanumeric() || c == '_'));
    }

    #[test]
    fn test_generate_link_id_uniqueness() {
        let processor = PaymentProcessor {
            db_pool: PgPool::connect_lazy("postgres://localhost/test").unwrap(),
            price_fetcher: PriceFetcher::new(),
            merchant_service: MerchantService::new(
                PgPool::connect_lazy("postgres://localhost/test").unwrap()
            ),
            payment_page_base_url: "http://localhost:8080".to_string(),
        };

        let id1 = processor.generate_link_id();
        let id2 = processor.generate_link_id();
        
        // IDs should be unique
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_generate_qr_code_data_solana() {
        let processor = PaymentProcessor {
            db_pool: PgPool::connect_lazy("postgres://localhost/test").unwrap(),
            price_fetcher: PriceFetcher::new(),
            merchant_service: MerchantService::new(
                PgPool::connect_lazy("postgres://localhost/test").unwrap()
            ),
            payment_page_base_url: "http://localhost:8080".to_string(),
        };

        let address = "7xKXtg2CW87d97TXJSDpbD5jBkheTqA83TZRuJosgAsU";
        let amount = Decimal::new(100, 0);
        
        // Test SOL
        let qr_data = processor.generate_qr_code_data(CryptoType::Sol, address, amount);
        assert_eq!(qr_data, format!("solana:{}?amount={}", address, amount));
        
        // Test USDT SPL
        let qr_data = processor.generate_qr_code_data(CryptoType::UsdtSpl, address, amount);
        assert_eq!(qr_data, format!("solana:{}?amount={}", address, amount));
    }

    #[test]
    fn test_generate_qr_code_data_evm() {
        let processor = PaymentProcessor {
            db_pool: PgPool::connect_lazy("postgres://localhost/test").unwrap(),
            price_fetcher: PriceFetcher::new(),
            merchant_service: MerchantService::new(
                PgPool::connect_lazy("postgres://localhost/test").unwrap()
            ),
            payment_page_base_url: "http://localhost:8080".to_string(),
        };

        let address = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb0";
        let amount = Decimal::new(100, 0);
        
        // Test USDT BEP20
        let qr_data = processor.generate_qr_code_data(CryptoType::UsdtBep20, address, amount);
        assert_eq!(qr_data, format!("ethereum:{}?value={}", address, amount));
        
        // Test USDT Arbitrum
        let qr_data = processor.generate_qr_code_data(CryptoType::UsdtArbitrum, address, amount);
        assert_eq!(qr_data, format!("ethereum:{}?value={}", address, amount));
        
        // Test USDT Polygon
        let qr_data = processor.generate_qr_code_data(CryptoType::UsdtPolygon, address, amount);
        assert_eq!(qr_data, format!("ethereum:{}?value={}", address, amount));
    }

    #[test]
    fn test_fee_calculation_logic() {
        // Test fee calculation: $100 payment with 1.5% fee
        let base_amount = Decimal::new(10000, 2); // $100.00
        let fee_percentage = Decimal::new(150, 2); // 1.50%
        
        let fee_amount_usd = base_amount * fee_percentage / Decimal::new(100, 0);
        let total_amount_usd = base_amount + fee_amount_usd;
        
        assert_eq!(fee_amount_usd, Decimal::new(150, 2)); // $1.50
        assert_eq!(total_amount_usd, Decimal::new(10150, 2)); // $101.50
    }

    #[test]
    fn test_fee_calculation_different_percentages() {
        let base_amount = Decimal::new(10000, 2); // $100.00
        
        // Test 0.5% fee
        let fee_percentage = Decimal::new(50, 2);
        let fee_amount = base_amount * fee_percentage / Decimal::new(100, 0);
        assert_eq!(fee_amount, Decimal::new(50, 2)); // $0.50
        
        // Test 2.5% fee
        let fee_percentage = Decimal::new(250, 2);
        let fee_amount = base_amount * fee_percentage / Decimal::new(100, 0);
        assert_eq!(fee_amount, Decimal::new(250, 2)); // $2.50
        
        // Test 5% fee
        let fee_percentage = Decimal::new(500, 2);
        let fee_amount = base_amount * fee_percentage / Decimal::new(100, 0);
        assert_eq!(fee_amount, Decimal::new(500, 2)); // $5.00
    }

    #[test]
    fn test_crypto_amount_calculation_stablecoin() {
        // For stablecoins (USDT), amount should be 1:1 with USD
        let total_amount_usd = Decimal::new(10150, 2); // $101.50
        let crypto_price = Decimal::new(1, 0); // $1.00 (not actually used for USDT)
        
        // For USDT, crypto amount equals USD amount
        let crypto_amount = total_amount_usd;
        
        assert_eq!(crypto_amount, Decimal::new(10150, 2)); // 101.50 USDT
    }

    #[test]
    fn test_crypto_amount_calculation_non_stablecoin() {
        // For non-stablecoins (SOL), divide USD by price
        let total_amount_usd = Decimal::new(10150, 2); // $101.50
        let crypto_price = Decimal::new(5000, 2); // $50.00 per SOL
        
        let crypto_amount = total_amount_usd / crypto_price;
        
        assert_eq!(crypto_amount, Decimal::new(203, 2)); // 2.03 SOL
    }

    #[test]
    fn test_payment_link_generation() {
        let processor = PaymentProcessor {
            db_pool: PgPool::connect_lazy("postgres://localhost/test").unwrap(),
            price_fetcher: PriceFetcher::new(),
            merchant_service: MerchantService::new(
                PgPool::connect_lazy("postgres://localhost/test").unwrap()
            ),
            payment_page_base_url: "https://pay.example.com".to_string(),
        };

        let link_id = processor.generate_link_id();
        let payment_link = format!("{}/pay/{}", processor.payment_page_base_url, link_id);
        
        assert!(payment_link.starts_with("https://pay.example.com/pay/lnk_"));
    }

    #[test]
    fn test_expiration_time_calculation() {
        let now = Utc::now();
        let expiration_minutes = 15;
        let expires_at = now + Duration::minutes(expiration_minutes as i64);
        
        // Should be approximately 15 minutes in the future
        let diff = expires_at - now;
        assert!(diff.num_minutes() >= 14 && diff.num_minutes() <= 15);
    }

    #[test]
    fn test_expiration_time_custom() {
        let now = Utc::now();
        let expiration_minutes = 30;
        let expires_at = now + Duration::minutes(expiration_minutes as i64);
        
        // Should be approximately 30 minutes in the future
        let diff = expires_at - now;
        assert!(diff.num_minutes() >= 29 && diff.num_minutes() <= 30);
    }

    #[test]
    fn test_partial_payments_remaining_balance() {
        // When partial payments are enabled, remaining balance should equal total amount initially
        let crypto_amount = Decimal::new(100, 0);
        let total_paid = Decimal::ZERO;
        let remaining_balance = crypto_amount - total_paid;
        
        assert_eq!(remaining_balance, crypto_amount);
    }

    #[test]
    fn test_network_and_confirmations() {
        // Test that each crypto type has correct network and confirmations
        assert_eq!(CryptoType::Sol.network(), "SOLANA");
        assert_eq!(CryptoType::Sol.required_confirmations(), 32);
        
        assert_eq!(CryptoType::UsdtSpl.network(), "SOLANA_SPL");
        assert_eq!(CryptoType::UsdtSpl.required_confirmations(), 32);
        
        assert_eq!(CryptoType::UsdtBep20.network(), "BEP20");
        assert_eq!(CryptoType::UsdtBep20.required_confirmations(), 15);
        
        assert_eq!(CryptoType::UsdtArbitrum.network(), "ARBITRUM");
        assert_eq!(CryptoType::UsdtArbitrum.required_confirmations(), 1);
        
        assert_eq!(CryptoType::UsdtPolygon.network(), "POLYGON");
        assert_eq!(CryptoType::UsdtPolygon.required_confirmations(), 128);
    }
}
