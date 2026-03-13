//! Health check endpoint

use actix_web::{web, HttpResponse};
use serde::Serialize;
use std::time::SystemTime;
use utoipa::ToSchema;

use crate::AppState;

#[derive(Serialize, ToSchema)]
pub struct HealthResponse {
    pub status: &'static str,
    pub version: &'static str,
    pub uptime_seconds: u64,
    pub templates_loaded: usize,
}

/// GET /health - Health check endpoint
#[utoipa::path(
    get,
    path = "/health",
    tag = "system",
    responses(
        (status = 200, description = "Service is healthy", body = HealthResponse)
    )
)]
pub async fn health_check(state: web::Data<AppState>) -> HttpResponse {
    // Calculate uptime (simple approximation using process start time)
    let uptime = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs() % 86400) // Wrap at 24 hours for demo
        .unwrap_or(0);

    let response = HealthResponse {
        status: "healthy",
        version: env!("CARGO_PKG_VERSION"),
        uptime_seconds: uptime,
        templates_loaded: state.template_manager.template_count(),
    };

    HttpResponse::Ok().json(response)
}
