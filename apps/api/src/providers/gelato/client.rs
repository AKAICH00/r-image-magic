//! Gelato API Client Implementation
//!
//! This module implements the PodProvider trait for Gelato,
//! providing access to their product catalog via the Gelato API.
//!
//! API Docs: https://docs.gelato.com/reference
//! Auth: X-API-KEY header (not Bearer token)
//! Rate limit: 300 req/min

use async_trait::async_trait;
use tracing::{debug, info, warn};

use super::mapper::GelatoMapper;
use super::models::*;
use crate::domain::catalog::{MockupAsset, UnifiedPrintArea, UnifiedProduct, UnifiedVariant};
use crate::providers::http_client::RateLimitedClient;
use crate::providers::traits::{
    CatalogPage, PodProvider, ProviderCredentials, ProviderError, ProviderResult,
};

/// Gelato API client
pub struct GelatoProvider {
    /// Rate-limited HTTP client
    client: RateLimitedClient,

    /// API key for authentication
    api_key: Option<String>,

    /// API base URL
    base_url: String,

    /// Whether authentication is valid
    authenticated: bool,
}

impl GelatoProvider {
    /// Create a new Gelato provider instance
    pub fn new(credentials: ProviderCredentials) -> Self {
        GelatoProvider {
            client: RateLimitedClient::new(300), // 300 req/min
            api_key: credentials.api_key,
            base_url: "https://order.gelatoapis.com/v4".to_string(),
            authenticated: false,
        }
    }

    /// Make an authenticated GET request
    async fn get<T: serde::de::DeserializeOwned>(&self, path: &str) -> ProviderResult<T> {
        let api_key = self
            .api_key
            .as_ref()
            .ok_or_else(|| ProviderError::AuthFailed("No API key configured".to_string()))?;

        let url = format!("{}{}", self.base_url, path);
        debug!(url = %url, "Gelato API request");

        let response = self
            .client
            .get(&url)
            .header("X-API-KEY", api_key)
            .send()
            .await?;

        // Check for error status
        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(ProviderError::ApiError {
                status: status.as_u16(),
                message: body,
            });
        }

        // Parse JSON response
        let text = response.text().await?;
        serde_json::from_str(&text).map_err(|e| {
            ProviderError::ParseError(format!(
                "JSON parse error: {} - Body: {}",
                e,
                &text[..text.len().min(500)]
            ))
        })
    }

    /// Fetch product detail (used by get_product, get_variants, get_print_areas)
    async fn fetch_product_detail(&self, product_uid: &str) -> ProviderResult<GelatoProduct> {
        let path = format!("/products/{}", product_uid);
        self.get(&path).await
    }
}

#[async_trait]
impl PodProvider for GelatoProvider {
    fn code(&self) -> &'static str {
        "gelato"
    }

    fn name(&self) -> &'static str {
        "Gelato"
    }

    fn base_url(&self) -> &str {
        &self.base_url
    }

    fn rate_limit(&self) -> u32 {
        300
    }

    async fn authenticate(&mut self) -> ProviderResult<()> {
        if self.api_key.is_none() {
            return Err(ProviderError::NotConfigured(
                "GELATO_API_KEY environment variable not set".to_string(),
            ));
        }

        // Verify API key by listing catalogs
        let response: Result<GelatoListResponse<GelatoCatalog>, _> =
            self.get("/catalogs").await;

        match response {
            Ok(_) => {
                self.authenticated = true;
                info!("Gelato authentication successful");
                Ok(())
            }
            Err(ProviderError::ApiError { status: 401, .. })
            | Err(ProviderError::ApiError { status: 403, .. }) => {
                Err(ProviderError::AuthFailed("Invalid API key".to_string()))
            }
            Err(e) => Err(e),
        }
    }

    fn is_authenticated(&self) -> bool {
        self.authenticated && self.api_key.is_some()
    }

    async fn refresh_auth(&mut self) -> ProviderResult<()> {
        // Gelato API keys don't expire, but we can re-validate
        self.authenticate().await
    }

    async fn get_products(
        &self,
        page: u32,
        per_page: u32,
    ) -> ProviderResult<CatalogPage<UnifiedProduct>> {
        // First, get catalogs to find products
        let catalogs: GelatoListResponse<GelatoCatalog> = self.get("/catalogs").await?;

        let first_catalog = catalogs.data.first().ok_or_else(|| {
            ProviderError::NotFound("No catalogs found in Gelato account".to_string())
        })?;

        debug!(
            catalog_uid = %first_catalog.catalog_uid,
            catalog_title = %first_catalog.title,
            "Fetching products from first Gelato catalog"
        );

        // Fetch products from the first catalog
        let path = format!(
            "/catalogs/{}/products?page={}&pageSize={}",
            first_catalog.catalog_uid, page, per_page
        );

        let response: GelatoListResponse<GelatoProduct> = self.get(&path).await?;

        let total = response
            .pagination
            .as_ref()
            .map(|p| p.total as u64)
            .unwrap_or(response.data.len() as u64);

        let items: Vec<UnifiedProduct> = response
            .data
            .into_iter()
            .map(GelatoMapper::map_product)
            .collect();

        Ok(CatalogPage::new(items, total, page, per_page))
    }

    async fn get_product(&self, external_id: &str) -> ProviderResult<UnifiedProduct> {
        let product = self.fetch_product_detail(external_id).await?;
        Ok(GelatoMapper::map_product(product))
    }

    async fn get_variants(&self, product_external_id: &str) -> ProviderResult<Vec<UnifiedVariant>> {
        let product = self.fetch_product_detail(product_external_id).await?;

        let variants: Vec<UnifiedVariant> = product
            .variants
            .unwrap_or_default()
            .into_iter()
            .map(GelatoMapper::map_variant)
            .collect();

        Ok(variants)
    }

    async fn get_print_areas(
        &self,
        product_external_id: &str,
    ) -> ProviderResult<Vec<UnifiedPrintArea>> {
        let product = self.fetch_product_detail(product_external_id).await?;

        let print_areas: Vec<UnifiedPrintArea> = product
            .print_areas
            .unwrap_or_default()
            .into_iter()
            .map(GelatoMapper::map_print_area)
            .collect();

        Ok(print_areas)
    }

    async fn get_mockup_urls(
        &self,
        _product_external_id: &str,
        _variant_external_id: Option<&str>,
    ) -> ProviderResult<Vec<MockupAsset>> {
        // Gelato doesn't expose mockup URLs via the catalog API
        warn!("Gelato does not provide mockup URLs through the catalog API");
        Ok(Vec::new())
    }

    fn rate_limit_remaining(&self) -> Option<u32> {
        self.client.remaining_requests()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_creation() {
        let creds = ProviderCredentials::default();
        let provider = GelatoProvider::new(creds);

        assert_eq!(provider.code(), "gelato");
        assert_eq!(provider.name(), "Gelato");
        assert_eq!(provider.base_url(), "https://order.gelatoapis.com/v4");
        assert_eq!(provider.rate_limit(), 300);
        assert!(!provider.is_authenticated());
    }

    #[test]
    fn test_provider_with_api_key() {
        let creds = ProviderCredentials {
            api_key: Some("test_gelato_key".to_string()),
            ..Default::default()
        };
        let provider = GelatoProvider::new(creds);

        // Still not authenticated until authenticate() is called
        assert!(!provider.is_authenticated());
    }

    #[test]
    fn test_provider_not_configured() {
        let creds = ProviderCredentials::default();
        let mut provider = GelatoProvider::new(creds);

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build();

        match rt {
            Ok(rt) => {
                let result = rt.block_on(provider.authenticate());
                assert!(result.is_err());
                if let Err(ProviderError::NotConfigured(msg)) = result {
                    assert!(msg.contains("GELATO_API_KEY"));
                }
            }
            Err(_) => {
                // Skip test if runtime can't be built
            }
        }
    }
}
