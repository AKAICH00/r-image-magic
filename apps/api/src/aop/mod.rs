pub mod artwork;
pub mod collar;
pub mod config;
pub mod debug;
pub mod displacement;
pub mod garment;
pub mod lighting;
pub mod panels;
pub mod render;
pub mod seams;
pub mod uv;
pub mod validation;

pub use config::{AopRenderConfig, FabricProfile, PrintMode};
pub use debug::DebugOverlayArtifact;
pub use garment::AopTemplateMetadata;
pub use render::{AopRenderError, AopRenderer};
pub use validation::ValidationIssue;
