//! API module - HTTP routes and handlers

pub mod handlers;
pub mod middleware;
pub mod openapi;

use actix_web::web;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::api::openapi::ApiDoc;

/// Configure all API routes
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .service(
                web::scope("/mockups")
                    .route("/generate", web::post().to(handlers::generate::generate_mockup))
            )
            .service(
                web::scope("/templates")
                    // More specific routes first
                    .route("/product-types", web::get().to(handlers::templates::list_product_types))
                    .route("/by-type/{product_type}", web::get().to(handlers::templates::get_by_product_type))
                    // General routes
                    .route("", web::get().to(handlers::templates::list_templates))
                    .route("/{template_id}", web::get().to(handlers::templates::get_template))
            )
            // API key management endpoints
            .service(
                web::scope("/keys")
                    .route("", web::post().to(handlers::keys::create_api_key))
                    .route("", web::get().to(handlers::keys::list_keys))
                    .route("/me", web::get().to(handlers::keys::get_my_key))
                    .route("/{id}", web::get().to(handlers::keys::get_key_by_id))
                    .route("/{id}", web::delete().to(handlers::keys::revoke_key))
            )
            // Usage statistics endpoints
            .service(
                web::scope("/usage")
                    .route("", web::get().to(handlers::usage::get_usage_stats))
                    .route("/history", web::get().to(handlers::usage::get_usage_history))
                    .route("/billing", web::get().to(handlers::usage::get_billing_summary))
                    .route("/month/{year_month}", web::get().to(handlers::usage::get_month_usage))
            )
    )
    .route("/health", web::get().to(handlers::health::health_check))
    // Swagger UI and OpenAPI spec
    .service(
        SwaggerUi::new("/swagger-ui/{_:.*}")
            .url("/api-docs/openapi.json", ApiDoc::openapi())
    );
}
