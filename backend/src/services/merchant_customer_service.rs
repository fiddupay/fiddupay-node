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
use sqlx::PgPool;
use rust_decimal::Decimal;
use std::str::FromStr;

const CUSTOMER_COLS: &str = "id, merchant_id, external_id, email, first_name, last_name, metadata, is_active, status, status_reason, can_withdraw, withdrawal_limit, created_at, updated_at";

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
    async fn get_verified_customer(&self, merchant_id: i64, external_id: &str) -> Result<MerchantCustomer, ServiceError> {
        let customer = sqlx::query_as::<_, MerchantCustomer>(
            &format!("SELECT {} FROM merchant_customers WHERE merchant_id = $1 AND external_id = $2", CUSTOMER_COLS)
        )
        .bind(merchant_id)
        .bind(external_id)
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
        let wallets = self.provision_wallets(merchant_id, &customer.external_id, vec![]).await
            .unwrap_or_else(|e| {
                tracing::warn!("Auto-provision wallets failed for customer {}: {}", customer.external_id, e);
                vec![]
            });

        Ok((customer, wallets))
    }

    /// Provision unique wallets for a customer across multiple networks
    pub async fn provision_wallets(
        &self,
        merchant_id: i64,
        external_id: &str,
        networks: Vec<String>,
    ) -> Result<Vec<MerchantCustomerWallet>, ServiceError> {
        let customer = self.get_verified_customer(merchant_id, external_id).await?;

        let encryption = Encryption::new()
            .map_err(|e| ServiceError::InternalError(format!("Encryption setup failed: {}", e)))?;

        let mut networks = networks;
        if networks.is_empty() {
             let merchant_networks: Vec<String> = sqlx::query_scalar::<_, String>(
                 "SELECT DISTINCT network FROM merchant_currencies WHERE merchant_id = $1"
             )
             .bind(merchant_id)
             .fetch_all(&self.db_pool)
             .await?;
             
             networks = merchant_networks;
        }

        let mut wallets: Vec<MerchantCustomerWallet> = Vec::new();

        for network_type in networks {
            let normalized = network_type.to_uppercase();
            match normalized.as_str() {
                "EVM" | "ETH" | "ERC20" | "BSC" | "BEP20" | "POLYGON" | "MATIC" | "ARB" | "ARBITRUM" | "NATIVE" => {
                    if wallets.iter().any(|w| w.network == "Ethereum") {
                        continue;
                    }
                    
                    let keypair = KeyGenerator::generate_evm_wallet()?;
                    let encrypted_key = encryption.encrypt(&keypair.private_key)
                        .map_err(|e| ServiceError::InternalError(format!("Encryption failed: {}", e)))?;

                    let evm_cryptos = vec![
                        CryptoType::Eth, CryptoType::UsdtEth,
                        CryptoType::Bnb, CryptoType::UsdtBep20,
                        CryptoType::Matic, CryptoType::UsdtPolygon,
                        CryptoType::Arb, CryptoType::UsdtArbitrum,
                    ];

                    for crypto in evm_cryptos {
                        let wallet = self.save_customer_wallet(
                            customer.id, merchant_id, crypto,
                            keypair.address.clone(), encrypted_key.clone(),
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
                        ).await?;
                        wallets.push(wallet);
                    }
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
    ) -> Result<MerchantCustomerWallet, ServiceError> {
        let network = crypto_type.network().to_string();
        let crypto_str = crypto_type.to_string();

        let wallet = sqlx::query_as::<_, MerchantCustomerWallet>(
            r#"
            INSERT INTO merchant_customer_wallets (customer_id, merchant_id, crypto_type, network, address, encrypted_private_key)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (customer_id, crypto_type) DO UPDATE SET address = EXCLUDED.address, encrypted_private_key = EXCLUDED.encrypted_private_key, updated_at = NOW()
            RETURNING id, customer_id, merchant_id, crypto_type, network, address, encrypted_private_key, created_at, updated_at
            "#
        )
        .bind(customer_id)
        .bind(merchant_id)
        .bind(&crypto_str)
        .bind(&network)
        .bind(&address)
        .bind(&encrypted_key)
        .fetch_one(&self.db_pool)
        .await?;

        // Initialize balance record
        sqlx::query(
            "INSERT INTO merchant_customer_balances (customer_id, merchant_id, crypto_type) VALUES ($1, $2, $3) ON CONFLICT (customer_id, crypto_type) DO NOTHING"
        )
        .bind(customer_id)
        .bind(merchant_id)
        .bind(&crypto_str)
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
    ) -> Result<Vec<MerchantCustomerBalance>, ServiceError> {
        let balances = sqlx::query_as::<_, MerchantCustomerBalance>(
            r#"
            SELECT mb.id, mb.customer_id, mb.merchant_id, mb.crypto_type, mb.available_balance, mb.locked_balance, mb.total_balance, mb.last_updated_at
            FROM merchant_customer_balances mb
            JOIN merchant_customers mc ON mc.id = mb.customer_id
            WHERE mc.merchant_id = $1 AND mc.external_id = $2
            "#
        )
        .bind(merchant_id)
        .bind(external_id)
        .fetch_all(&self.db_pool)
        .await?;

        Ok(balances)
    }

    pub async fn get_customer_wallets(
        &self,
        merchant_id: i64,
        external_id: &str,
    ) -> Result<Vec<MerchantCustomerWallet>, ServiceError> {
        let wallets = sqlx::query_as::<_, MerchantCustomerWallet>(
            r#"
            SELECT w.id, w.customer_id, w.merchant_id, w.crypto_type, w.network, w.address, w.encrypted_private_key, w.created_at, w.updated_at
            FROM merchant_customer_wallets w
            JOIN merchant_customers mc ON mc.id = w.customer_id
            WHERE mc.merchant_id = $1 AND mc.external_id = $2
            ORDER BY w.created_at
            "#
        )
        .bind(merchant_id)
        .bind(external_id)
        .fetch_all(&self.db_pool)
        .await?;

        Ok(wallets)
    }

    pub async fn get_deposit_address(
        &self,
        merchant_id: i64,
        external_id: &str,
        crypto_type_str: &str,
    ) -> Result<String, ServiceError> {
        let customer = self.get_verified_customer(merchant_id, external_id).await?;
        Self::check_permissions(&customer, "view")?;

        let wallet = sqlx::query_as::<_, MerchantCustomerWallet>(
            "SELECT id, customer_id, merchant_id, crypto_type, network, address, encrypted_private_key, created_at, updated_at FROM merchant_customer_wallets WHERE customer_id = $1 AND crypto_type = $2"
        )
        .bind(customer.id)
        .bind(crypto_type_str)
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
    ) -> Result<(Vec<CustomerTransaction>, i64), ServiceError> {
        let customer = self.get_verified_customer(merchant_id, external_id).await?;
        Self::check_permissions(&customer, "view")?;

        let total: i64 = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM customer_transactions WHERE customer_id = $1 AND merchant_id = $2"
        )
        .bind(customer.id)
        .bind(merchant_id)
        .fetch_one(&self.db_pool)
        .await?;

        let transactions = sqlx::query_as::<_, CustomerTransaction>(
            r#"
            SELECT id, customer_id, merchant_id, type, crypto_type, amount, fee, status,
                   destination_address, transaction_hash, reference_id, description,
                   created_at, updated_at
            FROM customer_transactions
            WHERE customer_id = $1 AND merchant_id = $2
            ORDER BY created_at DESC
            LIMIT $3 OFFSET $4
            "#
        )
        .bind(customer.id)
        .bind(merchant_id)
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
            "SELECT COUNT(*) FROM merchant_customers WHERE merchant_id = $1"
        )
        .bind(merchant_id)
        .fetch_one(&self.db_pool)
        .await?;

        let customers = sqlx::query_as::<_, MerchantCustomer>(
            &format!(r#"
            SELECT {} 
            FROM merchant_customers 
            WHERE merchant_id = $1
            ORDER BY created_at DESC 
            LIMIT $2 OFFSET $3
            "#, CUSTOMER_COLS)
        )
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
        let customer = self.get_verified_customer(merchant_id, external_id).await?;
        Self::check_withdrawal_permissions(&customer, amount)?;

        // 2. Fetch wallet
        let _wallet = sqlx::query_as::<_, MerchantCustomerWallet>(
            "SELECT id, customer_id, merchant_id, crypto_type, network, address, encrypted_private_key, created_at, updated_at FROM merchant_customer_wallets WHERE customer_id = $1 AND crypto_type = $2"
        )
        .bind(customer.id)
        .bind(crypto_type_str)
        .fetch_one(&self.db_pool)
        .await
        .map_err(|_| ServiceError::ValidationError(format!("Wallet for {} not found for this customer", crypto_type_str)))?;

        // 3. Check Balance
        let balance = sqlx::query_as::<_, MerchantCustomerBalance>(
            "SELECT id, customer_id, merchant_id, crypto_type, available_balance, locked_balance, total_balance, last_updated_at FROM merchant_customer_balances WHERE customer_id = $1 AND crypto_type = $2"
        )
        .bind(customer.id)
        .bind(crypto_type_str)
        .fetch_optional(&self.db_pool)
        .await?;

        match balance {
            Some(b) if b.available_balance >= amount => {},
            _ => return Err(ServiceError::InsufficientFunds(crypto_type_str.to_string())),
        }

        // 4. Lock funds and create withdrawal record
        let _crypto_type = CryptoType::from_str(crypto_type_str)
            .map_err(|_| ServiceError::ValidationError("Invalid crypto type".to_string()))?;

        let mut tx = self.db_pool.begin().await?;

        sqlx::query(
            "UPDATE merchant_customer_balances SET available_balance = available_balance - $1, locked_balance = locked_balance + $1, last_updated_at = NOW() WHERE customer_id = $2 AND crypto_type = $3"
        )
        .bind(amount)
        .bind(customer.id)
        .bind(crypto_type_str)
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
            INSERT INTO customer_transactions (customer_id, merchant_id, type, crypto_type, amount, fee, status, destination_address, reference_id, description)
            VALUES ($1, $2, 'WITHDRAWAL', $3, $4, 0, 'PENDING', $5, $6, 'Withdrawal to external address')
            "#
        )
        .bind(customer.id)
        .bind(merchant_id)
        .bind(crypto_type_str)
        .bind(amount)
        .bind(destination_address)
        .bind(&withdrawal_id)
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
        let customer = self.get_verified_customer(merchant_id, external_id).await?;
        Self::check_permissions(&customer, "pay")?;

        // 2. Get customer's wallet (need private key for on-chain tx)
        let wallet = sqlx::query_as::<_, MerchantCustomerWallet>(
            "SELECT id, customer_id, merchant_id, crypto_type, network, address, encrypted_private_key, created_at, updated_at FROM merchant_customer_wallets WHERE customer_id = $1 AND crypto_type = $2"
        )
        .bind(customer.id)
        .bind(crypto_type_str)
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

        // 4. Check customer balance
        let balance = sqlx::query_as::<_, MerchantCustomerBalance>(
            "SELECT id, customer_id, merchant_id, crypto_type, available_balance, locked_balance, total_balance, last_updated_at FROM merchant_customer_balances WHERE customer_id = $1 AND crypto_type = $2"
        )
        .bind(customer.id)
        .bind(crypto_type_str)
        .fetch_optional(&self.db_pool)
        .await?;

        match balance {
            Some(b) if b.available_balance >= amount => {},
            _ => return Err(ServiceError::InsufficientFunds(crypto_type_str.to_string())),
        }

        // 5. Lock customer funds and create transaction record
        let mut tx = self.db_pool.begin().await?;

        sqlx::query(
            "UPDATE merchant_customer_balances SET available_balance = available_balance - $1, locked_balance = locked_balance + $1, last_updated_at = NOW() WHERE customer_id = $2 AND crypto_type = $3"
        )
        .bind(amount)
        .bind(customer.id)
        .bind(crypto_type_str)
        .execute(&mut *tx)
        .await?;

        let tx_ref = reference_id.unwrap_or("").to_string();
        let tx_desc = description.unwrap_or("Payment to merchant").to_string();

        let customer_tx = sqlx::query_as::<_, CustomerTransaction>(
            r#"
            INSERT INTO customer_transactions (customer_id, merchant_id, type, crypto_type, amount, fee, status, destination_address, reference_id, description)
            VALUES ($1, $2, 'MERCHANT_PAYMENT', $3, $4, 0, 'PENDING', $5, $6, $7)
            RETURNING id, customer_id, merchant_id, type, crypto_type, amount, fee, status,
                      destination_address, transaction_hash, reference_id, description,
                      created_at, updated_at
            "#
        )
        .bind(customer.id)
        .bind(merchant_id)
        .bind(crypto_type_str)
        .bind(amount)
        .bind(&merchant_address)
        .bind(&tx_ref)
        .bind(&tx_desc)
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;

        // 6. The actual on-chain transaction will be processed by the withdrawal processor
        //    (same infrastructure as regular withdrawals). We create a withdrawal record
        //    pointing to the merchant's wallet address.
        let wd_id = format!("cp_{}", uuid::Uuid::new_v4().to_string().replace("-", ""));
        let _wd = sqlx::query(
            r#"
            INSERT INTO withdrawals (
                withdrawal_id, merchant_id, crypto_type, amount, destination_address,
                status, fee, net_amount, created_at, updated_at, sandbox_mode
            )
            VALUES ($1, $2, $3, $4, $5, 'PENDING', 0, $4, NOW(), NOW(), $6)
            "#
        )
        .bind(&wd_id)
        .bind(merchant_id)
        .bind(crypto_type_str)
        .bind(amount)
        .bind(&merchant_address)
        .bind(sandbox_mode)
        .execute(&self.db_pool)
        .await?;

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
        let customer = self.get_verified_customer(merchant_id, external_id).await?;

        let balance_record = sqlx::query_as::<_, MerchantCustomerBalance>(
            "SELECT id, customer_id, merchant_id, crypto_type, available_balance, locked_balance, total_balance, last_updated_at FROM merchant_customer_balances WHERE customer_id = $1 AND crypto_type = $2 FOR UPDATE"
        )
        .bind(customer.id)
        .bind(crypto_type_str)
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

        let _crypto_type = CryptoType::from_str(crypto_type_str)
            .map_err(|_| ServiceError::ValidationError("Invalid crypto type".to_string()))?;

        let mut tx = self.db_pool.begin().await?;

        sqlx::query(
            "UPDATE merchant_customer_balances SET available_balance = available_balance - $1, total_balance = total_balance - $1, last_updated_at = NOW() WHERE customer_id = $2 AND crypto_type = $3"
        )
        .bind(amount)
        .bind(customer.id)
        .bind(crypto_type_str)
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
            INSERT INTO customer_transactions (customer_id, merchant_id, type, crypto_type, amount, fee, status, description)
            VALUES ($1, $2, 'SWEEP', $3, $4, 0, 'COMPLETED', 'Funds swept to merchant balance')
            "#
        )
        .bind(customer.id)
        .bind(merchant_id)
        .bind(crypto_type_str)
        .bind(amount)
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
            WHERE merchant_id = $1 AND external_id = $2
            RETURNING {}
            "#, CUSTOMER_COLS)
        )
        .bind(merchant_id)
        .bind(external_id)
        .bind(status)
        .bind(reason)
        .fetch_one(&self.db_pool)
        .await
        .map_err(|_| ServiceError::ValidationError(format!("Customer {} not found", external_id)))?;

        Ok(customer)
    }

    pub async fn update_customer_permissions(
        &self,
        merchant_id: i64,
        external_id: &str,
        can_withdraw: Option<bool>,
        withdrawal_limit: Option<Decimal>,
    ) -> Result<MerchantCustomer, ServiceError> {
        let customer = sqlx::query_as::<_, MerchantCustomer>(
            &format!(r#"
            UPDATE merchant_customers 
            SET can_withdraw = COALESCE($3, can_withdraw),
                withdrawal_limit = COALESCE($4, withdrawal_limit),
                updated_at = NOW()
            WHERE merchant_id = $1 AND external_id = $2
            RETURNING {}
            "#, CUSTOMER_COLS)
        )
        .bind(merchant_id)
        .bind(external_id)
        .bind(can_withdraw)
        .bind(withdrawal_limit)
        .fetch_one(&self.db_pool)
        .await
        .map_err(|_| ServiceError::ValidationError(format!("Customer {} not found", external_id)))?;

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
            Ok(r) if r.rows_affected() > 0 => Ok(()),
            Ok(_) => Err(ServiceError::ValidationError(format!("Customer {} not found", external_id))),
            Err(e) => Err(ServiceError::DatabaseError(e.to_string())),
        }
    }
}
