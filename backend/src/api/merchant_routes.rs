// Merchant Routes
// All merchant-specific API endpoints with API key authentication

use crate::api::{merchant_handlers, wallet_management, security_monitoring};
use crate::middleware::auth;
use axum::{
    middleware as axum_middleware,
    routing::{get, post, put},
    Router,
};
use crate::api::state::AppState;

pub fn create_merchant_router(state: AppState) -> Router<AppState> {
    Router::new()
        // Merchant profile management
        .route("/api/v1/merchant/profile", get(merchant_handlers::get_merchant_profile))
        .route("/api/v1/merchant/environment/switch", post(merchant_handlers::switch_environment))
        .route("/api/v1/merchant/api-keys/generate", post(merchant_handlers::generate_api_key))
        .route("/api/v1/merchant/api-keys/rotate", post(merchant_handlers::rotate_api_key))
        .route("/api/v1/merchant/webhook", put(merchant_handlers::set_webhook))
        
        // Payment management
        .route("/api/v1/merchant/payments", post(merchant_handlers::create_payment))
        .route("/api/v1/merchant/payments", get(merchant_handlers::list_payments))
        .route("/api/v1/merchant/payments/:payment_id", get(merchant_handlers::get_payment))
        .route("/api/v1/merchant/payments/:payment_id/verify", post(merchant_handlers::verify_payment))
        
        // Refund management
        .route("/api/v1/merchant/refunds", post(merchant_handlers::create_refund))
        .route("/api/v1/merchant/refunds/:refund_id", get(merchant_handlers::get_refund))
        .route("/api/v1/merchant/refunds/:refund_id/complete", post(merchant_handlers::complete_refund))
        
        // Analytics and reporting
        .route("/api/v1/merchant/analytics", get(merchant_handlers::get_analytics))
        .route("/api/v1/merchant/analytics/export", get(merchant_handlers::export_analytics))
        .route("/api/v1/merchant/audit-logs", get(merchant_handlers::get_audit_logs))
        
        // Balance and financial
        .route("/api/v1/merchant/balance", get(merchant_handlers::get_balance))
        .route("/api/v1/merchant/balance/history", get(merchant_handlers::get_balance_history))
        
        // Withdrawal management
        .route("/api/v1/merchant/withdrawals", post(merchant_handlers::create_withdrawal))
        .route("/api/v1/merchant/withdrawals", get(merchant_handlers::list_withdrawals))
        .route("/api/v1/merchant/withdrawals/:withdrawal_id", get(merchant_handlers::get_withdrawal))
        .route("/api/v1/merchant/withdrawals/:withdrawal_id/cancel", post(merchant_handlers::cancel_withdrawal))
        .route("/api/v1/merchant/withdrawals/:withdrawal_id/process", post(wallet_management::process_withdrawal))
        
        // Wallet management
        .route("/api/v1/merchant/wallets", get(wallet_management::get_wallet_configs))
        .route("/api/v1/merchant/wallets", put(merchant_handlers::set_wallet))
        .route("/api/v1/merchant/wallets/configure-address", post(wallet_management::configure_address_only_wallet))
        .route("/api/v1/merchant/wallets/generate", post(wallet_management::generate_wallet))
        .route("/api/v1/merchant/wallets/import", post(wallet_management::import_wallet))
        .route("/api/v1/merchant/wallets/export-key", post(wallet_management::export_private_key))
        .route("/api/v1/merchant/wallets/gas-check", get(wallet_management::check_gas_requirements))
        .route("/api/v1/merchant/wallets/gas-estimates", get(wallet_management::get_gas_estimates))
        .route("/api/v1/merchant/wallets/withdrawal-capability/:crypto_type", get(wallet_management::check_withdrawal_capability))
        
        // Security settings (merchant's own security preferences)
        .route("/api/v1/merchant/security/settings", get(security_monitoring::get_security_settings))
        .route("/api/v1/merchant/security/settings", put(security_monitoring::update_security_settings))
        .route("/api/v1/merchant/security/events", get(security_monitoring::get_security_events))
        .route("/api/v1/merchant/security/alerts", get(security_monitoring::get_security_alerts))
        .route("/api/v1/merchant/security/alerts/:alert_id/acknowledge", post(security_monitoring::acknowledge_security_alert))
        .route("/api/v1/merchant/security/balance-alerts", get(security_monitoring::get_balance_alerts))
        .route("/api/v1/merchant/security/balance-alerts/:alert_id/resolve", post(security_monitoring::resolve_balance_alert))
        .route("/api/v1/merchant/security/gas-check", get(security_monitoring::check_gas_balances))
        
        // IP whitelist management
        .route("/api/v1/merchant/ip-whitelist", put(merchant_handlers::set_ip_whitelist))
        .route("/api/v1/merchant/ip-whitelist", get(merchant_handlers::get_ip_whitelist))
        
        // Invoice management
        .route("/api/v1/merchant/invoices", post(merchant_handlers::create_invoice))
        .route("/api/v1/merchant/invoices", get(merchant_handlers::list_invoices))
        .route("/api/v1/merchant/invoices/:invoice_id", get(merchant_handlers::get_invoice))
        
        // Sandbox testing
        .route("/api/v1/merchant/sandbox/enable", post(merchant_handlers::enable_sandbox))
        .route("/api/v1/merchant/sandbox/payments/:payment_id/simulate", post(merchant_handlers::simulate_payment))
        
        // Apply merchant API key authentication
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            auth::auth_middleware,
        ))
        .with_state(state)
}
