use image::{DynamicImage, Rgba, RgbaImage};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::config::{PrintMode, TransparencyMode, WhiteMode};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtworkKind {
    FullGarment,
    TiledPattern,
    PlacedGraphic,
    DerivedCollarInterior,
    CustomArtwork,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtworkTransform {
    pub scale_x: f32,
    pub scale_y: f32,
    pub rotation_degrees: f32,
    pub offset_x: i32,
    pub offset_y: i32,
    pub mirror_x: bool,
    pub mirror_y: bool,
}

impl Default for ArtworkTransform {
    fn default() -> Self {
        Self {
            scale_x: 1.0,
            scale_y: 1.0,
            rotation_degrees: 0.0,
            offset_x: 0,
            offset_y: 0,
            mirror_x: false,
            mirror_y: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtworkSource {
    PrimaryArtwork,
    CustomUrl(String),
    SolidColor(String),
    DebugColor(String),
}

pub async fn load_artwork(
    client: &Client,
    source: &ArtworkSource,
    primary: &DynamicImage,
) -> Result<DynamicImage, reqwest::Error> {
    match source {
        ArtworkSource::PrimaryArtwork => Ok(primary.clone()),
        ArtworkSource::SolidColor(hex) | ArtworkSource::DebugColor(hex) => {
            Ok(fill_image(primary.width(), primary.height(), hex))
        }
        ArtworkSource::CustomUrl(url) => {
            let bytes = client.get(url).send().await?.bytes().await?;
            Ok(image::load_from_memory(&bytes).unwrap_or_else(|_| primary.clone()))
        }
    }
}

pub fn default_artwork_kind(mode: PrintMode) -> ArtworkKind {
    match mode {
        PrintMode::StandardLogo => ArtworkKind::PlacedGraphic,
        PrintMode::AllOverPattern => ArtworkKind::TiledPattern,
        PrintMode::AllOverFullArtwork | PrintMode::AllOverHybrid => ArtworkKind::FullGarment,
    }
}

pub fn normalize_artwork(
    image: &DynamicImage,
    white_mode: WhiteMode,
    transparency_mode: TransparencyMode,
) -> DynamicImage {
    let rgba = image.to_rgba8();
    let (width, height) = rgba.dimensions();
    let mut out = RgbaImage::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let mut pixel = *rgba.get_pixel(x, y);
            let is_white = pixel.0[0] > 245 && pixel.0[1] > 245 && pixel.0[2] > 245;

            if matches!(white_mode, WhiteMode::TreatWhiteAsBaseFabric) && is_white {
                pixel.0[3] = 0;
            }

            if matches!(transparency_mode, TransparencyMode::TransparentMeansIgnore)
                && pixel.0[3] == 0
            {
                pixel.0[3] = 255;
            }

            out.put_pixel(x, y, pixel);
        }
    }

    DynamicImage::ImageRgba8(out)
}

fn fill_image(width: u32, height: u32, hex: &str) -> DynamicImage {
    let hex = hex.strip_prefix('#').unwrap_or(hex);
    let (r, g, b) = if hex.len() == 6 {
        (
            u8::from_str_radix(&hex[0..2], 16).unwrap_or(255),
            u8::from_str_radix(&hex[2..4], 16).unwrap_or(0),
            u8::from_str_radix(&hex[4..6], 16).unwrap_or(0),
        )
    } else {
        (255, 0, 0)
    };

    DynamicImage::ImageRgba8(RgbaImage::from_pixel(width, height, Rgba([r, g, b, 255])))
}
