# MeetMockup.com — Site Design & Product Entry Experience

**Date:** 2026-03-12
**Status:** Approved
**Author:** Aksel + Claude

---

## Strategic Summary

MeetMockup.com should feel like **Linear meets Replicate** — visually premium, technically credible, zero fluff. The homepage sells a result ("your mockups look real"), not a tool ("we have an API").

**Target audiences (in priority order):**
1. Technical POD sellers (Etsy, Shopify store owners)
2. Small POD-focused software builders / developers
3. Non-technical POD sellers (secondary, not v1 focus)

**Emotional arc of the page:**
1. "Wow, that looks real" (hero + comparison)
2. "Let me try it" (live demo)
3. "This is fast and easy" (demo result)
4. "I could automate this" (API section)
5. "The pricing is fair" (pricing teaser)
6. "I'm signing up" (CTA)

**Brand feel:** Clean, modern, premium, credible. More "high-performance SaaS + visual product" than "cheap template marketplace." Fast, polished, conversion-oriented. Minimal fluff.

---

## Sitemap (v1)

### Ship Now

| Route | Purpose |
|-------|---------|
| `/` | Homepage — sells the product |
| `/pricing` | Full pricing breakdown with tier comparison |
| `/demo` | Full-page interactive demo (also embedded on homepage) |
| `/templates` | Template gallery — browse by product type |
| `/docs` | Developer docs — getting started, API reference, code examples |
| `/login` | Auth entry |
| `/signup` | Registration + plan selection |
| `/dashboard` | Post-auth: API keys, usage stats, billing |

### Ship Later (not v1)

| Route | Why it can wait |
|-------|-----------------|
| `/blog` | No content yet, zero SEO value day one |
| `/changelog` | Premature — ship when you have 10+ updates |
| `/case-studies` | Need actual customers first |
| `/enterprise` | Contact form is enough on pricing page for now |
| `/sdks` | No SDKs built yet |

---

## Homepage — Section by Section

### Section 1: Hero
- **Purpose:** Instant visual proof that MeetMockup output is better
- **Audience:** Both (sellers see quality, developers see product-market fit)
- **Key message:** Your mockups should look real. Now they do.
- **UI:** Full-width dark background. Left: headline + subhead + 2 CTAs. Right: a single stunning mockup (white tee with a vibrant design, visible fabric displacement/wrinkles through the design). Subtle animation — the mockup fades in or rotates between 3 products.
- **CTA:** Primary: "Try it free" → `/demo`. Secondary: "View API docs" → `/docs`

### Section 2: Before/After Comparison
- **Purpose:** Make the quality gap undeniable
- **Audience:** Sellers primarily
- **Key message:** Flat overlays look fake. Displacement mapping looks real.
- **UI:** Side-by-side slider (drag to reveal). Left: "Flat overlay" — design pasted flat on tee. Right: "MeetMockup" — same design with displacement wrinkles. 2-3 product types shown (tee, hoodie, mug). Each pair is a clickable tab.
- **CTA:** None — this section builds conviction, not clicks.

### Section 3: Live Demo (Embedded)
- **Purpose:** Let them feel it. Trying > reading.
- **Audience:** Both
- **Key message:** See it work in 2 seconds.
- **UI:** Compact embedded version of `/demo`. Pre-loaded with a sample design. User can swap designs or upload their own. Shows template picker (6-8 popular templates). Generate button → result appears with generation time badge ("Generated in 1.4s"). No login required.
- **CTA:** "Sign up for 50 free mockups/month" below the result.

### Section 4: How It Works
- **Purpose:** Reduce complexity anxiety
- **Audience:** Both
- **Key message:** Three steps. That's it.
- **UI:** 3 horizontal cards:
  1. "Upload your design" — icon: upload arrow
  2. "Pick a template" — icon: grid of products
  3. "Get a realistic mockup" — icon: sparkle/download
- **CTA:** None

### Section 5: Template Gallery Preview
- **Purpose:** Show breadth — this isn't just tees
- **Audience:** Sellers primarily
- **Key message:** 44 templates across 9 product types
- **UI:** Horizontal scroll of template categories (T-shirts, Hoodies, Mugs, Phone Cases, Pillows, Posters, Totes, Wrapping Paper). Each card shows a sample mockup + count. Clicking goes to `/templates`.
- **CTA:** "Browse all templates →"

### Section 6: Developer/API Section
- **Purpose:** Establish technical credibility, open the developer door
- **Audience:** Developers
- **Key message:** One API call. Real mockups. Full automation.
- **UI:** Dark background code block showing a `curl` request + JSON response. Key stats: "~1.5s generation", "44 templates", "OpenAPI documented". Language tabs: curl, Python, Node.js.
- **CTA:** "Get your API key" → `/signup`

### Section 7: Pricing Teaser
- **Purpose:** Anchor price expectations, drive to pricing page
- **Audience:** Both
- **Key message:** Start free. Scale when you're ready.
- **UI:** 3 cards only (Free, Growth, Pro) — don't show all 6 here. Free emphasizes "50 mockups/mo, no credit card." Growth emphasizes "Most popular" badge. Pro emphasizes "For teams."
- **CTA:** "See all plans" → `/pricing`

### Section 8: Trust Strip
- **Purpose:** Reduce risk perception
- **Audience:** Both
- **Key message:** Fast, reliable, production-ready
- **UI:** Single row of trust signals: "Sub-2s generation" · "99.9% uptime" · "44 templates" · "9 product types"
- **CTA:** None

### Section 9: Final CTA
- **Purpose:** Catch everyone who scrolled the whole page
- **Audience:** Both
- **Key message:** Your mockups deserve better.
- **UI:** Full-width banner, dark. Headline + single CTA.
- **CTA:** "Start generating — it's free"

---

## Homepage Copy

**Hero headline:**
> Mockups that look real.

**Subheadline:**
> MeetMockup uses displacement mapping to wrap your designs onto products with real fabric texture — not flat overlays. Generate realistic mockups in under 2 seconds, one at a time or at scale through our API.

**Primary CTA:** `Try it free`
**Secondary CTA:** `View API docs →`

**3 Benefit cards (How It Works):**

| Card | Title | Description |
|------|-------|-------------|
| 1 | Upload your design | PNG or JPG. Transparent backgrounds work best. |
| 2 | Pick a product | T-shirts, hoodies, mugs, phone cases, and more. 44 templates. |
| 3 | Get a real mockup | Displacement-mapped onto the product. Download or automate via API. |

**Demo intro copy:**
> See the difference. Upload a design or use one of ours — your mockup generates in under 2 seconds.

**Pricing teaser copy:**
> Start with 50 free mockups per month. No credit card required. Upgrade when you're ready to scale.

**Developer/API teaser copy:**
> One API call. Realistic mockups.
> Automate your product listings with a single POST request. Full OpenAPI docs, sub-2s response times, and SDKs coming soon.

**Final CTA section:**
> Your listings deserve better mockups.
> `Start generating — it's free`

---

## Navigation & Footer

**Navbar:**
```
[MeetMockup logo]   Demo   Templates   Pricing   Docs     [Log in]  [Sign up free]
```

- Logo links to `/`
- 4 nav links, no dropdowns
- Auth buttons right-aligned
- Mobile: hamburger with same items, "Sign up free" stays visible

**Footer:**
```
Product          Developers       Company          Legal
─────────        ─────────        ─────────        ─────────
Demo             API Docs         About            Terms
Templates        API Reference    Contact          Privacy
Pricing          Status           Twitter/X
                 Changelog

© 2026 MeetMockup. All rights reserved.
```

---

## Live Demo Experience

**What the user sees:**
1. A pre-loaded sample design already placed on a white tee mockup
2. A template strip below showing 6-8 popular templates (different products) they can click to swap
3. An "Upload your design" button to replace the sample
4. Auto-generates on template/design change
5. Result appears with a subtle animation + "Generated in 1.4s" badge
6. "Download" button (watermarked on free/anon) + "Sign up for full quality" CTA

**Key decisions:**
- No login required — friction kills demos
- Sample design pre-loaded — show the result immediately
- Auto-generate on template click — remove extra steps
- Watermark anonymous results — small "MeetMockup.com" in corner, removed on signup
- Show generation time — 1.5s is a flex
- Don't build a placement editor in v1 — use sensible defaults (centered, 40% scale)
- Limit to 6-8 templates in demo — show variety, don't overwhelm
- Rate limit anonymous demo to 5 generations per session (cookie-based)

---

## Dual-Path Conversion Funnel

### Path A: Seller (visual-first)
1. **Lands on homepage** → sees hero mockup, scrolls to before/after → "wow, that looks better"
2. **Tries the demo** → uploads their own design, sees result in 1.5s → "I want this"
3. **Hits download** → sees watermark → signs up for free tier → gets 50 clean mockups/mo

### Path B: Developer (API-first)
1. **Lands on homepage** → scrolls past hero to API section → sees curl example + response time → "this is a real service"
2. **Clicks "View API docs"** → reads getting started guide, sees OpenAPI spec → "I can integrate this"
3. **Signs up** → gets API key → makes first API call → evaluates quality + speed → upgrades to Starter/Growth

---

## Pricing Page Structure

**Layout:** Horizontal tier cards, 3 visible by default on desktop (Starter, Growth, Pro as the focus row), with Free shown as a minimal line above and Platform/Enterprise as a row below.

Growth gets the "Most Popular" badge.

| | Free | Starter | Growth | Pro | Platform |
|---|---|---|---|---|---|
| Price | $0 | $19/mo | $49/mo | $99/mo | $299/mo |
| Mockups | 50/mo | 500/mo | 2,000/mo | 5,000/mo | 20,000/mo |
| Overage | — | $0.05/ea | $0.035/ea | $0.025/ea | Custom |
| Templates | 5 | All | All | All | All |
| Quality | Watermarked | Full | Full | Full | Full |
| Queue | Standard | Standard | Priority | Priority | Dedicated |
| API access | Yes | Yes | Yes | Yes | Yes |
| Batch generation | — | — | Yes | Yes | Yes |
| Webhooks | — | — | Yes | Yes | Yes |
| Team members | 1 | 1 | 1 | 5 | 10 |
| Support | Community | Email | Email | Priority | Dedicated |

**Enterprise:** Single line below: "Need custom volume, SLAs, or dedicated infrastructure? Contact us."

**CTA per tier:** Free = "Start free", Starter/Growth/Pro = "Start free trial" (14-day, no CC), Platform = "Contact sales"

**Annual billing toggle:** v2, not v1.

---

## Payments: Paddle

- **Paddle Billing** as merchant of record — handles VAT/sales tax, invoicing, dunning
- **Paddle.js** checkout overlay (no redirect to external page)
- **Dashboard billing:** Link to Paddle Customer Portal for plan changes/cancellations
- **Webhook:** `POST /api/webhooks/paddle` to sync subscription status → update API key tier on the backend
- Paddle handles global tax compliance — no need for Stripe Tax or manual VAT registration

---

## Next.js Implementation Plan

**Stack:** Next.js 15, React 19, App Router, Tailwind v4, shadcn/ui, Clerk auth, TypeScript strict mode, pnpm

### Route Structure

```
app/
├── (marketing)/
│   ├── layout.tsx          ← navbar + footer
│   ├── page.tsx            ← homepage
│   ├── pricing/page.tsx
│   ├── templates/page.tsx
│   └── demo/page.tsx
├── (docs)/
│   ├── layout.tsx          ← docs sidebar layout
│   └── docs/
│       ├── page.tsx        ← getting started
│       ├── api-reference/page.tsx
│       └── examples/page.tsx
├── (auth)/
│   ├── login/page.tsx
│   └── signup/page.tsx
├── (dashboard)/
│   ├── layout.tsx          ← authed layout
│   └── dashboard/
│       ├── page.tsx        ← overview + usage
│       ├── keys/page.tsx   ← API key management
│       └── billing/page.tsx
├── api/
│   ├── demo/generate/route.ts  ← proxy to api.meetmockup.com
│   └── webhooks/paddle/route.ts ← Paddle subscription webhooks
└── layout.tsx              ← root layout, fonts, meta
```

### Component Architecture

```
components/
├── ui/                     ← shadcn primitives (button, card, badge, tabs, slider)
├── marketing/
│   ├── Hero.tsx
│   ├── BeforeAfter.tsx     ← image comparison slider
│   ├── DemoEmbed.tsx       ← interactive demo
│   ├── HowItWorks.tsx
│   ├── TemplateGallery.tsx
│   ├── ApiShowcase.tsx     ← code block + stats
│   ├── PricingTeaser.tsx
│   ├── PricingTable.tsx    ← full pricing page
│   ├── TrustStrip.tsx
│   └── FinalCta.tsx
├── layout/
│   ├── Navbar.tsx
│   └── Footer.tsx
├── demo/
│   ├── DesignUploader.tsx
│   ├── TemplateStrip.tsx
│   ├── MockupResult.tsx
│   └── GenerateButton.tsx
└── dashboard/
    ├── UsageChart.tsx
    ├── ApiKeyCard.tsx
    └── BillingOverview.tsx
```

### Static vs Dynamic

| Content | Type | Why |
|---------|------|-----|
| Homepage sections | Static (RSC) | No user-specific data |
| Pricing tiers | Static (RSC) | Changes rarely, hardcode |
| Template gallery | Static + ISR | Fetch from API at build, revalidate daily |
| Demo | Client component | Interactive, calls API |
| Dashboard | Client + server | Authed, real-time usage data |
| Docs | MDX or static | Content-driven |

### v1 Stubs

- Dashboard billing → show usage chart with mock data, wire to real Paddle in v2
- Docs → 3 pages (Getting Started, API Reference link to Swagger, Code Examples)
- Template gallery → static grid from a JSON file

### Auth

Clerk — already in the stack from pattern-market.

---

## Build Brief (for coding agent)

**Project:** MeetMockup.com marketing site + dashboard
**Stack:** Next.js 15, React 19, App Router, Tailwind v4, shadcn/ui, Clerk auth, TypeScript strict mode, pnpm
**Repo:** New repo or subdirectory TBD

**API backend:** Already live at `https://api.meetmockup.com`. Demo should proxy mockup generation requests through a Next.js API route to avoid CORS and to rate-limit anonymous users.

**Design system:**
- Dark hero sections, light content sections
- Font: Inter or Geist
- Accent color: vibrant blue or teal — premium, not playful
- shadcn/ui for all primitives
- Responsive: mobile-first, single-column on mobile, max-w-7xl container on desktop

**Pages to build (in order):**
1. Root layout with Navbar + Footer
2. Homepage with all 9 sections
3. `/pricing` — full tier comparison table
4. `/demo` — full-page interactive demo
5. `/templates` — grid of all templates grouped by product type
6. `/docs` — 3 static MDX pages
7. Auth pages (Clerk)
8. `/dashboard` — API key display, usage chart stub, plan info

**Demo implementation:**
- `POST /api/demo/generate` route proxies to `https://api.meetmockup.com/api/v1/mockups/generate`
- Pre-load 3 sample designs as static assets in `/public/samples/`
- Show 8 templates in the template strip (one per product type)
- Anonymous users get watermarked results
- Rate limit anonymous demo to 5 generations per session (cookie-based)

**What NOT to build in v1:**
- Blog, changelog, case studies
- Design placement editor (positioning, scaling UI)
- Full docs platform (Mintlify, Nextra, etc.)
- Paddle billing integration (stub the billing page)
- SDKs page
- Annual billing toggle
- Team management

**Before/After assets needed:**
- 3 pairs of mockup images: flat overlay vs displacement-mapped
- Product types: t-shirt, hoodie, mug
- Generate from real API + flat composite for comparison

**Template gallery data:**
- `data/templates.json` with 44 templates grouped by product type
- Each entry: id, name, product_type, thumbnail_url
