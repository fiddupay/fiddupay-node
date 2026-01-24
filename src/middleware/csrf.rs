// CSRF Protection Middleware
// Prevents Cross-Site Request Forgery attacks

use axum::{
    extract::{Request, State},
    http::{HeaderMap, Method, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use uuid::Uuid;

/// CSRF token storage
pub struct CsrfTokenStore {
    tokens: RwLock<HashMap<String, String>>, // api_key -> csrf_token
}

impl CsrfTokenStore {
    pub fn new() -> Self {
        Self {
            tokens: RwLock::new(HashMap::new()),
        }
    }

    /// Generate CSRF token for API key
    pub async fn generate_token(&self, api_key: &str) -> String {
        let token = Uuid::new_v4().to_string();
        let mut tokens = self.tokens.write().await;
        tokens.insert(api_key.to_string(), token.clone());
        token
    }

    /// Validate CSRF token
    pub async fn validate_token(&self, api_key: &str, token: &str) -> bool {
        let tokens = self.tokens.read().await;
        tokens.get(api_key).map_or(false, |stored| stored == token)
    }

    /// Remove token after use
    pub async fn consume_token(&self, api_key: &str) {
        let mut tokens = self.tokens.write().await;
        tokens.remove(api_key);
    }
}

/// CSRF protection middleware
pub async fn csrf_middleware(
    State(csrf_store): State<Arc<CsrfTokenStore>>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, impl IntoResponse> {
    // Only check CSRF for state-changing methods
    if !matches!(request.method(), &Method::POST | &Method::PUT | &Method::DELETE) {
        return Ok(next.run(request).await);
    }

    // Extract API key from Authorization header
    let api_key = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|auth| {
            if auth.starts_with("Bearer ") {
                Some(auth[7..].to_string())
            } else {
                None
            }
        });

    // Extract CSRF token from header
    let csrf_token = headers
        .get("x-csrf-token")
        .and_then(|value| value.to_str().ok());

    match (api_key, csrf_token) {
        (Some(key), Some(token)) => {
            if csrf_store.validate_token(&key, token).await {
                csrf_store.consume_token(&key).await;
                Ok(next.run(request).await)
            } else {
                Err((
                    StatusCode::FORBIDDEN,
                    axum::Json(json!({
                        "error": "Invalid CSRF token",
                        "message": "CSRF token is missing or invalid"
                    }))
                ))
            }
        }
        _ => Err((
            StatusCode::FORBIDDEN,
            axum::Json(json!({
                "error": "CSRF protection",
                "message": "X-CSRF-Token header required for state-changing operations"
            }))
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_csrf_token_generation() {
        let store = CsrfTokenStore::new();
        let token = store.generate_token("test_key").await;
        
        assert!(!token.is_empty());
        assert!(store.validate_token("test_key", &token).await);
        assert!(!store.validate_token("test_key", "invalid").await);
    }

    #[tokio::test]
    async fn test_token_consumption() {
        let store = CsrfTokenStore::new();
        let token = store.generate_token("test_key").await;
        
        assert!(store.validate_token("test_key", &token).await);
        
        store.consume_token("test_key").await;
        assert!(!store.validate_token("test_key", &token).await);
    }
}
