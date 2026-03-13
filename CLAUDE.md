# r-image-magic Monorepo

## Structure

- `apps/api/` — Rust backend (Actix-Web). Build with `cargo` from this directory.
- `apps/web/` — Next.js 15 frontend. Build with `pnpm` from this directory.
- `docs/` — Shared documentation (architecture, API reference, engine details).
- `.github/workflows/` — CI/CD with path-filtered workflows.

## Building

```bash
# API
cd apps/api && cargo build --release && cargo test

# Web
cd apps/web && pnpm install && pnpm build && pnpm lint
```

## Deployment

- **API** → AWS ECS Fargate (ARM64). Deployed via `api-deploy.yml` on push to main when `apps/api/**` changes.
- **Web** → Vercel. Auto-deploys from `apps/web/` via Vercel git integration. Root directory set to `apps/web` in Vercel project settings.

## Key URLs

- API production: `https://api.meetmockup.com`
- Web: deployed via Vercel (check Vercel dashboard for URL)

## Conventions

- Commits follow Conventional Commits (`feat`, `fix`, `refactor`, `ci`, `chore`, `docs`)
- Feature branches: `feat/<description>`, bug fixes: `fix/<description>`
- Never commit secrets or .env files
