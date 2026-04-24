// Global Request Logger Middleware
// Produces one structured log line per API request with:
// - HTTP method, path, status code, latency
// - Client IP (masked for PII compliance)
// - Merchant ID (if authenticated)
// - Log-level routing: 2xx=INFO, 4xx=WARN, 5xx=ERROR

use crate::middleware::auth::MerchantContext;
use crate::utils::sanitizer::mask_ip;
use axum::{extract::Request, middleware::Next, response::Response};
use std::time::Instant;

/// Paths to exclude from request logging to reduce noise
const SKIP_PATHS: &[&str] = &[
    "/health",
    "/solana-sol-logo.png",
    "/binance-usd-busd-logo.png",
    "/favicon.ico",
];

/// Global request logging middleware
///
/// Captures timing, client IP, merchant context, and status code for every request.
/// Automatically masks client IPs and routes log levels by response status.
pub async fn request_logger(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let path = request.uri().path().to_string();
    let query = request
        .uri()
        .query()
        .map(|q| format!("?{}", q))
        .unwrap_or_default();

    // Skip noisy endpoints
    if SKIP_PATHS.iter().any(|&skip| path == skip) {
        return next.run(request).await;
    }

    // Extract client IP from x-forwarded-for or ConnectInfo
    let client_ip = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.split(',').next())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| {
            request
                .extensions()
                .get::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
                .map(|ci| ci.0.ip().to_string())
                .unwrap_or_else(|| "unknown".to_string())
        });

    // Extract merchant ID if auth middleware has already run
    let merchant_id = request
        .extensions()
        .get::<MerchantContext>()
        .map(|ctx| ctx.merchant_id);

    // Start timing
    let start = Instant::now();

    // Process the request
    let response = next.run(request).await;

    // Calculate latency
    let latency = start.elapsed();
    let latency_ms = latency.as_millis();

    // Extract status
    let status = response.status();
    let status_code = status.as_u16();

    // Build the log line
    let masked_ip = mask_ip(&client_ip);
    let merchant_suffix = merchant_id
        .map(|id| format!(" | MID: {}", id))
        .unwrap_or_default();

    let log_line = format!(
        "{} {}{} → {} ({}ms) | IP: {}{}",
        method, path, query, status_code, latency_ms, masked_ip, merchant_suffix
    );

    // Route to appropriate log level based on status code
    if status_code >= 500 {
        tracing::error!("{}", log_line);
    } else if status_code >= 400 {
        tracing::warn!("{}", log_line);
    } else {
        tracing::info!("{}", log_line);
    }

    response
}
