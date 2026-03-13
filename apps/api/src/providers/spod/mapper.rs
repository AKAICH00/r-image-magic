//! SPOD to Unified Model Mapper
//!
//! Maps SPOD API responses to our unified catalog models.

use super::models::*;
use crate::domain::catalog::{
    AssetType, MockupAsset, PrintConstraints, PrintPlacement, ProductType, UnifiedPrintArea,
    UnifiedProduct, UnifiedVariant,
};

/// Mapper for SPOD API responses
pub struct SpodMapper;

impl SpodMapper {
    /// Map SPOD article to unified product
    pub fn map_article(article: SpodArticle) -> UnifiedProduct {
        let category_name = article
            .article_category
            .as_ref()
            .map(|c| c.name.as_str())
            .unwrap_or("Other");
        let product_type = ProductType::from_str(category_name);
        let category_slug = product_type.category_slug().to_string();
        let brand_name = article.brand.as_ref().map(|b| b.name.clone());
        let metadata = serde_json::to_value(&article).unwrap_or_default();

        UnifiedProduct {
            external_id: article.id.to_string(),
            provider_code: "spod".to_string(),
            name: article.name,
            description: article.description,
            brand: brand_name,
            model: None,
            product_type,
            category_slug,
            is_available: true,
            regions: vec!["US".to_string(), "EU".to_string()],
            base_price_cents: None,
            currency: "USD".to_string(),
            variants: Vec::new(),
            print_areas: Vec::new(),
            provider_metadata: metadata,
        }
    }

    /// Map SPOD article detail (with appearances and sizes) to unified product
    pub fn map_article_detail(article: SpodArticle) -> UnifiedProduct {
        let appearances = article.appearances.clone().unwrap_or_default();
        let sizes = article.sizes.clone().unwrap_or_default();
        let print_areas_raw = article.print_areas.clone().unwrap_or_default();

        let mut product = Self::map_article(article);

        product.variants = Self::map_appearance_and_sizes(appearances, sizes);
        product.print_areas = print_areas_raw
            .into_iter()
            .map(Self::map_print_area)
            .collect();

        product
    }

    /// Map appearances x sizes cross-product to unified variants
    ///
    /// Each variant is a combination of one color (appearance) and one size.
    pub fn map_appearance_and_sizes(
        appearances: Vec<SpodAppearance>,
        sizes: Vec<SpodSize>,
    ) -> Vec<UnifiedVariant> {
        let mut variants = Vec::new();

        for appearance in &appearances {
            for size in &sizes {
                let external_id = format!("{}-{}", appearance.id, size.id);
                let color_hex = appearance
                    .hex_color
                    .as_ref()
                    .map(|h| {
                        if h.starts_with('#') {
                            h.clone()
                        } else {
                            format!("#{}", h)
                        }
                    });

                variants.push(UnifiedVariant {
                    external_id,
                    sku: None,
                    size: Some(size.name.clone()),
                    color_name: Some(appearance.name.clone()),
                    color_hex,
                    is_available: true,
                    price_cents: None,
                    in_stock: true,
                    provider_metadata: serde_json::json!({
                        "appearance_id": appearance.id,
                        "size_id": size.id,
                    }),
                });
            }
        }

        variants
    }

    /// Map SPOD print area to unified print area
    ///
    /// Converts millimeters to pixels using: px = mm * dpi / 25.4
    pub fn map_print_area(area: SpodPrintArea) -> UnifiedPrintArea {
        let placement = PrintPlacement::from_str(&area.name);
        let dpi = area.dpi.unwrap_or(300);

        let width_px = area
            .width_mm
            .map(|mm| (mm * dpi as f64 / 25.4).round() as i32)
            .unwrap_or(0);
        let height_px = area
            .height_mm
            .map(|mm| (mm * dpi as f64 / 25.4).round() as i32)
            .unwrap_or(0);

        UnifiedPrintArea {
            external_id: Some(area.id.to_string()),
            placement: placement.clone(),
            name: format!("{} Print Area", area.name),
            width_px,
            height_px,
            offset_x_px: 0,
            offset_y_px: 0,
            print_dpi: dpi,
            file_format: "PNG".to_string(),
            constraints: PrintConstraints {
                max_colors: None,
                technique: area.print_method.clone(),
                min_dpi: Some(dpi),
                max_file_size_mb: Some(100),
                file_formats: vec!["PNG".to_string(), "JPG".to_string()],
            },
        }
    }

    /// Map appearance images to mockup assets
    pub fn map_appearance_to_mockup_assets(appearance: SpodAppearance) -> Vec<MockupAsset> {
        let images = appearance.images.unwrap_or_default();

        images
            .into_iter()
            .map(|image| MockupAsset {
                asset_type: AssetType::BaseImage,
                placement: Some(PrintPlacement::Front),
                source_url: image.url,
                width_px: image.width,
                height_px: image.height,
                variant_external_id: Some(appearance.id.to_string()),
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_article() {
        let article = SpodArticle {
            id: 42,
            name: "Classic T-Shirt".to_string(),
            description: Some("A premium tee".to_string()),
            brand: Some(SpodBrand {
                id: 1,
                name: "Stanley/Stella".to_string(),
            }),
            article_category: Some(SpodCategory {
                id: 10,
                name: "T-Shirts".to_string(),
            }),
            appearances: None,
            sizes: None,
            print_areas: None,
        };

        let unified = SpodMapper::map_article(article);

        assert_eq!(unified.external_id, "42");
        assert_eq!(unified.provider_code, "spod");
        assert_eq!(unified.name, "Classic T-Shirt");
        assert_eq!(unified.product_type, ProductType::Tshirt);
        assert_eq!(unified.brand, Some("Stanley/Stella".to_string()));
        assert!(unified.is_available);
    }

    #[test]
    fn test_map_article_with_unknown_category() {
        let article = SpodArticle {
            id: 99,
            name: "Custom Widget".to_string(),
            description: None,
            brand: None,
            article_category: Some(SpodCategory {
                id: 99,
                name: "Novelty Gadgets".to_string(),
            }),
            appearances: None,
            sizes: None,
            print_areas: None,
        };

        let unified = SpodMapper::map_article(article);

        assert!(matches!(unified.product_type, ProductType::Other(_)));
    }

    #[test]
    fn test_map_appearance_and_sizes_cross_product() {
        let appearances = vec![
            SpodAppearance {
                id: 1,
                name: "White".to_string(),
                hex_color: Some("#FFFFFF".to_string()),
                images: None,
            },
            SpodAppearance {
                id: 2,
                name: "Black".to_string(),
                hex_color: Some("000000".to_string()),
                images: None,
            },
        ];
        let sizes = vec![
            SpodSize {
                id: 10,
                name: "S".to_string(),
            },
            SpodSize {
                id: 11,
                name: "M".to_string(),
            },
            SpodSize {
                id: 12,
                name: "L".to_string(),
            },
        ];

        let variants = SpodMapper::map_appearance_and_sizes(appearances, sizes);

        // 2 colors x 3 sizes = 6 variants
        assert_eq!(variants.len(), 6);

        // First variant: White / S
        assert_eq!(variants[0].external_id, "1-10");
        assert_eq!(variants[0].color_name, Some("White".to_string()));
        assert_eq!(variants[0].color_hex, Some("#FFFFFF".to_string()));
        assert_eq!(variants[0].size, Some("S".to_string()));

        // Fourth variant: Black / S
        assert_eq!(variants[3].external_id, "2-10");
        assert_eq!(variants[3].color_name, Some("Black".to_string()));
        assert_eq!(variants[3].color_hex, Some("#000000".to_string()));
        assert_eq!(variants[3].size, Some("S".to_string()));
    }

    #[test]
    fn test_map_appearance_and_sizes_empty() {
        let variants = SpodMapper::map_appearance_and_sizes(vec![], vec![]);
        assert!(variants.is_empty());
    }

    #[test]
    fn test_map_print_area_mm_to_px() {
        let area = SpodPrintArea {
            id: 5,
            name: "Front".to_string(),
            print_method: Some("DTG".to_string()),
            width_mm: Some(254.0),  // 254mm at 300dpi = 3000px
            height_mm: Some(381.0), // 381mm at 300dpi = 4500px
            dpi: Some(300),
        };

        let unified = SpodMapper::map_print_area(area);

        assert_eq!(unified.external_id, Some("5".to_string()));
        assert_eq!(unified.placement, PrintPlacement::Front);
        assert_eq!(unified.print_dpi, 300);
        assert_eq!(unified.width_px, 3000);
        assert_eq!(unified.height_px, 4500);
        assert_eq!(
            unified.constraints.technique,
            Some("DTG".to_string())
        );
    }

    #[test]
    fn test_map_print_area_defaults() {
        let area = SpodPrintArea {
            id: 6,
            name: "Back".to_string(),
            print_method: None,
            width_mm: None,
            height_mm: None,
            dpi: None,
        };

        let unified = SpodMapper::map_print_area(area);

        assert_eq!(unified.placement, PrintPlacement::Back);
        assert_eq!(unified.print_dpi, 300); // default
        assert_eq!(unified.width_px, 0);
        assert_eq!(unified.height_px, 0);
    }

    #[test]
    fn test_map_appearance_to_mockup_assets() {
        let appearance = SpodAppearance {
            id: 7,
            name: "Red".to_string(),
            hex_color: Some("#FF0000".to_string()),
            images: Some(vec![
                SpodImage {
                    url: "https://cdn.spod.com/img1.png".to_string(),
                    width: Some(800),
                    height: Some(1000),
                },
                SpodImage {
                    url: "https://cdn.spod.com/img2.png".to_string(),
                    width: Some(400),
                    height: Some(500),
                },
            ]),
        };

        let assets = SpodMapper::map_appearance_to_mockup_assets(appearance);

        assert_eq!(assets.len(), 2);
        assert_eq!(assets[0].asset_type, AssetType::BaseImage);
        assert_eq!(assets[0].source_url, "https://cdn.spod.com/img1.png");
        assert_eq!(assets[0].width_px, Some(800));
        assert_eq!(assets[0].variant_external_id, Some("7".to_string()));
    }

    #[test]
    fn test_map_appearance_to_mockup_assets_no_images() {
        let appearance = SpodAppearance {
            id: 8,
            name: "Blue".to_string(),
            hex_color: None,
            images: None,
        };

        let assets = SpodMapper::map_appearance_to_mockup_assets(appearance);
        assert!(assets.is_empty());
    }
}
