// Address-Only Mode with Auto-Forwarding (Phase 1)
// Supports native currencies only: ETH, BNB, MATIC, ARB, SOL

use crate::error::ServiceError;
use crate::payment::models::CryptoType;
use crate::services::gas_fee_service::GasFeeService;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct AddressOnlyPayment {
    pub id: i64,
    pub payment_id: String,
    pub merchant_id: i64,
    pub crypto_type: CryptoType,
    pub gateway_deposit_address: String,
    pub merchant_destination_address: String,
    pub requested_amount: Decimal,
    pub customer_amount: Decimal, // Amount customer needs to pay
    pub processing_fee: Decimal,
    pub forwarding_amount: Decimal,
    pub status: AddressOnlyStatus,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "address_only_status", rename_all = "snake_case")]
pub enum AddressOnlyStatus {
    PendingPayment,
    PaymentReceived,
    ForwardingInProgress,
    Completed,
    Failed,
}

#[derive(Clone)]
pub struct AddressOnlyService {
    db_pool: PgPool,
    gas_service: GasFeeService,
    config: crate::config::Config,
}

impl AddressOnlyService {
    pub fn new(db_pool: PgPool, gas_service: GasFeeService, config: crate::config::Config) -> Self {
        Self { db_pool, gas_service, config }
    }

    /// Create payment request for address-only mode (native currencies only)
    pub async fn create_payment_request(
        &self,
        merchant_id: i64,
        crypto_type: CryptoType,
        merchant_address: String,
        requested_amount: Decimal,
    ) -> Result<AddressOnlyPayment, ServiceError> {
        // Validate native currency only
        if !self.is_native_currency(crypto_type) {
            return Err(ServiceError::ValidationError(
                "Address-only mode currently supports native currencies only (ETH, BNB, MATIC, ARB, SOL)".to_string()
            ));
        }

        let payment_id = Uuid::new_v4().to_string();
        let gateway_deposit_address = self.generate_deposit_address(crypto_type).await?;
        
        // Get merchant fee configuration
        let merchant = sqlx::query!(
            "SELECT fee_percentage, COALESCE(customer_pays_fee, true) as customer_pays_fee FROM merchants WHERE id = $1",
            merchant_id
        )
        .fetch_one(&self.db_pool)
        .await?;

        // Calculate processing fee based on merchant configuration
        let processing_fee = requested_amount * (merchant.fee_percentage / Decimal::from(100)); // Convert percentage to decimal
        let customer_pays_fee = merchant.customer_pays_fee.unwrap_or(true); // Default to customer pays fee
        let (customer_amount, forwarding_amount) = if customer_pays_fee {
            // Customer pays fee: customer pays (requested + fee), merchant gets requested amount
            let customer_total = requested_amount + processing_fee;
            (customer_total, requested_amount)
        } else {
            // Merchant pays fee: customer pays requested amount, merchant gets (requested - fee)
            let merchant_receives = requested_amount - processing_fee;
            (requested_amount, merchant_receives)
        };

        let payment = sqlx::query_as!(
            AddressOnlyPayment,
            r#"
            INSERT INTO address_only_payments (
                payment_id, merchant_id, crypto_type, gateway_deposit_address,
                merchant_destination_address, requested_amount, processing_fee,
                forwarding_amount, status, customer_amount
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING id, payment_id, merchant_id, crypto_type as "crypto_type: CryptoType",
                     gateway_deposit_address, merchant_destination_address,
                     requested_amount, customer_amount as "customer_amount!",
                     processing_fee, forwarding_amount,
                     status as "status: AddressOnlyStatus", created_at
            "#,
            payment_id,
            merchant_id,
            crypto_type as CryptoType,
            gateway_deposit_address,
            merchant_address,
            requested_amount,
            processing_fee,
            forwarding_amount,
            AddressOnlyStatus::PendingPayment as i32,
            customer_amount
        )
        .fetch_one(&self.db_pool)
        .await?;

        Ok(payment)
    }

    /// Process received payment and initiate auto-forwarding
    pub async fn process_received_payment(
        &self,
        payment_id: &str,
        received_amount: Decimal,
        tx_hash: &str,
    ) -> Result<(), ServiceError> {
        // Get payment details
        let payment = self.get_payment_by_id(payment_id).await?;
        
        // Verify received amount matches or exceeds requested
        if received_amount < payment.requested_amount {
            return Err(ServiceError::ValidationError(
                "Received amount is less than requested".to_string()
            ));
        }

        // Update status to received
        self.update_payment_status(payment_id, AddressOnlyStatus::PaymentReceived).await?;

        // Initiate auto-forwarding
        self.initiate_auto_forwarding(&payment, tx_hash).await?;

        Ok(())
    }

    /// Auto-forward funds to merchant address
    async fn initiate_auto_forwarding(
        &self,
        payment: &AddressOnlyPayment,
        _received_tx_hash: &str,
    ) -> Result<(), ServiceError> {
        // Update status to forwarding
        self.update_payment_status(&payment.payment_id, AddressOnlyStatus::ForwardingInProgress).await?;

        // Get current gas estimate
        let gas_estimate = self.gas_service.get_gas_estimate(payment.crypto_type).await?;
        
        // Calculate net forwarding amount (deduct gas fee)
        let net_forwarding_amount = payment.forwarding_amount - gas_estimate.standard_fee;
        
        if net_forwarding_amount <= Decimal::ZERO {
            return Err(ServiceError::ValidationError(
                "Forwarding amount too small after gas fees".to_string()
            ));
        }

        // Send actual blockchain transaction
        let forwarding_tx_hash = self.send_forwarding_transaction(
            payment,
            net_forwarding_amount,
            &gas_estimate,
        ).await?;
        
        // Record forwarding transaction
        sqlx::query!(
            r#"
            INSERT INTO address_only_forwarding_txs (
                payment_id, destination_address, amount, gas_fee, tx_hash, status
            ) VALUES ($1, $2, $3, $4, $5, 'completed')
            "#,
            payment.payment_id,
            payment.merchant_destination_address,
            net_forwarding_amount,
            gas_estimate.standard_fee,
            forwarding_tx_hash
        )
        .execute(&self.db_pool)
        .await?;

        // Update payment status to completed
        self.update_payment_status(&payment.payment_id, AddressOnlyStatus::Completed).await?;

        // Send webhook notification
        if let Ok(updated_payment) = self.get_payment_by_id(&payment.payment_id).await {
            let webhook_service = crate::services::webhook_notification_service::WebhookNotificationService::new(self.db_pool.clone());
            let _ = webhook_service.notify_status_change(&updated_payment).await;
        }

        Ok(())
    }

    /// Generate unique deposit address for payment tracking
    async fn generate_deposit_address(&self, crypto_type: CryptoType) -> Result<String, ServiceError> {
        // Use existing KeyGenerator for real address generation
        use crate::utils::keygen::KeyGenerator;
        
        let network = match crypto_type {
            CryptoType::Eth => "ethereum",
            CryptoType::Bnb => "bsc", 
            CryptoType::Matic => "polygon",
            CryptoType::Arb => "arbitrum",
            CryptoType::Sol => "solana",
            _ => return Err(ServiceError::ValidationError("Unsupported crypto type".to_string())),
        };

        let wallet = match crypto_type {
            CryptoType::Sol => KeyGenerator::generate_solana_wallet()?,
            _ => KeyGenerator::generate_evm_wallet()?,
        };

        // Store private key securely for later forwarding
        let payment_id = uuid::Uuid::new_v4().to_string();
        self.store_deposit_keypair(&payment_id, &wallet.private_key, &wallet.address).await?;

        Ok(wallet.address)
    }

    /// Check if crypto type is native currency (Phase 1 support only)
    fn is_native_currency(&self, crypto_type: CryptoType) -> bool {
        matches!(crypto_type, 
            CryptoType::Eth | 
            CryptoType::Bnb | 
            CryptoType::Matic | 
            CryptoType::Arb | 
            CryptoType::Sol
        )
    }

    /// Store deposit keypair securely for forwarding
    async fn store_deposit_keypair(
        &self,
        payment_id: &str,
        private_key: &str,
        address: &str,
    ) -> Result<(), ServiceError> {
        use crate::utils::encryption::encrypt_data;
        
        let encrypted_key = encrypt_data(private_key)
            .map_err(|e| ServiceError::Internal(format!("Key encryption failed: {}", e)))?;

        sqlx::query!(
            "INSERT INTO deposit_keypairs (payment_id, address, encrypted_private_key) VALUES ($1, $2, $3)",
            payment_id,
            address,
            encrypted_key
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    /// Send forwarding transaction to blockchain
    async fn send_forwarding_transaction(
        &self,
        payment: &AddressOnlyPayment,
        amount: Decimal,
        gas_estimate: &crate::services::gas_fee_service::GasFeeEstimate,
    ) -> Result<String, ServiceError> {
        // Get private key for deposit address
        let private_key = self.get_deposit_private_key(&payment.gateway_deposit_address).await?;
        
        match payment.crypto_type {
            CryptoType::Sol => {
                self.send_solana_transaction(
                    &private_key,
                    &payment.merchant_destination_address,
                    amount,
                ).await
            }
            _ => {
                self.send_evm_transaction(
                    payment.crypto_type,
                    &private_key,
                    &payment.merchant_destination_address,
                    amount,
                    gas_estimate,
                ).await
            }
        }
    }

    /// Send Solana transaction
    async fn send_solana_transaction(
        &self,
        private_key: &str,
        to_address: &str,
        amount: Decimal,
    ) -> Result<String, ServiceError> {
        let tx_sender = crate::services::blockchain_transaction_sender::BlockchainTransactionSender::new(self.config.clone());
        tx_sender.send_native_transaction(CryptoType::Sol, private_key, to_address, amount, None).await
    }

    /// Send EVM transaction  
    async fn send_evm_transaction(
        &self,
        crypto_type: CryptoType,
        private_key: &str,
        to_address: &str,
        amount: Decimal,
        gas_estimate: &crate::services::gas_fee_service::GasFeeEstimate,
    ) -> Result<String, ServiceError> {
        let tx_sender = crate::services::blockchain_transaction_sender::BlockchainTransactionSender::new(self.config.clone());
        
        // Convert gas price to U256
        let gas_price_wei = (gas_estimate.standard_fee * Decimal::new(1_000_000_000_000_000_000i64, 0))
            .to_u128()
            .map(web3::types::U256::from);
            
        tx_sender.send_native_transaction(crypto_type, private_key, to_address, amount, gas_price_wei).await
    }

    /// Get private key for deposit address
    async fn get_deposit_private_key(&self, address: &str) -> Result<String, ServiceError> {
        let record = sqlx::query!(
            "SELECT encrypted_private_key FROM deposit_keypairs WHERE address = $1",
            address
        )
        .fetch_optional(&self.db_pool)
        .await?
        .ok_or_else(|| ServiceError::NotFound("Deposit keypair not found".to_string()))?;

        // For now, return a placeholder since we don't have decrypt_data
        // In production, this would decrypt the stored key
        Ok(format!("decrypted_key_for_{}", address))
    }

    /// Get merchant statistics for address-only payments
    pub async fn get_merchant_stats(&self, merchant_id: i64) -> Result<crate::api::address_only::AddressOnlyStats, ServiceError> {
        let stats = sqlx::query!(
            r#"
            SELECT 
                COUNT(*) as total_payments,
                COUNT(CASE WHEN status = 'Completed' THEN 1 END) as completed_payments,
                COUNT(CASE WHEN status = 'PendingPayment' THEN 1 END) as pending_payments,
                COALESCE(SUM(requested_amount), 0) as total_volume,
                COALESCE(SUM(processing_fee), 0) as total_fees_collected
            FROM address_only_payments 
            WHERE merchant_id = $1
            "#,
            merchant_id
        )
        .fetch_one(&self.db_pool)
        .await?;

        Ok(crate::api::address_only::AddressOnlyStats {
            total_payments: stats.total_payments.unwrap_or(0),
            completed_payments: stats.completed_payments.unwrap_or(0),
            pending_payments: stats.pending_payments.unwrap_or(0),
            total_volume: stats.total_volume.unwrap_or_else(|| Decimal::ZERO),
            total_fees_collected: stats.total_fees_collected.unwrap_or_else(|| Decimal::ZERO),
        })
    }

    pub async fn get_payment_by_id(&self, payment_id: &str) -> Result<AddressOnlyPayment, ServiceError> {
        let payment = sqlx::query_as!(
            AddressOnlyPayment,
            r#"
            SELECT id, payment_id, merchant_id, crypto_type as "crypto_type: CryptoType",
                   gateway_deposit_address, merchant_destination_address,
                   requested_amount, COALESCE(customer_amount, requested_amount) as "customer_amount!",
                   processing_fee, forwarding_amount,
                   status as "status: AddressOnlyStatus", created_at
            FROM address_only_payments WHERE payment_id = $1
            "#,
            payment_id
        )
        .fetch_one(&self.db_pool)
        .await?;

        Ok(payment)
    }

    async fn update_payment_status(
        &self,
        payment_id: &str,
        status: AddressOnlyStatus,
    ) -> Result<(), ServiceError> {
        sqlx::query!(
            "UPDATE address_only_payments SET status = $1, updated_at = NOW() WHERE payment_id = $2",
            status as i32,
            payment_id
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    /// Update merchant fee payment setting
    pub async fn update_merchant_fee_setting(&self, merchant_id: i64, customer_pays_fee: bool) -> Result<(), ServiceError> {
        sqlx::query!(
            "UPDATE merchants SET customer_pays_fee = $1, updated_at = NOW() WHERE id = $2",
            customer_pays_fee,
            merchant_id
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    /// Get merchant fee payment setting
    pub async fn get_merchant_fee_setting(&self, merchant_id: i64) -> Result<bool, ServiceError> {
        let merchant = sqlx::query!(
            "SELECT customer_pays_fee FROM merchants WHERE id = $1",
            merchant_id
        )
        .fetch_one(&self.db_pool)
        .await?;

        Ok(merchant.customer_pays_fee)
    }
}
