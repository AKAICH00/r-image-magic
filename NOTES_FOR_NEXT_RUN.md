# Notes For Next Run

## What This Project Is
- `r-image-magic` is the MeetMockup monorepo: a Rust/Actix mockup-generation API plus a Next.js marketing/demo app.
- The API focus is realistic product mockups with displacement mapping, AOP template support, POD provider catalog sync, and usage/key infrastructure.
- The web focus is a public MeetMockup funnel with an interactive demo backed by the API.

## Done In This Run
- Opened PR #2: generation request validation for URL shape, empty template IDs, tint color format, and template-specific displacement ranges.
- Opened PR #3: compositor fetch guards for malformed URLs, local/private IP literal targets, and oversized design images.
- Opened PR #4: demo API/upload route hardening for malformed request bodies and backend upload failures.

## Remaining Next
- Merge PRs #2, #3, and #4 after checks finish and branch protection allows it.
- Reduce API warning/lint debt so `cargo clippy --all-targets -- -D warnings` can become a useful gate.
- Add resolver-level SSRF protection if arbitrary public design hostnames remain supported.

## Risks Or Caveats
- Current SSRF guard blocks obvious local/private IP literals but does not prevent DNS rebinding.
- API tests pass with many existing warnings; strict clippy currently fails on baseline issues.
- Live deployed API behavior was not exercised because this run focused on local checks and PRs.
