"use client";

import Image from "next/image";
import { useState } from "react";

import { templateCategories } from "@/data/templates";

interface TemplateGridProps {
  initialType?: string | null;
}

export function TemplateGrid({ initialType = null }: TemplateGridProps) {
  const [activeType, setActiveType] = useState<string | null>(initialType);

  const filteredCategories = activeType
    ? templateCategories.filter((category) => category.productType === activeType)
    : templateCategories;

  return (
    <div>
      <div className="flex flex-wrap gap-2">
        <button
          onClick={() => setActiveType(null)}
          className={`rounded-full px-4 py-2 text-sm font-medium transition ${
            activeType === null
              ? "bg-[#151b26] text-white"
              : "bg-white text-muted-foreground ring-1 ring-black/8 hover:text-foreground"
          }`}
        >
          All categories
        </button>
        {templateCategories.map((category) => (
          <button
            key={category.productType}
            onClick={() => setActiveType(category.productType)}
            className={`rounded-full px-4 py-2 text-sm font-medium transition ${
              activeType === category.productType
                ? "bg-[#151b26] text-white"
                : "bg-white text-muted-foreground ring-1 ring-black/8 hover:text-foreground"
            }`}
          >
            {category.label} ({category.count})
          </button>
        ))}
      </div>

      <div className="mt-8 grid gap-4 md:grid-cols-2 xl:grid-cols-3">
        {filteredCategories.map((category) => (
          <div
            key={category.productType}
            className="overflow-hidden rounded-[1.9rem] border border-black/8 bg-white/84 shadow-[0_16px_50px_rgba(15,18,24,0.06)]"
          >
            <div className="relative aspect-[4/3] bg-[#eef2f7]">
              <Image
                src={category.thumbnailSrc}
                alt={category.label}
                fill
                className="object-cover"
                sizes="(max-width: 1280px) 50vw, 24rem"
              />
            </div>
            <div className="px-5 py-5">
              <div className="flex items-start justify-between gap-3">
                <div>
                  <h3 className="text-lg font-semibold text-foreground">
                    {category.label}
                  </h3>
                  <p className="mt-1 text-sm text-muted-foreground">
                    {category.count} live templates
                  </p>
                </div>
                <span className="rounded-full bg-muted px-3 py-1 text-xs font-semibold uppercase tracking-[0.18em] text-muted-foreground">
                  {category.productType}
                </span>
              </div>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
