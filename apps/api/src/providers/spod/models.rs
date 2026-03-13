//! SPOD API Response Models
//!
//! These models represent the JSON responses from the SPOD API.
//! SPOD uses "articles" for products, "appearances" for color variants,
//! and separate endpoints for sizes and print areas.
//!
//! API Documentation: https://docs.spod.com/

use serde::{Deserialize, Serialize};

// ============================================================================
// API Response Wrapper
// ============================================================================

/// Generic SPOD list response wrapper
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpodListResponse<T> {
    pub items: Vec<T>,
    #[serde(default)]
    pub count: Option<i64>,
    #[serde(default)]
    pub offset: Option<i64>,
    #[serde(default)]
    pub limit: Option<i64>,
}

// ============================================================================
// Articles (Products)
// ============================================================================

/// Article (product) from SPOD catalog
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpodArticle {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub brand: Option<SpodBrand>,
    pub article_category: Option<SpodCategory>,
    pub appearances: Option<Vec<SpodAppearance>>,
    pub sizes: Option<Vec<SpodSize>>,
    pub print_areas: Option<Vec<SpodPrintArea>>,
}

/// Brand info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpodBrand {
    pub id: i64,
    pub name: String,
}

/// Article category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpodCategory {
    pub id: i64,
    pub name: String,
}

// ============================================================================
// Appearances (Color Variants)
// ============================================================================

/// Appearance (color variant) for an article
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpodAppearance {
    pub id: i64,
    pub name: String,
    pub hex_color: Option<String>,
    pub images: Option<Vec<SpodImage>>,
}

/// Image associated with an appearance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpodImage {
    pub url: String,
    pub width: Option<i32>,
    pub height: Option<i32>,
}

// ============================================================================
// Sizes
// ============================================================================

/// Size variant for an article
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpodSize {
    pub id: i64,
    pub name: String,
}

// ============================================================================
// Print Areas
// ============================================================================

/// Print area for an article
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpodPrintArea {
    pub id: i64,
    pub name: String,
    pub print_method: Option<String>,
    pub width_mm: Option<f64>,
    pub height_mm: Option<f64>,
    pub dpi: Option<i32>,
}
