//! Database queries for templates

use super::models::DbTemplate;
use super::pool::{DbPool, DbError};
use tracing::info;

/// Repository for template database operations
pub struct TemplateRepository {
    pool: DbPool,
}

impl TemplateRepository {
    /// Create a new template repository
    pub fn new(pool: DbPool) -> Self {
        TemplateRepository { pool }
    }

    /// Get all active templates
    pub async fn get_all_active(&self) -> Result<Vec<DbTemplate>, DbError> {
        let client = self.pool.get().await?;

        let rows = client.query(
            r#"
            SELECT
                id, template_id, name, description, product_type, variant, color,
                print_area_x, print_area_y, print_area_width, print_area_height,
                base_image_path, displacement_map_path, mask_path,
                width, height, is_active, created_at, updated_at
            FROM templates
            WHERE is_active = true
            ORDER BY product_type, template_id
            "#,
            &[]
        ).await?;

        let templates: Vec<DbTemplate> = rows.iter().map(|row| {
            DbTemplate {
                id: row.get("id"),
                template_id: row.get("template_id"),
                name: row.get("name"),
                description: row.get("description"),
                product_type: row.get("product_type"),
                variant: row.get("variant"),
                color: row.get("color"),
                print_area_x: row.get("print_area_x"),
                print_area_y: row.get("print_area_y"),
                print_area_width: row.get("print_area_width"),
                print_area_height: row.get("print_area_height"),
                base_image_path: row.get("base_image_path"),
                displacement_map_path: row.get("displacement_map_path"),
                mask_path: row.get("mask_path"),
                width: row.get("width"),
                height: row.get("height"),
                is_active: row.get("is_active"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            }
        }).collect();

        info!("Loaded {} active templates from database", templates.len());
        Ok(templates)
    }

    /// Get a template by its template_id
    pub async fn get_by_template_id(&self, template_id: &str) -> Result<Option<DbTemplate>, DbError> {
        let client = self.pool.get().await?;

        let row = client.query_opt(
            r#"
            SELECT
                id, template_id, name, description, product_type, variant, color,
                print_area_x, print_area_y, print_area_width, print_area_height,
                base_image_path, displacement_map_path, mask_path,
                width, height, is_active, created_at, updated_at
            FROM templates
            WHERE template_id = $1 AND is_active = true
            "#,
            &[&template_id]
        ).await?;

        Ok(row.map(|row| {
            DbTemplate {
                id: row.get("id"),
                template_id: row.get("template_id"),
                name: row.get("name"),
                description: row.get("description"),
                product_type: row.get("product_type"),
                variant: row.get("variant"),
                color: row.get("color"),
                print_area_x: row.get("print_area_x"),
                print_area_y: row.get("print_area_y"),
                print_area_width: row.get("print_area_width"),
                print_area_height: row.get("print_area_height"),
                base_image_path: row.get("base_image_path"),
                displacement_map_path: row.get("displacement_map_path"),
                mask_path: row.get("mask_path"),
                width: row.get("width"),
                height: row.get("height"),
                is_active: row.get("is_active"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            }
        }))
    }

    /// Get templates by product type
    pub async fn get_by_product_type(&self, product_type: &str) -> Result<Vec<DbTemplate>, DbError> {
        let client = self.pool.get().await?;

        let rows = client.query(
            r#"
            SELECT
                id, template_id, name, description, product_type, variant, color,
                print_area_x, print_area_y, print_area_width, print_area_height,
                base_image_path, displacement_map_path, mask_path,
                width, height, is_active, created_at, updated_at
            FROM templates
            WHERE product_type = $1 AND is_active = true
            ORDER BY template_id
            "#,
            &[&product_type]
        ).await?;

        let templates: Vec<DbTemplate> = rows.iter().map(|row| {
            DbTemplate {
                id: row.get("id"),
                template_id: row.get("template_id"),
                name: row.get("name"),
                description: row.get("description"),
                product_type: row.get("product_type"),
                variant: row.get("variant"),
                color: row.get("color"),
                print_area_x: row.get("print_area_x"),
                print_area_y: row.get("print_area_y"),
                print_area_width: row.get("print_area_width"),
                print_area_height: row.get("print_area_height"),
                base_image_path: row.get("base_image_path"),
                displacement_map_path: row.get("displacement_map_path"),
                mask_path: row.get("mask_path"),
                width: row.get("width"),
                height: row.get("height"),
                is_active: row.get("is_active"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            }
        }).collect();

        Ok(templates)
    }

    /// Get count of templates by product type
    pub async fn get_product_type_counts(&self) -> Result<Vec<(String, i64)>, DbError> {
        let client = self.pool.get().await?;

        let rows = client.query(
            r#"
            SELECT product_type, COUNT(*) as count
            FROM templates
            WHERE is_active = true
            GROUP BY product_type
            ORDER BY count DESC
            "#,
            &[]
        ).await?;

        Ok(rows.iter().map(|row| {
            (row.get::<_, String>("product_type"), row.get::<_, i64>("count"))
        }).collect())
    }
}
