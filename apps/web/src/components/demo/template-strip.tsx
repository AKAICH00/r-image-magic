"use client";

import Image from "next/image";

import type { DemoTemplate } from "@/lib/demo-data";

interface TemplateStripProps {
  templates: DemoTemplate[];
  selectedTemplateId: string;
  onSelect: (template: DemoTemplate) => void;
}

export function TemplateStrip({
  templates,
  selectedTemplateId,
  onSelect,
}: TemplateStripProps) {
  return (
    <div className="section-frame px-5 py-5">
      <div className="flex items-center justify-between gap-4">
        <div>
          <h3 className="text-base font-semibold text-foreground">Pick a template</h3>
          <p className="mt-1 text-sm text-muted-foreground">
            Click a product and the demo will regenerate automatically.
          </p>
        </div>
      </div>
      <div className="mt-5 flex gap-3 overflow-x-auto pb-2">
        {templates.map((template, index) => (
          <button
            key={template.id}
            onClick={() => onSelect(template)}
            className={`group min-w-[8.25rem] rounded-[1.4rem] border px-3 py-3 text-left transition ${
              template.id === selectedTemplateId
                ? "border-[#151b26] bg-[#151b26] text-white shadow-lg"
                : "border-black/8 bg-white hover:border-[#151b26]/35"
            }`}
          >
            <div className="relative aspect-square overflow-hidden rounded-[1rem] bg-[#eef2f7]">
              <Image
                src={template.previewSrc}
                alt={template.label}
                fill
                className="object-contain transition duration-300 group-hover:scale-[1.02]"
                priority={index === 0}
                sizes="8.25rem"
              />
            </div>
            <span className="mt-3 block text-sm font-semibold">{template.label}</span>
            <span
              className={`text-xs uppercase tracking-[0.18em] ${
                template.id === selectedTemplateId ? "text-white/62" : "text-muted-foreground"
              }`}
            >
              {template.type}
            </span>
          </button>
        ))}
      </div>
    </div>
  );
}
