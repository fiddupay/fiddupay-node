use axum::{
    middleware as axum_middleware,
    routing::{get, post},
    Router,
};

use crate::api::p2p_handlers;
use crate::api::p2p_ws;
use crate::api::state::AppState;
use crate::middleware::auth;

pub fn create_p2p_router(state: AppState) -> Router<AppState> {
    Router::new()
        // Profile & Balances
        .route("/api/v1/p2p/profile", get(p2p_handlers::get_p2p_profile))
        .route("/api/v1/p2p/balance/:crypto_type", get(p2p_handlers::get_p2p_balance))
        
        // Ads (Authenticated)
        .route("/api/v1/p2p/ads", post(p2p_handlers::create_p2p_ad))
        
        // Trades (Authenticated)
        .route("/api/v1/p2p/trades", post(p2p_handlers::create_p2p_trade))
        .route("/api/v1/p2p/trades/:trade_id/release", post(p2p_handlers::release_p2p_trade))
        .route("/api/v1/p2p/trades/:trade_id/rating", post(p2p_handlers::submit_p2p_rating))
        
        // Support
        .route("/api/v1/p2p/support/tickets", post(p2p_handlers::create_p2p_support_ticket))
        
        // WebSockets (Real-time Chat & Status)
        .route("/api/v1/p2p/ws/trades/:trade_id", axum::routing::get(p2p_ws::p2p_trade_ws_handler))
        
        // Require Authentication
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            auth::auth_middleware,
        ))
        
        // Public / Unauthenticated Ad Browsing
        .route("/api/v1/p2p/public/ads/:fiat_currency/:crypto_type/:ad_type", get(p2p_handlers::list_p2p_ads))
        
        .with_state(state)
}
