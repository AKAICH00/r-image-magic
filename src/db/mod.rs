//! Database module for PostgreSQL connectivity
//!
//! Provides connection pool management, template queries, API key management,
//! and usage tracking for the r_image_magic database.

pub mod pool;
pub mod models;
pub mod queries;
pub mod api_keys;
pub mod usage;

pub use pool::DbPool;
pub use queries::TemplateRepository;
pub use api_keys::{ApiKeyRepository, CreateApiKeyRequest, CreateApiKeyResponse, DbApiKey, ApiKeyTier};
pub use usage::{UsageRepository, UsageLogEntry, UsageStats, MonthlyUsageSummary, RateLimitStatus};
