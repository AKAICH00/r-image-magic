# Demo WOW-Factor Polish

**Date:** 2026-03-13
**Status:** Approved
**Next phase:** Multi-product gallery generation (separate design)

## Goal

Make the interactive demo feel instant, impressive, and convincing to POD sellers evaluating the API. No new features — pure polish on the existing flow.

## Audience

- **Primary:** Print-on-demand sellers who want to see mockup generation in action
- **Secondary:** Developers evaluating the API, general visitors/investors

## Decisions

| Area | Decision |
|---|---|
| Speed | Feel instant + prove it with a visible live timer |
| Color UX | Client-side CSS tint preview, silent server swap (crossfade) |
| Scope | Polish only — no new API features, no layout restructure |
| Before/After | Replace comparison images with high-displacement templates |

## 1. Optimistic Color Tinting

**Problem:** Clicking a color swatch triggers a full API round-trip. The spinner kills the illusion of speed.

**Solution:** Instant client-side preview with server confirmation.

- On swatch click: immediately apply a CSS `mix-blend-mode: multiply` color overlay on the current mockup image using a positioned `<div>` with the selected color
- Simultaneously fire the real `/api/demo/generate` call with `tint_color`
- When the server result arrives, crossfade (300ms opacity transition) from the CSS preview to the real rendered image
- If the server call fails, remove the overlay and show the previous result

**Why this works:** The CSS multiply blend is the same math the Rust compositor uses. The preview will be visually close enough that the crossfade swap is imperceptible. Color changes feel zero-latency.

**Implementation:**
- Add a color overlay `<div>` in `MockupResult` with `mix-blend-mode: multiply`, `pointer-events: none`
- Overlay color comes from parent state; opacity toggles between 0 (real image shown) and 1 (preview shown)
- On new server result: set overlay opacity to 0 with CSS transition, swap `src`

## 2. Live Speed Badge

**Problem:** The current "Generated in X.Xs" badge appears after the fact. It's passive.

**Solution:** A live counting timer that creates tension and payoff.

- When API call fires, show a small counter ticking up from `0.0s` using `requestAnimationFrame`
- Counter is positioned in the badge area below the result
- When the result arrives, freeze the counter with a brief scale pop animation (scale 1 → 1.15 → 1, 200ms)
- Final badge reads: "Generated in 0.8s" (or whatever the actual time was)
- The ticking creates anticipation; the fast freeze creates the "wow, that was quick" moment

**Implementation:**
- New `SpeedBadge` component with `isRunning` and `finalTimeMs` props
- Uses `useRef` + `requestAnimationFrame` loop for smooth counting
- Parent controls start/stop via state

## 3. Smooth State Transitions

**Problem:** The spinner ("Generating your preview...") is a dead state. Users see a blank area.

**Solution:** Always show a mockup. Never show a spinner after the first result.

### First load (no result yet):
- Skeleton shimmer in the result frame (pulsing gradient, same aspect ratio)
- Text: "Generating your first preview..."

### Subsequent generations (result already exists):
- Keep the previous mockup visible
- Apply a subtle dim overlay (black at 8% opacity) + soft pulse animation
- The live speed counter ticks in the badge area
- When new result arrives: crossfade new image over old (300ms)

**Result:** The user always sees a product mockup. The transition between states is fluid, not jarring.

**Implementation:**
- `MockupResult` maintains a `previousUrl` ref
- During generation: show previous image with dim overlay
- On new result: crossfade via CSS opacity transition on two stacked `<img>` elements

## 4. Before/After Comparison Images

**Problem:** Current comparison images (white t-shirt, small design) show almost no visible displacement difference. The slider is underwhelming.

**Solution:** Replace with templates that dramatically showcase displacement mapping.

**Requirements for new comparison pairs:**
- **Hoodie:** Dark heathered fabric with visible creases and folds. Large, colorful design that wraps across the chest. Flat version should look obviously "stickered on" with sharp rectangular edges.
- **T-Shirt:** Crumpled or naturally draped (not flat-lay). Medium-dark color. Design should visibly warp along fabric folds in the displaced version.
- **Mug:** Curved ceramic surface. Design should wrap around the curve in displaced version vs. appear flat/distorted in the flat version.

**The flat versions should be generated without displacement mapping** (just a straight composite overlay) so the difference is stark and obvious.

## 5. Micro-polish

Small details that compound into a feeling of quality:

- **Template/design selection:** Subtle scale bounce on click (`transform: scale(0.95)` → `scale(1)`, 150ms ease-out)
- **Result image entrance:** Fade-in with soft shadow bloom on first appearance
- **Color swatches:** Brief ring-pulse on hover (ring expands outward and fades)
- **Download button:** Slides in from right when result loads (translateX animation)
- **Swatch selected state:** Brief "pop" scale animation on selection change

## What This Does NOT Include

- No drag-to-reposition (future)
- No multi-product gallery generation (next phase)
- No new API endpoints or features
- No layout restructure
- No new dependencies (all CSS transitions + requestAnimationFrame)

## Files to Modify

| File | Changes |
|---|---|
| `components/demo/mockup-result.tsx` | Color overlay layer, crossfade transitions, skeleton state, speed badge integration |
| `components/demo/demo-experience.tsx` | Optimistic color state, timer state management |
| `components/demo/color-strip.tsx` | Hover animations, selection pop |
| `components/demo/template-strip.tsx` | Click bounce animation |
| `components/demo/speed-badge.tsx` | New component — live counting timer |
| `components/marketing/before-after.tsx` | Update image paths if filenames change |
| `public/comparisons/*` | Replace with high-displacement comparison images |
