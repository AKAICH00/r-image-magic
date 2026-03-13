//! OpenAPI 3.0 specification definition

use utoipa::OpenApi;

use crate::api::handlers::{
    generate::{
        ApiError, Dimensions, ErrorResponse, GenerateMetadata, GenerateOptions, GenerateRequest,
        GenerateResponse,
    },
    health::HealthResponse,
    templates::{
        ProductTypeCount, ProductTypesResponse, TemplateApiError, TemplateErrorResponse,
        TemplateResponse, TemplatesListResponse,
    },
};
use crate::db::models::{DimensionsInfo, PrintAreaInfo, TemplateInfo};
use crate::domain::{CoordinateSpace, PlacementSpec, PlacementType};

#[derive(OpenApi)]
#[openapi(
    info(
        title = "R-Image-Magic API",
        version = "1.0.0",
        description = "High-performance mockup generation service for print-on-demand products",
        contact(
            name = "API Support",
            email = "support@example.com"
        ),
        license(
            name = "MIT",
            url = "https://opensource.org/licenses/MIT"
        )
    ),
    servers(
        (url = "/", description = "Current server")
    ),
    tags(
        (name = "system", description = "System health and status endpoints"),
        (name = "mockups", description = "Mockup generation endpoints"),
        (name = "templates", description = "Template management endpoints")
    ),
    paths(
        crate::api::handlers::health::health_check,
        crate::api::handlers::generate::generate_mockup,
        crate::api::handlers::templates::list_templates,
        crate::api::handlers::templates::get_template,
        crate::api::handlers::templates::list_product_types,
        crate::api::handlers::templates::get_by_product_type,
    ),
    components(
        schemas(
            // Health schemas
            HealthResponse,
            // Generate schemas
            GenerateRequest,
            GenerateOptions,
            GenerateResponse,
            GenerateMetadata,
            Dimensions,
            ErrorResponse,
            ApiError,
            // Template schemas
            TemplatesListResponse,
            TemplateResponse,
            ProductTypesResponse,
            ProductTypeCount,
            TemplateErrorResponse,
            TemplateApiError,
            TemplateInfo,
            DimensionsInfo,
            PrintAreaInfo,
            // Domain schemas
            PlacementSpec,
            PlacementType,
            CoordinateSpace,
        )
    )
)]
pub struct ApiDoc;
