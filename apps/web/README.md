# MeetMockup Web

Next.js 15 frontend for the MeetMockup marketing site, docs, demo, and lightweight developer dashboard.

## What lives here

- Marketing pages for the API-first mockup product
- Interactive demo routes that proxy to the Rust API
- Docs pages for onboarding and API usage
- Browser-local dashboard pages driven by a saved API key
- Clerk auth screens when Clerk is configured

## Local development

```bash
pnpm install
pnpm dev
```

Open `http://localhost:3000`.

## Useful environment variables

```bash
NEXT_PUBLIC_MEETMOCKUP_API_URL=http://localhost:8080
MEETMOCKUP_API_KEY=...                 # enables the live marketing demo proxy
NEXT_PUBLIC_SITE_URL=https://...       # needed for public demo asset URLs
NEXT_PUBLIC_CLERK_PUBLISHABLE_KEY=...  # optional
CLERK_SECRET_KEY=...                   # optional
```

If Clerk is not configured, `/signup` falls back to the self-serve API key flow and the dashboard stores the returned key in browser local storage.

## Checks

```bash
pnpm lint
pnpm build
```
