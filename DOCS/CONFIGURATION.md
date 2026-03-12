# R-Image-Magic Configuration

R-Image-Magic supports highly flexible configuration using environment variables, TOML files, and local overrides.

## 1. Configuration Sources

Configuration is loaded in the following priority (highest to lowest):

1.  **Environment Variables**: Prefixed with `MOCKUP_`. Nested values use `__`, for example `MOCKUP_SERVER__PORT=9000`.
2.  **Local Configuration**: `config/local.toml` (gitignored, for development).
3.  **Default Configuration**: `config/default.toml` (bundled defaults).

The service also accepts compatibility aliases used by ECS and legacy deployments:

- `CONFIG_PATH` or `CONFIG_DIR` for the config directory.
- `DATABASE_URL`, `MOCKUP_DATABASE__URL`, or legacy `MOCKUP__DATABASE__URL` for the database connection string.
- `R2_*` or `MOCKUP_R2__*` for R2 credentials and bucket settings.
- `MOCKUP_SERVICE__NAME` and `MOCKUP_SERVICE__PRICING_URL` for tenant-specific service metadata.

## 2. Server Settings (`server`)

| Variable | TOML Key | Default | Description |
|----------|----------|---------|-------------|
| `MOCKUP_SERVER__HOST` | `server.host` | `0.0.0.0` | Host to bind the HTTP server to. |
| `MOCKUP_SERVER__PORT` | `server.port` | `8080` | Port to listen on. |
| `MOCKUP_SERVER__WORKERS` | `server.workers` | (CPU * 2) | Number of Actix-Web worker threads. |
| `MOCKUP_SERVICE__NAME` | n/a | `r-image-magic` | Service name exposed in headers and user agent strings. |
| `MOCKUP_SERVICE__PRICING_URL` | n/a | `https://r-image-magic.com/pricing` | Upgrade URL returned by quota responses. |

## 3. Template Settings (`templates`)

| Variable | TOML Key | Default | Description |
|----------|----------|---------|-------------|
| `MOCKUP_TEMPLATES__PATH` | `templates.path` | `assets/templates` | Path to the directory containing template folders. |

## 4. Database Settings (`database`)

| Variable | TOML Key | Default | Description |
|----------|----------|---------|-------------|
| `DATABASE_URL` | `database.url` | (empty) | PostgreSQL connection string. Preferred for ECS secrets injection. |
| `MOCKUP_DATABASE__URL` | `database.url` | (empty) | Equivalent nested config override for the database connection string. |
| `MOCKUP__DATABASE__URL` | `database.url` | (empty) | Legacy alias kept for backward compatibility. |
| `MOCKUP_DATABASE__MAX_CONNECTIONS` | `database.max_connections` | `10` | Maximum number of DB pool connections. |

## 5. Cloudinary Settings (`cloudinary`)

*Optional: Required only if you want to upload generated mockups to Cloudinary.*

| Variable | TOML Key | Description |
|----------|----------|-------------|
| `MOCKUP_CLOUDINARY__CLOUD_NAME` | `cloudinary.cloud_name` | Your Cloudinary cloud name. |
| `MOCKUP_CLOUDINARY__API_KEY` | `cloudinary.api_key` | Your Cloudinary API key. |
| `MOCKUP_CLOUDINARY__API_SECRET` | `cloudinary.api_secret` | Your Cloudinary API secret. |
| `MOCKUP_CLOUDINARY__UPLOAD_PRESET` | `cloudinary.upload_preset` | (Optional) Cloudinary upload preset. |

## 6. Cloudflare R2 Settings (`r2`)

*Optional: Used for POD asset storage and syncing.*

| Variable | TOML Key | Description |
|----------|----------|-------------|
| `MOCKUP_R2__ACCOUNT_ID` | `r2.account_id` | Your Cloudflare account ID. |
| `MOCKUP_R2__ACCESS_KEY_ID` | `r2.access_key_id` | R2 Access Key ID. |
| `MOCKUP_R2__SECRET_ACCESS_KEY` | `r2.secret_access_key` | R2 Secret Access Key. |
| `MOCKUP_R2__BUCKET_NAME` | `r2.bucket_name` | Name of the R2 bucket. |
| `MOCKUP_R2__PUBLIC_URL_PREFIX` | `r2.public_url_prefix` | (Optional) CDN URL prefix for R2 assets. |

## 7. Logging Configuration

Logging is configured via the `RUST_LOG` environment variable (using `tracing-subscriber`).

- **Default**: `info`
- **Debug core engine**: `RUST_LOG=r_image_magic=debug,actix_web=info`
- **Trace everything**: `RUST_LOG=trace`
