//! Printify API Response Models
//!
//! These models represent the JSON responses from the Printify API (v1).
//! They are mapped to our unified models in the mapper module.
//!
//! Key differences from Printful:
//! - Products are called "blueprints"
//! - Variants come through "print providers" (a blueprint can have multiple)
//! - No pagination wrapper — responses are flat arrays
//! - Rate limit: 600 req/min

use serde::{Deserialize, Serialize};

// ============================================================================
// Catalog / Blueprints
// ============================================================================

/// Blueprint from catalog list (Printify's term for a product)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrintifyBlueprint {
    pub id: i64,
    pub title: String,
    pub description: String,
    pub brand: Option<String>,
    pub model: Option<String>,
    pub images: Vec<String>,
}

// ============================================================================
// Print Providers
// ============================================================================

/// Print provider for a blueprint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrintifyPrintProvider {
    pub id: i64,
    pub title: String,
    pub location: Option<PrintifyLocation>,
}

/// Print provider location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrintifyLocation {
    pub address1: Option<String>,
    pub city: Option<String>,
    pub country: Option<String>,
}

// ============================================================================
// Variants
// ============================================================================

/// Variant for a blueprint + print provider combination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrintifyVariantResponse {
    pub id: i64,
    pub title: String,
    pub variants: Vec<PrintifyVariant>,
}

/// Individual variant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrintifyVariant {
    pub id: i64,
    pub title: String,
    pub options: PrintifyVariantOptions,
    pub placeholders: Vec<PrintifyPlaceholder>,
}

/// Variant options (color, size)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrintifyVariantOptions {
    pub color: Option<String>,
    pub size: Option<String>,
}

// ============================================================================
// Placeholders (Print Areas)
// ============================================================================

/// Placeholder defines a printable area on a variant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrintifyPlaceholder {
    pub position: String,
    pub height: i32,
    pub width: i32,
}

// ============================================================================
// Shops
// ============================================================================

/// Shop from GET /v1/shops.json (used for auth verification)
#[derive(Debug, Clone, Deserialize)]
pub struct PrintifyShop {
    pub id: i64,
    pub title: String,
    pub sales_channel: String,
}
