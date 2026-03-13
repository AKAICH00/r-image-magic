-- R-Image-Magic POD Catalog Schema
-- Migration: 003_pod_catalog_schema.sql
-- Created: 2024-12-19
-- Purpose: Multi-provider POD (Print-on-Demand) catalog integration

-- ============================================================================
-- POD Providers Registry
-- ============================================================================
CREATE TABLE IF NOT EXISTS pod_providers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Provider identification
    code VARCHAR(50) UNIQUE NOT NULL,         -- 'printful', 'printify', 'gelato', 'spod', 'gooten'
    name VARCHAR(255) NOT NULL,

    -- API configuration
    api_base_url VARCHAR(500) NOT NULL,
    auth_type VARCHAR(50) NOT NULL,           -- 'oauth2', 'api_key', 'recipe_id'
    rate_limit_per_minute INTEGER NOT NULL,

    -- Status
    is_active BOOLEAN NOT NULL DEFAULT true,
    sync_enabled BOOLEAN NOT NULL DEFAULT false,

    -- Sync tracking
    last_sync_at TIMESTAMPTZ,
    sync_interval_hours INTEGER DEFAULT 24,

    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_pod_providers_code ON pod_providers(code);
CREATE INDEX IF NOT EXISTS idx_pod_providers_is_active ON pod_providers(is_active);

-- ============================================================================
-- Product Categories (unified across providers)
-- ============================================================================
CREATE TABLE IF NOT EXISTS product_categories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Category identification
    slug VARCHAR(100) UNIQUE NOT NULL,        -- 't-shirts', 'hoodies', 'mugs', etc.
    name VARCHAR(255) NOT NULL,
    description TEXT,

    -- Hierarchy support
    parent_category_id UUID REFERENCES product_categories(id),

    -- Display order
    sort_order INTEGER DEFAULT 0,

    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_product_categories_slug ON product_categories(slug);
CREATE INDEX IF NOT EXISTS idx_product_categories_parent ON product_categories(parent_category_id);

-- ============================================================================
-- POD Products (provider-specific product definitions)
-- ============================================================================
CREATE TABLE IF NOT EXISTS pod_products (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    provider_id UUID NOT NULL REFERENCES pod_providers(id) ON DELETE CASCADE,

    -- External identification
    external_product_id VARCHAR(255) NOT NULL,  -- Provider's product ID

    -- Category reference
    category_id UUID REFERENCES product_categories(id),

    -- Product info
    name VARCHAR(500) NOT NULL,
    description TEXT,
    brand VARCHAR(255),
    model VARCHAR(255),

    -- Product type classification
    product_type VARCHAR(100) NOT NULL,         -- 'tshirt', 'hoodie', 'mug', 'poster'

    -- Availability
    is_available BOOLEAN NOT NULL DEFAULT true,
    regions JSONB DEFAULT '[]',                 -- Available regions ['US', 'EU', 'ASIA']

    -- Pricing (base price in cents)
    base_price_cents INTEGER,
    currency VARCHAR(3) DEFAULT 'USD',

    -- Metadata from provider (raw JSON response)
    provider_metadata JSONB DEFAULT '{}',

    -- Sync tracking
    last_synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    sync_hash VARCHAR(64),                      -- SHA-256 hash of provider data for change detection

    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE(provider_id, external_product_id)
);

CREATE INDEX IF NOT EXISTS idx_pod_products_provider ON pod_products(provider_id);
CREATE INDEX IF NOT EXISTS idx_pod_products_category ON pod_products(category_id);
CREATE INDEX IF NOT EXISTS idx_pod_products_type ON pod_products(product_type);
CREATE INDEX IF NOT EXISTS idx_pod_products_is_available ON pod_products(is_available);
CREATE INDEX IF NOT EXISTS idx_pod_products_sync ON pod_products(last_synced_at);

-- ============================================================================
-- Product Variants (size/color combinations)
-- ============================================================================
CREATE TABLE IF NOT EXISTS pod_product_variants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    product_id UUID NOT NULL REFERENCES pod_products(id) ON DELETE CASCADE,

    -- External identification
    external_variant_id VARCHAR(255) NOT NULL,

    -- Variant attributes
    sku VARCHAR(255),
    size VARCHAR(50),
    color_name VARCHAR(100),
    color_hex VARCHAR(7),                       -- e.g., '#FFFFFF'

    -- Availability and pricing
    is_available BOOLEAN NOT NULL DEFAULT true,
    price_cents INTEGER,

    -- Inventory (if tracked)
    in_stock BOOLEAN DEFAULT true,

    -- Provider metadata
    provider_metadata JSONB DEFAULT '{}',

    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE(product_id, external_variant_id)
);

CREATE INDEX IF NOT EXISTS idx_pod_variants_product ON pod_product_variants(product_id);
CREATE INDEX IF NOT EXISTS idx_pod_variants_color ON pod_product_variants(color_name);
CREATE INDEX IF NOT EXISTS idx_pod_variants_size ON pod_product_variants(size);
CREATE INDEX IF NOT EXISTS idx_pod_variants_sku ON pod_product_variants(sku);

-- ============================================================================
-- Print Areas (per product, defines printable zones)
-- ============================================================================
CREATE TABLE IF NOT EXISTS pod_print_areas (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    product_id UUID NOT NULL REFERENCES pod_products(id) ON DELETE CASCADE,

    -- External identification (if provided by API)
    external_print_area_id VARCHAR(255),

    -- Area definition
    placement VARCHAR(50) NOT NULL,            -- 'front', 'back', 'sleeve_left', 'sleeve_right', 'hood', 'pocket'
    name VARCHAR(255) NOT NULL,

    -- Dimensions in pixels
    width_px INTEGER NOT NULL,
    height_px INTEGER NOT NULL,

    -- Position on mockup template (if provided)
    offset_x_px INTEGER DEFAULT 0,
    offset_y_px INTEGER DEFAULT 0,

    -- DPI requirements
    print_dpi INTEGER DEFAULT 300,

    -- Provider-specific print file format
    file_format VARCHAR(50) DEFAULT 'PNG',

    -- Additional constraints (max colors, technique, etc.)
    constraints JSONB DEFAULT '{}',

    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE(product_id, placement)
);

CREATE INDEX IF NOT EXISTS idx_pod_print_areas_product ON pod_print_areas(product_id);
CREATE INDEX IF NOT EXISTS idx_pod_print_areas_placement ON pod_print_areas(placement);

-- ============================================================================
-- Provider Mockup Assets (base images, displacement maps stored in R2)
-- ============================================================================
CREATE TABLE IF NOT EXISTS pod_mockup_assets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    product_id UUID NOT NULL REFERENCES pod_products(id) ON DELETE CASCADE,
    variant_id UUID REFERENCES pod_product_variants(id) ON DELETE SET NULL,

    -- Asset identification
    asset_type VARCHAR(50) NOT NULL,           -- 'base_image', 'mockup_template', 'printfile_preview', 'thumbnail'
    placement VARCHAR(50),                     -- 'front', 'back', etc. (NULL for product-level assets)

    -- Source URL (from provider)
    source_url TEXT NOT NULL,

    -- R2 storage location
    r2_bucket VARCHAR(255),
    r2_key VARCHAR(500),

    -- Asset metadata
    width_px INTEGER,
    height_px INTEGER,
    file_size_bytes BIGINT,
    content_type VARCHAR(100),
    checksum VARCHAR(64),                      -- SHA-256 hash of file content

    -- Processing status
    status VARCHAR(50) NOT NULL DEFAULT 'pending',  -- 'pending', 'downloading', 'downloaded', 'processed', 'failed'
    error_message TEXT,
    retry_count INTEGER DEFAULT 0,

    -- Download tracking
    downloaded_at TIMESTAMPTZ,
    processed_at TIMESTAMPTZ,

    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_pod_assets_product ON pod_mockup_assets(product_id);
CREATE INDEX IF NOT EXISTS idx_pod_assets_variant ON pod_mockup_assets(variant_id);
CREATE INDEX IF NOT EXISTS idx_pod_assets_status ON pod_mockup_assets(status);
CREATE INDEX IF NOT EXISTS idx_pod_assets_type ON pod_mockup_assets(asset_type);
CREATE INDEX IF NOT EXISTS idx_pod_assets_r2_key ON pod_mockup_assets(r2_key);

-- ============================================================================
-- Template Mappings (links POD products to our internal templates)
-- ============================================================================
CREATE TABLE IF NOT EXISTS pod_template_mappings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    template_id UUID NOT NULL REFERENCES templates(id) ON DELETE CASCADE,
    product_id UUID NOT NULL REFERENCES pod_products(id) ON DELETE CASCADE,
    variant_id UUID REFERENCES pod_product_variants(id) ON DELETE SET NULL,
    print_area_id UUID REFERENCES pod_print_areas(id) ON DELETE SET NULL,

    -- Generation metadata
    generation_source VARCHAR(50) NOT NULL DEFAULT 'auto_sync',  -- 'auto_sync', 'manual', 'custom'
    is_primary BOOLEAN DEFAULT false,          -- Primary template for this product

    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE(template_id, product_id, variant_id, print_area_id)
);

CREATE INDEX IF NOT EXISTS idx_pod_template_mappings_template ON pod_template_mappings(template_id);
CREATE INDEX IF NOT EXISTS idx_pod_template_mappings_product ON pod_template_mappings(product_id);

-- ============================================================================
-- Sync Jobs (tracking catalog sync operations)
-- ============================================================================
CREATE TABLE IF NOT EXISTS pod_sync_jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    provider_id UUID NOT NULL REFERENCES pod_providers(id) ON DELETE CASCADE,

    -- Job info
    job_type VARCHAR(50) NOT NULL,             -- 'full_catalog', 'incremental', 'product_detail', 'assets'
    status VARCHAR(50) NOT NULL DEFAULT 'pending',  -- 'pending', 'running', 'completed', 'failed', 'cancelled'

    -- Progress tracking
    total_items INTEGER DEFAULT 0,
    processed_items INTEGER DEFAULT 0,
    failed_items INTEGER DEFAULT 0,

    -- Timing
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,

    -- Error handling
    error_message TEXT,
    error_details JSONB,

    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_pod_sync_jobs_provider ON pod_sync_jobs(provider_id);
CREATE INDEX IF NOT EXISTS idx_pod_sync_jobs_status ON pod_sync_jobs(status);
CREATE INDEX IF NOT EXISTS idx_pod_sync_jobs_created ON pod_sync_jobs(created_at DESC);

-- ============================================================================
-- Provider Rate Limit Tracking
-- ============================================================================
CREATE TABLE IF NOT EXISTS provider_rate_limits (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    provider_id UUID NOT NULL REFERENCES pod_providers(id) ON DELETE CASCADE,
    window_start TIMESTAMPTZ NOT NULL,
    request_count INTEGER NOT NULL DEFAULT 1,

    UNIQUE(provider_id, window_start)
);

CREATE INDEX IF NOT EXISTS idx_provider_rate_limits_provider_window ON provider_rate_limits(provider_id, window_start DESC);

-- ============================================================================
-- Triggers for updated_at
-- ============================================================================
DROP TRIGGER IF EXISTS update_pod_providers_updated_at ON pod_providers;
CREATE TRIGGER update_pod_providers_updated_at
    BEFORE UPDATE ON pod_providers
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_pod_products_updated_at ON pod_products;
CREATE TRIGGER update_pod_products_updated_at
    BEFORE UPDATE ON pod_products
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_pod_product_variants_updated_at ON pod_product_variants;
CREATE TRIGGER update_pod_product_variants_updated_at
    BEFORE UPDATE ON pod_product_variants
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_pod_mockup_assets_updated_at ON pod_mockup_assets;
CREATE TRIGGER update_pod_mockup_assets_updated_at
    BEFORE UPDATE ON pod_mockup_assets
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- ============================================================================
-- Seed Data: POD Providers
-- ============================================================================
INSERT INTO pod_providers (code, name, api_base_url, auth_type, rate_limit_per_minute) VALUES
    ('printful', 'Printful', 'https://api.printful.com', 'oauth2', 120),
    ('printify', 'Printify', 'https://api.printify.com/v1', 'oauth2', 600),
    ('gelato', 'Gelato', 'https://order.gelatoapis.com', 'api_key', 300),
    ('spod', 'SPOD', 'https://api.spod.com', 'oauth2', 200),
    ('gooten', 'Gooten', 'https://api.gooten.com/v1', 'recipe_id', 300)
ON CONFLICT (code) DO UPDATE SET
    api_base_url = EXCLUDED.api_base_url,
    auth_type = EXCLUDED.auth_type,
    rate_limit_per_minute = EXCLUDED.rate_limit_per_minute,
    updated_at = NOW();

-- ============================================================================
-- Seed Data: Product Categories
-- ============================================================================
INSERT INTO product_categories (slug, name, description, sort_order) VALUES
    ('t-shirts', 'T-Shirts', 'All t-shirt styles including crew neck, v-neck, and more', 1),
    ('hoodies', 'Hoodies & Sweatshirts', 'Hooded sweatshirts, pullovers, and zip-ups', 2),
    ('tank-tops', 'Tank Tops', 'Sleeveless tops for all genders', 3),
    ('long-sleeves', 'Long Sleeve Shirts', 'Long sleeve t-shirts and tops', 4),
    ('mugs', 'Mugs & Drinkware', 'Coffee mugs, tumblers, and water bottles', 10),
    ('posters', 'Posters & Prints', 'Wall art, posters, and canvas prints', 20),
    ('phone-cases', 'Phone Cases', 'Cases for iPhone, Samsung, and other devices', 30),
    ('bags', 'Bags & Totes', 'Tote bags, backpacks, and duffle bags', 40),
    ('hats', 'Hats & Caps', 'Baseball caps, beanies, and bucket hats', 50),
    ('accessories', 'Accessories', 'Stickers, pins, patches, and more', 60)
ON CONFLICT (slug) DO NOTHING;

-- ============================================================================
-- Grant Permissions
-- ============================================================================
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO r_image_magic;
GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA public TO r_image_magic;
