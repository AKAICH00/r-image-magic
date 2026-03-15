use image::{DynamicImage, GrayImage, Rgba, RgbaImage};
use reqwest::Client;
use thiserror::Error;

use crate::engine::compositor::{MockupRequest, MockupResult};
use crate::engine::template::Template;
use crate::imageops::{apply_mask, composite_over, encode_png, encode_png_data_url, subtract_mask};

use super::artwork::normalize_artwork;
use super::collar::render_collar_interior;
use super::config::{AopRenderConfig, CollarInteriorMode, PrintMode, RenderIntent};
use super::debug::{build_mask_overlay, build_outline_overlay, DebugOverlayArtifact};
use super::displacement::apply_panel_displacement;
use super::lighting::apply_fabric_lighting;
use super::panels::resolve_panels;
use super::seams::seam_shadow_mask;
use super::uv::{mask_bbox, project_artwork};
use super::validation::{validate_render, ValidationIssue};

#[derive(Debug, Error)]
pub enum AopRenderError {
    #[error("AOP configuration error: {0}")]
    Configuration(String),
    #[error("AOP image error: {0}")]
    Image(#[from] image::ImageError),
    #[error("AOP http error: {0}")]
    Http(#[from] reqwest::Error),
}

pub struct AopRenderer;

impl AopRenderer {
    pub fn new() -> Self {
        Self
    }

    pub async fn render(
        &self,
        client: &Client,
        request: &MockupRequest,
        template: &Template,
        primary_artwork: &DynamicImage,
    ) -> Result<MockupResult, AopRenderError> {
        let config = request.aop.clone().unwrap_or_default();
        let garment = resolve_panels(template, &config.seam_policy)?;
        let garment_bbox = mask_bbox(&garment.garment_mask).ok_or_else(|| {
            AopRenderError::Configuration("Resolved garment has no mask bounds".to_string())
        })?;
        let primary_artwork =
            normalize_artwork(primary_artwork, config.white_mode, config.transparency_mode);

        let mut panel_renders = Vec::new();
        let mut stretch_overlays = Vec::new();
        let tile_scale = config.tile_scale.unwrap_or(1.0);
        let pattern_offset = (config.pattern_offset_x, config.pattern_offset_y);

        for panel in &garment.panels {
            if matches!(panel.id, super::garment::PanelId::CollarInnerVisible) {
                continue;
            }

            let panel_bbox = mask_bbox(&panel.mask).unwrap_or(garment_bbox);
            let (projected, stretch) = project_artwork(
                &primary_artwork,
                &panel.bleed_mask,
                panel.uv_map.as_ref(),
                garment_bbox,
                panel_bbox,
                panel.uv_mode,
                tile_scale,
                pattern_offset,
            );
            let displaced = apply_panel_displacement(
                &projected,
                &panel.bleed_mask,
                panel.displacement.as_ref(),
                request.displacement_strength * config.fabric.displacement_scale as f64,
            );
            let masked = apply_mask(&displaced, &panel.bleed_mask);
            panel_renders.push((panel.clone(), masked));
            stretch_overlays.push(stretch);
        }

        let mut art_canvas = DynamicImage::ImageRgba8(RgbaImage::new(
            template.base_image.width(),
            template.base_image.height(),
        ));
        for (_, render) in &panel_renders {
            art_canvas = composite_over(&art_canvas, render);
        }

        if let Some(seam_mask) = seam_shadow_mask(&garment.seams) {
            art_canvas = apply_seam_shadow(
                &art_canvas,
                &seam_mask,
                config.seam_policy.occlusion_strength,
            );
        }

        let lit = apply_fabric_lighting(&template.base_image, &art_canvas, &config.fabric);
        let front_projection = panel_renders
            .iter()
            .find(|(panel, _)| matches!(panel.id, super::garment::PanelId::FrontBody))
            .map(|(_, image)| image);

        let collar_panel = garment
            .panels
            .iter()
            .find(|panel| matches!(panel.id, super::garment::PanelId::CollarInnerVisible));
        let collar_overlay = match collar_panel {
            Some(panel) => {
                let mut collar_config = config.collar.clone();
                if config.qa_force_red_collar
                    || matches!(config.render_intent, RenderIntent::ProductionQa)
                {
                    collar_config.mode = CollarInteriorMode::DebugColor;
                }
                Some(
                    render_collar_interior(
                        client,
                        &collar_config,
                        &primary_artwork,
                        front_projection,
                        &panel.mask,
                    )
                    .await?,
                )
            }
            None => None,
        };

        let mut final_image = lit;
        if let Some(collar_overlay) = collar_overlay {
            final_image = composite_over(&final_image, &collar_overlay);
        }

        if config.brand_label_enabled {
            if let Some(label_panel) = garment.panels.iter().find(|panel| {
                matches!(panel.id, super::garment::PanelId::OptionalBackNeckLabelZone)
            }) {
                let label = fill_brand_label(&label_panel.mask);
                final_image = composite_over(&final_image, &label);
            }
        }

        let validation_issues = validate_render(
            &config,
            &garment,
            &panel_renders
                .iter()
                .map(|(panel, render)| (panel, render.clone()))
                .collect::<Vec<_>>(),
            &stretch_overlays,
        );
        let qa_overlays =
            build_debug_artifacts(&config, &garment, &stretch_overlays, &validation_issues)?;

        let png_bytes = encode_png(&final_image)?;
        let url = encode_png_data_url(&final_image)?;

        Ok(MockupResult {
            url,
            width: final_image.width(),
            height: final_image.height(),
            bytes: bytes::Bytes::from(png_bytes),
            validation_issues,
            qa_overlays,
            print_mode: request.print_mode,
            architecture_summary: Some(architecture_summary(request.print_mode)),
        })
    }
}

fn apply_seam_shadow(image: &DynamicImage, seam_mask: &GrayImage, strength: f32) -> DynamicImage {
    let rgba = image.to_rgba8();
    let (width, height) = rgba.dimensions();
    let mut out = RgbaImage::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let mask_alpha = seam_mask.get_pixel(x, y).0[0] as f32 / 255.0;
            let shadow = 1.0 - mask_alpha * strength;
            let pixel = rgba.get_pixel(x, y);
            out.put_pixel(
                x,
                y,
                Rgba([
                    (pixel.0[0] as f32 * shadow) as u8,
                    (pixel.0[1] as f32 * shadow) as u8,
                    (pixel.0[2] as f32 * shadow) as u8,
                    pixel.0[3],
                ]),
            );
        }
    }

    DynamicImage::ImageRgba8(out)
}

fn fill_brand_label(mask: &GrayImage) -> DynamicImage {
    let (width, height) = mask.dimensions();
    let mut out = RgbaImage::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let alpha = mask.get_pixel(x, y).0[0];
            if alpha > 0 {
                out.put_pixel(x, y, Rgba([245, 245, 245, alpha]));
            }
        }
    }

    DynamicImage::ImageRgba8(out)
}

fn build_debug_artifacts(
    config: &AopRenderConfig,
    garment: &super::garment::ResolvedGarment,
    stretch_overlays: &[DynamicImage],
    validation_issues: &[ValidationIssue],
) -> Result<Vec<DebugOverlayArtifact>, AopRenderError> {
    if !config.debug.enabled && !matches!(config.render_intent, RenderIntent::ProductionQa) {
        return Ok(Vec::new());
    }

    let mut overlays = Vec::new();
    let mut items = Vec::new();

    if config.debug.panel_boundaries {
        for panel in &garment.panels {
            items.push(build_outline_overlay(
                &format!("panel-{:?}", panel.id),
                &panel.mask,
                [0, 255, 255, 220],
            ));
        }
    }

    if config.debug.bleed_zones {
        for panel in &garment.panels {
            let bleed_only = subtract_mask(&panel.bleed_mask, &panel.mask);
            items.push(build_mask_overlay(
                &format!("bleed-{:?}", panel.id),
                &bleed_only,
                [0, 255, 120, 180],
            ));
        }
    }

    if config.debug.safe_margins {
        for panel in &garment.panels {
            let unsafe_margin = subtract_mask(&panel.mask, &panel.safe_mask);
            items.push(build_mask_overlay(
                &format!("safe-margin-{:?}", panel.id),
                &unsafe_margin,
                [255, 140, 0, 180],
            ));
        }
    }

    if config.debug.collar_inner_region {
        if let Some(panel) = garment
            .panels
            .iter()
            .find(|panel| matches!(panel.id, super::garment::PanelId::CollarInnerVisible))
        {
            items.push(build_mask_overlay(
                "collar-inner-region",
                &panel.mask,
                [255, 0, 0, 200],
            ));
        }
    }

    if config.debug.seam_zones {
        for seam in &garment.seams {
            items.push(build_mask_overlay(
                &format!("seam-{:?}", seam.kind),
                &seam.mask,
                [255, 255, 0, 180],
            ));
        }
    }

    if config.debug.stretch_heatmap {
        for (index, overlay) in stretch_overlays.iter().enumerate() {
            items.push((format!("stretch-heatmap-{index}"), overlay.clone()));
        }
    }

    if config.debug.transparency_warning_map && !validation_issues.is_empty() {
        items.push(build_mask_overlay(
            "transparency-warning-map",
            &garment.garment_mask,
            [255, 80, 80, 90],
        ));
    }

    for (name, image) in items {
        overlays.push(DebugOverlayArtifact {
            name,
            url: encode_png_data_url(&image)?,
        });
    }

    Ok(overlays)
}

fn architecture_summary(mode: PrintMode) -> String {
    format!(
        "{mode:?} uses seam-aware panel resolution, per-panel UV projection, displacement, seam shadowing, collar interior compositing, validation, and optional QA overlays."
    )
}
