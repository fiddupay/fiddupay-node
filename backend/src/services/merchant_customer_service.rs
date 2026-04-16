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
use alloy_primitives::U256;
use rust_decimal::Decimal;
use serde_json::json;
use sqlx::{PgPool, Row};
use std::str::FromStr;

const CUSTOMER_COLS: &str = "id, merchant_id, external_id, email, first_name, last_name, metadata, is_active, status, status_reason, can_withdraw, withdrawal_limit, created_at, updated_at, sandbox_mode";

use crate::services::notification_service::NotificationService;
use crate::services::price_service::PriceService;
use crate::services::volume_tracking_service::VolumeTrackingService;
use std::sync::Arc;

pub struct MerchantCustomerService {
    db_pool: PgPool,
    price_service: Arc<PriceService>,
    volume_tracking: Arc<VolumeTrackingService>,
    notification_service: Arc<NotificationService>,
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
    ) -> Self {
        Self {
            db_pool,
            price_service,
            volume_tracking,
            notification_service,
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
        sandbox_mode: bool,
    ) -> Result<MerchantCustomer, ServiceError> {
        let customer = sqlx::query_as::<_, MerchantCustomer>(
            &format!("SELECT {} FROM merchant_customers WHERE merchant_id = $1 AND external_id = $2 AND sandbox_mode = $3", CUSTOMER_COLS)
        )
        .bind(merchant_id)
        .bind(external_id)
        .bind(sandbox_mode)
        .fetch_one(&self.db_pool)
        .await
        .map_err(|_| ServiceError::ValidationError(format!("Customer {} not found", external_id)))?;

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
            INSERT INTO merchant_customers (merchant_id, external_id, email, first_name, last_name, metadata, is_active, sandbox_mode)
            VALUES ($1, $2, $3, $4, $5, $6, TRUE, $7)
            ON CONFLICT (merchant_id, external_id, sandbox_mode) 
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
        .bind(sandbox_mode)
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
        let customer = self
            .get_verified_customer(merchant_id, external_id, sandbox_mode)
            .await?;

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

                    let keypair = KeyGenerator::generate_evm_wallet()?;
                    let encrypted_key = encryption.encrypt(&keypair.private_key).map_err(|e| {
                        ServiceError::InternalError(format!("Encryption failed: {}", e))
                    })?;

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
                        let wallet = self
                            .save_customer_wallet(SaveWalletParams {
                                customer_id: customer.id,
                                merchant_id,
                                crypto_type: crypto,
                                address: keypair.address.clone(),
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

                    let keypair = KeyGenerator::generate_solana_wallet()?;
                    let encrypted_key = encryption.encrypt(&keypair.private_key).map_err(|e| {
                        ServiceError::InternalError(format!("Encryption failed: {}", e))
                    })?;

                    let sol_cryptos = vec![CryptoType::Sol, CryptoType::UsdtSpl];

                    for crypto in sol_cryptos {
                        let wallet = self
                            .save_customer_wallet(SaveWalletParams {
                                customer_id: customer.id,
                                merchant_id,
                                crypto_type: crypto,
                                address: keypair.address.clone(),
                                encrypted_key: encrypted_key.clone(),
                                sandbox_mode,
                                bypass_lock,
                            })
                            .await?;
                        wallets.push(wallet);
                    }
                }
                "BITCOIN" | "BTC" | "BITCOIN_MAINNET" | "BITCOIN_TESTNET" => {
                    if wallets
                        .iter()
                        .any(|w| w.network.to_uppercase() == "BITCOIN")
                    {
                        continue;
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
                "SELECT external_id FROM merchant_customers WHERE merchant_id = $1 AND sandbox_mode = $2"
            )
            .bind(merchant_id)
            .bind(sandbox_mode)
            .fetch_all(&self.db_pool)
            .await?
        } else if let Some(ids) = customer_ids {
            // Verify that all requested IDs belong to this merchant using a single batch query
            sqlx::query_scalar::<_, String>(
                "SELECT external_id FROM merchant_customers WHERE merchant_id = $1 AND sandbox_mode = $3 AND external_id = ANY($2)"
            )
            .bind(merchant_id)
            .bind(&ids)
            .bind(sandbox_mode)
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
                        encrypted_private_key, reason
                    )
                    VALUES ($1, $2, 'customer', $3, $4, $5, $6, 'managed', $7, $8)
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
            "#
        )
        .bind(merchant_id)
        .bind(external_id)
        .bind(sandbox_mode)
        .fetch_all(&self.db_pool)
        .await?;

        Ok(balances)
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
            ORDER BY w.created_at
            "#
        )
        .bind(merchant_id)
        .bind(external_id)
        .bind(sandbox_mode)
        .fetch_all(&self.db_pool)
        .await?;

        Ok(wallets)
    }

    pub async fn get_deposit_address(
        &self,
        merchant_id: i64,
        external_id: &str,
        crypto_type_str: &str,
        sandbox_mode: bool,
    ) -> Result<String, ServiceError> {
        let customer = self
            .get_verified_customer(merchant_id, external_id, sandbox_mode)
            .await?;
        Self::check_permissions(&customer, "view")?;

        let wallet = sqlx::query_as::<_, MerchantCustomerWallet>(
            "SELECT id, customer_id, merchant_id, crypto_type, network, address, encrypted_private_key, created_at, updated_at, sandbox_mode FROM merchant_customer_wallets WHERE customer_id = $1 AND crypto_type = $2 AND sandbox_mode = $3"
        )
        .bind(customer.id)
        .bind(crypto_type_str)
        .bind(sandbox_mode)
        .fetch_optional(&self.db_pool)
        .await?
        .ok_or_else(|| ServiceError::ValidationError(format!("No wallet found for {}", crypto_type_str)))?;

        Ok(wallet.address)
    }

    pub async fn get_customer_transactions(
        &self,
        merchant_id: i64,
        external_id: &str,
        limit: i64,
        offset: i64,
        sandbox_mode: bool,
    ) -> Result<(Vec<CustomerTransaction>, i64), ServiceError> {
        let customer = self
            .get_verified_customer(merchant_id, external_id, sandbox_mode)
            .await?;
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
        sandbox_mode: bool,
    ) -> Result<(Vec<MerchantCustomer>, i64), ServiceError> {
        let total_count: i64 = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM merchant_customers WHERE merchant_id = $1 AND sandbox_mode = $2",
        )
        .bind(merchant_id)
        .bind(sandbox_mode)
        .fetch_one(&self.db_pool)
        .await?;

        let customers = sqlx::query_as::<_, MerchantCustomer>(&format!(
            r#"
            SELECT {} 
            FROM merchant_customers 
            WHERE merchant_id = $1 AND sandbox_mode = $2
            ORDER BY created_at DESC 
            LIMIT $3 OFFSET $4
            "#,
            CUSTOMER_COLS
        ))
        .bind(merchant_id)
        .bind(sandbox_mode)
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
            .get_verified_customer(params.merchant_id, params.external_id, params.sandbox_mode)
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
        let customer = self
            .get_verified_customer(merchant_id, external_id, sandbox_mode)
            .await?;

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

        if !kyc_verified {
            let limit = daily_limit_usd.unwrap_or(config.daily_volume_limit_non_kyc_usd);
            let remaining = self
                .volume_tracking
                .get_remaining_daily_volume(merchant_id, limit, kyc_verified)
                .await?
                .unwrap_or(Decimal::ZERO);

            if total_sweep_usd > remaining {
                return Err(ServiceError::Forbidden(format!(
                    "Daily volume limit exceeded. This sweep would cost ${}, but you only have ${} remaining today. Please complete KYC to remove this limit.",
                    total_sweep_usd, remaining
                )));
            }
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
        let customer = self
            .get_verified_customer(merchant_id, external_id, sandbox_mode)
            .await?;

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
                    amount, normalized_crypto, external_id
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
        sandbox_mode: bool,
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
            WHERE merchant_id = $1 AND external_id = $2 AND sandbox_mode = $5
            RETURNING {}
            "#,
            CUSTOMER_COLS
        ))
        .bind(merchant_id)
        .bind(external_id)
        .bind(status)
        .bind(reason)
        .bind(sandbox_mode)
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
        sandbox_mode: bool,
    ) -> Result<MerchantCustomer, ServiceError> {
        let customer = sqlx::query_as::<_, MerchantCustomer>(&format!(
            r#"
            UPDATE merchant_customers 
            SET can_withdraw = COALESCE($3, can_withdraw),
                withdrawal_limit = COALESCE($4, withdrawal_limit),
                updated_at = NOW()
            WHERE merchant_id = $1 AND external_id = $2 AND sandbox_mode = $5
            RETURNING {}
            "#,
            CUSTOMER_COLS
        ))
        .bind(merchant_id)
        .bind(external_id)
        .bind(can_withdraw)
        .bind(withdrawal_limit)
        .bind(sandbox_mode)
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
        sandbox_mode: bool,
    ) -> Result<(), ServiceError> {
        let res = sqlx::query(
            "UPDATE merchant_customers SET is_active = FALSE, status = 'blocked', status_reason = 'Deactivated by merchant', updated_at = NOW() WHERE merchant_id = $1 AND external_id = $2 AND sandbox_mode = $3"
        )
        .bind(merchant_id)
        .bind(external_id)
        .bind(sandbox_mode)
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
            WHERE merchant_id = $1 AND sandbox_mode = $2
            "#,
        )
        .bind(merchant_id)
        .bind(sandbox_mode)
        .fetch_one(&self.db_pool)
        .await?;

        let total_count: i64 = counts_row.get("total");
        let active_count: i64 = counts_row.get("active");
        let flagged_count: i64 = counts_row.get("flagged");

        // 7 days recent count
        let recent_count: i64 = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM merchant_customers WHERE merchant_id = $1 AND sandbox_mode = $2 AND created_at > NOW() - INTERVAL '7 days'"
        )
        .bind(merchant_id)
        .bind(sandbox_mode)
        .fetch_one(&self.db_pool)
        .await?;

        // Aggregate customer balances
        let balance_rows = sqlx::query(
            r#"
            SELECT crypto_type, SUM(available_balance + locked_balance) as total_amount
            FROM merchant_customer_balances
            WHERE merchant_id = $1 AND sandbox_mode = $2
            GROUP BY crypto_type
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
}

fn mask_email(email: &str) -> String {
    if let Some(pos) = email.find('@') {
        let (name, domain) = email.split_at(pos);
        match name.len() {
            0 => email.to_string(),
            1 => format!("*{}", domain),
            2 => format!("{}*{}", &name[..1], domain),
            _ => {
                // Show first and last characters of the name part, e.g. j****n@example.com
                format!("{}****{}{}", &name[..1], &name[name.len() - 1..], domain)
            }
        }
    } else {
        email.to_string()
    }
}

fn mask_address(addr: &str) -> String {
    if addr.len() > 10 {
        format!("{}...{}", &addr[..6], &addr[addr.len() - 4..])
    } else {
        addr.to_string()
    }
}
