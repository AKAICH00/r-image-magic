//! Gooten API Client Implementation
//!
//! This module implements the PodProvider trait for Gooten,
//! providing access to their product catalog and mockup generation.
//!
//! API Docs: https://docs.gooten.com/
//!
//! Auth: Recipe ID as query parameter (not Bearer token)

use async_trait::async_trait;
use tracing::{debug, info, warn};

use super::mapper::GootenMapper;
use super::models::*;
use crate::domain::catalog::{MockupAsset, UnifiedPrintArea, UnifiedProduct, UnifiedVariant};
use crate::providers::http_client::RateLimitedClient;
use crate::providers::traits::{
    CatalogPage, PodProvider, ProviderCredentials, ProviderError, ProviderResult,
};

/// Gooten API client
pub struct GootenProvider {
    /// Rate-limited HTTP client
    client: RateLimitedClient,

    /// Recipe ID for authentication
    recipe_id: Option<String>,

    /// API base URL
    base_url: String,

    /// Whether authentication is valid
    authenticated: bool,
}

impl GootenProvider {
    /// Create a new Gooten provider instance
    pub fn new(credentials: ProviderCredentials) -> Self {
        GootenProvider {
            client: RateLimitedClient::new(300), // 300 req/min
            recipe_id: credentials.recipe_id,
            base_url: "https://api.gooten.com/v1".to_string(),
            authenticated: false,
        }
    }

    /// Make an authenticated GET request
    ///
    /// Appends `recipeId` as a query parameter. Handles paths that
    /// already contain `?` by using `&` instead.
    async fn get<T: serde::de::DeserializeOwned>(&self, path: &str) -> ProviderResult<T> {
        let recipe_id = self
            .recipe_id
            .as_ref()
            .ok_or_else(|| ProviderError::AuthFailed("No recipe ID configured".to_string()))?;

        let separator = if path.contains('?') { "&" } else { "?" };
        let url = format!(
            "{}{}{}recipeId={}",
            self.base_url, path, separator, recipe_id
        );
        debug!(url = %url, "Gooten API request");

        let response = self.client.get(&url).send().await?;

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
impl PodProvider for GootenProvider {
    fn code(&self) -> &'static str {
        "gooten"
    }

    fn name(&self) -> &'static str {
        "Gooten"
    }

    fn base_url(&self) -> &str {
        &self.base_url
    }

    fn rate_limit(&self) -> u32 {
        300
    }

    async fn authenticate(&mut self) -> ProviderResult<()> {
        if self.recipe_id.is_none() {
            return Err(ProviderError::NotConfigured(
                "GOOTEN_RECIPE_ID environment variable not set".to_string(),
            ));
        }

        // Verify recipe ID by fetching a single product
        let response: Result<GootenProductsResponse, _> =
            self.get("/source/api/products?limit=1").await;

        match response {
            Ok(_) => {
                self.authenticated = true;
                info!("Gooten authentication successful");
                Ok(())
            }
            Err(ProviderError::ApiError { status: 401, .. })
            | Err(ProviderError::ApiError { status: 403, .. }) => Err(ProviderError::AuthFailed(
                "Invalid recipe ID".to_string(),
            )),
            Err(e) => Err(e),
        }
    }

    fn is_authenticated(&self) -> bool {
        self.authenticated && self.recipe_id.is_some()
    }

    async fn refresh_auth(&mut self) -> ProviderResult<()> {
        // Recipe IDs don't expire, but re-validate
        self.authenticate().await
    }

    async fn get_products(
        &self,
        page: u32,
        per_page: u32,
    ) -> ProviderResult<CatalogPage<UnifiedProduct>> {
        let response: GootenProductsResponse = self.get("/source/api/products").await?;

        let all_items: Vec<UnifiedProduct> = response
            .products
            .into_iter()
            .map(GootenMapper::map_product)
            .collect();

        // Client-side pagination since Gooten returns all products
        let total = all_items.len() as u64;
        let start = ((page - 1) * per_page) as usize;
        let items: Vec<UnifiedProduct> = all_items
            .into_iter()
            .skip(start)
            .take(per_page as usize)
            .collect();

        Ok(CatalogPage::new(items, total, page, per_page))
    }

    async fn get_product(&self, external_id: &str) -> ProviderResult<UnifiedProduct> {
        // Gooten doesn't have a single-product endpoint in the same way;
        // filter from the full products list
        let response: GootenProductsResponse = self.get("/source/api/products").await?;

        let product = response
            .products
            .into_iter()
            .find(|p| p.id.to_string() == external_id)
            .ok_or_else(|| {
                ProviderError::NotFound(format!("Product {} not found", external_id))
            })?;

        Ok(GootenMapper::map_product(product))
    }

    async fn get_variants(&self, product_external_id: &str) -> ProviderResult<Vec<UnifiedVariant>> {
        let path = format!(
            "/source/api/productvariants?productId={}",
            product_external_id
        );

        let response: GootenVariantsResponse = self.get(&path).await?;

        let variants: Vec<UnifiedVariant> = response
            .product_variants
            .into_iter()
            .map(GootenMapper::map_variant)
            .collect();

        Ok(variants)
    }

    async fn get_print_areas(
        &self,
        product_external_id: &str,
    ) -> ProviderResult<Vec<UnifiedPrintArea>> {
        // First get variants to find a SKU for template lookup
        let variants = self.get_variants(product_external_id).await?;

        let sku = variants
            .first()
            .and_then(|v| v.sku.clone())
            .ok_or_else(|| {
                ProviderError::NotFound(format!(
                    "No variants with SKU found for product {}",
                    product_external_id
                ))
            })?;

        let path = format!("/source/api/producttemplates?sku={}", sku);
        let response: GootenTemplatesResponse = self.get(&path).await?;

        let mut print_areas = Vec::new();
        for template in response.options {
            for image in &template.images {
                if let Some(layers) = &image.layers {
                    for (i, layer) in layers.iter().enumerate() {
                        print_areas.push(GootenMapper::map_template_layer_to_print_area(
                            layer.clone(),
                            i,
                        ));
                    }
                }
            }
        }

        if print_areas.is_empty() {
            warn!(
                product_id = product_external_id,
                "No print areas found in templates, returning default"
            );
        }

        Ok(print_areas)
    }

    async fn get_mockup_urls(
        &self,
        product_external_id: &str,
        variant_external_id: Option<&str>,
    ) -> ProviderResult<Vec<MockupAsset>> {
        // If a specific variant SKU is provided, use it directly;
        // otherwise look up variants for the product
        let sku = if let Some(vid) = variant_external_id {
            vid.to_string()
        } else {
            let variants = self.get_variants(product_external_id).await?;
            variants
                .first()
                .and_then(|v| v.sku.clone())
                .ok_or_else(|| {
                    ProviderError::NotFound(format!(
                        "No variants with SKU found for product {}",
                        product_external_id
                    ))
                })?
        };

        let path = format!("/source/api/producttemplates?sku={}", sku);
        let response: GootenTemplatesResponse = self.get(&path).await?;

        let assets: Vec<MockupAsset> = response
            .options
            .into_iter()
            .flat_map(GootenMapper::map_template_to_mockup_assets)
            .collect();

        Ok(assets)
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
        let provider = GootenProvider::new(creds);

        assert_eq!(provider.code(), "gooten");
        assert_eq!(provider.name(), "Gooten");
        assert_eq!(provider.base_url(), "https://api.gooten.com/v1");
        assert_eq!(provider.rate_limit(), 300);
        assert!(!provider.is_authenticated());
    }

    #[test]
    fn test_provider_with_recipe_id() {
        let creds = ProviderCredentials {
            recipe_id: Some("test_recipe_id".to_string()),
            ..Default::default()
        };
        let provider = GootenProvider::new(creds);

        // Still not authenticated until authenticate() is called
        assert!(!provider.is_authenticated());
    }

    #[test]
    fn test_provider_no_recipe_id() {
        let creds = ProviderCredentials::default();
        let provider = GootenProvider::new(creds);

        assert!(provider.recipe_id.is_none());
        assert!(!provider.is_authenticated());
    }
}
