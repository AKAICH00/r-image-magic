//! Printify to Unified Model Mapper
//!
//! Maps Printify API responses to our unified catalog models.

use super::models::*;
use crate::domain::catalog::{
    PrintConstraints, PrintPlacement, ProductType, UnifiedPrintArea, UnifiedProduct,
    UnifiedVariant,
};

/// Mapper for Printify API responses
pub struct PrintifyMapper;

impl PrintifyMapper {
    /// Map Printify blueprint to unified product
    pub fn map_blueprint(blueprint: PrintifyBlueprint) -> UnifiedProduct {
        let product_type = ProductType::from_str(&blueprint.title);
        let category_slug = product_type.category_slug().to_string();
        let metadata = serde_json::to_value(&blueprint).unwrap_or_default();

        UnifiedProduct {
            external_id: blueprint.id.to_string(),
            provider_code: "printify".to_string(),
            name: blueprint.title,
            description: Some(blueprint.description),
            brand: blueprint.brand,
            model: blueprint.model,
            product_type,
            category_slug,
            is_available: true, // Printify blueprints in catalog are available
            regions: vec!["US".to_string(), "EU".to_string()], // Varies by print provider
            base_price_cents: None,
            currency: "USD".to_string(),
            variants: Vec::new(),
            print_areas: Vec::new(),
            provider_metadata: metadata,
        }
    }

    /// Map Printify variant to unified variant
    pub fn map_variant(variant: PrintifyVariant) -> UnifiedVariant {
        let metadata = serde_json::to_value(&variant).unwrap_or_default();

        UnifiedVariant {
            external_id: variant.id.to_string(),
            sku: None, // Printify doesn't expose SKU in catalog
            size: variant.options.size.clone(),
            color_name: variant.options.color.clone(),
            color_hex: None, // Printify doesn't provide hex codes in catalog
            is_available: true,
            price_cents: None, // Price depends on print provider, not available in catalog
            in_stock: true,
            provider_metadata: metadata,
        }
    }

    /// Map Printify placeholder to unified print area
    pub fn map_placeholder(placeholder: PrintifyPlaceholder, position: &str) -> UnifiedPrintArea {
        let placement = PrintPlacement::from_str(position);

        UnifiedPrintArea {
            external_id: None, // Printify placeholders don't have separate IDs
            placement: placement.clone(),
            name: format!("{} Print Area", position),
            width_px: placeholder.width,
            height_px: placeholder.height,
            offset_x_px: 0,
            offset_y_px: 0,
            print_dpi: 300, // Printify default DPI
            file_format: "PNG".to_string(),
            constraints: PrintConstraints {
                max_colors: None,
                technique: Some("DTG".to_string()),
                min_dpi: Some(300),
                max_file_size_mb: Some(200),
                file_formats: vec!["PNG".to_string(), "JPG".to_string()],
            },
        }
    }

    /// Extract all unique print areas from a list of variants
    pub fn extract_print_areas(variants: &[PrintifyVariant]) -> Vec<UnifiedPrintArea> {
        let mut seen_positions = std::collections::HashSet::new();
        let mut areas = Vec::new();

        for variant in variants {
            for placeholder in &variant.placeholders {
                if seen_positions.insert(placeholder.position.clone()) {
                    areas.push(Self::map_placeholder(
                        placeholder.clone(),
                        &placeholder.position,
                    ));
                }
            }
        }

        areas
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_blueprint() {
        let blueprint = PrintifyBlueprint {
            id: 6,
            title: "Unisex Heavy Cotton Tee".to_string(),
            description: "A heavy cotton t-shirt for everyday wear".to_string(),
            brand: Some("Gildan".to_string()),
            model: Some("5000".to_string()),
            images: vec!["https://example.com/image.png".to_string()],
        };

        let unified = PrintifyMapper::map_blueprint(blueprint);

        assert_eq!(unified.external_id, "6");
        assert_eq!(unified.provider_code, "printify");
        assert_eq!(unified.name, "Unisex Heavy Cotton Tee");
        assert_eq!(unified.product_type, ProductType::Tshirt);
        assert_eq!(unified.category_slug, "t-shirts");
        assert!(unified.is_available);
        assert_eq!(unified.brand, Some("Gildan".to_string()));
    }

    #[test]
    fn test_map_variant() {
        let variant = PrintifyVariant {
            id: 17390,
            title: "White / S".to_string(),
            options: PrintifyVariantOptions {
                color: Some("White".to_string()),
                size: Some("S".to_string()),
            },
            placeholders: vec![PrintifyPlaceholder {
                position: "front".to_string(),
                height: 4800,
                width: 4800,
            }],
        };

        let unified = PrintifyMapper::map_variant(variant);

        assert_eq!(unified.external_id, "17390");
        assert_eq!(unified.size, Some("S".to_string()));
        assert_eq!(unified.color_name, Some("White".to_string()));
        assert!(unified.is_available);
        assert!(unified.in_stock);
    }

    #[test]
    fn test_map_placeholder() {
        let placeholder = PrintifyPlaceholder {
            position: "front".to_string(),
            height: 4800,
            width: 4800,
        };

        let area = PrintifyMapper::map_placeholder(placeholder, "front");

        assert_eq!(area.placement, PrintPlacement::Front);
        assert_eq!(area.width_px, 4800);
        assert_eq!(area.height_px, 4800);
        assert_eq!(area.print_dpi, 300);
        assert_eq!(area.name, "front Print Area");
    }

    #[test]
    fn test_extract_print_areas_deduplicates() {
        let variants = vec![
            PrintifyVariant {
                id: 1,
                title: "White / S".to_string(),
                options: PrintifyVariantOptions {
                    color: Some("White".to_string()),
                    size: Some("S".to_string()),
                },
                placeholders: vec![
                    PrintifyPlaceholder {
                        position: "front".to_string(),
                        height: 4800,
                        width: 4800,
                    },
                    PrintifyPlaceholder {
                        position: "back".to_string(),
                        height: 4800,
                        width: 4800,
                    },
                ],
            },
            PrintifyVariant {
                id: 2,
                title: "White / M".to_string(),
                options: PrintifyVariantOptions {
                    color: Some("White".to_string()),
                    size: Some("M".to_string()),
                },
                placeholders: vec![PrintifyPlaceholder {
                    position: "front".to_string(),
                    height: 4800,
                    width: 4800,
                }],
            },
        ];

        let areas = PrintifyMapper::extract_print_areas(&variants);

        // Should deduplicate "front" — only 2 unique positions
        assert_eq!(areas.len(), 2);
        assert_eq!(areas[0].placement, PrintPlacement::Front);
        assert_eq!(areas[1].placement, PrintPlacement::Back);
    }
}
