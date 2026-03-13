from __future__ import annotations

import json
from pathlib import Path
from typing import Iterable

from PIL import Image, ImageChops, ImageDraw, ImageFont, ImageOps


PROJECT_ROOT = Path(__file__).resolve().parents[1]
SOURCE_ROOT = Path("/Volumes/T7 Storage/Projects/r-image-magic/assets/templates")
PUBLIC_ROOT = PROJECT_ROOT / "public"
SAMPLES_ROOT = PUBLIC_ROOT / "samples"
COMPARISONS_ROOT = PUBLIC_ROOT / "comparisons"
TEMPLATES_ROOT = PUBLIC_ROOT / "templates"

SCALE = 0.4
OFFSET_X = 0
OFFSET_Y = -50
DISPLACEMENT_STRENGTH = 8.0


def ensure_dirs(paths: Iterable[Path]) -> None:
    for path in paths:
        path.mkdir(parents=True, exist_ok=True)


def load_font(size: int) -> ImageFont.FreeTypeFont | ImageFont.ImageFont:
    candidates = [
        "/System/Library/Fonts/Supplemental/Arial Bold.ttf",
        "/System/Library/Fonts/Supplemental/Arial.ttf",
        "/System/Library/Fonts/Supplemental/Helvetica.ttc",
    ]

    for candidate in candidates:
        font_path = Path(candidate)
        if font_path.exists():
            try:
                return ImageFont.truetype(str(font_path), size=size)
            except OSError:
                continue

    return ImageFont.load_default()


def build_sample_designs() -> list[Path]:
    ensure_dirs([SAMPLES_ROOT])

    fonts = {
        "title": load_font(250),
        "small": load_font(90),
        "tag": load_font(72),
    }

    designs = []

    # Sample 1: stacked sunburst poster mark
    canvas = Image.new("RGBA", (1800, 1800), (0, 0, 0, 0))
    draw = ImageDraw.Draw(canvas)
    center = (900, 760)
    for radius, color in [
        (720, (0, 0, 0, 0)),
        (510, (247, 214, 92, 220)),
        (360, (255, 173, 79, 215)),
        (220, (35, 50, 89, 210)),
    ]:
        draw.ellipse(
            (
                center[0] - radius,
                center[1] - radius,
                center[0] + radius,
                center[1] + radius,
            ),
            fill=color,
        )
    for i in range(18):
        angle = i * 20
        draw.pieslice(
            (220, 80, 1580, 1440),
            start=angle,
            end=angle + 8,
            fill=(255, 248, 236, 150),
        )
    draw.rounded_rectangle((420, 1160, 1380, 1510), radius=90, fill=(15, 18, 24, 220))
    draw.text((900, 1225), "SUN WAVE", anchor="ma", fill=(255, 248, 236, 255), font=fonts["title"])
    draw.text((900, 1365), "MEETMOCKUP", anchor="ma", fill=(243, 214, 92, 255), font=fonts["small"])
    sample_path = SAMPLES_ROOT / "sample-design-1.png"
    canvas.save(sample_path)
    designs.append(sample_path)

    # Sample 2: geometric grid badge
    canvas = Image.new("RGBA", (1800, 1800), (0, 0, 0, 0))
    draw = ImageDraw.Draw(canvas)
    draw.rounded_rectangle((170, 170, 1630, 1630), radius=220, fill=(21, 27, 38, 235))
    for row in range(5):
        for column in range(5):
            x = 340 + column * 240
            y = 320 + row * 220
            fill = (243, 214, 92, 255) if (row + column) % 2 == 0 else (217, 235, 255, 215)
            draw.rounded_rectangle((x, y, x + 140, y + 140), radius=38, fill=fill)
    draw.text((900, 1210), "GRID", anchor="ma", fill=(255, 248, 236, 255), font=fonts["title"])
    draw.text((900, 1395), "WAVE", anchor="ma", fill=(255, 173, 79, 255), font=fonts["title"])
    sample_path = SAMPLES_ROOT / "sample-design-2.png"
    canvas.save(sample_path)
    designs.append(sample_path)

    # Sample 3: bold mono wordmark
    canvas = Image.new("RGBA", (1800, 1800), (0, 0, 0, 0))
    draw = ImageDraw.Draw(canvas)
    draw.rounded_rectangle((220, 260, 1580, 1540), radius=180, fill=(255, 248, 236, 238))
    draw.arc((320, 360, 1480, 1520), 210, 380, fill=(255, 173, 79, 255), width=80)
    draw.arc((480, 520, 1320, 1360), 20, 170, fill=(21, 27, 38, 220), width=70)
    draw.text((900, 770), "FLUX", anchor="ma", fill=(21, 27, 38, 255), font=fonts["title"])
    draw.text((900, 1050), "MONO", anchor="ma", fill=(21, 27, 38, 255), font=fonts["title"])
    draw.text((900, 1290), "PREMIUM POD", anchor="ma", fill=(255, 173, 79, 255), font=fonts["tag"])
    sample_path = SAMPLES_ROOT / "sample-design-3.png"
    canvas.save(sample_path)
    designs.append(sample_path)

    return designs


def template_paths(template_id: str) -> tuple[Path, Path, dict]:
    template_root = SOURCE_ROOT / template_id
    with open(template_root / "metadata.json", "r", encoding="utf-8") as handle:
        metadata = json.load(handle)
    return template_root / "base.png", template_root / "displacement.png", metadata


def compute_region(metadata: dict) -> tuple[int, int, int, int]:
    print_area = metadata["print_area"]
    design_width = int(print_area["width"] * SCALE)
    design_height = int(print_area["height"] * SCALE)
    center_x = print_area["width"] // 2
    center_y = print_area["height"] // 2
    rel_x = center_x + OFFSET_X - design_width // 2
    rel_y = center_y + OFFSET_Y - design_height // 2
    abs_x = print_area["x"] + rel_x
    abs_y = print_area["y"] + rel_y
    return abs_x, abs_y, design_width, design_height


def bilinear_sample(image: Image.Image, x: float, y: float) -> tuple[int, int, int, int]:
    width, height = image.size
    x0 = int(x)
    y0 = int(y)
    x1 = min(x0 + 1, width - 1)
    y1 = min(y0 + 1, height - 1)
    dx = x - x0
    dy = y - y0
    p00 = image.getpixel((x0, y0))
    p10 = image.getpixel((x1, y0))
    p01 = image.getpixel((x0, y1))
    p11 = image.getpixel((x1, y1))
    channels = []
    for idx in range(4):
        value = (
            p00[idx] * (1 - dx) * (1 - dy)
            + p10[idx] * dx * (1 - dy)
            + p01[idx] * (1 - dx) * dy
            + p11[idx] * dx * dy
        )
        channels.append(int(max(0, min(255, value))))
    return tuple(channels)  # type: ignore[return-value]


def apply_displacement(design: Image.Image, displacement: Image.Image, strength: float) -> Image.Image:
    design = design.convert("RGBA")
    displacement = displacement.convert("L").resize(design.size, Image.Resampling.LANCZOS)
    width, height = design.size
    output = Image.new("RGBA", design.size, (0, 0, 0, 0))
    output_pixels = output.load()
    displacement_pixels = displacement.load()

    for y in range(height):
        for x in range(width):
            disp_value = displacement_pixels[x, y] / 255.0 - 0.5
            src_x = min(max(x + disp_value * strength, 0), width - 1)
            src_y = min(max(y + disp_value * strength, 0), height - 1)
            output_pixels[x, y] = bilinear_sample(design, src_x, src_y)

    return output


def multiply_blend(base_region: Image.Image, design_region: Image.Image) -> Image.Image:
    multiplied = ImageChops.multiply(base_region.convert("RGBA"), design_region.convert("RGBA"))
    alpha = design_region.getchannel("A")
    return Image.composite(multiplied, base_region.convert("RGBA"), alpha)


def compose_mockup(template_id: str, design_path: Path, output_prefix: str) -> None:
    base_path, displacement_path, metadata = template_paths(template_id)
    base_image = Image.open(base_path).convert("RGBA")
    displacement_map = Image.open(displacement_path).convert("L")
    design = Image.open(design_path).convert("RGBA")

    abs_x, abs_y, design_width, design_height = compute_region(metadata)
    resized_design = design.resize((design_width, design_height), Image.Resampling.LANCZOS)

    flat_base = base_image.copy()
    flat_region = flat_base.crop((abs_x, abs_y, abs_x + design_width, abs_y + design_height))
    flat_composite = multiply_blend(flat_region, resized_design)
    flat_base.paste(flat_composite, (abs_x, abs_y))

    displaced_base = base_image.copy()
    displacement_crop = displacement_map.crop((abs_x, abs_y, abs_x + design_width, abs_y + design_height))
    displaced_design = apply_displacement(resized_design, displacement_crop, DISPLACEMENT_STRENGTH)
    displaced_region = displaced_base.crop((abs_x, abs_y, abs_x + design_width, abs_y + design_height))
    displaced_composite = multiply_blend(displaced_region, displaced_design)
    displaced_base.paste(displaced_composite, (abs_x, abs_y))

    flat_output = COMPARISONS_ROOT / f"{output_prefix}-flat.png"
    displaced_output = COMPARISONS_ROOT / f"{output_prefix}-displaced.png"
    flat_base.save(flat_output)
    displaced_base.save(displaced_output)


def build_template_thumbnail(template_id: str, output_name: str, centering: tuple[float, float] = (0.5, 0.36)) -> None:
    base_path, _, _ = template_paths(template_id)
    image = Image.open(base_path).convert("RGBA")
    thumb = ImageOps.fit(
        image,
        (900, 900),
        method=Image.Resampling.LANCZOS,
        centering=centering,
    )
    thumb.save(TEMPLATES_ROOT / output_name)


def main() -> None:
    ensure_dirs([COMPARISONS_ROOT, TEMPLATES_ROOT])
    sample_paths = build_sample_designs()

    compose_mockup("white_male_front", sample_paths[0], "tshirt")
    compose_mockup("hoodie-aop-front-132947", sample_paths[1], "hoodie")
    compose_mockup("mug-11oz-front-919", sample_paths[2], "mug")

    build_template_thumbnail("white_male_front", "template-tshirt.png", centering=(0.5, 0.24))
    build_template_thumbnail("hoodie-aop-front-132947", "template-hoodie.png", centering=(0.5, 0.24))
    build_template_thumbnail("mug-11oz-front-919", "template-mug.png", centering=(0.5, 0.52))
    build_template_thumbnail("phone-case-front-146439", "template-phone-case.png", centering=(0.5, 0.35))
    build_template_thumbnail("pillow-front-22665", "template-pillow.png", centering=(0.5, 0.45))
    build_template_thumbnail("poster-front-21372", "template-poster.png", centering=(0.5, 0.3))
    build_template_thumbnail("tote-front-1204", "template-tote.png", centering=(0.5, 0.32))
    build_template_thumbnail("wrapping-paper-front-196986", "template-wrapping-paper.png", centering=(0.5, 0.45))
    build_template_thumbnail("tank-aop-front-4245", "template-tank.png", centering=(0.5, 0.26))


if __name__ == "__main__":
    main()
