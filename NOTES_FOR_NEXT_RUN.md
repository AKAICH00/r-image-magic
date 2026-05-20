# Notes For Next Run

## What this project is
- `r-image-magic` is the MeetMockup monorepo: a Rust/Actix mockup-generation API plus a Next.js marketing, docs, and interactive demo app.
- The product direction is API-first for POD sellers and developers who need realistic displacement-based mockups.

## What was done in this run
- Added `POST /api/v1/keys/signup` so the product now has a real self-serve free-key path without waiting for Clerk or billing.
- Wired `/signup` and the dashboard pages to browser-local API-key sessions, live usage/billing/key endpoints, and a usable no-Clerk onboarding flow.
- Hardened catalog product filtering by switching the handler to parameterized SQL, escaped search terms, and clamped pagination with unit tests.

## What remains next
- Exercise signup, dashboard, and demo flows against a live API/database instead of only local build/test checks.
- Add rate-limiting or abuse controls around self-serve signup if the endpoint is exposed publicly.
- Reduce the existing Rust warning debt so stricter lint/static-analysis gates become practical.

## Risks or caveats
- The dashboard session is browser-local API key storage, not a durable account system.
- Demo uploads still depend on valid R2 configuration for the hosted-upload path.
- `cargo test` passes today with many warnings; `pnpm lint` and `pnpm build` pass after installing `apps/web` dependencies in the worktree.
