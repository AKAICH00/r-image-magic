//! SPOD API Client Implementation
//!
//! This module implements the PodProvider trait for SPOD,
//! providing access to their product catalog (articles), appearances,
//! sizes, print areas, and mockup images.
//!
//! API Docs: https://docs.spod.com/

use async_trait::async_trait;
use tracing::{debug, info, warn};

use super::mapper::SpodMapper;
use super::models::*;
use crate::domain::catalog::{MockupAsset, UnifiedPrintArea, UnifiedProduct, UnifiedVariant};
use crate::providers::http_client::RateLimitedClient;
use crate::providers::traits::{
    CatalogPage, PodProvider, ProviderCredentials, ProviderError, ProviderResult,
};

/// SPOD API client
pub struct SpodProvider {
    /// Rate-limited HTTP client
    client: RateLimitedClient,

    /// Bearer access token
    access_token: Option<String>,

    /// API base URL
    base_url: String,

    /// Whether authentication is valid
    authenticated: bool,
}

impl SpodProvider {
    /// Create a new SPOD provider instance
    pub fn new(credentials: ProviderCredentials) -> Self {
        SpodProvider {
            client: RateLimitedClient::new(200), // 200 req/min
            access_token: credentials.access_token,
            base_url: "https://api.spod.com/api/v1".to_string(),
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
        debug!(url = %url, "SPOD API request");

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
impl PodProvider for SpodProvider {
    fn code(&self) -> &'static str {
        "spod"
    }

    fn name(&self) -> &'static str {
        "SPOD"
    }

    fn base_url(&self) -> &str {
        &self.base_url
    }

    fn rate_limit(&self) -> u32 {
        200
    }

    async fn authenticate(&mut self) -> ProviderResult<()> {
        if self.access_token.is_none() {
            return Err(ProviderError::NotConfigured(
                "SPOD_ACCESS_TOKEN environment variable not set".to_string(),
            ));
        }

        // Verify token by fetching a single article
        let response: Result<SpodListResponse<SpodArticle>, _> =
            self.get("/articles?limit=1").await;

        match response {
            Ok(_) => {
                self.authenticated = true;
                info!("SPOD authentication successful");
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
        // SPOD tokens don't auto-expire, re-validate
        self.authenticate().await
    }

    async fn get_products(
        &self,
        page: u32,
        per_page: u32,
    ) -> ProviderResult<CatalogPage<UnifiedProduct>> {
        let offset = (page - 1) * per_page;
        let path = format!("/articles?offset={}&limit={}", offset, per_page);

        let response: SpodListResponse<SpodArticle> = self.get(&path).await?;

        let items: Vec<UnifiedProduct> = response
            .items
            .into_iter()
            .map(SpodMapper::map_article)
            .collect();

        let total = response.count.unwrap_or(items.len() as i64) as u64;

        Ok(CatalogPage::new(items, total, page, per_page))
    }

    async fn get_product(&self, external_id: &str) -> ProviderResult<UnifiedProduct> {
        let path = format!("/articles/{}", external_id);

        let article: SpodArticle = self.get(&path).await?;

        Ok(SpodMapper::map_article_detail(article))
    }

    async fn get_variants(&self, product_external_id: &str) -> ProviderResult<Vec<UnifiedVariant>> {
        let path = format!("/articles/{}", product_external_id);

        let article: SpodArticle = self.get(&path).await?;

        let appearances = article.appearances.unwrap_or_default();
        let sizes = article.sizes.unwrap_or_default();

        Ok(SpodMapper::map_appearance_and_sizes(appearances, sizes))
    }

    async fn get_print_areas(
        &self,
        product_external_id: &str,
    ) -> ProviderResult<Vec<UnifiedPrintArea>> {
        let path = format!("/articles/{}", product_external_id);

        let article: SpodArticle = self.get(&path).await?;

        let print_areas = article.print_areas.unwrap_or_default();

        Ok(print_areas
            .into_iter()
            .map(SpodMapper::map_print_area)
            .collect())
    }

    async fn get_mockup_urls(
        &self,
        product_external_id: &str,
        variant_external_id: Option<&str>,
    ) -> ProviderResult<Vec<MockupAsset>> {
        let path = format!("/articles/{}", product_external_id);

        let article: SpodArticle = self.get(&path).await?;

        let appearances = article.appearances.unwrap_or_default();

        // If a specific variant (appearance) is requested, filter to it
        let filtered: Vec<SpodAppearance> = if let Some(vid) = variant_external_id {
            // variant external_id is "appearance_id-size_id", extract appearance_id
            let appearance_id_str = vid.split('-').next().unwrap_or(vid);
            if let Ok(appearance_id) = appearance_id_str.parse::<i64>() {
                appearances
                    .into_iter()
                    .filter(|a| a.id == appearance_id)
                    .collect()
            } else {
                warn!(
                    variant_id = vid,
                    "Could not parse appearance ID from variant ID"
                );
                appearances
            }
        } else {
            appearances
        };

        let mut assets = Vec::new();
        for appearance in filtered {
            assets.extend(SpodMapper::map_appearance_to_mockup_assets(appearance));
        }

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
        let provider = SpodProvider::new(creds);

        assert_eq!(provider.code(), "spod");
        assert_eq!(provider.name(), "SPOD");
        assert_eq!(provider.rate_limit(), 200);
        assert!(!provider.is_authenticated());
    }

    #[test]
    fn test_provider_with_token() {
        let creds = ProviderCredentials {
            access_token: Some("test_spod_token".to_string()),
            ..Default::default()
        };
        let provider = SpodProvider::new(creds);

        // Still not authenticated until authenticate() is called
        assert!(!provider.is_authenticated());
    }

    #[test]
    fn test_provider_base_url() {
        let creds = ProviderCredentials::default();
        let provider = SpodProvider::new(creds);

        assert_eq!(provider.base_url(), "https://api.spod.com/api/v1");
    }
}
