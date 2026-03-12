# R-Image-Magic API Reference

The R-Image-Magic API is a high-performance RESTful service for mockup generation and template management.

## 1. Authentication

Authentication is handled via API Keys in the `X-API-Key` header.

```bash
curl -H "X-API-Key: your_api_key" http://localhost:8080/api/v1/health
```

*Note: If no `DATABASE_URL` is configured, the service runs in "local-only" mode and authentication is bypassed.*

## 2. Mockup Generation

### Generate Mockup
`POST /api/v1/mockups/generate`

Generates a photorealistic mockup by compositing a design onto a template with displacement mapping.

#### Request Body
| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `design_url` | String | Yes | Publicly accessible URL of the design image (PNG/JPG) |
| `template_id` | String | Yes | Unique ID of the template (e.g., `white_male_front`) |
| `placement` | Object | Yes | Positioning and scaling specification |
| `options` | Object | No | Additional generation parameters |

**Placement Object (`PlacementSpec`):**
| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `scale` | Float | `0.5` | Scale factor (0.1 to 1.0) relative to print area |
| `offset_x` | Integer | `0` | Horizontal offset from center in pixels |
| `offset_y` | Integer | `-50` | Vertical offset from center in pixels |
| `placement` | String | `front` | Target area: `front`, `back`, `sleeve_left`, `sleeve_right` |
| `coordinate_space` | String | `print` | `print` (1800x2400) or `display` (1000x1400) |

**Options Object (`GenerateOptions`):**
| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `displacement_strength` | Float | `10.0` | Strength of the fabric distortion effect (0-30) |

#### Example Request
```json
{
  "design_url": "https://example.com/designs/logo.png",
  "template_id": "black-tshirt-front",
  "placement": {
    "scale": 0.4,
    "offset_y": -100
  },
  "options": {
    "displacement_strength": 12.5
  }
}
```

#### Example Response
```json
{
  "success": true,
  "mockup_url": "data:image/png;base64,iVBORw0KGgoAAAANSUhEUg...",
  "metadata": {
    "generation_time_ms": 145,
    "template_used": "black-tshirt-front",
    "dimensions": {
      "width": 2000,
      "height": 2000
    }
  }
}
```

## 3. Template Management

### List Templates
`GET /api/v1/templates`

Returns a list of all available mockup templates.

### Get Template Details
`GET /api/v1/templates/{template_id}`

Returns metadata for a specific template.

### List Product Types
`GET /api/v1/templates/product-types`

Returns all unique product types (e.g., T-Shirt, Hoodie, Tank Top).

### List Templates by Product Type
`GET /api/v1/templates/by-type/{product_type}`

## 4. System Endpoints

### Health Check
`GET /health`

Returns service status, version, and loaded templates count.

#### Example Response
```json
{
  "status": "ok",
  "version": "1.0.0",
  "uptime_seconds": 3600,
  "templates_loaded": 42
}
```

## 5. Error Codes

| Code | Status | Description |
|------|--------|-------------|
| `TEMPLATE_NOT_FOUND` | 404 | The requested template ID does not exist |
| `INVALID_PLACEMENT` | 400 | Placement spec is out of bounds or has invalid scale |
| `FETCH_FAILED` | 502 | Could not download the design from the provided URL |
| `GENERATION_FAILED` | 500 | Internal engine error during image processing |
