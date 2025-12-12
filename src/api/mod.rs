//! API module - HTTP routes and handlers

pub mod handlers;

use actix_web::web;

/// Configure all API routes
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .service(
                web::scope("/mockups")
                    .route("/generate", web::post().to(handlers::generate::generate_mockup))
            )
    )
    .route("/health", web::get().to(handlers::health::health_check));
}
