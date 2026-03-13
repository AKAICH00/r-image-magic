//! Domain types and models

pub mod catalog;
mod placement;

pub use catalog::{
    AssetType, DbPodMockupAsset, DbPodPrintArea, DbPodProduct, DbPodProductVariant, DbPodProvider,
    DbPodSyncJob, MockupAsset, PrintConstraints, PrintPlacement, ProductType, UnifiedPrintArea,
    UnifiedProduct, UnifiedVariant,
};
pub use placement::{CoordinateSpace, PlacementSpec, PlacementType};
