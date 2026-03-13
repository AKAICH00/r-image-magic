//! Mockup generation engine
//!
//! This module contains the core mockup generation logic including:
//! - Template loading and management
//! - Displacement mapping algorithm
//! - Image compositing pipeline

mod compositor;
mod displacement;
mod template;

pub use compositor::MockupRequest;
pub use template::TemplateManager;
