//! Storage module for POD asset management
//!
//! Provides Cloudflare R2 integration for storing and retrieving POD mockup assets.
//! R2 is S3-compatible, so we use the AWS SDK.

mod r2;

pub use r2::{R2Client, R2Error, AssetPath, UploadResult};
