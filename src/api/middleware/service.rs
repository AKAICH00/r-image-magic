//! API Middleware Service
//!
//! Actix-web middleware that combines authentication, rate limiting,
//! and usage tracking into a single service wrapper.

use actix_web::{
    body::{BoxBody, EitherBody},
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    http::StatusCode,
    Error, HttpMessage, HttpResponse,
};
use futures::future::{ok, Ready, LocalBoxFuture};
use std::rc::Rc;
use std::sync::Arc;
use std::time::Instant;
use tracing::{info, warn};

use crate::db::{ApiKeyRepository, UsageRepository, DbPool, UsageLogEntry};
use super::auth::{extract_api_key, validate_api_key, ApiKeyAuth};
use super::rate_limit::{RATE_LIMIT_LIMIT, RATE_LIMIT_REMAINING, RATE_LIMIT_RESET};

/// Middleware factory for API authentication and rate limiting
pub struct ApiMiddleware {
    pool: Option<DbPool>,
    /// Paths that don't require authentication
    public_paths: Vec<String>,
}

impl ApiMiddleware {
    pub fn new(pool: Option<DbPool>) -> Self {
        Self {
            pool,
            public_paths: vec![
                "/health".to_string(),
                "/swagger-ui".to_string(),
                "/api-docs".to_string(),
            ],
        }
    }

    pub fn with_public_paths(mut self, paths: Vec<String>) -> Self {
        self.public_paths.extend(paths);
        self
    }

    fn is_public_path(&self, path: &str) -> bool {
        self.public_paths.iter().any(|p| path.starts_with(p))
    }
}

impl<S, B> Transform<S, ServiceRequest> for ApiMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B, BoxBody>>;
    type Error = Error;
    type Transform = ApiMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(ApiMiddlewareService {
            service: Rc::new(service),
            pool: self.pool.clone(),
            public_paths: self.public_paths.clone(),
        })
    }
}

/// The actual middleware service
pub struct ApiMiddlewareService<S> {
    service: Rc<S>,
    pool: Option<DbPool>,
    public_paths: Vec<String>,
}

impl<S> ApiMiddlewareService<S> {
    fn is_public_path(&self, path: &str) -> bool {
        self.public_paths.iter().any(|p| path.starts_with(p))
    }
}

impl<S, B> Service<ServiceRequest> for ApiMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B, BoxBody>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, ctx: &mut core::task::Context<'_>) -> core::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let pool = self.pool.clone();
        let path = req.path().to_string();
        let method = req.method().to_string();
        let is_public = self.is_public_path(&path);

        Box::pin(async move {
            let start = Instant::now();

            // Skip auth for public paths
            if is_public {
                let res = service.call(req).await?;
                return Ok(res.map_into_left_body());
            }

            // If no database pool, skip auth (development mode)
            let pool = match pool {
                Some(p) => p,
                None => {
                    // No DB available, skip auth checks
                    let res = service.call(req).await?;
                    return Ok(res.map_into_left_body());
                }
            };

            // Extract API key
            let api_key = match extract_api_key(&req) {
                Some(key) => key,
                None => {
                    let response = HttpResponse::Unauthorized()
                        .json(serde_json::json!({
                            "error": "unauthorized",
                            "message": "API key required. Provide via X-API-Key header or Authorization: Bearer <key>"
                        }));
                    return Ok(req.into_response(response).map_into_right_body());
                }
            };

            // Validate API key
            let api_key_repo = ApiKeyRepository::new(pool.clone());
            let db_key = match validate_api_key(&api_key, &api_key_repo).await {
                Ok(key) => key,
                Err(e) => {
                    let response = HttpResponse::Unauthorized()
                        .json(serde_json::json!({
                            "error": "unauthorized",
                            "message": e.to_string()
                        }));
                    return Ok(req.into_response(response).map_into_right_body());
                }
            };

            // Create auth info
            let auth = ApiKeyAuth::from(&db_key);
            let key_id = auth.key_id;
            let rate_limit = auth.rate_limit;
            let monthly_quota = auth.monthly_quota;

            // Check rate limit
            let usage_repo = UsageRepository::new(pool.clone());
            let rate_status = match usage_repo.check_rate_limit(key_id, rate_limit).await {
                Ok(status) => status,
                Err(e) => {
                    warn!(error = %e, "Rate limit check failed");
                    let response = HttpResponse::InternalServerError()
                        .json(serde_json::json!({
                            "error": "internal_error",
                            "message": "Rate limit check failed"
                        }));
                    return Ok(req.into_response(response).map_into_right_body());
                }
            };

            if !rate_status.allowed {
                let seconds_until_reset = (rate_status.reset_at - chrono::Utc::now()).num_seconds().max(1);
                let response = HttpResponse::TooManyRequests()
                    .insert_header((RATE_LIMIT_LIMIT, rate_status.limit.to_string()))
                    .insert_header((RATE_LIMIT_REMAINING, "0"))
                    .insert_header((RATE_LIMIT_RESET, rate_status.reset_at.timestamp().to_string()))
                    .insert_header(("Retry-After", seconds_until_reset.to_string()))
                    .json(serde_json::json!({
                        "error": "rate_limit_exceeded",
                        "message": format!("Rate limit exceeded. Maximum {} requests per minute.", rate_status.limit),
                        "limit": rate_status.limit,
                        "reset_at": rate_status.reset_at.to_rfc3339(),
                        "retry_after_seconds": seconds_until_reset
                    }));
                return Ok(req.into_response(response).map_into_right_body());
            }

            // Check monthly quota
            match usage_repo.check_quota(key_id, monthly_quota).await {
                Ok(true) => {} // Quota OK
                Ok(false) => {
                    let response = HttpResponse::PaymentRequired()
                        .json(serde_json::json!({
                            "error": "quota_exceeded",
                            "message": format!("Monthly quota of {} requests exceeded. Upgrade your plan for more requests.", monthly_quota),
                            "quota": monthly_quota,
                            "upgrade_url": "https://r-image-magic.com/pricing"
                        }));
                    return Ok(req.into_response(response).map_into_right_body());
                }
                Err(e) => {
                    warn!(error = %e, "Quota check failed");
                    // Continue anyway - don't block on quota check failure
                }
            }

            // Store auth info in request extensions
            req.extensions_mut().insert(auth.clone());

            // Extract info for usage logging
            let ip_address = super::usage::extract_client_ip(&req);
            let user_agent = super::usage::extract_user_agent(&req);

            // Call the actual service
            let res = service.call(req).await?;

            // Log usage asynchronously
            let status_code = res.status();
            let response_time_ms = start.elapsed().as_millis() as i32;

            let error_info = if status_code.is_client_error() || status_code.is_server_error() {
                Some((status_code.to_string(), status_code.canonical_reason().unwrap_or("Unknown error").to_string()))
            } else {
                None
            };

            let (error_code, error_message) = error_info.unzip();

            let log_pool = pool.clone();
            tokio::spawn(async move {
                let log_repo = UsageRepository::new(log_pool);
                let entry = UsageLogEntry {
                    api_key_id: key_id,
                    endpoint: path,
                    method,
                    template_id: None, // TODO: Extract from request body for generate endpoint
                    status_code: status_code.as_u16() as i32,
                    response_time_ms: Some(response_time_ms),
                    error_code,
                    error_message,
                    ip_address,
                    user_agent,
                };
                if let Err(e) = log_repo.log_usage(entry).await {
                    warn!(error = %e, "Failed to log usage");
                }
            });

            // Add rate limit headers to response
            let mut res = res.map_into_left_body();
            let headers = res.headers_mut();
            if let Ok(limit) = rate_status.limit.to_string().parse() {
                headers.insert(
                    RATE_LIMIT_LIMIT.parse().unwrap(),
                    limit,
                );
            }
            let remaining = (rate_status.limit - rate_status.current_count).max(0);
            if let Ok(rem) = remaining.to_string().parse() {
                headers.insert(
                    RATE_LIMIT_REMAINING.parse().unwrap(),
                    rem,
                );
            }
            if let Ok(reset) = rate_status.reset_at.timestamp().to_string().parse() {
                headers.insert(
                    RATE_LIMIT_RESET.parse().unwrap(),
                    reset,
                );
            }

            Ok(res)
        })
    }
}
