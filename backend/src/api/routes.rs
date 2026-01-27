// API Routes
// HTTP route definitions

use crate::api::{handlers, admin_handlers, wallet_management, security_monitoring, status, blog, careers};
use crate::api::state::AppState;
use crate::middleware::{auth, ip_whitelist, logging, rate_limit};
use axum::{
    http::{
        header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
        HeaderValue, Method,
    },
    middleware as axum_middleware,
    routing::{get, post, put, delete},
    Router,
};
use tower_http::cors::CorsLayer;

pub fn create_router(state: AppState) -> Router {
    // Create rate limiter with config
    let rate_limiter = rate_limit::create_rate_limiter(state.config.rate_limit_requests_per_minute);

    // Public routes (no auth required)
    let public_routes = Router::new()
        .route("/health", get(handlers::health_check))
        // .route("/test-auth/:api_key", get(handlers::debug_auth)) // DEBUG ENDPOINT - REMOVED FOR SECURITY
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
        
        // Admin endpoints
        .route("/api/v1/admin/dashboard", get(admin_handlers::get_admin_dashboard))
        .route("/api/v1/admin/merchants", get(admin_handlers::get_merchants_summary))
        .route("/api/v1/admin/merchants/:merchant_id", get(admin_handlers::get_merchant_details))
        .route("/api/v1/admin/merchants/:merchant_id/suspend", post(admin_handlers::suspend_merchant))
        .route("/api/v1/admin/merchants/:merchant_id/activate", post(admin_handlers::activate_merchant))
        .route("/api/v1/admin/merchants/:merchant_id/delete", delete(admin_handlers::delete_merchant))
        
        // Admin Security Management
        .route("/api/v1/admin/security/events", get(admin_handlers::get_admin_security_events))
        .route("/api/v1/admin/security/alerts", get(admin_handlers::get_admin_security_alerts))
        .route("/api/v1/admin/security/alerts/:alert_id/acknowledge", post(admin_handlers::acknowledge_admin_security_alert))
        .route("/api/v1/admin/security/settings", get(admin_handlers::get_security_settings))
        .route("/api/v1/admin/security/settings", put(admin_handlers::update_security_settings))
        
        // Admin System Configuration
        .route("/api/v1/admin/config/environment", get(admin_handlers::get_environment_config))
        .route("/api/v1/admin/config/environment", put(admin_handlers::update_environment_config))
        .route("/api/v1/admin/config/fees", get(admin_handlers::get_fee_config))
        .route("/api/v1/admin/config/fees", put(admin_handlers::update_fee_config))
        .route("/api/v1/admin/config/limits", get(admin_handlers::get_system_limits))
        .route("/api/v1/admin/config/limits", put(admin_handlers::update_system_limits))
        
        // Admin Payment Management
        .route("/api/v1/admin/payments", get(admin_handlers::get_all_payments))
        .route("/api/v1/admin/payments/:payment_id", get(admin_handlers::get_payment_details))
        .route("/api/v1/admin/payments/:payment_id/force-confirm", post(admin_handlers::force_confirm_payment))
        .route("/api/v1/admin/payments/:payment_id/force-fail", post(admin_handlers::force_fail_payment))
        
        // Admin Withdrawal Management
        .route("/api/v1/admin/withdrawals", get(admin_handlers::get_all_withdrawals))
        .route("/api/v1/admin/withdrawals/:withdrawal_id/approve", post(admin_handlers::approve_withdrawal))
        .route("/api/v1/admin/withdrawals/:withdrawal_id/reject", post(admin_handlers::reject_withdrawal))
        
        // Admin Analytics & Reporting
        .route("/api/v1/admin/analytics/platform", get(admin_handlers::get_platform_analytics))
        .route("/api/v1/admin/analytics/revenue", get(admin_handlers::get_revenue_analytics))
        .route("/api/v1/admin/reports/transactions", get(admin_handlers::get_transaction_reports))
        .route("/api/v1/admin/reports/merchants", get(admin_handlers::get_merchant_reports))
        
        // Admin Wallet Management
        .route("/api/v1/admin/wallets/hot", get(admin_handlers::get_hot_wallets))
        .route("/api/v1/admin/wallets/cold", get(admin_handlers::get_cold_wallets))
        .route("/api/v1/admin/wallets/balances", get(admin_handlers::get_wallet_balances))
        .route("/api/v1/admin/wallets/transfer", post(admin_handlers::transfer_funds))
        
        // Admin User Management
        .route("/api/v1/admin/users", get(admin_handlers::get_admin_users))
        .route("/api/v1/admin/users", post(admin_handlers::create_admin_user))
        .route("/api/v1/admin/users/:user_id", delete(admin_handlers::delete_admin_user))
        .route("/api/v1/admin/users/:user_id/permissions", put(admin_handlers::update_user_permissions))
        
        // Admin System Maintenance
        .route("/api/v1/admin/system/health", get(admin_handlers::get_system_health))
        .route("/api/v1/admin/system/logs", get(admin_handlers::get_system_logs))
        .route("/api/v1/admin/system/backup", post(admin_handlers::create_system_backup))
        .route("/api/v1/admin/system/maintenance", post(admin_handlers::toggle_maintenance_mode))
        
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
        .route("/api/v1/careers", get(careers::get_careers))
        .route("/api/v1/contact", post(handlers::submit_contact_form))
        .route("/api/v1/pricing", get(handlers::get_pricing_info));

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
