//! Sync module for POD catalog synchronization
//!
//! Handles downloading product catalogs and assets from POD providers
//! and storing them in our database and R2 storage.

mod asset_sync;
mod orchestrator;

pub use asset_sync::{AssetSyncError, AssetSyncResult, AssetSyncer};
pub use orchestrator::{SyncJob, SyncJobStatus, SyncJobType, SyncOrchestrator};
