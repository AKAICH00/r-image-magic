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

## ☁️ ECS

- The repo-owned ECS task definition template lives in [ecs/task-definition.json](ecs/task-definition.json).
- The deploy workflow renders that template in [.github/workflows/deploy.yml](.github/workflows/deploy.yml) and registers a fresh revision on every deploy.
- The workflow deploys to the GitHub Environment selected in `workflow_dispatch` (`staging` or `production`), and pushes from `main` default to `production`.
- Set environment-scoped GitHub variables for each target environment: `AWS_REGION`, `ECS_CLUSTER`, `ECS_SERVICE`, `ECS_TASK_FAMILY`, `ECS_EXECUTION_ROLE_ARN`, `ECS_TASK_ROLE_ARN`, `MOCKUP_DATABASE_URL_SECRET_ARN`, `R2_ACCOUNT_ID_SECRET_ARN`, `R2_ACCESS_KEY_ID_SECRET_ARN`, `R2_SECRET_ACCESS_KEY_SECRET_ARN`.
- Optional provider secret vars: `PRINTFUL_ACCESS_TOKEN_SECRET_ARN`, `PRINTIFY_API_KEY_SECRET_ARN`, `GELATO_API_KEY_SECRET_ARN`, `SPOD_ACCESS_TOKEN_SECRET_ARN`, `GOOTEN_RECIPE_ID_SECRET_ARN`.
- Optional environment-scoped tuning vars: `CONTAINER_PORT`, `TASK_CPU`, `TASK_MEMORY`, `AWS_LOG_GROUP`, `PRICING_URL`, `DEFAULT_R2_BUCKET`, `GHCR_SECRET_NAME`, `SERVICE_NAME`, `CONTAINER_NAME`.
- Render the final task definition locally before deploy with `./scripts/render-ecs-task-def.sh > task-def.rendered.json`.
- Example local dry-run: `IMAGE_TAG=$(git rev-parse --short HEAD) ECS_TASK_FAMILY=r-image-magic-staging ECS_EXECUTION_ROLE_ARN=arn:aws:iam::123:role/ecsExecution ECS_TASK_ROLE_ARN=arn:aws:iam::123:role/rImageMagicTask DATABASE_URL_SECRET_ARN=arn:aws:secretsmanager:us-east-1:123:secret:mockup-db ./scripts/render-ecs-task-def.sh task-def.rendered.json`.
- This keeps `r-image-magic` deployable as a sibling ECS service beside `teeswim` while reusing the same cluster, IAM baseline, and CI/CD pattern.

### Suggested GitHub Environment Matrix

Create two GitHub Environments named `staging` and `production`, then set these environment-scoped variables.

| Variable | `staging` example | `production` example |
|----------|-------------------|----------------------|
| `AWS_REGION` | `us-east-1` | `us-east-1` |
| `ECS_CLUSTER` | `akaich00-cluster` | `akaich00-cluster` |
| `ECS_SERVICE` | `r-image-magic-staging` | `r-image-magic` |
| `ECS_TASK_FAMILY` | `r-image-magic-staging` | `r-image-magic` |
| `SERVICE_NAME` | `r-image-magic-staging` | `r-image-magic` |
| `CONTAINER_NAME` | `r-image-magic` | `r-image-magic` |
| `CONTAINER_PORT` | `8080` | `8080` |
| `TASK_CPU` | `1024` | `2048` |
| `TASK_MEMORY` | `2048` | `4096` |
| `AWS_LOG_GROUP` | `/ecs/r-image-magic-staging` | `/ecs/r-image-magic` |
| `PRICING_URL` | `https://staging.rimagemagic.com/pricing` | `https://rimagemagic.com/pricing` |
| `DEFAULT_R2_BUCKET` | `r-image-magic-staging-assets` | `r-image-magic-pod-assets` |
| `GHCR_SECRET_NAME` | `r-image-magic/staging/ghcr-credentials` | `r-image-magic/production/ghcr-credentials` |
| `ECS_EXECUTION_ROLE_ARN` | `arn:aws:iam::123456789012:role/ecsTaskExecutionRole` | `arn:aws:iam::123456789012:role/ecsTaskExecutionRole` |
| `ECS_TASK_ROLE_ARN` | `arn:aws:iam::123456789012:role/rImageMagicStagingTaskRole` | `arn:aws:iam::123456789012:role/rImageMagicTaskRole` |
| `MOCKUP_DATABASE_URL_SECRET_ARN` | `arn:aws:secretsmanager:us-east-1:123456789012:secret:r-image-magic/staging/database-url` | `arn:aws:secretsmanager:us-east-1:123456789012:secret:r-image-magic/production/database-url` |
| `R2_ACCOUNT_ID_SECRET_ARN` | `arn:aws:secretsmanager:us-east-1:123456789012:secret:r-image-magic/staging/r2-account-id` | `arn:aws:secretsmanager:us-east-1:123456789012:secret:r-image-magic/production/r2-account-id` |
| `R2_ACCESS_KEY_ID_SECRET_ARN` | `arn:aws:secretsmanager:us-east-1:123456789012:secret:r-image-magic/staging/r2-access-key-id` | `arn:aws:secretsmanager:us-east-1:123456789012:secret:r-image-magic/production/r2-access-key-id` |
| `R2_SECRET_ACCESS_KEY_SECRET_ARN` | `arn:aws:secretsmanager:us-east-1:123456789012:secret:r-image-magic/staging/r2-secret-access-key` | `arn:aws:secretsmanager:us-east-1:123456789012:secret:r-image-magic/production/r2-secret-access-key` |

Optional provider secret ARN variables can follow the same naming pattern:

- `PRINTFUL_ACCESS_TOKEN_SECRET_ARN`
- `PRINTIFY_API_KEY_SECRET_ARN`
- `GELATO_API_KEY_SECRET_ARN`
- `SPOD_ACCESS_TOKEN_SECRET_ARN`
- `GOOTEN_RECIPE_ID_SECRET_ARN`

Bootstrap the GitHub Environment variables from your terminal:

```bash
# Preview staging commands
./scripts/bootstrap-github-environment.sh staging

# Apply production variables to the current repo
./scripts/bootstrap-github-environment.sh production --apply

# Apply to a specific repo
./scripts/bootstrap-github-environment.sh staging --repo akaich00/r-image-magic --apply
```

The bootstrap script uses `gh variable set --env ...` and can be overridden with real account-specific values via environment variables before running.

Bootstrap the matching AWS Secrets Manager placeholders:

```bash
# Preview staging secret creation commands
./scripts/bootstrap-aws-secrets.sh staging

# Create missing production secrets
./scripts/bootstrap-aws-secrets.sh production --apply
```

Both bootstrap scripts support `SERVICE_SLUG`, so you can reuse the same pattern for future sibling services without copying infrastructure logic.

Run the full sibling-service workflow with one command:

```bash
# Dry-run the full staging workflow
./scripts/orchestrate-ecs-service.sh staging \
  --service-slug r-image-magic \
  --repo akaich00/r-image-magic \
  --render-output /tmp/r-image-magic-staging-task.json

# Apply production bootstrap and compare against teeswim
./scripts/orchestrate-ecs-service.sh production \
  --service-slug r-image-magic \
  --repo akaich00/r-image-magic \
  --compare-with teeswim \
  --apply
```

This wrapper runs the same safe sequence every time:

- bootstrap Secrets Manager placeholders
- bootstrap GitHub Environment variables
- render the ECS task definition
- optionally compare the service against an existing sibling app such as `teeswim`

Recommended ALB/DNS split:

- `staging` → `staging-api.rimagemagic.com` → ECS service `r-image-magic-staging`
- `production` → `api.rimagemagic.com` → ECS service `r-image-magic`

Recommended Secrets Manager layout:

- `r-image-magic/staging/database-url`
- `r-image-magic/production/database-url`
- `r-image-magic/staging/r2-access-key-id`
- `r-image-magic/production/r2-access-key-id`

### Multi-Service Guardrails

To avoid breaking `teeswim` while sharing ECS infrastructure, keep these boundaries per service:

- Separate `ECS_SERVICE` and `ECS_TASK_FAMILY` values for every app.
- Separate ALB host rules or path rules and separate target groups.
- Separate Secrets Manager namespaces such as `service-name/staging/...` and `service-name/production/...`.
- Separate CloudWatch log groups per service and environment.
- Separate task roles when app permissions differ; reuse only the execution role where appropriate.
- Separate database URLs or schemas unless you explicitly design a shared multi-tenant data model.
- Separate GHCR pull secret names per service/environment so workflow changes never mutate another app's secret path.

This is the production-safe economies-of-scale model:

- Share the ECS cluster, VPC, ALB, CI/CD approach, and baseline IAM execution role.
- Isolate each application at the service, task, target group, secret, log group, and data layer.
- Standardize naming (`service/env/resource`) so the same bootstrap and deploy pattern can be reused for future projects.

Compare `teeswim` and `r-image-magic` before the first live deploy:

```bash
./scripts/compare-ecs-services.sh akaich00-cluster teeswim r-image-magic --region us-east-1
```

What should be different:

- ECS service names
- task definition families
- target groups
- CloudWatch log groups
- service-specific secret namespaces
- task roles when application permissions differ

What can be shared safely:

- ECS cluster
- VPC and subnets
- ALB listener (with separate host/path rules)
- baseline ECS execution role

If the comparison script reports `FAIL`, fix those boundaries before deploying. If it reports only `WARN`/`INFO`, review whether that sharing is intentional.

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run with logging enabled
RUST_LOG=debug cargo test
```

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
