use image::{DynamicImage, GrayImage};

use crate::engine::displacement::apply_displacement;
use crate::imageops::apply_mask;

pub fn apply_panel_displacement(
    artwork: &DynamicImage,
    panel_mask: &GrayImage,
    displacement: Option<&DynamicImage>,
    strength: f64,
) -> DynamicImage {
    let displaced = match displacement {
        Some(map) => apply_displacement(artwork, map, strength),
        None => artwork.clone(),
    };

    apply_mask(&displaced, panel_mask)
}
