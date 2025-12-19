//! R-Image-Magic
//!
//! High-performance image compositing and mockup generation API using Rust + Actix-Web.
//! Features true displacement mapping for realistic product mockups.
//! Designed for 10K+ concurrent connections.

use actix_web::{web, App, HttpServer, middleware};
use tracing::info;
use tracing_actix_web::TracingLogger;
use std::sync::Arc;

mod api;
mod domain;
mod engine;
mod config;
mod db;

use crate::config::Settings;
use crate::engine::TemplateManager;
use crate::db::{DbPool, TemplateRepository};
use crate::api::middleware::ApiMiddleware;

/// Application state shared across all handlers
pub struct AppState {
    pub settings: Settings,
    pub template_manager: Arc<TemplateManager>,
    pub db_pool: Option<DbPool>,
    pub template_repo: Option<TemplateRepository>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();

    // Initialize tracing subscriber for structured logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("r_image_magic=info".parse().unwrap())
                .add_directive("actix_web=info".parse().unwrap())
        )
        .json()
        .init();

    // Load configuration
    let settings = Settings::load().expect("Failed to load configuration");
    let bind_addr = format!("{}:{}", settings.server.host, settings.server.port);

    info!(
        "Starting R-Image-Magic v{} on {}",
        env!("CARGO_PKG_VERSION"),
        bind_addr
    );

    // Initialize template manager and load templates
    let template_manager = Arc::new(
        TemplateManager::new(&settings.templates.path)
            .expect("Failed to initialize template manager")
    );

    // Load all templates into memory at startup
    template_manager.load_all().await.expect("Failed to load templates");
    info!("Loaded {} templates", template_manager.template_count());

    // Initialize database connection if DATABASE_URL is configured
    let (db_pool, template_repo) = if !settings.database.url.is_empty() {
        match DbPool::new(&settings.database.url) {
            Ok(pool) => {
                // Test the connection
                if let Err(e) = pool.test_connection().await {
                    tracing::warn!("Database connection test failed: {}. Running without database.", e);
                    (None, None)
                } else {
                    let repo = TemplateRepository::new(pool.clone());
                    info!("Database pool initialized successfully");
                    (Some(pool), Some(repo))
                }
            }
            Err(e) => {
                tracing::warn!("Failed to create database pool: {}. Running without database.", e);
                (None, None)
            }
        }
    } else {
        info!("No DATABASE_URL configured, running without database");
        (None, None)
    };

    // Clone pool for middleware and handlers (before moving into AppState)
    let middleware_pool = db_pool.clone();
    let pool_data = db_pool.clone().map(web::Data::new);

    // Create shared application state
    let app_state = web::Data::new(AppState {
        settings: settings.clone(),
        template_manager,
        db_pool,
        template_repo,
    });

    // Configure and start HTTP server
    HttpServer::new(move || {
        let mut app = App::new()
            .app_data(app_state.clone());

        // Add database pool as web::Data if available
        if let Some(ref pool) = pool_data {
            app = app.app_data(pool.clone());
        }

        app
            // API middleware for auth, rate limiting, usage tracking
            // (handles missing DB gracefully by skipping auth)
            .wrap(ApiMiddleware::new(middleware_pool.clone()))
            // Middleware (order matters - these wrap around ApiMiddleware)
            .wrap(TracingLogger::default())
            .wrap(middleware::Compress::default())
            .wrap(
                middleware::DefaultHeaders::new()
                    .add(("X-Service", "r-image-magic"))
                    .add(("X-Version", env!("CARGO_PKG_VERSION")))
            )
            // Routes
            .configure(api::configure_routes)
    })
    .workers(num_cpus::get() * 2) // 2 workers per CPU for async I/O
    .bind(&bind_addr)?
    .run()
    .await
}
