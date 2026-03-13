//! Gelato API Response Models
//!
//! These models represent the JSON responses from the Gelato API.
//! They are mapped to our unified models in the mapper module.
//!
//! Gelato API uses camelCase for JSON fields.
//! Auth: X-API-KEY header
//! Rate limit: 300 req/min

use serde::{Deserialize, Serialize};

// ============================================================================
// Catalog
// ============================================================================

/// Catalog from Gelato catalog list
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GelatoCatalog {
    pub catalog_uid: String,
    pub title: String,
    pub product_count: Option<i32>,
}

// ============================================================================
// Products
// ============================================================================

/// Product from Gelato API
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GelatoProduct {
    pub product_uid: String,
    pub title: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub variants: Option<Vec<GelatoVariant>>,
    pub print_areas: Option<Vec<GelatoPrintArea>>,
}

/// Product variant from Gelato API
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GelatoVariant {
    pub variant_uid: String,
    pub title: String,
    pub product_uid: String,
    pub color: Option<String>,
    pub color_hex: Option<String>,
    pub size: Option<String>,
    pub price: Option<GelatoPrice>,
    pub is_active: Option<bool>,
}

/// Price from Gelato API
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GelatoPrice {
    pub amount: f64,
    pub currency: String,
}

// ============================================================================
// Print Areas
// ============================================================================

/// Print area from Gelato API
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GelatoPrintArea {
    pub position: String,
    pub width_mm: Option<f64>,
    pub height_mm: Option<f64>,
    pub dpi: Option<i32>,
}

// ============================================================================
// Generic Response Wrapper
// ============================================================================

/// Generic list response from Gelato API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GelatoListResponse<T> {
    pub data: Vec<T>,
    #[serde(default)]
    pub pagination: Option<GelatoPagination>,
}

/// Pagination info from Gelato API
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GelatoPagination {
    pub total: i64,
    pub page: i32,
    pub page_size: i32,
}
