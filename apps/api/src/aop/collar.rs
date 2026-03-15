use image::{DynamicImage, GrayImage, Rgba, RgbaImage};

use crate::imageops::apply_mask;

use super::artwork::{load_artwork, ArtworkSource};
use super::config::{CollarInteriorConfig, CollarInteriorMode};

pub async fn render_collar_interior(
    client: &reqwest::Client,
    config: &CollarInteriorConfig,
    primary_artwork: &DynamicImage,
    front_projection: Option<&DynamicImage>,
    collar_mask: &GrayImage,
) -> Result<DynamicImage, reqwest::Error> {
    let base = match config.mode {
        CollarInteriorMode::DerivedFromArtwork => derive_from_artwork(
            front_projection.unwrap_or(primary_artwork),
            collar_mask,
            config.darken,
            config.texture_shift_px,
        ),
        CollarInteriorMode::SolidColor => fill_mask(
            collar_mask,
            config.solid_color_hex.as_deref().unwrap_or("111111"),
        ),
        CollarInteriorMode::DebugColor => fill_mask(
            collar_mask,
            config.debug_color_hex.as_deref().unwrap_or("ff0033"),
        ),
        CollarInteriorMode::CustomArtwork => {
            let artwork = load_artwork(
                client,
                &ArtworkSource::CustomUrl(config.custom_artwork_url.clone().unwrap_or_default()),
                primary_artwork,
            )
            .await?;
            apply_mask(&artwork, collar_mask)
        }
    };

    Ok(base)
}

fn derive_from_artwork(
    artwork: &DynamicImage,
    collar_mask: &GrayImage,
    darken: f32,
    texture_shift_px: i32,
) -> DynamicImage {
    let art = artwork.to_rgba8();
    let (width, height) = art.dimensions();
    let mut out = RgbaImage::new(width, height);

    for y in 0..height {
        for x in 0..width {
            if collar_mask.get_pixel(x, y).0[0] == 0 {
                continue;
            }

            let sample_x = (x as i32 + texture_shift_px).clamp(0, width as i32 - 1) as u32;
            let sample_y = (y as i32 + texture_shift_px.max(1)).clamp(0, height as i32 - 1) as u32;
            let mut pixel = *art.get_pixel(sample_x, sample_y);
            pixel.0[0] = ((pixel.0[0] as f32) * (1.0 - darken)).round() as u8;
            pixel.0[1] = ((pixel.0[1] as f32) * (1.0 - darken)).round() as u8;
            pixel.0[2] = ((pixel.0[2] as f32) * (1.0 - darken)).round() as u8;
            pixel.0[3] = collar_mask.get_pixel(x, y).0[0];
            out.put_pixel(x, y, pixel);
        }
    }

    DynamicImage::ImageRgba8(out)
}

fn fill_mask(mask: &GrayImage, hex: &str) -> DynamicImage {
    let (r, g, b) = parse_hex(hex);
    let (width, height) = mask.dimensions();
    let mut out = RgbaImage::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let alpha = mask.get_pixel(x, y).0[0];
            if alpha > 0 {
                out.put_pixel(x, y, Rgba([r, g, b, alpha]));
            }
        }
    }

    DynamicImage::ImageRgba8(out)
}

fn parse_hex(hex: &str) -> (u8, u8, u8) {
    let hex = hex.strip_prefix('#').unwrap_or(hex);
    if hex.len() == 6 {
        (
            u8::from_str_radix(&hex[0..2], 16).unwrap_or(255),
            u8::from_str_radix(&hex[2..4], 16).unwrap_or(0),
            u8::from_str_radix(&hex[4..6], 16).unwrap_or(0),
        )
    } else {
        (255, 0, 0)
    }
}
