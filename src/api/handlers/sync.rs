//! Sync API handlers
//!
//! Endpoints for triggering and monitoring POD catalog synchronization.

use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db::DbPool;
use crate::storage::{R2Client, AssetPath};

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

/// Request to start a sync job
#[derive(Debug, Deserialize)]
pub struct StartSyncRequest {
    /// Type of sync to perform
    #[serde(default = "default_job_type")]
    pub job_type: String,
    /// Optional product ID for single product sync
    pub product_id: Option<String>,
}

fn default_job_type() -> String {
    "full_catalog".to_string()
}

/// R2 storage status response
#[derive(Debug, Serialize)]
pub struct R2StatusResponse {
    pub configured: bool,
    pub bucket_name: Option<String>,
    pub account_id: Option<String>,
}

/// List all sync jobs
pub async fn list_jobs(
    pool: web::Data<DbPool>,
) -> HttpResponse {
    let client = get_client!(pool);

    let sql = r#"
        SELECT
            j.id, pr.code as provider_code, j.job_type, j.status,
            j.total_items, j.processed_items, j.failed_items,
            j.started_at, j.completed_at, j.error_message
        FROM pod_sync_jobs j
        JOIN pod_providers pr ON j.provider_id = pr.id
        ORDER BY j.started_at DESC NULLS LAST
        LIMIT 50
    "#;

    match client.query(sql, &[]).await {
        Ok(rows) => {
            let jobs: Vec<serde_json::Value> = rows.iter().map(|row| {
                let started_at: Option<chrono::DateTime<chrono::Utc>> = row.get("started_at");
                let completed_at: Option<chrono::DateTime<chrono::Utc>> = row.get("completed_at");
                let total: i32 = row.get("total_items");
                let processed: i32 = row.get("processed_items");
                let progress = if total > 0 { (processed as f32 / total as f32) * 100.0 } else { 0.0 };

                let duration = match (started_at, completed_at) {
                    (Some(start), Some(end)) => Some((end - start).num_seconds()),
                    (Some(start), None) => Some((chrono::Utc::now() - start).num_seconds()),
                    _ => None,
                };

                serde_json::json!({
                    "id": row.get::<_, Uuid>("id"),
                    "provider_code": row.get::<_, String>("provider_code"),
                    "job_type": row.get::<_, String>("job_type"),
                    "status": row.get::<_, String>("status"),
                    "total_items": total,
                    "processed_items": processed,
                    "failed_items": row.get::<_, i32>("failed_items"),
                    "progress_percent": progress,
                    "started_at": started_at.map(|dt| dt.to_rfc3339()),
                    "completed_at": completed_at.map(|dt| dt.to_rfc3339()),
                    "duration_secs": duration,
                    "error_message": row.get::<_, Option<String>>("error_message"),
                })
            }).collect();

            HttpResponse::Ok().json(jobs)
        }
        Err(e) => {
            tracing::error!("Failed to list sync jobs: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to list sync jobs"
            }))
        }
    }
}

/// Get sync job by ID
pub async fn get_job(
    pool: web::Data<DbPool>,
    path: web::Path<Uuid>,
) -> HttpResponse {
    let client = get_client!(pool);
    let job_id = path.into_inner();

    let sql = r#"
        SELECT
            j.id, pr.code as provider_code, j.job_type, j.status,
            j.total_items, j.processed_items, j.failed_items,
            j.started_at, j.completed_at, j.error_message
        FROM pod_sync_jobs j
        JOIN pod_providers pr ON j.provider_id = pr.id
        WHERE j.id = $1
    "#;

    match client.query_opt(sql, &[&job_id]).await {
        Ok(Some(row)) => {
            let started_at: Option<chrono::DateTime<chrono::Utc>> = row.get("started_at");
            let completed_at: Option<chrono::DateTime<chrono::Utc>> = row.get("completed_at");
            let total: i32 = row.get("total_items");
            let processed: i32 = row.get("processed_items");
            let progress = if total > 0 { (processed as f32 / total as f32) * 100.0 } else { 0.0 };

            let duration = match (started_at, completed_at) {
                (Some(start), Some(end)) => Some((end - start).num_seconds()),
                (Some(start), None) => Some((chrono::Utc::now() - start).num_seconds()),
                _ => None,
            };

            HttpResponse::Ok().json(serde_json::json!({
                "id": row.get::<_, Uuid>("id"),
                "provider_code": row.get::<_, String>("provider_code"),
                "job_type": row.get::<_, String>("job_type"),
                "status": row.get::<_, String>("status"),
                "total_items": total,
                "processed_items": processed,
                "failed_items": row.get::<_, i32>("failed_items"),
                "progress_percent": progress,
                "started_at": started_at.map(|dt| dt.to_rfc3339()),
                "completed_at": completed_at.map(|dt| dt.to_rfc3339()),
                "duration_secs": duration,
                "error_message": row.get::<_, Option<String>>("error_message"),
            }))
        }
        Ok(None) => {
            HttpResponse::NotFound().json(serde_json::json!({
                "error": "Sync job not found"
            }))
        }
        Err(e) => {
            tracing::error!("Failed to get sync job: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to get sync job"
            }))
        }
    }
}

/// Start a sync job for a provider
pub async fn start_sync(
    pool: web::Data<DbPool>,
    path: web::Path<String>,
    body: web::Json<StartSyncRequest>,
) -> HttpResponse {
    let client = get_client!(pool);
    let provider_code = path.into_inner();

    // Get provider ID
    let provider_sql = "SELECT id FROM pod_providers WHERE code = $1 AND is_active = true";
    let provider_id: Uuid = match client.query_opt(provider_sql, &[&provider_code]).await {
        Ok(Some(row)) => row.get("id"),
        Ok(None) => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "error": format!("Provider '{}' not found or not active", provider_code)
            }));
        }
        Err(e) => {
            tracing::error!("Failed to get provider: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to get provider"
            }));
        }
    };

    // Check for running jobs
    let running_sql = r#"
        SELECT id FROM pod_sync_jobs
        WHERE provider_id = $1 AND status = 'running'
        LIMIT 1
    "#;
    if let Ok(rows) = client.query(running_sql, &[&provider_id]).await {
        if !rows.is_empty() {
            return HttpResponse::Conflict().json(serde_json::json!({
                "error": "A sync job is already running for this provider",
                "job_id": rows[0].get::<_, Uuid>("id")
            }));
        }
    }

    // Create new job
    let job_id = Uuid::new_v4();
    let job_type = &body.job_type;

    let insert_sql = r#"
        INSERT INTO pod_sync_jobs (id, provider_id, job_type, status, started_at)
        VALUES ($1, $2, $3, 'running', NOW())
        RETURNING id
    "#;

    match client.query_one(insert_sql, &[&job_id, &provider_id, job_type]).await {
        Ok(_) => {
            tracing::info!("Started sync job {} for provider {}", job_id, provider_code);

            HttpResponse::Accepted().json(serde_json::json!({
                "message": "Sync job started",
                "job_id": job_id,
                "provider": provider_code,
                "job_type": job_type,
                "status": "running"
            }))
        }
        Err(e) => {
            tracing::error!("Failed to create sync job: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to start sync job"
            }))
        }
    }
}

/// Get R2 storage status
pub async fn get_r2_status() -> HttpResponse {
    let account_id = std::env::var("R2_ACCOUNT_ID").ok();
    let access_key = std::env::var("R2_ACCESS_KEY_ID").ok();
    let bucket_name = std::env::var("R2_BUCKET_NAME").ok();

    let configured = account_id.is_some() && access_key.is_some();

    HttpResponse::Ok().json(R2StatusResponse {
        configured,
        bucket_name,
        account_id: account_id.map(|id| {
            if id.len() > 8 {
                format!("{}...", &id[..8])
            } else {
                id
            }
        }),
    })
}

/// Test R2 connectivity
pub async fn test_r2() -> HttpResponse {
    match R2Client::from_env().await {
        Ok(client) => {
            // Try to list objects (empty prefix = list root)
            match client.list("", Some(1)).await {
                Ok(keys) => {
                    HttpResponse::Ok().json(serde_json::json!({
                        "status": "ok",
                        "message": "R2 connection successful",
                        "bucket": client.bucket(),
                        "sample_keys": keys
                    }))
                }
                Err(e) => {
                    HttpResponse::ServiceUnavailable().json(serde_json::json!({
                        "status": "error",
                        "message": format!("R2 connection failed: {}", e),
                        "bucket": client.bucket()
                    }))
                }
            }
        }
        Err(e) => {
            HttpResponse::ServiceUnavailable().json(serde_json::json!({
                "status": "not_configured",
                "message": format!("R2 not configured: {}", e)
            }))
        }
    }
}

/// Upload test file to R2
pub async fn test_r2_upload() -> HttpResponse {
    match R2Client::from_env().await {
        Ok(client) => {
            let test_data = serde_json::json!({
                "test": true,
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "message": "R2 upload test from r-image-magic"
            });

            let path = AssetPath::base_image("test", "connectivity", "test.json");

            match client.upload(&path, serde_json::to_vec(&test_data).unwrap(), "application/json").await {
                Ok(result) => {
                    HttpResponse::Ok().json(serde_json::json!({
                        "status": "ok",
                        "message": "R2 upload successful",
                        "key": result.key,
                        "size": result.size,
                        "public_url": result.public_url
                    }))
                }
                Err(e) => {
                    HttpResponse::ServiceUnavailable().json(serde_json::json!({
                        "status": "error",
                        "message": format!("R2 upload failed: {}", e)
                    }))
                }
            }
        }
        Err(e) => {
            HttpResponse::ServiceUnavailable().json(serde_json::json!({
                "status": "not_configured",
                "message": format!("R2 not configured: {}", e)
            }))
        }
    }
}
