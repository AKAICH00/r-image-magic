//! Printful API Response Models
//!
//! These models represent the JSON responses from the Printful API.
//! They are mapped to our unified models in the mapper module.

use serde::{Deserialize, Serialize};

// ============================================================================
// API Response Wrapper
// ============================================================================

/// Generic Printful API response wrapper
#[derive(Debug, Deserialize)]
pub struct PrintfulResponse<T> {
    pub code: i32,
    pub result: T,
    #[serde(default)]
    pub paging: Option<PrintfulPaging>,
}

/// Pagination info
#[derive(Debug, Deserialize)]
pub struct PrintfulPaging {
    pub total: i64,
    pub offset: i64,
    pub limit: i64,
}

// ============================================================================
// Catalog/Products
// ============================================================================

/// Product from catalog list
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrintfulProduct {
    pub id: i64,
    pub main_category_id: Option<i64>,
    pub r#type: String,
    pub type_name: String,
    pub title: String,
    pub brand: Option<String>,
    pub model: Option<String>,
    pub image: Option<String>,
    pub variant_count: i32,
    pub currency: Option<String>,
    pub options: Option<Vec<PrintfulProductOption>>,
    pub is_discontinued: Option<bool>,
    pub description: Option<String>,
}

/// Product option (e.g., color, size)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrintfulProductOption {
    pub id: String,
    pub title: String,
    pub r#type: String,
    pub values: Option<serde_json::Value>,
}

/// Detailed product info (from /products/{id})
#[derive(Debug, Deserialize)]
pub struct PrintfulProductDetail {
    pub product: PrintfulProduct,
    pub variants: Vec<PrintfulVariant>,
}

/// Product variant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrintfulVariant {
    pub id: i64,
    pub product_id: i64,
    pub name: String,
    pub size: Option<String>,
    pub color: Option<String>,
    pub color_code: Option<String>,
    pub color_code2: Option<String>,
    pub image: Option<String>,
    pub price: Option<String>,
    pub in_stock: Option<bool>,
    pub availability_status: Option<String>,
}

// ============================================================================
// Printfiles (Print Areas)
// ============================================================================

/// Printfiles response (from /mockup-generator/printfiles/{id})
#[derive(Debug, Deserialize)]
pub struct PrintfulPrintfilesResponse {
    pub product_id: i64,
    pub available_placements: Option<serde_json::Value>,
    pub printfiles: Vec<PrintfulPrintfile>,
    pub variant_printfiles: Option<Vec<PrintfulVariantPrintfile>>,
}

/// Printfile definition (print area)
#[derive(Debug, Deserialize)]
pub struct PrintfulPrintfile {
    pub printfile_id: i64,
    pub width: i32,
    pub height: i32,
    pub dpi: i32,
    pub fill_mode: String,
    pub can_rotate: Option<bool>,
}

/// Variant-specific printfile
#[derive(Debug, Deserialize)]
pub struct PrintfulVariantPrintfile {
    pub variant_id: i64,
    pub placements: serde_json::Value,
}

// ============================================================================
// Mockup Templates
// ============================================================================

/// Mockup templates response (from /mockup-generator/templates/{id})
#[derive(Debug, Deserialize)]
pub struct PrintfulMockupTemplatesResponse {
    pub product_id: i64,
    pub variant_mapping: Option<Vec<PrintfulVariantMapping>>,
    pub templates: Vec<PrintfulMockupTemplate>,
}

/// Variant to template mapping
#[derive(Debug, Deserialize)]
pub struct PrintfulVariantMapping {
    pub variant_id: i64,
    pub templates: Vec<i64>,
}

/// Mockup template
#[derive(Debug, Clone, Deserialize)]
pub struct PrintfulMockupTemplate {
    pub template_id: i64,
    pub image_url: String,
    pub background_url: Option<String>,
    pub background_color: Option<String>,
    pub printfile_id: i64,
    pub template_positions: Option<PrintfulTemplatePositions>,
    pub template_width: Option<i32>,
    pub template_height: Option<i32>,
}

/// Template print area positions
#[derive(Debug, Clone, Deserialize)]
pub struct PrintfulTemplatePositions {
    pub area_width: i32,
    pub area_height: i32,
    pub width: i32,
    pub height: i32,
    pub top: i32,
    pub left: i32,
}

// ============================================================================
// Categories
// ============================================================================

/// Product category
#[derive(Debug, Deserialize)]
pub struct PrintfulCategory {
    pub id: i64,
    pub parent_id: Option<i64>,
    pub title: String,
    pub image_url: Option<String>,
}

// ============================================================================
// Serialization Helpers
// ============================================================================

impl PrintfulProduct {
    /// Check if product is available
    pub fn is_available(&self) -> bool {
        !self.is_discontinued.unwrap_or(false)
    }
}

impl PrintfulVariant {
    /// Check if variant is in stock
    pub fn is_in_stock(&self) -> bool {
        self.in_stock.unwrap_or(true)
    }

    /// Parse price as cents
    pub fn price_cents(&self) -> Option<i32> {
        self.price.as_ref().and_then(|p| {
            p.parse::<f64>().ok().map(|f| (f * 100.0) as i32)
        })
    }
}
