# R-Image-Magic Architecture Overview

R-Image-Magic is built with Rust and Actix-Web, designed for high performance, low latency, and massive concurrency in image-heavy workloads.

## 1. System Components

### Core Engine (`src/engine/`)
- **Template Manager**: Loads and caches mockup templates (base images, displacement maps) in memory. Handles on-demand template loading if needed.
- **Compositor**: The main pipeline for mockup generation. Orchestrates design fetching, processing, and compositing.
- **Displacement**: Implements the fabric distortion algorithm using Rayon for parallel pixel processing.

### API Layer (`src/api/`)
- **Handlers**: Actix-Web request handlers for generation, template queries, and health checks.
- **OpenAPI (`utoipa`)**: Auto-generates the OpenAPI 3.0 specification from Rust types and doc-comments.
- **Middleware**: Handles API key authentication, rate limiting, and request logging.

### Domain Model (`src/domain/`)
- **PlacementSpec**: The core coordinate system shared between the engine and the API. Ensures zero-drift between preview mockups and final print files.
- **Catalog Types**: Unified models for product types, variants, and providers.

### Data & Storage
- **Database (`src/db/`)**: PostgreSQL (via `sqlx`) for persistent storage of templates, API keys, and usage statistics.
- **Cloudflare R2**: Used for storing large assets and syncing with external print providers.
- **Cloudinary**: Optional integration for hosting generated mockups and providing CDN URLs.

## 2. Request Lifecycle

1.  **Request Entry**: An HTTP POST request arrives at `/api/v1/mockups/generate`.
2.  **Authentication**: Middleware verifies the API key (if configured).
3.  **Validation**: The `PlacementSpec` is validated for bounds and scale.
4.  **Template Retrieval**: The `TemplateManager` provides an `Arc<Template>` from memory.
5.  **Compositing**: The `Compositor` runs the parallelized generation pipeline.
6.  **Response Delivery**: The generated image is returned as a base64 string or stored and returned as a URL.

## 3. Performance Design Goals

- **Concurrency**: Leveraging Actix-Web's actor model and async/await for 10K+ concurrent connections.
- **Parallelism**: Rayon for CPU-bound image processing task splitting across all cores.
- **Memory Efficiency**: Heavy assets are loaded once and shared via `Arc` (Atomic Reference Counting).
- **Predictable Latency**: Average generation time (including displacement mapping) is targeted at <200ms for 2000x2000 images.
