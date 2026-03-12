# R-Image-Magic Templates Guide

Templates are the building blocks of R-Image-Magic mockups. Each template folder represents a specific product view (e.g., "white-tshirt-front").

## 1. Template Structure

Templates are stored in the directory specified by `TEMPLATES_PATH`. Each template is a subdirectory containing:

- `base.png`: The high-resolution product image (the "blank" shirt).
- `displacement.png`: (Optional) Grayscale displacement map for fabric distortion.
- `metadata.json`: Configuration for print area, displacement, and blend modes.

### Example structure:
```
assets/templates/
└── white-tshirt-front/
    ├── base.png
    ├── displacement.png
    └── metadata.json
```

## 2. Metadata Specification (`metadata.json`)

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | String | Yes | Unique ID matching the folder name. |
| `name` | String | Yes | Human-readable name (e.g., "White T-Shirt Front"). |
| `product_type` | String | Yes | Product category (e.g., "T-Shirt", "Hoodie"). |
| `print_area` | Object | Yes | Location and size of the design placement area. |
| `displacement` | Object | No | Configuration for fabric distortion. |
| `blend_mode` | String | No | Default: `normal`. Supported: `normal`, `multiply`, `screen`, `overlay`. |
| `default_opacity` | Integer | No | Default: `255` (opaque). Range: 0-255. |

### Print Area Object:
| Field | Type | Description |
|-------|------|-------------|
| `x` | Integer | Horizontal offset from top-left of base image to print area center. |
| `y` | Integer | Vertical offset from top-left of base image to print area center. |
| `width` | Integer | Width of the print area in pixels (e.g., 1800). |
| `height` | Integer | Height of the print area in pixels (e.g., 2400). |

### Displacement Object:
| Field | Type | Description |
|-------|------|-------------|
| `enabled` | Boolean | Whether to apply displacement mapping (requires `displacement.png`). |
| `path` | String | (Optional) Custom path to displacement file. Default: `displacement.png`. |

### Example `metadata.json`:
```json
{
  "id": "white-tshirt-front",
  "name": "White T-Shirt Front",
  "product_type": "T-Shirt",
  "print_area": {
    "x": 1000,
    "y": 1200,
    "width": 1800,
    "height": 2400
  },
  "displacement": {
    "enabled": true
  },
  "blend_mode": "multiply",
  "default_opacity": 235
}
```

## 3. Creating Displacement Maps

Displacement maps are used to "wrap" the design around product geometry. To create a displacement map:

1.  **Start with the base image**: Open `base.png` in an image editor (e.g., Photoshop).
2.  **Convert to grayscale**: Remove all color.
3.  **Adjust contrast**: Fabric folds and wrinkles should be clearly visible. High-contrast areas lead to stronger displacement.
4.  **Normalize to 128 (Gray)**: Flat areas with no displacement should be around RGB(128, 128, 128).
5.  **Save as PNG**: Export as `displacement.png` in the template folder.

## 4. Best Practices

- **Base Images**: Use high-resolution (2000px+) images with a transparent background if possible. PNG is the preferred format.
- **Print Area Accuracy**: Match your print area dimensions to your actual POD provider's requirements (e.g., 1800x2400 for Printful) for zero-drift positioning.
- **Optimization**: For maximum performance, minimize the size of displacement maps. The engine will resize them at runtime, but starting with a smaller (but still detailed) map reduces memory overhead.
