//! Printful to Unified Model Mapper
//!
//! Maps Printful API responses to our unified catalog models.

use crate::domain::catalog::{
    UnifiedProduct, UnifiedVariant, UnifiedPrintArea, MockupAsset,
    ProductType, PrintPlacement, AssetType, PrintConstraints,
};
use super::models::*;

/// Mapper for Printful API responses
pub struct PrintfulMapper;

impl PrintfulMapper {
    /// Map Printful product to unified product
    pub fn map_product(product: PrintfulProduct) -> UnifiedProduct {
        let product_type = ProductType::from_str(&product.type_name);
        let category_slug = product_type.category_slug().to_string();
        let currency = product.currency.clone().unwrap_or_else(|| "USD".to_string());
        let is_available = product.is_available();
        let metadata = serde_json::to_value(&product).unwrap_or_default();

        UnifiedProduct {
            external_id: product.id.to_string(),
            provider_code: "printful".to_string(),
            name: product.title,
            description: product.description,
            brand: product.brand,
            model: product.model,
            product_type,
            category_slug,
            is_available,
            regions: vec!["US".to_string(), "EU".to_string()], // Printful ships globally
            base_price_cents: None, // Set from variants
            currency,
            variants: Vec::new(), // Populated separately
            print_areas: Vec::new(), // Populated separately
            provider_metadata: metadata,
        }
    }

    /// Map Printful product detail (with variants) to unified product
    pub fn map_product_detail(detail: PrintfulProductDetail) -> UnifiedProduct {
        let mut product = Self::map_product(detail.product);

        // Map variants
        product.variants = detail.variants
            .into_iter()
            .map(Self::map_variant)
            .collect();

        // Set base price from first variant
        if let Some(first_variant) = product.variants.first() {
            product.base_price_cents = first_variant.price_cents;
        }

        product
    }

    /// Map Printful variant to unified variant
    pub fn map_variant(variant: PrintfulVariant) -> UnifiedVariant {
        UnifiedVariant {
            external_id: variant.id.to_string(),
            sku: None, // Printful doesn't expose SKU in catalog
            size: variant.size.clone(),
            color_name: variant.color.clone(),
            color_hex: variant.color_code.clone(),
            is_available: variant.is_in_stock(),
            price_cents: variant.price_cents(),
            in_stock: variant.is_in_stock(),
            provider_metadata: serde_json::to_value(&variant).unwrap_or_default(),
        }
    }

    /// Map Printful printfile to unified print area
    pub fn map_print_area(printfile: PrintfulPrintfile, placement_name: &str) -> UnifiedPrintArea {
        let placement = PrintPlacement::from_str(placement_name);

        UnifiedPrintArea {
            external_id: Some(printfile.printfile_id.to_string()),
            placement: placement.clone(),
            name: format!("{} Print Area", placement_name),
            width_px: printfile.width,
            height_px: printfile.height,
            offset_x_px: 0, // Position comes from templates
            offset_y_px: 0,
            print_dpi: printfile.dpi,
            file_format: "PNG".to_string(),
            constraints: PrintConstraints {
                max_colors: None,
                technique: Some("DTG".to_string()),
                min_dpi: Some(printfile.dpi),
                max_file_size_mb: Some(200),
                file_formats: vec!["PNG".to_string(), "JPG".to_string()],
            },
        }
    }

    /// Map Printful mockup template to mockup asset
    pub fn map_mockup_template(template: PrintfulMockupTemplate) -> MockupAsset {
        let (width, height) = match &template.template_positions {
            Some(pos) => (Some(pos.area_width), Some(pos.area_height)),
            None => (template.template_width, template.template_height),
        };

        MockupAsset {
            asset_type: AssetType::MockupTemplate,
            placement: Some(PrintPlacement::Front), // Default, may need context
            source_url: template.image_url,
            width_px: width,
            height_px: height,
            variant_external_id: None,
        }
    }

    /// Map mockup templates to assets with variant associations
    pub fn map_mockup_assets(
        response: PrintfulMockupTemplatesResponse,
        variant_id: Option<&str>,
    ) -> Vec<MockupAsset> {
        let mut assets = Vec::new();

        // Build variant-to-template map
        let variant_templates: std::collections::HashMap<i64, Vec<i64>> = response
            .variant_mapping
            .unwrap_or_default()
            .into_iter()
            .map(|m| (m.variant_id, m.templates))
            .collect();

        for template in response.templates {
            let mut asset = Self::map_mockup_template(template.clone());

            // Check if this template applies to the requested variant
            if let Some(vid) = variant_id {
                if let Ok(vid_i64) = vid.parse::<i64>() {
                    if let Some(template_ids) = variant_templates.get(&vid_i64) {
                        if !template_ids.contains(&template.template_id) {
                            continue; // Skip templates not for this variant
                        }
                        asset.variant_external_id = Some(vid.to_string());
                    }
                }
            }

            assets.push(asset);
        }

        assets
    }

    /// Extract placement from printfiles response
    pub fn extract_print_areas(response: PrintfulPrintfilesResponse) -> Vec<UnifiedPrintArea> {
        // Parse available_placements to get placement names
        let placements: Vec<String> = response
            .available_placements
            .and_then(|v| {
                if let Some(obj) = v.as_object() {
                    Some(obj.keys().cloned().collect())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| vec!["front".to_string()]);

        // Map each printfile with its placement
        response.printfiles
            .into_iter()
            .enumerate()
            .map(|(i, pf)| {
                let placement_name = placements.get(i).cloned()
                    .unwrap_or_else(|| "front".to_string());
                Self::map_print_area(pf, &placement_name)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_product() {
        let product = PrintfulProduct {
            id: 123,
            main_category_id: Some(1),
            r#type: "T-SHIRT".to_string(),
            type_name: "T-Shirt".to_string(),
            title: "Bella + Canvas 3001".to_string(),
            brand: Some("Bella + Canvas".to_string()),
            model: Some("3001".to_string()),
            image: None,
            variant_count: 100,
            currency: Some("USD".to_string()),
            options: None,
            is_discontinued: Some(false),
            description: Some("Premium t-shirt".to_string()),
        };

        let unified = PrintfulMapper::map_product(product);

        assert_eq!(unified.external_id, "123");
        assert_eq!(unified.provider_code, "printful");
        assert_eq!(unified.name, "Bella + Canvas 3001");
        assert_eq!(unified.product_type, ProductType::Tshirt);
        assert!(unified.is_available);
    }

    #[test]
    fn test_map_variant() {
        let variant = PrintfulVariant {
            id: 456,
            product_id: 123,
            name: "White / S".to_string(),
            size: Some("S".to_string()),
            color: Some("White".to_string()),
            color_code: Some("#FFFFFF".to_string()),
            color_code2: None,
            image: None,
            price: Some("12.50".to_string()),
            in_stock: Some(true),
            availability_status: None,
        };

        let unified = PrintfulMapper::map_variant(variant);

        assert_eq!(unified.external_id, "456");
        assert_eq!(unified.size, Some("S".to_string()));
        assert_eq!(unified.color_name, Some("White".to_string()));
        assert_eq!(unified.price_cents, Some(1250));
        assert!(unified.in_stock);
    }
}
