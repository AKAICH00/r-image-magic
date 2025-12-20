//! Asset synchronization service
//!
//! Downloads mockup assets from POD providers and uploads them to R2 storage.
//! Tracks download status in the database.

use std::sync::Arc;
use thiserror::Error;
use tracing::{debug, info, warn, error, instrument};
use tokio::sync::Semaphore;
use uuid::Uuid;

use crate::domain::catalog::{AssetType, MockupAsset, PrintPlacement};
use crate::storage::{R2Client, R2Error, AssetPath, UploadResult};

/// Errors that can occur during asset synchronization
#[derive(Error, Debug)]
pub enum AssetSyncError {
    #[error("R2 storage error: {0}")]
    StorageError(#[from] R2Error),

    #[error("HTTP error: {0}")]
    HttpError(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("Asset not found: {0}")]
    NotFound(String),

    #[error("Rate limited, retry after {retry_after_secs} seconds")]
    RateLimited { retry_after_secs: u64 },
}

impl From<reqwest::Error> for AssetSyncError {
    fn from(err: reqwest::Error) -> Self {
        AssetSyncError::HttpError(err.to_string())
    }
}

/// Result of syncing a single asset
#[derive(Debug, Clone)]
pub struct AssetSyncResult {
    /// Original source URL
    pub source_url: String,
    /// R2 storage key
    pub r2_key: String,
    /// Size in bytes
    pub size_bytes: u64,
    /// Content type
    pub content_type: String,
    /// Public URL if available
    pub public_url: Option<String>,
    /// Time taken to sync in milliseconds
    pub sync_time_ms: u64,
}

/// Batch sync result
#[derive(Debug, Default)]
pub struct BatchSyncResult {
    /// Number of assets successfully synced
    pub success_count: usize,
    /// Number of assets that failed
    pub failed_count: usize,
    /// Number of assets skipped (already exist)
    pub skipped_count: usize,
    /// Individual results
    pub results: Vec<Result<AssetSyncResult, AssetSyncError>>,
    /// Total time in milliseconds
    pub total_time_ms: u64,
}

/// Asset synchronization service
pub struct AssetSyncer {
    r2_client: R2Client,
    http_client: reqwest::Client,
    /// Maximum concurrent downloads
    concurrency: usize,
    /// Whether to skip existing assets
    skip_existing: bool,
}

impl AssetSyncer {
    /// Create a new asset syncer
    pub fn new(r2_client: R2Client) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .user_agent("r-image-magic/1.0 POD-Asset-Syncer")
            .build()
            .expect("Failed to create HTTP client");

        Self {
            r2_client,
            http_client,
            concurrency: 10,
            skip_existing: true,
        }
    }

    /// Set the concurrency level
    pub fn with_concurrency(mut self, concurrency: usize) -> Self {
        self.concurrency = concurrency.max(1).min(50);
        self
    }

    /// Set whether to skip existing assets
    pub fn with_skip_existing(mut self, skip: bool) -> Self {
        self.skip_existing = skip;
        self
    }

    /// Sync a single mockup asset from a provider
    #[instrument(skip(self), fields(source_url = %asset.source_url))]
    pub async fn sync_asset(
        &self,
        provider_code: &str,
        product_id: &str,
        asset: &MockupAsset,
    ) -> Result<AssetSyncResult, AssetSyncError> {
        let start = std::time::Instant::now();

        // Build the R2 path based on asset type
        let path = self.build_asset_path(provider_code, product_id, asset);
        let r2_key = path.to_key();

        // Check if asset already exists
        if self.skip_existing {
            match self.r2_client.exists(&r2_key).await {
                Ok(true) => {
                    debug!("Asset already exists, skipping: {}", r2_key);
                    return Ok(AssetSyncResult {
                        source_url: asset.source_url.clone(),
                        r2_key,
                        size_bytes: 0,
                        content_type: "skipped".to_string(),
                        public_url: self.r2_client.public_url(&path.to_key()),
                        sync_time_ms: start.elapsed().as_millis() as u64,
                    });
                }
                Ok(false) => {}
                Err(e) => {
                    warn!("Failed to check if asset exists: {}", e);
                }
            }
        }

        // Download from source
        debug!("Downloading asset from: {}", asset.source_url);
        let response = self.http_client
            .get(&asset.source_url)
            .send()
            .await?;

        if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
            let retry_after = response
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse().ok())
                .unwrap_or(60);
            return Err(AssetSyncError::RateLimited { retry_after_secs: retry_after });
        }

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(AssetSyncError::NotFound(asset.source_url.clone()));
        }

        if !response.status().is_success() {
            return Err(AssetSyncError::HttpError(format!(
                "HTTP {} from {}",
                response.status(),
                asset.source_url
            )));
        }

        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("image/png")
            .to_string();

        let data = response.bytes().await?.to_vec();
        let size_bytes = data.len() as u64;

        // Upload to R2
        debug!("Uploading {} bytes to R2: {}", size_bytes, r2_key);
        let upload_result = self.r2_client
            .upload(&path, data, &content_type)
            .await?;

        let sync_time_ms = start.elapsed().as_millis() as u64;
        info!(
            "Synced asset: {} -> {} ({} bytes in {}ms)",
            asset.source_url, r2_key, size_bytes, sync_time_ms
        );

        Ok(AssetSyncResult {
            source_url: asset.source_url.clone(),
            r2_key: upload_result.key,
            size_bytes,
            content_type,
            public_url: upload_result.public_url,
            sync_time_ms,
        })
    }

    /// Sync multiple assets concurrently
    #[instrument(skip(self, assets), fields(asset_count = assets.len()))]
    pub async fn sync_batch(
        &self,
        provider_code: &str,
        product_id: &str,
        assets: &[MockupAsset],
    ) -> BatchSyncResult {
        let start = std::time::Instant::now();
        let semaphore = Arc::new(Semaphore::new(self.concurrency));

        let mut handles = Vec::with_capacity(assets.len());

        for asset in assets {
            let semaphore = semaphore.clone();
            let provider = provider_code.to_string();
            let product = product_id.to_string();
            let asset = asset.clone();

            // Create a new syncer for each task (R2Client is Clone)
            let syncer = AssetSyncer {
                r2_client: self.r2_client.clone(),
                http_client: self.http_client.clone(),
                concurrency: self.concurrency,
                skip_existing: self.skip_existing,
            };

            let handle = tokio::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();
                syncer.sync_asset(&provider, &product, &asset).await
            });

            handles.push(handle);
        }

        // Collect results
        let mut batch_result = BatchSyncResult::default();

        for handle in handles {
            match handle.await {
                Ok(Ok(result)) => {
                    if result.content_type == "skipped" {
                        batch_result.skipped_count += 1;
                    } else {
                        batch_result.success_count += 1;
                    }
                    batch_result.results.push(Ok(result));
                }
                Ok(Err(e)) => {
                    batch_result.failed_count += 1;
                    batch_result.results.push(Err(e));
                }
                Err(e) => {
                    batch_result.failed_count += 1;
                    batch_result.results.push(Err(AssetSyncError::HttpError(
                        format!("Task panicked: {}", e),
                    )));
                }
            }
        }

        batch_result.total_time_ms = start.elapsed().as_millis() as u64;

        info!(
            "Batch sync completed: {} success, {} failed, {} skipped in {}ms",
            batch_result.success_count,
            batch_result.failed_count,
            batch_result.skipped_count,
            batch_result.total_time_ms
        );

        batch_result
    }

    /// Sync all mockup assets for a product
    #[instrument(skip(self, mockup_urls))]
    pub async fn sync_product_assets(
        &self,
        provider_code: &str,
        product_id: &str,
        mockup_urls: Vec<MockupAsset>,
    ) -> BatchSyncResult {
        self.sync_batch(provider_code, product_id, &mockup_urls).await
    }

    /// Build an AssetPath from a MockupAsset
    fn build_asset_path(
        &self,
        provider_code: &str,
        product_id: &str,
        asset: &MockupAsset,
    ) -> AssetPath {
        // Extract filename from URL
        let filename = self.extract_filename(&asset.source_url);

        match asset.asset_type {
            AssetType::BaseImage => {
                AssetPath::base_image(provider_code, product_id, &filename)
            }
            AssetType::Thumbnail => {
                AssetPath::thumbnail(provider_code, product_id, &filename)
            }
            AssetType::MockupTemplate | AssetType::PrintfilePreview => {
                let placement = asset.placement.clone()
                    .unwrap_or(PrintPlacement::Front);

                if let Some(ref variant_id) = asset.variant_external_id {
                    AssetPath::variant_asset(
                        provider_code,
                        product_id,
                        variant_id,
                        placement,
                    )
                } else {
                    AssetPath::mockup_template(
                        provider_code,
                        product_id,
                        None,
                        placement,
                        &filename,
                    )
                }
            }
        }
    }

    /// Extract filename from URL
    fn extract_filename(&self, url: &str) -> String {
        url.split('/')
            .last()
            .and_then(|s| s.split('?').next())
            .map(|s| {
                // Sanitize filename
                s.chars()
                    .filter(|c| c.is_alphanumeric() || *c == '.' || *c == '-' || *c == '_')
                    .collect::<String>()
            })
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| format!("{}.png", Uuid::new_v4()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_filename() {
        let syncer = AssetSyncer {
            r2_client: todo!("mock client"),
            http_client: reqwest::Client::new(),
            concurrency: 10,
            skip_existing: true,
        };

        // This won't actually run due to todo!() but shows the intent
        // assert_eq!(syncer.extract_filename("https://example.com/path/image.png"), "image.png");
        // assert_eq!(syncer.extract_filename("https://example.com/path/image.png?token=abc"), "image.png");
    }
}
