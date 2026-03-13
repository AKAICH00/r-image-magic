"use client";

import type { MockupColor } from "@/lib/colors";

interface ColorStripProps {
  colors: MockupColor[];
  selectedId: string;
  onSelect: (color: MockupColor) => void;
  disabled?: boolean;
}

export function ColorStrip({
  colors,
  selectedId,
  onSelect,
  disabled,
}: ColorStripProps) {
  return (
    <div className="section-frame px-5 py-5">
      <div>
        <h3 className="text-base font-semibold text-foreground">
          Product color
        </h3>
        <p className="mt-1 text-sm text-muted-foreground">
          {colors.find((c) => c.id === selectedId)?.name ?? "White"}
        </p>
      </div>
      <div className="mt-4 flex gap-2.5">
        {colors.map((color) => {
          const isSelected = color.id === selectedId;
          const isWhite = color.hex === "FFFFFF";
          return (
            <button
              key={color.id}
              disabled={disabled}
              onClick={() => onSelect(color)}
              aria-label={color.name}
              className={`relative size-9 rounded-full transition-all ${
                isWhite ? "border border-black/12" : ""
              } ${
                isSelected
                  ? "ring-2 ring-[#151b26] ring-offset-2"
                  : "hover:ring-2 hover:ring-[#151b26]/30 hover:ring-offset-2"
              } ${disabled ? "cursor-not-allowed opacity-50" : "cursor-pointer"}`}
              style={{ backgroundColor: `#${color.hex}` }}
            >
              {isSelected && (
                <svg
                  className="absolute inset-0 m-auto size-4"
                  viewBox="0 0 16 16"
                  fill="none"
                  stroke={isWhite ? "#151b26" : "#fff"}
                  strokeWidth="2.5"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                >
                  <path d="M3.5 8.5 6.5 11.5 12.5 5" />
                </svg>
              )}
            </button>
          );
        })}
      </div>
    </div>
  );
}
