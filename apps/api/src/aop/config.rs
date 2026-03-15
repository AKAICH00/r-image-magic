use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum PrintMode {
    StandardLogo,
    AllOverFullArtwork,
    AllOverPattern,
    AllOverHybrid,
}

impl Default for PrintMode {
    fn default() -> Self {
        Self::StandardLogo
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum WhiteMode {
    TreatWhiteAsPrintedInk,
    TreatWhiteAsBaseFabric,
}

impl Default for WhiteMode {
    fn default() -> Self {
        Self::TreatWhiteAsPrintedInk
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum TransparencyMode {
    TransparentMeansNoInk,
    TransparentMeansIgnore,
}

impl Default for TransparencyMode {
    fn default() -> Self {
        Self::TransparentMeansNoInk
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum RenderIntent {
    CatalogClean,
    EcommerceRealistic,
    ProductionQa,
}

impl Default for RenderIntent {
    fn default() -> Self {
        Self::EcommerceRealistic
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum CollarInteriorMode {
    DerivedFromArtwork,
    SolidColor,
    DebugColor,
    CustomArtwork,
}

impl Default for CollarInteriorMode {
    fn default() -> Self {
        Self::DerivedFromArtwork
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FabricProfile {
    pub id: String,
    pub displacement_scale: f32,
    pub shadow_strength: f32,
    pub highlight_strength: f32,
    pub seam_darkening: f32,
    pub texture_shift_px: i32,
}

impl FabricProfile {
    pub fn synthetic_aop_default() -> Self {
        Self {
            id: "synthetic_aop_default".to_string(),
            displacement_scale: 1.0,
            shadow_strength: 0.22,
            highlight_strength: 0.15,
            seam_darkening: 0.12,
            texture_shift_px: 6,
        }
    }

    pub fn cotton_aop_default() -> Self {
        Self {
            id: "cotton_aop_default".to_string(),
            displacement_scale: 0.85,
            shadow_strength: 0.28,
            highlight_strength: 0.1,
            seam_darkening: 0.16,
            texture_shift_px: 4,
        }
    }

    pub fn debug_flat() -> Self {
        Self {
            id: "debug_flat".to_string(),
            displacement_scale: 0.0,
            shadow_strength: 0.0,
            highlight_strength: 0.0,
            seam_darkening: 0.0,
            texture_shift_px: 0,
        }
    }
}

impl Default for FabricProfile {
    fn default() -> Self {
        Self::synthetic_aop_default()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SeamPolicy {
    pub bleed_px: u32,
    pub safe_margin_px: u32,
    pub blend_px: u32,
    pub occlusion_strength: f32,
}

impl Default for SeamPolicy {
    fn default() -> Self {
        Self {
            bleed_px: 18,
            safe_margin_px: 10,
            blend_px: 8,
            occlusion_strength: 0.18,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DebugOverlayOptions {
    pub enabled: bool,
    pub panel_boundaries: bool,
    pub seam_zones: bool,
    pub bleed_zones: bool,
    pub safe_margins: bool,
    pub collar_inner_region: bool,
    pub stretch_heatmap: bool,
    pub transparency_warning_map: bool,
}

impl Default for DebugOverlayOptions {
    fn default() -> Self {
        Self {
            enabled: false,
            panel_boundaries: true,
            seam_zones: true,
            bleed_zones: true,
            safe_margins: true,
            collar_inner_region: true,
            stretch_heatmap: true,
            transparency_warning_map: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CollarInteriorConfig {
    #[serde(default)]
    pub mode: CollarInteriorMode,
    pub solid_color_hex: Option<String>,
    pub debug_color_hex: Option<String>,
    pub custom_artwork_url: Option<String>,
    pub darken: f32,
    pub texture_shift_px: i32,
    pub use_panel_displacement: bool,
}

impl Default for CollarInteriorConfig {
    fn default() -> Self {
        Self {
            mode: CollarInteriorMode::DerivedFromArtwork,
            solid_color_hex: Some("111111".to_string()),
            debug_color_hex: Some("ff0033".to_string()),
            custom_artwork_url: None,
            darken: 0.18,
            texture_shift_px: 4,
            use_panel_displacement: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AopRenderConfig {
    #[serde(default)]
    pub render_intent: RenderIntent,
    #[serde(default)]
    pub fabric: FabricProfile,
    #[serde(default)]
    pub seam_policy: SeamPolicy,
    #[serde(default)]
    pub collar: CollarInteriorConfig,
    #[serde(default)]
    pub white_mode: WhiteMode,
    #[serde(default)]
    pub transparency_mode: TransparencyMode,
    #[serde(default)]
    pub debug: DebugOverlayOptions,
    pub tile_scale: Option<f32>,
    #[serde(default)]
    pub pattern_offset_x: i32,
    #[serde(default)]
    pub pattern_offset_y: i32,
    #[serde(default)]
    pub brand_label_enabled: bool,
    #[serde(default)]
    pub qa_force_red_collar: bool,
}

impl Default for AopRenderConfig {
    fn default() -> Self {
        Self {
            render_intent: RenderIntent::EcommerceRealistic,
            fabric: FabricProfile::synthetic_aop_default(),
            seam_policy: SeamPolicy::default(),
            collar: CollarInteriorConfig::default(),
            white_mode: WhiteMode::TreatWhiteAsPrintedInk,
            transparency_mode: TransparencyMode::TransparentMeansNoInk,
            debug: DebugOverlayOptions::default(),
            tile_scale: Some(1.0),
            pattern_offset_x: 0,
            pattern_offset_y: 0,
            brand_label_enabled: false,
            qa_force_red_collar: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_mode_defaults_to_standard_logo() {
        assert_eq!(PrintMode::default(), PrintMode::StandardLogo);
    }

    #[test]
    fn test_printful_style_defaults_are_seam_aware() {
        let config = AopRenderConfig::default();
        assert_eq!(config.white_mode, WhiteMode::TreatWhiteAsPrintedInk);
        assert_eq!(
            config.transparency_mode,
            TransparencyMode::TransparentMeansNoInk
        );
        assert_eq!(config.collar.mode, CollarInteriorMode::DerivedFromArtwork);
        assert!(config.seam_policy.bleed_px > 0);
    }
}
