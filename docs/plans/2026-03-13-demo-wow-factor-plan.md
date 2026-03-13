# Demo WOW-Factor Polish — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make the interactive demo feel instant and impressive by adding optimistic color tinting, a live speed timer, smooth crossfade transitions, and micro-animations — no new features, pure polish.

**Architecture:** All changes are client-side React/CSS. No API changes. The key trick is a CSS `mix-blend-mode: multiply` overlay for instant color preview while the real server render loads behind the scenes. A `requestAnimationFrame`-based timer creates visible speed proof.

**Tech Stack:** React 19, Next.js 15, Tailwind CSS v4, tw-animate-css (already installed), CSS transitions, requestAnimationFrame

**Design doc:** `docs/plans/2026-03-13-demo-wow-factor-design.md`

---

### Task 1: SpeedBadge Component

Create the live counting timer badge that ticks during generation and freezes with a pop when the result arrives.

**Files:**
- Create: `apps/web/src/components/demo/speed-badge.tsx`

**Step 1: Create SpeedBadge component**

```tsx
"use client";

import { useEffect, useRef, useState } from "react";

interface SpeedBadgeProps {
  isRunning: boolean;
  finalTimeMs: number | null;
}

export function SpeedBadge({ isRunning, finalTimeMs }: SpeedBadgeProps) {
  const [displayTime, setDisplayTime] = useState(0);
  const startRef = useRef(0);
  const rafRef = useRef(0);
  const [didPop, setDidPop] = useState(false);

  useEffect(() => {
    if (isRunning) {
      setDidPop(false);
      startRef.current = performance.now();

      const tick = () => {
        setDisplayTime(performance.now() - startRef.current);
        rafRef.current = requestAnimationFrame(tick);
      };
      rafRef.current = requestAnimationFrame(tick);

      return () => cancelAnimationFrame(rafRef.current);
    }
  }, [isRunning]);

  useEffect(() => {
    if (!isRunning && finalTimeMs !== null) {
      cancelAnimationFrame(rafRef.current);
      setDisplayTime(finalTimeMs);
      setDidPop(true);
      const timeout = setTimeout(() => setDidPop(false), 300);
      return () => clearTimeout(timeout);
    }
  }, [isRunning, finalTimeMs]);

  const seconds = (displayTime / 1000).toFixed(1);

  if (!isRunning && finalTimeMs === null) return null;

  return (
    <span
      className={`inline-flex items-center rounded-full border border-black/10 px-3 py-0.5 text-sm font-medium tabular-nums text-muted-foreground transition-transform duration-200 ${
        didPop ? "scale-115" : "scale-100"
      }`}
    >
      {isRunning ? (
        <>
          <span className="mr-1.5 size-1.5 animate-pulse rounded-full bg-amber-500" />
          {seconds}s
        </>
      ) : (
        <>Generated in {seconds}s</>
      )}
    </span>
  );
}
```

**Step 2: Verify it renders**

Run: `cd /Volumes/T7\ Storage/Projects/r-image-magic/apps/web && pnpm build`
Expected: Build succeeds with no type errors.

**Step 3: Commit**

```bash
git add apps/web/src/components/demo/speed-badge.tsx
git commit -m "feat(demo): add SpeedBadge live timer component"
```

---

### Task 2: Crossfade MockupResult — Always Show a Mockup

Replace the spinner with crossfade transitions. Keep the previous mockup visible during generation with a dim overlay. New results fade in over the old.

**Files:**
- Modify: `apps/web/src/components/demo/mockup-result.tsx`

**Step 1: Rewrite MockupResult with crossfade and overlay states**

Replace the entire component. Key changes:
- Add `previousUrlRef` to hold the last mockup URL
- During `isGenerating`: show previous image with dim overlay (no spinner)
- On new result: crossfade via opacity transition on two stacked images
- First load (no previous result): show skeleton shimmer instead of spinner
- Integrate `SpeedBadge` in the badge area, replacing the static badge

```tsx
import { useEffect, useRef, useState } from "react";

import { Badge } from "@/components/ui/badge";
import { ButtonLink } from "@/components/ui/button-link";
import { SpeedBadge } from "@/components/demo/speed-badge";

interface MockupResultProps {
  result: {
    mockupUrl: string;
    generationTimeMs: number;
    templateLabel: string;
  } | null;
  isGenerating: boolean;
  error: string | null;
  noticeMessage?: string | null;
  compact?: boolean;
}

export function MockupResult({
  result,
  isGenerating,
  error,
  noticeMessage,
  compact = false,
}: MockupResultProps) {
  const frameClassName = compact ? "aspect-[4/3]" : "aspect-[4/5]";
  const previousUrlRef = useRef<string | null>(null);
  const [showNew, setShowNew] = useState(false);

  useEffect(() => {
    if (result?.mockupUrl && result.mockupUrl !== previousUrlRef.current) {
      setShowNew(false);
      const raf = requestAnimationFrame(() => setShowNew(true));
      return () => cancelAnimationFrame(raf);
    }
  }, [result?.mockupUrl]);

  useEffect(() => {
    if (result?.mockupUrl) {
      previousUrlRef.current = result.mockupUrl;
    }
  }, [result?.mockupUrl]);

  if (error) {
    return (
      <div className="section-frame flex min-h-96 flex-col items-center justify-center px-6 py-10 text-center">
        <div className="max-w-lg">
          <h3 className="text-xl font-semibold text-foreground">
            Demo unavailable
          </h3>
          <p className="mt-3 leading-7 text-muted-foreground">{error}</p>
          <div className="mt-6">
            <ButtonLink href="/signup">Sign up for API access</ButtonLink>
          </div>
        </div>
      </div>
    );
  }

  const hasAnyImage = result?.mockupUrl || previousUrlRef.current;

  return (
    <div className="section-frame px-5 py-5">
      <div
        className={`relative overflow-hidden rounded-[1.6rem] bg-[#e7ebf1] ${frameClassName}`}
      >
        {hasAnyImage ? (
          <>
            {/* Previous image — always visible as base layer */}
            {previousUrlRef.current &&
              previousUrlRef.current !== result?.mockupUrl && (
                /* eslint-disable-next-line @next/next/no-img-element */
                <img
                  src={previousUrlRef.current}
                  alt="Previous mockup"
                  className="absolute inset-0 h-full w-full object-contain"
                />
              )}

            {/* Current result — fades in */}
            {result && (
              /* eslint-disable-next-line @next/next/no-img-element */
              <img
                src={result.mockupUrl}
                alt={`Generated ${result.templateLabel} mockup`}
                className={`absolute inset-0 h-full w-full object-contain transition-opacity duration-300 ${
                  showNew ? "opacity-100" : "opacity-0"
                }`}
              />
            )}

            {/* Dim overlay during generation */}
            <div
              className={`absolute inset-0 bg-black/8 transition-opacity duration-300 ${
                isGenerating ? "opacity-100" : "opacity-0 pointer-events-none"
              }`}
            />

            {/* Watermark badge */}
            {result && !isGenerating && (
              <div className="absolute bottom-4 right-4 rounded-full bg-black/50 px-3 py-1 text-[0.72rem] font-semibold uppercase tracking-[0.2em] text-white/84 backdrop-blur">
                MeetMockup preview
              </div>
            )}
          </>
        ) : isGenerating ? (
          /* First load — skeleton shimmer */
          <div className="flex h-full items-center justify-center">
            <div className="text-center">
              <div className="mx-auto h-48 w-36 animate-pulse rounded-2xl bg-gradient-to-br from-[#dde2ea] to-[#c8cfd9]" />
              <p className="mt-4 text-sm font-medium text-muted-foreground">
                Generating your first preview...
              </p>
            </div>
          </div>
        ) : (
          /* No result, not generating — placeholder */
          <div className="flex h-full items-center justify-center px-8 text-center">
            <div>
              <p className="text-sm font-medium text-muted-foreground">
                {noticeMessage ?? "Your generated mockup will appear here."}
              </p>
              <p className="mt-2 text-sm text-muted-foreground">
                {noticeMessage
                  ? "Add the required env and refresh to enable live previews."
                  : "Choose a sample design to render the first result."}
              </p>
            </div>
          </div>
        )}
      </div>
      <div className="mt-4 flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
        <div className="flex flex-wrap items-center gap-2">
          <Badge variant="secondary" className="rounded-full">
            Preview quality
          </Badge>
          <SpeedBadge
            isRunning={isGenerating}
            finalTimeMs={result?.generationTimeMs ?? null}
          />
        </div>
        <div className="flex flex-wrap gap-3">
          {result && !isGenerating ? (
            <a
              href={result.mockupUrl}
              download="meetmockup-preview.png"
              className="inline-flex h-8 items-center justify-center rounded-lg border border-black/10 bg-white px-3 text-sm font-medium text-foreground transition hover:bg-muted"
            >
              Download preview
            </a>
          ) : null}
          <ButtonLink href="/signup">Sign up for full quality</ButtonLink>
        </div>
      </div>
    </div>
  );
}
```

**Step 2: Verify build**

Run: `cd /Volumes/T7\ Storage/Projects/r-image-magic/apps/web && pnpm build`
Expected: Build succeeds.

**Step 3: Commit**

```bash
git add apps/web/src/components/demo/mockup-result.tsx
git commit -m "feat(demo): replace spinner with crossfade transitions and skeleton"
```

---

### Task 3: Optimistic Color Tinting

Add a CSS `mix-blend-mode: multiply` overlay that instantly previews the tint color while the real render loads.

**Files:**
- Modify: `apps/web/src/components/demo/mockup-result.tsx` (add overlay div)
- Modify: `apps/web/src/components/demo/demo-experience.tsx` (pass tint state)

**Step 1: Add `tintHex` prop to MockupResult**

In `mockup-result.tsx`, add to the props interface:

```tsx
interface MockupResultProps {
  result: { mockupUrl: string; generationTimeMs: number; templateLabel: string } | null;
  isGenerating: boolean;
  error: string | null;
  noticeMessage?: string | null;
  compact?: boolean;
  tintHex?: string | null;  // <-- add this
}
```

Destructure it:
```tsx
export function MockupResult({
  result,
  isGenerating,
  error,
  noticeMessage,
  compact = false,
  tintHex,           // <-- add this
}: MockupResultProps) {
```

**Step 2: Add the color overlay div**

Inside the `{hasAnyImage ? ( <>...</> )` block, after the dim overlay div and before the watermark badge, add:

```tsx
{/* Optimistic color tint overlay */}
{tintHex && tintHex !== "FFFFFF" && (
  <div
    className="absolute inset-0 transition-opacity duration-200"
    style={{
      backgroundColor: `#${tintHex}`,
      mixBlendMode: "multiply",
      opacity: isGenerating ? 1 : 0,
      pointerEvents: "none",
    }}
  />
)}
```

**Logic:** The overlay is visible (`opacity: 1`) only during generation. Once the real server result crossfades in, the overlay hides (`opacity: 0`) because `isGenerating` becomes false. The real render already has the correct tint baked in.

**Step 3: Pass `tintHex` from DemoExperience**

In `demo-experience.tsx`, update the `MockupResult` usage (around line 232):

```tsx
<MockupResult
  result={result}
  isGenerating={isGenerating}
  error={error}
  noticeMessage={!demoReady ? setupNotice : null}
  compact={compact}
  tintHex={TINTABLE_PRODUCT_TYPES.includes(selectedTemplate.type) ? selectedColor.hex : null}
/>
```

**Step 4: Verify build**

Run: `cd /Volumes/T7\ Storage/Projects/r-image-magic/apps/web && pnpm build`
Expected: Build succeeds.

**Step 5: Commit**

```bash
git add apps/web/src/components/demo/mockup-result.tsx apps/web/src/components/demo/demo-experience.tsx
git commit -m "feat(demo): add optimistic color tint overlay with multiply blend"
```

---

### Task 4: Micro-animations — Selection Bounce and Swatch Polish

Add subtle scale bounce on template/design clicks, and hover pulse on color swatches.

**Files:**
- Modify: `apps/web/src/components/demo/template-strip.tsx`
- Modify: `apps/web/src/components/demo/color-strip.tsx`

**Step 1: Add click bounce to template buttons**

In `template-strip.tsx`, the `<button>` at line 30 currently has `transition` in its className. Replace the button's className with:

```tsx
className={`group min-w-[8.25rem] rounded-[1.4rem] border px-3 py-3 text-left transition-all duration-150 active:scale-[0.96] ${
  template.id === selectedTemplateId
    ? "border-[#151b26] bg-[#151b26] text-white shadow-lg"
    : "border-black/8 bg-white hover:border-[#151b26]/35"
}`}
```

The key addition is `active:scale-[0.96]` and `transition-all duration-150` — gives a satisfying press-down feel on click.

**Step 2: Add hover pulse to color swatches**

In `color-strip.tsx`, the `<button>` at line 33 currently has hover ring styles. Update the className to add a hover scale:

```tsx
className={`relative size-9 rounded-full transition-all duration-150 ${
  isWhite ? "border border-black/12" : ""
} ${
  isSelected
    ? "ring-2 ring-[#151b26] ring-offset-2 scale-110"
    : "hover:ring-2 hover:ring-[#151b26]/30 hover:ring-offset-2 hover:scale-110"
} ${disabled ? "cursor-not-allowed opacity-50" : "cursor-pointer active:scale-95"}`}
```

Key additions: `scale-110` on selected (pop up), `hover:scale-110` on hover, `active:scale-95` on click.

**Step 3: Verify build**

Run: `cd /Volumes/T7\ Storage/Projects/r-image-magic/apps/web && pnpm build`
Expected: Build succeeds.

**Step 4: Commit**

```bash
git add apps/web/src/components/demo/template-strip.tsx apps/web/src/components/demo/color-strip.tsx
git commit -m "feat(demo): add selection bounce and swatch hover animations"
```

---

### Task 5: Download Button Slide-In

Make the download button slide in from the right when a result arrives.

**Files:**
- Modify: `apps/web/src/components/demo/mockup-result.tsx`

**Step 1: Add slide-in animation to download button**

In `mockup-result.tsx`, wrap the download `<a>` tag with a transition. Replace the download button block:

```tsx
{result && !isGenerating ? (
  <a
    href={result.mockupUrl}
    download="meetmockup-preview.png"
    className="inline-flex h-8 items-center justify-center rounded-lg border border-black/10 bg-white px-3 text-sm font-medium text-foreground transition-all duration-300 hover:bg-muted animate-in slide-in-from-right-4 fade-in"
  >
    Download preview
  </a>
) : null}
```

The `animate-in slide-in-from-right-4 fade-in` classes come from `tw-animate-css` which is already installed.

**Step 2: Verify build**

Run: `cd /Volumes/T7\ Storage/Projects/r-image-magic/apps/web && pnpm build`
Expected: Build succeeds.

**Step 3: Commit**

```bash
git add apps/web/src/components/demo/mockup-result.tsx
git commit -m "feat(demo): add slide-in animation for download button"
```

---

### Task 6: Generate New Before/After Comparison Images

The current comparison images (white t-shirt, small design) barely show the displacement difference. Generate new pairs that dramatically showcase the displacement mapping.

**Files:**
- Replace: `apps/web/public/comparisons/tshirt-flat.jpg` and `tshirt-displaced.jpg`
- Replace: `apps/web/public/comparisons/hoodie-flat.png` and `hoodie-displaced.png`
- Replace: `apps/web/public/comparisons/mug-flat.png` and `mug-displaced.png`

**Step 1: Generate comparison pairs via the API**

Use the MeetMockup API to generate two versions of each product:
1. **Flat version**: Use a simple composite overlay (no displacement) — or if the API doesn't support disabling displacement, use a basic image editor to paste the design flat onto the template
2. **Displaced version**: Normal API output with full displacement mapping

Requirements for maximum visual impact:
- **T-Shirt**: Use a dark or medium-colored template (not white). Pick a large, colorful design that spans the full chest area. The fabric folds and wrinkles should be clearly visible.
- **Hoodie**: Use a heathered or textured fabric template. Large graphic design. The hoodie pocket area and seams create great displacement opportunities.
- **Mug**: The curved ceramic surface should show the design wrapping. Use a design with straight lines or geometric shapes so the curve distortion is obvious.

**Step 2: Replace image files**

Save the new images at the same paths:
- `apps/web/public/comparisons/tshirt-flat.jpg`
- `apps/web/public/comparisons/tshirt-displaced.jpg`
- `apps/web/public/comparisons/hoodie-flat.png`
- `apps/web/public/comparisons/hoodie-displaced.png`
- `apps/web/public/comparisons/mug-flat.png`
- `apps/web/public/comparisons/mug-displaced.png`

No code changes needed — filenames stay the same.

**Note:** This task requires manual image generation. The implementer should use the live API at `https://api.meetmockup.com/api/v1/mockups/generate` with different templates and a colorful sample design. For the "flat" versions, take the same template blank and paste the design on top without displacement (e.g., in Figma/Photoshop, or ask the API team for a `disable_displacement` flag).

**Step 3: Verify images render in the slider**

Run: `cd /Volumes/T7\ Storage/Projects/r-image-magic/apps/web && pnpm dev`
Navigate to the landing page and verify the before/after slider shows a dramatic difference.

**Step 4: Commit**

```bash
git add apps/web/public/comparisons/
git commit -m "feat(demo): replace comparison images with high-displacement examples"
```

---

### Task 7: Final Build Verification and Lint

**Step 1: Run lint**

Run: `cd /Volumes/T7\ Storage/Projects/r-image-magic/apps/web && pnpm lint`
Expected: No errors.

**Step 2: Run build**

Run: `cd /Volumes/T7\ Storage/Projects/r-image-magic/apps/web && pnpm build`
Expected: Build succeeds.

**Step 3: Visual QA checklist**

Run `pnpm dev` and verify in browser:
- [ ] Click a color swatch → tint overlay appears instantly → real render crossfades in
- [ ] Speed badge counts up during generation → freezes with pop when done
- [ ] First load shows skeleton shimmer, not spinner
- [ ] Subsequent generations keep previous mockup visible with dim overlay
- [ ] Template buttons have press-down bounce on click
- [ ] Color swatches scale up on hover and selection
- [ ] Download button slides in from right when result appears
- [ ] Before/after slider shows dramatic displacement difference

**Step 4: Final commit if any fixes needed**

```bash
git add -u
git commit -m "fix(demo): address visual QA feedback"
```
