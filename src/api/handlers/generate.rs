//! Mockup generation endpoint

use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use tracing::{info, error};
use std::time::Instant;

use crate::AppState;
use crate::domain::PlacementSpec;
use crate::engine::MockupRequest;

/// Request body for mockup generation
#[derive(Debug, Deserialize)]
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
#[derive(Debug, Default, Deserialize)]
pub struct GenerateOptions {
    /// JPEG output quality (1-100, default 85)
    #[serde(default = "default_quality")]
    pub output_quality: u8,
    /// Displacement strength (0-30, default 10)
    #[serde(default = "default_displacement")]
    pub displacement_strength: f64,
}

fn default_quality() -> u8 { 85 }
fn default_displacement() -> f64 { 10.0 }

/// Response for successful mockup generation
#[derive(Serialize)]
pub struct GenerateResponse {
    pub success: bool,
    pub mockup_url: String,
    pub metadata: GenerateMetadata,
}

/// Metadata about the generation
#[derive(Serialize)]
pub struct GenerateMetadata {
    pub generation_time_ms: u64,
    pub template_used: String,
    pub dimensions: Dimensions,
}

#[derive(Serialize)]
pub struct Dimensions {
    pub width: u32,
    pub height: u32,
}

/// Error response
#[derive(Serialize)]
pub struct ErrorResponse {
    pub success: bool,
    pub error: ApiError,
}

#[derive(Serialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
}

/// POST /api/v1/mockups/generate - Generate a mockup
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

    // Validate template exists
    let _template = match state.template_manager.get(&body.template_id) {
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

    // Validate placement
    if let Err(e) = body.placement.validate() {
        error!(error = %e, "Invalid placement specification");
        return HttpResponse::BadRequest().json(ErrorResponse {
            success: false,
            error: ApiError {
                code: "INVALID_PLACEMENT".to_string(),
                message: e.to_string(),
            },
        });
    }

    // Create mockup request
    let request = MockupRequest {
        design_url: body.design_url.clone(),
        template_id: body.template_id.clone(),
        placement: body.placement.clone(),
        displacement_strength: body.options.displacement_strength,
        output_quality: body.options.output_quality,
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
