//! Template management endpoints

use actix_web::{web, HttpResponse};
use serde::Serialize;
use tracing::{info, error};
use utoipa::ToSchema;

use crate::AppState;
use crate::db::models::TemplateInfo;

/// Response for listing templates
#[derive(Serialize, ToSchema)]
pub struct TemplatesListResponse {
    pub success: bool,
    pub data: Vec<TemplateInfo>,
    pub count: usize,
}

/// Response for a single template
#[derive(Serialize, ToSchema)]
pub struct TemplateResponse {
    pub success: bool,
    pub data: TemplateInfo,
}

/// Product type with count
#[derive(Serialize, ToSchema)]
pub struct ProductTypeCount {
    pub product_type: String,
    pub count: i64,
}

/// Response for product types listing
#[derive(Serialize, ToSchema)]
pub struct ProductTypesResponse {
    pub success: bool,
    pub data: Vec<ProductTypeCount>,
}

/// Error response for template endpoints
#[derive(Serialize, ToSchema)]
pub struct TemplateErrorResponse {
    pub success: bool,
    pub error: TemplateApiError,
}

#[derive(Serialize, ToSchema)]
pub struct TemplateApiError {
    pub code: String,
    pub message: String,
}

/// GET /api/v1/templates - List all active templates
#[utoipa::path(
    get,
    path = "/api/v1/templates",
    tag = "templates",
    responses(
        (status = 200, description = "List of all active templates", body = TemplatesListResponse),
        (status = 503, description = "Database not available", body = TemplateErrorResponse)
    )
)]
pub async fn list_templates(state: web::Data<AppState>) -> HttpResponse {
    let repo = match &state.template_repo {
        Some(r) => r,
        None => {
            return HttpResponse::ServiceUnavailable().json(TemplateErrorResponse {
                success: false,
                error: TemplateApiError {
                    code: "DATABASE_UNAVAILABLE".to_string(),
                    message: "Database connection not available".to_string(),
                },
            });
        }
    };

    match repo.get_all_active().await {
        Ok(templates) => {
            let count = templates.len();
            let data: Vec<TemplateInfo> = templates.into_iter().map(|t| t.into()).collect();

            info!(count = count, "Retrieved templates list");

            HttpResponse::Ok().json(TemplatesListResponse {
                success: true,
                data,
                count,
            })
        }
        Err(e) => {
            error!(error = %e, "Failed to fetch templates");
            HttpResponse::InternalServerError().json(TemplateErrorResponse {
                success: false,
                error: TemplateApiError {
                    code: "DATABASE_ERROR".to_string(),
                    message: format!("Failed to fetch templates: {}", e),
                },
            })
        }
    }
}

/// GET /api/v1/templates/{template_id} - Get single template by ID
#[utoipa::path(
    get,
    path = "/api/v1/templates/{template_id}",
    tag = "templates",
    params(
        ("template_id" = String, Path, description = "Template identifier (e.g., 'white-tshirt-front')")
    ),
    responses(
        (status = 200, description = "Template details", body = TemplateResponse),
        (status = 404, description = "Template not found", body = TemplateErrorResponse),
        (status = 503, description = "Database not available", body = TemplateErrorResponse)
    )
)]
pub async fn get_template(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> HttpResponse {
    let template_id = path.into_inner();

    let repo = match &state.template_repo {
        Some(r) => r,
        None => {
            return HttpResponse::ServiceUnavailable().json(TemplateErrorResponse {
                success: false,
                error: TemplateApiError {
                    code: "DATABASE_UNAVAILABLE".to_string(),
                    message: "Database connection not available".to_string(),
                },
            });
        }
    };

    match repo.get_by_template_id(&template_id).await {
        Ok(Some(template)) => {
            info!(template_id = %template_id, "Retrieved template");

            HttpResponse::Ok().json(TemplateResponse {
                success: true,
                data: template.into(),
            })
        }
        Ok(None) => {
            HttpResponse::NotFound().json(TemplateErrorResponse {
                success: false,
                error: TemplateApiError {
                    code: "TEMPLATE_NOT_FOUND".to_string(),
                    message: format!("Template '{}' does not exist", template_id),
                },
            })
        }
        Err(e) => {
            error!(error = %e, template_id = %template_id, "Failed to fetch template");
            HttpResponse::InternalServerError().json(TemplateErrorResponse {
                success: false,
                error: TemplateApiError {
                    code: "DATABASE_ERROR".to_string(),
                    message: format!("Failed to fetch template: {}", e),
                },
            })
        }
    }
}

/// GET /api/v1/templates/product-types - List product types with counts
#[utoipa::path(
    get,
    path = "/api/v1/templates/product-types",
    tag = "templates",
    responses(
        (status = 200, description = "List of product types with template counts", body = ProductTypesResponse),
        (status = 503, description = "Database not available", body = TemplateErrorResponse)
    )
)]
pub async fn list_product_types(state: web::Data<AppState>) -> HttpResponse {
    let repo = match &state.template_repo {
        Some(r) => r,
        None => {
            return HttpResponse::ServiceUnavailable().json(TemplateErrorResponse {
                success: false,
                error: TemplateApiError {
                    code: "DATABASE_UNAVAILABLE".to_string(),
                    message: "Database connection not available".to_string(),
                },
            });
        }
    };

    match repo.get_product_type_counts().await {
        Ok(counts) => {
            let data: Vec<ProductTypeCount> = counts
                .into_iter()
                .map(|(product_type, count)| ProductTypeCount { product_type, count })
                .collect();

            info!(count = data.len(), "Retrieved product types");

            HttpResponse::Ok().json(ProductTypesResponse {
                success: true,
                data,
            })
        }
        Err(e) => {
            error!(error = %e, "Failed to fetch product types");
            HttpResponse::InternalServerError().json(TemplateErrorResponse {
                success: false,
                error: TemplateApiError {
                    code: "DATABASE_ERROR".to_string(),
                    message: format!("Failed to fetch product types: {}", e),
                },
            })
        }
    }
}

/// GET /api/v1/templates/by-type/{product_type} - Get templates by product type
#[utoipa::path(
    get,
    path = "/api/v1/templates/by-type/{product_type}",
    tag = "templates",
    params(
        ("product_type" = String, Path, description = "Product type (e.g., 'tshirt', 'mug', 'hoodie')")
    ),
    responses(
        (status = 200, description = "List of templates for the product type", body = TemplatesListResponse),
        (status = 503, description = "Database not available", body = TemplateErrorResponse)
    )
)]
pub async fn get_by_product_type(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> HttpResponse {
    let product_type = path.into_inner();

    let repo = match &state.template_repo {
        Some(r) => r,
        None => {
            return HttpResponse::ServiceUnavailable().json(TemplateErrorResponse {
                success: false,
                error: TemplateApiError {
                    code: "DATABASE_UNAVAILABLE".to_string(),
                    message: "Database connection not available".to_string(),
                },
            });
        }
    };

    match repo.get_by_product_type(&product_type).await {
        Ok(templates) => {
            let count = templates.len();
            let data: Vec<TemplateInfo> = templates.into_iter().map(|t| t.into()).collect();

            info!(product_type = %product_type, count = count, "Retrieved templates by type");

            HttpResponse::Ok().json(TemplatesListResponse {
                success: true,
                data,
                count,
            })
        }
        Err(e) => {
            error!(error = %e, product_type = %product_type, "Failed to fetch templates by type");
            HttpResponse::InternalServerError().json(TemplateErrorResponse {
                success: false,
                error: TemplateApiError {
                    code: "DATABASE_ERROR".to_string(),
                    message: format!("Failed to fetch templates: {}", e),
                },
            })
        }
    }
}
