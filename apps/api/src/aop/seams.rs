use image::{GrayImage, Luma};

use crate::imageops::{dilate_mask, intersect_masks};

use super::config::SeamPolicy;
use super::garment::{GarmentPanel, GarmentSeam, PanelId, SeamKind};

pub fn build_seams(panels: &[GarmentPanel], policy: &SeamPolicy) -> Vec<GarmentSeam> {
    let mut seams = Vec::new();

    for (left, right, kind) in [
        (
            PanelId::FrontBody,
            PanelId::LeftSleeve,
            SeamKind::LeftShoulder,
        ),
        (
            PanelId::FrontBody,
            PanelId::RightSleeve,
            SeamKind::RightShoulder,
        ),
        (
            PanelId::BackBody,
            PanelId::LeftSleeve,
            SeamKind::LeftUnderarm,
        ),
        (
            PanelId::BackBody,
            PanelId::RightSleeve,
            SeamKind::RightUnderarm,
        ),
    ] {
        let lhs = panels.iter().find(|panel| panel.id == left);
        let rhs = panels.iter().find(|panel| panel.id == right);
        if let (Some(lhs), Some(rhs)) = (lhs, rhs) {
            let seam_mask = intersect_masks(
                &dilate_mask(&lhs.mask, policy.blend_px),
                &dilate_mask(&rhs.mask, policy.blend_px),
            );
            if seam_mask.pixels().any(|pixel| pixel.0[0] > 0) {
                seams.push(GarmentSeam {
                    kind,
                    mask: seam_mask,
                });
            }
        }
    }

    for body_id in [PanelId::FrontBody, PanelId::BackBody] {
        let body = panels.iter().find(|panel| panel.id == body_id);
        let collar = panels.iter().find(|panel| panel.id == PanelId::CollarOuter);
        if let (Some(body), Some(collar)) = (body, collar) {
            let seam_mask = intersect_masks(
                &dilate_mask(&body.mask, policy.blend_px),
                &dilate_mask(&collar.mask, policy.blend_px),
            );
            if seam_mask.pixels().any(|pixel| pixel.0[0] > 0) {
                seams.push(GarmentSeam {
                    kind: SeamKind::CollarOpening,
                    mask: seam_mask,
                });
            }
        }
    }

    let label = panels
        .iter()
        .find(|panel| panel.id == PanelId::OptionalBackNeckLabelZone);
    let collar_inner = panels
        .iter()
        .find(|panel| panel.id == PanelId::CollarInnerVisible);
    if let (Some(label), Some(collar_inner)) = (label, collar_inner) {
        let seam_mask = intersect_masks(
            &dilate_mask(&label.mask, policy.blend_px),
            &dilate_mask(&collar_inner.mask, policy.blend_px),
        );
        if seam_mask.pixels().any(|pixel| pixel.0[0] > 0) {
            seams.push(GarmentSeam {
                kind: SeamKind::BackNeckLabel,
                mask: seam_mask,
            });
        }
    }

    seams
}

pub fn seam_bleed_mask(
    panel_mask: &GrayImage,
    garment_mask: &GrayImage,
    policy: &SeamPolicy,
) -> GrayImage {
    intersect_masks(&dilate_mask(panel_mask, policy.bleed_px), garment_mask)
}

pub fn seam_shadow_mask(seams: &[GarmentSeam]) -> Option<GrayImage> {
    let first = seams.first()?;
    let (width, height) = first.mask.dimensions();
    let mut out = GrayImage::new(width, height);

    for seam in seams {
        for y in 0..height {
            for x in 0..width {
                let value = out.get_pixel(x, y).0[0].max(seam.mask.get_pixel(x, y).0[0]);
                out.put_pixel(x, y, Luma([value]));
            }
        }
    }

    Some(out)
}
