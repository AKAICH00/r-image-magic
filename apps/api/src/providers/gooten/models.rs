//! Gooten API Response Models
//!
//! These models represent the JSON responses from the Gooten API.
//! They are mapped to our unified models in the mapper module.
//!
//! Gooten uses PascalCase for JSON field names.

use serde::{Deserialize, Serialize};

// ============================================================================
// Products
// ============================================================================

/// Product from Gooten catalog
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GootenProduct {
    pub id: i64,
    pub name: String,
    pub short_description: Option<String>,
    pub categories: Option<Vec<GootenCategory>>,
    pub images: Option<Vec<GootenImage>>,
    pub has_available_products: Option<bool>,
    pub max_zone_count: Option<i32>,
}

/// Product category
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GootenCategory {
    pub id: i64,
    pub name: String,
}

/// Product image
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GootenImage {
    pub url: String,
    pub index: Option<i32>,
    pub id: Option<String>,
    pub description: Option<String>,
}

// ============================================================================
// Variants
// ============================================================================

/// Product variant
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GootenVariant {
    pub sku: String,
    pub product_id: i64,
    pub max_images: Option<i32>,
    pub has_templates: Option<bool>,
    pub options: Option<Vec<GootenOption>>,
    pub price_info: Option<GootenPriceInfo>,
}

/// Variant option (e.g., color, size)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GootenOption {
    pub name: String,
    pub value: String,
}

/// Variant price info
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GootenPriceInfo {
    pub price: Option<f64>,
    pub currency_code: Option<String>,
}

// ============================================================================
// Templates (Mockups / Print Areas)
// ============================================================================

/// Mockup template
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GootenTemplate {
    pub sku: String,
    pub images: Vec<GootenTemplateImage>,
}

/// Template image
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GootenTemplateImage {
    pub url: String,
    pub index: Option<i32>,
    pub is_default: Option<bool>,
    pub layers: Option<Vec<GootenTemplateLayer>>,
}

/// Template layer (print area definition)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GootenTemplateLayer {
    pub name: Option<String>,
    pub x1: Option<f64>,
    pub y1: Option<f64>,
    pub x2: Option<f64>,
    pub y2: Option<f64>,
}

// ============================================================================
// API Response Wrappers
// ============================================================================

/// Products list response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GootenProductsResponse {
    pub products: Vec<GootenProduct>,
}

/// Variants list response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GootenVariantsResponse {
    pub product_variants: Vec<GootenVariant>,
}

/// Templates response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct GootenTemplatesResponse {
    pub options: Vec<GootenTemplate>,
}

// ============================================================================
// Helper Methods
// ============================================================================

impl GootenVariant {
    /// Extract a named option value (case-insensitive)
    pub fn option_value(&self, name: &str) -> Option<String> {
        self.options.as_ref().and_then(|opts| {
            opts.iter()
                .find(|o| o.name.eq_ignore_ascii_case(name))
                .map(|o| o.value.clone())
        })
    }

    /// Parse price as cents (USD)
    pub fn price_cents(&self) -> Option<i32> {
        self.price_info
            .as_ref()
            .and_then(|pi| pi.price.map(|p| (p * 100.0) as i32))
    }
}

impl GootenTemplateLayer {
    /// Calculate width from x1/x2 coordinates
    pub fn width(&self) -> Option<i32> {
        match (self.x1, self.x2) {
            (Some(x1), Some(x2)) => Some((x2 - x1).abs() as i32),
            _ => None,
        }
    }

    /// Calculate height from y1/y2 coordinates
    pub fn height(&self) -> Option<i32> {
        match (self.y1, self.y2) {
            (Some(y1), Some(y2)) => Some((y2 - y1).abs() as i32),
            _ => None,
        }
    }
}
