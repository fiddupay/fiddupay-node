// Admin Routes
// Separate admin routing with session-based authentication

use crate::api::admin_handlers;
use crate::middleware::admin_auth;
use axum::{
    middleware as axum_middleware,
    routing::{get, post, put, delete},
    Router,
};
use crate::api::state::AppState;

pub fn create_admin_router(state: AppState) -> Router<AppState> {
    // Public admin routes (no auth required)
    let public_admin_routes = Router::new()
        .route("/api/v1/admin/login", post(admin_handlers::admin_login));

    // Protected admin routes (session auth required)
    let protected_admin_routes = Router::new()
        .route("/api/v1/admin/logout", post(admin_handlers::admin_logout))
        .route("/api/v1/admin/dashboard", get(admin_handlers::get_admin_dashboard))
        .route("/api/v1/admin/merchants", get(admin_handlers::get_merchants_summary))
        .route("/api/v1/admin/merchants/:merchant_id", get(admin_handlers::get_merchant_details))
        .route("/api/v1/admin/merchants/:merchant_id/suspend", post(admin_handlers::suspend_merchant))
        .route("/api/v1/admin/merchants/:merchant_id/activate", post(admin_handlers::activate_merchant))
        .route("/api/v1/admin/merchants/:merchant_id/delete", delete(admin_handlers::delete_merchant))
        
        // Admin Security Management
        .route("/api/v1/admin/security/events", get(admin_handlers::get_security_events))
        .route("/api/v1/admin/security/alerts", get(admin_handlers::get_security_alerts))
        .route("/api/v1/admin/security/alerts/:alert_id/acknowledge", post(admin_handlers::acknowledge_alert))
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
        
        // Apply admin session authentication
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            admin_auth::admin_auth_middleware,
        ));

    public_admin_routes
        .merge(protected_admin_routes)
        .with_state(state)
}
