# R-Image-Magic

High-performance image compositing and mockup generation API built with Rust + Actix-Web.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-blue.svg)](https://www.rust-lang.org)

## 🚀 Features

- **True Displacement Mapping** - Realistic fabric distortion for photorealistic product mockups.
- **10K+ Concurrent Connections** - Built for extreme scale with async I/O and Actix-Web.
- **Parallel Processing** - Leverages Rayon for lightning-fast image processing across all CPU cores.
- **Template-Based Generation** - Preloaded templates with intelligent metadata for zero-drift positioning.
- **Cloud-Ready** - Optional integrations with Cloudflare R2 and Cloudinary.
- **Docker Optimized** - Multi-stage builds for minimal image size.

## 📚 Documentation

For detailed guides, please see the [DOCS/](./DOCS) directory:

- [**API Reference**](./DOCS/API.md) - Endpoints, authentication, and request/response schemas.
- [**Architecture Overview**](./DOCS/ARCHITECTURE.md) - System design and component breakdown.
- [**Generation Engine**](./DOCS/ENGINE.md) - Technical details on displacement mapping and the pipeline.
- [**Configuration Guide**](./DOCS/CONFIGURATION.md) - Environment variables and TOML settings.
- [**Templates Guide**](./DOCS/TEMPLATES.md) - How to create and manage mockup templates.

## 🛠️ Quick Start

### Prerequisites

- Rust 1.75+
- PostgreSQL (optional, for persistent storage)

### Local Development

```bash
# 1. Clone the repository
git clone https://github.com/akaich00/r-image-magic.git
cd r-image-magic

# 2. Setup configuration
cp config/default.toml config/local.toml

# 3. Run the server
cargo run --release
```

The server will start at `http://localhost:8080`. Check the health endpoint: `curl http://localhost:8080/health`.

## 🐳 Docker

```bash
# Build the image
docker build -t r-image-magic .

# Run the container
docker run -p 8080:8080 
  -v ./assets:/app/assets 
  -v ./config:/app/config 
  r-image-magic
```

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run with logging enabled
RUST_LOG=debug cargo test
```

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
