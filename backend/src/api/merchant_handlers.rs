// Merchant Handlers
// Re-exports for merchant-specific request handlers used by merchant_routes.rs

// Auth handlers
pub use crate::api::merchant_auth_handlers::{
    register_merchant,
    login_merchant,
    debug_auth,
};

// Settings handlers
pub use crate::api::settings_handlers::{
    get_merchant_profile,
    get_merchant_readiness,
    switch_environment,
    generate_api_key,
    rotate_api_key,
    set_wallet,
    update_merchant_settings,
    get_merchant_settings,
    toggle_wallet_lock,
    toggle_customer_wallet_lock,
    send_test_webhook,
    get_ip_whitelist,
    get_fee_setting,
    create_invoice,
    list_invoices,
    get_invoice,
};

// Payment handlers
pub use crate::api::payment_handlers::{
    create_payment,
    get_payment,
    cancel_payment,
    verify_payment,
    list_payments,
    finalize_payment_selection,
    create_refund,
    get_refund,
    complete_refund,
    enable_sandbox,
    simulate_payment,
};

// Analytics handlers
pub use crate::api::analytics_handlers::{
    get_analytics,
    export_analytics,
    get_audit_logs,
    get_balance,
    get_balance_history,
    list_unified_transactions,
};

// Withdrawal handlers
pub use crate::api::withdrawal_handlers::{
    create_withdrawal,
    get_withdrawal,
    list_withdrawals,
    cancel_withdrawal,
};
