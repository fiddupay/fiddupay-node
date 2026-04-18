// Idempotency Key Middleware
// Prevents duplicate processing of the same mutating request.
// Follows the Stripe idempotency model: hash(merchant_id + key + endpoint) → cached response.

use axum::{
    body::Body,
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use sha2::{Digest, Sha256};
use sqlx::{PgPool, Row};
use tracing::info;

/// Extract the idempotency key from request headers, check the database,
/// and either return the cached response or let the request proceed.
pub async fn idempotency_layer(
    State(db_pool): State<PgPool>,
    headers: HeaderMap,
    request: Request<Body>,
    next: Next,
) -> Response {
    // Only apply idempotency to requests that carry the header
    let idempotency_key = match headers
        .get("idempotency-key")
        .or(headers.get("Idempotency-Key"))
    {
        Some(val) => match val.to_str() {
            Ok(s) if !s.is_empty() => s.to_string(),
            _ => return next.run(request).await,
        },
        None => return next.run(request).await,
    };

    // Extract merchant_id from the auth extension (set by auth middleware)
    let merchant_id: i64 = request.extensions().get::<i64>().copied().unwrap_or(0);

    let endpoint = request.uri().path().to_string();

    // Build a unique hash: merchant_id + idempotency_key + endpoint
    let mut hasher = Sha256::new();
    hasher.update(format!("{}:{}:{}", merchant_id, idempotency_key, endpoint));
    let key_hash = hex::encode(hasher.finalize());

    // 1. Check if this key already has a cached response
    let existing = sqlx::query(
        "SELECT response_code, response_body FROM idempotency_keys WHERE key_hash = $1 AND expires_at > NOW()"
    )
    .bind(&key_hash)
    .fetch_optional(&db_pool)
    .await;

    if let Ok(Some(row)) = existing {
        let code: i16 = row.get("response_code");
        let body: serde_json::Value = row.get("response_body");

        info!(
            "Idempotency hit: returning cached response for key={} merchant={}",
            &idempotency_key, merchant_id
        );

        return (
            StatusCode::from_u16(code as u16).unwrap_or(StatusCode::OK),
            axum::Json(body),
        )
            .into_response();
    }

    // 2. Insert a placeholder to "lock" this key (prevents race conditions)
    let lock_result = sqlx::query(
        r#"
        INSERT INTO idempotency_keys (key_hash, merchant_id, endpoint)
        VALUES ($1, $2, $3)
        ON CONFLICT (key_hash) DO NOTHING
        "#,
    )
    .bind(&key_hash)
    .bind(merchant_id)
    .bind(&endpoint)
    .execute(&db_pool)
    .await;

    if let Ok(result) = &lock_result {
        if result.rows_affected() == 0 {
            // Another concurrent request already locked this key — return 409 Conflict
            return (
                StatusCode::CONFLICT,
                axum::Json(serde_json::json!({
                    "error": "A request with this idempotency key is already being processed"
                })),
            )
                .into_response();
        }
    }

    // 3. Let the actual handler execute
    let response = next.run(request).await;

    // 4. Cache the response for future idempotent replays
    let status_code = response.status().as_u16() as i16;

    // Read the body to cache it, then reconstruct the response
    let (parts, body) = response.into_parts();
    let body_bytes = match axum::body::to_bytes(body, 1_048_576).await {
        Ok(bytes) => bytes,
        Err(_) => {
            // If we can't read the body, delete the lock and return error
            let _ = sqlx::query("DELETE FROM idempotency_keys WHERE key_hash = $1")
                .bind(&key_hash)
                .execute(&db_pool)
                .await;
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to process response",
            )
                .into_response();
        }
    };

    // Try to parse as JSON for storage
    let response_json: serde_json::Value =
        serde_json::from_slice(&body_bytes).unwrap_or(serde_json::json!({"raw": true}));

    // Store the cached response
    let _ = sqlx::query(
        "UPDATE idempotency_keys SET response_code = $1, response_body = $2 WHERE key_hash = $3",
    )
    .bind(status_code)
    .bind(&response_json)
    .bind(&key_hash)
    .execute(&db_pool)
    .await;

    // Reconstruct the response
    Response::from_parts(parts, Body::from(body_bytes))
}
