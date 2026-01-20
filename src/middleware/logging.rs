// Logging Middleware
// Logs all API requests

use crate::middleware::auth::MerchantContext;
use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};
use tracing::info;

/// Request logging middleware
/// 
/// # Requirements
/// * 7.7: Log all API requests with timestamp, endpoint, and merchant ID
pub async fn logging_middleware(
    request: Request,
    next: Next,
) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let merchant_id = request
        .extensions()
        .get::<MerchantContext>()
        .map(|ctx| ctx.merchant_id);

    // Log request
    info!(
        "API Request: {} {} (merchant: {:?})",
        method,
        uri,
        merchant_id
    );

    // Process request
    let response = next.run(request).await;

    // Log response status
    info!(
        "API Response: {} {} -> {}",
        method,
        uri,
        response.status()
    );

    response
}
