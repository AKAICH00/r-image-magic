#!/usr/bin/env python3
"""
Printful Full Catalog Template Sync
====================================

Self-contained script that enumerates ALL Printful products (~300+),
downloads mockup templates, generates displacement maps, and creates
metadata.json files in the r-image-magic template directory structure.

Designed to run autonomously (e.g. via Codex agent) with built-in rate
limiting, resumability, and error tolerance.

Usage:
    export PRINTFUL_ACCESS_TOKEN="your_token_here"
    python3 scripts/sync-printful-templates.py

    # Resume from where you left off (skips already-downloaded templates):
    python3 scripts/sync-printful-templates.py --resume

    # Only sync specific product types:
    python3 scripts/sync-printful-templates.py --types tshirt hoodie mug

    # Dry run (list what would be downloaded):
    python3 scripts/sync-printful-templates.py --dry-run

Requirements:
    pip install requests Pillow numpy

Rate limits:
    Printful API: 120 requests/minute. This script uses 0.6s delay between
    requests (~100 req/min effective) to stay safely under the limit.
"""

import argparse
import json
import logging
import os
import sys
import time
from pathlib import Path
from typing import Any

import numpy as np
import requests
from PIL import Image, ImageFilter

# ---------------------------------------------------------------------------
# Config
# ---------------------------------------------------------------------------

PRINTFUL_API_BASE = "https://api.printful.com"
TEMPLATE_DIR = Path(__file__).resolve().parent.parent / "apps" / "api" / "assets" / "templates"
PROGRESS_FILE = Path(__file__).resolve().parent / ".sync-progress.json"

# Rate limiting: 0.6s between requests = ~100 req/min (limit is 120)
REQUEST_DELAY_S = 0.6

# Displacement map generation params
DISPLACEMENT_BLUR_RADIUS = 6
DISPLACEMENT_EDGE_THRESHOLD = 15

# Map Printful type names to our category slugs
TYPE_SLUG_MAP: dict[str, str] = {
    "T-SHIRT": "tshirt",
    "SHIRT": "shirt",
    "HOODIE": "hoodie",
    "SWEATSHIRT": "sweatshirt",
    "TANK-TOP": "tank",
    "TANK TOP": "tank",
    "CROP-TOP": "crop-top",
    "DRESS": "dress",
    "SKIRT": "skirt",
    "LEGGINGS": "leggings",
    "SHORTS": "shorts",
    "JOGGERS": "joggers",
    "SWIMSUIT": "swimsuit",
    "MUG": "mug",
    "POSTER": "poster",
    "CANVAS": "canvas",
    "PHONE CASE": "phone-case",
    "PHONE-CASE": "phone-case",
    "LAPTOP CASE": "laptop-case",
    "TOTE BAG": "tote",
    "TOTE-BAG": "tote",
    "BACKPACK": "backpack",
    "FANNY PACK": "fanny-pack",
    "HAT": "hat",
    "BEANIE": "beanie",
    "VISOR": "visor",
    "SOCKS": "socks",
    "FLIP FLOPS": "flip-flops",
    "SHOES": "shoes",
    "TOWEL": "towel",
    "BLANKET": "blanket",
    "PILLOW": "pillow",
    "MOUSE PAD": "mousepad",
    "MOUSEPAD": "mousepad",
    "NOTEBOOK": "notebook",
    "STICKER": "sticker",
    "COASTER": "coaster",
    "ORNAMENT": "ornament",
    "PUZZLE": "puzzle",
    "FLAG": "flag",
    "BANNER": "banner",
    "APRON": "apron",
    "FACE MASK": "face-mask",
    "PET BED": "pet-bed",
    "PET BANDANA": "pet-bandana",
    "ONESIE": "onesie",
    "BIB": "bib",
    "WALL ART": "wall-art",
    "SHOWER CURTAIN": "shower-curtain",
    "BATHMAT": "bathmat",
    "WRAPPING PAPER": "wrapping-paper",
    "GIFT WRAP": "wrapping-paper",
    "BEAN BAG": "bean-bag",
    "DUVET COVER": "duvet-cover",
    "PILLOW CASE": "pillow-case",
    "RUG": "rug",
    "CUTTING BOARD": "cutting-board",
    "BOTTLE": "bottle",
    "TUMBLER": "tumbler",
    "WINE GLASS": "wine-glass",
    "CAN COOLER": "can-cooler",
}

# Placement name normalization
PLACEMENT_MAP: dict[str, str] = {
    "front": "front",
    "back": "back",
    "left": "left",
    "right": "right",
    "sleeve_left": "sleeve-left",
    "sleeve_right": "sleeve-right",
    "neck_label": "neck-label",
    "outside_label": "outside-label",
    "default": "front",
    "label_outside": "outside-label",
    "label_inside": "inside-label",
    "embroidery_front": "front",
    "embroidery_back": "back",
    "embroidery_left": "left",
    "embroidery_right": "right",
}

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s %(levelname)-5s %(message)s",
    datefmt="%H:%M:%S",
)
log = logging.getLogger("sync")

# ---------------------------------------------------------------------------
# API Client
# ---------------------------------------------------------------------------


class PrintfulClient:
    """Minimal Printful API client with built-in rate limiting.

    The catalog API (products, templates, printfiles) is public and does not
    require authentication. A token is optional — only needed for store-specific
    endpoints (orders, shipping, etc.).
    """

    def __init__(self, token: str | None = None):
        self.token = token
        self.session = requests.Session()
        headers: dict[str, str] = {"Content-Type": "application/json"}
        if token:
            headers["Authorization"] = f"Bearer {token}"
        self.session.headers.update(headers)
        self._last_request_time = 0.0
        self.request_count = 0

    def _throttle(self) -> None:
        elapsed = time.time() - self._last_request_time
        if elapsed < REQUEST_DELAY_S:
            time.sleep(REQUEST_DELAY_S - elapsed)
        self._last_request_time = time.time()

    def get(self, path: str, params: dict | None = None) -> dict:
        self._throttle()
        url = f"{PRINTFUL_API_BASE}{path}"
        self.request_count += 1

        for attempt in range(3):
            try:
                resp = self.session.get(url, params=params, timeout=30)
            except requests.RequestException as e:
                log.warning("Request failed (attempt %d): %s", attempt + 1, e)
                if attempt < 2:
                    time.sleep(2 ** (attempt + 1))
                    continue
                raise

            if resp.status_code == 429:
                retry_after = int(resp.headers.get("Retry-After", "60"))
                log.warning("Rate limited — sleeping %ds", retry_after)
                time.sleep(retry_after)
                continue

            if resp.status_code >= 500:
                log.warning("Server error %d (attempt %d)", resp.status_code, attempt + 1)
                if attempt < 2:
                    time.sleep(2 ** (attempt + 1))
                    continue

            resp.raise_for_status()
            return resp.json()

        raise RuntimeError(f"Failed after 3 attempts: {url}")

    def get_all_products(self) -> list[dict]:
        """Paginate through all products in the catalog."""
        products: list[dict] = []
        offset = 0
        limit = 100

        while True:
            log.info("Fetching products offset=%d limit=%d", offset, limit)
            data = self.get("/products", params={"offset": offset, "limit": limit})
            batch = data.get("result", [])
            products.extend(batch)

            paging = data.get("paging", {})
            total = paging.get("total", len(products))
            log.info("  Got %d products (total: %d)", len(batch), total)

            if not batch or offset + limit >= total:
                break
            offset += limit

        return products

    def get_product_detail(self, product_id: int) -> dict:
        data = self.get(f"/products/{product_id}")
        return data.get("result", {})

    def get_templates(self, product_id: int) -> dict:
        data = self.get(f"/mockup-generator/templates/{product_id}")
        return data.get("result", {})

    def get_printfiles(self, product_id: int) -> dict:
        data = self.get(f"/mockup-generator/printfiles/{product_id}")
        return data.get("result", {})


# ---------------------------------------------------------------------------
# Displacement Map Generator
# ---------------------------------------------------------------------------


def generate_displacement_map(base_img: Image.Image) -> Image.Image:
    """
    Generate a displacement map from a mockup template base image.

    Strategy:
    1. Convert to grayscale
    2. Edge-detect to find product contours and fabric folds
    3. Gaussian blur to create smooth displacement gradients
    4. Normalize to 8-bit range with neutral gray (128) as zero-displacement

    For Printful line-art templates (728x728 with dotted print areas), this
    produces a soft gradient around the product outline — matching the existing
    displacement maps in the repo.
    """
    gray = base_img.convert("L")
    arr = np.array(gray, dtype=np.float32)

    # Edge detection via Sobel-like gradient magnitude
    # Compute horizontal and vertical gradients
    gx = np.zeros_like(arr)
    gy = np.zeros_like(arr)
    gx[:, 1:-1] = arr[:, 2:] - arr[:, :-2]
    gy[1:-1, :] = arr[2:, :] - arr[:-2, :]
    magnitude = np.sqrt(gx**2 + gy**2)

    # Threshold: keep edges above threshold
    edges = (magnitude > DISPLACEMENT_EDGE_THRESHOLD).astype(np.float32) * 255

    # Convert back to PIL for Gaussian blur
    edge_img = Image.fromarray(edges.astype(np.uint8), mode="L")

    # Heavy Gaussian blur to create smooth displacement gradients
    blurred = edge_img.filter(ImageFilter.GaussianBlur(radius=DISPLACEMENT_BLUR_RADIUS))

    # Normalize: center at 128 (neutral), edges push toward 0 or 255
    blur_arr = np.array(blurred, dtype=np.float32)
    if blur_arr.max() > 0:
        # Normalize to 0-127 range, then add to 128 baseline
        normalized = (blur_arr / blur_arr.max()) * 80
        # Invert so product interior is lighter (pushes design inward)
        result = 128 + (80 - normalized)
        result = np.clip(result, 0, 255)
    else:
        result = np.full_like(blur_arr, 128)

    return Image.fromarray(result.astype(np.uint8), mode="L")


# ---------------------------------------------------------------------------
# Template Processing
# ---------------------------------------------------------------------------


def slugify_type(type_name: str) -> str:
    """Convert Printful type name to our slug format."""
    upper = type_name.upper().strip()
    if upper in TYPE_SLUG_MAP:
        return TYPE_SLUG_MAP[upper]
    # Fallback: lowercase, replace spaces with hyphens
    return type_name.lower().replace(" ", "-").replace("_", "-")


def normalize_placement(placement: str) -> str:
    """Normalize placement name."""
    key = placement.lower().strip()
    return PLACEMENT_MAP.get(key, key.replace(" ", "-"))


def detect_aop(product: dict) -> bool:
    """Detect if this is an all-over-print product."""
    title = (product.get("title") or "").lower()
    type_name = (product.get("type_name") or "").lower()
    return "all-over" in title or "all over" in title or "aop" in type_name


def build_template_id(product: dict, template: dict, placement: str) -> str:
    """Build our template directory name: {type}-{placement}-{template_id}"""
    type_slug = slugify_type(product.get("type_name", "unknown"))

    # Add -aop suffix for all-over-print products
    if detect_aop(product):
        type_slug = f"{type_slug}-aop"

    norm_placement = normalize_placement(placement)
    template_id = template.get("template_id", 0)

    return f"{type_slug}-{norm_placement}-{template_id}"


def build_metadata(
    template_dir_name: str,
    product: dict,
    template: dict,
    printfiles: dict,
    placement: str,
    img_width: int,
    img_height: int,
) -> dict[str, Any]:
    """Build metadata.json content for a template."""
    type_slug = slugify_type(product.get("type_name", "unknown"))
    if detect_aop(product):
        type_slug = f"{type_slug}-aop"

    positions = template.get("template_positions") or {}
    template_w = template.get("template_width") or img_width
    template_h = template.get("template_height") or img_height

    # Print area from template positions
    if positions:
        print_area = {
            "x": positions.get("left", 0),
            "y": positions.get("top", 0),
            "width": positions.get("width", template_w),
            "height": positions.get("height", template_h),
        }
    else:
        # Fallback: center 60% of image
        margin_x = int(template_w * 0.2)
        margin_y = int(template_h * 0.15)
        print_area = {
            "x": margin_x,
            "y": margin_y,
            "width": template_w - 2 * margin_x,
            "height": template_h - 2 * margin_y,
        }

    # Printfile dimensions (the actual print resolution)
    printfile_id = template.get("printfile_id")
    pf_list = printfiles.get("printfiles", [])
    matched_pf = next((pf for pf in pf_list if pf.get("printfile_id") == printfile_id), None)

    metadata: dict[str, Any] = {
        "id": template_dir_name,
        "name": f"{product.get('title', 'Unknown')} ({normalize_placement(placement).title()})",
        "version": 1,
        "category": type_slug,
        "color": "white",
        "color_hex": None,
        "placement": normalize_placement(placement),
        "product": product.get("title", "Unknown"),
        "product_type": type_slug,
        "printful_product_id": product.get("id"),
        "printful_template_id": template.get("template_id"),
        "dimensions": {
            "width": template_w,
            "height": template_h,
        },
        "print_area": print_area,
        "anchor_point": {
            "x": print_area["x"] + print_area["width"] // 2,
            "y": print_area["y"] + print_area["height"] // 2,
        },
        "displacement": {
            "enabled": True,
            "strength_default": 8.0,
            "strength_range": [4.0, 16.0],
        },
        "blend_mode": "multiply",
        "default_opacity": 240,
    }

    # Add printfile info if available
    if matched_pf:
        metadata["printfile"] = {
            "id": printfile_id,
            "width": matched_pf.get("width"),
            "height": matched_pf.get("height"),
            "dpi": matched_pf.get("dpi", 150),
            "fill_mode": matched_pf.get("fill_mode", "fit"),
        }

    return metadata


def download_image(url: str, dest: Path) -> Image.Image | None:
    """Download an image from URL and save to disk."""
    try:
        resp = requests.get(url, timeout=30)
        resp.raise_for_status()
        dest.write_bytes(resp.content)
        return Image.open(dest)
    except Exception as e:
        log.error("Failed to download %s: %s", url, e)
        return None


# ---------------------------------------------------------------------------
# Progress Tracking
# ---------------------------------------------------------------------------


def load_progress() -> dict:
    if PROGRESS_FILE.exists():
        return json.loads(PROGRESS_FILE.read_text())
    return {"completed_products": [], "completed_templates": [], "errors": []}


def save_progress(progress: dict) -> None:
    PROGRESS_FILE.write_text(json.dumps(progress, indent=2))


# ---------------------------------------------------------------------------
# Main Sync
# ---------------------------------------------------------------------------


def sync_product(
    client: PrintfulClient,
    product: dict,
    progress: dict,
    dry_run: bool = False,
) -> int:
    """
    Sync all templates for a single product.
    Returns count of new templates created.
    """
    product_id = product["id"]
    title = product.get("title", "Unknown")
    created = 0

    if product.get("is_discontinued"):
        log.info("  Skipping discontinued product: %s", title)
        return 0

    # Fetch templates and printfiles
    try:
        templates_data = client.get_templates(product_id)
        printfiles_data = client.get_printfiles(product_id)
    except Exception as e:
        log.error("  Failed to fetch templates for %s (#%d): %s", title, product_id, e)
        progress["errors"].append(
            {"product_id": product_id, "title": title, "error": str(e)}
        )
        return 0

    templates = templates_data.get("templates", [])
    if not templates:
        log.info("  No templates for: %s (#%d)", title, product_id)
        return 0

    # Determine available placements
    variant_mapping = templates_data.get("variant_mapping", [])

    for tmpl in templates:
        template_id = tmpl.get("template_id")
        image_url = tmpl.get("image_url", "")

        if not image_url:
            continue

        # Determine placement from printfile -> available_placements mapping
        printfile_id = tmpl.get("printfile_id")
        placements_raw = printfiles_data.get("available_placements", {})
        placement = "front"  # default

        if isinstance(placements_raw, dict):
            for pl_name, pl_info in placements_raw.items():
                if isinstance(pl_info, dict) and pl_info.get("printfile_id") == printfile_id:
                    placement = pl_name
                    break
                elif isinstance(pl_info, list):
                    for pf_entry in pl_info:
                        if isinstance(pf_entry, dict) and pf_entry.get("printfile_id") == printfile_id:
                            placement = pl_name
                            break

        dir_name = build_template_id(product, tmpl, placement)

        # Skip if already done
        if dir_name in progress["completed_templates"]:
            log.debug("  Already synced: %s", dir_name)
            continue

        template_path = TEMPLATE_DIR / dir_name

        if template_path.exists() and (template_path / "metadata.json").exists():
            log.debug("  Already exists on disk: %s", dir_name)
            progress["completed_templates"].append(dir_name)
            continue

        if dry_run:
            log.info("  [DRY RUN] Would create: %s", dir_name)
            created += 1
            continue

        # Create template directory
        template_path.mkdir(parents=True, exist_ok=True)
        log.info("  Creating template: %s", dir_name)

        # Download base image
        base_path = template_path / "base.png"
        base_img = download_image(image_url, base_path)

        if base_img is None:
            log.error("  Failed to download base image for %s", dir_name)
            # Clean up empty directory
            if base_path.exists():
                base_path.unlink()
            template_path.rmdir()
            continue

        img_width, img_height = base_img.size

        # Generate displacement map
        try:
            disp_map = generate_displacement_map(base_img)
            disp_path = template_path / "displacement.png"
            disp_map.save(disp_path, "PNG")
            log.info("  Generated displacement map: %dx%d", disp_map.width, disp_map.height)
        except Exception as e:
            log.error("  Failed to generate displacement map for %s: %s", dir_name, e)
            # Still continue — template works without displacement

        # Build and write metadata
        metadata = build_metadata(
            dir_name, product, tmpl, printfiles_data, placement, img_width, img_height
        )
        metadata_path = template_path / "metadata.json"
        metadata_path.write_text(json.dumps(metadata, indent=2))

        # Download background image if available
        bg_url = tmpl.get("background_url")
        if bg_url:
            bg_path = template_path / "background.png"
            download_image(bg_url, bg_path)

        progress["completed_templates"].append(dir_name)
        created += 1

    return created


def main():
    parser = argparse.ArgumentParser(description="Sync Printful catalog templates")
    parser.add_argument("--resume", action="store_true", help="Resume from progress file")
    parser.add_argument("--dry-run", action="store_true", help="List templates without downloading")
    parser.add_argument("--types", nargs="*", help="Only sync specific product types (e.g., tshirt hoodie mug)")
    parser.add_argument("--product-ids", nargs="*", type=int, help="Only sync specific product IDs")
    parser.add_argument("--limit", type=int, default=0, help="Max products to process (0 = all)")
    parser.add_argument("--verbose", "-v", action="store_true", help="Verbose logging")
    args = parser.parse_args()

    if args.verbose:
        logging.getLogger().setLevel(logging.DEBUG)

    # Token is optional — catalog API is public
    token = os.environ.get("PRINTFUL_ACCESS_TOKEN")

    # Init client
    client = PrintfulClient(token)

    # Verify API access with a lightweight catalog call
    log.info("Verifying Printful API access...")
    try:
        test = client.get("/products", params={"limit": 1})
        log.info("API accessible — catalog has products")
        if token:
            try:
                store = client.get("/store")
                store_name = store.get("result", {}).get("name", "Unknown")
                log.info("Authenticated as store: %s", store_name)
            except Exception:
                log.info("Token provided but /store not accessible (catalog-only mode)")
    except Exception as e:
        log.error("API access failed: %s", e)
        sys.exit(1)

    # Load progress
    progress = load_progress() if args.resume else {"completed_products": [], "completed_templates": [], "errors": []}

    # Ensure template directory exists
    TEMPLATE_DIR.mkdir(parents=True, exist_ok=True)

    # Fetch full catalog
    log.info("Fetching full Printful product catalog...")
    all_products = client.get_all_products()
    log.info("Found %d products in catalog", len(all_products))

    # Filter by type if specified
    if args.types:
        type_set = {t.lower() for t in args.types}
        all_products = [
            p for p in all_products
            if slugify_type(p.get("type_name", "")).lower() in type_set
            or any(t in (p.get("type_name") or "").lower() for t in type_set)
        ]
        log.info("Filtered to %d products matching types: %s", len(all_products), args.types)

    # Filter by product ID if specified
    if args.product_ids:
        id_set = set(args.product_ids)
        all_products = [p for p in all_products if p.get("id") in id_set]
        log.info("Filtered to %d products matching IDs: %s", len(all_products), args.product_ids)

    # Apply limit
    if args.limit > 0:
        all_products = all_products[: args.limit]
        log.info("Limited to first %d products", args.limit)

    # Process each product
    total_created = 0
    total_skipped = 0
    start_time = time.time()

    for i, product in enumerate(all_products, 1):
        product_id = product["id"]
        title = product.get("title", "Unknown")
        type_name = product.get("type_name", "Unknown")

        if product_id in progress["completed_products"] and args.resume:
            log.info("[%d/%d] Skipping (already done): %s", i, len(all_products), title)
            total_skipped += 1
            continue

        log.info(
            "[%d/%d] Processing: %s (type=%s, id=%d)",
            i, len(all_products), title, type_name, product_id,
        )

        created = sync_product(client, product, progress, dry_run=args.dry_run)
        total_created += created

        progress["completed_products"].append(product_id)
        save_progress(progress)

        # Log progress periodically
        if i % 25 == 0:
            elapsed = time.time() - start_time
            rate = i / elapsed * 60 if elapsed > 0 else 0
            log.info(
                "--- Progress: %d/%d products, %d templates created, %.0f products/min ---",
                i, len(all_products), total_created, rate,
            )

    # Summary
    elapsed = time.time() - start_time
    log.info("=" * 60)
    log.info("Sync complete!")
    log.info("  Products processed: %d", len(all_products) - total_skipped)
    log.info("  Products skipped (resume): %d", total_skipped)
    log.info("  Templates created: %d", total_created)
    log.info("  Errors: %d", len(progress["errors"]))
    log.info("  API requests: %d", client.request_count)
    log.info("  Elapsed: %.1f minutes", elapsed / 60)
    log.info("  Template dir: %s", TEMPLATE_DIR)

    if progress["errors"]:
        log.info("Errors:")
        for err in progress["errors"]:
            log.info("  - Product #%s (%s): %s", err["product_id"], err["title"], err["error"])

    # Clean up progress file on successful full run
    if not args.resume and not progress["errors"]:
        PROGRESS_FILE.unlink(missing_ok=True)


if __name__ == "__main__":
    main()
