//! API Middleware Module
//!
//! Provides authentication, rate limiting, and usage tracking middleware
//! for the R-Image-Magic SaaS API.

pub mod auth;
pub mod rate_limit;
pub mod service;
pub mod usage;

pub use auth::{
    extract_api_key, validate_api_key, ApiKeyAuth, ApiKeyExt, AuthenticatedKey, API_KEY_HEADER,
};
pub use rate_limit::{
    add_rate_limit_headers, check_rate_limit, rate_limit_exceeded_response, RateLimitInfo,
    RATE_LIMIT_LIMIT, RATE_LIMIT_REMAINING, RATE_LIMIT_RESET, RETRY_AFTER,
};
pub use service::ApiMiddleware;
pub use usage::{
    check_quota, extract_client_ip, extract_user_agent, log_usage_async, QuotaExceededInfo,
    RequestTiming, UsageInfo, QUOTA_LIMIT, QUOTA_REMAINING, QUOTA_USED,
};
