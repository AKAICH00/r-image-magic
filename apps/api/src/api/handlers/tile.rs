//! Pattern tiling endpoint

use actix_web::{web, HttpResponse};
use base64::Engine;
use image::imageops::FilterType;
use image::{DynamicImage, GenericImage, ImageFormat, RgbaImage};
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use utoipa::ToSchema;

use crate::api::handlers::generate::{ApiError, ErrorResponse};

const DEFAULT_MAX_DIM: u32 = 4096;

#[derive(Debug, Deserialize, ToSchema)]
pub struct TileRequest {
    /// URL of the seamless tile image
    pub tile_url: String,
    /// Number of horizontal repeats
    #[serde(default = "default_repeat")]
    pub repeat_x: u32,
    /// Number of vertical repeats
    #[serde(default = "default_repeat")]
    pub repeat_y: u32,
    /// Optional max output width
    pub max_width: Option<u32>,
    /// Optional max output height
    pub max_height: Option<u32>,
    /// Optional preset: web | print | highRes | max
    pub preset: Option<String>,
    /// Layout mode (currently only "grid")
    #[serde(default = "default_layout")]
    pub layout: String,
}

fn default_repeat() -> u32 {
    4
}

fn default_layout() -> String {
    "grid".to_string()
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TileResponse {
    pub success: bool,
    pub tiled_url: String,
    pub metadata: TileMetadata,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TileMetadata {
    pub width: u32,
    pub height: u32,
    pub repeat_x: u32,
    pub repeat_y: u32,
}

fn preset_dim(preset: &str) -> Option<u32> {
    match preset {
        "web" => Some(2048),
        "print" => Some(4096),
        "highRes" => Some(8192),
        "max" => Some(10000),
        _ => None,
    }
}

fn resolve_max_dims(body: &TileRequest) -> Result<(u32, u32), String> {
    let preset_limit = match body.preset.as_deref() {
        Some(p) => Some(preset_dim(p).ok_or_else(|| {
            format!(
                "Invalid preset {}. Use one of: web, print, highRes, max",
                p
            )
        })?),
        None => None,
    };

    let max_width = body
        .max_width
        .or(preset_limit)
        .unwrap_or(DEFAULT_MAX_DIM)
        .max(1);
    let max_height = body
        .max_height
        .or(preset_limit)
        .unwrap_or(DEFAULT_MAX_DIM)
        .max(1);

    Ok((max_width, max_height))
}

#[utoipa::path(
    post,
    path = "/api/v1/tile",
    tag = "mockups",
    request_body = TileRequest,
    responses(
        (status = 200, description = "Tile image generated successfully", body = TileResponse),
        (status = 400, description = "Invalid tile request", body = ErrorResponse),
        (status = 500, description = "Tile generation failed", body = ErrorResponse)
    )
)]
pub async fn tile_pattern(body: web::Json<TileRequest>) -> HttpResponse {
    if body.repeat_x == 0 || body.repeat_y == 0 {
        return HttpResponse::BadRequest().json(ErrorResponse {
            success: false,
            error: ApiError {
                code: "INVALID_REPEAT".to_string(),
                message: "repeat_x and repeat_y must be greater than 0".to_string(),
            },
        });
    }

    if body.layout != "grid" {
        return HttpResponse::BadRequest().json(ErrorResponse {
            success: false,
            error: ApiError {
                code: "UNSUPPORTED_LAYOUT".to_string(),
                message: "Only grid layout is currently supported".to_string(),
            },
        });
    }

    let (max_width, max_height) = match resolve_max_dims(&body) {
        Ok(v) => v,
        Err(msg) => {
            return HttpResponse::BadRequest().json(ErrorResponse {
                success: false,
                error: ApiError {
                    code: "INVALID_PRESET".to_string(),
                    message: msg,
                },
            });
        }
    };

    let response = match reqwest::get(&body.tile_url).await {
        Ok(resp) => resp,
        Err(e) => {
            return HttpResponse::InternalServerError().json(ErrorResponse {
                success: false,
                error: ApiError {
                    code: "TILE_FETCH_FAILED".to_string(),
                    message: format!("Failed to fetch tile image: {}", e),
                },
            });
        }
    };

    if !response.status().is_success() {
        return HttpResponse::BadRequest().json(ErrorResponse {
            success: false,
            error: ApiError {
                code: "TILE_FETCH_FAILED".to_string(),
                message: format!("Tile URL returned HTTP {}", response.status()),
            },
        });
    }

    let bytes = match response.bytes().await {
        Ok(b) => b,
        Err(e) => {
            return HttpResponse::InternalServerError().json(ErrorResponse {
                success: false,
                error: ApiError {
                    code: "TILE_READ_FAILED".to_string(),
                    message: format!("Failed reading tile bytes: {}", e),
                },
            });
        }
    };

    let tile_img = match image::load_from_memory(&bytes) {
        Ok(img) => img.to_rgba8(),
        Err(e) => {
            return HttpResponse::BadRequest().json(ErrorResponse {
                success: false,
                error: ApiError {
                    code: "TILE_DECODE_FAILED".to_string(),
                    message: format!("Failed to decode tile image: {}", e),
                },
            });
        }
    };

    let tile_width = tile_img.width();
    let tile_height = tile_img.height();

    if tile_width == 0 || tile_height == 0 {
        return HttpResponse::BadRequest().json(ErrorResponse {
            success: false,
            error: ApiError {
                code: "INVALID_TILE".to_string(),
                message: "Tile image has invalid dimensions".to_string(),
            },
        });
    }

    let output_width = match tile_width.checked_mul(body.repeat_x) {
        Some(v) => v,
        None => {
            return HttpResponse::BadRequest().json(ErrorResponse {
                success: false,
                error: ApiError {
                    code: "OUTPUT_TOO_LARGE".to_string(),
                    message: "Computed output width is too large".to_string(),
                },
            });
        }
    };

    let output_height = match tile_height.checked_mul(body.repeat_y) {
        Some(v) => v,
        None => {
            return HttpResponse::BadRequest().json(ErrorResponse {
                success: false,
                error: ApiError {
                    code: "OUTPUT_TOO_LARGE".to_string(),
                    message: "Computed output height is too large".to_string(),
                },
            });
        }
    };

    let mut output: RgbaImage = RgbaImage::new(output_width, output_height);

    for y in 0..body.repeat_y {
        for x in 0..body.repeat_x {
            if let Err(e) = output.copy_from(&tile_img, x * tile_width, y * tile_height) {
                return HttpResponse::InternalServerError().json(ErrorResponse {
                    success: false,
                    error: ApiError {
                        code: "TILE_COMPOSE_FAILED".to_string(),
                        message: format!("Failed composing tile image: {}", e),
                    },
                });
            }
        }
    }

    let final_buffer = if output_width > max_width || output_height > max_height {
        let scale_w = max_width as f32 / output_width as f32;
        let scale_h = max_height as f32 / output_height as f32;
        let scale = scale_w.min(scale_h);

        let new_width = ((output_width as f32 * scale).round() as u32).max(1);
        let new_height = ((output_height as f32 * scale).round() as u32).max(1);

        image::imageops::resize(&output, new_width, new_height, FilterType::Lanczos3)
    } else {
        output
    };

    let final_width = final_buffer.width();
    let final_height = final_buffer.height();

    let dyn_img = DynamicImage::ImageRgba8(final_buffer);
    let mut cursor = Cursor::new(Vec::new());

    if let Err(e) = dyn_img.write_to(&mut cursor, ImageFormat::Png) {
        return HttpResponse::InternalServerError().json(ErrorResponse {
            success: false,
            error: ApiError {
                code: "PNG_ENCODE_FAILED".to_string(),
                message: format!("Failed to encode PNG: {}", e),
            },
        });
    }

    let png_bytes = cursor.into_inner();
    let data_url = format!(
        "data:image/png;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(&png_bytes)
    );

    HttpResponse::Ok().json(TileResponse {
        success: true,
        tiled_url: data_url,
        metadata: TileMetadata {
            width: final_width,
            height: final_height,
            repeat_x: body.repeat_x,
            repeat_y: body.repeat_y,
        },
    })
}
