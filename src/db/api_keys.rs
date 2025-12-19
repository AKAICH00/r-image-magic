//! API key database operations

use super::pool::{DbPool, DbError};
use chrono::{DateTime, Utc};
use rand::Rng;
use sha2::{Sha256, Digest};
use tracing::{info, warn};
use uuid::Uuid;

/// API key tier with associated limits
#[derive(Debug, Clone, PartialEq)]
pub enum ApiKeyTier {
    Free,
    Starter,
    Pro,
    Enterprise,
}

impl ApiKeyTier {
    pub fn as_str(&self) -> &'static str {
        match self {
            ApiKeyTier::Free => "free",
            ApiKeyTier::Starter => "starter",
            ApiKeyTier::Pro => "pro",
            ApiKeyTier::Enterprise => "enterprise",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "starter" => ApiKeyTier::Starter,
            "pro" => ApiKeyTier::Pro,
            "enterprise" => ApiKeyTier::Enterprise,
            _ => ApiKeyTier::Free,
        }
    }

    pub fn default_rate_limit(&self) -> i32 {
        match self {
            ApiKeyTier::Free => 10,
            ApiKeyTier::Starter => 30,
            ApiKeyTier::Pro => 100,
            ApiKeyTier::Enterprise => 1000,
        }
    }

    pub fn default_monthly_quota(&self) -> i32 {
        match self {
            ApiKeyTier::Free => 100,
            ApiKeyTier::Starter => 1000,
            ApiKeyTier::Pro => 10000,
            ApiKeyTier::Enterprise => 1000000,
        }
    }
}

/// Database model for API key
#[derive(Debug, Clone)]
pub struct DbApiKey {
    pub id: Uuid,
    pub key_prefix: String,
    pub key_hash: String,
    pub name: String,
    pub owner_email: String,
    pub owner_name: Option<String>,
    pub company: Option<String>,
    pub tier: String,
    pub rate_limit_per_minute: i32,
    pub monthly_quota: i32,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
}

impl DbApiKey {
    /// Check if the API key is valid (active and not expired)
    pub fn is_valid(&self) -> bool {
        if !self.is_active {
            return false;
        }
        if let Some(expires) = self.expires_at {
            if expires < Utc::now() {
                return false;
            }
        }
        true
    }

    pub fn tier_enum(&self) -> ApiKeyTier {
        ApiKeyTier::from_str(&self.tier)
    }
}

/// Request to create a new API key
#[derive(Debug)]
pub struct CreateApiKeyRequest {
    pub name: String,
    pub owner_email: String,
    pub owner_name: Option<String>,
    pub company: Option<String>,
    pub tier: ApiKeyTier,
    pub rate_limit_per_minute: Option<i32>,
    pub monthly_quota: Option<i32>,
    pub expires_at: Option<DateTime<Utc>>,
}

/// Response containing the new API key (only returned once!)
#[derive(Debug)]
pub struct CreateApiKeyResponse {
    pub id: Uuid,
    pub api_key: String, // The actual key - only shown once!
    pub key_prefix: String,
    pub name: String,
    pub tier: String,
    pub rate_limit_per_minute: i32,
    pub monthly_quota: i32,
}

/// Repository for API key operations
pub struct ApiKeyRepository {
    pub pool: DbPool,
}

impl ApiKeyRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    /// Generate a new API key
    /// Format: rim_<32 random alphanumeric chars>
    fn generate_api_key() -> String {
        const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
        let mut rng = rand::thread_rng();

        let key_body: String = (0..32)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect();

        format!("rim_{}", key_body)
    }

    /// Hash an API key using SHA-256
    fn hash_api_key(key: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Create a new API key
    pub async fn create(&self, request: CreateApiKeyRequest) -> Result<CreateApiKeyResponse, DbError> {
        let client = self.pool.get().await?;

        // Generate key and hash
        let api_key = Self::generate_api_key();
        let key_prefix = api_key[..12].to_string(); // "rim_" + 8 chars
        let key_hash = Self::hash_api_key(&api_key);

        // Use tier defaults if not specified
        let rate_limit = request.rate_limit_per_minute
            .unwrap_or_else(|| request.tier.default_rate_limit());
        let monthly_quota = request.monthly_quota
            .unwrap_or_else(|| request.tier.default_monthly_quota());

        let row = client.query_one(
            r#"
            INSERT INTO api_keys (
                key_prefix, key_hash, name, owner_email, owner_name, company,
                tier, rate_limit_per_minute, monthly_quota, expires_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING id
            "#,
            &[
                &key_prefix,
                &key_hash,
                &request.name,
                &request.owner_email,
                &request.owner_name,
                &request.company,
                &request.tier.as_str(),
                &rate_limit,
                &monthly_quota,
                &request.expires_at,
            ]
        ).await?;

        let id: Uuid = row.get("id");

        info!(
            key_id = %id,
            key_prefix = %key_prefix,
            tier = %request.tier.as_str(),
            owner = %request.owner_email,
            "Created new API key"
        );

        Ok(CreateApiKeyResponse {
            id,
            api_key, // Return the actual key (only once!)
            key_prefix,
            name: request.name,
            tier: request.tier.as_str().to_string(),
            rate_limit_per_minute: rate_limit,
            monthly_quota,
        })
    }

    /// Validate an API key and return its details
    pub async fn validate(&self, api_key: &str) -> Result<Option<DbApiKey>, DbError> {
        let client = self.pool.get().await?;

        // Extract prefix for efficient lookup
        if api_key.len() < 12 {
            return Ok(None);
        }
        let key_prefix = &api_key[..12];
        let key_hash = Self::hash_api_key(api_key);

        let row = client.query_opt(
            r#"
            SELECT
                id, key_prefix, key_hash, name, owner_email, owner_name, company,
                tier, rate_limit_per_minute, monthly_quota, is_active,
                created_at, updated_at, last_used_at, expires_at
            FROM api_keys
            WHERE key_prefix = $1 AND key_hash = $2
            "#,
            &[&key_prefix, &key_hash]
        ).await?;

        Ok(row.map(|row| DbApiKey {
            id: row.get("id"),
            key_prefix: row.get("key_prefix"),
            key_hash: row.get("key_hash"),
            name: row.get("name"),
            owner_email: row.get("owner_email"),
            owner_name: row.get("owner_name"),
            company: row.get("company"),
            tier: row.get("tier"),
            rate_limit_per_minute: row.get("rate_limit_per_minute"),
            monthly_quota: row.get("monthly_quota"),
            is_active: row.get("is_active"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            last_used_at: row.get("last_used_at"),
            expires_at: row.get("expires_at"),
        }))
    }

    /// Update last_used_at timestamp
    pub async fn touch(&self, key_id: Uuid) -> Result<(), DbError> {
        let client = self.pool.get().await?;

        client.execute(
            "UPDATE api_keys SET last_used_at = NOW() WHERE id = $1",
            &[&key_id]
        ).await?;

        Ok(())
    }

    /// Get API key by ID
    pub async fn get_by_id(&self, id: Uuid) -> Result<Option<DbApiKey>, DbError> {
        let client = self.pool.get().await?;

        let row = client.query_opt(
            r#"
            SELECT
                id, key_prefix, key_hash, name, owner_email, owner_name, company,
                tier, rate_limit_per_minute, monthly_quota, is_active,
                created_at, updated_at, last_used_at, expires_at
            FROM api_keys
            WHERE id = $1
            "#,
            &[&id]
        ).await?;

        Ok(row.map(|row| DbApiKey {
            id: row.get("id"),
            key_prefix: row.get("key_prefix"),
            key_hash: row.get("key_hash"),
            name: row.get("name"),
            owner_email: row.get("owner_email"),
            owner_name: row.get("owner_name"),
            company: row.get("company"),
            tier: row.get("tier"),
            rate_limit_per_minute: row.get("rate_limit_per_minute"),
            monthly_quota: row.get("monthly_quota"),
            is_active: row.get("is_active"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            last_used_at: row.get("last_used_at"),
            expires_at: row.get("expires_at"),
        }))
    }

    /// List all API keys for an owner
    pub async fn list_by_owner(&self, owner_email: &str) -> Result<Vec<DbApiKey>, DbError> {
        let client = self.pool.get().await?;

        let rows = client.query(
            r#"
            SELECT
                id, key_prefix, key_hash, name, owner_email, owner_name, company,
                tier, rate_limit_per_minute, monthly_quota, is_active,
                created_at, updated_at, last_used_at, expires_at
            FROM api_keys
            WHERE owner_email = $1
            ORDER BY created_at DESC
            "#,
            &[&owner_email]
        ).await?;

        Ok(rows.iter().map(|row| DbApiKey {
            id: row.get("id"),
            key_prefix: row.get("key_prefix"),
            key_hash: row.get("key_hash"),
            name: row.get("name"),
            owner_email: row.get("owner_email"),
            owner_name: row.get("owner_name"),
            company: row.get("company"),
            tier: row.get("tier"),
            rate_limit_per_minute: row.get("rate_limit_per_minute"),
            monthly_quota: row.get("monthly_quota"),
            is_active: row.get("is_active"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            last_used_at: row.get("last_used_at"),
            expires_at: row.get("expires_at"),
        }).collect())
    }

    /// Revoke (deactivate) an API key
    pub async fn revoke(&self, id: Uuid) -> Result<bool, DbError> {
        let client = self.pool.get().await?;

        let result = client.execute(
            "UPDATE api_keys SET is_active = false WHERE id = $1",
            &[&id]
        ).await?;

        if result > 0 {
            warn!(key_id = %id, "API key revoked");
        }

        Ok(result > 0)
    }

    /// Delete an API key permanently
    pub async fn delete(&self, id: Uuid) -> Result<bool, DbError> {
        let client = self.pool.get().await?;

        let result = client.execute(
            "DELETE FROM api_keys WHERE id = $1",
            &[&id]
        ).await?;

        if result > 0 {
            warn!(key_id = %id, "API key deleted");
        }

        Ok(result > 0)
    }
}
