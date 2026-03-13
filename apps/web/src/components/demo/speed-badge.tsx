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
