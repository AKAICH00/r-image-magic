//! Template management and loading

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use parking_lot::RwLock;
use image::{DynamicImage, ImageError};
use serde::Deserialize;
use thiserror::Error;
use tracing::{info, warn};

use super::compositor::{MockupRequest, MockupResult, Compositor};

/// Template-related errors
#[derive(Debug, Error)]
pub enum TemplateError {
    #[error("Template not found: {0}")]
    NotFound(String),
    #[error("Failed to load template image: {0}")]
    ImageLoad(#[from] ImageError),
    #[error("Failed to load metadata: {0}")]
    MetadataLoad(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Template metadata loaded from metadata.json
#[derive(Debug, Clone, Deserialize)]
pub struct TemplateMetadata {
    pub id: String,
    pub version: u32,
    pub category: String,
    pub color: String,
    pub color_hex: String,
    pub placement: String,
    pub gender: String,
    pub dimensions: TemplateDimensions,
    pub print_area: PrintArea,
    pub anchor_point: AnchorPoint,
    pub displacement: DisplacementConfig,
    pub blend_mode: String,
    pub default_opacity: u8,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TemplateDimensions {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PrintArea {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AnchorPoint {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DisplacementConfig {
    pub enabled: bool,
    pub strength_default: f64,
    pub strength_range: (f64, f64),
}

/// A loaded template with all assets in memory
pub struct Template {
    pub metadata: TemplateMetadata,
    pub base_image: DynamicImage,
    pub displacement_map: Option<DynamicImage>,
}

impl Template {
    /// Load a template from a directory
    pub fn load(path: &Path) -> Result<Self, TemplateError> {
        // Load metadata
        let metadata_path = path.join("metadata.json");
        let metadata_content = std::fs::read_to_string(&metadata_path)
            .map_err(|e| TemplateError::MetadataLoad(format!("{}: {}", metadata_path.display(), e)))?;
        let metadata: TemplateMetadata = serde_json::from_str(&metadata_content)?;

        // Load base image
        let base_path = path.join("base.png");
        let base_image = if base_path.exists() {
            image::open(&base_path)?
        } else {
            // Try jpg as fallback
            let jpg_path = path.join("base.jpg");
            image::open(&jpg_path)?
        };

        // Load displacement map (optional)
        let displacement_map = {
            let disp_path = path.join("displacement.png");
            if disp_path.exists() {
                Some(image::open(&disp_path)?)
            } else {
                let jpg_path = path.join("displacement.jpg");
                if jpg_path.exists() {
                    Some(image::open(&jpg_path)?)
                } else {
                    warn!("No displacement map found for template {}", metadata.id);
                    None
                }
            }
        };

        info!(
            id = %metadata.id,
            dimensions = ?metadata.dimensions,
            has_displacement = displacement_map.is_some(),
            "Loaded template"
        );

        Ok(Template {
            metadata,
            base_image,
            displacement_map,
        })
    }
}

/// Manages all templates in memory
pub struct TemplateManager {
    templates: RwLock<HashMap<String, Arc<Template>>>,
    base_path: PathBuf,
    compositor: Compositor,
}

impl TemplateManager {
    /// Create a new template manager
    pub fn new(base_path: &Path) -> Result<Self, TemplateError> {
        Ok(TemplateManager {
            templates: RwLock::new(HashMap::new()),
            base_path: base_path.to_path_buf(),
            compositor: Compositor::new(),
        })
    }

    /// Load all templates from the base directory
    pub async fn load_all(&self) -> Result<(), TemplateError> {
        let base_path = self.base_path.clone();

        // Spawn blocking task for file I/O
        let templates = tokio::task::spawn_blocking(move || {
            let mut loaded = HashMap::new();

            if !base_path.exists() {
                warn!("Templates directory does not exist: {}", base_path.display());
                return Ok(loaded);
            }

            for entry in std::fs::read_dir(&base_path)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_dir() {
                    // Check if this looks like a template directory
                    let metadata_path = path.join("metadata.json");
                    if metadata_path.exists() {
                        match Template::load(&path) {
                            Ok(template) => {
                                let id = template.metadata.id.clone();
                                loaded.insert(id, Arc::new(template));
                            }
                            Err(e) => {
                                warn!(
                                    path = %path.display(),
                                    error = %e,
                                    "Failed to load template"
                                );
                            }
                        }
                    }
                }
            }

            Ok::<_, TemplateError>(loaded)
        })
        .await
        .map_err(|e| TemplateError::MetadataLoad(format!("Task join error: {}", e)))??;

        // Update templates map
        let mut guard = self.templates.write();
        *guard = templates;

        Ok(())
    }

    /// Get a template by ID
    pub fn get(&self, id: &str) -> Option<Arc<Template>> {
        self.templates.read().get(id).cloned()
    }

    /// Get the number of loaded templates
    pub fn template_count(&self) -> usize {
        self.templates.read().len()
    }

    /// List all template IDs
    pub fn list_ids(&self) -> Vec<String> {
        self.templates.read().keys().cloned().collect()
    }

    /// Generate a mockup using the compositor
    pub async fn generate_mockup(&self, request: &MockupRequest) -> Result<MockupResult, TemplateError> {
        let template = self.get(&request.template_id)
            .ok_or_else(|| TemplateError::NotFound(request.template_id.clone()))?;

        self.compositor.generate(request, &template).await
            .map_err(|e| TemplateError::MetadataLoad(format!("Compositor error: {}", e)))
    }
}
