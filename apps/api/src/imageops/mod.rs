use base64::Engine;
use image::{DynamicImage, GenericImageView, GrayImage, Luma, Rgba, RgbaImage};

pub fn alpha_mask(image: &DynamicImage) -> GrayImage {
    let rgba = image.to_rgba8();
    let (width, height) = rgba.dimensions();
    let mut mask = GrayImage::new(width, height);

    for y in 0..height {
        for x in 0..width {
            mask.put_pixel(x, y, Luma([rgba.get_pixel(x, y).0[3]]));
        }
    }

    mask
}

pub fn bounding_box(mask: &GrayImage) -> Option<(u32, u32, u32, u32)> {
    let (width, height) = mask.dimensions();
    let mut min_x = width;
    let mut min_y = height;
    let mut max_x = 0;
    let mut max_y = 0;
    let mut found = false;

    for y in 0..height {
        for x in 0..width {
            if mask.get_pixel(x, y).0[0] > 0 {
                min_x = min_x.min(x);
                min_y = min_y.min(y);
                max_x = max_x.max(x);
                max_y = max_y.max(y);
                found = true;
            }
        }
    }

    found.then_some((min_x, min_y, max_x + 1, max_y + 1))
}

pub fn apply_mask(image: &DynamicImage, mask: &GrayImage) -> DynamicImage {
    let rgba = image.to_rgba8();
    let (width, height) = rgba.dimensions();
    let mut output = RgbaImage::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let mut pixel = *rgba.get_pixel(x, y);
            let mask_alpha = mask.get_pixel(x, y).0[0] as f32 / 255.0;
            pixel.0[3] = (pixel.0[3] as f32 * mask_alpha) as u8;
            output.put_pixel(x, y, pixel);
        }
    }

    DynamicImage::ImageRgba8(output)
}

pub fn intersect_masks(a: &GrayImage, b: &GrayImage) -> GrayImage {
    let (width, height) = a.dimensions();
    let mut out = GrayImage::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let value = a.get_pixel(x, y).0[0].min(b.get_pixel(x, y).0[0]);
            out.put_pixel(x, y, Luma([value]));
        }
    }

    out
}

pub fn subtract_mask(a: &GrayImage, b: &GrayImage) -> GrayImage {
    let (width, height) = a.dimensions();
    let mut out = GrayImage::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let lhs = a.get_pixel(x, y).0[0];
            let rhs = b.get_pixel(x, y).0[0];
            out.put_pixel(x, y, Luma([lhs.saturating_sub(rhs)]));
        }
    }

    out
}

pub fn union_masks(masks: &[GrayImage]) -> Option<GrayImage> {
    let first = masks.first()?;
    let (width, height) = first.dimensions();
    let mut out = GrayImage::new(width, height);

    for mask in masks {
        for y in 0..height {
            for x in 0..width {
                let current = out.get_pixel(x, y).0[0];
                let incoming = mask.get_pixel(x, y).0[0];
                out.put_pixel(x, y, Luma([current.max(incoming)]));
            }
        }
    }

    Some(out)
}

pub fn mask_outline(mask: &GrayImage) -> GrayImage {
    subtract_mask(&dilate_mask(mask, 1), mask)
}

pub fn dilate_mask(mask: &GrayImage, radius: u32) -> GrayImage {
    if radius == 0 {
        return mask.clone();
    }

    let (width, height) = mask.dimensions();
    let mut out = GrayImage::new(width, height);
    let radius = radius as i32;

    for y in 0..height {
        for x in 0..width {
            let mut value = 0u8;
            'neighbors: for ny in (y as i32 - radius)..=(y as i32 + radius) {
                if !(0..height as i32).contains(&ny) {
                    continue;
                }
                for nx in (x as i32 - radius)..=(x as i32 + radius) {
                    if !(0..width as i32).contains(&nx) {
                        continue;
                    }
                    if mask.get_pixel(nx as u32, ny as u32).0[0] > 0 {
                        value = 255;
                        break 'neighbors;
                    }
                }
            }
            out.put_pixel(x, y, Luma([value]));
        }
    }

    out
}

pub fn erode_mask(mask: &GrayImage, radius: u32) -> GrayImage {
    if radius == 0 {
        return mask.clone();
    }

    let (width, height) = mask.dimensions();
    let mut out = GrayImage::new(width, height);
    let radius = radius as i32;

    for y in 0..height {
        for x in 0..width {
            let mut value = 255u8;
            'neighbors: for ny in (y as i32 - radius)..=(y as i32 + radius) {
                if !(0..height as i32).contains(&ny) {
                    value = 0;
                    break;
                }
                for nx in (x as i32 - radius)..=(x as i32 + radius) {
                    if !(0..width as i32).contains(&nx)
                        || mask.get_pixel(nx as u32, ny as u32).0[0] == 0
                    {
                        value = 0;
                        break 'neighbors;
                    }
                }
            }
            out.put_pixel(x, y, Luma([value]));
        }
    }

    out
}

pub fn feather_mask(mask: &GrayImage, radius: f32) -> GrayImage {
    DynamicImage::ImageLuma8(mask.clone())
        .blur(radius)
        .to_luma8()
}

pub fn overlay_color_mask(mask: &GrayImage, color: [u8; 4]) -> DynamicImage {
    let (width, height) = mask.dimensions();
    let mut out = RgbaImage::new(width, height);

    for y in 0..height {
        for x in 0..width {
            if mask.get_pixel(x, y).0[0] > 0 {
                out.put_pixel(x, y, Rgba(color));
            }
        }
    }

    DynamicImage::ImageRgba8(out)
}

pub fn composite_over(base: &DynamicImage, overlay: &DynamicImage) -> DynamicImage {
    let mut out = base.to_rgba8();
    let overlay = overlay.to_rgba8();
    let (width, height) = out.dimensions();

    for y in 0..height {
        for x in 0..width {
            let dst = *out.get_pixel(x, y);
            let src = *overlay.get_pixel(x, y);
            if src.0[3] == 0 {
                continue;
            }

            let alpha = src.0[3] as f32 / 255.0;
            let inv_alpha = 1.0 - alpha;
            let blended = Rgba([
                (src.0[0] as f32 * alpha + dst.0[0] as f32 * inv_alpha) as u8,
                (src.0[1] as f32 * alpha + dst.0[1] as f32 * inv_alpha) as u8,
                (src.0[2] as f32 * alpha + dst.0[2] as f32 * inv_alpha) as u8,
                ((src.0[3] as f32) + dst.0[3] as f32 * inv_alpha).clamp(0.0, 255.0) as u8,
            ]);
            out.put_pixel(x, y, blended);
        }
    }

    DynamicImage::ImageRgba8(out)
}

pub fn sample_bilinear(image: &RgbaImage, x: f32, y: f32) -> Rgba<u8> {
    let (width, height) = image.dimensions();
    let x0 = x.floor().clamp(0.0, (width - 1) as f32) as u32;
    let y0 = y.floor().clamp(0.0, (height - 1) as f32) as u32;
    let x1 = (x0 + 1).min(width - 1);
    let y1 = (y0 + 1).min(height - 1);
    let dx = (x - x0 as f32).clamp(0.0, 1.0);
    let dy = (y - y0 as f32).clamp(0.0, 1.0);

    let p00 = image.get_pixel(x0, y0);
    let p10 = image.get_pixel(x1, y0);
    let p01 = image.get_pixel(x0, y1);
    let p11 = image.get_pixel(x1, y1);

    let mut out = [0u8; 4];
    for i in 0..4 {
        let value = p00.0[i] as f32 * (1.0 - dx) * (1.0 - dy)
            + p10.0[i] as f32 * dx * (1.0 - dy)
            + p01.0[i] as f32 * (1.0 - dx) * dy
            + p11.0[i] as f32 * dx * dy;
        out[i] = value.clamp(0.0, 255.0) as u8;
    }

    Rgba(out)
}

pub fn encode_png(image: &DynamicImage) -> Result<Vec<u8>, image::ImageError> {
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

pub fn encode_png_data_url(image: &DynamicImage) -> Result<String, image::ImageError> {
    let buffer = encode_png(image)?;
    Ok(format!(
        "data:image/png;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(buffer)
    ))
}
