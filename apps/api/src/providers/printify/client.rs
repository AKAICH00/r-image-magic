//! Printify API Client Implementation
//!
//! This module implements the PodProvider trait for Printify,
//! providing access to their product catalog via the v1 API.
//!
//! API Docs: https://developers.printify.com/docs/api

use async_trait::async_trait;
use tracing::{debug, info, warn};

use super::mapper::PrintifyMapper;
use super::models::*;
use crate::domain::catalog::{MockupAsset, UnifiedPrintArea, UnifiedProduct, UnifiedVariant};
use crate::providers::http_client::RateLimitedClient;
use crate::providers::traits::{
    CatalogPage, PodProvider, ProviderCredentials, ProviderError, ProviderResult,
};

/// Printify API client
pub struct PrintifyProvider {
    /// Rate-limited HTTP client
    client: RateLimitedClient,

    /// Bearer access token
    access_token: Option<String>,

    /// API base URL
    base_url: String,

    /// Whether authentication is valid
    authenticated: bool,
}

impl PrintifyProvider {
    /// Create a new Printify provider instance
    pub fn new(credentials: ProviderCredentials) -> Self {
        PrintifyProvider {
            client: RateLimitedClient::new(600), // 600 req/min
            access_token: credentials.access_token,
            base_url: "https://api.printify.com/v1".to_string(),
            authenticated: false,
        }
    }

    /// Make an authenticated GET request
    async fn get<T: serde::de::DeserializeOwned>(&self, path: &str) -> ProviderResult<T> {
        let token = self
            .access_token
            .as_ref()
            .ok_or_else(|| ProviderError::AuthFailed("No access token configured".to_string()))?;

        let url = format!("{}{}", self.base_url, path);
        debug!(url = %url, "Printify API request");

        let response = self.client.get(&url).bearer_auth(token).send().await?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(ProviderError::ApiError {
                status: status.as_u16(),
                message: body,
            });
        }

        let text = response.text().await?;
        serde_json::from_str(&text).map_err(|e| {
            ProviderError::ParseError(format!(
                "JSON parse error: {} - Body: {}",
                e,
                &text[..text.len().min(500)]
            ))
        })
    }
}

#[async_trait]
impl PodProvider for PrintifyProvider {
    fn code(&self) -> &'static str {
        "printify"
    }

    fn name(&self) -> &'static str {
        "Printify"
    }

    fn base_url(&self) -> &str {
        &self.base_url
    }

    fn rate_limit(&self) -> u32 {
        600
    }

    async fn authenticate(&mut self) -> ProviderResult<()> {
        if self.access_token.is_none() {
            return Err(ProviderError::NotConfigured(
                "PRINTIFY_ACCESS_TOKEN environment variable not set".to_string(),
            ));
        }

        // Verify token by listing shops
        let response: Result<Vec<PrintifyShop>, _> = self.get("/shops.json").await;

        match response {
            Ok(shops) => {
                self.authenticated = true;
                info!(shop_count = shops.len(), "Printify authentication successful");
                Ok(())
            }
            Err(ProviderError::ApiError { status: 401, .. }) => Err(ProviderError::AuthFailed(
                "Invalid access token".to_string(),
            )),
            Err(e) => Err(e),
        }
    }

    fn is_authenticated(&self) -> bool {
        self.authenticated && self.access_token.is_some()
    }

    async fn refresh_auth(&mut self) -> ProviderResult<()> {
        // Printify tokens don't expire via OAuth refresh, re-validate
        self.authenticate().await
    }

    async fn get_products(
        &self,
        page: u32,
        per_page: u32,
    ) -> ProviderResult<CatalogPage<UnifiedProduct>> {
        // Printify catalog endpoint returns all blueprints (no pagination)
        let blueprints: Vec<PrintifyBlueprint> =
            self.get("/catalog/blueprints.json").await?;

        let total = blueprints.len() as u64;

        // Apply manual pagination over the flat response
        let offset = ((page - 1) * per_page) as usize;
        let items: Vec<UnifiedProduct> = blueprints
            .into_iter()
            .skip(offset)
            .take(per_page as usize)
            .map(PrintifyMapper::map_blueprint)
            .collect();

        Ok(CatalogPage::new(items, total, page, per_page))
    }

    async fn get_product(&self, external_id: &str) -> ProviderResult<UnifiedProduct> {
        let path = format!("/catalog/blueprints/{}.json", external_id);

        let blueprint: PrintifyBlueprint = self.get(&path).await?;

        Ok(PrintifyMapper::map_blueprint(blueprint))
    }

    async fn get_variants(&self, product_external_id: &str) -> ProviderResult<Vec<UnifiedVariant>> {
        // First, get print providers for this blueprint
        let providers_path = format!(
            "/catalog/blueprints/{}/print_providers.json",
            product_external_id
        );
        let print_providers: Vec<PrintifyPrintProvider> = self.get(&providers_path).await?;

        // Use the first print provider to get variants
        let first_provider = print_providers.first().ok_or_else(|| {
            ProviderError::NotFound(format!(
                "No print providers found for blueprint {}",
                product_external_id
            ))
        })?;

        debug!(
            blueprint_id = product_external_id,
            print_provider_id = first_provider.id,
            print_provider_name = %first_provider.title,
            "Fetching variants from print provider"
        );

        let variants_path = format!(
            "/catalog/blueprints/{}/print_providers/{}/variants.json",
            product_external_id, first_provider.id
        );
        let variant_response: PrintifyVariantResponse = self.get(&variants_path).await?;

        let variants: Vec<UnifiedVariant> = variant_response
            .variants
            .into_iter()
            .map(PrintifyMapper::map_variant)
            .collect();

        Ok(variants)
    }

    async fn get_print_areas(
        &self,
        product_external_id: &str,
    ) -> ProviderResult<Vec<UnifiedPrintArea>> {
        // Get print providers, then variants from first provider to extract placeholders
        let providers_path = format!(
            "/catalog/blueprints/{}/print_providers.json",
            product_external_id
        );
        let print_providers: Vec<PrintifyPrintProvider> = self.get(&providers_path).await?;

        let first_provider = print_providers.first().ok_or_else(|| {
            ProviderError::NotFound(format!(
                "No print providers found for blueprint {}",
                product_external_id
            ))
        })?;

        let variants_path = format!(
            "/catalog/blueprints/{}/print_providers/{}/variants.json",
            product_external_id, first_provider.id
        );
        let variant_response: PrintifyVariantResponse = self.get(&variants_path).await?;

        Ok(PrintifyMapper::extract_print_areas(
            &variant_response.variants,
        ))
    }

    async fn get_mockup_urls(
        &self,
        _product_external_id: &str,
        _variant_external_id: Option<&str>,
    ) -> ProviderResult<Vec<MockupAsset>> {
        // Printify mockup generation is async (submit job, poll for result).
        // Direct mockup URLs are not available through the catalog API.
        warn!("Printify mockup generation is async and not supported via catalog API");
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
        let provider = PrintifyProvider::new(creds);

        assert_eq!(provider.code(), "printify");
        assert_eq!(provider.name(), "Printify");
        assert_eq!(provider.base_url(), "https://api.printify.com/v1");
        assert_eq!(provider.rate_limit(), 600);
        assert!(!provider.is_authenticated());
    }

    #[test]
    fn test_provider_with_token() {
        let creds = ProviderCredentials {
            access_token: Some("test_token".to_string()),
            ..Default::default()
        };
        let provider = PrintifyProvider::new(creds);

        // Still not authenticated until authenticate() is called
        assert!(!provider.is_authenticated());
    }

    #[test]
    fn test_provider_rate_limit() {
        let creds = ProviderCredentials::default();
        let provider = PrintifyProvider::new(creds);

        // Initial remaining should be 600
        assert_eq!(provider.rate_limit_remaining(), Some(600));
    }
}
