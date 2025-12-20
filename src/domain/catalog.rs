//! Unified Catalog Domain Models
//!
//! This module defines provider-agnostic domain models for POD products.
//! These models normalize data from different providers (Printful, Printify, etc.)
//! into a consistent internal format.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================================
// Product Types
// ============================================================================

/// Product type enumeration (unified across providers)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProductType {
    Tshirt,
    Hoodie,
    TankTop,
    LongSleeve,
    Sweatshirt,
    Mug,
    Poster,
    Canvas,
    PhoneCase,
    Bag,
    ToteBag,
    Hat,
    Cap,
    Beanie,
    Sticker,
    Other(String),
}

impl ProductType {
    /// Parse product type from a string (case-insensitive, fuzzy match)
    pub fn from_str(s: &str) -> Self {
        let lower = s.to_lowercase();

        if lower.contains("t-shirt") || lower.contains("tshirt") || lower.contains("tee") {
            ProductType::Tshirt
        } else if lower.contains("hoodie") || lower.contains("hooded") {
            ProductType::Hoodie
        } else if lower.contains("tank") {
            ProductType::TankTop
        } else if lower.contains("long sleeve") || lower.contains("longsleeve") {
            ProductType::LongSleeve
        } else if lower.contains("sweatshirt") || lower.contains("crew neck") {
            ProductType::Sweatshirt
        } else if lower.contains("mug") || lower.contains("cup") {
            ProductType::Mug
        } else if lower.contains("poster") || lower.contains("print") {
            ProductType::Poster
        } else if lower.contains("canvas") {
            ProductType::Canvas
        } else if lower.contains("phone") || lower.contains("case") {
            ProductType::PhoneCase
        } else if lower.contains("tote") {
            ProductType::ToteBag
        } else if lower.contains("bag") || lower.contains("backpack") {
            ProductType::Bag
        } else if lower.contains("beanie") {
            ProductType::Beanie
        } else if lower.contains("cap") {
            ProductType::Cap
        } else if lower.contains("hat") {
            ProductType::Hat
        } else if lower.contains("sticker") {
            ProductType::Sticker
        } else {
            ProductType::Other(s.to_string())
        }
    }

    /// Get the category slug for this product type
    pub fn category_slug(&self) -> &str {
        match self {
            ProductType::Tshirt => "t-shirts",
            ProductType::Hoodie | ProductType::Sweatshirt => "hoodies",
            ProductType::TankTop => "tank-tops",
            ProductType::LongSleeve => "long-sleeves",
            ProductType::Mug => "mugs",
            ProductType::Poster | ProductType::Canvas => "posters",
            ProductType::PhoneCase => "phone-cases",
            ProductType::Bag | ProductType::ToteBag => "bags",
            ProductType::Hat | ProductType::Cap | ProductType::Beanie => "hats",
            ProductType::Sticker => "accessories",
            ProductType::Other(_) => "accessories",
        }
    }
}

impl std::fmt::Display for ProductType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProductType::Tshirt => write!(f, "tshirt"),
            ProductType::Hoodie => write!(f, "hoodie"),
            ProductType::TankTop => write!(f, "tank_top"),
            ProductType::LongSleeve => write!(f, "long_sleeve"),
            ProductType::Sweatshirt => write!(f, "sweatshirt"),
            ProductType::Mug => write!(f, "mug"),
            ProductType::Poster => write!(f, "poster"),
            ProductType::Canvas => write!(f, "canvas"),
            ProductType::PhoneCase => write!(f, "phone_case"),
            ProductType::Bag => write!(f, "bag"),
            ProductType::ToteBag => write!(f, "tote_bag"),
            ProductType::Hat => write!(f, "hat"),
            ProductType::Cap => write!(f, "cap"),
            ProductType::Beanie => write!(f, "beanie"),
            ProductType::Sticker => write!(f, "sticker"),
            ProductType::Other(s) => write!(f, "{}", s),
        }
    }
}

// ============================================================================
// Print Placement Types
// ============================================================================

/// Print placement types (unified across providers)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrintPlacement {
    Front,
    Back,
    SleeveLeft,
    SleeveRight,
    Pocket,
    Hood,
    FullWrap,
    AllOver,
    Other(String),
}

impl PrintPlacement {
    /// Parse placement from a string
    pub fn from_str(s: &str) -> Self {
        let lower = s.to_lowercase();

        if lower.contains("front") {
            PrintPlacement::Front
        } else if lower.contains("back") {
            PrintPlacement::Back
        } else if lower.contains("left") && lower.contains("sleeve") {
            PrintPlacement::SleeveLeft
        } else if lower.contains("right") && lower.contains("sleeve") {
            PrintPlacement::SleeveRight
        } else if lower.contains("pocket") {
            PrintPlacement::Pocket
        } else if lower.contains("hood") {
            PrintPlacement::Hood
        } else if lower.contains("wrap") {
            PrintPlacement::FullWrap
        } else if lower.contains("all") && lower.contains("over") {
            PrintPlacement::AllOver
        } else {
            PrintPlacement::Other(s.to_string())
        }
    }

    /// Get the string representation of the placement
    pub fn as_str(&self) -> &str {
        match self {
            PrintPlacement::Front => "front",
            PrintPlacement::Back => "back",
            PrintPlacement::SleeveLeft => "sleeve_left",
            PrintPlacement::SleeveRight => "sleeve_right",
            PrintPlacement::Pocket => "pocket",
            PrintPlacement::Hood => "hood",
            PrintPlacement::FullWrap => "full_wrap",
            PrintPlacement::AllOver => "all_over",
            PrintPlacement::Other(s) => s.as_str(),
        }
    }
}

impl std::fmt::Display for PrintPlacement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PrintPlacement::Front => write!(f, "front"),
            PrintPlacement::Back => write!(f, "back"),
            PrintPlacement::SleeveLeft => write!(f, "sleeve_left"),
            PrintPlacement::SleeveRight => write!(f, "sleeve_right"),
            PrintPlacement::Pocket => write!(f, "pocket"),
            PrintPlacement::Hood => write!(f, "hood"),
            PrintPlacement::FullWrap => write!(f, "full_wrap"),
            PrintPlacement::AllOver => write!(f, "all_over"),
            PrintPlacement::Other(s) => write!(f, "{}", s),
        }
    }
}

// ============================================================================
// Unified Product
// ============================================================================

/// Unified product representation across all providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedProduct {
    /// Provider's product ID
    pub external_id: String,

    /// Provider code (e.g., "printful", "printify")
    pub provider_code: String,

    /// Product name
    pub name: String,

    /// Product description
    pub description: Option<String>,

    /// Brand name (e.g., "Bella + Canvas", "Gildan")
    pub brand: Option<String>,

    /// Model number/name
    pub model: Option<String>,

    /// Product type classification
    pub product_type: ProductType,

    /// Category slug (derived from product_type)
    pub category_slug: String,

    /// Is the product currently available
    pub is_available: bool,

    /// Available regions (e.g., ["US", "EU", "ASIA"])
    pub regions: Vec<String>,

    /// Base price in cents (USD)
    pub base_price_cents: Option<i32>,

    /// Currency code
    pub currency: String,

    /// Product variants (size/color combinations)
    pub variants: Vec<UnifiedVariant>,

    /// Print areas (placements where designs can be applied)
    pub print_areas: Vec<UnifiedPrintArea>,

    /// Raw metadata from provider (for debugging/extension)
    pub provider_metadata: serde_json::Value,
}

impl UnifiedProduct {
    /// Create a new unified product with minimal required fields
    pub fn new(external_id: String, provider_code: String, name: String, product_type: ProductType) -> Self {
        let category_slug = product_type.category_slug().to_string();
        UnifiedProduct {
            external_id,
            provider_code,
            name,
            description: None,
            brand: None,
            model: None,
            product_type,
            category_slug,
            is_available: true,
            regions: vec!["US".to_string()],
            base_price_cents: None,
            currency: "USD".to_string(),
            variants: Vec::new(),
            print_areas: Vec::new(),
            provider_metadata: serde_json::Value::Null,
        }
    }
}

// ============================================================================
// Unified Variant
// ============================================================================

/// Unified variant representation (size/color combination)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedVariant {
    /// Provider's variant ID
    pub external_id: String,

    /// SKU (stock keeping unit)
    pub sku: Option<String>,

    /// Size (e.g., "S", "M", "L", "XL")
    pub size: Option<String>,

    /// Color name (e.g., "White", "Black", "Navy")
    pub color_name: Option<String>,

    /// Color hex code (e.g., "#FFFFFF")
    pub color_hex: Option<String>,

    /// Is the variant currently available
    pub is_available: bool,

    /// Price in cents (USD) - may differ from base price
    pub price_cents: Option<i32>,

    /// Is currently in stock
    pub in_stock: bool,

    /// Raw metadata from provider
    pub provider_metadata: serde_json::Value,
}

impl UnifiedVariant {
    /// Create a new unified variant with minimal required fields
    pub fn new(external_id: String) -> Self {
        UnifiedVariant {
            external_id,
            sku: None,
            size: None,
            color_name: None,
            color_hex: None,
            is_available: true,
            price_cents: None,
            in_stock: true,
            provider_metadata: serde_json::Value::Null,
        }
    }
}

// ============================================================================
// Unified Print Area
// ============================================================================

/// Print constraints (technique-specific requirements)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PrintConstraints {
    /// Maximum number of colors (for screen printing)
    pub max_colors: Option<i32>,

    /// Printing technique (e.g., "DTG", "screen_print", "sublimation", "embroidery")
    pub technique: Option<String>,

    /// Minimum DPI requirement
    pub min_dpi: Option<i32>,

    /// Maximum file size in MB
    pub max_file_size_mb: Option<i32>,

    /// Supported file formats
    pub file_formats: Vec<String>,
}

/// Unified print area representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedPrintArea {
    /// Provider's print area ID (if available)
    pub external_id: Option<String>,

    /// Placement type (front, back, etc.)
    pub placement: PrintPlacement,

    /// Display name (e.g., "Front Print", "Back Full")
    pub name: String,

    /// Width in pixels
    pub width_px: i32,

    /// Height in pixels
    pub height_px: i32,

    /// X offset from template origin (if known)
    pub offset_x_px: i32,

    /// Y offset from template origin (if known)
    pub offset_y_px: i32,

    /// Required print DPI
    pub print_dpi: i32,

    /// Preferred file format
    pub file_format: String,

    /// Print constraints
    pub constraints: PrintConstraints,
}

impl UnifiedPrintArea {
    /// Create a new print area with minimal required fields
    pub fn new(placement: PrintPlacement, name: String, width_px: i32, height_px: i32) -> Self {
        UnifiedPrintArea {
            external_id: None,
            placement,
            name,
            width_px,
            height_px,
            offset_x_px: 0,
            offset_y_px: 0,
            print_dpi: 300,
            file_format: "PNG".to_string(),
            constraints: PrintConstraints::default(),
        }
    }
}

// ============================================================================
// Mockup Asset
// ============================================================================

/// Asset types for mockups
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetType {
    /// Base template image (background/product image)
    BaseImage,
    /// Mockup template with transparent print area
    MockupTemplate,
    /// Preview of print file positioning
    PrintfilePreview,
    /// Small thumbnail image
    Thumbnail,
}

impl std::fmt::Display for AssetType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AssetType::BaseImage => write!(f, "base_image"),
            AssetType::MockupTemplate => write!(f, "mockup_template"),
            AssetType::PrintfilePreview => write!(f, "printfile_preview"),
            AssetType::Thumbnail => write!(f, "thumbnail"),
        }
    }
}

/// Mockup asset from provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockupAsset {
    /// Type of asset
    pub asset_type: AssetType,

    /// Placement (if specific to a print area)
    pub placement: Option<PrintPlacement>,

    /// Source URL from provider
    pub source_url: String,

    /// Width in pixels
    pub width_px: Option<i32>,

    /// Height in pixels
    pub height_px: Option<i32>,

    /// Associated variant ID (for variant-specific mockups)
    pub variant_external_id: Option<String>,
}

impl MockupAsset {
    /// Create a new mockup asset
    pub fn new(asset_type: AssetType, source_url: String) -> Self {
        MockupAsset {
            asset_type,
            placement: None,
            source_url,
            width_px: None,
            height_px: None,
            variant_external_id: None,
        }
    }
}

// ============================================================================
// Database Models (for storing in PostgreSQL)
// ============================================================================

/// Database representation of a POD provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbPodProvider {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub api_base_url: String,
    pub auth_type: String,
    pub rate_limit_per_minute: i32,
    pub is_active: bool,
    pub sync_enabled: bool,
    pub last_sync_at: Option<DateTime<Utc>>,
    pub sync_interval_hours: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Database representation of a POD product
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbPodProduct {
    pub id: Uuid,
    pub provider_id: Uuid,
    pub external_product_id: String,
    pub category_id: Option<Uuid>,
    pub name: String,
    pub description: Option<String>,
    pub brand: Option<String>,
    pub model: Option<String>,
    pub product_type: String,
    pub is_available: bool,
    pub regions: serde_json::Value,
    pub base_price_cents: Option<i32>,
    pub currency: String,
    pub provider_metadata: serde_json::Value,
    pub last_synced_at: DateTime<Utc>,
    pub sync_hash: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Database representation of a product variant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbPodProductVariant {
    pub id: Uuid,
    pub product_id: Uuid,
    pub external_variant_id: String,
    pub sku: Option<String>,
    pub size: Option<String>,
    pub color_name: Option<String>,
    pub color_hex: Option<String>,
    pub is_available: bool,
    pub price_cents: Option<i32>,
    pub in_stock: bool,
    pub provider_metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Database representation of a print area
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbPodPrintArea {
    pub id: Uuid,
    pub product_id: Uuid,
    pub external_print_area_id: Option<String>,
    pub placement: String,
    pub name: String,
    pub width_px: i32,
    pub height_px: i32,
    pub offset_x_px: i32,
    pub offset_y_px: i32,
    pub print_dpi: i32,
    pub file_format: String,
    pub constraints: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

/// Database representation of a mockup asset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbPodMockupAsset {
    pub id: Uuid,
    pub product_id: Uuid,
    pub variant_id: Option<Uuid>,
    pub asset_type: String,
    pub placement: Option<String>,
    pub source_url: String,
    pub r2_bucket: Option<String>,
    pub r2_key: Option<String>,
    pub width_px: Option<i32>,
    pub height_px: Option<i32>,
    pub file_size_bytes: Option<i64>,
    pub content_type: Option<String>,
    pub checksum: Option<String>,
    pub status: String,
    pub error_message: Option<String>,
    pub retry_count: i32,
    pub downloaded_at: Option<DateTime<Utc>>,
    pub processed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Database representation of a sync job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbPodSyncJob {
    pub id: Uuid,
    pub provider_id: Uuid,
    pub job_type: String,
    pub status: String,
    pub total_items: i32,
    pub processed_items: i32,
    pub failed_items: i32,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    pub error_details: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_product_type_from_str() {
        assert_eq!(ProductType::from_str("T-Shirt"), ProductType::Tshirt);
        assert_eq!(ProductType::from_str("hoodie"), ProductType::Hoodie);
        assert_eq!(ProductType::from_str("Coffee Mug"), ProductType::Mug);
        assert!(matches!(ProductType::from_str("Unknown Item"), ProductType::Other(_)));
    }

    #[test]
    fn test_print_placement_from_str() {
        assert_eq!(PrintPlacement::from_str("Front"), PrintPlacement::Front);
        assert_eq!(PrintPlacement::from_str("Left Sleeve"), PrintPlacement::SleeveLeft);
    }

    #[test]
    fn test_product_type_category_slug() {
        assert_eq!(ProductType::Tshirt.category_slug(), "t-shirts");
        assert_eq!(ProductType::Hoodie.category_slug(), "hoodies");
        assert_eq!(ProductType::Mug.category_slug(), "mugs");
    }
}
