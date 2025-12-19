//! Domain types and models

mod placement;
pub mod catalog;

pub use placement::{PlacementSpec, PlacementType, CoordinateSpace};
pub use catalog::{
    ProductType, PrintPlacement, AssetType,
    UnifiedProduct, UnifiedVariant, UnifiedPrintArea, MockupAsset, PrintConstraints,
    DbPodProvider, DbPodProduct, DbPodProductVariant, DbPodPrintArea, DbPodMockupAsset, DbPodSyncJob,
};
