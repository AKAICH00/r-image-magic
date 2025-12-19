//! API Key Management Handlers
//!
//! Endpoints for creating, listing, and managing API keys.
//! Admin endpoints require admin authentication.

use actix_web::{web, HttpRequest, HttpResponse, HttpMessage};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use uuid::Uuid;

use crate::db::{
    ApiKeyRepository, CreateApiKeyRequest, ApiKeyTier, DbPool,
};
use crate::api::middleware::ApiKeyAuth;

/// Request to create a new API key
#[derive(Debug, Deserialize)]
pub struct CreateKeyRequest {
    pub name: String,
    pub owner_email: String,
    #[serde(default)]
    pub owner_name: Option<String>,
    #[serde(default)]
    pub company: Option<String>,
    #[serde(default = "default_tier")]
    pub tier: String,
    #[serde(default)]
    pub rate_limit_per_minute: Option<i32>,
    #[serde(default)]
    pub monthly_quota: Option<i32>,
    #[serde(default)]
    pub expires_at: Option<DateTime<Utc>>,
}

fn default_tier() -> String {
    "free".to_string()
}

/// Response after creating a new API key
#[derive(Debug, Serialize)]
pub struct CreateKeyResponse {
    pub id: Uuid,
    pub api_key: String,  // Only shown once!
    pub key_prefix: String,
    pub name: String,
    pub tier: String,
    pub rate_limit_per_minute: i32,
    pub monthly_quota: i32,
    pub message: String,
}

/// API key info (without sensitive data)
#[derive(Debug, Serialize)]
pub struct ApiKeyInfo {
    pub id: Uuid,
    pub key_prefix: String,
    pub name: String,
    pub owner_email: String,
    pub owner_name: Option<String>,
    pub company: Option<String>,
    pub tier: String,
    pub rate_limit_per_minute: i32,
    pub monthly_quota: i32,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
}

/// List of API keys response
#[derive(Debug, Serialize)]
pub struct ListKeysResponse {
    pub keys: Vec<ApiKeyInfo>,
    pub count: usize,
}

/// Create a new API key
/// POST /api/v1/keys
///
/// Requires admin API key authentication
pub async fn create_api_key(
    req: HttpRequest,
    pool: web::Data<DbPool>,
    body: web::Json<CreateKeyRequest>,
) -> HttpResponse {
    // Check if requester is admin (enterprise tier)
    let auth = match req.extensions().get::<ApiKeyAuth>().cloned() {
        Some(auth) if auth.tier == "enterprise" => auth,
        Some(_) => {
            return HttpResponse::Forbidden().json(serde_json::json!({
                "error": "forbidden",
                "message": "Only enterprise tier keys can create new API keys"
            }));
        }
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "unauthorized",
                "message": "API key required"
            }));
        }
    };

    let repo = ApiKeyRepository::new(pool.get_ref().clone());

    let tier = ApiKeyTier::from_str(&body.tier);

    let request = CreateApiKeyRequest {
        name: body.name.clone(),
        owner_email: body.owner_email.clone(),
        owner_name: body.owner_name.clone(),
        company: body.company.clone(),
        tier,
        rate_limit_per_minute: body.rate_limit_per_minute,
        monthly_quota: body.monthly_quota,
        expires_at: body.expires_at,
    };

    match repo.create(request).await {
        Ok(response) => {
            info!(
                key_id = %response.id,
                key_prefix = %response.key_prefix,
                tier = %response.tier,
                created_by = %auth.key_id,
                "API key created"
            );

            HttpResponse::Created().json(CreateKeyResponse {
                id: response.id,
                api_key: response.api_key,
                key_prefix: response.key_prefix,
                name: response.name,
                tier: response.tier,
                rate_limit_per_minute: response.rate_limit_per_minute,
                monthly_quota: response.monthly_quota,
                message: "API key created successfully. Save the api_key value - it won't be shown again!".to_string(),
            })
        }
        Err(e) => {
            warn!(error = %e, "Failed to create API key");
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "internal_error",
                "message": "Failed to create API key"
            }))
        }
    }
}

/// Get current API key info
/// GET /api/v1/keys/me
pub async fn get_my_key(
    req: HttpRequest,
    pool: web::Data<DbPool>,
) -> HttpResponse {
    let auth = match req.extensions().get::<ApiKeyAuth>().cloned() {
        Some(auth) => auth,
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "unauthorized",
                "message": "API key required"
            }));
        }
    };

    let repo = ApiKeyRepository::new(pool.get_ref().clone());

    match repo.get_by_id(auth.key_id).await {
        Ok(Some(key)) => {
            HttpResponse::Ok().json(ApiKeyInfo {
                id: key.id,
                key_prefix: key.key_prefix,
                name: key.name,
                owner_email: key.owner_email,
                owner_name: key.owner_name,
                company: key.company,
                tier: key.tier,
                rate_limit_per_minute: key.rate_limit_per_minute,
                monthly_quota: key.monthly_quota,
                is_active: key.is_active,
                created_at: key.created_at,
                last_used_at: key.last_used_at,
                expires_at: key.expires_at,
            })
        }
        Ok(None) => {
            HttpResponse::NotFound().json(serde_json::json!({
                "error": "not_found",
                "message": "API key not found"
            }))
        }
        Err(e) => {
            warn!(error = %e, "Failed to get API key");
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "internal_error",
                "message": "Failed to get API key"
            }))
        }
    }
}

/// Get API key by ID (admin only)
/// GET /api/v1/keys/{id}
pub async fn get_key_by_id(
    req: HttpRequest,
    pool: web::Data<DbPool>,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let auth = match req.extensions().get::<ApiKeyAuth>().cloned() {
        Some(auth) if auth.tier == "enterprise" => auth,
        Some(_) => {
            return HttpResponse::Forbidden().json(serde_json::json!({
                "error": "forbidden",
                "message": "Only enterprise tier keys can view other keys"
            }));
        }
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "unauthorized",
                "message": "API key required"
            }));
        }
    };

    let key_id = path.into_inner();
    let repo = ApiKeyRepository::new(pool.get_ref().clone());

    match repo.get_by_id(key_id).await {
        Ok(Some(key)) => {
            HttpResponse::Ok().json(ApiKeyInfo {
                id: key.id,
                key_prefix: key.key_prefix,
                name: key.name,
                owner_email: key.owner_email,
                owner_name: key.owner_name,
                company: key.company,
                tier: key.tier,
                rate_limit_per_minute: key.rate_limit_per_minute,
                monthly_quota: key.monthly_quota,
                is_active: key.is_active,
                created_at: key.created_at,
                last_used_at: key.last_used_at,
                expires_at: key.expires_at,
            })
        }
        Ok(None) => {
            HttpResponse::NotFound().json(serde_json::json!({
                "error": "not_found",
                "message": "API key not found"
            }))
        }
        Err(e) => {
            warn!(error = %e, "Failed to get API key");
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "internal_error",
                "message": "Failed to get API key"
            }))
        }
    }
}

/// List API keys by owner email (admin only)
/// GET /api/v1/keys?owner_email=xxx
pub async fn list_keys(
    req: HttpRequest,
    pool: web::Data<DbPool>,
    query: web::Query<ListKeysQuery>,
) -> HttpResponse {
    let auth = match req.extensions().get::<ApiKeyAuth>().cloned() {
        Some(auth) if auth.tier == "enterprise" => auth,
        Some(auth) => {
            // Non-admin can only list their own keys
            let owner_email = auth.owner_email.clone();
            let repo = ApiKeyRepository::new(pool.get_ref().clone());

            match repo.list_by_owner(&owner_email).await {
                Ok(keys) => {
                    let key_infos: Vec<ApiKeyInfo> = keys.into_iter().map(|key| ApiKeyInfo {
                        id: key.id,
                        key_prefix: key.key_prefix,
                        name: key.name,
                        owner_email: key.owner_email,
                        owner_name: key.owner_name,
                        company: key.company,
                        tier: key.tier,
                        rate_limit_per_minute: key.rate_limit_per_minute,
                        monthly_quota: key.monthly_quota,
                        is_active: key.is_active,
                        created_at: key.created_at,
                        last_used_at: key.last_used_at,
                        expires_at: key.expires_at,
                    }).collect();

                    let count = key_infos.len();
                    return HttpResponse::Ok().json(ListKeysResponse { keys: key_infos, count });
                }
                Err(e) => {
                    warn!(error = %e, "Failed to list API keys");
                    return HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": "internal_error",
                        "message": "Failed to list API keys"
                    }));
                }
            }
        }
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "unauthorized",
                "message": "API key required"
            }));
        }
    };

    // Admin can list by any owner
    let owner_email = query.owner_email.as_ref().map(|s| s.as_str()).unwrap_or(&auth.owner_email);
    let repo = ApiKeyRepository::new(pool.get_ref().clone());

    match repo.list_by_owner(owner_email).await {
        Ok(keys) => {
            let key_infos: Vec<ApiKeyInfo> = keys.into_iter().map(|key| ApiKeyInfo {
                id: key.id,
                key_prefix: key.key_prefix,
                name: key.name,
                owner_email: key.owner_email,
                owner_name: key.owner_name,
                company: key.company,
                tier: key.tier,
                rate_limit_per_minute: key.rate_limit_per_minute,
                monthly_quota: key.monthly_quota,
                is_active: key.is_active,
                created_at: key.created_at,
                last_used_at: key.last_used_at,
                expires_at: key.expires_at,
            }).collect();

            let count = key_infos.len();
            HttpResponse::Ok().json(ListKeysResponse { keys: key_infos, count })
        }
        Err(e) => {
            warn!(error = %e, "Failed to list API keys");
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "internal_error",
                "message": "Failed to list API keys"
            }))
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListKeysQuery {
    pub owner_email: Option<String>,
}

/// Revoke an API key
/// DELETE /api/v1/keys/{id}
pub async fn revoke_key(
    req: HttpRequest,
    pool: web::Data<DbPool>,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let auth = match req.extensions().get::<ApiKeyAuth>().cloned() {
        Some(auth) => auth,
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "unauthorized",
                "message": "API key required"
            }));
        }
    };

    let key_id = path.into_inner();
    let repo = ApiKeyRepository::new(pool.get_ref().clone());

    // Check if user owns the key or is admin
    if auth.tier != "enterprise" {
        match repo.get_by_id(key_id).await {
            Ok(Some(key)) if key.owner_email == auth.owner_email => {
                // Owner can revoke their own key
            }
            Ok(Some(_)) => {
                return HttpResponse::Forbidden().json(serde_json::json!({
                    "error": "forbidden",
                    "message": "You can only revoke your own API keys"
                }));
            }
            Ok(None) => {
                return HttpResponse::NotFound().json(serde_json::json!({
                    "error": "not_found",
                    "message": "API key not found"
                }));
            }
            Err(e) => {
                warn!(error = %e, "Failed to check API key ownership");
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "internal_error",
                    "message": "Failed to revoke API key"
                }));
            }
        }
    }

    match repo.revoke(key_id).await {
        Ok(true) => {
            info!(key_id = %key_id, revoked_by = %auth.key_id, "API key revoked");
            HttpResponse::Ok().json(serde_json::json!({
                "message": "API key revoked successfully",
                "key_id": key_id
            }))
        }
        Ok(false) => {
            HttpResponse::NotFound().json(serde_json::json!({
                "error": "not_found",
                "message": "API key not found"
            }))
        }
        Err(e) => {
            warn!(error = %e, "Failed to revoke API key");
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "internal_error",
                "message": "Failed to revoke API key"
            }))
        }
    }
}
