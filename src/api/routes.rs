// API Routes
// HTTP route definitions

use crate::api::handlers;
use crate::api::state::AppState;
use crate::middleware::{auth, ip_whitelist, logging, rate_limit};
use axum::{
    middleware as axum_middleware,
    routing::{get, post, put},
    Router,
};

pub fn create_router(state: AppState) -> Router {
    // Create rate limiter
    let rate_limiter = rate_limit::create_rate_limiter();

    // Public routes (no auth required)
    let public_routes = Router::new()
        .route("/health", get(handlers::health_check))
        .route("/pay/:link_id", get(handlers::payment_page))
        .route("/pay/:link_id/status", get(handlers::payment_status))
        .route("/api/v1/merchants/register", post(handlers::register_merchant));

    // Protected routes (auth required)
    let protected_routes = Router::new()
        // Merchant endpoints
        .route("/api/v1/merchants/api-keys/rotate", post(handlers::rotate_api_key))
        .route("/api/v1/merchants/wallets", put(handlers::set_wallet))
        .route("/api/v1/merchants/webhook", put(handlers::set_webhook))
        
        // Payment endpoints
        .route("/api/v1/payments", post(handlers::create_payment))
        .route("/api/v1/payments", get(handlers::list_payments))
        .route("/api/v1/payments/:payment_id", get(handlers::get_payment))
        .route("/api/v1/payments/:payment_id/verify", post(handlers::verify_payment))
        
        // Refund endpoints
        .route("/api/v1/refunds", post(handlers::create_refund))
        .route("/api/v1/refunds/:refund_id", get(handlers::get_refund))
        .route("/api/v1/refunds/:refund_id/complete", post(handlers::complete_refund))
        
        // Analytics endpoints
        .route("/api/v1/analytics", get(handlers::get_analytics))
        .route("/api/v1/analytics/export", get(handlers::export_analytics))
        
        // Sandbox endpoints
        .route("/api/v1/sandbox/enable", post(handlers::enable_sandbox))
        .route("/api/v1/sandbox/payments/:payment_id/simulate", post(handlers::simulate_payment))
        
        // IP Whitelist endpoints
        .route("/api/v1/merchants/ip-whitelist", put(handlers::set_ip_whitelist))
        .route("/api/v1/merchants/ip-whitelist", get(handlers::get_ip_whitelist))
        
        // Audit Log endpoints
        .route("/api/v1/audit-logs", get(handlers::get_audit_logs))
        
        // Balance endpoints
        .route("/api/v1/merchants/balance", get(handlers::get_balance))
        .route("/api/v1/merchants/balance/history", get(handlers::get_balance_history))
        
        // Withdrawal endpoints
        .route("/api/v1/withdrawals", post(handlers::create_withdrawal))
        .route("/api/v1/withdrawals", get(handlers::list_withdrawals))
        .route("/api/v1/withdrawals/:withdrawal_id", get(handlers::get_withdrawal))
        .route("/api/v1/withdrawals/:withdrawal_id/cancel", post(handlers::cancel_withdrawal))
        
        // Apply middleware in order: logging -> rate limit -> auth -> IP whitelist
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            ip_whitelist::ip_whitelist_middleware,
        ))
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            auth::auth_middleware,
        ))
        .layer(axum_middleware::from_fn(move |req, next| {
            rate_limit::rate_limit_middleware(rate_limiter.clone(), req, next)
        }))
        .layer(axum_middleware::from_fn(logging::logging_middleware));

    // Combine routes
    public_routes
        .merge(protected_routes)
        .with_state(state)
}
