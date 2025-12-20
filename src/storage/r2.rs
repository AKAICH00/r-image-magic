//! Cloudflare R2 storage client for POD assets
//!
//! R2 is S3-compatible, so we use aws-sdk-s3 with custom endpoint configuration.
//!
//! ## Folder Structure
//! ```text
//! r-image-magic-pod-assets/
//! ├── {provider}/                     # printful, printify, gelato, spod, gooten
//! │   ├── products/
//! │   │   └── {product_id}/
//! │   │       ├── base/               # Base product images
//! │   │       │   └── {filename}.png
//! │   │       ├── mockups/            # Mockup template images
//! │   │       │   └── {placement}_{variant_id}.png
//! │   │       └── thumbnails/         # Thumbnail images
//! │   │           └── thumb_{filename}.png
//! │   └── variants/
//! │       └── {variant_id}/
//! │           └── {placement}.png     # Variant-specific mockups
//! └── generated/                      # User-generated mockups (optional cache)
//!     └── {date}/
//!         └── {uuid}.png
//! ```

use aws_sdk_s3::{
    Client as S3Client,
    config::{Builder, Credentials, Region},
    primitives::ByteStream,
    error::SdkError,
    operation::put_object::PutObjectError,
    operation::get_object::GetObjectError,
    operation::head_object::HeadObjectError,
    operation::delete_object::DeleteObjectError,
    operation::list_objects_v2::ListObjectsV2Error,
};
use std::fmt;
use thiserror::Error;
use tracing::{debug, info, warn, instrument};

use crate::config::R2Settings;
use crate::domain::catalog::{AssetType, PrintPlacement};

/// Errors that can occur during R2 operations
#[derive(Error, Debug)]
pub enum R2Error {
    #[error("R2 not configured")]
    NotConfigured,

    #[error("Upload failed: {0}")]
    UploadFailed(String),

    #[error("Download failed: {0}")]
    DownloadFailed(String),

    #[error("Object not found: {0}")]
    NotFound(String),

    #[error("Delete failed: {0}")]
    DeleteFailed(String),

    #[error("List failed: {0}")]
    ListFailed(String),

    #[error("Invalid path: {0}")]
    InvalidPath(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

impl<E: fmt::Debug> From<SdkError<E>> for R2Error {
    fn from(err: SdkError<E>) -> Self {
        R2Error::UploadFailed(format!("{:?}", err))
    }
}

/// Represents a path to an asset in R2
#[derive(Debug, Clone)]
pub struct AssetPath {
    /// Provider code (printful, printify, etc.)
    pub provider: String,
    /// Product ID from the provider
    pub product_id: String,
    /// Optional variant ID
    pub variant_id: Option<String>,
    /// Asset type (base, mockup, thumbnail)
    pub asset_type: AssetType,
    /// Print placement (front, back, etc.)
    pub placement: Option<PrintPlacement>,
    /// Filename with extension
    pub filename: String,
}

impl AssetPath {
    /// Create a new asset path for a base product image
    pub fn base_image(provider: &str, product_id: &str, filename: &str) -> Self {
        Self {
            provider: provider.to_lowercase(),
            product_id: product_id.to_string(),
            variant_id: None,
            asset_type: AssetType::BaseImage,
            placement: None,
            filename: filename.to_string(),
        }
    }

    /// Create a new asset path for a mockup template
    pub fn mockup_template(
        provider: &str,
        product_id: &str,
        variant_id: Option<&str>,
        placement: PrintPlacement,
        filename: &str,
    ) -> Self {
        Self {
            provider: provider.to_lowercase(),
            product_id: product_id.to_string(),
            variant_id: variant_id.map(String::from),
            asset_type: AssetType::MockupTemplate,
            placement: Some(placement),
            filename: filename.to_string(),
        }
    }

    /// Create a new asset path for a thumbnail
    pub fn thumbnail(provider: &str, product_id: &str, filename: &str) -> Self {
        Self {
            provider: provider.to_lowercase(),
            product_id: product_id.to_string(),
            variant_id: None,
            asset_type: AssetType::Thumbnail,
            placement: None,
            filename: format!("thumb_{}", filename),
        }
    }

    /// Create a path for a variant-specific asset
    pub fn variant_asset(
        provider: &str,
        product_id: &str,
        variant_id: &str,
        placement: PrintPlacement,
    ) -> Self {
        let filename = format!("{}.png", placement.as_str());
        Self {
            provider: provider.to_lowercase(),
            product_id: product_id.to_string(),
            variant_id: Some(variant_id.to_string()),
            asset_type: AssetType::MockupTemplate,
            placement: Some(placement),
            filename,
        }
    }

    /// Convert to R2 object key
    pub fn to_key(&self) -> String {
        let asset_folder = match self.asset_type {
            AssetType::BaseImage => "base",
            AssetType::MockupTemplate => "mockups",
            AssetType::Thumbnail => "thumbnails",
            AssetType::PrintfilePreview => "printfiles",
        };

        if let Some(ref variant_id) = self.variant_id {
            // Variant-specific path: {provider}/variants/{variant_id}/{filename}
            format!(
                "{}/variants/{}/{}",
                self.provider,
                variant_id,
                self.filename
            )
        } else {
            // Product-level path: {provider}/products/{product_id}/{asset_folder}/{filename}
            format!(
                "{}/products/{}/{}/{}",
                self.provider,
                self.product_id,
                asset_folder,
                self.filename
            )
        }
    }

    /// Parse an R2 key back into an AssetPath
    pub fn from_key(key: &str) -> Result<Self, R2Error> {
        let parts: Vec<&str> = key.split('/').collect();

        if parts.len() < 4 {
            return Err(R2Error::InvalidPath(format!("Key too short: {}", key)));
        }

        let provider = parts[0].to_string();

        if parts[1] == "variants" && parts.len() >= 4 {
            // Variant path: {provider}/variants/{variant_id}/{filename}
            let variant_id = parts[2].to_string();
            let filename = parts[3..].join("/");

            Ok(Self {
                provider,
                product_id: String::new(), // Not in variant path
                variant_id: Some(variant_id),
                asset_type: AssetType::MockupTemplate,
                placement: None,
                filename,
            })
        } else if parts[1] == "products" && parts.len() >= 5 {
            // Product path: {provider}/products/{product_id}/{asset_folder}/{filename}
            let product_id = parts[2].to_string();
            let asset_folder = parts[3];
            let filename = parts[4..].join("/");

            let asset_type = match asset_folder {
                "base" => AssetType::BaseImage,
                "mockups" => AssetType::MockupTemplate,
                "thumbnails" => AssetType::Thumbnail,
                "printfiles" => AssetType::PrintfilePreview,
                _ => AssetType::BaseImage,
            };

            Ok(Self {
                provider,
                product_id,
                variant_id: None,
                asset_type,
                placement: None,
                filename,
            })
        } else {
            Err(R2Error::InvalidPath(format!("Unrecognized path structure: {}", key)))
        }
    }
}

/// Result of an upload operation
#[derive(Debug, Clone)]
pub struct UploadResult {
    /// The R2 object key
    pub key: String,
    /// Size in bytes
    pub size: u64,
    /// Content type
    pub content_type: String,
    /// ETag from R2
    pub etag: Option<String>,
    /// Public URL if configured
    pub public_url: Option<String>,
}

/// Cloudflare R2 client for POD asset storage
#[derive(Clone)]
pub struct R2Client {
    client: S3Client,
    bucket: String,
    public_url_prefix: Option<String>,
}

impl R2Client {
    /// Create a new R2 client from settings
    pub async fn new(settings: &R2Settings) -> Result<Self, R2Error> {
        // R2 endpoint format: https://{account_id}.r2.cloudflarestorage.com
        let endpoint = format!(
            "https://{}.r2.cloudflarestorage.com",
            settings.account_id
        );

        debug!("Creating R2 client with endpoint: {}", endpoint);

        let credentials = Credentials::new(
            &settings.access_key_id,
            &settings.secret_access_key,
            None, // session token
            None, // expiry
            "r2-static-credentials",
        );

        let config = Builder::new()
            .endpoint_url(&endpoint)
            .region(Region::new("auto")) // R2 uses "auto" region
            .credentials_provider(credentials)
            .force_path_style(true) // Required for R2
            .build();

        let client = S3Client::from_conf(config);

        Ok(Self {
            client,
            bucket: settings.bucket_name.clone(),
            public_url_prefix: settings.public_url_prefix.clone(),
        })
    }

    /// Create from environment variables
    pub async fn from_env() -> Result<Self, R2Error> {
        let account_id = std::env::var("R2_ACCOUNT_ID")
            .map_err(|_| R2Error::NotConfigured)?;
        let access_key_id = std::env::var("R2_ACCESS_KEY_ID")
            .map_err(|_| R2Error::NotConfigured)?;
        let secret_access_key = std::env::var("R2_SECRET_ACCESS_KEY")
            .map_err(|_| R2Error::NotConfigured)?;
        let bucket_name = std::env::var("R2_BUCKET_NAME")
            .unwrap_or_else(|_| "r-image-magic-pod-assets".to_string());
        let public_url_prefix = std::env::var("R2_PUBLIC_URL_PREFIX").ok();

        let settings = R2Settings {
            account_id,
            access_key_id,
            secret_access_key,
            bucket_name,
            public_url_prefix,
        };

        Self::new(&settings).await
    }

    /// Get the bucket name
    pub fn bucket(&self) -> &str {
        &self.bucket
    }

    /// Upload bytes to R2
    #[instrument(skip(self, data), fields(key = %path.to_key(), size = data.len()))]
    pub async fn upload(
        &self,
        path: &AssetPath,
        data: Vec<u8>,
        content_type: &str,
    ) -> Result<UploadResult, R2Error> {
        let key = path.to_key();
        let size = data.len() as u64;

        debug!("Uploading {} bytes to R2: {}", size, key);

        let result = self.client
            .put_object()
            .bucket(&self.bucket)
            .key(&key)
            .body(ByteStream::from(data))
            .content_type(content_type)
            .send()
            .await
            .map_err(|e| R2Error::UploadFailed(format!("{:?}", e)))?;

        let etag = result.e_tag().map(String::from);

        let public_url = self.public_url_prefix.as_ref().map(|prefix| {
            format!("{}/{}", prefix.trim_end_matches('/'), key)
        });

        info!("Uploaded to R2: {} ({} bytes)", key, size);

        Ok(UploadResult {
            key,
            size,
            content_type: content_type.to_string(),
            etag,
            public_url,
        })
    }

    /// Upload a file from disk to R2
    #[instrument(skip(self), fields(key = %path.to_key()))]
    pub async fn upload_file(
        &self,
        path: &AssetPath,
        file_path: &std::path::Path,
        content_type: &str,
    ) -> Result<UploadResult, R2Error> {
        let data = tokio::fs::read(file_path).await?;
        self.upload(path, data, content_type).await
    }

    /// Download an object from R2
    #[instrument(skip(self))]
    pub async fn download(&self, key: &str) -> Result<Vec<u8>, R2Error> {
        debug!("Downloading from R2: {}", key);

        let result = self.client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| {
                if is_not_found_error(&e) {
                    R2Error::NotFound(key.to_string())
                } else {
                    R2Error::DownloadFailed(format!("{:?}", e))
                }
            })?;

        let data = result.body
            .collect()
            .await
            .map_err(|e| R2Error::DownloadFailed(format!("Failed to read body: {:?}", e)))?
            .into_bytes()
            .to_vec();

        debug!("Downloaded {} bytes from R2: {}", data.len(), key);

        Ok(data)
    }

    /// Check if an object exists in R2
    #[instrument(skip(self))]
    pub async fn exists(&self, key: &str) -> Result<bool, R2Error> {
        match self.client
            .head_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
        {
            Ok(_) => Ok(true),
            Err(e) if is_not_found_error(&e) => Ok(false),
            Err(e) => Err(R2Error::DownloadFailed(format!("{:?}", e))),
        }
    }

    /// Delete an object from R2
    #[instrument(skip(self))]
    pub async fn delete(&self, key: &str) -> Result<(), R2Error> {
        debug!("Deleting from R2: {}", key);

        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| R2Error::DeleteFailed(format!("{:?}", e)))?;

        info!("Deleted from R2: {}", key);
        Ok(())
    }

    /// List objects with a prefix
    #[instrument(skip(self))]
    pub async fn list(&self, prefix: &str, max_keys: Option<i32>) -> Result<Vec<String>, R2Error> {
        debug!("Listing R2 objects with prefix: {}", prefix);

        let mut keys = Vec::new();
        let mut continuation_token: Option<String> = None;
        let max = max_keys.unwrap_or(1000);

        loop {
            let mut request = self.client
                .list_objects_v2()
                .bucket(&self.bucket)
                .prefix(prefix)
                .max_keys(max);

            if let Some(token) = continuation_token.take() {
                request = request.continuation_token(token);
            }

            let result = request
                .send()
                .await
                .map_err(|e| R2Error::ListFailed(format!("{:?}", e)))?;

            if let Some(contents) = result.contents {
                for object in contents {
                    if let Some(key) = object.key {
                        keys.push(key);
                    }
                }
            }

            if result.is_truncated.unwrap_or(false) {
                continuation_token = result.next_continuation_token;
            } else {
                break;
            }

            // Respect max_keys limit
            if keys.len() >= max as usize {
                keys.truncate(max as usize);
                break;
            }
        }

        debug!("Listed {} objects with prefix: {}", keys.len(), prefix);
        Ok(keys)
    }

    /// List all products for a provider
    pub async fn list_products(&self, provider: &str) -> Result<Vec<String>, R2Error> {
        let prefix = format!("{}/products/", provider.to_lowercase());
        let keys = self.list(&prefix, None).await?;

        // Extract unique product IDs
        let mut product_ids: std::collections::HashSet<String> = std::collections::HashSet::new();

        for key in keys {
            let parts: Vec<&str> = key.split('/').collect();
            if parts.len() >= 3 && parts[1] == "products" {
                product_ids.insert(parts[2].to_string());
            }
        }

        Ok(product_ids.into_iter().collect())
    }

    /// Get the public URL for an asset
    pub fn public_url(&self, key: &str) -> Option<String> {
        self.public_url_prefix.as_ref().map(|prefix| {
            format!("{}/{}", prefix.trim_end_matches('/'), key)
        })
    }

    /// Download an asset from a URL and upload to R2
    #[instrument(skip(self, http_client))]
    pub async fn mirror_from_url(
        &self,
        source_url: &str,
        path: &AssetPath,
        http_client: &reqwest::Client,
    ) -> Result<UploadResult, R2Error> {
        debug!("Mirroring {} to R2", source_url);

        let response = http_client
            .get(source_url)
            .send()
            .await
            .map_err(|e| R2Error::DownloadFailed(format!("HTTP error: {:?}", e)))?;

        if !response.status().is_success() {
            return Err(R2Error::DownloadFailed(format!(
                "HTTP {} from {}",
                response.status(),
                source_url
            )));
        }

        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("image/png")
            .to_string();

        let data = response
            .bytes()
            .await
            .map_err(|e| R2Error::DownloadFailed(format!("Body error: {:?}", e)))?
            .to_vec();

        self.upload(path, data, &content_type).await
    }
}

/// Helper to check if an SDK error is a "not found" error
fn is_not_found_error<E: fmt::Debug>(err: &SdkError<E>) -> bool {
    let debug_str = format!("{:?}", err);
    debug_str.contains("NoSuchKey") || debug_str.contains("NotFound") || debug_str.contains("404")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_path_to_key() {
        let path = AssetPath::base_image("printful", "12345", "front.png");
        assert_eq!(path.to_key(), "printful/products/12345/base/front.png");

        let path = AssetPath::mockup_template(
            "printful",
            "12345",
            Some("var-001"),
            PrintPlacement::Front,
            "mockup.png",
        );
        assert_eq!(path.to_key(), "printful/variants/var-001/mockup.png");

        let path = AssetPath::thumbnail("printify", "67890", "preview.png");
        assert_eq!(path.to_key(), "printify/products/67890/thumbnails/thumb_preview.png");
    }

    #[test]
    fn test_asset_path_from_key() {
        let path = AssetPath::from_key("printful/products/12345/base/front.png").unwrap();
        assert_eq!(path.provider, "printful");
        assert_eq!(path.product_id, "12345");
        assert!(matches!(path.asset_type, AssetType::BaseImage));
        assert_eq!(path.filename, "front.png");

        let path = AssetPath::from_key("printify/variants/var-001/back.png").unwrap();
        assert_eq!(path.provider, "printify");
        assert_eq!(path.variant_id, Some("var-001".to_string()));
        assert_eq!(path.filename, "back.png");
    }
}
