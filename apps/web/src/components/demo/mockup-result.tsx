"use client";

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
  tintHex?: string | null;
}

export function MockupResult({
  result,
  isGenerating,
  error,
  noticeMessage,
  compact = false,
  tintHex,
}: MockupResultProps) {
  const frameClassName = compact ? "aspect-[4/3]" : "aspect-[4/5]";
  const previousUrlRef = useRef<string | null>(null);
  const [showNew, setShowNew] = useState(false);

  useEffect(() => {
    if (result?.mockupUrl && result.mockupUrl !== previousUrlRef.current) {
      setShowNew(false);
      requestAnimationFrame(() => {
        setShowNew(true);
      });
      // Update previous ref AFTER the 300ms crossfade completes
      const timeout = setTimeout(() => {
        previousUrlRef.current = result.mockupUrl;
      }, 300);
      return () => clearTimeout(timeout);
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

  const hasPrevious = previousUrlRef.current !== null;
  const isFirstLoad = isGenerating && !result && !hasPrevious;

  return (
    <div className="section-frame px-5 py-5">
      <div
        className={`relative overflow-hidden rounded-[1.6rem] bg-[#e7ebf1] ${frameClassName}`}
      >
        {isFirstLoad ? (
          <div className="flex h-full items-center justify-center px-8 text-center">
            <div>
              <div className="mx-auto mb-6 h-48 w-64 animate-pulse rounded-xl bg-gradient-to-br from-[#d5dbe5] to-[#e7ebf1]" />
              <p className="text-sm font-medium text-muted-foreground">
                Generating your first preview...
              </p>
            </div>
          </div>
        ) : result ? (
          <>
            {previousUrlRef.current &&
              previousUrlRef.current !== result.mockupUrl && (
                /* eslint-disable-next-line @next/next/no-img-element */
                <img
                  src={previousUrlRef.current}
                  alt="Previous mockup"
                  className="absolute inset-0 h-full w-full object-contain"
                />
              )}
            {/* eslint-disable-next-line @next/next/no-img-element */}
            <img
              src={result.mockupUrl}
              alt={`Generated ${result.templateLabel} mockup`}
              className="absolute inset-0 h-full w-full object-contain transition-opacity duration-300"
              style={{ opacity: showNew ? 1 : 0 }}
            />
            {isGenerating && (
              <div className="absolute inset-0 bg-black/8 transition-opacity duration-300" />
            )}
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
            {result && !isGenerating && (
              <div className="absolute bottom-4 right-4 rounded-full bg-black/50 px-3 py-1 text-[0.72rem] font-semibold uppercase tracking-[0.2em] text-white/84 backdrop-blur">
                MeetMockup preview
              </div>
            )}
          </>
        ) : (
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
              className="inline-flex h-8 items-center justify-center rounded-lg border border-black/10 bg-white px-3 text-sm font-medium text-foreground transition hover:bg-muted animate-in slide-in-from-right-4 fade-in duration-300"
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
