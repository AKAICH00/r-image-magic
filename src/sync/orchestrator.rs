//! Sync orchestrator for POD catalog synchronization
//!
//! Manages sync jobs, tracks progress, and coordinates between providers,
//! database, and R2 storage.

use std::sync::Arc;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{debug, info, warn, error, instrument};
use uuid::Uuid;

use crate::db::DbPool;
use crate::domain::catalog::{MockupAsset, UnifiedProduct};
use crate::providers::{PodProvider, ProviderFactory, ProviderCredentials, ProviderError};
use crate::storage::R2Client;

use super::asset_sync::{AssetSyncer, AssetSyncError, BatchSyncResult};

/// Errors that can occur during sync orchestration
#[derive(Error, Debug)]
pub enum SyncOrchestratorError {
    #[error("Provider error: {0}")]
    ProviderError(#[from] ProviderError),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Asset sync error: {0}")]
    AssetSyncError(#[from] AssetSyncError),

    #[error("Provider not found: {0}")]
    ProviderNotFound(String),

    #[error("Job not found: {0}")]
    JobNotFound(Uuid),

    #[error("Job already running for provider: {0}")]
    JobAlreadyRunning(String),
}

/// Type of sync job
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncJobType {
    /// Full catalog sync from provider
    FullCatalog,
    /// Incremental sync (only new/changed items)
    Incremental,
    /// Sync only assets for existing products
    AssetsOnly,
    /// Sync a single product
    SingleProduct,
}

impl std::fmt::Display for SyncJobType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyncJobType::FullCatalog => write!(f, "full_catalog"),
            SyncJobType::Incremental => write!(f, "incremental"),
            SyncJobType::AssetsOnly => write!(f, "assets_only"),
            SyncJobType::SingleProduct => write!(f, "single_product"),
        }
    }
}

/// Status of a sync job
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncJobStatus {
    /// Job is pending (not yet started)
    Pending,
    /// Job is currently running
    Running,
    /// Job completed successfully
    Completed,
    /// Job failed with an error
    Failed,
    /// Job was cancelled
    Cancelled,
}

impl std::fmt::Display for SyncJobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyncJobStatus::Pending => write!(f, "pending"),
            SyncJobStatus::Running => write!(f, "running"),
            SyncJobStatus::Completed => write!(f, "completed"),
            SyncJobStatus::Failed => write!(f, "failed"),
            SyncJobStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

/// Represents a sync job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncJob {
    /// Unique job ID
    pub id: Uuid,
    /// Provider code (printful, printify, etc.)
    pub provider_code: String,
    /// Type of sync
    pub job_type: SyncJobType,
    /// Current status
    pub status: SyncJobStatus,
    /// Total items to process
    pub total_items: u32,
    /// Items processed so far
    pub processed_items: u32,
    /// Items that failed
    pub failed_items: u32,
    /// When the job was created
    pub created_at: DateTime<Utc>,
    /// When the job started running
    pub started_at: Option<DateTime<Utc>>,
    /// When the job completed/failed
    pub completed_at: Option<DateTime<Utc>>,
    /// Error message if failed
    pub error_message: Option<String>,
    /// Optional: specific product ID for SingleProduct jobs
    pub product_id: Option<String>,
}

impl SyncJob {
    /// Create a new pending job
    pub fn new(provider_code: &str, job_type: SyncJobType) -> Self {
        Self {
            id: Uuid::new_v4(),
            provider_code: provider_code.to_string(),
            job_type,
            status: SyncJobStatus::Pending,
            total_items: 0,
            processed_items: 0,
            failed_items: 0,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            error_message: None,
            product_id: None,
        }
    }

    /// Mark the job as started
    pub fn start(&mut self) {
        self.status = SyncJobStatus::Running;
        self.started_at = Some(Utc::now());
    }

    /// Set total items to process
    pub fn set_total(&mut self, total: u32) {
        self.total_items = total;
    }

    /// Increment processed count
    pub fn increment_processed(&mut self) {
        self.processed_items += 1;
    }

    /// Increment failed count
    pub fn increment_failed(&mut self) {
        self.failed_items += 1;
    }

    /// Mark the job as completed
    pub fn complete(&mut self) {
        self.status = SyncJobStatus::Completed;
        self.completed_at = Some(Utc::now());
    }

    /// Mark the job as failed
    pub fn fail(&mut self, error: &str) {
        self.status = SyncJobStatus::Failed;
        self.completed_at = Some(Utc::now());
        self.error_message = Some(error.to_string());
    }

    /// Get progress percentage
    pub fn progress(&self) -> f32 {
        if self.total_items == 0 {
            0.0
        } else {
            (self.processed_items as f32 / self.total_items as f32) * 100.0
        }
    }

    /// Get duration in seconds
    pub fn duration_secs(&self) -> Option<i64> {
        let end = self.completed_at.or(Some(Utc::now()))?;
        let start = self.started_at?;
        Some((end - start).num_seconds())
    }
}

/// Sync progress callback
pub type ProgressCallback = Box<dyn Fn(&SyncJob) + Send + Sync>;

/// Sync orchestrator for managing catalog synchronization
pub struct SyncOrchestrator {
    db_pool: Option<DbPool>,
    r2_client: Option<R2Client>,
    /// Active jobs by provider code
    active_jobs: std::sync::RwLock<std::collections::HashMap<String, SyncJob>>,
}

impl SyncOrchestrator {
    /// Create a new sync orchestrator
    pub fn new(db_pool: Option<DbPool>, r2_client: Option<R2Client>) -> Self {
        Self {
            db_pool,
            r2_client,
            active_jobs: std::sync::RwLock::new(std::collections::HashMap::new()),
        }
    }

    /// Check if a sync job is running for a provider
    pub fn is_running(&self, provider_code: &str) -> bool {
        let jobs = self.active_jobs.read().unwrap();
        jobs.get(provider_code)
            .map(|j| j.status == SyncJobStatus::Running)
            .unwrap_or(false)
    }

    /// Get the current job for a provider
    pub fn get_job(&self, provider_code: &str) -> Option<SyncJob> {
        let jobs = self.active_jobs.read().unwrap();
        jobs.get(provider_code).cloned()
    }

    /// Get all jobs
    pub fn get_all_jobs(&self) -> Vec<SyncJob> {
        let jobs = self.active_jobs.read().unwrap();
        jobs.values().cloned().collect()
    }

    /// Start a full catalog sync for a provider
    #[instrument(skip(self, on_progress))]
    pub async fn start_full_sync(
        &self,
        provider_code: &str,
        on_progress: Option<ProgressCallback>,
    ) -> Result<SyncJob, SyncOrchestratorError> {
        // Check if already running
        if self.is_running(provider_code) {
            return Err(SyncOrchestratorError::JobAlreadyRunning(
                provider_code.to_string(),
            ));
        }

        // Create new job
        let mut job = SyncJob::new(provider_code, SyncJobType::FullCatalog);
        job.start();

        // Store job
        {
            let mut jobs = self.active_jobs.write().unwrap();
            jobs.insert(provider_code.to_string(), job.clone());
        }

        // Get provider credentials and create provider
        let credentials = ProviderCredentials::from_env(provider_code);
        let mut provider = ProviderFactory::create(provider_code, credentials)
            .ok_or_else(|| SyncOrchestratorError::ProviderNotFound(provider_code.to_string()))?;

        // Authenticate
        provider.authenticate().await?;

        info!("Starting full catalog sync for {}", provider_code);

        // Sync products in pages
        let mut page = 1;
        let per_page = 50;
        let mut total_products = 0;

        loop {
            match provider.get_products(page, per_page).await {
                Ok(catalog_page) => {
                    let products = catalog_page.items;
                    let has_more = catalog_page.has_more;

                    if page == 1 {
                        // Set total from catalog page
                        job.set_total(catalog_page.total as u32);
                        self.update_job(&job);
                    }

                    for product in &products {
                        // Process product
                        match self.sync_product(provider_code, product, &*provider).await {
                            Ok(_) => {
                                job.increment_processed();
                            }
                            Err(e) => {
                                warn!("Failed to sync product {}: {}", product.external_id, e);
                                job.increment_failed();
                            }
                        }

                        total_products += 1;

                        // Update job and call progress callback
                        self.update_job(&job);
                        if let Some(ref callback) = on_progress {
                            callback(&job);
                        }
                    }

                    if !has_more {
                        break;
                    }
                    page += 1;
                }
                Err(e) => {
                    error!("Failed to get products page {}: {}", page, e);
                    job.fail(&e.to_string());
                    self.update_job(&job);
                    return Err(e.into());
                }
            }
        }

        // Update final counts
        job.set_total(total_products);
        job.complete();
        self.update_job(&job);

        info!(
            "Completed full sync for {}: {} products ({} failed)",
            provider_code, job.processed_items, job.failed_items
        );

        Ok(job)
    }

    /// Sync a single product and its assets
    #[instrument(skip(self, product, provider))]
    async fn sync_product(
        &self,
        provider_code: &str,
        product: &UnifiedProduct,
        provider: &dyn PodProvider,
    ) -> Result<(), SyncOrchestratorError> {
        debug!("Syncing product: {} - {}", product.external_id, product.name);

        // Get mockup URLs for the product
        let mockup_assets = provider
            .get_mockup_urls(&product.external_id, None)
            .await
            .unwrap_or_default();

        if mockup_assets.is_empty() {
            debug!("No mockup assets for product {}", product.external_id);
            return Ok(());
        }

        // Sync assets to R2 if R2 client is configured
        if let Some(ref r2_client) = self.r2_client {
            let syncer = AssetSyncer::new(r2_client.clone())
                .with_concurrency(5)
                .with_skip_existing(true);

            let result = syncer
                .sync_product_assets(provider_code, &product.external_id, mockup_assets)
                .await;

            debug!(
                "Synced {} assets for product {} ({} failed, {} skipped)",
                result.success_count,
                product.external_id,
                result.failed_count,
                result.skipped_count
            );
        }

        // TODO: Store product in database
        // if let Some(ref pool) = self.db_pool {
        //     // Insert/update product in pod_products table
        // }

        Ok(())
    }

    /// Update job in storage
    fn update_job(&self, job: &SyncJob) {
        let mut jobs = self.active_jobs.write().unwrap();
        jobs.insert(job.provider_code.clone(), job.clone());

        // TODO: Persist to database
    }

    /// Cancel a running job
    pub fn cancel_job(&self, provider_code: &str) -> Result<SyncJob, SyncOrchestratorError> {
        let mut jobs = self.active_jobs.write().unwrap();
        if let Some(mut job) = jobs.get_mut(provider_code) {
            if job.status == SyncJobStatus::Running {
                job.status = SyncJobStatus::Cancelled;
                job.completed_at = Some(Utc::now());
                return Ok(job.clone());
            }
        }
        Err(SyncOrchestratorError::JobNotFound(Uuid::nil()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_job_progress() {
        let mut job = SyncJob::new("printful", SyncJobType::FullCatalog);
        job.set_total(100);
        job.processed_items = 50;
        assert_eq!(job.progress(), 50.0);
    }

    #[test]
    fn test_sync_job_lifecycle() {
        let mut job = SyncJob::new("printify", SyncJobType::FullCatalog);
        assert_eq!(job.status, SyncJobStatus::Pending);

        job.start();
        assert_eq!(job.status, SyncJobStatus::Running);
        assert!(job.started_at.is_some());

        job.complete();
        assert_eq!(job.status, SyncJobStatus::Completed);
        assert!(job.completed_at.is_some());
    }
}
