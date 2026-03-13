"use client";

import Image from "next/image";
import { useEffect, useState } from "react";

const slides = [
  {
    title: "White tee",
    metric: "1.4s generation",
    stat: "Displacement-mapped texture",
    imageSrc: "/comparisons/tshirt-displaced.png",
  },
  {
    title: "AOP hoodie",
    metric: "44 templates live",
    stat: "Fabric folds stay visible",
    imageSrc: "/comparisons/hoodie-displaced.png",
  },
  {
    title: "Ceramic mug",
    metric: "REST-ready workflow",
    stat: "Preview and API share the same engine",
    imageSrc: "/comparisons/mug-displaced.png",
  },
];

export function HeroRotator() {
  const [activeIndex, setActiveIndex] = useState(0);

  useEffect(() => {
    const timer = window.setInterval(() => {
      setActiveIndex((current) => (current + 1) % slides.length);
    }, 3200);

    return () => window.clearInterval(timer);
  }, []);

  return (
    <div className="relative mx-auto w-full max-w-[34rem]">
      <div className="absolute inset-8 rounded-[2.6rem] bg-[radial-gradient(circle_at_top,rgba(243,214,92,0.26),transparent_45%)] blur-3xl" />
      <div className="relative overflow-hidden rounded-[2.4rem] border border-white/10 bg-white/6 p-4 shadow-[0_30px_120px_rgba(0,0,0,0.34)] backdrop-blur-xl">
        <div className="absolute inset-x-0 top-0 h-px bg-[linear-gradient(90deg,transparent,rgba(243,214,92,0.75),transparent)]" />
        <div className="flex items-center justify-between border-b border-white/10 px-2 pb-4">
          <div>
            <p className="text-sm font-semibold text-white">{slides[activeIndex].title}</p>
            <p className="text-xs uppercase tracking-[0.2em] text-white/50">
              MeetMockup preview
            </p>
          </div>
          <div className="rounded-full border border-white/10 bg-white/8 px-3 py-1 text-xs text-white/72">
            {slides[activeIndex].metric}
          </div>
        </div>
        <div className="relative mt-4 aspect-[4/5] overflow-hidden rounded-[2rem] bg-[#0f1218]">
          {slides.map((slide, index) => (
            <div
              key={slide.title}
              className={`absolute inset-0 transition-all duration-700 ${
                index === activeIndex
                  ? "translate-y-0 opacity-100"
                  : "translate-y-4 opacity-0"
              }`}
            >
              <Image
                src={slide.imageSrc}
                alt={slide.title}
                fill
                className="object-contain"
                priority={index === 0}
                sizes="(max-width: 1024px) 100vw, 34rem"
              />
            </div>
          ))}
          <div className="absolute inset-x-0 bottom-0 bg-[linear-gradient(180deg,transparent,rgba(15,18,24,0.88))] px-6 py-6">
            <p className="text-sm font-medium text-white/88">
              {slides[activeIndex].stat}
            </p>
          </div>
        </div>
        <div className="mt-4 flex items-center justify-between gap-3">
          <div className="flex gap-2">
            {slides.map((slide, index) => (
              <button
                key={slide.title}
                onClick={() => setActiveIndex(index)}
                className={`h-2 rounded-full transition-all ${
                  index === activeIndex ? "w-10 bg-[var(--brand-signal)]" : "w-5 bg-white/20"
                }`}
                aria-label={`Show ${slide.title}`}
              />
            ))}
          </div>
          <p className="text-xs uppercase tracking-[0.22em] text-white/45">
            Real assets from the engine
          </p>
        </div>
      </div>
    </div>
  );
}
