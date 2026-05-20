//! Catalog API handlers
//!
//! Endpoints for browsing POD provider catalogs, products, and categories.

use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use tokio_postgres::types::ToSql;
use uuid::Uuid;

use crate::db::DbPool;

/// Query parameters for listing products
#[derive(Debug, Deserialize)]
pub struct ProductsQuery {
    /// Filter by provider code
    pub provider: Option<String>,
    /// Filter by category slug
    pub category: Option<String>,
    /// Filter by product type
    pub product_type: Option<String>,
    /// Search by name
    pub search: Option<String>,
    /// Page number (1-based)
    #[serde(default = "default_page")]
    pub page: u32,
    /// Items per page
    #[serde(default = "default_per_page")]
    pub per_page: u32,
}

fn default_page() -> u32 {
    1
}
fn default_per_page() -> u32 {
    50
}

const MAX_PER_PAGE: u32 = 100;

/// Provider response
#[derive(Debug, Serialize)]
pub struct ProviderResponse {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub is_active: bool,
    pub sync_enabled: bool,
    pub last_sync_at: Option<String>,
    pub rate_limit_per_minute: i32,
}

/// Category response
#[derive(Debug, Serialize)]
pub struct CategoryResponse {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
    pub product_count: i64,
}

/// Product summary response
#[derive(Debug, Serialize)]
pub struct ProductSummaryResponse {
    pub id: Uuid,
    pub provider_code: String,
    pub external_product_id: String,
    pub name: String,
    pub product_type: String,
    pub category_slug: Option<String>,
    pub is_available: bool,
    pub variant_count: i64,
}

/// Product detail response
#[derive(Debug, Serialize)]
pub struct ProductDetailResponse {
    pub id: Uuid,
    pub provider_code: String,
    pub external_product_id: String,
    pub name: String,
    pub brand: Option<String>,
    pub product_type: String,
    pub category_slug: Option<String>,
    pub is_available: bool,
    pub base_price_cents: Option<i32>,
    pub variants: Vec<VariantResponse>,
    pub print_areas: Vec<PrintAreaResponse>,
}

/// Variant response
#[derive(Debug, Serialize)]
pub struct VariantResponse {
    pub id: Uuid,
    pub external_variant_id: String,
    pub sku: Option<String>,
    pub size: Option<String>,
    pub color_name: Option<String>,
    pub color_hex: Option<String>,
    pub is_available: bool,
    pub price_cents: Option<i32>,
}

/// Print area response
#[derive(Debug, Serialize)]
pub struct PrintAreaResponse {
    pub id: Uuid,
    pub placement: String,
    pub name: String,
    pub width_px: i32,
    pub height_px: i32,
    pub offset_x_px: i32,
    pub offset_y_px: i32,
    pub print_dpi: i32,
}

/// Paginated response wrapper
#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: u32,
    pub per_page: u32,
    pub total_pages: u32,
}

#[derive(Debug, PartialEq, Eq)]
struct ProductFilterSql {
    where_clause: String,
    values: Vec<String>,
}

fn escape_like_term(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

fn normalize_pagination(query: &ProductsQuery) -> (u32, u32) {
    (query.page.max(1), query.per_page.clamp(1, MAX_PER_PAGE))
}

fn push_filter(
    conditions: &mut Vec<String>,
    values: &mut Vec<String>,
    sql_template: impl FnOnce(usize) -> String,
    raw_value: &Option<String>,
) {
    if let Some(value) = raw_value
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        values.push(value.to_string());
        conditions.push(sql_template(values.len()));
    }
}

fn build_product_filter_sql(query: &ProductsQuery) -> ProductFilterSql {
    let mut conditions = Vec::new();
    let mut values = Vec::new();

    push_filter(
        &mut conditions,
        &mut values,
        |idx| format!("pr.code = ${idx}"),
        &query.provider,
    );
    push_filter(
        &mut conditions,
        &mut values,
        |idx| format!("c.slug = ${idx}"),
        &query.category,
    );
    push_filter(
        &mut conditions,
        &mut values,
        |idx| format!("p.product_type = ${idx}"),
        &query.product_type,
    );

    if let Some(search) = query
        .search
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        values.push(format!("%{}%", escape_like_term(search)));
        conditions.push(format!("p.name ILIKE ${} ESCAPE '\\'", values.len()));
    }

    ProductFilterSql {
        where_clause: if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        },
        values,
    }
}

fn as_sql_params(values: &[String]) -> Vec<&(dyn ToSql + Sync)> {
    values
        .iter()
        .map(|value| value as &(dyn ToSql + Sync))
        .collect()
}

/// Helper macro to get database client
macro_rules! get_client {
    ($pool:expr) => {
        match $pool.get().await {
            Ok(c) => c,
            Err(e) => {
                tracing::error!("Failed to get database connection: {}", e);
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Database connection failed"
                }));
            }
        }
    };
}

/// List all POD providers
pub async fn list_providers(pool: web::Data<DbPool>) -> HttpResponse {
    let client = get_client!(pool);

    let query = r#"
        SELECT
            id, code, name, is_active, sync_enabled,
            last_sync_at, rate_limit_per_minute
        FROM pod_providers
        ORDER BY name
    "#;

    match client.query(query, &[]).await {
        Ok(rows) => {
            let providers: Vec<ProviderResponse> = rows
                .iter()
                .map(|row| ProviderResponse {
                    id: row.get("id"),
                    code: row.get("code"),
                    name: row.get("name"),
                    is_active: row.get("is_active"),
                    sync_enabled: row.get("sync_enabled"),
                    last_sync_at: row
                        .get::<_, Option<chrono::DateTime<chrono::Utc>>>("last_sync_at")
                        .map(|dt| dt.to_rfc3339()),
                    rate_limit_per_minute: row.get("rate_limit_per_minute"),
                })
                .collect();

            HttpResponse::Ok().json(providers)
        }
        Err(e) => {
            tracing::error!("Failed to list providers: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to list providers"
            }))
        }
    }
}

/// List all product categories with counts
pub async fn list_categories(pool: web::Data<DbPool>) -> HttpResponse {
    let client = get_client!(pool);

    let query = r#"
        SELECT
            c.id, c.slug, c.name,
            COUNT(p.id) as product_count
        FROM product_categories c
        LEFT JOIN pod_products p ON p.category_id = c.id
        GROUP BY c.id, c.slug, c.name
        ORDER BY c.sort_order, c.name
    "#;

    match client.query(query, &[]).await {
        Ok(rows) => {
            let categories: Vec<CategoryResponse> = rows
                .iter()
                .map(|row| CategoryResponse {
                    id: row.get("id"),
                    slug: row.get("slug"),
                    name: row.get("name"),
                    product_count: row.get("product_count"),
                })
                .collect();

            HttpResponse::Ok().json(categories)
        }
        Err(e) => {
            tracing::error!("Failed to list categories: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to list categories"
            }))
        }
    }
}

/// List products with filtering and pagination
pub async fn list_products(
    pool: web::Data<DbPool>,
    query_params: web::Query<ProductsQuery>,
) -> HttpResponse {
    let client = get_client!(pool);
    let (page, per_page) = normalize_pagination(&query_params);
    let offset = ((page - 1) * per_page) as i64;
    let limit = per_page as i64;
    let filter_sql = build_product_filter_sql(&query_params);
    let count_params = as_sql_params(&filter_sql.values);

    // Count query
    let count_sql = format!(
        r#"
        SELECT COUNT(*) as total
        FROM pod_products p
        JOIN pod_providers pr ON p.provider_id = pr.id
        LEFT JOIN product_categories c ON p.category_id = c.id
        {}
    "#,
        filter_sql.where_clause
    );

    let total: i64 = match client.query_one(&count_sql, &count_params).await {
        Ok(row) => row.get("total"),
        Err(e) => {
            tracing::error!("Failed to count products: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to list products"
            }));
        }
    };

    let limit_placeholder = filter_sql.values.len() + 1;
    let offset_placeholder = filter_sql.values.len() + 2;

    // Data query
    let data_sql = format!(
        r#"
        SELECT
            p.id, pr.code as provider_code, p.external_product_id,
            p.name, p.product_type, c.slug as category_slug,
            p.is_available,
            (SELECT COUNT(*) FROM pod_product_variants v WHERE v.product_id = p.id) as variant_count
        FROM pod_products p
        JOIN pod_providers pr ON p.provider_id = pr.id
        LEFT JOIN product_categories c ON p.category_id = c.id
        {}
        ORDER BY p.name
        LIMIT ${} OFFSET ${}
    "#,
        filter_sql.where_clause, limit_placeholder, offset_placeholder
    );

    let mut data_params = count_params;
    data_params.push(&limit);
    data_params.push(&offset);

    match client.query(&data_sql, &data_params).await {
        Ok(rows) => {
            let products: Vec<ProductSummaryResponse> = rows
                .iter()
                .map(|row| ProductSummaryResponse {
                    id: row.get("id"),
                    provider_code: row.get("provider_code"),
                    external_product_id: row.get("external_product_id"),
                    name: row.get("name"),
                    product_type: row.get("product_type"),
                    category_slug: row.get("category_slug"),
                    is_available: row.get("is_available"),
                    variant_count: row.get("variant_count"),
                })
                .collect();

            let total_pages = if total == 0 {
                0
            } else {
                ((total as f64) / (per_page as f64)).ceil() as u32
            };

            HttpResponse::Ok().json(PaginatedResponse {
                items: products,
                total,
                page,
                per_page,
                total_pages,
            })
        }
        Err(e) => {
            tracing::error!("Failed to list products: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to list products"
            }))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{build_product_filter_sql, escape_like_term, normalize_pagination, ProductsQuery};

    fn sample_query() -> ProductsQuery {
        ProductsQuery {
            provider: None,
            category: None,
            product_type: None,
            search: None,
            page: 1,
            per_page: 50,
        }
    }

    #[test]
    fn builds_parameterized_filters_in_order() {
        let mut query = sample_query();
        query.provider = Some("printful".to_string());
        query.category = Some("shirts".to_string());
        query.product_type = Some("hoodie".to_string());
        query.search = Some("logo tee".to_string());

        let filters = build_product_filter_sql(&query);

        assert_eq!(
            filters.where_clause,
            "WHERE pr.code = $1 AND c.slug = $2 AND p.product_type = $3 AND p.name ILIKE $4 ESCAPE '\\'"
        );
        assert_eq!(
            filters.values,
            vec![
                "printful".to_string(),
                "shirts".to_string(),
                "hoodie".to_string(),
                "%logo tee%".to_string(),
            ]
        );
    }

    #[test]
    fn escapes_like_wildcards_in_search_terms() {
        assert_eq!(
            escape_like_term(r"100% cotton_\ready"),
            r"100\% cotton\_\\ready"
        );
    }

    #[test]
    fn ignores_blank_filters_and_clamps_pagination() {
        let mut query = sample_query();
        query.provider = Some("   ".to_string());
        query.search = Some("   ".to_string());
        query.page = 0;
        query.per_page = 500;

        let filters = build_product_filter_sql(&query);

        assert_eq!(filters.where_clause, "");
        assert!(filters.values.is_empty());
        assert_eq!(normalize_pagination(&query), (1, 100));
    }
}

/// Get product details by ID
pub async fn get_product(pool: web::Data<DbPool>, path: web::Path<Uuid>) -> HttpResponse {
    let client = get_client!(pool);
    let product_id = path.into_inner();

    // Get product
    let product_sql = r#"
        SELECT
            p.id, pr.code as provider_code, p.external_product_id,
            p.name, p.brand, p.product_type, c.slug as category_slug,
            p.is_available, p.base_price_cents
        FROM pod_products p
        JOIN pod_providers pr ON p.provider_id = pr.id
        LEFT JOIN product_categories c ON p.category_id = c.id
        WHERE p.id = $1
    "#;

    let product_row = match client.query_opt(product_sql, &[&product_id]).await {
        Ok(Some(row)) => row,
        Ok(None) => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "error": "Product not found"
            }));
        }
        Err(e) => {
            tracing::error!("Failed to get product: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to get product"
            }));
        }
    };

    // Get variants
    let variants_sql = r#"
        SELECT id, external_variant_id, sku, size, color_name, color_hex, is_available, price_cents
        FROM pod_product_variants
        WHERE product_id = $1
        ORDER BY size, color_name
    "#;

    let variants: Vec<VariantResponse> = match client.query(variants_sql, &[&product_id]).await {
        Ok(rows) => rows
            .iter()
            .map(|row| VariantResponse {
                id: row.get("id"),
                external_variant_id: row.get("external_variant_id"),
                sku: row.get("sku"),
                size: row.get("size"),
                color_name: row.get("color_name"),
                color_hex: row.get("color_hex"),
                is_available: row.get("is_available"),
                price_cents: row.get("price_cents"),
            })
            .collect(),
        Err(_) => Vec::new(),
    };

    // Get print areas
    let areas_sql = r#"
        SELECT id, placement, name, width_px, height_px, offset_x_px, offset_y_px, print_dpi
        FROM pod_print_areas
        WHERE product_id = $1
        ORDER BY placement
    "#;

    let print_areas: Vec<PrintAreaResponse> = match client.query(areas_sql, &[&product_id]).await {
        Ok(rows) => rows
            .iter()
            .map(|row| PrintAreaResponse {
                id: row.get("id"),
                placement: row.get("placement"),
                name: row.get("name"),
                width_px: row.get("width_px"),
                height_px: row.get("height_px"),
                offset_x_px: row.get("offset_x_px"),
                offset_y_px: row.get("offset_y_px"),
                print_dpi: row.get("print_dpi"),
            })
            .collect(),
        Err(_) => Vec::new(),
    };

    HttpResponse::Ok().json(ProductDetailResponse {
        id: product_row.get("id"),
        provider_code: product_row.get("provider_code"),
        external_product_id: product_row.get("external_product_id"),
        name: product_row.get("name"),
        brand: product_row.get("brand"),
        product_type: product_row.get("product_type"),
        category_slug: product_row.get("category_slug"),
        is_available: product_row.get("is_available"),
        base_price_cents: product_row.get("base_price_cents"),
        variants,
        print_areas,
    })
}

/// Get print areas for a product
pub async fn get_print_areas(pool: web::Data<DbPool>, path: web::Path<Uuid>) -> HttpResponse {
    let client = get_client!(pool);
    let product_id = path.into_inner();

    let sql = r#"
        SELECT id, placement, name, width_px, height_px, offset_x_px, offset_y_px, print_dpi
        FROM pod_print_areas
        WHERE product_id = $1
        ORDER BY placement
    "#;

    match client.query(sql, &[&product_id]).await {
        Ok(rows) => {
            let areas: Vec<PrintAreaResponse> = rows
                .iter()
                .map(|row| PrintAreaResponse {
                    id: row.get("id"),
                    placement: row.get("placement"),
                    name: row.get("name"),
                    width_px: row.get("width_px"),
                    height_px: row.get("height_px"),
                    offset_x_px: row.get("offset_x_px"),
                    offset_y_px: row.get("offset_y_px"),
                    print_dpi: row.get("print_dpi"),
                })
                .collect();

            HttpResponse::Ok().json(areas)
        }
        Err(e) => {
            tracing::error!("Failed to get print areas: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to get print areas"
            }))
        }
    }
}
