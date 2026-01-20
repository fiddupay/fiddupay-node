// Error Types
// Centralized error handling for the gateway

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Invalid API key")]
    InvalidApiKey,

    #[error("Payment not found")]
    PaymentNotFound,

    #[error("Merchant not found")]
    MerchantNotFound,

    #[error("Invalid wallet address: {0}")]
    InvalidWalletAddress(String),

    #[error("Wallet not found")]
    WalletNotFound,

    #[error("Invalid webhook URL: {0}")]
    InvalidWebhookUrl(String),

    #[error("Webhook delivery failed: {0}")]
    WebhookDeliveryFailed(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("IP not whitelisted")]
    IpNotWhitelisted,

    #[error("Invalid fee percentage: {0}")]
    InvalidFeePercentage(String),

    #[error("Refund not found")]
    RefundNotFound,

    #[error("Invalid refund amount: {0}")]
    InvalidRefundAmount(String),

    #[error("Internal server error: {0}")]
    Internal(String),
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: ErrorDetail,
}

#[derive(Debug, Serialize)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
    pub request_id: String,
}

impl IntoResponse for ServiceError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            ServiceError::InvalidApiKey => (
                StatusCode::UNAUTHORIZED,
                "INVALID_API_KEY",
                "Invalid or missing API key",
            ),
            ServiceError::PaymentNotFound => (
                StatusCode::NOT_FOUND,
                "PAYMENT_NOT_FOUND",
                "Payment not found",
            ),
            ServiceError::MerchantNotFound => (
                StatusCode::NOT_FOUND,
                "MERCHANT_NOT_FOUND",
                "Merchant not found",
            ),
            ServiceError::InvalidWalletAddress(ref msg) => (
                StatusCode::BAD_REQUEST,
                "INVALID_WALLET_ADDRESS",
                msg.as_str(),
            ),
            ServiceError::WalletNotFound => (
                StatusCode::NOT_FOUND,
                "WALLET_NOT_FOUND",
                "Wallet not found for this blockchain",
            ),
            ServiceError::InvalidWebhookUrl(ref msg) => (
                StatusCode::BAD_REQUEST,
                "INVALID_WEBHOOK_URL",
                msg.as_str(),
            ),
            ServiceError::WebhookDeliveryFailed(ref msg) => (
                StatusCode::BAD_GATEWAY,
                "WEBHOOK_DELIVERY_FAILED",
                msg.as_str(),
            ),
            ServiceError::RateLimitExceeded => (
                StatusCode::TOO_MANY_REQUESTS,
                "RATE_LIMIT_EXCEEDED",
                "Too many requests",
            ),
            ServiceError::IpNotWhitelisted => (
                StatusCode::FORBIDDEN,
                "IP_NOT_WHITELISTED",
                "IP address not whitelisted",
            ),
            ServiceError::InvalidFeePercentage(ref msg) => (
                StatusCode::BAD_REQUEST,
                "INVALID_FEE_PERCENTAGE",
                msg.as_str(),
            ),
            ServiceError::RefundNotFound => (
                StatusCode::NOT_FOUND,
                "REFUND_NOT_FOUND",
                "Refund not found",
            ),
            ServiceError::InvalidRefundAmount(ref msg) => (
                StatusCode::BAD_REQUEST,
                "INVALID_REFUND_AMOUNT",
                msg.as_str(),
            ),
            ServiceError::Database(_) | ServiceError::Internal(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_SERVER_ERROR",
                "Internal server error",
            ),
            ServiceError::Forbidden(ref msg) => (
                StatusCode::FORBIDDEN,
                "FORBIDDEN",
                msg.as_str(),
            ),
            ServiceError::NotFound(ref msg) => (
                StatusCode::NOT_FOUND,
                "NOT_FOUND",
                msg.as_str(),
            ),
        };

        let error_response = ErrorResponse {
            error: ErrorDetail {
                code: code.to_string(),
                message: message.to_string(),
                request_id: uuid::Uuid::new_v4().to_string(),
            },
        };

        (status, Json(error_response)).into_response()
    }
}
