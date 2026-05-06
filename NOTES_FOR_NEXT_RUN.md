# Notes For Next Run

## What this project is
- `r-image-magic` is the MeetMockup monorepo: a Rust/Actix mockup-generation API plus a Next.js marketing, docs, and interactive demo app.
- The product direction is API-first for POD sellers and developers who need realistic displacement-based mockups.

## What was done in this run
- Added backend guards around remote design fetches so malformed URLs, private/local IP literals, and oversized design downloads fail before decode.
- Hardened the public demo generation and upload routes to return controlled JSON errors for bad input and upstream/storage failures.
- Aligned the web/docs onboarding copy with the actual product path and refreshed this handoff note.

## What remains next
- Add resolver-level protections if arbitrary external design hosts remain supported; literal-IP blocking does not cover DNS rebinding.
- Reduce existing Rust warning debt so stricter static-analysis gates become practical.
- Exercise the demo and upload flows against live configured credentials, not just local build/test checks.

## Risks or caveats
- Demo uploads still depend on valid R2 configuration for the hosted-upload path.
- The compositor security guard is a meaningful improvement, not a complete SSRF solution.
- `cargo test` passes today with many warnings; web checks require installing `apps/web` dependencies first.
