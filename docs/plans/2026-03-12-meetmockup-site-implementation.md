# MeetMockup.com Site Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build the MeetMockup.com marketing site, interactive demo, pricing page, and stub dashboard as a Next.js 15 app.

**Architecture:** Standalone Next.js 15 App Router project at `/Volumes/T7 Storage/Projects/meetmockup-web`. Route groups for layout isolation: `(marketing)` with shared navbar/footer, `(docs)` with sidebar, `(auth)` with Clerk, `(dashboard)` with authed layout. Demo proxies API calls through a Next.js route handler to `https://api.meetmockup.com`. Static marketing pages, client-side interactive demo, server components wherever possible.

**Tech Stack:** Next.js 15, React 19, TypeScript (strict), Tailwind v4, shadcn/ui, Clerk auth, Paddle.js (stubbed), pnpm

**Design doc:** `/Volumes/T7 Storage/Projects/r-image-magic/DOCS/plans/2026-03-12-meetmockup-site-design.md`

**API backend (live):** `https://api.meetmockup.com`
- `GET /api/v1/templates` → `{ data: TemplateInfo[], count: number }`
- `GET /api/v1/templates/product-types` → product type list with counts
- `POST /api/v1/mockups/generate` → `{ success: bool, mockup_url: string (base64 data URI), metadata: { generation_time_ms, template_used, dimensions } }`
- `GET /health` → `{ status, version, uptime_seconds, templates_loaded }`

---

## Task 1: Project Scaffold

**Files:**
- Create: `meetmockup-web/package.json`
- Create: `meetmockup-web/tsconfig.json`
- Create: `meetmockup-web/next.config.ts`
- Create: `meetmockup-web/tailwind.config.ts`
- Create: `meetmockup-web/src/app/layout.tsx`
- Create: `meetmockup-web/src/app/globals.css`
- Create: `meetmockup-web/.env.local.example`

**Step 1: Initialize Next.js project**

Run from `/Volumes/T7 Storage/Projects/`:
```bash
pnpm create next-app@latest meetmockup-web \
  --typescript \
  --tailwind \
  --eslint \
  --app \
  --src-dir \
  --import-alias "@/*" \
  --use-pnpm
```

**Step 2: Install dependencies**

```bash
cd /Volumes/T7 Storage/Projects/meetmockup-web
pnpm add @clerk/nextjs
pnpm add -D @types/node
```

**Step 3: Initialize shadcn/ui**

```bash
pnpm dlx shadcn@latest init
```

Choose: New York style, Zinc base color, CSS variables: yes.

Then add the components we'll use:
```bash
pnpm dlx shadcn@latest add button card badge tabs dialog input separator sheet
```

**Step 4: Create `.env.local.example`**

```env
# Clerk Auth
NEXT_PUBLIC_CLERK_PUBLISHABLE_KEY=
CLERK_SECRET_KEY=
NEXT_PUBLIC_CLERK_SIGN_IN_URL=/login
NEXT_PUBLIC_CLERK_SIGN_UP_URL=/signup

# MeetMockup API
MEETMOCKUP_API_URL=https://api.meetmockup.com
MEETMOCKUP_API_KEY=

# Paddle (stubbed for v1)
NEXT_PUBLIC_PADDLE_CLIENT_TOKEN=
PADDLE_API_KEY=
```

**Step 5: Update root layout**

`src/app/layout.tsx`:
```tsx
import type { Metadata } from "next";
import { Inter } from "next/font/google";
import { ClerkProvider } from "@clerk/nextjs";
import "./globals.css";

const inter = Inter({ subsets: ["latin"] });

export const metadata: Metadata = {
  title: "MeetMockup — Realistic Product Mockups",
  description:
    "Generate photorealistic product mockups with displacement mapping. API-first, built for POD sellers and developers.",
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <ClerkProvider>
      <html lang="en">
        <body className={inter.className}>{children}</body>
      </html>
    </ClerkProvider>
  );
}
```

**Step 6: Verify it runs**

```bash
pnpm dev
```

Expected: Next.js dev server starts at http://localhost:3000 with default page.

**Step 7: Initialize git and commit**

```bash
git init
git add -A
git commit -m "chore: scaffold Next.js 15 project with Tailwind, shadcn/ui, Clerk"
```

---

## Task 2: Shared Layout — Navbar & Footer

**Files:**
- Create: `src/components/layout/navbar.tsx`
- Create: `src/components/layout/footer.tsx`
- Create: `src/components/layout/mobile-nav.tsx`
- Create: `src/app/(marketing)/layout.tsx`
- Create: `src/app/(marketing)/page.tsx` (placeholder)

**Step 1: Create Navbar**

`src/components/layout/navbar.tsx`:
```tsx
import Link from "next/link";
import { SignedIn, SignedOut, UserButton } from "@clerk/nextjs";
import { Button } from "@/components/ui/button";

const navLinks = [
  { href: "/demo", label: "Demo" },
  { href: "/templates", label: "Templates" },
  { href: "/pricing", label: "Pricing" },
  { href: "/docs", label: "Docs" },
];

export function Navbar() {
  return (
    <header className="sticky top-0 z-50 w-full border-b border-border/40 bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
      <div className="mx-auto flex h-16 max-w-7xl items-center justify-between px-4 sm:px-6 lg:px-8">
        <div className="flex items-center gap-8">
          <Link href="/" className="text-xl font-bold tracking-tight">
            MeetMockup
          </Link>
          <nav className="hidden items-center gap-6 md:flex">
            {navLinks.map((link) => (
              <Link
                key={link.href}
                href={link.href}
                className="text-sm font-medium text-muted-foreground transition-colors hover:text-foreground"
              >
                {link.label}
              </Link>
            ))}
          </nav>
        </div>
        <div className="flex items-center gap-3">
          <SignedOut>
            <Link href="/login">
              <Button variant="ghost" size="sm">
                Log in
              </Button>
            </Link>
            <Link href="/signup">
              <Button size="sm">Sign up free</Button>
            </Link>
          </SignedOut>
          <SignedIn>
            <Link href="/dashboard">
              <Button variant="ghost" size="sm">
                Dashboard
              </Button>
            </Link>
            <UserButton />
          </SignedIn>
        </div>
      </div>
    </header>
  );
}
```

**Step 2: Create Footer**

`src/components/layout/footer.tsx`:
```tsx
import Link from "next/link";

const footerSections = [
  {
    title: "Product",
    links: [
      { href: "/demo", label: "Demo" },
      { href: "/templates", label: "Templates" },
      { href: "/pricing", label: "Pricing" },
    ],
  },
  {
    title: "Developers",
    links: [
      { href: "/docs", label: "API Docs" },
      { href: "/docs/api-reference", label: "API Reference" },
    ],
  },
  {
    title: "Company",
    links: [
      { href: "mailto:hello@meetmockup.com", label: "Contact" },
    ],
  },
  {
    title: "Legal",
    links: [
      { href: "/terms", label: "Terms" },
      { href: "/privacy", label: "Privacy" },
    ],
  },
];

export function Footer() {
  return (
    <footer className="border-t border-border/40 bg-background">
      <div className="mx-auto max-w-7xl px-4 py-12 sm:px-6 lg:px-8">
        <div className="grid grid-cols-2 gap-8 md:grid-cols-4">
          {footerSections.map((section) => (
            <div key={section.title}>
              <h3 className="text-sm font-semibold text-foreground">
                {section.title}
              </h3>
              <ul className="mt-4 space-y-3">
                {section.links.map((link) => (
                  <li key={link.href}>
                    <Link
                      href={link.href}
                      className="text-sm text-muted-foreground transition-colors hover:text-foreground"
                    >
                      {link.label}
                    </Link>
                  </li>
                ))}
              </ul>
            </div>
          ))}
        </div>
        <div className="mt-12 border-t border-border/40 pt-8">
          <p className="text-sm text-muted-foreground">
            &copy; {new Date().getFullYear()} MeetMockup. All rights reserved.
          </p>
        </div>
      </div>
    </footer>
  );
}
```

**Step 3: Create marketing layout**

`src/app/(marketing)/layout.tsx`:
```tsx
import { Navbar } from "@/components/layout/navbar";
import { Footer } from "@/components/layout/footer";

export default function MarketingLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <div className="flex min-h-screen flex-col">
      <Navbar />
      <main className="flex-1">{children}</main>
      <Footer />
    </div>
  );
}
```

**Step 4: Create placeholder homepage**

`src/app/(marketing)/page.tsx`:
```tsx
export default function HomePage() {
  return (
    <div className="flex items-center justify-center py-40">
      <h1 className="text-4xl font-bold">MeetMockup</h1>
    </div>
  );
}
```

**Step 5: Verify layout renders**

```bash
pnpm dev
```

Open http://localhost:3000. Should see navbar with logo + links, centered "MeetMockup" heading, footer at bottom.

**Step 6: Commit**

```bash
git add src/components/layout/ src/app/\(marketing\)/
git commit -m "feat: add navbar, footer, and marketing layout"
```

---

## Task 3: Homepage — Hero Section

**Files:**
- Create: `src/components/marketing/hero.tsx`
- Modify: `src/app/(marketing)/page.tsx`

**Step 1: Create Hero component**

`src/components/marketing/hero.tsx`:
```tsx
import Link from "next/link";
import { Button } from "@/components/ui/button";

export function Hero() {
  return (
    <section className="relative overflow-hidden bg-zinc-950 text-white">
      <div className="mx-auto max-w-7xl px-4 py-24 sm:px-6 sm:py-32 lg:px-8">
        <div className="grid items-center gap-12 lg:grid-cols-2">
          <div>
            <h1 className="text-5xl font-bold tracking-tight sm:text-6xl lg:text-7xl">
              Mockups that look real.
            </h1>
            <p className="mt-6 max-w-lg text-lg text-zinc-400">
              MeetMockup uses displacement mapping to wrap your designs onto
              products with real fabric texture — not flat overlays. Generate
              realistic mockups in under 2 seconds, one at a time or at scale
              through our API.
            </p>
            <div className="mt-8 flex items-center gap-4">
              <Link href="/demo">
                <Button size="lg" className="bg-white text-zinc-950 hover:bg-zinc-200">
                  Try it free
                </Button>
              </Link>
              <Link href="/docs">
                <Button variant="link" size="lg" className="text-zinc-400 hover:text-white">
                  View API docs &rarr;
                </Button>
              </Link>
            </div>
          </div>
          <div className="flex items-center justify-center">
            {/* Placeholder for hero mockup image — replace with real generated mockup */}
            <div className="aspect-square w-full max-w-md rounded-2xl bg-zinc-800 flex items-center justify-center text-zinc-600 text-sm">
              Hero mockup image
            </div>
          </div>
        </div>
      </div>
    </section>
  );
}
```

**Step 2: Wire into homepage**

`src/app/(marketing)/page.tsx`:
```tsx
import { Hero } from "@/components/marketing/hero";

export default function HomePage() {
  return <Hero />;
}
```

**Step 3: Verify, commit**

```bash
pnpm dev
# Check http://localhost:3000 — dark hero with headline, subhead, 2 CTAs, placeholder image
git add src/components/marketing/hero.tsx src/app/\(marketing\)/page.tsx
git commit -m "feat: add hero section to homepage"
```

---

## Task 4: Homepage — Before/After Comparison

**Files:**
- Create: `src/components/marketing/before-after.tsx`
- Modify: `src/app/(marketing)/page.tsx`

**Step 1: Create BeforeAfter component**

This is a draggable image comparison slider. Uses a client component with mouse/touch events.

`src/components/marketing/before-after.tsx`:
```tsx
"use client";

import { useRef, useState, useCallback } from "react";

interface ComparisonPair {
  label: string;
  before: string;
  after: string;
}

const pairs: ComparisonPair[] = [
  { label: "T-Shirt", before: "/comparisons/tshirt-flat.png", after: "/comparisons/tshirt-displaced.png" },
  { label: "Hoodie", before: "/comparisons/hoodie-flat.png", after: "/comparisons/hoodie-displaced.png" },
  { label: "Mug", before: "/comparisons/mug-flat.png", after: "/comparisons/mug-displaced.png" },
];

export function BeforeAfter() {
  const [activeIndex, setActiveIndex] = useState(0);
  const [sliderPosition, setSliderPosition] = useState(50);
  const containerRef = useRef<HTMLDivElement>(null);
  const isDragging = useRef(false);

  const handleMove = useCallback((clientX: number) => {
    if (!isDragging.current || !containerRef.current) return;
    const rect = containerRef.current.getBoundingClientRect();
    const x = Math.max(0, Math.min(clientX - rect.left, rect.width));
    setSliderPosition((x / rect.width) * 100);
  }, []);

  const handleMouseDown = () => { isDragging.current = true; };
  const handleMouseUp = () => { isDragging.current = false; };
  const handleMouseMove = (e: React.MouseEvent) => handleMove(e.clientX);
  const handleTouchMove = (e: React.TouchEvent) => handleMove(e.touches[0].clientX);

  const pair = pairs[activeIndex];

  return (
    <section className="bg-white py-20">
      <div className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
        <div className="text-center">
          <h2 className="text-3xl font-bold tracking-tight text-zinc-950 sm:text-4xl">
            See the difference
          </h2>
          <p className="mt-4 text-lg text-zinc-600">
            Flat overlays look fake. Displacement mapping looks real.
          </p>
        </div>

        <div className="mt-8 flex justify-center gap-2">
          {pairs.map((p, i) => (
            <button
              key={p.label}
              onClick={() => setActiveIndex(i)}
              className={`rounded-full px-4 py-2 text-sm font-medium transition-colors ${
                i === activeIndex
                  ? "bg-zinc-950 text-white"
                  : "bg-zinc-100 text-zinc-600 hover:bg-zinc-200"
              }`}
            >
              {p.label}
            </button>
          ))}
        </div>

        <div
          ref={containerRef}
          className="relative mx-auto mt-10 aspect-square max-w-2xl cursor-col-resize select-none overflow-hidden rounded-2xl bg-zinc-100"
          onMouseDown={handleMouseDown}
          onMouseUp={handleMouseUp}
          onMouseLeave={handleMouseUp}
          onMouseMove={handleMouseMove}
          onTouchStart={handleMouseDown}
          onTouchEnd={handleMouseUp}
          onTouchMove={handleTouchMove}
        >
          {/* Placeholder images — replace with real comparison assets */}
          <div className="absolute inset-0 flex items-center justify-center bg-zinc-200 text-zinc-500">
            <span className="text-sm">After: {pair.label} (MeetMockup)</span>
          </div>
          <div
            className="absolute inset-0 flex items-center justify-center bg-zinc-300 text-zinc-500"
            style={{ clipPath: `inset(0 ${100 - sliderPosition}% 0 0)` }}
          >
            <span className="text-sm">Before: {pair.label} (Flat overlay)</span>
          </div>
          <div
            className="absolute top-0 bottom-0 w-1 bg-white shadow-lg"
            style={{ left: `${sliderPosition}%`, transform: "translateX(-50%)" }}
          >
            <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 rounded-full bg-white p-2 shadow-lg">
              <svg className="h-4 w-4 text-zinc-600" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                <path strokeLinecap="round" strokeLinejoin="round" d="M8 9l4-4 4 4m0 6l-4 4-4-4" />
              </svg>
            </div>
          </div>
        </div>

        <div className="mt-6 flex justify-center gap-8 text-sm text-zinc-500">
          <span>&#8592; Flat overlay</span>
          <span>MeetMockup &#8594;</span>
        </div>
      </div>
    </section>
  );
}
```

**Step 2: Add to homepage**

Modify `src/app/(marketing)/page.tsx`:
```tsx
import { Hero } from "@/components/marketing/hero";
import { BeforeAfter } from "@/components/marketing/before-after";

export default function HomePage() {
  return (
    <>
      <Hero />
      <BeforeAfter />
    </>
  );
}
```

**Step 3: Verify, commit**

```bash
pnpm dev
# Check slider interaction at http://localhost:3000
git add src/components/marketing/before-after.tsx src/app/\(marketing\)/page.tsx
git commit -m "feat: add before/after comparison slider"
```

---

## Task 5: Homepage — Remaining Sections

**Files:**
- Create: `src/components/marketing/how-it-works.tsx`
- Create: `src/components/marketing/template-gallery-preview.tsx`
- Create: `src/components/marketing/api-showcase.tsx`
- Create: `src/components/marketing/pricing-teaser.tsx`
- Create: `src/components/marketing/trust-strip.tsx`
- Create: `src/components/marketing/final-cta.tsx`
- Create: `src/data/templates.ts` (static template data)
- Modify: `src/app/(marketing)/page.tsx`

**Step 1: Create HowItWorks**

`src/components/marketing/how-it-works.tsx`:
```tsx
import { Card, CardContent } from "@/components/ui/card";

const steps = [
  {
    step: "1",
    title: "Upload your design",
    description: "PNG or JPG. Transparent backgrounds work best.",
  },
  {
    step: "2",
    title: "Pick a product",
    description: "T-shirts, hoodies, mugs, phone cases, and more. 44 templates.",
  },
  {
    step: "3",
    title: "Get a real mockup",
    description: "Displacement-mapped onto the product. Download or automate via API.",
  },
];

export function HowItWorks() {
  return (
    <section className="bg-zinc-50 py-20">
      <div className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
        <div className="text-center">
          <h2 className="text-3xl font-bold tracking-tight text-zinc-950 sm:text-4xl">
            How it works
          </h2>
          <p className="mt-4 text-lg text-zinc-600">Three steps. That&apos;s it.</p>
        </div>
        <div className="mt-12 grid gap-8 sm:grid-cols-3">
          {steps.map((s) => (
            <Card key={s.step} className="border-0 bg-white shadow-sm">
              <CardContent className="pt-6 text-center">
                <div className="mx-auto flex h-12 w-12 items-center justify-center rounded-full bg-zinc-950 text-lg font-bold text-white">
                  {s.step}
                </div>
                <h3 className="mt-4 text-lg font-semibold text-zinc-950">{s.title}</h3>
                <p className="mt-2 text-sm text-zinc-600">{s.description}</p>
              </CardContent>
            </Card>
          ))}
        </div>
      </div>
    </section>
  );
}
```

**Step 2: Create static template data**

`src/data/templates.ts`:
```typescript
export interface TemplateCategory {
  productType: string;
  label: string;
  count: number;
  thumbnailId: string;
}

export const templateCategories: TemplateCategory[] = [
  { productType: "tshirt", label: "T-Shirts", count: 7, thumbnailId: "white_male_front" },
  { productType: "hoodie", label: "Hoodies", count: 6, thumbnailId: "hoodie-aop-front-132947" },
  { productType: "mug", label: "Mugs", count: 9, thumbnailId: "mug-11oz-front-919" },
  { productType: "phone-case", label: "Phone Cases", count: 6, thumbnailId: "phone-case-front-146439" },
  { productType: "pillow", label: "Pillows", count: 6, thumbnailId: "pillow-front-22665" },
  { productType: "poster", label: "Posters", count: 6, thumbnailId: "poster-front-21372" },
  { productType: "tote", label: "Totes", count: 3, thumbnailId: "tote-front-1204" },
  { productType: "wrapping-paper", label: "Wrapping Paper", count: 3, thumbnailId: "wrapping-paper-front-196986" },
  { productType: "tank", label: "Tanks", count: 2, thumbnailId: "tank-aop-front-4245" },
];

export const totalTemplates = templateCategories.reduce((sum, c) => sum + c.count, 0);
```

**Step 3: Create TemplateGalleryPreview**

`src/components/marketing/template-gallery-preview.tsx`:
```tsx
import Link from "next/link";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { templateCategories, totalTemplates } from "@/data/templates";

export function TemplateGalleryPreview() {
  return (
    <section className="bg-white py-20">
      <div className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
        <div className="text-center">
          <h2 className="text-3xl font-bold tracking-tight text-zinc-950 sm:text-4xl">
            {totalTemplates} templates across {templateCategories.length} product types
          </h2>
        </div>
        <div className="mt-12 flex gap-4 overflow-x-auto pb-4">
          {templateCategories.map((cat) => (
            <Link key={cat.productType} href={`/templates?type=${cat.productType}`} className="shrink-0">
              <Card className="w-48 border-0 bg-zinc-50 shadow-sm transition-shadow hover:shadow-md">
                <CardContent className="pt-6">
                  <div className="aspect-square w-full rounded-lg bg-zinc-200" />
                  <h3 className="mt-3 text-sm font-semibold text-zinc-950">{cat.label}</h3>
                  <p className="text-xs text-zinc-500">{cat.count} templates</p>
                </CardContent>
              </Card>
            </Link>
          ))}
        </div>
        <div className="mt-8 text-center">
          <Link href="/templates">
            <Button variant="outline">Browse all templates &rarr;</Button>
          </Link>
        </div>
      </div>
    </section>
  );
}
```

**Step 4: Create ApiShowcase**

`src/components/marketing/api-showcase.tsx`:
```tsx
import Link from "next/link";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";

const curlExample = `curl -X POST https://api.meetmockup.com/api/v1/mockups/generate \\
  -H "X-API-Key: rim_your_key_here" \\
  -H "Content-Type: application/json" \\
  -d '{
    "design_url": "https://example.com/logo.png",
    "template_id": "white_male_front",
    "placement": {
      "scale": 0.4,
      "offset_x": 0,
      "offset_y": -50
    }
  }'`;

const stats = [
  { value: "~1.5s", label: "Generation time" },
  { value: "44", label: "Templates" },
  { value: "REST", label: "OpenAPI documented" },
];

export function ApiShowcase() {
  return (
    <section className="bg-zinc-950 py-20 text-white">
      <div className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
        <div className="grid items-center gap-12 lg:grid-cols-2">
          <div>
            <Badge variant="secondary" className="mb-4">
              For developers
            </Badge>
            <h2 className="text-3xl font-bold tracking-tight sm:text-4xl">
              One API call. Realistic mockups.
            </h2>
            <p className="mt-4 text-lg text-zinc-400">
              Automate your product listings with a single POST request. Full
              OpenAPI docs, sub-2s response times, and SDKs coming soon.
            </p>
            <div className="mt-8 flex gap-6">
              {stats.map((stat) => (
                <div key={stat.label}>
                  <div className="text-2xl font-bold">{stat.value}</div>
                  <div className="text-sm text-zinc-500">{stat.label}</div>
                </div>
              ))}
            </div>
            <div className="mt-8">
              <Link href="/signup">
                <Button size="lg" className="bg-white text-zinc-950 hover:bg-zinc-200">
                  Get your API key
                </Button>
              </Link>
            </div>
          </div>
          <div className="overflow-hidden rounded-xl bg-zinc-900 p-6">
            <div className="flex items-center gap-2 mb-4">
              <div className="h-3 w-3 rounded-full bg-red-500" />
              <div className="h-3 w-3 rounded-full bg-yellow-500" />
              <div className="h-3 w-3 rounded-full bg-green-500" />
            </div>
            <pre className="overflow-x-auto text-sm text-zinc-300">
              <code>{curlExample}</code>
            </pre>
          </div>
        </div>
      </div>
    </section>
  );
}
```

**Step 5: Create PricingTeaser**

`src/components/marketing/pricing-teaser.tsx`:
```tsx
import Link from "next/link";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";

const tiers = [
  {
    name: "Free",
    price: "$0",
    period: "/month",
    description: "50 mockups/mo, no credit card",
    popular: false,
  },
  {
    name: "Growth",
    price: "$49",
    period: "/month",
    description: "2,000 mockups, batch generation, webhooks",
    popular: true,
  },
  {
    name: "Pro",
    price: "$99",
    period: "/month",
    description: "5,000 mockups, team access, priority rendering",
    popular: false,
  },
];

export function PricingTeaser() {
  return (
    <section className="bg-zinc-50 py-20">
      <div className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
        <div className="text-center">
          <h2 className="text-3xl font-bold tracking-tight text-zinc-950 sm:text-4xl">
            Start free. Scale when you&apos;re ready.
          </h2>
          <p className="mt-4 text-lg text-zinc-600">
            No credit card required. Upgrade when you need more.
          </p>
        </div>
        <div className="mt-12 grid gap-8 sm:grid-cols-3">
          {tiers.map((tier) => (
            <Card
              key={tier.name}
              className={`relative border-0 shadow-sm ${
                tier.popular ? "ring-2 ring-zinc-950" : ""
              }`}
            >
              {tier.popular && (
                <Badge className="absolute -top-3 left-1/2 -translate-x-1/2">
                  Most popular
                </Badge>
              )}
              <CardHeader className="text-center">
                <CardTitle className="text-lg">{tier.name}</CardTitle>
                <div className="mt-2">
                  <span className="text-4xl font-bold text-zinc-950">{tier.price}</span>
                  <span className="text-zinc-500">{tier.period}</span>
                </div>
              </CardHeader>
              <CardContent className="text-center">
                <p className="text-sm text-zinc-600">{tier.description}</p>
              </CardContent>
            </Card>
          ))}
        </div>
        <div className="mt-10 text-center">
          <Link href="/pricing">
            <Button variant="outline" size="lg">
              See all plans &rarr;
            </Button>
          </Link>
        </div>
      </div>
    </section>
  );
}
```

**Step 6: Create TrustStrip**

`src/components/marketing/trust-strip.tsx`:
```tsx
const signals = [
  "Sub-2s generation",
  "99.9% uptime",
  "44 templates",
  "9 product types",
  "OpenAPI documented",
];

export function TrustStrip() {
  return (
    <section className="border-y border-zinc-200 bg-white py-8">
      <div className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
        <div className="flex flex-wrap items-center justify-center gap-x-8 gap-y-4">
          {signals.map((signal) => (
            <span key={signal} className="text-sm font-medium text-zinc-500">
              {signal}
            </span>
          ))}
        </div>
      </div>
    </section>
  );
}
```

**Step 7: Create FinalCta**

`src/components/marketing/final-cta.tsx`:
```tsx
import Link from "next/link";
import { Button } from "@/components/ui/button";

export function FinalCta() {
  return (
    <section className="bg-zinc-950 py-20 text-white">
      <div className="mx-auto max-w-3xl px-4 text-center sm:px-6 lg:px-8">
        <h2 className="text-3xl font-bold tracking-tight sm:text-4xl">
          Your listings deserve better mockups.
        </h2>
        <div className="mt-8">
          <Link href="/signup">
            <Button size="lg" className="bg-white text-zinc-950 hover:bg-zinc-200">
              Start generating — it&apos;s free
            </Button>
          </Link>
        </div>
      </div>
    </section>
  );
}
```

**Step 8: Assemble full homepage**

`src/app/(marketing)/page.tsx`:
```tsx
import { Hero } from "@/components/marketing/hero";
import { BeforeAfter } from "@/components/marketing/before-after";
import { HowItWorks } from "@/components/marketing/how-it-works";
import { TemplateGalleryPreview } from "@/components/marketing/template-gallery-preview";
import { ApiShowcase } from "@/components/marketing/api-showcase";
import { PricingTeaser } from "@/components/marketing/pricing-teaser";
import { TrustStrip } from "@/components/marketing/trust-strip";
import { FinalCta } from "@/components/marketing/final-cta";

export default function HomePage() {
  return (
    <>
      <Hero />
      <BeforeAfter />
      <HowItWorks />
      <TemplateGalleryPreview />
      <ApiShowcase />
      <PricingTeaser />
      <TrustStrip />
      <FinalCta />
    </>
  );
}
```

**Step 9: Verify, commit**

```bash
pnpm dev
# Scroll through all 8 sections at http://localhost:3000
git add src/components/marketing/ src/data/ src/app/\(marketing\)/page.tsx
git commit -m "feat: add all homepage sections (how-it-works, templates, API, pricing, trust, CTA)"
```

---

## Task 6: Demo Page + API Proxy

**Files:**
- Create: `src/app/api/demo/generate/route.ts`
- Create: `src/app/(marketing)/demo/page.tsx`
- Create: `src/components/demo/demo-page.tsx`
- Create: `src/components/demo/design-uploader.tsx`
- Create: `src/components/demo/template-strip.tsx`
- Create: `src/components/demo/mockup-result.tsx`
- Create: `public/samples/sample-design-1.png` (placeholder — replace with real design)

**Step 1: Create the API proxy route**

`src/app/api/demo/generate/route.ts`:
```typescript
import { NextRequest, NextResponse } from "next/server";
import { cookies } from "next/headers";

const API_URL = process.env.MEETMOCKUP_API_URL ?? "https://api.meetmockup.com";
const DEMO_LIMIT = 5;

export async function POST(request: NextRequest) {
  const cookieStore = await cookies();
  const countCookie = cookieStore.get("demo_count");
  const count = countCookie ? parseInt(countCookie.value, 10) : 0;

  if (count >= DEMO_LIMIT) {
    return NextResponse.json(
      { error: "Demo limit reached. Sign up for 50 free mockups/month." },
      { status: 429 },
    );
  }

  const body = await request.json();

  const apiResponse = await fetch(`${API_URL}/api/v1/mockups/generate`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });

  const data = await apiResponse.json();

  const response = NextResponse.json(data, { status: apiResponse.status });
  response.cookies.set("demo_count", String(count + 1), {
    maxAge: 60 * 60,
    path: "/",
    httpOnly: true,
    sameSite: "strict",
  });

  return response;
}
```

**Step 2: Create the demo client component**

`src/components/demo/demo-page.tsx`:
```tsx
"use client";

import { useState, useCallback } from "react";
import { DesignUploader } from "./design-uploader";
import { TemplateStrip } from "./template-strip";
import { MockupResult } from "./mockup-result";

const DEMO_TEMPLATES = [
  { id: "white_male_front", label: "T-Shirt", type: "tshirt" },
  { id: "hoodie-aop-front-132947", label: "Hoodie", type: "hoodie" },
  { id: "mug-11oz-front-919", label: "Mug", type: "mug" },
  { id: "phone-case-front-146439", label: "Phone Case", type: "phone-case" },
  { id: "pillow-front-22665", label: "Pillow", type: "pillow" },
  { id: "poster-front-21372", label: "Poster", type: "poster" },
  { id: "tote-front-1204", label: "Tote", type: "tote" },
  { id: "tank-aop-front-4245", label: "Tank", type: "tank" },
];

export function DemoPage() {
  const [designUrl, setDesignUrl] = useState<string>("/samples/sample-design-1.png");
  const [selectedTemplate, setSelectedTemplate] = useState(DEMO_TEMPLATES[0]);
  const [result, setResult] = useState<{
    mockupUrl: string;
    generationTimeMs: number;
  } | null>(null);
  const [isGenerating, setIsGenerating] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const generate = useCallback(async (templateId: string, design: string) => {
    setIsGenerating(true);
    setError(null);
    try {
      const res = await fetch("/api/demo/generate", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          design_url: design,
          template_id: templateId,
          placement: { scale: 0.4, offset_x: 0, offset_y: -50 },
        }),
      });
      const data = await res.json();
      if (!res.ok) {
        setError(data.error ?? "Generation failed");
        return;
      }
      setResult({
        mockupUrl: data.mockup_url,
        generationTimeMs: data.metadata?.generation_time_ms ?? 0,
      });
    } catch {
      setError("Something went wrong. Please try again.");
    } finally {
      setIsGenerating(false);
    }
  }, []);

  const handleTemplateSelect = (template: typeof DEMO_TEMPLATES[number]) => {
    setSelectedTemplate(template);
    generate(template.id, designUrl);
  };

  const handleDesignChange = (url: string) => {
    setDesignUrl(url);
    generate(selectedTemplate.id, url);
  };

  return (
    <div className="mx-auto max-w-5xl px-4 py-12 sm:px-6 lg:px-8">
      <div className="text-center">
        <h1 className="text-3xl font-bold tracking-tight text-zinc-950 sm:text-4xl">
          Try MeetMockup
        </h1>
        <p className="mt-4 text-lg text-zinc-600">
          See the difference. Upload a design or use one of ours — your mockup
          generates in under 2 seconds.
        </p>
      </div>

      <div className="mt-10">
        <DesignUploader currentUrl={designUrl} onDesignChange={handleDesignChange} />
      </div>

      <div className="mt-6">
        <TemplateStrip
          templates={DEMO_TEMPLATES}
          selected={selectedTemplate.id}
          onSelect={handleTemplateSelect}
        />
      </div>

      <div className="mt-10">
        <MockupResult
          result={result}
          isGenerating={isGenerating}
          error={error}
        />
      </div>
    </div>
  );
}
```

**Step 3: Create DesignUploader**

`src/components/demo/design-uploader.tsx`:
```tsx
"use client";

import { useRef } from "react";
import { Button } from "@/components/ui/button";

interface DesignUploaderProps {
  currentUrl: string;
  onDesignChange: (url: string) => void;
}

export function DesignUploader({ currentUrl, onDesignChange }: DesignUploaderProps) {
  const inputRef = useRef<HTMLInputElement>(null);

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;
    const objectUrl = URL.createObjectURL(file);
    onDesignChange(objectUrl);
  };

  return (
    <div className="flex items-center justify-center gap-4">
      <div className="h-20 w-20 overflow-hidden rounded-lg border border-zinc-200 bg-zinc-50">
        {/* eslint-disable-next-line @next/next/no-img-element */}
        <img
          src={currentUrl}
          alt="Current design"
          className="h-full w-full object-contain"
        />
      </div>
      <div>
        <Button
          variant="outline"
          size="sm"
          onClick={() => inputRef.current?.click()}
        >
          Upload your design
        </Button>
        <input
          ref={inputRef}
          type="file"
          accept="image/png,image/jpeg"
          className="hidden"
          onChange={handleFileChange}
        />
        <p className="mt-1 text-xs text-zinc-500">PNG or JPG, transparent backgrounds work best</p>
      </div>
    </div>
  );
}
```

**Step 4: Create TemplateStrip**

`src/components/demo/template-strip.tsx`:
```tsx
interface Template {
  id: string;
  label: string;
  type: string;
}

interface TemplateStripProps {
  templates: Template[];
  selected: string;
  onSelect: (template: Template) => void;
}

export function TemplateStrip({ templates, selected, onSelect }: TemplateStripProps) {
  return (
    <div className="flex gap-3 overflow-x-auto pb-2">
      {templates.map((t) => (
        <button
          key={t.id}
          onClick={() => onSelect(t)}
          className={`shrink-0 rounded-xl border-2 p-3 transition-colors ${
            selected === t.id
              ? "border-zinc-950 bg-zinc-950 text-white"
              : "border-zinc-200 bg-zinc-50 text-zinc-700 hover:border-zinc-400"
          }`}
        >
          <div className="h-16 w-16 rounded-lg bg-zinc-200" />
          <span className="mt-2 block text-xs font-medium">{t.label}</span>
        </button>
      ))}
    </div>
  );
}
```

**Step 5: Create MockupResult**

`src/components/demo/mockup-result.tsx`:
```tsx
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import Link from "next/link";

interface MockupResultProps {
  result: { mockupUrl: string; generationTimeMs: number } | null;
  isGenerating: boolean;
  error: string | null;
}

export function MockupResult({ result, isGenerating, error }: MockupResultProps) {
  if (error) {
    return (
      <div className="rounded-2xl border border-red-200 bg-red-50 p-8 text-center">
        <p className="text-red-600">{error}</p>
        {error.includes("limit") && (
          <Link href="/signup" className="mt-4 inline-block">
            <Button>Sign up for 50 free mockups/month</Button>
          </Link>
        )}
      </div>
    );
  }

  if (isGenerating) {
    return (
      <div className="flex aspect-square max-w-2xl mx-auto items-center justify-center rounded-2xl bg-zinc-100">
        <div className="text-center">
          <div className="mx-auto h-8 w-8 animate-spin rounded-full border-4 border-zinc-300 border-t-zinc-950" />
          <p className="mt-4 text-sm text-zinc-500">Generating mockup...</p>
        </div>
      </div>
    );
  }

  if (!result) {
    return (
      <div className="flex aspect-square max-w-2xl mx-auto items-center justify-center rounded-2xl bg-zinc-100">
        <p className="text-sm text-zinc-500">Select a template to generate a mockup</p>
      </div>
    );
  }

  return (
    <div className="mx-auto max-w-2xl">
      <div className="overflow-hidden rounded-2xl bg-zinc-100">
        {/* eslint-disable-next-line @next/next/no-img-element */}
        <img
          src={result.mockupUrl}
          alt="Generated mockup"
          className="h-full w-full object-contain"
        />
      </div>
      <div className="mt-4 flex items-center justify-between">
        <Badge variant="secondary">
          Generated in {(result.generationTimeMs / 1000).toFixed(1)}s
        </Badge>
        <div className="flex gap-3">
          <Button variant="outline" size="sm" asChild>
            <a href={result.mockupUrl} download="mockup.png">
              Download
            </a>
          </Button>
          <Link href="/signup">
            <Button size="sm">Sign up for full quality</Button>
          </Link>
        </div>
      </div>
    </div>
  );
}
```

**Step 6: Create the demo page route**

`src/app/(marketing)/demo/page.tsx`:
```tsx
import type { Metadata } from "next";
import { DemoPage } from "@/components/demo/demo-page";

export const metadata: Metadata = {
  title: "Try MeetMockup — Generate a Realistic Mockup",
  description: "Upload your design and generate a photorealistic product mockup in under 2 seconds. No signup required.",
};

export default function Demo() {
  return <DemoPage />;
}
```

**Step 7: Verify, commit**

```bash
pnpm dev
# Navigate to http://localhost:3000/demo — should see demo UI
# Note: actual generation will fail until sample design exists and API is reachable
git add src/app/api/demo/ src/app/\(marketing\)/demo/ src/components/demo/ public/samples/
git commit -m "feat: add interactive demo page with API proxy and rate limiting"
```

---

## Task 7: Pricing Page

**Files:**
- Create: `src/app/(marketing)/pricing/page.tsx`
- Create: `src/components/marketing/pricing-table.tsx`
- Create: `src/data/pricing.ts`

**Step 1: Create pricing data**

`src/data/pricing.ts`:
```typescript
export interface PricingTier {
  name: string;
  price: string;
  period: string;
  mockups: string;
  overage: string;
  description: string;
  features: string[];
  cta: string;
  ctaVariant: "default" | "outline";
  popular: boolean;
  href: string;
}

export const pricingTiers: PricingTier[] = [
  {
    name: "Free",
    price: "$0",
    period: "/month",
    mockups: "50/month",
    overage: "—",
    description: "Test quality and integration",
    features: [
      "50 mockups per month",
      "5 templates",
      "Watermarked output",
      "API access",
      "Standard queue",
    ],
    cta: "Start free",
    ctaVariant: "outline",
    popular: false,
    href: "/signup?plan=free",
  },
  {
    name: "Starter",
    price: "$19",
    period: "/month",
    mockups: "500/month",
    overage: "$0.05/mockup",
    description: "For solo sellers",
    features: [
      "500 mockups per month",
      "All templates",
      "Full quality, no watermark",
      "API access",
      "Commercial use",
      "Email support",
    ],
    cta: "Start free trial",
    ctaVariant: "outline",
    popular: false,
    href: "/signup?plan=starter",
  },
  {
    name: "Growth",
    price: "$49",
    period: "/month",
    mockups: "2,000/month",
    overage: "$0.035/mockup",
    description: "For growing stores",
    features: [
      "2,000 mockups per month",
      "All templates",
      "Batch generation",
      "Priority queue",
      "Webhook support",
      "Email support",
    ],
    cta: "Start free trial",
    ctaVariant: "default",
    popular: true,
    href: "/signup?plan=growth",
  },
  {
    name: "Pro",
    price: "$99",
    period: "/month",
    mockups: "5,000/month",
    overage: "$0.025/mockup",
    description: "For teams and agencies",
    features: [
      "5,000 mockups per month",
      "All templates",
      "Up to 5 team members",
      "Priority rendering",
      "Usage analytics",
      "Priority support",
    ],
    cta: "Start free trial",
    ctaVariant: "outline",
    popular: false,
    href: "/signup?plan=pro",
  },
  {
    name: "Platform",
    price: "$299",
    period: "/month",
    mockups: "20,000/month",
    overage: "Custom",
    description: "For apps and resellers",
    features: [
      "20,000 mockups per month",
      "All templates",
      "Up to 10 team members",
      "Multi-key support",
      "Reseller / embedded rights",
      "Higher rate limits",
      "SLA-lite support",
    ],
    cta: "Contact sales",
    ctaVariant: "outline",
    popular: false,
    href: "mailto:hello@meetmockup.com?subject=Platform%20plan",
  },
];
```

**Step 2: Create PricingTable**

`src/components/marketing/pricing-table.tsx`:
```tsx
import Link from "next/link";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { pricingTiers } from "@/data/pricing";

export function PricingTable() {
  return (
    <div className="grid gap-6 lg:grid-cols-5 md:grid-cols-3 sm:grid-cols-2">
      {pricingTiers.map((tier) => (
        <Card
          key={tier.name}
          className={`relative flex flex-col border-0 shadow-sm ${
            tier.popular ? "ring-2 ring-zinc-950" : ""
          }`}
        >
          {tier.popular && (
            <Badge className="absolute -top-3 left-1/2 -translate-x-1/2">
              Most popular
            </Badge>
          )}
          <CardHeader>
            <CardTitle className="text-lg">{tier.name}</CardTitle>
            <div className="mt-2">
              <span className="text-4xl font-bold text-zinc-950">{tier.price}</span>
              <span className="text-zinc-500">{tier.period}</span>
            </div>
            <p className="mt-1 text-sm text-zinc-500">{tier.description}</p>
          </CardHeader>
          <CardContent className="flex flex-1 flex-col">
            <div className="mb-4 text-sm">
              <span className="font-medium text-zinc-950">{tier.mockups}</span>
              {tier.overage !== "—" && (
                <span className="text-zinc-500"> · then {tier.overage}</span>
              )}
            </div>
            <ul className="flex-1 space-y-2">
              {tier.features.map((f) => (
                <li key={f} className="flex items-start gap-2 text-sm text-zinc-600">
                  <svg className="mt-0.5 h-4 w-4 shrink-0 text-zinc-400" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                    <path strokeLinecap="round" strokeLinejoin="round" d="M5 13l4 4L19 7" />
                  </svg>
                  {f}
                </li>
              ))}
            </ul>
            <div className="mt-6">
              <Link href={tier.href} className="block">
                <Button
                  variant={tier.ctaVariant}
                  className="w-full"
                >
                  {tier.cta}
                </Button>
              </Link>
            </div>
          </CardContent>
        </Card>
      ))}
    </div>
  );
}
```

**Step 3: Create pricing page**

`src/app/(marketing)/pricing/page.tsx`:
```tsx
import type { Metadata } from "next";
import { PricingTable } from "@/components/marketing/pricing-table";

export const metadata: Metadata = {
  title: "Pricing — MeetMockup",
  description: "Start free with 50 mockups/month. Scale to thousands with simple, transparent pricing.",
};

export default function PricingPage() {
  return (
    <div className="py-20">
      <div className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
        <div className="text-center">
          <h1 className="text-3xl font-bold tracking-tight text-zinc-950 sm:text-4xl">
            Simple, transparent pricing
          </h1>
          <p className="mt-4 text-lg text-zinc-600">
            Start free. Pay only when you scale.
          </p>
        </div>
        <div className="mt-16">
          <PricingTable />
        </div>
        <div className="mt-16 text-center">
          <p className="text-sm text-zinc-500">
            Need custom volume, SLAs, or dedicated infrastructure?{" "}
            <a
              href="mailto:hello@meetmockup.com?subject=Enterprise%20inquiry"
              className="font-medium text-zinc-950 underline hover:no-underline"
            >
              Contact us
            </a>
          </p>
        </div>
      </div>
    </div>
  );
}
```

**Step 4: Verify, commit**

```bash
pnpm dev
# Check http://localhost:3000/pricing
git add src/app/\(marketing\)/pricing/ src/components/marketing/pricing-table.tsx src/data/pricing.ts
git commit -m "feat: add pricing page with tier comparison table"
```

---

## Task 8: Templates Gallery Page

**Files:**
- Create: `src/app/(marketing)/templates/page.tsx`
- Create: `src/components/marketing/template-grid.tsx`

**Step 1: Create TemplateGrid**

`src/components/marketing/template-grid.tsx`:
```tsx
"use client";

import { useState } from "react";
import { templateCategories } from "@/data/templates";

export function TemplateGrid() {
  const [activeType, setActiveType] = useState<string | null>(null);

  const filtered = activeType
    ? templateCategories.filter((c) => c.productType === activeType)
    : templateCategories;

  return (
    <div>
      <div className="flex flex-wrap gap-2">
        <button
          onClick={() => setActiveType(null)}
          className={`rounded-full px-4 py-2 text-sm font-medium transition-colors ${
            activeType === null
              ? "bg-zinc-950 text-white"
              : "bg-zinc-100 text-zinc-600 hover:bg-zinc-200"
          }`}
        >
          All
        </button>
        {templateCategories.map((cat) => (
          <button
            key={cat.productType}
            onClick={() => setActiveType(cat.productType)}
            className={`rounded-full px-4 py-2 text-sm font-medium transition-colors ${
              activeType === cat.productType
                ? "bg-zinc-950 text-white"
                : "bg-zinc-100 text-zinc-600 hover:bg-zinc-200"
            }`}
          >
            {cat.label} ({cat.count})
          </button>
        ))}
      </div>
      <div className="mt-8 grid gap-6 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4">
        {filtered.map((cat) => (
          <div
            key={cat.thumbnailId}
            className="overflow-hidden rounded-xl border border-zinc-200 bg-white"
          >
            <div className="aspect-square bg-zinc-100" />
            <div className="p-4">
              <h3 className="text-sm font-semibold text-zinc-950">{cat.label}</h3>
              <p className="text-xs text-zinc-500">{cat.count} templates</p>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
```

**Step 2: Create templates page**

`src/app/(marketing)/templates/page.tsx`:
```tsx
import type { Metadata } from "next";
import { TemplateGrid } from "@/components/marketing/template-grid";
import { totalTemplates, templateCategories } from "@/data/templates";

export const metadata: Metadata = {
  title: "Templates — MeetMockup",
  description: `Browse ${totalTemplates} product mockup templates across ${templateCategories.length} categories.`,
};

export default function TemplatesPage() {
  return (
    <div className="py-20">
      <div className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
        <div className="text-center">
          <h1 className="text-3xl font-bold tracking-tight text-zinc-950 sm:text-4xl">
            Template library
          </h1>
          <p className="mt-4 text-lg text-zinc-600">
            {totalTemplates} templates across {templateCategories.length} product types.
          </p>
        </div>
        <div className="mt-12">
          <TemplateGrid />
        </div>
      </div>
    </div>
  );
}
```

**Step 3: Verify, commit**

```bash
pnpm dev
# Check http://localhost:3000/templates — filter tabs + grid
git add src/app/\(marketing\)/templates/ src/components/marketing/template-grid.tsx
git commit -m "feat: add template gallery page with category filtering"
```

---

## Task 9: Docs Pages (Static MDX)

**Files:**
- Create: `src/app/(docs)/layout.tsx`
- Create: `src/app/(docs)/docs/page.tsx`
- Create: `src/app/(docs)/docs/api-reference/page.tsx`
- Create: `src/app/(docs)/docs/examples/page.tsx`
- Create: `src/components/docs/sidebar.tsx`

**Step 1: Create docs sidebar**

`src/components/docs/sidebar.tsx`:
```tsx
"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";

const docsNav = [
  { href: "/docs", label: "Getting Started" },
  { href: "/docs/api-reference", label: "API Reference" },
  { href: "/docs/examples", label: "Code Examples" },
];

export function DocsSidebar() {
  const pathname = usePathname();

  return (
    <nav className="w-56 shrink-0">
      <div className="sticky top-20">
        <h3 className="text-sm font-semibold text-zinc-950">Documentation</h3>
        <ul className="mt-4 space-y-1">
          {docsNav.map((item) => (
            <li key={item.href}>
              <Link
                href={item.href}
                className={`block rounded-md px-3 py-2 text-sm transition-colors ${
                  pathname === item.href
                    ? "bg-zinc-100 font-medium text-zinc-950"
                    : "text-zinc-600 hover:bg-zinc-50 hover:text-zinc-950"
                }`}
              >
                {item.label}
              </Link>
            </li>
          ))}
        </ul>
      </div>
    </nav>
  );
}
```

**Step 2: Create docs layout**

`src/app/(docs)/layout.tsx`:
```tsx
import { Navbar } from "@/components/layout/navbar";
import { Footer } from "@/components/layout/footer";
import { DocsSidebar } from "@/components/docs/sidebar";

export default function DocsLayout({ children }: { children: React.ReactNode }) {
  return (
    <div className="flex min-h-screen flex-col">
      <Navbar />
      <div className="mx-auto flex w-full max-w-7xl flex-1 gap-10 px-4 py-10 sm:px-6 lg:px-8">
        <DocsSidebar />
        <main className="min-w-0 flex-1">{children}</main>
      </div>
      <Footer />
    </div>
  );
}
```

**Step 3: Create Getting Started page**

`src/app/(docs)/docs/page.tsx`:
```tsx
import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "Getting Started — MeetMockup Docs",
};

export default function DocsGettingStarted() {
  return (
    <article className="prose prose-zinc max-w-none">
      <h1>Getting Started</h1>
      <p>
        MeetMockup generates photorealistic product mockups with displacement
        mapping. This guide walks you through your first API call.
      </p>

      <h2>1. Get your API key</h2>
      <p>
        Sign up at <a href="/signup">meetmockup.com/signup</a> to get a free API
        key. Free tier includes 50 mockups per month.
      </p>

      <h2>2. Make your first request</h2>
      <pre><code>{`curl -X POST https://api.meetmockup.com/api/v1/mockups/generate \\
  -H "X-API-Key: YOUR_API_KEY" \\
  -H "Content-Type: application/json" \\
  -d '{
    "design_url": "https://example.com/your-design.png",
    "template_id": "white_male_front",
    "placement": {
      "scale": 0.4,
      "offset_x": 0,
      "offset_y": -50
    }
  }'`}</code></pre>

      <h2>3. Get your mockup</h2>
      <p>
        The response includes a base64-encoded PNG in the <code>mockup_url</code>{" "}
        field, along with generation metadata.
      </p>
      <pre><code>{`{
  "success": true,
  "mockup_url": "data:image/png;base64,iVBORw0KGgo...",
  "metadata": {
    "generation_time_ms": 1450,
    "template_used": "white_male_front",
    "dimensions": { "width": 2000, "height": 2000 }
  }
}`}</code></pre>

      <h2>4. Browse templates</h2>
      <p>
        List available templates with <code>GET /api/v1/templates</code> or browse
        them visually at <a href="/templates">/templates</a>.
      </p>

      <h2>Next steps</h2>
      <ul>
        <li><a href="/docs/api-reference">Full API reference</a></li>
        <li><a href="/docs/examples">Code examples</a> in Python, Node.js, and curl</li>
      </ul>
    </article>
  );
}
```

**Step 4: Create API Reference page**

`src/app/(docs)/docs/api-reference/page.tsx`:
```tsx
import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "API Reference — MeetMockup Docs",
};

export default function ApiReferencePage() {
  return (
    <article className="prose prose-zinc max-w-none">
      <h1>API Reference</h1>
      <p>
        MeetMockup exposes a REST API documented with OpenAPI 3.0. The full
        interactive reference is available via Swagger UI.
      </p>

      <h2>Base URL</h2>
      <pre><code>https://api.meetmockup.com</code></pre>

      <h2>Authentication</h2>
      <p>
        Pass your API key in the <code>X-API-Key</code> header with every request.
      </p>

      <h2>Interactive docs</h2>
      <p>
        Browse and test all endpoints at{" "}
        <a href="https://api.meetmockup.com/swagger-ui/" target="_blank" rel="noopener noreferrer">
          api.meetmockup.com/swagger-ui/
        </a>
      </p>

      <h2>Key endpoints</h2>
      <table>
        <thead>
          <tr><th>Method</th><th>Path</th><th>Description</th></tr>
        </thead>
        <tbody>
          <tr><td>POST</td><td>/api/v1/mockups/generate</td><td>Generate a mockup</td></tr>
          <tr><td>GET</td><td>/api/v1/templates</td><td>List all templates</td></tr>
          <tr><td>GET</td><td>/api/v1/templates/&#123;id&#125;</td><td>Get template details</td></tr>
          <tr><td>GET</td><td>/api/v1/templates/product-types</td><td>List product types</td></tr>
          <tr><td>GET</td><td>/api/v1/usage</td><td>Current usage stats</td></tr>
          <tr><td>GET</td><td>/api/v1/keys/me</td><td>Current API key info</td></tr>
          <tr><td>GET</td><td>/health</td><td>Health check</td></tr>
        </tbody>
      </table>
    </article>
  );
}
```

**Step 5: Create Code Examples page**

`src/app/(docs)/docs/examples/page.tsx`:
```tsx
import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "Code Examples — MeetMockup Docs",
};

export default function ExamplesPage() {
  return (
    <article className="prose prose-zinc max-w-none">
      <h1>Code Examples</h1>

      <h2>Python</h2>
      <pre><code>{`import requests

response = requests.post(
    "https://api.meetmockup.com/api/v1/mockups/generate",
    headers={
        "X-API-Key": "YOUR_API_KEY",
        "Content-Type": "application/json",
    },
    json={
        "design_url": "https://example.com/design.png",
        "template_id": "white_male_front",
        "placement": {"scale": 0.4, "offset_x": 0, "offset_y": -50},
    },
)

data = response.json()
# data["mockup_url"] contains the base64 PNG`}</code></pre>

      <h2>Node.js</h2>
      <pre><code>{`const response = await fetch(
  "https://api.meetmockup.com/api/v1/mockups/generate",
  {
    method: "POST",
    headers: {
      "X-API-Key": "YOUR_API_KEY",
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      design_url: "https://example.com/design.png",
      template_id: "white_male_front",
      placement: { scale: 0.4, offset_x: 0, offset_y: -50 },
    }),
  },
);

const data = await response.json();
// data.mockup_url contains the base64 PNG`}</code></pre>

      <h2>curl</h2>
      <pre><code>{`curl -X POST https://api.meetmockup.com/api/v1/mockups/generate \\
  -H "X-API-Key: YOUR_API_KEY" \\
  -H "Content-Type: application/json" \\
  -d '{
    "design_url": "https://example.com/design.png",
    "template_id": "white_male_front",
    "placement": {
      "scale": 0.4,
      "offset_x": 0,
      "offset_y": -50
    }
  }'`}</code></pre>
    </article>
  );
}
```

**Step 6: Install Tailwind typography plugin**

```bash
pnpm add @tailwindcss/typography
```

Add to `tailwind.config.ts` plugins array: `require('@tailwindcss/typography')`.

**Step 7: Verify, commit**

```bash
pnpm dev
# Check http://localhost:3000/docs — sidebar + getting started content
git add src/app/\(docs\)/ src/components/docs/
git commit -m "feat: add docs pages (getting started, API reference, examples)"
```

---

## Task 10: Auth Pages (Clerk)

**Files:**
- Create: `src/app/(auth)/layout.tsx`
- Create: `src/app/(auth)/login/[[...login]]/page.tsx`
- Create: `src/app/(auth)/signup/[[...signup]]/page.tsx`
- Create: `src/middleware.ts`

**Step 1: Create auth layout**

`src/app/(auth)/layout.tsx`:
```tsx
export default function AuthLayout({ children }: { children: React.ReactNode }) {
  return (
    <div className="flex min-h-screen items-center justify-center bg-zinc-50">
      {children}
    </div>
  );
}
```

**Step 2: Create login page**

`src/app/(auth)/login/[[...login]]/page.tsx`:
```tsx
import { SignIn } from "@clerk/nextjs";

export default function LoginPage() {
  return <SignIn />;
}
```

**Step 3: Create signup page**

`src/app/(auth)/signup/[[...signup]]/page.tsx`:
```tsx
import { SignUp } from "@clerk/nextjs";

export default function SignupPage() {
  return <SignUp />;
}
```

**Step 4: Create middleware for protected routes**

`src/middleware.ts`:
```typescript
import { clerkMiddleware, createRouteMatcher } from "@clerk/nextjs/server";

const isProtectedRoute = createRouteMatcher(["/dashboard(.*)"]);

export default clerkMiddleware(async (auth, req) => {
  if (isProtectedRoute(req)) {
    await auth.protect();
  }
});

export const config = {
  matcher: ["/((?!_next|[^?]*\\.(?:html?|css|js(?!on)|jpe?g|webp|png|gif|svg|ttf|woff2?|ico|csv|docx?|xlsx?|zip|webmanifest)).*)"],
};
```

**Step 5: Verify, commit**

```bash
pnpm dev
# Check http://localhost:3000/login and /signup
# Note: Clerk needs valid env vars to render, will show error without them
git add src/app/\(auth\)/ src/middleware.ts
git commit -m "feat: add Clerk auth pages and route protection middleware"
```

---

## Task 11: Dashboard (Stub)

**Files:**
- Create: `src/app/(dashboard)/layout.tsx`
- Create: `src/app/(dashboard)/dashboard/page.tsx`
- Create: `src/app/(dashboard)/dashboard/keys/page.tsx`
- Create: `src/app/(dashboard)/dashboard/billing/page.tsx`
- Create: `src/components/dashboard/dashboard-nav.tsx`

**Step 1: Create dashboard nav**

`src/components/dashboard/dashboard-nav.tsx`:
```tsx
"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { UserButton } from "@clerk/nextjs";

const navItems = [
  { href: "/dashboard", label: "Overview" },
  { href: "/dashboard/keys", label: "API Keys" },
  { href: "/dashboard/billing", label: "Billing" },
];

export function DashboardNav() {
  const pathname = usePathname();

  return (
    <header className="border-b border-zinc-200 bg-white">
      <div className="mx-auto flex h-16 max-w-7xl items-center justify-between px-4 sm:px-6 lg:px-8">
        <div className="flex items-center gap-8">
          <Link href="/" className="text-xl font-bold tracking-tight">
            MeetMockup
          </Link>
          <nav className="flex items-center gap-1">
            {navItems.map((item) => (
              <Link
                key={item.href}
                href={item.href}
                className={`rounded-md px-3 py-2 text-sm font-medium transition-colors ${
                  pathname === item.href
                    ? "bg-zinc-100 text-zinc-950"
                    : "text-zinc-600 hover:bg-zinc-50 hover:text-zinc-950"
                }`}
              >
                {item.label}
              </Link>
            ))}
          </nav>
        </div>
        <UserButton />
      </div>
    </header>
  );
}
```

**Step 2: Create dashboard layout**

`src/app/(dashboard)/layout.tsx`:
```tsx
import { DashboardNav } from "@/components/dashboard/dashboard-nav";

export default function DashboardLayout({ children }: { children: React.ReactNode }) {
  return (
    <div className="min-h-screen bg-zinc-50">
      <DashboardNav />
      <main className="mx-auto max-w-7xl px-4 py-10 sm:px-6 lg:px-8">
        {children}
      </main>
    </div>
  );
}
```

**Step 3: Create dashboard overview**

`src/app/(dashboard)/dashboard/page.tsx`:
```tsx
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";

export default function DashboardPage() {
  return (
    <div>
      <h1 className="text-2xl font-bold text-zinc-950">Dashboard</h1>
      <div className="mt-6 grid gap-6 sm:grid-cols-3">
        <Card>
          <CardHeader><CardTitle className="text-sm text-zinc-500">Mockups this month</CardTitle></CardHeader>
          <CardContent><div className="text-3xl font-bold">0 / 50</div></CardContent>
        </Card>
        <Card>
          <CardHeader><CardTitle className="text-sm text-zinc-500">Plan</CardTitle></CardHeader>
          <CardContent><div className="text-3xl font-bold">Free</div></CardContent>
        </Card>
        <Card>
          <CardHeader><CardTitle className="text-sm text-zinc-500">API Keys</CardTitle></CardHeader>
          <CardContent><div className="text-3xl font-bold">0</div></CardContent>
        </Card>
      </div>
      <div className="mt-10">
        <Card>
          <CardHeader><CardTitle>Usage over time</CardTitle></CardHeader>
          <CardContent>
            <div className="flex h-48 items-center justify-center text-sm text-zinc-400">
              Usage chart — coming soon
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
```

**Step 4: Create API keys page**

`src/app/(dashboard)/dashboard/keys/page.tsx`:
```tsx
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";

export default function ApiKeysPage() {
  return (
    <div>
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold text-zinc-950">API Keys</h1>
        <Button>Create API Key</Button>
      </div>
      <div className="mt-6">
        <Card>
          <CardHeader><CardTitle>Your keys</CardTitle></CardHeader>
          <CardContent>
            <div className="flex h-32 items-center justify-center text-sm text-zinc-400">
              No API keys yet. Create one to get started.
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
```

**Step 5: Create billing page**

`src/app/(dashboard)/dashboard/billing/page.tsx`:
```tsx
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";

export default function BillingPage() {
  return (
    <div>
      <h1 className="text-2xl font-bold text-zinc-950">Billing</h1>
      <div className="mt-6 grid gap-6 sm:grid-cols-2">
        <Card>
          <CardHeader><CardTitle>Current plan</CardTitle></CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">Free</div>
            <p className="mt-1 text-sm text-zinc-500">50 mockups per month</p>
            <Button className="mt-4" variant="outline">Upgrade plan</Button>
          </CardContent>
        </Card>
        <Card>
          <CardHeader><CardTitle>Payment method</CardTitle></CardHeader>
          <CardContent>
            <div className="flex h-20 items-center justify-center text-sm text-zinc-400">
              Managed by Paddle — coming soon
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
```

**Step 6: Verify, commit**

```bash
pnpm dev
# Check http://localhost:3000/dashboard (requires Clerk auth)
git add src/app/\(dashboard\)/ src/components/dashboard/
git commit -m "feat: add stub dashboard with overview, API keys, and billing pages"
```

---

## Task 12: Generate Comparison Assets + Sample Designs

**Files:**
- Create: `public/comparisons/tshirt-flat.png`
- Create: `public/comparisons/tshirt-displaced.png`
- Create: `public/comparisons/hoodie-flat.png`
- Create: `public/comparisons/hoodie-displaced.png`
- Create: `public/comparisons/mug-flat.png`
- Create: `public/comparisons/mug-displaced.png`
- Create: `public/samples/sample-design-1.png`
- Create: `public/samples/sample-design-2.png`
- Create: `public/samples/sample-design-3.png`

**Step 1: Generate displaced mockups from the real API**

Use the live API to generate 3 mockups (tee, hoodie, mug) with a vibrant, high-contrast design.

```bash
# Pick a colorful public-domain design URL and generate each:
curl -s -X POST https://api.meetmockup.com/api/v1/mockups/generate \
  -H "Content-Type: application/json" \
  -d '{"design_url":"DESIGN_URL","template_id":"white_male_front","placement":{"scale":0.4,"offset_x":0,"offset_y":-50}}' \
  | jq -r '.mockup_url' | sed 's/data:image\/png;base64,//' | base64 -d > public/comparisons/tshirt-displaced.png
```

**Step 2: Create flat overlay versions**

Use ImageMagick or a simple script to composite the same design flat onto the base template (no displacement) for comparison. The base templates are in the r-image-magic repo at `assets/templates/*/base.png`.

**Step 3: Add 3 sample designs to `public/samples/`**

Pick 3 colorful, high-contrast designs (PNG with transparent background) that show off displacement well. These can be from your own design library or public domain sources.

**Step 4: Commit**

```bash
git add public/comparisons/ public/samples/
git commit -m "feat: add comparison assets and sample designs for demo"
```

---

## Task 13: Final Polish + Build Verification

**Step 1: Run production build**

```bash
pnpm build
```

Fix any TypeScript errors or build warnings.

**Step 2: Test all routes**

Visit each route and verify rendering:
- `/` — all 8 homepage sections
- `/pricing` — tier cards
- `/demo` — upload + template strip + result
- `/templates` — category filter + grid
- `/docs` — sidebar + content
- `/docs/api-reference`
- `/docs/examples`
- `/login` — Clerk sign-in
- `/signup` — Clerk sign-up
- `/dashboard` — overview (requires auth)
- `/dashboard/keys`
- `/dashboard/billing`

**Step 3: Mobile responsive check**

Open Chrome DevTools, test all pages at 375px width (iPhone SE).

**Step 4: Commit any fixes**

```bash
git add -A
git commit -m "fix: build errors and responsive polish"
```

---

## Not in v1 (Defer)

- Stripe/Paddle integration (webhook + checkout)
- Real usage data in dashboard
- API key creation flow (needs backend endpoint wiring)
- Blog / changelog / case studies
- Design placement editor
- Annual billing toggle
- Team management
- SDK docs
- Full docs platform (Mintlify/Nextra)
- SEO metadata for all pages
- OG images
- Analytics (Plausible/PostHog)
