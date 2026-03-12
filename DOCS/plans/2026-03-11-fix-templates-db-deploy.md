# Fix Templates, Database & Deploy Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Get all 58 templates loading (especially Bella Canvas 3001 AOP), fix the Aurora database, and deploy the updated service.

**Architecture:** Fix `TemplateMetadata` serde deserialization to handle optional/missing fields, create the `r_image_magic` database on Aurora, commit all changes (fmt + functional), push to trigger CI/CD deploy.

**Tech Stack:** Rust (actix-web, serde, tokio-postgres), AWS ECS/Aurora/Secrets Manager, GitHub Actions

---

### Task 1: Fix TemplateMetadata to Accept All Template JSON Fields

**Files:**
- Modify: `src/engine/template.rs:30-45`

The current struct requires `color_hex: String` and `gender: String`, but:
- Printful-synced templates have `color_hex: null` (20 templates)
- Gender-neutral products omit `gender` entirely (12 templates)
- Printful templates include extra fields (`name`, `product`, `product_type`, `printful_product_id`, `printful_template_id`) not in the struct — serde will error on unknown fields unless we allow them
- Working templates have a `zones` map not in the struct

**Step 1: Update the TemplateMetadata struct**

Change `src/engine/template.rs` lines 30-45 from:

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct TemplateMetadata {
    pub id: String,
    pub version: u32,
    pub category: String,
    pub color: String,
    pub color_hex: String,
    pub placement: String,
    pub gender: String,
    pub dimensions: TemplateDimensions,
    pub print_area: PrintArea,
    pub anchor_point: AnchorPoint,
    pub displacement: DisplacementConfig,
    pub blend_mode: String,
    pub default_opacity: u8,
}
```

To:

```rust
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)] // REMOVE this if present, or better: don't add it
pub struct TemplateMetadata {
    pub id: String,
    pub version: u32,
    pub category: String,
    pub color: String,
    #[serde(default)]
    pub color_hex: Option<String>,
    pub placement: String,
    #[serde(default)]
    pub gender: Option<String>,
    // Printful-synced fields (optional - only present in synced templates)
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub product: Option<String>,
    #[serde(default)]
    pub product_type: Option<String>,
    #[serde(default)]
    pub printful_product_id: Option<u64>,
    #[serde(default)]
    pub printful_template_id: Option<u64>,
    pub dimensions: TemplateDimensions,
    pub print_area: PrintArea,
    pub anchor_point: AnchorPoint,
    pub displacement: DisplacementConfig,
    pub blend_mode: String,
    pub default_opacity: u8,
    // Zones for sub-area targeting (optional)
    #[serde(default)]
    pub zones: Option<std::collections::HashMap<String, serde_json::Value>>,
}
```

**Step 2: Check for usages of `color_hex` and `gender` across codebase**

Run: `grep -rn '\.color_hex\|\.gender' src/`

Update any code that accesses these fields to handle `Option`. Most likely candidates:
- `src/api/handlers/` — if any handler serializes metadata
- `src/engine/compositor.rs` — if compositor uses color_hex for blending

For each usage, change `metadata.color_hex` to `metadata.color_hex.as_deref().unwrap_or("")` or similar.

**Step 3: Run cargo check**

Run: `cargo check`
Expected: No errors. Warnings from fmt are OK.

**Step 4: Run cargo test (if tests exist)**

Run: `cargo test`
Expected: All pass (or no tests — the project may not have tests yet).

**Step 5: Commit**

```bash
git add src/engine/template.rs
git commit -m "fix(templates): make color_hex and gender optional, accept Printful metadata fields

Templates from Printful sync have color_hex: null and may omit gender
for gender-neutral products. Also accept name, product, product_type,
printful_product_id, printful_template_id, and zones fields."
```

---

### Task 2: Create Aurora Database and Schema

**Context:**
- Aurora proxy: `akaich00-rds-proxy.proxy-cu76keyg87x5.us-east-1.rds.amazonaws.com:5432`
- Credentials in Secrets Manager: `r-image-magic/DATABASE_URL-Zz84CO`
- DB user: `adminuser`, password URL-encoded in the secret
- Default database on cluster: `main`
- Needed database: `r_image_magic`

**Step 1: Connect to Aurora and create database**

Use psql or similar through the RDS proxy. The DATABASE_URL from Secrets Manager connects to `r_image_magic` which doesn't exist, so connect to `main` first:

```bash
# Get the secret value
doppler run -p ezer-mirror -c prd -- aws secretsmanager get-secret-value \
  --secret-id "r-image-magic/DATABASE_URL-Zz84CO" --region us-east-1 \
  --query SecretString --output text

# Modify the URL to connect to 'main' database, then:
psql "<url-with-main-instead-of-r_image_magic>" -c "CREATE DATABASE r_image_magic;"
```

**Step 2: Create the templates table**

Connect to `r_image_magic` and create the schema based on `src/db/queries.rs`:

```sql
CREATE TABLE IF NOT EXISTS templates (
    id SERIAL PRIMARY KEY,
    template_id VARCHAR(255) UNIQUE NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    product_type VARCHAR(100) NOT NULL,
    variant VARCHAR(100),
    color VARCHAR(100),
    print_area_x INTEGER NOT NULL,
    print_area_y INTEGER NOT NULL,
    print_area_width INTEGER NOT NULL,
    print_area_height INTEGER NOT NULL,
    base_image_path TEXT,
    displacement_map_path TEXT,
    mask_path TEXT,
    width INTEGER NOT NULL,
    height INTEGER NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_templates_product_type ON templates(product_type);
CREATE INDEX IF NOT EXISTS idx_templates_template_id ON templates(template_id);
CREATE INDEX IF NOT EXISTS idx_templates_is_active ON templates(is_active);
```

**Step 3: Seed templates from metadata.json files**

Write a quick script or SQL to insert all 58 templates' metadata into the DB so the `/api/v1/templates` endpoint works.

**Step 4: Verify connection**

Force new ECS deployment to test DB connectivity (or just wait for the next deploy in Task 4).

---

### Task 3: Commit All Working Tree Changes

**Step 1: Commit cargo fmt changes**

```bash
# Stage only .rs files (the formatting changes)
git add src/
git commit -m "chore: apply cargo fmt formatting"
```

**Step 2: Commit functional changes**

```bash
git add src/config/mod.rs src/main.rs Cargo.toml config/default.toml Dockerfile
git commit -m "fix(config): resolve settings from ECS env vars, install rustls CryptoProvider

- Add DATABASE_URL fallback chain for ECS Secrets Manager injection
- Add R2 settings resolution from env vars
- Install rustls ring CryptoProvider at startup for TLS
- Remove hardcoded database credentials from default.toml"
```

**Step 3: Commit deploy workflow and scripts**

```bash
git add .github/workflows/deploy.yml k8s/deployment.yaml
git add ecs/ scripts/ Dockerfile.prebuilt
git commit -m "ci(deploy): add staging/production environments with templated ECS task def

- Support workflow_dispatch with environment chooser
- Render task def from ecs/task-definition.json template
- Inject secrets via AWS Secrets Manager ARNs
- Add bootstrap and orchestration scripts"
```

---

### Task 4: Build and Deploy

**Step 1: Push to main**

```bash
git push origin main
```

This triggers the GitHub Actions deploy workflow.

**Step 2: Monitor deployment**

```bash
# Watch workflow
gh run watch

# Or check ECS
doppler run -p ezer-mirror -c prd -- aws ecs describe-services \
  --cluster akaich00-cluster --services r-image-magic --region us-east-1 \
  --query 'services[0].{running:runningCount,desired:desiredCount,events:events[:3]}'
```

**Step 3: Verify all templates load**

```bash
curl -s https://api.meetmockup.com/health | python3 -m json.tool
# Expect: templates_loaded: 58 (was 3)
```

**Step 4: Verify generate endpoint**

```bash
curl -s -X POST https://api.meetmockup.com/api/v1/mockups/generate \
  -H 'Content-Type: application/json' \
  -d '{
    "design_url": "https://via.placeholder.com/600x600/ff0000/ffffff",
    "template_id": "tshirt-aop-front-3920",
    "placement": {
      "x": 0, "y": 0, "width": 100, "height": 100,
      "print_area_width": 2332, "print_area_height": 3000
    }
  }'
```

**Step 5: Verify templates API (requires DB)**

```bash
curl -s https://api.meetmockup.com/api/v1/templates | python3 -m json.tool | head -20
# Expect: success: true, with template data
```

---

### Task 5: Password Rotation (Post-Deploy)

**Step 1: Rotate the leaked password**

The password `RImageMagic2024Pass` is in git history. After confirming the new deploy works:

1. Change the Aurora user password
2. Update the Secrets Manager secret with the new password
3. Force new ECS deployment to pick up the new secret

---

### Notes

- **No back-view templates exist yet.** All 58 templates are front-only. To get Bella Canvas back views, run the Printful sync module (`/api/v1/sync/printful/start`) after deploy — the sync code should fetch additional placements.
- The in-memory `TemplateManager` is used by the generate endpoint. The DB `TemplateRepository` is used by the listing/browse API. Both need to work.
- The health check only reports in-memory template count, not DB templates.
