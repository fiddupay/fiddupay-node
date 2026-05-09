// Address-Only Mode with Auto-Forwarding (Phase 1)
// Supports native and major tokens: ETH, BNB, MATIC, ARB, SOL, USDT, BUSD

use crate::error::ServiceError;
use crate::payment::models::CryptoType;
use crate::services::gas_fee_service::GasFeeService;
use crate::services::notification_service::NotificationService;
use alloy_primitives::U256;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
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
    pub last_tx_hash: Option<String>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "address_only_status", rename_all = "snake_case")]
pub enum AddressOnlyStatus {
    PendingPayment,
    PaymentReceived,
    PartialPaymentReceived,
    ForwardingInProgress,
    Completed,
    Failed,
}

#[derive(Clone)]
pub struct AddressOnlyService {
    db_pool: PgPool,
    gas_service: GasFeeService,
    config: crate::config::Config,
    notification_service: Arc<NotificationService>,
    balance_service: Arc<crate::services::balance_service::BalanceService>,
}

impl AddressOnlyService {
    pub fn new(
        db_pool: PgPool,
        gas_service: GasFeeService,
        config: crate::config::Config,
        notification_service: Arc<NotificationService>,
        balance_service: Arc<crate::services::balance_service::BalanceService>,
    ) -> Self {
        Self {
            db_pool,
            gas_service,
            config,
            notification_service,
            balance_service,
        }
    }

    /// Create payment request for address-only mode (native currencies only)
    pub async fn create_payment_request(
        &self,
        merchant_id: i64,
        crypto_type: CryptoType,
        merchant_address: String,
        requested_amount: Decimal,
    ) -> Result<AddressOnlyPayment, ServiceError> {
        // Validate supported currency
        if !self.is_supported_currency(crypto_type) {
            return Err(ServiceError::ValidationError(
                "Address-only mode currently supports ETH, BNB, MATIC, ARB, SOL, BTC, USDT, and BUSD".to_string()
            ));
        }

        let payment_id = Uuid::new_v4().to_string();
        let gateway_deposit_address = self.generate_deposit_address(crypto_type).await?;

        // Get merchant fee configuration
        let merchant_row = sqlx::query(
            "SELECT fee_percentage, COALESCE(customer_pays_fee, true) as customer_pays_fee FROM merchants WHERE id = $1"
        )
        .bind(merchant_id)
        .fetch_one(&self.db_pool)
        .await?;

        use sqlx::Row;
        // Calculate processing fee based on merchant configuration
        let fee_pct: Decimal = merchant_row.get("fee_percentage");
        let processing_fee = requested_amount * (fee_pct / Decimal::from(100)); // Convert percentage to decimal
        let customer_pays_fee: bool = merchant_row.get("customer_pays_fee");
        let (customer_amount, forwarding_amount) = if customer_pays_fee {
            // Customer pays fee: customer pays (requested + fee), merchant gets requested amount
            let customer_total = requested_amount + processing_fee;
            (customer_total, requested_amount)
        } else {
            // Merchant pays fee: customer pays requested amount, merchant gets (requested - fee)
            let merchant_receives = requested_amount - processing_fee;
            (requested_amount, merchant_receives)
        };

        let payment = sqlx::query_as::<_, AddressOnlyPayment>(
            r#"
            INSERT INTO address_only_payments (
                payment_id, merchant_id, crypto_type, gateway_deposit_address,
                merchant_destination_address, requested_amount, processing_fee,
                forwarding_amount, status, customer_amount, last_tx_hash
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, NULL)
            RETURNING id, payment_id, merchant_id, crypto_type,
                     gateway_deposit_address, merchant_destination_address,
                     requested_amount, customer_amount,
                     processing_fee, forwarding_amount,
                     status, last_tx_hash, created_at
            "#,
        )
        .bind(&payment_id)
        .bind(merchant_id)
        .bind(crypto_type as CryptoType)
        .bind(&gateway_deposit_address)
        .bind(&merchant_address)
        .bind(requested_amount)
        .bind(processing_fee)
        .bind(forwarding_amount)
        .bind(AddressOnlyStatus::PendingPayment as i32)
        .bind(customer_amount)
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
                "Received amount is less than requested".to_string(),
            ));
        }

        // Update status to received and save hash
        self.update_payment_status(
            payment_id,
            AddressOnlyStatus::PaymentReceived,
            Some(tx_hash),
        )
        .await?;

        // Create in-app notification with tx hash
        let _ = self
            .notification_service
            .create_notification(
                payment.merchant_id,
                "Payment Received (Address-Only)",
                &format!(
                    "Received {} {} (Tx: {}) (Payment ID: {})",
                    crate::utils::format::format_crypto_amount(received_amount),
                    payment.crypto_type,
                    tx_hash,
                    payment_id
                ),
                "success",
                "payment.received",
                false, // Mode tracking not fully implemented for AddressOnly yet
            )
            .await;

        // Initiate auto-forwarding
        self.initiate_auto_forwarding(&payment, tx_hash).await?;

        // 4. Trigger real-time UI update and invalidate dashboard cache
        let _ = self
            .balance_service
            .broadcast_balance_update(payment.merchant_id, false)
            .await;

        Ok(())
    }

    /// Process partial payment and notify merchant
    pub async fn process_partial_payment(
        &self,
        payment_id: &str,
        received_amount: Decimal,
        tx_hash: &str,
    ) -> Result<(), ServiceError> {
        // Get payment details
        let payment = self.get_payment_by_id(payment_id).await?;

        // Update status to partial payment received and save hash
        match payment.status {
            AddressOnlyStatus::PendingPayment | AddressOnlyStatus::PartialPaymentReceived => {
                self.update_payment_status(
                    payment_id,
                    AddressOnlyStatus::PartialPaymentReceived,
                    Some(tx_hash),
                )
                .await?;
            }
            _ => {
                // If already completed or forwarding, do nothing or log warning
                return Ok(());
            }
        }

        // Send webhook notification with partial payment details
        if let Ok(updated_payment) = self.get_payment_by_id(payment_id).await {
            // Create in-app notification with tx hash
            let _ = self
                .notification_service
                .create_notification(
                    updated_payment.merchant_id,
                    "Partial Payment Received (Address-Only)",
                    &format!(
                        "Received partial payment of {} {} (Tx: {}) (Payment ID: {})",
                        crate::utils::format::format_crypto_amount(received_amount),
                        updated_payment.crypto_type,
                        tx_hash,
                        payment_id
                    ),
                    "info",
                    "payment.partial",
                    false,
                )
                .await;

            let webhook_service =
                crate::services::webhook_notification_service::WebhookNotificationService::new(
                    self.db_pool.clone(),
                );
            let _ = webhook_service.notify_status_change(&updated_payment).await;
        }

        Ok(())
    }

    /// Auto-forward funds to merchant address
    async fn initiate_auto_forwarding(
        &self,
        payment: &AddressOnlyPayment,
        received_tx_hash: &str,
    ) -> Result<(), ServiceError> {
        // Update status to forwarding and preserve hash
        self.update_payment_status(
            &payment.payment_id,
            AddressOnlyStatus::ForwardingInProgress,
            Some(received_tx_hash),
        )
        .await?;

        // Get current gas estimate
        let gas_estimate = self
            .gas_service
            .get_gas_estimate(payment.crypto_type)
            .await?;

        // Calculate net forwarding amount (deduct gas fee)
        let net_forwarding_amount = payment.forwarding_amount - gas_estimate.standard_fee;

        if net_forwarding_amount <= Decimal::ZERO {
            return Err(ServiceError::ValidationError(
                "Forwarding amount too small after gas fees".to_string(),
            ));
        }

        // Send actual blockchain transaction
        let forwarding_tx_hash = self
            .send_forwarding_transaction(payment, net_forwarding_amount, &gas_estimate)
            .await?;

        // Record forwarding transaction
        sqlx::query(
            r#"
            INSERT INTO address_only_forwarding_txs (
                payment_id, destination_address, amount, gas_fee, tx_hash, status
            ) VALUES ($1, $2, $3, $4, $5, 'completed')
            "#,
        )
        .bind(&payment.payment_id)
        .bind(&payment.merchant_destination_address)
        .bind(net_forwarding_amount)
        .bind(gas_estimate.standard_fee)
        .bind(&forwarding_tx_hash)
        .execute(&self.db_pool)
        .await?;

        // Update payment status to completed
        self.update_payment_status(&payment.payment_id, AddressOnlyStatus::Completed, None)
            .await?;

        // Send webhook notification
        if let Ok(updated_payment) = self.get_payment_by_id(&payment.payment_id).await {
            let webhook_service =
                crate::services::webhook_notification_service::WebhookNotificationService::new(
                    self.db_pool.clone(),
                );
            let _ = webhook_service.notify_status_change(&updated_payment).await;
        }

        Ok(())
    }

    /// Generate unique deposit address for payment tracking
    async fn generate_deposit_address(
        &self,
        crypto_type: CryptoType,
    ) -> Result<String, ServiceError> {
        // Use existing KeyGenerator for real address generation
        use crate::utils::keygen::KeyGenerator;

        match crypto_type {
            CryptoType::Eth => "ethereum",
            CryptoType::Bnb => "bsc",
            CryptoType::Matic => "polygon",
            CryptoType::Arb => "arbitrum",
            CryptoType::Sol => "solana",
            CryptoType::Btc => "bitcoin",
            _ => {
                return Err(ServiceError::ValidationError(
                    "Unsupported crypto type".to_string(),
                ))
            }
        };

        let wallet = match crypto_type {
            CryptoType::Sol => KeyGenerator::generate_solana_wallet()?,
            CryptoType::Btc => {
                let is_sandbox = self.config.bitcoin_rpc_url.contains("testnet");
                KeyGenerator::generate_btc_wallet(is_sandbox)?
            }
            _ => KeyGenerator::generate_evm_wallet()?,
        };

        // Store private key securely for later forwarding
        let payment_id = uuid::Uuid::new_v4().to_string();
        self.store_deposit_keypair(&payment_id, &wallet.private_key, &wallet.address)
            .await?;

        Ok(wallet.address)
    }

    /// Check if crypto type is supported (Phase 1+)
    fn is_supported_currency(&self, crypto_type: CryptoType) -> bool {
        matches!(
            crypto_type,
            CryptoType::Eth
                | CryptoType::Bnb
                | CryptoType::Matic
                | CryptoType::Arb
                | CryptoType::Sol
                | CryptoType::Btc
                | CryptoType::UsdtBep20
                | CryptoType::BusdBep20
                | CryptoType::UsdtEth
                | CryptoType::UsdtPolygon
                | CryptoType::UsdtArbitrum
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

        sqlx::query(
            "INSERT INTO deposit_keypairs (payment_id, address, encrypted_private_key) VALUES ($1, $2, $3)"
        )
        .bind(payment_id)
        .bind(address)
        .bind(&encrypted_key)
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
        let private_key = self
            .get_deposit_private_key(&payment.gateway_deposit_address)
            .await?;

        match payment.crypto_type {
            CryptoType::Sol => {
                self.send_solana_transaction(
                    &private_key,
                    &payment.merchant_destination_address,
                    amount,
                )
                .await
            }
            CryptoType::Btc => {
                self.send_bitcoin_transaction(
                    &private_key,
                    &payment.merchant_destination_address,
                    amount,
                )
                .await
            }
            _ => {
                self.send_evm_transaction(
                    payment.crypto_type,
                    &private_key,
                    &payment.merchant_destination_address,
                    amount,
                    gas_estimate,
                )
                .await
            }
        }
    }

    /// Send Bitcoin transaction
    async fn send_bitcoin_transaction(
        &self,
        private_key: &str,
        to_address: &str,
        amount: Decimal,
    ) -> Result<String, ServiceError> {
        let tx_sender =
            crate::services::blockchain_transaction_sender::BlockchainTransactionSender::new(
                self.config.clone(),
            );
        let is_sandbox = self.config.bitcoin_rpc_url.contains("testnet");
        tx_sender
            .send_transaction(
                CryptoType::Btc,
                private_key,
                to_address,
                amount,
                None,
                is_sandbox,
            )
            .await
    }

    /// Send Solana transaction
    async fn send_solana_transaction(
        &self,
        private_key: &str,
        to_address: &str,
        amount: Decimal,
    ) -> Result<String, ServiceError> {
        let tx_sender =
            crate::services::blockchain_transaction_sender::BlockchainTransactionSender::new(
                self.config.clone(),
            );
        // For Phase 1 address-only mode, we default to false (mainnet) as the legacy table
        // doesn't have a sandbox_mode column yet.
        tx_sender
            .send_transaction(
                CryptoType::Sol,
                private_key,
                to_address,
                amount,
                None,
                false,
            )
            .await
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
        let tx_sender =
            crate::services::blockchain_transaction_sender::BlockchainTransactionSender::new(
                self.config.clone(),
            );

        // Convert gas price to U256
        let gas_price_wei = (gas_estimate.standard_fee
            * Decimal::new(1_000_000_000_000_000_000i64, 0))
        .to_u128()
        .map(U256::from);

        tx_sender
            .send_transaction(
                crypto_type,
                private_key,
                to_address,
                amount,
                gas_price_wei,
                false,
            )
            .await
    }

    /// Get private key for deposit address
    async fn get_deposit_private_key(&self, address: &str) -> Result<String, ServiceError> {
        let record_res =
            sqlx::query("SELECT encrypted_private_key FROM deposit_keypairs WHERE address = $1")
                .bind(address)
                .fetch_optional(&self.db_pool)
                .await;

        let _record = record_res?
            .ok_or_else(|| ServiceError::NotFound("Deposit keypair not found".to_string()))?;

        // For now, return a placeholder since we don't have decrypt_data
        // In production, this would decrypt the stored key
        Ok(format!("decrypted_key_for_{}", address))
    }

    /// Get merchant statistics for address-only payments
    pub async fn get_merchant_stats(
        &self,
        merchant_id: i64,
    ) -> Result<crate::api::address_only::AddressOnlyStats, ServiceError> {
        let stats_row = sqlx::query(
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
        )
        .bind(merchant_id)
        .fetch_one(&self.db_pool)
        .await?;

        use sqlx::Row;
        Ok(crate::api::address_only::AddressOnlyStats {
            total_payments: stats_row.get::<i64, _>("total_payments"),
            completed_payments: stats_row.get::<i64, _>("completed_payments"),
            pending_payments: stats_row.get::<i64, _>("pending_payments"),
            total_volume: stats_row.get::<Decimal, _>("total_volume"),
            total_fees_collected: stats_row.get::<Decimal, _>("total_fees_collected"),
        })
    }

    pub async fn get_payment_by_id(
        &self,
        payment_id: &str,
    ) -> Result<AddressOnlyPayment, ServiceError> {
        let payment = sqlx::query_as::<_, AddressOnlyPayment>(
            r#"
            SELECT id, payment_id, merchant_id, crypto_type,
                   gateway_deposit_address, merchant_destination_address,
                   requested_amount, COALESCE(customer_amount, requested_amount) as customer_amount,
                   processing_fee, forwarding_amount,
                   status, last_tx_hash, created_at
            FROM address_only_payments WHERE payment_id = $1
            "#,
        )
        .bind(payment_id)
        .fetch_one(&self.db_pool)
        .await?;

        Ok(payment)
    }

    async fn update_payment_status(
        &self,
        payment_id: &str,
        status: AddressOnlyStatus,
        tx_hash: Option<&str>,
    ) -> Result<(), ServiceError> {
        if let Some(hash) = tx_hash {
            sqlx::query(
                "UPDATE address_only_payments SET status = $1, last_tx_hash = $2, updated_at = NOW() WHERE payment_id = $3"
            )
            .bind(status as i32)
            .bind(hash)
            .bind(payment_id)
            .execute(&self.db_pool)
            .await?;
        } else {
            sqlx::query(
                "UPDATE address_only_payments SET status = $1, updated_at = NOW() WHERE payment_id = $2"
            )
            .bind(status as i32)
            .bind(payment_id)
            .execute(&self.db_pool)
            .await?;
        }

        Ok(())
    }

    /// Update merchant fee payment setting
    pub async fn update_merchant_fee_setting(
        &self,
        merchant_id: i64,
        customer_pays_fee: bool,
    ) -> Result<(), ServiceError> {
        sqlx::query(
            "UPDATE merchants SET customer_pays_fee = $1, updated_at = NOW() WHERE id = $2",
        )
        .bind(customer_pays_fee)
        .bind(merchant_id)
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    /// Get merchant fee payment setting
    pub async fn get_merchant_fee_setting(&self, merchant_id: i64) -> Result<bool, ServiceError> {
        let merchant = sqlx::query("SELECT COALESCE(customer_pays_fee, true) as customer_pays_fee FROM merchants WHERE id = $1")
            .bind(merchant_id)
            .fetch_optional(&self.db_pool)
            .await?;

        match merchant {
            Some(row) => {
                use sqlx::Row;
                Ok(row.get("customer_pays_fee"))
            }
            None => Err(ServiceError::MerchantNotFound),
        }
    }
}
