# Codex Agent: Full Printful Catalog Template Sync

## Objective

Run `scripts/sync-printful-templates.py` to enumerate ALL ~300+ Printful products, download their mockup templates, generate displacement maps, and create metadata.json files. Then verify the results and commit.

## Environment Setup

```bash
# Install Python dependencies
pip install requests Pillow numpy

# Set Printful API token (get from Doppler or .env)
export PRINTFUL_ACCESS_TOKEN="$PRINTFUL_ACCESS_TOKEN"
```

## Execution Steps

### Step 1: Verify API access
```bash
cd /Volumes/T7\ Storage/Projects/r-image-magic
python3 scripts/sync-printful-templates.py --dry-run --limit 5
```
Confirm it lists product names and template counts.

### Step 2: Run full sync
```bash
python3 scripts/sync-printful-templates.py --resume 2>&1 | tee sync-output.log
```

The `--resume` flag makes this idempotent — if interrupted, re-run the same command and it picks up where it left off. Progress is saved to `scripts/.sync-progress.json`.

**Expected behavior:**
- ~300 products processed
- ~1000-2000 templates created (each product has multiple templates/placements)
- Rate limited at ~100 API requests/minute
- Total runtime: ~30-45 minutes
- Each template creates a directory under `apps/api/assets/templates/` with:
  - `base.png` — product mockup template downloaded from Printful
  - `displacement.png` — auto-generated displacement map
  - `metadata.json` — template metadata with print area, dimensions, etc.

### Step 3: Verify output quality

Pick 5 random templates and verify:
```bash
# Count total templates
ls -d apps/api/assets/templates/*/ | wc -l

# Check a template has all 3 files
ls apps/api/assets/templates/hoodie-front-*/

# Validate a metadata.json
python3 -c "
import json, pathlib, random
dirs = sorted(pathlib.Path('apps/api/assets/templates').iterdir())
sample = random.sample([d for d in dirs if d.is_dir()], min(5, len(dirs)))
for d in sample:
    meta = d / 'metadata.json'
    if meta.exists():
        m = json.loads(meta.read_text())
        has_base = (d / 'base.png').exists()
        has_disp = (d / 'displacement.png').exists()
        print(f'{d.name}: base={has_base} disp={has_disp} area={m.get(\"print_area\")} product={m.get(\"product\")[:40]}')
    else:
        print(f'{d.name}: MISSING metadata.json')
"
```

### Step 4: Clean up and commit
```bash
# Remove progress file
rm -f scripts/.sync-progress.json

# Stage everything
git add apps/api/assets/templates/
git add scripts/sync-printful-templates.py

# Commit (the template images will be large — consider .gitattributes for LFS)
git commit -m "feat(templates): sync full Printful catalog (~300 products)

Adds mockup templates for all active Printful products with
auto-generated displacement maps and metadata.

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>"
```

## Rate Limit Safety

- Printful allows 120 requests/minute
- Script uses 0.6s delay between requests (~100 req/min effective)
- Built-in retry with exponential backoff on 429/5xx responses
- If rate limited, script sleeps for the Retry-After duration and continues

## Recovery

If the script crashes or is interrupted:
```bash
# Just re-run with --resume — picks up where it left off
python3 scripts/sync-printful-templates.py --resume
```

## Selective Sync

To sync only specific product types:
```bash
# Only t-shirts and hoodies
python3 scripts/sync-printful-templates.py --types tshirt hoodie

# Only specific product IDs
python3 scripts/sync-printful-templates.py --product-ids 71 257 380
```

## Notes

- Displacement maps are auto-generated using edge detection + Gaussian blur — they work well for line-art templates but may need manual refinement for photo-realistic templates
- The script does NOT modify any existing templates — it only creates new ones
- Template directory names follow the pattern: `{type}-{placement}-{template_id}` (e.g., `hoodie-front-47694`)
- All-over-print products get `-aop` suffix (e.g., `tshirt-aop-front-3920`)
