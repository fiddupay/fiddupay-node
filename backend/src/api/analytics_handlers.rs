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
use serde_json::json;
use serde::Deserialize;



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
    let from = query.from_date.unwrap_or_else(|| chrono::Utc::now() - chrono::Duration::days(30));
    let to = query.to_date.unwrap_or_else(|| chrono::Utc::now());
    
    match state.analytics_service.get_analytics(
        context.merchant_id, 
        from, 
        to, 
        query.blockchain, 
        query.status, 
        Some(context.sandbox_mode)
    ).await {
        Ok(response) => (StatusCode::OK, Json(response)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    }
}

pub async fn export_analytics(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
    Query(query): Query<AnalyticsQuery>,
) -> impl IntoResponse {
    let from = query.from_date.unwrap_or_else(|| chrono::Utc::now() - chrono::Duration::days(30));
    let to = query.to_date.unwrap_or_else(|| chrono::Utc::now());
    
    if query.format.as_deref() == Some("json") {
        match state.analytics_service.get_analytics(
            context.merchant_id, 
            from, 
            to, 
            query.blockchain.clone(), 
            query.status.clone(), 
            Some(context.sandbox_mode)
        ).await {
            Ok(report) => (StatusCode::OK, Json(report)).into_response(),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
        }
    } else {
        match state.analytics_service.export_csv(context.merchant_id, from, to, query.blockchain, query.status, Some(context.sandbox_mode)).await {
            Ok(csv) => (StatusCode::OK, csv).into_response(),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
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
    let limit = params.limit.unwrap_or(50).min(100).max(1);

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
            use sqlx::Row;
            let txns: Vec<serde_json::Value> = rows.into_iter().map(|row| {
                json!({
                    "type": row.get::<String, _>("txn_type"),
                    "id": row.get::<String, _>("id"),
                    "crypto_amount": row.get::<String, _>("crypto_amount"),
                    "usd_amount": row.get::<String, _>("usd_amount"),
                    "crypto_type": row.get::<String, _>("crypto_type"),
                    "status": row.get::<String, _>("status"),
                    "transaction_hash": row.get::<Option<String>, _>("transaction_hash"),
                    "created_at": row.get::<chrono::DateTime<chrono::Utc>, _>("created_at").to_rfc3339()
                })
            }).collect();
            
            (StatusCode::OK, Json(json!({"transactions": txns}))).into_response()
        },
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
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
    match state.audit_service.get_logs(AuditScope::Merchant(context.merchant_id), query).await {
        Ok(logs) => (StatusCode::OK, Json(logs)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    }
}

// ============================================================================
// Balance
// ============================================================================

pub async fn get_balance(
    State(state): State<AppState>,
    Extension(context): Extension<MerchantContext>,
) -> impl IntoResponse {
    match state.balance_service.get_all_balances(context.merchant_id, context.sandbox_mode).await {
        Ok(balance) => {
            let mut response_balances = vec![];
            use rust_decimal::Decimal;
            let mut total_usd = Decimal::ZERO;
            let mut available_usd = Decimal::ZERO;
            let mut reserved_usd = Decimal::ZERO;

            for b in balance.balances {
                let price = state.price_service.get_price(b.crypto_type).await.unwrap_or(0.0);
                use rust_decimal::prelude::FromPrimitive;
                let price_dec = Decimal::from_f64(price).unwrap_or(Decimal::ZERO);
                
                let cur_avail_usd = b.available_balance * price_dec;
                let cur_res_avail_usd = b.reserved_balance * price_dec;
                let cur_total_usd = b.total_balance * price_dec;

                available_usd += cur_avail_usd;
                reserved_usd += cur_res_avail_usd;
                total_usd += cur_total_usd;

                response_balances.push(json!({
                    "crypto_type": b.crypto_type.to_string(),
                    "available_balance": b.available_balance,
                    "available_usd": cur_avail_usd,
                    "reserved_balance": b.reserved_balance,
                    "reserved_usd": cur_res_avail_usd,
                    "total_balance": b.total_balance,
                    "total_usd": cur_total_usd,
                    "balance_usd": cur_total_usd, // Frontend looks for balance_usd on rows
                    "last_updated": b.last_updated,
                }));
            }
            
            let response_obj = json!({
                "total_usd": total_usd,
                "available_usd": available_usd,
                "reserved_usd": reserved_usd,
                "balances": response_balances
            });

            (StatusCode::OK, Json(response_obj)).into_response()
        },
        Err(e) => {
            tracing::error!("Failed to get balances: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response()
        },
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
    
    match state.analytics_service.get_balance_history(
        context.merchant_id,
        limit,
        context.sandbox_mode
    ).await {
        Ok(history) => (StatusCode::OK, Json(history)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    }
}
