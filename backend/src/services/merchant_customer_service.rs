// Merchant Customer Service
// Manages sub-accounts, designated user wallets, permissions, and customer transactions

use crate::error::ServiceError;
use crate::models::merchant_customer::{
    MerchantCustomer, MerchantCustomerWallet, CreateCustomerRequest, 
    MerchantCustomerBalance, CustomerTransaction
};
use crate::payment::models::CryptoType;
use crate::utils::keygen::KeyGenerator;
use crate::utils::encryption::Encryption;
use sqlx::{PgPool, Row};
use serde_json::json;
use rust_decimal::Decimal;
use std::str::FromStr;

const CUSTOMER_COLS: &str = "id, merchant_id, external_id, email, first_name, last_name, metadata, is_active, status, status_reason, can_withdraw, withdrawal_limit, created_at, updated_at, sandbox_mode";

pub struct MerchantCustomerService {
    db_pool: PgPool,
}

impl MerchantCustomerService {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }

    // =========================================================================
    // Permission Helpers
    // =========================================================================

    /// Check if a customer can perform the given action. Returns the customer if OK.
    fn check_permissions(customer: &MerchantCustomer, action: &str) -> Result<(), ServiceError> {
        if !customer.is_active {
            return Err(ServiceError::ValidationError("Customer account is deactivated".to_string()));
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
            },
            "suspended" | "blocked" => {
                Err(ServiceError::ValidationError(format!(
                    "Customer account is {}: {}",
                    customer.status,
                    customer.status_reason.as_deref().unwrap_or("Contact support")
                )))
            },
            _ => Ok(()),
        }
    }

    /// Additional check for withdrawal-specific permissions
    fn check_withdrawal_permissions(customer: &MerchantCustomer, amount: Decimal) -> Result<(), ServiceError> {
        Self::check_permissions(customer, "withdraw")?;

        if !customer.can_withdraw {
            return Err(ServiceError::ValidationError("Withdrawals are disabled for this customer".to_string()));
        }

        if let Some(limit) = customer.withdrawal_limit {
            if amount > limit {
                return Err(ServiceError::ValidationError(format!(
                    "Amount {} exceeds withdrawal limit of {}", amount, limit
                )));
            }
        }

        Ok(())
    }

    /// Fetch and validate a customer
    async fn get_verified_customer(&self, merchant_id: i64, external_id: &str, sandbox_mode: bool) -> Result<MerchantCustomer, ServiceError> {
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
        let wallets = self.provision_wallets(merchant_id, &customer.external_id, vec![], sandbox_mode, true).await
            .unwrap_or_else(|e| {
                tracing::warn!("Auto-provision wallets failed for customer {}: {}", customer.external_id, e);
                vec![]
            });

        let audit_details = serde_json::json!({
            "external_id": req.external_id,
            "email": req.email.as_ref().map(|e| mask_email(e))
        });
        let _ = sqlx::query("INSERT INTO audit_logs (merchant_id, action_type, details) VALUES ($1, $2, $3)")
            .bind(merchant_id)
            .bind("customer.registered")
            .bind(&audit_details)
            .execute(&self.db_pool)
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
        let customer = self.get_verified_customer(merchant_id, external_id, sandbox_mode).await?;

        let encryption = Encryption::new()
            .map_err(|e| ServiceError::InternalError(format!("Encryption setup failed: {}", e)))?;

        let mut networks = networks;
        if networks.is_empty() {
             let merchant_networks: Vec<String> = sqlx::query_scalar::<_, String>(
                 "SELECT DISTINCT network FROM merchant_wallets WHERE merchant_id = $1 AND sandbox_mode = $2 AND is_active = true"
             )
             .bind(merchant_id)
             .bind(sandbox_mode)
             .fetch_all(&self.db_pool)
             .await?;
             
             networks = merchant_networks;
        }

        let mut wallets: Vec<MerchantCustomerWallet> = Vec::new();

        for network_type in networks {
            let normalized = network_type.to_uppercase();
            match normalized.as_str() {
                "EVM" | "ETH" | "ERC20" | "BSC" | "BEP20" | "POLYGON" | "MATIC" | "ARB" | "ARBITRUM" | "NATIVE" | "ETHEREUM" => {
                    if wallets.iter().any(|w| w.network == "Ethereum") {
                        continue;
                    }
                    
                    let keypair = KeyGenerator::generate_evm_wallet()?;
                    let encrypted_key = encryption.encrypt(&keypair.private_key)
                        .map_err(|e| ServiceError::InternalError(format!("Encryption failed: {}", e)))?;

                    let evm_cryptos = vec![
                        CryptoType::Eth, CryptoType::UsdtEth,
                        CryptoType::Bnb, CryptoType::UsdtBep20, CryptoType::BusdBep20,
                        CryptoType::Matic, CryptoType::UsdtPolygon,
                        CryptoType::Arb, CryptoType::UsdtArbitrum,
                    ];

                    for crypto in evm_cryptos {
                        let wallet = self.save_customer_wallet(
                            customer.id, merchant_id, crypto,
                            keypair.address.clone(), encrypted_key.clone(),
                            sandbox_mode, bypass_lock,
                        ).await?;
                        wallets.push(wallet);
                    }
                },
                "SOLANA" | "SOL" | "SPL" => {
                    if wallets.iter().any(|w| w.network == "Solana") {
                        continue;
                    }

                    let keypair = KeyGenerator::generate_solana_wallet()?;
                    let encrypted_key = encryption.encrypt(&keypair.private_key)
                        .map_err(|e| ServiceError::InternalError(format!("Encryption failed: {}", e)))?;

                    let sol_cryptos = vec![CryptoType::Sol, CryptoType::UsdtSpl];

                    for crypto in sol_cryptos {
                        let wallet = self.save_customer_wallet(
                            customer.id, merchant_id, crypto,
                            keypair.address.clone(), encrypted_key.clone(),
                            sandbox_mode, bypass_lock,
                        ).await?;
                        wallets.push(wallet);
                    }
                },
                "BITCOIN" | "BTC" => {
                    if wallets.iter().any(|w| w.network == "Bitcoin") {
                        continue;
                    }

                    let keypair = KeyGenerator::generate_btc_wallet(sandbox_mode)?;
                    let encrypted_key = encryption.encrypt(&keypair.private_key)
                        .map_err(|e| ServiceError::InternalError(format!("Encryption failed: {}", e)))?;

                    let wallet = self.save_customer_wallet(
                        customer.id, merchant_id, CryptoType::Btc,
                        keypair.address.clone(), encrypted_key.clone(),
                        sandbox_mode, bypass_lock,
                    ).await?;
                    wallets.push(wallet);
                },
                _ => return Err(ServiceError::ValidationError(format!("Unsupported network type: {}", network_type))),
            }
        }

        Ok(wallets)
    }

    async fn save_customer_wallet(
        &self,
        customer_id: i64,
        merchant_id: i64,
        crypto_type: CryptoType,
        address: String,
        encrypted_key: String,
        sandbox_mode: bool,
        bypass_lock: bool,
    ) -> Result<MerchantCustomerWallet, ServiceError> {
        let network = crypto_type.network().to_string();
        let crypto_str = crypto_type.to_string();

        tracing::info!(
            "save_customer_wallet: customer={}, merchant={}, crypto={}, sandbox={}",
            customer_id, merchant_id, crypto_str, sandbox_mode
        );

        // 1. Check if customer wallets are locked for this merchant
        let customer_wallets_locked = sqlx::query_scalar::<_, bool>(
            "SELECT customer_wallets_locked FROM merchants WHERE id = $1"
        )
        .bind(merchant_id)
        .fetch_one(&self.db_pool)
        .await
        .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        // 2. Check if this is an existing customer with any wallets
        let has_existing_wallets = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM merchant_customer_wallets WHERE customer_id = $1 AND sandbox_mode = $2"
        )
        .bind(customer_id)
        .bind(sandbox_mode)
        .fetch_one(&self.db_pool)
        .await
        .map_err(|e| ServiceError::DatabaseError(e.to_string()))? > 0;

        // 3. Fetch current wallet if it exists for this specific crypto
        let current_wallet = sqlx::query(
            "SELECT address, encrypted_private_key FROM merchant_customer_wallets WHERE customer_id = $1 AND crypto_type = $2 AND sandbox_mode = $3"
        )
        .bind(customer_id)
        .bind(&crypto_str)
        .bind(sandbox_mode)
        .fetch_optional(&self.db_pool)
        .await
        .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;

        if current_wallet.is_none() && has_existing_wallets && customer_wallets_locked && !bypass_lock {
            tracing::warn!("Blocked new currency provisioning for existing customer {} (customer wallets locked)", customer_id);
            return Err(ServiceError::BadRequest(
                "Customer wallets are locked. Please unlock in settings to provision new currencies for this user.".to_string()
            ));
        }

        if let Some(row) = current_wallet {
            let current_address: String = row.get("address");
            let current_key: String = row.get("encrypted_private_key");

            if current_address != address {
                if customer_wallets_locked {
                    tracing::warn!("Blocked customer wallet change for merchant {} (customer wallets locked)", merchant_id);
                    return Err(ServiceError::BadRequest(
                        "Customer wallets are locked. Please unlock in settings to change.".to_string()
                    ));
                }

                tracing::info!(
                    "Archiving customer wallet state for customer {}: address changed",
                    customer_id
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
                    "#
                )
                .bind(merchant_id)
                .bind(customer_id)
                .bind(&crypto_str)
                .bind(&network)
                .bind(&current_address)
                .bind(&address)
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
        .bind(customer_id)
        .bind(merchant_id)
        .bind(&crypto_str)
        .bind(&network)
        .bind(&address)
        .bind(&encrypted_key)
        .bind(sandbox_mode)
        .fetch_one(&self.db_pool)
        .await?;

        // Initialize balance record
        sqlx::query(
            "INSERT INTO merchant_customer_balances (customer_id, merchant_id, crypto_type, sandbox_mode) VALUES ($1, $2, $3, $4) ON CONFLICT (customer_id, crypto_type, sandbox_mode) DO NOTHING"
        )
        .bind(customer_id)
        .bind(merchant_id)
        .bind(&crypto_str)
        .bind(sandbox_mode)
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
        let customer = self.get_verified_customer(merchant_id, external_id, sandbox_mode).await?;
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
        let customer = self.get_verified_customer(merchant_id, external_id, sandbox_mode).await?;
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
            SELECT id, customer_id, merchant_id, type, crypto_type, amount, fee, status,
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
            "SELECT COUNT(*) FROM merchant_customers WHERE merchant_id = $1 AND sandbox_mode = $2"
        )
        .bind(merchant_id)
        .bind(sandbox_mode)
        .fetch_one(&self.db_pool)
        .await?;

        let customers = sqlx::query_as::<_, MerchantCustomer>(
            &format!(r#"
            SELECT {} 
            FROM merchant_customers 
            WHERE merchant_id = $1 AND sandbox_mode = $2
            ORDER BY created_at DESC 
            LIMIT $3 OFFSET $4
            "#, CUSTOMER_COLS)
        )
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

    pub async fn withdraw_from_customer(
        &self,
        merchant_id: i64,
        external_id: &str,
        crypto_type_str: &str,
        amount_str: &str,
        destination_address: &str,
        sandbox_mode: bool,
    ) -> Result<crate::models::withdrawal::Withdrawal, ServiceError> {
        let amount = Decimal::from_str(amount_str)
            .map_err(|_| ServiceError::ValidationError(format!("Invalid amount format: {}", amount_str)))?;

        // 1. Verify customer and check permissions
        let customer = self.get_verified_customer(merchant_id, external_id, sandbox_mode).await?;
        Self::check_withdrawal_permissions(&customer, amount)?;

        // 2. Fetch wallet
        let _wallet = sqlx::query_as::<_, MerchantCustomerWallet>(
            "SELECT id, customer_id, merchant_id, crypto_type, network, address, encrypted_private_key, created_at, updated_at, sandbox_mode FROM merchant_customer_wallets WHERE customer_id = $1 AND crypto_type = $2 AND sandbox_mode = $3"
        )
        .bind(customer.id)
        .bind(crypto_type_str)
        .bind(sandbox_mode)
        .fetch_one(&self.db_pool)
        .await
        .map_err(|_| ServiceError::ValidationError(format!("Wallet for {} not found for this customer", crypto_type_str)))?;

        // 3. Check Balance
        let balance = sqlx::query_as::<_, MerchantCustomerBalance>(
            "SELECT id, customer_id, merchant_id, crypto_type, available_balance, locked_balance, total_balance, last_updated_at, sandbox_mode FROM merchant_customer_balances WHERE customer_id = $1 AND crypto_type = $2 AND sandbox_mode = $3"
        )
        .bind(customer.id)
        .bind(crypto_type_str)
        .bind(sandbox_mode)
        .fetch_optional(&self.db_pool)
        .await?;

        match balance {
            Some(b) if b.available_balance >= amount => {},
            _ => return Err(ServiceError::InsufficientFunds(crypto_type_str.to_string())),
        }

        // 4. Lock funds and create withdrawal record
        let mut tx = self.db_pool.begin().await?;

        sqlx::query(
            "UPDATE merchant_customer_balances SET available_balance = available_balance - $1, locked_balance = locked_balance + $1, last_updated_at = NOW() WHERE customer_id = $2 AND crypto_type = $3 AND sandbox_mode = $4"
        )
        .bind(amount)
        .bind(customer.id)
        .bind(crypto_type_str)
        .bind(sandbox_mode)
        .execute(&mut *tx)
        .await?;

        let withdrawal_id = format!("wd_{}", uuid::Uuid::new_v4().to_string().replace("-", ""));
        let withdrawal = sqlx::query_as::<_, crate::models::withdrawal::Withdrawal>(
            r#"
            INSERT INTO withdrawals (
                withdrawal_id, merchant_id, crypto_type, amount, destination_address,
                status, fee, net_amount, created_at, updated_at, sandbox_mode
            )
            VALUES ($1, $2, $3, $4, $5, 'PENDING', $6, $7, NOW(), NOW(), $8)
            RETURNING id, withdrawal_id, merchant_id, crypto_type, 
                     amount, destination_address, status, fee, net_amount, transaction_hash,
                     rejection_reason, requires_approval, approved_by, approved_at, 
                     completed_at, created_at, updated_at
            "#
        )
        .bind(&withdrawal_id)
        .bind(merchant_id)
        .bind(crypto_type_str)
        .bind(amount)
        .bind(destination_address)
        .bind(Decimal::ZERO)
        .bind(amount)
        .bind(sandbox_mode)
        .fetch_one(&mut *tx)
        .await?;

        // Record in customer_transactions ledger
        sqlx::query(
            r#"
            INSERT INTO customer_transactions (customer_id, merchant_id, type, crypto_type, amount, fee, status, destination_address, reference_id, description, sandbox_mode)
            VALUES ($1, $2, 'WITHDRAWAL', $3, $4, 0, 'PENDING', $5, $6, 'Withdrawal to external address', $7)
            "#
        )
        .bind(customer.id)
        .bind(merchant_id)
        .bind(crypto_type_str)
        .bind(amount)
        .bind(destination_address)
        .bind(&withdrawal_id)
        .bind(sandbox_mode)
        .execute(&mut *tx)
        .await?;

        let audit_details = serde_json::json!({
            "customer_external_id": external_id,
            "amount": amount_str,
            "crypto_type": crypto_type_str,
            "destination_address": mask_address(destination_address),
            "withdrawal_id": withdrawal_id
        });
        sqlx::query("INSERT INTO audit_logs (merchant_id, action_type, details) VALUES ($1, $2, $3)")
            .bind(merchant_id)
            .bind("customer.withdrawal")
            .bind(&audit_details)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;

        Ok(withdrawal)
    }

    /// Customer pays merchant — real on-chain transaction
    pub async fn pay_merchant(
        &self,
        merchant_id: i64,
        external_id: &str,
        crypto_type_str: &str,
        amount_str: &str,
        reference_id: Option<&str>,
        description: Option<&str>,
        sandbox_mode: bool,
    ) -> Result<CustomerTransaction, ServiceError> {
        let amount = Decimal::from_str(amount_str)
            .map_err(|_| ServiceError::ValidationError(format!("Invalid amount format: {}", amount_str)))?;

        if amount <= Decimal::ZERO {
            return Err(ServiceError::ValidationError("Amount must be greater than zero".to_string()));
        }

        // 1. Verify customer permissions
        let customer = self.get_verified_customer(merchant_id, external_id, sandbox_mode).await?;
        Self::check_permissions(&customer, "pay")?;

        // 2. Get customer's wallet (need private key for on-chain tx)
        let wallet = sqlx::query_as::<_, MerchantCustomerWallet>(
            "SELECT id, customer_id, merchant_id, crypto_type, network, address, encrypted_private_key, created_at, updated_at, sandbox_mode FROM merchant_customer_wallets WHERE customer_id = $1 AND crypto_type = $2 AND sandbox_mode = $3"
        )
        .bind(customer.id)
        .bind(crypto_type_str)
        .bind(sandbox_mode)
        .fetch_one(&self.db_pool)
        .await
        .map_err(|_| ServiceError::ValidationError(format!("Wallet for {} not found", crypto_type_str)))?;

        // 3. Get merchant's receiving wallet address
        let merchant_wallet_address: Option<String> = sqlx::query_scalar::<_, String>(
            "SELECT address FROM merchant_wallets WHERE merchant_id = $1 AND crypto_type = $2 AND is_active = true AND sandbox_mode = $3"
        )
        .bind(merchant_id)
        .bind(crypto_type_str)
        .bind(sandbox_mode)
        .fetch_optional(&self.db_pool)
        .await?;

        let merchant_address = merchant_wallet_address
            .ok_or_else(|| ServiceError::ValidationError(format!("Merchant has no active wallet for {}", crypto_type_str)))?;

        // 4. Check customer balance (locked for update in transaction)
        let mut tx = self.db_pool.begin().await?;

        let balance = sqlx::query_as::<_, MerchantCustomerBalance>(
            "SELECT id, customer_id, merchant_id, crypto_type, available_balance, locked_balance, total_balance, last_updated_at, sandbox_mode FROM merchant_customer_balances WHERE customer_id = $1 AND crypto_type = $2 AND sandbox_mode = $3 FOR UPDATE"
        )
        .bind(customer.id)
        .bind(crypto_type_str)
        .bind(sandbox_mode)
        .fetch_optional(&mut *tx)
        .await?;

        match balance {
            Some(b) if b.available_balance >= amount => {},
            _ => return Err(ServiceError::InsufficientFunds(crypto_type_str.to_string())),
        }

        // 5. Deduct customer funds and credit merchant off-chain instantly

        sqlx::query(
            "UPDATE merchant_customer_balances SET available_balance = available_balance - $1, total_balance = total_balance - $1, last_updated_at = NOW() WHERE customer_id = $2 AND crypto_type = $3 AND sandbox_mode = $4"
        )
        .bind(amount)
        .bind(customer.id)
        .bind(crypto_type_str)
        .bind(sandbox_mode)
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO merchant_balances (merchant_id, crypto_type, available_balance, reserved_balance, last_updated, sandbox_mode)
            VALUES ($1, $2, $3, 0, NOW(), $4)
            ON CONFLICT (merchant_id, crypto_type, sandbox_mode) 
            DO UPDATE SET 
                available_balance = merchant_balances.available_balance + $3,
                last_updated = NOW()
            "#
        )
        .bind(merchant_id)
        .bind(crypto_type_str)
        .bind(amount)
        .bind(sandbox_mode)
        .execute(&mut *tx)
        .await?;

        let tx_ref = reference_id.unwrap_or("").to_string();
        let tx_desc = description.unwrap_or("Payment to merchant").to_string();

        let customer_tx = sqlx::query_as::<_, CustomerTransaction>(
            r#"
            INSERT INTO customer_transactions (customer_id, merchant_id, type, crypto_type, amount, fee, status, destination_address, reference_id, description, sandbox_mode)
            VALUES ($1, $2, 'MERCHANT_PAYMENT', $3, $4, 0, 'COMPLETED', $5, $6, $7, $8)
            RETURNING id, customer_id, merchant_id, type, crypto_type, amount, fee, status,
                      destination_address, transaction_hash, reference_id, description,
                      created_at, updated_at, sandbox_mode
            "#
        )
        .bind(customer.id)
        .bind(merchant_id)
        .bind(crypto_type_str)
        .bind(amount)
        .bind(&merchant_address)
        .bind(&tx_ref)
        .bind(&tx_desc)
        .bind(sandbox_mode)
        .fetch_one(&mut *tx)
        .await?;

        let audit_details = serde_json::json!({
            "customer_external_id": external_id,
            "amount": amount_str,
            "crypto_type": crypto_type_str,
            "reference_id": reference_id.unwrap_or(""),
            "description": description.unwrap_or("")
        });
        sqlx::query("INSERT INTO audit_logs (merchant_id, action_type, details) VALUES ($1, $2, $3)")
            .bind(merchant_id)
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
        crypto_type_str: &str,
        amount_str: Option<String>,
        sandbox_mode: bool,
    ) -> Result<Decimal, ServiceError> {
        let customer = self.get_verified_customer(merchant_id, external_id, sandbox_mode).await?;

        let balance_record = sqlx::query_as::<_, MerchantCustomerBalance>(
            "SELECT id, customer_id, merchant_id, crypto_type, available_balance, locked_balance, total_balance, last_updated_at, sandbox_mode FROM merchant_customer_balances WHERE customer_id = $1 AND crypto_type = $2 AND sandbox_mode = $3 FOR UPDATE"
        )
        .bind(customer.id)
        .bind(crypto_type_str)
        .bind(sandbox_mode)
        .fetch_optional(&self.db_pool)
        .await?
        .ok_or_else(|| ServiceError::ValidationError(format!("No balance record found for {}", crypto_type_str)))?;

        let amount = match amount_str {
            Some(ref amt) => Decimal::from_str(amt)
                .map_err(|_| ServiceError::ValidationError(format!("Invalid amount format: {}", amt)))?,
            None => balance_record.available_balance,
        };

        if amount <= Decimal::ZERO {
             return Err(ServiceError::ValidationError("Amount to sweep must be greater than zero".to_string()));
        }

        if balance_record.available_balance < amount {
            return Err(ServiceError::InsufficientFunds(crypto_type_str.to_string()));
        }

        let mut tx = self.db_pool.begin().await?;

        sqlx::query(
            "UPDATE merchant_customer_balances SET available_balance = available_balance - $1, total_balance = total_balance - $1, last_updated_at = NOW() WHERE customer_id = $2 AND crypto_type = $3 AND sandbox_mode = $4"
        )
        .bind(amount)
        .bind(customer.id)
        .bind(crypto_type_str)
        .bind(sandbox_mode)
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO merchant_balances (merchant_id, crypto_type, available_balance, reserved_balance, last_updated, sandbox_mode)
            VALUES ($1, $2, $3, 0, NOW(), $4)
            ON CONFLICT (merchant_id, crypto_type, sandbox_mode) 
            DO UPDATE SET 
                available_balance = merchant_balances.available_balance + $3,
                last_updated = NOW()
            "#
        )
        .bind(merchant_id)
        .bind(crypto_type_str)
        .bind(amount)
        .bind(sandbox_mode)
        .execute(&mut *tx)
        .await?;

        // Record in customer_transactions
        sqlx::query(
            r#"
            INSERT INTO customer_transactions (customer_id, merchant_id, type, crypto_type, amount, fee, status, description, sandbox_mode)
            VALUES ($1, $2, 'SWEEP', $3, $4, 0, 'COMPLETED', 'Funds swept to merchant balance', $5)
            "#
        )
        .bind(customer.id)
        .bind(merchant_id)
        .bind(crypto_type_str)
        .bind(amount)
        .bind(sandbox_mode)
        .execute(&mut *tx)
        .await?;

        let audit_details = serde_json::json!({
            "customer_external_id": external_id,
            "amount": amount.to_string(),
            "crypto_type": crypto_type_str
        });
        sqlx::query("INSERT INTO audit_logs (merchant_id, action_type, details) VALUES ($1, $2, $3)")
            .bind(merchant_id)
            .bind("customer.sweep")
            .bind(&audit_details)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;

        Ok(amount)
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
            "active" | "flagged" | "suspended" | "blocked" => {},
            _ => return Err(ServiceError::ValidationError(format!("Invalid status: {}. Use: active, flagged, suspended, blocked", status))),
        }

        let customer = sqlx::query_as::<_, MerchantCustomer>(
            &format!(r#"
            UPDATE merchant_customers 
            SET status = $3, status_reason = $4, updated_at = NOW()
            WHERE merchant_id = $1 AND external_id = $2 AND sandbox_mode = $5
            RETURNING {}
            "#, CUSTOMER_COLS)
        )
        .bind(merchant_id)
        .bind(external_id)
        .bind(status)
        .bind(reason)
        .bind(sandbox_mode)
        .fetch_one(&self.db_pool)
        .await
        .map_err(|_| ServiceError::ValidationError(format!("Customer {} not found", external_id)))?;

        let audit_details = serde_json::json!({
            "customer_external_id": external_id,
            "status": status,
            "reason": reason.unwrap_or("")
        });
        let _ = sqlx::query("INSERT INTO audit_logs (merchant_id, action_type, details) VALUES ($1, $2, $3)")
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
        let customer = sqlx::query_as::<_, MerchantCustomer>(
            &format!(r#"
            UPDATE merchant_customers 
            SET can_withdraw = COALESCE($3, can_withdraw),
                withdrawal_limit = COALESCE($4, withdrawal_limit),
                updated_at = NOW()
            WHERE merchant_id = $1 AND external_id = $2 AND sandbox_mode = $5
            RETURNING {}
            "#, CUSTOMER_COLS)
        )
        .bind(merchant_id)
        .bind(external_id)
        .bind(can_withdraw)
        .bind(withdrawal_limit)
        .bind(sandbox_mode)
        .fetch_one(&self.db_pool)
        .await
        .map_err(|_| ServiceError::ValidationError(format!("Customer {} not found", external_id)))?;

        let audit_details = serde_json::json!({
            "customer_external_id": external_id,
            "can_withdraw": can_withdraw,
            "withdrawal_limit": withdrawal_limit.map(|l| l.to_string())
        });
        let _ = sqlx::query("INSERT INTO audit_logs (merchant_id, action_type, details) VALUES ($1, $2, $3)")
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
            },
            Ok(_) => Err(ServiceError::ValidationError(format!("Customer {} not found", external_id))),
            Err(e) => Err(ServiceError::DatabaseError(e.to_string())),
        }
    }
}

fn mask_email(email: &str) -> String {
    if let Some(pos) = email.find('@') {
        let (name, domain) = email.split_at(pos);
        if name.len() > 6 {
            format!("{}...{}{}", &name[..3], &name[name.len()-3..], domain)
        } else {
            format!("***{}", domain)
        }
    } else {
        email.to_string()
    }
}

fn mask_address(addr: &str) -> String {
    if addr.len() > 10 {
        format!("{}...{}", &addr[..6], &addr[addr.len()-4..])
    } else {
        addr.to_string()
    }
}
