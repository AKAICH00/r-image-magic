//! Usage Tracking Middleware
//!
//! Records API usage for billing and analytics.
//! Tracks request counts, response times, and errors per API key.

use actix_web::{
    dev::ServiceRequest,
    http::StatusCode,
    HttpMessage,
};
use chrono::Utc;
use std::net::IpAddr;
use std::time::Instant;
use tracing::{info, warn};

use crate::db::{UsageRepository, UsageLogEntry};
use super::auth::ApiKeyAuth;

/// Request timing context
#[derive(Clone, Debug)]
pub struct RequestTiming {
    pub start: Instant,
}

impl RequestTiming {
    pub fn new() -> Self {
        Self { start: Instant::now() }
    }

    pub fn elapsed_ms(&self) -> i32 {
        self.start.elapsed().as_millis() as i32
    }
}

impl Default for RequestTiming {
    fn default() -> Self {
        Self::new()
    }
}

/// Extract client IP from request
pub fn extract_client_ip(req: &ServiceRequest) -> Option<IpAddr> {
    // Try X-Forwarded-For first (for proxied requests)
    if let Some(forwarded) = req.headers().get("X-Forwarded-For") {
        if let Ok(forwarded_str) = forwarded.to_str() {
            // X-Forwarded-For may contain multiple IPs, take the first (client)
            if let Some(first_ip) = forwarded_str.split(',').next() {
                if let Ok(ip) = first_ip.trim().parse::<IpAddr>() {
                    return Some(ip);
                }
            }
        }
    }

    // Try X-Real-IP
    if let Some(real_ip) = req.headers().get("X-Real-IP") {
        if let Ok(ip_str) = real_ip.to_str() {
            if let Ok(ip) = ip_str.parse::<IpAddr>() {
                return Some(ip);
            }
        }
    }

    // Fall back to connection info
    req.peer_addr().map(|addr| addr.ip())
}

/// Extract user agent from request
pub fn extract_user_agent(req: &ServiceRequest) -> Option<String> {
    req.headers()
        .get("User-Agent")
        .and_then(|ua| ua.to_str().ok())
        .map(|s| s.chars().take(500).collect()) // Limit length
}

/// Log API usage asynchronously (fire and forget)
pub fn log_usage_async(
    usage_repo: UsageRepository,
    auth: ApiKeyAuth,
    endpoint: String,
    method: String,
    template_id: Option<String>,
    status_code: StatusCode,
    response_time_ms: Option<i32>,
    error_info: Option<(String, String)>, // (error_code, error_message)
    ip_address: Option<IpAddr>,
    user_agent: Option<String>,
) {
    tokio::spawn(async move {
        let (error_code, error_message) = error_info.unzip();

        let entry = UsageLogEntry {
            api_key_id: auth.key_id,
            endpoint,
            method,
            template_id,
            status_code: status_code.as_u16() as i32,
            response_time_ms,
            error_code,
            error_message,
            ip_address,
            user_agent,
        };

        if let Err(e) = usage_repo.log_usage(entry).await {
            warn!(error = %e, key_id = %auth.key_id, "Failed to log usage");
        }
    });
}

/// Check if API key has remaining quota
pub async fn check_quota(
    auth: &ApiKeyAuth,
    usage_repo: &UsageRepository,
) -> Result<bool, String> {
    usage_repo
        .check_quota(auth.key_id, auth.monthly_quota)
        .await
        .map_err(|e| format!("Quota check failed: {}", e))
}

/// Quota exceeded response details
#[derive(Debug, Clone)]
pub struct QuotaExceededInfo {
    pub monthly_quota: i32,
    pub current_usage: i32,
    pub tier: String,
}

impl QuotaExceededInfo {
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "error": "quota_exceeded",
            "message": format!(
                "Monthly quota of {} requests exceeded. Current usage: {}. Upgrade your plan for more requests.",
                self.monthly_quota, self.current_usage
            ),
            "quota": self.monthly_quota,
            "usage": self.current_usage,
            "tier": self.tier,
            "upgrade_url": "https://r-image-magic.com/pricing"
        })
    }
}

/// Usage summary for response headers
#[derive(Debug, Clone)]
pub struct UsageInfo {
    pub monthly_quota: i32,
    pub monthly_used: i32,
    pub monthly_remaining: i32,
}

/// Headers for usage info
pub const QUOTA_LIMIT: &str = "X-Quota-Limit";
pub const QUOTA_USED: &str = "X-Quota-Used";
pub const QUOTA_REMAINING: &str = "X-Quota-Remaining";
