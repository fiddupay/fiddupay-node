// Merchant Handlers
// Re-exports for merchant-specific request handlers used by merchant_routes.rs

// Auth handlers
pub use crate::api::merchant_auth_handlers::{login_merchant, register_merchant};

// Settings handlers
pub use crate::api::settings_handlers::{
    create_invoice, generate_api_key, get_fee_setting, get_invoice, get_ip_whitelist,
    get_merchant_profile, get_merchant_readiness, get_merchant_settings, list_invoices,
    rotate_api_key, send_test_webhook, set_transaction_pin, switch_environment,
    toggle_customer_wallet_lock, toggle_wallet_lock, update_merchant_settings,
    verify_transaction_pin,
};

// Payment handlers
pub use crate::api::payment_handlers::{
    cancel_payment, complete_refund, create_payment, create_refund, finalize_payment_selection,
    get_payment, get_refund, list_payments, list_refunds, verify_payment,
};

// Analytics handlers
pub use crate::api::analytics_handlers::{
    export_analytics, get_analytics, get_audit_logs, get_balance, get_balance_history,
    list_unified_transactions,
};

// Withdrawal handlers
pub use crate::api::withdrawal_handlers::{
    cancel_withdrawal, create_withdrawal, get_withdrawal, list_withdrawals,
};
