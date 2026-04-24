// Analytics Handlers
// Analytics, audit logs, balance, and unified transaction endpoints

use crate::api::state::AppState;
use crate::middleware::auth::MerchantContext;
use axum::{
    extract::{Extension, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use rust_decimal::Decimal;
use serde::Deserialize;
use serde_json::json;
use sqlx::Row;

// ============================================================================
// Analytics
// ============================================================================

#[derive(Deserialize)]
pub struct AnalyticsQuery {
    pub from_date: Option<chrono::DateTime<chrono::Utc>>,
    pub to_date: Option<chrono::DateTime<chrono::Utc>>,
    pub status: Option<String>,
    pub blockchain: Option<String>,
    pub format: Option<String>,
}

pub async fn get_analytics(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Query(query): Query<AnalyticsQuery>,
) -> impl IntoResponse {
    let from = query
        .from_date
        .unwrap_or_else(|| chrono::Utc::now() - chrono::Duration::days(30));
    let to = query.to_date.unwrap_or_else(chrono::Utc::now);

    match state
        .analytics_service
        .get_analytics(
            context.merchant_id,
            from,
            to,
            query.blockchain,
            query.status,
            Some(context.sandbox_mode),
        )
        .await
    {
        Ok(mut response) => {
            // Filter by_blockchain map
            response.by_blockchain.retain(|network, _| {
                // Network strings in analytics are usually "Solana", "Ethereum", etc.
                // or ticker "SOL", "ETH", etc.
                // We need to map them to CryptoType for checking.
                let ct_str = match network.to_uppercase().as_str() {
                    "SOLANA" | "SOL" => "SOL",
                    "ETHEREUM" | "ETH" => "ETH",
                    "BSC" | "BNB" => "BNB",
                    "POLYGON" | "MATIC" => "MATIC",
                    "ARBITRUM" | "ARB" => "ARB",
                    "BITCOIN" | "BTC" => "BTC",
                    _ => return true, // Keep unknown ones or handle specifically
                };

                if let Ok(ct) = crate::payment::models::CryptoType::from_string(ct_str) {
                    state.config.is_blockchain_enabled(&ct)
                } else {
                    true
                }
            });
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn export_analytics(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Query(query): Query<AnalyticsQuery>,
) -> impl IntoResponse {
    let from = query
        .from_date
        .unwrap_or_else(|| chrono::Utc::now() - chrono::Duration::days(30));
    let to = query.to_date.unwrap_or_else(chrono::Utc::now);

    let data = match state
        .report_service
        .get_payment_data(
            context.merchant_id,
            from,
            to,
            query.blockchain.clone(),
            query.status.clone(),
            Some(context.sandbox_mode),
        )
        .await
    {
        Ok(d) => d,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
                .into_response()
        }
    };

    match query.format.as_deref().unwrap_or("csv") {
        "json" => (StatusCode::OK, Json(data)).into_response(),
        "pdf" => {
            let business_name: String =
                match sqlx::query_scalar("SELECT business_name FROM merchants WHERE id = $1")
                    .bind(context.merchant_id)
                    .fetch_one(&state.db_pool)
                    .await
                {
                    Ok(name) => name,
                    Err(_) => "FidduPay Merchant".to_string(),
                };

            match state
                .report_service
                .generate_pdf(&business_name, from, to, data)
                .await
            {
                Ok(pdf) => (
                    StatusCode::OK,
                    [
                        ("Content-Type", "application/pdf"),
                        ("Content-Disposition", "attachment; filename=\"report.pdf\""),
                    ],
                    pdf,
                )
                    .into_response(),
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": e.to_string()})),
                )
                    .into_response(),
            }
        }
        _ => {
            // Default to CSV
            match state.report_service.generate_csv(data).await {
                Ok(csv) => (
                    StatusCode::OK,
                    [
                        ("Content-Type", "text/csv"),
                        ("Content-Disposition", "attachment; filename=\"report.csv\""),
                    ],
                    csv,
                )
                    .into_response(),
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": e.to_string()})),
                )
                    .into_response(),
            }
        }
    }
}

// ============================================================================
// Unified Transactions
// ============================================================================

#[derive(Deserialize)]
pub struct UnifiedTransactionQuery {
    pub limit: Option<i64>,
}

pub async fn list_unified_transactions(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Query(params): Query<UnifiedTransactionQuery>,
) -> impl IntoResponse {
    let merchant_id = context.merchant_id;
    let is_sandbox = context.sandbox_mode;
    let limit = params.limit.unwrap_or(50).clamp(1, 100);

    let query = r#"
        (SELECT 
            'payment' as txn_type,
            payment_id as id,
            amount::text as crypto_amount,
            amount_usd::text as usd_amount,
            crypto_type,
            status,
            transaction_hash,
            created_at
        FROM payment_transactions
        WHERE merchant_id = $1 AND sandbox_mode = $2)
        
        UNION ALL
        
        (SELECT 
            'refund' as txn_type,
            r.refund_id as id,
            r.amount::text as crypto_amount,
            r.amount_usd::text as usd_amount,
            p.crypto_type,
            r.status,
            r.transaction_hash,
            r.created_at
        FROM refunds r
        JOIN payment_transactions p ON r.payment_id = p.id
        WHERE r.merchant_id = $1 AND r.sandbox_mode = $2)
        
        UNION ALL
        
        (SELECT 
            'withdrawal' as txn_type,
            withdrawal_id as id,
            amount::text as crypto_amount,
            amount_usd::text as usd_amount,
            crypto_type,
            status,
            transaction_hash,
            created_at
        FROM withdrawals
        WHERE merchant_id = $1 AND sandbox_mode = $2)

        UNION ALL
        
        (SELECT 
            LOWER(type) as txn_type,
            id::text as id,
            amount::text as crypto_amount,
            amount_usd::text as usd_amount,
            crypto_type,
            status,
            transaction_hash,
            created_at
        FROM customer_transactions
        WHERE merchant_id = $1 AND sandbox_mode = $2)
        
        ORDER BY created_at DESC
        LIMIT $3
    "#;

    match sqlx::query(query)
        .bind(merchant_id)
        .bind(is_sandbox)
        .bind(limit)
        .fetch_all(&state.db_pool)
        .await
    {
        Ok(rows) => {
            use std::collections::HashMap;

            // 1. Gather all unique crypto types from rows
            let mut unique_cryptos = std::collections::HashSet::new();
            for row in &rows {
                let crypto_type_str: String = row.get("crypto_type");
                unique_cryptos.insert(crypto_type_str);
            }

            // 2. Fetch all required prices in parallel once
            let mut price_map = HashMap::new();
            let price_tasks = unique_cryptos.into_iter().map(|ct_str| {
                let state = state.clone();
                async move {
                    if let Ok(ct_enum) = crate::payment::models::CryptoType::from_string(&ct_str) {
                        let price = state.price_service.get_price(ct_enum).await.unwrap_or(0.0);
                        Some((ct_str, price))
                    } else {
                        None
                    }
                }
            });

            use futures::future::join_all;
            let price_results = join_all(price_tasks).await;
            for res in price_results.into_iter().flatten() {
                price_map.insert(res.0, res.1);
            }

            // 3. Process transactions using the pre-fetched prices
            let txns: Vec<_> = rows
                .into_iter()
                .map(|row| {
                    let txn_type = row.get::<String, _>("txn_type");
                    let crypto_id = row.get::<String, _>("id");
                    let crypto_amount_str = row.get::<String, _>("crypto_amount");
                    let mut usd_amount_str = row.get::<String, _>("usd_amount");
                    let crypto_type_str = row.get::<String, _>("crypto_type");
                    let status = row.get::<String, _>("status");
                    let transaction_hash = row.get::<Option<String>, _>("transaction_hash");
                    let created_at = row.get::<chrono::DateTime<chrono::Utc>, _>("created_at");

                    // Parse amounts for calculation
                    let crypto_amount = crypto_amount_str
                        .parse::<Decimal>()
                        .unwrap_or(Decimal::ZERO);
                    let usd_amount = usd_amount_str.parse::<Decimal>().unwrap_or(Decimal::ZERO);

                    // If USD value is missing or zero, and it's a relevant transaction type, recalculate it
                    // Note: database might use 'payment' for deposits and 'merchant_payment' for payments
                    let is_zero_usd = usd_amount.is_zero();
                    let is_recalculable_type = txn_type == "withdrawal"
                        || txn_type == "merchant_payment"
                        || txn_type == "payment"
                        || txn_type == "deposit"
                        || txn_type == "sweep";

                    if is_zero_usd && is_recalculable_type {
                        if let Some(&price) = price_map.get(&crypto_type_str) {
                            let price_decimal =
                                Decimal::from_f64_retain(price).unwrap_or(Decimal::ZERO);
                            let usd_val = (crypto_amount * price_decimal).round_dp(2);
                            usd_amount_str = usd_val.to_string();
                        }
                    }

                    json!({
                        "type": txn_type,
                        "id": crypto_id,
                        "crypto_amount": crypto_amount_str,
                        "usd_amount": usd_amount_str,
                        "crypto_type": crypto_type_str,
                        "status": status,
                        "transaction_hash": transaction_hash,
                        "created_at": created_at.to_rfc3339()
                    })
                })
                .collect();

            (StatusCode::OK, Json(json!({"transactions": txns}))).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

// ============================================================================
// Audit Logs
// ============================================================================

#[derive(Deserialize)]
pub struct AuditLogQueryParams {
    pub from: Option<String>,
    pub to: Option<String>,
    pub action_type: Option<String>,
    pub limit: Option<i64>,
}

pub async fn get_audit_logs(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Query(params): Query<AuditLogQueryParams>,
) -> impl IntoResponse {
    let query = crate::services::audit_service::AuditLogQuery {
        from: params.from.and_then(|s| s.parse().ok()),
        to: params.to.and_then(|s| s.parse().ok()),
        action_type: params.action_type,
        limit: params.limit,
    };

    use crate::services::audit_service::AuditScope;
    match state
        .audit_service
        .get_logs(AuditScope::Merchant(context.merchant_id), query)
        .await
    {
        Ok(logs) => (StatusCode::OK, Json(logs)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

// ============================================================================
// Balance
// ============================================================================

pub async fn get_balance(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
) -> impl IntoResponse {
    let balance_res = state
        .balance_service
        .get_all_balances(context.merchant_id, context.sandbox_mode, true)
        .await;

    // Trigger on-demand on-chain balance check (Lazy-Check)
    // Runs in background to avoid blocking the API response
    let balance_monitor = state.balance_monitor.clone();
    let merchant_id = context.merchant_id;
    let is_live = !context.sandbox_mode;
    tokio::spawn(async move {
        let _ = balance_monitor
            .check_merchant_on_demand(merchant_id, is_live)
            .await;
    });

    match balance_res {
        Ok(balance) => {
            use futures::future::join_all;
            use std::collections::HashMap;

            // 1. Gather all unique crypto types
            let mut unique_cryptos = std::collections::HashSet::new();
            for b in &balance.balances {
                unique_cryptos.insert(b.crypto_type);
            }

            // 2. Fetch required prices in parallel once
            let mut price_map = HashMap::new();
            let price_tasks = unique_cryptos.into_iter().map(|ct| {
                let state = state.clone();
                async move {
                    let price = state.price_service.get_price(ct).await.unwrap_or(0.0);
                    (ct, price)
                }
            });

            let price_results = join_all(price_tasks).await;
            for (ct, price) in price_results {
                price_map.insert(ct, price);
            }

            // 3. Process balances using pre-fetched prices
            let mut total_available_usd = Decimal::ZERO;
            let mut total_reserved_usd = Decimal::ZERO;
            let mut overall_total_usd = Decimal::ZERO;

            let balances_json: Vec<_> = balance
                .balances
                .into_iter()
                .filter(|b| state.config.is_blockchain_enabled(&b.crypto_type))
                .map(|b| {
                    let price = price_map.get(&b.crypto_type).copied().unwrap_or(0.0);
                    let price_dec = Decimal::from_f64_retain(price).unwrap_or(Decimal::ZERO);

                    let cur_avail_usd = (b.available_balance * price_dec).round_dp(2);
                    let cur_res_avail_usd = (b.reserved_balance * price_dec).round_dp(2);
                    let cur_total_usd = (b.total_balance * price_dec).round_dp(2);

                    total_available_usd += cur_avail_usd;
                    total_reserved_usd += cur_res_avail_usd;
                    overall_total_usd += cur_total_usd;

                    json!({
                        "crypto_type": b.crypto_type.to_string(),
                        "available_balance": b.available_balance.to_string(),
                        "available_usd": cur_avail_usd.to_string(),
                        "reserved_balance": b.reserved_balance.to_string(),
                        "reserved_usd": cur_res_avail_usd.to_string(),
                        "total_balance": b.total_balance.to_string(),
                        "total_usd": cur_total_usd.to_string(),
                        "balance_usd": cur_total_usd.to_string() // Compatibility with legacy components
                    })
                })
                .collect();

            (
                StatusCode::OK,
                Json(json!({
                    "available_usd": total_available_usd,
                    "reserved_usd": total_reserved_usd,
                    "total_usd": overall_total_usd,
                    "balances": balances_json
                })),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!("Failed to get balances: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
                .into_response()
        }
    }
}

#[derive(Deserialize)]
pub struct BalanceHistoryQuery {
    pub limit: Option<i64>,
}

pub async fn get_balance_history(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Query(params): Query<BalanceHistoryQuery>,
) -> impl IntoResponse {
    let limit = params.limit.unwrap_or(100).min(1000);

    match state
        .analytics_service
        .get_balance_history(context.merchant_id, limit, context.sandbox_mode)
        .await
    {
        Ok(history) => (StatusCode::OK, Json(history)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}
