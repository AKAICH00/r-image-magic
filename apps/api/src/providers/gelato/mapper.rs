//! Gelato to Unified Model Mapper
//!
//! Maps Gelato API responses to our unified catalog models.

use super::models::*;
use crate::domain::catalog::{
    PrintConstraints, PrintPlacement, ProductType, UnifiedPrintArea, UnifiedProduct,
    UnifiedVariant,
};

/// Mapper for Gelato API responses
pub struct GelatoMapper;

impl GelatoMapper {
    /// Map Gelato product to unified product
    pub fn map_product(product: GelatoProduct) -> UnifiedProduct {
        let product_type = product
            .category
            .as_deref()
            .map(ProductType::from_str)
            .unwrap_or_else(|| ProductType::from_str(&product.title));
        let category_slug = product_type.category_slug().to_string();
        let metadata = serde_json::to_value(&product).unwrap_or_default();

        let variants: Vec<UnifiedVariant> = product
            .variants
            .as_deref()
            .unwrap_or_default()
            .iter()
            .cloned()
            .map(Self::map_variant)
            .collect();

        let print_areas: Vec<UnifiedPrintArea> = product
            .print_areas
            .as_deref()
            .unwrap_or_default()
            .iter()
            .cloned()
            .map(Self::map_print_area)
            .collect();

        let base_price_cents = variants.first().and_then(|v| v.price_cents);

        UnifiedProduct {
            external_id: product.product_uid,
            provider_code: "gelato".to_string(),
            name: product.title,
            description: product.description,
            brand: None, // Gelato doesn't expose brand separately
            model: None,
            product_type,
            category_slug,
            is_available: true,
            regions: vec!["US".to_string(), "EU".to_string()], // Gelato ships globally
            base_price_cents,
            currency: variants
                .first()
                .and_then(|_| {
                    // Extract currency from the original variant data
                    product
                        .variants
                        .as_ref()
                        .and_then(|vs| vs.first())
                        .and_then(|v| v.price.as_ref())
                        .map(|p| p.currency.clone())
                })
                .unwrap_or_else(|| "USD".to_string()),
            variants,
            print_areas,
            provider_metadata: metadata,
        }
    }

    /// Map Gelato variant to unified variant
    pub fn map_variant(variant: GelatoVariant) -> UnifiedVariant {
        let price_cents = variant
            .price
            .as_ref()
            .map(|p| (p.amount * 100.0) as i32);

        let is_active = variant.is_active.unwrap_or(true);
        let metadata = serde_json::to_value(&variant).unwrap_or_default();

        UnifiedVariant {
            external_id: variant.variant_uid,
            sku: None, // Gelato doesn't expose SKU in catalog
            size: variant.size,
            color_name: variant.color,
            color_hex: variant.color_hex,
            is_available: is_active,
            price_cents,
            in_stock: is_active,
            provider_metadata: metadata,
        }
    }

    /// Map Gelato print area to unified print area
    ///
    /// Converts mm dimensions to pixels at the given DPI (default 300).
    /// Formula: px = mm * dpi / 25.4
    pub fn map_print_area(area: GelatoPrintArea) -> UnifiedPrintArea {
        let dpi = area.dpi.unwrap_or(300);
        let width_px = area
            .width_mm
            .map(|mm| (mm * dpi as f64 / 25.4).round() as i32)
            .unwrap_or(0);
        let height_px = area
            .height_mm
            .map(|mm| (mm * dpi as f64 / 25.4).round() as i32)
            .unwrap_or(0);

        let placement = PrintPlacement::from_str(&area.position);

        UnifiedPrintArea {
            external_id: None,
            placement: placement.clone(),
            name: format!("{} Print Area", area.position),
            width_px,
            height_px,
            offset_x_px: 0,
            offset_y_px: 0,
            print_dpi: dpi,
            file_format: "PNG".to_string(),
            constraints: PrintConstraints {
                max_colors: None,
                technique: Some("DTG".to_string()),
                min_dpi: Some(dpi),
                max_file_size_mb: Some(200),
                file_formats: vec!["PNG".to_string(), "JPG".to_string()],
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_product() {
        let product = GelatoProduct {
            product_uid: "gelato-prod-001".to_string(),
            title: "Classic T-Shirt".to_string(),
            description: Some("A classic unisex tee".to_string()),
            category: Some("T-Shirt".to_string()),
            variants: None,
            print_areas: None,
        };

        let unified = GelatoMapper::map_product(product);

        assert_eq!(unified.external_id, "gelato-prod-001");
        assert_eq!(unified.provider_code, "gelato");
        assert_eq!(unified.name, "Classic T-Shirt");
        assert_eq!(unified.product_type, ProductType::Tshirt);
        assert!(unified.is_available);
        assert_eq!(unified.currency, "USD");
    }

    #[test]
    fn test_map_product_with_variants() {
        let product = GelatoProduct {
            product_uid: "gelato-prod-002".to_string(),
            title: "Premium Hoodie".to_string(),
            description: None,
            category: Some("Hoodie".to_string()),
            variants: Some(vec![GelatoVariant {
                variant_uid: "var-001".to_string(),
                title: "Black / M".to_string(),
                product_uid: "gelato-prod-002".to_string(),
                color: Some("Black".to_string()),
                color_hex: Some("#000000".to_string()),
                size: Some("M".to_string()),
                price: Some(GelatoPrice {
                    amount: 25.99,
                    currency: "USD".to_string(),
                }),
                is_active: Some(true),
            }]),
            print_areas: None,
        };

        let unified = GelatoMapper::map_product(product);

        assert_eq!(unified.product_type, ProductType::Hoodie);
        assert_eq!(unified.variants.len(), 1);
        assert_eq!(unified.base_price_cents, Some(2599));
        assert_eq!(unified.currency, "USD");
    }

    #[test]
    fn test_map_variant() {
        let variant = GelatoVariant {
            variant_uid: "var-123".to_string(),
            title: "White / S".to_string(),
            product_uid: "prod-456".to_string(),
            color: Some("White".to_string()),
            color_hex: Some("#FFFFFF".to_string()),
            size: Some("S".to_string()),
            price: Some(GelatoPrice {
                amount: 12.50,
                currency: "USD".to_string(),
            }),
            is_active: Some(true),
        };

        let unified = GelatoMapper::map_variant(variant);

        assert_eq!(unified.external_id, "var-123");
        assert_eq!(unified.size, Some("S".to_string()));
        assert_eq!(unified.color_name, Some("White".to_string()));
        assert_eq!(unified.color_hex, Some("#FFFFFF".to_string()));
        assert_eq!(unified.price_cents, Some(1250));
        assert!(unified.in_stock);
        assert!(unified.is_available);
    }

    #[test]
    fn test_map_variant_inactive() {
        let variant = GelatoVariant {
            variant_uid: "var-inactive".to_string(),
            title: "Discontinued".to_string(),
            product_uid: "prod-456".to_string(),
            color: None,
            color_hex: None,
            size: None,
            price: None,
            is_active: Some(false),
        };

        let unified = GelatoMapper::map_variant(variant);

        assert!(!unified.in_stock);
        assert!(!unified.is_available);
        assert_eq!(unified.price_cents, None);
    }

    #[test]
    fn test_map_print_area() {
        let area = GelatoPrintArea {
            position: "front".to_string(),
            width_mm: Some(254.0),   // ~10 inches
            height_mm: Some(355.6),  // ~14 inches
            dpi: Some(300),
        };

        let unified = GelatoMapper::map_print_area(area);

        assert_eq!(unified.placement, PrintPlacement::Front);
        assert_eq!(unified.print_dpi, 300);
        // 254mm * 300 / 25.4 = 3000px
        assert_eq!(unified.width_px, 3000);
        // 355.6mm * 300 / 25.4 = 4200px
        assert_eq!(unified.height_px, 4200);
        assert_eq!(unified.file_format, "PNG");
    }

    #[test]
    fn test_map_print_area_default_dpi() {
        let area = GelatoPrintArea {
            position: "back".to_string(),
            width_mm: Some(254.0),
            height_mm: Some(254.0),
            dpi: None, // Should default to 300
        };

        let unified = GelatoMapper::map_print_area(area);

        assert_eq!(unified.print_dpi, 300);
        // 254mm * 300 / 25.4 = 3000px
        assert_eq!(unified.width_px, 3000);
        assert_eq!(unified.height_px, 3000);
    }

    #[test]
    fn test_map_print_area_missing_dimensions() {
        let area = GelatoPrintArea {
            position: "front".to_string(),
            width_mm: None,
            height_mm: None,
            dpi: Some(300),
        };

        let unified = GelatoMapper::map_print_area(area);

        assert_eq!(unified.width_px, 0);
        assert_eq!(unified.height_px, 0);
    }
}
