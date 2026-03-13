# Monorepo Restructure Design

**Date:** 2026-03-13
**Status:** Approved

## Decision

Consolidate `r-image-magic` (Rust backend) and `meetmockup-web` (Next.js frontend) into a single monorepo under the existing `r-image-magic` GitHub repo.

## Structure

```
r-image-magic/
├── apps/
│   ├── api/          # Rust backend → ECS Fargate
│   └── web/          # Next.js frontend → Vercel
├── docs/             # Shared documentation
├── .github/workflows/
│   ├── api-build.yml
│   ├── api-deploy.yml
│   └── web-check.yml
├── .gitignore
├── CLAUDE.md
└── README.md
```

## Approach

- Subtree import for meetmockup-web (preserves git history)
- Single atomic restructure on feature branch, merged to main
- Path-filtered CI workflows (each app triggers only its own CI)
- Vercel root directory set to `apps/web`
- No monorepo orchestrator (Turbo/Nx) — apps share no code

## Migration Steps

1. `git mv` Rust files into `apps/api/`, move `DOCS/` to `docs/`
2. `git subtree add` meetmockup-web as `apps/web/`
3. Rename/update CI workflows with path filters
4. Create root `.gitignore`, `CLAUDE.md`, `README.md`
5. Verify builds for both apps
