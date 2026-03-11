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
        .route("/api/v1/merchants/profile", get(merchant_handlers::get_merchant_profile))
        .route("/api/v1/merchants/status", get(merchant_handlers::get_merchant_readiness))
        .route("/api/v1/merchants/environment/switch", post(merchant_handlers::switch_environment))
        .route("/api/v1/merchants/api-keys/generate", post(merchant_handlers::generate_api_key))
        .route("/api/v1/merchants/api-keys/rotate", post(merchant_handlers::rotate_api_key))
        .route("/api/v1/merchants/settings", get(merchant_handlers::get_merchant_settings).patch(merchant_handlers::update_merchant_settings))
        .route("/api/v1/merchants/webhook/test", post(merchant_handlers::send_test_webhook))
        
        // Payment management
        .route("/api/v1/merchants/payments", post(merchant_handlers::create_payment))
        .route("/api/v1/merchants/payments", get(merchant_handlers::list_payments))
        .route("/api/v1/merchants/payments/:payment_id", get(merchant_handlers::get_payment))
        .route("/api/v1/merchants/payments/:payment_id/cancel", post(merchant_handlers::cancel_payment))
        .route("/api/v1/merchants/payments/:payment_id/verify", post(merchant_handlers::verify_payment))
        .route("/api/v1/merchants/payments/:payment_id/select", post(merchant_handlers::finalize_payment_selection))
        
        // Refund management
        .route("/api/v1/merchants/refunds", post(merchant_handlers::create_refund))
        .route("/api/v1/merchants/refunds/:refund_id", get(merchant_handlers::get_refund))
        .route("/api/v1/merchants/refunds/:refund_id/complete", post(merchant_handlers::complete_refund))
        
        // Analytics and reporting
        .route("/api/v1/merchants/analytics", get(merchant_handlers::get_analytics))
        .route("/api/v1/merchants/transactions", get(merchant_handlers::list_unified_transactions))
        .route("/api/v1/merchants/analytics/export", get(merchant_handlers::export_analytics))
        .route("/api/v1/merchants/audit-logs", get(merchant_handlers::get_audit_logs))

        // Fee Settings (GET only — updates via PATCH /settings)
        .route("/api/v1/merchants/fee-setting", get(merchant_handlers::get_fee_setting))
        
        // Balance and financial
        .route("/api/v1/merchants/balance", get(merchant_handlers::get_balance))
        .route("/api/v1/merchants/balance/history", get(merchant_handlers::get_balance_history))
        
        // Withdrawal management
        .route("/api/v1/merchants/withdrawals", post(merchant_handlers::create_withdrawal))
        .route("/api/v1/merchants/withdrawals", get(merchant_handlers::list_withdrawals))
        .route("/api/v1/merchants/withdrawals/:withdrawal_id", get(merchant_handlers::get_withdrawal))
        .route("/api/v1/merchants/withdrawals/:withdrawal_id/cancel", post(merchant_handlers::cancel_withdrawal))
        .route("/api/v1/merchants/withdrawals/:withdrawal_id/process", post(wallet_management::process_withdrawal))
        
        // Wallet management (unified setup via POST /wallets with mode field)
        .route("/api/v1/merchants/wallets/balances", get(wallet_management::get_wallet_balances))
        .route("/api/v1/merchants/wallets", get(wallet_management::get_wallets).post(wallet_management::setup_wallet))
        .route("/api/v1/merchants/wallets/export-key", post(wallet_management::export_private_key))
        .route("/api/v1/merchants/wallets/:crypto_type", axum::routing::delete(wallet_management::delete_wallet))
        .route("/api/v1/merchants/wallets/gas-check", get(wallet_management::check_gas_requirements))
        .route("/api/v1/merchants/wallets/gas-estimates", get(wallet_management::get_gas_estimates))
        .route("/api/v1/merchants/wallets/withdrawal-capability/:crypto_type", get(wallet_management::check_withdrawal_capability))
        
        // Security settings (merchant's own security preferences)
        .route("/api/v1/merchants/security/settings", get(security_monitoring::get_security_settings).put(security_monitoring::update_security_settings))
        .route("/api/v1/merchants/security/events", get(security_monitoring::get_security_events))
        .route("/api/v1/merchants/security/alerts", get(security_monitoring::get_security_alerts))
        .route("/api/v1/merchants/security/alerts/:alert_id/acknowledge", post(security_monitoring::acknowledge_security_alert))
        .route("/api/v1/merchants/security/balance-alerts", get(security_monitoring::get_balance_alerts))
        .route("/api/v1/merchants/security/balance-alerts/:alert_id/resolve", post(security_monitoring::resolve_balance_alert))
        .route("/api/v1/merchants/security/gas-check", get(security_monitoring::check_gas_balances))
        
        // IP whitelist management (GET only — updates via PATCH /settings)
        .route("/api/v1/merchants/ip-whitelist", get(merchant_handlers::get_ip_whitelist))
        
        // Customer management (Sub-Account Designated Wallets)
        .route("/api/v1/merchants/customers", get(crate::api::customer_handlers::list_customers).post(crate::api::customer_handlers::register_customer))
        .route("/api/v1/merchants/customers/:external_id/wallets", get(crate::api::customer_handlers::get_customer_wallets).post(crate::api::customer_handlers::provision_customer_wallets))
        .route("/api/v1/merchants/customers/:external_id/balances", get(crate::api::customer_handlers::get_customer_balances))
        .route("/api/v1/merchants/customers/:external_id/withdraw", post(crate::api::customer_handlers::withdraw_from_customer))
        .route("/api/v1/merchants/customers/:external_id/sweep", post(crate::api::customer_handlers::sweep_customer_wallet))
        .route("/api/v1/merchants/customers/:external_id/deactivate", post(crate::api::customer_handlers::deactivate_customer))
        
        // Invoice management
        .route("/api/v1/merchants/invoices", post(merchant_handlers::create_invoice))
        .route("/api/v1/merchants/invoices", get(merchant_handlers::list_invoices))
        .route("/api/v1/merchants/invoices/:invoice_id", get(merchant_handlers::get_invoice))
        
        // Sandbox testing
        .route("/api/v1/merchants/sandbox/enable", post(merchant_handlers::enable_sandbox))
        .route("/api/v1/merchants/sandbox/payments/:payment_id/simulate", post(merchant_handlers::simulate_payment))
        
        // Apply merchant API key authentication
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            auth::auth_middleware,
        ))
        .with_state(state)
}
