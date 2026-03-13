//! Gooten to Unified Model Mapper
//!
//! Maps Gooten API responses to our unified catalog models.

use super::models::*;
use crate::domain::catalog::{
    AssetType, MockupAsset, PrintConstraints, PrintPlacement, ProductType, UnifiedPrintArea,
    UnifiedProduct, UnifiedVariant,
};

/// Mapper for Gooten API responses
pub struct GootenMapper;

impl GootenMapper {
    /// Map Gooten product to unified product
    pub fn map_product(product: GootenProduct) -> UnifiedProduct {
        let product_type = product
            .categories
            .as_ref()
            .and_then(|cats| cats.first())
            .map(|cat| ProductType::from_str(&cat.name))
            .unwrap_or_else(|| ProductType::Other("Unknown".to_string()));

        let category_slug = product_type.category_slug().to_string();
        let is_available = product.has_available_products.unwrap_or(true);
        let metadata = serde_json::to_value(&product).unwrap_or_default();

        UnifiedProduct {
            external_id: product.id.to_string(),
            provider_code: "gooten".to_string(),
            name: product.name,
            description: product.short_description,
            brand: None,
            model: None,
            product_type,
            category_slug,
            is_available,
            regions: vec!["US".to_string()],
            base_price_cents: None,
            currency: "USD".to_string(),
            variants: Vec::new(),
            print_areas: Vec::new(),
            provider_metadata: metadata,
        }
    }

    /// Map Gooten variant to unified variant
    pub fn map_variant(variant: GootenVariant) -> UnifiedVariant {
        let color = variant.option_value("Color");
        let size = variant.option_value("Size");
        let price_cents = variant.price_cents();
        let metadata = serde_json::to_value(&variant).unwrap_or_default();

        UnifiedVariant {
            external_id: variant.sku.clone(),
            sku: Some(variant.sku),
            size,
            color_name: color,
            color_hex: None,
            is_available: true,
            price_cents,
            in_stock: true,
            provider_metadata: metadata,
        }
    }

    /// Map Gooten template images to mockup assets
    pub fn map_template_to_mockup_assets(template: GootenTemplate) -> Vec<MockupAsset> {
        template
            .images
            .into_iter()
            .map(|img| {
                let is_default = img.is_default.unwrap_or(false);
                let asset_type = if is_default {
                    AssetType::MockupTemplate
                } else {
                    AssetType::BaseImage
                };

                MockupAsset {
                    asset_type,
                    placement: Some(PrintPlacement::Front),
                    source_url: img.url,
                    width_px: None,
                    height_px: None,
                    variant_external_id: Some(template.sku.clone()),
                }
            })
            .collect()
    }

    /// Map a template layer to a unified print area
    pub fn map_template_layer_to_print_area(
        layer: GootenTemplateLayer,
        index: usize,
    ) -> UnifiedPrintArea {
        let placement_name = layer
            .name
            .as_deref()
            .unwrap_or(if index == 0 { "front" } else { "back" });
        let placement = PrintPlacement::from_str(placement_name);
        let display_name = format!("{} Print Area", placement_name);

        let width = layer.width().unwrap_or(0);
        let height = layer.height().unwrap_or(0);
        let offset_x = layer.x1.map(|x| x as i32).unwrap_or(0);
        let offset_y = layer.y1.map(|y| y as i32).unwrap_or(0);

        UnifiedPrintArea {
            external_id: layer.name.clone(),
            placement,
            name: display_name,
            width_px: width,
            height_px: height,
            offset_x_px: offset_x,
            offset_y_px: offset_y,
            print_dpi: 300,
            file_format: "PNG".to_string(),
            constraints: PrintConstraints {
                max_colors: None,
                technique: Some("DTG".to_string()),
                min_dpi: Some(300),
                max_file_size_mb: None,
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
        let product = GootenProduct {
            id: 42,
            name: "Canvas Wrap".to_string(),
            short_description: Some("A wrapped canvas print".to_string()),
            categories: Some(vec![GootenCategory {
                id: 1,
                name: "Canvas".to_string(),
            }]),
            images: None,
            has_available_products: Some(true),
            max_zone_count: Some(1),
        };

        let unified = GootenMapper::map_product(product);

        assert_eq!(unified.external_id, "42");
        assert_eq!(unified.provider_code, "gooten");
        assert_eq!(unified.name, "Canvas Wrap");
        assert_eq!(unified.product_type, ProductType::Canvas);
        assert_eq!(unified.category_slug, "posters");
        assert!(unified.is_available);
        assert_eq!(
            unified.description,
            Some("A wrapped canvas print".to_string())
        );
    }

    #[test]
    fn test_map_product_no_category() {
        let product = GootenProduct {
            id: 99,
            name: "Mystery Item".to_string(),
            short_description: None,
            categories: None,
            images: None,
            has_available_products: None,
            max_zone_count: None,
        };

        let unified = GootenMapper::map_product(product);

        assert!(matches!(unified.product_type, ProductType::Other(_)));
        assert!(unified.is_available);
    }

    #[test]
    fn test_map_variant_with_options() {
        let variant = GootenVariant {
            sku: "GT-SHIRT-WHT-M".to_string(),
            product_id: 10,
            max_images: Some(1),
            has_templates: Some(true),
            options: Some(vec![
                GootenOption {
                    name: "Color".to_string(),
                    value: "White".to_string(),
                },
                GootenOption {
                    name: "Size".to_string(),
                    value: "M".to_string(),
                },
            ]),
            price_info: Some(GootenPriceInfo {
                price: Some(14.99),
                currency_code: Some("USD".to_string()),
            }),
        };

        let unified = GootenMapper::map_variant(variant);

        assert_eq!(unified.external_id, "GT-SHIRT-WHT-M");
        assert_eq!(unified.sku, Some("GT-SHIRT-WHT-M".to_string()));
        assert_eq!(unified.color_name, Some("White".to_string()));
        assert_eq!(unified.size, Some("M".to_string()));
        assert_eq!(unified.price_cents, Some(1499));
        assert!(unified.in_stock);
    }

    #[test]
    fn test_map_variant_no_options() {
        let variant = GootenVariant {
            sku: "GT-PLAIN".to_string(),
            product_id: 10,
            max_images: None,
            has_templates: None,
            options: None,
            price_info: None,
        };

        let unified = GootenMapper::map_variant(variant);

        assert_eq!(unified.external_id, "GT-PLAIN");
        assert!(unified.color_name.is_none());
        assert!(unified.size.is_none());
        assert!(unified.price_cents.is_none());
    }

    #[test]
    fn test_map_template_to_mockup_assets() {
        let template = GootenTemplate {
            sku: "GT-SKU-001".to_string(),
            images: vec![
                GootenTemplateImage {
                    url: "https://example.com/default.png".to_string(),
                    index: Some(0),
                    is_default: Some(true),
                    layers: None,
                },
                GootenTemplateImage {
                    url: "https://example.com/alt.png".to_string(),
                    index: Some(1),
                    is_default: Some(false),
                    layers: None,
                },
            ],
        };

        let assets = GootenMapper::map_template_to_mockup_assets(template);

        assert_eq!(assets.len(), 2);
        assert_eq!(assets[0].asset_type, AssetType::MockupTemplate);
        assert_eq!(assets[0].source_url, "https://example.com/default.png");
        assert_eq!(
            assets[0].variant_external_id,
            Some("GT-SKU-001".to_string())
        );
        assert_eq!(assets[1].asset_type, AssetType::BaseImage);
    }

    #[test]
    fn test_map_template_layer_to_print_area() {
        let layer = GootenTemplateLayer {
            name: Some("front".to_string()),
            x1: Some(100.0),
            y1: Some(200.0),
            x2: Some(500.0),
            y2: Some(700.0),
        };

        let area = GootenMapper::map_template_layer_to_print_area(layer, 0);

        assert_eq!(area.placement, PrintPlacement::Front);
        assert_eq!(area.width_px, 400);
        assert_eq!(area.height_px, 500);
        assert_eq!(area.offset_x_px, 100);
        assert_eq!(area.offset_y_px, 200);
        assert_eq!(area.print_dpi, 300);
    }

    #[test]
    fn test_map_template_layer_defaults() {
        let layer = GootenTemplateLayer {
            name: None,
            x1: None,
            y1: None,
            x2: None,
            y2: None,
        };

        let area = GootenMapper::map_template_layer_to_print_area(layer, 0);

        assert_eq!(area.placement, PrintPlacement::Front);
        assert_eq!(area.width_px, 0);
        assert_eq!(area.height_px, 0);
    }

    #[test]
    fn test_map_template_layer_back_placement() {
        let layer = GootenTemplateLayer {
            name: None,
            x1: Some(0.0),
            y1: Some(0.0),
            x2: Some(300.0),
            y2: Some(400.0),
        };

        let area = GootenMapper::map_template_layer_to_print_area(layer, 1);

        assert_eq!(area.placement, PrintPlacement::Back);
    }
}
