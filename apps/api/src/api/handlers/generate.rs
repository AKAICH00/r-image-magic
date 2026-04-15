//! Mockup generation endpoint

use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use std::time::Instant;
use tracing::{error, info};
use url::Url;
use utoipa::ToSchema;

use crate::domain::PlacementSpec;
use crate::engine::MockupRequest;
use crate::AppState;

/// Request body for mockup generation
#[derive(Debug, Deserialize, ToSchema)]
pub struct GenerateRequest {
    /// URL of the design image to composite
    pub design_url: String,
    /// Template ID (e.g., "white_male_front")
    pub template_id: String,
    /// Placement specification
    pub placement: PlacementSpec,
    /// Optional generation options
    #[serde(default)]
    pub options: GenerateOptions,
}

/// Optional generation options
#[derive(Debug, Default, Deserialize, ToSchema)]
pub struct GenerateOptions {
    /// Displacement strength (0-30, default 10)
    #[serde(default = "default_displacement")]
    pub displacement_strength: f64,
    /// Hex color to tint the product template (e.g. "0D0D0D" for black)
    pub tint_color: Option<String>,
}

fn default_displacement() -> f64 {
    10.0
}

fn validate_design_url(design_url: &str) -> Result<(), String> {
    let parsed = Url::parse(design_url).map_err(|_| "design_url must be an absolute URL")?;
    match parsed.scheme() {
        "http" | "https" => Ok(()),
        scheme => Err(format!("design_url scheme '{}' is not supported", scheme)),
    }
}

fn validate_tint_color(tint_color: Option<&str>) -> Result<(), String> {
    let Some(color) = tint_color else {
        return Ok(());
    };

    let hex = color.strip_prefix('#').unwrap_or(color);
    if hex.len() == 6 && hex.chars().all(|c| c.is_ascii_hexdigit()) {
        Ok(())
    } else {
        Err("tint_color must be a 6-digit hex color".to_string())
    }
}

fn validate_displacement_strength(strength: f64, range: (f64, f64)) -> Result<(), String> {
    if !strength.is_finite() {
        return Err("displacement_strength must be finite".to_string());
    }

    let (min, max) = range;
    if strength < min || strength > max {
        Err(format!(
            "displacement_strength must be between {} and {}",
            min, max
        ))
    } else {
        Ok(())
    }
}

/// Response for successful mockup generation
#[derive(Serialize, ToSchema)]
pub struct GenerateResponse {
    pub success: bool,
    pub mockup_url: String,
    pub metadata: GenerateMetadata,
}

/// Metadata about the generation
#[derive(Serialize, ToSchema)]
pub struct GenerateMetadata {
    pub generation_time_ms: u64,
    pub template_used: String,
    pub dimensions: Dimensions,
}

#[derive(Serialize, ToSchema)]
pub struct Dimensions {
    pub width: u32,
    pub height: u32,
}

/// Error response
#[derive(Serialize, ToSchema)]
pub struct ErrorResponse {
    pub success: bool,
    pub error: ApiError,
}

#[derive(Serialize, ToSchema)]
pub struct ApiError {
    pub code: String,
    pub message: String,
}

/// POST /api/v1/mockups/generate - Generate a mockup
#[utoipa::path(
    post,
    path = "/api/v1/mockups/generate",
    tag = "mockups",
    request_body = GenerateRequest,
    responses(
        (status = 200, description = "Mockup generated successfully", body = GenerateResponse),
        (status = 400, description = "Invalid placement specification", body = ErrorResponse),
        (status = 404, description = "Template not found", body = ErrorResponse),
        (status = 500, description = "Generation failed", body = ErrorResponse)
    )
)]
pub async fn generate_mockup(
    state: web::Data<AppState>,
    body: web::Json<GenerateRequest>,
) -> HttpResponse {
    let start = Instant::now();

    info!(
        template_id = %body.template_id,
        design_url = %body.design_url,
        "Processing mockup generation request"
    );

    if body.template_id.trim().is_empty() {
        return HttpResponse::BadRequest().json(ErrorResponse {
            success: false,
            error: ApiError {
                code: "INVALID_TEMPLATE_ID".to_string(),
                message: "template_id is required".to_string(),
            },
        });
    }

    if let Err(e) = validate_design_url(&body.design_url) {
        return HttpResponse::BadRequest().json(ErrorResponse {
            success: false,
            error: ApiError {
                code: "INVALID_DESIGN_URL".to_string(),
                message: e,
            },
        });
    }

    // Validate template exists and get its print area dimensions
    let template = match state.template_manager.get(&body.template_id) {
        Some(t) => t,
        None => {
            error!(template_id = %body.template_id, "Template not found");
            return HttpResponse::NotFound().json(ErrorResponse {
                success: false,
                error: ApiError {
                    code: "TEMPLATE_NOT_FOUND".to_string(),
                    message: format!("Template '{}' does not exist", body.template_id),
                },
            });
        }
    };

    if let Err(e) = validate_displacement_strength(
        body.options.displacement_strength,
        template.metadata.displacement.strength_range,
    ) {
        return HttpResponse::BadRequest().json(ErrorResponse {
            success: false,
            error: ApiError {
                code: "INVALID_OPTIONS".to_string(),
                message: e,
            },
        });
    }

    if let Err(e) = validate_tint_color(body.options.tint_color.as_deref()) {
        return HttpResponse::BadRequest().json(ErrorResponse {
            success: false,
            error: ApiError {
                code: "INVALID_OPTIONS".to_string(),
                message: e,
            },
        });
    }

    // Create placement with template's actual print area dimensions
    let mut placement = body.placement.clone();
    placement.print_area_width = template.metadata.print_area.width as i32;
    placement.print_area_height = template.metadata.print_area.height as i32;

    // Validate placement with correct dimensions
    if let Err(e) = placement.validate() {
        error!(error = %e, "Invalid placement specification");
        return HttpResponse::BadRequest().json(ErrorResponse {
            success: false,
            error: ApiError {
                code: "INVALID_PLACEMENT".to_string(),
                message: e.to_string(),
            },
        });
    }

    // Create mockup request with adjusted placement
    let request = MockupRequest {
        design_url: body.design_url.clone(),
        template_id: body.template_id.clone(),
        placement,
        displacement_strength: body.options.displacement_strength,
        tint_color: body.options.tint_color.clone(),
    };

    // Generate mockup (this is the heavy lifting)
    match state.template_manager.generate_mockup(&request).await {
        Ok(result) => {
            let elapsed = start.elapsed().as_millis() as u64;

            info!(
                template_id = %body.template_id,
                generation_time_ms = elapsed,
                "Mockup generated successfully"
            );

            HttpResponse::Ok().json(GenerateResponse {
                success: true,
                mockup_url: result.url,
                metadata: GenerateMetadata {
                    generation_time_ms: elapsed,
                    template_used: body.template_id.clone(),
                    dimensions: Dimensions {
                        width: result.width,
                        height: result.height,
                    },
                },
            })
        }
        Err(e) => {
            error!(error = %e, "Mockup generation failed");
            HttpResponse::InternalServerError().json(ErrorResponse {
                success: false,
                error: ApiError {
                    code: "GENERATION_FAILED".to_string(),
                    message: e.to_string(),
                },
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_design_url_allows_http_urls() {
        assert!(validate_design_url("https://example.com/design.png").is_ok());
        assert!(validate_design_url("http://example.com/design.png").is_ok());
    }

    #[test]
    fn validate_design_url_rejects_relative_and_non_http_urls() {
        assert!(validate_design_url("/samples/design.png").is_err());
        assert!(validate_design_url("data:image/png;base64,abc").is_err());
        assert!(validate_design_url("file:///tmp/design.png").is_err());
    }

    #[test]
    fn validate_tint_color_accepts_six_digit_hex() {
        assert!(validate_tint_color(Some("0D0D0D")).is_ok());
        assert!(validate_tint_color(Some("#ff00AA")).is_ok());
        assert!(validate_tint_color(None).is_ok());
    }

    #[test]
    fn validate_tint_color_rejects_invalid_hex() {
        assert!(validate_tint_color(Some("#fff")).is_err());
        assert!(validate_tint_color(Some("#GGGGGG")).is_err());
    }

    #[test]
    fn validate_displacement_strength_enforces_template_range() {
        assert!(validate_displacement_strength(10.0, (0.0, 30.0)).is_ok());
        assert!(validate_displacement_strength(-1.0, (0.0, 30.0)).is_err());
        assert!(validate_displacement_strength(31.0, (0.0, 30.0)).is_err());
        assert!(validate_displacement_strength(f64::NAN, (0.0, 30.0)).is_err());
    }
}
