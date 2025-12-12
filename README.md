# R-Image-Magic

High-performance image compositing and mockup generation API built with Rust + Actix-Web.

## Features

- **True Displacement Mapping** - Realistic fabric distortion for product mockups
- **10K+ Concurrent Connections** - Built for scale with async I/O
- **Template-Based Generation** - Preloaded templates for fast mockup creation
- **Multi-format Output** - PNG, JPEG, WebP support
- **Docker Ready** - Optimized multi-stage Dockerfile included

## Quick Start

### Prerequisites

- Rust 1.75+
- Docker (optional)

### Local Development

```bash
# Clone the repository
git clone https://github.com/akaich00/r-image-magic.git
cd r-image-magic

# Create .env file
cp .env.example .env

# Run the server
cargo run --release

# Server starts at http://localhost:8080
```

### Docker

```bash
# Build the image
docker build -t r-image-magic .

# Run the container
docker run -p 8080:8080 -v ./assets:/app/assets -v ./config:/app/config r-image-magic
```

## API Endpoints

### Health Check
```
GET /health
```

### Generate Mockup
```
POST /api/v1/mockup
Content-Type: application/json

{
  "template_id": "white-tshirt-front",
  "design_url": "https://example.com/design.png",
  "position": {
    "x": 0.5,
    "y": 0.3
  },
  "scale": 0.4
}
```

### List Templates
```
GET /api/v1/templates
```

## Configuration

Environment variables:

| Variable | Default | Description |
|----------|---------|-------------|
| `HOST` | `0.0.0.0` | Server bind address |
| `PORT` | `8080` | Server port |
| `RUST_LOG` | `info` | Log level |
| `CONFIG_DIR` | `./config` | Configuration directory |
| `TEMPLATES_PATH` | `./assets/templates` | Templates directory |

## Project Structure

```
r-image-magic/
├── src/
│   ├── main.rs          # Application entry point
│   ├── api/             # HTTP handlers and routes
│   ├── config/          # Configuration loading
│   ├── domain/          # Business logic and types
│   └── engine/          # Image processing engine
├── assets/
│   └── templates/       # Mockup templates
├── config/              # Configuration files
├── Cargo.toml
├── Dockerfile
└── README.md
```

## License

MIT
