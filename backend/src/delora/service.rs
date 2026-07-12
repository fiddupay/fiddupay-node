// Delora Service
// Business logic: quotes, registration, status tracking

use crate::config::DeloraConfig;
use crate::delora::cache::DeloraCache;
use crate::delora::client::{DeloraClient, DeloraMetricsSummary};
use crate::delora::constants::{chain_name, QUOTE_EXPIRY_SECONDS, REGISTRATION_GRACE_SECONDS};
use crate::delora::error::DeloraError;
use crate::delora::models::*;
use crate::delora::validation;
use chrono::Utc;
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::str::FromStr;
use std::sync::Arc;
use tracing::{debug, warn};
use uuid::Uuid;

/// Lookup result from payment_links for the service layer
#[derive(Debug)]
pub struct PaymentLinkInfo {
    pub payment_id: i64,
    pub merchant_id: i64,
    pub invoice_id: Option<Uuid>,
    pub amount: Option<rust_decimal::Decimal>,
    pub amount_usd: Option<rust_decimal::Decimal>,
    pub crypto_type: Option<String>,
    pub to_address: Option<String>,
    pub sandbox_mode: bool,
}

pub struct DeloraService {
    client: Arc<DeloraClient>,
    cache: Arc<DeloraCache>,
    db: PgPool,
    config: DeloraConfig,
}

impl DeloraService {
    pub fn new(
        client: Arc<DeloraClient>,
        cache: Arc<DeloraCache>,
        db: PgPool,
        config: DeloraConfig,
    ) -> Self {
        Self {
            client,
            cache,
            db,
            config,
        }
    }

    // ── Public API ────────────────────────────────────────────────────────

    /// Get a cross-chain quote for paying an invoice with a non-standard token/chain.
    pub async fn get_cross_chain_quote(
        &self,
        link_id: &str,
        sender_address: &str,
        origin_chain_id: u64,
        origin_currency_address: &str,
    ) -> Result<CrossChainQuoteResponse, DeloraError> {
        // 1. Lookup payment link
        let payment = self.lookup_payment_link(link_id).await?;

        // 2. Resolve destination (merchant target)
        let (dest_chain_id, dest_currency_address, merchant_address) =
            self.resolve_destination(&payment).await?;

        // 3. Determine quote amount — use invoice amount or minimum
        let amount = self
            .resolve_quote_amount(&payment, dest_chain_id, &dest_currency_address)
            .await?;

        // 4. If same-chain + same-currency → skip Delora
        if origin_chain_id == dest_chain_id
            && origin_currency_address.to_lowercase() == dest_currency_address.to_lowercase()
        {
            return Err(DeloraError::Config(
                "Same-chain same-currency payment — use direct payment flow".into(),
            ));
        }

        // 5. Resolve token metadata for display
        let (origin_meta, dest_meta) = tokio::try_join!(
            self.resolve_token_summary(origin_chain_id, origin_currency_address),
            self.resolve_token_summary(dest_chain_id, &dest_currency_address),
        )?;

        // 6. Get quote from Delora (simple first, advanced fallback)
        let quote_id = Uuid::new_v4();
        let quote_req = QuoteRequest {
            sender_address: sender_address.to_string(),
            receiver_address: merchant_address.clone(),
            origin_chain_id,
            destination_chain_id: dest_chain_id,
            amount: amount.clone(),
            origin_currency: origin_currency_address.to_string(),
            destination_currency: dest_currency_address.clone(),
            integrator: self.config.integrator_id.clone(),
            fee: self.config.default_integrator_fee,
            slippage: Some(self.config.default_slippage),
            include_bridges: None,
            exclude_bridges: None,
            include_exchanges: None,
            exclude_exchanges: None,
        };

        let (quote, route_summary) = match self.client.get_quote(&quote_req).await {
            Ok(q) => {
                debug!("Simple quote succeeded for quote_id={}", quote_id);
                (q, None)
            }
            Err(e) => {
                warn!("Simple quote failed, trying advanced: {}", e);
                self.get_advanced_quote_fallback(&quote_req).await?
            }
        };

        // 7. Validate calldata
        validation::validate_quote_calldata(&quote.calldata, origin_chain_id)?;

        // 8. Store quote snapshot in cache
        self.cache.store_quote_snapshot(&quote_id, &quote).await?;

        // 9. Persist cross_chain_payment row in DB
        self.persist_cross_chain_payment(
            &quote_id,
            &payment,
            &origin_meta,
            &dest_meta,
            &merchant_address,
            &quote,
            sender_address,
            &route_summary,
        )
        .await?;

        // 10. Build response
        let expires_at = Utc::now() + chrono::Duration::seconds(QUOTE_EXPIRY_SECONDS);

        Ok(CrossChainQuoteResponse {
            quote_id: quote_id.to_string(),
            origin_chain_id,
            destination_chain_id: dest_chain_id,
            origin_currency: origin_meta,
            destination_currency: dest_meta,
            input_amount: quote.input_amount.clone(),
            input_amount_display: quote.input_amount.clone(),
            output_amount: quote.output_amount.clone(),
            output_amount_display: quote.output_amount.clone(),
            min_output_amount: quote.min_output_amount.clone().unwrap_or_default(),
            fees: self.build_fee_breakdown(&quote),
            estimated_time_sec: quote.estimated_time_sec,
            calldata: quote.calldata.clone(),
            gas: quote.gas.clone(),
            approval_address: quote.approval_address.clone(),
            warnings: quote.warnings.clone(),
            expires_at: expires_at.to_rfc3339(),
            route: route_summary,
        })
    }

    /// Register a transaction hash after the customer signs and broadcasts.
    pub async fn register_cross_chain_tx(
        &self,
        quote_id: &Uuid,
        tx_hash: &str,
        sender_address: &str,
    ) -> Result<CrossChainStatusResponse, DeloraError> {
        // Validate tx_hash format
        if !validation::validate_tx_hash(tx_hash) {
            return Err(DeloraError::InvalidCalldata(
                "Invalid transaction hash format".into(),
            ));
        }

        // 1. Lookup with FOR UPDATE lock
        let mut tx = self.db.begin().await?;

        let cc_payment = sqlx::query_as::<_, CrossChainPaymentRow>(
            r#"SELECT * FROM cross_chain_payments 
               WHERE quote_id = $1 AND deleted_at IS NULL 
               FOR UPDATE"#,
        )
        .bind(quote_id)
        .fetch_optional(&mut *tx)
        .await?;

        let cc_payment = cc_payment.ok_or(DeloraError::QuoteNotFound(*quote_id))?;

        // 2. Idempotency — already registered?
        if matches!(
            cc_payment.status,
            CrossChainPaymentStatus::TxSubmitted
                | CrossChainPaymentStatus::TxConfirmed
                | CrossChainPaymentStatus::BridgePending
                | CrossChainPaymentStatus::BridgeComplete
                | CrossChainPaymentStatus::Completed
        ) {
            tx.rollback().await?;
            return Ok(self.build_status_response(&cc_payment));
        }

        // 3. Check quote expiry with grace period
        let grace_deadline =
            cc_payment.quote_expires_at + chrono::Duration::seconds(REGISTRATION_GRACE_SECONDS);
        if Utc::now() > grace_deadline {
            sqlx::query(
                r#"UPDATE cross_chain_payments 
                   SET status = 'expired', failed_reason = 'Quote expired before registration', updated_at = NOW()
                   WHERE id = $1"#,
            )
            .bind(cc_payment.id)
            .execute(&mut *tx)
            .await?;
            tx.commit().await?;
            return Err(DeloraError::QuoteExpired);
        }

        // 4. Verify sender_address matches (if already set)
        if let Some(ref stored_sender) = cc_payment.sender_address {
            if !stored_sender.is_empty()
                && stored_sender.to_lowercase() != sender_address.to_lowercase()
            {
                tx.rollback().await?;
                return Err(DeloraError::SenderMismatch);
            }
        }

        // 5. Check origin_tx_hash uniqueness (with composite chain_id)
        let existing = sqlx::query_scalar::<_, bool>(
            r#"SELECT EXISTS(
                SELECT 1 FROM cross_chain_payments 
                WHERE origin_tx_hash = $1 AND origin_chain_id = $2 
                AND id != $3 AND deleted_at IS NULL
            )"#,
        )
        .bind(tx_hash)
        .bind(cc_payment.origin_chain_id)
        .bind(cc_payment.id)
        .fetch_one(&mut *tx)
        .await?;

        if existing {
            tx.rollback().await?;
            return Err(DeloraError::TransactionAlreadyRegistered);
        }

        // 6. Update to tx_submitted
        sqlx::query(
            r#"UPDATE cross_chain_payments 
               SET status = 'tx_submitted', origin_tx_hash = $1, sender_address = $2,
                   tx_submitted_at = NOW(), updated_at = NOW()
               WHERE id = $3"#,
        )
        .bind(tx_hash)
        .bind(sender_address)
        .bind(cc_payment.id)
        .execute(&mut *tx)
        .await?;

        // 7. Link to payment_transactions if present
        if let Some(pt_id) = cc_payment.payment_transaction_id {
            sqlx::query(
                r#"UPDATE payment_transactions 
                   SET status = 'CONFIRMING', transaction_hash = $1, last_verification_at = NOW()
                   WHERE id = $2 AND status IN ('PENDING', 'SELECTION_REQUIRED')"#,
            )
            .bind(tx_hash)
            .bind(pt_id)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        // Side effects can go here (Redis pub/sub)

        let updated = self.get_cross_chain_payment(quote_id).await?;
        Ok(self.build_status_response(&updated))
    }

    /// Get real-time status of a cross-chain payment.
    pub async fn get_cross_chain_status(
        &self,
        link_id: &str,
    ) -> Result<CrossChainStatusResponse, DeloraError> {
        let row = sqlx::query_as::<_, CrossChainPaymentRow>(
            r#"SELECT ccp.* FROM cross_chain_payments ccp
               JOIN payment_links pl ON pl.payment_id = ccp.payment_transaction_id
               WHERE pl.link_id = $1 AND ccp.deleted_at IS NULL
               ORDER BY ccp.created_at DESC LIMIT 1"#,
        )
        .bind(link_id)
        .fetch_optional(&self.db)
        .await?;

        let row = row.ok_or(DeloraError::PaymentLinkNotFound(link_id.into()))?;
        Ok(self.build_status_response(&row))
    }

    /// Get supported chains (cached).
    pub async fn get_supported_chains(&self) -> Result<Vec<ChainSummary>, DeloraError> {
        let client = self.client.clone();
        let chains = self
            .cache
            .get_chains(move || {
                let c = client.clone();
                async move { c.get_chains().await }
            })
            .await?;

        Ok(chains
            .into_iter()
            .map(|c| ChainSummary {
                chain_id: c.chain_id,
                name: c.name,
                native_symbol: c.native_currency.map(|n| n.symbol).unwrap_or_default(),
                enabled: true,
            })
            .collect())
    }

    /// Get supported tokens for a chain (cached).
    pub async fn get_supported_tokens(
        &self,
        chain_id: u64,
    ) -> Result<Vec<TokenSummary>, DeloraError> {
        let client = self.client.clone();
        let all_tokens = self
            .cache
            .get_tokens(move || {
                let c = client.clone();
                async move { c.get_tokens().await }
            })
            .await?;

        let chain_key = chain_id.to_string();
        Ok(all_tokens
            .tokens
            .get(&chain_key)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .map(|t| TokenSummary {
                address: t.address,
                symbol: t.symbol,
                name: t.name,
                decimals: t.decimals,
                chain_id: t.chain_id,
            })
            .collect())
    }

    /// Get Delora client metrics for health monitoring.
    pub fn metrics(&self) -> DeloraMetricsSummary {
        self.client.metrics_summary()
    }

    // ── Internal ───────────────────────────────────────────────────────────

    async fn lookup_payment_link(&self, link_id: &str) -> Result<PaymentLinkInfo, DeloraError> {
        let row = sqlx::query_as::<
            _,
            (
                i64,
                i64,
                Option<Uuid>,
                Option<Decimal>,
                Option<Decimal>,
                Option<String>,
                Option<String>,
                bool,
            ),
        >(
            r#"SELECT pt.id, pt.merchant_id, pt.invoice_id, pt.amount, pt.amount_usd,
                      pt.crypto_type::text, pt.to_address, COALESCE(pt.sandbox_mode, false)
               FROM payment_transactions pt
               JOIN payment_links pl ON pl.payment_id = pt.id
               WHERE pl.link_id = $1"#,
        )
        .bind(link_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or(DeloraError::PaymentLinkNotFound(link_id.into()))?;

        Ok(PaymentLinkInfo {
            payment_id: row.0,
            merchant_id: row.1,
            invoice_id: row.2,
            amount: row.3,
            amount_usd: row.4,
            crypto_type: row.5,
            to_address: row.6,
            sandbox_mode: row.7,
        })
    }

    async fn resolve_destination(
        &self,
        payment: &PaymentLinkInfo,
    ) -> Result<(u64, String, String), DeloraError> {
        // Get the merchant's wallet for the configured crypto_type
        let crypto_type_str = payment.crypto_type.as_deref().unwrap_or("USDT_SPL");

        let row = sqlx::query_as::<_, (String,)>(
            r#"SELECT address FROM merchant_wallets
               WHERE merchant_id = $1 AND crypto_type = $2 AND is_active = true
               AND sandbox_mode = $3
               LIMIT 1"#,
        )
        .bind(payment.merchant_id)
        .bind(crypto_type_str)
        .bind(payment.sandbox_mode)
        .fetch_optional(&self.db)
        .await?;

        let merchant_address = match row {
            Some((addr,)) => addr,
            None => {
                // Fall back to to_address from the payment itself
                payment.to_address.clone().ok_or(DeloraError::Config(
                    "No destination address configured for merchant".into(),
                ))?
            }
        };

        // Map crypto_type to chain_id and token address
        let (chain_id, token_address) = self.map_crypto_to_chain_token(crypto_type_str)?;

        Ok((chain_id, token_address, merchant_address))
    }

    fn map_crypto_to_chain_token(&self, crypto_type: &str) -> Result<(u64, String), DeloraError> {
        // Map crypto_type to chain_id and token contract address
        match crypto_type {
            "SOL" => Ok((900, "So11111111111111111111111111111111111111112".into())),
            "USDT_SPL" => Ok((900, "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB".into())),
            "WSOL" => Ok((900, "So11111111111111111111111111111111111111112".into())),
            "ETH" => Ok((1, "0x0000000000000000000000000000000000000000".into())),
            "USDT_ETH" => Ok((1, "0xdAC17F958D2ee523a2206206994597C13D831ec7".into())),
            "BNB" => Ok((56, "0x0000000000000000000000000000000000000000".into())),
            "USDT_BEP20" => Ok((56, "0x55d398326f99059fF775485246999027B3197955".into())),
            "BUSD_BEP20" => Ok((56, "0xe9e7CEA3DedcA5984780Bafc599bD69ADd087D56".into())),
            "MATIC" => Ok((137, "0x0000000000000000000000000000000000000000".into())),
            "USDT_POLYGON" => Ok((137, "0xc2132D05D31c914a87C6611C10748AEb04B58e8F".into())),
            "ARB" => Ok((42161, "0x0000000000000000000000000000000000000000".into())),
            "USDT_ARBITRUM" => Ok((42161, "0xFd086bC7CD5C481DCC9C85ebE478A1C0b69FCbb9".into())),
            "BTC" => Err(DeloraError::Config(
                "Bitcoin not supported by Delora".into(),
            )),
            _ => Err(DeloraError::Config(format!(
                "Unsupported crypto type: {}",
                crypto_type
            ))),
        }
    }

    async fn resolve_quote_amount(
        &self,
        payment: &PaymentLinkInfo,
        _dest_chain_id: u64,
        _dest_currency: &str,
    ) -> Result<String, DeloraError> {
        // For now: use the payment amount directly.
        // In production: this would convert amount_usd to origin token amount
        // using cached prices.
        match &payment.amount {
            Some(amt) => {
                let scaled = amt * Decimal::from(10u64.pow(6)); // Assume 6 decimals for stablecoins
                Ok(scaled.to_string())
            }
            None => Err(DeloraError::Config(
                "Payment has no amount configured".into(),
            )),
        }
    }

    async fn resolve_token_summary(
        &self,
        chain_id: u64,
        address: &str,
    ) -> Result<CurrencySummary, DeloraError> {
        // Try cache first
        let addr_lower = address.to_lowercase();
        // Try to get from token list cache
        let client = self.client.clone();
        let all_tokens = self
            .cache
            .get_tokens(move || {
                let c = client.clone();
                async move { c.get_tokens().await }
            })
            .await
            .unwrap_or(TokenListResponse {
                tokens: Default::default(),
            });

        let chain_key = chain_id.to_string();
        let tokens_for_chain = all_tokens
            .tokens
            .get(&chain_key)
            .cloned()
            .unwrap_or_default();

        let found = tokens_for_chain
            .iter()
            .find(|t| t.address.to_lowercase() == addr_lower);

        match found {
            Some(t) => Ok(CurrencySummary {
                symbol: t.symbol.clone(),
                name: t.name.clone(),
                address: t.address.clone(),
                chain_id,
                chain_name: chain_name(chain_id).into(),
                decimals: t.decimals,
            }),
            None => {
                // Fallback: try single token lookup
                match self.client.get_token(chain_id, address).await {
                    Ok(t) => Ok(CurrencySummary {
                        symbol: t.symbol.clone(),
                        name: t.name.clone(),
                        address: t.address.clone(),
                        chain_id,
                        chain_name: chain_name(chain_id).into(),
                        decimals: t.decimals,
                    }),
                    Err(_) => {
                        // Last resort: return basic info
                        Ok(CurrencySummary {
                            symbol: "UNKNOWN".into(),
                            name: address.to_string(),
                            address: address.to_string(),
                            chain_id,
                            chain_name: chain_name(chain_id).into(),
                            decimals: 18,
                        })
                    }
                }
            }
        }
    }

    async fn get_advanced_quote_fallback(
        &self,
        simple_req: &QuoteRequest,
    ) -> Result<(QuoteResponse, Option<AdvancedRouteSummary>), DeloraError> {
        let adv_req = AdvancedRoutesRequest {
            sender_address: simple_req.sender_address.clone(),
            receiver_address: simple_req.receiver_address.clone(),
            origin_chain_id: simple_req.origin_chain_id,
            destination_chain_id: simple_req.destination_chain_id,
            amount: simple_req.amount.clone(),
            origin_currency: simple_req.origin_currency.clone(),
            destination_currency: simple_req.destination_currency.clone(),
            integrator: simple_req.integrator.clone(),
            fee: simple_req.fee,
            slippage: simple_req.slippage,
            include_bridges: simple_req.include_bridges.clone(),
            exclude_bridges: simple_req.exclude_bridges.clone(),
            include_exchanges: simple_req.include_exchanges.clone(),
            exclude_exchanges: simple_req.exclude_exchanges.clone(),
            max_routes: Some(3),
        };

        let routes_resp = self.client.get_advanced_routes(&adv_req).await?;
        let best_route = routes_resp
            .routes
            .into_iter()
            .next()
            .ok_or(DeloraError::NoRoutesFound)?;

        let first_step = best_route
            .steps
            .first()
            .cloned()
            .ok_or(DeloraError::NoStepsInRoute)?;

        let populated = self
            .client
            .populate_step_transaction(
                &first_step,
                &StepTransactionContext {
                    sender_address: simple_req.sender_address.clone(),
                    receiver_address: simple_req.receiver_address.clone(),
                },
            )
            .await?;

        let tx_req = populated.transaction_request.unwrap_or(TransactionRequest {
            to: String::new(),
            value: "0x0".into(),
            data: String::new(),
            gas: None,
        });

        let quote = QuoteResponse {
            input_amount: best_route.input_amount,
            output_amount: best_route.output_amount.unwrap_or_default(),
            min_output_amount: best_route.min_output_amount,
            adapter: best_route.adapter.clone(),
            calldata: Calldata {
                to: tx_req.to,
                value: tx_req.value,
                data: tx_req.data,
            },
            fees: best_route.fees.unwrap_or(FeeInfo {
                total: FeeItem {
                    amount: "0".into(),
                    currency_symbol: "UNKNOWN".into(),
                    currency_address: None,
                    chain_id: simple_req.origin_chain_id,
                    decimals: None,
                    fee_type: None,
                    amount_usd: None,
                },
                breakdown: vec![],
                total_usd: None,
            }),
            gas: tx_req.gas,
            warnings: best_route.warnings.unwrap_or_default(),
            approval_address: populated.estimate.approval_address,
            estimated_time_sec: best_route.estimated_time_sec,
            bridge_scan: populated.bridge_scan,
            transaction_size: None,
            usd: None,
        };

        let summary = AdvancedRouteSummary {
            route_id: best_route.id,
            adapter: best_route.adapter,
            is_multistep: best_route.is_multistep,
            steps_count: best_route.steps.len() as u32,
        };

        Ok((quote, Some(summary)))
    }

    fn build_fee_breakdown(&self, quote: &QuoteResponse) -> QuoteFeeBreakdown {
        let delora_fee = quote.fees.total.amount.clone();
        let delora_fee_usd = quote
            .fees
            .total_usd
            .clone()
            .unwrap_or_else(|| "0.00".into());

        // Integrator fee = inputAmount * fee_rate
        let integrator_fee = "0".into(); // Simplification: actual calculation needs input amount
        let integrator_fee_usd = "0.00".into();

        let gas_estimate = quote
            .gas
            .as_ref()
            .and_then(|g| g.gas_price.clone())
            .unwrap_or_else(|| "0".into());
        let gas_estimate_usd = "0.00".into();

        let total_fee = delora_fee.clone();
        let total_fee_usd = delora_fee_usd.clone();

        QuoteFeeBreakdown {
            delora_fee,
            delora_fee_usd,
            integrator_fee,
            integrator_fee_usd,
            gas_fee_estimate: gas_estimate,
            gas_fee_estimate_usd: gas_estimate_usd,
            total_fee,
            total_fee_usd,
        }
    }

    #[allow(clippy::too_many_arguments)]
    async fn persist_cross_chain_payment(
        &self,
        quote_id: &Uuid,
        payment: &PaymentLinkInfo,
        origin_meta: &CurrencySummary,
        dest_meta: &CurrencySummary,
        merchant_address: &str,
        quote: &QuoteResponse,
        sender_addr: &str,
        route_summary: &Option<AdvancedRouteSummary>,
    ) -> Result<(), DeloraError> {
        let is_advanced = route_summary.is_some();
        let is_multistep = route_summary
            .as_ref()
            .map(|r| r.is_multistep)
            .unwrap_or(false);
        let route_id = route_summary.as_ref().map(|r| r.route_id.clone());
        let adapter = route_summary
            .as_ref()
            .map(|r| r.adapter.clone())
            .unwrap_or_else(|| quote.adapter.clone());

        let calldata_json = serde_json::to_value(&quote.calldata)?;
        let warnings_json = serde_json::to_value(&quote.warnings)?;
        let gas_json = serde_json::to_value(&quote.gas)?;

        let expires_at = Utc::now() + chrono::Duration::seconds(QUOTE_EXPIRY_SECONDS);

        sqlx::query(
            r#"INSERT INTO cross_chain_payments (
                quote_id, payment_transaction_id, merchant_id, invoice_id,
                origin_chain_id, origin_currency_address, origin_currency_symbol, origin_currency_decimals,
                destination_chain_id, destination_currency_address, destination_currency_symbol, destination_currency_decimals,
                input_amount, output_amount, min_output_amount,
                integrator_fee_rate, adapter, route_id, is_multistep, is_advanced,
                status, sender_address, merchant_destination_address,
                calldata, calldata_to, approval_address,
                delora_warnings, estimated_time_sec, gas_info,
                quote_expires_at, sandbox_mode
            ) VALUES (
                $1, $2, $3, $4,
                $5, $6, $7, $8,
                $9, $10, $11, $12,
                $13, $14, $15,
                $16, $17, $18, $19, $20,
                'quote_requested', $21, $22,
                $23, $24, $25,
                $26, $27, $28,
                $29, $30
            )"#,
        )
        .bind(quote_id)
        .bind(payment.payment_id)
        .bind(payment.merchant_id)
        .bind(payment.invoice_id)
        .bind(origin_meta.chain_id as i64)
        .bind(&origin_meta.address)
        .bind(&origin_meta.symbol)
        .bind(origin_meta.decimals as i32)
        .bind(dest_meta.chain_id as i64)
        .bind(&dest_meta.address)
        .bind(&dest_meta.symbol)
        .bind(dest_meta.decimals as i32)
        .bind(&quote.input_amount)
        .bind(&quote.output_amount)
        .bind(quote.min_output_amount.as_deref().unwrap_or("0"))
        .bind(Decimal::from_str(&format!("{:.4}", self.config.default_integrator_fee)).unwrap_or(Decimal::new(5, 3)))
        .bind(&adapter)
        .bind(route_id.as_deref())
        .bind(is_multistep)
        .bind(is_advanced)
        .bind(sender_addr)
        .bind(merchant_address)
        .bind(&calldata_json)
        .bind(&quote.calldata.to)
        .bind(quote.approval_address.as_deref())
        .bind(&warnings_json)
        .bind(quote.estimated_time_sec.map(|v| v as i64))
        .bind(&gas_json)
        .bind(expires_at)
        .bind(payment.sandbox_mode)
        .execute(&self.db)
        .await?;

        debug!("Persisted cross-chain payment: quote_id={}", quote_id);
        Ok(())
    }

    async fn get_cross_chain_payment(
        &self,
        quote_id: &Uuid,
    ) -> Result<CrossChainPaymentRow, DeloraError> {
        sqlx::query_as::<_, CrossChainPaymentRow>(
            "SELECT * FROM cross_chain_payments WHERE quote_id = $1 AND deleted_at IS NULL",
        )
        .bind(quote_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or(DeloraError::QuoteNotFound(*quote_id))
    }

    fn build_status_response(&self, payment: &CrossChainPaymentRow) -> CrossChainStatusResponse {
        let bridge_scan_url = payment
            .bridge_scan_metadata
            .as_ref()
            .and_then(|v| v.as_object())
            .and_then(|o| o.get("url"))
            .and_then(|u| u.as_str())
            .map(String::from);

        CrossChainStatusResponse {
            payment_id: payment.quote_id.to_string(),
            status: payment.status.clone(),
            origin_tx_hash: payment.origin_tx_hash.clone(),
            destination_tx_hash: payment.destination_tx_hash.clone(),
            bridge_scan_url,
            confirmations: payment.origin_confirmations,
            estimated_completion_sec: payment.estimated_time_sec.map(|v| v as u64),
        }
    }
}
