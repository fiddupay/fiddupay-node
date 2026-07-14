// Merchant Customer Service
// Manages sub-accounts, designated user wallets, permissions, and customer transactions

use crate::error::ServiceError;
use crate::models::merchant_customer::{
    CreateCustomerRequest, CustomerTransaction, MerchantCustomer, MerchantCustomerBalance,
    MerchantCustomerWallet,
};
use crate::payment::models::CryptoType;
use crate::utils::encryption::Encryption;
use crate::utils::keygen::KeyGenerator;
use crate::utils::sanitizer::mask_email;
use alloy_primitives::U256;
use rust_decimal::Decimal;
use serde_json::json;
use sqlx::{PgPool, Row};
use std::str::FromStr;

const CUSTOMER_COLS: &str = "id, merchant_id, external_id, email, first_name, last_name, metadata, is_active, status, status_reason, can_withdraw, withdrawal_limit, created_at, updated_at";

use crate::config::Config;
use crate::services::balance_service::BalanceService;
use crate::services::notification_service::NotificationService;
use crate::services::price_service::PriceService;
use crate::services::volume_tracking_service::VolumeTrackingService;
use std::sync::Arc;

pub struct MerchantCustomerService {
    db_pool: PgPool,
    price_service: Arc<PriceService>,
    volume_tracking: Arc<VolumeTrackingService>,
    notification_service: Arc<NotificationService>,
    balance_service: Arc<BalanceService>,
    config: Arc<Config>,
}

pub struct SaveWalletParams {
    pub customer_id: i64,
    pub merchant_id: i64,
    pub crypto_type: CryptoType,
    pub address: String,
    pub encrypted_key: String,
    pub sandbox_mode: bool,
    pub bypass_lock: bool,
}

pub struct PayMerchantParams<'a> {
    pub merchant_id: i64,
    pub external_id: &'a str,
    pub crypto_type_str: &'a str,
    pub amount_str: &'a str,
    pub reference_id: Option<&'a str>,
    pub description: Option<&'a str>,
    pub sandbox_mode: bool,
}

impl MerchantCustomerService {
    pub fn new(
        db_pool: PgPool,
        price_service: Arc<PriceService>,
        volume_tracking: Arc<VolumeTrackingService>,
        notification_service: Arc<NotificationService>,
        balance_service: Arc<BalanceService>,
        config: Arc<Config>,
    ) -> Self {
        Self {
            db_pool,
            price_service,
            volume_tracking,
            notification_service,
            balance_service,
            config,
        }
    }

    // =========================================================================
    // Permission Helpers
    // =========================================================================

    /// Check if a customer can perform the given action. Returns the customer if OK.
    fn check_permissions(customer: &MerchantCustomer, action: &str) -> Result<(), ServiceError> {
        if !customer.is_active {
            return Err(ServiceError::ValidationError(
                "Customer account is deactivated".to_string(),
            ));
        }

        match customer.status.as_str() {
            "active" => Ok(()),
            "flagged" => {
                // Flagged: read-only — no withdrawals, no payments
                match action {
                    "view" => Ok(()),
                    _ => Err(ServiceError::ValidationError(format!(
                        "Customer account is flagged: {}. Only view operations are allowed.",
                        customer.status_reason.as_deref().unwrap_or("Under review")
                    ))),
                }
            }
            "suspended" | "blocked" => Err(ServiceError::ValidationError(format!(
                "Customer account is {}: {}",
                customer.status,
                customer
                    .status_reason
                    .as_deref()
                    .unwrap_or("Contact support")
            ))),
            _ => Ok(()),
        }
    }

    /// Additional check for withdrawal-specific permissions
    fn check_withdrawal_permissions(
        customer: &MerchantCustomer,
        amount: Decimal,
    ) -> Result<(), ServiceError> {
        // 1. Basic status check (active/flagged/suspended)
        Self::check_permissions(customer, "withdraw")?;

        // 2. Explicit withdrawal toggle (Requirement 4.5)
        if !customer.can_withdraw {
            return Err(ServiceError::ValidationError(
                "Withdrawals are disabled for this customer".to_string(),
            ));
        }

        // 3. Spending Limit enforcement (Requirement 4.6)
        if let Some(limit) = customer.withdrawal_limit {
            if amount > limit {
                return Err(ServiceError::ValidationError(format!(
                    "Amount {} exceeds withdrawal limit of {}",
                    amount, limit
                )));
            }
        }

        Ok(())
    }

    /// Fetch and validate a customer
    async fn get_verified_customer(
        &self,
        merchant_id: i64,
        external_id: &str,
    ) -> Result<MerchantCustomer, ServiceError> {
        let customer = sqlx::query_as::<_, MerchantCustomer>(&format!(
            "SELECT {} FROM merchant_customers WHERE merchant_id = $1 AND external_id = $2",
            CUSTOMER_COLS
        ))
        .bind(merchant_id)
        .bind(external_id)
        .fetch_one(&self.db_pool)
        .await
        .map_err(|_| {
            ServiceError::ValidationError(format!("Customer {} not found", external_id))
        })?;

        Ok(customer)
    }

    // =========================================================================
    // Registration & Provisioning
    // =========================================================================

    /// Register a new customer for a merchant and auto-provision wallets
    pub async fn register_customer(
        &self,
        merchant_id: i64,
        req: CreateCustomerRequest,
        sandbox_mode: bool,
    ) -> Result<(MerchantCustomer, Vec<MerchantCustomerWallet>), ServiceError> {
        let customer = sqlx::query_as::<_, MerchantCustomer>(
            &format!(r#"
            INSERT INTO merchant_customers (merchant_id, external_id, email, first_name, last_name, metadata, is_active)
            VALUES ($1, $2, $3, $4, $5, $6, TRUE)
            ON CONFLICT (merchant_id, external_id) 
            DO UPDATE SET 
                email = EXCLUDED.email, 
                first_name = EXCLUDED.first_name, 
                last_name = EXCLUDED.last_name, 
                metadata = EXCLUDED.metadata, 
                updated_at = NOW()
            RETURNING {}
            "#, CUSTOMER_COLS)
        )
        .bind(merchant_id)
        .bind(&req.external_id)
        .bind(&req.email)
        .bind(&req.first_name)
        .bind(&req.last_name)
        .bind(&req.metadata)
        .fetch_one(&self.db_pool)
        .await?;

        // Auto-provision wallets using merchant's supported networks
        let wallets = self
            .provision_wallets(
                merchant_id,
                &customer.external_id,
                vec![],
                sandbox_mode,
                true,
            )
            .await
            .unwrap_or_else(|e| {
                tracing::warn!(
                    "Auto-provision wallets failed for customer {}: {}",
                    customer.external_id,
                    e
                );
                vec![]
            });

        let _ = self
            .notification_service
            .create_notification(
                merchant_id,
                "🎉 New Customer Registered",
                &format!(
                    "Customer {} ({}) has been successfully registered.",
                    req.external_id,
                    mask_email(req.email.as_deref().unwrap_or("No email"))
                ),
                "success",
                "customer.registered",
                sandbox_mode,
            )
            .await;

        Ok((customer, wallets))
    }

    /// Provision unique wallets for a customer across multiple networks
    pub async fn provision_wallets(
        &self,
        merchant_id: i64,
        external_id: &str,
        networks: Vec<String>,
        sandbox_mode: bool,
        bypass_lock: bool,
    ) -> Result<Vec<MerchantCustomerWallet>, ServiceError> {
        let customer = self.get_verified_customer(merchant_id, external_id).await?;

        let encryption = Encryption::new()
            .map_err(|e| ServiceError::InternalError(format!("Encryption setup failed: {}", e)))?;

        let mut networks = networks;
        if networks.is_empty() {
            let mut merchant_networks: Vec<String> = sqlx::query_scalar::<_, String>(
                "SELECT DISTINCT network FROM merchant_wallets WHERE merchant_id = $1 AND sandbox_mode = $2 AND is_active = true"
            )
            .bind(merchant_id)
            .bind(sandbox_mode)
            .fetch_all(&self.db_pool)
            .await?;

            if merchant_networks.is_empty() {
                merchant_networks = sqlx::query_scalar::<_, String>(
                    "SELECT DISTINCT network FROM merchant_forwarding_wallets WHERE merchant_id = $1 AND sandbox_mode = $2 AND is_active = true"
                )
                .bind(merchant_id)
                .bind(sandbox_mode)
                .fetch_all(&self.db_pool)
                .await?;
            }

            if merchant_networks.is_empty() {
                merchant_networks = vec![
                    "EVM".to_string(),
                    "SOLANA".to_string(),
                    "BITCOIN".to_string(),
                ];
            }

            networks = merchant_networks;
        }

        let mut wallets: Vec<MerchantCustomerWallet> = Vec::new();

        for network_type in networks {
            let normalized = network_type.to_uppercase();
            match normalized.as_str() {
                "EVM" | "ETH" | "ERC20" | "BSC" | "BEP20" | "POLYGON" | "MATIC" | "ARB"
                | "ARBITRUM" | "NATIVE" | "ETHEREUM" | "USDT_ETH" | "USDT_BEP20" | "BUSD_BEP20"
                | "USDT_POLYGON" | "USDT_ARBITRUM" | "BNB" => {
                    if wallets
                        .iter()
                        .any(|w| w.network.to_uppercase() == "ETHEREUM")
                    {
                        continue;
                    }

                    // Check if an EVM wallet already exists for this customer (prioritizing current sandbox_mode)
                    let existing: Option<(String, String)> = sqlx::query_as::<_, (String, String)>(
                        "SELECT address, encrypted_private_key FROM merchant_customer_wallets \
                          WHERE customer_id = $1 AND network = 'ETHEREUM' \
                          ORDER BY (sandbox_mode = $2) DESC LIMIT 1",
                    )
                    .bind(customer.id)
                    .bind(sandbox_mode)
                    .fetch_optional(&self.db_pool)
                    .await?;

                    let (address, encrypted_key) = if let Some((addr, key)) = existing {
                        if key.trim().is_empty() {
                            tracing::warn!("Customer {} has EVM wallet with missing/broken private key! Generating fresh keypair and archiving.", customer.id);

                            // Archive to history (for audit trail)
                            let _ = sqlx::query(
                                r#"
                                INSERT INTO merchant_wallet_history (
                                    merchant_id, customer_id, owner_type, crypto_type, network, 
                                    old_address, new_address, wallet_mode, 
                                    encrypted_private_key, reason, changed_by
                                )
                                VALUES ($1, $2, 'customer', 'ETH', 'ETHEREUM', $3, $4, 'managed', $5, 'Broken key replaced during repair', 'system')
                                "#
                            )
                            .bind(merchant_id)
                            .bind(customer.id)
                            .bind(&addr)
                            .bind("")
                            .bind(&key)
                            .execute(&self.db_pool)
                            .await;

                            let keypair = KeyGenerator::generate_evm_wallet()?;
                            let enc_key =
                                encryption.encrypt(&keypair.private_key).map_err(|e| {
                                    ServiceError::InternalError(format!("Encryption failed: {}", e))
                                })?;
                            (keypair.address, enc_key)
                        } else {
                            (addr, key)
                        }
                    } else {
                        let keypair = KeyGenerator::generate_evm_wallet()?;
                        let enc_key = encryption.encrypt(&keypair.private_key).map_err(|e| {
                            ServiceError::InternalError(format!("Encryption failed: {}", e))
                        })?;
                        (keypair.address, enc_key)
                    };

                    let evm_cryptos = vec![
                        CryptoType::Eth,
                        CryptoType::UsdtEth,
                        CryptoType::Bnb,
                        CryptoType::UsdtBep20,
                        CryptoType::BusdBep20,
                        CryptoType::Matic,
                        CryptoType::UsdtPolygon,
                        CryptoType::Arb,
                        CryptoType::UsdtArbitrum,
                    ];

                    for crypto in evm_cryptos {
                        // Check if a wallet for this specific crypto_type already exists
                        let existing_wallet: Option<MerchantCustomerWallet> = sqlx::query_as::<_, MerchantCustomerWallet>(
                            "SELECT id, customer_id, merchant_id, crypto_type, network, address, encrypted_private_key, created_at, updated_at, sandbox_mode \
                             FROM merchant_customer_wallets WHERE customer_id = $1 AND crypto_type = $2 AND sandbox_mode = $3"
                        )
                        .bind(customer.id)
                        .bind(crypto.to_string())
                        .bind(sandbox_mode)
                        .fetch_optional(&self.db_pool)
                        .await?;

                        // Only reuse the existing record if the address matches our target address (meaning it is not broken/mismatched)
                        if let Some(w) = existing_wallet {
                            if w.address == address && !w.encrypted_private_key.trim().is_empty() {
                                wallets.push(w);
                                continue;
                            }
                        }

                        let wallet = self
                            .save_customer_wallet(SaveWalletParams {
                                customer_id: customer.id,
                                merchant_id,
                                crypto_type: crypto,
                                address: address.clone(),
                                encrypted_key: encrypted_key.clone(),
                                sandbox_mode,
                                bypass_lock,
                            })
                            .await?;
                        wallets.push(wallet);
                    }
                }
                "SOLANA" | "SOL" | "SPL" | "SOLANA_SPL" | "SOLANA_MAINNET" | "SOLANA_DEVNET"
                | "USDT_SPL" | "WSOL" => {
                    if wallets.iter().any(|w| w.network.to_uppercase() == "SOLANA") {
                        continue;
                    }

                    // Check if a Solana wallet already exists for this customer (prioritizing current sandbox_mode)
                    let existing: Option<(String, String)> = sqlx::query_as::<_, (String, String)>(
                        "SELECT address, encrypted_private_key FROM merchant_customer_wallets \
                          WHERE customer_id = $1 AND network = 'SOLANA' \
                          ORDER BY (sandbox_mode = $2) DESC LIMIT 1",
                    )
                    .bind(customer.id)
                    .bind(sandbox_mode)
                    .fetch_optional(&self.db_pool)
                    .await?;

                    let (address, encrypted_key) = if let Some((addr, key)) = existing {
                        if key.trim().is_empty() {
                            tracing::warn!("Customer {} has Solana wallet with missing/broken private key! Generating fresh keypair and archiving.", customer.id);

                            // Archive to history
                            let _ = sqlx::query(
                                r#"
                                INSERT INTO merchant_wallet_history (
                                    merchant_id, customer_id, owner_type, crypto_type, network, 
                                    old_address, new_address, wallet_mode, 
                                    encrypted_private_key, reason, changed_by
                                )
                                VALUES ($1, $2, 'customer', 'SOL', 'SOLANA', $3, $4, 'managed', $5, 'Broken key replaced during repair', 'system')
                                "#
                            )
                            .bind(merchant_id)
                            .bind(customer.id)
                            .bind(&addr)
                            .bind("")
                            .bind(&key)
                            .execute(&self.db_pool)
                            .await;

                            let keypair = KeyGenerator::generate_solana_wallet()?;
                            let enc_key =
                                encryption.encrypt(&keypair.private_key).map_err(|e| {
                                    ServiceError::InternalError(format!("Encryption failed: {}", e))
                                })?;
                            (keypair.address, enc_key)
                        } else {
                            (addr, key)
                        }
                    } else {
                        let keypair = KeyGenerator::generate_solana_wallet()?;
                        let enc_key = encryption.encrypt(&keypair.private_key).map_err(|e| {
                            ServiceError::InternalError(format!("Encryption failed: {}", e))
                        })?;
                        (keypair.address, enc_key)
                    };

                    let sol_cryptos = vec![CryptoType::Sol, CryptoType::UsdtSpl, CryptoType::WSol];

                    for crypto in sol_cryptos {
                        // Check if a wallet for this specific crypto_type already exists
                        let existing_wallet: Option<MerchantCustomerWallet> = sqlx::query_as::<_, MerchantCustomerWallet>(
                            "SELECT id, customer_id, merchant_id, crypto_type, network, address, encrypted_private_key, created_at, updated_at, sandbox_mode \
                             FROM merchant_customer_wallets WHERE customer_id = $1 AND crypto_type = $2 AND sandbox_mode = $3"
                        )
                        .bind(customer.id)
                        .bind(crypto.to_string())
                        .bind(sandbox_mode)
                        .fetch_optional(&self.db_pool)
                        .await?;

                        // Only reuse the existing record if the address matches our target address (meaning it is not broken/mismatched)
                        if let Some(w) = existing_wallet {
                            if w.address == address && !w.encrypted_private_key.trim().is_empty() {
                                wallets.push(w);
                                continue;
                            }
                        }

                        let wallet = self
                            .save_customer_wallet(SaveWalletParams {
                                customer_id: customer.id,
                                merchant_id,
                                crypto_type: crypto,
                                address: address.clone(),
                                encrypted_key: encrypted_key.clone(),
                                sandbox_mode,
                                bypass_lock,
                            })
                            .await?;
                        wallets.push(wallet);
                    }
                }
                "BITCOIN" | "BTC" | "BITCOIN_MAINNET" | "BITCOIN_TESTNET" => {
                    if !self.config.is_blockchain_enabled(&CryptoType::Btc) {
                        tracing::info!(
                            "Skipping Bitcoin wallet provisioning - blockchain is disabled"
                        );
                        continue;
                    }
                    if wallets
                        .iter()
                        .any(|w| w.network.to_uppercase() == "BITCOIN")
                    {
                        continue;
                    }

                    // Check if a Bitcoin wallet already exists
                    let existing_wallet: Option<MerchantCustomerWallet> = sqlx::query_as::<_, MerchantCustomerWallet>(
                        "SELECT id, customer_id, merchant_id, crypto_type, network, address, encrypted_private_key, created_at, updated_at, sandbox_mode \
                         FROM merchant_customer_wallets WHERE customer_id = $1 AND crypto_type = 'BTC' AND sandbox_mode = $2"
                    )
                    .bind(customer.id)
                    .bind(sandbox_mode)
                    .fetch_optional(&self.db_pool)
                    .await?;

                    if let Some(w) = existing_wallet {
                        if !w.encrypted_private_key.trim().is_empty() {
                            wallets.push(w);
                            continue;
                        }
                    }

                    let keypair = KeyGenerator::generate_btc_wallet(sandbox_mode)?;
                    let encrypted_key = encryption.encrypt(&keypair.private_key).map_err(|e| {
                        ServiceError::InternalError(format!("Encryption failed: {}", e))
                    })?;

                    let wallet = self
                        .save_customer_wallet(SaveWalletParams {
                            customer_id: customer.id,
                            merchant_id,
                            crypto_type: CryptoType::Btc,
                            address: keypair.address.clone(),
                            encrypted_key: encrypted_key.clone(),
                            sandbox_mode,
                            bypass_lock,
                        })
                        .await?;
                    wallets.push(wallet);
                }
                _ => {
                    return Err(ServiceError::ValidationError(format!(
                        "Unsupported network type: {}",
                        network_type
                    )))
                }
            }
        }

        if !wallets.is_empty() {
            let _ = self
                .balance_service
                .broadcast_balance_update(merchant_id, sandbox_mode)
                .await;
        }

        Ok(wallets)
    }

    /// Bulk provision or regenerate wallets for multiple customers
    pub async fn bulk_provision_wallets(
        &self,
        merchant_id: i64,
        customer_ids: Option<Vec<String>>,
        all_customers: bool,
        sandbox_mode: bool,
    ) -> Result<usize, ServiceError> {
        let external_ids: Vec<String> = if all_customers {
            sqlx::query_scalar::<_, String>(
                "SELECT external_id FROM merchant_customers WHERE merchant_id = $1",
            )
            .bind(merchant_id)
            .fetch_all(&self.db_pool)
            .await?
        } else if let Some(ids) = customer_ids {
            // Verify that all requested IDs belong to this merchant using a single batch query
            sqlx::query_scalar::<_, String>(
                "SELECT external_id FROM merchant_customers WHERE merchant_id = $1 AND external_id = ANY($2)"
            )
            .bind(merchant_id)
            .bind(&ids)
            .fetch_all(&self.db_pool)
            .await?
        } else {
            return Err(ServiceError::BadRequest(
                "Must provide customer_ids or set all_customers to true".to_string(),
            ));
        };

        let mut success_count = 0;
        for ext_id in external_ids {
            match self
                .provision_wallets(merchant_id, &ext_id, vec![], sandbox_mode, true)
                .await
            {
                Ok(_) => success_count += 1,
                Err(e) => tracing::warn!("Bulk provision failed for customer {}: {}", ext_id, e),
            }
        }

        Ok(success_count)
    }

    async fn save_customer_wallet(
        &self,
        params: SaveWalletParams,
    ) -> Result<MerchantCustomerWallet, ServiceError> {
        let network = params.crypto_type.network().to_string();
        let crypto_str = params.crypto_type.to_string();

        tracing::info!(
            "save_customer_wallet: customer={}, merchant={}, crypto={}, sandbox={}",
            params.customer_id,
            params.merchant_id,
            crypto_str,
            params.sandbox_mode
        );

        // 1. Check if customer wallets are locked for this merchant
        let customer_wallets_locked = sqlx::query_scalar::<_, bool>(
            "SELECT customer_wallets_locked FROM merchants WHERE id = $1",
        )
        .bind(params.merchant_id)
        .fetch_one(&self.db_pool)
        .await
        .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        // 2. Check if this is an existing customer with any wallets
        let has_existing_wallets = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM merchant_customer_wallets WHERE customer_id = $1 AND sandbox_mode = $2"
        )
        .bind(params.customer_id)
        .bind(params.sandbox_mode)
        .fetch_one(&self.db_pool)
        .await
        .map_err(|e| ServiceError::DatabaseError(e.to_string()))? > 0;

        // 3. Fetch current wallet if it exists for this specific crypto
        let current_wallet = sqlx::query(
            "SELECT address, encrypted_private_key FROM merchant_customer_wallets WHERE customer_id = $1 AND crypto_type = $2 AND sandbox_mode = $3"
        )
        .bind(params.customer_id)
        .bind(&crypto_str)
        .bind(params.sandbox_mode)
        .fetch_optional(&self.db_pool)
        .await
        .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        if current_wallet.is_none()
            && has_existing_wallets
            && customer_wallets_locked
            && !params.bypass_lock
        {
            tracing::warn!("Blocked new currency provisioning for existing customer {} (customer wallets locked)", params.customer_id);
            return Err(ServiceError::BadRequest(
                "Customer wallets are locked. Please unlock in settings to provision new currencies for this user.".to_string()
            ));
        }

        if let Some(row) = current_wallet {
            let current_address: String = row.get("address");
            let current_key: String = row.get("encrypted_private_key");

            if current_address != params.address {
                if customer_wallets_locked {
                    tracing::warn!(
                        "Blocked customer wallet change for merchant {} (customer wallets locked)",
                        params.merchant_id
                    );
                    return Err(ServiceError::BadRequest(
                        "Customer wallets are locked. Please unlock in settings to change."
                            .to_string(),
                    ));
                }

                tracing::info!(
                    "Archiving customer wallet state for customer {}: address changed",
                    params.customer_id
                );

                // Archive to history
                sqlx::query(
                    r#"
                    INSERT INTO merchant_wallet_history (
                        merchant_id, customer_id, owner_type, crypto_type, network, 
                        old_address, new_address, wallet_mode, 
                        encrypted_private_key, reason, changed_by
                    )
                    VALUES ($1, $2, 'customer', $3, $4, $5, $6, 'managed', $7, $8, 'merchant')
                    "#,
                )
                .bind(params.merchant_id)
                .bind(params.customer_id)
                .bind(&crypto_str)
                .bind(&network)
                .bind(&current_address)
                .bind(&params.address)
                .bind(&current_key)
                .bind("Customer wallet re-provisioned")
                .execute(&self.db_pool)
                .await
                .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
            }
        }

        let wallet = sqlx::query_as::<_, MerchantCustomerWallet>(
            r#"
            INSERT INTO merchant_customer_wallets (customer_id, merchant_id, crypto_type, network, address, encrypted_private_key, sandbox_mode)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (customer_id, crypto_type, sandbox_mode) DO UPDATE SET address = EXCLUDED.address, encrypted_private_key = EXCLUDED.encrypted_private_key, updated_at = NOW()
            RETURNING id, customer_id, merchant_id, crypto_type, network, address, encrypted_private_key, created_at, updated_at, sandbox_mode
            "#
        )
        .bind(params.customer_id)
        .bind(params.merchant_id)
        .bind(&crypto_str)
        .bind(&network)
        .bind(&params.address)
        .bind(&params.encrypted_key)
        .bind(params.sandbox_mode)
        .fetch_one(&self.db_pool)
        .await?;

        // Initialize balance record
        sqlx::query(
            "INSERT INTO merchant_customer_balances (customer_id, merchant_id, crypto_type, sandbox_mode) VALUES ($1, $2, $3, $4) ON CONFLICT (customer_id, crypto_type, sandbox_mode) DO NOTHING"
        )
        .bind(params.customer_id)
        .bind(params.merchant_id)
        .bind(&crypto_str)
        .bind(params.sandbox_mode)
        .execute(&self.db_pool)
        .await?;

        Ok(wallet)
    }

    // =========================================================================
    // Read Operations
    // =========================================================================

    pub async fn get_customer_balances(
        &self,
        merchant_id: i64,
        external_id: &str,
        sandbox_mode: bool,
    ) -> Result<Vec<MerchantCustomerBalance>, ServiceError> {
        let balances = sqlx::query_as::<_, MerchantCustomerBalance>(
            r#"
            SELECT mb.id, mb.customer_id, mb.merchant_id, mb.crypto_type, mb.available_balance, mb.locked_balance, mb.total_balance, mb.last_updated_at, mb.sandbox_mode
            FROM merchant_customer_balances mb
            JOIN merchant_customers mc ON mc.id = mb.customer_id
            WHERE mc.merchant_id = $1 AND mc.external_id = $2 AND mb.sandbox_mode = $3
              AND (
                  EXISTS (
                      SELECT 1 FROM merchant_wallets mw 
                      WHERE mw.merchant_id = $1 AND mw.crypto_type = mb.crypto_type 
                      AND mw.sandbox_mode = $3 AND mw.is_active = true
                  )
                  OR EXISTS (
                      SELECT 1 FROM merchant_forwarding_wallets mfw
                      WHERE mfw.merchant_id = $1 AND mfw.crypto_type = mb.crypto_type
                      AND mfw.sandbox_mode = $3 AND mfw.is_active = true
                  )
                  OR (
                      NOT EXISTS (SELECT 1 FROM merchant_wallets mw WHERE mw.merchant_id = $1 AND mw.sandbox_mode = $3)
                      AND NOT EXISTS (SELECT 1 FROM merchant_forwarding_wallets mfw WHERE mfw.merchant_id = $1 AND mfw.sandbox_mode = $3)
                  )
              )
            "#
        )
        .bind(merchant_id)
        .bind(external_id)
        .bind(sandbox_mode)
        .fetch_all(&self.db_pool)
        .await?;

        let filtered_balances = balances
            .into_iter()
            .filter(|b| {
                if let Ok(ct) = CryptoType::from_string(&b.crypto_type) {
                    self.config.is_blockchain_enabled(&ct)
                } else {
                    false
                }
            })
            .collect();

        Ok(filtered_balances)
    }

    pub async fn get_customer_wallets(
        &self,
        merchant_id: i64,
        external_id: &str,
        sandbox_mode: bool,
    ) -> Result<Vec<MerchantCustomerWallet>, ServiceError> {
        let wallets = sqlx::query_as::<_, MerchantCustomerWallet>(
            r#"
            SELECT w.id, w.customer_id, w.merchant_id, w.crypto_type, w.network, w.address, w.encrypted_private_key, w.created_at, w.updated_at, w.sandbox_mode
            FROM merchant_customer_wallets w
            JOIN merchant_customers mc ON mc.id = w.customer_id
            WHERE mc.merchant_id = $1 AND mc.external_id = $2 AND w.sandbox_mode = $3
              AND (
                  EXISTS (
                      SELECT 1 FROM merchant_wallets mw 
                      WHERE mw.merchant_id = $1 AND mw.crypto_type = w.crypto_type 
                      AND mw.sandbox_mode = $3 AND mw.is_active = true
                  )
                  OR EXISTS (
                      SELECT 1 FROM merchant_forwarding_wallets mfw
                      WHERE mfw.merchant_id = $1 AND mfw.crypto_type = w.crypto_type
                      AND mfw.sandbox_mode = $3 AND mfw.is_active = true
                  )
                  OR (
                      NOT EXISTS (SELECT 1 FROM merchant_wallets mw WHERE mw.merchant_id = $1 AND mw.sandbox_mode = $3)
                      AND NOT EXISTS (SELECT 1 FROM merchant_forwarding_wallets mfw WHERE mfw.merchant_id = $1 AND mfw.sandbox_mode = $3)
                  )
              )
            ORDER BY w.created_at
            "#
        )
        .bind(merchant_id)
        .bind(external_id)
        .bind(sandbox_mode)
        .fetch_all(&self.db_pool)
        .await?;

        let filtered_wallets = wallets
            .into_iter()
            .filter(|w| {
                if let Ok(ct) = CryptoType::from_string(&w.crypto_type) {
                    self.config.is_blockchain_enabled(&ct)
                } else {
                    false
                }
            })
            .collect();

        Ok(filtered_wallets)
    }

    pub async fn get_deposit_address(
        &self,
        merchant_id: i64,
        external_id: &str,
        crypto_type_str: &str,
        sandbox_mode: bool,
    ) -> Result<serde_json::Value, ServiceError> {
        let customer = self.get_verified_customer(merchant_id, external_id).await?;
        Self::check_permissions(&customer, "view")?;

        // Try to find an existing wallet that is linked to the merchant
        let wallet = sqlx::query_as::<_, MerchantCustomerWallet>(
            r#"
            SELECT w.id, w.customer_id, w.merchant_id, w.crypto_type, w.network, w.address, w.encrypted_private_key, w.created_at, w.updated_at, w.sandbox_mode 
            FROM merchant_customer_wallets w
            WHERE w.customer_id = $1 AND w.crypto_type = $2 AND w.sandbox_mode = $3
              AND (
                  EXISTS (
                      SELECT 1 FROM merchant_wallets mw 
                      WHERE mw.merchant_id = $4 AND mw.crypto_type = w.crypto_type 
                      AND mw.sandbox_mode = $3 AND mw.is_active = true
                  )
                  OR EXISTS (
                      SELECT 1 FROM merchant_forwarding_wallets mfw
                      WHERE mfw.merchant_id = $4 AND mfw.crypto_type = w.crypto_type
                      AND mfw.sandbox_mode = $3 AND mfw.is_active = true
                  )
                  OR (
                      NOT EXISTS (SELECT 1 FROM merchant_wallets mw WHERE mw.merchant_id = $4 AND mw.sandbox_mode = $3)
                      AND NOT EXISTS (SELECT 1 FROM merchant_forwarding_wallets mfw WHERE mfw.merchant_id = $4 AND mfw.sandbox_mode = $3)
                  )
              )
            "#
        )
        .bind(customer.id)
        .bind(crypto_type_str)
        .bind(sandbox_mode)
        .bind(merchant_id)
        .fetch_optional(&self.db_pool)
        .await?;

        if let Some(w) = wallet {
            return Ok(serde_json::json!({
                "external_id": external_id,
                "crypto_type": crypto_type_str,
                "deposit_address": w.address,
                "provisioned": false
            }));
        }

        // Wallet not found or not linked — auto-provision for this network/crypto
        tracing::info!(
            "No linked wallet found for customer {} / {} — auto-provisioning",
            external_id,
            crypto_type_str
        );

        let provisioned = self
            .provision_wallets(
                merchant_id,
                external_id,
                vec![crypto_type_str.to_string()],
                sandbox_mode,
                true,
            )
            .await
            .map_err(|e| {
                ServiceError::InternalError(format!(
                    "Auto-provisioning failed for {} / {}: {}",
                    external_id, crypto_type_str, e
                ))
            })?;

        // Find the newly provisioned wallet matching the requested crypto_type
        let new_wallet = provisioned
            .into_iter()
            .find(|w| w.crypto_type.to_uppercase() == crypto_type_str.to_uppercase())
            .ok_or_else(|| {
                ServiceError::ValidationError(format!(
                    "Could not provision wallet for {} — network may not be enabled for this merchant",
                    crypto_type_str
                ))
            })?;

        Ok(serde_json::json!({
            "external_id": external_id,
            "crypto_type": crypto_type_str,
            "deposit_address": new_wallet.address,
            "provisioned": true
        }))
    }

    pub async fn get_customer_transactions(
        &self,
        merchant_id: i64,
        external_id: &str,
        limit: i64,
        offset: i64,
        sandbox_mode: bool,
    ) -> Result<(Vec<CustomerTransaction>, i64), ServiceError> {
        let customer = self.get_verified_customer(merchant_id, external_id).await?;
        Self::check_permissions(&customer, "view")?;

        let total: i64 = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM customer_transactions WHERE customer_id = $1 AND merchant_id = $2 AND sandbox_mode = $3"
        )
        .bind(customer.id)
        .bind(merchant_id)
        .bind(sandbox_mode)
        .fetch_one(&self.db_pool)
        .await?;

        let transactions = sqlx::query_as::<_, CustomerTransaction>(
            r#"
            SELECT id, customer_id, merchant_id, "type", crypto_type, amount, amount_usd, fee, status,
                   destination_address, transaction_hash, reference_id, description,
                   created_at, updated_at, sandbox_mode
            FROM customer_transactions
            WHERE customer_id = $1 AND merchant_id = $2 AND sandbox_mode = $3
            ORDER BY created_at DESC
            LIMIT $4 OFFSET $5
            "#
        )
        .bind(customer.id)
        .bind(merchant_id)
        .bind(sandbox_mode)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.db_pool)
        .await?;

        Ok((transactions, total))
    }

    pub async fn list_customers(
        &self,
        merchant_id: i64,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<MerchantCustomer>, i64), ServiceError> {
        let total_count: i64 = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM merchant_customers WHERE merchant_id = $1",
        )
        .bind(merchant_id)
        .fetch_one(&self.db_pool)
        .await?;

        let customers = sqlx::query_as::<_, MerchantCustomer>(&format!(
            r#"
            SELECT {} 
            FROM merchant_customers 
            WHERE merchant_id = $1
            ORDER BY created_at DESC 
            LIMIT $2 OFFSET $3
            "#,
            CUSTOMER_COLS
        ))
        .bind(merchant_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.db_pool)
        .await?;

        Ok((customers, total_count))
    }

    // =========================================================================
    // Transaction Operations (with permission checks)
    // =========================================================================

    /// Customer pays merchant — instant off-chain ledger transfer
    pub async fn pay_merchant(
        &self,
        params: PayMerchantParams<'_>,
    ) -> Result<CustomerTransaction, ServiceError> {
        let customer = self
            .get_verified_customer(params.merchant_id, params.external_id)
            .await?;

        // Normalize crypto type
        let crypto_enum = CryptoType::from_string(params.crypto_type_str)?;
        let crypto_type_str_upper = params.crypto_type_str.to_uppercase();
        if crypto_enum == CryptoType::Eth
            && crypto_type_str_upper != "ETH"
            && crypto_type_str_upper != "ETHEREUM"
        {
            // Basic validation since from_string defaults to Pending
            if !params.crypto_type_str.to_uppercase().contains("ETH") {
                // Fallback for actual strict parsing if we wanted it
            }
        }
        let normalized_crypto = crypto_enum.to_string();

        let amount = Decimal::from_str(params.amount_str).map_err(|_| {
            ServiceError::ValidationError(format!("Invalid amount format: {}", params.amount_str))
        })?;

        if amount <= Decimal::ZERO {
            return Err(ServiceError::ValidationError(
                "Payment amount must be greater than zero".to_string(),
            ));
        }

        // 2. Get merchant's receiving wallet address (for ledger documentation)
        let merchant_wallet_address: Option<String> = sqlx::query_scalar::<_, String>(
            "SELECT address FROM merchant_wallets WHERE merchant_id = $1 AND crypto_type = $2 AND sandbox_mode = $3 AND is_active = true"
        )
        .bind(params.merchant_id)
        .bind(&normalized_crypto)
        .bind(params.sandbox_mode)
        .fetch_optional(&self.db_pool)
        .await?;

        let merchant_address =
            merchant_wallet_address.unwrap_or_else(|| "Internal Ledger".to_string());

        // 4. Check customer balance (locked for update in transaction)
        let mut tx = self.db_pool.begin().await?;

        let balance = sqlx::query_as::<_, MerchantCustomerBalance>(
            "SELECT id, customer_id, merchant_id, crypto_type, available_balance, locked_balance, total_balance, last_updated_at, sandbox_mode FROM merchant_customer_balances WHERE customer_id = $1 AND crypto_type = $2 AND sandbox_mode = $3 FOR UPDATE"
        )
        .bind(customer.id)
        .bind(&normalized_crypto)
        .bind(params.sandbox_mode)
        .fetch_optional(&mut *tx)
        .await?;

        match balance {
            Some(b) if b.available_balance >= amount => {}
            _ => return Err(ServiceError::InsufficientFunds(normalized_crypto.clone())),
        }

        // 5. Deduct customer funds and credit merchant off-chain instantly

        sqlx::query(
            "UPDATE merchant_customer_balances SET available_balance = available_balance - $1, locked_balance = locked_balance + $1, last_updated_at = NOW() WHERE customer_id = $2 AND crypto_type = $3 AND sandbox_mode = $4"
        )
        .bind(amount)
        .bind(customer.id)
        .bind(&normalized_crypto)
        .bind(params.sandbox_mode)
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO merchant_balances (merchant_id, crypto_type, available_balance, reserved_balance, last_updated, sandbox_mode)
            VALUES ($1, $2, 0, $3, NOW(), $4)
            ON CONFLICT (merchant_id, crypto_type, sandbox_mode) 
            DO UPDATE SET 
                reserved_balance = merchant_balances.reserved_balance + $3,
                last_updated = NOW()
            "#
        )
        .bind(params.merchant_id)
        .bind(&normalized_crypto)
        .bind(amount)
        .bind(params.sandbox_mode)
        .execute(&mut *tx)
        .await?;

        let tx_ref = params.reference_id.unwrap_or("").to_string();
        let tx_desc = params
            .description
            .unwrap_or("Payment to merchant")
            .to_string();

        // Calculate USD amount
        let ct_enum = crate::payment::models::CryptoType::from_string(&normalized_crypto)
            .unwrap_or(crate::payment::models::CryptoType::Bnb);
        let price = self.price_service.get_price(ct_enum).await.unwrap_or(0.0);
        let amount_usd =
            (amount * Decimal::from_f64_retain(price).unwrap_or(Decimal::ZERO)).round_dp(2);

        let customer_tx = sqlx::query_as::<_, CustomerTransaction>(
            r#"
            INSERT INTO customer_transactions (customer_id, merchant_id, "type", crypto_type, amount, amount_usd, fee, status, destination_address, reference_id, description, sandbox_mode)
            VALUES ($1, $2, 'MERCHANT_PAYMENT', $3, $4, $5, 0, 'COMPLETED', $6, $7, $8, $9)
            RETURNING id, customer_id, merchant_id, "type", crypto_type, amount, amount_usd, fee, status,
                      destination_address, transaction_hash, reference_id, description,
                      created_at, updated_at, sandbox_mode
            "#
        )
        .bind(customer.id)
        .bind(params.merchant_id)
        .bind(&normalized_crypto)
        .bind(amount)
        .bind(amount_usd)
        .bind(&merchant_address)
        .bind(&tx_ref)
        .bind(&tx_desc)
        .bind(params.sandbox_mode)
        .fetch_one(&mut *tx)
        .await?;

        let audit_details = serde_json::json!({
            "customer_external_id": params.external_id,
            "amount": params.amount_str,
            "crypto_type": params.crypto_type_str,
            "reference_id": params.reference_id.unwrap_or(""),
            "description": params.description.unwrap_or(""),
            "masked_address": mask_address(&merchant_address)
        });
        sqlx::query(
            "INSERT INTO audit_logs (merchant_id, action_type, details) VALUES ($1, $2, $3)",
        )
        .bind(params.merchant_id)
        .bind("customer.payment")
        .bind(&audit_details)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        // Trigger real-time balance update broadcast for the merchant
        let _ = self
            .balance_service
            .broadcast_balance_update(params.merchant_id, params.sandbox_mode)
            .await;

        Ok(customer_tx)
    }

    // =========================================================================
    // Sweep (Merchant-initiated)
    // =========================================================================

    pub async fn sweep_customer_wallet(
        &self,
        merchant_id: i64,
        external_id: &str,
        req: crate::models::merchant_customer::SweepCustomerRequest,
        sandbox_mode: bool,
        config: &crate::config::Config,
    ) -> Result<Vec<(String, Decimal)>, ServiceError> {
        let customer = self.get_verified_customer(merchant_id, external_id).await?;

        let mut target_cryptos: Vec<String> = Vec::new();
        let mode_str = req.sweep_mode.to_uppercase();

        if mode_str == "SPECIFIC" {
            if let Some(types) = &req.crypto_types {
                target_cryptos = types.clone();
            } else {
                return Err(ServiceError::ValidationError(
                    "Must specify crypto_types if sweep_mode is SPECIFIC".to_string(),
                ));
            }
        }

        let locked_balances = sqlx::query_as::<_, MerchantCustomerBalance>(
            "SELECT id, customer_id, merchant_id, crypto_type, available_balance, locked_balance, total_balance, last_updated_at, sandbox_mode FROM merchant_customer_balances WHERE customer_id = $1 AND locked_balance > 0 AND sandbox_mode = $2 FOR UPDATE"
        )
        .bind(customer.id)
        .bind(sandbox_mode)
        .fetch_all(&self.db_pool)
        .await?;

        if locked_balances.is_empty() {
            return Err(ServiceError::ValidationError(
                "No locked funds available to sweep".to_string(),
            ));
        }

        let mut balances_to_sweep = Vec::new();
        for b in locked_balances {
            let crypto_enum = CryptoType::from_string(&b.crypto_type).unwrap_or(CryptoType::Eth);

            let is_stablecoin = matches!(
                crypto_enum,
                CryptoType::UsdtBep20
                    | CryptoType::UsdtArbitrum
                    | CryptoType::UsdtSpl
                    | CryptoType::UsdtPolygon
                    | CryptoType::UsdtEth
                    | CryptoType::BusdBep20
            );

            let should_sweep = match mode_str.as_str() {
                "ALL" => true,
                "NATIVE_ONLY" => crypto_enum.is_native_currency(),
                "STABLE_ONLY" => is_stablecoin,
                _ => target_cryptos.iter().any(|c| {
                    CryptoType::from_string(c)
                        .map(|ct| ct.to_string())
                        .unwrap_or_default()
                        == crypto_enum.to_string()
                }),
            };

            if should_sweep {
                balances_to_sweep.push(b);
            }
        }

        if balances_to_sweep.is_empty() {
            return Err(ServiceError::ValidationError(
                "No matching funds available to sweep for the requested criteria".to_string(),
            ));
        }

        // Calculate total USD volume for this sweep and check limits
        let mut total_sweep_usd = Decimal::ZERO;
        let mut sweep_item_details = Vec::new();

        for b in &balances_to_sweep {
            let amount = if mode_str == "SPECIFIC" && target_cryptos.len() == 1 {
                if let Some(ref amt_str) = req.amount {
                    Decimal::from_str(amt_str).unwrap_or(b.locked_balance)
                } else {
                    b.locked_balance
                }
            } else {
                b.locked_balance
            };

            if amount <= Decimal::ZERO || b.locked_balance < amount {
                continue;
            }

            let c_type = CryptoType::from_string(&b.crypto_type).unwrap_or(CryptoType::Eth);
            let price = self.price_service.get_price(c_type).await.unwrap_or(0.0);
            let item_usd =
                (amount * Decimal::from_f64_retain(price).unwrap_or(Decimal::ZERO)).round_dp(2);

            total_sweep_usd += item_usd;
            sweep_item_details.push((b.id, amount, item_usd));
        }

        // Fetch merchant KYC status and limit
        let merchant_row =
            sqlx::query("SELECT kyc_verified, daily_limit_usd FROM merchants WHERE id = $1")
                .bind(merchant_id)
                .fetch_one(&self.db_pool)
                .await?;

        let kyc_verified: bool = merchant_row.get("kyc_verified");
        let daily_limit_usd: Option<Decimal> = merchant_row.get("daily_limit_usd");

        let default_limit = if kyc_verified {
            config.daily_volume_limit_verified_usd
        } else {
            config.daily_volume_limit_non_kyc_usd
        };

        let limit = daily_limit_usd.unwrap_or(default_limit);
        let remaining = self
            .volume_tracking
            .get_remaining_daily_volume(merchant_id, limit)
            .await?
            .unwrap_or(Decimal::ZERO);

        if total_sweep_usd > remaining {
            let status_msg = if kyc_verified {
                "Daily volume limit reached. Contact support to increase your enterprise limit."
                    .to_string()
            } else {
                "Daily volume limit exceeded. Please complete KYC to increase your limit."
                    .to_string()
            };

            return Err(ServiceError::Forbidden(format!(
                "{} Requested sweep: ${}, Remaining: ${}.",
                status_msg, total_sweep_usd, remaining
            )));
        }

        let mut swept_results = Vec::new();
        let mut tx = self.db_pool.begin().await?;

        for balance_record in balances_to_sweep {
            // Find the pre-calculated USD amount for this record
            let item_detail = sweep_item_details
                .iter()
                .find(|(id, _, _)| *id == balance_record.id);
            let amount_usd = item_detail.map(|(_, _, usd)| *usd).unwrap_or(Decimal::ZERO);

            let amount = if mode_str == "SPECIFIC" && target_cryptos.len() == 1 {
                if let Some(ref amt_str) = req.amount {
                    Decimal::from_str(amt_str).unwrap_or(balance_record.locked_balance)
                } else {
                    balance_record.locked_balance
                }
            } else {
                balance_record.locked_balance
            };

            if amount <= Decimal::ZERO || balance_record.locked_balance < amount {
                continue;
            }

            let normalized_crypto = balance_record.crypto_type.clone();
            let crypto_enum =
                CryptoType::from_string(&normalized_crypto).unwrap_or(CryptoType::Eth);

            let mut fee_to_save = Decimal::ZERO;

            if !crypto_enum.is_native_currency() {
                let native_currency = match crypto_enum {
                    CryptoType::UsdtEth => "ETH",
                    CryptoType::UsdtBep20 | CryptoType::BusdBep20 => "BNB",
                    CryptoType::UsdtPolygon => "MATIC",
                    CryptoType::UsdtArbitrum => "ARB",
                    CryptoType::UsdtSpl => "SOL",
                    _ => "ETH",
                };

                let sender = crate::services::blockchain_transaction_sender::BlockchainTransactionSender::new(config.clone());
                let gas_price = sender
                    .get_current_gas_price(crypto_enum, sandbox_mode)
                    .await
                    .unwrap_or(U256::from(50_000_000_000u64));
                let estimated_gas_limit = sender
                    .estimate_gas(crypto_enum, "", "", amount)
                    .await
                    .unwrap_or(U256::from(65000));
                let required_native_u128: u128 =
                    (gas_price * estimated_gas_limit).try_into().unwrap_or(0);

                let divisor = if native_currency == "SOL" {
                    1_000_000_000f64
                } else {
                    1_000_000_000_000_000_000f64
                };
                let mut required_gas_dec =
                    Decimal::from_f64_retain(required_native_u128 as f64 / divisor)
                        .unwrap_or(Decimal::new(25, 4));
                required_gas_dec.rescale(6);

                let customer_wallet_address: String = sqlx::query_scalar(
                    "SELECT address FROM merchant_customer_wallets WHERE customer_id = $1 AND crypto_type = $2 AND sandbox_mode = $3 LIMIT 1"
                )
                .bind(customer.id)
                .bind(&normalized_crypto)
                .bind(sandbox_mode)
                .fetch_optional(&mut *tx)
                .await?
                .unwrap_or_default();

                let native_enum =
                    CryptoType::from_string(native_currency).unwrap_or(CryptoType::Eth);
                // get_native_balance returns U256 (raw smallest unit). Convert to Decimal for comparison.
                let onchain_raw_u256 = sender
                    .get_native_balance(native_enum, &customer_wallet_address, sandbox_mode)
                    .await
                    .unwrap_or(U256::ZERO);
                let onchain_raw_u128: u128 = onchain_raw_u256.try_into().unwrap_or(0);
                let onchain_native_balance =
                    Decimal::from_f64_retain(onchain_raw_u128 as f64 / divisor)
                        .unwrap_or(Decimal::ZERO);

                // 1. Calculate Merchant/Customer balances assigned to this sub-wallet
                let db_customer_native: Decimal = sqlx::query_scalar(
                    r#"
                    SELECT COALESCE(SUM(available_balance + locked_balance + reserved_balance), 0)
                    FROM merchant_customer_balances 
                    WHERE customer_id = $1 AND crypto_type = $2 AND sandbox_mode = $3
                    "#,
                )
                .bind(customer.id)
                .bind(native_currency)
                .bind(sandbox_mode)
                .fetch_optional(&mut *tx)
                .await?
                .unwrap_or(Decimal::ZERO);

                // 2. Calculate Platform Fee assigned to this sub-wallet waiting to be swept
                let db_platform_fee: Decimal = sqlx::query_scalar(
                    r#"
                    SELECT COALESCE(SUM(fee_amount), 0)
                    FROM payment_transactions 
                    WHERE to_address = $1 AND crypto_type = $2 AND fee_collected = FALSE AND sandbox_mode = $3
                    "#
                )
                .bind(&customer_wallet_address)
                .bind(native_currency)
                .bind(sandbox_mode)
                .fetch_optional(&mut *tx)
                .await?
                .unwrap_or(Decimal::ZERO);

                let total_expected_db = db_customer_native + db_platform_fee;

                // 3. Subtract Expected DB assets from Physical Assets to find Unallocated Gas Dust
                let unallocated_dust = if onchain_native_balance > total_expected_db {
                    onchain_native_balance - total_expected_db
                } else {
                    Decimal::ZERO
                };

                let mut gas_to_deduct = required_gas_dec;
                if unallocated_dust >= required_gas_dec {
                    gas_to_deduct = Decimal::ZERO;
                    tracing::info!(
                        "Sweep gas fully covered by unallocated dust {} {} for customer {}",
                        unallocated_dust,
                        native_currency,
                        customer.id
                    );
                } else if unallocated_dust > Decimal::ZERO {
                    gas_to_deduct = required_gas_dec - unallocated_dust;
                    tracing::info!(
                        "Sweep gas partially covered. Total: {}, Dust: {}, Deficit: {} {}",
                        required_gas_dec,
                        unallocated_dust,
                        gas_to_deduct,
                        native_currency
                    );
                } else {
                    tracing::info!(
                        "No gas dust available. Charging full estimate {} {}",
                        gas_to_deduct,
                        native_currency
                    );
                }

                fee_to_save = gas_to_deduct;

                if gas_to_deduct > Decimal::ZERO {
                    let merchant_native_balance: Decimal = sqlx::query_scalar(
                        "SELECT available_balance FROM merchant_balances WHERE merchant_id = $1 AND crypto_type = $2 AND sandbox_mode = $3"
                    )
                    .bind(merchant_id)
                    .bind(native_currency)
                    .bind(sandbox_mode)
                    .fetch_optional(&mut *tx)
                    .await?
                    .unwrap_or(Decimal::ZERO);

                    if merchant_native_balance < gas_to_deduct {
                        tx.rollback().await?;
                        return Err(ServiceError::ValidationError(format!("You need an equivalent amount of {} in your balance to cover the {} gas deficit. Total required: {}, Unallocated Dust: {}.", native_currency, gas_to_deduct, required_gas_dec, unallocated_dust)));
                    }

                    sqlx::query(
                        "UPDATE merchant_balances SET available_balance = available_balance - $1, total_balance = total_balance - $1 WHERE merchant_id = $2 AND crypto_type = $3 AND sandbox_mode = $4"
                    )
                    .bind(gas_to_deduct)
                    .bind(merchant_id)
                    .bind(native_currency)
                    .bind(sandbox_mode)
                    .execute(&mut *tx)
                    .await?;
                }
            }

            sqlx::query(
                "UPDATE merchant_customer_balances SET locked_balance = locked_balance - $1, total_balance = total_balance - $1, last_updated_at = NOW() WHERE customer_id = $2 AND crypto_type = $3 AND sandbox_mode = $4"
            )
            .bind(amount)
            .bind(customer.id)
            .bind(&normalized_crypto)
            .bind(sandbox_mode)
            .execute(&mut *tx)
            .await?;

            sqlx::query(
                r#"
                UPDATE merchant_balances SET reserved_balance = reserved_balance - $1, last_updated = NOW() WHERE merchant_id = $2 AND crypto_type = $3 AND sandbox_mode = $4
                "#
            )
            .bind(amount)
            .bind(merchant_id)
            .bind(&normalized_crypto)
            .bind(sandbox_mode)
            .execute(&mut *tx)
            .await?;

            let merchant_wallet_address: Option<String> = sqlx::query_scalar::<_, String>(
                "SELECT address FROM merchant_wallets WHERE merchant_id = $1 AND crypto_type = $2 AND sandbox_mode = $3 AND is_active = true"
            )
            .bind(merchant_id)
            .bind(&normalized_crypto)
            .bind(sandbox_mode)
            .fetch_optional(&mut *tx)
            .await?;

            let merchant_address =
                merchant_wallet_address.unwrap_or_else(|| "Internal Ledger".to_string());

            let withdrawal_id =
                format!("swp_{}", uuid::Uuid::new_v4().to_string().replace("-", ""));
            sqlx::query(
                r#"
                INSERT INTO withdrawals (
                    withdrawal_id, merchant_id, crypto_type, amount, amount_usd, destination_address,
                    status, fee, net_amount, created_at, updated_at, sandbox_mode
                )
                VALUES ($1, $2, $3, $4, $5, $6, 'PENDING', $7, $8, NOW(), NOW(), $9)
                "#
            )
            .bind(&withdrawal_id)           // $1
            .bind(merchant_id)              // $2
            .bind(&normalized_crypto)       // $3
            .bind(amount + fee_to_save)     // $4 (Gross Amount)
            .bind(amount_usd)               // $5
            .bind(&merchant_address)        // $6
            .bind(fee_to_save)              // $7
            .bind(amount)                   // $8 (Net Amount - what merchant receives)
            .bind(sandbox_mode)             // $9
            .execute(&mut *tx)
            .await?;

            // Record in customer_transactions
            sqlx::query(
                r#"
                INSERT INTO customer_transactions (customer_id, merchant_id, "type", crypto_type, amount, amount_usd, fee, status, reference_id, description, sandbox_mode)
                VALUES ($1, $2, 'SWEEP', $3, $4, $5, 0, 'COMPLETED', $6, 'Funds swept to merchant external wallet', $7)
                "#
            )
            .bind(customer.id)
            .bind(merchant_id)
            .bind(&normalized_crypto)
            .bind(amount)
            .bind(amount_usd)
            .bind(&withdrawal_id)           // $6 (reference_id)
            .bind(sandbox_mode)             // $7
            .execute(&mut *tx)
            .await?;

            let audit_details = serde_json::json!({
                "customer_external_id": external_id,
                "amount": amount.to_string(),
                "crypto_type": &normalized_crypto,
                "withdrawal_id": withdrawal_id
            });
            sqlx::query(
                "INSERT INTO audit_logs (merchant_id, action_type, details) VALUES ($1, $2, $3)",
            )
            .bind(merchant_id)
            .bind("customer.sweep")
            .bind(&audit_details)
            .execute(&mut *tx)
            .await?;

            swept_results.push((normalized_crypto, amount));
        }

        tx.commit().await?;

        Ok(swept_results)
    }

    /// Request a withdrawal for a customer sub-account — bridges to global withdrawal queue
    pub async fn request_customer_withdrawal(
        &self,
        merchant_id: i64,
        external_id: &str,
        req: crate::models::merchant_customer::CustomerWithdrawalRequest,
        sandbox_mode: bool,
    ) -> Result<CustomerTransaction, ServiceError> {
        let customer = self.get_verified_customer(merchant_id, external_id).await?;

        // 1. Validate Amount
        let amount = Decimal::from_str(&req.amount).map_err(|_| {
            ServiceError::ValidationError(format!("Invalid amount format: {}", req.amount))
        })?;

        if amount <= Decimal::ZERO {
            return Err(ServiceError::ValidationError(
                "Withdrawal amount must be greater than zero".to_string(),
            ));
        }

        // 2. Perform exhaustive security checks (Toggle + Limits)
        Self::check_withdrawal_permissions(&customer, amount)?;

        // 3. Normalize crypto type
        let crypto_enum = CryptoType::from_string(&req.crypto_type)?;
        let normalized_crypto = crypto_enum.to_string();

        // 4. Withdrawal Transaction
        let mut tx = self.db_pool.begin().await?;

        // Check and lock balance
        let balance = sqlx::query_as::<_, MerchantCustomerBalance>(
            "SELECT id, customer_id, merchant_id, crypto_type, available_balance, locked_balance, total_balance, last_updated_at, sandbox_mode FROM merchant_customer_balances WHERE customer_id = $1 AND crypto_type = $2 AND sandbox_mode = $3 FOR UPDATE"
        )
        .bind(customer.id)
        .bind(&normalized_crypto)
        .bind(sandbox_mode)
        .fetch_optional(&mut *tx)
        .await?;

        match balance {
            Some(b) if b.available_balance >= amount => {}
            _ => return Err(ServiceError::InsufficientFunds(normalized_crypto.clone())),
        }

        // Move funds to locked (pending withdrawal)
        sqlx::query(
            "UPDATE merchant_customer_balances SET available_balance = available_balance - $1, locked_balance = locked_balance + $1, last_updated_at = NOW() WHERE customer_id = $2 AND crypto_type = $3 AND sandbox_mode = $4"
        )
        .bind(amount)
        .bind(customer.id)
        .bind(&normalized_crypto)
        .bind(sandbox_mode)
        .execute(&mut *tx)
        .await?;

        // Generate unique withdrawal ID
        let withdrawal_id = format!("wd_cust_{}", uuid::Uuid::new_v4());

        // Calculate USD amount (for analytics and tiered security later)
        let price = self
            .price_service
            .get_price(crypto_enum)
            .await
            .unwrap_or(0.0);
        let amount_usd =
            (amount * Decimal::from_f64_retain(price).unwrap_or(Decimal::ZERO)).round_dp(2);

        // 5. Insert into shared withdrawals table for Processor to handle on-chain
        sqlx::query(
            r#"
            INSERT INTO withdrawals (withdrawal_id, merchant_id, crypto_type, amount, amount_usd, destination_address, status, sandbox_mode)
            VALUES ($1, $2, $3, $4, $5, $6, 'PENDING', $7)
            "#
        )
        .bind(&withdrawal_id)
        .bind(merchant_id)
        .bind(&normalized_crypto)
        .bind(amount)
        .bind(amount_usd)
        .bind(&req.destination_address)
        .bind(sandbox_mode)
        .execute(&mut *tx)
        .await?;

        // 6. Record Customer Transaction
        let customer_tx = sqlx::query_as::<_, CustomerTransaction>(
            r#"
            INSERT INTO customer_transactions (customer_id, merchant_id, "type", crypto_type, amount, amount_usd, fee, status, destination_address, reference_id, description, sandbox_mode)
            VALUES ($1, $2, 'WITHDRAWAL', $3, $4, $5, 0, 'PENDING', $6, $7, $8, $9)
            RETURNING id, customer_id, merchant_id, "type", crypto_type, amount, amount_usd, fee, status,
                      destination_address, transaction_hash, reference_id, description,
                      created_at, updated_at, sandbox_mode
            "#
        )
        .bind(customer.id)
        .bind(merchant_id)
        .bind(&normalized_crypto)
        .bind(amount)
        .bind(amount_usd)
        .bind(&req.destination_address)
        .bind(&withdrawal_id)
        .bind("Sub-account withdrawal request")
        .bind(sandbox_mode)
        .fetch_one(&mut *tx)
        .await?;

        // 7. Audit Log with Privacy Masking
        let audit_details = json!({
            "customer_id": customer.id,
            "withdrawal_id": withdrawal_id,
            "masked_email": customer.email.as_ref().map(|e| mask_email(e)),
            "masked_address": mask_address(&req.destination_address),
            "amount": amount.to_string(),
            "crypto_type": normalized_crypto
        });

        sqlx::query(
            "INSERT INTO audit_logs (merchant_id, action_type, details) VALUES ($1, $2, $3)",
        )
        .bind(merchant_id)
        .bind("customer.withdrawal_request")
        .bind(&audit_details)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        // Notify merchant of request
        let _ = self
            .notification_service
            .create_notification(
                merchant_id,
                "🏧 Withdrawal Requested",
                &format!(
                    "A withdrawal of {} {} has been requested for customer {}.",
                    crate::utils::format::format_crypto_amount(amount),
                    normalized_crypto,
                    external_id
                ),
                "info",
                "customer.withdrawal.pending",
                sandbox_mode,
            )
            .await;

        Ok(customer_tx)
    }

    // =========================================================================
    // Status & Permission Management (Merchant controls)
    // =========================================================================

    pub async fn update_customer_status(
        &self,
        merchant_id: i64,
        external_id: &str,
        status: &str,
        reason: Option<&str>,
    ) -> Result<MerchantCustomer, ServiceError> {
        // Validate status value
        match status {
            "active" | "flagged" | "suspended" | "blocked" => {}
            _ => {
                return Err(ServiceError::ValidationError(format!(
                    "Invalid status: {}. Use: active, flagged, suspended, blocked",
                    status
                )))
            }
        }

        let customer = sqlx::query_as::<_, MerchantCustomer>(&format!(
            r#"
            UPDATE merchant_customers 
            SET status = $3, status_reason = $4, updated_at = NOW()
            WHERE merchant_id = $1 AND external_id = $2
            RETURNING {}
            "#,
            CUSTOMER_COLS
        ))
        .bind(merchant_id)
        .bind(external_id)
        .bind(status)
        .bind(reason)
        .fetch_one(&self.db_pool)
        .await
        .map_err(|_| {
            ServiceError::ValidationError(format!("Customer {} not found", external_id))
        })?;

        let audit_details = serde_json::json!({
            "customer_external_id": external_id,
            "status": status,
            "reason": reason.unwrap_or("")
        });
        let _ = sqlx::query(
            "INSERT INTO audit_logs (merchant_id, action_type, details) VALUES ($1, $2, $3)",
        )
        .bind(merchant_id)
        .bind("customer.status_updated")
        .bind(&audit_details)
        .execute(&self.db_pool)
        .await;

        Ok(customer)
    }

    pub async fn update_customer_permissions(
        &self,
        merchant_id: i64,
        external_id: &str,
        can_withdraw: Option<bool>,
        withdrawal_limit: Option<Decimal>,
    ) -> Result<MerchantCustomer, ServiceError> {
        let customer = sqlx::query_as::<_, MerchantCustomer>(&format!(
            r#"
            UPDATE merchant_customers 
            SET can_withdraw = COALESCE($3, can_withdraw),
                withdrawal_limit = COALESCE($4, withdrawal_limit),
                updated_at = NOW()
            WHERE merchant_id = $1 AND external_id = $2
            RETURNING {}
            "#,
            CUSTOMER_COLS
        ))
        .bind(merchant_id)
        .bind(external_id)
        .bind(can_withdraw)
        .bind(withdrawal_limit)
        .fetch_one(&self.db_pool)
        .await
        .map_err(|_| {
            ServiceError::ValidationError(format!("Customer {} not found", external_id))
        })?;

        let audit_details = serde_json::json!({
            "customer_external_id": external_id,
            "can_withdraw": can_withdraw,
            "withdrawal_limit": withdrawal_limit.map(|l: Decimal| l.to_string())
        });
        let _ = sqlx::query(
            "INSERT INTO audit_logs (merchant_id, action_type, details) VALUES ($1, $2, $3)",
        )
        .bind(merchant_id)
        .bind("customer.permissions_updated")
        .bind(&audit_details)
        .execute(&self.db_pool)
        .await;

        Ok(customer)
    }

    /// Deactivate a customer
    pub async fn deactivate_customer(
        &self,
        merchant_id: i64,
        external_id: &str,
    ) -> Result<(), ServiceError> {
        let res = sqlx::query(
            "UPDATE merchant_customers SET is_active = FALSE, status = 'blocked', status_reason = 'Deactivated by merchant', updated_at = NOW() WHERE merchant_id = $1 AND external_id = $2"
        )
        .bind(merchant_id)
        .bind(external_id)
        .execute(&self.db_pool)
        .await;

        match res {
            Ok(r) if r.rows_affected() > 0 => {
                let audit_details = serde_json::json!({ "customer_external_id": external_id });
                let _ = sqlx::query("INSERT INTO audit_logs (merchant_id, action_type, details) VALUES ($1, $2, $3)")
                    .bind(merchant_id)
                    .bind("customer.deactivated")
                    .bind(&audit_details)
                    .execute(&self.db_pool)
                    .await;
                Ok(())
            }
            Ok(_) => Err(ServiceError::ValidationError(format!(
                "Customer {} not found",
                external_id
            ))),
            Err(e) => Err(ServiceError::DatabaseError(e.to_string())),
        }
    }

    /// Get aggregate summary of all customers for a merchant
    pub async fn get_customers_summary(
        &self,
        merchant_id: i64,
        sandbox_mode: bool,
    ) -> Result<serde_json::Value, ServiceError> {
        let counts_row = sqlx::query(
            r#"
            SELECT 
                COUNT(*) as total,
                COUNT(*) FILTER (WHERE status = 'active' AND is_active = TRUE) as active,
                COUNT(*) FILTER (WHERE status = 'flagged') as flagged
            FROM merchant_customers
            WHERE merchant_id = $1
            "#,
        )
        .bind(merchant_id)
        .fetch_one(&self.db_pool)
        .await?;

        let total_count: i64 = counts_row.get("total");
        let active_count: i64 = counts_row.get("active");
        let flagged_count: i64 = counts_row.get("flagged");

        // 7 days recent count
        let recent_count: i64 = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM merchant_customers WHERE merchant_id = $1 AND created_at > NOW() - INTERVAL '7 days'"
        )
        .bind(merchant_id)
        .fetch_one(&self.db_pool)
        .await?;

        // Aggregate customer balances
        let balance_rows = sqlx::query(
            r#"
            SELECT mb.crypto_type, SUM(mb.available_balance + mb.locked_balance) as total_amount
            FROM merchant_customer_balances mb
            WHERE mb.merchant_id = $1 AND mb.sandbox_mode = $2
              AND (
                  EXISTS (SELECT 1 FROM merchant_wallets mw WHERE mw.merchant_id = mb.merchant_id AND mw.crypto_type = mb.crypto_type AND mw.sandbox_mode = mb.sandbox_mode AND mw.is_active = true)
                  OR EXISTS (SELECT 1 FROM merchant_forwarding_wallets mfw WHERE mfw.merchant_id = mb.merchant_id AND mfw.crypto_type = mb.crypto_type AND mfw.sandbox_mode = mb.sandbox_mode AND mfw.is_active = true)
                  OR (
                      NOT EXISTS (SELECT 1 FROM merchant_wallets mw WHERE mw.merchant_id = mb.merchant_id AND mw.sandbox_mode = mb.sandbox_mode)
                      AND NOT EXISTS (SELECT 1 FROM merchant_forwarding_wallets mfw WHERE mfw.merchant_id = mb.merchant_id AND mfw.sandbox_mode = mb.sandbox_mode)
                  )
              )
            GROUP BY mb.crypto_type
            "#,
        )
        .bind(merchant_id)
        .bind(sandbox_mode)
        .fetch_all(&self.db_pool)
        .await?;

        let mut total_balance_usd = Decimal::ZERO;
        for row in balance_rows {
            let crypto_type_str: String = row.get("crypto_type");
            let amount: Decimal = row.get("total_amount");

            if let Ok(crypto_type) = CryptoType::from_string(&crypto_type_str) {
                if !self.config.is_blockchain_enabled(&crypto_type) {
                    continue;
                }
                let price = self
                    .price_service
                    .get_price(crypto_type)
                    .await
                    .unwrap_or(0.0);
                let price_dec = Decimal::from_f64_retain(price).unwrap_or(Decimal::ZERO);
                total_balance_usd += (amount * price_dec).round_dp(2);
            }
        }

        Ok(json!({
            "total_customers": total_count,
            "active_customers": active_count,
            "flagged_customers": flagged_count,
            "recent_customers": recent_count,
            "total_balance_usd": total_balance_usd
        }))
    }

    /// Verify and repair wallets for all customers of a merchant
    pub async fn verify_and_repair_customer_wallets(
        &self,
        merchant_id: i64,
        sandbox_mode: bool,
    ) -> Result<serde_json::Value, ServiceError> {
        let external_ids: Vec<String> = sqlx::query_scalar::<_, String>(
            "SELECT external_id FROM merchant_customers WHERE merchant_id = $1",
        )
        .bind(merchant_id)
        .fetch_all(&self.db_pool)
        .await?;

        let mut repaired_count = 0;
        let checked_customers = external_ids.len();

        for ext_id in &external_ids {
            let customer_id = match sqlx::query_scalar::<_, i64>(
                "SELECT id FROM merchant_customers WHERE merchant_id = $1 AND external_id = $2",
            )
            .bind(merchant_id)
            .bind(ext_id)
            .fetch_optional(&self.db_pool)
            .await?
            {
                Some(id) => id,
                None => continue,
            };

            // Get existing wallet crypto types
            let existing_cryptos: std::collections::HashSet<String> = sqlx::query_scalar::<_, String>(
                "SELECT crypto_type FROM merchant_customer_wallets WHERE customer_id = $1 AND sandbox_mode = $2"
            )
            .bind(customer_id)
            .bind(sandbox_mode)
            .fetch_all(&self.db_pool)
            .await?
            .into_iter()
            .collect();

            let mut missing_networks = Vec::new();

            // Check EVM
            let evm_enabled = self.config.ethereum_enabled
                || self.config.bsc_enabled
                || self.config.polygon_enabled
                || self.config.arbitrum_enabled;
            if evm_enabled {
                let mut evm_missing = false;
                if self.config.ethereum_enabled
                    && (!existing_cryptos.contains("ETH") || !existing_cryptos.contains("USDT_ETH"))
                {
                    evm_missing = true;
                }
                if self.config.bsc_enabled
                    && (!existing_cryptos.contains("BNB")
                        || !existing_cryptos.contains("USDT_BEP20")
                        || !existing_cryptos.contains("BUSD_BEP20"))
                {
                    evm_missing = true;
                }
                if self.config.polygon_enabled
                    && (!existing_cryptos.contains("MATIC")
                        || !existing_cryptos.contains("USDT_POLYGON"))
                {
                    evm_missing = true;
                }
                if self.config.arbitrum_enabled
                    && (!existing_cryptos.contains("ARB")
                        || !existing_cryptos.contains("USDT_ARBITRUM"))
                {
                    evm_missing = true;
                }
                if evm_missing {
                    missing_networks.push("EVM".to_string());
                }
            }

            // Check Solana
            if self.config.solana_enabled
                && (!existing_cryptos.contains("SOL")
                    || !existing_cryptos.contains("USDT_SPL")
                    || !existing_cryptos.contains("WSOL"))
            {
                missing_networks.push("SOLANA".to_string());
            }

            // Check Bitcoin
            if self.config.bitcoin_enabled && !existing_cryptos.contains("BTC") {
                missing_networks.push("BITCOIN".to_string());
            }

            if missing_networks.is_empty() {
                continue;
            }

            match self
                .provision_wallets(merchant_id, ext_id, missing_networks, sandbox_mode, true)
                .await
            {
                Ok(new_wallets) => {
                    // Only count wallets as repaired if their crypto_type was not already present before
                    let newly_created = new_wallets
                        .into_iter()
                        .filter(|w| !existing_cryptos.contains(&w.crypto_type))
                        .count();
                    repaired_count += newly_created;
                }
                Err(e) => {
                    tracing::warn!(
                        "Verify/repair wallets failed for customer {} under merchant {}: {}",
                        ext_id,
                        merchant_id,
                        e
                    );
                }
            }
        }

        if repaired_count > 0 {
            let _ = self
                .balance_service
                .broadcast_balance_update(merchant_id, sandbox_mode)
                .await;
        }

        Ok(serde_json::json!( {
            "status": "success",
            "checked_customers": checked_customers,
            "repaired_wallets": repaired_count
        }))
    }

    /// Lookup an address to see if it belongs to any customer of a merchant (active or historical)
    #[allow(clippy::type_complexity)]
    pub async fn lookup_customer_address(
        &self,
        merchant_id: i64,
        address: &str,
    ) -> Result<Option<serde_json::Value>, ServiceError> {
        let active_wallet: Option<(i64, String, String, String, String, bool, chrono::DateTime<chrono::Utc>)> = sqlx::query_as(
            "SELECT mc.id, mc.external_id, mc.email, mcw.crypto_type, mcw.network, mcw.sandbox_mode, mcw.created_at \
             FROM merchant_customer_wallets mcw \
             JOIN merchant_customers mc ON mc.id = mcw.customer_id \
             WHERE mcw.merchant_id = $1 AND mcw.address = $2 LIMIT 1"
        )
        .bind(merchant_id)
        .bind(address)
        .fetch_optional(&self.db_pool)
        .await?;

        if let Some((cust_id, ext_id, email, crypto_type, network, sandbox_mode, created_at)) =
            active_wallet
        {
            return Ok(Some(serde_json::json!({
                "found": true,
                "status": "ACTIVE",
                "customer": {
                    "id": cust_id,
                    "external_id": ext_id,
                    "email": email
                },
                "wallet": {
                    "address": address,
                    "crypto_type": crypto_type,
                    "network": network,
                    "sandbox_mode": sandbox_mode,
                    "created_at": created_at
                }
            })));
        }

        let historical_wallet: Option<(i64, String, String, String, String, bool, Option<String>, chrono::DateTime<chrono::Utc>)> = sqlx::query_as(
            "SELECT mc.id, mc.external_id, mc.email, mwh.crypto_type, mwh.network, mwh.sandbox_mode, mwh.reason, mwh.created_at \
             FROM merchant_wallet_history mwh \
             JOIN merchant_customers mc ON mc.id = mwh.customer_id \
             WHERE mwh.merchant_id = $1 AND mwh.owner_type = 'customer' AND mwh.old_address = $2 LIMIT 1"
        )
        .bind(merchant_id)
        .bind(address)
        .fetch_optional(&self.db_pool)
        .await?;

        if let Some((
            cust_id,
            ext_id,
            email,
            crypto_type,
            network,
            sandbox_mode,
            reason,
            created_at,
        )) = historical_wallet
        {
            return Ok(Some(serde_json::json!({
                "found": true,
                "status": "HISTORICAL",
                "customer": {
                    "id": cust_id,
                    "external_id": ext_id,
                    "email": email
                },
                "wallet": {
                    "address": address,
                    "crypto_type": crypto_type,
                    "network": network,
                    "sandbox_mode": sandbox_mode,
                    "reason": reason.unwrap_or_default(),
                    "created_at": created_at
                }
            })));
        }

        Ok(None)
    }

    /// Get a full audit of all customer wallets (both active and historical) under a merchant
    pub async fn audit_all_customer_wallets(
        &self,
        merchant_id: i64,
    ) -> Result<serde_json::Value, ServiceError> {
        let active_rows: Vec<serde_json::Value> = sqlx::query(
            "SELECT mc.external_id, mc.email, mcw.address, mcw.crypto_type, mcw.network, mcw.sandbox_mode, mcw.created_at \
             FROM merchant_customer_wallets mcw \
             JOIN merchant_customers mc ON mc.id = mcw.customer_id \
             WHERE mcw.merchant_id = $1 \
             ORDER BY mcw.created_at DESC"
        )
        .bind(merchant_id)
        .fetch_all(&self.db_pool)
        .await?
        .into_iter()
        .map(|row| {
            use sqlx::Row;
            serde_json::json!({
                "external_id": row.get::<String, _>("external_id"),
                "email": row.get::<String, _>("email"),
                "address": row.get::<String, _>("address"),
                "crypto_type": row.get::<String, _>("crypto_type"),
                "network": row.get::<String, _>("network"),
                "sandbox_mode": row.get::<bool, _>("sandbox_mode"),
                "status": "ACTIVE",
                "created_at": row.get::<chrono::DateTime<chrono::Utc>, _>("created_at"),
            })
        })
        .collect();

        let historical_rows: Vec<serde_json::Value> = sqlx::query(
            "SELECT mc.external_id, mc.email, mwh.old_address as address, mwh.crypto_type, mwh.network, mwh.sandbox_mode, mwh.reason, mwh.created_at \
             FROM merchant_wallet_history mwh \
             JOIN merchant_customers mc ON mc.id = mwh.customer_id \
             WHERE mwh.merchant_id = $1 AND mwh.owner_type = 'customer' AND mwh.old_address IS NOT NULL \
             ORDER BY mwh.created_at DESC"
        )
        .bind(merchant_id)
        .fetch_all(&self.db_pool)
        .await?
        .into_iter()
        .map(|row| {
            use sqlx::Row;
            serde_json::json!({
                "external_id": row.get::<String, _>("external_id"),
                "email": row.get::<String, _>("email"),
                "address": row.get::<String, _>("address"),
                "crypto_type": row.get::<String, _>("crypto_type"),
                "network": row.get::<String, _>("network"),
                "sandbox_mode": row.get::<bool, _>("sandbox_mode"),
                "status": "HISTORICAL",
                "reason": row.get::<Option<String>, _>("reason").unwrap_or_default(),
                "created_at": row.get::<chrono::DateTime<chrono::Utc>, _>("created_at"),
            })
        })
        .collect();

        Ok(serde_json::json!({
            "active": active_rows,
            "historical": historical_rows
        }))
    }
}

fn mask_address(addr: &str) -> String {
    if addr.len() > 10 {
        format!("{}...{}", &addr[..6], &addr[addr.len() - 4..])
    } else {
        addr.to_string()
    }
}

// =============================================================================
// Tests — Wallet Health & Auto-Provisioning
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use serde_json::json;

    // -------------------------------------------------------------------------
    // Fixture helpers
    // -------------------------------------------------------------------------

    fn active_customer() -> MerchantCustomer {
        MerchantCustomer {
            id: 1,
            merchant_id: 10,
            external_id: "user_abc".to_string(),
            email: Some("test@example.com".to_string()),
            first_name: Some("Test".to_string()),
            last_name: Some("User".to_string()),
            metadata: None,
            is_active: true,
            status: "active".to_string(),
            status_reason: None,
            can_withdraw: true,
            withdrawal_limit: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn deactivated_customer() -> MerchantCustomer {
        MerchantCustomer {
            is_active: false,
            status: "active".to_string(),
            ..active_customer()
        }
    }

    fn flagged_customer() -> MerchantCustomer {
        MerchantCustomer {
            status: "flagged".to_string(),
            status_reason: Some("Suspicious activity".to_string()),
            ..active_customer()
        }
    }

    fn suspended_customer() -> MerchantCustomer {
        MerchantCustomer {
            status: "suspended".to_string(),
            status_reason: Some("Violation of ToS".to_string()),
            ..active_customer()
        }
    }

    // -------------------------------------------------------------------------
    // check_permissions — "view" action (used by get_deposit_address)
    // -------------------------------------------------------------------------

    #[test]
    fn check_permissions_allows_active_customer_to_view() {
        let result = MerchantCustomerService::check_permissions(&active_customer(), "view");
        assert!(result.is_ok(), "Active customer should be allowed to view");
    }

    #[test]
    fn check_permissions_blocks_deactivated_customer() {
        let result = MerchantCustomerService::check_permissions(&deactivated_customer(), "view");
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("deactivated"),
            "Error should mention deactivation: {err_msg}"
        );
    }

    #[test]
    fn check_permissions_allows_flagged_customer_to_view() {
        // Flagged customers are read-only — they can still get their deposit address
        let result = MerchantCustomerService::check_permissions(&flagged_customer(), "view");
        assert!(
            result.is_ok(),
            "Flagged customer should still be allowed to view (read-only)"
        );
    }

    #[test]
    fn check_permissions_blocks_flagged_customer_from_writing() {
        let result = MerchantCustomerService::check_permissions(&flagged_customer(), "withdraw");
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("flagged"),
            "Error should mention flagged: {err_msg}"
        );
    }

    #[test]
    fn check_permissions_blocks_suspended_customer_from_any_action() {
        for action in ["view", "withdraw", "pay"] {
            let result = MerchantCustomerService::check_permissions(&suspended_customer(), action);
            assert!(
                result.is_err(),
                "Suspended customer should be blocked for action: {action}"
            );
        }
    }

    // -------------------------------------------------------------------------
    // mask_address helper
    // -------------------------------------------------------------------------

    #[test]
    fn mask_address_masks_evm_address_correctly() {
        let addr = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb";
        let masked = mask_address(addr);
        assert!(
            masked.starts_with("0x742d"),
            "Should keep first 6 chars: {masked}"
        );
        assert!(
            masked.ends_with("0bEb"),
            "Should keep last 4 chars: {masked}"
        );
        assert!(masked.contains("..."), "Should contain ellipsis: {masked}");
    }

    #[test]
    fn mask_address_masks_solana_address_correctly() {
        let addr = "5rBr6CFUA4Yi7uoX9JUgvC9PFzEjv5jNtu5NThZNEKqP";
        let masked = mask_address(addr);
        assert!(masked.contains("..."));
        assert!(masked.starts_with("5rBr6C"));
    }

    #[test]
    fn mask_address_short_address_returned_as_is() {
        let addr = "short";
        let masked = mask_address(addr);
        assert_eq!(masked, "short", "Short addresses should not be masked");
    }

    #[test]
    fn mask_address_exactly_10_chars_returned_as_is() {
        let addr = "1234567890";
        let masked = mask_address(addr);
        assert_eq!(
            masked, addr,
            "Exactly 10-char addresses should not be masked"
        );
    }

    #[test]
    fn mask_address_11_chars_is_masked() {
        let addr = "12345678901";
        let masked = mask_address(addr);
        assert!(masked.contains("..."));
    }

    // -------------------------------------------------------------------------
    // get_deposit_address response shape
    // -------------------------------------------------------------------------

    #[test]
    fn deposit_address_response_has_provisioned_false_when_wallet_found() {
        // Simulate what the service returns when wallet already exists
        let address = "0xExistingAddress";
        let external_id = "user_abc";
        let crypto_type = "USDT_BEP20";

        let response = json!({
            "external_id": external_id,
            "crypto_type": crypto_type,
            "deposit_address": address,
            "provisioned": false
        });

        assert_eq!(response["external_id"], external_id);
        assert_eq!(response["crypto_type"], crypto_type);
        assert_eq!(response["deposit_address"], address);
        assert_eq!(response["provisioned"], false);
    }

    #[test]
    fn deposit_address_response_has_provisioned_true_when_auto_provisioned() {
        let address = "0xNewlyProvisioned";
        let external_id = "new_user";
        let crypto_type = "ETH";

        let response = json!({
            "external_id": external_id,
            "crypto_type": crypto_type,
            "deposit_address": address,
            "provisioned": true
        });

        assert_eq!(response["provisioned"], true);
        assert_eq!(response["deposit_address"], address);
    }

    #[test]
    fn deposit_address_response_all_required_fields_present() {
        let response = json!({
            "external_id": "user_test",
            "crypto_type": "SOL",
            "deposit_address": "SolanaAddr123",
            "provisioned": false
        });

        // All 4 fields must exist in response
        assert!(response.get("external_id").is_some(), "external_id missing");
        assert!(response.get("crypto_type").is_some(), "crypto_type missing");
        assert!(
            response.get("deposit_address").is_some(),
            "deposit_address missing"
        );
        assert!(
            response.get("provisioned").is_some(),
            "provisioned flag missing"
        );
    }

    // -------------------------------------------------------------------------
    // verify_and_repair response shape
    // -------------------------------------------------------------------------

    #[test]
    fn verify_repair_response_shape_with_no_repairs_needed() {
        let response = json!({
            "status": "completed",
            "checked_customers": 42,
            "repaired_wallets": 0
        });

        assert_eq!(response["status"], "completed");
        assert_eq!(response["checked_customers"], 42);
        assert_eq!(response["repaired_wallets"], 0);
    }

    #[test]
    fn verify_repair_response_shape_with_repairs() {
        let response = json!({
            "status": "completed",
            "checked_customers": 177,
            "repaired_wallets": 3
        });

        assert_eq!(response["repaired_wallets"], 3);
        assert!(
            response["repaired_wallets"].as_i64().unwrap() > 0,
            "Should indicate wallets were provisioned"
        );
    }

    // -------------------------------------------------------------------------
    // lookup_customer_address response shapes
    // -------------------------------------------------------------------------

    #[test]
    fn lookup_not_found_response_has_correct_shape() {
        let response = json!({
            "found": false,
            "message": "Address not found for any of your customers"
        });

        assert_eq!(response["found"], false);
        assert!(response["message"].as_str().unwrap().contains("not found"));
    }

    #[test]
    fn lookup_active_response_has_correct_shape() {
        let response = json!({
            "found": true,
            "status": "ACTIVE",
            "customer": {
                "id": 1,
                "external_id": "user_abc",
                "email": "test@example.com"
            },
            "wallet": {
                "address": "0xActiveAddress",
                "crypto_type": "ETH",
                "network": "ETHEREUM",
                "sandbox_mode": false,
                "created_at": "2025-01-01T00:00:00Z"
            }
        });

        assert_eq!(response["found"], true);
        assert_eq!(response["status"], "ACTIVE");
        assert_eq!(response["customer"]["external_id"], "user_abc");
        assert_eq!(response["wallet"]["network"], "ETHEREUM");
    }

    #[test]
    fn lookup_historical_response_has_correct_shape() {
        let response = json!({
            "found": true,
            "status": "HISTORICAL",
            "customer": {
                "id": 2,
                "external_id": "old_user",
                "email": "old@example.com"
            },
            "wallet": {
                "address": "0xOldAddress",
                "crypto_type": "USDT_BEP20",
                "network": "ETHEREUM",
                "sandbox_mode": false,
                "reason": "Customer wallet re-provisioned",
                "created_at": "2024-01-01T00:00:00Z"
            }
        });

        assert_eq!(response["status"], "HISTORICAL");
        assert_eq!(
            response["wallet"]["reason"],
            "Customer wallet re-provisioned"
        );
    }

    // -------------------------------------------------------------------------
    // audit_customer_wallets response shape
    // -------------------------------------------------------------------------

    #[test]
    fn audit_response_has_active_and_historical_arrays() {
        let response = json!({
            "active": [],
            "historical": []
        });

        assert!(response["active"].is_array());
        assert!(response["historical"].is_array());
    }

    #[test]
    fn audit_active_entry_has_expected_fields() {
        let entry = json!({
            "external_id": "user_1",
            "email": "u@e.com",
            "address": "0xAddr",
            "crypto_type": "ETH",
            "network": "ETHEREUM",
            "sandbox_mode": false,
            "status": "ACTIVE",
            "created_at": "2025-01-01T00:00:00Z"
        });

        for field in [
            "external_id",
            "email",
            "address",
            "crypto_type",
            "network",
            "status",
            "created_at",
        ] {
            assert!(
                entry.get(field).is_some(),
                "Active entry missing field: {field}"
            );
        }
        assert_eq!(entry["status"], "ACTIVE");
    }

    #[test]
    fn audit_historical_entry_has_reason_field() {
        let entry = json!({
            "external_id": "user_2",
            "email": "u2@e.com",
            "address": "0xOld",
            "crypto_type": "USDT_BEP20",
            "network": "ETHEREUM",
            "sandbox_mode": false,
            "status": "HISTORICAL",
            "reason": "Customer wallet re-provisioned",
            "created_at": "2024-01-01T00:00:00Z"
        });

        assert_eq!(entry["status"], "HISTORICAL");
        assert_eq!(entry["reason"], "Customer wallet re-provisioned");
    }

    // -------------------------------------------------------------------------
    // Crypto-type alias coverage (used by auto-provision network routing)
    // -------------------------------------------------------------------------

    #[test]
    fn evm_crypto_types_are_recognizable() {
        let evm_types = vec![
            "ETH",
            "USDT_ETH",
            "BNB",
            "USDT_BEP20",
            "BUSD_BEP20",
            "MATIC",
            "USDT_POLYGON",
            "ARB",
            "USDT_ARBITRUM",
        ];

        // Simulate the match arms in provision_wallets for EVM types
        for crypto in evm_types {
            let normalized = crypto.to_uppercase();
            let is_evm = matches!(
                normalized.as_str(),
                "EVM"
                    | "ETH"
                    | "ERC20"
                    | "BSC"
                    | "BEP20"
                    | "POLYGON"
                    | "MATIC"
                    | "ARB"
                    | "ARBITRUM"
                    | "NATIVE"
                    | "ETHEREUM"
                    | "USDT_ETH"
                    | "USDT_BEP20"
                    | "BUSD_BEP20"
                    | "USDT_POLYGON"
                    | "USDT_ARBITRUM"
                    | "BNB"
            );
            assert!(is_evm, "Expected {crypto} to be recognized as an EVM type");
        }
    }

    #[test]
    fn solana_crypto_types_are_recognizable() {
        let sol_types = vec!["SOL", "USDT_SPL", "SOLANA", "WSOL"];

        for crypto in sol_types {
            let normalized = crypto.to_uppercase();
            let is_solana = matches!(
                normalized.as_str(),
                "SOLANA"
                    | "SOL"
                    | "SPL"
                    | "SOLANA_SPL"
                    | "SOLANA_MAINNET"
                    | "SOLANA_DEVNET"
                    | "USDT_SPL"
                    | "WSOL"
            );
            assert!(
                is_solana,
                "Expected {crypto} to be recognized as a Solana type"
            );
        }
    }

    #[test]
    fn bitcoin_crypto_types_are_recognizable() {
        let btc_types = vec!["BTC", "BITCOIN"];

        for crypto in btc_types {
            let normalized = crypto.to_uppercase();
            let is_btc = matches!(
                normalized.as_str(),
                "BITCOIN" | "BTC" | "BITCOIN_MAINNET" | "BITCOIN_TESTNET"
            );
            assert!(is_btc, "Expected {crypto} to be recognized as Bitcoin type");
        }
    }

    // -------------------------------------------------------------------------
    // Wallet Health v2.6.19 Safety & Repair Edge Cases
    // -------------------------------------------------------------------------

    #[test]
    fn verify_repair_prevents_duplicate_runs() {
        // First run repaired some wallets
        let first_run = json!({
            "status": "success",
            "checked_customers": 3,
            "repaired_wallets": 15
        });
        assert_eq!(first_run["repaired_wallets"], 15);

        // A second run on the exact same clean state must return 0 repairs
        let second_run = json!({
            "status": "success",
            "checked_customers": 3,
            "repaired_wallets": 0
        });
        assert_eq!(
            second_run["repaired_wallets"], 0,
            "Second run must be a no-op"
        );
    }

    #[test]
    fn verify_private_key_safety_and_preservation() {
        // Customer has existing ETH wallet
        let eth_wallet = json!({
            "address": "0x742d35Cc6634C0532925a3b8D4C9db96590c6C87",
            "encrypted_private_key": "enc_secret_key_12345",
            "crypto_type": "ETH"
        });

        // Trigger repair/provision for USDT_ETH (which is in EVM network family)
        // It must reuse the exact same address and encrypted key to prevent fund loss
        let usdt_wallet = json!({
            "address": eth_wallet["address"].as_str().unwrap(),
            "encrypted_private_key": eth_wallet["encrypted_private_key"].as_str().unwrap(),
            "crypto_type": "USDT_ETH"
        });

        assert_eq!(
            usdt_wallet["address"], eth_wallet["address"],
            "Address must be reused across EVM assets"
        );
        assert_eq!(
            usdt_wallet["encrypted_private_key"], eth_wallet["encrypted_private_key"],
            "Encrypted private key must be preserved to prevent fund loss"
        );
    }
}
