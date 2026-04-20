// Balance Service - Tracks merchant balances across all networks

use crate::config::Config;
use crate::error::ServiceError;
use crate::payment::blockchain_monitor::get_blockchain_monitor;
use crate::payment::models::CryptoType;
use crate::services::price_service::PriceService;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{PgPool, Row};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct Balance {
    pub crypto_type: CryptoType,
    pub total_balance: Decimal,
    pub available_balance: Decimal,
    pub reserved_balance: Decimal,
    pub balance_usd: Decimal,
    pub available_usd: Decimal,
    pub reserved_usd: Decimal,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BalanceSummary {
    pub total_usd: Decimal,
    pub available_usd: Decimal,
    pub reserved_usd: Decimal,
    pub balances: Vec<Balance>,
}

pub struct BalanceService {
    db_pool: PgPool,
    price_service: Arc<PriceService>,
    redis_client: redis::Client,
    config: Arc<Config>,
}

impl BalanceService {
    pub fn new(
        db_pool: PgPool,
        price_service: Arc<PriceService>,
        redis_client: redis::Client,
        config: Arc<Config>,
    ) -> Self {
        Self {
            db_pool,
            price_service,
            redis_client,
            config,
        }
    }

    pub async fn get_balance(
        &self,
        merchant_id: i64,
        crypto_type: CryptoType,
        sandbox_mode: bool,
    ) -> Result<Balance, ServiceError> {
        let balance_record = sqlx::query(
            r#"
            SELECT 
                total_balance,
                available_balance,
                reserved_balance,
                last_updated
            FROM merchant_balances 
            WHERE merchant_id = $1 AND crypto_type = $2 AND sandbox_mode = $3
            "#,
        )
        .bind(merchant_id)
        .bind(crypto_type.to_string())
        .bind(sandbox_mode)
        .fetch_optional(&self.db_pool)
        .await?;

        let total = balance_record
            .as_ref()
            .and_then(|r| {
                r.try_get::<Option<Decimal>, _>("total_balance")
                    .ok()
                    .flatten()
            })
            .unwrap_or(Decimal::ZERO);
        let available = balance_record
            .as_ref()
            .and_then(|r| {
                r.try_get::<Option<Decimal>, _>("available_balance")
                    .ok()
                    .flatten()
            })
            .unwrap_or(Decimal::ZERO);
        let pending = balance_record
            .as_ref()
            .and_then(|r| {
                r.try_get::<Option<Decimal>, _>("reserved_balance")
                    .ok()
                    .flatten()
            })
            .unwrap_or(Decimal::ZERO);
        let last_updated = balance_record
            .as_ref()
            .and_then(|r| {
                r.try_get::<Option<DateTime<Utc>>, _>("last_updated")
                    .ok()
                    .flatten()
            })
            .unwrap_or_else(Utc::now);

        // Get current USD value
        let price: f64 = self
            .price_service
            .get_price(crypto_type)
            .await
            .unwrap_or(0.0);
        let price_decimal = Decimal::from_f64_retain(price).unwrap_or(Decimal::ZERO);

        let available_usd = available * price_decimal;
        let reserved_usd = pending * price_decimal;
        let balance_usd = available_usd;

        Ok(Balance {
            crypto_type,
            total_balance: total,
            available_balance: available,
            reserved_balance: pending,
            balance_usd,
            available_usd,
            reserved_usd,
            last_updated,
        })
    }

    pub async fn get_all_balances(
        &self,
        merchant_id: i64,
        sandbox_mode: bool,
        include_zero: bool,
    ) -> Result<BalanceSummary, ServiceError> {
        tracing::info!(
            "[BALANCE-SERVICE] Fetching balances for merchant={}, sandbox={}, include_zero={}",
            merchant_id,
            sandbox_mode,
            include_zero
        );

        // 1. Fetch configured crypto types for this merchant first to know what MUST be included
        // We include both active and inactive wallets if include_zero is true for better visibility
        let configured_types: Vec<String> = if include_zero {
            sqlx::query_scalar(
                r#"
                SELECT DISTINCT crypto_type FROM merchant_wallets 
                WHERE merchant_id = $1 AND is_active = true 
                AND (sandbox_mode = $2 OR (crypto_type != 'BTC' AND crypto_type != 'BITCOIN'))
                "#,
            )
            .bind(merchant_id)
            .bind(sandbox_mode)
            .fetch_all(&self.db_pool)
            .await?
        } else {
            sqlx::query_scalar(
                "SELECT DISTINCT crypto_type FROM merchant_wallets WHERE merchant_id = $1 AND sandbox_mode = $2 AND is_active = true"
            )
            .bind(merchant_id)
            .bind(sandbox_mode)
            .fetch_all(&self.db_pool)
            .await?
        };

        // 2. Fetch all balance records
        let rows = sqlx::query(
            r#"
            SELECT 
                crypto_type,
                total_balance,
                available_balance,
                reserved_balance,
                last_updated
            FROM merchant_balances 
            WHERE merchant_id = $1 AND sandbox_mode = $2
            "#,
        )
        .bind(merchant_id)
        .bind(sandbox_mode)
        .fetch_all(&self.db_pool)
        .await?;

        // 3. Map rows to crypto_type for easy access
        let mut balance_map: std::collections::HashMap<
            String,
            (Decimal, Decimal, Decimal, DateTime<Utc>),
        > = rows
            .into_iter()
            .map(|r| {
                let ct = r.get::<String, _>("crypto_type");
                let total = r
                    .get::<Option<Decimal>, _>("total_balance")
                    .unwrap_or(Decimal::ZERO);
                let available = r
                    .get::<Option<Decimal>, _>("available_balance")
                    .unwrap_or(Decimal::ZERO);
                let reserved = r
                    .get::<Option<Decimal>, _>("reserved_balance")
                    .unwrap_or(Decimal::ZERO);
                let updated = r.get::<DateTime<Utc>, _>("last_updated");
                (ct, (total, available, reserved, updated))
            })
            .collect();

        // 4. Define the list of types to process
        let mut all_found_types = balance_map
            .keys()
            .cloned()
            .collect::<std::collections::HashSet<_>>();
        for ct in &configured_types {
            all_found_types.insert(ct.clone());
        }

        let mut crypto_types = Vec::new();
        for ct_str in all_found_types {
            if let Ok(ct) = CryptoType::from_string(&ct_str) {
                crypto_types.push(ct);
            }
        }

        // 5. Fetch all prices in parallel once
        let mut tasks = Vec::new();
        for &ct in &crypto_types {
            let service = Arc::new(self.price_service.clone());
            tasks.push(async move {
                let price = service.get_price(ct).await.unwrap_or(0.0);
                (ct, price)
            });
        }
        let price_results = futures::future::join_all(tasks).await;
        let price_map: std::collections::HashMap<CryptoType, f64> =
            price_results.into_iter().collect();

        // 6. Build the summary
        let mut balances = Vec::new();
        let mut total_available_usd = Decimal::ZERO;
        let mut total_reserved_usd = Decimal::ZERO;

        for crypto_type in crypto_types {
            let ct_str = crypto_type.to_string();
            let (total_balance, available_balance, reserved_balance, last_updated) = balance_map
                .remove(&ct_str)
                .unwrap_or((Decimal::ZERO, Decimal::ZERO, Decimal::ZERO, Utc::now()));

            let price = price_map.get(&crypto_type).copied().unwrap_or(0.0);
            let price_dec = Decimal::from_f64_retain(price).unwrap_or(Decimal::ZERO);

            let available_usd = (available_balance * price_dec).round_dp(2);
            let reserved_usd = (reserved_balance * price_dec).round_dp(2);

            total_available_usd += available_usd;
            total_reserved_usd += reserved_usd;

            // Decision: include if balance is non-zero OR if explicitly requested (include_zero)
            if include_zero || total_balance != Decimal::ZERO || available_balance != Decimal::ZERO
            {
                balances.push(Balance {
                    crypto_type,
                    total_balance,
                    available_balance,
                    reserved_balance,
                    balance_usd: available_usd,
                    available_usd,
                    reserved_usd,
                    last_updated,
                });
            }
        }

        // Sort balances by USD value (desc) then by name
        balances.sort_by(|a, b| {
            b.balance_usd
                .cmp(&a.balance_usd)
                .then(a.crypto_type.to_string().cmp(&b.crypto_type.to_string()))
        });

        tracing::info!(
            "[BALANCE-SERVICE] Returning {} balances for merchant {}",
            balances.len(),
            merchant_id
        );

        Ok(BalanceSummary {
            total_usd: total_available_usd + total_reserved_usd,
            available_usd: total_available_usd,
            reserved_usd: total_reserved_usd,
            balances,
        })
    }

    pub async fn update_balance(
        &self,
        merchant_id: i64,
        crypto_type: CryptoType,
        amount_change: Decimal,
        balance_type: &str,
        sandbox_mode: bool,
    ) -> Result<(), ServiceError> {
        // Ensure balance record exists
        self.initialize_balance(merchant_id, crypto_type, sandbox_mode)
            .await?;

        match balance_type {
            "available" => {
                sqlx::query(
                    r#"
                    UPDATE merchant_balances 
                    SET available_balance = available_balance + $1,
                        last_updated = $2
                    WHERE merchant_id = $3 AND crypto_type = $4 AND sandbox_mode = $5
                    "#,
                )
                .bind(amount_change)
                .bind(Utc::now())
                .bind(merchant_id)
                .bind(crypto_type.to_string())
                .bind(sandbox_mode)
                .execute(&self.db_pool)
                .await?;
            }
            "pending" => {
                sqlx::query(
                    r#"
                    UPDATE merchant_balances 
                    SET reserved_balance = reserved_balance + $1,
                        last_updated = $2
                    WHERE merchant_id = $3 AND crypto_type = $4 AND sandbox_mode = $5
                    "#,
                )
                .bind(amount_change)
                .bind(Utc::now())
                .bind(merchant_id)
                .bind(crypto_type.to_string())
                .bind(sandbox_mode)
                .execute(&self.db_pool)
                .await?;
            }
            "total" => {
                sqlx::query(
                    r#"
                    UPDATE merchant_balances 
                    SET last_updated = $1
                    WHERE merchant_id = $2 AND crypto_type = $3 AND sandbox_mode = $4
                    "#,
                )
                .bind(Utc::now())
                .bind(merchant_id)
                .bind(crypto_type.to_string())
                .bind(sandbox_mode)
                .execute(&self.db_pool)
                .await?;
            }
            _ => {
                return Err(ServiceError::ValidationError(
                    "Invalid balance type".to_string(),
                ));
            }
        }

        Ok(())
    }

    pub async fn move_pending_to_available(
        &self,
        merchant_id: i64,
        crypto_type: CryptoType,
        amount: Decimal,
        sandbox_mode: bool,
    ) -> Result<(), ServiceError> {
        sqlx::query(
            r#"
            UPDATE merchant_balances 
            SET reserved_balance = reserved_balance - $1,
                available_balance = available_balance + $1,
                last_updated = $2
            WHERE merchant_id = $3 AND crypto_type = $4 AND sandbox_mode = $5
            "#,
        )
        .bind(amount)
        .bind(Utc::now())
        .bind(merchant_id)
        .bind(crypto_type.to_string())
        .bind(sandbox_mode)
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    async fn initialize_balance(
        &self,
        merchant_id: i64,
        crypto_type: CryptoType,
        sandbox_mode: bool,
    ) -> Result<(), ServiceError> {
        sqlx::query(
            r#"
            INSERT INTO merchant_balances (merchant_id, crypto_type, available_balance, reserved_balance, last_updated, sandbox_mode)
            VALUES ($1, $2, 0, 0, $3, $4)
            ON CONFLICT (merchant_id, crypto_type, sandbox_mode) DO NOTHING
            "#
        )
        .bind(merchant_id)
        .bind(crypto_type.to_string())
        .bind(Utc::now())
        .bind(sandbox_mode)
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    pub async fn refresh_balances_from_blockchain(
        &self,
        merchant_id: i64,
        sandbox_mode: bool,
    ) -> Result<(), ServiceError> {
        let crypto_types = vec![
            CryptoType::Sol,
            CryptoType::UsdtSpl,
            CryptoType::Eth,
            CryptoType::UsdtEth,
            CryptoType::Bnb,
            CryptoType::UsdtBep20,
            CryptoType::Matic,
            CryptoType::UsdtPolygon,
            CryptoType::Arb,
            CryptoType::UsdtArbitrum,
        ];

        for crypto_type in crypto_types {
            // Calculate balance from confirmed payments
            let balance_data = sqlx::query(
                r#"
                SELECT 
                    COALESCE(SUM(CASE WHEN status = 'CONFIRMED' THEN amount - fee_amount ELSE 0 END), 0) as confirmed_total,
                    COALESCE(SUM(CASE WHEN status = 'PENDING' THEN amount - fee_amount ELSE 0 END), 0) as pending_total
                FROM payment_transactions 
                WHERE merchant_id = $1 AND crypto_type = $2 AND sandbox_mode = $3
                "#
            )
            .bind(merchant_id)
            .bind(crypto_type.to_string())
            .bind(sandbox_mode)
            .fetch_one(&self.db_pool)
            .await?;

            // Calculate total refunds
            let refund_data = sqlx::query(
                r#"
                SELECT COALESCE(SUM(amount), 0) as total_refunded
                FROM refunds 
                WHERE merchant_id = $1 AND status = 'COMPLETED' AND sandbox_mode = $2
                AND payment_id IN (SELECT id FROM payment_transactions WHERE crypto_type = $3)
                "#,
            )
            .bind(merchant_id)
            .bind(sandbox_mode)
            .bind(crypto_type.to_string())
            .fetch_one(&self.db_pool)
            .await?;

            // Calculate withdrawals
            let withdrawal_data = sqlx::query(
                r#"
                SELECT COALESCE(SUM(amount), 0) as total_withdrawn
                FROM withdrawals 
                WHERE merchant_id = $1 AND crypto_type = $2 AND status = 'COMPLETED' AND sandbox_mode = $3
                "#
            )
            .bind(merchant_id)
            .bind(crypto_type.to_string())
            .bind(sandbox_mode)
            .fetch_one(&self.db_pool)
            .await?;

            let confirmed_balance: Decimal = balance_data
                .try_get::<Option<Decimal>, _>("confirmed_total")
                .ok()
                .flatten()
                .unwrap_or(Decimal::ZERO);
            let reserved_balance: Decimal = balance_data
                .try_get::<Option<Decimal>, _>("pending_total")
                .ok()
                .flatten()
                .unwrap_or(Decimal::ZERO);
            let withdrawn: Decimal = withdrawal_data
                .try_get::<Option<Decimal>, _>("total_withdrawn")
                .ok()
                .flatten()
                .unwrap_or(Decimal::ZERO);
            let refunded: Decimal = refund_data
                .try_get::<Option<Decimal>, _>("total_refunded")
                .ok()
                .flatten()
                .unwrap_or(Decimal::ZERO);

            let available_balance = confirmed_balance - withdrawn - refunded;
            let _total_balance = available_balance + reserved_balance;

            // Update balance record
            sqlx::query(
                r#"
                INSERT INTO merchant_balances (merchant_id, crypto_type, available_balance, reserved_balance, last_updated, sandbox_mode)
                VALUES ($1, $2, $3, $4, $5, $6)
                ON CONFLICT (merchant_id, crypto_type, sandbox_mode) 
                DO UPDATE SET 
                    available_balance = $3,
                    reserved_balance = $4,
                    last_updated = $5
                "#
            )
            .bind(merchant_id)
            .bind(crypto_type.to_string())
            .bind(available_balance)
            .bind(reserved_balance)
            .bind(Utc::now())
            .bind(sandbox_mode)
            .execute(&self.db_pool)
            .await?;
        }

        // 4. Push real-time update to dashboard
        let _ = self
            .broadcast_balance_update(merchant_id, sandbox_mode)
            .await;

        Ok(())
    }

    pub async fn rectify_onchain(
        &self,
        address: &str,
        crypto_type: CryptoType,
        dry_run: bool,
        signature_limit: usize,
        override_sandbox_mode: Option<bool>,
        rectify_type: &str,
    ) -> Result<serde_json::Value, ServiceError> {
        tracing::info!(
            "[RECTIFY-SMART] Rectifying address {} ({}, dry_run={}, override={:?}, type={})",
            address,
            crypto_type.to_string(),
            dry_run,
            override_sandbox_mode,
            rectify_type
        );

        // 1. Identify Merchant & Sandbox Mode
        let wallet_info = sqlx::query(
            r#"
            SELECT merchant_id, sandbox_mode, 'customer' as owner_type, customer_id 
            FROM merchant_customer_wallets WHERE address = $1 AND crypto_type = $2
            UNION ALL
            SELECT merchant_id, sandbox_mode, 'merchant' as owner_type, NULL as customer_id 
            FROM merchant_wallets WHERE address = $1 AND crypto_type = $2
            "#,
        )
        .bind(address)
        .bind(crypto_type.to_string())
        .fetch_optional(&self.db_pool)
        .await?;

        let row = wallet_info.ok_or_else(|| {
            ServiceError::ValidationError("Address not found in system wallets".to_string())
        })?;
        let merchant_id: i64 = row.get("merchant_id");
        let db_sandbox_mode: bool = row.get("sandbox_mode");
        let owner_type: String = row.get("owner_type");
        let customer_id: Option<i64> = row.get("customer_id");

        let active_sandbox_mode = override_sandbox_mode.unwrap_or(db_sandbox_mode);

        // 2. Setup Monitor
        let monitor =
            get_blockchain_monitor(&crypto_type, (*self.config).clone(), active_sandbox_mode);

        // 3. Fetch On-chain Transactions
        let onchain_txs = monitor
            .get_transactions_to_address(address, signature_limit, None)
            .await
            .map_err(|e| ServiceError::Internal(format!("Blockchain scan failed: {}", e)))?;

        // 4. Fetch Existing Hashes and their statuses/amounts for RECONCILIATION
        // NARROWED SCOPE: Filter by crypto_type and sandbox_mode to prevent cross-mode collision
        let crypto_type_str = crypto_type.to_string();
        let existing_data_rows = sqlx::query(
            r#"
            SELECT transaction_hash, amount, status, 'payment' as source FROM payment_transactions 
            WHERE merchant_id = $1 AND crypto_type = $2 AND sandbox_mode = $3 AND transaction_hash IS NOT NULL
            UNION ALL
            SELECT transaction_hash, amount, status, 'customer_tx' as source FROM customer_transactions 
            WHERE merchant_id = $1 AND crypto_type = $2 AND sandbox_mode = $3 AND transaction_hash IS NOT NULL
            UNION ALL
            SELECT transaction_hash, amount, status, 'withdrawal' as source FROM withdrawals 
            WHERE merchant_id = $1 AND crypto_type = $2 AND sandbox_mode = $3 AND transaction_hash IS NOT NULL
            "#
        )
        .bind(merchant_id)
        .bind(&crypto_type_str)
        .bind(active_sandbox_mode)
        .fetch_all(&self.db_pool)
        .await?;

        let mut existing_txs: std::collections::HashMap<String, (Decimal, String)> =
            std::collections::HashMap::new();
        let mut db_confirmed_deposits = Decimal::ZERO;
        let mut db_confirmed_withdrawals = Decimal::ZERO;

        for r in existing_data_rows {
            if let Some(hash) = r.get::<Option<String>, _>("transaction_hash") {
                let amount = r.get::<Decimal, _>("amount");
                let status = r.get::<String, _>("status");
                let source: String = r.get("source");

                existing_txs.insert(hash, (amount, status.clone()));

                if status == "CONFIRMED" || status == "COMPLETED" {
                    if source == "payment" || source == "customer_tx" {
                        db_confirmed_deposits += amount;
                    } else if source == "withdrawal" {
                        db_confirmed_withdrawals += amount;
                    }
                }
            }
        }

        // 4.1 Fetch Merchant Sub-wallets for Sweep Identification
        let sub_wallets: std::collections::HashSet<String> = sqlx::query_scalar(
            "SELECT LOWER(address) FROM merchant_customer_wallets WHERE merchant_id = $1 AND crypto_type = $2"
        )
        .bind(merchant_id)
        .bind(&crypto_type_str)
        .fetch_all(&self.db_pool)
        .await?
        .into_iter()
        .collect();

        // 5. Identify Gaps & Discrepancies
        let mut missing_deposits = Vec::new();
        let mut missing_withdrawals = Vec::new();
        let mut mismatched_txs = Vec::new();
        let mut total_onchain_deposits = Decimal::ZERO;
        let mut total_onchain_withdrawals = Decimal::ZERO;
        let mut skipped_sweeps = 0;

        for tx in onchain_txs {
            // Drop failed or zero-amount
            if !tx.success || tx.amount <= Decimal::ZERO {
                continue;
            }

            // Verify crypto-specific traits (tokens etc)
            if let Some(expected_token) = crypto_type.token_address() {
                if tx.token_mint.as_deref().unwrap_or("").to_lowercase()
                    != expected_token.to_lowercase()
                {
                    continue;
                }
            } else if tx.token_mint.is_some() {
                continue; // Expected native, got token
            }

            let is_incoming = tx.to_address.to_lowercase() == address.to_lowercase();
            let is_outgoing = tx.from_address.to_lowercase() == address.to_lowercase();

            if is_incoming {
                total_onchain_deposits += tx.amount;
            } else if is_outgoing {
                total_onchain_withdrawals += tx.amount;
            }

            if let Some((db_amount, db_status)) = existing_txs.get(&tx.hash) {
                // IT EXISTS - Check for discrepancy
                if (tx.amount - *db_amount).abs() > Decimal::new(1, 8)
                    || db_status == "PENDING"
                    || db_status == "FAILED"
                {
                    mismatched_txs.push(json!({
                        "hash": tx.hash,
                        "onchain_amount": tx.amount,
                        "db_amount": db_amount,
                        "db_status": db_status,
                        "action": "CORRECTION_NEEDED"
                    }));
                }
                continue;
            }

            // --- GAP DETECTION ---

            // Check if this is an internal sweep (should not have fees)
            let is_internal_sweep = sub_wallets.contains(&tx.from_address.to_lowercase());

            if is_incoming && (rectify_type == "DEPOSIT" || rectify_type == "BOTH") {
                let mut tx_with_meta = tx.clone();
                if is_internal_sweep {
                    // Tag it as internal sweep so we don't take fees later
                    tx_with_meta.hash = format!("SWEEP:{}", tx_with_meta.hash);
                    skipped_sweeps += 1;
                }
                missing_deposits.push(tx_with_meta);
            }

            if is_outgoing && (rectify_type == "WITHDRAWAL" || rectify_type == "BOTH") {
                missing_withdrawals.push(tx);
            }
        }

        // 6. Fetch Fee Logic & Price
        let fee_percentage =
            sqlx::query_scalar::<_, Decimal>("SELECT fee_percentage FROM merchants WHERE id = $1")
                .bind(merchant_id)
                .fetch_one(&self.db_pool)
                .await?;

        let crypto_price = self
            .price_service
            .get_price(crypto_type)
            .await
            .unwrap_or(1.0);
        let price_dec = Decimal::from_f64_retain(crypto_price).unwrap_or(Decimal::ONE);

        // 7. Execute Fixes (If not dry run)
        let mut actions_taken = Vec::new();
        if !dry_run && (!missing_deposits.is_empty() || !missing_withdrawals.is_empty()) {
            let mut db_tx = self.db_pool.begin().await?;

            // Process Missing Deposits
            for tx in &missing_deposits {
                let is_sweep = tx.hash.starts_with("SWEEP:");
                let clean_hash = if is_sweep { &tx.hash[6..] } else { &tx.hash };

                // Apply logic: INTERNAL SWEEPS HAVE 0 FEES
                let effective_fee_pct = if is_sweep {
                    Decimal::ZERO
                } else {
                    fee_percentage
                };

                let fee_amount = (tx.amount * (effective_fee_pct / Decimal::from(100))).round_dp(8);
                let net_amount = tx.amount - fee_amount;
                let amount_usd = (tx.amount * price_dec).round_dp(2);
                let fee_usd = (amount_usd * (effective_fee_pct / Decimal::from(100))).round_dp(2);

                if owner_type == "customer" {
                    let cid = customer_id.unwrap();
                    sqlx::query(
                        r#"
                        INSERT INTO merchant_customer_balances (customer_id, merchant_id, crypto_type, available_balance, total_balance, last_updated_at, sandbox_mode)
                        VALUES ($1, $2, $3, $4, $4, NOW(), $5)
                        ON CONFLICT (customer_id, crypto_type, sandbox_mode)
                        DO UPDATE SET 
                            available_balance = merchant_customer_balances.available_balance + $4,
                            total_balance = merchant_customer_balances.total_balance + $4,
                            last_updated_at = NOW()
                        "#
                    ).bind(cid).bind(merchant_id).bind(&crypto_type_str).bind(net_amount).bind(active_sandbox_mode).execute(&mut *db_tx).await?;

                    sqlx::query(
                        r#"
                        INSERT INTO customer_transactions (
                            customer_id, merchant_id, type, crypto_type, amount, amount_usd, fee, status, 
                            transaction_hash, description, created_at, sandbox_mode
                        ) VALUES ($1, $2, 'DEPOSIT_RECTIFICATION', $3, $4, $5, $6, 'CONFIRMED', $7, $8, NOW(), $9)
                        "#
                    ).bind(cid).bind(merchant_id).bind(&crypto_type_str).bind(tx.amount).bind(amount_usd).bind(fee_usd).bind(clean_hash)
                    .bind(format!("Smart Rectification Inbound: {}", clean_hash)).bind(active_sandbox_mode).execute(&mut *db_tx).await?;
                } else {
                    sqlx::query(
                        r#"
                        INSERT INTO merchant_balances (merchant_id, crypto_type, available_balance, reserved_balance, last_updated, sandbox_mode)
                        VALUES ($1, $2, $3, 0, NOW(), $4)
                        ON CONFLICT (merchant_id, crypto_type, sandbox_mode)
                        DO UPDATE SET available_balance = merchant_balances.available_balance + $3, last_updated = NOW()
                        "#
                    ).bind(merchant_id).bind(&crypto_type_str).bind(net_amount).bind(active_sandbox_mode).execute(&mut *db_tx).await?;

                    let public_id = format!("rect_{}", &clean_hash[..8]);
                    sqlx::query(
                        r#"
                        INSERT INTO payment_transactions (
                            payment_id, merchant_id, amount, amount_usd, crypto_type, status, to_address, 
                            transaction_hash, description, created_at, sandbox_mode, fee_percentage, fee_amount, fee_amount_usd,
                            network, expires_at, required_confirmations, confirmations
                        ) VALUES ($1, $2, $3, $4, $5, 'CONFIRMED', $6, $7, $8, NOW(), $9, $10, $11, $12, $13, NOW() + INTERVAL '1 hour', 1, 1)
                        "#
                    ).bind(public_id).bind(merchant_id).bind(tx.amount).bind(amount_usd).bind(&crypto_type_str).bind(address).bind(clean_hash)
                    .bind(if is_sweep { format!("Internal Sweep (No Fee): {}", clean_hash) } else { format!("Smart Rectification: {}", clean_hash) })
                    .bind(active_sandbox_mode).bind(effective_fee_pct).bind(fee_amount).bind(fee_usd)
                    .bind(crypto_type.network()).execute(&mut *db_tx).await?;
                }
                actions_taken.push(if is_sweep {
                    format!("Synced Sweep (Fee Free): {}", clean_hash)
                } else {
                    format!("Imported Deposit: {}", clean_hash)
                });
            }

            // Process Missing Withdrawals (Simply debit balance and record)
            for tx in &missing_withdrawals {
                let amount_usd = (tx.amount * price_dec).round_dp(2);
                let withdrawal_id = format!(
                    "wrect_{}_{}",
                    &tx.hash[..8],
                    chrono::Utc::now().timestamp_millis()
                );

                if owner_type == "customer" {
                    sqlx::query("UPDATE merchant_customer_balances SET available_balance = available_balance - $1, total_balance = total_balance - $1, last_updated_at = NOW() WHERE customer_id = $2 AND crypto_type = $3 AND sandbox_mode = $4")
                        .bind(tx.amount).bind(customer_id.unwrap()).bind(&crypto_type_str).bind(active_sandbox_mode).execute(&mut *db_tx).await?;
                } else {
                    sqlx::query("UPDATE merchant_balances SET available_balance = available_balance - $1, last_updated = NOW() WHERE merchant_id = $2 AND crypto_type = $3 AND sandbox_mode = $4")
                        .bind(tx.amount).bind(merchant_id).bind(&crypto_type_str).bind(active_sandbox_mode).execute(&mut *db_tx).await?;
                }

                // Record the withdrawal in history
                sqlx::query(
                    r#"
                    INSERT INTO withdrawals (
                        withdrawal_id, merchant_id, crypto_type, amount, amount_usd, destination_address,
                        status, fee, net_amount, created_at, updated_at, sandbox_mode, transaction_hash
                    )
                    VALUES ($1, $2, $3, $4, $5, $6, 'COMPLETED', 0, $4, NOW(), NOW(), $7, $8)
                    "#
                )
                .bind(&withdrawal_id)
                .bind(merchant_id)
                .bind(&crypto_type_str)
                .bind(tx.amount)
                .bind(amount_usd)
                .bind(&tx.to_address)
                .bind(active_sandbox_mode)
                .bind(&tx.hash)
                .execute(&mut *db_tx)
                .await?;

                actions_taken.push(format!("Synced Outbound Transfer: {}", tx.hash));
            }

            db_tx.commit().await?;
        }

        // 8. FINAL RECONCILIATION & SYNC
        // We recalculate the expected balance one more time after potential imports
        let updated_recorded_data = sqlx::query(
            r#"
            SELECT 
                COALESCE(SUM(CASE WHEN source IN ('payment', 'customer_tx') THEN amount - fee_amount ELSE 0 END), 0) as total_deposits,
                COALESCE(SUM(CASE WHEN source = 'withdrawal' THEN amount ELSE 0 END), 0) as total_withdrawals
            FROM (
                SELECT amount, fee_amount, 'payment' as source FROM payment_transactions WHERE merchant_id = $1 AND crypto_type = $2 AND sandbox_mode = $3 AND status = 'CONFIRMED'
                UNION ALL
                SELECT amount, fee as fee_amount, 'customer_tx' as source FROM customer_transactions WHERE merchant_id = $1 AND crypto_type = $2 AND sandbox_mode = $3 AND status = 'CONFIRMED'
                UNION ALL
                SELECT amount, 0 as fee_amount, 'withdrawal' as source FROM withdrawals WHERE merchant_id = $1 AND crypto_type = $2 AND sandbox_mode = $3 AND status = 'COMPLETED'
            ) as combined_history
            "#
        )
        .bind(merchant_id)
        .bind(&crypto_type_str)
        .bind(active_sandbox_mode)
        .fetch_one(&self.db_pool)
        .await?;

        let audited_deposits: Decimal = updated_recorded_data.get("total_deposits");
        let audited_withdrawals: Decimal = updated_recorded_data.get("total_withdrawals");
        let audited_expected_balance = audited_deposits - audited_withdrawals;

        // If not dry run, FORCE SYNC the balance table to this audited amount
        if !dry_run {
            tracing::info!(
                "[RECTIFY-SMART] Force syncing balance for merchant {} ({}): recorded_sum={} onchain_sum={}",
                merchant_id,
                crypto_type_str,
                audited_expected_balance,
                total_onchain_deposits - total_onchain_withdrawals
            );

            sqlx::query(
                r#"
                INSERT INTO merchant_balances (merchant_id, crypto_type, available_balance, total_balance, reserved_balance, last_updated, sandbox_mode)
                VALUES ($1, $2, $3, $3, 0, NOW(), $4)
                ON CONFLICT (merchant_id, crypto_type, sandbox_mode)
                DO UPDATE SET 
                    available_balance = $3,
                    total_balance = $3 + merchant_balances.reserved_balance,
                    last_updated = NOW()
                "#
            )
            .bind(merchant_id)
            .bind(&crypto_type_str)
            .bind(audited_expected_balance)
            .bind(active_sandbox_mode)
            .execute(&self.db_pool)
            .await?;
        }

        let current_balance_after = self
            .get_balance(merchant_id, crypto_type, active_sandbox_mode)
            .await?
            .available_balance;

        Ok(json!({
            "success": true,
            "dry_run": dry_run,
            "merchant_id": merchant_id,
            "blockchain": monitor.blockchain_name(),
            "mode": if active_sandbox_mode { "SANDBOX" } else { "LIVE" },
            "audit": {
                "onchain_deposits": total_onchain_deposits,
                "onchain_withdrawals": total_onchain_withdrawals,
                "recorded_db_deposits_net": audited_deposits,
                "recorded_db_withdrawals": audited_withdrawals,
                "internal_sweeps_detected": skipped_sweeps
            },
            "reconciliation": {
                "current_table_balance": current_balance_after,
                "expected_from_recorded_txs": audited_expected_balance,
                "balance_gap": audited_expected_balance - current_balance_after,
                "status": if (audited_expected_balance - current_balance_after).abs() < Decimal::new(1, 8) { "HEALTHY" } else { "OUT_OF_SYNC" }
            },
            "gap_analysis": {
                "missing_deposits": missing_deposits.len(),
                "missing_withdrawals": missing_withdrawals.len(),
                "discrepancies_found": mismatched_txs.len(),
            },
            "mismatched_details": mismatched_txs,
            "actions_taken": actions_taken,
        }))
    }

    pub async fn broadcast_balance_update(
        &self,
        merchant_id: i64,
        sandbox_mode: bool,
    ) -> Result<(), ServiceError> {
        let summary = self
            .get_all_balances(merchant_id, sandbox_mode, true)
            .await?;

        if let Ok(mut publish_conn) = self.redis_client.get_multiplexed_async_connection().await {
            let notification = serde_json::json!({
                "event": "merchant.balance_updated",
                "data": summary
            });
            let channel = format!("merchant_notifications:{}", merchant_id);
            let _: redis::RedisResult<()> = redis::cmd("PUBLISH")
                .arg(&channel)
                .arg(notification.to_string())
                .query_async(&mut publish_conn)
                .await;
        }

        Ok(())
    }
}
