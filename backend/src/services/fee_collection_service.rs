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

    /// Collect accrued platform fees for a specific merchant wallet.
    ///
    /// This sweeps ALL unswept fees from CONFIRMED payments for this merchant wallet
    /// in a single on-chain transaction.
    pub async fn sweep_wallet_fees(
        &self,
        merchant_id: i64,
        crypto_type_str: &str,
        merchant_wallet_address: &str,
    ) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
        let crypto_type = CryptoType::from_string(crypto_type_str)?;

        // 1. Check merchant settlement mode
        let merchant = sqlx::query(
            "SELECT settlement_mode FROM merchants WHERE id = $1"
        )
        .bind(merchant_id)
        .fetch_one(&self.db_pool)
        .await?;

        use sqlx::Row;
        let settlement_mode: String = merchant.get("settlement_mode");
        match settlement_mode.as_str() {
            "managed" | "imported" => {}
            _ => {
                info!("Skipping fee sweep for merchant {} (mode: {})", merchant_id, settlement_mode);
                return Ok(None);
            }
        }

        // 2. Sum unswept fees for this wallet
        let sum_record = sqlx::query(
            r#"
            SELECT COALESCE(SUM(fee_amount), 0) as total_fee
            FROM payment_transactions
            WHERE merchant_id = $1
              AND to_address = $2
              AND crypto_type = $3
              AND status = 'CONFIRMED'
              AND fee_collected = FALSE
            "#
        )
        .bind(merchant_id)
        .bind(merchant_wallet_address)
        .bind(crypto_type_str)
        .fetch_one(&self.db_pool)
        .await?;

        let total_fee: Decimal = sum_record.get("total_fee");

        if total_fee <= Decimal::ZERO {
            info!("No accumulated fees to sweep for wallet {}", merchant_wallet_address);
            return Ok(None);
        }

        // 3. Get the encrypted private key for the merchant's wallet
        let wallet_record = sqlx::query(
            "SELECT encrypted_private_key FROM merchant_wallets WHERE merchant_id = $1 AND address = $2 AND is_active = true"
        )
        .bind(merchant_id)
        .bind(merchant_wallet_address)
        .fetch_optional(&self.db_pool)
        .await?
        .ok_or_else(|| format!("Wallet not found for merchant {} at {}", merchant_id, merchant_wallet_address))?;

        let encrypted_key: String = wallet_record.get("encrypted_private_key");

        let encryption = Encryption::new().map_err(|e| format!("Encryption init failed: {}", e))?;
        let private_key = encryption.decrypt(&encrypted_key).map_err(|e| format!("Key decryption failed: {}", e))?;

        // 4. Get the platform fee wallet for this network
        let network = self.crypto_type_to_network(crypto_type.clone());
        let platform_wallet = sqlx::query(
            "SELECT address FROM platform_fee_wallets WHERE network = $1"
        )
        .bind(&network)
        .fetch_optional(&self.db_pool)
        .await?
        .ok_or_else(|| format!("No platform fee wallet configured for network: {}", network))?;

        let platform_wallet_address: String = platform_wallet.get("address");

        info!(
            "Sweeping total fee {} {} from wallet {} -> platform wallet {}",
            total_fee, crypto_type_str, merchant_wallet_address, platform_wallet_address
        );

        let tx_sender = crate::services::blockchain_transaction_sender::BlockchainTransactionSender::new(self.config.clone());

        // For simplicity in batching, we assume sandbox status is consistent with the merchant
        let merchant_sandbox: bool = sqlx::query("SELECT sandbox_mode FROM merchants WHERE id = $1")
            .bind(merchant_id)
            .fetch_one(&self.db_pool)
            .await?
            .get("sandbox_mode");

        let tx_hash = tx_sender
            .send_transaction(crypto_type, &private_key, &platform_wallet_address, total_fee, None, merchant_sandbox)
            .await
            .map_err(|e| format!("Fee sweep transfer failed: {}", e))?;

        // 5. Update payment records to mark fees as collected
        sqlx::query(
            r#"
            UPDATE payment_transactions
            SET fee_collected = TRUE
            WHERE merchant_id = $1
              AND to_address = $2
              AND crypto_type = $3
              AND status = 'CONFIRMED'
              AND fee_collected = FALSE
            "#
        )
        .bind(merchant_id)
        .bind(merchant_wallet_address)
        .bind(crypto_type_str)
        .execute(&self.db_pool)
        .await?;

        // 6. Record the fee sweep transaction
        sqlx::query(
            r#"
            INSERT INTO fee_collections (
                merchant_id, network, fee_amount,
                from_address, to_address, transaction_hash, status,
                created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, 'COMPLETED', NOW())
            "#
        )
        .bind(merchant_id)
        .bind(&network)
        .bind(total_fee)
        .bind(merchant_wallet_address)
        .bind(&platform_wallet_address)
        .bind(&tx_hash)
        .execute(&self.db_pool)
        .await?;

        info!("✅ Smart sweep collected {} {} -> tx {}", total_fee, crypto_type_str, tx_hash);

        Ok(Some(tx_hash))
    }

    /// Evaluates thresholds and triggers sweeps for all eligible merchant wallets on a given network.
    pub async fn sweep_all_eligible(
        &self,
        network_str: &str,
    ) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        // 1. Check if auto sweep is enabled and get settings
        let settings = match sqlx::query(
            "SELECT is_auto_sweep_enabled, min_accumulated_usd FROM fee_sweep_settings LIMIT 1"
        )
        .fetch_optional(&self.db_pool)
        .await? {
            Some(row) => row,
            None => {
                info!("Fee sweep settings not found. Auto-sweep skipped.");
                return Ok(vec![]);
            }
        };

        use sqlx::Row;
        let is_enabled: bool = settings.get("is_auto_sweep_enabled");
        if !is_enabled {
            return Ok(vec![]);
        }

        let min_usd: Decimal = settings.try_get("min_accumulated_usd").unwrap_or(Decimal::ZERO);

        // 2. Find all eligible wallets
        let eligible_wallets = sqlx::query(
            r#"
            SELECT merchant_id, crypto_type, to_address, SUM(fee_amount_usd) as total_usd
            FROM payment_transactions
            WHERE network = $1
              AND status = 'CONFIRMED'
              AND fee_collected = FALSE
            GROUP BY merchant_id, crypto_type, to_address
            HAVING SUM(fee_amount_usd) >= $2
            "#
        )
        .bind(network_str)
        .bind(min_usd)
        .fetch_all(&self.db_pool)
        .await?;

        let mut successful_txs = Vec::new();

        for wallet in eligible_wallets {
            let merchant_id: i64 = wallet.get("merchant_id");
            let crypto_type_str: String = wallet.get("crypto_type");
            let to_address: String = wallet.get("to_address");

            match self.sweep_wallet_fees(merchant_id, &crypto_type_str, &to_address).await {
                Ok(Some(tx_hash)) => successful_txs.push(tx_hash),
                Ok(None) => {}, // Skipped
                Err(e) => {
                    error!("Error sweeping fees for merchant {} wallet {}: {}", merchant_id, to_address, e);
                }
            }
        }

        Ok(successful_txs)
    }

    /// Background task to run scheduled sweeps
    pub async fn start_auto_sweeper(&self) {
        let networks = vec!["ETHEREUM", "BSC", "POLYGON", "ARBITRUM", "SOLANA", "BITCOIN"];
        
        loop {
            // Check once per hour
            tokio::time::sleep(std::time::Duration::from_secs(3600)).await;

            // Simple verification if sweep is enabled
            let settings = sqlx::query(
                "SELECT is_auto_sweep_enabled FROM fee_sweep_settings LIMIT 1"
            )
            .fetch_optional(&self.db_pool)
            .await;

            let is_enabled = match settings {
                Ok(Some(row)) => {
                    use sqlx::Row;
                    row.get("is_auto_sweep_enabled")
                },
                _ => false,
            };

            if is_enabled {
                info!("Running periodic auto-sweep check for eligible wallets...");
                for network in &networks {
                    if let Err(e) = self.sweep_all_eligible(network).await {
                        error!("Auto-sweep failed for network {}: {}", network, e);
                    }
                }
            }
        }
    }

    /// Map CryptoType to the network name used in platform_fee_wallets table
    fn crypto_type_to_network(&self, crypto_type: CryptoType) -> &'static str {
        match crypto_type {
            CryptoType::Sol | CryptoType::UsdtSpl | CryptoType::WSol => "SOLANA",
            CryptoType::Eth | CryptoType::UsdtEth => "ETHEREUM",
            CryptoType::Bnb | CryptoType::UsdtBep20 | CryptoType::BusdBep20 => "BSC",
            CryptoType::Matic | CryptoType::UsdtPolygon => "POLYGON",
            CryptoType::Arb | CryptoType::UsdtArbitrum => "ARBITRUM",
            CryptoType::Btc => "BITCOIN",
        }
    }
}
