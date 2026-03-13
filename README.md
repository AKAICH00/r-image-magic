# r-image-magic

High-performance mockup generation platform — Rust API + Next.js frontend.

## Structure

```
apps/
  api/     Rust backend (Actix-Web) → AWS ECS Fargate
  web/     Next.js 15 frontend      → Vercel
docs/      Shared documentation
```

## Apps

### API (`apps/api/`)

Image compositing engine with displacement mapping, template management, and print-on-demand provider integrations. Deployed on ECS Fargate (ARM64).

```bash
cd apps/api
cargo run --release        # http://localhost:8080
cargo test                 # run all tests
```

See [apps/api/README.md](apps/api/README.md) for full setup, Docker, and ECS deployment details.

### Web (`apps/web/`)

MeetMockup marketing site and interactive demo. Deployed on Vercel.

```bash
cd apps/web
pnpm install
pnpm dev                   # http://localhost:3000
```

## Documentation

- [API Reference](docs/API.md)
- [Architecture](docs/ARCHITECTURE.md)
- [Engine](docs/ENGINE.md)
- [Configuration](docs/CONFIGURATION.md)
- [Templates](docs/TEMPLATES.md)

## CI/CD

- **`api-build.yml`** — Build + push Docker image on `apps/api/**` changes
- **`api-deploy.yml`** — Deploy to ECS on push to main (API changes only)
- **`web-check.yml`** — Lint + build check on PRs (web changes only)
- Vercel auto-deploys `apps/web/` via git integration
