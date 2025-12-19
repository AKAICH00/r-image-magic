//! Usage Statistics Handlers
//!
//! Endpoints for viewing API usage statistics, quotas, and billing info.

use actix_web::{web, HttpRequest, HttpResponse, HttpMessage};
use serde::{Deserialize, Serialize};
use tracing::warn;
use uuid::Uuid;

use crate::db::{UsageRepository, UsageStats, MonthlyUsageSummary, DbPool};
use crate::api::middleware::ApiKeyAuth;

/// Usage stats response
#[derive(Debug, Serialize)]
pub struct UsageStatsResponse {
    pub api_key_id: Uuid,
    pub tier: String,
    pub current_month: MonthlyUsageResponse,
    pub quota: QuotaInfo,
}

/// Monthly usage response
#[derive(Debug, Serialize)]
pub struct MonthlyUsageResponse {
    pub year_month: String,
    pub total_requests: i32,
    pub successful_requests: i32,
    pub failed_requests: i32,
    pub billable_requests: i32,
    pub overage_requests: i32,
}

impl From<MonthlyUsageSummary> for MonthlyUsageResponse {
    fn from(summary: MonthlyUsageSummary) -> Self {
        Self {
            year_month: summary.year_month,
            total_requests: summary.total_requests,
            successful_requests: summary.successful_requests,
            failed_requests: summary.failed_requests,
            billable_requests: summary.billable_requests,
            overage_requests: summary.overage_requests,
        }
    }
}

/// Quota information
#[derive(Debug, Serialize)]
pub struct QuotaInfo {
    pub monthly_quota: i32,
    pub used: i32,
    pub remaining: i32,
    pub percentage_used: f64,
    pub is_exceeded: bool,
}

/// Usage history response
#[derive(Debug, Serialize)]
pub struct UsageHistoryResponse {
    pub api_key_id: Uuid,
    pub months: Vec<MonthlyUsageResponse>,
}

/// Get current usage stats
/// GET /api/v1/usage
pub async fn get_usage_stats(
    req: HttpRequest,
    pool: web::Data<DbPool>,
) -> HttpResponse {
    let auth = match req.extensions().get::<ApiKeyAuth>().cloned() {
        Some(auth) => auth,
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "unauthorized",
                "message": "API key required"
            }));
        }
    };

    let repo = UsageRepository::new(pool.get_ref().clone());

    match repo.get_usage_stats(auth.key_id, auth.monthly_quota).await {
        Ok(stats) => {
            let response = UsageStatsResponse {
                api_key_id: stats.api_key_id,
                tier: auth.tier.clone(),
                current_month: MonthlyUsageResponse::from(stats.current_month),
                quota: QuotaInfo {
                    monthly_quota: stats.quota,
                    used: stats.quota - stats.quota_remaining,
                    remaining: stats.quota_remaining,
                    percentage_used: stats.quota_percentage_used,
                    is_exceeded: stats.quota_remaining <= 0,
                },
            };

            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            warn!(error = %e, "Failed to get usage stats");
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "internal_error",
                "message": "Failed to get usage stats"
            }))
        }
    }
}

/// Query params for usage history
#[derive(Debug, Deserialize)]
pub struct UsageHistoryQuery {
    #[serde(default = "default_months")]
    pub months: i32,
}

fn default_months() -> i32 {
    6
}

/// Get usage history
/// GET /api/v1/usage/history?months=6
pub async fn get_usage_history(
    req: HttpRequest,
    pool: web::Data<DbPool>,
    query: web::Query<UsageHistoryQuery>,
) -> HttpResponse {
    let auth = match req.extensions().get::<ApiKeyAuth>().cloned() {
        Some(auth) => auth,
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "unauthorized",
                "message": "API key required"
            }));
        }
    };

    let months = query.months.clamp(1, 24); // Limit to 24 months max
    let repo = UsageRepository::new(pool.get_ref().clone());

    match repo.get_monthly_history(auth.key_id, months).await {
        Ok(history) => {
            let response = UsageHistoryResponse {
                api_key_id: auth.key_id,
                months: history.into_iter().map(MonthlyUsageResponse::from).collect(),
            };

            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            warn!(error = %e, "Failed to get usage history");
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "internal_error",
                "message": "Failed to get usage history"
            }))
        }
    }
}

/// Get specific month usage
/// GET /api/v1/usage/month/{year_month}
pub async fn get_month_usage(
    req: HttpRequest,
    pool: web::Data<DbPool>,
    path: web::Path<String>,
) -> HttpResponse {
    let auth = match req.extensions().get::<ApiKeyAuth>().cloned() {
        Some(auth) => auth,
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "unauthorized",
                "message": "API key required"
            }));
        }
    };

    let year_month = path.into_inner();

    // Validate format (YYYY-MM)
    if !is_valid_year_month(&year_month) {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "invalid_format",
            "message": "Year-month must be in YYYY-MM format"
        }));
    }

    let repo = UsageRepository::new(pool.get_ref().clone());

    // Get history including the requested month
    match repo.get_monthly_history(auth.key_id, 24).await {
        Ok(history) => {
            if let Some(month_data) = history.into_iter().find(|m| m.year_month == year_month) {
                HttpResponse::Ok().json(MonthlyUsageResponse::from(month_data))
            } else {
                // Return empty data for the month if not found
                HttpResponse::Ok().json(MonthlyUsageResponse {
                    year_month: year_month.clone(),
                    total_requests: 0,
                    successful_requests: 0,
                    failed_requests: 0,
                    billable_requests: 0,
                    overage_requests: 0,
                })
            }
        }
        Err(e) => {
            warn!(error = %e, "Failed to get month usage");
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "internal_error",
                "message": "Failed to get month usage"
            }))
        }
    }
}

/// Validate year-month format
fn is_valid_year_month(s: &str) -> bool {
    if s.len() != 7 {
        return false;
    }

    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() != 2 {
        return false;
    }

    let year: Result<i32, _> = parts[0].parse();
    let month: Result<u32, _> = parts[1].parse();

    match (year, month) {
        (Ok(y), Ok(m)) => y >= 2020 && y <= 2100 && m >= 1 && m <= 12,
        _ => false,
    }
}

/// Billing summary response
#[derive(Debug, Serialize)]
pub struct BillingSummaryResponse {
    pub api_key_id: Uuid,
    pub tier: String,
    pub tier_quota: i32,
    pub current_month: BillingMonthInfo,
    pub pricing: PricingInfo,
}

#[derive(Debug, Serialize)]
pub struct BillingMonthInfo {
    pub year_month: String,
    pub billable_requests: i32,
    pub overage_requests: i32,
    pub estimated_cost: f64,
}

#[derive(Debug, Serialize)]
pub struct PricingInfo {
    pub tier_price: f64,
    pub overage_price_per_1k: f64,
    pub currency: String,
}

/// Get billing summary for current month
/// GET /api/v1/usage/billing
pub async fn get_billing_summary(
    req: HttpRequest,
    pool: web::Data<DbPool>,
) -> HttpResponse {
    let auth = match req.extensions().get::<ApiKeyAuth>().cloned() {
        Some(auth) => auth,
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "unauthorized",
                "message": "API key required"
            }));
        }
    };

    let repo = UsageRepository::new(pool.get_ref().clone());

    match repo.get_current_month_usage(auth.key_id).await {
        Ok(usage) => {
            // Calculate tier pricing
            let (tier_price, overage_price) = get_tier_pricing(&auth.tier);
            let overage_cost = (usage.overage_requests as f64 / 1000.0) * overage_price;
            let estimated_cost = tier_price + overage_cost;

            let response = BillingSummaryResponse {
                api_key_id: auth.key_id,
                tier: auth.tier.clone(),
                tier_quota: auth.monthly_quota,
                current_month: BillingMonthInfo {
                    year_month: usage.year_month,
                    billable_requests: usage.billable_requests,
                    overage_requests: usage.overage_requests,
                    estimated_cost,
                },
                pricing: PricingInfo {
                    tier_price,
                    overage_price_per_1k: overage_price,
                    currency: "USD".to_string(),
                },
            };

            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            warn!(error = %e, "Failed to get billing summary");
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "internal_error",
                "message": "Failed to get billing summary"
            }))
        }
    }
}

/// Get tier pricing
fn get_tier_pricing(tier: &str) -> (f64, f64) {
    match tier {
        "free" => (0.0, 0.0), // No overage on free tier
        "starter" => (29.0, 5.0),
        "pro" => (99.0, 3.0),
        "enterprise" => (0.0, 2.0), // Custom pricing, no base
        _ => (0.0, 0.0),
    }
}
