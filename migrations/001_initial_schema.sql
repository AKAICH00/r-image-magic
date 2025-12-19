-- R-Image-Magic Initial Schema
-- Migration: 001_initial_schema.sql
-- Created: 2024-12-18

-- Templates table for mockup templates
CREATE TABLE IF NOT EXISTS templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    template_id VARCHAR(255) UNIQUE NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    product_type VARCHAR(100) NOT NULL,
    variant VARCHAR(100),
    color VARCHAR(50),
    print_area_x DOUBLE PRECISION NOT NULL DEFAULT 0,
    print_area_y DOUBLE PRECISION NOT NULL DEFAULT 0,
    print_area_width DOUBLE PRECISION NOT NULL,
    print_area_height DOUBLE PRECISION NOT NULL,
    base_image_path VARCHAR(500) NOT NULL,
    displacement_map_path VARCHAR(500),
    mask_path VARCHAR(500),
    width INTEGER NOT NULL,
    height INTEGER NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for common queries
CREATE INDEX IF NOT EXISTS idx_templates_product_type ON templates(product_type);
CREATE INDEX IF NOT EXISTS idx_templates_is_active ON templates(is_active);
CREATE INDEX IF NOT EXISTS idx_templates_template_id ON templates(template_id);

-- Seed initial templates from metadata
INSERT INTO templates (
    template_id, name, description, product_type, variant, color,
    print_area_x, print_area_y, print_area_width, print_area_height,
    base_image_path, displacement_map_path, width, height, is_active
) VALUES
    (
        'white_male_front',
        'White Male T-Shirt Front',
        'Bella + Canvas 3001 - Unisex white t-shirt front view',
        'tshirt',
        'front',
        'white',
        550, 450, 1300, 1600,
        'assets/templates/white_male_front/base.png',
        'assets/templates/white_male_front/displacement.png',
        2400, 3200, true
    ),
    (
        'white_male_front_9x16',
        'White Male T-Shirt 9:16',
        'Bella + Canvas 3001 - Male white t-shirt in 9:16 aspect ratio',
        'tshirt',
        'front',
        'white',
        248, 443, 585, 720,
        'assets/templates/white_male_front_9x16/base.png',
        'assets/templates/white_male_front_9x16/displacement.png',
        1080, 1920, true
    ),
    (
        'white_female_front_9x16',
        'White Female T-Shirt 9:16',
        'Bella + Canvas 6004 - Female white t-shirt in 9:16 aspect ratio',
        'tshirt',
        'front',
        'white',
        248, 443, 585, 720,
        'assets/templates/white_female_front_9x16/base.png',
        'assets/templates/white_female_front_9x16/displacement.png',
        1080, 1920, true
    )
ON CONFLICT (template_id) DO UPDATE SET
    name = EXCLUDED.name,
    description = EXCLUDED.description,
    print_area_x = EXCLUDED.print_area_x,
    print_area_y = EXCLUDED.print_area_y,
    print_area_width = EXCLUDED.print_area_width,
    print_area_height = EXCLUDED.print_area_height,
    width = EXCLUDED.width,
    height = EXCLUDED.height,
    updated_at = NOW();
