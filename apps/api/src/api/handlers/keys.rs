//! API Key Management Handlers
//!
//! Endpoints for creating, listing, and managing API keys.
//! Admin endpoints require admin authentication.

use actix_web::{web, HttpMessage, HttpRequest, HttpResponse};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use uuid::Uuid;

use crate::api::middleware::ApiKeyAuth;
use crate::db::pool::DbError;
use crate::db::{ApiKeyRepository, ApiKeyTier, CreateApiKeyRequest, CreateApiKeyResponse, DbPool};

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

#[derive(Debug)]
struct ValidatedCreateKeyRequest {
    name: String,
    owner_email: String,
    owner_name: Option<String>,
    company: Option<String>,
    tier: ApiKeyTier,
    rate_limit_per_minute: Option<i32>,
    monthly_quota: Option<i32>,
    expires_at: Option<DateTime<Utc>>,
}

/// Response after creating a new API key
#[derive(Debug, Serialize)]
pub struct CreateKeyResponse {
    pub id: Uuid,
    pub api_key: String, // Only shown once!
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

#[derive(Debug, Deserialize)]
pub struct SelfServeSignupRequest {
    pub email: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub company: Option<String>,
    #[serde(default)]
    pub project_name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SelfServeSignupResponse {
    pub id: Uuid,
    pub api_key: String,
    pub key_prefix: String,
    pub tier: String,
    pub rate_limit_per_minute: i32,
    pub monthly_quota: i32,
    pub owner_email: String,
    pub message: String,
}

fn normalize_optional_text(value: Option<&str>) -> Option<String> {
    value.and_then(|raw| {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

fn validate_email(email: &str) -> bool {
    let trimmed = email.trim();

    if trimmed.len() < 3 || trimmed.len() > 254 || trimmed.contains(' ') {
        return false;
    }

    let mut parts = trimmed.split('@');
    let local = parts.next().unwrap_or_default();
    let domain = parts.next().unwrap_or_default();

    parts.next().is_none()
        && !local.is_empty()
        && !domain.is_empty()
        && domain.contains('.')
        && !domain.starts_with('.')
        && !domain.ends_with('.')
}

fn validate_create_key_request(
    body: &CreateKeyRequest,
) -> Result<ValidatedCreateKeyRequest, HttpResponse> {
    let name = body.name.trim();
    if name.is_empty() || name.len() > 80 {
        return Err(HttpResponse::BadRequest().json(serde_json::json!({
            "error": "invalid_name",
            "message": "Key name must be between 1 and 80 characters"
        })));
    }

    let owner_email = body.owner_email.trim().to_lowercase();
    if !validate_email(&owner_email) {
        return Err(HttpResponse::BadRequest().json(serde_json::json!({
            "error": "invalid_email",
            "message": "owner_email must be a valid email address"
        })));
    }

    if let Some(rate_limit) = body.rate_limit_per_minute {
        if rate_limit <= 0 {
            return Err(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "invalid_rate_limit",
                "message": "rate_limit_per_minute must be greater than 0"
            })));
        }
    }

    if let Some(monthly_quota) = body.monthly_quota {
        if monthly_quota <= 0 {
            return Err(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "invalid_monthly_quota",
                "message": "monthly_quota must be greater than 0"
            })));
        }
    }

    Ok(ValidatedCreateKeyRequest {
        name: name.to_string(),
        owner_email,
        owner_name: normalize_optional_text(body.owner_name.as_deref()),
        company: normalize_optional_text(body.company.as_deref()),
        tier: ApiKeyTier::from_str(&body.tier),
        rate_limit_per_minute: body.rate_limit_per_minute,
        monthly_quota: body.monthly_quota,
        expires_at: body.expires_at,
    })
}

fn validate_self_serve_signup(
    body: &SelfServeSignupRequest,
) -> Result<ValidatedCreateKeyRequest, HttpResponse> {
    let owner_email = body.email.trim().to_lowercase();
    if !validate_email(&owner_email) {
        return Err(HttpResponse::BadRequest().json(serde_json::json!({
            "error": "invalid_email",
            "message": "email must be a valid email address"
        })));
    }

    let project_name = normalize_optional_text(body.project_name.as_deref())
        .or_else(|| normalize_optional_text(body.company.as_deref()))
        .unwrap_or_else(|| "Starter API key".to_string());

    Ok(ValidatedCreateKeyRequest {
        name: project_name,
        owner_email,
        owner_name: normalize_optional_text(body.name.as_deref()),
        company: normalize_optional_text(body.company.as_deref()),
        tier: ApiKeyTier::Free,
        rate_limit_per_minute: None,
        monthly_quota: None,
        expires_at: None,
    })
}

async fn create_key_from_validated_request(
    repo: &ApiKeyRepository,
    request: ValidatedCreateKeyRequest,
) -> Result<CreateApiKeyResponse, DbError> {
    repo.create(CreateApiKeyRequest {
        name: request.name,
        owner_email: request.owner_email,
        owner_name: request.owner_name,
        company: request.company,
        tier: request.tier,
        rate_limit_per_minute: request.rate_limit_per_minute,
        monthly_quota: request.monthly_quota,
        expires_at: request.expires_at,
    })
    .await
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
    let request = match validate_create_key_request(&body) {
        Ok(request) => request,
        Err(response) => return response,
    };

    match create_key_from_validated_request(&repo, request).await {
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

/// Create a free-tier API key without an existing admin key.
/// POST /api/v1/keys/signup
pub async fn signup_for_api_key(
    pool: web::Data<DbPool>,
    body: web::Json<SelfServeSignupRequest>,
) -> HttpResponse {
    let repo = ApiKeyRepository::new(pool.get_ref().clone());
    let request = match validate_self_serve_signup(&body) {
        Ok(request) => request,
        Err(response) => return response,
    };

    match repo.list_by_owner(&request.owner_email).await {
        Ok(existing_keys) => {
            if existing_keys.iter().any(|key| key.is_active) {
                return HttpResponse::Conflict().json(serde_json::json!({
                    "error": "key_exists",
                    "message": "An active API key already exists for this email. Use the existing key or contact support."
                }));
            }
        }
        Err(e) => {
            warn!(error = %e, "Failed to check existing self-serve API keys");
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "internal_error",
                "message": "Failed to create API key"
            }));
        }
    }

    let owner_email = request.owner_email.clone();

    match create_key_from_validated_request(&repo, request).await {
        Ok(response) => {
            info!(
                key_id = %response.id,
                key_prefix = %response.key_prefix,
                owner_email = %owner_email,
                "Self-serve API key created"
            );

            HttpResponse::Created().json(SelfServeSignupResponse {
                id: response.id,
                api_key: response.api_key,
                key_prefix: response.key_prefix,
                tier: response.tier,
                rate_limit_per_minute: response.rate_limit_per_minute,
                monthly_quota: response.monthly_quota,
                owner_email,
                message: "Free API key created successfully. Save the api_key value now because it will not be shown again.".to_string(),
            })
        }
        Err(e) => {
            warn!(error = %e, "Failed to create self-serve API key");
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "internal_error",
                "message": "Failed to create API key"
            }))
        }
    }
}

/// Get current API key info
/// GET /api/v1/keys/me
pub async fn get_my_key(req: HttpRequest, pool: web::Data<DbPool>) -> HttpResponse {
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
        Ok(Some(key)) => HttpResponse::Ok().json(ApiKeyInfo {
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
        }),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "not_found",
            "message": "API key not found"
        })),
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
        Ok(Some(key)) => HttpResponse::Ok().json(ApiKeyInfo {
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
        }),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "not_found",
            "message": "API key not found"
        })),
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
                    let key_infos: Vec<ApiKeyInfo> = keys
                        .into_iter()
                        .map(|key| ApiKeyInfo {
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
                        .collect();

                    let count = key_infos.len();
                    return HttpResponse::Ok().json(ListKeysResponse {
                        keys: key_infos,
                        count,
                    });
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
    let owner_email = query
        .owner_email
        .as_ref()
        .map(|s| s.as_str())
        .unwrap_or(&auth.owner_email);
    let repo = ApiKeyRepository::new(pool.get_ref().clone());

    match repo.list_by_owner(owner_email).await {
        Ok(keys) => {
            let key_infos: Vec<ApiKeyInfo> = keys
                .into_iter()
                .map(|key| ApiKeyInfo {
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
                .collect();

            let count = key_infos.len();
            HttpResponse::Ok().json(ListKeysResponse {
                keys: key_infos,
                count,
            })
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
        Ok(false) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "not_found",
            "message": "API key not found"
        })),
        Err(e) => {
            warn!(error = %e, "Failed to revoke API key");
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "internal_error",
                "message": "Failed to revoke API key"
            }))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        validate_create_key_request, validate_email, validate_self_serve_signup, CreateKeyRequest,
        SelfServeSignupRequest,
    };

    #[test]
    fn accepts_reasonable_email_addresses() {
        assert!(validate_email("dev@meetmockup.com"));
        assert!(validate_email("pod.team+ops@example.co"));
        assert!(!validate_email("not-an-email"));
        assert!(!validate_email("missing-domain@"));
        assert!(!validate_email("@missing-local.com"));
    }

    #[test]
    fn self_serve_signup_defaults_to_free_tier_project_name() {
        let request = SelfServeSignupRequest {
            email: "hello@example.com".to_string(),
            name: Some("Alex".to_string()),
            company: Some("Example Studio".to_string()),
            project_name: None,
        };

        let validated = validate_self_serve_signup(&request).expect("request should validate");

        assert_eq!(validated.owner_email, "hello@example.com");
        assert_eq!(validated.name, "Example Studio");
        assert_eq!(validated.tier.as_str(), "free");
    }

    #[test]
    fn create_key_validation_rejects_invalid_quota_values() {
        let request = CreateKeyRequest {
            name: "Starter".to_string(),
            owner_email: "hello@example.com".to_string(),
            owner_name: None,
            company: None,
            tier: "starter".to_string(),
            rate_limit_per_minute: Some(0),
            monthly_quota: Some(-1),
            expires_at: None,
        };

        assert!(validate_create_key_request(&request).is_err());
    }
}
