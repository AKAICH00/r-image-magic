use image::{DynamicImage, GrayImage};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::imageops::{bounding_box, subtract_mask};

use super::config::{AopRenderConfig, TransparencyMode};
use super::garment::{GarmentPanel, PanelId, ResolvedGarment, SeamKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ValidationIssue {
    pub severity: Severity,
    pub code: String,
    pub message: String,
    pub panel: Option<String>,
}

pub fn validate_render(
    config: &AopRenderConfig,
    garment: &ResolvedGarment,
    panel_renders: &[(&GarmentPanel, DynamicImage)],
    stretch_overlays: &[DynamicImage],
) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    if !garment
        .panels
        .iter()
        .any(|panel| matches!(panel.id, PanelId::CollarInnerVisible))
    {
        issues.push(issue(
            Severity::Error,
            "unmapped_collar_interior",
            "CollarInnerVisible is missing from the resolved garment.",
            None,
        ));
    }

    for (panel, image) in panel_renders {
        if matches!(
            panel.id,
            PanelId::LeftSleeve | PanelId::RightSleeve | PanelId::FrontBody | PanelId::BackBody
        ) {
            let unsafe_pixels = unsafe_margin_hits(panel, image);
            if unsafe_pixels > 0 {
                let code = match panel.id {
                    PanelId::LeftSleeve | PanelId::RightSleeve => {
                        "artwork_too_close_to_sleeve_seam"
                    }
                    _ => "artwork_too_close_to_collar_seam",
                };
                issues.push(issue(
                    Severity::Warning,
                    code,
                    "Opaque artwork extends into the configured seam-safe margin.",
                    Some(format!("{:?}", panel.id)),
                ));
            }
        }

        if matches!(
            config.transparency_mode,
            TransparencyMode::TransparentMeansNoInk
        ) && unexpected_transparent_gaps(&panel.mask, image)
        {
            issues.push(issue(
                Severity::Warning,
                "unexpected_transparent_gaps",
                "The panel contains interior transparent gaps that will render as unprinted fabric.",
                Some(format!("{:?}", panel.id)),
            ));
        }
    }

    issues.extend(high_distortion_near_seams(garment, stretch_overlays));

    let mismatch_count = sleeve_continuation_issues(garment, panel_renders, &mut issues);
    if mismatch_count > 0 {
        issues.push(issue(
            Severity::Warning,
            "visible_discontinuity_across_shoulders",
            "Panel colors diverge sharply across at least one body-to-sleeve seam.",
            None,
        ));
    }

    issues
}

fn unsafe_margin_hits(panel: &GarmentPanel, image: &DynamicImage) -> usize {
    let art_mask = image.to_rgba8();
    let unsafe_mask = subtract_mask(&panel.mask, &panel.safe_mask);
    let (width, height) = unsafe_mask.dimensions();
    let mut hits = 0usize;

    for y in 0..height {
        for x in 0..width {
            if unsafe_mask.get_pixel(x, y).0[0] > 0 && art_mask.get_pixel(x, y).0[3] > 16 {
                hits += 1;
            }
        }
    }

    hits
}

fn unexpected_transparent_gaps(panel_mask: &GrayImage, image: &DynamicImage) -> bool {
    let art = image.to_rgba8();
    if let Some((left, top, right, bottom)) = bounding_box(panel_mask) {
        for y in top..bottom {
            for x in left..right {
                if panel_mask.get_pixel(x, y).0[0] > 200 && art.get_pixel(x, y).0[3] == 0 {
                    return true;
                }
            }
        }
    }

    false
}

fn high_distortion_near_seams(
    garment: &ResolvedGarment,
    stretch_overlays: &[DynamicImage],
) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();
    if stretch_overlays.is_empty() {
        return issues;
    }

    for seam in &garment.seams {
        let average = average_stretch_in_mask(stretch_overlays, &seam.mask);
        if average < 96.0 {
            continue;
        }

        let (code, message) = match seam.kind {
            SeamKind::CollarOpening => (
                "high_distortion_near_collar",
                "Stretch is high near the collar opening; artwork continuity may break on the sewn edge.",
            ),
            SeamKind::LeftUnderarm | SeamKind::RightUnderarm => (
                "high_distortion_near_underarm",
                "Stretch is high near the underarm seam; panel mapping may over-compress the artwork.",
            ),
            _ => continue,
        };

        issues.push(issue(
            Severity::Warning,
            code,
            message,
            Some(format!("{:?}", seam.kind)),
        ));
    }

    issues
}

fn sleeve_continuation_issues(
    garment: &ResolvedGarment,
    panel_renders: &[(&GarmentPanel, DynamicImage)],
    issues: &mut Vec<ValidationIssue>,
) -> usize {
    let mut mismatches = 0usize;

    for (seam_kind, body_id, sleeve_id) in [
        (
            SeamKind::LeftShoulder,
            PanelId::FrontBody,
            PanelId::LeftSleeve,
        ),
        (
            SeamKind::RightShoulder,
            PanelId::FrontBody,
            PanelId::RightSleeve,
        ),
        (
            SeamKind::LeftUnderarm,
            PanelId::BackBody,
            PanelId::LeftSleeve,
        ),
        (
            SeamKind::RightUnderarm,
            PanelId::BackBody,
            PanelId::RightSleeve,
        ),
    ] {
        let Some(seam) = garment.seams.iter().find(|seam| seam.kind == seam_kind) else {
            continue;
        };
        let Some((_, body_image)) = panel_renders.iter().find(|(panel, _)| panel.id == body_id)
        else {
            continue;
        };
        let Some((_, sleeve_image)) = panel_renders
            .iter()
            .find(|(panel, _)| panel.id == sleeve_id)
        else {
            continue;
        };

        let Some(body_color) = average_rgb_in_mask(body_image, &seam.mask) else {
            continue;
        };
        let Some(sleeve_color) = average_rgb_in_mask(sleeve_image, &seam.mask) else {
            continue;
        };

        if color_distance(body_color, sleeve_color) > 52.0 {
            mismatches += 1;
            issues.push(issue(
                Severity::Warning,
                "sleeve_continuation_mismatch",
                "Artwork color continuity breaks across a body-to-sleeve seam.",
                Some(format!("{:?}", seam.kind)),
            ));
        }
    }

    mismatches
}

fn average_stretch_in_mask(stretch_overlays: &[DynamicImage], mask: &GrayImage) -> f32 {
    let (width, height) = mask.dimensions();
    let mut total = 0.0f32;
    let mut count = 0u32;

    for overlay in stretch_overlays {
        let rgba = overlay.to_rgba8();
        for y in 0..height {
            for x in 0..width {
                if mask.get_pixel(x, y).0[0] == 0 {
                    continue;
                }
                let pixel = rgba.get_pixel(x, y);
                if pixel.0[3] == 0 {
                    continue;
                }
                total += pixel.0[0] as f32;
                count += 1;
            }
        }
    }

    if count == 0 {
        0.0
    } else {
        total / count as f32
    }
}

fn average_rgb_in_mask(image: &DynamicImage, mask: &GrayImage) -> Option<[f32; 3]> {
    let rgba = image.to_rgba8();
    let (width, height) = mask.dimensions();
    let mut total = [0.0f32; 3];
    let mut count = 0u32;

    for y in 0..height {
        for x in 0..width {
            if mask.get_pixel(x, y).0[0] == 0 {
                continue;
            }
            let pixel = rgba.get_pixel(x, y);
            if pixel.0[3] == 0 {
                continue;
            }
            total[0] += pixel.0[0] as f32;
            total[1] += pixel.0[1] as f32;
            total[2] += pixel.0[2] as f32;
            count += 1;
        }
    }

    (count > 0).then_some([
        total[0] / count as f32,
        total[1] / count as f32,
        total[2] / count as f32,
    ])
}

fn color_distance(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dr = a[0] - b[0];
    let dg = a[1] - b[1];
    let db = a[2] - b[2];
    (dr * dr + dg * dg + db * db).sqrt()
}

fn issue(severity: Severity, code: &str, message: &str, panel: Option<String>) -> ValidationIssue {
    ValidationIssue {
        severity,
        code: code.to_string(),
        message: message.to_string(),
        panel,
    }
}

#[cfg(test)]
mod tests {
    use image::{DynamicImage, GrayImage, Luma, Rgba, RgbaImage};

    use crate::aop::config::AopRenderConfig;
    use crate::aop::garment::{GarmentPanel, ResolvedGarment};

    use super::*;

    #[test]
    fn test_validation_flags_missing_collar() {
        let mask = GrayImage::from_pixel(16, 16, Luma([255]));
        let panel = GarmentPanel {
            id: PanelId::FrontBody,
            mask: mask.clone(),
            bleed_mask: mask.clone(),
            safe_mask: mask.clone(),
            displacement: None,
            uv_map: None,
            uv_mode: crate::aop::uv::UvMode::PlanarFront,
        };
        let garment = ResolvedGarment {
            panels: vec![panel.clone()],
            seams: Vec::new(),
            garment_mask: mask,
        };
        let image =
            DynamicImage::ImageRgba8(RgbaImage::from_pixel(16, 16, Rgba([255, 255, 255, 255])));
        let issues = validate_render(
            &AopRenderConfig::default(),
            &garment,
            &[(&panel, image)],
            &[],
        );
        assert!(issues
            .iter()
            .any(|issue| issue.code == "unmapped_collar_interior"));
    }
}
