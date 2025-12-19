//! Printful API Client Implementation
//!
//! This module implements the PodProvider trait for Printful,
//! providing access to their product catalog and mockup generation.
//!
//! API Docs: https://developers.printful.com/docs/

use async_trait::async_trait;
use tracing::{debug, info, warn};

use crate::providers::traits::{
    PodProvider, ProviderResult, ProviderError, CatalogPage, ProviderCredentials
};
use crate::providers::http_client::RateLimitedClient;
use crate::domain::catalog::{
    UnifiedProduct, UnifiedVariant, UnifiedPrintArea, MockupAsset
};
use super::models::*;
use super::mapper::PrintfulMapper;

/// Printful API client
pub struct PrintfulProvider {
    /// Rate-limited HTTP client
    client: RateLimitedClient,

    /// OAuth access token
    access_token: Option<String>,

    /// API base URL
    base_url: String,

    /// Whether authentication is valid
    authenticated: bool,
}

impl PrintfulProvider {
    /// Create a new Printful provider instance
    pub fn new(credentials: ProviderCredentials) -> Self {
        PrintfulProvider {
            client: RateLimitedClient::new(120), // 120 req/min
            access_token: credentials.access_token,
            base_url: "https://api.printful.com".to_string(),
            authenticated: false,
        }
    }

    /// Make an authenticated GET request
    async fn get<T: serde::de::DeserializeOwned>(&self, path: &str) -> ProviderResult<T> {
        let token = self.access_token.as_ref()
            .ok_or_else(|| ProviderError::AuthFailed("No access token configured".to_string()))?;

        let url = format!("{}{}", self.base_url, path);
        debug!(url = %url, "Printful API request");

        let response = self.client
            .get(&url)
            .bearer_auth(token)
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
        serde_json::from_str(&text)
            .map_err(|e| ProviderError::ParseError(format!("JSON parse error: {} - Body: {}", e, &text[..text.len().min(500)])))
    }
}

#[async_trait]
impl PodProvider for PrintfulProvider {
    fn code(&self) -> &'static str {
        "printful"
    }

    fn name(&self) -> &'static str {
        "Printful"
    }

    fn base_url(&self) -> &str {
        &self.base_url
    }

    fn rate_limit(&self) -> u32 {
        120
    }

    async fn authenticate(&mut self) -> ProviderResult<()> {
        if self.access_token.is_none() {
            return Err(ProviderError::NotConfigured(
                "PRINTFUL_ACCESS_TOKEN environment variable not set".to_string()
            ));
        }

        // Verify token by making a simple API call
        let response: Result<PrintfulResponse<serde_json::Value>, _> = self.get("/store").await;

        match response {
            Ok(_) => {
                self.authenticated = true;
                info!("Printful authentication successful");
                Ok(())
            }
            Err(ProviderError::ApiError { status: 401, .. }) => {
                Err(ProviderError::AuthFailed("Invalid access token".to_string()))
            }
            Err(e) => Err(e),
        }
    }

    fn is_authenticated(&self) -> bool {
        self.authenticated && self.access_token.is_some()
    }

    async fn refresh_auth(&mut self) -> ProviderResult<()> {
        // Printful tokens don't expire, but we can re-validate
        self.authenticate().await
    }

    async fn get_products(&self, page: u32, per_page: u32) -> ProviderResult<CatalogPage<UnifiedProduct>> {
        let offset = (page - 1) * per_page;
        let path = format!("/products?offset={}&limit={}", offset, per_page);

        let response: PrintfulResponse<Vec<PrintfulProduct>> = self.get(&path).await?;

        let items: Vec<UnifiedProduct> = response.result
            .into_iter()
            .map(PrintfulMapper::map_product)
            .collect();

        let total = response.paging
            .map(|p| p.total as u64)
            .unwrap_or(items.len() as u64);

        Ok(CatalogPage::new(items, total, page, per_page))
    }

    async fn get_product(&self, external_id: &str) -> ProviderResult<UnifiedProduct> {
        let path = format!("/products/{}", external_id);

        let response: PrintfulResponse<PrintfulProductDetail> = self.get(&path).await?;

        Ok(PrintfulMapper::map_product_detail(response.result))
    }

    async fn get_variants(&self, product_external_id: &str) -> ProviderResult<Vec<UnifiedVariant>> {
        let path = format!("/products/{}", product_external_id);

        let response: PrintfulResponse<PrintfulProductDetail> = self.get(&path).await?;

        let variants: Vec<UnifiedVariant> = response.result.variants
            .into_iter()
            .map(PrintfulMapper::map_variant)
            .collect();

        Ok(variants)
    }

    async fn get_print_areas(&self, product_external_id: &str) -> ProviderResult<Vec<UnifiedPrintArea>> {
        let path = format!("/mockup-generator/printfiles/{}", product_external_id);

        let response: PrintfulResponse<PrintfulPrintfilesResponse> = self.get(&path).await?;

        Ok(PrintfulMapper::extract_print_areas(response.result))
    }

    async fn get_mockup_urls(
        &self,
        product_external_id: &str,
        variant_external_id: Option<&str>,
    ) -> ProviderResult<Vec<MockupAsset>> {
        let path = format!("/mockup-generator/templates/{}", product_external_id);

        let response: PrintfulResponse<PrintfulMockupTemplatesResponse> = self.get(&path).await?;

        Ok(PrintfulMapper::map_mockup_assets(response.result, variant_external_id))
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
        let provider = PrintfulProvider::new(creds);

        assert_eq!(provider.code(), "printful");
        assert_eq!(provider.name(), "Printful");
        assert_eq!(provider.rate_limit(), 120);
        assert!(!provider.is_authenticated());
    }

    #[test]
    fn test_provider_with_token() {
        let creds = ProviderCredentials {
            access_token: Some("test_token".to_string()),
            ..Default::default()
        };
        let provider = PrintfulProvider::new(creds);

        // Still not authenticated until authenticate() is called
        assert!(!provider.is_authenticated());
    }
}
