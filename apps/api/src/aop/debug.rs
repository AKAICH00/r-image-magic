use image::{DynamicImage, GrayImage};

use crate::imageops::{composite_over, feather_mask, mask_outline, overlay_color_mask};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct DebugOverlayArtifact {
    pub name: String,
    pub url: String,
}

pub fn build_mask_overlay(name: &str, mask: &GrayImage, color: [u8; 4]) -> (String, DynamicImage) {
    (name.to_string(), overlay_color_mask(mask, color))
}

pub fn build_outline_overlay(
    name: &str,
    mask: &GrayImage,
    color: [u8; 4],
) -> (String, DynamicImage) {
    let outline = feather_mask(&mask_outline(mask), 0.8);
    (name.to_string(), overlay_color_mask(&outline, color))
}

pub fn merge_overlays(base: &DynamicImage, overlays: &[DynamicImage]) -> DynamicImage {
    overlays.iter().fold(base.clone(), |image, overlay| {
        composite_over(&image, overlay)
    })
}
