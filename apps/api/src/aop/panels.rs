use std::path::Path;

use image::{DynamicImage, GrayImage, Luma};

use crate::engine::template::Template;
use crate::imageops::{
    alpha_mask, bounding_box, dilate_mask, erode_mask, intersect_masks, subtract_mask, union_masks,
};

use super::config::SeamPolicy;
use super::garment::{GarmentPanel, PanelAssetConfig, PanelId, ResolvedGarment};
use super::render::AopRenderError;
use super::seams::{build_seams, seam_bleed_mask};
use super::uv::{mask_bbox, UvMode};

pub fn resolve_panels(
    template: &Template,
    seam_policy: &SeamPolicy,
) -> Result<ResolvedGarment, AopRenderError> {
    let garment_mask = template
        .print_mask
        .as_ref()
        .map(|mask| mask.to_luma8())
        .unwrap_or_else(|| alpha_mask(&template.base_image));

    let panels = if let Some(aop) = template.metadata.aop.as_ref() {
        if !aop.panels.is_empty() {
            let mut explicit =
                resolve_explicit_panels(template, &garment_mask, seam_policy, &aop.panels)?;
            if aop.derive_from_print_mask {
                for panel in derive_panels_from_masks(template, &garment_mask, seam_policy)? {
                    if !explicit.iter().any(|existing| existing.id == panel.id) {
                        explicit.push(panel);
                    }
                }
            }
            explicit
        } else {
            derive_panels_from_masks(template, &garment_mask, seam_policy)?
        }
    } else {
        derive_panels_from_masks(template, &garment_mask, seam_policy)?
    };

    let seams = build_seams(&panels, seam_policy);

    Ok(ResolvedGarment {
        garment_mask,
        panels,
        seams,
    })
}

fn derive_panels_from_masks(
    template: &Template,
    garment_mask: &GrayImage,
    seam_policy: &SeamPolicy,
) -> Result<Vec<GarmentPanel>, AopRenderError> {
    let (left, top, right, bottom) = bounding_box(garment_mask).ok_or_else(|| {
        AopRenderError::Configuration("Template has no printable garment mask".to_string())
    })?;
    let width = right - left;
    let height = bottom - top;

    let torso_left = left + ((width as f32) * 0.18) as u32;
    let torso_right = right.saturating_sub(((width as f32) * 0.18) as u32);
    let torso_top = top + ((height as f32) * 0.16) as u32;
    let sleeve_bottom = top + ((height as f32) * 0.34) as u32;

    let front_or_back = rect_mask_like(garment_mask, torso_left, torso_top, torso_right, bottom);
    let left_sleeve = rect_mask_like(garment_mask, left, top, torso_left, sleeve_bottom);
    let right_sleeve = rect_mask_like(garment_mask, torso_right, top, right, sleeve_bottom);

    let collar_outer = template
        .preserve_masks
        .first()
        .map(DynamicImage::to_luma8)
        .unwrap_or_else(|| GrayImage::new(garment_mask.width(), garment_mask.height()));
    let collar_inner = template
        .preserve_masks
        .get(1)
        .map(DynamicImage::to_luma8)
        .unwrap_or_else(|| GrayImage::new(garment_mask.width(), garment_mask.height()));
    let back_label = derive_back_label(&collar_inner);

    let body_mask = subtract_mask(
        &front_or_back,
        &union_masks(&[
            left_sleeve.clone(),
            right_sleeve.clone(),
            collar_outer.clone(),
            collar_inner.clone(),
        ])
        .unwrap_or_else(|| GrayImage::new(garment_mask.width(), garment_mask.height())),
    );

    let panel_displacement = template.displacement_map.as_ref();
    let mut panels = vec![
        build_panel(
            panel_id_for_template(template, PanelId::FrontBody, PanelId::BackBody),
            body_mask,
            garment_mask,
            panel_displacement,
            seam_policy,
            UvMode::DerivedProjection,
        ),
        build_panel(
            PanelId::LeftSleeve,
            left_sleeve,
            garment_mask,
            panel_displacement,
            seam_policy,
            UvMode::SleeveCylindrical,
        ),
        build_panel(
            PanelId::RightSleeve,
            right_sleeve,
            garment_mask,
            panel_displacement,
            seam_policy,
            UvMode::SleeveCylindrical,
        ),
        build_panel(
            PanelId::CollarOuter,
            collar_outer,
            garment_mask,
            panel_displacement,
            seam_policy,
            UvMode::CollarArc,
        ),
        build_panel(
            PanelId::CollarInnerVisible,
            collar_inner,
            garment_mask,
            panel_displacement,
            seam_policy,
            UvMode::CollarArc,
        ),
    ];

    if back_label.pixels().any(|pixel| pixel.0[0] > 0) {
        panels.push(build_panel(
            PanelId::OptionalBackNeckLabelZone,
            back_label,
            garment_mask,
            panel_displacement,
            seam_policy,
            UvMode::PlanarBack,
        ));
    }

    Ok(panels)
}

fn resolve_explicit_panels(
    template: &Template,
    garment_mask: &GrayImage,
    seam_policy: &SeamPolicy,
    configs: &[PanelAssetConfig],
) -> Result<Vec<GarmentPanel>, AopRenderError> {
    let mut panels = Vec::with_capacity(configs.len());

    for config in configs {
        let mask_path = config.mask_path.as_deref().ok_or_else(|| {
            AopRenderError::Configuration(format!("AOP panel {:?} is missing mask_path", config.id))
        })?;
        let mask = load_image(&template.root_path, mask_path)?.to_luma8();
        if mask.dimensions() != garment_mask.dimensions() {
            return Err(AopRenderError::Configuration(format!(
                "AOP panel {:?} mask dimensions do not match garment mask",
                config.id
            )));
        }

        let displacement = match config.displacement_path.as_deref() {
            Some(path) => Some(load_image(&template.root_path, path)?),
            None => template.displacement_map.clone(),
        };
        let uv_map = match config.uv_map_path.as_deref() {
            Some(path) => Some(load_image(&template.root_path, path)?),
            None => None,
        };

        let bleed_px = config.bleed_px.unwrap_or(seam_policy.bleed_px);
        let safe_margin_px = config.safe_margin_px.unwrap_or(seam_policy.safe_margin_px);
        let bleed_mask = intersect_masks(&dilate_mask(&mask, bleed_px), garment_mask);
        let safe_mask = erode_mask(&mask, safe_margin_px);

        panels.push(GarmentPanel {
            id: config.id,
            mask,
            bleed_mask,
            safe_mask,
            displacement,
            uv_map,
            uv_mode: config.uv_mode,
        });
    }

    Ok(panels)
}

fn load_image(root_path: &Path, relative_path: &str) -> Result<DynamicImage, AopRenderError> {
    let image_path = root_path.join(relative_path);
    if !image_path.exists() {
        return Err(AopRenderError::Configuration(format!(
            "AOP asset not found: {}",
            image_path.display()
        )));
    }

    Ok(image::open(image_path)?)
}

fn panel_id_for_template(template: &Template, front: PanelId, back: PanelId) -> PanelId {
    if template.metadata.placement.eq_ignore_ascii_case("back") {
        back
    } else {
        front
    }
}

fn build_panel(
    id: PanelId,
    mask: GrayImage,
    garment_mask: &GrayImage,
    displacement: Option<&DynamicImage>,
    seam_policy: &SeamPolicy,
    uv_mode: UvMode,
) -> GarmentPanel {
    let bleed_mask = seam_bleed_mask(&mask, garment_mask, seam_policy);
    let safe_mask = erode_mask(&mask, seam_policy.safe_margin_px);

    GarmentPanel {
        id,
        bleed_mask,
        safe_mask,
        displacement: displacement.cloned(),
        uv_map: None,
        uv_mode,
        mask,
    }
}

fn rect_mask_like(source: &GrayImage, left: u32, top: u32, right: u32, bottom: u32) -> GrayImage {
    let (width, height) = source.dimensions();
    let mut out = GrayImage::new(width, height);

    for y in top..bottom.min(height) {
        for x in left..right.min(width) {
            if source.get_pixel(x, y).0[0] > 0 {
                out.put_pixel(x, y, Luma([255]));
            }
        }
    }

    intersect_masks(source, &out)
}

fn derive_back_label(collar_inner: &GrayImage) -> GrayImage {
    let (width, height) = collar_inner.dimensions();
    let mut out = GrayImage::new(width, height);

    if let Some((left, _top, right, bottom)) = mask_bbox(collar_inner) {
        let label_top = bottom + 12;
        let label_bottom = (label_top + 80).min(height);
        let center = (left + right) / 2;
        let label_left = center.saturating_sub(120);
        let label_right = (center + 120).min(width);
        for y in label_top..label_bottom {
            for x in label_left..label_right {
                out.put_pixel(x, y, Luma([255]));
            }
        }
    }

    dilate_mask(&out, 1)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use image::{DynamicImage, GrayImage, Luma, Rgba, RgbaImage};

    use crate::engine::template::{
        AnchorPoint, DisplacementConfig, PrintArea, Template, TemplateDimensions, TemplateMetadata,
    };

    use super::*;

    #[test]
    fn test_resolve_panels_derives_collar_and_sleeves() {
        let mut print_mask = GrayImage::new(100, 120);
        for y in 10..110 {
            for x in 20..80 {
                print_mask.put_pixel(x, y, Luma([255]));
            }
        }
        for y in 10..40 {
            for x in 5..20 {
                print_mask.put_pixel(x, y, Luma([255]));
            }
            for x in 80..95 {
                print_mask.put_pixel(x, y, Luma([255]));
            }
        }

        let mut collar_outer = GrayImage::new(100, 120);
        for y in 10..18 {
            for x in 35..65 {
                collar_outer.put_pixel(x, y, Luma([255]));
            }
        }
        let mut collar_inner = GrayImage::new(100, 120);
        for y in 18..26 {
            for x in 42..58 {
                collar_inner.put_pixel(x, y, Luma([255]));
            }
        }

        let template = Template {
            root_path: PathBuf::from("/tmp"),
            metadata: TemplateMetadata {
                id: "aop-test".to_string(),
                version: 1,
                category: "tshirt-aop".to_string(),
                color: "white".to_string(),
                color_hex: None,
                placement: "front".to_string(),
                gender: None,
                dimensions: TemplateDimensions {
                    width: 100,
                    height: 120,
                },
                print_area: PrintArea {
                    x: 0,
                    y: 0,
                    width: 100,
                    height: 120,
                },
                anchor_point: AnchorPoint { x: 50, y: 60 },
                displacement: DisplacementConfig {
                    enabled: true,
                    strength_default: 8.0,
                    strength_range: (4.0, 12.0),
                },
                blend_mode: "multiply".to_string(),
                default_opacity: 255,
                name: None,
                product: None,
                product_type: None,
                printful_product_id: None,
                printful_template_id: None,
                print_mask: None,
                preserve_masks: Vec::new(),
                collar_zone: None,
                aop: None,
                zones: None,
            },
            base_image: DynamicImage::ImageRgba8(RgbaImage::from_pixel(
                100,
                120,
                Rgba([255, 255, 255, 255]),
            )),
            displacement_map: None,
            print_mask: Some(DynamicImage::ImageLuma8(print_mask)),
            preserve_masks: vec![
                DynamicImage::ImageLuma8(collar_outer),
                DynamicImage::ImageLuma8(collar_inner),
            ],
        };

        let garment = resolve_panels(&template, &SeamPolicy::default()).unwrap();
        assert!(garment
            .panels
            .iter()
            .any(|panel| matches!(panel.id, PanelId::CollarInnerVisible)));
        assert!(garment
            .panels
            .iter()
            .any(|panel| matches!(panel.id, PanelId::LeftSleeve)));
        assert!(garment
            .panels
            .iter()
            .any(|panel| matches!(panel.id, PanelId::RightSleeve)));
    }
}
