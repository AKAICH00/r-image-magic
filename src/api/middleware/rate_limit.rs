//! Rate Limiting Middleware
//!
//! Implements sliding window rate limiting using database-backed counters.
//! Rate limits are applied per API key based on tier settings.

use actix_web::{
    dev::{ServiceRequest, ServiceResponse},
    error::ErrorTooManyRequests,
    http::header::{HeaderName, HeaderValue},
    Error, HttpResponse,
};
use std::sync::Arc;
use tracing::{info, warn};

use crate::db::{UsageRepository, RateLimitStatus};
use super::auth::ApiKeyAuth;

/// Rate limit headers
pub const RATE_LIMIT_LIMIT: &str = "X-RateLimit-Limit";
pub const RATE_LIMIT_REMAINING: &str = "X-RateLimit-Remaining";
pub const RATE_LIMIT_RESET: &str = "X-RateLimit-Reset";
pub const RETRY_AFTER: &str = "Retry-After";

/// Check rate limit for an authenticated request
pub async fn check_rate_limit(
    auth: &ApiKeyAuth,
    usage_repo: &UsageRepository,
) -> Result<RateLimitStatus, Error> {
    let status = usage_repo
        .check_rate_limit(auth.key_id, auth.rate_limit)
        .await
        .map_err(|e| {
            warn!(error = %e, "Failed to check rate limit");
            ErrorTooManyRequests("Rate limit check failed")
        })?;

    if !status.allowed {
        warn!(
            key_id = %auth.key_id,
            current = status.current_count,
            limit = status.limit,
            "Rate limit exceeded"
        );
        return Err(ErrorTooManyRequests(format!(
            "Rate limit exceeded. Limit: {} requests per minute. Try again at {}",
            status.limit,
            status.reset_at.format("%H:%M:%S UTC")
        )));
    }

    Ok(status)
}

/// Add rate limit headers to response
pub fn add_rate_limit_headers(
    response: &mut HttpResponse,
    status: &RateLimitStatus,
) {
    let headers = response.headers_mut();

    // X-RateLimit-Limit: Maximum requests per minute
    if let Ok(name) = HeaderName::try_from(RATE_LIMIT_LIMIT) {
        headers.insert(name, HeaderValue::from(status.limit));
    }

    // X-RateLimit-Remaining: Requests remaining in window
    let remaining = (status.limit - status.current_count).max(0);
    if let Ok(name) = HeaderName::try_from(RATE_LIMIT_REMAINING) {
        headers.insert(name, HeaderValue::from(remaining));
    }

    // X-RateLimit-Reset: Unix timestamp when window resets
    let reset_timestamp = status.reset_at.timestamp();
    if let Ok(name) = HeaderName::try_from(RATE_LIMIT_RESET) {
        if let Ok(val) = HeaderValue::try_from(reset_timestamp.to_string()) {
            headers.insert(name, val);
        }
    }
}

/// Create rate limit exceeded response with proper headers
pub fn rate_limit_exceeded_response(status: &RateLimitStatus) -> HttpResponse {
    let seconds_until_reset = (status.reset_at - chrono::Utc::now()).num_seconds().max(1);

    HttpResponse::TooManyRequests()
        .insert_header((RATE_LIMIT_LIMIT, status.limit.to_string()))
        .insert_header((RATE_LIMIT_REMAINING, "0"))
        .insert_header((RATE_LIMIT_RESET, status.reset_at.timestamp().to_string()))
        .insert_header((RETRY_AFTER, seconds_until_reset.to_string()))
        .json(serde_json::json!({
            "error": "rate_limit_exceeded",
            "message": format!("Rate limit exceeded. Maximum {} requests per minute.", status.limit),
            "limit": status.limit,
            "reset_at": status.reset_at.to_rfc3339(),
            "retry_after_seconds": seconds_until_reset
        }))
}

/// Rate limit info for response enrichment
#[derive(Clone, Debug)]
pub struct RateLimitInfo {
    pub limit: i32,
    pub remaining: i32,
    pub reset_at: chrono::DateTime<chrono::Utc>,
}

impl From<&RateLimitStatus> for RateLimitInfo {
    fn from(status: &RateLimitStatus) -> Self {
        Self {
            limit: status.limit,
            remaining: (status.limit - status.current_count).max(0),
            reset_at: status.reset_at,
        }
    }
}
