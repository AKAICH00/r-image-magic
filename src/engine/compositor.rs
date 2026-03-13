//! Image compositing pipeline
//!
//! Combines design images with t-shirt templates using displacement mapping
//! and blend modes for photorealistic mockups.

use base64::Engine;
use bytes::Bytes;
use image::{DynamicImage, GenericImageView, Rgba, RgbaImage};
use std::sync::Arc;
use thiserror::Error;
use tracing::{debug, info};

use super::displacement::{apply_displacement, apply_opacity};
use super::template::Template;
use crate::config::service_user_agent;
use crate::domain::PlacementSpec;

/// Compositing errors
#[derive(Debug, Error)]
pub enum CompositorError {
    #[error("Failed to fetch design image: {0}")]
    FetchFailed(String),
    #[error("Failed to decode image: {0}")]
    DecodeFailed(#[from] image::ImageError),
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),
}

/// Request for mockup generation
#[derive(Debug, Clone)]
pub struct MockupRequest {
    pub design_url: String,
    pub template_id: String,
    pub placement: PlacementSpec,
    pub displacement_strength: f64,
    pub tint_color: Option<String>,
}

/// Result of mockup generation
pub struct MockupResult {
    pub url: String,
    pub width: u32,
    pub height: u32,
    pub bytes: Bytes,
}

/// Parse a hex color string (with or without leading '#') into (r, g, b)
fn parse_hex_color(hex: &str) -> Option<(u8, u8, u8)> {
    let hex = hex.strip_prefix('#').unwrap_or(hex);
    if hex.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some((r, g, b))
}

/// Image compositor for generating mockups
pub struct Compositor {
    http_client: reqwest::Client,
}

impl Compositor {
    /// Create a new compositor
    pub fn new() -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .user_agent(service_user_agent())
            .build()
            .expect("Failed to create HTTP client");

        Compositor { http_client }
    }

    /// Generate a mockup from a request and template
    pub async fn generate(
        &self,
        request: &MockupRequest,
        template: &Arc<Template>,
    ) -> Result<MockupResult, CompositorError> {
        debug!(
            design_url = %request.design_url,
            template_id = %request.template_id,
            displacement = request.displacement_strength,
            "Starting mockup generation"
        );

        // 1. Fetch design image
        let design = self.fetch_design(&request.design_url).await?;

        // NOTE: White background removal is intentionally skipped for seamless/AOP patterns.
        // Patterns fill the entire print area — removing white would punch holes in the design.
        // Re-enable only for logo/artwork-on-white-bg use cases.
        // let design = self.remove_white_background(&design);

        // 2. Resize design according to placement
        let (design_width, design_height) = request.placement.get_design_dimensions();
        let resized_design = design.resize_exact(
            design_width as u32,
            design_height as u32,
            image::imageops::FilterType::Lanczos3,
        );

        // 4. Composite position (needed before displacement crop)
        let (rel_x, rel_y) = request.placement.get_absolute_position();
        let abs_x = rel_x + template.metadata.print_area.x as i32;
        let abs_y = rel_y + template.metadata.print_area.y as i32;

        // 3. Apply displacement mapping if available
        // Crop the displacement map to the exact region where the design lands,
        // so fabric wrinkles match the actual print position (not the whole shirt scaled down).
        let processed_design = if let Some(ref disp_map) = template.displacement_map {
            if template.metadata.displacement.enabled {
                let (disp_w, disp_h) = disp_map.dimensions();
                let crop_x = (abs_x.max(0) as u32).min(disp_w.saturating_sub(1));
                let crop_y = (abs_y.max(0) as u32).min(disp_h.saturating_sub(1));
                let crop_w = (design_width as u32).min(disp_w.saturating_sub(crop_x));
                let crop_h = (design_height as u32).min(disp_h.saturating_sub(crop_y));
                let disp_crop = disp_map.crop_imm(crop_x, crop_y, crop_w, crop_h);
                apply_displacement(&resized_design, &disp_crop, request.displacement_strength)
            } else {
                resized_design
            }
        } else {
            resized_design
        };

        debug!(
            rel_x = rel_x,
            rel_y = rel_y,
            print_area_x = template.metadata.print_area.x,
            print_area_y = template.metadata.print_area.y,
            abs_x = abs_x,
            abs_y = abs_y,
            "Calculated design position"
        );

        // Apply product color tinting if requested (skip for white / no tint)
        let tinted_base;
        let base_ref = match request.tint_color.as_deref().and_then(|hex| {
            let parsed = parse_hex_color(hex)?;
            // Skip tinting for white — no visible change
            if parsed == (255, 255, 255) {
                None
            } else {
                Some(parsed)
            }
        }) {
            Some((r, g, b)) => {
                debug!(r, g, b, "Applying product tint");
                tinted_base = Self::tint_template(&template.base_image, r, g, b);
                &tinted_base
            }
            None => &template.base_image,
        };

        let composited = self.composite_design(
            base_ref,
            &processed_design,
            abs_x,
            abs_y,
            template.metadata.default_opacity,
            &template.metadata.blend_mode,
        );

        // 5. Encode to PNG (preserves RGBA transparency)
        let (width, height) = composited.dimensions();
        let png_bytes = self.encode_png(&composited)?;

        info!(
            width = width,
            height = height,
            bytes = png_bytes.len(),
            "Mockup generation complete (PNG with transparency)"
        );

        // For now, return the bytes directly (Cloudinary upload can be added later)
        Ok(MockupResult {
            url: format!(
                "data:image/png;base64,{}",
                base64::engine::general_purpose::STANDARD.encode(&png_bytes)
            ),
            width,
            height,
            bytes: Bytes::from(png_bytes),
        })
    }

    /// Fetch design image from URL
    async fn fetch_design(&self, url: &str) -> Result<DynamicImage, CompositorError> {
        debug!(url = %url, "Fetching design image");

        let response = self.http_client.get(url).send().await?;

        if !response.status().is_success() {
            return Err(CompositorError::FetchFailed(format!(
                "HTTP {}: {}",
                response.status(),
                url
            )));
        }

        let bytes = response.bytes().await?;
        let image = image::load_from_memory(&bytes)?;

        debug!(
            width = image.width(),
            height = image.height(),
            "Design image loaded"
        );

        Ok(image)
    }

    /// Composite design onto base template
    fn composite_design(
        &self,
        base: &DynamicImage,
        design: &DynamicImage,
        x_offset: i32,
        y_offset: i32,
        opacity: u8,
        blend_mode: &str,
    ) -> DynamicImage {
        let mut base_rgba = base.to_rgba8();
        let design_rgba = design.to_rgba8();
        let (base_width, base_height) = base_rgba.dimensions();
        let (design_width, design_height) = design_rgba.dimensions();

        // Apply opacity to design
        let design_with_opacity = apply_opacity(&design_rgba, opacity);

        // Composite based on blend mode
        for dy in 0..design_height {
            let y = y_offset + dy as i32;
            if y < 0 || y >= base_height as i32 {
                continue;
            }

            for dx in 0..design_width {
                let x = x_offset + dx as i32;
                if x < 0 || x >= base_width as i32 {
                    continue;
                }

                let design_pixel = design_with_opacity.get_pixel(dx, dy);
                let base_pixel = base_rgba.get_pixel(x as u32, y as u32);

                // Skip fully transparent pixels
                if design_pixel.0[3] == 0 {
                    continue;
                }

                let blended = match blend_mode {
                    "multiply" => self.blend_multiply_pixel(base_pixel, design_pixel),
                    "screen" => self.blend_screen_pixel(base_pixel, design_pixel),
                    "overlay" => self.blend_overlay_pixel(base_pixel, design_pixel),
                    _ => self.blend_normal_pixel(base_pixel, design_pixel),
                };

                base_rgba.put_pixel(x as u32, y as u32, blended);
            }
        }

        DynamicImage::ImageRgba8(base_rgba)
    }

    /// Normal alpha blending
    fn blend_normal_pixel(&self, base: &Rgba<u8>, overlay: &Rgba<u8>) -> Rgba<u8> {
        let alpha = overlay.0[3] as f64 / 255.0;
        let inv_alpha = 1.0 - alpha;

        Rgba([
            ((overlay.0[0] as f64 * alpha + base.0[0] as f64 * inv_alpha) as u8),
            ((overlay.0[1] as f64 * alpha + base.0[1] as f64 * inv_alpha) as u8),
            ((overlay.0[2] as f64 * alpha + base.0[2] as f64 * inv_alpha) as u8),
            255,
        ])
    }

    /// Multiply blend mode
    fn blend_multiply_pixel(&self, base: &Rgba<u8>, overlay: &Rgba<u8>) -> Rgba<u8> {
        let alpha = overlay.0[3] as f64 / 255.0;

        let mut result = [0u8; 4];
        for i in 0..3 {
            let multiplied = (base.0[i] as u32 * overlay.0[i] as u32) / 255;
            result[i] = (multiplied as f64 * alpha + base.0[i] as f64 * (1.0 - alpha)) as u8;
        }
        result[3] = 255;

        Rgba(result)
    }

    /// Screen blend mode
    fn blend_screen_pixel(&self, base: &Rgba<u8>, overlay: &Rgba<u8>) -> Rgba<u8> {
        let alpha = overlay.0[3] as f64 / 255.0;

        let mut result = [0u8; 4];
        for i in 0..3 {
            let screened = 255 - ((255 - base.0[i] as u32) * (255 - overlay.0[i] as u32)) / 255;
            result[i] = (screened as f64 * alpha + base.0[i] as f64 * (1.0 - alpha)) as u8;
        }
        result[3] = 255;

        Rgba(result)
    }

    /// Overlay blend mode
    fn blend_overlay_pixel(&self, base: &Rgba<u8>, overlay: &Rgba<u8>) -> Rgba<u8> {
        let alpha = overlay.0[3] as f64 / 255.0;

        let mut result = [0u8; 4];
        for i in 0..3 {
            let b = base.0[i] as f64 / 255.0;
            let o = overlay.0[i] as f64 / 255.0;

            let overlayed = if b < 0.5 {
                2.0 * b * o
            } else {
                1.0 - 2.0 * (1.0 - b) * (1.0 - o)
            };

            let blended = overlayed * alpha + b * (1.0 - alpha);
            result[i] = (blended * 255.0).clamp(0.0, 255.0) as u8;
        }
        result[3] = 255;

        Rgba(result)
    }

    /// Apply a multiply-blend tint to a white-base template image.
    /// White pixels become the tint color; darker fabric texture pixels become proportionally darker.
    fn tint_template(base: &DynamicImage, r: u8, g: u8, b: u8) -> DynamicImage {
        let rgba = base.to_rgba8();
        let (width, height) = rgba.dimensions();
        let mut output = RgbaImage::new(width, height);

        for y in 0..height {
            for x in 0..width {
                let pixel = rgba.get_pixel(x, y);
                if pixel.0[3] == 0 {
                    output.put_pixel(x, y, *pixel);
                    continue;
                }
                let tinted_r = (pixel.0[0] as u16 * r as u16 / 255) as u8;
                let tinted_g = (pixel.0[1] as u16 * g as u16 / 255) as u8;
                let tinted_b = (pixel.0[2] as u16 * b as u16 / 255) as u8;
                output.put_pixel(x, y, Rgba([tinted_r, tinted_g, tinted_b, pixel.0[3]]));
            }
        }

        DynamicImage::ImageRgba8(output)
    }

    /// Encode image to PNG bytes (preserves RGBA transparency)
    fn encode_png(&self, image: &DynamicImage) -> Result<Vec<u8>, CompositorError> {
        let mut buffer = Vec::new();
        let encoder = image::codecs::png::PngEncoder::new(&mut buffer);
        encoder.encode(
            image.as_bytes(),
            image.width(),
            image.height(),
            image.color().into(),
        )?;
        Ok(buffer)
    }

    /// Remove white/near-white background from an image by converting to transparency
    /// Uses edge-aware algorithm to preserve design details while removing backgrounds
    fn remove_white_background(&self, image: &DynamicImage) -> DynamicImage {
        let rgba = image.to_rgba8();
        let (width, height) = rgba.dimensions();
        let mut output = RgbaImage::new(width, height);

        // Thresholds for "white-ish" detection
        // Lower value = more aggressive removal (catches more off-white)
        const WHITE_THRESHOLD: u8 = 245; // Pure white detection
        const LIGHT_THRESHOLD: u8 = 230; // Light color detection
        const EDGE_FEATHER: u8 = 25; // Feather range for smooth edges

        for y in 0..height {
            for x in 0..width {
                let pixel = rgba.get_pixel(x, y);
                let r = pixel.0[0];
                let g = pixel.0[1];
                let b = pixel.0[2];

                // Calculate luminance (human eye weighted)
                let luminance = (0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32) as u8;

                // Check color variance (white has low variance between channels)
                let max_channel = r.max(g).max(b);
                let min_channel = r.min(g).min(b);
                let variance = max_channel - min_channel;

                // Detect white/near-white: high luminance + low color variance
                if luminance >= WHITE_THRESHOLD && variance <= 15 {
                    // Pure white - fully transparent
                    output.put_pixel(x, y, Rgba([r, g, b, 0]));
                } else if luminance >= LIGHT_THRESHOLD && variance <= 25 {
                    // Light gray/off-white - gradual transparency based on how white
                    let alpha = ((255 - luminance) as f32 / (255 - LIGHT_THRESHOLD) as f32 * 255.0)
                        .min(255.0) as u8;
                    output.put_pixel(x, y, Rgba([r, g, b, alpha]));
                } else if luminance >= LIGHT_THRESHOLD - EDGE_FEATHER && variance <= 35 {
                    // Edge feathering zone
                    let alpha = ((LIGHT_THRESHOLD - luminance.saturating_sub(EDGE_FEATHER)) as f32
                        / EDGE_FEATHER as f32
                        * 255.0)
                        .min(255.0) as u8;
                    output.put_pixel(x, y, Rgba([r, g, b, alpha]));
                } else {
                    // Keep pixel fully opaque
                    output.put_pixel(x, y, Rgba([r, g, b, 255]));
                }
            }
        }

        debug!(
            width = width,
            height = height,
            "Removed white background from design"
        );

        DynamicImage::ImageRgba8(output)
    }
}

impl Default for Compositor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hex_color() {
        assert_eq!(parse_hex_color("0D0D0D"), Some((13, 13, 13)));
        assert_eq!(parse_hex_color("#FF0000"), Some((255, 0, 0)));
        assert_eq!(parse_hex_color("FFFFFF"), Some((255, 255, 255)));
        assert_eq!(parse_hex_color("abc"), None);
        assert_eq!(parse_hex_color("ZZZZZZ"), None);
        assert_eq!(parse_hex_color(""), None);
    }

    #[test]
    fn test_tint_white_pixel() {
        let img = DynamicImage::ImageRgba8(RgbaImage::from_pixel(1, 1, Rgba([255, 255, 255, 255])));
        let tinted = Compositor::tint_template(&img, 13, 13, 13);
        let pixel = tinted.to_rgba8().get_pixel(0, 0).0;
        assert_eq!(pixel, [13, 13, 13, 255]);
    }

    #[test]
    fn test_tint_preserves_transparency() {
        let img = DynamicImage::ImageRgba8(RgbaImage::from_pixel(1, 1, Rgba([255, 255, 255, 0])));
        let tinted = Compositor::tint_template(&img, 13, 13, 13);
        let pixel = tinted.to_rgba8().get_pixel(0, 0).0;
        assert_eq!(pixel[3], 0);
    }

    #[test]
    fn test_tint_gray_pixel() {
        let img = DynamicImage::ImageRgba8(RgbaImage::from_pixel(1, 1, Rgba([128, 128, 128, 255])));
        let tinted = Compositor::tint_template(&img, 255, 0, 0);
        let pixel = tinted.to_rgba8().get_pixel(0, 0).0;
        // 128 * 255 / 255 = 128, 128 * 0 / 255 = 0
        assert_eq!(pixel, [128, 0, 0, 255]);
    }
}
