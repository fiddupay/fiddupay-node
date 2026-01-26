// API Routes
// HTTP route definitions

use crate::api::{handlers, wallet_management, security_monitoring, status, blog, careers};
use crate::api::state::AppState;
use crate::middleware::{auth, ip_whitelist, logging, rate_limit};
use axum::{
    http::{
        header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
        HeaderValue, Method,
    },
    middleware as axum_middleware,
    routing::{get, post, put},
    Router,
};
use tower_http::cors::CorsLayer;

pub fn create_router(state: AppState) -> Router {
    // Create rate limiter with config
    let rate_limiter = rate_limit::create_rate_limiter(state.config.rate_limit_requests_per_minute);

    // Public routes (no auth required)
    let public_routes = Router::new()
        .route("/health", get(handlers::health_check))
        .route("/test-auth/:api_key", get(handlers::debug_auth)) // DEBUG ENDPOINT
        .route("/pay/:link_id", get(handlers::payment_page))
        .route("/pay/:link_id/status", get(handlers::payment_status))
        .route("/api/v1/merchants/register", post(handlers::register_merchant))
        .route("/api/v1/merchants/login", post(handlers::login_merchant))
        .route("/api/v1/currencies/supported", get(handlers::get_supported_currencies));

    // Protected routes (auth required)
    let protected_routes = Router::new()
        // Merchant endpoints
        .route("/api/v1/merchants/profile", get(handlers::get_merchant_profile))
        .route("/api/v1/merchants/environment/switch", post(handlers::switch_environment))
        .route("/api/v1/merchants/api-keys/generate", post(handlers::generate_api_key))
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
        .route("/api/v1/withdrawals/:withdrawal_id/process", post(wallet_management::process_withdrawal))
        
        // Wallet Management endpoints
        .route("/api/v1/wallets", get(wallet_management::get_wallet_configs))
        .route("/api/v1/wallets/configure-address", post(wallet_management::configure_address_only_wallet))
        .route("/api/v1/wallets/generate", post(wallet_management::generate_wallet))
        .route("/api/v1/wallets/import", post(wallet_management::import_wallet))
        .route("/api/v1/wallets/export-key", post(wallet_management::export_private_key))
        .route("/api/v1/wallets/gas-check", get(wallet_management::check_gas_requirements))
        .route("/api/v1/wallets/gas-estimates", get(wallet_management::get_gas_estimates))
        .route("/api/v1/wallets/withdrawal-capability/:crypto_type", get(wallet_management::check_withdrawal_capability))
        
        // Security Monitoring endpoints
        .route("/api/v1/security/events", get(security_monitoring::get_security_events))
        .route("/api/v1/security/alerts", get(security_monitoring::get_security_alerts))
        .route("/api/v1/security/alerts/:alert_id/acknowledge", post(security_monitoring::acknowledge_security_alert))
        .route("/api/v1/security/balance-alerts", get(security_monitoring::get_balance_alerts))
        .route("/api/v1/security/balance-alerts/:alert_id/resolve", post(security_monitoring::resolve_balance_alert))
        .route("/api/v1/security/gas-check", get(security_monitoring::check_gas_balances))
        .route("/api/v1/security/settings", get(security_monitoring::get_security_settings))
        .route("/api/v1/security/settings", put(security_monitoring::update_security_settings))
        
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

    // Additional public routes (no auth required)
    let additional_public_routes = Router::new()
        .route("/api/v1/status", get(status::get_system_status))
        .route("/api/v1/blog", get(blog::get_blog_posts))
        .route("/api/v1/careers", get(careers::get_careers));

    // Combine routes with CORS
    let cors = CorsLayer::new()
        .allow_origin(
            std::env::var("FRONTEND_URL")
                .unwrap_or_else(|_| "http://localhost:3000".to_string())
                .parse::<HeaderValue>()
                .unwrap()
        )
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE])
        .allow_credentials(true);

    public_routes
        .merge(additional_public_routes)
        .merge(protected_routes)
        .layer(cors)
        .with_state(state)
}
