-- R-Image-Magic SaaS Schema
-- Migration: 002_saas_schema.sql
-- Created: 2024-12-18
-- Purpose: API keys, usage tracking, and billing infrastructure

-- API Keys table for authentication
CREATE TABLE IF NOT EXISTS api_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Key identification
    key_prefix VARCHAR(12) NOT NULL,          -- First 12 chars for fast lookup (e.g., "rim_abc12345")
    key_hash VARCHAR(64) NOT NULL UNIQUE,     -- SHA-256 hash of full key
    name VARCHAR(255) NOT NULL,               -- User-friendly name for the key

    -- Owner information
    owner_email VARCHAR(255) NOT NULL,
    owner_name VARCHAR(255),
    company VARCHAR(255),

    -- Tier and limits
    tier VARCHAR(50) NOT NULL DEFAULT 'free', -- free, starter, pro, enterprise
    rate_limit_per_minute INTEGER NOT NULL DEFAULT 10,
    monthly_quota INTEGER NOT NULL DEFAULT 100,

    -- Status
    is_active BOOLEAN NOT NULL DEFAULT true,

    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_used_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ                    -- NULL means never expires
);

-- Indexes for API keys
CREATE INDEX IF NOT EXISTS idx_api_keys_prefix ON api_keys(key_prefix);
CREATE INDEX IF NOT EXISTS idx_api_keys_owner_email ON api_keys(owner_email);
CREATE INDEX IF NOT EXISTS idx_api_keys_tier ON api_keys(tier);
CREATE INDEX IF NOT EXISTS idx_api_keys_is_active ON api_keys(is_active);

-- Usage logs - per-request tracking
CREATE TABLE IF NOT EXISTS usage_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    api_key_id UUID NOT NULL REFERENCES api_keys(id) ON DELETE CASCADE,

    -- Request details
    endpoint VARCHAR(100) NOT NULL,
    method VARCHAR(10) NOT NULL DEFAULT 'POST',
    template_id VARCHAR(255),

    -- Response details
    status_code INTEGER NOT NULL,
    response_time_ms INTEGER,
    error_code VARCHAR(50),
    error_message TEXT,

    -- Request metadata
    ip_address INET,
    user_agent VARCHAR(500),

    -- Timestamp
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for usage logs (partitioned by time for performance)
CREATE INDEX IF NOT EXISTS idx_usage_logs_api_key_id ON usage_logs(api_key_id);
CREATE INDEX IF NOT EXISTS idx_usage_logs_created_at ON usage_logs(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_usage_logs_api_key_created ON usage_logs(api_key_id, created_at DESC);

-- Monthly usage aggregation for billing
CREATE TABLE IF NOT EXISTS monthly_usage (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    api_key_id UUID NOT NULL REFERENCES api_keys(id) ON DELETE CASCADE,
    year_month VARCHAR(7) NOT NULL,           -- Format: '2024-12'

    -- Counters
    total_requests INTEGER NOT NULL DEFAULT 0,
    successful_requests INTEGER NOT NULL DEFAULT 0,
    failed_requests INTEGER NOT NULL DEFAULT 0,

    -- Billing
    billable_requests INTEGER NOT NULL DEFAULT 0,
    overage_requests INTEGER NOT NULL DEFAULT 0,

    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE(api_key_id, year_month)
);

CREATE INDEX IF NOT EXISTS idx_monthly_usage_api_key ON monthly_usage(api_key_id);
CREATE INDEX IF NOT EXISTS idx_monthly_usage_year_month ON monthly_usage(year_month);

-- Generated mockups history (for user dashboard)
CREATE TABLE IF NOT EXISTS generations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    api_key_id UUID NOT NULL REFERENCES api_keys(id) ON DELETE CASCADE,

    -- Generation details
    template_id VARCHAR(255) NOT NULL,
    design_url TEXT NOT NULL,
    result_url TEXT,

    -- Placement info (stored as JSON)
    placement_json JSONB NOT NULL,
    options_json JSONB,

    -- Performance metrics
    generation_time_ms INTEGER,
    image_size_bytes INTEGER,

    -- Status
    status VARCHAR(20) NOT NULL DEFAULT 'completed', -- pending, processing, completed, failed
    error_message TEXT,

    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_generations_api_key ON generations(api_key_id);
CREATE INDEX IF NOT EXISTS idx_generations_created ON generations(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_generations_status ON generations(status);

-- Rate limit tracking (for sliding window)
CREATE TABLE IF NOT EXISTS rate_limit_windows (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    api_key_id UUID NOT NULL REFERENCES api_keys(id) ON DELETE CASCADE,
    window_start TIMESTAMPTZ NOT NULL,
    request_count INTEGER NOT NULL DEFAULT 1,

    UNIQUE(api_key_id, window_start)
);

CREATE INDEX IF NOT EXISTS idx_rate_limit_api_key_window ON rate_limit_windows(api_key_id, window_start DESC);

-- Function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Triggers for updated_at
DROP TRIGGER IF EXISTS update_api_keys_updated_at ON api_keys;
CREATE TRIGGER update_api_keys_updated_at
    BEFORE UPDATE ON api_keys
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_monthly_usage_updated_at ON monthly_usage;
CREATE TRIGGER update_monthly_usage_updated_at
    BEFORE UPDATE ON monthly_usage
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Function to increment monthly usage
CREATE OR REPLACE FUNCTION increment_monthly_usage(
    p_api_key_id UUID,
    p_success BOOLEAN
)
RETURNS VOID AS $$
DECLARE
    v_year_month VARCHAR(7);
    v_quota INTEGER;
    v_current_total INTEGER;
BEGIN
    v_year_month := to_char(NOW(), 'YYYY-MM');

    -- Get quota for the API key
    SELECT monthly_quota INTO v_quota FROM api_keys WHERE id = p_api_key_id;

    -- Upsert monthly usage
    INSERT INTO monthly_usage (api_key_id, year_month, total_requests, successful_requests, failed_requests, billable_requests)
    VALUES (p_api_key_id, v_year_month, 1,
            CASE WHEN p_success THEN 1 ELSE 0 END,
            CASE WHEN p_success THEN 0 ELSE 1 END,
            1)
    ON CONFLICT (api_key_id, year_month) DO UPDATE SET
        total_requests = monthly_usage.total_requests + 1,
        successful_requests = monthly_usage.successful_requests + CASE WHEN p_success THEN 1 ELSE 0 END,
        failed_requests = monthly_usage.failed_requests + CASE WHEN p_success THEN 0 ELSE 1 END,
        billable_requests = LEAST(monthly_usage.billable_requests + 1, v_quota),
        overage_requests = GREATEST(monthly_usage.total_requests + 1 - v_quota, 0);
END;
$$ LANGUAGE plpgsql;

-- Create a default admin API key for testing
-- Key: rim_admin_test_key_do_not_use_in_production
-- This should be revoked in production!
INSERT INTO api_keys (
    key_prefix,
    key_hash,
    name,
    owner_email,
    tier,
    rate_limit_per_minute,
    monthly_quota
) VALUES (
    'rim_admi',
    'e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855', -- placeholder hash
    'Admin Test Key',
    'admin@r-image-magic.local',
    'enterprise',
    1000,
    1000000
) ON CONFLICT DO NOTHING;

-- Grant permissions
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO r_image_magic;
GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA public TO r_image_magic;
GRANT EXECUTE ON ALL FUNCTIONS IN SCHEMA public TO r_image_magic;
