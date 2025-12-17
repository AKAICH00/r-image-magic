//! Database module for PostgreSQL connectivity
//!
//! Provides connection pool management and template queries for the r_image_magic database.

pub mod pool;
pub mod models;
pub mod queries;

pub use pool::DbPool;
pub use queries::TemplateRepository;
