"use client";

import Image from "next/image";
import { UploadIcon } from "lucide-react";
import { useRef } from "react";

import type { DemoDesign } from "@/lib/demo-data";
import { Button } from "@/components/ui/button";

interface DesignUploaderProps {
  currentPreviewUrl: string;
  samples: DemoDesign[];
  selectedDesignId: string;
  isUploading: boolean;
  onSampleSelect: (design: DemoDesign) => void;
  onFileSelect: (file: File) => void;
}

export function DesignUploader({
  currentPreviewUrl,
  samples,
  selectedDesignId,
  isUploading,
  onSampleSelect,
  onFileSelect,
}: DesignUploaderProps) {
  const inputRef = useRef<HTMLInputElement>(null);

  return (
    <div className="section-frame px-5 py-5">
      <div className="flex flex-col gap-5 md:flex-row md:items-center md:justify-between">
        <div>
          <h3 className="text-base font-semibold text-foreground">Choose a design</h3>
          <p className="mt-1 text-sm text-muted-foreground">
            Start with one of the sample graphics or upload your own PNG/JPG.
          </p>
        </div>
        <div className="flex items-center gap-3">
          <div className="relative flex h-16 w-16 items-center justify-center overflow-hidden rounded-[1.1rem] border border-black/8 bg-white">
            {currentPreviewUrl ? (
              // eslint-disable-next-line @next/next/no-img-element
              <img
                src={currentPreviewUrl}
                alt="Selected design preview"
                className="h-full w-full object-contain"
              />
            ) : (
              <Image
                src="/samples/sample-design-1.png"
                alt="Sample design"
                fill
                className="object-contain"
                sizes="4rem"
              />
            )}
          </div>
          <Button
            onClick={() => inputRef.current?.click()}
            variant="outline"
            className="h-10 rounded-full px-4"
            disabled={isUploading}
          >
            <UploadIcon className="size-4" />
            {isUploading ? "Uploading..." : "Upload your design"}
          </Button>
          <input
            ref={inputRef}
            type="file"
            accept="image/png,image/jpeg"
            className="hidden"
            onChange={(event) => {
              const file = event.target.files?.[0];
              if (file) {
                onFileSelect(file);
              }
              event.currentTarget.value = "";
            }}
          />
        </div>
      </div>
      <div className="mt-5 flex flex-wrap gap-3">
        {samples.map((sample) => (
          <button
            key={sample.id}
            onClick={() => onSampleSelect(sample)}
            className={`group flex items-center gap-3 rounded-full border px-3 py-2 transition ${
              sample.id === selectedDesignId
                ? "border-[#151b26] bg-[#151b26] text-white"
                : "border-black/8 bg-white text-foreground hover:border-[#151b26]/35"
            }`}
          >
            <span className="relative block size-9 overflow-hidden rounded-full bg-[#eef2f7]">
              <Image
                src={sample.previewSrc}
                alt={sample.label}
                fill
                className="object-contain"
                sizes="2.25rem"
              />
            </span>
            <span className="text-sm font-medium">{sample.label}</span>
          </button>
        ))}
      </div>
    </div>
  );
}
