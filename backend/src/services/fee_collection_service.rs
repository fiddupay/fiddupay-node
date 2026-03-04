// Fee Collection Service
// Handles on-chain fee sweeping from merchant wallets to platform fee wallets.
//
// Works for both "managed" and "imported" settlement modes since the platform
// holds the encrypted private key in both cases.
//
// Fee logic & customer_pays_fee:
// ──────────────────────────────
// The fee amount to sweep is ALWAYS `payment.fee_amount`, regardless of who pays.
// This is because the fee split is already resolved at payment creation time:
//
//   customer_pays_fee = TRUE:
//     Customer paid (base + fee) → wallet received (base + fee)
//     Platform sweeps fee → merchant keeps base ✓
//
//   customer_pays_fee = FALSE:
//     Customer paid base → wallet received base
//     Platform sweeps fee → merchant keeps (base - fee) ✓
//
// The stored `fee_amount` is the correct sweep amount in both cases.

use crate::error::ServiceError;
use crate::payment::models::CryptoType;
use crate::utils::encryption::Encryption;
use rust_decimal::Decimal;
use sqlx::PgPool;
use tracing::{info, warn, error};

pub struct FeeCollectionService {
    db_pool: PgPool,
    config: crate::config::Config,
}

impl FeeCollectionService {
    pub fn new(db_pool: PgPool, config: crate::config::Config) -> Self {
        Self { db_pool, config }
    }

    /// Collect the platform fee after a payment is confirmed.
    ///
    /// 1. Check if the merchant uses "managed" or "imported" settlement mode
    ///    (both modes store encrypted private keys the platform controls).
    /// 2. Look up the merchant's wallet (encrypted private key).
    /// 3. Look up the platform fee wallet for the payment's network.
    /// 4. Send the fee amount from the merchant wallet to the platform fee wallet.
    /// 5. Record the fee collection transaction in the database.
    pub async fn collect_fee(
        &self,
        payment_id: i64,
        merchant_id: i64,
    ) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
        // 1. Check merchant settlement mode — only managed & imported have platform-held keys
        let merchant = sqlx::query(
            "SELECT settlement_mode FROM merchants WHERE id = $1"
        )
        .bind(merchant_id)
        .fetch_one(&self.db_pool)
        .await?;

        use sqlx::Row;
        let settlement_mode: String = merchant.get("settlement_mode");
        let settlement_mode = settlement_mode.as_str();
        match settlement_mode {
            "managed" | "imported" => {
                // Platform holds the private key — proceed with on-chain sweep
            }
            _ => {
                // "forwarding" or unknown — fees cannot be swept on-chain
                info!(
                    "Skipping fee collection for merchant {} (mode: {})",
                    merchant_id, settlement_mode
                );
                return Ok(None);
            }
        }

        // 2. Get payment details (crypto_type, fee_amount, to_address)
        let payment = sqlx::query(
            r#"
            SELECT crypto_type, fee_amount, to_address, network
            FROM payment_transactions
            WHERE id = $1
            "#
        )
        .bind(payment_id)
        .fetch_one(&self.db_pool)
        .await?;

        let crypto_type_str: Option<String> = payment.get("crypto_type");
        let crypto_type_str = crypto_type_str
            .as_deref()
            .ok_or("Payment has no crypto_type")?;
        let crypto_type = CryptoType::from_string(crypto_type_str)?;

        let fee_amount: Option<Decimal> = payment.get("fee_amount");
        let fee_amount = fee_amount
            .ok_or("Payment has no fee_amount")?;

        if fee_amount <= Decimal::ZERO {
            info!("No fee to collect for payment {}", payment_id);
            return Ok(None);
        }

        let merchant_wallet_address: Option<String> = payment.get("to_address");
        let merchant_wallet_address = merchant_wallet_address
            .as_deref()
            .ok_or("Payment has no to_address (merchant wallet)")?;

        // 3. Get the encrypted private key for the merchant's wallet
        let wallet_record = sqlx::query(
            "SELECT encrypted_private_key FROM merchant_wallets WHERE merchant_id = $1 AND address = $2 AND is_active = true"
        )
        .bind(merchant_id)
        .bind(merchant_wallet_address)
        .fetch_optional(&self.db_pool)
        .await?
        .ok_or_else(|| format!(
            "Wallet not found for merchant {} at address {}",
            merchant_id, merchant_wallet_address
        ))?;

        let encrypted_key: Option<String> = wallet_record.get("encrypted_private_key");
        let encrypted_key = encrypted_key
            .ok_or("Wallet has no encrypted private key")?;

        // Decrypt the private key
        let encryption = Encryption::new()
            .map_err(|e| format!("Encryption init failed: {}", e))?;
        let private_key = encryption.decrypt(&encrypted_key)
            .map_err(|e| format!("Key decryption failed: {}", e))?;

        // 4. Get the platform fee wallet for this network
        let network = self.crypto_type_to_network(crypto_type);
        let platform_wallet = sqlx::query(
            "SELECT address FROM platform_fee_wallets WHERE network = $1"
        )
        .bind(&network)
        .fetch_optional(&self.db_pool)
        .await?
        .ok_or_else(|| format!("No platform fee wallet configured for network: {}", network))?;

        let platform_wallet_address: String = platform_wallet.get("address");
        if platform_wallet_address.is_empty() {
            warn!("Platform fee wallet for {} is empty, skipping fee collection", network);
            return Ok(None);
        }

        // 5. Send the fee transaction
        info!(
            "Collecting fee {} {} from merchant {} wallet {} -> platform wallet {}",
            fee_amount, crypto_type_str, merchant_id, merchant_wallet_address, platform_wallet_address
        );

        let tx_sender = crate::services::blockchain_transaction_sender::BlockchainTransactionSender::new(
            self.config.clone()
        );

        // Fetch sandbox_mode from the payment table
        let payment_data = sqlx::query(
            "SELECT sandbox_mode FROM payment_transactions WHERE id = $1"
        )
        .bind(payment_id)
        .fetch_one(&self.db_pool)
        .await?;

        let sandbox_mode_val: bool = payment_data.get("sandbox_mode");

        let tx_hash = tx_sender
            .send_native_transaction(crypto_type, &private_key, &platform_wallet_address, fee_amount, None, sandbox_mode_val)
            .await
            .map_err(|e| {
                error!("Fee collection transaction failed for payment {}: {}", payment_id, e);
                format!("Fee transfer failed: {}", e)
            })?;

        // 6. Record the fee collection
        sqlx::query(
            r#"
            INSERT INTO fee_collections (
                payment_id, merchant_id, network, fee_amount,
                from_address, to_address, transaction_hash, status,
                created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, 'COMPLETED', NOW())
            "#
        )
        .bind(payment_id)
        .bind(merchant_id)
        .bind(&network)
        .bind(fee_amount)
        .bind(merchant_wallet_address)
        .bind(&platform_wallet_address)
        .bind(&tx_hash)
        .execute(&self.db_pool)
        .await?;

        info!(
            "✅ Fee collected for payment {}: {} {} -> tx {}",
            payment_id, fee_amount, crypto_type_str, tx_hash
        );

        Ok(Some(tx_hash))
    }

    /// Map CryptoType to the network name used in platform_fee_wallets table
    fn crypto_type_to_network(&self, crypto_type: CryptoType) -> &'static str {
        match crypto_type {
            CryptoType::Sol | CryptoType::UsdtSpl => "SOLANA",
            CryptoType::Eth | CryptoType::UsdtEth => "ETHEREUM",
            CryptoType::Bnb | CryptoType::UsdtBep20 => "BSC",
            CryptoType::Matic | CryptoType::UsdtPolygon => "POLYGON",
            CryptoType::Arb | CryptoType::UsdtArbitrum => "ARBITRUM",
        }
    }
}
