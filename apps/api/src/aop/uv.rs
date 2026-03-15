use image::{DynamicImage, GenericImageView, GrayImage, RgbaImage};
use serde::{Deserialize, Serialize};

use crate::imageops::{bounding_box, sample_bilinear};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UvMode {
    PlanarFront,
    PlanarBack,
    SleeveCylindrical,
    CollarArc,
    DerivedProjection,
    ExplicitUvMap,
}

impl Default for UvMode {
    fn default() -> Self {
        Self::DerivedProjection
    }
}

pub fn project_artwork(
    source: &DynamicImage,
    panel_mask: &GrayImage,
    uv_map: Option<&DynamicImage>,
    garment_bbox: (u32, u32, u32, u32),
    panel_bbox: (u32, u32, u32, u32),
    uv_mode: UvMode,
    tile_scale: f32,
    pattern_offset: (i32, i32),
) -> (DynamicImage, DynamicImage) {
    let source = source.to_rgba8();
    let (width, height) = panel_mask.dimensions();
    let mut output = RgbaImage::new(width, height);
    let mut stretch = RgbaImage::new(width, height);

    let garment_width = (garment_bbox.2 - garment_bbox.0).max(1) as f32;
    let garment_height = (garment_bbox.3 - garment_bbox.1).max(1) as f32;
    let panel_width = (panel_bbox.2 - panel_bbox.0).max(1) as f32;
    let panel_height = (panel_bbox.3 - panel_bbox.1).max(1) as f32;
    let tile_scale = tile_scale.max(0.1);
    let (sw, sh) = source.dimensions();
    let uv_map = uv_map.map(DynamicImage::to_rgba8);

    for y in 0..height {
        for x in 0..width {
            if panel_mask.get_pixel(x, y).0[0] == 0 {
                continue;
            }

            let (u, v, stretch_factor) = match uv_mode {
                UvMode::PlanarFront | UvMode::PlanarBack | UvMode::DerivedProjection => (
                    (x.saturating_sub(garment_bbox.0) as f32 / garment_width),
                    (y.saturating_sub(garment_bbox.1) as f32 / garment_height),
                    0.35,
                ),
                UvMode::SleeveCylindrical => {
                    let x_norm =
                        (x.saturating_sub(panel_bbox.0) as f32 / panel_width).clamp(0.0, 1.0);
                    let y_norm =
                        (y.saturating_sub(panel_bbox.1) as f32 / panel_height).clamp(0.0, 1.0);
                    let u = ((x_norm - 0.5) * std::f32::consts::PI).sin() * 0.5 + 0.5;
                    (u, y_norm, (x_norm - 0.5).abs())
                }
                UvMode::CollarArc => {
                    let cx = (panel_bbox.0 + panel_bbox.2) as f32 * 0.5;
                    let cy = (panel_bbox.1 + panel_bbox.3) as f32 * 0.5;
                    let dx = x as f32 - cx;
                    let dy = y as f32 - cy;
                    let angle = dy.atan2(dx);
                    let radius = (dx * dx + dy * dy).sqrt();
                    let u = (angle / std::f32::consts::TAU + 0.5).fract();
                    let v = (radius / panel_width.max(panel_height)).clamp(0.0, 1.0);
                    (u, v, 0.5)
                }
                UvMode::ExplicitUvMap => {
                    if let Some(ref uv_map) = uv_map {
                        let uv_x = ((x as f32 / width.max(1) as f32) * (uv_map.width() - 1) as f32)
                            .round() as u32;
                        let uv_y = ((y as f32 / height.max(1) as f32)
                            * (uv_map.height() - 1) as f32)
                            .round() as u32;
                        let uv_pixel = uv_map.get_pixel(uv_x, uv_y);
                        (
                            uv_pixel.0[0] as f32 / 255.0,
                            uv_pixel.0[1] as f32 / 255.0,
                            (uv_pixel.0[2] as f32 / 255.0).max(0.25),
                        )
                    } else {
                        (
                            x.saturating_sub(panel_bbox.0) as f32 / panel_width,
                            y.saturating_sub(panel_bbox.1) as f32 / panel_height,
                            0.25,
                        )
                    }
                }
            };

            let sx = ((u * sw as f32) / tile_scale + pattern_offset.0 as f32).rem_euclid(sw as f32);
            let sy = ((v * sh as f32) / tile_scale + pattern_offset.1 as f32).rem_euclid(sh as f32);
            let pixel = sample_bilinear(&source, sx, sy);
            output.put_pixel(x, y, pixel);

            let heat = (stretch_factor * 255.0).clamp(0.0, 255.0) as u8;
            stretch.put_pixel(
                x,
                y,
                image::Rgba([heat, 0, 255u8.saturating_sub(heat), 180]),
            );
        }
    }

    (
        DynamicImage::ImageRgba8(output),
        DynamicImage::ImageRgba8(stretch),
    )
}

pub fn mask_bbox(mask: &GrayImage) -> Option<(u32, u32, u32, u32)> {
    bounding_box(mask)
}
