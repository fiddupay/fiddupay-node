// Delora API Handlers
// Axum handler functions for cross-chain payment endpoints

use crate::api::state::AppState;
use crate::delora::error::DeloraError;
use axum::{
    extract::{Json, Path, Query, State},
    http::HeaderMap,
    response::IntoResponse,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CrossChainQuoteQuery {
    pub link_id: String,
    pub sender_address: String,
    pub origin_chain_id: u64,
    pub origin_currency: String,
}

fn extract_ip(headers: &HeaderMap) -> String {
    headers
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.split(',').next())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

/// GET /api/v1/payments/cross-chain-quote
pub async fn get_cross_chain_quote(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<CrossChainQuoteQuery>,
) -> Result<impl IntoResponse, DeloraError> {
    if !state.config.delora.enabled {
        return Err(DeloraError::Config(
            "Delora integration is not enabled".into(),
        ));
    }

    if params.sender_address.trim().is_empty() {
        return Err(DeloraError::InvalidCalldata(
            "sender_address required".into(),
        ));
    }

    // Per-endpoint rate limit: 30 quotes/min per IP (protects our Delora API quota)
    if let Some(ref limiter) = state.delora_rate_limiter {
        let ip = extract_ip(&headers);
        limiter.check_quote(&ip).await.map_err(|e| match e {
            crate::delora::rate_limiter::DeloraRateLimitError::Banned => {
                DeloraError::Config("Access temporarily suspended".into())
            }
            _ => DeloraError::Config("Too many quote requests. Please wait.".into()),
        })?;
    }

    let quote = state
        .delora_service
        .get_cross_chain_quote(
            &params.link_id,
            &params.sender_address,
            params.origin_chain_id,
            &params.origin_currency,
        )
        .await?;

    Ok(Json(quote))
}

#[derive(Debug, Deserialize)]
pub struct RegisterCrossChainRequest {
    pub quote_id: uuid::Uuid,
    pub tx_hash: String,
    pub sender_address: String,
}

/// POST /api/v1/payments/cross-chain-register
pub async fn register_cross_chain_tx(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<RegisterCrossChainRequest>,
) -> Result<impl IntoResponse, DeloraError> {
    if !state.config.delora.enabled {
        return Err(DeloraError::Config(
            "Delora integration is not enabled".into(),
        ));
    }

    // Per-endpoint rate limit: 10 registrations/min per IP
    if let Some(ref limiter) = state.delora_rate_limiter {
        let ip = extract_ip(&headers);
        limiter.check_register(&ip).await.map_err(|e| match e {
            crate::delora::rate_limiter::DeloraRateLimitError::Banned => {
                DeloraError::Config("Access temporarily suspended".into())
            }
            _ => DeloraError::Config("Too many registration requests. Please wait.".into()),
        })?;
    }

    let status = state
        .delora_service
        .register_cross_chain_tx(&payload.quote_id, &payload.tx_hash, &payload.sender_address)
        .await?;

    Ok(Json(status))
}

/// GET /api/v1/payments/cross-chain-status/:link_id
pub async fn get_cross_chain_status(
    State(state): State<AppState>,
    Path(link_id): Path<String>,
) -> Result<impl IntoResponse, DeloraError> {
    if !state.config.delora.enabled {
        return Err(DeloraError::Config(
            "Delora integration is not enabled".into(),
        ));
    }

    let status = state
        .delora_service
        .get_cross_chain_status(&link_id)
        .await?;
    Ok(Json(status))
}

/// GET /api/v1/payments/cross-chain/chains
pub async fn get_supported_chains(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, DeloraError> {
    if !state.config.delora.enabled {
        return Err(DeloraError::Config(
            "Delora integration is not enabled".into(),
        ));
    }

    let chains = state.delora_service.get_supported_chains().await?;
    Ok(Json(chains))
}

/// GET /api/v1/payments/cross-chain/tokens/:chain_id
pub async fn get_supported_tokens(
    State(state): State<AppState>,
    Path(chain_id): Path<u64>,
) -> Result<impl IntoResponse, DeloraError> {
    if !state.config.delora.enabled {
        return Err(DeloraError::Config(
            "Delora integration is not enabled".into(),
        ));
    }

    let tokens = state.delora_service.get_supported_tokens(chain_id).await?;
    Ok(Json(tokens))
}
