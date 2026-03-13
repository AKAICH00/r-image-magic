//! API Key Authentication Middleware
//!
//! Validates API keys from the X-API-Key header or Authorization: Bearer token.
//! Stores authenticated key info in request extensions for downstream handlers.

use actix_web::{
    dev::{ServiceRequest, ServiceResponse},
    error::ErrorUnauthorized,
    http::header::{HeaderValue, AUTHORIZATION},
    Error, HttpMessage,
};
use std::sync::Arc;
use tracing::{info, warn};

use crate::db::{ApiKeyRepository, DbApiKey};

/// Extension type for storing authenticated API key in request
#[derive(Clone)]
pub struct AuthenticatedKey {
    pub key: DbApiKey,
}

/// Header name for API key
pub const API_KEY_HEADER: &str = "X-API-Key";

/// Extract API key from request headers
pub fn extract_api_key(req: &ServiceRequest) -> Option<String> {
    // First try X-API-Key header
    if let Some(key) = req.headers().get(API_KEY_HEADER) {
        if let Ok(key_str) = key.to_str() {
            return Some(key_str.to_string());
        }
    }

    // Then try Authorization: Bearer <key>
    if let Some(auth) = req.headers().get(AUTHORIZATION) {
        if let Ok(auth_str) = auth.to_str() {
            if auth_str.starts_with("Bearer ") {
                return Some(auth_str[7..].to_string());
            }
        }
    }

    None
}

/// Validate API key and return authenticated key info
pub async fn validate_api_key(
    api_key: &str,
    api_key_repo: &ApiKeyRepository,
) -> Result<DbApiKey, Error> {
    // Validate key format
    if !api_key.starts_with("rim_") || api_key.len() < 36 {
        warn!("Invalid API key format");
        return Err(ErrorUnauthorized("Invalid API key format"));
    }

    // Look up key in database
    match api_key_repo.validate(api_key).await {
        Ok(Some(key)) => {
            // Check if key is valid (active and not expired)
            if !key.is_valid() {
                warn!(key_id = %key.id, "API key is inactive or expired");
                return Err(ErrorUnauthorized("API key is inactive or expired"));
            }

            // Update last used timestamp (fire and forget)
            let key_id = key.id;
            let repo_clone = api_key_repo.pool.clone();
            tokio::spawn(async move {
                let repo = ApiKeyRepository::new(repo_clone);
                let _ = repo.touch(key_id).await;
            });

            info!(
                key_id = %key.id,
                key_prefix = %key.key_prefix,
                tier = %key.tier,
                "API key validated"
            );

            Ok(key)
        }
        Ok(None) => {
            warn!("API key not found");
            Err(ErrorUnauthorized("Invalid API key"))
        }
        Err(e) => {
            warn!(error = %e, "Failed to validate API key");
            Err(ErrorUnauthorized("Authentication failed"))
        }
    }
}

/// Authentication result for use in handlers
#[derive(Clone)]
pub struct ApiKeyAuth {
    pub key_id: uuid::Uuid,
    pub tier: String,
    pub rate_limit: i32,
    pub monthly_quota: i32,
    pub owner_email: String,
}

impl From<&DbApiKey> for ApiKeyAuth {
    fn from(key: &DbApiKey) -> Self {
        Self {
            key_id: key.id,
            tier: key.tier.clone(),
            rate_limit: key.rate_limit_per_minute,
            monthly_quota: key.monthly_quota,
            owner_email: key.owner_email.clone(),
        }
    }
}

/// Trait for adding authenticated key info to request extensions
pub trait ApiKeyExt {
    fn api_key(&self) -> Option<ApiKeyAuth>;
}

impl<T: HttpMessage> ApiKeyExt for T {
    fn api_key(&self) -> Option<ApiKeyAuth> {
        self.extensions().get::<ApiKeyAuth>().cloned()
    }
}
