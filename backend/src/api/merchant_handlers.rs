// Merchant Handlers
// All merchant-specific request handlers

use crate::api::state::AppState;
use crate::middleware::auth::MerchantContext;
use axum::{
    extract::{Path, Query, State, Extension},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};
use serde_json::json;

// Re-export merchant-specific handlers from main handlers
pub use crate::api::handlers::{
    // Profile management
    get_merchant_profile,
    switch_environment,
    generate_api_key,
    rotate_api_key,
    set_webhook,
    
    // Payment management
    create_payment,
    list_payments,
    get_payment,
    verify_payment,
    
    // Refund management
    create_refund,
    get_refund,
    complete_refund,
    
    // Analytics
    get_analytics,
    export_analytics,
    get_audit_logs,
    
    // Balance
    get_balance,
    get_balance_history,
    
    // Withdrawals
    create_withdrawal,
    list_withdrawals,
    get_withdrawal,
    cancel_withdrawal,
    
    // Wallet management
    set_wallet,
    
    // Security
    set_ip_whitelist,
    get_ip_whitelist,
    
    // Invoices
    create_invoice,
    list_invoices,
    get_invoice,
    
    // Sandbox
    enable_sandbox,
    simulate_payment,
    
    // Auth
    register_merchant,
    login_merchant,
};
