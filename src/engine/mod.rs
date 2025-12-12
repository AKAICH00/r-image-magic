//! Mockup generation engine
//!
//! This module contains the core mockup generation logic including:
//! - Template loading and management
//! - Displacement mapping algorithm
//! - Image compositing pipeline

mod template;
mod compositor;
mod displacement;

pub use template::TemplateManager;
pub use compositor::MockupRequest;
