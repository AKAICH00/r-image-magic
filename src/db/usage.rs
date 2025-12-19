//! Usage tracking and rate limiting database operations

use super::pool::{DbPool, DbError};
use chrono::{DateTime, Utc, Datelike, Timelike};
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;
use std::net::IpAddr;

/// Usage log entry for recording API requests
#[derive(Debug)]
pub struct UsageLogEntry {
    pub api_key_id: Uuid,
    pub endpoint: String,
    pub method: String,
    pub template_id: Option<String>,
    pub status_code: i32,
    pub response_time_ms: Option<i32>,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
    pub ip_address: Option<IpAddr>,
    pub user_agent: Option<String>,
}

/// Monthly usage summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthlyUsageSummary {
    pub year_month: String,
    pub total_requests: i32,
    pub successful_requests: i32,
    pub failed_requests: i32,
    pub billable_requests: i32,
    pub overage_requests: i32,
}

/// Usage statistics for an API key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageStats {
    pub api_key_id: Uuid,
    pub current_month: MonthlyUsageSummary,
    pub quota: i32,
    pub quota_remaining: i32,
    pub quota_percentage_used: f64,
}

/// Rate limit check result
#[derive(Debug, Clone)]
pub struct RateLimitStatus {
    pub allowed: bool,
    pub current_count: i32,
    pub limit: i32,
    pub reset_at: DateTime<Utc>,
}

/// Repository for usage tracking operations
pub struct UsageRepository {
    pool: DbPool,
}

impl UsageRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    /// Log a usage entry
    pub async fn log_usage(&self, entry: UsageLogEntry) -> Result<(), DbError> {
        let client = self.pool.get().await?;

        // Convert IpAddr to string for storage
        let ip_str = entry.ip_address.map(|ip| ip.to_string());

        client.execute(
            r#"
            INSERT INTO usage_logs (
                api_key_id, endpoint, method, template_id,
                status_code, response_time_ms, error_code, error_message,
                ip_address, user_agent
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9::inet, $10)
            "#,
            &[
                &entry.api_key_id,
                &entry.endpoint,
                &entry.method,
                &entry.template_id,
                &entry.status_code,
                &entry.response_time_ms,
                &entry.error_code,
                &entry.error_message,
                &ip_str,
                &entry.user_agent,
            ]
        ).await?;

        // Also update monthly aggregation
        let success = entry.status_code >= 200 && entry.status_code < 400;
        self.increment_monthly_usage(entry.api_key_id, success).await?;

        Ok(())
    }

    /// Increment monthly usage counter
    async fn increment_monthly_usage(&self, api_key_id: Uuid, success: bool) -> Result<(), DbError> {
        let client = self.pool.get().await?;

        let now = Utc::now();
        let year_month = format!("{:04}-{:02}", now.year(), now.month());

        // Get quota for API key
        let quota_row = client.query_one(
            "SELECT monthly_quota FROM api_keys WHERE id = $1",
            &[&api_key_id]
        ).await?;
        let quota: i32 = quota_row.get("monthly_quota");

        // Upsert monthly usage
        client.execute(
            r#"
            INSERT INTO monthly_usage (
                api_key_id, year_month, total_requests,
                successful_requests, failed_requests, billable_requests, overage_requests
            ) VALUES (
                $1, $2, 1,
                CASE WHEN $3 THEN 1 ELSE 0 END,
                CASE WHEN $3 THEN 0 ELSE 1 END,
                1, 0
            )
            ON CONFLICT (api_key_id, year_month) DO UPDATE SET
                total_requests = monthly_usage.total_requests + 1,
                successful_requests = monthly_usage.successful_requests + CASE WHEN $3 THEN 1 ELSE 0 END,
                failed_requests = monthly_usage.failed_requests + CASE WHEN $3 THEN 0 ELSE 1 END,
                billable_requests = LEAST(monthly_usage.billable_requests + 1, $4),
                overage_requests = GREATEST(monthly_usage.total_requests + 1 - $4, 0),
                updated_at = NOW()
            "#,
            &[&api_key_id, &year_month, &success, &quota]
        ).await?;

        Ok(())
    }

    /// Get current month's usage for an API key
    pub async fn get_current_month_usage(&self, api_key_id: Uuid) -> Result<MonthlyUsageSummary, DbError> {
        let client = self.pool.get().await?;

        let now = Utc::now();
        let year_month = format!("{:04}-{:02}", now.year(), now.month());

        let row = client.query_opt(
            r#"
            SELECT year_month, total_requests, successful_requests, failed_requests,
                   billable_requests, overage_requests
            FROM monthly_usage
            WHERE api_key_id = $1 AND year_month = $2
            "#,
            &[&api_key_id, &year_month]
        ).await?;

        Ok(row.map(|r| MonthlyUsageSummary {
            year_month: r.get("year_month"),
            total_requests: r.get("total_requests"),
            successful_requests: r.get("successful_requests"),
            failed_requests: r.get("failed_requests"),
            billable_requests: r.get("billable_requests"),
            overage_requests: r.get("overage_requests"),
        }).unwrap_or(MonthlyUsageSummary {
            year_month,
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            billable_requests: 0,
            overage_requests: 0,
        }))
    }

    /// Get usage statistics for an API key
    pub async fn get_usage_stats(&self, api_key_id: Uuid, quota: i32) -> Result<UsageStats, DbError> {
        let current_month = self.get_current_month_usage(api_key_id).await?;

        let quota_remaining = quota - current_month.total_requests;
        let quota_percentage = (current_month.total_requests as f64 / quota as f64) * 100.0;

        Ok(UsageStats {
            api_key_id,
            current_month,
            quota,
            quota_remaining: quota_remaining.max(0),
            quota_percentage_used: quota_percentage.min(100.0),
        })
    }

    /// Get monthly usage history for an API key
    pub async fn get_monthly_history(
        &self,
        api_key_id: Uuid,
        months: i32
    ) -> Result<Vec<MonthlyUsageSummary>, DbError> {
        let client = self.pool.get().await?;

        let rows = client.query(
            r#"
            SELECT year_month, total_requests, successful_requests, failed_requests,
                   billable_requests, overage_requests
            FROM monthly_usage
            WHERE api_key_id = $1
            ORDER BY year_month DESC
            LIMIT $2
            "#,
            &[&api_key_id, &months]
        ).await?;

        Ok(rows.iter().map(|r| MonthlyUsageSummary {
            year_month: r.get("year_month"),
            total_requests: r.get("total_requests"),
            successful_requests: r.get("successful_requests"),
            failed_requests: r.get("failed_requests"),
            billable_requests: r.get("billable_requests"),
            overage_requests: r.get("overage_requests"),
        }).collect())
    }

    /// Check rate limit using sliding window in database
    pub async fn check_rate_limit(
        &self,
        api_key_id: Uuid,
        limit: i32
    ) -> Result<RateLimitStatus, DbError> {
        let client = self.pool.get().await?;

        let now = Utc::now();
        // Round down to minute boundary for sliding window
        let window_start = now
            .with_nanosecond(0).unwrap()
            .with_second(0).unwrap();

        // Count requests in current window
        let count_row = client.query_one(
            r#"
            SELECT COALESCE(SUM(request_count), 0)::INTEGER as count
            FROM rate_limit_windows
            WHERE api_key_id = $1 AND window_start > NOW() - INTERVAL '1 minute'
            "#,
            &[&api_key_id]
        ).await?;

        let current_count: i32 = count_row.get("count");
        let allowed = current_count < limit;

        // If allowed, increment counter
        if allowed {
            client.execute(
                r#"
                INSERT INTO rate_limit_windows (api_key_id, window_start, request_count)
                VALUES ($1, $2, 1)
                ON CONFLICT (api_key_id, window_start) DO UPDATE
                SET request_count = rate_limit_windows.request_count + 1
                "#,
                &[&api_key_id, &window_start]
            ).await?;
        }

        // Calculate reset time (next minute)
        let reset_at = window_start + chrono::Duration::minutes(1);

        Ok(RateLimitStatus {
            allowed,
            current_count: if allowed { current_count + 1 } else { current_count },
            limit,
            reset_at,
        })
    }

    /// Check if API key has exceeded monthly quota
    pub async fn check_quota(&self, api_key_id: Uuid, quota: i32) -> Result<bool, DbError> {
        let current = self.get_current_month_usage(api_key_id).await?;
        Ok(current.total_requests < quota)
    }

    /// Clean up old rate limit windows (call periodically)
    pub async fn cleanup_rate_limits(&self) -> Result<u64, DbError> {
        let client = self.pool.get().await?;

        let result = client.execute(
            "DELETE FROM rate_limit_windows WHERE window_start < NOW() - INTERVAL '5 minutes'",
            &[]
        ).await?;

        if result > 0 {
            info!(deleted = result, "Cleaned up old rate limit windows");
        }

        Ok(result)
    }

    /// Clean up old usage logs (call periodically for data retention)
    pub async fn cleanup_old_logs(&self, retention_days: i32) -> Result<u64, DbError> {
        let client = self.pool.get().await?;

        let result = client.execute(
            "DELETE FROM usage_logs WHERE created_at < NOW() - ($1 || ' days')::INTERVAL",
            &[&retention_days.to_string()]
        ).await?;

        if result > 0 {
            info!(deleted = result, retention_days, "Cleaned up old usage logs");
        }

        Ok(result)
    }
}
