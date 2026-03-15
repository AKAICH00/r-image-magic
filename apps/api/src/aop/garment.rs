use image::{DynamicImage, GrayImage};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::uv::UvMode;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum PanelId {
    FrontBody,
    BackBody,
    LeftSleeve,
    RightSleeve,
    CollarOuter,
    CollarInnerVisible,
    OptionalBackNeckLabelZone,
    SeamZones,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum SeamKind {
    LeftShoulder,
    RightShoulder,
    LeftUnderarm,
    RightUnderarm,
    LeftSide,
    RightSide,
    CollarOpening,
    BackNeckLabel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelAssetConfig {
    pub id: PanelId,
    pub mask_path: Option<String>,
    pub displacement_path: Option<String>,
    pub uv_map_path: Option<String>,
    #[serde(default)]
    pub uv_mode: UvMode,
    #[serde(default)]
    pub bleed_px: Option<u32>,
    #[serde(default)]
    pub safe_margin_px: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AopTemplateMetadata {
    pub preset: Option<String>,
    #[serde(default)]
    pub derive_from_print_mask: bool,
    #[serde(default)]
    pub panels: Vec<PanelAssetConfig>,
}

#[derive(Debug, Clone)]
pub struct GarmentPanel {
    pub id: PanelId,
    pub mask: GrayImage,
    pub bleed_mask: GrayImage,
    pub safe_mask: GrayImage,
    pub displacement: Option<DynamicImage>,
    pub uv_map: Option<DynamicImage>,
    pub uv_mode: UvMode,
}

#[derive(Debug, Clone)]
pub struct GarmentSeam {
    pub kind: SeamKind,
    pub mask: GrayImage,
}

#[derive(Debug, Clone)]
pub struct ResolvedGarment {
    pub panels: Vec<GarmentPanel>,
    pub seams: Vec<GarmentSeam>,
    pub garment_mask: GrayImage,
}
