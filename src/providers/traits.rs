//! Provider trait definitions for POD integrations
//!
//! This module defines the contract that all POD provider implementations must follow.
//! Each provider (Printful, Printify, etc.) implements the `PodProvider` trait to
//! provide a unified interface for catalog access.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::domain::catalog::{
    UnifiedProduct, UnifiedVariant, UnifiedPrintArea, MockupAsset
};

// ============================================================================
// Error Types
// ============================================================================

/// Provider error types
#[derive(Debug, Error)]
pub enum ProviderError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Authentication failed: {0}")]
    AuthFailed(String),

    #[error("Rate limited, retry after {retry_after_secs} seconds")]
    RateLimited { retry_after_secs: u64 },

    #[error("API error: {status} - {message}")]
    ApiError { status: u16, message: String },

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Provider not configured: {0}")]
    NotConfigured(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Result type for provider operations
pub type ProviderResult<T> = Result<T, ProviderError>;

// ============================================================================
// Pagination Types
// ============================================================================

/// Catalog page for paginated results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogPage<T> {
    pub items: Vec<T>,
    pub total: u64,
    pub page: u32,
    pub per_page: u32,
    pub has_more: bool,
}

impl<T> CatalogPage<T> {
    /// Create a new catalog page
    pub fn new(items: Vec<T>, total: u64, page: u32, per_page: u32) -> Self {
        let has_more = (page as u64 * per_page as u64) < total;
        CatalogPage {
            items,
            total,
            page,
            per_page,
            has_more,
        }
    }

    /// Create an empty page
    pub fn empty() -> Self {
        CatalogPage {
            items: Vec::new(),
            total: 0,
            page: 1,
            per_page: 50,
            has_more: false,
        }
    }
}

// ============================================================================
// Credentials
// ============================================================================

/// Provider credentials for authentication
#[derive(Debug, Clone)]
pub struct ProviderCredentials {
    /// OAuth access token (for Printful, Printify, SPOD)
    pub access_token: Option<String>,

    /// API key (for Gelato)
    pub api_key: Option<String>,

    /// Recipe ID (for Gooten)
    pub recipe_id: Option<String>,

    /// OAuth refresh token (if applicable)
    pub refresh_token: Option<String>,

    /// OAuth client ID (for refreshing tokens)
    pub client_id: Option<String>,

    /// OAuth client secret (for refreshing tokens)
    pub client_secret: Option<String>,
}

impl ProviderCredentials {
    /// Create credentials from environment variables for a specific provider
    pub fn from_env(provider_code: &str) -> Self {
        let prefix = provider_code.to_uppercase();
        ProviderCredentials {
            access_token: std::env::var(format!("{}_ACCESS_TOKEN", prefix)).ok(),
            api_key: std::env::var(format!("{}_API_KEY", prefix)).ok(),
            recipe_id: std::env::var(format!("{}_RECIPE_ID", prefix)).ok(),
            refresh_token: std::env::var(format!("{}_REFRESH_TOKEN", prefix)).ok(),
            client_id: std::env::var(format!("{}_CLIENT_ID", prefix)).ok(),
            client_secret: std::env::var(format!("{}_CLIENT_SECRET", prefix)).ok(),
        }
    }

    /// Check if any credentials are configured
    pub fn is_configured(&self) -> bool {
        self.access_token.is_some()
            || self.api_key.is_some()
            || self.recipe_id.is_some()
    }
}

impl Default for ProviderCredentials {
    fn default() -> Self {
        ProviderCredentials {
            access_token: None,
            api_key: None,
            recipe_id: None,
            refresh_token: None,
            client_id: None,
            client_secret: None,
        }
    }
}

// ============================================================================
// Provider Trait
// ============================================================================

/// Provider trait for POD integrations
///
/// All POD provider implementations must implement this trait to provide
/// a unified interface for accessing product catalogs, variants, and mockups.
#[async_trait]
pub trait PodProvider: Send + Sync {
    /// Provider code (e.g., "printful", "printify")
    fn code(&self) -> &'static str;

    /// Provider display name (e.g., "Printful", "Printify")
    fn name(&self) -> &'static str;

    /// API base URL
    fn base_url(&self) -> &str;

    /// Rate limit (requests per minute)
    fn rate_limit(&self) -> u32;

    /// Authenticate with the provider
    ///
    /// This should validate credentials and obtain any necessary tokens.
    async fn authenticate(&mut self) -> ProviderResult<()>;

    /// Check if authentication is valid
    fn is_authenticated(&self) -> bool;

    /// Refresh authentication token (if supported)
    async fn refresh_auth(&mut self) -> ProviderResult<()>;

    /// Get all products (paginated)
    ///
    /// # Arguments
    /// * `page` - Page number (1-indexed)
    /// * `per_page` - Number of items per page (max 100)
    async fn get_products(&self, page: u32, per_page: u32) -> ProviderResult<CatalogPage<UnifiedProduct>>;

    /// Get product details by external ID
    ///
    /// # Arguments
    /// * `external_id` - Provider's product ID
    async fn get_product(&self, external_id: &str) -> ProviderResult<UnifiedProduct>;

    /// Get product variants
    ///
    /// # Arguments
    /// * `product_external_id` - Provider's product ID
    async fn get_variants(&self, product_external_id: &str) -> ProviderResult<Vec<UnifiedVariant>>;

    /// Get print areas for a product
    ///
    /// # Arguments
    /// * `product_external_id` - Provider's product ID
    async fn get_print_areas(&self, product_external_id: &str) -> ProviderResult<Vec<UnifiedPrintArea>>;

    /// Get mockup/printfile URLs for a product variant
    ///
    /// # Arguments
    /// * `product_external_id` - Provider's product ID
    /// * `variant_external_id` - Optional variant ID for variant-specific mockups
    async fn get_mockup_urls(
        &self,
        product_external_id: &str,
        variant_external_id: Option<&str>,
    ) -> ProviderResult<Vec<MockupAsset>>;

    /// Get rate limit status (remaining requests in current window)
    fn rate_limit_remaining(&self) -> Option<u32>;
}

// ============================================================================
// Provider Factory
// ============================================================================

/// Provider factory for creating provider instances
pub struct ProviderFactory;

impl ProviderFactory {
    /// Create a provider instance by code
    ///
    /// # Arguments
    /// * `code` - Provider code (e.g., "printful", "printify")
    /// * `credentials` - Provider credentials
    ///
    /// # Returns
    /// A boxed provider instance, or None if the provider code is unknown
    pub fn create(code: &str, credentials: ProviderCredentials) -> Option<Box<dyn PodProvider>> {
        match code {
            "printful" => Some(Box::new(crate::providers::printful::PrintfulProvider::new(credentials))),
            // "printify" => Some(Box::new(crate::providers::printify::PrintifyProvider::new(credentials))),
            // "gelato" => Some(Box::new(crate::providers::gelato::GelatoProvider::new(credentials))),
            // "spod" => Some(Box::new(crate::providers::spod::SpodProvider::new(credentials))),
            // "gooten" => Some(Box::new(crate::providers::gooten::GootenProvider::new(credentials))),
            _ => None,
        }
    }

    /// Create all configured providers from environment variables
    pub fn create_all_from_env() -> Vec<Box<dyn PodProvider>> {
        let provider_codes = ["printful", "printify", "gelato", "spod", "gooten"];
        let mut providers = Vec::new();

        for code in provider_codes {
            let credentials = ProviderCredentials::from_env(code);
            if credentials.is_configured() {
                if let Some(provider) = Self::create(code, credentials) {
                    providers.push(provider);
                }
            }
        }

        providers
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_catalog_page_new() {
        let items = vec!["a", "b", "c"];
        let page = CatalogPage::new(items, 10, 1, 3);

        assert_eq!(page.items.len(), 3);
        assert_eq!(page.total, 10);
        assert_eq!(page.page, 1);
        assert_eq!(page.per_page, 3);
        assert!(page.has_more);
    }

    #[test]
    fn test_catalog_page_no_more() {
        let items = vec!["a", "b", "c"];
        let page = CatalogPage::new(items, 3, 1, 10);

        assert!(!page.has_more);
    }

    #[test]
    fn test_credentials_default() {
        let creds = ProviderCredentials::default();
        assert!(!creds.is_configured());
    }
}
