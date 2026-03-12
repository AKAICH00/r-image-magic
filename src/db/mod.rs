//! Database module for PostgreSQL connectivity
//!
//! Provides connection pool management, template queries, API key management,
//! and usage tracking for the r_image_magic database.

pub mod api_keys;
pub mod models;
pub mod pool;
pub mod queries;
pub mod usage;

pub use api_keys::{
    ApiKeyRepository, ApiKeyTier, CreateApiKeyRequest, CreateApiKeyResponse, DbApiKey,
};
pub use pool::DbPool;
pub use queries::TemplateRepository;
pub use usage::{MonthlyUsageSummary, RateLimitStatus, UsageLogEntry, UsageRepository, UsageStats};
