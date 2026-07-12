// Delora Error Types
// Delora-specific error handling

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum DeloraError {
    #[error("Circuit breaker is open — cross-chain swaps temporarily unavailable")]
    CircuitBreakerOpen,

    #[error("Delora API rate limited, retry after {retry_after_secs}s")]
    RateLimited { retry_after_secs: u64 },

    #[error("HTTP transport error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Delora API error {status}: {code} - {message}")]
    ApiError {
        status: u16,
        code: String,
        message: String,
    },

    #[error("No routes found for the requested swap")]
    NoRoutesFound,

    #[error("Route has no executable steps")]
    NoStepsInRoute,

    #[error("Quote not found: {0}")]
    QuoteNotFound(Uuid),

    #[error("Quote has expired, please request a new one")]
    QuoteExpired,

    #[error("Sender address mismatch: expected the wallet that requested the quote")]
    SenderMismatch,

    #[error("Transaction already registered for this cross-chain payment")]
    TransactionAlreadyRegistered,

    #[error("Invalid router contract: got {got} for chain {chain_id}")]
    InvalidRouterContract { got: String, chain_id: u64 },

    #[error("Invalid calldata: {0}")]
    InvalidCalldata(String),

    #[error("Amount exceeds maximum: ${amount} > ${max}")]
    AmountExceedsLimit { amount: f64, max: f64 },

    #[error("Amount below minimum: ${amount} < ${min}")]
    AmountBelowLimit { amount: f64, min: f64 },

    #[error("Payment link not found: {0}")]
    PaymentLinkNotFound(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Configuration error: {0}")]
    Config(String),
}

#[derive(Debug, Serialize)]
pub struct DeloraErrorResponse {
    pub error: DeloraErrorDetail,
}

#[derive(Debug, Serialize)]
pub struct DeloraErrorDetail {
    pub code: String,
    pub message: String,
    pub request_id: String,
}

impl IntoResponse for DeloraError {
    fn into_response(self) -> Response {
        let (status, code, message) = match &self {
            DeloraError::CircuitBreakerOpen => (
                StatusCode::SERVICE_UNAVAILABLE,
                "CIRCUIT_BREAKER_OPEN",
                "Cross-chain swap service temporarily unavailable",
            ),
            DeloraError::RateLimited { .. } => (
                StatusCode::TOO_MANY_REQUESTS,
                "RATE_LIMITED",
                "Too many swap requests, please try again shortly",
            ),
            DeloraError::ApiError {
                status: s,
                code: c,
                message: m,
            } => (
                StatusCode::from_u16(*s).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
                c.as_str(),
                m.as_str(),
            ),
            DeloraError::NoRoutesFound => (
                StatusCode::BAD_REQUEST,
                "NO_ROUTES_FOUND",
                "No swap route available for this token pair",
            ),
            DeloraError::QuoteNotFound(_) => (
                StatusCode::NOT_FOUND,
                "QUOTE_NOT_FOUND",
                "Quote not found or has expired",
            ),
            DeloraError::QuoteExpired => (
                StatusCode::GONE,
                "QUOTE_EXPIRED",
                "Quote has expired, please request a new one",
            ),
            DeloraError::SenderMismatch => (
                StatusCode::FORBIDDEN,
                "SENDER_MISMATCH",
                "Sender address does not match the address that requested the quote",
            ),
            DeloraError::TransactionAlreadyRegistered => (
                StatusCode::CONFLICT,
                "TX_ALREADY_REGISTERED",
                "This transaction has already been registered",
            ),
            DeloraError::InvalidRouterContract { .. } => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INVALID_ROUTER_CONTRACT",
                "Internal routing error",
            ),
            DeloraError::InvalidCalldata(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INVALID_CALLDATA",
                "Internal transaction construction error",
            ),
            DeloraError::AmountExceedsLimit { .. } => (
                StatusCode::BAD_REQUEST,
                "AMOUNT_EXCEEDS_LIMIT",
                "Swap amount exceeds maximum allowed",
            ),
            DeloraError::AmountBelowLimit { .. } => (
                StatusCode::BAD_REQUEST,
                "AMOUNT_BELOW_LIMIT",
                "Swap amount below minimum allowed",
            ),
            DeloraError::PaymentLinkNotFound(_) => (
                StatusCode::NOT_FOUND,
                "PAYMENT_LINK_NOT_FOUND",
                "Payment link not found",
            ),
            DeloraError::Database(e) => {
                tracing::error!("Delora DB error: {:?}", e);
                let msg = e.to_string();
                if msg.contains("unique constraint") || msg.contains("duplicate key") {
                    (
                        StatusCode::CONFLICT,
                        "ALREADY_EXISTS",
                        "Resource already exists",
                    )
                } else {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "DATABASE_ERROR",
                        "Internal server error",
                    )
                }
            }
            DeloraError::Http(e) => {
                tracing::error!("Delora HTTP error: {:?}", e);
                (
                    StatusCode::BAD_GATEWAY,
                    "SWAP_SERVICE_UNAVAILABLE",
                    "Cross-chain swap service is temporarily unreachable",
                )
            }
            DeloraError::Redis(e) => {
                tracing::error!("Delora Redis error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "CACHE_ERROR",
                    "Internal cache error",
                )
            }
            DeloraError::Serialization(e) => {
                tracing::error!("Delora serialization error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL_ERROR",
                    "Internal processing error",
                )
            }
            DeloraError::NoStepsInRoute | DeloraError::Config(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal configuration error",
            ),
        };

        let body = DeloraErrorResponse {
            error: DeloraErrorDetail {
                code: code.to_string(),
                message: message.to_string(),
                request_id: Uuid::new_v4().to_string(),
            },
        };

        (status, Json(body)).into_response()
    }
}
