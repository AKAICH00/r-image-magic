//! Displacement mapping algorithm
//!
//! Applies displacement effects to make designs follow fabric wrinkles and folds.

use image::{DynamicImage, GenericImageView, Rgba, RgbaImage};
use rayon::prelude::*;

/// Apply displacement mapping to a design image
///
/// The displacement map is a grayscale image where:
/// - Black (0) = push pixels left/up
/// - Gray (128) = no displacement
/// - White (255) = push pixels right/down
///
/// # Arguments
/// * `design` - The design image to displace
/// * `displacement_map` - Grayscale displacement map
/// * `strength` - Displacement strength in pixels (typical: 5-15)
///
/// # Returns
/// A new image with displacement applied
pub fn apply_displacement(
    design: &DynamicImage,
    displacement_map: &DynamicImage,
    strength: f64,
) -> DynamicImage {
    let (width, height) = design.dimensions();
    let design_rgba = design.to_rgba8();
    let disp_gray = displacement_map.to_luma8();

    // Resize displacement map to match design if needed
    let disp_resized = if disp_gray.dimensions() != (width, height) {
        image::imageops::resize(
            &disp_gray,
            width,
            height,
            image::imageops::FilterType::Lanczos3,
        )
    } else {
        disp_gray
    };

    let mut output = RgbaImage::new(width, height);

    // Process rows in parallel using Rayon
    let rows: Vec<_> = (0..height)
        .into_par_iter()
        .map(|y| {
            let mut row = vec![Rgba([0u8, 0, 0, 0]); width as usize];

            for x in 0..width {
                // Get displacement value (0-255 normalized to -0.5 to 0.5)
                let disp_value = disp_resized.get_pixel(x, y).0[0] as f64 / 255.0 - 0.5;

                // Calculate source coordinates with displacement
                let src_x = (x as f64 + disp_value * strength).clamp(0.0, (width - 1) as f64);
                let src_y = (y as f64 + disp_value * strength).clamp(0.0, (height - 1) as f64);

                // Bilinear interpolation for smooth sampling
                let pixel = bilinear_sample(&design_rgba, src_x, src_y);
                row[x as usize] = pixel;
            }

            row
        })
        .collect();

    // Copy rows to output image
    for (y, row) in rows.into_iter().enumerate() {
        for (x, pixel) in row.into_iter().enumerate() {
            output.put_pixel(x as u32, y as u32, pixel);
        }
    }

    DynamicImage::ImageRgba8(output)
}

/// Bilinear interpolation for smooth pixel sampling
fn bilinear_sample(image: &RgbaImage, x: f64, y: f64) -> Rgba<u8> {
    let (width, height) = image.dimensions();

    let x0 = x.floor() as u32;
    let y0 = y.floor() as u32;
    let x1 = (x0 + 1).min(width - 1);
    let y1 = (y0 + 1).min(height - 1);

    let dx = x - x0 as f64;
    let dy = y - y0 as f64;

    let p00 = image.get_pixel(x0, y0);
    let p10 = image.get_pixel(x1, y0);
    let p01 = image.get_pixel(x0, y1);
    let p11 = image.get_pixel(x1, y1);

    let mut result = [0u8; 4];
    for i in 0..4 {
        let v00 = p00.0[i] as f64;
        let v10 = p10.0[i] as f64;
        let v01 = p01.0[i] as f64;
        let v11 = p11.0[i] as f64;

        // Bilinear interpolation formula
        let value = v00 * (1.0 - dx) * (1.0 - dy)
            + v10 * dx * (1.0 - dy)
            + v01 * (1.0 - dx) * dy
            + v11 * dx * dy;

        result[i] = value.clamp(0.0, 255.0) as u8;
    }

    Rgba(result)
}

/// Apply multiply blend mode
///
/// Darkens the base image based on the overlay, useful for fabric shadows
#[allow(dead_code)]
pub fn blend_multiply(base: &RgbaImage, overlay: &RgbaImage) -> RgbaImage {
    let (width, height) = base.dimensions();
    let mut output = RgbaImage::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let base_pixel = base.get_pixel(x, y);
            let overlay_pixel = overlay.get_pixel(x, y);

            let mut result = [0u8; 4];
            for i in 0..3 {
                // Multiply: (base * overlay) / 255
                result[i] = ((base_pixel.0[i] as u32 * overlay_pixel.0[i] as u32) / 255) as u8;
            }
            // Alpha: use overlay alpha
            result[3] = overlay_pixel.0[3];

            output.put_pixel(x, y, Rgba(result));
        }
    }

    output
}

/// Apply opacity to an image
pub fn apply_opacity(image: &RgbaImage, opacity: u8) -> RgbaImage {
    let opacity_factor = opacity as f64 / 255.0;
    let (width, height) = image.dimensions();
    let mut output = RgbaImage::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let pixel = image.get_pixel(x, y);
            let new_alpha = (pixel.0[3] as f64 * opacity_factor) as u8;
            output.put_pixel(x, y, Rgba([pixel.0[0], pixel.0[1], pixel.0[2], new_alpha]));
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bilinear_sample_center() {
        let mut img = RgbaImage::new(2, 2);
        img.put_pixel(0, 0, Rgba([100, 100, 100, 255]));
        img.put_pixel(1, 0, Rgba([200, 200, 200, 255]));
        img.put_pixel(0, 1, Rgba([100, 100, 100, 255]));
        img.put_pixel(1, 1, Rgba([200, 200, 200, 255]));

        let result = bilinear_sample(&img, 0.5, 0.5);
        // Should be average of all 4 pixels = 150
        assert!((result.0[0] as i32 - 150).abs() < 5);
    }
}
