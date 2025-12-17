//! Database models for template data

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// Template record from the database
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DbTemplate {
    pub id: Uuid,
    pub template_id: String,
    pub name: String,
    pub description: Option<String>,
    pub product_type: String,
    pub variant: Option<String>,
    pub color: Option<String>,
    pub print_area_x: f64,
    pub print_area_y: f64,
    pub print_area_width: f64,
    pub print_area_height: f64,
    pub base_image_path: String,
    pub displacement_map_path: Option<String>,
    pub mask_path: Option<String>,
    pub width: i32,
    pub height: i32,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Simplified template info for API responses
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TemplateInfo {
    pub template_id: String,
    pub name: String,
    pub description: Option<String>,
    pub product_type: String,
    pub variant: Option<String>,
    pub color: Option<String>,
    pub print_area: PrintAreaInfo,
    pub dimensions: DimensionsInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PrintAreaInfo {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DimensionsInfo {
    pub width: i32,
    pub height: i32,
}

impl From<DbTemplate> for TemplateInfo {
    fn from(t: DbTemplate) -> Self {
        TemplateInfo {
            template_id: t.template_id,
            name: t.name,
            description: t.description,
            product_type: t.product_type,
            variant: t.variant,
            color: t.color,
            print_area: PrintAreaInfo {
                x: t.print_area_x,
                y: t.print_area_y,
                width: t.print_area_width,
                height: t.print_area_height,
            },
            dimensions: DimensionsInfo {
                width: t.width,
                height: t.height,
            },
        }
    }
}
