// Merchant Customer Service
// Manages sub-accounts and designated user wallets

use crate::error::ServiceError;
use crate::models::merchant_customer::{
    MerchantCustomer, MerchantCustomerWallet, CreateCustomerRequest, MerchantCustomerBalance
};
use crate::payment::models::CryptoType;
use crate::utils::keygen::KeyGenerator;
use crate::utils::encryption::Encryption;
use sqlx::PgPool;

pub struct MerchantCustomerService {
    db_pool: PgPool,
}

impl MerchantCustomerService {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }

    /// Register a new customer for a merchant
    pub async fn register_customer(
        &self,
        merchant_id: i64,
        req: CreateCustomerRequest,
    ) -> Result<MerchantCustomer, ServiceError> {
        let customer_res: Result<MerchantCustomer, sqlx::Error> = sqlx::query_as::<_, MerchantCustomer>(
            r#"
            INSERT INTO merchant_customers (merchant_id, external_id, email, metadata)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (merchant_id, external_id) DO UPDATE SET email = EXCLUDED.email, metadata = EXCLUDED.metadata, updated_at = NOW()
            RETURNING id, merchant_id, external_id, email, metadata, created_at, updated_at
            "#
        )
        .bind(merchant_id)
        .bind(&req.external_id)
        .bind(&req.email)
        .bind(&req.metadata)
        .fetch_one(&self.db_pool)
        .await;

        let customer = customer_res?;

        Ok(customer)
    }

    /// Provision unique wallets for a customer across multiple networks
    pub async fn provision_wallets(
        &self,
        merchant_id: i64,
        external_id: &str,
        networks: Vec<String>,
    ) -> Result<Vec<MerchantCustomerWallet>, ServiceError> {
        // 1. Find customer
        let customer_res: Result<MerchantCustomer, sqlx::Error> = sqlx::query_as::<_, MerchantCustomer>(
            "SELECT id, merchant_id, external_id, email, metadata, created_at, updated_at FROM merchant_customers WHERE merchant_id = $1 AND external_id = $2"
        )
        .bind(merchant_id)
        .bind(external_id)
        .fetch_one(&self.db_pool)
        .await;
        
        let customer = customer_res.map_err(|_| ServiceError::ValidationError(format!("Customer {} not found", external_id)))?;

        let encryption = Encryption::new()
            .map_err(|e| ServiceError::InternalError(format!("Encryption setup failed: {}", e)))?;

        let mut wallets = Vec::new();

        for network_type in networks {
            match network_type.to_lowercase().as_str() {
                "evm" => {
                    // Generate one key for ALL EVM networks
                    let keypair = KeyGenerator::generate_evm_wallet()?;
                    let encrypted_key = encryption.encrypt(&keypair.private_key)
                        .map_err(|e| ServiceError::InternalError(format!("Encryption failed: {}", e)))?;

                    let evm_cryptos = vec![
                        CryptoType::Eth,
                        CryptoType::UsdtEth,
                        CryptoType::Bnb,
                        CryptoType::UsdtBep20,
                        CryptoType::Matic,
                        CryptoType::UsdtPolygon,
                        CryptoType::Arb,
                        CryptoType::UsdtArbitrum,
                    ];

                    for crypto in evm_cryptos {
                        let wallet = self.save_customer_wallet(
                            customer.id,
                            merchant_id,
                            crypto,
                            keypair.address.clone(),
                            encrypted_key.clone(),
                        ).await?;
                        wallets.push(wallet);
                    }
                },
                "solana" => {
                    let keypair = KeyGenerator::generate_solana_wallet()?;
                    let encrypted_key = encryption.encrypt(&keypair.private_key)
                        .map_err(|e| ServiceError::InternalError(format!("Encryption failed: {}", e)))?;

                    let sol_cryptos = vec![CryptoType::Sol, CryptoType::UsdtSpl];

                    for crypto in sol_cryptos {
                        let wallet = self.save_customer_wallet(
                            customer.id,
                            merchant_id,
                            crypto,
                            keypair.address.clone(),
                            encrypted_key.clone(),
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

        let wallet_res: Result<MerchantCustomerWallet, sqlx::Error> = sqlx::query_as::<_, MerchantCustomerWallet>(
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
        .await;

        let wallet = wallet_res?;

        // Also initialize balance record
        let _init_res = sqlx::query(
            r#"
            INSERT INTO merchant_customer_balances (customer_id, merchant_id, crypto_type)
            VALUES ($1, $2, $3)
            ON CONFLICT (customer_id, crypto_type) DO NOTHING
            "#
        )
        .bind(customer_id)
        .bind(merchant_id)
        .bind(&crypto_str)
        .execute(&self.db_pool)
        .await;
        _init_res?;

        Ok(wallet)
    }

    pub async fn get_customer_balances(
        &self,
        merchant_id: i64,
        external_id: &str,
    ) -> Result<Vec<MerchantCustomerBalance>, ServiceError> {
        let balances_res: Result<Vec<MerchantCustomerBalance>, sqlx::Error> = sqlx::query_as::<_, MerchantCustomerBalance>(
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
        .await;

        let balances = balances_res?;

        Ok(balances)
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

        let customers_res: Result<Vec<MerchantCustomer>, sqlx::Error> = sqlx::query_as::<_, MerchantCustomer>(
            r#"
            SELECT id, merchant_id, external_id, email, metadata, created_at, updated_at 
            FROM merchant_customers 
            WHERE merchant_id = $1
            ORDER BY created_at DESC 
            LIMIT $2 OFFSET $3
            "#
        )
        .bind(merchant_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.db_pool)
        .await;

        let customers = customers_res?;

        Ok((customers, total_count))
    }

    pub async fn withdraw_from_customer(
        &self,
        merchant_id: i64,
        external_id: &str,
        crypto_type_str: &str,
        amount_str: &str,
        destination_address: &str,
        sandbox_mode: bool,
    ) -> Result<crate::models::withdrawal::Withdrawal, ServiceError> {
        use rust_decimal::Decimal;
        use std::str::FromStr;

        let amount = Decimal::from_str(amount_str)
            .map_err(|_| ServiceError::ValidationError(format!("Invalid amount format: {}", amount_str)))?;

        // 1. Verify customer belongs to merchant
        let customer_res: Result<MerchantCustomer, sqlx::Error> = sqlx::query_as::<_, MerchantCustomer>(
            "SELECT id, merchant_id, external_id, email, metadata, created_at, updated_at FROM merchant_customers WHERE merchant_id = $1 AND external_id = $2"
        )
        .bind(merchant_id)
        .bind(external_id)
        .fetch_one(&self.db_pool)
        .await;

        let customer = customer_res.map_err(|_| ServiceError::ValidationError(format!("Customer {} not found", external_id)))?;

        // 2. Fetch the wallet to ensure it exists
        let wallet_res: Result<MerchantCustomerWallet, sqlx::Error> = sqlx::query_as::<_, MerchantCustomerWallet>(
            "SELECT id, customer_id, merchant_id, crypto_type, network, address, encrypted_private_key, created_at, updated_at FROM merchant_customer_wallets WHERE customer_id = $1 AND crypto_type = $2"
        )
        .bind(customer.id)
        .bind(crypto_type_str)
        .fetch_one(&self.db_pool)
        .await;

        let _wallet = wallet_res.map_err(|_| ServiceError::ValidationError(format!("Wallet for {} not found for this customer", crypto_type_str)))?;

        // 3. Check Balance
        let balance_res: Result<Option<MerchantCustomerBalance>, sqlx::Error> = sqlx::query_as::<_, MerchantCustomerBalance>(
            "SELECT id, customer_id, merchant_id, crypto_type, available_balance, locked_balance, total_balance, last_updated_at FROM merchant_customer_balances WHERE customer_id = $1 AND crypto_type = $2"
        )
        .bind(customer.id)
        .bind(crypto_type_str)
        .fetch_optional(&self.db_pool)
        .await;

        let balance = balance_res?;

        let has_sufficient_balance = match balance {
            Some(b) => b.available_balance >= amount,
            None => false,
        };

        if !has_sufficient_balance {
            return Err(ServiceError::InsufficientFunds(crypto_type_str.to_string()));
        }

        // 4. Lock funds and create withdrawal record
        let crypto_type = CryptoType::from_str(crypto_type_str)
            .map_err(|_| ServiceError::ValidationError("Invalid crypto type".to_string()))?;

        // Start transaction
        let mut tx = self.db_pool.begin().await?;

        // Deduct from available, add to locked
        let update_res = sqlx::query(
            "UPDATE merchant_customer_balances SET available_balance = available_balance - $1, locked_balance = locked_balance + $1, last_updated_at = NOW() WHERE customer_id = $2 AND crypto_type = $3"
        )
        .bind(amount)
        .bind(customer.id)
        .bind(crypto_type_str)
        .execute(&mut *tx)
        .await;
        update_res?;

        // The actual withdrawal processing is handled by the unified wallet system / cron jobs,
        // so we just create a standard withdrawal record linked to the merchant.
        // For sub-accounts, the source address is the customer's wallet address.
        let withdrawal_id = format!("wd_{}", uuid::Uuid::new_v4().to_string().replace("-", ""));
        let withdrawal_res: Result<crate::models::withdrawal::Withdrawal, sqlx::Error> = sqlx::query_as::<_, crate::models::withdrawal::Withdrawal>(
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
        .await;

        let withdrawal = withdrawal_res?;

        tx.commit().await?;

        Ok(withdrawal)
    }

    pub async fn sweep_customer_wallet(
        &self,
        merchant_id: i64,
        external_id: &str,
        crypto_type_str: &str,
        amount_str: Option<String>,
        sandbox_mode: bool,
    ) -> Result<rust_decimal::Decimal, ServiceError> {
        use rust_decimal::Decimal;
        use std::str::FromStr;

        // 1. Verify customer belongs to merchant
        let customer_res: Result<MerchantCustomer, sqlx::Error> = sqlx::query_as::<_, MerchantCustomer>(
            "SELECT id, merchant_id, external_id, email, metadata, created_at, updated_at FROM merchant_customers WHERE merchant_id = $1 AND external_id = $2"
        )
        .bind(merchant_id)
        .bind(external_id)
        .fetch_one(&self.db_pool)
        .await;

        let customer = customer_res.map_err(|_| ServiceError::ValidationError(format!("Customer {} not found", external_id)))?;

        // 2. Fetch Customer Balance explicitly to lock it for update if possible, or just read it
        let balance_res: Result<Option<MerchantCustomerBalance>, sqlx::Error> = sqlx::query_as::<_, MerchantCustomerBalance>(
            "SELECT id, customer_id, merchant_id, crypto_type, available_balance, locked_balance, total_balance, last_updated_at FROM merchant_customer_balances WHERE customer_id = $1 AND crypto_type = $2 FOR UPDATE"
        )
        .bind(customer.id)
        .bind(crypto_type_str)
        .fetch_optional(&self.db_pool)
        .await;

        let balance = balance_res?;

        let balance_record = balance.ok_or_else(|| ServiceError::ValidationError(format!("No balance record found for {}", crypto_type_str)))?;

        // 3. Determine amount to sweep
        let amount = match amount_str {
            Some(ref amt) => Decimal::from_str(amt)
                .map_err(|_| ServiceError::ValidationError(format!("Invalid amount format: {}", amt)))?,
            None => balance_record.available_balance, // Sweep everything available
        };

        if amount <= Decimal::ZERO {
             return Err(ServiceError::ValidationError("Amount to sweep must be greater than zero".to_string()));
        }

        if balance_record.available_balance < amount {
            return Err(ServiceError::InsufficientFunds(crypto_type_str.to_string()));
        }

        let crypto_type = CryptoType::from_str(crypto_type_str)
            .map_err(|_| ServiceError::ValidationError("Invalid crypto type".to_string()))?;

        // Start transaction
        let mut tx = self.db_pool.begin().await?;

        // 4. Deduct from customer's available balance
        let deduct_res = sqlx::query(
            "UPDATE merchant_customer_balances SET available_balance = available_balance - $1, total_balance = total_balance - $1, last_updated_at = NOW() WHERE customer_id = $2 AND crypto_type = $3"
        )
        .bind(amount)
        .bind(customer.id)
        .bind(crypto_type_str)
        .execute(&mut *tx)
        .await;
        deduct_res?;

        // 5. Initialize/Update Merchant's main balance using standard logic
        // We will insert/update directly here to stay within the transaction
        let add_res = sqlx::query(
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
        .await;
        add_res?;

        tx.commit().await?;

        Ok(amount)
    }
}
