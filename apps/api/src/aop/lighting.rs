use image::{DynamicImage, GenericImageView, Rgba, RgbaImage};

use super::config::FabricProfile;

pub fn apply_fabric_lighting(
    base: &DynamicImage,
    artwork: &DynamicImage,
    fabric: &FabricProfile,
) -> DynamicImage {
    let base = base.to_rgba8();
    let art = artwork.to_rgba8();
    let (width, height) = base.dimensions();
    let mut out = RgbaImage::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let base_pixel = *base.get_pixel(x, y);
            let art_pixel = *art.get_pixel(x, y);
            if art_pixel.0[3] == 0 {
                out.put_pixel(x, y, base_pixel);
                continue;
            }

            let luminance = (0.2126 * base_pixel.0[0] as f32
                + 0.7152 * base_pixel.0[1] as f32
                + 0.0722 * base_pixel.0[2] as f32)
                / 255.0;
            let shadow = 1.0 - (1.0 - luminance) * fabric.shadow_strength;
            let highlight = (luminance - 0.5).max(0.0) * fabric.highlight_strength;

            let mut blended = [0u8; 4];
            for channel in 0..3 {
                let lit = art_pixel.0[channel] as f32 * shadow + 255.0 * highlight * 0.15;
                blended[channel] = lit.clamp(0.0, 255.0) as u8;
            }
            blended[3] = art_pixel.0[3];
            out.put_pixel(x, y, Rgba(blended));
        }
    }

    DynamicImage::ImageRgba8(out)
}
