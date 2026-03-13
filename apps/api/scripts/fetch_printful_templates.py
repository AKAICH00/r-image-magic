#!/usr/bin/env python3
"""Fetch Printful mockup templates and create r-image-magic template packages."""

import os
import sys
import json
import urllib.request
import subprocess
from pathlib import Path

# Config
TEMPLATES_DIR = Path(__file__).parent.parent / "assets" / "templates"
PRINTFUL_API_KEY = os.environ.get("PRINTFUL_API_KEY", "")

PRODUCTS = [
    {"id": 257, "name": "AOP T-Shirt", "type": "tshirt-aop", "product": "Bella + Canvas 3001 AOP", "category": "t-shirt-aop", "displacement_strength": 8.0},
    {"id": 83, "name": "Pillow 18x18", "type": "pillow-18x18", "product": "Pillow 18x18", "category": "pillow", "displacement_strength": 4.0},
    {"id": 19, "name": "Mug 11oz", "type": "mug-11oz", "product": "Mug 11oz", "category": "mug", "displacement_strength": 2.0},
    {"id": 84, "name": "Tote Bag", "type": "tote-bag", "product": "Tote Bag", "category": "tote", "displacement_strength": 6.0},
    {"id": 380, "name": "AOP Hoodie", "type": "hoodie-aop", "product": "AOP Hoodie", "category": "hoodie-aop", "displacement_strength": 8.0},
    {"id": 601, "name": "Phone Case", "type": "phone-case", "product": "Phone Case", "category": "phone-case", "displacement_strength": 1.0},
    {"id": 1, "name": "Poster 18x24", "type": "poster-18x24", "product": "Poster 18x24", "category": "poster", "displacement_strength": 0.5},
]

def fetch_json(url):
    req = urllib.request.Request(url, headers={
        "Authorization": f"Bearer {PRINTFUL_API_KEY}",
        "Content-Type": "application/json"
    })
    with urllib.request.urlopen(req) as resp:
        return json.loads(resp.read())

def download_image(url, path):
    urllib.request.urlretrieve(url, path)

def generate_displacement_map(base_path, output_path, strength="medium"):
    """Generate displacement map using ImageMagick (available on macOS)."""
    # Convert to grayscale, edge detect, blur, normalize to 128-center
    subprocess.run([
        "convert", str(base_path),
        "-colorspace", "Gray",
        "-edge", "1",
        "-blur", "0x3",
        "-level", "40%,60%",  # narrow range around mid-gray
        "-evaluate", "set", "50%",  # start from 50% gray
        "-compose", "plus",
        str(base_path), "-colorspace", "Gray", "-edge", "1", "-blur", "0x3",
        "-level", "45%,55%",
        "-compose", "over", "-composite",
        str(output_path)
    ], check=False)
    
    # Simpler fallback: just grayscale + edge + normalize
    if not output_path.exists():
        subprocess.run([
            "convert", str(base_path),
            "-colorspace", "Gray",
            "-edge", "1", 
            "-blur", "0x4",
            "-normalize",
            "-evaluate", "multiply", "0.3",
            "-evaluate", "add", "35%",
            str(output_path)
        ], check=True)

def process_product(product):
    print(f"\n{'='*60}")
    print(f"Processing: {product['name']} (ID: {product['id']})")
    print(f"{'='*60}")
    
    try:
        data = fetch_json(f"https://api.printful.com/mockup-generator/templates/{product['id']}")
    except Exception as e:
        print(f"  ERROR fetching templates: {e}")
        return []
    
    templates = data.get("result", {}).get("templates", [])
    print(f"  Found {len(templates)} templates")
    
    created = []
    for i, tmpl in enumerate(templates[:4]):  # Max 4 templates per product
        tmpl_id = tmpl.get("template_id", f"unknown_{i}")
        image_url = tmpl.get("image_url", "")
        width = int(tmpl.get("template_width", 0))
        height = int(tmpl.get("template_height", 0))
        pa_x = int(float(tmpl.get("print_area_left", 0)))
        pa_y = int(float(tmpl.get("print_area_top", 0)))
        pa_w = int(float(tmpl.get("print_area_width", 0)))
        pa_h = int(float(tmpl.get("print_area_height", 0)))
        placement = tmpl.get("placement", "front") or "front"
        
        if not image_url:
            print(f"  Skipping template {tmpl_id}: no image URL")
            continue
        
        # Create folder
        folder_name = f"{product['type']}-{placement}-{tmpl_id}"
        folder = TEMPLATES_DIR / folder_name
        folder.mkdir(parents=True, exist_ok=True)
        
        # Download base image
        base_path = folder / "base.png"
        if not base_path.exists():
            print(f"  Downloading base image for {folder_name}...")
            try:
                download_image(image_url, base_path)
                print(f"    Downloaded: {base_path.stat().st_size / 1024:.0f}KB")
            except Exception as e:
                print(f"    ERROR downloading: {e}")
                continue
        else:
            print(f"  Base image already exists: {folder_name}")
        
        # Generate displacement map
        disp_path = folder / "displacement.png"
        if not disp_path.exists():
            print(f"  Generating displacement map...")
            try:
                generate_displacement_map(base_path, disp_path)
                if disp_path.exists():
                    print(f"    Generated: {disp_path.stat().st_size / 1024:.0f}KB")
                else:
                    print(f"    WARNING: displacement map not created, using flat gray")
                    # Create flat 128-gray displacement as fallback
                    subprocess.run([
                        "convert", "-size", f"{width}x{height}", 
                        "xc:gray50", str(disp_path)
                    ], check=True)
            except Exception as e:
                print(f"    ERROR: {e}")
        
        # Create metadata.json
        metadata = {
            "id": folder_name,
            "name": f"{product['name']} ({placement.title()})",
            "version": 1,
            "category": product["category"],
            "color": "white",
            "color_hex": "#FFFFFF",
            "placement": placement,
            "product": product["product"],
            "product_type": product["name"],
            "printful_product_id": product["id"],
            "printful_template_id": tmpl_id,
            "dimensions": {"width": width, "height": height},
            "print_area": {"x": pa_x, "y": pa_y, "width": pa_w, "height": pa_h},
            "anchor_point": {"x": pa_x + pa_w // 2, "y": pa_y + pa_h // 2},
            "displacement": {
                "enabled": product["displacement_strength"] > 1.0,
                "strength_default": product["displacement_strength"],
                "strength_range": [product["displacement_strength"] * 0.5, product["displacement_strength"] * 2.0]
            },
            "blend_mode": "multiply",
            "default_opacity": 240
        }
        
        meta_path = folder / "metadata.json"
        with open(meta_path, "w") as f:
            json.dump(metadata, f, indent=2)
        
        created.append(folder_name)
        print(f"  ✅ Created: {folder_name}")
    
    return created

if __name__ == "__main__":
    if not PRINTFUL_API_KEY:
        print("ERROR: PRINTFUL_API_KEY not set")
        sys.exit(1)
    
    print("MeetMockup Template Builder")
    print(f"Output: {TEMPLATES_DIR}")
    print(f"Products: {len(PRODUCTS)}")
    
    all_created = []
    for product in PRODUCTS:
        created = process_product(product)
        all_created.extend(created)
    
    print(f"\n{'='*60}")
    print(f"DONE! Created {len(all_created)} templates:")
    for name in all_created:
        print(f"  - {name}")
