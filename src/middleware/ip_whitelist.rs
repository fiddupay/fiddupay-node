// IP Whitelist Middleware
// Restricts API access to whitelisted IPs

use crate::api::state::AppState;
use crate::middleware::auth::MerchantContext;
use axum::{
    extract::{ConnectInfo, Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde_json::json;
use std::net::SocketAddr;

/// IP whitelist middleware
/// 
/// # Requirements
/// * 18.2: Reject requests from non-whitelisted IPs when enabled
/// * 18.3: Return 403 for non-whitelisted IPs
/// * 18.7: Allow all IPs when whitelist is empty
pub async fn ip_whitelist_middleware(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request,
    next: Next,
) -> Result<Response, impl IntoResponse> {
    // Get merchant context (must run after auth middleware)
    let merchant_context = request.extensions().get::<MerchantContext>().cloned();
    
    if let Some(context) = merchant_context {
        // Get IP whitelist for merchant
        let whitelist = match sqlx::query!(
            "SELECT ip_address FROM ip_whitelist WHERE merchant_id = $1 AND is_active = true",
            context.merchant_id
        )
        .fetch_all(&state.db_pool)
        .await
        {
            Ok(ips) => ips,
            Err(e) => {
                tracing::error!("Failed to fetch IP whitelist: {}", e);
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    axum::Json(json!({
                        "error": "Internal server error",
                        "message": "Failed to check IP whitelist"
                    }))
                ));
            }
        };

        // If whitelist is empty, allow all IPs
        if whitelist.is_empty() {
            return Ok(next.run(request).await);
        }

        // Check if request IP is in whitelist
        let request_ip = addr.ip().to_string();
        let is_whitelisted = whitelist.iter().any(|entry| {
            // Simple IP matching (could be enhanced with CIDR support)
            entry.ip_address == request_ip
        });

        if !is_whitelisted {
            // Log rejected request
            tracing::warn!(
                "IP {} rejected for merchant {} (not in whitelist)",
                request_ip,
                context.merchant_id
            );

            return Err((
                StatusCode::FORBIDDEN,
                axum::Json(json!({
                    "error": "IP not whitelisted",
                    "message": "Your IP address is not authorized to access this resource"
                }))
            ));
        }
    }

    // IP is whitelisted or no merchant context, continue
    Ok(next.run(request).await)
}
