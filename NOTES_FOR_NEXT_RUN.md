# Next Run Notes

## What this project is
- API-first mockup generation platform: Rust/Actix image API in `apps/api`, Next.js marketing/demo/docs/dashboard in `apps/web`.
- Product identity is developer-first and POD-focused, not a generic design tool.

## What was done in this run
- Added public free-tier API key signup at `POST /api/v1/keys/signup` with request validation and Rust tests.
- Replaced the Clerk-disabled signup dead end with a working self-serve key flow in the web app.
- Replaced dashboard/billing/API-key stubs with API-key-backed views that read a saved browser key and call the live backend.
- Updated web/docs readmes so onboarding matches the real flow.

## What remains next
- Add backend tests that exercise the signup handler against a real or test database path, not just validation helpers.
- Decide whether self-serve should allow multiple active keys per owner email or a rotation path without manual revoke.
- Wire billing/payments and key issuance into a real account model once Clerk and checkout are ready.

## Risks or caveats
- Self-serve signup currently blocks a second active key for the same owner email.
- Dashboard state is browser-local only; there is no server-side user/account session unless Clerk is configured.
- `cargo test`, `pnpm lint`, and `pnpm build` passed; `cargo fmt --check` still fails on pre-existing formatting drift outside this run’s scope.
