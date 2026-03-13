"use client";

import Image from "next/image";
import { useRef, useState } from "react";

interface ComparisonPair {
  label: string;
  before: string;
  after: string;
}

const pairs: ComparisonPair[] = [
  {
    label: "T-Shirt",
    before: "/comparisons/tshirt-flat.png",
    after: "/comparisons/tshirt-displaced.png",
  },
  {
    label: "Hoodie",
    before: "/comparisons/hoodie-flat.png",
    after: "/comparisons/hoodie-displaced.png",
  },
  {
    label: "Mug",
    before: "/comparisons/mug-flat.png",
    after: "/comparisons/mug-displaced.png",
  },
];

export function BeforeAfter() {
  const [activeIndex, setActiveIndex] = useState(0);
  const [sliderPosition, setSliderPosition] = useState(54);
  const containerRef = useRef<HTMLDivElement>(null);
  const draggingRef = useRef(false);

  const pair = pairs[activeIndex];

  const updatePosition = (clientX: number) => {
    if (!containerRef.current) return;

    const rect = containerRef.current.getBoundingClientRect();
    const x = Math.max(0, Math.min(clientX - rect.left, rect.width));
    setSliderPosition((x / rect.width) * 100);
  };

  return (
    <section className="bg-background py-20 sm:py-24">
      <div className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
        <div className="section-frame px-6 py-8 sm:px-8 sm:py-10">
          <div className="flex flex-col gap-8 lg:flex-row lg:items-end lg:justify-between">
            <div className="section-copy">
              <span className="kicker">See the difference</span>
              <h2 className="mt-5 font-display text-3xl font-semibold tracking-[-0.06em] text-foreground sm:text-4xl">
                Flat overlays look fake. Displacement mapping looks real.
              </h2>
              <p className="mt-4 text-lg leading-8 text-muted-foreground">
                Drag the slider and switch product types. The design keeps its
                shape, but the product texture comes through instead of getting
                buried under a square sticker effect.
              </p>
            </div>
            <div className="flex flex-wrap gap-2">
              {pairs.map((item, index) => (
                <button
                  key={item.label}
                  onClick={() => {
                    setActiveIndex(index);
                    setSliderPosition(54);
                  }}
                  className={`rounded-full px-4 py-2 text-sm font-medium transition ${
                    index === activeIndex
                      ? "bg-[#151b26] text-white shadow-lg"
                      : "bg-white text-muted-foreground ring-1 ring-black/8 hover:text-foreground"
                  }`}
                >
                  {item.label}
                </button>
              ))}
            </div>
          </div>

          <div
            ref={containerRef}
            className="relative mt-10 aspect-[4/3] overflow-hidden rounded-[2rem] border border-black/8 bg-[#dde2ea] shadow-[0_20px_60px_rgba(15,18,24,0.12)]"
            onMouseDown={(event) => {
              draggingRef.current = true;
              updatePosition(event.clientX);
            }}
            onMouseMove={(event) => {
              if (draggingRef.current) updatePosition(event.clientX);
            }}
            onMouseUp={() => {
              draggingRef.current = false;
            }}
            onMouseLeave={() => {
              draggingRef.current = false;
            }}
            onTouchStart={(event) => {
              draggingRef.current = true;
              updatePosition(event.touches[0].clientX);
            }}
            onTouchMove={(event) => updatePosition(event.touches[0].clientX)}
            onTouchEnd={() => {
              draggingRef.current = false;
            }}
          >
            <div className="absolute inset-0">
              <Image
                src={pair.before}
                alt={`${pair.label} flat overlay mockup`}
                fill
                className="object-contain"
                priority
                sizes="(max-width: 1280px) 100vw, 70rem"
              />
            </div>
            <div
              className="absolute inset-0 overflow-hidden"
              style={{ clipPath: `inset(0 0 0 ${sliderPosition}%)` }}
            >
              <Image
                src={pair.after}
                alt={`${pair.label} MeetMockup displaced mockup`}
                fill
                className="object-contain"
                priority
                sizes="(max-width: 1280px) 100vw, 70rem"
              />
            </div>
            <div
              className="absolute inset-y-0 w-0.5 bg-white shadow-[0_0_0_1px_rgba(15,18,24,0.08)]"
              style={{ left: `${sliderPosition}%` }}
            >
              <div className="absolute left-1/2 top-1/2 flex h-12 w-12 -translate-x-1/2 -translate-y-1/2 items-center justify-center rounded-full border border-white/70 bg-white text-xs font-semibold tracking-[0.2em] text-[#151b26] shadow-xl">
                DRAG
              </div>
            </div>
            <div className="absolute left-4 top-4 rounded-full bg-black/40 px-3 py-1 text-xs font-medium uppercase tracking-[0.18em] text-white/88 backdrop-blur">
              Flat overlay
            </div>
            <div className="absolute right-4 top-4 rounded-full bg-white/88 px-3 py-1 text-xs font-medium uppercase tracking-[0.18em] text-[#151b26] shadow">
              MeetMockup
            </div>
          </div>

          <div className="mt-5 flex flex-wrap items-center justify-between gap-3 text-sm text-muted-foreground">
            <span>Same artwork. Same placement. Different level of realism.</span>
            <span>Move the slider to compare every wrinkle and shadow.</span>
          </div>
        </div>
      </div>
    </section>
  );
}
