// Admin Routes
// Separate admin routing with session-based authentication

use crate::api::admin;
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
        .route("/api/v1/admin/login", post(admin::admin_login));

    // Protected admin routes (session auth required)
    let protected_admin_routes = Router::new()
        .route("/api/v1/admin/logout", post(admin::admin_logout))
        .route("/api/v1/admin/dashboard", get(admin::get_admin_dashboard))
        .route("/api/v1/admin/merchants", get(admin::get_merchants_summary))
        .route("/api/v1/admin/merchants/:merchant_id", get(admin::get_merchant_details))
        .route("/api/v1/admin/merchants/:merchant_id/suspend", post(admin::suspend_merchant))
        .route("/api/v1/admin/merchants/:merchant_id/activate", post(admin::activate_merchant))
        .route("/api/v1/admin/merchants/:merchant_id/delete", delete(admin::delete_merchant))
        .route("/api/v1/admin/merchants/:merchant_id/fee", put(admin::update_merchant_fee))
        
        // Admin Security Management
        .route("/api/v1/admin/security/events", get(admin::get_security_events))
        .route("/api/v1/admin/security/alerts", get(admin::get_security_alerts))
        .route("/api/v1/admin/security/alerts/:alert_id/acknowledge", post(admin::acknowledge_alert))
        .route("/api/v1/admin/security/settings", get(admin::get_security_settings))
        
        // Unified Admin Configuration (single PATCH for all config updates)
        .route("/api/v1/admin/config", axum::routing::patch(admin::update_admin_config))
        .route("/api/v1/admin/config/fees", get(admin::get_fee_config))
        .route("/api/v1/admin/config/limits", get(admin::get_system_limits))
        
        // Smart Fee Sweeping
        .route("/api/v1/admin/fee-sweep/settings", get(admin::get_fee_sweep_settings))
        .route("/api/v1/admin/fee-sweep/settings", put(admin::update_fee_sweep_settings))
        .route("/api/v1/admin/fee-sweep/trigger/:network", post(admin::trigger_manual_sweep))
        
        // Admin Payment Management
        .route("/api/v1/admin/payments", get(admin::get_all_payments))
        .route("/api/v1/admin/payments/:payment_id", get(admin::get_payment_details))
        .route("/api/v1/admin/payments/:payment_id/force-confirm", post(admin::force_confirm_payment))
        .route("/api/v1/admin/payments/:payment_id/force-fail", post(admin::force_fail_payment))
        .route("/api/v1/admin/transactions/reverify", post(admin::reverify_transaction))
        
        // Admin Withdrawal Management
        .route("/api/v1/admin/withdrawals", get(admin::get_all_withdrawals))
        .route("/api/v1/admin/withdrawals/:withdrawal_id/approve", post(admin::approve_withdrawal))
        .route("/api/v1/admin/withdrawals/:withdrawal_id/reject", post(admin::reject_withdrawal))
        .route("/api/v1/admin/withdrawals/:withdrawal_id/resolve-failed-refund", post(admin::resolve_failed_refund))
        
        // Admin Analytics & Reporting
        .route("/api/v1/admin/analytics/platform", get(admin::get_platform_analytics))
        .route("/api/v1/admin/analytics/revenue", get(admin::get_revenue_analytics))
        .route("/api/v1/admin/reports/transactions", get(admin::get_transaction_reports))
        .route("/api/v1/admin/reports/merchants", get(admin::get_merchant_reports))
        .route("/api/v1/admin/audit-logs", get(admin::get_admin_audit_logs))
        
        // Admin Wallet Management (unified view with query params)
        .route("/api/v1/admin/wallets", get(admin::get_all_wallets))
        .route("/api/v1/admin/wallets/transfer", post(admin::transfer_funds))
        
        // Admin User Management
        .route("/api/v1/admin/users", get(admin::get_admin_users))
        .route("/api/v1/admin/users", post(admin::create_admin_user))
        .route("/api/v1/admin/users/:user_id", delete(admin::delete_admin_user))
        .route("/api/v1/admin/users/:user_id/permissions", put(admin::update_user_permissions))
        
        // Admin System Maintenance
        .route("/api/v1/admin/system/health", get(admin::get_system_health))
        .route("/api/v1/admin/system/logs", get(admin::get_system_logs))
        .route("/api/v1/admin/system/backup", post(admin::create_system_backup))
        .route("/api/v1/admin/system/maintenance", post(admin::toggle_maintenance_mode))
        
        // Apply admin session authentication
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            admin_auth::admin_auth_middleware,
        ));

    public_admin_routes
        .merge(protected_admin_routes)
        .with_state(state)
}
